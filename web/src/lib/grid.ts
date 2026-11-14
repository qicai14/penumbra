// Regional point grid (T24 #25): operable and first-lethal hours per 100 m cell and landing date, computed natively and shipped as derived numbers.
// Pure decode and lookups; no model maths. Cell centres follow the tiles: x0 + (i + 0.5) * pixel, y0 - (j + 0.5) * pixel.

export type GridMeta = {
  region: string; pixel_m: number; x0_m: number; y0_m: number; width: number; height: number;
  span_days: number; mast_m: number; dates: string[]; landing_unix_s: number[]; url: string;
};
export type Grid = { meta: GridMeta; operable: Uint16Array[]; lethal: Uint16Array[] };

export const NODATA = 65535; // no data or outside the usable area
export const NO_LETHAL = 65534; // lethal shadow: none in the span
export const CONTOUR_H = 120; // the 5-operable-day line
export const NEAR_RADIUS_M = 1000;

/** Little-endian u16: per date in order, operable_h[w*h] then lethal_h[w*h], row 0 at the top. */
export function decodeGrid(buf: ArrayBuffer, meta: GridMeta): Grid {
  const n = meta.width * meta.height, d = meta.dates.length;
  if (buf.byteLength !== n * d * 4) throw new Error(`BAD_INPUT: ${meta.url} has ${buf.byteLength} bytes, expected ${n * d * 4}`);
  const v = new DataView(buf);
  const u16 = (off: number) => { const a = new Uint16Array(n); for (let i = 0; i < n; i++) a[i] = v.getUint16(off + 2 * i, true); return a; };
  const operable: Uint16Array[] = [], lethal: Uint16Array[] = [];
  for (let k = 0; k < d; k++) { operable.push(u16(k * n * 4)); lethal.push(u16(k * n * 4 + n * 2)); }
  return { meta, operable, lethal };
}

export async function fetchGrid(meta: GridMeta): Promise<Grid> {
  const res = await fetch(`${import.meta.env?.BASE_URL ?? "/"}${meta.url}`);
  if (!res.ok) throw new Error(`NO_DATA: ${meta.url} returned ${res.status}`);
  return decodeGrid(await res.arrayBuffer(), meta);
}

/** Index of the grid date equal to `date` (YYYY-MM-DD), or -1. */
export const dateIndex = (g: Grid, date: string) => g.meta.dates.indexOf(date);

/** Operable hours of the cell containing the map point, or null outside the grid or on a no-data cell. */
export function operableAt(g: Grid, k: number, x: number, y: number): number | null {
  const m = g.meta, i = Math.floor((x - m.x0_m) / m.pixel_m), j = Math.floor((m.y0_m - y) / m.pixel_m);
  if (i < 0 || j < 0 || i >= m.width || j >= m.height) return null;
  const h = g.operable[k][j * m.width + i];
  return h === NODATA ? null : h;
}

export type Better = { kind: "better"; plusDays: number; distM: number; bearingDeg: number; x: number; y: number } | { kind: "none"; partial: boolean };

/** Nearest cell within `radiusM` with at least `minGainH` more operable hours than the site's own cell (default one day,
 *  so a 1 h gain never reads "+0.0 d"); null when the site has no data. `partial` marks a disc that runs off the grid. */
export function nearestBetter(g: Grid, k: number, x: number, y: number, radiusM = NEAR_RADIUS_M, minGainH = 24): Better | null {
  const here = operableAt(g, k, x, y);
  if (here === null) return null;
  const m = g.meta, cx = (x - m.x0_m) / m.pixel_m - 0.5, cy = (m.y0_m - y) / m.pixel_m - 0.5, r = Math.ceil(radiusM / m.pixel_m) + 1;
  const partial = x - radiusM < m.x0_m || x + radiusM > m.x0_m + m.width * m.pixel_m || y + radiusM > m.y0_m || y - radiusM < m.y0_m - m.height * m.pixel_m;
  let best: Better = { kind: "none", partial }, bestD = Infinity;
  for (let j = Math.max(0, Math.floor(cy) - r); j <= Math.min(m.height - 1, Math.ceil(cy) + r); j++)
    for (let i = Math.max(0, Math.floor(cx) - r); i <= Math.min(m.width - 1, Math.ceil(cx) + r); i++) {
      const h = g.operable[k][j * m.width + i];
      if (h === NODATA || h < here + minGainH) continue;
      const px = m.x0_m + (i + 0.5) * m.pixel_m, py = m.y0_m - (j + 0.5) * m.pixel_m;
      const d = Math.hypot(px - x, py - y);
      if (d > radiusM || d >= bestD) continue;
      bestD = d;
      best = { kind: "better", plusDays: (h - here) / 24, distM: d, bearingDeg: (Math.atan2(px - x, py - y) * 180 / Math.PI + 360) % 360, x: px, y: py };
    }
  return best;
}

