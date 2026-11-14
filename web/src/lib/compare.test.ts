import { expect, test } from "bun:test";
import type { NearMeta, Preset } from "./engine";
import { applyPresetEdits, compareSentence, decodeCompare, diffDays, encodeCompare, presetEdits, type Pair } from "./compare";
import { xyToLatlon } from "./fixture-projection";

const A: Pair = { lat: -84.7906, lon: 29.1957, date: "2026-11-14" };
// The manifest example near tile: 30.72 km square, upper-left (61740, 153360).
const near: NearMeta = { width: 256, height: 256, x0_m: 61740, y0_m: 153360, pixel_m: 120, scale_m: 0.25, offset_m: 0, nodata: -32768 };
const preset: Preset = {
  capacity_wh: 10000, start_wh: 10000, min_wh: 0, panel_area_m2: 1.5, efficiency: 0.3, panel: "tracking",
  load_w: 220, earth_margin_deg: 2, mast_m: 2, cited: { load_w: true },
};
const dec = (s: string) => decodeCompare(s, { fallback: A, spanDays: 30, mastM: 2, metas: [] });

test("two pairs and the preset edits round-trip through the URL query", () => {
  const b: Pair = { lat: -85.1, lon: 31, date: "2026-11-17" };
  const edits: [any, number][] = [["efficiency", 25], ["load_w", 300]];
  const q = encodeCompare({ a: A, b, presetEdits: edits, mastM: 2, spanDays: 30 });
  expect(q).toBe("a=-84.7906,29.1957@2026-11-14&b=-85.1,31@2026-11-17&preset=efficiency:25,load_w:300");
  expect(dec(q)).toEqual({ active: true, a: A, b, presetEdits: edits, mastM: 2, spanDays: 30 });
});

test("no preset edits leaves out the preset parameter", () => {
  expect(encodeCompare({ a: A, b: A, presetEdits: [], mastM: 2, spanDays: 30 })).not.toContain("preset");
});

test("a query without a or b is not the compare view; with either it is", () => {
  expect(dec("").active).toBe(false);
  expect(dec("?x=1").active).toBe(false);
  expect(dec("?b=-85,30@2026-12-01").active).toBe(true);
});

test("a bad A falls back to the default pair; a bad B falls back to A", () => {
  const r = dec("a=oops&b=-84.8,29@2026-02-30");
  expect(r.a).toEqual(A);
  expect(r.b).toEqual(A);
  expect(dec("a=-85,30@2026-12-01&b=junk").b).toEqual({ lat: -85, lon: 30, date: "2026-12-01" });
  expect(dec("a=-85,30@2026-12-01&b=-85,31@2026-12-05").b).toEqual({ lat: -85, lon: 31, date: "2026-12-05" });
});

test("dates outside the ephemeris table and out-of-range coordinates are refused at the boundary", () => {
  expect(dec("a=-85,30@2031-03-01").a).toEqual(A);
  expect(dec("a=-85,30@2030-12-20").a).toEqual(A); // plus the 30 d span runs past the table
  expect(dec("a=-95,30@2026-12-01").a).toEqual(A);
  expect(dec("a=NaN,30@2026-12-01").a).toEqual(A);
  expect(dec("a=,@2026-12-01").a).toEqual(A);
});

test("with tile metadata a site outside every tile or too close to its edge is refused", () => {
  const [lat, lon] = xyToLatlon(61740 + 15360, 153360 - 15360); // tile centre
  const [elat, elon] = xyToLatlon(61740 + 1000, 153360 - 15360); // 1 km from the edge
  const centre: Pair = { lat: +lat.toFixed(4), lon: +lon.toFixed(4), date: "2026-12-01" };
  const o = { fallback: centre, spanDays: 30, mastM: 2, metas: [near] };
  expect(decodeCompare(`a=${lat},${lon}@2026-12-01`, o).a).toEqual({ lat, lon, date: "2026-12-01" });
  expect(decodeCompare("a=-80,30@2026-12-01", o).a).toEqual(centre);
  expect(decodeCompare(`a=${elat},${elon}@2026-12-01`, o).a).toEqual(centre);
});

test("bad preset entries are dropped, good ones kept", () => {
  expect(dec("a=-85,30@2026-12-01&preset=load_w:300,nope:3,load_w:abc,capacity_wh:").presetEdits).toEqual([["load_w", 300]]);
});

