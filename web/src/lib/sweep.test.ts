import { expect, test } from "bun:test";
import { bestPoint, sweep, sweepDates } from "./sweep";
import { parseDate } from "./site";

// One segment: 2026-11-01 to 2027-01-01 00:00 UTC, hourly.
const t0 = parseDate("2026-11-01")!;
const eph = [{ id: "a", url: "a", t0_unix_s: t0, step_s: 3600, rows: 61 * 24 + 1 }];

test("30 consecutive dates from the picked date", () => {
  const d = sweepDates("2026-11-14", 14, eph);
  expect(d.length).toBe(30);
  expect(d[0]).toBe("2026-11-14");
  expect(d[29]).toBe("2026-12-13");
});

test("dates whose span leaves the ephemeris range are dropped", () => {
  // span 30 d: the last start that fits is 2026-12-02
  const d = sweepDates("2026-11-25", 30, eph);
  expect(d).toEqual(["2026-11-25", "2026-11-26", "2026-11-27", "2026-11-28", "2026-11-29", "2026-11-30", "2026-12-01", "2026-12-02"]);
  expect(sweepDates("2027-06-01", 30, eph)).toEqual([]);
});

test("sweep returns one point per date, in order, and reports progress", async () => {
  const dates = ["2026-11-14", "2026-11-15", "2026-11-16"];
  const seen: number[] = [];
  const out = await sweep(dates, async (d) => ({ operable_h: Number(d.slice(-2)), lethal_h: d === "2026-11-15" ? 12 : null }), (n) => seen.push(n));
  expect(out.map((p) => p.date)).toEqual(dates);
  expect(out.map((p) => p.operable_h)).toEqual([14, 15, 16]);
  expect(seen).toEqual([1, 2, 3]);
});

test("sweep stops when cancelled", async () => {
  let live = true;
  const out = await sweep(["2026-11-14", "2026-11-15", "2026-11-16"], async () => { live = false; return { operable_h: 1, lethal_h: null }; }, undefined, () => !live);
  expect(out.length).toBe(1);
});

test("best date is the most operable days, earliest on a tie", () => {
  const p = (date: string, operable_h: number) => ({ date, operable_h, lethal_h: null });
  expect(bestPoint([p("a", 5), p("b", 9), p("c", 9), p("d", 1)])?.date).toBe("b");
  expect(bestPoint([])).toBeNull();
});
