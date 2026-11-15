# Penumbra

Penumbra tells lunar landing planners how many hours of sunlight and Earth contact a lander gets at a south-pole site and date, and when the first lethal shadow arrives. One Rust model computes it natively for validation and as WASM in the browser, checked against JPL Horizons and NASA LOLA illumination maps.

## Toolchain
- Rust stable + `wasm32-unknown-unknown` + `wasm-bindgen` CLI (no wasm-pack). Bun for all JS/TS (`bun run <script>`, see package.json); no npm, no Python.
- Git Bash on Windows. `scripts/bootstrap.sh` sets up a clone.

## Layout
- `crates/` Cargo workspace: `core` (pure maths, no I/O), `pipeline` (native, offline), `wasm` (bindings)
- `web/` Svelte 5 + Vite app, Three.js and uPlot · `scripts/` Bun TS tooling
- `design/` brand, tokens, wireframes · `docs/` specs and process

## Working with the user
Treat every request as a draft. Before any edit or command that changes something:
1. Restate the request as an improved, specific prompt.
2. Name the skill or `/command` that fits and why (e.g. `/ticket 12`, `/design-pass`, `/tdd`, `/diagnosing-bugs`).
3. Wait for a yes.

Skip the confirmation only for read-only lookups, or when the user says "go" or "just do it". Charls is new to Claude Code: explain each choice in one plain line, and end with the next prompt he should type.

## Workflow
Every session starts with `/ticket <n>` (or `/design-pass` for design work).

## Docs (read when the task touches them)
- Science model, numbers, tests: [docs/SCIENCE-SPEC.md](docs/SCIENCE-SPEC.md) · product and scope tiers: [docs/IDEA.md](docs/IDEA.md)
- Stack choices and why: [docs/STACK.md](docs/STACK.md) · domain terms: [docs/GLOSSARY.md](docs/GLOSSARY.md)
- Any number or citation: [docs/EVIDENCE.md](docs/EVIDENCE.md)
- Where we are and what to paste first: [docs/HANDOFF.md](docs/HANDOFF.md) · team process and budget: [docs/WORKFLOW.md](docs/WORKFLOW.md) · deliverables and rubric: [docs/SUBMISSION.md](docs/SUBMISSION.md)

## Agent skills
- Issue tracker: GitHub Issues via `gh`. See `docs/agents/issue-tracker.md`.
- Triage labels: the five defaults, plus `owner:science|app|design`, `hard-science`, `in-progress`. See `docs/agents/triage-labels.md`.
- Domain docs: single-context, `docs/GLOSSARY.md` + `docs/adr/`. See `docs/agents/domain.md`.
