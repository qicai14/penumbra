import { expect, test } from "bun:test";
import type { Preset, Timeline } from "./engine";
import { buildChartData, buildSocData, clipRunsAt, countdown, fmtDH, indexAtTime, lethalUnixS, operableMask, operableRuns, thresholdLabel } from "./timelines";

// T26 (#27): countdowns read the timeline columns only.
const hourly = (n: number) => Float64Array.from({ length: n }, (_, i) => i * 3600);

test("fmtDH is 'Xd Yh', minutes under an hour", () => {
  expect(fmtDH(2 * 86400 + 4 * 3600 + 1800)).toBe("2d 4h");
  expect(fmtDH(5 * 3600)).toBe("0d 5h");
  expect(fmtDH(600)).toBe("10m");
  expect(fmtDH(3599)).toBe("59m"); // never "60m"
  // Up now and the very next 10-minute step is the set.
  expect(countdown(Float64Array.from([0, 600, 1200]), Float64Array.from([1, 0, 0]), (x) => x > 0, 0, "sunset")).toBe("Sunset in 10m");
});

test("sunset is the next step where illumination drops to 0", () => {
  const illum = Float64Array.from([1, 0.5, 0.2, 0, 0, 1]);
  expect(countdown(hourly(6), illum, (x) => x > 0, 0, "sunset")).toBe("Sunset in 0d 3h");
  expect(countdown(hourly(6), illum, (x) => x > 0, 2, "sunset")).toBe("Sunset in 0d 1h");
});

test("when it is already down, count to the next set after it rises, never negative", () => {
  const illum = Float64Array.from([0, 0, 1, 1, 0, 0]);
  expect(countdown(hourly(6), illum, (x) => x > 0, 0, "sunset")).toBe("Sunset in 0d 4h");
});

test("no crossing left in the span says 'no sunset in …', never blank", () => {
  const up = Float64Array.from([1, 1, 1, 1]);
  expect(countdown(hourly(4), up, (x) => x > 0, 0, "sunset")).toBe("no sunset in 3 h");
  const down = Float64Array.from([0, 0, 0]);
  // Down for the rest of the span: there is nothing to set, so it says no rise (a shadowed site never reads as sunlit).
  expect(countdown(hourly(3), down, (x) => x > 0, 0, "Earthset")).toBe("no Earthrise in 2 h");
  expect(countdown(hourly(3), down, (x) => x > 0, 0, "sunset")).toBe("no sunrise in 2 h");
  // Rises but does not set again: still "no sunset".
  expect(countdown(hourly(3), Float64Array.from([0, 1, 1]), (x) => x > 0, 0, "sunset")).toBe("no sunset in 2 h");
  expect(countdown(Float64Array.from([0, 30 * 86400]), Float64Array.from([1, 1]), (x) => x > 0, 0, "sunset")).toBe("no sunset in 30 days");
  expect(countdown(hourly(4), up, (x) => x > 0, 3, "sunset")).toBe("no sunset before the span ends"); // last step, never "0 h"
});

test("Earthset uses contact dropping below 0.5", () => {
  const contact = Float64Array.from([1, 1, 0, 1]);
  expect(countdown(hourly(4), contact, (x) => x >= 0.5, 0, "Earthset")).toBe("Earthset in 0d 2h");
});

test("soc data pairs the model's trace with the lethal-charge line", () => {
  const d = buildSocData(Float64Array.from([0, 600]), Float64Array.from([10, 5]), { min_wh: 1 } as unknown as Preset);
  expect(d).toEqual([[0, 600], [10, 5], [1, 1]]);
});

test("a trace that stops at a lethal shadow is padded with gaps to the time axis", () => {
  const d = buildSocData(Float64Array.from([0, 600, 1200]), Float64Array.from([10]), { min_wh: 1 } as unknown as Preset);
  expect(d).toEqual([[0, 600, 1200], [10, null, null], [1, 1, 1]]);
});

test("lethal time is start plus lethal hours, null when none", () => {
  expect(lethalUnixS(1000, 2)).toBe(8200);
  expect(lethalUnixS(1000, null)).toBeNull();
});

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
  const raw = await Bun.file(new URL("../../tests/fixture/scenario.json", import.meta.url)).json();
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

test("operable runs are cut off at the lethal time", () => {
  const runs: [number, number][] = [[600, 1800], [2400, 3000]];
  expect(clipRunsAt(runs, null)).toEqual(runs);
  expect(clipRunsAt(runs, 1000)).toEqual([[600, 1000]]); // spanning run ends exactly there
  expect(clipRunsAt(runs, 2400)).toEqual([[600, 1800]]); // a run starting at the lethal second is dropped
  expect(clipRunsAt(runs, 3600)).toEqual(runs);
  expect(clipRunsAt(runs, 600)).toEqual([]);
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

// T18 (#19): band formatting and the ill-conditioned threshold.
import { bandInterval, bandLabel, isIllConditioned, parseBands, SIGMA_Z_TEXT, UNBOUNDED_HALF_WIDTH_MIN } from "./timelines";

const T = Date.UTC(2026, 10, 14, 14, 20) / 1000;

test("a normal band reads ± minutes; sunrise and sunset use their own word", () => {
  expect(bandLabel({ t_unix_s: T, kind: "sunset", half_width_min: 40.2 })).toBe("sunset 14:20 ± 40 min");
  expect(bandLabel({ t_unix_s: T, kind: "sunrise", half_width_min: 5 }, true)).toBe("sunrise 2026-11-14 14:20 ± 5 min");
  expect(bandLabel({ t_unix_s: T, kind: "sunset", half_width_min: 0.2 })).toBe("sunset 14:20 ± <1 min");
});

test("a band wider than 6 h is ill-conditioned, not minutes", () => {
  expect(isIllConditioned(179)).toBe(false); // 358 min wide
  expect(isIllConditioned(181)).toBe(true); // 362 min wide
  expect(bandLabel({ t_unix_s: T, kind: "sunrise", half_width_min: 400 })).toBe("sunrise 14:20 ill-conditioned");
});

test("core's unbounded marker and non-finite widths are ill-conditioned", () => {
  for (const w of [UNBOUNDED_HALF_WIDTH_MIN, Infinity, NaN]) {
    expect(bandLabel({ t_unix_s: T, kind: "sunset", half_width_min: w })).toBe("sunset 14:20 ill-conditioned");
    const [a, b] = bandInterval({ t_unix_s: T, kind: "sunset", half_width_min: w });
    expect(Number.isFinite(a) && Number.isFinite(b)).toBe(true);
  }
});

test("parseBands reads the 3 × k column-major layout; the σ_z text says assumed", () => {
  expect(parseBands(Float64Array.from([10, 20, 1, -1, 3, 4]))).toEqual([
    { t_unix_s: 10, kind: "sunrise", half_width_min: 3 }, { t_unix_s: 20, kind: "sunset", half_width_min: 4 },
  ]);
  expect(parseBands(new Float64Array(0))).toEqual([]);
  expect(SIGMA_Z_TEXT).toBe("σ_z = 0.5 m (assumed)");
});
