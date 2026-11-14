<script lang="ts">
  import { untrack } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { windowBands, type Scenario } from "./engine";
  import { fmtUtc } from "./panorama";
  import { type Band, bandInterval, bandLabel, parseBands, SIGMA_Z_M, SIGMA_Z_TEXT, buildChartData, buildSocData, citedTag, clipRunsAt, indexAtTime, lethalUnixS, operableMask, operableRuns, thresholdLabel } from "./timelines";

  let { scenario, idx = $bindable(0) }: { scenario: Scenario; idx: number } = $props();

  const SYNC = "penumbra-timelines";
  let width = $state(800);
  let host: HTMLDivElement;
  let socHost: HTMLDivElement;
  let charts: uPlot[] = [];
  let full: [number, number] = [0, 1]; // the whole span, set by build
  let vis = $state<[number, number]>([0, 1]); // the visible x range, from the charts
  let placing = false; // true while we move the cursors ourselves, so the cursor hook does not echo back

  let bands = $state.raw<Band[]>([]);
  let bandError = $state("");
  const tl = $derived(scenario.timeline);
  const label = $derived(thresholdLabel(scenario.preset));

  function colours() {
    const s = getComputedStyle(host);
    const c = (n: string) => s.getPropertyValue(n).trim();
    return {
      iron: c("--color-iron"), ash: c("--color-ash"), fog: c("--color-fog"), paper: c("--color-paper"),
      sun: c("--color-sun"), earthLine: c("--color-earth-line"), font: `${c("--text-caption")} ${c("--font-mono")}`,
    };
  }

  /** Shades the operable window behind the series, from the model's illumination and contact columns. */
  function bandPlugin(runs: [number, number][], fill: string): uPlot.Plugin {
    return {
      hooks: {
        drawClear: (u) => {
          const { ctx, bbox } = u;
          ctx.save();
          ctx.fillStyle = fill;
          for (const [a, b] of runs) {
            const x0 = Math.max(bbox.left, u.valToPos(a, "x", true));
            const x1 = Math.min(bbox.left + bbox.width, u.valToPos(b, "x", true));
            if (x1 > x0) ctx.fillRect(x0, bbox.top, x1 - x0, bbox.height);
          }
          ctx.restore();
        },
      },
    };
  }

  /** Sunrise and sunset ± uncertainty: a hatched interval around each crossing with its label, from the WASM window_bands. Not the operable band. */
  function uncertaintyPlugin(getList: () => Band[], colour: string, chip: string, font: string): uPlot.Plugin {
    let hatch: CanvasPattern | null = null;
    return {
      hooks: {
        draw: (u) => {
          const list = getList();
          const { ctx, bbox } = u;
          const px = uPlot.pxRatio;
          if (!hatch) {
            const c = document.createElement("canvas"); c.width = c.height = 6 * px;
            const g = c.getContext("2d")!; g.strokeStyle = colour; g.lineWidth = px;
            g.beginPath(); g.moveTo(0, c.height); g.lineTo(c.width, 0); g.stroke();
            hatch = ctx.createPattern(c, "repeat");
          }
          ctx.save();
          ctx.beginPath(); ctx.rect(bbox.left, bbox.top, bbox.width, bbox.height); ctx.clip();
          ctx.font = font.replace(/(\d+)px/, (_, n) => `${Number(n) * px}px`);
          ctx.textBaseline = "top";
          const edge = [-Infinity, -Infinity]; // per row (sunrise, sunset): right edge of the last label drawn, so labels never overprint
          list.forEach((b) => {
            const [a, z] = bandInterval(b);
            const x0 = u.valToPos(a, "x", true), x1 = u.valToPos(z, "x", true), xc = u.valToPos(b.t_unix_s, "x", true);
            if (x1 < bbox.left || x0 > bbox.left + bbox.width) return;
            ctx.fillStyle = hatch!; ctx.globalAlpha = 0.9;
            ctx.fillRect(Math.min(x0, xc - 3 * px), bbox.top, Math.max(x1 - x0, 6 * px), bbox.height); // at least 6 px wide, so a narrow band still reads
            ctx.globalAlpha = 1; ctx.strokeStyle = colour; ctx.lineWidth = px;
            ctx.beginPath(); ctx.moveTo(xc, bbox.top); ctx.lineTo(xc, bbox.top + bbox.height); ctx.stroke();
            const text = bandLabel(b), w = ctx.measureText(text).width;
            const row = b.kind === "sunrise" ? 0 : 1;
            const right = xc + 4 * px + w <= bbox.left + bbox.width;
            const left = right ? xc + 4 * px : xc - 4 * px - w;
            if (left < Math.max(edge[row], bbox.left)) return;
            const y = bbox.top + bbox.height * (0.45 + row * 0.25);
            ctx.fillStyle = chip; ctx.fillRect(left - 3 * px, y - 2 * px, w + 6 * px, 16 * px);
            ctx.fillStyle = colour; ctx.textAlign = "left";
            ctx.fillText(text, left, y);
            edge[row] = left + w + 8 * px;
          });
          ctx.restore();
        },
      },
    };
  }

  /** Marks the first lethal shadow: a vertical line on every chart, and on the charge chart a ✕ with its label. */
  function lethalPlugin(tS: number | null, colour: string, font: string, text: boolean): uPlot.Plugin {
    return {
      hooks: {
        draw: (u) => {
          if (tS === null) return;
          const { ctx, bbox } = u;
          const x = u.valToPos(tS, "x", true);
          if (x < bbox.left || x > bbox.left + bbox.width) return;
          ctx.save();
          ctx.strokeStyle = colour; ctx.fillStyle = colour; ctx.lineWidth = 1 * uPlot.pxRatio;
          ctx.beginPath(); ctx.moveTo(x, bbox.top); ctx.lineTo(x, bbox.top + bbox.height); ctx.stroke();
          if (text) {
            ctx.font = font.replace(/(\d+)px/, (_, n) => `${Number(n) * uPlot.pxRatio}px`);
            ctx.textBaseline = "top";
            const label = "✕ first lethal shadow";
            const w = ctx.measureText(label).width;
            const left = x + 6 * uPlot.pxRatio + w < bbox.left + bbox.width;
            ctx.textAlign = left ? "left" : "right";
            ctx.fillText(label, x + (left ? 6 : -6) * uPlot.pxRatio, bbox.top + 4 * uPlot.pxRatio);
          }
          ctx.restore();
        },
      },
    };
  }

  /** Ctrl/⌘+wheel zooms around the pointer; with shift it pans. A plain wheel scrolls the page. The scale change syncs to the other charts. */
  function wheelPlugin(): uPlot.Plugin {
    return {
      hooks: {
        ready: (u) => {
          u.over.addEventListener("wheel", (e) => {
            if (!e.ctrlKey && !e.metaKey) return;
            e.preventDefault();
            const { min, max } = u.scales.x as { min: number; max: number };
            const span = max - min;
            let lo: number, hi: number;
            if (e.shiftKey) {
              const d = (e.deltaY > 0 ? 1 : -1) * span * 0.1;
              lo = min + d; hi = max + d;
            } else {
              const at = u.posToVal(e.clientX - u.over.getBoundingClientRect().left, "x");
              const k = e.deltaY > 0 ? 1.25 : 0.8;
              lo = at - (at - min) * k; hi = at + (max - at) * k;
            }
            const w = Math.min(hi - lo, full[1] - full[0]);
            lo = Math.min(Math.max(lo, full[0]), full[1] - w);
            u.setScale("x", { min: lo, max: lo + w });
          }, { passive: false });
        },
      },
    };
  }

  function build() {
    const k = colours();
    const d = buildChartData(tl, scenario.preset);
    const t = d.illum[0];
    full = [t[0], t[t.length - 1] + scenario.step_s];
    const lethalS = lethalUnixS(scenario.start_unix_s, scenario.survival.lethal_h);
    const runs = clipRunsAt(operableRuns(tl.t_unix_s, operableMask(tl), scenario.step_s), lethalS);
    const band = (text = false) => [bandPlugin(runs, k.fog + "26"), lethalPlugin(lethalS, k.paper, k.font, text), wheelPlugin()];
    const x = (show: boolean): uPlot.Axis => ({ show, stroke: k.ash, grid: { stroke: k.iron, width: 1 }, ticks: { stroke: k.iron }, font: k.font });
    const y = (extra: Partial<uPlot.Axis> = {}): uPlot.Axis => ({ size: 56, stroke: k.ash, grid: { stroke: k.iron, width: 1 }, ticks: { stroke: k.iron }, font: k.font, ...extra });
    const base = (height: number, show: boolean) => ({
      width, height,
      tzDate: (ts: number) => uPlot.tzDate(new Date(ts * 1000), "Etc/UTC"),
      scales: { x: { time: true } },
      cursor: { sync: { key: SYNC, scales: ["x", null] as [string, null] }, y: false, points: { show: false }, drag: { x: true, y: false, setScale: true } },
      legend: { live: true },
      padding: [8, 12, 0, 0] as [number, number, number, number],
      axes: [x(show), y()],
    });
    const hooks = (): uPlot.Hooks.Arrays => ({
      setCursor: [onCursor],
      setScale: [(u, key) => { if (key === "x") { const { min, max } = u.scales.x; if (min != null && max != null) vis = [min, max]; placeCursors(); } }],
    });
    const illum = new uPlot({
      ...base(150, false), hooks: hooks(), plugins: [...band(), uncertaintyPlugin(() => bands, k.sun, getComputedStyle(host).getPropertyValue("--surface-carbon").trim() || k.iron, k.font)],
      scales: { x: { time: true }, y: { range: [0, 1.05] } },
      series: [
        {},
        { label: "Illumination (disc fraction)", stroke: k.sun, width: 2, value: (_u, v) => (v == null ? "--" : v.toFixed(2)) },
        { label: "Visibility (any of disc)", stroke: k.paper, width: 1, dash: [4, 3], value: (_u, v) => (v == null ? "--" : v.toFixed(0)) },
      ],
    }, d.illum, host.children[0] as HTMLElement);
    const margin = new uPlot({
      ...base(130, false), hooks: hooks(), plugins: band(),
      axes: [x(false), y({ values: (_u, v) => v.map((n) => `${n}°`) })],
      series: [
        {},
        { label: "Earth margin (deg above mask)", stroke: k.earthLine, width: 2, value: (_u, v) => (v == null ? "--" : `${v.toFixed(2)}°`) },
        { label: label, stroke: k.paper, width: 1, dash: [6, 4], value: (_u, v) => (v == null ? "--" : `${v}°`) },
      ],
    }, d.margin, host.children[1] as HTMLElement);
    const contact = new uPlot({
      ...base(100, false), hooks: hooks(), plugins: band(),
      scales: { x: { time: true }, y: { range: [-0.15, 1.15] } },
      axes: [x(false), y({ splits: () => [0, 1], values: () => ["no", "yes"] })],
      series: [
        {},
        { label: "Earth contact", stroke: k.earthLine, width: 2, paths: uPlot.paths.stepped!({ align: 1 }), value: (_u, v) => (v == null ? "--" : v >= 0.5 ? "yes" : "no") },
      ],
    }, d.contact, host.children[2] as HTMLElement);
    const cap = scenario.preset.capacity_wh;
    const soc = new uPlot({
      ...base(130, true), hooks: hooks(), plugins: band(true),
      scales: { x: { time: true }, y: { range: [0, cap * 1.05] } },
      axes: [x(true), y({ size: 56, values: (_u, v) => v.map((n) => `${n}`) })],
      series: [
        {},
        { label: "Battery charge (Wh)", stroke: k.paper, width: 2, value: (_u, v) => (v == null ? "--" : `${v.toFixed(0)} Wh`) },
        { label: `Lethal charge ${scenario.preset.min_wh} Wh (${citedTag(scenario.preset, "min_wh")})`, stroke: k.fog, width: 1, dash: [6, 4], value: (_u, v) => (v == null ? "--" : `${v} Wh`) },
      ],
    }, buildSocData(tl.t_unix_s, scenario.survival.soc_wh, scenario.preset), socHost);
    charts = [illum, margin, contact, soc];
    for (const u of charts) u.setScale("x", { min: full[0], max: full[1] });
    placeCursors();
  }

  /** Move every chart's cursor to the shared time index, or hide it when that time is zoomed out of view. */
  function placeCursors() {
    placing = true;
    const t = tl.t_unix_s[idx];
    for (const u of charts) {
      const left = u.valToPos(t, "x");
      u.setCursor({ left: left >= 0 && left <= u.bbox.width / uPlot.pxRatio ? left : -10, top: -10 }, false);
    }
    placing = false;
  }

  /** − / + / Reset: every chart gets the same x range, so the synced charts and the time cursor stay in step. */
  function zoom(factor: number | null) {
    const [a, b] = full, [lo, hi] = vis;
    const w = factor === null ? b - a : Math.min(Math.max((hi - lo) * factor, 4 * scenario.step_s), b - a);
    const min = factor === null ? a : Math.min(Math.max((lo + hi) / 2 - w / 2, a), b - w);
    for (const u of charts) u.setScale("x", { min, max: min + w });
  }

  function onCursor(u: uPlot) {
    if (placing) return;
    const left = u.cursor.left ?? -1;
    if (left < 0) { placeCursors(); return; } // pointer left the chart: return to the shared time
    const t = u.posToVal(left, "x");
    const i = indexAtTime(tl.t_unix_s, t);
    if (i !== idx) idx = i;
  }

  $effect(() => {
    void scenario;
    untrack(build);
    return () => { for (const u of charts) u.destroy(); charts = []; };
  });
  $effect(() => {
    const s = scenario;
    let live = true;
    bands = []; // never draw the previous scenario's bands on the new timeline
    // the plugin reads bands at draw time, so a redraw shows them without rebuilding (and keeps the user's zoom)
    const show = (b: Band[], err: string) => { bands = b; bandError = err; for (const u of charts) u.redraw(false, false); };
    windowBands(s, SIGMA_Z_M).then((f) => { if (live) show(parseBands(f), ""); },
      (e) => { if (live) show([], String(e?.message ?? e)); });
    return () => { live = false; };
  });
  $effect(() => { const w = width; for (const u of charts) u.setSize({ width: w, height: u.height }); });
  $effect(() => { void idx; untrack(placeCursors); });
