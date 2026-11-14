import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import init, { api_version, survival } from "./wasm/penumbra_wasm.js";
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
    const inp = { lat_deg: -84.79, lon_deg: 29.2, mast_m: 2, start_unix_s: 1794614400, span_days: 30 };
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
    const s = await computeScenario({ lat_deg: -84.79, lon_deg: 29.2, mast_m: 2, start_unix_s: 1794614400, span_days: 30, preset });
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
    const inp = { lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 3, start_unix_s: 1794614400, span_days: 30 };
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
    const s = await computeScenario({ lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 2, start_unix_s: 1794614400, span_days: 30 });
    expect(s.site.ground_m).toBeCloseTo(5321.92, 1);
    const el = Array.from(s.horizon.el_deg);
    expect(Math.max(...el)).toBeCloseTo(4.04, 1);
    expect(el.indexOf(Math.max(...el)) * 0.5).toBe(117.5);
    expect(Math.min(...el)).toBeCloseTo(-1.475, 1);
    await expect(computeScenario({ lat_deg: -84.7906, lon_deg: 10, mast_m: 2, start_unix_s: 1794614400, span_days: 30 })).rejects.toThrow("OUT_OF_COVERAGE");
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

test("computeScenario timelines are Engine.timeline on the real IM-2 horizon, not the fixture (T08 #9)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const { loadEngine } = await import("./engine");
    const s = await computeScenario({ lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 2, start_unix_s: 1794614400, span_days: 30 });
    const engine = await loadEngine();
    const n = s.count, t0 = s.timeline.t_unix_s[0];
    const hz = Float64Array.from([...s.horizon.el_deg, ...s.horizon.dist_m]);
    const flat = engine.timeline(hz, s.site.lat_deg, s.site.lon_deg, s.site.ground_m, s.site.mast_m, t0, s.step_s, n, s.preset.earth_margin_deg);
    const names = ["t_unix_s", "sun_az_deg", "sun_el_deg", "illumination", "visibility", "earth_az_deg", "earth_el_deg", "earth_margin_deg", "contact", "sun_dist_km"] as const;
    for (const i of [0, 1234, n - 1]) for (const [c, k] of names.entries()) expect(s.timeline[k][i]).toBe(flat[c * n + i]);
    for (let i = 0; i < n; i++) expect(s.timeline.visibility[i]).toBeGreaterThanOrEqual(s.timeline.illumination[i]); // P1
    expect(s.timeline.illumination).not.toEqual(parseScenario(raw, api_version()).timeline.illumination);
  } finally {
    globalThis.fetch = realFetch;
  }
});

// A flat 10 × n timeline (docs/API.md): every 10 min from 2026-11-14Z, Sun at az 90°, el 0°, 1 AU, contact on.
const flatTimeline = (n: number, illum: number) => {
  const cols = [Array.from({ length: n }, (_, i) => 1794614400 + 600 * i), Array(n).fill(90), Array(n).fill(0),
    Array(n).fill(illum), Array(n).fill(illum > 0 ? 1 : 0), Array(n).fill(0), Array(n).fill(5), Array(n).fill(5),
    Array(n).fill(1), Array(n).fill(149597870.7)];
  return Float64Array.from(cols.flat());
};
// capacity, start, min, area, η, mode, east, north, up, load, 0, 0
const presetArr = (mode: number, e: number, nn: number, u: number, load: number) => Float64Array.of(10000, 705, 0, 1, 1, mode, e, nn, u, load, 0, 0);

test("WASM survival: T-1e-1 header, soc length = header n, NaN when no lethal shadow (T09 #10)", () => {
  const dead = survival(flatTimeline(60, 0), presetArr(0, 0, 0, 0, 100), 1794614400);
  expect(Math.abs(dead[1] - 7.05)).toBeLessThan(1e-9); // SCIENCE-SPEC T-1e-1
  expect(dead[7]).toBe(43);
  expect(dead.length).toBe(8 + 43);
  const alive = survival(flatTimeline(60, 1), presetArr(0, 0, 0, 0, 100), 1794614400);
  expect(Number.isNaN(alive[1])).toBe(true);
  expect(alive[7]).toBe(60);
});

test("WASM survival reads the fixed-panel normal from slots 6–8 (T09 #10)", () => {
  // Sun at az 90°, el 0°. Panel facing east (1, 0, 0): n̂·ŝ = 1, 1366.1 W for 1/6 h on 705 Wh, no load.
  const east = survival(flatTimeline(2, 1), presetArr(1, 1, 0, 0, 0), 1794614400);
  expect(east[9]).toBeCloseTo(705 + 1366.1 / 6, 9);
  // Facing up (0, 0, 1): n̂·ŝ = sin 0° = 0, no charge.
  expect(survival(flatTimeline(2, 1), presetArr(1, 0, 0, 1, 0), 1794614400)[9]).toBe(705);
  expect(() => survival(flatTimeline(2, 1), presetArr(2, 0, 0, 1, 0), 1794614400)).toThrow("BAD_INPUT");
});

