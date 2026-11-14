# Handoff: start every new session here

Status as of **Sun Nov 15, morning**. Public repo `qicai14/penumbra`; the private planning repo `qicai14/penumbra-burner` stays private. Team **Fractured Toes**: Qi Cai = owner and science; Charls (`ambatuCode887`) = app and design. The project is **Penumbra**; only the deploy URL is `penumbra-lunar.pages.dev`.

## Clock
| When | What | Source |
|---|---|---|
| Sun 08:30 / 09:00 / 13:00 | Review, project page opens, proofread | [SUBMISSION](SUBMISSION.md) |
| **Sun 15:00** | **Freeze (G5):** after it, D-tickets only | [checkpoint 2](checkpoints/2026-11-15.md) |
| Sun 17:00 / 21:00 | Demo, submitted | [WORKFLOW](WORKFLOW.md) |
| **Hard deadline** | **Sun Nov 15, 22:00** (NSAKL26 Telegram, Nov 14) | SUBMISSION |

## Where we are
- **All T-tickets are closed except T24 (#25, region heatmap, in progress) and T25 (#26, needs-info: no cited region centres).**
- **Scoreboard** (`web/public/scoreboard.json`): V1, V2, V4, V5 pass. V2b fails honestly on SP-01 (inside Mazarico's 6 h step, reported as a fail, nothing tuned). V3 not run: no NAC images were downloaded.
- **#48 merged (PR #86):** the three-tier horizon march (20 m near, 80 m middle to 31 km, 240 m far) is the only march; T-1a-4 holds at 0.05°.
- **Gate G2 passed via U18** (no separate kernel benchmark): median horizon times are 13–34 ms across native, WASM and browser, under the 50 ms target (one WASM SP-01 call reached 62 ms), so slip cut 3 is not needed (DATA-NOTES U18).
- Checkpoint audits: [Nov 14](checkpoints/2026-11-14.md), [Nov 15](checkpoints/2026-11-15.md). Their docs-only leftovers closed in #78.
- After the event: #77 (V2 follow-ups); the notes from #62, #64, #69, #72/#73 are recorded with no action (checkpoint 2).

## Ticket order (checkpoint 2 and the C3 plan)
1. T24 cut A (#25), then cut B. T25 (#26) only if T24 A + B merged by 12:00.
2. Checkpoint 3 UI, before freeze: #79 four views first, then #80, #81, #82; #83 Should; #84 phone and impeccable audit last; #85 tour only if #79–#84 merged.
3. After freeze, D-tickets: D1 (#29) doc, D2 poster, D3/D4 videos, D5–D7 links and page fields.

The Nov 14 session order (T05 V1 #6 and T10 V5 #11 last, behind T07–T09) is done: both are closed.

## Next prompt
> /ticket 25 (T24)

Each ticket gets its own session: `/ticket <n>` (or `/design-pass` for design work). Pull `main` first.

## Why `/ticket` and not `/implement` or `/implement-spec`
`/ticket` is this repo's skill (`.claude/skills/ticket/`): it stops on open blockers, gives each issue its own worktree, routes to our agents (`science-core`, `validator`, `frontend-3d`, `science-hard`, `designer`) with SCIENCE-SPEC ids as pointers, runs `reviewer`, and opens a "Closes #n" PR with no AI attribution. `/implement-spec` merges through one integration branch with parallel subagents, which burns a Claude Pro window fast and skips the per-issue PRs.

## Rules
- Never push `burner-main` or any branch made from it to `origin`; never `git push --all` or `--mirror` (the hook blocks both). `/ticket` branches from `origin/main`, so it is safe.
- `main` needs checks `test` and `build`; repo auto-merge is on.
- Charls has no corpus; his researcher agent uses NotebookLM (docs/EVIDENCE.md).

## Model policy
Opus, effort medium, during the event. Fable only if truly stuck (the `science-hard` agent, or a `hard-science` ticket).