</script>

<section class="panel" aria-labelledby="tl-h">
  <header>
    <h2 id="tl-h">Timelines · sunlight and Earth over the span</h2>
    <div class="zoom" role="group" aria-label="Zoom the timelines" data-testid="zoom" data-span={vis[1] - vis[0]}>
      <button type="button" onclick={() => zoom(2)} aria-label="Zoom out" data-testid="zoom-out">−</button>
      <button type="button" onclick={() => zoom(0.5)} aria-label="Zoom in" data-testid="zoom-in">+</button>
      <button type="button" onclick={() => zoom(null)} data-testid="zoom-reset">Reset zoom</button>
    </div>
  </header>

  <div class="charts" bind:this={host} bind:clientWidth={width} data-testid="timelines" role="group"
    aria-label="Timelines chart. Illumination, visibility, Earth margin, Earth contact; the shaded band is the operable window; hatched intervals with a label are the plus or minus uncertainty of each sunrise and sunset. Move the time slider to move the cursor. The zoom buttons, or Ctrl or Command with the wheel, zoom; add shift to pan; drag to zoom to a range; double-click resets.">
    <div></div><div></div><div></div>
  </div>
  <div class="charts" bind:this={socHost} data-testid="soc-chart" role="group"
    aria-label="Battery state of charge in watt-hours over the span. A vertical line and a cross mark the first lethal shadow, when charge reaches the lethal charge."></div>

  <p class="now" data-testid="timelines-now">
    {fmtUtc(tl.t_unix_s[idx])} · illumination {tl.illumination[idx].toFixed(2)} · visibility {tl.visibility[idx].toFixed(0)} · Earth margin {tl.earth_margin_deg[idx].toFixed(2)}° (threshold {label}) · contact {tl.contact[idx] >= 0.5 ? "yes" : "no"}
  </p>
  <p class="key"><span class="swatch"></span>Shaded band: operable window (illumination above zero and Earth contact).</p>
  <p class="key" data-testid="band-key"><span class="swatch hatch"></span>Hatched: ± uncertainty from DEM height error, {SIGMA_Z_TEXT}. "ill-conditioned" = wider than 6 h [est].</p>
  {#if bandError}<p class="key" role="alert">Sunrise and sunset bands unavailable: {bandError}</p>{/if}
  <details class="bands" data-testid="band-list">
    <summary>Sunrise and sunset times with ± uncertainty ({bands.length})</summary>
    <ul>{#each bands as b (b.t_unix_s + b.kind)}<li>{bandLabel(b, true)}</li>{/each}</ul>
  </details>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .zoom { display: flex; }
  .zoom button {
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); background: transparent;
    border: 1px solid var(--color-slate); padding: var(--spacing-8) var(--spacing-16); margin-left: -1px; cursor: pointer; min-width: 44px;
  }
  .zoom button:first-child { margin-left: 0; border-radius: var(--radius-sm) 0 0 var(--radius-sm); }
  .zoom button:last-child { border-radius: 0 var(--radius-sm) var(--radius-sm) 0; }
  .zoom button:hover { color: var(--color-paper); background: var(--surface-iron); }
  .zoom button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
  .charts { width: 100%; }
  .now, .key { margin: var(--spacing-8) 0 0; font: var(--text-caption)/var(--leading-caption) var(--font-mono); font-variant-numeric: tabular-nums; }
  .now { color: var(--color-fog); }
  .key { color: var(--color-ash); display: flex; align-items: center; gap: var(--spacing-8); }
  .swatch { width: 16px; height: 8px; background: var(--color-fog); opacity: 0.15; border: 1px solid var(--color-slate); }
  .swatch.hatch { opacity: 1; background: repeating-linear-gradient(135deg, var(--color-sun) 0 1px, transparent 1px 4px); }
  .bands { margin-top: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .bands ul { margin: var(--spacing-8) 0 0; padding-left: var(--spacing-16); max-height: 12em; overflow: auto; font-variant-numeric: tabular-nums; }
  .charts :global(.u-legend) { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); text-align: left; }
  .charts :global(.u-legend .u-marker) { border-radius: 0; }
  .charts :global(.u-cursor-x) { border-right: 1px dashed var(--color-fog) !important; }
  .charts :global(.u-select) { background: var(--color-fog); opacity: 0.12; }
</style>
