// Landing-window sweep (#108): the survival headline's operable hours for each of the next 30 landing dates.
// No model maths here: `compute` is the same engine call the headline uses, so a bar equals the headline for its date.
import type { EphSegment } from "./engine";
import { checkDate, parseDate } from "./site";

export const SWEEP_DAYS = 30;
export type SweepPoint = { date: string; operable_h: number; lethal_h: number | null };

/** Up to `n` consecutive landing dates from `from`; a date whose whole span does not fit one ephemeris segment ends the list. */
export function sweepDates(from: string, spanDays: number, eph: readonly EphSegment[], n = SWEEP_DAYS): string[] {
  const t = parseDate(from);
  if (t === null) return [];
  const out: string[] = [];
  for (let i = 0; i < n; i++) {
    const d = new Date((t + i * 86400) * 1000).toISOString().slice(0, 10);
    if (!checkDate(d, spanDays, eph).ok) break;
    out.push(d);
  }
  return out;
}

/** Runs `compute` once per date, yielding to the event loop between dates so the page stays responsive. `cancelled` is polled between dates. */
export async function sweep(
  dates: readonly string[],
  compute: (date: string) => Promise<Pick<SweepPoint, "operable_h" | "lethal_h">>,
  onProgress?: (done: number) => void,
  cancelled: () => boolean = () => false,
): Promise<SweepPoint[]> {
  const out: SweepPoint[] = [];
  for (const date of dates) {
    if (cancelled()) break;
    const { operable_h, lethal_h } = await compute(date);
    out.push({ date, operable_h, lethal_h });
    onProgress?.(out.length);
    await new Promise((r) => setTimeout(r, 0));
  }
  return out;
}

/** The date with the most operable hours; the earliest on a tie. */
export function bestPoint(pts: readonly SweepPoint[]): SweepPoint | null {
  return pts.reduce<SweepPoint | null>((b, p) => (b === null || p.operable_h > b.operable_h ? p : b), null);
}
