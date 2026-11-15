import { expect, test } from "bun:test";
import type { NearMeta } from "./engine";
import { checkDate, checkPick, dateRange, edgeDistanceM, parseDate, pickBorderM, slopeText } from "./site";
import { latlonToXy, xyToLatlon } from "./projection";

// 30.72 km square, upper-left (61740, 153360), as in the manifest example.
const near: NearMeta = { width: 256, height: 256, x0_m: 61740, y0_m: 153360, pixel_m: 120, scale_m: 0.25, offset_m: 0, nodata: -32768 };
const cx = 61740 + 15360, cy = 153360 - 15360;

test("edge distance is the nearest of the four edges, negative outside", () => {
  expect(edgeDistanceM(cx, cy, near)).toBe(15360);
  expect(edgeDistanceM(61740 + 3000, cy, near)).toBe(3000);
  expect(edgeDistanceM(cx, 153360 - 100, near)).toBe(100);
  expect(edgeDistanceM(61740 - 50, cy, near)).toBe(-50);
});

// The two shipped near tiles (1536 px at 20 m); the same two numbers are asserted in crates/core/tests/horizon.rs.
const shipped = (x0_m: number, y0_m: number): NearMeta => ({ width: 1536, height: 1536, x0_m, y0_m, pixel_m: 20, scale_m: 0.04, offset_m: 0, nodata: -32768 });
test("pickBorderM: 7 km of ground times the farthest-corner scale factor plus half a pixel (matches Rust)", () => {
  expect(Math.abs(pickBorderM(shipped(61740, 153360)) - 7028.5914)).toBeLessThan(1e-4); // mons-mouton
  expect(Math.abs(pickBorderM(shipped(-26360, 3360)) - 7010.8368)).toBeLessThan(1e-4); // shackleton-ridge
});

// A 120 m test tile at corner (61740, 153360) has a 7078.59 m border: a pick at 7078 m is refused, 7079 m passes.
test("a pick under the tile's border is refused with a reason; at the border it passes", () => {
  expect(checkPick(cx, cy, near).ok).toBe(true);
  expect(checkPick(61740 + 7079, cy, near).ok).toBe(true);
  const r = checkPick(61740 + 7078, cy, near);
  expect(r.ok).toBe(false);
  expect(!r.ok && r.reason).toContain("7 km");
  const out = checkPick(0, 0, near);
  expect(!out.ok && out.reason).toContain("outside");
});

test("parseDate is strict", () => {
  expect(parseDate("2026-11-14")).toBe(1794614400);
  expect(parseDate("2026-02-30")).toBeNull();
  expect(parseDate("14/11/2026")).toBeNull();
  expect(parseDate("")).toBeNull();
});

// The shipped manifest today: one segment, 2026-01-01 00:00 .. 2031-01-01 00:00 hourly.
const MAIN = { id: "main", url: "", t0_unix_s: 1767225600, step_s: 3600, rows: 43825 };
// A replay segment (IM-2 landed 2025-03-06): 2025-03-01 .. 2025-04-01, hourly, with a gap before MAIN.
const REPLAY = { id: "replay", url: "", t0_unix_s: 1740787200, step_s: 3600, rows: 745 };

test("a date outside 2026-01-01..2030-12-31 is refused (P16)", () => {
  const eph = [MAIN];
  expect(checkDate("2026-01-01", 30, eph).ok).toBe(true);
  expect(checkDate("2030-11-30", 30, eph).ok).toBe(true);
  expect(checkDate("2025-12-31", 30, eph).ok).toBe(false);
  expect(checkDate("2031-01-01", 30, eph).ok).toBe(false);
  const e = checkDate("2030-12-20", 30, eph);
  expect(!e.ok && e.reason).toContain("past the end");
  expect(!e.ok && e.reason).toContain("(2030-12-31)");
  const before = checkDate("2025-12-31", 30, eph);
  expect(!before.ok && before.reason).toContain("starts (2026-01-01)");
  expect(checkDate("nope", 30, eph).ok).toBe(false);
  expect(dateRange(eph)).toEqual({ min: "2026-01-01", max: "2030-12-31" });
});