test("computeScenario survival maps the WASM header: tracking vs fixed, lethal_h null vs number (T09 #10)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const base = parseScenario(raw, api_version());
    const inp = { lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 2, start_unix_s: 1794614400, span_days: 30 };
    const s = await computeScenario({ ...inp, preset: base.preset });
    const flat = Float64Array.from(["t_unix_s", "sun_az_deg", "sun_el_deg", "illumination", "visibility", "earth_az_deg", "earth_el_deg", "earth_margin_deg", "contact", "sun_dist_km"]
      .flatMap((k) => Array.from(s.timeline[k as keyof typeof s.timeline])));
    const p = base.preset, t0 = s.timeline.t_unix_s[0];
    const want = survival(flat, Float64Array.of(p.capacity_wh, p.start_wh, p.min_wh, p.panel_area_m2, p.efficiency, 0, 0, 0, 0, p.load_w, 0, 0), t0);
    expect(s.survival.operable_h).toBe(want[0]);
    expect(s.survival.lethal_h).toBe(want[1]); // the cited preset dies in the IM-2 span
    expect(s.survival.soc_wh.length).toBe(want[7]);
    const fixed = await computeScenario({ ...inp, preset: { ...p, panel: { east: 0, north: 1, up: 0 }, start_wh: 1000, load_w: 0 } });
    const wantFixed = survival(flat, Float64Array.of(p.capacity_wh, 1000, p.min_wh, p.panel_area_m2, p.efficiency, 1, 0, 1, 0, 0, 0, 0), t0);
    const asTracking = survival(flat, Float64Array.of(p.capacity_wh, 1000, p.min_wh, p.panel_area_m2, p.efficiency, 0, 0, 0, 0, 0, 0, 0), t0);
    expect(wantFixed.slice(8)).not.toEqual(asTracking.slice(8)); // the layouts really differ
    expect(fixed.survival.lethal_h).toBeNull(); // no load: no lethal shadow
    expect(fixed.survival.soc_wh).toEqual(wantFixed.slice(8));
    expect(fixed.survival.soc_wh.length).toBe(s.count);
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("computeScenario runs on the landing date and span, not the fixture grid (#13)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const start = Date.UTC(2027, 2, 1) / 1000; // 2027-03-01: inside the table, not the fixture's 2026-11-14
    const s = await computeScenario({ lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 2, start_unix_s: start, span_days: 14 });
    expect(s.fake).toBe(false);
    expect(s.start_unix_s).toBe(start);
    expect(s.count).toBe(14 * 144); // 10-minute steps
    expect(s.track.t_unix_s[0]).toBe(start);
    expect(s.track.t_unix_s.length).toBe(s.count);
    expect(s.timeline.t_unix_s[s.count - 1]).toBe(start + (s.count - 1) * 600);
    // The values are the Engine's for 2027-03-01, not the fixture's, at both ends of the grid.
    const { loadEngine } = await import("./engine");
    const engine = await loadEngine();
    const n = s.count, hz = Float64Array.from([...s.horizon.el_deg, ...s.horizon.dist_m]);
    const flat = engine.timeline(hz, s.site.lat_deg, s.site.lon_deg, s.site.ground_m, s.site.mast_m, start, 600, n, s.preset.earth_margin_deg);
    for (const i of [0, n - 1]) {
      expect(s.timeline.sun_az_deg[i]).toBe(flat[n + i]);
      expect(s.timeline.illumination[i]).toBe(flat[3 * n + i]);
    }
    expect(s.track.sun_az_deg[0]).not.toBe(parseScenario(raw, api_version()).track.sun_az_deg[0]);
    // Survival is the WASM result on that same 2027 timeline, with landing at its first step (T09).
    const p = s.preset;
    const want = survival(flat, Float64Array.of(p.capacity_wh, p.start_wh, p.min_wh, p.panel_area_m2, p.efficiency, 0, 0, 0, 0, p.load_w, 0, 0), start);
    expect(s.survival.lethal_h).toBe(Number.isNaN(want[1]) ? null : want[1]);
    expect(s.survival.soc_wh).toEqual(want.slice(8, 8 + want[7]));
    const site = { lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 2 };
    await expect(computeScenario({ ...site, start_unix_s: Date.UTC(2025, 2, 6) / 1000, span_days: 14 })).rejects.toThrow("OUT_OF_TABLE"); // before the table
    await expect(computeScenario({ ...site, start_unix_s: Date.UTC(2030, 11, 25) / 1000, span_days: 14 })).rejects.toThrow("OUT_OF_TABLE"); // runs past its end (P16)
    for (const span_days of [0, -1, NaN]) await expect(computeScenario({ ...site, start_unix_s: start, span_days })).rejects.toThrow("BAD_INPUT");
  } finally {
    globalThis.fetch = realFetch;
  }
});
