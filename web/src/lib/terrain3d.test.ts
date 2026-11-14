import { expect, test } from "bun:test";
import { maskAt, maskGrid, maskPixels, MASK_DARK_ALPHA, MASK_LIT_ALPHA, type MaskResult } from "./mask";
import { EXAGGERATIONS, gridIndices, gridPositions, groundAt, heightRange, sunDirection, toWorld, vertexColors } from "./terrain3d";

// 40 x 30 tile of 20 m pixels, y0 north edge; counts are 2 m steps over a 100 m offset.
const M = { width: 40, height: 30, x0_m: 1000, y0_m: 5000, pixel_m: 20, scale_m: 2, offset_m: 100, nodata: -32768 };
const data = Int16Array.from({ length: M.width * M.height }, (_, k) => (k % M.width) + 10 * Math.floor(k / M.width));
const linear = (b: number) => { const c = b / 255; return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; };

test("map metres go to world metres: tile centre is the origin, east is +x, north is -z", () => {
  const cx = M.x0_m + 400, cy = M.y0_m - 300;
  expect(toWorld(M, cx, cy)).toEqual([0, -0]);
  expect(toWorld(M, cx + 50, cy + 70)).toEqual([50, -70]);
});

test("mesh vertices sit on the mask grid, row-major, at the DEM height of that pixel", () => {
  const g = maskGrid(M, 8); // 5 x 4
  expect([g.cols, g.rows]).toEqual([5, 4]);
  const p = gridPositions(M, data, g, 1, heightRange(M, data).lo_m);
  const i = 3, j = 2, k = j * g.cols + i;
  const px = i * 8, py = j * 8;
  const [x, z] = toWorld(M, M.x0_m + 20 * (px + 0.5), M.y0_m - 20 * (py + 0.5));
  expect(Array.from(p.positions.slice(k * 3, k * 3 + 3))).toEqual([x, 100 + 2 * data[py * M.width + px], z]);
  // the same grid the mask reads: the map point at a vertex maps back to that vertex
  const lit = new Uint8Array(g.cols * g.rows); lit[k] = 1;
  const r: MaskResult = { ...g, lit, t_unix_s: 0, regionId: "x", mastM: 2 };
  expect(maskAt(r, M, M.x0_m + 20 * (px + 0.5), M.y0_m - 20 * (py + 0.5))).toBe(1);
});

test("exaggeration scales heights only, and true scale is the first option", () => {
  expect(EXAGGERATIONS[0]).toBe(1);
  const g = maskGrid(M, 8);
  const lo = heightRange(M, data).lo_m, a = gridPositions(M, data, g, 1, lo), b = gridPositions(M, data, g, 3, lo);
  for (let k = 0; k < g.cols * g.rows; k++) {
    expect(b.positions[k * 3 + 1]).toBeCloseTo(a.positions[k * 3 + 1] * 3, 3);
    expect(b.positions[k * 3]).toBe(a.positions[k * 3]);
    expect(b.positions[k * 3 + 2]).toBe(a.positions[k * 3 + 2]);
  }
});

test("a nodata pixel is drawn at the lowest height and counted, never as 0 m", () => {
  const d = data.slice(); d[0] = M.nodata;
  const p = gridPositions(M, d, maskGrid(M, 8), 1, heightRange(M, d).lo_m);
  expect(p.nodata).toBe(1);
  expect(p.heights_m[0]).toBe(heightRange(M, d).lo_m);
  expect(heightRange(M, d).nodata).toBe(1);
  expect(groundAt(M, d, M.x0_m + 1, M.y0_m - 1)).toBeNull();
  expect(groundAt(M, data, M.x0_m + 25, M.y0_m - 45)).toBe(100 + 2 * data[2 * M.width + 1]);
  expect(groundAt(M, data, M.x0_m - 1, M.y0_m - 1)).toBeNull();
});

