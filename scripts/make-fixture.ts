// Writes web/public/fixture/scenario.json and web/public/scoreboard.json (ARCHITECTURE §3.5).
// Sine waves only: no lunar maths. Every value is FAKE and exists so layouts and axes can be built.
const N = 4320, STEP = 600, T0 = 1794614400; // 2026-11-14T00:00:00Z
const r = (x: number) => Math.round(x * 1e4) / 1e4;
const idx = Array.from({ length: N }, (_, i) => i);
const col = (f: (i: number) => number) => idx.map((i) => r(f(i)));
const clamp = (x: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, x));
const TAU = 2 * Math.PI;

const sunEl = (i: number) => 6.5 * Math.sin((TAU * i) / 4380);
const earthEl = (i: number) => 7.5 * Math.sin((TAU * i) / 4320 + 1);
const illum = (i: number) => r(clamp(0.5 + 1.2 * Math.sin((TAU * i) / 720), 0, 1)); // dark ~0.3 d, lit ~0.3 d, steps between 0 and 1
const earthMargin = (i: number) => earthEl(i) - 2;

const scoreboard = [
  ["V1", "JPL Horizons", "worst az/el difference, deg", 0.002, 20],
  ["V2", "PGDA AVGVISIB_85S_060M_201608", "pooled median visibility error, points", 5, 10000],
  ["V3", "PGDA map-projected NACs", "shadow IoU gap to NASA _MODEL.TIF", 0.05, 3],
  ["V4", "IM-1 / IM-2 published timelines", "consistent with published timeline", null, 2],
  ["V5", "Red-team edge cases", "automated tests passing", null, null],
].map(([id, judge, metric, threshold, sample_size]) => ({
  id, judge, metric, threshold, value: null, pass: null, sample_size, notes: "not run", commit: null, date: null,
}));

const scenario = {
  fake: true,
  note: "FAKE DATA for layout only. Not a model result.",
  api_version: 1,
  site: { lat_deg: -84.7906, lon_deg: 29.1957, ground_m: 0, mast_m: 2 },
  start_unix_s: T0,
  step_s: STEP,
  count: N,
  preset: {
    capacity_wh: 10000, start_wh: 10000, min_wh: 0, panel_area_m2: 1.5, efficiency: 0.3, panel: "tracking",
    load_w: 220, earth_margin_deg: 2, mast_m: 2,
    cited: { capacity_wh: true, panel_area_m2: true, efficiency: true, load_w: true },
  },
  horizon: {
    el_deg: Array.from({ length: 720 }, (_, i) => r(6.5 + 8.5 * Math.sin((TAU * 3 * i) / 720))), // -2..15
    dist_m: Array.from({ length: 720 }, (_, i) => r(8000 + 7000 * Math.sin((TAU * 5 * i) / 720))),
  },
  track: {
    t_unix_s: col((i) => T0 + i * STEP),
    sun_az_deg: col((i) => (360 - ((i * 360) / 4248) % 360) % 360), // decreasing, as the real south-pole Sun (SCIENCE-SPEC P9)
    sun_el_deg: col(sunEl),
    sun_radius_deg: col(() => 0.267),
    sun_dist_km: col((i) => 1.496e8 + 2.5e6 * Math.sin((TAU * i) / 4320)),
    earth_az_deg: col((i) => ((i * 360) / 4320 + 120) % 360),
    earth_el_deg: col(earthEl),
    earth_radius_deg: col(() => 0.95), // angular radius: asin(6371 / 384400) ≈ 0.95°, so 1.9° across
    earth_dist_km: col((i) => 384400 + 20000 * Math.sin((TAU * i) / 3960)),
  },
  timeline: {
    t_unix_s: col((i) => T0 + i * STEP),
    sun_az_deg: col((i) => (360 - ((i * 360) / 4248) % 360) % 360), // decreasing, as the real south-pole Sun (SCIENCE-SPEC P9)
    sun_el_deg: col(sunEl),
    illumination: idx.map(illum),
    visibility: idx.map((i) => (illum(i) > 0 ? 1 : 0)),
    earth_az_deg: col((i) => ((i * 360) / 4320 + 120) % 360),
    earth_el_deg: col(earthEl),
    earth_margin_deg: col(earthMargin),
    contact: idx.map((i) => (earthMargin(i) > 0 ? 1 : 0)),
    sun_dist_km: col((i) => 1.496e8 + 2.5e6 * Math.sin((TAU * i) / 4320)),
  },
  survival: {
    operable_h: 212.5, lethal_h: 301.2, longest_dark_h: 41.0, longest_any_sun_h: 180.0, sunlit_h: 390.0, contact_h: 330.0,
    soc_wh: col((i) => 10000 * Math.max(0, 1 - i / 1807) * (0.9 + 0.1 * Math.cos((TAU * i) / 720))),
  },
  scoreboard,
};

await Bun.write(new URL("../web/public/fixture/scenario.json", import.meta.url), JSON.stringify(scenario) + "\n");
await Bun.write(new URL("../web/public/scoreboard.json", import.meta.url), JSON.stringify(scoreboard, null, 2) + "\n");
