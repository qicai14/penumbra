// Pure helpers for the IM-2 replay view (T16 #17): the fixed inputs, the pit-input refusal rules and the d_c* sentence.
// No lunar maths: every number in the sentence is computed by the WASM model and passed in.

/** IM-2 landing site (U8) and replay window. Touchdown 2025-03-06T17:28:00Z is assumed (U8); the date is in the replay-im2 ephemeris segment. */
export const IM2 = { lat_deg: -84.7906, lon_deg: 29.1957, touchdown_unix_s: 1741282080, date: "2025-03-06", span_days: 3 } as const;
/** ESA degraded-crater slope, "11° slope from bottom to top" of the rim (E112). The depth it gives comes from the WASM, not from here. */
export const DEGRADED_SLOPE_DEG = 11;
export const DEFAULT_PIT_DIAMETER_M = 20; // W6
/** Panel height above the pit floor. Not published for IM-2, so it is assumed (U10). */
export const DEFAULT_PANEL_M = 1;

export type PitInput = { diameter_m: number; depth_m: number; panel_m: number };

/** The reason a pit input is refused, or null. Mirrors what the model throws BAD_INPUT on (diameter above 0, finite numbers); a depth or height below 0 is meaningless. */
export function checkPit(p: PitInput): string | null {
  if (!Number.isFinite(p.diameter_m) || p.diameter_m <= 0) return "Pit diameter must be a number above 0 m.";
  if (!Number.isFinite(p.depth_m) || p.depth_m < 0) return "Pit depth must be a number, 0 m or more.";
  if (!Number.isFinite(p.panel_m) || p.panel_m < 0) return "Panel height must be a number, 0 m or more.";
  return null;
}

export type SentenceInput = {
  elSunDeg: number; clearPct: number; diameterM: number; panelM: number; dStarM: number; typicalDepthM: number;
};
export const num = (x: number, d: number) => String(+x.toFixed(d));

/** The d_c* answer in plain words (SCIENCE-SPEC V4 statement 3). It reports the depth that would hide the Sun; it never says the lander was in shadow (N10). */
export function pitSentence(a: SentenceInput): string {
  const rel = a.typicalDepthM > a.dStarM ? "deeper than" : a.typicalDepthM < a.dStarM ? "shallower than" : "as deep as";
  return `From regional terrain the Sun is ${num(a.elSunDeg, 1)}° up with ${num(a.clearPct, 0)} % of its disc clear. ` +
    `A ${num(a.diameterM, 2)} m crater would need to be ${num(a.dStarM, 2)} m deep to hide the Sun from a panel ${num(a.panelM, 2)} m above its floor. ` +
    `A typical degraded crater that wide is ${num(a.typicalDepthM, 1)} m deep, ${rel} that.`;
}
