import { expect, test } from "bun:test";
import type { Preset, Timeline } from "./engine";
import { buildChartData, indexAtTime, operableMask, operableRuns, thresholdLabel } from "./timelines";

const preset = { earth_margin_deg: 2, cited: { load_w: true } } as unknown as Preset;
const f = (...v: number[]) => Float64Array.from(v);

test("threshold label is exactly '2° (assumed)' until cited", () => {
  expect(thresholdLabel(preset)).toBe("2° (assumed)");
  expect(thresholdLabel({ ...preset, cited: { earth_margin_deg: true } } as unknown as Preset)).toBe("2°");
});

test("operable = illumination > 0 and contact", () => {
  const m = operableMask({ illumination: f(0, 0.5, 1, 0.2, 0), contact: f(1, 1, 0, 1, 1) });
  expect([...m]).toEqual([0, 1, 0, 1, 0]);
});

test("fixture band sums to its own operable_h (SCIENCE-SPEC 1e)", async () => {
  const raw = await Bun.file(new URL("../../public/fixture/scenario.json", import.meta.url)).json();
  const tl = raw.timeline;
  const mask = operableMask({ illumination: Float64Array.from(tl.illumination), contact: Float64Array.from(tl.contact) });
  const h = (mask.reduce((s, v) => s + v, 0) * raw.step_s) / 3600;
  expect(h).toBeCloseTo(raw.survival.operable_h, 3);
});

test("runs are half-open and include the trailing step", () => {
  const t = [0, 600, 1200, 1800, 2400];
  expect(operableRuns(t, [0, 1, 1, 0, 1], 600)).toEqual([[600, 1800], [2400, 3000]]);
  expect(operableRuns(t, [0, 0, 0, 0, 0], 600)).toEqual([]);
});

test("indexAtTime picks the nearest step and clamps", () => {
  const t = [0, 600, 1200];
  expect(indexAtTime(t, -50)).toBe(0);
  expect(indexAtTime(t, 290)).toBe(0);
  expect(indexAtTime(t, 310)).toBe(1);
  expect(indexAtTime(t, 9999)).toBe(2);
});

test("chart data keeps illumination and visibility as separate series, with a flat threshold", () => {
  const tl = { t_unix_s: f(0, 1), illumination: f(0.2, 0.4), visibility: f(1, 1), earth_margin_deg: f(3, 4), contact: f(0, 1) } as unknown as Timeline;
  const d = buildChartData(tl, preset);
  expect(d.illum[1]).toEqual([0.2, 0.4]);
  expect(d.illum[2]).toEqual([1, 1]);
  expect(d.margin[1]).toEqual([3, 4]);
  expect(d.margin[2]).toEqual([2, 2]);
  expect(d.contact[1]).toEqual([0, 1]);
});
