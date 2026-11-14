<script lang="ts">
  import { latlonToXy, xyToLatlon, type EphSegment, type NearTile, type Region } from "./engine";
  import { checkDate, checkPick, dateRange, edgeDistanceM, PICK_BORDER_M, SPANS_DAYS } from "./site";
  import type { Controls } from "./state.svelte";
  import { coalescing, maskAt, maskGrid, maskPixels, MASK_STRIDE, MASK_STRIDE_COARSE, type MaskResult, type Runner } from "./mask";
  import { fmtUtc } from "./panorama";

  // `t` is the shared time marker (Unix s UTC); null until a landing date parses. The mask is drawn for it.
  let { controls, tile, onregion, t = null }: { controls: Controls; tile: NearTile & { regions: Region[]; eph: EphSegment[] }; onregion: (id: string) => void; t?: number | null } = $props();

  let canvas: HTMLCanvasElement;
  // svelte-ignore state_referenced_locally -- drafts start from the current value, then follow typing
  let latDraft = $state(String(controls.lat));
  // svelte-ignore state_referenced_locally
  let lonDraft = $state(String(controls.lon));
  // svelte-ignore state_referenced_locally
  let dateDraft = $state(controls.date);
  let refusal = $state<string | null>(null);
  let rejected = $state<[number, number] | null>(null); // last refused point, map metres

  // A region change moves the site from outside this component: re-sync the boxes and clear the old refusal.
  $effect(() => { latDraft = String(controls.lat); lonDraft = String(controls.lon); refusal = null; rejected = null; });

  // Point-Sun mask (T22): computed by the WASM Engine in a Web Worker; the page thread only paints the returned bits.
  let mask = $state.raw<MaskResult | null>(null);
  let maskError = $state<string | null>(null);
  let maskBusy = $state(false);
  let worker: Worker | undefined;
  const calls = new Map<number, { ok: (v: any) => void; fail: (e: Error) => void }>();
  let callId = 0;
  // A crashed or unloadable worker rejects every pending call and is dropped; the next request builds a fresh one.
  function dropWorker(why: string) {
    worker?.terminate(); worker = undefined;
    for (const c of calls.values()) c.fail(new Error(why));
    calls.clear();
  }
  const run: Runner = (req) => {
    if (!worker) {
      const w = (worker = new Worker(new URL("./maskWorker.ts", import.meta.url), { type: "module" }));
      w.onmessage = (e) => {
        const c = calls.get(e.data.id); calls.delete(e.data.id);
        if (e.data.error) c?.fail(new Error(e.data.error)); else c?.ok(e.data);
      };
      w.onerror = (e) => { if (worker === w) dropWorker(e.message || "the mask worker failed to run"); };
      w.onmessageerror = () => { if (worker === w) dropWorker("the mask worker sent an unreadable message"); };
    }
    return new Promise((ok, fail) => { const id = ++callId; calls.set(id, { ok, fail }); maskBusy = true; worker!.postMessage({ id, ...req }); });
  };
  const requestMask = coalescing(run,
    (r) => { mask = r; maskError = null; maskBusy = false; },
    (e) => { maskError = String((e as Error)?.message ?? e); maskBusy = false; });
  // While the marker moves, a coarse mask (one point per 320 m) keeps the sweep going; 300 ms after it rests, the fine one replaces it.
  $effect(() => {
    if (t === null || !Number.isFinite(t)) return;
    const req = { regionId: tile.region.id, t_unix_s: t, mastM: controls.mastM };
    requestMask({ ...req, stride: MASK_STRIDE_COARSE });
    const rest = setTimeout(() => requestMask({ ...req, stride: MASK_STRIDE }), 300);
    return () => clearTimeout(rest);
  });
  $effect(() => () => worker?.terminate());
  // A mask from another region is not this tile's.
  const shown = $derived(mask && mask.regionId === tile.region.id ? mask : null);

  const dates = $derived(dateRange(tile.eph));
  const m = $derived(tile.meta);
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

  const mapToPx = (x: number, y: number): [number, number] => [(x - m.x0_m) / m.pixel_m, (m.y0_m - y) / m.pixel_m];
  const siteXy = $derived(latlonToXy(controls.lat, controls.lon));
  const siteLit = $derived(shown ? maskAt(shown, m, siteXy[0], siteXy[1]) : null);
  const edgeKm = $derived(edgeDistanceM(siteXy[0], siteXy[1], m) / 1000);

  // Whatever form the token takes (hex, rgb(), named), paint it on a 1x1 canvas and read the pixel back.
  function rgb(color: string): [number, number, number] {
    const px = document.createElement("canvas"); px.width = px.height = 1;
    const g = px.getContext("2d", { willReadFrequently: true })!;
    g.fillStyle = color; g.fillRect(0, 0, 1, 1);
    const d = g.getImageData(0, 0, 1, 1).data;
    return [d[0], d[1], d[2]];
  }
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
      off.getContext("2d")!.putImageData(new ImageData(maskPixels(shown, rgb(c("--color-sun")), 0.6, [0, 0, 0], 0.5), gr.cols, gr.rows), 0, 0);
      g.imageSmoothingEnabled = false;
      g.drawImage(off, 0.5 - gr.stride / 2, 0.5 - gr.stride / 2, gr.cols * gr.stride, gr.rows * gr.stride);
    }
    // Dim the 7 km border: picks there are refused.
    const b = PICK_BORDER_M / m.pixel_m;
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
    refusal = null; rejected = null;
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
</script>

