// Run with `bun test scripts`: make-fixture must not wipe measured scoreboard rows (#63).
import { expect, test } from "bun:test";
import { fileURLToPath } from "node:url";

test("make-fixture keeps measured scoreboard rows", async () => {
  const sb = new URL("../web/public/scoreboard.json", import.meta.url);
  const row = (rows: any[], id: string) => rows.find((e) => e.id === id);
  const before = await Bun.file(sb).json();
  const proc = Bun.spawnSync(["bun", fileURLToPath(new URL("make-fixture.ts", import.meta.url))]);
  expect(proc.exitCode).toBe(0);
  const after = await Bun.file(sb).json();
  for (const id of ["V1", "V4"]) expect(row(after, id)).toEqual(row(before, id));
  expect(row(after, "V1").value).not.toBeNull();
});
