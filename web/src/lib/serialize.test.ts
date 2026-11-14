import { expect, test } from "bun:test";
import { serialize } from "./serialize";

const tick = () => new Promise((r) => setTimeout(r, 0));

test("calls run one at a time, in order", async () => {
  const log: string[] = [];
  const run = serialize(async (name: string, ms: number) => {
    log.push(`start ${name}`);
    await new Promise((r) => setTimeout(r, ms));
    log.push(`end ${name}`);
    return name;
  });
  const out = await Promise.all([run("a", 20), run("b", 0), run("c", 0)]);
  expect(out).toEqual(["a", "b", "c"]);
  expect(log).toEqual(["start a", "end a", "start b", "end b", "start c", "end c"]);
});

test("a rejection reaches its own caller and the next call still runs", async () => {
  const run = serialize(async (fail: boolean) => {
    await tick();
    if (fail) throw new Error("boom");
    return "ok";
  });
  const bad = run(true);
  const good = run(false);
  await expect(bad).rejects.toThrow("boom");
  expect(await good).toBe("ok");
  expect(await run(false)).toBe("ok");
});
