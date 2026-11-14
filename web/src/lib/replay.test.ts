import { expect, test } from "bun:test";
import { checkPit, pitSentence } from "./replay";

test("checkPit accepts the IM-2 defaults and a depth of zero", () => {
  expect(checkPit({ diameter_m: 20, depth_m: 1.94, panel_m: 1 })).toBeNull();
  expect(checkPit({ diameter_m: 20, depth_m: 0, panel_m: 0 })).toBeNull();
});

test("checkPit refuses what the model would throw BAD_INPUT on, and a negative depth or height", () => {
  for (const bad of [0, -5, NaN, Infinity]) expect(checkPit({ diameter_m: bad, depth_m: 1, panel_m: 1 })).toContain("diameter");
  for (const bad of [-0.1, NaN, Infinity]) expect(checkPit({ diameter_m: 20, depth_m: bad, panel_m: 1 })).toContain("depth");
  for (const bad of [-1, NaN, Infinity]) expect(checkPit({ diameter_m: 20, depth_m: 1, panel_m: bad })).toContain("height");
});

const base = { elSunDeg: 3.456, clearPct: 87.2, diameterM: 20, panelM: 2, dStarM: 2.9218, typicalDepthM: 1.9438 };

test("pitSentence words the d_c* answer and says deeper or shallower", () => {
  const s = pitSentence(base);
  expect(s).toContain("3.5° up");
  expect(s).toContain("87 % of its disc clear");
  expect(s).toContain("20 m crater would need to be 2.92 m deep");
  expect(s).toContain("2 m above its floor");
  expect(s).toContain("1.9 m deep");
  expect(s).toContain("shallower than");
  expect(pitSentence({ ...base, dStarM: 1.5 })).toContain("deeper than");
  expect(pitSentence({ ...base, dStarM: 1.9438 })).toContain("as deep as");
});

test("pitSentence never claims the lander was in shadow (N10)", () => {
  expect(pitSentence(base)).not.toMatch(/was in shadow|were dark|validated/i);
});
