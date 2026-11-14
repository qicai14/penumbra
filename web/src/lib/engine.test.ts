import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import init, { api_version } from "./wasm/penumbra_wasm.js";
import { checkApiVersion, computeScenario, parseScenario, validate } from "./engine";

const raw = JSON.parse(readFileSync(new URL("../../public/fixture/scenario.json", import.meta.url), "utf8"));
await init({ module_or_path: readFileSync(new URL("./wasm/penumbra_wasm_bg.wasm", import.meta.url)) });

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

test("an edited preset reaches computeScenario (T14 recompute wiring)", async () => {
  const realFetch = globalThis.fetch;
  globalThis.fetch = (async () => new Response(JSON.stringify(raw))) as unknown as typeof fetch;
  try {
    const base = parseScenario(raw, api_version());
    const preset = { ...base.preset, load_w: 300 };
    const s = await computeScenario({ lat_deg: -84.79, lon_deg: 29.2, mast_m: 2, start_unix_s: 0, span_days: 30, preset });
    expect(s.preset).toBe(preset);
  } finally {
    globalThis.fetch = realFetch;
  }
});
