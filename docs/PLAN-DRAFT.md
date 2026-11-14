# PLAN-DRAFT — Penumbra build plan

Progress: COMPLETE (Nov 11) · §1 Spec · §2 Tickets (T00–T27, parking, D1–D7) · §3 Schedule · §4 Slip cuts · §5 Day-1 checklist · §6 Open decisions · Summary. Burner-repo update applied.

Draft written Nov 11 2026 from IDEA.md, docs/SCIENCE-SPEC.md (F1–F22 applied), docs/STACK.md, docs/SUBMISSION.md and the scaffold layout in AGENTS.md. `design/` does not exist in this folder yet, so no design decisions are reflected. Revise on Nov 13 against the Resources tab and the rubric. **Done Nov 14:** see IDEA.md "What the challenge page stresses", SUBMISSION.md "Global judging", and Q10 below. File as GitHub issues on Nov 14 at 09:00, in the public `qicai14/penumbra` repo that is created that morning from the private `qicai14/penumbra-burner` without the corpus (§5).

Conventions
- **[est]** = an estimate this plan could not ground in a source document. Hours without the tag come from SCIENCE-SPEC §4 (the split of a §4 item across tickets is mine; the sum matches §4).
- Test IDs (T-1a-1, P3, N7, V5 case 3, U5…) are the ones in SCIENCE-SPEC §1–§5 and in SCIENCE-SPEC-REVIEW §4 (the N-rows).
- Ticket IDs here are T00–T27 and D1–D7. GitHub will renumber them; file in this order so the mapping is T00 → #1, T01 → #2 and so on.
- No code in this file. Paths follow the scaffold layout in AGENTS.md (`crates/{core,pipeline,wasm}`, `web/`, `scripts/`, `docs/`); file names inside those folders are proposals.
- The /to-tickets format says to avoid file paths. This plan lists them anyway because the brief asks for them.

Sections: 1 Spec · 2 Tickets · 3 Schedule · 4 Slip cuts · 5 Day-1 checklist · 6 Open decisions · Summary

---

## 1. Spec (/to-spec shape)

### Problem Statement

A lander at the lunar south pole lives or dies by a few degrees of terrain. At the pole the Sun's centre stays within 1.54° of the horizon, so a 10 m rise casts a shadow hundreds of metres long, and moving the site 100 m or the date two days can decide the mission. IM-2 Athena ended on its side in a 20 m crater where a low Sun and the panel orientation stopped charging.

A landing planner today has to combine Moon Trek, QuickMap and Horizons by hand. None of them answers the planner's question for one site and one date: does this lander get enough power and Earth contact to survive, and when does the first shadow arrive that would kill it?

### Users

- **Primary:** CLPS and Artemis landing and operations planners. Concrete case: replay IM-2's site and date, then look for a nearby site or date that gives a longer operable window.
- **Secondary:** students and the public, who see why the lunar pole is unlike anywhere on Earth.
- **Judges** (local NSAKL26 rubric, then global): they read the validation scoreboard and the project document.

### Solution (the outcome)

A static web page. The planner picks a site in an Artemis region, a landing date in 2026–2030 and a lander preset. Penumbra returns:

1. A 360° panorama of the terrain horizon at the chosen mast height, with the Sun and Earth tracks drawn on it.
2. Timelines of illumination (Sun-disc fraction), Sun visibility, Earth elevation margin and the operable window (Sun AND Earth).
3. The survival headline: operable days and time to first lethal shadow, from a battery state-of-charge simulation.
4. An IM-2 replay: what regional terrain says, and how deep a 20 m pit must be to hide the Sun from a panel at height H.
5. A scoreboard of outside-judge checks (V1–V5) with the real numbers, including misses.

Every number on the page comes from one Rust model that runs natively for validation and as WASM in the browser.

### User Stories

1. As a landing planner, I want to pick a point inside an Artemis region, so that I evaluate the exact site and not a region average.
2. As a landing planner, I want to pick a landing date between 2026 and 2030, so that I can compare launch windows.
3. As a landing planner, I want a panorama of the terrain horizon at my mast height, so that I can see what blocks the Sun and the Earth.
4. As a landing planner, I want the Sun and Earth tracks drawn on that panorama, so that I can see where each one sets behind terrain.
5. As a landing planner, I want the distance to the obstacle that sets the horizon at each azimuth, so that I can tell a nearby boulder-scale artefact from a far ridge.
6. As a landing planner, I want an illumination timeline as the fraction of the Sun's disc, so that I can size power.
7. As a landing planner, I want an Earth-contact timeline with a stated elevation margin, so that I can plan direct-to-Earth passes.
8. As a landing planner, I want the operable window (Sun AND Earth), so that I know when the lander can both charge and talk.
9. As a landing planner, I want the time to first lethal shadow for my lander preset, so that I know how long the mission survives.
10. As a landing planner, I want the longest continuous darkness and the longest lit period, so that I can size the battery.
11. As a landing planner, I want to edit the lander preset (battery, panel, load, mast height, panel orientation), so that the answer fits my vehicle.
12. As a landing planner, I want every preset value marked "cited" or "assumed", so that I know which inputs to replace with my own.
13. As a landing planner, I want to move the pin or shift the date and see the headline change, so that I can find a better site or date.
14. As a landing planner, I want an error when the site is outside the DEM coverage, so that missing terrain is never shown as open sky.
15. As a landing planner, I want a ± band on each sunset time, so that I do not plan to the minute on a 20 m map.
16. As a landing planner, I want a "local pit" control (diameter, depth), so that I can see what terrain below the DEM resolution would do.
17. As a member of the public, I want to replay IM-2's site and date, so that I can see why the lander lost power.
18. As a member of the public, I want one headline number per site and date, so that I understand the result without reading charts.
19. As a judge, I want a scoreboard of checks against JPL Horizons and published LOLA illumination results, so that I can trust the numbers.
20. As a judge, I want misses reported with their numbers, so that I can see the tool's limits.
21. As a judge, I want every dataset listed with its role and citation, so that I can score the use of NASA data.
22. As a judge, I want a public link that works without login, so that I can try the tool myself.
23. As Charls (app), I want a frozen API and a fixture from hour 0, so that I can build the page before the engine exists.
24. As Qi Cai (science), I want every equation to have a test with a hand-worked number, so that an agent cannot drift from the spec.
25. As either of us, I want every ticket to end on the deployed page or in the scoreboard, so that progress is visible and nothing waits for a big merge.
26. As the next lander team (Stretch), I want a JSON API, so that I can check a site before I fly.

### Implementation Decisions

**Modules**

| Module | What it is | Depends on | Rule |
|--------|-----------|------------|------|
| `core` | Pure maths: projection, local frame, az/el, table interpolation, horizon marcher, disc fraction, timeline, state of charge, pit model, sunset band | nothing (no I/O, no clock, no network) | f64 only (N7). Radians inside, degrees at the API edge. Returns errors, never a silent default (P5, P16). |
| `pipeline` | Native binary: builds the ephemeris table with ANISE + hifitime, reads GeoTIFFs tile by tile with the `tiff` crate, crops and encodes DEM tiles, runs V2/A1/V5 data checks, writes `scoreboard.json` | `core`, ANISE, `tiff` | Never shipped to the browser. Needs `data/` (gitignored). |
| `wasm` | wasm-bindgen wrapper over `core`. No logic of its own. | `core` | Takes typed arrays in, returns typed arrays or a small object. |
| `web` | Svelte 5 + Vite page: panorama (2D canvas), timelines (uPlot), controls, headline, scoreboard page | `wasm`, static data files | Calls the model through one adapter module. No science in TypeScript. |
| `scripts` | Bun + TypeScript: Horizons fixture fetch, downloads, issue filing | — | Offline only. |

**The seam.** There is one test seam: the public functions of `core`. Native tests call them directly; the browser calls the same functions through `wasm`. The DEM enters `core` through one sampler interface (map x, y → height or "no data"). `pipeline` has two samplers (shipped tiles; full GeoTIFF for T-1a-4 only) and `wasm` has one (shipped tiles).

**WASM API, proposed for the hour-0 freeze** (names, units and layouts; change them at 09:10 on Nov 14 or not at all)

Full Rust signatures, array layouts and ownership rules: [ARCHITECTURE.md §3.2](ARCHITECTURE.md). Results are flat column-major `Float64Array`s with time as column 0, copied out; there is no P13 flag (the reader tests `dist_m < 60`); `h_max` is computed at load.

| Function | In | Out |
|----------|----|-----|
| `api_version` | none | 1. The fixture and the manifest carry the same number; the adapter refuses a mismatch. |
| `latlon_to_xy`, `xy_to_latlon` | lat °, east lon ° / x m, y m | `[x_m, y_m]` / `[lat_deg, lon_deg]` |
| `Engine` (stateful) | `load_ephemeris`, `load_near`, `load_far`: each array is copied in once and owned by the engine | Errors: `NOT_LOADED`, `OUT_OF_COVERAGE`, `NO_DATA`, `OUT_OF_TABLE`, `BAD_INPUT` |
| `Engine.ground_height` | lat °, east lon ° | ground height m |
| `Engine.sun_earth_track` | site (lat °, lon °, ground m, mast m), start time, step s, count | 9 × n: `t_unix_s`, Sun az °, el °, angular radius °, distance km; Earth az °, el °, angular radius °, distance km. Error if any time is outside the table (P16). |
| `Engine.horizon` | lat °, lon °, mast m | 1440: `el_deg[720]` then `dist_m[720]`; index i is azimuth i × 0.5°, clockwise from north. Error when the march leaves coverage or hits no-data (P5). |
| `Engine.timeline` | horizon, site, start time, step s (default 600), count, Earth margin ° (default 2, "assumed") | 10 × n: `t_unix_s`, Sun az, Sun el, **illumination** (0–1), **visibility** (0 or 1), Earth az, Earth el, Earth margin °, contact (0 or 1), Sun distance km |
| `survival` (free function) | timeline, 12-value lander preset, landing time | 8 + n: header (operable h, lethal h or NaN, longest dark h, longest any-Sun h, sunlit h, contact h, reserved, n), then state of charge Wh |
| `pit_horizon`, `pit_depth_to_hide` (Should, free functions) | horizon, pit diameter m, depth m, panel height m; Sun el and radius | raised horizon (same layout); d_c* in m |
| `window_bands` (Should, free function) | timeline, horizon, σ_z m (default 0.5, "assumed") | 3 × k: `t_unix_s`, kind (+1 sunrise, −1 sunset), half-width min |

