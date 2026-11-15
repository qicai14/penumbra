import { describe, expect, test } from "bun:test";
import { NODATA, type Grid } from "./grid";
import { deltaDays, fmtDelta, withinRadius } from "./whatChanges";

// 5x5 grid, 100 m cells, x0 = 0, y0 = 500; cell (i, j) centre is (100 i + 50, 450 - 100 j).
const grid = (cells: Record<string, number>): Grid => {
  const a = new Uint16Array(25).fill(240);
  for (const [k, v] of Object.entries(cells)) { const [i, j] = k.split(",").map(Number); a[j * 5 + i] = v; }
  return { meta: { region: "t", pixel_m: 100, x0_m: 0, y0_m: 500, width: 5, height: 5, span_days: 30, mast_m: 2, dates: ["d"], landing_unix_s: [0], url: "" }, operable: [a], lethal: [new Uint16Array(25)] };
};

describe("deltaDays and fmtDelta", () => {
  test("hours convert to signed days", () => {
    expect(deltaDays(312, 240)).toBe(3);
    expect(deltaDays(216, 240)).toBe(-1);
    expect(fmtDelta(3)).toBe("+3.0 d");
    expect(fmtDelta(-1)).toBe("−1.0 d");
    expect(fmtDelta(0.02)).toBe("0.0 d");
  });
});

describe("withinRadius", () => {
  test("best and worst among cells whose centre is within 300 m, own cell included", () => {
    const g = grid({ "2,2": 200, "3,2": 480, "0,0": 960, "4,2": 0 }); // (3,2) is 100 m away; (4,2) is 200 m; (0,0) is 283 m away
    const n = withinRadius(g, 0, 250, 250)!;
    expect(n.hereH).toBe(200);
    expect(n.bestH).toBe(960); // (0,0): hypot(200,200) = 283 m, inside
    expect(n.worstH).toBe(0);
    expect(n.partial).toBe(true); // a 500 m grid cannot hold a 300 m disc around its middle
  });
  test("a cell just beyond 300 m is ignored", () => {
    const g = grid({ "2,2": 200, "0,2": 960 }); // 200 m away: in
    expect(withinRadius(g, 0, 250, 250)!.bestH).toBe(960);
    const far = grid({ "2,2": 200, "0,0": 960 });
    expect(withinRadius(far, 0, 250, 250, 250)!.bestH).toBe(240); // (0,0) is 283 m away, out at 250 m
  });
  test("no-data cells are skipped; own no-data or off-grid gives null; a disc past the edge is partial", () => {
    const g = grid({ "2,2": 200, "3,2": NODATA, "1,2": 100 });
    const n = withinRadius(g, 0, 250, 250)!;
    expect(n.worstH).toBe(100);
    expect(withinRadius(grid({ "2,2": NODATA }), 0, 250, 250)).toBeNull();
    expect(withinRadius(g, 0, -10, 250)).toBeNull();
    expect(withinRadius(g, 0, 50, 450)!.partial).toBe(true);
  });
});
