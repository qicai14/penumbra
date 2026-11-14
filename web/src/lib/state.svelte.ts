// Shared controls state (T11). App reads it to call computeScenario; SitePicker writes it after the refusal checks pass.
export class Controls {
  lat = $state(-84.7906);
  lon = $state(29.1957);
  date = $state("2026-11-14"); // landing date, UTC
  spanDays = $state(30);
  mastM = $state(2);
}
