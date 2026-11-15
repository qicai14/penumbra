// The only module that imports lib/wasm/ (ARCHITECTURE §3.5).
// Typed arrays use platform byte order; every browser target is little-endian, so Int16Array over a file is safe.
// sun_earth_track (T02), horizon (T07), timeline (T08) and survival (T09) are WASM. The app never loads the FAKE layout fixture
// (web/tests/fixture/scenario.json): the step and the preset below are the real values the pipeline uses.
export type Site = { lat_deg: number; lon_deg: number; ground_m: number; mast_m: number };
export type Horizon = { el_deg: Float64Array; dist_m: Float64Array }; // 720 each
export type Track = {
  t_unix_s: Float64Array; sun_az_deg: Float64Array; sun_el_deg: Float64Array;
  sun_radius_deg: Float64Array; sun_dist_km: Float64Array; earth_az_deg: Float64Array;
  earth_el_deg: Float64Array; earth_radius_deg: Float64Array; earth_dist_km: Float64Array;
};
export type Timeline = {
  t_unix_s: Float64Array; sun_az_deg: Float64Array; sun_el_deg: Float64Array;
  illumination: Float64Array; visibility: Float64Array; earth_az_deg: Float64Array;
  earth_el_deg: Float64Array; earth_margin_deg: Float64Array; contact: Float64Array;
  sun_dist_km: Float64Array;
};
export type Survival = {
  operable_h: number; lethal_h: number | null; longest_dark_h: number;
  longest_any_sun_h: number; sunlit_h: number; contact_h: number; soc_wh: Float64Array;
};
export type Preset = {
  capacity_wh: number; start_wh: number; min_wh: number; panel_area_m2: number;
  efficiency: number; panel: "tracking" | { east: number; north: number; up: number };
  load_w: number; earth_margin_deg: number; mast_m: number;
  cited: Record<string, boolean>; // false = "assumed"
};
export type EngineError = {
  code: "NOT_LOADED" | "OUT_OF_COVERAGE" | "NO_DATA" | "OUT_OF_TABLE" | "BAD_INPUT";
  message: string;
};
export type Scenario = {
  api_version: number; site: Site; start_unix_s: number; step_s: number;
  count: number; preset: Preset; horizon: Horizon; track: Track; timeline: Timeline; survival: Survival;
};

/** Timeline step, s: the same as the pipeline's GRID_STEP_S (crates/pipeline/src/main.rs), so the heatmap and the headline agree. */
export const STEP_S = 600;
/** The lander preset: penumbra_core::power::ROVER_PRESET (E58a–d) and ROVER_CITED_FIELDS; engine.test.ts checks they match. */
export const DEFAULT_PRESET: Preset = {
  capacity_wh: 10000, start_wh: 10000, min_wh: 0, panel_area_m2: 1.5, efficiency: 0.3, panel: "tracking",
  load_w: 220, earth_margin_deg: 2, mast_m: 2,
  cited: { capacity_wh: true, panel_area_m2: true, efficiency: true, load_w: true },
};

/** Throws when a manifest (or the test fixture) was built for a different API than this engine. */
export function checkApiVersion(found: number, expected: number): void {
  if (found !== expected) throw new Error(`BAD_INPUT: api_version ${found} does not match engine ${expected}`);
}

/** The engine's own api_version(); loads the WASM module on first use. */
export async function engineApiVersion(): Promise<number> {
  const wasm = await import("./wasm/penumbra_wasm.js");
  await wasm.default();
  return wasm.api_version();
}

// A null or non-number cell becomes NaN, so validate() rejects it instead of reading 0.
const f64 = (v: unknown) => Float64Array.from(v as unknown[], (x) => (typeof x === "number" ? x : NaN));
const arrays = <T extends object>(o: Record<string, unknown>): T =>
  Object.fromEntries(Object.entries(o).map(([k, v]) => [k, f64(v)])) as T;

/** Turns parsed scenario JSON into typed arrays and checks it. `engineVersion` is what api_version() returns.
 *  Only the tests call it now (web/tests/fixture/scenario.json); the app computes every scenario in WASM. */
export function parseScenario(raw: any, engineVersion: number): Scenario {
  checkApiVersion(raw.api_version, engineVersion);
  const s: Scenario = {
    ...raw,
    horizon: arrays<Horizon>(raw.horizon),
    track: arrays<Track>(raw.track),
    timeline: arrays<Timeline>(raw.timeline),
    survival: { ...raw.survival, soc_wh: f64(raw.survival.soc_wh) },
  };
  validate(s.horizon); validate(s.track); validate(s.timeline); validate(s.survival);
  for (const [k, n] of [["track", s.track.t_unix_s.length], ["timeline", s.timeline.t_unix_s.length], ["soc_wh", s.survival.soc_wh.length]] as const)
    if (n !== raw.count) throw new Error(`BAD_INPUT: ${k} has ${n} rows, count is ${raw.count}`);
  return s;
}


