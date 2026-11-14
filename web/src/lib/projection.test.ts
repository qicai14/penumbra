import { expect, test } from "bun:test";
import { latlonToXy, xyToLatlon } from "./projection";
import ref from "./projection-ref.json";

// projection-ref.json is core::geometry's answer (pinned by crates/pipeline/tests/projection_ref.rs to 0.5 mm).
// Holding TS to the same file within 0.5 mm keeps TS and Rust within 1 mm (#58).
const R = 1737400, D = Math.PI / 180, TOL_M = 0.0005;

function groundM(aLat: number, aLon: number, bLat: number, bLon: number): number {
  const dLon = ((aLon - bLon + 540) % 360) - 180;
  return Math.hypot((aLat - bLat) * D * R, dLon * D * R * Math.cos(aLat * D));
}

test("the reference covers SP-01, IM-2 and four corners of both near tiles", () => {
  expect(ref.map((p) => p.name)).toEqual(["SP-01", "IM-2", ...["mons-mouton", "shackleton-ridge"].flatMap((id) => ["NW", "NE", "SW", "SE"].map((c) => `${id} ${c}`))]);
});

for (const p of ref) {
  test(`${p.name}: TS matches core::geometry to 0.5 mm both ways`, () => {
    const [x, y] = latlonToXy(p.lat_deg, p.lon_deg);
    expect(Math.hypot(x - p.x_m, y - p.y_m)).toBeLessThanOrEqual(TOL_M);
    const [lat, lon] = xyToLatlon(p.x_m, p.y_m);
    expect(groundM(lat, lon, p.lat_deg, p.lon_deg)).toBeLessThanOrEqual(TOL_M);
    expect(lon).toBeGreaterThanOrEqual(0);
    expect(lon).toBeLessThan(360);
  });
}