const WIND = ["up", "upper right", "right", "lower right", "down", "lower left", "left", "upper left"];
/** Map-frame direction: 0° is the top of the map, clockwise. At the pole grid north is not lunar north, so these are not compass points. */
export const mapDirection = (deg: number) => WIND[Math.round(deg / 45) % 8];

export function readout(b: Better | null): string {
  if (!b) return "Nearest site within 1 km with more operable days: no grid data at this site.";
  if (b.kind === "none") return `Nearest site within 1 km with more operable days: none gains a full day${b.partial ? " (the grid covers only part of the 1 km disc here)" : ""}.`;
  return `Nearest site within 1 km with more operable days: +${b.plusDays.toFixed(1)} d, ${Math.round(b.distM)} m away (${Math.round(b.bearingDeg)}° from map up, ${mapDirection(b.bearingDeg)} on the map)`;
}

/** Marching squares at `level` hours over cell centres; segments in map metres. Squares touching a no-data cell are skipped. */
export function contour(g: Grid, k: number, level = CONTOUR_H): [number, number, number, number][] {
  const m = g.meta, a = g.operable[k], out: [number, number, number, number][] = [];
  const X = (i: number) => m.x0_m + (i + 0.5) * m.pixel_m, Y = (j: number) => m.y0_m - (j + 0.5) * m.pixel_m;
  for (let j = 0; j < m.height - 1; j++)
    for (let i = 0; i < m.width - 1; i++) {
      const v = [a[j * m.width + i], a[j * m.width + i + 1], a[(j + 1) * m.width + i + 1], a[(j + 1) * m.width + i]]; // tl, tr, br, bl
      if (v.includes(NODATA)) continue;
      const hi = v.map((x) => x >= level);
      const n = hi.filter(Boolean).length;
      if (n === 0 || n === 4) continue;
      const c = [[i, j], [i + 1, j], [i + 1, j + 1], [i, j + 1]];
      // Edge e joins corner e and e+1; a crossing sits where the linear interpolation reaches `level`.
      const cross = (e: number): [number, number] | null => {
        const f = (e + 1) % 4;
        if (hi[e] === hi[f]) return null;
        const t = (level - v[e]) / (v[f] - v[e]);
        return [X(c[e][0]) + t * (X(c[f][0]) - X(c[e][0])), Y(c[e][1]) + t * (Y(c[f][1]) - Y(c[e][1]))];
      };
      const pts = [0, 1, 2, 3].map(cross).filter((p): p is [number, number] => p !== null);
      if (pts.length === 2) out.push([...pts[0], ...pts[1]]);
      else if (pts.length === 4) out.push([...pts[0], ...pts[3]], [...pts[1], ...pts[2]]); // saddle: fixed pairing, rare
    }
  return out;
}

export type Best = { operableH: number; i: number; j: number; x: number; y: number };

/** Cell with the most operable hours on date index `k`. Ties go to the first in row-major order (top row first, then left to
 *  right in the map frame); null when every cell is no-data. */
export function bestCell(g: Grid, k: number): Best | null {
  const m = g.meta, a = g.operable[k];
  let bi = -1, bh = -1;
  for (let n = 0; n < a.length; n++) if (a[n] !== NODATA && a[n] > bh) { bh = a[n]; bi = n; }
  if (bi < 0) return null;
  const i = bi % m.width, j = Math.floor(bi / m.width);
  return { operableH: bh, i, j, x: m.x0_m + (i + 0.5) * m.pixel_m, y: m.y0_m - (j + 0.5) * m.pixel_m };
}
