<script lang="ts">
  import { latlonToXy, TIER_TEXT, xyToLatlon, type EphSegment, type NearTile, type Region } from "./engine";
  import { checkDate, checkPick, dateRange, edgeDistanceM, isoDay, parseDate, pickBorderM, slopeText, SPANS_DAYS } from "./site";
  import type { Controls } from "./state.svelte";
  import { maskAt, maskGrid, maskPixels, MASK_STRIDE, MASK_LIT_ALPHA, MASK_DARK_ALPHA } from "./mask";
  import { rgb } from "./color";
  import type { MaskFeed } from "./maskFeed.svelte";
  import { fmtUtc } from "./panorama";
  import Hero from "./Hero.svelte";
  import { coverageRegion, latLonText, MOON_TREK_URL, QUICKMAP_URL, SITES, type CuratedSite } from "./sites";
  import { computeSlope, loadManifest } from "./engine";
  import { outsideAvgvisib, rankBests } from "./ranking";
  import { bestCell, contour, CONTOUR_H, dateIndex, NODATA, NO_LETHAL, type Grid } from "./grid";

  // `feed` is the shared point-Sun mask for the time marker; the 3D terrain paints the same one.
  let { controls, tile, onregion, feed, grid = null, gridK = -1, grids = [] }: { controls: Controls; tile: NearTile & { regions: Region[]; eph: EphSegment[] }; onregion: (id: string, at?: [number, number]) => void; feed: MaskFeed; grid?: Grid | null; gridK?: number; grids?: Grid[] } = $props();

  let canvas: HTMLCanvasElement;
  // svelte-ignore state_referenced_locally -- drafts start from the current value, then follow typing
  let latDraft = $state(String(controls.lat));
  // svelte-ignore state_referenced_locally
  let lonDraft = $state(String(controls.lon));
  // svelte-ignore state_referenced_locally
  let dateDraft = $state(controls.date);
  // The landing window (#108) sets controls.date from outside; keep the input in step.
  $effect(() => { dateDraft = controls.date; });
  let refusal = $state<string | null>(null);
  let rejected = $state<[number, number] | null>(null); // last refused point, map metres
  let regionMetas = $state.raw<{ id: string; near: NearTile["meta"] }[]>([]); // every region's near tile, for the curated-site coverage check
  loadManifest().then((mf) => (regionMetas = mf.regions), () => {}); // no manifest: the list stays disabled, the tile itself already reports it
  let outSite = $state<CuratedSite | null>(null); // a curated site chosen outside every tile: the site does not move
  let copied = $state("");

  // A region change moves the site from outside this component: re-sync the boxes and clear the old refusal.
  $effect(() => { latDraft = String(controls.lat); lonDraft = String(controls.lon); refusal = null; rejected = null; outSite = null; copied = ""; });

  // A mask from another region is not this tile's.
  const shown = $derived(feed.mask && feed.mask.regionId === tile.region.id ? feed.mask : null);

  const dates = $derived(dateRange(tile.eph));
  const m = $derived(tile.meta);
  const borderM = $derived(pickBorderM(m)); // per tile: the map length of 7 km of ground grows away from the pole
  const heightM = (v: number) => m.offset_m + m.scale_m * v;

  // Grey image straight from the int16 counts; nodata stays transparent.
  const grey = $derived.by(() => {
    let lo = Infinity, hi = -Infinity;
    for (const v of tile.data) if (v !== m.nodata) { if (v < lo) lo = v; if (v > hi) hi = v; }
    const img = new ImageData(m.width, m.height);
    const k = hi > lo ? 255 / (hi - lo) : 0;
    for (let i = 0; i < tile.data.length; i++) {
      const v = tile.data[i];
      if (v === m.nodata) continue;
      const g = Math.round((v - lo) * k);
      img.data.set([g, g, g, 255], i * 4);
    }
    return { img, lo_m: heightM(lo), hi_m: heightM(hi) };
  });

  const HEAT_ALPHA = 200;
  const mapToPx = (x: number, y: number): [number, number] => [(x - m.x0_m) / m.pixel_m, (m.y0_m - y) / m.pixel_m];
  const siteXy = $derived(latlonToXy(controls.lat, controls.lon));
  const siteLit = $derived(shown ? maskAt(shown, m, siteXy[0], siteXy[1]) : null);
  const edgeKm = $derived(edgeDistanceM(siteXy[0], siteXy[1], m) / 1000);
  // Slope from the Engine (20 m DEM). Cleared first so a stale value never shows for a new pin; undefined = no line.
  let slope = $state<number | undefined>(undefined);
  $effect(() => {
    const lat = controls.lat, lon = controls.lon, region = tile.region.id;
    let live = true;
    slope = undefined;
    computeSlope(lat, lon, region).then((v) => { if (live) slope = v; }, () => {});
    return () => { live = false; };
  });
  const slopeLine = $derived(slopeText(slope));

  function draw() {
    const g = canvas.getContext("2d")!;
    const s = getComputedStyle(canvas);
    const c = (n: string) => s.getPropertyValue(n).trim();
    canvas.width = m.width; canvas.height = m.height;
    g.putImageData(grey.img, 0, 0);
    if (shown) {
      // Sunlit points take the Sun's ember, shadowed points go dark; one mask point covers stride x stride tile pixels.
      const gr = maskGrid(m, shown.stride);
      const off = document.createElement("canvas");
      off.width = gr.cols; off.height = gr.rows;
      off.getContext("2d")!.putImageData(new ImageData(maskPixels(shown, rgb(c("--color-sun")), MASK_LIT_ALPHA, [0, 0, 0], MASK_DARK_ALPHA), gr.cols, gr.rows), 0, 0);
      g.imageSmoothingEnabled = false;
      g.drawImage(off, 0.5 - gr.stride / 2, 0.5 - gr.stride / 2, gr.cols * gr.stride, gr.rows * gr.stride);
    }
    if (grid && gridK >= 0) {
      // Regional point grid (derived operable hours): sequential steel-to-earth-line ramp (steel keeps the low end at 3:1 on carbon) (the mask keeps the Sun colour), one line at 5 operable days.
      const gm = grid.meta, lo = rgb(c("--color-steel")), hi = rgb(c("--color-earth-line")), top = gm.span_days * 24;
      const img = new ImageData(gm.width, gm.height), a = grid.operable[gridK];
      for (let i = 0; i < a.length; i++) {
        if (a[i] === NODATA) continue;
        const t = Math.min(1, a[i] / top);
        img.data.set([0, 1, 2].map((k) => Math.round(lo[k] + t * (hi[k] - lo[k]))).concat(HEAT_ALPHA), i * 4);
      }
      const off = document.createElement("canvas");
      off.width = gm.width; off.height = gm.height;
      off.getContext("2d")!.putImageData(img, 0, 0);
      const k = gm.pixel_m / m.pixel_m, [gx, gy] = mapToPx(gm.x0_m, gm.y0_m);
      g.imageSmoothingEnabled = false;
      g.drawImage(off, gx, gy, gm.width * k, gm.height * k);
      g.strokeStyle = c("--color-paper"); g.lineWidth = 1.5;
      g.beginPath();
      for (const [x1, y1, x2, y2] of contour(grid, gridK)) { const [a1, b1] = mapToPx(x1, y1), [a2, b2] = mapToPx(x2, y2); g.moveTo(a1, b1); g.lineTo(a2, b2); }
      g.stroke();
    }
    // Dim the border (7 km of ground): picks there are refused.
    const b = borderM / m.pixel_m;
    const shade = new Path2D();
    shade.rect(0, 0, m.width, m.height); shade.rect(b, b, m.width - 2 * b, m.height - 2 * b);
    g.fillStyle = "rgba(0,0,0,0.6)"; g.fill(shade, "evenodd");
    g.strokeStyle = c("--color-fog"); g.lineWidth = 1; g.setLineDash([3, 3]);
    g.strokeRect(b + 0.5, b + 0.5, m.width - 2 * b, m.height - 2 * b);
    g.setLineDash([]);
    const mark = ([x, y]: [number, number], col: string, fill: boolean) => {
      const [px, py] = mapToPx(x, y);
      g.strokeStyle = col; g.fillStyle = col; g.lineWidth = 2;
      g.beginPath(); g.arc(px, py, 5, 0, Math.PI * 2);
      if (fill) g.fill(); else g.stroke();
    };
    if (rejected) mark(rejected, c("--color-ash"), false);
    mark(siteXy, c("--color-paper"), true);
  }
  $effect(() => { draw(); });

  function setSite(lat: number, lon: number) {
    controls.lat = lat; controls.lon = lon;
    latDraft = String(lat); lonDraft = String(lon);
    refusal = null; rejected = null; outSite = null; copied = "";
  }
  function onsite(id: string) {
    const s = SITES.find((x) => x.id === id);
    if (!s) return;
    const r = coverageRegion(s.lat, s.lon, regionMetas);
    if (r === null) { outSite = s; refusal = null; rejected = null; return; }
    if (r === tile.region.id) setSite(s.lat, s.lon); else onregion(r, [s.lat, s.lon]);
  }
  const linkSite = $derived<[number, number]>(outSite ? [outSite.lat, outSite.lon] : [controls.lat, controls.lon]);
  const linkText = $derived(latLonText(linkSite[0], linkSite[1]));
  async function copy() {
    try { await navigator.clipboard.writeText(linkText); copied = "Copied"; } catch { copied = "Copy blocked: select the text instead"; }
  }
  // P5 / ARCHITECTURE §2.1: a nodata pixel is an error, never 0 m.
  function verdict(x: number, y: number) {
    const v = checkPick(x, y, m);
    if (!v.ok) return v;
    const [px, py] = mapToPx(x, y);
    return tile.data[Math.floor(py) * m.width + Math.floor(px)] === m.nodata
      ? { ok: false as const, reason: "No terrain data at that point. Pick another site." }
      : v;
  }
  function pick(x: number, y: number) {
    const v = verdict(x, y);
    if (!v.ok) { refusal = v.reason; rejected = [x, y]; return; }
    const [lat, lon] = xyToLatlon(x, y);
    setSite(+lat.toFixed(4), +lon.toFixed(4));
  }
  // Region ranking on the picked date: best cell of each grid (first in row-major order on a tie), most operable hours first,
  // a later first lethal shadow breaking a tie; a region whose grid lacks the date goes last, unranked.
  const bests = $derived(rankBests(grids.map((g) => {
    const k = dateIndex(g, controls.date), b = k < 0 ? null : bestCell(g, k);
    const [lat, lon] = b ? xyToLatlon(b.x, b.y) : [0, 0];
    const l = b ? g.lethal[k][b.j * g.meta.width + b.i] : NODATA;
    const lethalH = l === NODATA ? undefined : l === NO_LETHAL ? Infinity : l; // undefined: unknown, Infinity: none in span
    const reg = tile.regions.find((r) => r.id === g.meta.region), near = regionMetas.find((r) => r.id === g.meta.region)?.near;
    const tileLat = near ? xyToLatlon(near.x0_m + (near.width * near.pixel_m) / 2, near.y0_m - (near.height * near.pixel_m) / 2)[0] : -90;
    const mark = [reg?.tier === 3 ? "validation tile, not a 2024 region" : reg?.tier === 2 ? "IM-2 site tile" : "", outsideAvgvisib(tileLat) ? "not checked against LOLA AVGVISIB (north of 85°S)" : ""].filter(Boolean).join(" · ");
    return { g, b, lat: +lat.toFixed(4), lon: +lon.toFixed(4), lethalH, name: reg?.name ?? g.meta.region, mark };
  })));
  const gridDiffers = $derived(grids.length > 0 && (controls.preset !== null || controls.mastM !== grids[0].meta.mast_m || controls.spanDays !== grids[0].meta.span_days));
  const noGridMsg = "Region ranking and heatmap are precomputed for 3 landing dates only (data budget). This site's own timelines and headline still work for any date.";
  const gridDateList = $derived([...new Set(grids.flatMap((g) => g.meta.dates))].sort());
  const allUnranked = $derived(bests.length > 0 && bests.every((x) => !x.b));
  const notes = $derived([...new Set(bests.flatMap((x) => (x.mark ? x.mark.split(" · ") : [])))]); // each caveat once, as a numbered footnote
  function setDate(d: string) { if (tryDate(d, controls.spanDays)) { controls.date = d; dateDraft = d; } }
  function goBest(id: string, lat: number, lon: number) {
    if (id === tile.region.id) setSite(lat, lon); else onregion(id, [lat, lon]);
  }
  function onclick(e: MouseEvent) {
    const r = canvas.getBoundingClientRect();
    pick(m.x0_m + ((e.clientX - r.left) / r.width) * m.width * m.pixel_m, m.y0_m - ((e.clientY - r.top) / r.height) * m.height * m.pixel_m);
  }
  function onsubmit(e: SubmitEvent) {
    e.preventDefault();
    const lat = Number(latDraft), lon = Number(lonDraft);
    if (!latDraft.trim() || !lonDraft.trim() || !Number.isFinite(lat) || !Number.isFinite(lon) || lat < -90 || lat > 90 || lon < -180 || lon > 360) {
      refusal = "Enter a latitude between −90 and 90 and a longitude between −180 and 360."; return;
    }
    const [x, y] = latlonToXy(lat, lon);
    const v = verdict(x, y);
    if (!v.ok) { refusal = v.reason; rejected = [x, y]; return; }
    setSite(lat, lon);
  }
  function tryDate(date: string, span: number): boolean {
    const v = checkDate(date, span, tile.eph);
    refusal = v.ok ? null : v.reason;
    return v.ok;
  }
  const ondate = () => { if (tryDate(dateDraft, controls.spanDays)) controls.date = dateDraft; else dateDraft = controls.date; };
  const onspan = (s: number) => { if (tryDate(controls.date, s)) controls.spanDays = s; };
  // Hero scenarios: only the picker's own setters. IM-2 comes from the curated list; best region is the top of `bests`.
  const HERO_DATE = "2026-11-14"; // the IM-2 landing date the first scenario uses
  const heroIm2 = (date: string) => { onsite("im2"); setDate(date); };
  const heroSlip = () => heroIm2(isoDay(parseDate(HERO_DATE)! - 2 * 86400));
  function heroBest() { const t = bests[0]; if (t?.b) goBest(t.g.meta.region, t.lat, t.lon); }
