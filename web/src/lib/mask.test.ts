import { expect, test } from "bun:test";
import init from "./wasm/penumbra_wasm.js";
import { readFileSync } from "node:fs";
import { coalescing, maskAt, maskCell, maskGrid, maskPixels, checkMask, MASK_STRIDE, type MaskRequest, type MaskResult } from "./mask";
import { computeScenario, loadManifest, loadMaskEngine, latlonToXy, xyToLatlon } from "./engine";
import { horizonAt } from "./panorama";
import { parseDate } from "./site";

const M = { width: 1536, height: 1536, x0_m: 1000, y0_m: 5000, pixel_m: 20, scale_m: 1, offset_m: 0, nodata: -32768 };

test("grid size follows docs/API.md and a wrong length is refused", () => {
  expect(maskGrid(M, 8)).toEqual({ cols: 192, rows: 192, stride: 8 });
  expect(maskGrid({ width: 10, height: 7 }, 4)).toEqual({ cols: 3, rows: 2, stride: 4 });
  expect(() => checkMask(new Uint8Array(5), maskGrid(M, 8))).toThrow("mask has 5");
});

test("a map point reads the nearest mask point, row-major, north up", () => {
  const g = maskGrid(M, 8);
  const lit = new Uint8Array(g.cols * g.rows);
  lit[3 * g.cols + 5] = 1; // col 5, row 3
  const r: MaskResult = { ...g, lit, t_unix_s: 0, regionId: "x", mastM: 2 };
  // mask point (5, 3) is the centre of tile pixel (40, 24)
  const x = M.x0_m + M.pixel_m * 40.5, y = M.y0_m - M.pixel_m * 24.5;
  expect(maskAt(r, M, x, y)).toBe(1);
  expect(maskAt(r, M, x + 70, y)).toBe(1); // within half a stride (80 m)
  expect(maskAt(r, M, x + 90, y)).toBe(0);
  expect(maskAt(r, M, x, y - 90)).toBe(0);
  expect(maskCell(g, M, M.x0_m - 500, y)).toBeNull();
});

test("a rejected run frees the coalescer: the waiting request still runs and the error is reported", async () => {
  const ran: number[] = [], got: number[] = [], errs: string[] = [];
  const gates: ((ok: boolean) => void)[] = [];
  const ask = coalescing(
    (r) => new Promise((ok, fail) => { ran.push(r.t_unix_s); gates.push((good) => (good ? ok({ lit: new Uint8Array(1), cols: 1, rows: 1 }) : fail(new Error("worker died")))); }),
    (r) => got.push(r.t_unix_s), (e) => errs.push((e as Error).message));
  const req = (t: number): MaskRequest => ({ regionId: "r", t_unix_s: t, mastM: 2, stride: 8 });
  ask(req(1)); ask(req(2)); ask(req(3));
  gates[0](false);
  await new Promise((r) => setTimeout(r, 0));
  expect(errs).toEqual(["worker died"]);
  expect(ran).toEqual([1, 3]);
  gates[1](true);
  await new Promise((r) => setTimeout(r, 0));
  expect(got).toEqual([3]);
  ask(req(4)); // not stuck busy
  expect(ran).toEqual([1, 3, 4]);
});

test("overlay pixels: lit and shadowed get their own colour and alpha", () => {
  const px = maskPixels({ cols: 2, rows: 1, stride: 8, lit: Uint8Array.of(1, 0) }, [218, 92, 44], 0.4, [0, 0, 0], 0.6);
  expect([...px]).toEqual([218, 92, 44, 102, 0, 0, 0, 153]);
});

