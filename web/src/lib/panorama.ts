// Pure drawing and lookup helpers for the panorama. No model maths: every number comes from the engine.
import type { Horizon } from "./engine";

export const BINS = 720; // horizon index i = azimuth i * 0.5 deg
export const BIN_DEG = 0.5;
export const NEAR_M = 60; // ARCHITECTURE §3: dist_m < 60 reads as "within 3 pixels"

export type View = {
  x0: number; w: number; y0: number; h: number; // plot rectangle in CSS px
  centre: number; span: number;                 // azimuth shown across the width (deg)
  elMin: number; elMax: number;                 // elevation shown up the height (deg)
};

/** Wrap to [0, 360). */
export const wrap360 = (az: number) => ((az % 360) + 360) % 360;
/** Wrap to [-180, 180). */
export const wrap180 = (d: number) => wrap360(d + 180) - 180;

export const azToX = (az: number, v: View) => v.x0 + v.w / 2 + (wrap180(az - v.centre) / v.span) * v.w;
/** Azimuth in [0, 360) under pixel x. */
export const xToAz = (x: number, v: View) => wrap360(v.centre + ((x - v.x0 - v.w / 2) / v.w) * v.span);
export const elToY = (el: number, v: View) => v.y0 + ((v.elMax - el) / (v.elMax - v.elMin)) * v.h;
export const pxPerDegX = (v: View) => v.w / v.span;
export const pxPerDegY = (v: View) => v.h / (v.elMax - v.elMin);

/** Horizon bin nearest an azimuth; 359.9 wraps to bin 0. */
export const horizonIndex = (az: number) => Math.round(wrap360(az) / BIN_DEG) % BINS;

export type HorizonReading = { az_deg: number; el_deg: number; dist_m: number; near: boolean };
export function horizonAt(h: Horizon, az: number): HorizonReading {
  const i = horizonIndex(az);
  const dist_m = h.dist_m[i];
  return { az_deg: i * BIN_DEG, el_deg: h.el_deg[i], dist_m, near: dist_m < NEAR_M };
}

/** Pixel polyline for a track, split where it crosses the wrap edge so no line spans the plot. */
export function trackSegments(az: Float64Array, el: Float64Array, v: View): [number, number][][] {
  const out: [number, number][][] = [];
  let cur: [number, number][] = [];
  let px = NaN;
  for (let i = 0; i < az.length; i++) {
    const x = azToX(az[i], v);
    if (cur.length && Math.abs(x - px) > v.w / 2) { out.push(cur); cur = []; }
    cur.push([x, elToY(el[i], v)]);
    px = x;
  }
  if (cur.length) out.push(cur);
  return out;
}

/** Evenly spaced tick values covering [a, b]. */
export function ticks(a: number, b: number, step: number): number[] {
  const out: number[] = [];
  for (let t = Math.ceil(a / step) * step; t <= b + 1e-9; t += step) out.push(t);
  return out;
}

export const fmtDist = (m: number) => (m >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`);
export function fmtEl(el: number): string {
  const s = Math.abs(el).toFixed(1);
  return `${el < 0 && s !== "0.0" ? "−" : "+"}${s}°`; // -0.04 prints +0.0°, never −0.0°
}
export const fmtUtc = (tUnix: number) => new Date(tUnix * 1000).toISOString().slice(0, 16).replace("T", " ") + " UTC";
