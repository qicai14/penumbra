import { describe, expect, test } from "bun:test";
import { fmtLength, maxElevation, shadowLengthM } from "./moonVsEarth";

describe("shadowLengthM", () => {
  test("matches 2 / tan(el)", () => {
    for (const el of [1, 1.5, 5, 30, 45, 89])
      expect(shadowLengthM(el)!).toBeCloseTo(2 / Math.tan((el * Math.PI) / 180), 9);
  });
  test("45 degrees gives the rock's height", () => expect(shadowLengthM(45)!).toBeCloseTo(2, 9));
  test("1 degree is about 115 m", () => expect(shadowLengthM(1)!).toBeCloseTo(114.6, 1));
  test("Sun at or below the horizon has no finite shadow", () => {
    expect(shadowLengthM(0)).toBeNull();
    expect(shadowLengthM(-3)).toBeNull();
    expect(shadowLengthM(NaN)).toBeNull();
  });
});

describe("maxElevation", () => {
  test("picks the highest, null when empty", () => {
    expect(maxElevation(Float64Array.from([-2, 1.4, 0.3]))).toBe(1.4);
    expect(maxElevation([])).toBeNull();
  });
});

test("fmtLength", () => {
  expect(fmtLength(114.6)).toBe("115 m");
  expect(fmtLength(4.44)).toBe("4.4 m");
});
