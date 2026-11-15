import { expect, test } from "bun:test";
import { outsideAvgvisib, rankBests } from "./ranking";

const r = (id: string, h: number | null, lethalH?: number) => ({ id, b: h === null ? null : { operableH: h }, lethalH });

test("rankBests: hours, then later lethal shadow, ties keep order, no-grid last", () => {
  const out = rankBests([r("a", 100, 50), r("none", null), r("b", 700, undefined), r("c", 700, 300), r("d", 700, Infinity), r("zero", 0, 5), r("e", 100, 50)]);
  expect(out.map((x) => x.id)).toEqual(["d", "c", "b", "a", "e", "zero", "none"]);
});

test("rankBests does not mutate its input", () => {
  const a = [r("x", 1), r("y", 2)];
  rankBests(a);
  expect(a.map((x) => x.id)).toEqual(["x", "y"]);
});

test("outsideAvgvisib: north of 85°S only", () => {
  expect(outsideAvgvisib(-84.9)).toBe(true);
  expect(outsideAvgvisib(-85)).toBe(false);
  expect(outsideAvgvisib(-89.5)).toBe(false);
});