/** Cheap guard against a units or ordering slip: lengths, monotonic time, az, el, illumination. Throws on the first fault. */
export function validate(result: Horizon | Track | Timeline | Survival): void {
  const cols = Object.entries(result).filter((e): e is [string, Float64Array] => e[1] instanceof Float64Array);
  const n = cols[0]?.[1].length ?? 0;
  const fail = (m: string): never => { throw new Error(`BAD_INPUT: ${m}`); };
  for (const [k, a] of cols) {
    if (a.length !== n) fail(`${k} has ${a.length} values, expected ${n}`);
    const bad = a.findIndex((x) =>
      !Number.isFinite(x) ||
      (/_az_deg$/.test(k) && !(x >= 0 && x < 360)) ||
      (/(^|_)el_deg$/.test(k) && Math.abs(x) > 90) ||
      (k === "dist_m" && x < 0) ||
      (k === "illumination" && !(x >= 0 && x <= 1)));
    if (bad >= 0) fail(`${k}[${bad}] = ${a[bad]} is out of range`);
  }
  if ("el_deg" in result && n !== 720) fail(`horizon has ${n} values, expected 720`);
  const t = (result as Partial<Track>).t_unix_s;
  if (t) for (let i = 1; i < t.length; i++) if (!(t[i] > t[i - 1])) fail(`t_unix_s is not increasing at ${i}`);
}

type TileMeta = { url: string; width: number; height: number; x0_m: number; y0_m: number; pixel_m: number; scale_m: number; offset_m: number; nodata: number };
export type EphSegment = { id: string; url: string; t0_unix_s: number; step_s: number; rows: number };
export type Manifest = {
  format_version: number; api_version: number;
  ephemeris: EphSegment[];
  far: TileMeta; regions: { id: string; name: string; near: TileMeta; mid: TileMeta }[];
  grids?: GridMeta[]; // regional point grids (T24); optional so the app runs without them
};

let manifestLoad: Promise<Manifest> | undefined;
/** The manifest, fetched once per page (a failed load is retried). */
export function loadManifest(): Promise<Manifest> {
  return (manifestLoad ??= fetchManifest().catch((e) => { manifestLoad = undefined; throw e; }));
}
async function fetchManifest(): Promise<Manifest> {
  const base = import.meta.env?.BASE_URL ?? "/";
  const res = await fetch(`${base}data/manifest.json`, { cache: "no-cache" });
  if (!res.ok) throw new Error(`NO_DATA: manifest.json returned ${res.status}`);
  const m: Manifest = await res.json();
  checkApiVersion(m.api_version, await engineApiVersion());
  return m;
}

// IM-2 tracer call (T02): site, mast, 2026-11-14T00:00Z, hourly, 24 rows.
export const TRACER = { lat_deg: -84.7906, lon_deg: 29.1957, ground_m: 0, mast_m: 2, start_unix_s: 1794614400, step_s: 3600, count: 24 };

/** An int16 tile exactly as shipped; each tile is fetched once. */
const tiles = new Map<string, Promise<Int16Array>>();
function fetchTile(t: TileMeta): Promise<Int16Array> {
  let p = tiles.get(t.url);
  if (!p) {
    p = (async () => {
      const res = await fetch(`${import.meta.env?.BASE_URL ?? "/"}${t.url}`);
      if (!res.ok) throw new Error(`NO_DATA: ${t.url} returned ${res.status}`);
      const data = new Int16Array(await res.arrayBuffer());
      if (data.length !== t.width * t.height) throw new Error(`BAD_INPUT: ${t.url} has ${data.length} values, expected ${t.width * t.height}`);
      return data;
    })();
    tiles.set(t.url, p);
    p.catch(() => tiles.delete(t.url));
  }
  return p;
}

async function fetchSegment(seg: EphSegment): Promise<Float64Array> {
  const res = await fetch(`${import.meta.env?.BASE_URL ?? "/"}${seg.url}`);
  if (!res.ok) throw new Error(`NO_DATA: ${seg.url} returned ${res.status}`);
  const buf = await res.arrayBuffer();
  if (buf.byteLength % 8) throw new Error(`BAD_INPUT: ${seg.url} is ${buf.byteLength} bytes, not whole f64s`);
  const data = new Float64Array(buf);
  if (data.length !== 6 * seg.rows) throw new Error(`BAD_INPUT: ${seg.url} has ${data.length} values, expected ${6 * seg.rows}`);
  return data;
}

