# Scaffold log (unattended run, Nov 11)

Progress: COMPLETE (Nov 11). Steps 1–11 done; PR #1 is green but unmerged, and the live URL and /ticket loop wait on the items below.

## Needs Qi Cai
- **Review and merge PR #1** (https://github.com/qicai14/penumbra-burner/pull/1). CI is green (test, build with smoke, deploy skipped cleanly). The auto-mode classifier refused to let me merge without a human review.
- After the merge: `/ticket 7` (throwaway issue: add core::sub with a test) to prove the issue → branch → review → PR → merge loop end to end.
- Run `bash scripts/wizards/owner-setup.sh` (6 stages): Cloudflare token + account id (→ .env and GitHub secrets), create the Pages project, merge the permission allowlist into .claude/settings.json, protect main (PR + CI `test`/`build`), confirm Charls's access (he is already a collaborator). Then `gh workflow run CI --ref main` for the live URL.
- Permission allowlist: the auto-mode classifier refused to let me write `permissions.allow` into .claude/settings.json (self-modification). Wizard stage 4 does it; commit the result via a PR.
- Check these decisions:
  - GLOSSARY "scenario" = site + date + lander preset. I defined it because no source does.
  - CHARLS-START.md lists F2 (BUSINESS.md) and F3 (story scripts), but the kickoff's issue list doesn't. I filed only the kickoff's 5 issues (#2–#6), so decide whether F2 and F3 become issues. Also decide whether CHARLS-START.md goes, now that ONBOARDING.md plus the issues replace it.
  - Our copy of review-animations has `disable-model-invocation` removed so the designer agent can preload it. ponytail-review is copied in rather than enabling the ponytail plugin for the project.
  - TypeScript is pinned to ~6 for svelte-check.
- Removed in the pre-event docs PR: AUTOMATION-RUNBOOK.md, SCAFFOLD-KICKOFF.md, SCAFFOLD-UNATTENDED.md and FABLE-ARCH-REVIEW.md (root); they stay in git history.
- Installed locally outside the repo: wasm-bindgen-cli 0.2.130 (cargo) and Playwright chromium-headless-shell v1254.

## Handoff (focus: Nov 13–14)
- State: PR #1 (`scaffold`) holds the whole template. Issues #2–#4 and #6 are assigned to Charls, #5 (paper) to you, and #7 is the throwaway /ticket proof. Labels exist (owner:*, hard-science, in-progress, triage five).
- Nov 13 after 15:00: follow docs/WORKFLOW.md. Rubric/Resources diff → docs/IDEA.md + docs/STACK.md; review design/DECISIONS.md from Charls's #2.
- Nov 14 09:00: public `penumbra` from a fresh-history export of this repo without `docs/research/**/corpus/` + Apache-2.0 LICENSE; this burner repo stays private. Re-run `scripts/wizards/owner-setup.sh` there, since secrets, Pages project and protection are per repo, and the wizard's REPO variable must change. Then `/to-spec` → `/to-tickets` from docs/PLAN-DRAFT.md.
- Suggested skills next session: mattpocock-skills:to-spec, to-tickets (user-invoked), wizard, pr.
- Unverified: the Cloudflare deploy path, including wrangler-action v4 outputs and the deployed smoke test, has never run, because no secrets exist yet. The first `gh workflow run CI --ref main` after the wizard is its test.

## Resume notes
- Work happens on branch `scaffold` in this folder. Root AUTOMATION-RUNBOOK.md and SCAFFOLD-*.md stay at root (the parent session reads them); never commit parent edits to AUTOMATION-RUNBOOK.md.

