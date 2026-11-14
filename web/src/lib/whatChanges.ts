// "What changes what" (T83 #83): signed change in operable days against the current pick. Pure; the day figures come from
// model output (the scenario's survival and the shipped grid), never from anything computed here.
import { NODATA, type Grid } from "./grid";

export const PIN_RADIUS_M = 300;
export const DATE_SHIFT_D = 2;

/** Signed change in days between two operable-hour figures. */
export const deltaDays = (hours: number, baseHours: number) => (hours - baseHours) / 24;

/** "+1.3 d", "−0.5 d", "0.0 d" (a true minus sign, so it reads at a glance). */
export function fmtDelta(d: number): string {
  const r = Math.round(Math.abs(d) * 10) / 10;
  return `${r === 0 ? "" : d > 0 ? "+" : "−"}${r.toFixed(1)} d`;
}

export type Near = { hereH: number; bestH: number; worstH: number; partial: boolean };

/** Operable hours of the site's cell, and the best and worst cell whose centre lies within `radiusM` of the point (the site's
 *  own cell counts, so best >= here >= worst). null when the site is outside the grid or on a no-data cell.
 *  `partial` marks a disc that runs off the grid. */
export function withinRadius(g: Grid, k: number, x: number, y: number, radiusM = PIN_RADIUS_M): Near | null {
  const m = g.meta, a = g.operable[k];
  const i0 = Math.floor((x - m.x0_m) / m.pixel_m), j0 = Math.floor((m.y0_m - y) / m.pixel_m);
  if (i0 < 0 || j0 < 0 || i0 >= m.width || j0 >= m.height || a[j0 * m.width + i0] === NODATA) return null;
  const hereH = a[j0 * m.width + i0];
  const r = Math.ceil(radiusM / m.pixel_m) + 1;
  let bestH = hereH, worstH = hereH;
  for (let j = Math.max(0, j0 - r); j <= Math.min(m.height - 1, j0 + r); j++)
    for (let i = Math.max(0, i0 - r); i <= Math.min(m.width - 1, i0 + r); i++) {
      const h = a[j * m.width + i];
      if (h === NODATA) continue;
      if (Math.hypot(m.x0_m + (i + 0.5) * m.pixel_m - x, m.y0_m - (j + 0.5) * m.pixel_m - y) > radiusM) continue;
      if (h > bestH) bestH = h;
      if (h < worstH) worstH = h;
    }
  const partial = x - radiusM < m.x0_m || x + radiusM > m.x0_m + m.width * m.pixel_m || y + radiusM > m.y0_m || y - radiusM < m.y0_m - m.height * m.pixel_m;
  return { hereH, bestH, worstH, partial };
}