test("with two segments a date and its span must fit inside one; the gap is refused (P16)", () => {
  const eph = [REPLAY, MAIN];
  expect(checkDate("2025-03-06", 14, eph).ok).toBe(true);
  expect(checkDate("2025-03-20", 14, eph).ok).toBe(false); // 03-20 + 14 d = 04-03, past the replay segment (ends 04-01)
  expect(checkDate("2026-01-01", 30, eph).ok).toBe(true);
  const gap = checkDate("2025-08-01", 14, eph);
  expect(!gap.ok && gap.reason).toContain("gap between ephemeris tables (2025-03-31 to 2026-01-01)");
  expect(checkDate("2024-01-01", 14, eph).ok).toBe(false);
  expect(checkDate("2031-01-01", 14, eph).ok).toBe(false);
  expect(dateRange(eph)).toEqual({ min: "2025-03-01", max: "2030-12-31" });
});

test("projection round-trips and matches the pinned IM-2 point", () => {
  const [x, y] = latlonToXy(-84.7906, 29.1957);
  expect(x / 1000).toBeCloseTo(77.1, 0);
  expect(y / 1000).toBeCloseTo(138.0, 0);
  const [lat, lon] = xyToLatlon(x, y);
  expect(lat).toBeCloseTo(-84.7906, 8);
  expect(lon).toBeCloseTo(29.1957, 8);
});

test("checkPick refuses non-finite points (pole projects to NaN)", () => {
  expect(checkPick(NaN, Infinity, near).ok).toBe(false);
  expect(checkPick(...latlonToXy(90, 0), near).ok).toBe(false);
});

test("the refusal text shows this tile's border and rounds the distance down", () => {
  const r = checkPick(61740 + 7078, cy, near);
  expect(!r.ok && r.reason).toContain("7.07 km");
  expect(!r.ok && r.reason).toContain("7.08 km map border");
});

// Overlapping fixture tiles, 100 px at 100 m: A spans x 0..10000, B spans x 4000..14000 (y 10000..0). Borders ~7.05 km, so no point passes both in-border.
const tile = (x0_m: number): NearMeta => ({ width: 100, height: 100, x0_m, y0_m: 10000, pixel_m: 100, scale_m: 1, offset_m: 0, nodata: -32768 });
const big = (x0_m: number): NearMeta => ({ ...tile(x0_m), width: 400, height: 400, y0_m: 40000 }); // 40 km square, centre far from every edge
test("pickRegion: selected region, else first passing its border, else first holding the site", async () => {
  const { pickRegion } = await import("./site");
  const regs = [{ id: "a", near: big(0) }, { id: "b", near: big(5000) }];
  const [x, y] = [20000, 20000]; // inside both, past both borders
  expect(pickRegion(regs, x, y)?.id).toBe("a"); // manifest order
  expect(pickRegion(regs, x, y, "b")?.id).toBe("b"); // the UI's choice, whatever was loaded before
  expect(pickRegion(regs, x, y, "nowhere")?.id).toBe("a");
  // x=8000: 3 km inside b's west edge (fails its border), 8 km inside a's (passes)
  expect(pickRegion(regs, 8000, y, "b")?.id).toBe("a");
  // inside both tiles but within both borders (20 m from the shared far edges): first holding tile
  const small = [{ id: "s1", near: tile(0) }, { id: "s2", near: tile(4000) }];
  expect(pickRegion(small, 5000, 5000, "s2")?.id).toBe("s1");
  expect(pickRegion(small, 20000, 5000)).toBeUndefined();
});

test("pickBorderM for the shipped peak-near-cabeus-b near tile matches the Rust formula", async () => {
  const { readFileSync } = await import("node:fs");
  const mf = JSON.parse(readFileSync(new URL("../../public/data/manifest.json", import.meta.url), "utf8"));
  const t = mf.regions.find((r: { id: string }) => r.id === "peak-near-cabeus-b").near as NearMeta;
  const x = Math.max(Math.abs(t.x0_m), Math.abs(t.x0_m + t.width * t.pixel_m)), y = Math.max(Math.abs(t.y0_m), Math.abs(t.y0_m - t.height * t.pixel_m));
  const R = 1_737_400; // independent restatement: 7 km * (1 + r^2 / 4R^2) + half a pixel at the farthest corner
  expect(Math.abs(pickBorderM(t) - (7000 * (1 + (x * x + y * y) / (4 * R * R)) + t.pixel_m / 2))).toBeLessThan(1e-6);
  expect(pickBorderM(t)).toBeGreaterThan(7010); // about 7.1 km this far from the pole
});

test("slope line: one decimal, and no line at all when the engine has none (never 0°)", () => {
  expect(slopeText(7.26)).toBe("Slope 7.3° (20 m DEM, 60 m window)");
  expect(slopeText(0)).toBe("Slope 0.0° (20 m DEM, 60 m window)"); // a real flat pixel is a value
  expect(slopeText(undefined)).toBeNull();
  expect(slopeText(NaN)).toBeNull();
});
