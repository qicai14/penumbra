// Pure helpers for the Compare A vs B view (T27): the URL query for two site-date pairs and the preset edits, and the
// B - A text. Text and validation only; every operable-time figure comes from the model's `survival` result.
import type { EphSegment, NearMeta, Preset } from "./engine";
import { latlonToXy, xyToLatlon } from "./projection";
import { presetFields, setPresetField, type FieldKey } from "./headline";
import { checkDate, checkPick, DEFAULT_MAST_M, DEFAULT_SPAN_DAYS, MAST_MAX_M, parseDate, SPANS_DAYS } from "./site";
import { fmtDays } from "./timelines";
import { dateIndex, nearestBetter, type Grid } from "./grid";
import { DATE_SHIFT_D, PIN_RADIUS_M, withinRadius } from "./whatChanges";

export type Pair = { lat: number; lon: number; date: string }; // date: landing day, UTC "YYYY-MM-DD"
export type PresetEdit = [FieldKey, number]; // value in the preset form's units (efficiency in percent)
export type CompareState = { a: Pair; b: Pair; presetEdits: PresetEdit[]; mastM: number; spanDays: number };

const KEYS: FieldKey[] = ["capacity_wh", "start_wh", "min_wh", "panel_area_m2", "efficiency", "load_w", "earth_margin_deg"];

/** The query string (no "?"): `a=<lat>,<lon>@<date>&b=...&preset=<field>:<value>,...`. The preset part is left out when unedited. */
export function encodeCompare(c: CompareState): string {
  const pair = (p: Pair) => `${p.lat},${p.lon}@${p.date}`;
  const q = [`a=${pair(c.a)}`, `b=${pair(c.b)}`];
  if (c.mastM !== DEFAULT_MAST_M) q.push(`mast=${c.mastM}`);
  if (c.spanDays !== DEFAULT_SPAN_DAYS) q.push(`span=${c.spanDays}`);
  if (c.presetEdits.length) q.push(`preset=${c.presetEdits.map(([k, v]) => `${k}:${v}`).join(",")}`);
  return q.join("&");
}

type DecodeOpts = { fallback: Pair; spanDays: number; mastM?: number; metas: NearMeta[]; eph: EphSegment[] };

/** One pair from "lat,lon@date", or null when malformed or refused (coordinate range, P16 date, P5 tile edge when tiles are given). */
function parsePair(s: string | null, o: DecodeOpts): Pair | null {
  const m = s?.match(/^([^,@]+),([^,@]+)@(\d{4}-\d{2}-\d{2})$/);
  if (!m) return null;
  const lat = Number(m[1]), lon = Number(m[2]), date = m[3];
  if (!Number.isFinite(lat) || !Number.isFinite(lon) || lat < -90 || lat > 90 || lon < -180 || lon > 360) return null;
  if (!checkDate(date, o.spanDays, o.eph).ok) return null;
  if (o.metas.length) {
    const [x, y] = latlonToXy(lat, lon);
    if (!o.metas.some((t) => checkPick(x, y, t).ok)) return null;
  }
  return { lat, lon, date };
}

/** Reads the query. `active` is true when `a` or `b` is present. A bad A becomes the fallback pair; a bad B becomes A; bad preset entries are dropped. */
export function decodeCompare(search: string, o: DecodeOpts): CompareState & { active: boolean } {
  const q = new URLSearchParams(search);
  // Same rules as the site view: a span from its buttons, a mast from its 0..10 m slider; anything else is the default.
  const span = Number(q.get("span"));
  const spanDays = (SPANS_DAYS as readonly number[]).includes(span) ? span : o.spanDays;
  const mastRaw = q.get("mast");
  const mast = mastRaw?.trim() ? Number(mastRaw) : NaN;
  const mastM = Number.isFinite(mast) && mast >= 0 && mast <= MAST_MAX_M ? mast : (o.mastM ?? DEFAULT_MAST_M);
  const po = { ...o, spanDays };
  const a = parsePair(q.get("a"), po) ?? o.fallback;
  const b = parsePair(q.get("b"), po) ?? a;
  const presetEdits: PresetEdit[] = [];
  for (const part of (q.get("preset") ?? "").split(",")) {
    const [k, v] = part.split(":");
    const n = v?.trim() ? Number(v) : NaN;
    if (KEYS.includes(k as FieldKey) && Number.isFinite(n)) presetEdits.push([k as FieldKey, n]);
  }
  return { active: q.has("a") || q.has("b"), a, b, presetEdits, mastM, spanDays };
}

