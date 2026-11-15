# Idea brief — Lunar Landing Site Navigator (input for /be-ambitious)

**Track:** [Lunar Landing Site Navigator](https://www.spaceappschallenge.org/2026/challenges/lunar-landing-site-navigator/) · Space Apps Nov 14–15, 2026 · team of 2 · AI coding agents.

**Challenge ask:** compare south-pole landing sites × dates by showing Sun and Earth against the real local horizon → power potential, direct-to-Earth comm windows, time to sunset/Earthset. Data: JPL Horizons, LOLA, precomputed ephemerides.

**Baseline idea:**
- Science core: LOLA polar DEM → precomputed 360° horizon profiles per site; DE440/SPICE (MOON_ME) Sun + Earth az/el over time incl. libration; derived metrics: solar elevation, % illumination, Earth visibility, **joint Sun∧Earth "operable window"**, time-to-sunset/Earthset.
- App: 3D south-pole terrain; per-site horizon panorama with Sun/Earth tracks; side-by-side site × date comparison; timeline scrubber.
- Validation: Horizons cross-check (<0.006°); reproduce Mazarico 2011 / Barker 2023 (Shackleton rim 82–92% illumination, PSR ~21,000 km²); back-test IM-2 Athena low-sun tipping.
- Extras: DEM-error uncertainty bands (Barker 2020); SAM 3.1 boulder/shadow hazard segmentation on LRO NAC; Muse Spark explainer grounded in our papers.
- Users: CLPS/Artemis mission planners (primary), public/students (secondary).

**Constraints:** deployed & public; static-first (Rust→WASM compute, Bun/Hono only where needed); runtime data ≤100 MB; no Python in the app; Meta Model API credits ($600 total) available; visuals must carry the science.

**Research:** NotebookLM notebook `5d807093-69f3-410b-bd6a-898489882eeb` (60 sources); `docs/research/lunar-landing/sources.md`, `decision-qa.md`; winners' lessons in `docs/KICKOFF.md`.

**Desired output:** `docs/IDEA.md` — hook sentence ("For [user], I show [number] using [dataset]"), problem, user + concrete case, science model, data plan, validation, demo story, scope tiers (must / should / stretch), two-person split.
