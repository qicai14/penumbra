# Handoff: start every new session here

Status as of **Sat Nov 14 (early morning)**. Planning repo `qicai14/penumbra-burner` is private and stays private; event work moves to the public `qicai14/penumbra` (not created yet, steps below). Team **Fractured Toes**: Qi Cai = owner and science; Charls (`ambatuCode887`) = app and design. The project is **Penumbra**; only the deploy URL is `penumbra-lunar.pages.dev`.

## Timeline (where each clock lives)
| When | What | Source |
|---|---|---|
| Sat 09:00–10:00 | Merge PRs #28 and #29, create the public repo (below), copy data, file tickets, API freeze 09:10 | PLAN-DRAFT §5 |
| Sat and Sun, hour by hour | Who does which ticket when | PLAN-DRAFT §3 |
| Gates and triggers | G2 12:30 (cut 3), G1 14:00, freeze Sun 15:00 | PLAN-DRAFT §3 "Gates", §4 "Time triggers" |
| Event checkpoints | Footage Sat 13:00, review Sun 08:30, page opens Sun 09:00, proofread Sun 13:00 | [SUBMISSION](SUBMISSION.md) |
| Milestones | Deployed Sat 12:00, freeze Sun 15:00, demo Sun 17:00, submitted Sun 21:00 | [WORKFLOW](WORKFLOW.md) |
| **Hard deadline** | **Sun Nov 15, 22:00** (NSAKL26 Telegram, Nov 14; the global guide's 23:59 does not apply to us) | SUBMISSION |

## Done
- Template scaffolded (PR #1): Rust workspace, Svelte 5 + Vite, CI, hooks, agents; deploys work (Cloudflare project `penumbra-lunar`).
- Docs: [IDEA](IDEA.md), [SCIENCE-SPEC](SCIENCE-SPEC.md), [STACK](STACK.md), [PLAN-DRAFT](PLAN-DRAFT.md) (T00–T27, D1–D7), [ARCHITECTURE](ARCHITECTURE.md), [SUBMISSION](SUBMISSION.md), [WORKFLOW](WORKFLOW.md), [briefs](briefs/README.md). ADRs 0001–0006 accepted. Precedence: ADRs > ARCHITECTURE > SCIENCE-SPEC > PLAN-DRAFT > IDEA > STACK.
- **Stack preflight (Nov 12, PR #25):** ANISE 0.10.6 loads the three kernels; `tiff` 0.11.3 reads the DEM headers; 20 MB deploy served in 1.3 s (Q5); U6, U14 closed.
- **Resources tab and judging review (Nov 14, PR #28):**
  - Resources tab = QuickMap, Moon Trek, Horizons, CSA Moon Exploration; no new dataset.
  - All 10 `MANIFEST.tsv` files still match their source (HTTP 200, same size, Last-Modified before our download).
  - Global judging = 5 criteria × 1–5 (SUBMISSION.md "Global judging"); References must list all code, fonts and images.
  - T27 compare promoted to Should (PLAN-DRAFT Q10).
  - NotebookLM 5d807093 now has 62 sources (+ CSA).
- **Design picks (Nov 14, PR #28, design/DECISIONS.md):** Site v2 B, Planner v2 B, Terrain v2 C, Replay v2 A, Scoreboard v2 C, each with fixes. New screen 6 "Compare A vs B" to design (Charls, `/design-pass`).
- **Agents and tooling (Nov 14, PR #29):**
  - science-core owns `crates/wasm` too.
  - Tests are the oracle: never weakened without id, reason and source; the reviewer blocks it.
  - fmt/clippy commands everywhere match CI.
  - Workspace clippy lints for float compares and int casts.
  - Project plugins: ponytail (coding agents only) and humanizer.
  - Official `svelte-core-bestpractices` skill for frontend-3d.
  - Deny `gh repo delete`; ask before `gh pr merge`.
- Brand: Axiom style (PR #24). Use of AI: no write-up; disclose on the form only if it asks.

## Left for Saturday (by design, not forgotten)
- T02 first 20 min: ANISE Sun/Earth positions against one Horizons request.
- T04: decode one `tiff` tile. GDAL fallback: QGIS on Qi Cai's laptop.
- PLAN-DRAFT Q10: which hour pays for T27 (T24 heatmap or D2 polish).

## Create the public repo (Qi Cai, Sat 09:00, same folder)
**Status (Nov 13 night):** steps 1–6 are done in this folder. `main` is the single public commit, `origin` = `qicai14/penumbra` (created, empty), `burner` = the private repo, and the old `main` is `burner-main`. Left: step 7 (Qi Cai, after 09:00), then a new session for steps 8–9.

Why this way: one folder, the corpus stays on disk for local research, and the public repo starts from one commit with no burner history. `main` is a new **orphan** branch, so nothing from the burner's history (where the corpus lives) can reach the public repo.

Do it in a Claude session in this folder: paste "Create the public repo: follow HANDOFF.md 'Create the public repo' steps 1–6, stop before step 7". **Step 7 you type yourself**: the git guard hook blocks Claude from pushing to `main`, on purpose.

1. Merge PRs #28 and #29 on GitHub, then `git switch main && git pull`. The tree must be clean (`git status`).
2. Keep the burner reachable: `git remote rename origin burner` and `git branch -m main burner-main`.
3. Start the new history in place (files stay on disk): `git checkout --orphan main`.
4. Untrack the corpus (it stays on disk; `.gitignore` already ignores it): `git rm -r --cached --quiet docs/research/lunar-landing/corpus`.
5. Commit: `git commit -m "Penumbra: initial public commit"`.
6. Check before anything leaves the machine:
   - `git log --oneline` prints exactly 1 line.
   - `git ls-files | grep /corpus/` prints nothing.
   - `git ls-files | grep -i -E '\.(pdf|tif|bsp|bpc)$'` prints nothing.
   - `gh repo create qicai14/penumbra --public --description "Penumbra: lunar south-pole landing survival planner"` (no `--source`/`--push`)
   - `git remote add origin https://github.com/qicai14/penumbra.git`
   - `gh repo set-default qicai14/penumbra`
7. **You type:** `git push -u origin main`.
8. Set up the new repo: `REPO=qicai14/penumbra bash scripts/wizards/owner-setup.sh` (Cloudflare secrets, `CF_PAGES_PROJECT=penumbra-lunar`, Pages project, branch protection, invite `ambatuCode887`). Then create the labels: owner:science, owner:app, owner:design, hard-science, ready-for-agent, in-progress (PLAN-DRAFT §5 09:05).
9. Check: CI green on `main`, and `penumbra-lunar.pages.dev` serves the hello-world from the new repo.

Rules from now on:
- Never push `burner-main` or any branch made from it to `origin`. `/ticket` branches from `origin/main`, so it is safe.
- Never `git push --all` or `--mirror`; the hook blocks both.
- Charls clones `qicai14/penumbra` fresh. He has no corpus, so his researcher agent uses NotebookLM (docs/EVIDENCE.md).

## Sat 09:10: first message after the repo is live (paste as is)
> /mattpocock-skills:to-tickets File the tickets in docs/PLAN-DRAFT.md §2 (T00–T27 and D1–D7) as GitHub issues in qicai14/penumbra with labels, owners and blocking edges. T27 is Should, not overflow (Q10). PLAN-DRAFT is the spec, so no /to-spec. Don't re-plan.

Then each ticket goes in a new session: `/ticket <n>`.

## Why `/ticket` and not `/implement` or `/implement-spec`
`/ticket` is this repo's skill (`.claude/skills/ticket/`):
- it stops on open blockers
- gives each issue its own worktree
- routes to our agents (`science-core`, `validator`, `frontend-3d`, `science-hard`) with SCIENCE-SPEC ids as pointers
- runs `reviewer`, which runs Matt Pocock's code-review
- opens a "Closes #n" PR with no AI attribution, and checks usage

`/implement` is a short generic loop (tdd, code-review, commit) that `/ticket` already contains. `/implement-spec` merges everything through one integration branch with parallel subagents; that burns a Claude Pro window fast and skips our per-issue PRs and owner split.

## Model policy
Opus, effort medium, during the event. Fable only if truly stuck (the `science-hard` agent).
