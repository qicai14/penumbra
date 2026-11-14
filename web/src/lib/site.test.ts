import { expect, test } from "bun:test";
import type { NearMeta } from "./engine";
import { checkDate, checkPick, edgeDistanceM, parseDate } from "./site";
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

test("a pick under 7 km from the edge is refused with a reason; exactly 7 km passes", () => {
  expect(checkPick(cx, cy, near).ok).toBe(true);
  expect(checkPick(61740 + 7000, cy, near).ok).toBe(true);
  const r = checkPick(61740 + 6999, cy, near);
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

test("a date outside 2026-01-01..2030-12-31 is refused (P16)", () => {
  expect(checkDate("2026-01-01", 30).ok).toBe(true);
  expect(checkDate("2030-11-30", 30).ok).toBe(true);
  expect(checkDate("2025-12-31", 30).ok).toBe(false);
  expect(checkDate("2031-01-01", 30).ok).toBe(false);
  const e = checkDate("2030-12-20", 30);
  expect(!e.ok && e.reason).toContain("past the end");
  expect(checkDate("nope", 30).ok).toBe(false);
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
