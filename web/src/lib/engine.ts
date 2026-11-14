// The only module that imports lib/wasm/ or reads the fixture (ARCHITECTURE §3.5).
// Typed arrays use platform byte order; every browser target is little-endian, so Int16Array over a file is safe.
// Stub: every function is still on the fixture except sun_earth_track (WASM, T02). Flip each when its science ticket merges.
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

export type Manifest = {
  format_version: number; api_version: number;
  ephemeris: { id: string; url: string; t0_unix_s: number; step_s: number; rows: number }[];
};

export async function loadManifest(): Promise<Manifest> {
  const base = import.meta.env?.BASE_URL ?? "/";
  const res = await fetch(`${base}data/manifest.json`, { cache: "no-cache" });
  if (!res.ok) throw new Error(`NO_DATA: manifest.json returned ${res.status}`);
  const m: Manifest = await res.json();
  checkApiVersion(m.api_version, await engineApiVersion());
  return m;
}

// IM-2 tracer call (T02): site, mast, 2026-11-14T00:00Z, hourly, 24 rows.
export const TRACER = { lat_deg: -84.7906, lon_deg: 29.1957, ground_m: 0, mast_m: 2, start_unix_s: 1794614400, step_s: 3600, count: 24 };

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

// ---- Site picker and recompute (T11) ----
// Fixture stand-ins until T01 #2 (projection) and the science tickets (model) merge.
export { latlonToXy, xyToLatlon } from "./fixture-projection";

export type NearMeta = {
  width: number; height: number; x0_m: number; y0_m: number; pixel_m: number;
  scale_m: number; offset_m: number; nodata: number;
};
export type NearTile = { fake: boolean; region: { id: string; name: string }; meta: NearMeta; data: Int16Array };
export type Inputs = { lat_deg: number; lon_deg: number; mast_m: number; start_unix_s: number; span_days: number; preset?: Preset };

/** The near tile of the first region in the manifest; the int16 counts exactly as shipped. */
export async function loadNearTile(root = import.meta.env?.BASE_URL ?? "/"): Promise<NearTile> {
  const res = await fetch(`${root}fixture/manifest.json`, { cache: "no-cache" });
  if (!res.ok) throw new Error(`NO_DATA: manifest returned ${res.status}`);
  const m = await res.json();
  checkApiVersion(m.api_version, await engineApiVersion());
  const region = m.regions[0], { url, ...meta } = region.near;
  const tile = await fetch(`${root}${url}`);
  if (!tile.ok) throw new Error(`NO_DATA: ${url} returned ${tile.status}`);
  const data = new Int16Array(await tile.arrayBuffer());
  if (data.length !== meta.width * meta.height) throw new Error(`BAD_INPUT: ${url} has ${data.length} values, expected ${meta.width * meta.height}`);
  return { fake: m.fake === true, region: { id: region.id, name: region.name }, meta, data };
}

/** The panorama's horizon terrain is still the fixture; T07 (#8) sets this false. Kept apart from `scenario.fake`,
 *  which T08/T09 turn off for the timelines and survival while the terrain may still be fake. */
export const TERRAIN_FAKE = true;

let fixtureBase: Promise<Scenario> | undefined;
/**
 * Sun and Earth tracks (the panorama's) come from the WASM Engine for the chosen site, on the fixture timeline's own
 * time grid so the shared cursor stays aligned with the timelines. Horizon, timelines and survival are still fixture
 * (T07/T08/T09), so `fake` stays true; the panorama badge reads TERRAIN_FAKE instead.
 */
export async function computeScenario(inp: Inputs): Promise<Scenario> {
  const s = await (fixtureBase ??= loadFixture());
  // shortcut: ground_m stays the fixture's 0 until a DEM height lookup lands; a few km of ground height moves Earth's
  // topocentric position by about 0.0003° per km (Moon–Earth 384,000 km) and the Sun far less, below the panorama's 0.1°.
  const site = { ...s.site, lat_deg: inp.lat_deg, lon_deg: inp.lon_deg, mast_m: inp.mast_m };
  const track = await sunEarthTrack(site, s.timeline.t_unix_s[0], s.step_s, s.count);
  return { ...s, preset: inp.preset ?? s.preset, site, track };
}
