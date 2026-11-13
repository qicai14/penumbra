# Fable science review — kickoff

**When:** Nov 11–12, alongside the scaffold. Finish **before Fri Nov 13 15:00**: it spends this week's quota, which resets then.
**Where:** a **new, separate session** opened in this folder. Not this planning chat, and not a subagent.
**Model:** Fable 5.2, effort **high**. Don't use xhigh or max on a Pro plan; raise the effort only for a single stuck derivation, then drop back.
**Subagents:** allowed. The prompt sends mechanical work to Haiku or Sonnet.
**Rule:** documents only. No code, no mission-data downloads. Papers are research, so they're fine.

## Before Fable (cheap prep, so Fable never reasons over hallucinated numbers)
1. **You:** run `notebooklm login`. The session expired on Nov 10.
2. **Corpus export (no LLM):**
   - Run `cd docs/research/lunar-landing && bun export-corpus.ts`.
   - It writes NotebookLM's own indexed full text for all 60 sources to `corpus/<id>.md`, plus `corpus/INDEX.tsv`.
   - This is exactly what NotebookLM sees, so it can be checked with grep. The folder is gitignored, because the papers are copyrighted.
3. **Haiku agent, the 4 open facts:**
   - IM-2 coordinates from a primary source (an LROC image)
   - AVGVISIB height above the surface
   - NAC label time and projection fields
   - citable IM-1/IM-2 battery and panel values
   - It appends each one to `facts-grill-1.md` with a URL and a verbatim quote.

## Prompt (paste into the Fable session)

```
Review the science plan for Penumbra (NASA Space Apps 2026, Lunar Landing Site Navigator, team of 2, 39-hour event, both on Claude Pro). Read docs/IDEA.md, docs/STACK.md and docs/research/lunar-landing/facts-grill-1.md.

PRE-EVENT RULE: write documents only. No code, no mission-data downloads.

CONTEXT BUDGET: 1M window, auto-compact set at 500K (`/autocompact 500k`). Treat 450K as the limit: checkpoint to SCIENCE-SPEC.md before you reach it.
- Bulk reading (whole papers, multi-file sweeps) goes to subagents, which return verbatim quotes plus file:line. You may read short targeted sections yourself to check a quote or a derivation.
- Your own grep/Bash output: use `grep -n -m5` and `| head -40`.
- USAGE: the parent session watches the 5-hour plan limit. On a "PAUSE AND CHECKPOINT" message, stop new work, write everything to SCIENCE-SPEC.md, and end your turn.
- Write docs/SCIENCE-SPEC.md incrementally, one section at a time, with a "Progress:" line at the top listing the sections that are done.
- After any compaction, re-read only SCIENCE-SPEC.md and this prompt block, then continue from the Progress line.

SUBAGENTS: Delegate mechanical work and keep the derivations and judgment calls yourself. Never spawn a Fable subagent.
- model "haiku": grep, fetching a URL, simple fact lookups
- model "sonnet": summarising one paper section, cross-checking a table
- Each subagent gets a precise question and returns quotes with file paths, not opinions.

EVIDENCE RULES (NotebookLM can misquote even though it is grounded):
- Primary evidence is docs/research/lunar-landing/corpus/: <id>.txt holds NotebookLM's indexed text, and pdf/<name>.txt holds full papers where they were downloaded. INDEX.tsv maps IDs to titles.
- Nov 10: all 26 abstract-only paper sources were replaced with full PDFs, including Mazarico 2011 (the V2 reference), Barker 2016, Mahanti 2024 and Bryant 2009. Their text is in corpus/<id>.txt, and pdf/MANIFEST.tsv and pdf/REPLACED.tsv map the files.
- AVGVISIB's observer height is still [U]. Mazarico 2011 simulates at the surface and at 10 m. Settle the height from that paper or the PGDA readme. Don't read a missing method or number into an abstract. Use the full-paper text, or mark the claim [U].
- Every number, equation or parameter you use needs a verbatim quote. Confirm it with grep in the corpus, and record the file and the quote.
- NotebookLM (notebook 5d807093-69f3-410b-bd6a-898489882eeb) is a LOCATOR only. Use it to find which source discusses something, never as the source of a value.
  - Search passages with `notebooklm source search "<terms>" -n <nb>`.
  - When you ask a question, limit it to the relevant sources with `-s <id>`, repeated per source, and use `--json` to get references.
  - Treat any NotebookLM claim you can't grep in the corpus as unverified.
- Tag each claim [V] (a quote was found) or [U] (nothing quotable). Never invent a value to fill a gap. Name the source that would settle it.
- Don't re-read whole papers. grep first, then read only the matching sections.

DELIVER docs/SCIENCE-SPEC.md, which science-core and validator implement from on Nov 14:

0. Ambition check, brief (/be-ambitious rules: better, not bigger):
   - Is there ONE higher-value outcome, an outside judge, or a method a top lunar-illumination scientist would use that docs/IDEA.md misses?
   - At most 3 proposals, each with its judge, its hours, and what it replaces.
   - If there is none, say so in one line.

1. Equations, with units, frames and the citation for each:
   a. horizon elevation with lunar curvature from a polar-stereographic DEM (projection parameters, reference radius, datum)
   b. the visible fraction of the Sun's disk against a horizon profile
   c. topocentric Sun and Earth az/el in MOON_ME (light time and aberration, MOON_PA→MOON_ME)
   d. Earth elevation margin
   e. the battery state-of-charge time step
   f. Monte Carlo over DEM error
2. The pitfalls most likely to fail V1–V5, ranked, each with the test that catches it. Examples:
   - Horizons observer settings
   - ME vs PA frames
   - DEM frame and datum
   - the 0.53° disk
   - pixel-centre conventions
   - libration
   - UTC/TDB
3. Whether each V1–V5 pass threshold is reachable, with an honest target for each.
   - V2: PGDA AVGVISIB is an hourly average over 18.6 years. State what span, height and sampling we must match, and the compute cost.
4. Scope recalibrated for 2 people × ~16 working hours on Pro limits:
   - Re-rank Must/Should/Stretch by (judge impact × confidence) ÷ hours.
   - Name what to cut first if "deployed by Sat 12:00" slips.
   - Give docs/IDEA.md edits as a diff list. Do NOT apply them.
5. Open facts, each with the exact source that would settle it.

Outside judge: a reviewer must be able to grep every [V] quote in the corpus. Finish with a count: N claims [V], M claims [U].
```

## After
- Back in a normal session on Opus medium: apply the docs/IDEA.md diff list (after you approve it). Also fix the subagent prompts if SCIENCE-SPEC.md changes who owns what.
