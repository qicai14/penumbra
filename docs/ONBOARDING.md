# Onboarding

1. Clone: `gh repo clone qicai14/penumbra-burner && cd penumbra-burner` (Git Bash on Windows).
2. Bootstrap: `bash scripts/bootstrap.sh`. It installs what it safely can and prints the exact command for the rest; re-run it until it says Ready.
3. Open Claude Code in the repo, set the model to Opus with effort medium (`/model`). If anything is still missing, a "Penumbra setup incomplete" message appears at session start; fix it before working.
4. Approve the project's plugin and hooks when Claude Code asks (once per clone).

## What to run
- Design work (before Nov 14): `/design-pass` (or `/design-pass 2` for one step). Everything goes in `design/`.
- Build work (from Nov 14): `/ticket <n>`, one fresh session per issue.
- Your tasks are GitHub issues assigned to you. Start each with /ticket <n> or /design-pass.

## Budget (Claude Pro, 5-hour windows)
- Orchestrator: Opus, effort medium. No Opus high by default.
- Fable only through the `science-hard` agent (hard-science label).
- Check `/usage` at the end of every ticket; above 40% context, run `/handoff` and start fresh.
- Stagger our windows so one of us always has quota. Full rules: `docs/WORKFLOW.md`.

## Evidence and the corpus
- Corpus note: `docs/research/lunar-landing/corpus/` holds the paper text (committed here because this repo is private; the raw PDFs are gitignored). It is stripped before the public repo. For evidence work, use it or the shared NotebookLM notebook, under `docs/EVIDENCE.md`.