</script>

<Hero onim2={() => heroIm2(HERO_DATE)} onbest={heroBest} onslip={heroSlip} ready={regionMetas.length > 0} bestReady={!!bests[0]?.b} rankedDates={gridDateList} />

<section class="panel" aria-labelledby="site-h">
  <h2 id="site-h">Site and date</h2>

  <div class="body">
    <figure>
      <canvas
        bind:this={canvas}
        {onclick}
        aria-label="Greyscale height image of the {tile.region.name} near tile, overlaid with the point-Sun mask at the time marker. Click to pick a site, or type latitude and longitude."
      ></canvas>
      <div class="legend" data-testid="mask-legend" data-t={shown?.t_unix_s ?? ""}>
        <p class="what"><strong>Point-Sun mask</strong>, not illumination{#if shown}{" · "}{fmtUtc(shown.t_unix_s)}{/if}{#if feed.busy}{" · updating"}{/if}</p>
        <p><i class="sw lit" aria-hidden="true"></i>Sun's centre above the terrain <i class="sw dark" aria-hidden="true"></i>Sun's centre below it</p>
        <details class="more" data-testid="map-notes">
          <summary>How to read the map</summary>
          <p class="note">Orange ground has the Sun in view at the chosen time; dark ground is in its shadow. The coloured heatmap scores each spot by how many days a lander there could work.</p>
          <p class="note">One ray toward the Sun per {(shown?.stride ?? MASK_STRIDE) * m.pixel_m} m point on this tile. The disc, Earth and terrain beyond the tile are not in it, so points near the edge can read lit.</p>
          <p class="note">{tile.region.name} · black {grey.lo_m.toFixed(0)} m, white {grey.hi_m.toFixed(0)} m · dashed line: {(borderM / 1000).toFixed(2)} km from the edge, picks outside it are refused</p>
        </details>
        {#if feed.error}<p class="refusal" role="alert">Mask unavailable. {feed.error}</p>{/if}
      </div>
      {#if grid}
        <div class="legend" data-testid="heat-legend">
          {#if gridK >= 0}
            <p class="what"><strong>Operable days</strong>, regional terrain, point grid {grid.meta.pixel_m} m</p>
            <p><i class="ramp" aria-hidden="true"></i>0 to {grid.meta.span_days} operable d <i class="line" aria-hidden="true"></i>{CONTOUR_H / 24} d line</p>
            <p class="note">Landing {controls.date} 00:00 UTC,{grid.meta.span_days} d span, mast {grid.meta.mast_m} m: set at grid build, not the controls on the right.</p>
          {:else}
            <p class="note" data-testid="heat-dates">Regional terrain, point grid {grid.meta.pixel_m} m: no heatmap for {controls.date}; pick one of the three grid dates.</p>
          {/if}
        </div>
      {/if}
    </figure>

    <div class="fields">
      <label class="row"><span>Region</span>
        <select aria-label="Region" value={tile.region.id} onchange={(e) => onregion(e.currentTarget.value)}>
          {#each tile.regions as r (r.id)}<option value={r.id}>{r.name}{r.tier ? ` · tier ${r.tier}` : ""}</option>{/each}
        </select>
      </label>
      {#if tile.region.tier || tile.region.label}
        <p class="now region-note" data-testid="region-label">{#if tile.region.tier}Tier {tile.region.tier}, {TIER_TEXT[tile.region.tier]}.{/if} {tile.region.label ?? ""}</p>
      {/if}

      <label class="row"><span>Curated site</span>
        <select data-testid="curated" aria-label="Curated site" disabled={!regionMetas.length} value="" onchange={(e) => { onsite(e.currentTarget.value); e.currentTarget.value = ""; }}>
          <option value="">Choose a site…</option>
          {#each SITES as s (s.id)}<option value={s.id}>{s.name}{regionMetas.length && coverageRegion(s.lat, s.lon, regionMetas) === null ? " · out of coverage" : ""}</option>{/each}
        </select>
      </label>
      {#if outSite}
        <p class="oc" role="status" data-testid="out-of-coverage">
          OUT_OF_COVERAGE · {outSite.name} ({latLonText(outSite.lat, outSite.lon)}) is outside every shipped near tile, so the model does not run there. The site stays at {controls.lat.toFixed(4)}°, {controls.lon.toFixed(4)}°E.
        </p>
      {/if}

      <form class="latlon" {onsubmit}>
        <label><span>Latitude °</span><input data-testid="lat" bind:value={latDraft} inputmode="decimal" autocomplete="off" /></label>
        <label><span>Longitude °E</span><input data-testid="lon" bind:value={lonDraft} inputmode="decimal" autocomplete="off" /></label>
        <button type="submit">Set site</button>
      </form>
      <p class="now" data-testid="site-now">
        {edgeKm.toFixed(1)} km from the tile edge
      </p>
      {#if slopeLine}<p class="now" data-testid="slope">{slopeLine}</p>{/if}
      <p class="now" role="status" aria-live="polite" aria-atomic="true" data-testid="mask-site" data-lit={siteLit ?? ""} data-t={shown?.t_unix_s ?? ""} data-stride={shown?.stride ?? ""}>
        Nearest mask point (within {((shown?.stride ?? MASK_STRIDE) * m.pixel_m) / 2} m of the site), point Sun: {siteLit === null ? "computing" : siteLit ? "lit" : "in shadow"}
      </p>

      <div class="tools" data-testid="tool-links">
        <p>Look at {outSite ? outSite.name : "the current site"} in a map tool:</p>
        <p>
          <a data-testid="quickmap" href={QUICKMAP_URL} target="_blank" rel="noopener" aria-label="Open in QuickMap (opens in a new tab)">Open in QuickMap</a>
          <a data-testid="moontrek" href={MOON_TREK_URL} target="_blank" rel="noopener" aria-label="Open in Moon Trek (opens in a new tab)">Open in Moon Trek</a>
        </p>
        <p>Paste into the tool's search box: <code data-testid="latlon-text">{linkText}</code> <button type="button" onclick={copy}>Copy</button> <span role="status" aria-live="polite">{copied}</span></p>
      </div>

      <label class="row"><span>Landing date (UTC)</span>
        <input data-testid="date" type="date" min={dates.min} max={dates.max} bind:value={dateDraft} onchange={ondate} />
      </label>
      <p class="now" data-testid="date-note">Ranking and heatmap exist for 3 landing dates only (data budget); timelines and headline work for any date.</p>

      <div class="row"><span id="span-l">Span</span>
        <div class="group" role="group" aria-labelledby="span-l">
          {#each SPANS_DAYS as s (s)}
            <button type="button" aria-pressed={controls.spanDays === s} onclick={() => onspan(s)}>{s} d</button>
          {/each}
        </div>
      </div>

      <label class="row"><span>Mast height</span>
        <input data-testid="mast" type="range" min="0" max="10" step="0.5" bind:value={controls.mastM} />
        <output>{controls.mastM.toFixed(1)} m</output>
      </label>

      {#if refusal}
        <p class="refusal" role="alert" data-testid="refusal">Refused. {refusal}</p>
      {/if}
    </div>

    {#if grids.length}
      <div class="best" data-testid="best-cells">
        {#if allUnranked}
        <h3 data-testid="ranking-title">Region ranking for {controls.date} · default preset, mast {grids[0].meta.mast_m} m, {grids[0].meta.span_days} d span</h3>
          <p class="gridmsg" data-testid="no-grid-msg">{noGridMsg}</p>
          <div class="chips">{#each gridDateList as d (d)}<button type="button" data-testid="date-chip" onclick={() => setDate(d)}>{d}</button>{/each}</div>
        {:else}
        <!-- A closed dropdown: the summary already names the date and the #1 site; open it for all ten. -->
        <details class="rankbox" data-testid="ranking">
        <summary><span data-testid="ranking-title">Region ranking for {controls.date} · default preset, mast {grids[0].meta.mast_m} m, {grids[0].meta.span_days} d span</span>{#if bests[0]?.b}<span class="top" data-testid="ranking-top">{" · best: "}{bests[0].name}, {(bests[0].b.operableH / 24).toFixed(1)} operable days</span>{/if}</summary>
        <!-- One table, not ten cards: every region stays comparable at a glance; the row or its → moves the site to that best cell. -->
        <table class="rank" aria-label="Ranked best cells, one row per tile">
          <thead><tr><th scope="col">#</th><th scope="col">Region</th><th scope="col" class="num">Operable</th><th scope="col">First lethal shadow</th><th scope="col"><span class="sr-only">Move site</span></th></tr></thead>
          <tbody>
          {#each bests as { g, b, lat, lon, lethalH, name, mark }, n (g.meta.region)}
            {@const refs = mark ? mark.split(" · ").map((x) => notes.indexOf(x) + 1) : []}
            <tr data-testid="best-{g.meta.region}" onclick={b ? () => goBest(g.meta.region, lat, lon) : undefined} class:go={b}>
              <td class="n">{b ? n + 1 : "–"}</td>
              <td class="who">{name}{#if refs.length}<sup data-testid="mark-{g.meta.region}" title={mark}>{refs.join(",")}</sup>{/if}</td>
              {#if b}
                <td class="num days">{(b.operableH / 24).toFixed(1)} d</td>
                <td class="lethal" title={lethalH === Infinity ? "No lethal shadow in the span" : undefined}>{lethalH === undefined ? "unknown" : lethalH === Infinity ? "none" : `+${(lethalH / 24).toFixed(1)} d`}</td>
                <td class="act"><button type="button" aria-label="Move site to {name} best cell, {lat.toFixed(4)}°, {lon.toFixed(4)}°E" title="{lat.toFixed(4)}°, {lon.toFixed(4)}°E" onclick={(e) => { e.stopPropagation(); goBest(g.meta.region, lat, lon); }}>→</button></td>
              {:else}
                <td colspan="3">No grid for this date.</td>
              {/if}
            </tr>
          {/each}
          </tbody>
        </table>
        {#if notes.length}<p class="note" data-testid="ranking-notes">{#each notes as x, i (x)}<span><sup>{i + 1}</sup> {x}</span>{/each}</p>{/if}
        <details class="more">
          <summary>How it's ranked</summary>
          <p class="note" data-testid="ranking-scope">The ranking covers each shipped tile's pickable square (about 16.6 km a side), not the whole region: Haworth is 888 km², Mons Mouton Plateau 4452 km².</p>
          <p class="note">Ranked by operable hours, then by a later first lethal shadow. Best cell of each {grids[0].meta.pixel_m} m point grid on the landing date; a tie goes to the first cell, top row then left to right on the map. Built for the default preset, mast {grids[0].meta.mast_m} m and {grids[0].meta.span_days} d span.</p>
        </details>
        </details>
        {/if}
        {#if gridDiffers}<p class="note warn">Your controls differ from the default preset, mast {grids[0].meta.mast_m} m and {grids[0].meta.span_days} d span, so these days will not match the headline.</p>{/if}
      </div>
    {/if}
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .body { display: flex; flex-wrap: wrap; gap: var(--spacing-24); align-items: flex-start; }
  figure { margin: 0; flex: 0 1 384px; min-width: 240px; }
  canvas { display: block; width: 100%; aspect-ratio: 1; border-radius: var(--radius-sm); cursor: crosshair; image-rendering: pixelated; }
  .legend { margin-top: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  .legend p { margin: 0 0 4px; }
  .legend .what strong { font-weight: var(--font-weight-regular); color: var(--color-paper); }
  .legend .note { color: var(--color-ash); }
  .legend .more { margin: 0 0 4px; }
  .legend .more > summary { cursor: pointer; width: fit-content; color: var(--color-fog); text-decoration: underline; text-underline-offset: 4px; }
  .legend .more > summary:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .sw { display: inline-block; width: 12px; height: 12px; margin: 0 var(--spacing-8) -2px 0; border: 1px solid var(--color-slate); }
  .sw.lit { background: color-mix(in srgb, var(--color-sun) 60%, var(--color-slate)); }
  .sw.dark { background: color-mix(in srgb, black 50%, var(--color-fog)); margin-left: var(--spacing-16); }
  .ramp { display: inline-block; width: 48px; height: 10px; margin: 0 var(--spacing-8) 0 0; background: linear-gradient(to right, var(--color-steel), var(--color-earth-line)); }
  .line { display: inline-block; width: 16px; border-top: 2px solid var(--color-paper); margin: 0 var(--spacing-8) 3px var(--spacing-16); }
  .fields { flex: 1 1 280px; min-width: 0; display: flex; flex-direction: column; gap: var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .row { display: flex; align-items: center; gap: var(--spacing-8); }
  .row > span:first-child { flex: 0 0 150px; }
  .row > select { flex: 1 1 0; width: 0; min-width: 0; } /* a long option name must not widen the row past a 375 px screen */
  @media (max-width: 480px) { .row > span:first-child { flex-basis: 96px; } }
  .row input[type="range"] { flex: 1; accent-color: var(--color-fog); }
  output { min-width: 5ch; color: var(--color-paper); font-variant-numeric: tabular-nums; }
  .latlon { display: flex; flex-wrap: wrap; gap: var(--spacing-8); align-items: flex-end; }
  .latlon label { display: flex; flex-direction: column; gap: 4px; flex: 1 1 110px; }
  input:not([type="range"]), select {
    font: var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-paper); background: var(--surface-iron);
    border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: 4px var(--spacing-8); min-width: 0; color-scheme: dark;
  }
  .now { margin: calc(-1 * var(--spacing-8)) 0 0; color: var(--color-fog); font-variant-numeric: tabular-nums; }
  .refusal { margin: 0; padding: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-pewter); border-radius: var(--radius-sm); background: var(--surface-iron); color: var(--color-paper); }
  .oc { margin: 0; padding: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-slate); border-radius: var(--radius-sm); background: var(--surface-iron); color: var(--color-paper); }
  .tools p { margin: 0 0 4px; }
  .tools a { color: var(--color-fog); margin-right: var(--spacing-16); }
  .tools a:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  @media (pointer: coarse) { .tools a { display: inline-flex; align-items: center; min-height: 44px; } } /* touch: 44 px target like the buttons */
  code { color: var(--color-paper); font-family: inherit; user-select: all; }
  .best { flex: 1 1 100%; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); display: grid; grid-template-columns: 1fr; gap: var(--spacing-8) var(--spacing-16); border: 1px solid var(--color-slate); border-radius: var(--radius-sm); padding: var(--spacing-8) var(--spacing-16); }
  .best h3 { grid-column: 1 / -1; margin: 0; font: var(--font-weight-regular) var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-paper); }
  .best .note { margin: 0 0 4px; }
  .rankbox > summary { cursor: pointer; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-paper); }
  .rankbox > summary:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .rankbox > summary .top { color: var(--color-fog); }
  .rankbox[open] > summary { margin-bottom: var(--spacing-8); }
  .rankbox > .note, .rankbox > .more { margin-top: var(--spacing-8); }
  .region-note { margin: calc(-1 * var(--spacing-8)) 0 0; color: var(--color-ash); font-variant-numeric: normal; }
  .rank { grid-column: 1 / -1; width: 100%; border-collapse: collapse; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); font-variant-numeric: tabular-nums; }
  .rank th { text-align: left; font-weight: var(--font-weight-regular); color: var(--color-ash); padding: 0 var(--spacing-8) 4px 0; border-bottom: 1px solid var(--color-iron); }
  .rank td { padding: 2px var(--spacing-8) 2px 0; border-bottom: 1px solid var(--color-iron); vertical-align: middle; }
  .rank .num { text-align: right; }
  .rank .days, .rank .lethal { white-space: nowrap; }
  .rank .n { color: var(--color-ash); width: 2.5ch; }
  .rank .who { color: var(--color-paper); }
  .rank .days { color: var(--color-paper); }
  .rank sup { color: var(--color-ash); margin-left: 2px; }
  .rank .act { width: 1%; padding-right: 0; text-align: right; }
  .rank .act button { padding: 2px var(--spacing-8); }
  .rank tr.go { cursor: pointer; }
  .rank tr.go:hover td { background: var(--surface-iron); }
  .best .note sup { margin-right: 2px; }
  .best .note span + span::before { content: " · "; }
  .best .warn { color: var(--color-paper); }
  .best .more > summary { cursor: pointer; width: fit-content; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); text-decoration: underline; text-underline-offset: 4px; }
  .best .more > summary:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .best .more > .note { margin-top: 4px; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  .gridmsg { grid-column: 1 / -1; margin: 0; color: var(--color-paper); }
  .chips { grid-column: 1 / -1; display: flex; flex-wrap: wrap; gap: var(--spacing-8); }
  .chips button { margin-left: 0; border-radius: var(--radius-sm); min-height: 44px; }
  .best .note { grid-column: 1 / -1; color: var(--color-ash); }
  .best button { margin-left: 0; border-radius: var(--radius-sm); }
  .group { display: flex; }
  button {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 1px solid var(--color-slate); padding: var(--spacing-8) var(--spacing-16); margin-left: -1px; cursor: pointer;
  }
  .group button:first-child { margin-left: 0; border-radius: var(--radius-sm) 0 0 var(--radius-sm); }
  .group button:last-child { border-radius: 0 var(--radius-sm) var(--radius-sm) 0; }
  .latlon button { margin-left: 0; border-radius: var(--radius-sm); }
  button[aria-pressed="true"] { color: var(--color-paper); background: var(--surface-iron); border-color: var(--color-pewter); }
  button:focus-visible, input:focus-visible, select:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
</style>