- Time everywhere at the edge is Unix seconds, UTC. The browser never converts time scales (SCIENCE-SPEC §1c).
- Illumination and visibility are separate columns with those exact names (P1). Earth "any fraction visible" (for V2-Earth) and Earth "contact" (centre + margin) are separate functions with different names (P14).
- Lander preset fields: battery capacity Wh, starting energy Wh, minimum energy Wh (default 0), panel area m², efficiency, panel mode (Sun-tracking, or fixed with a normal vector), load W, Earth margin °, mast height m. Each field carries a "cited" or "assumed" tag (U10).
- The §4 A-order ends "state of charge → WASM → V5". Here the WASM export is spread: T02, T07, T08 and T09 each end with their own export, because the tracer rule needs every slice on the deployed page. The order of the science is unchanged.

**The adapter (web).** One module returns each API result either from the fixture or from WASM, switched per function. Charls builds every view on the fixture; each science ticket flips one switch when it merges. A visible "FIXTURE DATA" badge shows while any view still reads the fixture.

**Data artefacts and binary formats** (all little-endian, row-major, no headers; shapes and scales live in the manifest)

| Artefact | Format | Size | Built by | Notes |
|----------|--------|------|----------|-------|
| `manifest.json` | JSON: for each file its URL, dtype, shape, and metadata | small | `pipeline` | The only file the page fetches by fixed name. |
| Ephemeris table, main segment | f64, rows × 6: Moon-centred apparent Sun x, y, z then Earth x, y, z, km, MOON_ME. Row k is 2026-01-01T00:00:00Z + k hours. | 43 825 rows × 48 B = 2.10 MB [D: 1826 days × 24 + 1] | T02 | UTC grid built with hifitime. Linear interpolation in the browser. |
| Ephemeris table, replay segments | same layout; manifest gives each segment's start and row count | IM-2: about 45 days, 52 kB [est] | T15 | Needed because the replay dates (Mar 2025, Feb 2024) are outside 2026–2030. |
| Near-field tile, one per region | int16, 20 m pixels, north-up. Manifest: upper-left corner x₀, y₀ (m), pixel size, pixel-is-area flag, `scale` ≤ 0.25 m, `offset` m, no-data value. height = offset + scale × value. | 1536 × 1536 = 4.7 MB, 30.7 km square [est] | T06 | A site must be at least 7 km inside the tile edge, so the pickable area is about 16.7 km square [D]. |
| Far-field tile, shared | int16, 240 m pixels, same conventions, `h_max` is computed at load, not stored | ±382 km: 3184 × 3184 = 20.3 MB | T06 | File names carry a content hash. `scale` 0.5 m here: far-field relief may exceed the ±8.2 km that 0.25 m allows [est]. The encoder fails on clipping. |
| Horizon grid (slip cut 3 only) | int16, 720 per point at 0.002° per count [est], 100 m grid | about 14 MB per 10 × 10 km | T07 fallback | Built only if cut 3 fires. |
| `scoreboard.json` | JSON: one entry per check: id, judge, metric compared, threshold, value, pass, sample size, notes, commit, date | small | T05 onward | Read by the scoreboard page and by the Typst paper. |
| API fixture | JSON with the exact output shapes above: fake horizon, fake 30-day timeline, fake survival, sample scoreboard | under 1 MB [est] | T00 | Labelled fake inside the file. |
| Horizons fixtures | JSON per pair: the request text and the response | small | T02, T05 | Tests never call the network. |

- Built artefacts are committed under the web app's static folder, not under `data/` (which is gitignored and holds raw downloads). CI cannot rebuild them: it has no DEM. [est: check Cloudflare Pages' per-file limit against the 20.3 MB far field; split it in two if needed.]
- Runtime total with two regions: 2 × 4.7 + 20.3 + 2.1 ≈ 32 MB. Nine regions: about 65 MB. Both are under the 100 MB budget.

**Regions.** Two first: Mons Mouton (IM-2 site) and the Shackleton ridge (SP-01, Barker Site 1, PSR floor, the V2 area). The other seven are a later widening ticket (T25).

### Testing Decisions

- A good test here checks a number a reader can derive by hand or find in a paper, at the `core` seam. It does not check how the marcher loops.
- `cargo test` holds the equation tests (T-1a to T-1e), V1 (Horizons fixtures) and V5 (red team). They run in CI when they need no DEM.
- Tests that need `data/` (T-1a-2 data part, T-1a-4, T-1a-5, V5 case 3, V2, A1) run locally through `pipeline`, print a notice when the data is missing, and write their result to `scoreboard.json`. CI never downloads the DEM.
- V1 goes through the shipped table and the interpolation, not through ANISE directly (N3).
- Every sensitivity test proves the fixture can see the bug: aberration off must move the Sun by 0.005–0.006° (P3); PA instead of ME must differ by more than 0.01° (P8).
- One Playwright smoke test runs against the deployed URL. Prior art: the template's hello-world smoke test.
- No LLM evals. V1–V5 are the evals.

### Out of Scope (non-goals)

- Cut: SAM 3.1, DSN-complex visibility, the 10 m DEM, boulder hazards, a chatbot, any runtime backend, any runtime LLM.
- Not modelled, and stated on the page: limb darkening, refraction, charge and discharge efficiency, a separate night heater load, terrain smaller than a 20 m pixel (only the user-set pit).
- Not attempted: the full AVGVISIB raster (about 4 core-days), Barker's 100-clone DEM Monte Carlo, the `_ERR` raster download (3.7 GB) outside Stretch.
- Stretch, not planned into the 16 hours: heatmaps for nine regions, V3 relative IoU, Monte Carlo bands, IM-1 replay, share links, JSON API.

### Further Notes: gaps found while planning

These are places where the source documents do not add up. Each has a default in this plan and a line in §6.

1. **Submission hours are not in SCIENCE-SPEC §4.** §4 spends all 16 hours on build. D1–D7 need about 1.5 h of Qi Cai's and 4.75 h of Charls's time before the freeze [est]. That removes most of Qi Cai's 3 h of slack and all of the room for 3D terrain.
2. **3D terrain (4 h) does not fit for Charls** once D2–D5 are counted. It also needs a grid-shading function from `core` that §4 never budgets (T21, 0.75 h [est]). The default here is no shaded map; the 2D shaded map and then 3D are pull-forwards.
3. **The hook depends on V2 or A1.** "Checked against … NASA's published LOLA illumination maps" is true only if V2 (2 h, overflow) lands. If only A1 lands, say "published LOLA illumination results". If neither lands, the hook can claim Horizons only.
4. **Kernels are missing from the hour-0 download list.** The ephemeris ticket needs them by 10:30. They go first in the queue.
5. **Replay dates are outside the 2026–2030 table.** The table needs extra segments (see formats above).
6. **Demo story step 4 (rank nine regions) needs the heatmap** (2 h science + 1 h app [est]), which is overflow. Step 6 (open API) is Stretch. The video script should rest on steps 1–3 and 5.
7. **IM-1 is 300 km from the pole.** Its far field would have to reach 500 km, beyond the ±382 km tile. IM-1 stays Stretch.
8. **The public repo needs a fresh history.** The corpus text is committed in the private `penumbra-burner`, so a fork, a mirror push or "use this template" would publish it. `penumbra` starts from an export without `research/**/corpus/` and one new commit. Labels, secrets, the Pages project, branch protection and the collaborator do not carry over and are set up again on Nov 14 (§5).

---

## 2. Tickets (/to-tickets shape)

Tracer rule: T00 sets up, T01 is the one prefactor, and **T02 is the tracer**: one site, one date, real numbers on the deployed page. Nothing else on the science side starts before T02 merges. After it, each science ticket ends on the deployed page or in the scoreboard.

★ = on the critical path (§3). Owners by name: **Qi Cai** = owner:science, **Charls** = owner:app and owner:design.

### Index

