// The only module that imports lib/wasm/ or reads the fixture (ARCHITECTURE §3.5).
// Typed arrays use platform byte order; every browser target is little-endian, so Int16Array over a file is safe.
// sun_earth_track (T02), horizon (T07), timeline (T08) and survival (T09) are WASM; the fixture still supplies the time grid and preset defaults.
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
export type ScoreEntry = {
  id: string; judge: string; metric: string; threshold: number | null; value: number | null;
  pass: boolean | null; sample_size: number | null; notes: string; commit: string | null; date: string | null;
};
export type Scenario = {
  fake: boolean; note: string; api_version: number; site: Site; start_unix_s: number; step_s: number;
  count: number; preset: Preset; horizon: Horizon; track: Track; timeline: Timeline; survival: Survival;
  scoreboard: ScoreEntry[];
};

/** Throws when a fixture or manifest was built for a different API than this engine. */
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

/** Turns parsed scenario JSON into typed arrays and checks it. `engineVersion` is what api_version() returns. */
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

export async function loadFixture(url = `${import.meta.env?.BASE_URL ?? "/"}fixture/scenario.json`): Promise<Scenario> {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`NO_DATA: ${url} returned ${res.status}`);
  return parseScenario(await res.json(), await engineApiVersion());
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
export type Manifest = {
  format_version: number; api_version: number;
  ephemeris: { id: string; url: string; t0_unix_s: number; step_s: number; rows: number }[];
  far: TileMeta; regions: { id: string; name: string; near: TileMeta }[];
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

type WasmEngine = InstanceType<typeof import("./wasm/penumbra_wasm.js").Engine>;
let engineLoad: Promise<WasmEngine> | undefined;
/** The WASM Engine with the shipped ephemeris table loaded; built once and cached (a failed load is retried). */
export function loadEngine(): Promise<WasmEngine> {
  return (engineLoad ??= (async () => {
    const wasm = await import("./wasm/penumbra_wasm.js");
    await wasm.default();
    const m = await loadManifest();
    const seg = m.ephemeris[0];
    const res = await fetch(`${import.meta.env?.BASE_URL ?? "/"}${seg.url}`);
    if (!res.ok) throw new Error(`NO_DATA: ${seg.url} returned ${res.status}`);
    const buf = await res.arrayBuffer();
    if (buf.byteLength % 8) throw new Error(`BAD_INPUT: ${seg.url} is ${buf.byteLength} bytes, not whole f64s`);
    const data = new Float64Array(buf);
    if (data.length !== 6 * seg.rows) throw new Error(`BAD_INPUT: ${seg.url} has ${data.length} values, expected ${6 * seg.rows}`);
    const engine = new wasm.Engine();
    engine.load_ephemeris(seg.t0_unix_s, seg.step_s, data);
    const far = await fetchTile(m.far);
    engine.load_far(far, m.far.width, m.far.height, m.far.x0_m, m.far.y0_m, m.far.pixel_m, m.far.scale_m, m.far.offset_m, m.far.nodata);
    return engine;
  })().catch((e) => { engineLoad = undefined; throw e; }));
}

/** Sun and Earth az/el from the WASM Engine for one site and time grid (SCIENCE-SPEC 1c, degrees, Unix s UTC). */
export async function sunEarthTrack(site: Site, start_unix_s: number, step_s: number, count: number): Promise<Track> {
  const engine = await loadEngine();
  const flat = engine.sun_earth_track(site.lat_deg, site.lon_deg, site.ground_m, site.mast_m, start_unix_s, step_s, count);
  const names = ["t_unix_s", "sun_az_deg", "sun_el_deg", "sun_radius_deg", "sun_dist_km", "earth_az_deg", "earth_el_deg", "earth_radius_deg", "earth_dist_km"];
  const track = Object.fromEntries(names.map((k, i) => [k, flat.slice(i * count, (i + 1) * count)])) as Track;
  validate(track);
  return track;
}

/** Sun and Earth az/el for the IM-2 tracer. */
export function loadTracerTrack(): Promise<Track> {
  const t = TRACER;
  return sunEarthTrack(t, t.start_unix_s, t.step_s, t.count);
}

import { edgeDistanceM } from "./site";
import { latlonToXy } from "./fixture-projection";
// ---- Site picker and recompute (T11) ----
// The projection is still a fixture stand-in until T01 #2 merges; the model calls above are the WASM Engine.
export { latlonToXy, xyToLatlon } from "./fixture-projection";

export type NearMeta = {
  width: number; height: number; x0_m: number; y0_m: number; pixel_m: number;
  scale_m: number; offset_m: number; nodata: number;
};
export type NearTile = { fake: boolean; region: { id: string; name: string }; meta: NearMeta; data: Int16Array };
export type Inputs = { lat_deg: number; lon_deg: number; mast_m: number; start_unix_s: number; span_days: number; preset?: Preset };

export type Region = { id: string; name: string };
/** The near tile of one region (the first when no id is given); the int16 counts exactly as shipped. */
export async function loadNearTile(regionId?: string): Promise<NearTile & { regions: Region[] }> {
  const m = await loadManifest();
  const region = regionId === undefined ? m.regions[0] : m.regions.find((r) => r.id === regionId);
  if (!region) throw new Error(`BAD_INPUT: unknown region ${regionId}`);
  const { url, ...meta } = region.near;
  return { fake: false, region: { id: region.id, name: region.name }, regions: m.regions.map(({ id, name }) => ({ id, name })), meta, data: await fetchTile(region.near) };
}

let nearId: string | undefined; // region now in the Engine
/** Loads the near tile of the region whose tile holds the site; OUT_OF_COVERAGE when none does. */
async function ensureNear(engine: WasmEngine, lat: number, lon: number): Promise<void> {
  const m = await loadManifest();
  const [x, y] = latlonToXy(lat, lon);
  const r = m.regions.find(({ near }) => edgeDistanceM(x, y, near) >= 0);
  if (!r) throw new Error("OUT_OF_COVERAGE: that site is outside every shipped near tile");
  const t = r.near, data = await fetchTile(t);
  if (nearId !== r.id) { engine.load_near(data, t.width, t.height, t.x0_m, t.y0_m, t.pixel_m, t.scale_m, t.offset_m, t.nodata); nearId = r.id; }
}

/** The panorama's horizon terrain is the real DEM since T07 (#8); the switch stays. Kept apart from `scenario.fake`,
 *  which T08/T09 turn off for the timelines and survival while the terrain may still be fake. */
export const TERRAIN_FAKE = false;
/** Survival (headline, charge, lethal time) is the WASM `survival` since T09 (#10); same pattern as TERRAIN_FAKE. */
export const SURVIVAL_FAKE = false;

let fixtureBase: Promise<Scenario> | undefined;
/**
 * Sun and Earth tracks (the panorama's) come from the WASM Engine for the chosen site, on the fixture timeline's own
 * time grid so the shared cursor stays aligned with the timelines. Horizon is the real DEM (T07) and the timelines are Engine.timeline (T08) and survival is
 * the WASM `survival` (T09). `fake` stays true while the grid is the fixture's dates; the panorama badge reads TERRAIN_FAKE instead.
 */
export async function computeScenario(inp: Inputs): Promise<Scenario> {
  const s = await (fixtureBase ??= loadFixture());
  const engine = await loadEngine();
  await ensureNear(engine, inp.lat_deg, inp.lon_deg);
  const ground_m = engine.ground_height(inp.lat_deg, inp.lon_deg);
  const site = { ...s.site, lat_deg: inp.lat_deg, lon_deg: inp.lon_deg, ground_m, mast_m: inp.mast_m };
  const raw = engine.horizon(inp.lat_deg, inp.lon_deg, inp.mast_m);
  const horizon: Horizon = { el_deg: raw.slice(0, 720), dist_m: raw.slice(720, 1440) };
  validate(horizon);
  const preset = inp.preset ?? s.preset, t0 = s.timeline.t_unix_s[0], n = s.count;
  const track = await sunEarthTrack(site, t0, s.step_s, n);
  // Timelines from the WASM Engine (T08 #9), on the same grid and against the same horizon.
  const flat = engine.timeline(raw, site.lat_deg, site.lon_deg, ground_m, inp.mast_m, t0, s.step_s, n, preset.earth_margin_deg);
  const names = ["t_unix_s", "sun_az_deg", "sun_el_deg", "illumination", "visibility", "earth_az_deg", "earth_el_deg", "earth_margin_deg", "contact", "sun_dist_km"];
  const timeline = Object.fromEntries(names.map((k, i) => [k, flat.slice(i * n, (i + 1) * n)])) as Timeline;
  validate(timeline);
  // Survival from the WASM free function (T09 #10) on that timeline; landing is its first step. NaN lethal_h = none in span.
  const wasm = await import("./wasm/penumbra_wasm.js");
  const pn = preset.panel === "tracking" ? [0, 0, 0, 0] : [1, preset.panel.east, preset.panel.north, preset.panel.up];
  const sv = wasm.survival(flat, Float64Array.of(preset.capacity_wh, preset.start_wh, preset.min_wh, preset.panel_area_m2, preset.efficiency, ...pn, preset.load_w, 0, 0), t0);
  const survival: Survival = { operable_h: sv[0], lethal_h: Number.isNaN(sv[1]) ? null : sv[1], longest_dark_h: sv[2],
    longest_any_sun_h: sv[3], sunlit_h: sv[4], contact_h: sv[5], soc_wh: sv.slice(8, 8 + sv[7]) };
  validate(survival);
  return { ...s, preset, site, track, horizon, timeline, survival };
}
