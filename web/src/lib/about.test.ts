import { expect, test } from "bun:test";
import { limits, shownVideos, videoSlots } from "./about";

test("every limitation carries an id, and the #29 lines are all there", () => {
  for (const l of limits) {
    expect(l.id).toMatch(/^(V\d[a-z]?|#\d+|[EU]\d+)$/);
    expect(l.text.trim().length).toBeGreaterThan(0);
  }
  const ids = limits.map((l) => l.id);
  for (const id of ["V2b", "V3", "#75", "#48"]) expect(ids).toContain(id);
  expect(limits.find((l) => l.id === "V2b")!.text).toContain("0.21 d");
  expect(limits.find((l) => l.id === "#75")!.text).toContain("123 sampled pixels above 60 %");
});

test("a video slot without a URL is not shown", () => {
  expect(shownVideos(videoSlots)).toEqual([]);
  expect(shownVideos([{ id: "x", label: "l", url: "  " }])).toEqual([]);
  expect(shownVideos([{ id: "x", label: "l", url: "https://e.org/v" }])).toHaveLength(1);
});