| ID | Title | Owner | Labels | Est (h) | Tier | Blocked by |
|----|-------|-------|--------|---------|------|-----------|
| T00 ★ | Hour 0: repo, downloads, API freeze, fixture | both | owner:science, owner:app | 0.5 each | Must | none |
| T01 ★ | Site geometry without data | Qi Cai | owner:science | 1 | Must | T00 |
| T02 ★ | **Tracer:** real Sun and Earth az/el for one site and date on the deployed page | Qi Cai | owner:science, hard-science | 1.5 | Must | T00, T01 |
| T03 | Panorama page on the frozen API | Charls | owner:app | 2.5 [est] | Must | T00 |
| T04 | TIFF decode probe on the PSR file | Qi Cai (background) | owner:science | 0.25 [est] | Must | T00 |
| T05 ★ | V1: 20 Horizons pairs through the table path | Qi Cai | owner:science, hard-science | 1 | Must | T02 |
| T06 ★ | DEM tiles for two regions and the far field | Qi Cai | owner:science | 1.5 | Must | T01, T04 |
| T07 ★ | Terrain horizon in the browser | Qi Cai | owner:science, hard-science | 1.75 | Must | T06 |
| T08 ★ | Sun-disc fraction and timelines | Qi Cai | owner:science | 1.25 | Must | T02, T07 |
| T09 ★ | Survival: state of charge, operable window, first lethal shadow | Qi Cai | owner:science | 1.5 | Must | T08 |
| T10 ★ | V5 red-team suite | Qi Cai | owner:science | 1 | Must | T05, T07, T08, T09 |
| T11 | Site and date controls | Charls | owner:app | 1 [est] | Must | T03 |
| T12 | Timelines chart | Charls | owner:app | 2 [est] | Must | T03 (real data: T08) |
| T13 | Scoreboard and About page | Charls | owner:app | 1.5 | Must | T00 (real data: T05) |
| T14 | Survival headline and lander preset form | Charls | owner:app | 1.5 [est] | Must | T11, T12 (real data: T09) |
| T15 ★ | IM-2 replay: replay segment, local pit, V4 statement 3 | Qi Cai | owner:science | 1.5 | Should | T08 |
| T16 ★ | IM-2 replay view | Charls | owner:app | 1 [est] | Should | T14, T15 |
| T17 | Sunset ± band | Qi Cai | owner:science | 1 | Should | T08 |
| T18 | Band display | Charls | owner:app | 0.5 [est] | Should | T12, T17 |
| T19 | A1: longest-night table check | Qi Cai | owner:science, hard-science | 1 | Should (slack) | T10 |
| T20 | V2 sample against AVGVISIB | Qi Cai | owner:science, hard-science | 2 | Should (overflow) | T10 |
| T21 | Shadow mask over a tile | Qi Cai | owner:science | 0.75 [est] | Should (overflow) | T07 |
| T22 | 2D shaded map | Charls | owner:app | 1.5 [est] | Should (overflow) | T11, T21 |
| T23 | 3D terrain coloured by the model | Charls | owner:app | 4 | Should (overflow) | T11, T21 |
| T24 | Heatmap for one region | Qi Cai + Charls | owner:science, owner:app | 2 + 1 [est] | Should (overflow) | T09 |
| T25 | Widen to nine regions | Qi Cai | owner:science | 0.5 [est] | Should (overflow) | T06 |
| T26 | Sun and Earth elevation readout with sunset and Earthset countdown | Charls | owner:app | 0.5 [est] | Must | T03 (real data: T08) |
| T27 | Two site-date pairs side by side (A vs B) | Charls | owner:app | 1 [est] | Should (overflow) | T14 |
| D1 | Project document (Typst → PDF) | Qi Cai + Charls | owner:science, owner:design | 1 + 0.5 [est] | Submission | T05; final: freeze |
| D2 | Team poster | Charls | owner:design | 1 [est] | Submission | T03 |
| D3 | 30-second demo video (D3a: footage by Sat 13:00) | Charls | owner:design | 0.5 + 1 [est] | Submission | D3a: T02, T03 · cut: T14 |
| D4 | 3–5 minute project video | Charls + Qi Cai | owner:design, owner:science | 3.5 + 0.5 [est] | Submission | T16; recording: freeze |
| D5 | Global demo link | Charls | owner:design | 0.25 [est] | Submission | D3 |
| D6 | Final project link | Qi Cai | owner:science | 0.5 [est] | Submission | freeze |
| D7 | Project page fields | Qi Cai | owner:science | 1.25 [est] | Submission | D3, D4, D6 |

