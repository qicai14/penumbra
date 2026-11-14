<script lang="ts">
  // T27 (#28): two site-date pairs side by side with one shared lander preset. Each column is its own scenario, so its
  // clocks follow its own landing time (Qi Cai, 2026-11-14). Every figure is the model's; the page only subtracts and words it.
  import { computeScenario, latlonToXy, type NearMeta, type Preset, type Scenario } from "./engine";
  import { compareSentence, diffDays, encodeCompare, presetEdits, type Pair } from "./compare";
  import Countdown from "./Countdown.svelte";
  import { headline, presetFields, setPresetField, spanDays, type FieldKey } from "./headline";
  import { checkDate, checkPick } from "./site";
  import type { Controls } from "./state.svelte";

  let { controls, pairs, base, metas }: { controls: Controls; pairs: Record<"a" | "b", Pair>; base: Preset; metas: NearMeta[] } = $props();

  type Side = "a" | "b";
  const SIDES: Side[] = ["a", "b"];
  // svelte-ignore state_referenced_locally -- drafts start from the pairs, then follow typing
  let drafts = $state<Record<Side, { lat: string; lon: string; date: string }>>({
    a: { lat: String(pairs.a.lat), lon: String(pairs.a.lon), date: pairs.a.date },
    b: { lat: String(pairs.b.lat), lon: String(pairs.b.lon), date: pairs.b.date },
  });
  let refusals = $state<Record<Side, string | null>>({ a: null, b: null });
  let presetRefusal = $state<string | null>(null);

  const preset = $derived(controls.preset ?? base);
  const fields = $derived(presetFields(preset));

  // The pairs and preset live in the address bar, so a copy of the URL is the shared comparison.
  $effect(() => {
    const q = encodeCompare({
      a: pairs.a, b: pairs.b, mastM: controls.mastM, spanDays: controls.spanDays,
      presetEdits: controls.preset ? presetEdits(base, controls.preset) : [],
    });
    try { history.replaceState(null, "", `?${q}`); } catch { /* sandboxed frame: sharing by URL is unavailable, the view still works */ }
  });

  // A then B, one at a time (the Engine holds one near tile). A stale answer never overwrites a newer one.
  // Each result keeps the pair it was computed for, so headers and the sentence never pair new labels with old figures.
  type Result = { pair: Pair; sc: Scenario | null; err: string | null };
  let results = $state.raw<Record<Side, Result | null>>({ a: null, b: null });
  let gen = 0;
  $effect(() => {
    const used = SIDES.map((s) => ({ ...pairs[s] }));
    const reqs = SIDES.map((s) => ({
      lat_deg: pairs[s].lat, lon_deg: pairs[s].lon, mast_m: controls.mastM,
      start_unix_s: Date.parse(`${pairs[s].date}T00:00:00Z`) / 1000, span_days: controls.spanDays, preset: controls.preset ?? undefined,
    }));
    const mine = ++gen;
    (async () => {
      const out: Result[] = [];
      for (const [i, r] of reqs.entries()) {
        try { out.push({ pair: used[i], sc: await computeScenario(r), err: null }); }
        catch (e: any) { out.push({ pair: used[i], sc: null, err: String(e?.message ?? e) }); }
        if (mine !== gen) return;
      }
      results = { a: out[0], b: out[1] };
    })();
  });

  function apply(s: Side) {
    const d = drafts[s];
    const lat = Number(d.lat), lon = Number(d.lon);
    const fail = (m: string) => { refusals[s] = m; };
    if (!d.lat.trim() || !d.lon.trim() || !Number.isFinite(lat) || !Number.isFinite(lon) || lat < -90 || lat > 90 || lon < -180 || lon > 360)
      return fail("Enter a latitude between −90 and 90 and a longitude between −180 and 360.");
    const dv = checkDate(d.date, controls.spanDays);
    if (!dv.ok) return fail(dv.reason);
    const [x, y] = latlonToXy(lat, lon);
    const picks = metas.map((t) => checkPick(x, y, t));
    if (picks.length && !picks.some((p) => p.ok)) {
      const first = picks.find((p) => !p.ok);
      return fail(first && !first.ok ? first.reason : "That site is outside every shipped near tile.");
    }
    refusals[s] = null;
    pairs[s] = { lat, lon, date: d.date };
  }

  function editPreset(key: FieldKey, label: string, e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const next = setPresetField(preset, key, input.valueAsNumber);
    if (typeof next !== "string") { controls.preset = next; presetRefusal = null; }
    else { presetRefusal = `${label}: ${next}.`; input.value = String(fields.find((f) => f.key === key)!.value); }
  }

  const sa = $derived(results.a?.sc ?? null);
  const sb = $derived(results.b?.sc ?? null);
  const sentence = $derived(sa && sb && results.a && results.b
    ? compareSentence({ pair: results.a.pair, operableH: sa.survival.operable_h }, { pair: results.b.pair, operableH: sb.survival.operable_h })
    : null);
</script>