test("preset edits against a base: only changed fields are encoded; efficiency is in percent", () => {
  const next = applyPresetEdits(preset, [["efficiency", 25], ["load_w", 300]]);
  expect(next.load_w).toBe(300);
  expect(next.efficiency).toBeCloseTo(0.25, 10);
  expect(presetEdits(preset, next)).toEqual([["efficiency", 25], ["load_w", 300]]);
  expect(presetEdits(preset, preset)).toEqual([]);
});

test("an edit the planner would refuse is skipped when applied", () => {
  expect(applyPresetEdits(preset, [["efficiency", 250], ["load_w", 5]]).load_w).toBe(5);
  expect(applyPresetEdits(preset, [["efficiency", 250]]).efficiency).toBe(0.3);
});

const res = (p: Pair, operableH: number) => ({ pair: p, operableH });

test("sentence: site changed, B better", () => {
  const s = compareSentence(res(A, 240), res({ ...A, lat: -85.2 }, 288));
  expect(s).toContain("site B");
  expect(s).toContain("same landing date");
  expect(s).toContain("2 days more");
  expect(s).toContain("B 12 days, A 10 days");
});

test("sentence: date changed later, B worse", () => {
  const s = compareSentence(res(A, 240), res({ ...A, date: "2026-11-17" }, 216));
  expect(s).toContain("3 days later");
  expect(s).toContain("same site");
  expect(s).toContain("1 day less");
});

test("sentence: date earlier reads singular and earlier", () => {
  expect(compareSentence(res(A, 240), res({ ...A, date: "2026-11-13" }, 250))).toContain("1 day earlier");
});

test("sentence: both changed", () => {
  const s = compareSentence(res(A, 240), res({ lat: -85.2, lon: 30, date: "2026-12-01" }, 300));
  expect(s).toContain("site B");
  expect(s).toContain("17 days later");
  expect(s).toContain("2.5 days more");
});

test("sentence: a change that makes no difference", () => {
  const s = compareSentence(res(A, 240), res({ ...A, date: "2026-11-15" }, 240));
  expect(s).toContain("no difference");
  expect(s).toContain("both give 10 days");
});

test("sentence: B is the same site and date as A", () => {
  expect(compareSentence(res(A, 240), res({ ...A }, 240))).toContain("same site and date");
});

test("sentence: differences under a day read in hours", () => {
  expect(compareSentence(res(A, 240), res({ ...A, lat: -85 }, 246.5))).toContain("6.5 h more");
});

test("mast and span are written only when they differ from the defaults, and read back", () => {
  const base = { a: A, b: A, presetEdits: [] as [any, number][] };
  expect(encodeCompare({ ...base, mastM: 2, spanDays: 30 })).not.toMatch(/mast|span/);
  const q = encodeCompare({ ...base, mastM: 6.5, spanDays: 14 });
  expect(q).toContain("mast=6.5");
  expect(q).toContain("span=14");
  const r = dec(q);
  expect(r.mastM).toBe(6.5);
  expect(r.spanDays).toBe(14);
});

test("a span that is not one of the site view's choices, or a mast outside 0..10 m, falls back to the default", () => {
  expect(dec("a=-85,30@2026-12-01&span=45").spanDays).toBe(30);
  expect(dec("a=-85,30@2026-12-01&span=abc").spanDays).toBe(30);
  expect(dec("a=-85,30@2026-12-01&mast=11").mastM).toBe(2);
  expect(dec("a=-85,30@2026-12-01&mast=-1").mastM).toBe(2);
  expect(dec("a=-85,30@2026-12-01&mast=x").mastM).toBe(2);
  expect(dec("a=-85,30@2026-12-01&mast=0").mastM).toBe(0);
});

test("dates are checked against the decoded span", () => {
  // 2030-12-10 + 30 d runs past the table; + 14 d does not.
  expect(dec("a=-85,30@2030-12-10").a).toEqual(A);
  expect(dec("a=-85,30@2030-12-10&span=14").a).toEqual({ lat: -85, lon: 30, date: "2030-12-10" });
});

test("B - A row for operable days is signed", () => {
  expect(diffDays(240, 288)).toBe("+2 days");
  expect(diffDays(240, 216)).toBe("−1 day");
  expect(diffDays(240, 240)).toBe("no difference");
  expect(diffDays(240, 246.5)).toBe("+6.5 h");
});
