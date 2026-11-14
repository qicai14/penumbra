// Pure helpers for the survival headline and the preset form (T14). Text and field state only; every number shown
// comes from the model's `survival` result or the preset itself (SCIENCE-SPEC 1e). Nothing here computes physics.
import type { Preset, Survival } from "./engine";
import { citedTag, fmtDays } from "./timelines";
export { fmtDays }; // lives in timelines.ts so the two modules import one way only

export type FieldKey = "capacity_wh" | "start_wh" | "min_wh" | "panel_area_m2" | "efficiency" | "load_w" | "earth_margin_deg";
export type PresetField = { key: FieldKey; label: string; unit: string; value: number; step: number; tag: "cited" | "assumed"; note?: string };

const FIELDS: [FieldKey, string, string, number][] = [
  ["capacity_wh", "Battery capacity", "Wh", 100],
  ["start_wh", "Charge at landing", "Wh", 100],
  ["min_wh", "Lethal charge (E_min)", "Wh", 100],
  ["panel_area_m2", "Panel area", "m²", 0.1],
  ["efficiency", "Cell efficiency", "%", 1],
  ["load_w", "Load", "W", 10],
  ["earth_margin_deg", "Earth margin m_min", "°", 0.5],
];

/** Length of the computed span in days, from its own rows. */
export const spanDays = (count: number, stepS: number): number => Math.round((count * stepS) / 864) / 100;

export type Headline = { operable: string; lethal: string; none: boolean };

/** The two headline figures; with no lethal event in the span the second reads "no lethal shadow in N days". */
export function headline(s: Pick<Survival, "operable_h" | "lethal_h">, span: number): Headline {
  const none = s.lethal_h === null;
  return {
    operable: fmtDays(s.operable_h),
    lethal: none ? `no lethal shadow in ${span} ${span === 1 ? "day" : "days"}` : fmtDays(s.lethal_h as number),
    none,
  };
}

/** Beside the load while it still reads "cited": the 220 W is the rover driving load (E58d); holding it for 24 h is our assumption. */
export const LOAD_NOTE = "cited value (rover driving load, E58d); constant 24 h use is assumed";

/** Rows for the preset form. Efficiency is a fraction in the model and a percent on screen. */
export function presetFields(p: Preset): PresetField[] {
  return FIELDS.map(([key, label, unit, step]) => ({
    key, label, unit, step, tag: citedTag(p, key),
    ...(key === "load_w" && citedTag(p, key) === "cited" ? { note: LOAD_NOTE } : {}),
    value: key === "efficiency" ? Math.round(p.efficiency * 1e6) / 1e4 : p[key],
  }));
}

/** A copy of the preset with one field set (efficiency in percent), or the reason the value is refused.
 *  An edited field is the planner's own number, so it reads "assumed" from then on.
 *  Lowering capacity lowers the charges above it too, so capacity can always be reduced.
 *  These bounds mirror the SCIENCE-SPEC 1e preset fields; T09 (#10) must refuse the same values in Rust. */
export function setPresetField(p: Preset, key: FieldKey, value: number): Preset | string {
  if (!Number.isFinite(value) || value < 0) return "must be a number, 0 or more";
  const v = key === "efficiency" ? value / 100 : value;
  const next: Preset = { ...p, [key]: v, cited: { ...p.cited, [key]: false } };
  if (key === "capacity_wh") {
    for (const k of ["start_wh", "min_wh"] as const) {
      if (next[k] > v) { next[k] = v; next.cited = { ...next.cited, [k]: false }; }
    }
  }
  if (next.capacity_wh <= 0) return "battery capacity must be above 0 Wh";
  if (next.efficiency > 1) return "cell efficiency must be at most 100 %"; // 0 is valid: battery only
  if (next.earth_margin_deg > 90) return "Earth margin must be at most 90°";
  if (next.start_wh > next.capacity_wh) return `charge at landing must not exceed battery capacity (${next.capacity_wh} Wh)`;
  if (next.min_wh > next.capacity_wh) return `lethal charge must not exceed battery capacity (${next.capacity_wh} Wh)`;
  return next;
}
