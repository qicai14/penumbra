// Pure checks for the site picker and date controls. No model maths: only the refusal rules P5 and P16.
import type { NearMeta } from "./engine";

// P5: a site must be at least 7 km inside the near tile. The horizon needs 7 km of ground, which is up to 7.05 km of map
// (polar stereographic length scale), else the Engine answers OUT_OF_COVERAGE.
export const PICK_BORDER_M = 7050;
export const EPH_START_S = Date.UTC(2026, 0, 1) / 1000;
export const EPH_END_S = Date.UTC(2031, 0, 1) / 1000; // last table row: 2026-01-01 + 1826 days
export const SPANS_DAYS = [14, 30, 60] as const;

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
  if (d < PICK_BORDER_M)
    return { ok: false, reason: `Too close to the tile edge: ${(Math.floor(d / 10) / 100).toFixed(2)} km. A site needs at least 7 km of terrain around it, which is a ${PICK_BORDER_M / 1000} km map border (P5).` };
  return { ok: true };
}

/** "YYYY-MM-DD" to Unix seconds at 00:00 UTC; null if malformed or not a real day. */
export function parseDate(s: string): number | null {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(s)) return null;
  const ms = Date.parse(`${s}T00:00:00Z`);
  return Number.isNaN(ms) || new Date(ms).toISOString().slice(0, 10) !== s ? null : ms / 1000;
}

export function checkDate(s: string, spanDays: number): Verdict {
  const t = parseDate(s);
  if (t === null) return { ok: false, reason: "Enter a real date as YYYY-MM-DD." };
  if (t < EPH_START_S) return { ok: false, reason: `${s} is before the ephemeris table starts (2026-01-01). The model does not extrapolate (P16).` };
  if (t >= EPH_END_S) return { ok: false, reason: `${s} is after the ephemeris table ends (2030-12-31). The model does not extrapolate (P16).` };
  if (t + spanDays * 86400 > EPH_END_S)
    return { ok: false, reason: `${s} plus ${spanDays} days runs past the end of the ephemeris table (2030-12-31). Pick an earlier date or a shorter span (P16).` };
  return { ok: true };
}
