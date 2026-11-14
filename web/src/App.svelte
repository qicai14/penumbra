<script lang="ts">
  import { computeScenario, loadFixture, loadManifest, loadNearTile, xyToLatlon, type EphSegment, type Inputs, type NearMeta, type NearTile, type Preset, type Region, type Scenario } from "./lib/engine";
  import Compare from "./lib/Compare.svelte";
  import Replay from "./lib/Replay.svelte";
  import { applyPresetEdits, decodeCompare, type Pair } from "./lib/compare";
  import Panorama from "./lib/Panorama.svelte";
  import Timelines from "./lib/Timelines.svelte";
  import Survival from "./lib/Survival.svelte";
  import Countdown from "./lib/Countdown.svelte";
  import Tracer from "./lib/Tracer.svelte";
  import Scoreboard from "./lib/Scoreboard.svelte";
  import About from "./lib/About.svelte";
  import SitePicker from "./lib/SitePicker.svelte";
  import { parseDate } from "./lib/site";
  import { Controls } from "./lib/state.svelte";

  const controls = new Controls();
  let tile = $state.raw<(NearTile & { regions: Region[]; eph: EphSegment[] }) | null>(null);
  let scenario = $state.raw<Scenario | null>(null);
  let computedFor = $state.raw<Inputs | null>(null);
  let error = $state<string | null>(null);
  let idx = $state(0);
  let gen = 0;

  let tileError = $state<string | null>(null); // kept apart so a later compute success cannot hide it
  loadNearTile().then((t) => (tile = t), (e) => (tileError = String(e?.message ?? e)));
  // A new region puts the site at the middle of its tile, which is always inside the 7.05 km border.
  function onregion(id: string) {
    loadNearTile(id).then((t) => {
      tile = t; tileError = null;
      const { x0_m, y0_m, width, height, pixel_m } = t.meta;
      const [lat, lon] = xyToLatlon(x0_m + (width * pixel_m) / 2, y0_m - (height * pixel_m) / 2);
      controls.lat = +lat.toFixed(4); controls.lon = +lon.toFixed(4);
    }, (e) => (tileError = String(e?.message ?? e)));
  }

  // Compare A vs B (T27). The URL query holds both pairs and the preset (`?a=<lat>,<lon>@<date>&b=...&preset=...`); a query
  // with `a` or `b` opens the compare view on load. The site view keeps a clean URL.
  let view = $state<"site" | "compare" | "replay">("site");
  let cmp = $state<{ pairs: { a: Pair; b: Pair }; base: Preset; metas: NearMeta[]; eph: EphSegment[] } | null>(null); // deep state: Compare edits the pairs in place, so they survive a trip to the site view
  let cmpError = $state<string | null>(null);
  async function openCompare(search: string) {
    view = "compare";
    if (cmp && !search) return; // coming back keeps the pairs (and the URL is rewritten when Compare mounts)
    try {
      const [m, fx] = await Promise.all([loadManifest(), loadFixture()]);
      const metas = m.regions.map((r) => r.near);
      const init = decodeCompare(search, { fallback: { lat: controls.lat, lon: controls.lon, date: controls.date }, spanDays: controls.spanDays, mastM: controls.mastM, metas, eph: m.ephemeris });
      if (init.presetEdits.length) controls.preset = applyPresetEdits(fx.preset, init.presetEdits);
      controls.mastM = init.mastM; controls.spanDays = init.spanDays;
      cmp = { pairs: { a: init.a, b: init.b }, base: fx.preset, metas, eph: m.ephemeris }; cmpError = null;
    } catch (e: any) { cmpError = String(e?.message ?? e); }
  }
  function openReplay() {
    view = "replay";
    try { history.replaceState(null, "", location.pathname); } catch { /* ignore */ }
  }
  function openSite() {
    view = "site";
    try { history.replaceState(null, "", location.pathname); } catch { /* ignore */ }
  }
  if (new URLSearchParams(location.search).has("a") || new URLSearchParams(location.search).has("b")) openCompare(location.search);

  // Any change to site, date, span or mast recomputes; a stale answer never overwrites a newer one.
  $effect(() => {
    const req: Inputs = {
      lat_deg: controls.lat, lon_deg: controls.lon, mast_m: controls.mastM,
      start_unix_s: parseDate(controls.date)!, span_days: controls.spanDays, preset: controls.preset ?? undefined, // the picker only stores dates that parse
    };
    const mine = ++gen;
    computeScenario(req).then(
      // A shorter span has fewer steps, so keep the shared time cursor inside the new grid.
      (s) => { if (mine === gen) { idx = Math.min(idx, s.count - 1); scenario = s; computedFor = req; error = null; } },
      (e) => { if (mine === gen) error = String(e?.message ?? e); },
    );
  });