<section class="panel" aria-labelledby="cmp-h">
  <h2 id="cmp-h">Compare A vs B · two site-date pairs, one lander preset</h2>

  <div class="pickers">
    {#each SIDES as s (s)}
      <form class="picker" onsubmit={(e) => { e.preventDefault(); apply(s); }} aria-label="Site and date {s.toUpperCase()}">
        <h3>{s.toUpperCase()}</h3>
        <label><span>Latitude °</span><input data-testid="{s}-lat" bind:value={drafts[s].lat} inputmode="decimal" autocomplete="off" /></label>
        <label><span>Longitude °E</span><input data-testid="{s}-lon" bind:value={drafts[s].lon} inputmode="decimal" autocomplete="off" /></label>
        <label><span>Landing date (UTC)</span>
          <input data-testid="{s}-date" type="date" min="2026-01-01" max="2030-12-31" bind:value={drafts[s].date} onchange={() => apply(s)} />
        </label>
        <button type="submit">Set {s.toUpperCase()}</button>
        {#if refusals[s]}<p class="refusal" role="alert" data-testid="{s}-refusal">Refused. {refusals[s]}</p>{/if}
      </form>
    {/each}
  </div>

  <h3 class="sub">Lander preset (same for A and B)</h3>
  <form class="preset" onsubmit={(e) => e.preventDefault()} aria-label="Lander preset">
    {#each fields as f (f.key)}
      <label>
        <span class="name">{f.label}</span>
        <input type="number" min="0" step={f.step} max={f.key === "earth_margin_deg" ? 90 : undefined} value={f.value} data-testid="cmp-preset-{f.key}" onchange={(e) => editPreset(f.key, f.label, e)} />
        <span class="unit">{f.unit}</span>
        <span class="tag">{f.tag}</span>
      </label>
    {/each}
  </form>
  {#if presetRefusal}<p class="refusal" role="alert">{presetRefusal}</p>{/if}
  <p class="note">The address bar holds both pairs and the preset. Copy it to share this comparison.</p>
</section>

<div class="cols">
  {#each SIDES as s (s)}
    {@const sc = results[s]}
    <section class="panel col" aria-labelledby="{s}-h" data-testid="col-{s}">
      <h2 id="{s}-h">{s.toUpperCase()}{#if sc} · {sc.pair.lat.toFixed(4)}°, {sc.pair.lon.toFixed(4)}°E · landing {sc.pair.date}{/if}</h2>
      {#if sc?.err}
        <p class="refusal" role="alert" data-testid="{s}-error">{sc.err}</p>
      {:else if sc?.sc}
        {@const head = headline(sc.sc.survival, spanDays(sc.sc.count, sc.sc.step_s))}
        <p class="fig"><span class="cap">Operable</span><strong data-testid="{s}-operable">{head.operable}</strong></p>
        <p class="fig"><span class="cap">First lethal shadow in</span><strong data-testid="{s}-lethal">{#if !head.none}✕ {/if}{head.lethal}</strong></p>
        <Countdown scenario={sc.sc} idx={0} prefix="{s}-" title="At landing" sticky={false} />
      {:else}
        <p class="note">Loading…</p>
      {/if}
    </section>
  {/each}
</div>

<section class="panel diff" aria-label="B minus A">
  <span class="cap">Operable days, B − A</span>
  <strong data-testid="diff-operable">{sa && sb ? diffDays(sa.survival.operable_h, sb.survival.operable_h) : "—"}</strong>
</section>

<p class="sentence" data-testid="compare-sentence">{sentence ?? "B − A needs both sides computed; see the messages above."}</p>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  h3 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); }
  h3.sub { margin: var(--spacing-24) 0 var(--spacing-8); color: var(--color-fog); }
  .pickers, .cols { display: grid; grid-template-columns: 1fr; gap: var(--spacing-16); }
  .cols { margin-bottom: var(--spacing-16); }
  .cols .panel { margin: 0; }
  @media (min-width: 768px) { .pickers, .cols { grid-template-columns: 1fr 1fr; } }
  .picker { display: flex; flex-direction: column; gap: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .picker label { display: flex; flex-direction: column; gap: 4px; }
  .picker button { align-self: flex-start; }
  .preset { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(340px, 100%), 1fr)); gap: var(--spacing-8) var(--spacing-24); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .preset label { display: flex; flex-wrap: wrap; align-items: center; gap: var(--spacing-8); }
  .name { flex: 0 0 150px; }
  .unit { flex: 0 0 3ch; }
  .tag { margin-left: auto; border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8); color: var(--color-fog); }
  input {
    font: var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); background: var(--surface-iron);
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: 4px var(--spacing-8); min-width: 0; color-scheme: dark;
    font-variant-numeric: tabular-nums;
  }
  .preset input { width: 9ch; flex: none; }
  button {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: var(--spacing-8) var(--spacing-16); cursor: pointer;
  }
  button:focus-visible, input:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .fig { margin: 0 0 var(--spacing-16); display: flex; flex-direction: column; }
  .cap { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  strong { font: var(--font-weight-regular) var(--text-display)/var(--leading-display) var(--font-mono); color: var(--color-paper); font-variant-numeric: tabular-nums; }
  .diff { display: flex; flex-direction: column; }
  .note { margin: var(--spacing-8) 0 0; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .refusal { margin: 0; padding: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-pewter); border-radius: var(--radius-sm); background: var(--surface-iron); color: var(--color-paper); }
  .sentence { margin: 0 0 var(--spacing-24); font: var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); }
</style>
