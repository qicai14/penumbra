import { expect, test } from "bun:test";
import type { Preset, Survival } from "./engine";
import { fmtDays, headline, presetFields, setPresetField, spanDays } from "./headline";

const preset: Preset = {
  capacity_wh: 10000, start_wh: 10000, min_wh: 0, panel_area_m2: 1.5, efficiency: 0.3, panel: "tracking",
  load_w: 220, earth_margin_deg: 2, mast_m: 2,
  cited: { capacity_wh: true, panel_area_m2: true, efficiency: true, load_w: true },
};
const surv = (over: Partial<Survival>) => ({ operable_h: 240, lethal_h: 302.4, longest_dark_h: 41, longest_any_sun_h: 180, sunlit_h: 0, contact_h: 0, soc_wh: new Float64Array(0), ...over });

test("days format to one decimal, whole days drop the point", () => {
  expect(fmtDays(240)).toBe("10 days");
  expect(fmtDays(302.4)).toBe("12.6 days");
  expect(fmtDays(24)).toBe("1 day");
  expect(fmtDays(23.97)).toBe("1 day");
});

test("under a day reads in hours, never '0 days'", () => {
  expect(fmtDays(7.05)).toBe("7.1 h"); // SCIENCE-SPEC T-1e-1 lethal time
  expect(fmtDays(0.5)).toBe("0.5 h");
  expect(fmtDays(0)).toBe("0 h");
});

test("span in days comes from the scenario's own rows", () => {
  expect(spanDays(4320, 600)).toBe(30);
});

test("headline gives operable days and time to first lethal shadow", () => {
  expect(headline(surv({}), 30)).toEqual({ operable: "10 days", lethal: "12.6 days", none: false });
});

test("no lethal shadow in the span says so with the span length", () => {
  const h = headline(surv({ lethal_h: null }), 30);
  expect(h).toEqual({ operable: "10 days", lethal: "no lethal shadow in 30 days", none: true });
});

test("a lethal time of zero is a lethal shadow, not 'none'", () => {
  expect(headline(surv({ lethal_h: 0 }), 30).none).toBe(false);
});

test("every field carries cited or assumed; uncited ones read assumed", () => {
  const rows = presetFields(preset);
  const by = Object.fromEntries(rows.map((r) => [r.key, r.tag]));
  expect(by.capacity_wh).toBe("cited");
  expect(by.start_wh).toBe("assumed");
  expect(by.min_wh).toBe("assumed");
  expect(by.earth_margin_deg).toBe("assumed");
  expect(rows.every((r) => r.tag === "cited" || r.tag === "assumed")).toBe(true);
});

test("efficiency is shown in percent", () => {
  expect(presetFields(preset).find((r) => r.key === "efficiency")!.value).toBe(30);
});

test("editing a field changes it and marks it assumed", () => {
  const p = setPresetField(preset, "load_w", 300) as Preset;
  expect(p.load_w).toBe(300);
  expect(p.cited.load_w).toBe(false);
  expect(preset.load_w).toBe(220); // input untouched
  expect((setPresetField(preset, "efficiency", 25) as Preset).efficiency).toBeCloseTo(0.25, 12);
});

test("lowering capacity lowers the charges above it instead of being refused", () => {
  const p = setPresetField({ ...preset, min_wh: 9000 }, "capacity_wh", 8000) as Preset;
  expect([p.capacity_wh, p.start_wh, p.min_wh]).toEqual([8000, 8000, 8000]);
  expect(p.cited.start_wh).toBe(false);
});

test("bad edits are refused with the rule they break", () => {
  expect(setPresetField(preset, "load_w", NaN)).toContain("number");
  expect(setPresetField(preset, "load_w", -1)).toContain("0 or more");
  expect(setPresetField(preset, "capacity_wh", 0)).toContain("capacity");
  expect(setPresetField(preset, "efficiency", 120)).toContain("efficiency");
  expect(setPresetField(preset, "efficiency", -1)).toContain("0 or more");
  expect(setPresetField(preset, "earth_margin_deg", 500)).toContain("90°");
  expect(setPresetField(preset, "start_wh", 20000)).toContain("charge at landing");
  expect(setPresetField(preset, "min_wh", 10001)).toContain("lethal charge");
});

test("panel area 0 and efficiency 0 are valid (battery only), as in the Rust check", () => {
  expect((setPresetField(preset, "panel_area_m2", 0) as Preset).panel_area_m2).toBe(0);
  expect((setPresetField(preset, "efficiency", 0) as Preset).efficiency).toBe(0);
  expect(setPresetField(preset, "panel_area_m2", -0.1)).toContain("0 or more");
});

test("the load shows its caveat while cited, and not once the planner edits it", () => {
  const load = (p: Preset) => presetFields(p).find((r) => r.key === "load_w")!;
  expect(load(preset).tag).toBe("cited");
  expect(load(preset).note).toContain("E58d");
  expect(load(setPresetField(preset, "load_w", 100) as Preset).note).toBeUndefined();
});