type WasmEngine = InstanceType<typeof import("./wasm/penumbra_wasm.js").Engine>;
let engineLoad: Promise<WasmEngine> | undefined;
/** The WASM Engine with the shipped ephemeris table loaded; built once and cached (a failed load is retried). */
export function loadEngine(): Promise<WasmEngine> {
  return (engineLoad ??= (async () => {
    const wasm = await import("./wasm/penumbra_wasm.js");
    await wasm.default();
    const m = await loadManifest();
    // Every segment, fetched together and loaded in manifest order (docs/API.md: load_ephemeris once per segment).
    const segs = await Promise.all(m.ephemeris.map(fetchSegment));
    const engine = new wasm.Engine();
    m.ephemeris.forEach((seg, i) => engine.load_ephemeris(seg.t0_unix_s, seg.step_s, segs[i]));
    const far = await fetchTile(m.far);
    engine.load_far(far, m.far.width, m.far.height, m.far.x0_m, m.far.y0_m, m.far.pixel_m, m.far.scale_m, m.far.offset_m, m.far.nodata);
    return engine;
  })().catch((e) => { engineLoad = undefined; throw e; }));
}

/** Column order of the flat arrays from the WASM `sun_earth_track` and `timeline` calls (docs/API.md). */
export const TRACK_COLUMNS = ["t_unix_s", "sun_az_deg", "sun_el_deg", "sun_radius_deg", "sun_dist_km", "earth_az_deg", "earth_el_deg", "earth_radius_deg", "earth_dist_km"] as const;
export const TIMELINE_COLUMNS = ["t_unix_s", "sun_az_deg", "sun_el_deg", "illumination", "visibility", "earth_az_deg", "earth_el_deg", "earth_margin_deg", "contact", "sun_dist_km"] as const;

/** Sun and Earth az/el from the WASM Engine for one site and time grid (SCIENCE-SPEC 1c, degrees, Unix s UTC). */
export async function sunEarthTrack(site: Site, start_unix_s: number, step_s: number, count: number): Promise<Track> {
  const engine = await loadEngine();
  const flat = engine.sun_earth_track(site.lat_deg, site.lon_deg, site.ground_m, site.mast_m, start_unix_s, step_s, count);
  const track = Object.fromEntries(TRACK_COLUMNS.map((k, i) => [k, flat.slice(i * count, (i + 1) * count)])) as Track;
  validate(track);
  return track;
}

/** Sun and Earth az/el for the IM-2 tracer. */
export function loadTracerTrack(): Promise<Track> {
  const t = TRACER;
  return sunEarthTrack(t, t.start_unix_s, t.step_s, t.count);
}

import type { GridMeta } from "./grid";
import { edgeDistanceM } from "./site";
import { serialize } from "./serialize";
import { checkPit, DEGRADED_SLOPE_DEG } from "./replay";
import { latlonToXy } from "./projection";
// ---- Site picker and recompute (T11) ----
// The projection is TS (projection.ts), pinned to core::geometry by projection.test.ts (#58); the model calls above are the WASM Engine.
export { latlonToXy, xyToLatlon } from "./projection";

export type NearMeta = {
  width: number; height: number; x0_m: number; y0_m: number; pixel_m: number;
  scale_m: number; offset_m: number; nodata: number;
};
export type NearTile = { region: { id: string; name: string }; meta: NearMeta; data: Int16Array };
export type Inputs = { lat_deg: number; lon_deg: number; mast_m: number; start_unix_s: number; span_days: number; preset?: Preset };

export type Region = { id: string; name: string };
/** The near tile of one region (the first when no id is given); the int16 counts exactly as shipped. */
export async function loadNearTile(regionId?: string): Promise<NearTile & { regions: Region[]; eph: EphSegment[] }> {
  const m = await loadManifest();
  const region = regionId === undefined ? m.regions[0] : m.regions.find((r) => r.id === regionId);
  if (!region) throw new Error(`BAD_INPUT: unknown region ${regionId}`);
  const { url, ...meta } = region.near;
  return { region: { id: region.id, name: region.name }, regions: m.regions.map(({ id, name }) => ({ id, name })), eph: m.ephemeris, meta, data: await fetchTile(region.near) };
}

