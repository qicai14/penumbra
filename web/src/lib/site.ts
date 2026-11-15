// Pure checks for the site picker and date controls. No model maths: only the refusal rules P5 and P16.
import type { EphSegment, NearMeta } from "./engine";

const NEAR_RANGE_M = 7000, MOON_RADIUS_M = 1_737_400;
/**
 * P5: a site must have 7 km of ground inside the near tile, else the Engine answers OUT_OF_COVERAGE. On the polar
 * stereographic map that is 7000 x k metres, k = 1 + r^2 / 4R^2 at the tile corner farthest from the pole, plus half a
 * pixel (the outermost pixel centre sits that far inside the edge). Same rule as pick_border_m in crates/core/src/horizon.rs.
 */
export function pickBorderM(t: NearMeta): number {
  const x1 = t.x0_m + t.width * t.pixel_m, y1 = t.y0_m - t.height * t.pixel_m;
  const x = Math.max(Math.abs(t.x0_m), Math.abs(x1)), y = Math.max(Math.abs(t.y0_m), Math.abs(y1));
  return NEAR_RANGE_M * (1 + (x * x + y * y) / (4 * MOON_RADIUS_M * MOON_RADIUS_M)) + t.pixel_m / 2;
}
export const SPANS_DAYS = [14, 30, 60] as const;
export const DEFAULT_SPAN_DAYS = 30;
export const DEFAULT_MAST_M = 2;
export const MAST_MAX_M = 10; // the site view's mast slider runs 0..10 m in 0.5 m steps

export type Verdict = { ok: true } | { ok: false; reason: string };

/** Metres from (x, y) to the nearest tile edge; negative outside the tile. */
export function edgeDistanceM(x: number, y: number, t: NearMeta): number {
  const x1 = t.x0_m + t.width * t.pixel_m, y1 = t.y0_m - t.height * t.pixel_m;
  return Math.min(x - t.x0_m, x1 - x, y - y1, t.y0_m - y);
}

export function checkPick(x: number, y: number, t: NearMeta): Verdict {
  const d = edgeDistanceM(x, y, t);
  // !(d >= 0), not d < 0: also catches NaN from a pole or non-finite projection
  if (!(d >= 0)) return { ok: false, reason: "That point is outside the near tile. Pick inside the image." };
  const border = pickBorderM(t);
  if (d < border)
    return { ok: false, reason: `Too close to the tile edge: ${(Math.floor(d / 10) / 100).toFixed(2)} km. A site needs at least 7 km of terrain around it, which is a ${(Math.ceil(border / 10) / 100).toFixed(2)} km map border on this tile (P5).` };
  return { ok: true };
}

/** "YYYY-MM-DD" to Unix seconds at 00:00 UTC; null if malformed or not a real day. */
export function parseDate(s: string): number | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(s)) return null;
  const ms = Date.parse(`${s}T00:00:00Z`);
  return Number.isNaN(ms) || new Date(ms).toISOString().slice(0, 10) !== s ? null : ms / 1000;
}

const isoDay = (s: number) => new Date(s * 1000).toISOString().slice(0, 10);
/** One segment covers [t0, t0 + (rows - 1) * step]. Segments are never summed: a gap between two is not covered. */
const segEnd = (g: EphSegment) => g.t0_unix_s + (g.rows - 1) * g.step_s;

/** P16: the date and its whole span must fit inside ONE ephemeris segment (the model does not extrapolate or bridge gaps). */
export function checkDate(s: string, spanDays: number, eph: readonly EphSegment[]): Verdict {
  const t = parseDate(s);
  if (t === null) return { ok: false, reason: "Enter a real date as YYYY-MM-DD." };
  if (!eph.length) return { ok: false, reason: "No ephemeris table is loaded. The model does not extrapolate (P16)." };
  const end = t + spanDays * 86400;
  if (eph.some((g) => t >= g.t0_unix_s && end <= segEnd(g))) return { ok: true };
  const first = Math.min(...eph.map((g) => g.t0_unix_s)), last = Math.max(...eph.map(segEnd));
  // The last day named is the last whole day before the end row, as the old messages did (2031-01-01 00:00 -> 2030-12-31).
  const lastDay = isoDay(last - 86400);
  if (t < first) return { ok: false, reason: `${s} is before the ephemeris table starts (${isoDay(first)}). The model does not extrapolate (P16).` };
  if (t >= last) return { ok: false, reason: `${s} is after the ephemeris table ends (${lastDay}). The model does not extrapolate (P16).` };
  const inside = eph.find((g) => t >= g.t0_unix_s && t < segEnd(g));
  if (!inside) {
    const prev = eph.filter((g) => segEnd(g) <= t).reduce((a, g) => Math.max(a, segEnd(g)), -Infinity);
    const next = Math.min(...eph.filter((g) => g.t0_unix_s > t).map((g) => g.t0_unix_s));
    return { ok: false, reason: `${s} falls in a gap between ephemeris tables (${isoDay(prev - 86400)} to ${isoDay(next)}). The model does not bridge gaps (P16).` };
  }
  return { ok: false, reason: `${s} plus ${spanDays} days runs past the end of the ephemeris table (${isoDay(segEnd(inside) - 86400)}). Pick an earlier date or a shorter span (P16).` };
}

/** Earliest first day and latest last day over all segments, for the date input min/max (the span check still refuses a date too near the end or in a gap). */
export function dateRange(eph: readonly EphSegment[]): { min: string; max: string } {
  if (!eph.length) return { min: "", max: "" };
  return { min: isoDay(Math.min(...eph.map((g) => g.t0_unix_s))), max: isoDay(Math.max(...eph.map(segEnd)) - 86400) };
}

/**
 * Which region's near tile answers a site. Overlapping tiles carry different int16 scales, so the choice must not depend on
 * session history: the region the UI selected (if it passes its pick border), else the first in manifest order that does,
 * else the first whose tile merely holds the site (shared links a few metres inside the border still load).
 */
export function pickRegion<R extends { id: string; near: NearMeta }>(regions: R[], x: number, y: number, preferId?: string): R | undefined {
  const ok = (r: R) => edgeDistanceM(x, y, r.near) >= pickBorderM(r.near);
  const sel = regions.find((r) => r.id === preferId);
  return (sel && ok(sel) ? sel : undefined) ?? regions.find(ok) ?? regions.find((r) => edgeDistanceM(x, y, r.near) >= 0);
}

/** The slope line for the pin; no line at all (never "0°") when the engine had no slope there. */
export const slopeText = (deg: number | undefined): string | null =>
  deg === undefined || !Number.isFinite(deg) ? null : `Slope ${deg.toFixed(1)}° (20 m DEM, 60 m window)`;
