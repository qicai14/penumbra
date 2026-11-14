// Pure geometry for the 3D terrain (T23). No model maths: heights are the shipped DEM counts, the lit/shadowed bit of every
// vertex is the T21 point-Sun mask, and the Sun's azimuth and elevation come from the model's track.
// World frame (metres, relative to the tile centre): +x east on the map, +y up, -z map north, so the map reads north-up from the south.
import type { NearMeta } from "./engine";
import { MASK_DARK_ALPHA, MASK_LIT_ALPHA, type MaskGrid } from "./mask";

const D = Math.PI / 180;
type Rgb = [number, number, number];

/** Map metres to world metres: x east, z = -north, relative to the tile centre. */
export function toWorld(m: NearMeta, x_m: number, y_m: number): [number, number] {
  return [x_m - (m.x0_m + (m.width * m.pixel_m) / 2), -(y_m - (m.y0_m - (m.height * m.pixel_m) / 2))];
}

/** Lowest and highest valid height in metres, and how many pixels are nodata. Same range the picker's grey image uses. */
export function heightRange(m: NearMeta, data: Int16Array): { lo_m: number; hi_m: number; nodata: number } {
  let lo = Infinity, hi = -Infinity, nodata = 0;
  for (const v of data) { if (v === m.nodata) nodata++; else { if (v < lo) lo = v; if (v > hi) hi = v; } }
  return { lo_m: m.offset_m + m.scale_m * lo, hi_m: m.offset_m + m.scale_m * hi, nodata };
}

/** DEM height in metres at the tile pixel holding a map point, or null off the tile or on nodata. */
export function groundAt(m: NearMeta, data: Int16Array, x_m: number, y_m: number): number | null {
  const px = Math.floor((x_m - m.x0_m) / m.pixel_m), py = Math.floor((m.y0_m - y_m) / m.pixel_m);
  if (px < 0 || py < 0 || px >= m.width || py >= m.height) return null;
  const v = data[py * m.width + px];
  return v === m.nodata ? null : m.offset_m + m.scale_m * v;
}

/** Mesh vertices sit exactly on the mask grid: vertex (i, j) is mask point (i, j), the centre of tile pixel (i*stride, j*stride), row-major j*cols+i. */
export function gridPositions(m: NearMeta, data: Int16Array, g: MaskGrid, exaggeration: number, lo_m: number): { positions: Float32Array; heights_m: Float32Array; nodata: number } {
  const positions = new Float32Array(g.cols * g.rows * 3), heights_m = new Float32Array(g.cols * g.rows);
  let nodata = 0;
  for (let j = 0; j < g.rows; j++) {
    for (let i = 0; i < g.cols; i++) {
      const px = i * g.stride, py = j * g.stride, k = j * g.cols + i;
      const v = data[py * m.width + px];
      // a nodata pixel is drawn at the tile's lowest height and counted, never read as 0 m
      const h = v === m.nodata ? (nodata++, lo_m) : m.offset_m + m.scale_m * v;
      const [x, z] = toWorld(m, m.x0_m + m.pixel_m * (px + 0.5), m.y0_m - m.pixel_m * (py + 0.5));
      heights_m[k] = h;
      positions.set([x, h * exaggeration, z], k * 3);
    }
  }
  return { positions, heights_m, nodata };
}

/** Two triangles per grid cell, counter-clockwise seen from above. */
export function gridIndices(cols: number, rows: number): Uint32Array {
  const idx = new Uint32Array(Math.max(0, cols - 1) * Math.max(0, rows - 1) * 6);
  let n = 0;
  for (let j = 0; j < rows - 1; j++) {
    for (let i = 0; i < cols - 1; i++) {
      const a = j * cols + i, b = a + 1, c = a + cols, d = c + 1;
      idx.set([a, c, b, b, c, d], n); n += 6;
    }
  }
  return idx;
}

const toLinear = (b: number) => { const c = b / 255; return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; };

/**
 * Linear-RGB vertex colours, three per vertex. Each vertex is the 2D map's pixel: the height as grey, tinted by that
 * vertex's own mask bit (lit: Sun ember at MASK_LIT_ALPHA, shadowed: black at MASK_DARK_ALPHA). Without a mask it is the grey alone.
 * The tint is the only thing that carries lit or shadowed; no light or shadow map is involved.
 */
export function vertexColors(heights_m: Float32Array, range: { lo_m: number; hi_m: number }, lit: Uint8Array | null, sunRgb: Rgb, out?: Float32Array): Float32Array {
  const col = out ?? new Float32Array(heights_m.length * 3);
  const k = range.hi_m > range.lo_m ? 255 / (range.hi_m - range.lo_m) : 0;
  for (let v = 0; v < heights_m.length; v++) {
    const grey = Math.round((heights_m[v] - range.lo_m) * k);
    for (let c = 0; c < 3; c++) {
      let b = grey;
      if (lit) { const on = lit[v] !== 0, a = Math.round(255 * (on ? MASK_LIT_ALPHA : MASK_DARK_ALPHA)) / 255; b = grey * (1 - a) + (on ? sunRgb[c] * a : 0); } // alpha quantised to 8 bits like maskPixels
      col[v * 3 + c] = toLinear(Math.round(b));
    }
  }
  return col;
}

/** Unit vector toward the Sun in the world frame, from azimuth (clockwise from local north) and elevation at a site of longitude `lon_deg`.
 * Local north on the polar map is radially outward, (sin lon, cos lon); local east is (cos lon, -sin lon); the map is conformal, so angles hold. */
export function sunDirection(az_deg: number, el_deg: number, lon_deg: number): [number, number, number] {
  const a = az_deg * D, e = el_deg * D, l = lon_deg * D, h = Math.cos(e);
  const mx = Math.cos(a) * Math.sin(l) + Math.sin(a) * Math.cos(l), my = Math.cos(a) * Math.cos(l) - Math.sin(a) * Math.sin(l);
  return [h * mx, Math.sin(e), -h * my];
}

/** Vertical scale options; 1 is true scale. */
export const EXAGGERATIONS = [1, 2, 3] as const;
export const exaggerationLabel = (k: number) => (k === 1 ? "1× (true scale)" : `${k}× (exaggerated)`);
