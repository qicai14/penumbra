import { afterAll, beforeAll, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { computeScenario, loadManifest, xyToLatlon } from "./engine";
import init from "./wasm/penumbra_wasm.js";
import { decodeGrid, NODATA, NO_LETHAL, type GridMeta } from "./grid";

// #25: five random cells per date of the shipped grid equal the live in-browser engine (computeScenario: the site view's
// own call), exact in whole hours, for every grid in the manifest. Fixed seed, so the same cells every run.
await init({ module_or_path: readFileSync(new URL("./wasm/penumbra_wasm_bg.wasm", import.meta.url)) });
const pub = (p: string) => new URL(`../../public/${p.replace(/^\//, "")}`, import.meta.url);
const realFetch = globalThis.fetch;
beforeAll(() => { globalThis.fetch = (async (u: string) => new Response(readFileSync(pub(String(u))))) as unknown as typeof fetch; });
afterAll(() => { globalThis.fetch = realFetch; });

function rng(seed: number) { // mulberry32
  return () => { seed = (seed + 0x6d2b79f5) | 0; let t = Math.imul(seed ^ (seed >>> 15), 1 | seed); t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t; return ((t ^ (t >>> 14)) >>> 0) / 4294967296; };
}

test("5 random cells per date per region: browser engine operable_h and lethal_h equal the stored grid", async () => {
  for (const meta of (await loadManifest()).grids!) await checkGrid(meta);
}, 240_000);

async function checkGrid(meta: GridMeta) {
  const g = decodeGrid(readFileSync(pub(meta.url)).buffer.slice(0) as ArrayBuffer, meta);
  const rand = rng(25), n = meta.width * meta.height;
  for (let d = 0; d < meta.dates.length; d++) {
    for (let picked = 0; picked < 5;) {
      const k = Math.floor(rand() * n);
      if (g.operable[d][k] === NODATA) continue;
      picked++;
      const x = meta.x0_m + ((k % meta.width) + 0.5) * meta.pixel_m, y = meta.y0_m - (Math.floor(k / meta.width) + 0.5) * meta.pixel_m;
      const [lat_deg, lon_deg] = xyToLatlon(x, y);
      const s = await computeScenario({ lat_deg, lon_deg, mast_m: meta.mast_m, start_unix_s: meta.landing_unix_s[d], span_days: meta.span_days });
      expect(s.count).toBe(4320);
      const lethal = s.survival.lethal_h === null ? NO_LETHAL : Math.round(s.survival.lethal_h) + 0; // + 0: Math.round gives -0 for -0.4
      expect({ date: meta.dates[d], k, op: Math.round(s.survival.operable_h) + 0, lethal }).toEqual({ date: meta.dates[d], k, op: g.operable[d][k], lethal: g.lethal[d][k] });
    }
  }
}
