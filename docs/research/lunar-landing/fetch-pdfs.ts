// Download open-access PDFs for the NotebookLM sources that were indexed as abstract pages only,
// then extract their text with pdftotext. Output: corpus/pdf/<sourceId8>.pdf and .txt, plus corpus/pdf/MANIFEST.tsv.
// Usage: bun fetch-pdfs.ts
import { mkdirSync, existsSync, writeFileSync } from "node:fs";

const corpus = import.meta.dir + "/corpus/";
const out = corpus + "pdf/";
mkdirSync(out, { recursive: true });
const UA = { "User-Agent": "penumbra-research/0.1 (literature download for personal study)" };

const sources: { id: string; title: string; url: string }[] = JSON.parse(await Bun.file(corpus + "sources.json").text());
const rows = ["source\tpdf_url\tbytes\ttext_chars\tstatus\ttitle"];

for (const s of sources) {
  const len = (await Bun.file(corpus + s.id + ".txt").text()).length;
  if (len >= 12000 || !/ntrs\.nasa\.gov\/citations|arxiv\.org\/abs/.test(s.url ?? "")) continue;
  const key = s.id.slice(0, 8);
  let pdfUrl = "";
  try {
    if (s.url.includes("arxiv.org/abs/")) pdfUrl = s.url.replace("/abs/", "/pdf/");
    else {
      const cid = s.url.split("/citations/")[1];
      const meta = await (await fetch(`https://ntrs.nasa.gov/api/citations/${cid}`, { headers: UA })).json();
      const d = (meta.downloads ?? []).find((x: any) => x.links?.pdf) ?? (meta.downloads ?? [])[0];
      if (d?.links?.pdf) pdfUrl = "https://ntrs.nasa.gov" + d.links.pdf;
      else if (d?.links?.original) pdfUrl = "https://ntrs.nasa.gov" + d.links.original;
    }
    if (!pdfUrl) { rows.push(`${key}\t\t0\t0\tNO_PDF\t${s.title}`); continue; }
    const pdf = out + key + ".pdf";
    if (!existsSync(pdf)) {
      const r = await fetch(pdfUrl, { headers: UA });
      if (!r.ok) { rows.push(`${key}\t${pdfUrl}\t0\t0\tHTTP_${r.status}\t${s.title}`); continue; }
      await Bun.write(pdf, await r.arrayBuffer());
    }
    Bun.spawnSync(["pdftotext", "-layout", pdf, out + key + ".txt"]);
    const txt = existsSync(out + key + ".txt") ? (await Bun.file(out + key + ".txt").text()).length : 0;
    rows.push(`${key}\t${pdfUrl}\t${Bun.file(pdf).size}\t${txt}\t${txt > len ? "ok" : "NO_GAIN"}\t${s.title}`);
  } catch (e) {
    rows.push(`${key}\t${pdfUrl}\t0\t0\tERR ${String(e).slice(0, 80)}\t${s.title}`);
  }
  console.log(rows.at(-1));
}
writeFileSync(out + "MANIFEST.tsv", rows.join("\n") + "\n");
