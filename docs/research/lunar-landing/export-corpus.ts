// Export the full text of every NotebookLM source in the Lunar notebook to ./corpus/,
// so that facts can be checked with grep against what NotebookLM actually indexed.
// Usage (after `notebooklm login`): bun export-corpus.ts
import { mkdirSync, writeFileSync } from "node:fs";

const NB = "5d807093-69f3-410b-bd6a-898489882eeb";
const dir = import.meta.dir + "/corpus/";
mkdirSync(dir, { recursive: true });

const run = (args: string[]) => {
  const p = Bun.spawnSync(["notebooklm", ...args], { stdout: "pipe", stderr: "pipe" });
  return { ok: p.exitCode === 0, out: p.stdout.toString(), err: p.stderr.toString() };
};

const list = run(["source", "list", "-n", NB, "--json"]);
const parsed = JSON.parse(list.out.slice(list.out.indexOf("{"))); // CLI may print a "Matched:" line first
if (parsed.error) throw new Error(parsed.message);
const sources: { id: string; title?: string }[] = Array.isArray(parsed) ? parsed : parsed.sources;
writeFileSync(dir + "sources.json", JSON.stringify(sources, null, 2));

const index: string[] = ["id\ttitle\tstatus"];
for (const s of sources) {
  const r = run(["source", "fulltext", s.id, "-n", NB, "-o", `${dir}${s.id}.txt`, "--force"]);
  index.push(`${s.id}\t${s.title ?? ""}\t${r.ok ? "ok" : "FAIL " + r.err.trim().slice(0, 120)}`);
  console.log(index.at(-1));
}
writeFileSync(dir + "INDEX.tsv", index.join("\n") + "\n");
