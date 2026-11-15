// "Why the Moon is different" (#114): numbers derived from the scenario's Sun elevation series. Pure.
export const ROCK_M = 2;

/** Length in metres of the shadow a rock of `heightM` casts with the Sun `elDeg` above the horizon; null when the Sun is not up. */
export function shadowLengthM(elDeg: number, heightM = ROCK_M): number | null {
  if (!(elDeg > 0)) return null;
  return heightM / Math.tan((elDeg * Math.PI) / 180);
}

/** Highest value of a Sun-elevation series (degrees), or null when empty. */
export function maxElevation(el: ArrayLike<number>): number | null {
  let m = -Infinity;
  for (let i = 0; i < el.length; i++) if (el[i] > m) m = el[i];
  return m === -Infinity ? null : m;
}

export function fmtLength(m: number): string {
  return m >= 10 ? `${Math.round(m)} m` : `${m.toFixed(1)} m`;
}