let nearId: string | undefined; // region now in the Engine
/** Loads the near tile of the region whose tile holds the site; OUT_OF_COVERAGE when none does. */
async function ensureNear(engine: WasmEngine, lat: number, lon: number): Promise<void> {
  const m = await loadManifest();
  const [x, y] = latlonToXy(lat, lon);
  const r = m.regions.find(({ near }) => edgeDistanceM(x, y, near) >= 0);
  if (!r) throw new Error("OUT_OF_COVERAGE: that site is outside every shipped near tile");
  const t = r.near, u = r.mid, [data, midData] = await Promise.all([fetchTile(t), fetchTile(u)]);
  if (nearId !== r.id) { engine.load_near(data, t.width, t.height, t.x0_m, t.y0_m, t.pixel_m, t.scale_m, t.offset_m, t.nodata); engine.load_mid(midData, u.width, u.height, u.x0_m, u.y0_m, u.pixel_m, u.scale_m, u.offset_m, u.nodata); nearId = r.id; }
}


/** Timeline (T08) and survival (T09) for one horizon in the 1440 layout; landing is the first step. NaN lethal_h = none in span. */
async function timelineAndSurvival(engine: WasmEngine, rawHorizon: Float64Array, site: Site, preset: Preset, t0: number, step_s: number, n: number) {
  const flat = engine.timeline(rawHorizon, site.lat_deg, site.lon_deg, site.ground_m, site.mast_m, t0, step_s, n, preset.earth_margin_deg);
  const timeline = Object.fromEntries(TIMELINE_COLUMNS.map((k, i) => [k, flat.slice(i * n, (i + 1) * n)])) as Timeline;
  validate(timeline);
  const wasm = await import("./wasm/penumbra_wasm.js");
  const pn = preset.panel === "tracking" ? [0, 0, 0, 0] : [1, preset.panel.east, preset.panel.north, preset.panel.up];
  const sv = wasm.survival(flat, Float64Array.of(preset.capacity_wh, preset.start_wh, preset.min_wh, preset.panel_area_m2, preset.efficiency, ...pn, preset.load_w, 0, 0), t0);
  const survival: Survival = { operable_h: sv[0], lethal_h: Number.isNaN(sv[1]) ? null : sv[1], longest_dark_h: sv[2],
    longest_any_sun_h: sv[3], sunlit_h: sv[4], contact_h: sv[5], soc_wh: sv.slice(8, 8 + sv[7]) };
  validate(survival);
  return { timeline, survival };
}

/**
 * Everything is the WASM Engine on the landing date and span at a 10-minute step: the panorama track, the
 * real DEM horizon (T07), Engine.timeline (T08) and the WASM `survival` (T09). Track and timelines share one grid, so the
 * cursor stays aligned.
 */
async function computeNow(inp: Inputs): Promise<Scenario> {
  const engine = await loadEngine();
  await ensureNear(engine, inp.lat_deg, inp.lon_deg);
  const ground_m = engine.ground_height(inp.lat_deg, inp.lon_deg);
  const site: Site = { lat_deg: inp.lat_deg, lon_deg: inp.lon_deg, ground_m, mast_m: inp.mast_m };
  const raw = engine.horizon(inp.lat_deg, inp.lon_deg, inp.mast_m);
  const horizon: Horizon = { el_deg: raw.slice(0, 720), dist_m: raw.slice(720, 1440) };
  validate(horizon);
  const preset = inp.preset ?? DEFAULT_PRESET, t0 = inp.start_unix_s, n = Math.round((inp.span_days * 86400) / STEP_S);
  if (!(n >= 1) || !Number.isFinite(t0)) throw new Error(`BAD_INPUT: span ${inp.span_days} d from ${t0} gives no time steps`);
  const track = await sunEarthTrack(site, t0, STEP_S, n);
  const { timeline, survival } = await timelineAndSurvival(engine, raw, site, preset, t0, STEP_S, n);
  return { api_version: await engineApiVersion(), step_s: STEP_S, start_unix_s: t0, count: n, preset, site, track, horizon, timeline, survival };
}

// The Engine holds one near tile at a time (ensureNear swaps it), so two computes must never interleave.
/** computeNow, one at a time: a second call waits for the first (the compare view runs A then B, and the site view may be mid-compute). */
export const computeScenario = serialize(computeNow);

// ---- IM-2 replay (T16 #17): the same site and times with the horizon raised to a pit rim ----
/**
 * A copy of `base` whose horizon is the WASM `pit_horizon` (SCIENCE-SPEC 1f) and whose timeline and survival are recomputed against it.
 * The panel height H is base.site.mast_m, measured from the pit floor. Throws BAD_INPUT for a refused pit.
 */
