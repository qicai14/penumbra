import { expect, test } from "bun:test";
import { headline, parseScoreboard, rowStatus, tally, type Row } from "./scoreboard";

const row = (id: string, pass: boolean | null, value: number | null = null): Row => ({
  id, judge: "j", metric: "m", threshold: 1, value, pass, sample_size: 1, notes: "", commit: null, date: null,
});

test("pass null is not run, even with a value set", () => {
  expect(rowStatus(row("V1", null))).toBe("not run");
  expect(rowStatus(row("V1", null, 0.001))).toBe("not run");
  expect(rowStatus(row("V1", true, 0.001))).toBe("pass");
  expect(rowStatus(row("V1", false, 9))).toBe("miss");
});

test("tally counts only checks that ran", () => {
  const t = tally([row("V1", true), row("V2", true), row("V3", null), row("V4", true), row("V5", false)]);
  expect(t).toEqual({ total: 5, ran: 4, passed: 3, missed: ["V5"], notRun: ["V3"] });
});

test("headline is honest when nothing has run", () => {
  const all = ["V1", "V2", "V3", "V4", "V5"].map((id) => row(id, null));
  expect(headline(tally(all))).toBe("No checks have run yet (0 of 5 checks run).");
});

test("headline counts only checks that ran and names the rest", () => {
  const rows = [row("V1", true), row("V2", true), row("V3", null), row("V4", true), row("V5", null)];
  expect(headline(tally(rows))).toBe("3 of the 3 checks we ran pass; V3 and V5 not run.");
  const one = [row("V1", true), row("V2", true), row("V3", null), row("V4", true), row("V5", false)];
  expect(headline(tally(one))).toBe("3 of the 4 checks we ran pass; V5 missed; V3 not run.");
  expect(headline(tally([row("V1", true)]))).toBe("1 of the 1 check we ran passes.");
});

test("parseScoreboard rejects a malformed file", () => {
  expect(() => parseScoreboard({})).toThrow();
  expect(() => parseScoreboard([{ id: "V1" }])).toThrow();
  expect(parseScoreboard([row("V1", null)])).toHaveLength(1);
  for (const pass of ["false", 1, 0, undefined]) expect(() => parseScoreboard([{ ...row("V1", null), pass }])).toThrow();
});
