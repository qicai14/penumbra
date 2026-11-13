# NASA Space Apps 2026 — Solo Kickoff Plan

Research + ideation lives in this folder. Scaffolding/build happens in a **new session** (Phase 4+).

- Solo participant. Tools: Claude Code (Opus/Sonnet/Haiku 5.5), GitHub Copilot (student).
- Links: [event](https://www.spaceappschallenge.org/2026/) · [challenges](https://www.spaceappschallenge.org/2026/challenges/) · [2025 winners](https://www.nasa.gov/learning-resources/stem-engagement-at-nasa/nasa-announces-2025-international-space-apps-challenge-global-winners/)

## ⏰ Timeline (today: Sun Nov 8, 2026)

The event runs **Nov 14–15, 2026**, with the theme "The Next Frontier". Submissions close **Sun Nov 15 at 22:00 local time** (NSAKL26 Telegram, Nov 14; the global rule is 23:59).
- **Team of 2 (decided Nov 8).** The team now qualifies for the Universal Event as well as Local Events. Pick whichever has the better judging path, and confirm in the team-creation UI whether the challenge can be changed after creation.
  - For scoring, solo feasibility matters less now. Data-heavy tracks such as Hot Spots and Analogs gain the most, because one person can run the data pipeline while the other builds the app. The ranking doesn't change.
  - Suggested split: A = science core (model, data pipeline, validation); B = app (UI/3D, deploy, demo video). Both share the story.
- AI use is allowed. Qi Cai (Nov 11): no separate Use-of-AI section; disclose only if the project page form asks.
- The Submission Guide, the Judging rubric and each challenge's **Resources tab are published on Nov 13**.
- Check the rules on pre-event work. The safe reading is that research, the stack and generic tooling can be done beforehand, and app code is written Nov 14–15.

| Day | Do |
|-----|----|
| Nov 8–9 | Register for a Local Event. Shortlist tracks. Create NotebookLM notebooks for the shortlist. |
| Nov 10–11 | Deep research. Pick the topic. Run `/be-ambitious` and `/grilling`. Write `docs/IDEA.md` and `STACK.md`. |
| Nov 12 | New session: lean tooling scaffold (AGENTS.md, agents, skills, deploy pipeline with a hello-world). Spike the riskiest technique, such as Rust→WASM or a data format, in a throwaway repo. |
| Nov 13 | Read the rubric and the Resources tab. Adjust `docs/IDEA.md`. |
| Nov 14–15 | Build. Deploy by Sat noon. Record the demo Sun afternoon. Submit with hours to spare. |

## Shortlist (from `docs/research/challenges-2026.md`, scores out of 30)

| # | Track | Score | Why |
|---|-------|-------|-----|
| 1 | **Lunar Landing Site Navigator** | 27 | Real physics: LOLA terrain, horizon ray-casting, sun elevation for power and Earth visibility for comms. Tied to Artemis and CLPS. Natural 3D south-pole demo. Rust→WASM fits it well. |
| 2 | MODIS/VIIRS Hot Spots Harmonization | 25 | Cross-sensor calibration grounded in the literature, with error estimates. Real responder users. |
| 3 | Earth System Trend Detective | 24 | Strongest statistics, safest solo. Risk: looks like a dashboard. |
| 4 | Earth Analogs of Moon Base/Mars | 23 | Explainable multi-factor similarity scoring on a 3D globe. |

Avoid: Jukebox, Abandoned but not Forgotten, Mission Design Game, Junior Trainer, Health Monitoring. Also worth a look: NISAR (SAR), SPHEREx, Flame in Freefall (most paper-heavy, scores 5 on papers).

## Goals (ranked)

| # | Goal | Must? |
|---|------|-------|
| 1 | Deployed, publicly usable (also a submission requirement) | MUST |
| 2 | Grounded in scientific papers — claims and models cite sources | Strong |
| 3 | Solves a real, current problem for a named user (NASA / researchers / public) | Strong |
| 4 | Uses NASA data (and partner-agency data) | Strong |
| 5 | High-impact visuals: 3D / AR / game | Nice — drop for depth |
| ✗ | Avoid shallow "visuals + general knowledge" tracks (e.g. Earth Information Jukebox) | — |

## Phases

| Phase | Output | Where |
|-------|--------|-------|
| 0. Recon | `docs/research/challenges-2026.md`, `docs/research/winners-2025.md`, `docs/research/skills-and-tools.md` | this session (done by subagents) |
| 1. Shortlist ✅ | top 4 confirmed Nov 8 | — |
| 2. Deep research ✅ → **Lunar Landing Site Navigator** chosen Nov 8 (`docs/research/track-decision.md`) | 4 notebooks live → `docs/research/notebooks.md`; each `docs/research/<track>/sources.md` | this session |
| 3. Pick + ambition | 1 track, 1 idea → **switch to Opus 5.5, high effort, run `/be-ambitious`** → `/grilling` to stress-test | this session |
| 3b. Stack | `docs/STACK.md` with a justification per choice (ADR style) | this session |
| 4. Scaffold | lean repo, AGENTS.md, subagents, skills | **new session** |
| 5. Build | matt-pocock skill loop | new session(s) |
| 6. Ship | deploy, 30s demo video, README, submission | new session |

## Phase 2 — How to research (answers to your questions)

**NotebookLM: yes, but not 14 notebooks.** Use it as a *grounded Q&A + synthesis* layer, not as storage.
- Phase 0 already covers all 14 tracks cheaply (each challenge page lists its own resources). A notebook per track is ~14× the curation work for tracks you'll drop.
- Create notebooks only for the **3–4 shortlisted tracks**, then **one deep notebook for the final topic** (up to 100 sources: key papers, NASA dataset docs, challenge page, recent news).
- Use it for: cited answers ("what models predict X?"), finding gaps/contradictions between papers, audio overview to absorb a field fast, and later fact-checking your pitch.
- Claude ↔ NotebookLM bridge: **use `teng-lin/notebooklm-py`** (CLI + MCP + skill, ~20K★, can add sources *and* query). Skip PleasePrompto/notebooklm-skill (archived Sep 2026, needs public notebooks). LeeJuOh connector = fallback via Claude-in-Chrome (40K-char cap). Caveat: unofficial API, use a dedicated Google account; it's Python, but it's a dev tool only, not in the app. Details: `docs/research/skills-and-tools.md`.
- Paper search MCP: `openags/paper-search-mcp` (arXiv + Semantic Scholar + OpenAlex + PubMed in one) — install in this research session only.

**Finding papers (Claude does the legwork via cheap subagents):**
- Discovery: OpenAlex / Semantic Scholar / arXiv APIs (free, JSON, citation counts) → rank by citations + recency. NASA ADS for astro/heliophysics. NASA NTRS (ntrs.nasa.gov) for NASA's own technical reports.
- Data: the challenge page's resource list first, then Earthdata (CMR search), NASA Open APIs, partner agencies (ESA, JAXA, CSA, ISRO…).
- Firecrawl: scrape JS-heavy pages / docs sites into markdown. Context7: library docs during build (needs `/mcp` auth — currently unauthorized).
- Matt Pocock `/research` skill: delegates a question to a background agent and writes a cited `.md` into the repo — use it per research question.

**Storing papers locally: store metadata + notes, not a library.**
- Keep `docs/research/<topic>/papers.md` (title, DOI/URL, 3-line takeaway, which feature it justifies) — this is what agents read; it's tiny.
- Download PDFs only for the ~5–10 papers your model/algorithm actually implements (equations, constants, validation data). Put them in `docs/research/<topic>/pdf/` (gitignored); extract the needed equations to markdown once so agents never re-read PDFs.
- Everything else lives in NotebookLM.

## Phase 3 — Pick + be ambitious

When a topic is chosen I'll prompt you to: switch model to **Opus 5.5, effort high**, run `/be-ambitious` on the idea brief, then `/grilling` (matt-pocock) to attack it, then `/domain-modeling` to lock the glossary. Output: `docs/IDEA.md` (problem, user, data, science model, demo story, stretch goals).

## Budget: Meta Model API credits (dropped for Lunar on Nov 10)

Not used. Penumbra has no runtime LLM and no API keys (docs/STACK.md, docs/SCIENCE-SPEC.md §4). The earlier SAM 3.1 and Muse price table was removed.

## Phase 3b — Stack rules

Every choice gets one line of justification vs the mainstream default. Candidates (decided per idea, not now):
- Runtime/API: **Bun + Hono** (fast start, TS end-to-end, deploys to Cloudflare Workers / Bun anywhere).
- Heavy compute: **Rust → WASM** in the browser (orbit propagation, raster math, simulations) — no Python, no server cost.
- Data: **SQLite / DuckDB-WASM** (query Parquet/CSV client-side), Postgres only if multi-user writes are needed. Preprocess NASA data at build time into static Parquet/COG/PMTiles.
- 3D/geo: Three.js (or Threlte/R3F), CesiumJS / deck.gl / MapLibre for globe+maps, WebXR for AR.
- Hosting: Cloudflare Pages/Workers (free, global, static-first) — keeps the app alive after judging.
- n8n: only if there's a scheduled data-ingest pipeline; otherwise skip.

## Phase 4 — Scaffold (new session) — kickoff prompt draft

> Scaffold a lean repo for `docs/IDEA.md` using `docs/STACK.md`. Run `/setup-matt-pocock`. Root `AGENTS.md` ≤ 30 lines: pitch, package manager, top-level layout, links to `docs/*.md` (reference pattern). Symlink `CLAUDE.md → AGENTS.md`. Nested `AGENTS.md` only per app/package with distinct rules. No auto-generated instructions; add rules only after a repeated mistake. Create subagents in `.claude/agents/` (see roster). Install only skills that the stack uses.

Lean-config rules (from your video notes): AGENTS.md is an index, not a rulebook · docs/ for conventions (loaded on demand) · skills for one-off procedures · nested AGENTS.md per package · symlink not stub · never auto-generate · grow reactively.

**Subagent roster (draft — model/effort tuned to cost):**

| Agent | Model / effort | Skills |
|-------|----------------|--------|
| architect | Opus 5.5 / high | codebase-design, domain-modeling |
| science-core (models, data pipeline, Rust/WASM) | Opus 5.5 / high | tdd, research |
| frontend / 3D | Sonnet 5.5 / medium | impeccable (or chosen frontend skill), tdd |
| researcher / fact-checker | Haiku 5.5 / low | research, find-docs |
| reviewer | Sonnet 5.5 / medium | code-review, ponytail-review |

**Build loop (matt-pocock backbone):** `/prototype` risky bits → `/tdd` per feature → `/code-review` per branch → `/diagnosing-bugs` when stuck → `/pr`. `/wizard` for deploy secrets/infra steps only you can do.

## What won in 2025 → our rules (from `docs/research/winners-2025.md`)

- One-sentence hook: **"For [user], I show [number] using [dataset]"** — written before any code.
- A **real model** (score, optimizer, physics, solver), not a chart over a CSV. Hand-verify one worked example and show it on the page.
- **≥2 NASA datasets**, ideally live APIs, each with a stated role.
- Named persona + one concrete case with numbers.
- One **validation**: control, before/after, or sensitivity run. State limitations.
- 3D in only 2/10 winners, AR in 0 → visuals are a differentiator *only* if they carry the science.
- 3/10 winners came from the LEO-commercialization track: analysis-heavy, less crowded tracks pay off.
- Agents do scaffolding/UI; your human time goes to the model, the validation and the story.

## Phase 6 — Ship checklist
- [ ] Deployed early (day 1), tested in a private window, kept live ≥ 6 weeks (a 2025 winner's domain lapsed)
- [ ] 30–120 s demo: hook → persona → insight → result → next step; **public YouTube**, not Drive
- [ ] Project page reads as a pitch, with limitations; no leftover agent/prompt text
- [ ] Tidy public repo: README, screenshot, run steps, ≥1 core test, data sources + paper citations
- [ ] AI-use disclosure (all 2025 winners had one)
