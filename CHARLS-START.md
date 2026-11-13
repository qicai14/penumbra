# Charls: start here

The planning repo is **private** at `github.com/qicai14/penumbra-burner`, and you're a collaborator. Clone it and open Claude Code in it. Everything you make goes in `design/` or `docs/`. Commit on a branch, open a PR, and Qi Cai reviews it.

Team name: **Fractured Toes**. Project name: **Penumbra**. Only the deploy URL is `penumbra-lunar.pages.dev`. The poster and the title slide carry the team name and both member names (Qi Cai, Charls).

**Pre-event rule (Space Apps FAQ):** until Nov 14, only planning and design: no app code, no lunar data processing. Everything below is allowed.

## 0. Setup (no Claude usage, ~30 min, Windows PowerShell)
- `winget install Git.Git GitHub.cli` → `gh auth login`
- `winget install Rustlang.Rustup` → `rustup default stable` → `rustup target add wasm32-unknown-unknown` (also needs Visual Studio Build Tools, C++ workload)
- Bun: `powershell -c "irm bun.sh/install.ps1 | iex"` · Typst: `winget install Typst.Typst` · uv: `winget install astral-sh.uv`
- Read `docs/IDEA.md` and `docs/SUBMISSION.md` (deliverables and rubric).

## Your Fable tasks, in order (Fable 5.2, effort high, one new session per task)
The research corpus (full paper text) is in docs/research/lunar-landing/corpus/; INDEX.tsv maps IDs to titles. Use it for evidence, not NotebookLM. Each task: paste the prompt, let it work, then review the output yourself before opening a PR. Pause at about 80% of your 5-hour limit, and resume after it resets.

**F1. Brand → wireframes → hi-fi prototype** (`design/`)
> Read docs/IDEA.md, docs/SCIENCE-SPEC.md §4 (scope tiers) and docs/SUBMISSION.md. Penumbra is a dense scientific web tool for lunar landing planners, not a marketing site.
> (1) Brand: install the brand-identity skill (`npx skills add fatihaydost/brand-identity-skill@brand-identity -g`; read its SKILL.md and scripts first) and impeccable (`npx skills add pbakaus/impeccable@impeccable -g`). Make 3 directions (lockup, palette readable on dark terrain and in charts at WCAG AA, type scale) on one HTML page → design/brand/. I pick one.
> (2) Lo-fi: for each screen (horizon panorama + timelines, pin/date controls + survival headline, 3D terrain, IM-2 replay, validation scoreboard), make 3 greyscale structural directions as static HTML in design/wireframes/<screen>/, with an index.html side by side. I pick per screen.
> (3) Hi-fi: apply the brand to the picks in design/prototype/ (static HTML/CSS, fake data labelled FAKE). Put the tokens (type, colour, spacing, motion) in design/DESIGN.md. Run an impeccable critique pass. Log decisions in design/DECISIONS.md.
> No app code in web/, and no real lunar data.

**F2. Commercialization and impact brief** (`docs/BUSINESS.md`, rubric criteria 6 and 7, up to 20 points)
> Read docs/IDEA.md and docs/SUBMISSION.md. Write docs/BUSINESS.md: who pays (CLPS vendors, payload and rover teams, Artemis ops, non-US agencies, insurers), why they'd pay when NASA's tools are free, an open-core model (free public tool and API under Apache-2.0, plus a paid mission-planning service), the cost of a failed landing versus the cost of the tool, landings planned through 2030, and competitors (Moon Trek, QuickMap, in-house tools). Every number needs a source URL and a verbatim quote. Tag unconfirmed claims [U] and never invent figures. Finish with a 1-slide summary and a 150-word section for the paper.

**F3. Story scripts** (`docs/submission/`)
> Read docs/IDEA.md (demo story), docs/SUBMISSION.md (D3–D5 specs) and docs/BUSINESS.md. The team is "Fractured Toes" (Qi Cai, Charls) and the project is "Penumbra". Write: the 30-s video script (guidebook beats at 0:00/0:02/0:05/0:10/0:20/0:30), a 4:30 video script with a shot list, the 7-slide deck outline (title slide counts), and the poster copy. Put result numbers in as {{placeholders}}. English, with on-screen captions.

**F4. Judge's-eye review** (`docs/JUDGE-REVIEW.md`)
> Act as an NSAKL26 local judge. Score docs/IDEA.md, BUSINESS.md and the scripts against the 10 criteria in docs/SUBMISSION.md (each out of 10). Give gaps and fixes ranked by points per hour.

Then, in Canva: a brand kit from DESIGN.md, plus poster and slide templates with placeholders.