/** The base preset with each edit applied through the planner's own checks; a refused edit is skipped. */
export function applyPresetEdits(base: Preset, edits: PresetEdit[]): Preset {
  let p = base;
  for (const [k, v] of edits) {
    const next = setPresetField(p, k, v);
    if (typeof next !== "string") p = next;
  }
  return p;
}

/** The fields where `p` differs from `base`, in the form's units. */
export function presetEdits(base: Preset, p: Preset): PresetEdit[] {
  const was = new Map(presetFields(base).map((f) => [f.key, f.value]));
  return presetFields(p).filter((f) => f.value !== was.get(f.key)).map((f) => [f.key, f.value]);
}

const MINUS = "−";

/** Whether two operable times read the same on screen (fmtDays shows 0.1 h under a day). */
const same = (diffH: number) => fmtDays(Math.abs(diffH)) === "0 h";

/** The B - A row for operable days: "+2 days", "−1 day", "+6.5 h", or "no difference". */
export function diffDays(aH: number, bH: number): string {
  const d = bH - aH;
  return same(d) ? "no difference" : `${d > 0 ? "+" : MINUS}${fmtDays(Math.abs(d))}`;
}

export type PairResult = { pair: Pair; operableH: number };

/** One sentence on B - A operable time that names what changed: the site, the date, both, or nothing. */
export function compareSentence(a: PairResult, b: PairResult): string {
  const siteChanged = a.pair.lat !== b.pair.lat || a.pair.lon !== b.pair.lon;
  const shift = Math.round((parseDate(b.pair.date)! - parseDate(a.pair.date)!) / 86400);
  if (!siteChanged && shift === 0) return "B is the same site and date as A, so there is nothing to compare.";
  const days = `${Math.abs(shift)} ${Math.abs(shift) === 1 ? "day" : "days"} ${shift > 0 ? "later" : "earlier"}`;
  const change = siteChanged
    ? shift === 0 ? "Moving from site A to site B on the same landing date" : `Moving from site A to site B and landing ${days}`
    : `Landing ${days} at the same site`;
  const d = b.operableH - a.operableH;
  const result = same(d)
    ? `makes no difference to operable time: both give ${fmtDays(a.operableH)}`
    : `gives ${fmtDays(Math.abs(d))} ${d > 0 ? "more" : "less"} operable time than A (B ${fmtDays(b.operableH)}, A ${fmtDays(a.operableH)})`;
  return `${change} ${result}.`;
}

/** B when the URL gives none (C4-3): the best cell within the pin radius, the figure of "Pin moved … m: best cell" in What
 *  changes what (same grid, same radius, same 24 h bar for "better"); else the same site DATE_SHIFT_D days later. The grid is
 *  only valid for the default preset, mast and span, as everywhere else. */
export function defaultB(a: Pair, grids: Grid[], o: { mastM: number; spanDays: number; hasPreset: boolean; eph: Parameters<typeof checkDate>[2] }): { pair: Pair; note: string } {
  const [x, y] = latlonToXy(a.lat, a.lon);
  for (const g of grids) {
    const k = dateIndex(g, a.date);
    if (k < 0 || o.hasPreset || o.mastM !== g.meta.mast_m || o.spanDays !== g.meta.span_days) continue;
    const near = withinRadius(g, k, x, y);
    if (!near) continue;
    const b = nearestBetter(g, k, x, y, PIN_RADIUS_M, near.bestH - near.hereH);
    if (b?.kind === "better" && near.bestH - near.hereH >= 24) {
      const [lat, lon] = xyToLatlon(b.x, b.y);
      return { pair: { lat: +lat.toFixed(4), lon: +lon.toFixed(4), date: a.date }, note: `B is the best cell within ${PIN_RADIUS_M} m of A, ${Math.round(b.distM)} m away, on the same date.` };
    }
  }
  // Later first; at the end of the ephemeris table (P16) try earlier, else keep A's date rather than give B an error.
  for (const d of [DATE_SHIFT_D, -DATE_SHIFT_D]) {
    const date = new Date((parseDate(a.date)! + d * 86400) * 1000).toISOString().slice(0, 10);
    if (checkDate(date, o.spanDays, o.eph).ok)
      return { pair: { ...a, date }, note: `B is the same site ${DATE_SHIFT_D} days ${d > 0 ? "later" : "earlier"}: no better cell within ${PIN_RADIUS_M} m of A, or no grid for this setup.` };
  }
  return { pair: { ...a }, note: `B is the same as A: no better cell within ${PIN_RADIUS_M} m, and no other date fits the ephemeris table. Edit B to compare.` };
}
