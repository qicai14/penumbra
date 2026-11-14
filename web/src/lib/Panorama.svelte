<script lang="ts">
  import { TERRAIN_FAKE, type Scenario } from "./engine";
  import {
    azToX, elToY, fmtDist, fmtEl, fmtUtc, horizonAt, pxPerDegX, pxPerDegY, ticks, trackSegments,
    wrap360, xToAz, BIN_DEG, BINS, type View,
  } from "./panorama";

  let { scenario, idx = $bindable(0) }: { scenario: Scenario; idx?: number } = $props();

  const HEIGHT = 360;
  const PAD = { l: 44, r: 12, t: 12, b: 24 };
  const SPANS = [360, 90, 30] as const;

  let elZoom = $state(false); // false: -10..+20 deg, true: +-10 deg
  let span = $state<number>(360);
  let cursorAz = $state<number | null>(null);
  let width = $state(800);
  let canvas: HTMLCanvasElement;

  const tr = $derived(scenario.track);
  const view = $derived<View>({
    x0: PAD.l, w: Math.max(100, width - PAD.l - PAD.r), y0: PAD.t, h: HEIGHT - PAD.t - PAD.b,
    centre: span === 360 ? 180 : tr.sun_az_deg[idx], span,
    elMin: -10, elMax: elZoom ? 10 : 20,
  });
  const reading = $derived(cursorAz === null ? null : horizonAt(scenario.horizon, cursorAz));

  function css(el: HTMLElement) {
    const s = getComputedStyle(el);
    const c = (n: string) => s.getPropertyValue(n).trim();
    return {
      carbon: c("--color-carbon"), graphite: c("--color-graphite"), iron: c("--color-iron"), slate: c("--color-slate"),
      ash: c("--color-ash"), fog: c("--color-fog"), paper: c("--color-paper"), sun: c("--color-sun"),
      earth: c("--color-earth"), earthLine: c("--color-earth-line"), font: c("--font-mono"), caption: c("--text-caption"),
    };
  }

  function draw() {
    const dpr = window.devicePixelRatio || 1;
    const v = view;
    canvas.width = Math.round(width * dpr);
    canvas.height = Math.round(HEIGHT * dpr);
    const g = canvas.getContext("2d")!;
    g.setTransform(dpr, 0, 0, dpr, 0, 0);
    const k = css(canvas);
    g.fillStyle = k.carbon;
    g.fillRect(0, 0, width, HEIGHT);
    g.font = `${k.caption} ${k.font}`;

    // Gridlines and axis labels.
    const azStep = span === 360 ? 30 : span === 90 ? 10 : 5;
    const elStep = elZoom ? 5 : 10;
    g.lineWidth = 1;
    g.strokeStyle = k.iron;
    g.fillStyle = k.ash;
    g.textAlign = "right"; g.textBaseline = "middle";
    for (const e of ticks(v.elMin, v.elMax, elStep)) {
      const y = Math.round(elToY(e, v)) + 0.5;
      g.beginPath(); g.moveTo(v.x0, y); g.lineTo(v.x0 + v.w, y); g.stroke();
      g.fillText(`${e}°`, v.x0 - 6, y);
    }
    g.textAlign = "center"; g.textBaseline = "top";
    const cardinal: Record<number, string> = { 0: "N", 90: "E", 180: "S", 270: "W" };
    const lo = v.centre - v.span / 2;
    for (const a of ticks(lo, lo + v.span, azStep)) {
      const az = wrap360(a);
      const x = Math.round(azToX(az, v)) + 0.5;
      if (x < v.x0 || x > v.x0 + v.w) continue;
      g.beginPath(); g.moveTo(x, v.y0); g.lineTo(x, v.y0 + v.h); g.stroke();
      g.fillText(cardinal[az] ?? `${az}°`, x, v.y0 + v.h + 6);
    }

    g.save();
    g.beginPath(); g.rect(v.x0, v.y0, v.w, v.h); g.clip();

    // Terrain path: one point per 0.5 deg bin, unwrapped around the view centre.
    const start = Math.round((v.centre - 180) / BIN_DEG);
    const terrain = new Path2D();
    const hz = scenario.horizon;
    for (let j = 0; j <= BINS; j++) {
      const i = (((start + j) % BINS) + BINS) % BINS;
      const x = v.x0 + v.w / 2 + (((start + j) * BIN_DEG - v.centre) / v.span) * v.w;
      const y = elToY(hz.el_deg[i], v);
      if (j === 0) terrain.moveTo(x, y); else terrain.lineTo(x, y);
    }
    const ridge = new Path2D(terrain);
    const xEnd = v.x0 + v.w / 2 + (((start + BINS) * BIN_DEG - v.centre) / v.span) * v.w;
    const xBeg = v.x0 + v.w / 2 + ((start * BIN_DEG - v.centre) / v.span) * v.w;
    terrain.lineTo(xEnd, v.y0 + v.h + 2); terrain.lineTo(xBeg, v.y0 + v.h + 2); terrain.closePath();
    g.fillStyle = k.graphite;
    g.fill(terrain);

    // Tracks over the whole span.
    g.lineJoin = "round";
    g.strokeStyle = k.sun; g.lineWidth = 2.5; g.setLineDash([]);
    for (const seg of trackSegments(tr.sun_az_deg, tr.sun_el_deg, v)) {
      g.beginPath(); seg.forEach(([x, y], i) => (i ? g.lineTo(x, y) : g.moveTo(x, y))); g.stroke();
    }
    g.strokeStyle = k.earthLine; g.lineWidth = 2; g.setLineDash([6, 4]);
    for (const seg of trackSegments(tr.earth_az_deg, tr.earth_el_deg, v)) {
      g.beginPath(); seg.forEach(([x, y], i) => (i ? g.lineTo(x, y) : g.moveTo(x, y))); g.stroke();
    }
    g.setLineDash([]);

    // Ridge line.
    g.strokeStyle = k.ash; g.lineWidth = 1.5; g.stroke(ridge);

    // Discs at the marker time. True angular size: ellipse radii in px per degree on each axis.
    const disc = (az: number, el: number, rDeg: number, fill: string, line: string, label: string, labelColor: string) => {
      const x = azToX(az, v), y = elToY(el, v);
      const rx = Math.max(rDeg * pxPerDegX(v), 0.75), ry = Math.max(rDeg * pxPerDegY(v), 0.75);
      g.beginPath(); g.ellipse(x, y, rx, ry, 0, 0, Math.PI * 2);
      g.strokeStyle = line; g.lineWidth = 1; g.stroke();
      g.save(); // terrain hides what is below the horizon: fill only the part above it

      const above = new Path2D();
      above.rect(v.x0, v.y0, v.w, v.h);
      above.addPath(terrain);
      g.clip(above, "evenodd");
      g.beginPath(); g.ellipse(x, y, rx, ry, 0, 0, Math.PI * 2);
      g.fillStyle = fill; g.fill();
      g.restore();
      g.fillStyle = labelColor; g.textAlign = "left"; g.textBaseline = "bottom";
      g.fillText(label, x + rx + 6, y - ry - 2);
    };
    disc(tr.earth_az_deg[idx], tr.earth_el_deg[idx], tr.earth_radius_deg[idx], k.earth, k.earthLine, "Earth", k.earthLine);
    disc(tr.sun_az_deg[idx], tr.sun_el_deg[idx], tr.sun_radius_deg[idx], k.sun, k.sun, "Sun", k.paper);

    // Pointer / keyboard cursor and its horizon sample.
    if (reading) {
      const x = azToX(reading.az_deg, v);
      g.strokeStyle = k.fog; g.lineWidth = 1; g.setLineDash([2, 3]);
      g.beginPath(); g.moveTo(x, v.y0); g.lineTo(x, v.y0 + v.h); g.stroke();
      g.setLineDash([]);
      g.fillStyle = k.paper;
      g.beginPath(); g.arc(x, elToY(reading.el_deg, v), 3, 0, Math.PI * 2); g.fill();
    }
    g.restore();
  }

  $effect(() => { draw(); });

  function azAt(e: PointerEvent) {
    const r = canvas.getBoundingClientRect();
    const x = e.clientX - r.left;
    return x >= view.x0 && x <= view.x0 + view.w ? xToAz(x, view) : null;
  }
  function onkeydown(e: KeyboardEvent) {
    const d = e.key === "ArrowRight" ? 1 : e.key === "ArrowLeft" ? -1 : 0;
    if (!d) return;
    e.preventDefault();
    cursorAz = wrap360((cursorAz ?? tr.sun_az_deg[idx]) + d * (e.shiftKey ? 5 : BIN_DEG));
  }
