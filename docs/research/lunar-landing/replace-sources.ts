// Replace abstract-only NotebookLM sources with their full PDFs (from fetch-pdfs.ts).
// For each source: upload the PDF, wait until it's ready, and only then delete the old abstract source.
// Log: corpus/pdf/REPLACED.tsv. Usage: bun replace-sources.ts
import { appendFileSync, existsSync } from "node:fs";

const NB = "5d807093-69f3-410b-bd6a-898489882eeb";
const corpus = import.meta.dir + "/corpus/";
const pdfDir = corpus + "pdf/";
const log = pdfDir + "REPLACED.tsv";
const sources: { id: string; title: string }[] = JSON.parse(await Bun.file(corpus + "sources.json").text());
const done = existsSync(log) ? await Bun.file(log).text() : "";
// Already uploaded in a test run: fc4450fc -> e1760284 (still needs its old source deleted).
const preUploaded: Record<string, string> = { fc4450fc: "e1760284-8bd7-4131-af83-a948a9a8d33f" };

const nb = (args: string[]) => {
  const p = Bun.spawnSync(["notebooklm", ...args, "-n", NB], { stdout: "pipe", stderr: "pipe" });
  return { code: p.exitCode, out: p.stdout.toString(), err: p.stderr.toString() };
};

const manifest = (await Bun.file(pdfDir + "MANIFEST.tsv").text()).trim().split("\n").slice(1).map((l) => l.split("\t"));
for (const [key, url, , , status] of manifest) {
  if (!url || done.includes(key + "\t")) continue;
  if (status !== "ok" && key !== "6e8bcb20" && key !== "d5686fb8") continue; // keep the scanned PDF and the docx
  const old = sources.find((s) => s.id.startsWith(key))!;
  const ext = url.endsWith(".docx") ? "docx" : "pdf";
  const file = pdfDir + key + "." + ext;
  if (ext === "docx" && !existsSync(file)) await Bun.write(file, Bun.file(pdfDir + key + ".pdf")); // fetch saved it as .pdf
  const title = old.title.replace(/ - NASA Technical Reports Server \(NTRS\)$/, "") + " [full text]";
  let newId = preUploaded[key];
  if (!newId) {
    const add = nb(["source", "add", file, "--type", "file", "--title", title, "--timeout", "180", "--json"]);
    try { newId = JSON.parse(add.out.slice(add.out.indexOf("{"))).source.id; } catch { appendFileSync(log, `${key}\t\tADD_FAIL ${add.err.slice(0, 100)}\n`); continue; }
  }
  const w = nb(["source", "wait", newId, "--timeout", "300"]);
  if (w.code !== 0) { appendFileSync(log, `${key}\t${newId}\tWAIT_FAIL code=${w.code}; old kept\n`); continue; }
  const d = nb(["source", "delete", old.id, "-y"]);
  appendFileSync(log, `${key}\t${newId}\t${d.code === 0 ? "replaced" : "DELETE_FAIL"}\t${title}\n`);
  console.log(key, newId, d.code === 0 ? "replaced" : "DELETE_FAIL");
}