</script>

<header class="bar">
  <h1 class="wordmark">Penumbra</h1>
  <nav aria-label="View">
    <button type="button" aria-pressed={view === "site"} onclick={openSite}>Site</button>
    <button type="button" aria-pressed={view === "compare"} onclick={() => openCompare("")}>Compare A vs B</button>
    <button type="button" aria-pressed={view === "replay"} onclick={openReplay}>IM-2 replay</button>
  </nav>
</header>

<main>
  {#if view === "replay"}
    <p class="tag">~/replay</p>
    <Replay />
  {:else if view === "compare"}
    <p class="tag">~/compare</p>
    {#if cmpError}<p class="msg" role="alert">{cmpError}</p>{/if}
    {#if cmp}<Compare {controls} pairs={cmp.pairs} base={cmp.base} metas={cmp.metas} eph={cmp.eph} />{:else if !cmpError}<p class="msg">Loading…</p>{/if}
  {:else}
  <p class="tag">~/site</p>
  {#if tile}<SitePicker {controls} {tile} {onregion} />{/if}
  {#if tileError}<p class="msg" role="alert">{tileError}</p>{/if}
  {#if error}<p class="msg" role="alert">{error}</p>{/if}
  {#if scenario && computedFor}
    <p class="msg" data-testid="computed-for">
      Computed for {scenario.site.lat_deg.toFixed(4)}°, {scenario.site.lon_deg.toFixed(4)}°E · mast {scenario.site.mast_m.toFixed(1)} m · landing {controls.date} · {computedFor.span_days} d
    </p>
    <Survival {scenario} {controls} />
    <Countdown {scenario} {idx} />
    <Panorama {scenario} bind:idx />
    <Timelines {scenario} bind:idx />
  {:else if !error}
    <p class="msg">Loading…</p>
  {/if}
  <Tracer />
  <Scoreboard />
  <About />
  {/if}
</main>

<style>
  :global(body) {
    margin: 0; background: var(--surface-void); color: var(--color-paper);
    font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono);
  }
  .bar { padding: var(--spacing-16); display: flex; align-items: center; justify-content: space-between; gap: var(--spacing-16); border-bottom: 1px solid var(--color-iron); }
  nav { display: flex; }
  nav button {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 1px solid var(--color-slate); padding: var(--spacing-8) var(--spacing-16); margin-left: -1px; cursor: pointer;
  }
  nav button:first-child { margin-left: 0; border-radius: var(--radius-sm) 0 0 var(--radius-sm); }
  nav button:last-child { border-radius: 0 var(--radius-sm) var(--radius-sm) 0; }
  nav button:not(:first-child):not(:last-child) { border-radius: 0; }
  nav button[aria-pressed="true"] { color: var(--color-paper); background: var(--surface-iron); border-color: var(--color-pewter); }
  nav button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .wordmark { margin: 0; font: italic 800 var(--text-base)/1 var(--font-wordmark); font-stretch: 112%; text-transform: uppercase; }
  main { max-width: var(--page-max-width); margin: 0 auto; padding: var(--spacing-24) var(--spacing-16); }
  .tag { margin: 0 0 var(--spacing-16); font-size: var(--text-caption); line-height: var(--leading-caption); color: var(--color-ash); }
  .msg { color: var(--color-fog); }
</style>