test("two triangles per cell, counter-clockwise from above", () => {
  const idx = gridIndices(3, 2);
  expect(Array.from(idx)).toEqual([0, 3, 1, 1, 3, 4, 1, 4, 2, 2, 4, 5]);
  expect(gridIndices(1, 1).length).toBe(0);
});

test("vertex colour is the map's pixel: grey tinted by that vertex's own mask bit", () => {
  const heights = Float32Array.of(100, 200, 300), range = { lo_m: 100, hi_m: 300 }, sun: [number, number, number] = [218, 92, 44];
  const lit = Uint8Array.of(1, 0, 1);
  const col = vertexColors(heights, range, lit, sun);
  // vertex 1: grey 128, shadowed -> 128 * (1 - dark alpha), same on all channels
  const dark = Math.round(Math.round((200 - 100) * (255 / 200)) * (1 - Math.round(255 * MASK_DARK_ALPHA) / 255));
  for (let c = 0; c < 3; c++) expect(col[3 + c]).toBeCloseTo(linear(dark), 6);
  // vertex 2: grey 255, lit -> ember mixed at the lit alpha
  for (let c = 0; c < 3; c++) expect(col[6 + c]).toBeCloseTo(linear(Math.round(255 * (1 - Math.round(255 * MASK_LIT_ALPHA) / 255) + sun[c] * Math.round(255 * MASK_LIT_ALPHA) / 255)), 6);
  // flipping one bit changes only that vertex
  const flipped = vertexColors(heights, range, Uint8Array.of(1, 1, 1), sun);
  expect(Array.from(flipped.slice(0, 3))).toEqual(Array.from(col.slice(0, 3)));
  expect(Array.from(flipped.slice(3, 6))).not.toEqual(Array.from(col.slice(3, 6)));
  // no mask: plain grey
  expect(vertexColors(heights, range, null, sun)[0]).toBe(0);
});

test("Sun direction: azimuth is clockwise from local north on the polar map, elevation is up", () => {
  const close = (a: number[], b: number[]) => a.forEach((v, i) => expect(v).toBeCloseTo(b[i], 9));
  // at longitude 0 local north is map +y, which is world -z; east is +x
  close(sunDirection(0, 0, 0), [0, 0, -1]);
  close(sunDirection(90, 0, 0), [1, 0, 0]);
  close(sunDirection(180, 0, 0), [0, 0, 1]);
  // at longitude 90 east, north is map +x, east is map -y (world +z)
  close(sunDirection(0, 0, 90), [1, 0, 0]);
  close(sunDirection(90, 0, 90), [0, 0, 1]);
  // elevation lifts it; a 1 degree Sun stays nearly horizontal, and the vector is unit length
  const d = sunDirection(37, 1, 123);
  expect(d[1]).toBeCloseTo(Math.sin(Math.PI / 180), 12);
  expect(Math.hypot(...d)).toBeCloseTo(1, 12);
  expect(sunDirection(10, -5, 0)[1]).toBeLessThan(0);
});

test("the mask's own overlay and the vertex colours agree bit for bit at grey 0 and 255", () => {
  const sun: [number, number, number] = [218, 92, 44];
  const lit = Uint8Array.of(1, 0, 1, 0);
  const heights = Float32Array.of(0, 0, 255, 255); // grey 0, 0, 255, 255 over range 0..255
  const grid = { cols: 4, rows: 1, stride: 8 };
  const px = maskPixels({ ...grid, lit }, sun, MASK_LIT_ALPHA, [0, 0, 0], MASK_DARK_ALPHA); // what the 2D map composites
  const col = vertexColors(heights, { lo_m: 0, hi_m: 255 }, lit, sun);
  const toByte = (l: number) => Math.round(255 * (l <= 0.0031308 ? l * 12.92 : 1.055 * l ** (1 / 2.4) - 0.055));
  for (let v = 0; v < 4; v++) {
    const grey = heights[v], A = px[v * 4 + 3] / 255;
    for (let c = 0; c < 3; c++) expect(toByte(col[v * 3 + c])).toBe(Math.round(grey * (1 - A) + px[v * 4 + c] * A));
  }
});