## Step diffs
### Step 1 (repo) — commit 3e3074d
- Moved IDEA.md, IDEA-brief.md, KICKOFF.md, FABLE-*.md and research/ into docs/; rewrote plain-text paths (IDEA.md → docs/IDEA.md, research/ → docs/research/) in all docs except PLAN-DRAFT.md (not to be edited) and SCAFFOLD-KICKOFF.md (the brief).
- Kept at root: CHARLS-START.md (paths fixed).
- .gitignore: corpus pattern moved to docs/research/**/corpus/pdf/*.pdf; `git status --ignored` shows only the 26 PDFs and .claude/ local files ignored; corpus .txt/.tsv stay tracked. Added .env.example (names only) and .gitattributes (LF everywhere, so hooks and .sh run in Git Bash and CI).
### Step 2 (agent config)
- AGENTS.md (27 lines: pitch, toolchain, layout, /ticket rule, doc pointers); CLAUDE.md = `@AGENTS.md`.
- Nested crates/, web/, design/ AGENTS.md (5–6 lines each) plus a `@AGENTS.md` CLAUDE.md beside each, because Claude Code loads nested CLAUDE.md, not AGENTS.md.
- docs/EVIDENCE.md (4 rules). No symlinks (Windows git).
### Step 3 (Matt Pocock workflow)
- /setup-matt-pocock-skills done by hand (skill is user-invoked only): GitHub Issues, default triage labels, single-context. Wrote docs/agents/{issue-tracker,triage-labels,domain}.md; domain.md points at docs/GLOSSARY.md (the kickoff path) instead of a root GLOSSARY.md. The "## Agent skills" block went into AGENTS.md, since CLAUDE.md must stay `@AGENTS.md`.
- `claude plugin install mattpocock-skills@claude-plugins-official --scope project` → .claude/settings.json enabledPlugins.
- docs/GLOSSARY.md seeded with the 8 kickoff terms plus region, illumination, visibility. "Scenario" is not defined in IDEA/SCIENCE-SPEC; I defined it as site + date + lander preset (check it).
- GitHub labels created: needs-triage, needs-info, ready-for-agent, ready-for-human, owner:science, owner:app, owner:design, hard-science, in-progress (wontfix existed).
### Step 4 (subagents)
- .claude/agents/: science-core, validator, frontend-3d, designer, researcher, reviewer, science-hard, with the kickoff's model/effort/skills table. Each prompt: role, owned paths, docs to read, done-criteria; each description says when to use it.
- Plugin skills are preloaded as `mattpocock-skills:<name>` (plugin skills are namespaced). reviewer and science-hard have Edit/Write/NotebookEdit disallowed; researcher has a read-only tool list (no Edit/Write).
- science-core's prompt treats SCIENCE-SPEC test ids as the pre-agreed tdd seams, because a subagent can't ask the user to confirm seams.
### Step 5 (project skills)
- .claude/skills/ticket/SKILL.md and .claude/skills/design-pass/SKILL.md, both user-invoked (`disable-model-invocation: true`), arguments via $0.
- /ticket can't call /implement or /handoff itself: both are user-only skills in mattpocock-skills (disable-model-invocation). So /ticket dispatches the owning agent (which preloads tdd) and tells the user to run /handoff above 40% context.
### Step 6 (skills)
- Installed at project scope as real folders (`npx skills add … -a claude-code --copy`): impeccable, emil-design-eng, review-animations, animation-performance, threejs-fundamentals, svelte-code-writer, playwright-cli, notebooklm. Provenance + hashes in skills-lock.json.
- Extra skills the agents preload: find-docs (upstash/context7, researcher) and ponytail-review (copied from the MIT ponytail plugin with its LICENSE, so the reviewer works in Charls's clone without enabling the whole ponytail plugin and its session hooks).
- notebooklm: the installer copied the whole repo (124 MB, 2,227 files); trimmed to SKILL.md + LICENSE (its references are GitHub URLs).
- review-animations ships `disable-model-invocation: true`, which stops the designer agent from preloading it; removed that line in our copy. Re-adding it on `skills update` would silently undo the preload.
- impeccable's `scripts/impeccable context` downloads a checksum-pinned engine binary on first run; I did not run it.
- /find-skills: "rust" → apollographql/skills@rust-best-practices (22.7K installs, Apollo) installed. "wasm-bindgen" (best 115 installs) and "cloudflare pages" (best 359) had nothing at 1K+, so nothing installed.
- Read: every SKILL.md header and a scan for install/network/credential instructions; impeccable SKILL.md read in full. No MCP servers added.
### Step 9 (hello-world), done before 7–8 because bootstrap needs the pinned versions
- Cargo workspace (edition 2024): penumbra-core `add()` + test, penumbra-pipeline (native bin stub), penumbra-wasm (wasm-bindgen `=0.2.130`, exact pin so the CLI matches).
- Root package.json (bun workspace over web/) with `wasm`, `dev`, `build`, `check`, `test`, `smoke`. web/: Svelte 5 + Vite 8, Three.js empty canvas, uPlot empty chart, text "2 + 3 = 5 (from Rust/WASM)".
- TypeScript pinned to ~6: svelte-check 4.7 refuses TS 7 without the tsgo flag dance.
- Local proof: `bun run build`, `bun run check` (0 errors), `cargo fmt/clippy/test` clean, `bun run smoke` (Playwright vs `vite preview`) passed. Installed locally: wasm-bindgen-cli 0.2.130 (cargo install) and Playwright's chromium-headless-shell v1254 (needed by @playwright/test 1.65).
- .github/workflows/ci.yml: test (fmt, clippy, cargo test) → build (wasm, svelte-check, vite build, smoke vs preview) → deploy to Cloudflare Pages (main = production, PR branch = preview) + smoke vs deployed URL. Deploy steps are skipped with a notice when either Cloudflare secret is empty, so the job stays green.
### Step 7 (foolproofing)
- scripts/bootstrap.sh (idempotent; 1.7 s when all present): checks rustup + wasm32 (adds the target), wasm-bindgen-cli at the crate's pin (cargo install), bun, gh auth, claude, uv/notebooklm (optional); runs `claude plugin install … --scope project` for each enabled plugin not installed for this clone, `bun install --frozen-lockfile`, sets core.hooksPath to .githooks, builds the WASM. Prints the exact command for anything it won't install.
- .claude/hooks/check-prereqs.sh (SessionStart, 0.49 s): one factual line per missing item, shown to the user (systemMessage) and the model (additionalContext); silent when all is well. scripts/check-plugins.ts compares enabledPlugins with ~/.claude/plugins/installed_plugins.json (user scope, or project scope for this clone's path).
- Proof: fresh clone of origin/scaffold into the scratchpad → hook listed 2 missing items → bootstrap → hook silent. With the plugin registry pointed at an empty dir, the checker reports "mattpocock-skills plugin not installed in this clone". `claude -p --model haiku` in the clone (hooksPath unset on purpose) quoted the hook's "Penumbra setup incomplete" context verbatim and listed the 7 agents plus the mattpocock-skills and project skills. So SessionStart hooks do run on Windows. /ticket and /design-pass don't show in the model's list because they're user-invoked; that's intended.
- docs/ONBOARDING.md (20 lines) with the issues line and the corpus note (corpus is committed here, stripped before the public repo).
### Step 8 (guardrails)
- .claude/hooks/block-dangerous-git.sh (from git-guardrails-claude-code, customised): blocks force/mirror/delete pushes, any push naming main, a bare push while on main, hard reset, clean -f, branch -D, checkout/restore of `.`. Feature-branch pushes pass (/ticket needs them). Parses the payload with jq, else bun. It fired on this session too: it blocked one of my own log-writing commands whose heredoc text merely mentioned a hard reset, so I narrowed the non-push patterns to the same git invocation.
- .claude/settings.json: hooks (SessionStart prereqs, PreToolUse Bash guardrail), enabledPlugins, extraKnownMarketplaces. The permission allowlist is NOT in it (see Needs Qi Cai).
- .githooks/pre-commit (plain git hook, no lefthook install): cargo fmt --check + clippy -D warnings when crates/ or Cargo.* are staged; svelte-check when web/ is staged.
- scripts/wizards/owner-setup.sh from the /wizard template (Cloudflare, Pages project, allowlist, branch protection, collaborator). `bash -n` clean; not run (needs a human). CI gained `workflow_dispatch` so the wizard can trigger a deploy of main.
### Step 10 (WORKFLOW.md)
- docs/WORKFLOW.md (31 lines): Nov 11–13, Nov 13 15:00, Nov 14 09:00, per ticket, budget, milestones, ship checklist → docs/KICKOFF.md.
### PR and step 11 (issues)
- PR #1 scaffold → main: CI run 42902293784 green (test 35 s, build 49 s, deploy 5 s with all deploy steps skipped by the secrets gate). Merge refused by the auto-mode classifier ("Merge Without Review"), so it waits for you.
- Issues: #2 Design pass, #3 Judge's-eye review, #4 Submission templates (owner:design, Charls); #5 Paper skeleton (owner:science, you); #6 Backup Fable quota (hard-science, Charls); #7 throwaway /ticket proof (owner:science, unassigned).
