// Pure helpers for the Timelines chart. Everything plotted is a column of the API's `timeline` result (docs/API.md);
// nothing here computes physics. The operable window is the glossary's AND of two model columns (SCIENCE-SPEC §2.5).
import type { Preset, Timeline } from "./engine";

export type Run = [startS: number, endS: number];

/** "cited" or "assumed" for a preset field (U10): only a field the preset marks cited is cited. */
export function citedTag(preset: Preset, key: string): "cited" | "assumed" {
  return preset.cited[key] === true ? "cited" : "assumed";
}

/** Label for the Earth margin threshold: "2° (assumed)" unless the preset cites it. */
export function thresholdLabel(preset: Preset): string {
  return `${preset.earth_margin_deg}°${citedTag(preset, "earth_margin_deg") === "cited" ? "" : " (assumed)"}`;
}

/** Operable steps: illumination above zero and Earth contact (SCIENCE-SPEC §2.5). */
export function operableMask(tl: Pick<Timeline, "illumination" | "contact">): Uint8Array {
  const n = tl.illumination.length;
  const m = new Uint8Array(n);
  for (let i = 0; i < n; i++) m[i] = tl.illumination[i] > 0 && tl.contact[i] >= 0.5 ? 1 : 0;
  return m;
}

/** Contiguous operable runs as [start, end) in Unix seconds; a step covers `stepS`. */
export function operableRuns(t: ArrayLike<number>, mask: ArrayLike<number>, stepS: number): Run[] {
  const runs: Run[] = [];
  let start = -1;
  for (let i = 0; i <= mask.length; i++) {
    const on = i < mask.length && mask[i] === 1;
    if (on && start < 0) start = i;
    if (!on && start >= 0) {
      runs.push([t[start], t[i - 1] + stepS]);
      start = -1;
    }
  }
  return runs;
}

/** Index of the step nearest to a time, clamped to the span. */
export function indexAtTime(t: ArrayLike<number>, tUnix: number): number {
  let lo = 0, hi = t.length - 1;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (t[mid] < tUnix) lo = mid + 1; else hi = mid;
  }
  if (lo > 0 && Math.abs(t[lo - 1] - tUnix) <= Math.abs(t[lo] - tUnix)) lo--;
  return lo;
}

export type ChartData = {
  illum: [number[], number[], number[]];
  margin: [number[], number[], number[]];
  contact: [number[], number[]];
};
const arr = (a: Float64Array) => Array.from(a);

/** State of charge (Wh) from the model's `soc_wh`, with the preset's lethal charge as a flat reference line.
 *  `soc_wh` stops at a lethal shadow, so it is padded with null (a uPlot gap) to the time axis. */
export function buildSocData(t: Float64Array, socWh: Float64Array, preset: Preset): [number[], (number | null)[], number[]] {
  return [arr(t), Array.from(t, (_, i) => (i < socWh.length ? socWh[i] : null)), Array.from(t, () => preset.min_wh)];
}

/** Unix time of the first lethal shadow, or null when none falls in the span. */
export function lethalUnixS(startUnixS: number, lethalH: number | null): number | null {
  return lethalH === null ? null : startUnixS + lethalH * 3600;
}

/** uPlot data arrays, one aligned set per chart. x is Unix seconds. */
export function buildChartData(tl: Timeline, preset: Preset): ChartData {
  const t = arr(tl.t_unix_s);
  return {
    illum: [t, arr(tl.illumination), arr(tl.visibility)],
    margin: [t, arr(tl.earth_margin_deg), t.map(() => preset.earth_margin_deg)],
    contact: [t, arr(tl.contact)],
  };
}
