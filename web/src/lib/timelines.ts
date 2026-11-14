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

/** Operable runs cut off at the lethal time (SCIENCE-SPEC 1e: operable hours stop there): a run spanning it ends exactly there, later runs are dropped. No lethal time leaves the runs as they are. */
export function clipRunsAt(runs: Run[], lethalS: number | null): Run[] {
  if (lethalS === null) return runs;
  return runs.filter(([a]) => a < lethalS).map(([a, b]): Run => [a, Math.min(b, lethalS)]);
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

/** Under a day reads in hours ("7.1 h"), so a lander lost in 30 minutes never reads "0 days";
 *  from a day up, whole days drop the decimal point: "10 days", "12.6 days", "1 day". */
export function fmtDays(hours: number): string {
  if (hours < 23.95) return `${Math.round(hours * 10) / 10} h`;
  const d = Math.round((hours / 24) * 10) / 10;
  return `${Number.isInteger(d) ? d : d.toFixed(1)} ${d === 1 ? "day" : "days"}`;
}

/** A countdown as "Xd Yh"; under an hour, "Ym". */
export function fmtDH(s: number): string {
  if (s < 3600) return `${Math.floor(s / 60)}m`;
  return `${Math.floor(s / 86400)}d ${Math.floor((s % 86400) / 3600)}h`;
}

const WORDS = {
  sunset: { set: "Sunset", none: "sunset", rise: "sunrise" },
  Earthset: { set: "Earthset", none: "Earthset", rise: "Earthrise" },
} as const;

/**
 * Time from step `idx` to the next set: the first later step where `up` turns false after being true (T26, §1d).
 * Already down at `idx` counts to the set after the next rise. With no set left in the span it says so, never blank;
 * down for the rest of the span says "no sunrise", so a site in shadow never reads like a sunlit one.
 */
export function countdown(t: ArrayLike<number>, col: ArrayLike<number>, up: (x: number) => boolean, idx: number, what: keyof typeof WORDS): string {
  const w = WORDS[what];
  let rises = up(col[idx]);
  for (let j = idx + 1; j < col.length; j++) {
    if (up(col[j - 1]) && !up(col[j])) return `${w.set} in ${fmtDH(t[j] - t[idx])}`;
    rises ||= up(col[j]);
  }
  const left = t[t.length - 1] - t[idx];
  const word = rises ? w.none : w.rise;
  return left < 3600 ? `no ${word} before the span ends` : `no ${word} in ${fmtDays(left / 3600)}`;
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

// ---- Sunrise and sunset ± bands (T18, SCIENCE-SPEC 1f step 1). Values come from the WASM `window_bands`; this only formats them. ----

/** σ_z, the DEM height error at observer and obstacle, metres: assumed, not cited. Used for the call and for the text. */
export const SIGMA_Z_M = 0.5;
export const SIGMA_Z_TEXT = `σ_z = ${SIGMA_Z_M} m (assumed)`;
/** A band wider than this, hours, is written "ill-conditioned", not as minutes. [est] */
export const ILL_CONDITIONED_BAND_H = 6;
/** core::band::MAX_HALF_WIDTH_MIN: a half width equal to this is core's "unbounded" marker, one 29.5-day synodic month. */
export const UNBOUNDED_HALF_WIDTH_MIN = 29.5 * 24 * 60;

export type Band = { t_unix_s: number; kind: "sunrise" | "sunset"; half_width_min: number };

/** The flat 3 × k column-major result of `window_bands` (t_unix_s, kind +1/−1, half_width_min) as rows. */
export function parseBands(flat: ArrayLike<number>): Band[] {
  const k = Math.floor(flat.length / 3);
  return Array.from({ length: k }, (_, i) => ({
    t_unix_s: flat[i], kind: flat[k + i] > 0 ? "sunrise" : "sunset", half_width_min: flat[2 * k + i],
  }));
}

/** True when the band is wider than the threshold (full width, 2 × half), or core marks it unbounded or non-finite. */
export function isIllConditioned(half_width_min: number): boolean {
  return !Number.isFinite(half_width_min) || half_width_min >= UNBOUNDED_HALF_WIDTH_MIN || 2 * half_width_min > ILL_CONDITIONED_BAND_H * 60;
}

/** The shaded interval [start, end) in Unix s; an unbounded or non-finite band is bounded at one synodic month each side so it can be drawn. */
export function bandInterval(b: Band): Run {
  const half = (Number.isFinite(b.half_width_min) ? Math.min(b.half_width_min, UNBOUNDED_HALF_WIDTH_MIN) : UNBOUNDED_HALF_WIDTH_MIN) * 60;
  return [b.t_unix_s - half, b.t_unix_s + half];
}

/** "sunset 14:20 ± 40 min" (UTC), or "sunset 14:20 ill-conditioned". `withDate` prefixes "2026-11-14 ". */
export function bandLabel(b: Band, withDate = false): string {
  const iso = new Date(b.t_unix_s * 1000).toISOString();
  const when = `${withDate ? iso.slice(0, 10) + " " : ""}${iso.slice(11, 16)}`;
  if (isIllConditioned(b.half_width_min)) return `${b.kind} ${when} ill-conditioned`;
  const m = Math.round(b.half_width_min);
  return `${b.kind} ${when} ± ${m < 1 ? "<1" : m} min`;
}