<section class="panel" aria-labelledby="site-h">
  <header>
    <h2 id="site-h">Site and date · where and when you land</h2>
    {#if tile.fake}<span class="pill" data-testid="tile-fake">FAKE TILE</span>{/if}
  </header>

  <div class="body">
    <figure>
      <canvas
        bind:this={canvas}
        {onclick}
        aria-label="Greyscale height image of the {tile.region.name} near tile, overlaid with the point-Sun mask at the time marker. Click to pick a site, or type latitude and longitude."
      ></canvas>
      <div class="legend" data-testid="mask-legend" data-t={shown?.t_unix_s ?? ""}>
        <p class="what"><strong>Point-Sun mask</strong>, not illumination{#if shown}{" · "}{fmtUtc(shown.t_unix_s)}{/if}{#if maskBusy}{" · updating"}{/if}</p>
        <p><i class="sw lit" aria-hidden="true"></i>Sun's centre above the terrain <i class="sw dark" aria-hidden="true"></i>Sun's centre below it</p>
        <p class="note">One ray toward the Sun per {(shown?.stride ?? MASK_STRIDE) * m.pixel_m} m point on this tile. The disc, Earth and terrain beyond the tile are not in it, so points near the edge can read lit.</p>
        {#if maskError}<p class="refusal" role="alert">Mask unavailable. {maskError}</p>{/if}
      </div>
      <figcaption>
        {tile.region.name} · black {grey.lo_m.toFixed(0)} m, white {grey.hi_m.toFixed(0)} m · dashed line: {PICK_BORDER_M / 1000} km from the edge, picks outside it are refused
      </figcaption>
    </figure>

    <div class="fields">
      <label class="row"><span>Region</span>
        <select aria-label="Region" value={tile.region.id} onchange={(e) => onregion(e.currentTarget.value)}>
          {#each tile.regions as r (r.id)}<option value={r.id}>{r.name}</option>{/each}
        </select>
      </label>

      <form class="latlon" {onsubmit}>
        <label><span>Latitude °</span><input data-testid="lat" bind:value={latDraft} inputmode="decimal" autocomplete="off" /></label>
        <label><span>Longitude °E</span><input data-testid="lon" bind:value={lonDraft} inputmode="decimal" autocomplete="off" /></label>
        <button type="submit">Set site</button>
      </form>
      <p class="now" data-testid="site-now">
        Site {controls.lat.toFixed(4)}°, {controls.lon.toFixed(4)}°E · {edgeKm.toFixed(1)} km from the tile edge
      </p>
      <p class="now" role="status" aria-live="polite" aria-atomic="true" data-testid="mask-site" data-lit={siteLit ?? ""} data-t={shown?.t_unix_s ?? ""} data-stride={shown?.stride ?? ""}>
        Nearest mask point (within {((shown?.stride ?? MASK_STRIDE) * m.pixel_m) / 2} m of the site), point Sun: {siteLit === null ? "computing" : siteLit ? "lit" : "in shadow"}
      </p>

      <label class="row"><span>Landing date (UTC)</span>
        <input data-testid="date" type="date" min={dates.min} max={dates.max} bind:value={dateDraft} onchange={ondate} />
      </label>

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
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-bottom: var(--spacing-16); }
  header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .pill {
    border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8);
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); text-transform: uppercase; white-space: nowrap;
  }
  .body { display: flex; flex-wrap: wrap; gap: var(--spacing-24); align-items: flex-start; }
  figure { margin: 0; flex: 0 1 384px; min-width: 240px; }
  canvas { display: block; width: 100%; aspect-ratio: 1; border-radius: var(--radius-sm); cursor: crosshair; image-rendering: pixelated; }
  .legend { margin-top: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); }
  .legend p { margin: 0 0 4px; }
  .legend .what strong { font-weight: var(--font-weight-regular); color: var(--color-paper); }
  .legend .note { color: var(--color-ash); }
  .sw { display: inline-block; width: 12px; height: 12px; margin: 0 var(--spacing-8) -2px 0; border: 1px solid var(--color-slate); }
  .sw.lit { background: color-mix(in srgb, var(--color-sun) 60%, var(--color-slate)); }
  .sw.dark { background: color-mix(in srgb, black 50%, var(--color-fog)); margin-left: var(--spacing-16); }
  figcaption { margin-top: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .fields { flex: 1 1 280px; display: flex; flex-direction: column; gap: var(--spacing-16); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .row { display: flex; align-items: center; gap: var(--spacing-8); }
  .row > span:first-child { flex: 0 0 150px; }
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
