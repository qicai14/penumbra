import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import init, { api_version, survival } from "./wasm/penumbra_wasm.js";
import { checkApiVersion, computeScenario, DEFAULT_PRESET, parseScenario, resetEngineForTests, TIMELINE_COLUMNS, TRACK_COLUMNS, validate } from "./engine";

const raw = JSON.parse(readFileSync(new URL("../../tests/fixture/scenario.json", import.meta.url), "utf8"));
await init({ module_or_path: readFileSync(new URL("./wasm/penumbra_wasm_bg.wasm", import.meta.url)) });
// computeScenario now loads the manifest and ephemeris table, so fetch serves the real files under public/.
const servePublic = (async (u: string) =>
  new Response(readFileSync(new URL(`../../public/${String(u).replace(/^\//, "")}`, import.meta.url)))) as unknown as typeof fetch;

test("fixture matches the engine api_version and validates", () => {
  const s = parseScenario(raw, api_version());
  expect((s as { fake?: boolean }).fake).toBe(true);
  expect(s.timeline.t_unix_s.length).toBe(4320);
});

test("a mismatched api_version is refused", () => {
  expect(() => parseScenario({ ...raw, api_version: 99 }, api_version())).toThrow("api_version");
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

// T16: loader error paths. The loaded Engine is cached for the process, and another test file (grid.equality) may have
// loaded it first in CI's file order, so each load-failure test drops the cache before it starts.
const eph = (reply: () => Response) => (async (u: string) => (/eph-/.test(String(u)) ? reply() : servePublic(u))) as unknown as typeof fetch;
const inp16 = { lat_deg: -84.79, lon_deg: 29.2, mast_m: 2, start_unix_s: 1794614400, span_days: 30 };
test("a 404 on an ephemeris segment is NO_DATA (T16)", async () => {
  resetEngineForTests();
  const realFetch = globalThis.fetch;
  globalThis.fetch = eph(() => new Response("", { status: 404 }));
  try { await expect(computeScenario(inp16)).rejects.toThrow("NO_DATA"); } finally { globalThis.fetch = realFetch; }
});
test("a segment whose size is not a whole number of f64s, or has the wrong row count, is BAD_INPUT (T16)", async () => {
  resetEngineForTests();
  const realFetch = globalThis.fetch;
  try {
    globalThis.fetch = eph(() => new Response(new Uint8Array(13)));
    await expect(computeScenario(inp16)).rejects.toThrow("BAD_INPUT");
    globalThis.fetch = eph(() => new Response(new Float64Array(48)));
    await expect(computeScenario(inp16)).rejects.toThrow("BAD_INPUT");
  } finally { globalThis.fetch = realFetch; }
});

test("a failed ephemeris load is retried on the next recompute, not cached (T03 #4)", async () => {
  resetEngineForTests();
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
    const names = TRACK_COLUMNS;
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
    const names = TIMELINE_COLUMNS;
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
    const flat = Float64Array.from(TIMELINE_COLUMNS
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
    expect("fake" in s).toBe(false); // nor its FAKE flag: only named fields are copied
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
    await expect(computeScenario({ ...site, start_unix_s: Date.UTC(2025, 0, 1) / 1000, span_days: 14 })).rejects.toThrow("OUT_OF_TABLE"); // before every segment (2025-03-06 is in the replay-im2 segment since T16)
    await expect(computeScenario({ ...site, start_unix_s: Date.UTC(2030, 11, 25) / 1000, span_days: 14 })).rejects.toThrow("OUT_OF_TABLE"); // runs past its end (P16)
    await expect(computeScenario({ ...site, start_unix_s: Date.UTC(2025, 5, 1) / 1000, span_days: 14 })).rejects.toThrow("OUT_OF_TABLE"); // the gap between replay-im2 and main
    await expect(computeScenario({ ...site, start_unix_s: Date.UTC(2025, 3, 5) / 1000, span_days: 14 })).rejects.toThrow("OUT_OF_TABLE"); // runs past the end of replay-im2
    for (const span_days of [0, -1, NaN]) await expect(computeScenario({ ...site, start_unix_s: start, span_days })).rejects.toThrow("BAD_INPUT");
  } finally {
    globalThis.fetch = realFetch;
  }
});

// ---- T16 (#17): every ephemeris segment is loaded, so IM-2 dates pass; the pit answer is the WASM pit_horizon ----
const IM2_TOUCHDOWN = 1741282080; // 2025-03-06T17:28:00Z, assumed (U8)

test("every manifest segment is loaded: IM-2 touchdown (replay-im2 segment) computes (T16)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const s = await computeScenario({ lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 1, start_unix_s: IM2_TOUCHDOWN, span_days: 3 });
    expect(s.track.t_unix_s[0]).toBe(IM2_TOUCHDOWN);
    expect(s.track.sun_el_deg[0]).toBeLessThan(6.8); // E104: the Sun never rises above 6.75 deg here
    expect(s.track.sun_el_deg[0]).toBeGreaterThan(-1);
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("loadNearTile offers both ephemeris segments to the date check (T16)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const { loadNearTile } = await import("./engine");
    expect((await loadNearTile()).eph.map((e) => e.id)).toEqual(["main", "replay-im2"]);
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("computePit raises the horizon to the pit rim and can only lower illumination (T16)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = servePublic;
  try {
    const { computePit, defaultPitDepth } = await import("./engine");
    const reg = await computeScenario({ lat_deg: -84.7906, lon_deg: 29.1957, mast_m: 0, start_unix_s: IM2_TOUCHDOWN, span_days: 3 });
    const depth = await defaultPitDepth(20);
    expect(Math.abs(depth - 1.9438)).toBeLessThan(1e-4); // (D/2) tan 11 deg, from the WASM
    const pit = await computePit(reg, { diameter_m: 20, depth_m: depth });
    // H = 0: el_pit = 11.0 deg (SCIENCE-SPEC 1f), so no bin is below it
    expect(Math.min(...pit.horizon.el_deg)).toBeGreaterThan(10.999);
    expect(pit.count).toBe(reg.count);
    for (let i = 0; i < reg.count; i++) expect(pit.timeline.illumination[i]).toBeLessThanOrEqual(reg.timeline.illumination[i]);
    // the Sun never exceeds 6.75 deg and the rim is at 11 deg: fully hidden whenever it is above terrain
    expect(Math.max(...pit.timeline.illumination)).toBe(0);
    await expect(computePit(reg, { diameter_m: 0, depth_m: 1 })).rejects.toThrow("BAD_INPUT");
    for (const depth_m of [NaN, -1]) await expect(computePit(reg, { diameter_m: 20, depth_m })).rejects.toThrow("BAD_INPUT");
    await expect(computePit({ ...reg, site: { ...reg.site, mast_m: -1 } }, { diameter_m: 20, depth_m: 1 })).rejects.toThrow("BAD_INPUT");
  } finally {
    globalThis.fetch = realFetch;
  }
});

test("DEFAULT_PRESET matches penumbra_core::power::ROVER_PRESET and its cited fields", () => {
  const rs = readFileSync(new URL("../../../crates/core/src/power.rs", import.meta.url), "utf8");
  const block = rs.slice(rs.indexOf("pub const ROVER_PRESET"), rs.indexOf("};", rs.indexOf("pub const ROVER_PRESET")));
  const num = (k: string) => Number(block.match(new RegExp(`${k}: ([0-9_.]+)`))![1].replaceAll("_", ""));
  for (const k of ["capacity_wh", "start_wh", "min_wh", "panel_area_m2", "efficiency", "load_w"] as const) expect(DEFAULT_PRESET[k]).toBe(num(k));
  expect(block).toContain("Panel::SunTracking");
  expect(DEFAULT_PRESET.panel).toBe("tracking");
  const cited = rs.match(/ROVER_CITED_FIELDS: \[&str; \d+\] = \[([^\]]+)\]/)![1].match(/"(\w+)"/g)!.map((s) => s.slice(1, -1));
  expect(Object.keys(DEFAULT_PRESET.cited).filter((k) => DEFAULT_PRESET.cited[k]).sort()).toEqual(cited.sort());
});
