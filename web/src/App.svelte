<script lang="ts">
  import { computeScenario, DEFAULT_PRESET, latlonToXy, loadManifest, loadNearTile, xyToLatlon, type EphSegment, type Inputs, type NearMeta, type NearTile, type Preset, type Region, type Scenario } from "./lib/engine";
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
  import Tour, { tourSeen } from "./lib/Tour.svelte";
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
      return `Better site: reset preset, mast ${grid.meta.mast_m} m and ${grid.meta.span_days} d span to compare.`;
    const [x, y] = latlonToXy(controls.lat, controls.lon);
    return readout(nearestBetter(grid, gridK, x, y));
  });

  // Compare A vs B (T27). The URL query holds both pairs and the preset (`?a=<lat>,<lon>@<date>&b=...&preset=...`); a query
  // with `a` or `b` opens the compare view on load. The site view keeps a clean URL.
  // The views follow the wireframes-v2 nav (design/DECISIONS.md, Nov 14 picks), plus Compare and About.
  const VIEWS = [["site", "Site"], ["planner", "Planner"], ["terrain", "Terrain"], ["compare", "Compare"], ["replay", "IM-2 replay"], ["validation", "Validation"], ["about", "About"]] as const;
  type View = (typeof VIEWS)[number][0];
  let view = $state<View>("site");
  let tourOpen = $state(!tourSeen());
  let cmp = $state<{ pairs: { a: Pair; b: Pair }; base: Preset; metas: NearMeta[]; eph: EphSegment[]; why: { pair: Pair; text: string } | null } | null>(null); // deep state: Compare edits the pairs in place, so they survive a trip to the site view
  let cmpError = $state<string | null>(null);
  async function openCompare(search: string) {
    view = "compare";
    if (cmp && !search) return; // coming back keeps the pairs (and the URL is rewritten when Compare mounts)
    try {
      const m = await loadManifest();
      const metas = m.regions.map((r) => r.near);
      const init = decodeCompare(search, { fallback: { lat: controls.lat, lon: controls.lon, date: controls.date }, spanDays: controls.spanDays, mastM: controls.mastM, metas, eph: m.ephemeris });
      if (init.presetEdits.length) controls.preset = applyPresetEdits(DEFAULT_PRESET, init.presetEdits);
      controls.mastM = init.mastM; controls.spanDays = init.spanDays;
      // No b in the URL: B defaults to the nearest better cell, else the same site 2 days later, and says why. A shared b wins.
      let why: { pair: Pair; text: string } | null = null;
      if (!new URLSearchParams(search).has("b")) {
        const gs = grids.length ? grids : await Promise.all((m.grids ?? []).map(fetchGrid)).catch(() => []);
        const d = defaultB(init.a, gs, { mastM: init.mastM, spanDays: init.spanDays, hasPreset: init.presetEdits.length > 0, eph: m.ephemeris });
        init.b = d.pair; why = { pair: d.pair, text: d.note };
      }
      cmp = { pairs: { a: init.a, b: init.b }, base: DEFAULT_PRESET, metas, eph: m.ephemeris, why }; cmpError = null;
    } catch (e: any) { cmpError = String(e?.message ?? e); }
  }
  function open(v: View) {
    if (v === "compare") return openCompare("");
    view = v;
    try { history.replaceState(null, "", location.pathname); } catch { /* ignore */ }
  }
  if (new URLSearchParams(location.search).has("a") || new URLSearchParams(location.search).has("b")) openCompare(location.search);

  // One point-Sun mask for the time marker, shared by the picker's 2D map and the 3D terrain so both show the same bits.
  const maskFeed = new MaskFeed();
  const markerT = $derived(scenario?.track.t_unix_s[idx] ?? parseDate(controls.date));
  $effect(() => {
    if ((view !== "site" && view !== "terrain") || !tile || markerT === null || !Number.isFinite(markerT)) return;
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
  <!-- Lockup from design/brand/index.html set A (Grazing Sun): live Archivo text, the Sun and horizon drawn over it in lockup units (text spans x 6–336). -->
  <h1 class="wordmark">
    <svg class="sunrise" viewBox="6 0 347 39" aria-hidden="true"><defs><clipPath id="pn-sky"><rect x="250" y="0" width="92" height="35"/></clipPath></defs><g clip-path="url(#pn-sky)" class="sun"><circle cx="295" cy="35" r="6"/><polygon points="304.22,31.82 334.78,30.82 304.68,36.19"/><polygon points="303.36,29.97 320.68,26.66 304.71,34.16"/><polygon points="302.13,28.34 331.37,14 304.33,32.16"/><polygon points="300.59,27.01 316.55,15.6 303.53,30.28"/><polygon points="298.8,26.02 316.75,5.07 302.36,28.61"/><polygon points="296.85,25.43 305.17,12.16 300.87,27.22"/><polygon points="294.82,25.25 301.86,2.72 299.13,26.17"/><polygon points="292.8,25.5 295,5 297.2,25.5"/><polygon points="290.87,26.17 287.72,0.76 295.18,25.25"/><polygon points="289.13,27.22 284.42,11.25 293.15,25.43"/><polygon points="287.64,28.61 272.08,3.45 291.2,26.02"/><polygon points="286.47,30.28 274.19,16.26 289.41,27.01"/><polygon points="285.67,32.16 257.76,13.5 287.87,28.34"/><polygon points="285.29,34.16 270.27,26.97 286.64,29.97"/><polygon points="285.32,36.19 257.21,31.03 285.78,31.82"/></g><polygon class="horizon" points="96,38 353,34.5 353,39"/></svg>
    <span>Penumbra</span>
  </h1>
  <nav aria-label="View">
    {#each VIEWS as [v, label] (v)}
      <button type="button" aria-pressed={view === v} onclick={() => open(v)}>{label}</button>
    {/each}
  </nav>
  <a class="gh" href="https://github.com/qicai14/penumbra" target="_blank" rel="noopener noreferrer" aria-label="GitHub (opens in a new tab)"><svg aria-hidden="true" viewBox="0 0 16 16" width="20" height="20" fill="currentColor"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.6 7.6 0 0 1 4 0c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8z"/></svg></a>
</header>

<main>
  {#if view === "about"}
    <p class="tag">~/about</p>
    <About />
  {:else if view === "validation"}
    <p class="tag">~/validation</p>
    <Scoreboard />
    <Tracer />
  {:else if view === "replay"}
    <p class="tag">~/replay</p>
    <Replay />
  {:else if view === "compare"}
    <p class="tag">~/compare</p>
    {#if cmpError}<p class="msg" role="alert">{cmpError}</p>{/if}
    {#if cmp}<Compare {controls} pairs={cmp.pairs} base={cmp.base} metas={cmp.metas} eph={cmp.eph} why={cmp.why} />{:else if !cmpError}<p class="msg">Loading…</p>{/if}
  {:else}
  <p class="tag">~/{view}</p>
  {#if view === "site" && tile}<SitePicker {scenario} {controls} {tile} {onregion} feed={maskFeed} {grid} {gridK} {grids} />{/if}
  {#if tileError}<p class="msg" role="alert">{tileError}</p>{/if}
  {#if error}<p class="msg" role="alert">{error}</p>{/if}
  {#if scenario && computedFor}
    {#if view !== "site"}<!-- Site shows the inputs themselves; other views say what the numbers are for -->
      <p class="msg" data-testid="computed-for">
        Computed for {scenario.site.lat_deg.toFixed(4)}°, {scenario.site.lon_deg.toFixed(4)}°E · mast {scenario.site.mast_m.toFixed(1)} m · landing {controls.date} · {computedFor.span_days} d · <button type="button" class="link" onclick={() => open("site")}>change site</button>
      </p>
    {/if}
    {#if view === "site"}
      <Countdown {scenario} {idx} sticky={false} />
      <Panorama {scenario} bind:idx />
      <Timelines {scenario} bind:idx />
    {:else if view === "planner"}
      <Headline {scenario} {nearby} onmove={betterAt ? movePin : undefined} />
      <Survival {scenario} {controls} />
      <WhatChanges {scenario} {computedFor} {grid} {gridK} gridUsable={!controls.preset && controls.mastM === grid?.meta.mast_m && controls.spanDays === grid?.meta.span_days} />
      {#if tile}<LandingWindow {computedFor} eph={tile.eph} date={controls.date} onpick={(d) => (controls.date = d)} />{/if}
    {:else if tile}
      <Countdown {scenario} {idx} sticky={false} />
      <Terrain3D {controls} {tile} feed={maskFeed} t={markerT} bind:idx count={scenario.count} times={scenario.track.t_unix_s} sunAz={scenario.track.sun_az_deg[idx]} sunEl={scenario.track.sun_el_deg[idx]} sunLon={computedFor.lon_deg} />
    {/if}
  {:else if !error}
    <p class="msg">Loading…</p>
  {/if}
  {/if}
</main>
<Tour bind:open={tourOpen} />
<!-- The tour card docks in the same corner, so the button only shows while the tour is closed. -->
{#if !tourOpen}
  <button type="button" class="help" aria-label="Show the quick tour" onclick={() => { open("site"); tourOpen = true; }}>?</button>
{/if}

<style>
  :global(body) {
    margin: 0; background: var(--surface-void); color: var(--color-paper);
    font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono);
  }
  .bar { position: relative; background: var(--surface-void); padding: var(--spacing-16); display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: var(--spacing-16); border-bottom: 1px solid var(--color-iron); }
  /* Axiom's `>` field (DESIGN.md §7): slate chevrons in the page margins only, never under the content column or the bar; gone below ~1250 px. */
  :global(body)::before {
    content: ""; position: fixed; inset: 0; z-index: -1; pointer-events: none; background: var(--color-slate);
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='108' height='138'%3E%3Cpath d='M4 6l5 5-5 5M22 6l5 5-5 5M40 6l5 5-5 5M94 29l5 5-5 5M4 29l5 5-5 5M22 29l5 5-5 5M76 52l5 5-5 5M94 52l5 5-5 5M4 52l5 5-5 5M58 75l5 5-5 5M76 75l5 5-5 5M94 75l5 5-5 5M40 98l5 5-5 5M58 98l5 5-5 5M76 98l5 5-5 5M22 121l5 5-5 5M40 121l5 5-5 5M58 121l5 5-5 5' fill='none' stroke='%23000' stroke-width='1.2'/%3E%3C/svg%3E") 0 0 / 108px 138px repeat,
      linear-gradient(to right, #000 calc(50% - var(--page-max-width) / 2 - 24px), transparent calc(50% - var(--page-max-width) / 2), transparent calc(50% + var(--page-max-width) / 2), #000 calc(50% + var(--page-max-width) / 2 + 24px));
    mask-composite: intersect;
  }
  nav { display: flex; flex-wrap: nowrap; overflow-x: auto; white-space: nowrap; gap: var(--spacing-16); scroll-snap-type: x proximity; }
  nav button { scroll-snap-align: start; }
  /* Right-edge fade says more tabs are off screen; padding lets the last tab scroll clear of it. */
  @media (max-width: 600px) {
    nav { padding-right: 32px; mask-image: linear-gradient(to right, #000 85%, transparent); }
  }
  /* Touch: every control gets a 44 px target (WCAG 2.5.5 AAA / Apple HIG); mouse layouts keep the compact size. */
  @media (pointer: coarse) {
    :global(button), :global(select), :global(input), :global(summary) { min-height: 44px; }
    .gh { min-height: 44px; }
  }
  .gh { display: inline-flex; align-items: center; color: var(--color-fog); text-decoration: none; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); }
  .gh:hover { color: var(--color-paper); }
  .gh:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .help { position: fixed; z-index: 10; right: var(--spacing-16); bottom: var(--spacing-16); width: 44px; height: 44px; font: inherit; color: var(--color-fog); background: var(--surface-void); border: 1px solid var(--color-iron); cursor: pointer; }
  .help:hover { color: var(--color-paper); border-color: var(--color-paper); }
  .help:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  nav button {
    font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 0; border-radius: 0; padding: var(--spacing-8) 0; cursor: pointer;
    text-decoration: underline; text-decoration-color: transparent; text-underline-offset: 6px; text-decoration-thickness: 1px;
  }
  nav button:hover { color: var(--color-paper); }
  nav button[aria-pressed="true"] { color: var(--color-paper); text-decoration-color: var(--color-paper); }
  nav button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  /* Room above the word for the Sun: the art is ~0.81em tall at this word width (6.83em, measured on the loaded Archivo). */
  .wordmark { position: relative; margin: 0; padding-top: 0.86em; display: inline-block; font: italic 800 var(--text-heading-sm)/1 var(--font-wordmark); font-stretch: 112%; text-transform: uppercase; }
  .wordmark span { display: block; }
  /* Sized from the word's own width, so it never stretches the text. 347 of the lockup's 330 text units: the horizon runs a little past the last letter, as in the brand file. */
  .sunrise { position: absolute; left: 0; top: 0; width: calc(100% * 347 / 330); height: auto; overflow: visible; }
  .sunrise .sun { fill: var(--color-sun); }
  .sunrise .horizon { fill: var(--color-paper); }
  main :global(.panel + .panel) { margin-top: var(--spacing-24); } /* panels moved between views; keep one gap whatever sits above */
  .link { font: inherit; color: var(--color-paper); background: none; border: 0; padding: 0; cursor: pointer; text-decoration: underline; text-underline-offset: 3px; }
  .link:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  main { max-width: var(--page-max-width); margin: 0 auto; padding: var(--spacing-24) var(--spacing-16); }
  .tag { margin: 0 0 var(--spacing-16); font-size: var(--text-caption); line-height: var(--leading-caption); color: var(--color-ash); }
  .msg { color: var(--color-fog); }
</style>