Hours check. Qi Cai Must: 0.5 + 1 + 1.5 + 1 + 1.5 + 1.75 + 1.25 + 1.5 + 1 = 11.0 (§4: 10.5 + hour 0). Charls Must: 0.5 + 2.5 + 1 + 2 + 1.5 + 1.5 + 0.5 = 9.5 (§4: 4 + 2 + 2 = 8, plus hour 0, plus 0.5 for switching views from fixture to real [est]; T26 added Nov 11 for the challenge's "time remaining before sunset or Earthset").

"Overflow" means the ticket is filed but is not on the schedule. It is pulled forward only through a gate in §3.

### Must: science chain

#### T00 ★ Hour 0: repo, downloads, API freeze, fixture
- **Labels:** owner:science, owner:app · **Owners:** Qi Cai and Charls · **Est:** 0.5 h each · **Blocked by:** None (can start immediately)
- **Touches:** `docs/API.md`, the API fixture JSON and the adapter stub under `web/src/lib/`, a sample `scoreboard.json` in the web static folder, `scripts/` (export, repo-setup, download and issue-filing scripts), `data/` (gitignored)
- **What to build:** after 30 minutes both people can work without waiting for each other. Charls has a fixture with the final shapes; Qi Cai has every download running.
- **Acceptance criteria**
  - [ ] Public `qicai14/penumbra` exists, Apache-2.0, made from an export of `penumbra-burner` with a fresh history. No path under `research/**/corpus/` is in its tree or in any commit (`git log --all` on that path prints nothing).
  - [ ] On `penumbra`: Charls (`ambatuCode887`) is a collaborator; labels, the two Cloudflare secrets, the Pages project `penumbra-lunar` (repo variable `CF_PAGES_PROJECT`) and branch protection are set; the hello-world deploys from it and the smoke test is green.
  - [ ] Downloads running, small files first: the three kernels (`de440.bsp`, `moon_pa_de440_200625.bpc`, `moon_fk_de440.epa`), `LPSR_80S_20MPP_ADJ.TIF` (148 MB), `LDEM_60S_240MPP_ADJ.TIF` (210 MB), `AVGVISIB_85S_060M_201608.TIF` and `_EARTH` (34 + 25 MB), `LDEM_80S_20MPP_ADJ.TIF` (2.6 GB). **Not** `_ERR` (3.7 GB). The DEM's ETA is written in the issue.
  - [ ] `docs/API.md` holds the §1 API table, approved by both in the PR. After this PR the API changes only with both names on the change.
  - [ ] The fixture has a fake horizon (720 + 720 values), a fake 30-day timeline at 10 minutes (4320 rows × 10), a fake survival result and a sample scoreboard, and says "fake" inside the file (SCIENCE-SPEC §4 Hour 0).
  - [ ] Issues filed for T01–T25 and D1–D7 in this order, with labels and "Blocked by".
  - [ ] `api_version` is present in the engine, the fixture and the manifest, and the adapter's `validate()` refuses a mismatch. The fixture is at `web/public/fixture/scenario.json` with the keys of [ARCHITECTURE.md §3.5](ARCHITECTURE.md).

#### T01 ★ Site geometry without data
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1 h (part of §4 item 7) · **Blocked by:** T00
- **Touches:** `crates/core/src/` (projection, local frame), `crates/core/tests/`
- **What to build:** convert between lat/lon, map x/y and MOON_ME unit vectors; build the local east-north-up frame at a site; return azimuth and elevation of any 3D point seen from the site. No DEM and no ephemeris. This is the one prefactor before the tracer.
- **Acceptance criteria**
  - [ ] T-1a-2 (no-data part): forward then inverse projection returns lat/lon to 1e-9 rad. SP-01 (222.69°E, 89.45°S) maps to (−11.31, −12.26) km. The IM-2 site (84.7906°S, 29.1957°E) maps to (+77.1, +138.0) km.
  - [ ] The vector form and the closed form of elevation agree; the curvature drop is 0.7 m at 1.6 km, 720 m at 50 km and 6.5 km at 150 km (§1a).
  - [ ] Flat-sphere dip is −0.0869° at H = 2 m and −0.01944° at H = 0.1 m; everything is f64 (N7).
  - [ ] At the pole the frame is north = +X, east = +Y and no output is NaN (V5 case 1, frame part).
  - [ ] P9: from a site at 85°S, 0°E a point toward the body's north pole has azimuth 0° and a point to the east has azimuth 90°.

#### T02 ★ Tracer: real Sun and Earth az/el for one site and date on the deployed page
- **Labels:** owner:science, hard-science · **Owner:** Qi Cai · **Est:** 1.5 h (part of §4 item 5) · **Blocked by:** T00, T01
- **Touches:** `crates/pipeline/src/` (table builder), `crates/core/src/` (interpolation, topocentric step), `crates/wasm/src/lib.rs`, the ephemeris table and `manifest.json` in the web static folder, one switch in the adapter, one Horizons fixture under `crates/core/tests/fixtures/horizons/`
- **What to build:** the deployed page shows the Sun's and the Earth's azimuth and elevation for the IM-2 site (84.7906°S, 29.1957°E, mast height 2 m) on 2026-11-14, computed in the browser by `core` from the real hourly table. Ground height is 0 m and the horizon is the flat-sphere dip, both labelled "no terrain yet".
- **Acceptance criteria**
  - [ ] First step: `LT_S` exists in ANISE 0.10.6 (checked Nov 12). Use `default-features = false` (defaults pull ureq and TLS). Confirm the kernels load.
  - [ ] The table is on whole UTC hours built with hifitime; positions are apparent (light time and stellar aberration), Moon-centred, translated in J2000 and then rotated to MOON_ME (§1c steps 1–2).
  - [ ] T-1c-1: table + interpolation against native ANISE at 1000 random instants, max error under 0.001°.
  - [ ] One Horizons pair for this site and date agrees within 0.01° for both bodies, azimuth compared as Δaz·cos(el). The request text is stored with the fixture (P15).
  - [ ] A time outside the table returns an error (P16).
  - [ ] The deployed URL shows the same four numbers as the native run, and the FIXTURE badge is off for this view. If T03 has not merged, a plain text readout is enough.
  - [ ] Merged and deployed by **Sat 12:00**.

#### T04 TIFF decode probe on the PSR file
- **Labels:** owner:science · **Owner:** Qi Cai, as a background agent during T01 · **Est:** 0.25 h [est] · **Blocked by:** T00 (the 148 MB download)
- **Touches:** `crates/pipeline/src/` (GeoTIFF reader), `docs/DATA-NOTES.md`
- **What to build:** proof that the `tiff` crate reads the PGDA cloud-optimized GeoTIFFs tile by tile, before the 2.6 GB DEM arrives.
- **Acceptance criteria**
  - [ ] One tile of `LPSR_80S_20MPP_ADJ.TIF` is decoded without reading the whole raster (N6, F19); compression, predictor and tile size are already known (next line); only the decode remains.
  - [x] The GeoTIFF tags are written down (Nov 12, `stack_preflight` test, all three DEMs): BigTIFF, Deflate (8), floating-point predictor (3), Float32, 512² tiles, tiepoint upper-left (−304 000, 304 000) m at 20 m (240 m file: ±931 200), GTRasterType = 1 pixel-is-area, no-data "nan". tiff 0.11.3 supports this combination.
  - [ ] If decoding fails: say so in the issue, and convert once with `gdal_translate -co COMPRESS=NONE` on a machine that has GDAL. T06 must not wait more than 30 minutes for this.

#### T05 ★ V1: 20 Horizons pairs through the table path
- **Labels:** owner:science, hard-science · **Owner:** Qi Cai · **Est:** 1 h (rest of §4 item 5) · **Blocked by:** T02
- **Touches:** `scripts/` (fixture fetch), `crates/core/tests/fixtures/horizons/`, `crates/core/tests/` (V1 and Earth sanity tests), `scoreboard.json`
- **What to build:** the V1 row of the scoreboard, with a real worst-case error.
- **Acceptance criteria**
  - [ ] 20 site/time pairs as in §3 "V1 details": the pole, a point 0.1° from it, SP-01, IM-1, IM-2, a far-side point, region centres; times across 2026–2030 including each libration extreme found in the table and both sides of a year boundary; two sites at +5 km and −3 km altitude (P15).
  - [ ] Requests follow §1c (`coord@301`, geodetic east longitude, latitude, height in km, quantity 4, UT, degrees, extra precision). The first response header confirms the 1737.400 km radius (U13).
  - [ ] **V1: worst case ≤ 0.002° over the 20 pairs**, Sun and Earth, azimuth as Δaz·cos(el) with the 0/360 wrap handled (N8). The comparison goes through the shipped table and interpolation (N3).
  - [ ] P3: turning aberration off moves the Sun by 0.005–0.006°. P4: positions are continuous from 2026-12-31T23:59 to 2027-01-01T00:01 UTC. P8: PA instead of ME differs by more than 0.01°.
  - [ ] P9: at 85°S, 0°E the Earth is within 9° of azimuth 0, and the Sun's azimuth decreases at about 0.5°/h.
  - [ ] T-1d-1: at (−60°, 180°) on a flat sphere the Earth's upper limb stays below −20° through 2026–2030. T-1d-1b: at (−85°, 180°) the Earth-centre elevation peaks between +1.0° and +1.7° and is negative more than half the time.
  - [ ] `scoreboard.json` has the V1 entry. A miss is written as a miss with its number.

#### T06 ★ DEM tiles for two regions and the far field
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1.5 h (rest of §4 item 7) · **Blocked by:** T01, T04, and the DEM download
- **Touches:** `crates/pipeline/src/` (GeoTIFF reader, crop, encode), `crates/core/src/` (DEM sampler interface, bilinear lookup), two near tiles, one far tile and `manifest.json` in the web static folder, `docs/DATA-NOTES.md`
- **What to build:** the shipped terrain for Mons Mouton and the Shackleton ridge, in the §1 formats, with a sampler that fails loudly outside coverage.
- **Acceptance criteria**
  - [ ] U5 and U6 are read from `LDEM_80S_20MPP_ADJ.TIF` and written down before any crop: tiepoint, pixel scale, pixel-is-area or point, no-data value, height reference.
  - [ ] T-1a-2 (data part): SP-01 lands on a ridge crest, and all seven Barker Site 1 RoI centroids are higher than the mean height on a 200 m ring around them (N9).
  - [ ] Two near tiles (20 m) and one far tile (240 m, ±382 km) are written. Near `scale` is the finest of 0.0625, 0.125 and 0.25 m that fits the tile (P11). The encoder fails if any height clips.
  - [ ] The manifest has no `h_max` (the engine computes it at load). Data file names carry a content hash.
  - [ ] Round trip: decoded heights are within half a `scale` step of the source at 1000 random pixels.
  - [ ] The sampler returns an error for a point outside coverage or on no-data, never 0 m (P5).
  - [ ] Tile centres are chosen so that the IM-2 site, SP-01, the seven RoIs and at least 20 PSR-floor pixels are each at least 7 km inside a tile edge [est].

#### T07 ★ Terrain horizon in the browser
- **Labels:** owner:science, hard-science · **Owner:** Qi Cai · **Est:** 1.75 h (part of §4 item 8) · **Blocked by:** T06
- **Touches:** `crates/core/src/` (horizon marcher), `crates/core/tests/`, `crates/pipeline/src/` (ablation run), `crates/wasm/src/lib.rs`, one switch in the adapter, `docs/DATA-NOTES.md`
- **What to build:** pick a site and see its real 720-azimuth skyline in the panorama, computed in the browser from the shipped tiles.
- **Acceptance criteria**
  - [ ] T-1a-1: flat sphere, H = 2 m gives −0.0869° at every azimuth.
  - [ ] T-1a-3: a 1000 m cone at 50 km, at an azimuth off the map axes, matches the closed form to 0.002° (P10: the march follows the great circle).
  - [ ] N11: at the highest point of a tile, the horizon with the early exit on equals the horizon with it off.
  - [ ] The observer's own pixel is skipped; an obstacle under 3 pixels away raises the flag (P13).
  - [ ] A site 5 km inside the far-field boundary returns a coverage error (V5 case 6).
  - [ ] T-1a-4 report (native): shipped tiles against the full 20 m DEM at 5 points, at H = 0 and H = 2 m. Above 0.05°, open a follow-up for an 80 m tier or a wider near tile.
  - [ ] T-1a-5 report: SP-01, H = 10 m, range 150 km against the tile edge.
  - [ ] U18: the time of one native horizon and one in-browser horizon is written down. Two thresholds: 50 ms is the target; over 2 s [est] in the browser fires slip cut 3.
  - [ ] The kernel benchmark (horizon march on a synthetic tile) runs Sat 10:30–12:30 in the background (§3 schedule, gate G2), not here.
  - [ ] The deployed panorama shows the real skyline at the IM-2 site, and the FIXTURE badge is off for the horizon.

#### T08 ★ Sun-disc fraction and timelines
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1.25 h (rest of §4 item 8) · **Blocked by:** T02, T07
- **Touches:** `crates/core/src/` (disc fraction, timeline), `crates/core/tests/`, `crates/wasm/src/lib.rs`, one switch in the adapter
- **What to build:** the 30-day, 10-minute timeline for the picked site from real data: illumination, visibility, Earth margin and contact.
- **Acceptance criteria**
  - [ ] T-1b-1: a flat horizon at u·ρ gives f = (acos u − u·√(1 − u²)) / π within 2e-3 using 32 midpoint strips; u = 0 gives 0.5.
  - [ ] T-1b-2: a hidden disc gives exactly 0. N2: a clear disc gives exactly 1 (normalised by Σ 2w).
  - [ ] ρ is recomputed at each step from the Sun's distance and stays within 0.261°–0.272° (Moon-centred; widened Nov 14).
  - [ ] P1: visibility ≥ illumination at every step, and the two outputs have those names.
  - [ ] N8: the margin is continuous as the Sun crosses azimuth 0/360.
  - [ ] Earth: margin = el_earth − el_hor(az_earth); contact = margin ≥ 2° by default, labelled "assumed". "Any fraction of the Earth visible" is a separate, differently named function (P14).
  - [ ] The deployed timelines for the IM-2 site read real data.

#### T09 ★ Survival: state of charge, operable window, first lethal shadow
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1.5 h (§4 item 1) · **Blocked by:** T08
- **Touches:** `crates/core/src/` (power, presets), `crates/core/tests/`, `crates/wasm/src/lib.rs`, one switch in the adapter
- **What to build:** the survival headline for one lander preset: operable days and time to first lethal shadow.
- **Acceptance criteria**
  - [ ] T-1e-1: no generation, 100 W load, 705 Wh start, E_min = 0 gives death at 7.050 h ± 1e-9, interpolated from the unclamped value.
  - [ ] T-1e-2: energy is conserved when no clamp is hit.
  - [ ] Solar flux is 1366.1 W/m² scaled by (1 AU / d)². A Sun-tracking panel and a fixed panel both work; a vertical panel facing azimuth a gives cos(el)·cos(az − a).
  - [ ] The simulation stops at the first lethal event. "No lethal shadow in the span" is its own result, not a large number.
  - [ ] Operable window = illuminated AND in Earth contact. Longest darkness (f = 0) and longest any-part-visible period follow the E123 definitions.
  - [ ] One preset uses the cited values (10 000 Wh, 1.5 m², 30 %, 220 W) marked "cited"; every other value is marked "assumed" (U10).
  - [ ] The deployed headline for the IM-2 site reads real data.
  - [ ] The pure tests need no DEM. If the DEM download is late, do this ticket's `core` part before T06.

#### T10 ★ V5 red-team suite
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1 h (§4 item 2) · **Blocked by:** T05, T07, T08, T09
- **Touches:** `crates/core/tests/` (red-team tests), `crates/pipeline/src/` (PSR run), `scoreboard.json`
- **What to build:** the V5 row of the scoreboard. Must is complete when this merges.
- **Acceptance criteria** (V5 cases 1–7 in §3)
  - [ ] Case 1: at the pole no output is NaN, and the flat-sphere horizon is −0.0869° at H = 2 m at every azimuth.
  - [ ] Case 2: flat pole, 2026–2030: Sun elevation within ±1.6°, Earth-centre elevation within ±7.2°.
  - [ ] Case 3: 20 pixels well inside regions flagged in `LPSR_80S_20MPP_ADJ.TIF`: visibility is exactly 0 at every hourly step of 18.6 years (vectors generated natively, not shipped). No edge pixels (U15).
  - [ ] Case 4: year boundary and both table ends (P4, P16).
  - [ ] Case 5: far side at 60°S, Earth never visible (T-1d-1).
  - [ ] Case 6: DEM edge returns a coverage error (P5).
  - [ ] Case 7: T-1e-1 and T-1e-2.
  - [ ] `scoreboard.json` lists each case as pass or miss.


### Must: app chain

Every app ticket is built on the fixture first and switches to WASM when the science ticket named under "real data" merges. A switch is one line in the adapter plus a look at the deployed page.

#### T03 Panorama page on the frozen API
- **Labels:** owner:app · **Owner:** Charls · **Est:** 2.5 h [est] (part of §4's 4 h for panorama and timelines) · **Blocked by:** T00
- **Touches:** `web/src/lib/` (panorama component, adapter), `web/src/` (page shell, design tokens from `design/DESIGN.md`), the Playwright smoke test
- **What to build:** a 360° panorama: azimuth across, elevation up, the horizon line, the Sun and Earth discs and their tracks over the chosen span, and a marker for the current time.
- **Acceptance criteria**
  - [ ] The fixture horizon and tracks are drawn and the page is deployed by Sat 11:00 with the FIXTURE badge showing.
  - [ ] Discs are drawn to scale (Sun 0.53°, Earth 1.9° across) and the elevation axis can zoom to ±10°, so a Sun at 1° is readable.
  - [ ] Pointing at an azimuth shows the horizon elevation and the distance to the obstacle that sets it (P13).
  - [ ] After T02 merges, the Sun and Earth numbers on the page equal T02's native output, and the badge is off for that view.
  - [ ] The smoke test on the deployed URL checks that the readout exists and whether the badge shows.
  - [ ] Colours, type and spacing come from the design tokens only.

#### T11 Site and date controls
- **Labels:** owner:app · **Owner:** Charls · **Est:** 1 h [est] (part of §4's 2 h for controls and headline) · **Blocked by:** T03
- **Touches:** `web/src/lib/` (controls, site picker, shared state)
- **What to build:** choose a region, then a site by clicking a greyscale height image of the near tile or by typing lat/lon; choose a landing date, a span and a mast height.
- **Acceptance criteria**
  - [ ] The height image is drawn from the shipped int16 tile itself (the fixture tile until T06); no extra image asset.
  - [ ] A click closer than 7 km to the tile edge is refused with the reason shown (P5).
  - [ ] A date outside the ephemeris table is refused with the reason shown (P16).
  - [ ] Changing the site, date or mast height recomputes every view on the page.

#### T12 Timelines chart
- **Labels:** owner:app · **Owner:** Charls · **Est:** 2 h [est] (1.5 build + 0.5 switch to real) · **Blocked by:** T03 (real data: T08)
- **Touches:** `web/src/lib/` (uPlot timelines component, adapter switch)
- **What to build:** timelines under the panorama: illumination, visibility, Earth margin with its threshold line, Earth contact, and the operable window as a band.
- **Acceptance criteria**
  - [ ] 30 days at 10 minutes (4320 points per series) pans and zooms smoothly.
  - [ ] The chart cursor and the panorama's time marker move together.
  - [ ] Illumination and visibility are separate, labelled series (P1). The Earth margin threshold is labelled "2° (assumed)".
  - [ ] After T07 and T08 merge, the chart and the panorama read real data and the badge is off for both.

#### T13 Scoreboard and About page
- **Labels:** owner:app · **Owner:** Charls · **Est:** 1.5 h (§4's 2 h for deploy and scoreboard, less the deploy that the template already has) · **Blocked by:** T00 (real data: T05)
- **Touches:** `web/src/lib/` (scoreboard and About components); reads `scoreboard.json`
- **What to build:** the validation scoreboard: one row per check, V1 to V5, with the outside judge and the real number.
- **Acceptance criteria**
  - [ ] Each row shows the judge, the metric compared (it says "visibility" or "illumination", P1), the threshold, our value, the sample size and pass or miss.
  - [ ] A check that has not run shows "not run". It is never blank and never a pass.
  - [ ] A Limitations list: no limb darkening; terrain smaller than a 20 m pixel is not in the DEM; the Earth margin and σ_z are assumed; no charge or discharge efficiency; lander values are assumed unless marked cited.
  - [ ] Every dataset is listed with its role and citation (rubric 4 and 10).
  - [ ] It renders the sample scoreboard at first and the real V1 row after T05 merges.

#### T14 Survival headline and lander preset form
- **Labels:** owner:app · **Owner:** Charls · **Est:** 1.5 h [est] (1 build + 0.5 switch to real) · **Blocked by:** T11, T12 (real data: T09)
- **Touches:** `web/src/lib/` (headline, preset form)
- **What to build:** the headline a planner reads first: operable days and time to first lethal shadow for the chosen lander preset.
- **Acceptance criteria**
  - [ ] The headline shows operable days and time to first lethal shadow, or "no lethal shadow in N days" when there is none in the span.
  - [ ] The state-of-charge trace is drawn on the timelines, with the lethal moment marked.
  - [ ] Every preset field shows "cited" or "assumed" (U10). Editing a field recomputes the headline.
  - [ ] After T09 merges, the headline reads real data and no FIXTURE badge is left on the page.

#### T26 Sun and Earth elevation readout with sunset and Earthset countdown
- **Labels:** owner:app · **Owner:** Charls · **Est:** 0.5 h [est] · **Blocked by:** T03 (real data: T08)
- **Touches:** `web/src/lib/` (readout component, beside the panorama)
- **What to build:** a large, always-visible readout at the current time marker: Sun elevation, Earth elevation, and the time left until the next sunset and the next Earthset. The challenge asks for this directly (docs/research/challenges-2026.md §11).
- **Acceptance criteria**
  - [ ] Sun and Earth elevation (degrees, one decimal) update as the time marker moves.
  - [ ] "Sunset in Xd Yh" is the time to the next step where the disc fraction drops to 0; "Earthset in Xd Yh" is the time to the next step where Earth falls below the terrain mask plus margin (§1d). Both are read from the T08 timelines, with no new science call.
  - [ ] When there is no crossing in the span, it says "no sunset in N days", never a blank or a negative time.
  - [ ] After T08 merges, it reads real data and no FIXTURE badge is left on it.

### Should: on the schedule

#### T15 ★ IM-2 replay: replay segment, local pit, V4 statement 3
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1.5 h (§4 item 4) · **Blocked by:** T08
- **Touches:** `crates/core/src/` (pit model), `crates/core/tests/`, `crates/pipeline/src/` (replay segment of the ephemeris table), the replay table and `manifest.json` in the web static folder, `crates/wasm/src/lib.rs`, `scoreboard.json` (V4)
- **What to build:** the IM-2 site on its landing date, with two answers: what regional terrain says, and how deep a 20 m pit must be to hide the Sun from a panel at height H.
- **Acceptance criteria**
  - [ ] A replay segment of the ephemeris table covers 2025-03-06 with margin; T-1c-1 holds on it.
  - [ ] Pit model (§1f): a 20 m pit with an 11° slope is 1.94 m deep; el_pit is 11.0° at H = 0, 5.4° at 1 m, 2.9° at 1.44 m and −0.3° at 2 m.
  - [ ] The Sun's elevation at the IM-2 latitude never exceeds 6.75°.
  - [ ] V4 statement 3: el_sun at touchdown (2025-03-06 17:28 UTC, marked "assumed", U8) from regional terrain; d_c* = H + (D_c/2)·tan(el_sun + ρ); and whether the 1.9 m degraded-crater depth exceeds d_c*.
  - [ ] The result is worded "consistent with reports". It is never called validated and never stated as what happened (N10).
  - [ ] IM-1 (statements 1 and 2) is not attempted here: its far field is out of range.

#### T16 ★ IM-2 replay view
- **Labels:** owner:app · **Owner:** Charls · **Est:** 1 h [est] · **Blocked by:** T14, T15
- **Touches:** `web/src/lib/` (replay view, pit control)
- **What to build:** one click loads the IM-2 site, date and preset and shows the two answers side by side (A3).
- **Acceptance criteria**
  - [ ] "Regional terrain says X" and "a pit of D × d says Y" are shown together, in the panorama and in the headline.
  - [ ] Pit diameter and depth are editable; the defaults are 20 m and the 11° depth.
  - [ ] The d_c* sentence from T15 is shown in plain words.
  - [ ] Touchdown time, lander values and "lasted about a day" carry "assumed" or "unconfirmed" tags (U8, U10, U19).

#### T17 Sunset ± band
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1 h (§4 item 3) · **Blocked by:** T08
- **Touches:** `crates/core/src/` (band), `crates/core/tests/`, `crates/wasm/src/lib.rs`
- **What to build:** a ± time on every sunrise and sunset in the timeline, from first-order propagation of DEM height error (§1f step 1).
- **Acceptance criteria**
  - [ ] σ_el = √(σ_obstacle² + σ_observer²) / D, with σ_z = 0.5 m labelled "assumed". The `_ERR` raster is not downloaded.
  - [ ] The rate is the central difference of el_sun − el_hor(az_sun) across the crossing in the 10-minute timeline, not 0.5°/h times a slope.
  - [ ] Checks: a 0.5 m error on an obstacle 1 km away is 0.03°, and 0.003° at 10 km.
  - [ ] At H = 5 m the band is small (E28).
  - [ ] A near-flat horizon returns a wide band. It never returns a single time and never divides by zero.

#### T18 Band display
- **Labels:** owner:app · **Owner:** Charls · **Est:** 0.5 h [est] · **Blocked by:** T12, T17
- **Touches:** `web/src/lib/` (timelines component)
- **What to build:** each sunrise and sunset on the timelines shows its band ("sunset 14:20 ± 40 min").
- **Acceptance criteria**
  - [ ] The band is drawn as a shaded interval and written as ± minutes; "σ_z = 0.5 m (assumed)" is visible.
  - [ ] A band wider than 6 hours [est] is written as "ill-conditioned", not as a number of minutes.

#### T19 A1: longest-night table check
- **Labels:** owner:science, hard-science · **Owner:** Qi Cai · **Est:** 1 h (§4 item 6) · **Blocked by:** T10 (§4: starts only after Must is deployed) · **Scheduled:** in slack only
- **Touches:** `crates/pipeline/src/` (table check run), `scoreboard.json`
- **What to build:** a scoreboard row that checks the headline metric (longest night) against published tables.
- **Acceptance criteria**
  - [ ] Barker Site 1, Δz = 5 m, 2024-01-01 to 2026-01-01, hourly; for each RoI, our 20 m pixels within √(Area/π) of the centroid are averaged. Pass if LCSP ≤ LCSP-99 + 1 day and LCIP ≥ LCIP-1 − 10 days for at least 6 of 7 RoIs (values in SCIENCE-SPEC §3 V2b).
  - [ ] Mazarico SP-01, H = 10 m, one cycle from 2020 (marked "assumed", U20): 2.75 days lies inside the range of longest total night over every 20 m pixel within 150 m of (−11.31, −12.26) km. Both the f > 0 and the f = 1 day lengths are reported.
  - [ ] The row says the test is one-sided and names the DEM each reference used (P12).

### Should: overflow (filed, not on the schedule; pulled only through a gate in §3)

#### T20 V2 sample against AVGVISIB
- **Labels:** owner:science, hard-science · **Owner:** Qi Cai · **Est:** 2 h (§4 item 9) · **Blocked by:** T10
- **Touches:** `crates/pipeline/src/` (V2 run), `scoreboard.json`, `docs/DATA-NOTES.md`
- **What to build:** the V2 row: our visibility against NASA's `AVGVISIB_85S_060M_201608` at 10 000 stratified pixels, and the Earth twin.
- **Acceptance criteria**
  - [ ] Setup as in §3 "V2 details": visibility metric (P1), H = 0, hourly over 18.6 years from 2010-01-01, our near field averaged 3 × 3 to 60 m.
  - [ ] 10 000 pixels in five bins of the reference value; a bin with fewer than 2 000 takes all of them and prints the count.
  - [ ] Reported per bin and pooled: median, p90, mean signed error, Pearson r. **Pass: pooled median ≤ 5 points, |mean signed error| ≤ 2, r ≥ 0.9.**
  - [ ] Also reported: a 500-pixel subsample at H = 2 m (P2); the change when the start shifts by 4.65 years, expected under 0.5 points (U2); pixels poleward of 88°S apart from the rest (U3); pixels whose obstacle is closer than 180 m apart from the rest (N5).
  - [ ] The Earth twin uses "any fraction visible" (P14).
  - [ ] The residual map is looked at for paired bright and dark fringes on shadow edges (P7).

#### T21 Shadow mask over a tile
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 0.75 h [est] (not in §4) · **Blocked by:** T07
- **Touches:** `crates/core/src/` (grid shading), `crates/core/tests/`, `crates/wasm/src/lib.rs`
- **What to build:** for one instant, lit or shadowed at every point of a grid over the near tile, using one ray toward the Sun per point (the §3 V3 method).
- **Acceptance criteria**
  - [ ] At 20 points the mask agrees with the full-horizon visibility from T08 wherever the Sun's whole disc is clear or hidden.
  - [ ] The mask covers the near tile only, or its time limit is relaxed: a full march for 256 × 256 is 75.6 M samples, so "under 1 s" does not fit ([ARCHITECTURE.md §2.6](ARCHITECTURE.md)).
  - [ ] The output is labelled as a point-Sun mask, not as illumination.

#### T22 2D shaded map
- **Labels:** owner:app · **Owner:** Charls · **Est:** 1.5 h [est] · **Blocked by:** T11, T21
- **Touches:** `web/src/lib/` (site picker)
- **What to build:** the site picker's height image coloured by the shadow mask at the current time, so scrubbing time sweeps the shadow edge across the map. This is what slip cut 1 leaves in place of 3D.
- **Acceptance criteria**
  - [ ] Scrubbing the time marker redraws the mask without blocking the page.
  - [ ] The map and the panorama agree at the pinned site: lit on the map when the Sun is above the skyline.

#### T23 3D terrain coloured by the model
- **Labels:** owner:app · **Owner:** Charls · **Est:** 4 h (§4) · **Blocked by:** T11, T21
- **Touches:** `web/src/lib/` (Three.js terrain view)
- **What to build:** the near tile as a 3D mesh whose vertex colours come from the model's shadow mask. No GPU shadow maps (STACK).
- **Acceptance criteria**
  - [ ] Vertex shading equals the T21 mask at the current time.
  - [ ] The pin, the Sun direction and the time marker stay in step with the panorama.
  - [ ] No vertical exaggeration by default; any exaggeration is labelled.

#### T24 Heatmap for one region
- **Labels:** owner:science, owner:app · **Owners:** Qi Cai (2 h, §4 item 10), Charls (1 h [est]) · **Blocked by:** T09
- **Touches:** `crates/pipeline/src/` (grid run), a derived-numbers grid in the web static folder, `web/src/lib/` (heatmap layer)
- **What to build:** for one region and one preset, a map of operable days by site at 100 m spacing, for a small set of landing dates.
- **Acceptance criteria**
  - [ ] Derived numbers are stored, not horizons (§4 compute notes).
  - [ ] Five grid points picked at random equal the live in-browser result at the same site and date.

#### T25 Widen to nine regions
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 0.5 h [est], mostly machine time · **Blocked by:** T06
- **Touches:** seven more near tiles and `manifest.json` in the web static folder
- **What to build:** the other seven Artemis III regions in the region picker.
- **Acceptance criteria**
  - [ ] Every tile passes T06's round-trip and coverage checks.
  - [ ] The runtime data total stays under 100 MB.

#### T27 Two site-date pairs side by side (A vs B)
- **Status (Nov 14):** promoted from overflow to Should (Q10). The challenge page asks for this comparison twice. Wireframe: screen 6 in design/DECISIONS.md.
- **Labels:** owner:app · **Owner:** Charls · **Est:** 1 h [est] · **Blocked by:** T14
- **Touches:** `web/src/lib/` (compare view), the URL state
- **What to build:** pin two site-date pairs and see their survival headlines and timelines next to each other. The challenge asks for quick scenario comparison across site-date pairs.
- **Acceptance criteria**
  - [ ] A and B each show operable days, time to first lethal shadow and the T26 countdown, using the same lander preset.
  - [ ] The difference (B minus A) is shown for operable days.
  - [ ] Both pairs live in the URL, so a comparison can be shared.

### Stretch parking (not filed on Nov 14 unless pulled)

| Item | Owner | Est (h) | Needs |
|------|-------|---------|-------|
| Monte Carlo bands (§1f step 2) | Qi Cai | 1.5 | T17 |
| IM-1 replay (V4 statements 1 and 2) | Qi Cai | 1.5 | a far field out to 500 km; a Feb 2024 replay segment |
| V3 relative IoU (A2) | Qi Cai | 3 | T21; NAC acquisition times (U7) |
| JSON API and dataset | Qi Cai | 1.5 | T09 |
| Share links | Charls | 0.5 [est] | T11 |
| Heatmaps for all nine regions | both | [est] 2 | T24, T25 |

### Submission (D1–D7, owners as in SUBMISSION.md)

Two event checkpoints bind these: **footage by Sat 13:00** and **proofreading at Sun 13:00**. None of these tickets is ever cut.

#### D1 Project document (Typst → PDF)
- **Labels:** owner:science, owner:design · **Owners:** Qi Cai (science sections), Charls (figures and layout) · **Est:** Qi Cai 1 h [est] (0.5 review before the freeze, 0.5 final after), Charls 0.5 h [est] · **Blocked by:** T05 for the first real numbers; the final build waits for the freeze
- **Due:** draft Sun 12:30 for the 13:00 proofreading; final PDF Sun 17:00
- **Touches:** `docs/paper/` (Typst source, references, figures); reads `scoreboard.json`
- **What to build:** the project PDF in the research-paper outline. A background sonnet agent drafts Methodology from SCIENCE-SPEC §1 while Qi Cai works on T10; Qi Cai reviews, and does not write from scratch.
- **Acceptance criteria**
  - [ ] Sections: Introduction, Methodology, Results, Discussion, Conclusion, References; the project name with the team poster is item 1.
  - [ ] Methodology carries the §1 equations (rubric 5) and an architecture diagram: one Rust model, native and WASM (rubric 3).
  - [ ] A dataset table gives each NASA dataset its role and citation (rubric 4 and 10).
  - [ ] A comparison table against Moon Trek, QuickMap and Horizons (rubric 2).
  - [ ] A commercialization paragraph: who pays, the open API plus a paid mission-planning service, the cost of a failed lander against the cost of the tool (rubric 7).
  - [ ] The hook says "Next Frontier" (rubric 1) and claims only the checks that ran (§4 hook wording).
  - [ ] Results numbers are pulled from `scoreboard.json`, misses included. A Limitations section matches the About page.

#### D2 Team poster
- **Labels:** owner:design · **Owner:** Charls · **Est:** 1 h [est] · **Blocked by:** T03 (first real screenshot) · **Due:** Sun 10:30
- **Touches:** Canva (template made before the event); export to `docs/submission/`
- **Acceptance criteria**
  - [ ] Project name (Penumbra), team name (Fractured Toes) and member names (Qi Cai, Charls), the hook, one real screenshot, the scoreboard numbers that exist, the NASA data used.
  - [ ] Exported as PDF and placed in the shared Drive folder.

#### D3 30-second demo video (D3a: first footage)
- **Labels:** owner:design · **Owner:** Charls · **Est:** D3a 0.5 h [est], cut 1 h [est] · **Blocked by:** D3a: T02 and T03; cut: T14
- **Due:** D3a on the shared Drive by **Sat 13:00**; cut by Sun 10:00
- **Touches:** screen capture, Canva video; `docs/submission/` for the script
- **Acceptance criteria**
  - [ ] D3a: a capture of the deployed tracer page (real Sun and Earth tracks on the panorama) is on the shared Drive before the Sat 13:00 footage review. A second capture with the real skyline and headline follows Sun 08:30.
  - [ ] The cut follows the guidebook beats: 0:00 intro, 0:02 challenge, 0:05 name and tagline, 0:10 how it works and the NASA data, 0:20 the future, 0:30 end.
  - [ ] mp4, 30 seconds, every frame from the real app.

#### D4 3–5 minute project video
- **Labels:** owner:design, owner:science · **Owners:** Charls (script, recording, edit), Qi Cai (science voice-over) · **Est:** Charls 3.5 h [est] (script 1 before the freeze; record and edit 2.5 after), Qi Cai 0.5 h [est] · **Blocked by:** T16 for the story; recording waits for the freeze
- **Due:** script by Sun 13:00 for proofreading; recorded by Sun 17:00
- **Touches:** `docs/submission/` (script), screen capture, Canva video
- **Acceptance criteria**
  - [ ] Aim for 4:30 (the guidebook says 3–5 minutes on one page and 3–7 on another).
  - [ ] The story rests on what exists at the freeze: IM-2 on its side; the same site and date in Penumbra; regional terrain against the pit; move the pin or the date; the scoreboard. Ranking nine regions and the open API appear only if T24 or the API landed.
  - [ ] It shows the architecture diagram and names each NASA dataset on screen (rubric 3, 4, 10) and has one commercialization line (rubric 7).
  - [ ] No number is spoken that is not on the scoreboard or the page.

#### D5 Global demo link
- **Labels:** owner:design · **Owner:** Charls · **Est:** 0.25 h [est] · **Blocked by:** D3
- **Acceptance criteria**
  - [ ] The 30-second video is the global demo link (the guidebook allows the video or a 7-slide deck). The deck is made only if Charls is ahead at Sun 13:30.
  - [ ] The link opens for a logged-out viewer.

#### D6 Final project link
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 0.5 h [est] · **Blocked by:** the freeze · **Due:** Sun 15:30
- **Touches:** `README.md`, the deployed site
- **Acceptance criteria**
  - [ ] The deployed URL opens logged-out in a private window, and the smoke test is green on the final commit.
  - [ ] The repo is the public `penumbra`: Apache-2.0, a README with a screenshot, run steps, data sources and paper citations, at least one visible core test, and no leftover agent or prompt text.
  - [ ] No file from `research/**/corpus/` is in the public repo's tree or in any of its commits.

#### D7 Project page fields
- **Labels:** owner:science · **Owner:** Qi Cai · **Est:** 1.25 h [est] (0.25 review before the freeze; 1 fill and submit after) · **Blocked by:** D3, D4, D6 for their links
- **Due:** drafts by Sun 12:30 for proofreading; submitted by **Sun 21:00**
- **Touches:** `docs/submission/` (one draft per field)
- **Acceptance criteria**
  - [ ] Fields drafted: high-level summary, demo link, final project link, details (tools and languages), NASA data and partner data with citations, references. A background agent drafts them from IDEA.md and `scoreboard.json`; Qi Cai reviews.
  - [ ] Page editing opens Sun 09:00. Lock the page while editing, save the draft often, then Publish, then Submit for Judging.
  - [ ] Every Drive link is "Anyone with the link: Viewer" and is opened once logged-out. The videos are also on YouTube (KICKOFF ship checklist).

---

## 3. Schedule

**Assumption (see §6 Q1):** the 16 working hours each are build time before the freeze: 10 h on Saturday and 6 h on Sunday. Sunday 15:00–21:00 is submission-only time on top of that. All clock times are [est]; the ticket hours behind them are tagged in §2.

Fixed points: **deployed Sat 12:00** · footage on the Drive Sat 13:00 · final project review Sun 08:30 · project page opens Sun 09:00 · proofreading Sun 13:00 · **freeze Sun 15:00** · demo recorded Sun 17:00 · submitted Sun 21:00 · **hard deadline Sun 22:00** (NSAKL26 Telegram, Nov 14; was 23:59).

### Saturday Nov 14

| Time | Qi Cai (science) | Charls (app, design) |
|------|------------------|----------------------|
| 09:00–09:30 | T00 ★ hour 0 | T00 ★ hour 0 |
| 09:30–10:30 | T01 ★ site geometry · T04 probe in the background | T03 panorama on the fixture |
| 10:30–12:00 | T02 ★ tracer → **deployed 12:00** · background `science-core` agent: horizon kernel on a synthetic tile + native `--release` timing (ARCHITECTURE §5 risk 1) | T03; fixture page deployed 11:00; switch to real 11:30–12:00 |
| 12:00–13:00 | 12:00 time the kernel in WASM on the deployed page (10 min) → **G2 at 12:30** · T05 ★ V1 | D3a footage → Drive by 13:00 (to 12:30) · T11 first half |
| 13:00–14:00 | lunch · footage review | lunch · footage review |
| 14:00–15:30 | T06 ★ DEM tiles | T11 second half (to 14:30) · T12 timelines |
| 15:30–17:15 | T07 ★ terrain horizon | T12 (to 16:00) · T13 scoreboard (to 17:30) |
| 17:15–18:30 | T08 ★ disc fraction, timelines | T14 headline (17:30–18:30) |
| 18:30–20:00 | T09 ★ survival | switch T12 to real (to 19:00) · T26 readout (to 19:30) · D4 script draft · D2 poster start |

End of Saturday, if on plan: every Must feature is on the deployed page with real numbers at the IM-2 site; V1 is on the scoreboard; V5 is the only Must item left. That state is what the Sun 08:30 review sees.

### Sunday Nov 15

| Time | Qi Cai (science) | Charls (app, design) |
|------|------------------|----------------------|
| 08:30–09:30 | T10 ★ V5 → **Must complete 09:30** · agents draft D1 and D7 in the background | switch T14 to real, footage v2 (to 09:00) · D3 cut |
| 09:30–11:00 | T15 ★ IM-2 replay science | D3 cut (to 10:00) · D2 finish (to 10:30) · D1 figures |
| 11:00–12:00 | T17 sunset band | T16 ★ IM-2 replay view |
| 12:00–12:30 | lunch | lunch |
| 12:30–13:00 | D1 and D7 review; rebuild the PDF from `scoreboard.json` | D4 script final · D5 |
| 13:00–13:30 | **proofreading:** read Charls's poster, 30-s video and script | **proofreading:** read Qi Cai's PDF and page fields |
| 13:30–14:30 | slack (1 h); T19 has first claim (gate G4) | T18 band display (to 14:00) · slack (0.25 h) |
| 14:30–15:00 | final scoreboard run, merge → **freeze 15:00** | final smoke test → **freeze 15:00** |
| 15:00–17:00 | D6 final link checks · D4 voice-over | D4 recording → **recorded 17:00** |
| 17:00–19:00 | D1 final PDF · D7 fields on the project page | D4 edit · export D2, D3, D4 |
| 19:00–21:00 | uploads, logged-out link check, Publish, **Submit by 21:00** | uploads, logged-out link check |

### Hours before the freeze

| | Must | Should on schedule | Submission before freeze | Slack | Total |
|---|------|--------------------|--------------------------|-------|-------|
| Qi Cai | 11.0 (T00–T02, T05–T10) | 2.5 (T15, T17) | 1.5 (D1 and D7 review, proofreading, final run) | 1.0 | 16.0 |
| Charls | 9.5 (T00, T03, T11–T14, T26) | 1.5 (T16, T18) | 4.75 (D3a, D3, D2, D1 figures, D4 script, D5, proofreading, final test) | 0.25 | 16.0 |

SCIENCE-SPEC §4 gave Qi Cai 3 h of slack. Submission work and hour 0 take 2 h of it.

### Critical path

T00 → T01 → T02 (**deployed Sat 12:00**) → T05 → T06 → T07 → T08 → T09 → T10 → T15 → T16 → D4.

- Qi Cai's tickets run one after another, so the science chain has no float until Sun 13:30. Any hour lost there comes out of the 1 h of slack first, then out of T17.
- Charls's tickets have float: each is finished on the fixture and waits only for a one-line switch. The exception is T16, which cannot start before T15 merges.
- The DEM is pre-downloaded (Nov 11) and verified (Nov 12); it is off the critical path unless the copy fails.

### Gates

| Gate | When | Test | If it fails or passes |
|------|------|------|-----------------------|
| G2 | Sat 12:30 | Does the in-browser horizon work (synthetic-tile benchmark, ARCHITECTURE §5 risk 1)? Target 50 ms per click, cut at 2 s [est] | No: slip cut 3 (§4). |
| G1 | Sat 14:00 | Is the 2.6 GB DEM in `data/` and readable? | No: do T09's `core` part now, then T06 when the file is fixed. |
| G3 | Sat 18:30 | Is Charls at least 1 h ahead, and does Qi Cai have 0.75 h? | Yes: pull T21 then T22. T23 (3D) only if Charls is 4 h ahead, which these estimates do not predict. |
| G4 | Sun 13:30 | Is Qi Cai's 1 h of slack unspent? | Yes: T19 (A1). |
| G5 | Sun 15:00 | Freeze | Anything unmerged is left out. No new feature after this, only D-tickets. |

### Claude usage

Both are on Claude Pro. [est: this assumes a 5-hour window opens at the first message; check `/usage` on the day.]

- **Offset the windows by two hours.** Charls sends a first message unrelated to the challenge at 07:00 on Saturday and 06:30 on Sunday. Qi Cai's first message is at 09:00 on Saturday and 08:30 on Sunday.

| | Charls's windows | Qi Cai's windows |
|---|------------------|------------------|
| Sat | 07–12 · 12–17 · 17–22 | 09–14 · 14–19 · 19–24 |
| Sun | 06:30–11:30 · 11:30–16:30 · 16:30–21:30 | 08:30–13:30 · 13:30–18:30 · 18:30–23:30 |

- A reset then arrives every two to three hours for one of the two, so they never both run dry in the same hour.
- **Heavy blocks** (long test-first agent runs): Qi Cai Sat 10:30–13:00 (T02, T05) and Sat 14:00–18:30 (T06–T08, all in one window); Charls Sat 09:30–12:00 (T03) and Sat 14:30–17:30 (T12, T13). Charls's window resets at 12:00 and 17:00, in the middle of Qi Cai's two heavy blocks.
- **Borrow rule:** if `/usage` shows under 30 % left at the end of T07, Qi Cai runs T08 from Charls's account (fresh at 17:00) while Charls is on T14.
- **Model rules** (WORKFLOW): the orchestrator is Opus at medium effort; no Opus high; Fable only through `science-hard`, and only on a hard-science ticket (T02, T05, T07, T19, T20) or after `science-core` fails the same test twice. Check `/usage` at the end of every ticket.
- **Light work to switch to when low:** the Horizons fixture fetch, watching downloads, footage capture, Canva, proofreading.
- Background agents for D1 and D7 drafts run on sonnet.

---

## 4. Slip cuts

### The four cuts from SCIENCE-SPEC §4, in its order

| # | Cut | Tickets | Trigger | State in this plan |
|---|-----|---------|---------|--------------------|
| 1 | 3D terrain with live shadows → a 2D map coloured by the same model output | T23 does not start; T22 (with T21) stands in | G3 | **Already taken.** Neither is on the schedule. |
| 2 | Nine regions → two (Mons Mouton, the Shackleton ridge) | T25 does not start | — | **Already taken.** T06 builds two tiles. |
| 3 | In-browser horizon in WASM → horizons precomputed natively on a 100 m grid for the two regions | T07 drops its WASM export and writes the grid (720 int16 per point, about 14 MB per 10 × 10 km); T11's pin snaps to the grid; T08 onward unchanged | G2 | Available. The panorama and timelines still work; only free clicking is lost. |
| 4 | V2 sample → the A1 table check alone | T20 does not start; T19 only | G4 | **Already taken.** T20 is overflow. |

Cuts 1, 2 and 4 are already spent: the D1–D7 work took the hours they would have freed. If the plan slips, they give nothing back.

### What is left to cut, in order (this list is mine, not from §4)

| # | Lever | Tickets | Saves |
|---|-------|---------|-------|
| a | Drop the sunset band | T17, T18 | Qi Cai 1 h, Charls 0.5 h |
| b | IM-2 replay as a statement on the scoreboard page instead of a view | T16 | Charls 1 h |
| c | V5 case 3 on 5 PSR pixels instead of 20 | T10 | machine time only |
| d | Skip A1 | T19 | nothing on the schedule; the hook loses its illumination check |

Never cut: T05 (V1), T10 (V5), D1–D7.

### Time triggers

| When | If | Then |
|------|----|------|
| Sat 10:45 | T01 is not merged | At 12:00 deploy the fixture page with its badge. Real numbers by 13:00. Sunday's slack is spent. |
| Sat 12:30 | In-browser horizon too slow or broken on the synthetic-tile benchmark (ARCHITECTURE §5 risk 1) | Cut 3. |
| Sat 13:00 | T02 still shows no real numbers | Lever a. Footage v1 is the fixture page, labelled; real footage follows by 15:00. |
| Sat 14:00 | DEM not readable from `data/` | G1. |
| Sat 20:00 | T09 is not merged | Finish it Sun 08:30, T10 after it, and take lever a. |
| Sun 11:00 | T15 is not merged | Lever b. The D4 script uses the regional-terrain statement only. |
| Sun 13:30 | Slack is gone | Lever d. Use the hook wording below. |
| Sun 15:00 | Anything unmerged | Left out. |

### Hook wording, by what landed

| Landed | Hook says the model is checked against |
|--------|----------------------------------------|
| V2 (T20) | "JPL Horizons and NASA's published LOLA illumination maps" (as in IDEA.md) |
| A1 only (T19) | "JPL Horizons and published LOLA illumination results (Barker 2021, Mazarico 2011)" |
| Neither | "JPL Horizons". D1 and the About page say the illumination checks were not run. |

---

## 5. Day-1 checklist: the first 60 minutes on Nov 14

Repo plan (update of Nov 11): planning lives in the private `qicai14/penumbra-burner`, which has the research corpus text committed. At 09:00 a **public** `qicai14/penumbra` is created from it **without the corpus**. The corpus is in the burner's history, so the public repo must start from a fresh history: export the tree without `research/**/corpus/`, `git init`, one commit, push. **Method chosen Nov 14:** the same local folder, an orphan `main` branch (no parent, so no burner history), the corpus untracked but kept on disk (gitignored), the burner kept as a second remote `burner`. Steps in [HANDOFF.md](HANDOFF.md) "Create the public repo". A fork, a mirror push or "use this template" would each carry the corpus, in the history or in the tree. All event work, issues and deploys happen in `penumbra`; the burner stays private as the evidence store.

### Before 09:00 (no challenge work)

| Qi Cai | Charls |
|--------|--------|
| Laptop on power; `scripts/bootstrap.sh` green in the burner clone; `/usage` checked | Same, in his burner clone |
| The export script has been dry-run to a local folder (Nov 13) and the folder holds no corpus file | `design/DESIGN.md` tokens final; Canva templates open |
| Cloudflare token and account ID at hand (not in any file) | Shared Drive folder set to "Anyone with the link: Viewer"; screen capture tested |
| Data verified against `MANIFEST.tsv` (done Nov 12) | First Claude message at 07:00, unrelated to the challenge (§3 usage) |

### 09:00–10:00

| Time | Qi Cai | Charls |
|------|--------|--------|
| 09:00 | Run the export; create public `qicai14/penumbra`; push the single commit. Check that `git log --all -- "research/**/corpus/*"` prints nothing. | Stand by for the invite |
| 09:03 | Copy the pre-downloaded files from `D:\DownloadHDD\penumbra-resource` into `data/` and check them against its `MANIFEST.tsv` (all 10 matched on Nov 12). Download again only if a check fails. | — |
| 09:05 | On `penumbra`: add `ambatuCode887` as collaborator; create the labels (owner:science, owner:app, owner:design, hard-science, ready-for-agent, in-progress); set the two Cloudflare secrets and the repo variable `CF_PAGES_PROJECT=penumbra-lunar`; point the Pages project at the new repo; turn on branch protection | Accept the invite; clone `penumbra`; bootstrap; the dev server shows the hello-world |
| 09:08 | A haiku or sonnet session files T00–T27 and D1–D7 as issues from this file, in order, with labels and "Blocked by" | — |
| 09:10 | **API freeze, together:** walk the §1 API table line by line. Change names, units or layouts now or not at all. | same |
| 09:25 | Merge `docs/API.md` and the fixture. Check: issues filed · fixture on main · hello-world deployed from `penumbra` and smoke test green · DEM in `data/` and readable | Check the fixture loads in the page |
| 09:30 | `/ticket` T01. Start T04 in a second session when the PSR file lands. | `/ticket` T03, on the fixture |
| 10:00 | Check: projection tests going green · T04 result in · kernels on disk for T02 at 10:30 | Check: the panorama canvas draws the fixture horizon and tracks locally · on course for the fixture page deployed at 11:00 |

If the public repo or the deploy is not working by 09:30, Charls starts T03 in his burner clone on a branch and moves the commits across when `penumbra` is ready. T01 can start the same way.

### Prep this plan needs before Nov 14 (tooling and reading only, all allowed by the pre-event rule)

1. ~~An export script.~~ Replaced Nov 14 by the same-folder orphan-branch steps in HANDOFF.md, with a corpus check before the push.
2. A repo-setup script or `/wizard` for the public repo: labels, secrets, Pages project, branch protection, collaborator.
3. An issue-filing script that turns §2 of this file into `gh issue create` calls.
4. ~~Confirm ANISE `LT_S` (U14).~~ Done Nov 12: `LT_S` exists, all three kernels load, CI builds it.
5. ~~Open every download URL.~~ Done: all files pre-downloaded Nov 11, checksums match `MANIFEST.tsv` (Nov 12). Old note: The PGDA paths are in `research/lunar-landing/facts-grill-1.md`. The NAIF kernel URLs are not listed there and must be found; only `moon_fk_de440.epa` has one.
6. The Typst skeleton with a figure slot per V-test, and the D3/D4 storyboard (already in SUBMISSION.md's prep list).
7. In the public tree, any doc that points at the corpus (the SCIENCE-SPEC ledger, `docs/EVIDENCE.md`) gets one line saying the corpus is held privately.
8. `owner-setup.sh` is done and deploys work (HANDOFF). 20 MB dummy deploy done Nov 12 (Q5). GDAL machine: Qi Cai's laptop via QGIS (ARCHITECTURE §6.3 Q4).

---

## 6. Open decisions for Nov 13

| # | Decision | Default in this plan | Recommendation |
|---|----------|----------------------|----------------|
| Q1 | Are the 16 hours build time before the freeze, or the total including submission? | Build time; Sun 15:00–21:00 is extra | Confirm. If 16 h is the total, take lever a now and expect no slack. |
| Q2 | Order of Qi Cai's Should work. §4's score order is band, IM-2, A1, V2. | IM-2 first (the video script needs it by Sun 13:00), then the band; A1 only in slack | Swap the band and A1. The hook needs an illumination check, and §0 names A1, not the band, as something that separates us from other teams. |
| Q3 | No shaded map and no 3D terrain by default | Accepted (gap 2). The panorama's time scrub shows the Sun sliding behind the skyline. | Accept, and decide on Nov 13 whether Charls's design leans on the 3D screen. If it does, the hours must come from T16 or D2. |
| Q4 | Demo story steps 4 (rank nine regions) and 6 (open API) | Out; the script rests on steps 1–3 and 5 | Rewrite the 4:30 script on Nov 13. |
| Q5 | Built data files (about 32 MB) are committed to the public repo | Yes | **Done Nov 12:** a 20 300 000-byte random `web/public/data/dummy-far.bin` was tracked by git, deployed (CI run 43243545081) and served 200 in full in 1.3 s. Pages sent it uncompressed (`application/octet-stream`, no `Content-Encoding` even for `br`), so plan on raw size; `DecompressionStream` stays the fallback. |
| Q6 | Where the videos live | Drive (local rule) and YouTube (KICKOFF checklist) | Do both. |
| Q7 | D5 is the 30-second video, not the 7-slide deck | Yes | Confirm with the local lead that the video alone is accepted. |
| Q8 | T00 is 0.5 h and now also holds the public-repo export and setup | 0.5 h, with the scripts from §5 prep written beforehand | Without those scripts T00 is about 1 h [est] and the Sat 12:00 deploy has no margin. |
| Q9 | Ticket numbering after filing | T00 → issue #1 and so on | If GitHub numbers differ, put the T-number in each issue title. |
| Q10 | T27 (A vs B compare) is overflow, but the challenge page asks for comparing sites and dates twice | **Changed Nov 14:** Should, after T14. Pay for its 1 h with T24 (heatmap) or D2 polish. | Qi Cai confirm which hour pays for it. Add QuickMap and Moon Trek deep links to T11 (minutes). |

---

## Summary

1. The tracer is T02: the IM-2 site on 2026-11-14, real Sun and Earth az/el from the hourly table, on the deployed page by Sat 12:00. T00 (hour 0) and T01 (geometry, no data) lead into it.
2. 28 build tickets (T00–T27) and 7 submission tickets (D1–D7). Sixteen are Must, five are Should on the schedule, seven are filed as overflow.
3. Critical path: T00 → T01 → T02 → T05 → T06 → T07 → T08 → T09 → T10 → T15 → T16 → D4. The science chain has no float until Sun 13:30.
4. Before the freeze each person has 16 h: Qi Cai has 1 h of slack, Charls 0.25 h.
5. D1–D7 are not in SCIENCE-SPEC §4's hours. They take most of Qi Cai's slack and all the room for 3D terrain, so slip cuts 1, 2 and 4 are already spent.
6. The hook's "illumination maps" claim needs V2, which is overflow. §4 has fallback wording.
7. Gaps fixed in the plan: kernels added to the hour-0 downloads; replay dates need their own ephemeris segments; the shaded map needs a function §4 never budgeted (T21).
8. The public `penumbra` repo must start from a fresh history, because the corpus is committed in the burner.
9. Nine decisions wait for Nov 13 (§6); Q1 and Q2 change the schedule.
10. Every clock time and every app or submission estimate is [est]; science hours come from §4.
