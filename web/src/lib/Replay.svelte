<script lang="ts">
  import Explain from "./Explain.svelte";
  // T16 (#17): the IM-2 replay. One click loads the site, the touchdown time and the lander preset, then shows two answers side by side:
  // regional terrain, and a pit of D x d around the lander (A3). Every figure is the model's; this file only words and lays them out.
  import { tick } from "svelte";
  import { computePit, computeScenario, DEFAULT_PRESET, defaultPitDepth, loadManifest, pitDepthToHide, type Scenario } from "./engine";
  import { fmtDays, headline, presetFields, spanDays } from "./headline";
  import Panorama from "./Panorama.svelte";
  import { fmtUtc } from "./panorama";
  import { checkPit, DEFAULT_PANEL_M, DEFAULT_PIT_DIAMETER_M, IM2, num, pitSentence } from "./replay";
  import { checkDate } from "./site";

  type Answer = { regional: Scenario; pit: Scenario; typicalDepth: number; dStar: number };
  let answer = $state.raw<Answer | null>(null);
  let error = $state<string | null>(null);
  let refused = $state<string | null>(null);
  let busy = $state(false);
  let idx = $state(0); // the panorama cursor; 0 is touchdown

  let diameter = $state(DEFAULT_PIT_DIAMETER_M);
  let depth = $state<number | null>(null); // null = follow the 11° depth for this diameter
  let panel = $state(DEFAULT_PANEL_M);
  let gen = 0;
  let resultsHead = $state<HTMLElement | null>(null);
  let focusResults = false; // set by the Load button only: later edits keep focus where it is

  async function run() {
    const mine = ++gen;
    busy = true;
    try {
      const m = await loadManifest();
      const ok = checkDate(IM2.date, IM2.span_days, m.ephemeris);
      if (!ok.ok) throw new Error(ok.reason);
      const regional = await computeScenario({
        lat_deg: IM2.lat_deg, lon_deg: IM2.lon_deg, mast_m: panel, start_unix_s: IM2.touchdown_unix_s, span_days: IM2.span_days, preset: DEFAULT_PRESET,
      });
      const typicalDepth = await defaultPitDepth(diameter);
      const bad = checkPit({ diameter_m: diameter, depth_m: depth ?? typicalDepth, panel_m: panel });
      if (bad) throw new Error(bad);
      const pit = await computePit(regional, { diameter_m: diameter, depth_m: depth ?? typicalDepth });
      const dStar = await pitDepthToHide(diameter, panel, regional.track.sun_el_deg[0], regional.track.sun_radius_deg[0]);
      if (mine !== gen) return;
      answer = { regional, pit, typicalDepth, dStar };
      error = null;
      if (focusResults) { focusResults = false; await tick(); resultsHead?.focus(); }
    } catch (e: any) {
      if (mine === gen) error = String(e?.message ?? e);
    } finally {
      if (mine === gen) busy = false;
    }
  }

  const shownDepth = $derived(depth ?? answer?.typicalDepth ?? 0);
  // The inputs refuse a bad value with a reason and keep the last good one on screen.
  function edit(which: "diameter" | "depth" | "panel", e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const v = input.valueAsNumber;
    const next = { diameter_m: diameter, depth_m: shownDepth, panel_m: panel };
    next[which === "diameter" ? "diameter_m" : which === "depth" ? "depth_m" : "panel_m"] = v;
    const bad = checkPit(next);
    if (bad) {
      refused = bad;
      input.value = String(which === "diameter" ? diameter : which === "depth" ? +shownDepth.toFixed(2) : panel);
      return;
    }
    refused = null;
    if (which === "diameter") diameter = v; else if (which === "depth") depth = v; else panel = v;
    run();
  }
  function reset() { depth = null; diameter = DEFAULT_PIT_DIAMETER_M; refused = null; run(); }

  const fields = $derived(answer ? presetFields(answer.regional.preset) : []);
  const span = $derived(answer ? spanDays(answer.regional.count, answer.regional.step_s) : 0);
  const regionalHead = $derived(answer ? headline(answer.regional.survival, span) : null);
  const pitHead = $derived(answer ? headline(answer.pit.survival, span) : null);
  const sentence = $derived(answer && pitSentence({
    elSunDeg: answer.regional.track.sun_el_deg[0], clearPct: answer.regional.timeline.illumination[0] * 100, diameterM: diameter,
    panelM: panel, dStarM: answer.dStar, typicalDepthM: answer.typicalDepth,
  }));
