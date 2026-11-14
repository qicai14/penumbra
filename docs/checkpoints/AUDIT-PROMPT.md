# Checkpoint audit prompt (confirmed 2026-11-14)

Paste this into a fresh session after a usage reset. It is read-only.

> go: checkpoint audit, read-only, no code changes. For every merged PR and closed issue so far:
> 1. With `gh`, compare each PR's diff, body and review comments against its issue's acceptance criteria. List anything left undone, quietly skipped or deferred.
> 2. Flag every place a spec, tolerance, bound or test was loosened to make code pass. #50's ρ widening is already checked (correct; distances tagged U17).
> 3. Have **science-hard** check the flagged science changes against `docs/SCIENCE-SPEC.md` and `docs/EVIDENCE.md`: OK, OK with a citation, or wrong.
> 4. Search past Claude Code session transcripts for roadblocks and mid-course decisions that never reached the docs or ADRs.
> 5. For each finding, give severity, the later tickets it affects, and where the fix goes: a new issue, a comment on an existing open issue, or a docs-only fix.
>
> Append findings to `docs/checkpoints/2026-11-14.md` after each PR so an interrupted run can resume from the last PR listed. File or edit no issues until I approve the list.

## Leads already found
- #48 (needs-triage): shipped tiles miss the 0.05° horizon tolerance from T-1a-4, left over from #49.
- T09 (#10) is closed, but PR #50 says the state-of-charge chart is still fixture data ("FAKE CHARGE DATA" badge). Check whether T09 or T12 (#13) owns wiring the real charge data.
