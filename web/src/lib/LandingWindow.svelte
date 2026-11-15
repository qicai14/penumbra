<script lang="ts">
  import { untrack } from "svelte";
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { computeScenario, type EphSegment, type Inputs } from "./engine";
  import { parseDate } from "./site";
  import { bestPoint, sweep, sweepDates, type SweepPoint } from "./sweep";
  import { fmtDays } from "./timelines";

  let { computedFor, eph, date, onpick }: { computedFor: Inputs; eph: EphSegment[]; date: string; onpick: (date: string) => void } = $props();

  // The window stays put while the planner clicks inside it; it moves only when the picked date leaves it.
  let anchor = untrack(() => date);
  const dates = $derived.by(() => {
    let w = sweepDates(anchor, computedFor.span_days, eph);
    if (!w.includes(date)) { anchor = date; w = sweepDates(date, computedFor.span_days, eph); }
    return w;
  });
  // Everything that changes a bar, as a string, so a click inside the window does not rerun the sweep.
  const key = $derived(JSON.stringify([computedFor.lat_deg, computedFor.lon_deg, computedFor.mast_m, computedFor.span_days, computedFor.region, computedFor.preset, dates]));

  let pts = $state.raw<SweepPoint[]>([]);
  let done = $state(0);
  let error = $state<string | null>(null);
  const best = $derived(bestPoint(pts));
  const running = $derived(done < dates.length && !error);

  $effect(() => {
    void key;
    const inp = untrack(() => computedFor), list = untrack(() => dates);
    let live = true;
    pts = []; done = 0; error = null;
    const t = performance.now();
    // Same engine call as the headline (computeScenario), one date at a time, yielding between dates.
    sweep(list, async (d) => (await computeScenario({ ...inp, start_unix_s: parseDate(d)! })).survival,
      (n) => { if (live) done = n; }, () => !live)
      .then((r) => { if (live) { pts = r; (window as any).__landingWindowMs = Math.round(performance.now() - t); } },
        (e) => { if (live) error = String(e?.message ?? e); });
    return () => { live = false; };
  });

  let host: HTMLDivElement;
  let width = $state(800);
  let plot: uPlot | undefined;

  $effect(() => {
    const list = pts, w = untrack(() => width);
    if (!list.length) return;
    const s = getComputedStyle(host), c = (n: string) => s.getPropertyValue(n).trim();
    const font = `${c("--text-caption")} ${c("--font-mono")}`;
    const days = (p: SweepPoint) => p.operable_h / 24;
    const data: uPlot.AlignedData = [list.map((_, i) => i), list.map((p) => (p.lethal_h === null ? days(p) : null)), list.map((p) => (p.lethal_h !== null ? days(p) : null))];
    const bars = uPlot.paths.bars!({ size: [0.8, 48] });
    const u = new uPlot({
      width: w, height: 180, padding: [8, 12, 0, 0], legend: { show: false },
      cursor: { x: false, y: false, points: { show: false } },
      scales: { x: { time: false, range: [-0.6, list.length - 0.4] }, y: { range: (_u, _lo, hi) => [0, Math.max(hi, 1) * 1.1] } },
      axes: [
        { stroke: c("--color-ash"), grid: { show: false }, ticks: { stroke: c("--color-iron") }, font, space: 60,
          values: (_u, v) => v.map((i) => (Number.isInteger(i) && list[i] ? list[i].date.slice(5) : "")) },
        { size: 44, stroke: c("--color-ash"), grid: { stroke: c("--color-iron"), width: 1 }, ticks: { stroke: c("--color-iron") }, font, values: (_u, v) => v.map((n) => `${n}`) },
      ],
      series: [
        {},
        { label: "no lethal shadow", fill: c("--color-sun"), stroke: c("--color-sun"), paths: bars, points: { show: false } },
        { label: "lethal shadow", fill: c("--color-fog"), stroke: c("--color-fog"), paths: bars, points: { show: false } },
      ],
      hooks: { ready: [(u) => u.over.addEventListener("click", () => { const left = u.cursor.left ?? -1; if (left < 0) return; const p = list[Math.round(u.posToVal(left, "x"))]; if (p) onpick(p.date); })] },
    }, data, host);
    u.root.querySelectorAll("canvas").forEach((cv) => cv.setAttribute("aria-hidden", "true"));
    u.over.style.cursor = "pointer";
    plot = u;
    return () => { u.destroy(); plot = undefined; };
  });
  $effect(() => { const w = width; plot?.setSize({ width: w, height: 180 }); });
