// Point-Sun mask helpers (T22). No model maths: the lit/shadowed bits are Engine.point_sun_mask (T21, SCIENCE-SPEC §3 V3).
// This file only places those bits on the map and coalesces requests so a scrub never queues work.
import type { NearMeta } from "./engine";

/** Timings are [est]: measured locally in bun's WASM, not in the deployed page.
 * Grid stride in near-tile pixels (20 m each). Fine: one point per 160 m, 0.35 to 1.1 s per mask [est]. Coarse: 320 m, 0.08 to 0.27 s [est], used while the marker moves. */
export const MASK_STRIDE = 8;
export const MASK_STRIDE_COARSE = 16;

export type MaskGrid = { cols: number; rows: number; stride: number };
export type MaskResult = MaskGrid & { lit: Uint8Array; t_unix_s: number; regionId: string; mastM: number };

/** docs/API.md: the WASM result carries no cols/rows, so they come from the formula. */
export function maskGrid(m: Pick<NearMeta, "width" | "height">, stride: number): MaskGrid {
  return { cols: Math.ceil((m.width - 1) / stride), rows: Math.ceil((m.height - 1) / stride), stride };
}

/** Throws unless the Engine's array has exactly the grid the formula gives. */
export function checkMask(lit: Uint8Array, g: MaskGrid): void {
  if (lit.length !== g.cols * g.rows) throw new Error(`BAD_INPUT: mask has ${lit.length} values, expected ${g.cols}x${g.rows}`);
}

/** Mask point i,j sits at the centre of tile pixel (i·stride, j·stride). The grid point nearest a map point, or null off the grid. */
export function maskCell(g: MaskGrid, m: NearMeta, x_m: number, y_m: number): { i: number; j: number } | null {
  const px = (x_m - m.x0_m) / m.pixel_m - 0.5, py = (m.y0_m - y_m) / m.pixel_m - 0.5;
  const i = Math.round(px / g.stride), j = Math.round(py / g.stride);
  return i >= 0 && j >= 0 && i < g.cols && j < g.rows ? { i, j } : null;
}

/** 1 lit, 0 shadowed, at the grid point nearest a map point. */
export function maskAt(r: MaskResult, m: NearMeta, x_m: number, y_m: number): 0 | 1 | null {
  const c = maskCell(r, m, x_m, y_m);
  return c ? (r.lit[c.j * r.cols + c.i] ? 1 : 0) : null;
}

/** RGBA overlay, one pixel per mask point: lit gets `litRgb` at `litAlpha`, shadowed gets `darkRgb` at `darkAlpha`. */
export function maskPixels(r: MaskGrid & { lit: Uint8Array }, litRgb: [number, number, number], litAlpha: number, darkRgb: [number, number, number], darkAlpha: number): Uint8ClampedArray<ArrayBuffer> {
  const out = new Uint8ClampedArray(new ArrayBuffer(r.cols * r.rows * 4));
  for (let k = 0; k < r.lit.length; k++) {
    const on = r.lit[k] !== 0;
    out.set(on ? litRgb : darkRgb, k * 4);
    out[k * 4 + 3] = Math.round(255 * (on ? litAlpha : darkAlpha));
  }
  return out;
}

export type MaskRequest = { regionId: string; t_unix_s: number; mastM: number; stride: number };
export type Runner = (req: MaskRequest) => Promise<Omit<MaskResult, "regionId" | "mastM" | "t_unix_s" | "stride"> & { lit: Uint8Array }>;

/**
 * One request in flight and at most one waiting: a newer request replaces the waiting one, so a fast scrub computes
 * only the time the marker rests on (plus the one already running). Results arrive in request order.
 */
export function coalescing(run: Runner, onResult: (r: MaskResult) => void, onError: (e: unknown) => void) {
  let busy = false, waiting: MaskRequest | null = null;
  async function go(req: MaskRequest) {
    busy = true;
    try {
      const r = await run(req);
      onResult({ ...r, stride: req.stride, t_unix_s: req.t_unix_s, regionId: req.regionId, mastM: req.mastM });
    } catch (e) { onError(e); }
    busy = false;
    const next = waiting; waiting = null;
    if (next) void go(next);
  }
  return (req: MaskRequest) => { if (busy) waiting = req; else void go(req); };
}
