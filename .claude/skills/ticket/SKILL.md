---
name: ticket
description: Orchestrate one GitHub issue from claim to PR.
argument-hint: "[issue-number]"
disable-model-invocation: true
---

This session is the orchestrator for issue #$0 and nothing else. Keep your own context small: hand implementation to the owning agent with pointers, not pasted content: the issue text, file paths, and the `docs/SCIENCE-SPEC.md` section and test ids the issue names (so time scales, frames and units reach the agent).

0. **Confirm intent.** After reading the issue (step 1), show the user, in under 8 lines:
   - the issue title
   - your restated goal
   - the route: which agent, which skills
   - any blockers found
   - anything ambiguous

   Then wait for a yes before step 2. If the user adds notes, fold them in and confirm once more.
1. **Read and gate.** `gh issue view $0 --json number,title,body,labels,assignees`. Find blockers: `gh api repos/{owner}/{repo}/issues/$0 --jq .issue_dependencies_summary` plus any `Blocked by: #n` line in the body. If any blocker is open, report it and stop.
2. **Claim.** Make a slug from the title (3–5 words, kebab-case). Create a worktree on branch `$0-<slug>` from `origin/main` (EnterWorktree tool, else `git worktree add ../penumbra-$0-<slug> -b $0-<slug> origin/main` and work there). Then `gh issue edit $0 --add-assignee @me --add-label in-progress`.
3. **Build.** Route by label:
   - `owner:science` → `science-core` agent; V1–V5 or scoreboard work → `validator`.
   - `owner:app` → `frontend-3d`. `owner:design` → `designer`.
   - `hard-science`, or science-core failed the same test twice → `science-hard` for the diagnosis, then `science-core` applies its fix.
   - CI and deploy (`.github/`, Cloudflare Pages), the D1 paper (`docs/paper/`, `bun run paper`) and `scripts/` outside validation have no agent: do them yourself.
   - A change under ~30 lines in total: make it yourself, test-first where it touches Rust.
   Done when the agent reports its done-criteria met and you have re-run the tests it names.
4. **Review.** Dispatch the `reviewer` agent on this branch for issue #$0. Fix every Blocking finding (back through the owning agent if large) and re-run the tests.
5. **Ship.** Push the branch. Write the PR body with the `pr` skill, starting with `Closes #$0`. No AI attribution anywhere: no Co-Authored-By, no "Generated with" line. `gh pr create`, then `gh issue comment $0` with the PR link and what is left, if anything.
6. **Report.** If the same kind of failure happened twice in this ticket (a wrong frame, a lint, a flaky step), propose a one-line rule for the owning agent's file in `.claude/agents/` so it does not happen again. End with: PR link, tests run, and the session cost. Ask the user to check `/usage`; if context is above 40%, tell them to run `/handoff` and start a fresh session rather than continue here.