</script>

<section class="panel" aria-labelledby="lw-h">
  <h2 id="lw-h">Landing window · operable days by landing date</h2>
  {#if error}<p class="note" role="alert">Landing window unavailable: {error}</p>{/if}
  {#if running}<p class="note" data-testid="lw-progress" role="status"><progress max={dates.length} value={done}></progress> {done} of {dates.length} dates</p>{/if}
  {#if best}
    <p class="best" data-testid="lw-best">Best landing date: <strong>{best.date}</strong> · {fmtDays(best.operable_h)} operable{best.lethal_h === null ? ", no lethal shadow in the span" : ""}</p>
  {/if}
  <div class="chart" bind:this={host} bind:clientWidth={width} data-testid="lw-chart" role="group"
    aria-label="Operable days for each of the next {dates.length} landing dates, one bar per date. Orange bars: no lethal shadow in the span; grey bars: a lethal shadow arrives. Click a bar to set the landing date. The table below lists the same values."></div>
  <p class="key"><span class="sw sun"></span>no lethal shadow in span <span class="sw fog"></span>lethal shadow arrives · bar height: operable days</p>
  <details class="tbl">
    <summary>Table of landing dates ({pts.length})</summary>
    <ul data-testid="lw-table">
      {#each pts as p (p.date)}
        <li>
          <button type="button" data-testid="lw-date" data-date={p.date} data-operable={fmtDays(p.operable_h)} aria-current={p.date === date ? "date" : undefined} onclick={() => onpick(p.date)}>
            {p.date} · {fmtDays(p.operable_h)} operable · {p.lethal_h === null ? "no lethal shadow" : `lethal shadow in ${fmtDays(p.lethal_h)}`}{p === best ? " · best" : ""}
          </button>
        </li>
      {/each}
    </ul>
  </details>
</section>

<style>
  .panel { background: var(--surface-carbon); border-radius: var(--radius-sm); padding: var(--spacing-16); margin-top: var(--spacing-24); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-regular) var(--text-body)/var(--leading-body) var(--font-mono); color: var(--color-fog); }
  .note, .best, .key { margin: 0 0 var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-fog); font-variant-numeric: tabular-nums; }
  .best strong { color: var(--color-paper); font-weight: var(--font-weight-regular); }
  .key { color: var(--color-ash); margin: var(--spacing-8) 0 0; }
  .sw { display: inline-block; width: 10px; height: 10px; margin-right: var(--spacing-8); }
  .sw.sun { background: var(--color-sun); }
  .sw.fog { background: var(--color-fog); margin-left: var(--spacing-16); }
  progress { vertical-align: middle; accent-color: var(--color-sun); }
  .chart { width: 100%; min-height: 180px; }
  .tbl { margin-top: var(--spacing-8); font: var(--text-caption)/var(--leading-caption) var(--font-mono); color: var(--color-ash); }
  .tbl ul { list-style: none; margin: var(--spacing-8) 0 0; padding: 0; max-height: 14em; overflow: auto; }
  .tbl button { min-height: 32px; width: 100%; text-align: left; font: inherit; color: var(--color-fog); background: transparent; border: 0; padding: 2px var(--spacing-8); cursor: pointer; }
  .tbl button[aria-current="date"] { color: var(--color-paper); background: var(--surface-iron); }
  .tbl button:hover { color: var(--color-paper); }
  .tbl button:focus-visible { outline: 1px solid var(--color-paper); outline-offset: 2px; }
</style>
