# Fable session 3: draft spec and tickets

**Runs after:** docs/FABLE-SPEC-CHECK.md has finished and you've approved its fixes, which are then applied to SCIENCE-SPEC.md.
**Model:** Fable 5.2, effort high, 1M context. Subagents allowed (haiku or sonnet only). Documents only.
**Output:** `docs/PLAN-DRAFT.md`. On Nov 13, revise it against the Resources tab and the rubric. On Nov 14 at 09:00, file it as GitHub issues.

```
Draft the build plan for Penumbra (NASA Space Apps 2026, Lunar Landing Site Navigator; team of 2: A = science core, B = app; ~16 working hours each; both on Claude Pro; the event runs Nov 14 09:00 to Nov 15 23:59). It's a draft: GitHub issues are filed only on Nov 14.

Read: docs/IDEA.md, docs/SCIENCE-SPEC.md (review fixes F1–F22 applied), docs/SCIENCE-SPEC-REVIEW.md, docs/STACK.md, docs/SUBMISSION.md (local deliverables, rubric and event-day checkpoints), SCAFFOLD-KICKOFF.md (repo layout, agents, labels, the /ticket workflow), and design/ if it exists.
The plan must include submission tickets (D1–D7 in SUBMISSION.md) with owners and times. Respect the event checkpoints: footage by Sat 13:00, proofreading Sun 13:00. Use "Qi Cai" (owner:science) and "Charls" (owner:app/design) by name.
Read the mattpocock skills' formats for to-spec and to-tickets (the plugin's SKILL.md files), and use their structure.

CONTEXT AND USAGE: as in docs/FABLE-SPEC-CHECK.md. Write incrementally with a Progress line, and honour PAUSE AND CHECKPOINT.

DELIVER docs/PLAN-DRAFT.md:
1. Spec, in /to-spec shape: problem, users, the outcome, the modules and their interfaces (Rust core, pipeline, wasm, web), data artefacts and their binary formats, non-goals.
2. Tickets in /to-tickets shape. These are tracer bullets: the first ticket goes end-to-end (one site, one date, real numbers to a deployed page) before anything gets wide. Each ticket has:
   - a title
   - an owner label (owner:science / owner:app / owner:design), plus hard-science where it applies
   - blocking edges (by ticket number)
   - an estimate in hours
   - the files and paths it touches
   - done-criteria that name the V-test or SCIENCE-SPEC example it must pass
3. A schedule: both people's tickets laid onto Sat and Sun hour blocks, with the critical path marked, "deployed by Sat 12:00" and freeze at Sun 15:00. Stagger the two people's heavy Claude usage so their 5-hour windows don't peak together.
4. Pre-written slip cuts, mapped to tickets (from SCIENCE-SPEC §4).
5. Day-1 checklist: the first 60 minutes on Nov 14, for each person.

Rule: no code, only ticket text. Mark every estimate you couldn't ground as [est]. Finish with a ≤10-line summary.
```
