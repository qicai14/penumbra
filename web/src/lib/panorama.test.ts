import { expect, test } from "bun:test";
import { azToX, elToY, fmtDist, fmtEl, horizonAt, horizonIndex, trackSegments, wrap360, xToAz, type View } from "./panorama";

const v: View = { x0: 40, w: 720, y0: 10, h: 200, centre: 180, span: 360, elMin: -10, elMax: 10 };
const horizon = {
  el_deg: Float64Array.from({ length: 720 }, (_, i) => i / 10),
  dist_m: Float64Array.from({ length: 720 }, (_, i) => (i === 3 ? 30 : 1000 + i)),
};

test("az and x round-trip, 0 at the left edge, wrap at 360", () => {
  expect(azToX(0, v)).toBeCloseTo(40);
  expect(azToX(180, v)).toBeCloseTo(400);
  expect(azToX(359.5, v)).toBeCloseTo(40 + 719);
  for (const az of [0, 12.3, 180, 359.9]) expect(xToAz(azToX(az, v), v)).toBeCloseTo(az, 6);
  expect(wrap360(-1)).toBe(359);
  expect(wrap360(360)).toBe(0);
});

test("zoomed azimuth window wraps across north", () => {
  const z: View = { ...v, centre: 359, span: 40 };
  expect(azToX(359, z)).toBeCloseTo(400);
  expect(azToX(1, z)).toBeGreaterThan(azToX(359, z));
  expect(xToAz(azToX(2, z), z)).toBeCloseTo(2, 6);
});

test("el maps to y, up is smaller y", () => {
  expect(elToY(10, v)).toBeCloseTo(10);
  expect(elToY(-10, v)).toBeCloseTo(210);
  expect(elToY(1, v)).toBeLessThan(elToY(0, v));
});

test("horizon lookup uses 0.5 deg bins and wraps", () => {
  expect(horizonIndex(0)).toBe(0);
  expect(horizonIndex(359.9)).toBe(0);
  expect(horizonIndex(359.7)).toBe(719);
  expect(horizonIndex(-0.3)).toBe(719);
  const r = horizonAt(horizon, 1.5);
  expect(r.az_deg).toBe(1.5);
  expect(r.el_deg).toBeCloseTo(0.3);
  expect(r.near).toBe(true);
  expect(horizonAt(horizon, 2).near).toBe(false);
});

test("fmtEl never prints a negative zero", () => {
  expect(fmtEl(-0.04)).toBe("+0.0°");
  expect(fmtEl(0)).toBe("+0.0°");
  expect(fmtEl(-1.26)).toBe("−1.3°");
  expect(fmtEl(5.8)).toBe("+5.8°");
});

test("fmtDist switches to km at 1000 m", () => {
  expect(fmtDist(59.6)).toBe("60 m");
  expect(fmtDist(2400)).toBe("2.4 km");
});

test("trackSegments keeps one pass together and splits at the wrap", () => {
  const segs = trackSegments(Float64Array.from([10, 20, 350, 355]), Float64Array.from([1, 1, 1, 1]), v);
  expect(segs.map((s) => s.length)).toEqual([2, 2]);
});

test("trackSegments splits at the wrap edge", () => {
  const segs = trackSegments(Float64Array.from([358, 359, 1, 2]), Float64Array.from([0, 0, 0, 0]), v);
  expect(segs.length).toBe(2);
  expect(segs[0].length).toBe(2);
});