</script>

<section class="panel" aria-labelledby="rp-h">
  <header>
    <h2 id="rp-h">IM-2 replay</h2>
  </header>

  <Explain id="rp" topic="IM-2 replay" plain="A real mission: the IM-2 landing site and date, on open ground and in a small pit.">
    <p>IM-2 landed near Mons Mouton in March 2025 and ended up on its side in a crater about 20 m wide, about one terrain-map pixel. Reports say it operated for under a day (unconfirmed). This view runs the model twice: once with the open regional terrain, once with the lander inside a pit, to show how a small crater can cut its sunlight.</p>
    <p>It is a reconstruction to compare with the mission reports. It does not say what happened.</p>
  </Explain>
  <ul class="tags" data-testid="replay-assumptions">
    <li><span class="tag">assumed · U8</span> Touchdown 2025-03-06 17:28 UTC. The time is not in a primary source.</li>
    <li><span class="tag">assumed · U10</span> Lander values and panel height. IM-2 battery and array values are not published.</li>
    <li><span class="tag">unconfirmed · U19</span> Reports say the lander operated for about a day.</li>
  </ul>

  {#if !answer}
    <button type="button" class="go" data-testid="replay-load" disabled={busy} onclick={() => { focusResults = true; run(); }}>{busy ? "Loading…" : "Load the IM-2 replay"}</button>
  {/if}
  {#if error}<p class="refusal" role="alert" data-testid="replay-error">{error}</p>{/if}

  {#if answer && regionalHead && pitHead}
    <div class="pair" aria-live="polite" aria-atomic="true">
      {#snippet column(id: string, sc: Scenario, hd: NonNullable<typeof regionalHead>)}
        <p class="fig"><span class="cap">Sun at touchdown</span><strong data-testid="{id}-clear">{num(sc.timeline.illumination[0] * 100, 0)} % clear</strong></p>
        <p class="fig"><span class="cap">Sunlit</span><strong data-testid="{id}-sunlit">{fmtDays(sc.survival.sunlit_h)}</strong></p>
        <p class="fig"><span class="cap">Operable</span><strong data-testid="{id}-operable">{hd.operable}</strong></p>
        <p class="fig">
          <span class="cap">{!hd.none && sc.survival.sunlit_h === 0 ? "Battery empty in" : "First lethal shadow in"}</span>
          <strong data-testid="{id}-lethal">{hd.lethal}</strong>
        </p>
      {/snippet}
      <article class="col" data-testid="answer-regional" aria-labelledby="ar-h">
        <h3 id="ar-h" tabindex="-1" bind:this={resultsHead}>Regional terrain says</h3>
        {@render column("regional", answer.regional, regionalHead)}
      </article>
      <article class="col" data-testid="answer-pit" aria-labelledby="ap-h">
        <h3 id="ap-h">A pit of {num(diameter, 2)} × {num(shownDepth, 2)} m says</h3>
        {@render column("pit", answer.pit, pitHead)}
      </article>
    </div>
    <p class="cap">Operable = Sun above the horizon and Earth in contact. Battery time uses the assumed load.</p>

    <p class="sentence" data-testid="dstar-sentence">{sentence} <span class="tag">assumed · U8, U9, U10</span></p>

    <form class="fields" onsubmit={(e) => e.preventDefault()} aria-label="Pit">
      <div class="row"><label class="name" for="pit-d">Pit diameter</label>
        <input id="pit-d" type="number" min="0.1" step="1" value={diameter} data-testid="pit-diameter" aria-describedby="pit-d-tag" onchange={(e) => edit("diameter", e)} /><span class="unit">m</span><span class="tag" id="pit-d-tag">cited · W6</span></div>
      <div class="row"><label class="name" for="pit-z">Pit depth</label>
        <input id="pit-z" type="number" min="0" step="0.1" value={+shownDepth.toFixed(2)} data-testid="pit-depth" aria-describedby="pit-z-tag" onchange={(e) => edit("depth", e)} /><span class="unit">m</span>
        <span class="tag" id="pit-z-tag" data-testid="pit-depth-tag">{depth === null ? "assumed · U9, E112 (11° slope)" : "assumed · your value"}</span></div>
      <div class="row"><label class="name" for="pit-h">Panel height H</label>
        <input id="pit-h" type="number" min="0" step="0.5" value={panel} data-testid="pit-panel" aria-describedby="pit-h-tag" onchange={(e) => edit("panel", e)} /><span class="unit">m</span><span class="tag" id="pit-h-tag">assumed · U10</span></div>
      <button type="button" class="reset" onclick={reset}>Reset to 20 m and the 11° depth</button>
      {#if refused}<p class="refusal" role="alert" data-testid="pit-refusal">{refused}</p>{/if}
    </form>

    <details class="preset">
      <summary>Lander values used (all assumed, U10)</summary>
      <ul>
        {#each fields as f (f.key)}<li>{f.label}: {num(f.value, 2)} {f.unit} <span class="tag">assumed</span></li>{/each}
      </ul>
    </details>

    <p class="cap">From touchdown, {fmtUtc(answer.regional.track.t_unix_s[0])} · {span} days · {num(IM2.lat_deg, 4)}°, {num(IM2.lon_deg, 4)}°E</p>
    <Panorama scenario={answer.regional} pit={answer.pit} bind:idx />
  {/if}
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); }
  header { margin-bottom: var(--spacing-16); }
  h2 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  h3 { margin: 0 0 var(--spacing-8); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .tags { margin: 0 0 var(--spacing-16); padding: 0; list-style: none; display: flex; flex-direction: column; gap: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .tag { border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8); color: var(--color-fog); white-space: nowrap; font: var(--text-caption)/var(--leading-caption) var(--font-mono); }
  .pair { display: flex; flex-wrap: wrap; gap: var(--spacing-16); margin-bottom: var(--spacing-16); }
  .col { flex: 1 1 280px; background: var(--surface-iron); border-radius: var(--radius-sm); padding: var(--spacing-16); display: flex; flex-direction: column; gap: var(--spacing-8); }
  .fig { margin: 0; display: flex; flex-direction: column; }
  .cap { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); margin: 0 0 var(--spacing-8); }
  .fig .cap { margin: 0; }
  strong { font: var(--font-weight-regular) var(--text-heading)/var(--leading-heading) var(--font-mono); color: var(--color-paper); font-variant-numeric: tabular-nums; }
  .sentence { margin: 0 0 var(--spacing-16); color: var(--color-paper); max-width: 72ch; }
  .fields { display: flex; flex-direction: column; gap: var(--spacing-8); margin-bottom: var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .row { display: flex; align-items: center; gap: var(--spacing-8); flex-wrap: wrap; }
  .name { flex: 0 0 150px; }
  .unit { flex: 0 0 3ch; }
  input {
    width: 9ch; font: var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); background: var(--surface-iron);
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: 4px var(--spacing-8); color-scheme: dark; font-variant-numeric: tabular-nums;
  }
  input:focus-visible, button:focus-visible, summary:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  button { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-paper); background: transparent; border: 1px solid var(--color-pewter); border-radius: var(--radius-sm); padding: var(--spacing-8) var(--spacing-16); cursor: pointer; }
  button:disabled { color: var(--color-ash); cursor: default; }
  .reset { align-self: flex-start; color: var(--color-fog); border-color: var(--color-slate); }
  .go { font-size: var(--text-body); }
  .preset { margin-bottom: var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  .preset ul { margin: var(--spacing-8) 0 0; padding-left: var(--spacing-24); }
  .refusal { margin: 0; padding: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-pewter); border-radius: var(--radius-sm); background: var(--surface-iron); color: var(--color-paper); }
</style>
