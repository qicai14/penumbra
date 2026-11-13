# Workflow

## Nov 11–13 (pre-event: planning, design and tooling only)
- Charls: `/design-pass` (issue "Design pass"), then the other design issues assigned to him.
- Qi Cai: docs/SCIENCE-SPEC.md with Fable.

## Nov 13, after 15:00
1. Read the rubric, the submission guide, the Resources tab and the collaborator offers.
2. Diff them against docs/IDEA.md, docs/SUBMISSION.md and docs/PLAN-DRAFT.md; propose edits as a PR.
3. Add the new datasets to the NotebookLM notebook.
4. Choose one lo-fi direction (A, B or C) per screen from `design/wireframes-v2/` (PR #24) and log it in design/DECISIONS.md; skip the 3D terrain screen (cut 1).

## Nov 14, 09:00
1. Create the public repo `penumbra` as a fresh-history export of this one without `docs/research/**/corpus/`, and add the Apache-2.0 LICENSE there. This burner repo stays private.
2. `/to-tickets` only (don't re-plan; HANDOFF has the message): tracer-bullet issues with blocking edges and owner labels (start from docs/PLAN-DRAFT.md).
3. Charls starts from the screens picked in design/DECISIONS.md (wireframes-v2) and turns them into web/ tickets.

## Per ticket
- A new Claude Code session → `/ticket <n>`. One issue per session.

## Budget (Claude Pro, 5-hour windows)
- Orchestrator: Opus, effort medium. No Opus high by default.
- Fable only through the `science-hard` agent.
- Check `/usage` at the end of every ticket; above 40% context, `/handoff` and start fresh.
- Stagger our 5-hour windows so one of us always has quota.

## Milestones
- Deployed: Sat 12:00 · feature freeze: Sun 15:00 · demo recorded: Sun 17:00 · submitted: Sun 21:00 · hard deadline: Sun 22:00 (NSAKL26 Telegram, Nov 14).

## Ship checklist
- docs/KICKOFF.md
