# Workflow

## Per ticket
- A new Claude Code session → `/ticket <n>`. One issue per session.

## Budget (Claude Pro, 5-hour windows)
- Orchestrator: Opus, effort medium. No Opus high by default.
- Fable only through the `science-hard` agent.
- Check `/usage` at the end of every ticket; above 40% context, `/handoff` and start fresh.
- Stagger our 5-hour windows so one of us always has quota.

## Milestones
- Deployed: Sat 12:00 · feature freeze: Sun Nov 15 19:30, all PRs merged by 20:00 · demo recorded: Sun 17:00 · submitted: Sun 21:00 · hard deadline: Sun 22:00 (NSAKL26 Telegram, Nov 14).

## Ship checklist
(from docs/archive/KICKOFF.md)
- [ ] Deployed early (day 1), tested in a private window, kept live ≥ 6 weeks (a 2025 winner's domain lapsed)
- [ ] 30–120 s demo: hook → persona → insight → result → next step; **public YouTube**, not Drive
- [ ] Project page reads as a pitch, with limitations; no leftover agent/prompt text
- [ ] Tidy public repo: README, screenshot, run steps, ≥1 core test, data sources + paper citations
- [ ] AI-use disclosure (all 2025 winners had one)