</script>

<section class="panel" aria-labelledby="pano-h">
  <header>
    <h2 id="pano-h">Horizon · where the Sun and Earth are</h2>
    {#if TERRAIN_FAKE}<span class="pill" data-testid="fake-badge">FAKE TERRAIN</span>{/if}
  </header>

  <canvas
    bind:this={canvas}
    style:height="{HEIGHT}px"
    style:width="100%"
    tabindex="0"
    aria-label="Horizon panorama. Left and right arrow keys move the cursor in half-degree steps."
    onpointermove={(e) => (cursorAz = azAt(e) ?? cursorAz)}
    onpointerleave={(e) => { if (e.pointerType === "mouse") cursorAz = null; }}
    {onkeydown}
    onblur={() => (cursorAz = null)}
    onfocus={() => (cursorAz ??= tr.sun_az_deg[idx])}
  ></canvas>
  <div class="measure" bind:clientWidth={width}></div>

  <p class="readout" data-testid="readout" aria-live="off">
    {#if reading}
      Az {reading.az_deg.toFixed(1)}° · horizon {fmtEl(reading.el_deg)} · obstacle {fmtDist(reading.dist_m)}{reading.near ? " · within 3 DEM pixels (<60 m)" : ""}
    {:else}
      Point at the panorama, or focus it and use the arrow keys, to read the horizon.
    {/if}
  </p>

  <p class="now" data-testid="now" role="status">
    {fmtUtc(tr.t_unix_s[idx])} · Sun az {tr.sun_az_deg[idx].toFixed(1)}° el {fmtEl(tr.sun_el_deg[idx])} · Earth az {tr.earth_az_deg[idx].toFixed(1)}° el {fmtEl(tr.earth_el_deg[idx])}
  </p>

  <div class="controls">
    <label class="time">
      <span>Time</span>
      <input type="range" min="0" max={tr.t_unix_s.length - 1} step="1" bind:value={idx} aria-label="Time within the span" />
    </label>
    <div class="group" role="group" aria-label="Elevation range">
      <button type="button" aria-pressed={!elZoom} onclick={() => (elZoom = false)}>−10…+20°</button>
      <button type="button" aria-pressed={elZoom} onclick={() => (elZoom = true)}>±10°</button>
    </div>
    <div class="group" role="group" aria-label="Azimuth span">
      {#each SPANS as s (s)}
        <button type="button" aria-pressed={span === s} onclick={() => (span = s)}>{s}°</button>
      {/each}
    </div>
  </div>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); position: relative; }
  header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .pill {
    border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8);
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); text-transform: uppercase; white-space: nowrap;
  }
  canvas { display: block; border-radius: var(--radius-sm); outline-offset: 2px; touch-action: pan-y; }
  canvas:focus-visible { outline: 1px solid var(--color-paper); }
  .measure { height: 0; width: 100%; }
  .readout, .now { margin: var(--spacing-8) 0 0; font: var(--text-body)/var(--leading-body) var(--font-mono); font-variant-numeric: tabular-nums; }
  .readout { color: var(--color-paper); min-height: 1.71em; }
  .now { color: var(--color-fog); font-size: var(--text-caption); line-height: var(--leading-caption); }
  .controls { display: flex; flex-wrap: wrap; gap: var(--spacing-16); align-items: center; margin-top: var(--spacing-16); }
  .time { display: flex; align-items: center; gap: var(--spacing-8); flex: 1 1 240px; font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .time input { flex: 1; accent-color: var(--color-fog); }
  .group { display: flex; }
  button {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 1px solid var(--color-slate); padding: var(--spacing-8) var(--spacing-16); margin-left: -1px; cursor: pointer;
  }
  button:first-child { margin-left: 0; border-radius: var(--radius-sm) 0 0 var(--radius-sm); }
  button:last-child { border-radius: 0 var(--radius-sm) var(--radius-sm) 0; }
  button[aria-pressed="true"] { color: var(--color-paper); background: var(--surface-iron); border-color: var(--color-pewter); position: relative; }
  button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
</style>
