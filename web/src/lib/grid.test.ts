import { expect, test } from "bun:test";
import { bestCell, contour, decodeGrid, nearestBetter, NODATA, readout, operableAt, type GridMeta } from "./grid";

// 6x4 grid, 100 m cells, top-left (1000, 5000); one date. Operable hours rise to the right: 0, 50, ... 250 per row.
const meta: GridMeta = { region: "t", pixel_m: 100, x0_m: 1000, y0_m: 5000, width: 6, height: 4, span_days: 30, mast_m: 2, dates: ["2025-03-06"], landing_unix_s: [0], url: "x" };
const op = Uint16Array.from({ length: 24 }, (_, n) => (n % 6) * 50);
op[23] = NODATA;
const lethal = new Uint16Array(24).fill(10);
const bytes = new Uint8Array(24 * 4);
const dv = new DataView(bytes.buffer);
[...op, ...lethal].forEach((v, i) => dv.setUint16(2 * i, v, true));
const g = decodeGrid(bytes.buffer, meta);

test("decode round-trips operable and lethal and checks the size", () => {
  expect(Array.from(g.operable[0])).toEqual(Array.from(op));
  expect(g.lethal[0][5]).toBe(10);
  expect(() => decodeGrid(new ArrayBuffer(8), meta)).toThrow(/BAD_INPUT/);
});

test("cell lookup uses cell centres, top row first; no-data is null", () => {
  expect(operableAt(g, 0, 1150, 4950)).toBe(50);
  expect(operableAt(g, 0, 1550, 4650)).toBeNull();
  expect(operableAt(g, 0, 999, 4950)).toBeNull();
});

test("nearest better site: next cell to the right is +50 h = +2.1 d, 100 m, bearing 90 deg", () => {
  const b = nearestBetter(g, 0, 1150, 4950, 1000);
  expect(b).toMatchObject({ kind: "better", distM: 100, bearingDeg: 90 });
  expect((b as any).plusDays).toBeCloseTo(50 / 24, 6);
  expect(readout(b)).toBe("Better site within 1 km: +2.1 d, 100 m, right on the map.");
});

test("best cell in the radius reports none; no-data site reports no data", () => {
  expect(nearestBetter(g, 0, 1550, 4950, 1000)).toEqual({ kind: "none", partial: true });
  expect(readout(nearestBetter(g, 0, 1550, 4950, 1000))).toContain("grid covers only part of the 1 km disc");
  expect(nearestBetter(g, 0, 1550, 4650)).toBeNull();
  expect(readout(null)).toContain("no grid data");
});

test("radius is a hard limit", () => {
  expect(nearestBetter(g, 0, 1050, 4950, 90)).toEqual({ kind: "none", partial: true });
});

test("a cell must gain at least the minimum: a 60 h floor skips the +50 h neighbour for the +100 h one 200 m away", () => {
  const b = nearestBetter(g, 0, 1150, 4950, 1000, 60);
  expect(b?.kind === "better" && Math.round(b.distM)).toBe(200);
});

test("contour at 120 h crosses between the 100 h and 150 h columns, 40% past the 100 h centre", () => {
  const segs = contour(g, 0, 120);
  expect(segs.length).toBeGreaterThan(0);
  for (const [x1, , x2] of segs) { expect(x1).toBeCloseTo(1250 + 40, 6); expect(x2).toBeCloseTo(x1, 6); }
});

test("best cell: most operable hours, ties to the first in row-major order, no-data skipped", () => {
  // Every row rises to 250 h at column 5 (row 3's is no-data), so rows 0-2 tie and the top one wins.
  expect(bestCell(g, 0)).toEqual({ operableH: 250, i: 5, j: 0, x: 1550, y: 4950 });
  expect(bestCell(decodeGrid(new Uint8Array(24 * 4).fill(0xff).buffer, meta), 0)).toBeNull();
});