export async function computePit(base: Scenario, pit: { diameter_m: number; depth_m: number }): Promise<Scenario> {
  const bad = checkPit({ ...pit, panel_m: base.site.mast_m });
  if (bad) throw new Error(`BAD_INPUT: ${bad}`);
  const wasm = await import("./wasm/penumbra_wasm.js");
  await wasm.default();
  const regional = new Float64Array(1440);
  regional.set(base.horizon.el_deg); regional.set(base.horizon.dist_m, 720);
  const raw = wasm.pit_horizon(regional, pit.diameter_m, pit.depth_m, base.site.mast_m);
  const horizon: Horizon = { el_deg: raw.slice(0, 720), dist_m: raw.slice(720, 1440) };
  validate(horizon);
  const engine = await loadEngine();
  const { timeline, survival } = await timelineAndSurvival(engine, raw, base.site, base.preset, base.start_unix_s, base.step_s, base.count);
  return { ...base, horizon, timeline, survival };
}

/** d_c* in metres: the pit depth that hides the whole solar disc from a panel `panel_m` above the floor (V4 statement 3). WASM. */
export async function pitDepthToHide(diameter_m: number, panel_m: number, sun_el_deg: number, sun_radius_deg: number): Promise<number> {
  const wasm = await import("./wasm/penumbra_wasm.js");
  await wasm.default();
  return wasm.pit_depth_to_hide(diameter_m, panel_m, sun_el_deg, sun_radius_deg);
}

// The model refuses a Sun radius of 0, so the point source has a radius far below any depth tolerance (1e-9 deg moves 10 m by 2e-10 m).
const TINY_SUN_DEG = 1e-9;
/** The ESA degraded-crater depth (D/2)·tan 11° (E112) from the WASM: the depth that hides a (near) point source sitting 11° up, from the floor (H = 0).
 *  shortcut: stands in for the unexported `degraded_crater_depth_m` (crates/core/src/pit.rs:29) until the WASM exports it (both-owners follow-up, api_version bump). */
export const defaultPitDepth = (diameter_m: number) => pitDepthToHide(diameter_m, 0, DEGRADED_SLOPE_DEG, TINY_SUN_DEG);

/** Sunrise and sunset ± bands (SCIENCE-SPEC 1f step 1) for a scenario's timeline against its horizon: the WASM `window_bands`, flat 3 × k. Throws BAD_INPUT. */
export async function windowBands(s: Scenario, sigma_z_m: number): Promise<Float64Array> {
  const wasm = await import("./wasm/penumbra_wasm.js");
  await wasm.default();
  const flat = new Float64Array(1440);
  flat.set(s.horizon.el_deg); flat.set(s.horizon.dist_m, 720);
  const tl = new Float64Array(10 * s.count);
  TIMELINE_COLUMNS.forEach((k, i) => tl.set(s.timeline[k], i * s.count));
  return wasm.window_bands(tl, flat, sigma_z_m);
}

// ---- Point-Sun mask (T22): an Engine with only the ephemeris and one near tile, for the mask Web Worker ----
const maskEngines = new Map<string, Promise<{ engine: WasmEngine; meta: NearMeta }>>();
/** A WASM Engine holding the ephemeris and the near tile of `regionId` (no far tile: the mask marches only the near tile). One per region, cached. */
export function loadMaskEngine(regionId: string): Promise<{ engine: WasmEngine; meta: NearMeta }> {
  let p = maskEngines.get(regionId);
  if (!p) {
    p = (async () => {
      const wasm = await import("./wasm/penumbra_wasm.js");
      await wasm.default();
      const m = await loadManifest();
      const region = m.regions.find((r) => r.id === regionId);
      if (!region) throw new Error(`BAD_INPUT: unknown region ${regionId}`);
      const [segs, data] = await Promise.all([Promise.all(m.ephemeris.map(fetchSegment)), fetchTile(region.near)]);
      const engine = new wasm.Engine();
      m.ephemeris.forEach((seg, i) => engine.load_ephemeris(seg.t0_unix_s, seg.step_s, segs[i]));
      const t = region.near;
      engine.load_near(data, t.width, t.height, t.x0_m, t.y0_m, t.pixel_m, t.scale_m, t.offset_m, t.nodata);
      const { url, ...meta } = t;
      return { engine, meta };
    })();
    maskEngines.set(regionId, p);
    p.catch(() => maskEngines.delete(regionId));
  }
  return p;
}
