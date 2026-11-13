# Fable session 2: independent check of SCIENCE-SPEC.md

**Model:** Fable 5.2, effort high, 1M context, auto-compact around 500–600K. Subagents allowed (haiku or sonnet only).
**Rule:** documents only. No code, no mission-data downloads (Space Apps pre-event rule).
**Output:** `docs/SCIENCE-SPEC-REVIEW.md`. Don't edit SCIENCE-SPEC.md itself.

```
You are an independent referee for docs/SCIENCE-SPEC.md, the science spec for Penumbra (NASA Space Apps 2026, Lunar Landing Site Navigator). Another session wrote it and graded its own work. Your job is to find what it got wrong before two people build on it in a 39-hour hackathon. Assume there are errors.

Read: docs/SCIENCE-SPEC.md, docs/IDEA.md, docs/STACK.md. Evidence: docs/research/lunar-landing/corpus/. INDEX.tsv maps IDs to titles; pdf/ holds the full papers and their text. NotebookLM 5d807093 is a locator only; never take a value from it.

CONTEXT AND USAGE
- Bulk reading goes to haiku/sonnet subagents. They return verbatim quotes plus file:line, at most 15 lines. Never spawn fable subagents.
- Write docs/SCIENCE-SPEC-REVIEW.md incrementally, with a "Progress:" line at the top.
- On a "PAUSE AND CHECKPOINT" message, stop new work, write everything to the review file (Progress plus Resume notes), and end your turn.

CHECKS, in priority order
1. Re-derive independently, from the papers rather than from the spec's text:
   a. horizon elevation with curvature
   b. visible Sun-disk fraction
   c. topocentric Sun/Earth az/el, including frames, light time and aberration
   d. the battery state-of-charge step
   e. the sunset-time uncertainty band
   For each, do a units/sign/frame check plus one hand-worked numeric example with the inputs and expected output. A validator can turn that example into a test.
2. Audit the evidence:
   - Use a script to grep-check a random 30% of the [V] ledger rows at their file:line.
   - Check whether each quote actually supports the claim it's cited for, not just whether it exists.
   - List any [V] that should be [U].
3. Thresholds (V1–V5): attack each feasibility argument. Is the target honest? Is the comparison like-for-like (metric, height, span, sampling, projection)?
4. Pitfalls: what's missing from §2? Rank any additions by how likely they are to fail a V-test.
5. Scope (§4): sanity-check the hours and the dependency order for 2 people × ~16 h.

FORMAT for each finding: severity (BLOCKER / MAJOR / MINOR), the spec location, what's wrong, evidence (quote plus file:line, or a derivation), and the exact fix as replacement text.

End with a verdict: "spec usable as-is", "usable after fixes", or "redo section X". Include counts by severity. Then send a ≤10-line summary.
```
