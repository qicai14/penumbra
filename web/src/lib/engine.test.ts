import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import init, { api_version } from "./wasm/penumbra_wasm.js";
import { checkApiVersion, computeScenario, parseScenario, validate } from "./engine";

const raw = JSON.parse(readFileSync(new URL("../../public/fixture/scenario.json", import.meta.url), "utf8"));
await init({ module_or_path: readFileSync(new URL("./wasm/penumbra_wasm_bg.wasm", import.meta.url)) });
// computeScenario now loads the manifest and ephemeris table, so fetch serves the real files under public/.
const servePublic = (async (u: string) =>
  new Response(readFileSync(new URL(`../../public/${String(u).replace(/^\//, "")}`, import.meta.url)))) as unknown as typeof fetch;

test("fixture matches the engine api_version and validates", () => {
  const s = parseScenario(raw, api_version());
  expect(s.fake).toBe(true);
  expect(s.timeline.t_unix_s.length).toBe(4320);
  expect(s.scoreboard.map((e) => e.id)).toEqual(["V1", "V2", "V3", "V4", "V5"]);
});

test("a mismatched api_version is refused", () => {
  expect(() => parseScenario({ ...raw, api_version: 2 }, api_version())).toThrow("api_version");
  expect(() => checkApiVersion(2, 1)).toThrow();
});

test("validate rejects bad data", () => {
  const s = parseScenario(raw, api_version());
  s.timeline.sun_az_deg[5] = 360;
  expect(() => validate(s.timeline)).toThrow("sun_az_deg");
  s.horizon.el_deg[3] = 200;
  expect(() => validate(s.horizon)).toThrow("el_deg");
});

test("parseScenario rejects null cells and short columns", () => {
  const nulls = structuredClone(raw);
  nulls.timeline.illumination[0] = null;
  expect(() => parseScenario(nulls, api_version())).toThrow("illumination");
  const short = structuredClone(raw);
  for (const k of Object.keys(short.timeline)) short.timeline[k] = short.timeline[k].slice(0, 100);
  expect(() => parseScenario(short, api_version())).toThrow("count");
});

// Must run before any test that loads the Engine successfully: the loaded Engine is cached for the module's life.
test("a failed ephemeris load is retried on the next recompute, not cached (T03 #4)", async () => {
  const realFetch = globalThis.fetch;
  let failed = false;
  globalThis.fetch = (async (u: string) => {
    if (!failed && !/scenario\.json|manifest\.json/.test(String(u))) { failed = true; return new Response("", { status: 500 }); }
    return servePublic(u);
  }) as unknown as typeof fetch;
  try {
    const inp = { lat_deg: -84.79, lon_deg: 29.2, mast_m: 2, start_unix_s: 0, span_days: 30 };
    await expect(computeScenario(inp)).rejects.toThrow("NO_DATA");
    expect(failed).toBe(true);
    expect((await computeScenario(inp)).track.t_unix_s.length).toBe(4320);
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("an edited preset reaches computeScenario (T14 recompute wiring)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const base = parseScenario(raw, api_version());
    const preset = { ...base.preset, load_w: 300 };
    const s = await computeScenario({ lat_deg: -84.79, lon_deg: 29.2, mast_m: 2, start_unix_s: 0, span_days: 30, preset });
    expect(s.preset).toBe(preset);
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("panorama track equals Engine.sun_earth_track on the fixture grid (T03 #4)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const { loadEngine } = await import("./engine");
    const inp = { lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 3, start_unix_s: 0, span_days: 30 };
    const s = await computeScenario(inp);
    expect(s.site.mast_m).toBe(3);
    const engine = await loadEngine();
    const n = s.count, t0 = s.timeline.t_unix_s[0];
    const flat = engine.sun_earth_track(inp.lat_deg, inp.lon_deg, s.site.ground_m, inp.mast_m, t0, s.step_s, n);
    const names = ["t_unix_s", "sun_az_deg", "sun_el_deg", "sun_radius_deg", "sun_dist_km", "earth_az_deg", "earth_el_deg", "earth_radius_deg", "earth_dist_km"] as const;
    for (const i of [0, 1234, n - 1]) for (const [c, k] of names.entries()) expect(s.track[k][i]).toBe(flat[c * n + i]);
    expect(s.track.t_unix_s[1234]).toBe(s.timeline.t_unix_s[1234]); // one shared time grid
    expect(s.track.sun_az_deg[0]).not.toBe(parseScenario(raw, api_version()).track.sun_az_deg[0]); // not the fixture
    expect(await loadEngine()).toBe(engine); // cached
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("computeScenario returns the real IM-2 horizon and ground height; outside the tiles is OUT_OF_COVERAGE (T07 #8)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const s = await computeScenario({ lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 2, start_unix_s: 0, span_days: 30 });
    expect(s.site.ground_m).toBeCloseTo(5321.92, 1);
    const el = Array.from(s.horizon.el_deg);
    expect(Math.max(...el)).toBeCloseTo(4.04, 1);
    expect(el.indexOf(Math.max(...el)) * 0.5).toBe(117.5);
    expect(Math.min(...el)).toBeCloseTo(-1.475, 1);
    await expect(computeScenario({ lat_deg: -84.7906, lon_deg: 10, mast_m: 2, start_unix_s: 0, span_days: 30 })).rejects.toThrow("OUT_OF_COVERAGE");
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("fixture projection puts IM-2 at about (77.11, 137.99) km (DATA-NOTES), guarding drift until T01", async () => {
  const { latlonToXy } = await import("./fixture-projection");
  const [x, y] = latlonToXy(-84.7906, 29.1957);
  expect(x / 1000).toBeCloseTo(77.11, 1);
  expect(y / 1000).toBeCloseTo(137.99, 1);
});

test("an unknown region id is refused", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const { loadNearTile } = await import("./engine");
    await expect(loadNearTile("nowhere")).rejects.toThrow("BAD_INPUT");
  } finally {
    globalThis.fetch = realFetch;
  }
});