test("a fast scrub computes only the first and the last time (coalescing)", async () => {
  const ran: number[] = [], got: number[] = [];
  const gates: (() => void)[] = [];
  const req = (t: number): MaskRequest => ({ regionId: "r", t_unix_s: t, mastM: 2, stride: 8 });
  const ask = coalescing(
    (r) => new Promise((ok) => { ran.push(r.t_unix_s); gates.push(() => ok({ lit: new Uint8Array(1), cols: 1, rows: 1 })); }),
    (r) => got.push(r.t_unix_s), (e) => { throw e; });
  for (let t = 1; t <= 50; t++) ask(req(t)); // 50 scrub events while the first is computing
  gates[0]();
  await Promise.resolve(); await Promise.resolve(); await Promise.resolve();
  gates[1]();
  await new Promise((r) => setTimeout(r, 0));
  expect(ran).toEqual([1, 50]);
  expect(got).toEqual([1, 50]);
});

// Criterion 2 (#23): at the pinned site the map and the panorama agree. Real WASM, real tiles.
await init({ module_or_path: readFileSync(new URL("./wasm/penumbra_wasm_bg.wasm", import.meta.url)) });
globalThis.fetch = (async (u: string) =>
  new Response(readFileSync(new URL(`../../public/${String(u).replace(/^\//, "")}`, import.meta.url)))) as unknown as typeof fetch;

test("at the pinned site the mask is lit exactly when the Sun is above the panorama skyline (outside the disc-straddling band)", async () => {
  const man = await loadManifest();
  const [x0, y0] = latlonToXy(-84.7906, 29.1957);
  const region = man.regions.find((r) => { const n = r.near; return x0 >= n.x0_m && x0 < n.x0_m + n.width * n.pixel_m && y0 <= n.y0_m && y0 > n.y0_m - n.height * n.pixel_m; });
  const near = (region ?? man.regions[0]).near; // the default site's tile
  const g = maskGrid(near, MASK_STRIDE);
  // pin the site on a mask point: this checks agreement AT the mask point (the picker reads the nearest one, up to stride/2 x 20 m away)
  const i = Math.round((x0 - near.x0_m) / near.pixel_m / MASK_STRIDE), j = Math.round((near.y0_m - y0) / near.pixel_m / MASK_STRIDE);
  const [lat, lon] = xyToLatlon(near.x0_m + near.pixel_m * (i * MASK_STRIDE + 0.5), near.y0_m - near.pixel_m * (j * MASK_STRIDE + 0.5));
  const mast = 2;
  const sc = await computeScenario({ lat_deg: lat, lon_deg: lon, mast_m: mast, start_unix_s: parseDate("2026-11-14")!, span_days: 30 });
  const { engine } = await loadMaskEngine((region ?? man.regions[0]).id);
  const BAND_DEG = 0.7; // solar radius 0.27 deg, 0.5 deg panorama bins, plus the DEM's own relief inside a bin
  const decisive: { k: number; above: boolean }[] = [];
  for (let k = 0; k < sc.count; k++) {
    const h = horizonAt(sc.horizon, sc.track.sun_az_deg[k]);
    const d = sc.track.sun_el_deg[k] - h.el_deg;
    if (Math.abs(d) > BAND_DEG && h.dist_m < 20000) decisive.push({ k, above: d > 0 }); // blocker inside the near tile, which the mask marches
  }
  expect(decisive.some((c) => c.above)).toBe(true);
  expect(decisive.some((c) => !c.above)).toBe(true);
  // sample ~12 decisive times spread over the span, both kinds
  const picks = [...decisive.filter((c) => c.above), ...decisive.filter((c) => !c.above)].filter((_, n, a) => n % Math.max(1, Math.floor(a.length / 12)) === 0);
  for (const c of picks) {
    const lit = engine.point_sun_mask(sc.track.t_unix_s[c.k], mast, MASK_STRIDE);
    checkMask(lit, g);
    const r: MaskResult = { ...g, lit, t_unix_s: sc.track.t_unix_s[c.k], regionId: "", mastM: mast };
    expect(maskAt(r, near, ...(latlonToXy(lat, lon) as [number, number])) === 1, `step ${c.k}`).toBe(c.above);
  }
}, 120000);
