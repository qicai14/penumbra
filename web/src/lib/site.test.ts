import { expect, test } from "bun:test";
import type { NearMeta } from "./engine";
import { checkDate, checkPick, dateRange, edgeDistanceM, parseDate } from "./site";
import { latlonToXy, xyToLatlon } from "./fixture-projection";

// 30.72 km square, upper-left (61740, 153360), as in the manifest example.
const near: NearMeta = { width: 256, height: 256, x0_m: 61740, y0_m: 153360, pixel_m: 120, scale_m: 0.25, offset_m: 0, nodata: -32768 };
const cx = 61740 + 15360, cy = 153360 - 15360;

test("edge distance is the nearest of the four edges, negative outside", () => {
  expect(edgeDistanceM(cx, cy, near)).toBe(15360);
  expect(edgeDistanceM(61740 + 3000, cy, near)).toBe(3000);
  expect(edgeDistanceM(cx, 153360 - 100, near)).toBe(100);
  expect(edgeDistanceM(61740 - 50, cy, near)).toBe(-50);
});

test("a pick under 7 km from the edge is refused with a reason; exactly 7.05 km passes", () => {
  expect(checkPick(cx, cy, near).ok).toBe(true);
  expect(checkPick(61740 + 7050, cy, near).ok).toBe(true);
  const r = checkPick(61740 + 7049, cy, near);
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

test("stand-in projection round-trips and matches the pinned IM-2 point", () => {
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

test("the refusal text shows the 7.05 km border and rounds the distance down", () => {
  const r = checkPick(61740 + 7049, cy, near);
  expect(!r.ok && r.reason).toContain("7.04 km");
  expect(!r.ok && r.reason).toContain("7.05 km");
});
