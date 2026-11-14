<script lang="ts">
  import { untrack } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import type { Scenario } from "./engine";
  import { fmtUtc } from "./panorama";
  import { buildChartData, buildSocData, citedTag, indexAtTime, lethalUnixS, operableMask, operableRuns, thresholdLabel } from "./timelines";

  let { scenario, idx = $bindable(0) }: { scenario: Scenario; idx: number } = $props();

  const SYNC = "penumbra-timelines";
  let width = $state(800);
  let host: HTMLDivElement;
  let socHost: HTMLDivElement;
  let charts: uPlot[] = [];
  let placing = false; // true while we move the cursors ourselves, so the cursor hook does not echo back

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
  function wheelPlugin(full: [number, number]): uPlot.Plugin {
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
    const full: [number, number] = [t[0], t[t.length - 1] + scenario.step_s];
    const runs = operableRuns(tl.t_unix_s, operableMask(tl), scenario.step_s);
    const lethalS = lethalUnixS(scenario.start_unix_s, scenario.survival.lethal_h);
    const band = (text = false) => [bandPlugin(runs, k.fog + "26"), lethalPlugin(lethalS, k.paper, k.font, text), wheelPlugin(full)];
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
      setScale: [(u, key) => { if (key === "x") placeCursors(); }],
    });
    const illum = new uPlot({
      ...base(150, false), hooks: hooks(), plugins: band(),
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
  $effect(() => { const w = width; for (const u of charts) u.setSize({ width: w, height: u.height }); });
  $effect(() => { void idx; untrack(placeCursors); });
</script>

<section class="panel" aria-labelledby="tl-h">
  <header>
    <h2 id="tl-h">Timelines · sunlight and Earth over the span</h2>
    {#if scenario.fake}<span class="pill" data-testid="fake-badge-timelines">FAKE DATA</span>{/if}
  </header>

  <div class="charts" bind:this={host} bind:clientWidth={width} data-testid="timelines" role="group"
    aria-label="Timelines chart. Illumination, visibility, Earth margin, Earth contact; the shaded band is the operable window. Move the time slider to move the cursor. Ctrl or Command with the wheel zooms, add shift to pan, drag to zoom to a range, double-click resets.">
    <div></div><div></div><div></div>
  </div>
  <div class="charts" bind:this={socHost} data-testid="soc-chart" role="group"
    aria-label="Battery state of charge in watt-hours over the span. A vertical line and a cross mark the first lethal shadow, when charge reaches the lethal charge."></div>

  <p class="now" data-testid="timelines-now">
    {fmtUtc(tl.t_unix_s[idx])} · illumination {tl.illumination[idx].toFixed(2)} · visibility {tl.visibility[idx].toFixed(0)} · Earth margin {tl.earth_margin_deg[idx].toFixed(2)}° (threshold {label}) · contact {tl.contact[idx] >= 0.5 ? "yes" : "no"}
  </p>
  <p class="key"><span class="swatch"></span>Shaded band: operable window (illumination above zero and Earth contact).</p>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  header { display: flex; justify-content: space-between; align-items: center; gap: var(--spacing-16); margin-bottom: var(--spacing-16); }
  h2 { margin: 0; font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .pill {
    border: 1px solid var(--color-slate); border-radius: var(--radius-full); padding: 0 var(--spacing-8);
    font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); text-transform: uppercase; white-space: nowrap;
  }
  .charts { width: 100%; }
  .now, .key { margin: var(--spacing-8) 0 0; font: var(--text-caption)/var(--leading-caption) var(--font-mono); font-variant-numeric: tabular-nums; }
  .now { color: var(--color-fog); }
  .key { color: var(--color-ash); display: flex; align-items: center; gap: var(--spacing-8); }
  .swatch { width: 16px; height: 8px; background: var(--color-fog); opacity: 0.15; border: 1px solid var(--color-slate); }
  .charts :global(.u-legend) { font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); text-align: left; }
  .charts :global(.u-legend .u-marker) { border-radius: 0; }
  .charts :global(.u-cursor-x) { border-right: 1px dashed var(--color-fog) !important; }
  .charts :global(.u-select) { background: var(--color-fog); opacity: 0.12; }
</style>
