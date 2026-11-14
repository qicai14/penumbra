// Shared controls state (T11). App reads it to call computeScenario; SitePicker writes it after the refusal checks pass.
import type { Preset } from "./engine";
import { DEFAULT_MAST_M, DEFAULT_SPAN_DAYS } from "./site";
export class Controls {
  lat = $state(-84.7906);
  lon = $state(29.1957);
  date = $state("2026-11-14"); // landing date, UTC
  spanDays = $state<number>(DEFAULT_SPAN_DAYS);
  mastM = $state<number>(DEFAULT_MAST_M);
  preset = $state.raw<Preset | null>(null); // null until the planner edits a field; then it overrides the scenario preset
}
