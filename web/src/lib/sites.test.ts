import { expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { SITES, coverageRegion } from "./sites";
import { xyToLatlon } from "./projection";

const manifest = JSON.parse(readFileSync(new URL("../../public/data/manifest.json", import.meta.url), "utf8"));
const ledger = readFileSync(new URL("../../../docs/SCIENCE-SPEC.md", import.meta.url), "utf8");

test("every curated site cites a source id that exists in the SCIENCE-SPEC §6 ledger", () => {
  for (const s of SITES) {
    expect(s.src.length).toBeGreaterThan(0);
    for (const id of s.src.split(",").map((t) => t.trim()).filter((t) => t !== "D"))
      expect(ledger.includes(`| ${id} |`), `${s.name}: ${id}`).toBe(true);
  }
});

test("Barker Site 1 is the RoI 1 centroid (E129, E130) through the inverse projection, within 0.001 deg", () => {
  const [lat, lon] = xyToLatlon(-11582.5, -12462.5);
  const s = SITES.find((x) => x.id === "barker1")!;
  expect(Math.abs(lat - s.lat)).toBeLessThan(0.001);
  expect(Math.abs(lon - s.lon)).toBeLessThan(0.001);
});

test("coverage: IM-2 is inside a shipped tile, Apollo 11 is not", () => {
  const get = (id: string) => SITES.find((x) => x.id === id)!;
  expect(coverageRegion(get("im2").lat, get("im2").lon, manifest.regions)).not.toBeNull();
  expect(coverageRegion(get("a11").lat, get("a11").lon, manifest.regions)).toBeNull();
  expect(coverageRegion(get("im1").lat, get("im1").lon, manifest.regions)).toBeNull();
});
