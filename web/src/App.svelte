<script lang="ts">
  import { computeScenario, latlonToXy, loadFixture, loadManifest, loadNearTile, xyToLatlon, type EphSegment, type Inputs, type NearMeta, type NearTile, type Preset, type Region, type Scenario } from "./lib/engine";
  import Compare from "./lib/Compare.svelte";
  import Replay from "./lib/Replay.svelte";
  import { applyPresetEdits, decodeCompare, defaultB, type Pair } from "./lib/compare";
  import Panorama from "./lib/Panorama.svelte";
  import Timelines from "./lib/Timelines.svelte";
  import Survival from "./lib/Survival.svelte";
  import Headline from "./lib/Headline.svelte";
  import LandingWindow from "./lib/LandingWindow.svelte";
  import WhatChanges from "./lib/WhatChanges.svelte";
  import Countdown from "./lib/Countdown.svelte";
  import Tracer from "./lib/Tracer.svelte";
  import Scoreboard from "./lib/Scoreboard.svelte";
  import About from "./lib/About.svelte";
  import VerdictStrip from "./lib/VerdictStrip.svelte";
  import SitePicker from "./lib/SitePicker.svelte";
  import Terrain3D from "./lib/Terrain3D.svelte";
  import { MaskFeed } from "./lib/maskFeed.svelte";
  import { parseDate } from "./lib/site";
  import { dateIndex, fetchGrid, nearestBetter, readout, type Grid } from "./lib/grid";
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
  // A new region puts the site at the middle of its tile, which is always inside the pick border.
  function onregion(id: string, at?: [number, number]) { // `at` (lat, lon) overrides the tile middle
    loadNearTile(id).then((t) => {
      tile = t; tileError = null;
      const { x0_m, y0_m, width, height, pixel_m } = t.meta;
      const [lat, lon] = xyToLatlon(x0_m + (width * pixel_m) / 2, y0_m - (height * pixel_m) / 2);
      controls.lat = at ? at[0] : +lat.toFixed(4); controls.lon = at ? at[1] : +lon.toFixed(4);
    }, (e) => (tileError = String(e?.message ?? e)));
  }

  // Regional point grid (T24) for the picked region, if the manifest lists one; the app works without it.
  // Every region's grid is loaded once, for the picker's best-cell block; `grid` is the picked region's.
  let grids = $state.raw<Grid[]>([]);
  loadManifest().then((m) => Promise.all((m.grids ?? []).map(fetchGrid))).then((g) => (grids = g), (e) => (tileError = `Heatmap not loaded: ${e?.message ?? e}`));
  const grid = $derived(grids.find((g) => g.meta.region === tile?.region.id) ?? null);
  const gridK = $derived(grid ? dateIndex(grid, controls.date) : -1);
  const betterAt = $derived.by((): [number, number] | null => {
    if (!grid || gridK < 0 || controls.preset || controls.mastM !== grid.meta.mast_m || controls.spanDays !== grid.meta.span_days) return null;
    const b = nearestBetter(grid, gridK, ...latlonToXy(controls.lat, controls.lon));
    if (b?.kind !== "better") return null;
    const [lat, lon] = xyToLatlon(b.x, b.y);
    return [+lat.toFixed(4), +lon.toFixed(4)];
  });
  const movePin = () => { if (betterAt) [controls.lat, controls.lon] = betterAt; };
  const nearby = $derived.by(() => {
    if (!grid || gridK < 0) return null;
    // The grid is built for the default preset, mast and span; once the planner changes any, its numbers no longer match the headline.
    if (controls.preset || controls.mastM !== grid.meta.mast_m || controls.spanDays !== grid.meta.span_days)
      return `Nearest better site: the grid is built for the default preset, mast ${grid.meta.mast_m} m and ${grid.meta.span_days} d span; reset those to compare.`;
    const [x, y] = latlonToXy(controls.lat, controls.lon);
    return readout(nearestBetter(grid, gridK, x, y)) + ` · grid: ${grid.meta.span_days} d span, mast ${grid.meta.mast_m} m. Direction is on the map, not lunar north.`;
  });

  // Compare A vs B (T27). The URL query holds both pairs and the preset (`?a=<lat>,<lon>@<date>&b=...&preset=...`); a query
  // with `a` or `b` opens the compare view on load. The site view keeps a clean URL.
  let view = $state<"site" | "compare" | "replay" | "about">("site");
  let cmp = $state<{ pairs: { a: Pair; b: Pair }; base: Preset; metas: NearMeta[]; eph: EphSegment[]; why: { pair: Pair; text: string } | null } | null>(null); // deep state: Compare edits the pairs in place, so they survive a trip to the site view
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
      // No b in the URL: B defaults to the nearest better cell, else the same site 2 days later, and says why. A shared b wins.
      let why: { pair: Pair; text: string } | null = null;
      if (!new URLSearchParams(search).has("b")) {
        const gs = grids.length ? grids : await Promise.all((m.grids ?? []).map(fetchGrid)).catch(() => []);
        const d = defaultB(init.a, gs, { mastM: init.mastM, spanDays: init.spanDays, hasPreset: init.presetEdits.length > 0, eph: m.ephemeris });
        init.b = d.pair; why = { pair: d.pair, text: d.note };
      }
      cmp = { pairs: { a: init.a, b: init.b }, base: fx.preset, metas, eph: m.ephemeris, why }; cmpError = null;
    } catch (e: any) { cmpError = String(e?.message ?? e); }
  }
  function openReplay() {
    view = "replay";
    try { history.replaceState(null, "", location.pathname); } catch { /* ignore */ }
  }
  function openAbout() {
    view = "about";
    try { history.replaceState(null, "", location.pathname); } catch { /* ignore */ }
  }
  function openSite() {
    view = "site";
    try { history.replaceState(null, "", location.pathname); } catch { /* ignore */ }
  }
  if (new URLSearchParams(location.search).has("a") || new URLSearchParams(location.search).has("b")) openCompare(location.search);

  // One point-Sun mask for the time marker, shared by the picker's 2D map and the 3D terrain so both show the same bits.
  const maskFeed = new MaskFeed();
  const markerT = $derived(scenario?.track.t_unix_s[idx] ?? parseDate(controls.date));
  $effect(() => {
    if (view !== "site" || !tile || markerT === null || !Number.isFinite(markerT)) return;
    return maskFeed.follow(tile.region.id, markerT, controls.mastM);
  });
  $effect(() => () => maskFeed.dispose());

  // Any change to site, date, span or mast recomputes; a stale answer never overwrites a newer one.
  $effect(() => {
    const req: Inputs = {
      lat_deg: controls.lat, lon_deg: controls.lon, mast_m: controls.mastM,
      start_unix_s: parseDate(controls.date)!, span_days: controls.spanDays, preset: controls.preset ?? undefined, region: tile?.region.id, // the picker only stores dates that parse
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
    <button type="button" aria-pressed={view === "about"} onclick={openAbout}>About</button>
  </nav>
  <a class="gh" href="https://github.com/qicai14/penumbra" target="_blank" rel="noopener noreferrer" aria-label="GitHub (opens in a new tab)"><svg aria-hidden="true" viewBox="0 0 16 16" width="16" height="16" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.6 7.6 0 0 1 4 0c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg>GitHub</a>
</header>

<main>
  {#if view === "about"}
    <p class="tag">~/about</p>
    <Tracer />
    <Scoreboard />
    <About />
  {:else if view === "replay"}
    <p class="tag">~/replay</p>
    <Replay />
  {:else if view === "compare"}
    <p class="tag">~/compare</p>
    {#if cmpError}<p class="msg" role="alert">{cmpError}</p>{/if}
    {#if cmp}<Compare {controls} pairs={cmp.pairs} base={cmp.base} metas={cmp.metas} eph={cmp.eph} why={cmp.why} />{:else if !cmpError}<p class="msg">Loading…</p>{/if}
  {:else}
  <p class="tag">~/site</p>
  <VerdictStrip onopen={openAbout} />
  {#if tile}<SitePicker {controls} {tile} {onregion} feed={maskFeed} {grid} {gridK} {grids} />{/if}
  {#if tileError}<p class="msg" role="alert">{tileError}</p>{/if}
  {#if error}<p class="msg" role="alert">{error}</p>{/if}
  {#if scenario && computedFor}
    <p class="msg" data-testid="computed-for">
      Computed for {scenario.site.lat_deg.toFixed(4)}°, {scenario.site.lon_deg.toFixed(4)}°E · mast {scenario.site.mast_m.toFixed(1)} m · landing {controls.date} · {computedFor.span_days} d
    </p>
    <div class="toprow">
      <Countdown {scenario} {idx} sticky={false} />
      <Headline {scenario} {nearby} onmove={betterAt ? movePin : undefined} />
    </div>
    <WhatChanges {scenario} {computedFor} {grid} {gridK} gridUsable={!controls.preset && controls.mastM === grid?.meta.mast_m && controls.spanDays === grid?.meta.span_days} />
    {#if tile}<LandingWindow {computedFor} eph={tile.eph} date={controls.date} onpick={(d) => (controls.date = d)} />{/if}
    <Survival {scenario} {controls} />
    <Panorama {scenario} bind:idx />
    {#if tile}<Terrain3D {controls} {tile} feed={maskFeed} t={markerT} sunAz={scenario.track.sun_az_deg[idx]} sunEl={scenario.track.sun_el_deg[idx]} sunLon={computedFor.lon_deg} />{/if}
    <Timelines {scenario} bind:idx />
  {:else if !error}
    <p class="msg">Loading…</p>
  {/if}
  {/if}
</main>

<style>
  :global(body) {
    margin: 0; background: var(--surface-void); color: var(--color-paper);
    font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono);
  }
  .bar { padding: var(--spacing-16); display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--spacing-16); border-bottom: 1px solid var(--color-iron); }
  /* Site v2 B: countdown first, survival headline beside it; the row stays pinned only where it fits on one line and leaves room below (T26). */
  .toprow { display: flex; flex-wrap: wrap; gap: var(--spacing-16); margin-top: var(--spacing-24); }
  .toprow > :global(:first-child) { flex: 2 1 420px; }
  .toprow > :global(:last-child) { flex: 1 1 260px; }
  .toprow :global(.panel) { margin-top: 0; }
  @media (min-width: 1100px) and (min-height: 800px) { .toprow { position: sticky; top: 0; z-index: 1; background: var(--surface-void); padding-bottom: var(--spacing-8); } }
  nav { display: flex; }
  /* Touch: every control gets a 44 px target (WCAG 2.5.5 AAA / Apple HIG); mouse layouts keep the compact size. */
  @media (pointer: coarse) {
    :global(button), :global(select), :global(input), :global(summary) { min-height: 44px; }
    .gh { min-height: 44px; }
  }
  .gh { display: inline-flex; align-items: center; gap: var(--spacing-8); color: var(--color-fog); text-decoration: none; font: var(--text-caption)/var(--leading-caption) var(--font-mono); }
  .gh:hover { color: var(--color-paper); }
  .gh:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
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
