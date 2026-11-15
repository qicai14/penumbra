# IDEA — Lunar Landing Site Navigator (project: **Penumbra**, team: **Fractured Toes**)

> **Hook:** For CLPS and Artemis landing planners, Penumbra shows how many hours of sunlight and Earth contact a lander gets at any south-pole site and landing date, and **when the first shadow arrives that would kill it**. It is computed from LOLA terrain and DE440 ephemerides, then checked against JPL Horizons and NASA's published LOLA illumination maps.

Source of the ambition pass: `/be-ambitious docs/IDEA-brief.md` (Nov 8). Research: NotebookLM `5d807093…` and `docs/research/lunar-landing/`.

## Problem
- At the south pole at the pole itself the Sun's centre stays within 1.54° of the horizon; at 85°S it reaches about 6.5° (SCIENCE-SPEC E104; the 6.5° is derived [D]: 5° + 1.54°). A 10 m hill casts a shadow hundreds of metres long, so moving the site 100 m or the date by 2 days can decide whether a mission survives.
- **IM-1 Odysseus** (Malapert A, Feb 22 2024) tipped over on a 12° slope in low sun and ran for about 6 days. **IM-2 Athena** (near Mons Mouton, Mar 6 2025) landed on its side in a small crater, where a low Sun angle and the panels' orientation stopped charging (SCIENCE-SPEC E73, E74), and lasted about 1 day (unconfirmed, SCIENCE-SPEC U19).
- Existing tools (Moon Trek, QuickMap, Horizons) each answer one layer. None answers the planner's real question for a site and date: does the lander get enough power and contact to survive the mission? Published site rankings combine topography, illumination and mineralogy (SCIENCE-SPEC E140) but not a chosen date and lander.

## What the challenge page stresses (Resources and Details tabs, read Nov 14)
- **Resources tab:** QuickMap (ASU/LROC), Moon Trek (NASA, 3D, "how terrain affects horizon positions"), JPL Horizons, and the CSA Moon Exploration portal. No dataset beyond what MANIFEST.tsv already holds.
- **Details tab, repeated themes:**
  - fast side-by-side comparison of sites **and** dates, for when a schedule slip forces a re-plan
  - a curated site list (Apollo, CLPS, other commercial and other countries' sites) plus custom coordinates
  - time left before sunset and Earthset
  - libration
  - planners, educators and the public
- **What we do about it:**
  - The compare view (PLAN-DRAFT T27) is promoted from overflow to Should. It gets its own wireframe (screen 6 in design/DECISIONS.md).
  - Non-polar curated sites (Apollo, etc.) are listed but marked `OUT_OF_COVERAGE`, because our DEM stops at 80°S. They are not silently dropped.
  - Each site gets "open in QuickMap" and "open in Moon Trek" links. They cost minutes and tie us to the listed tools.

## Users
- **Primary:** CLPS/Artemis landing and ops planners. Concrete case: replay IM-2's site and date, then find the nearest site within 1 km, or the nearest landing date, that would have given it more than 5 days of operations.
- **Secondary:** students and the public, who see why the lunar pole is unlike anywhere on Earth.

## Science model (what is actually computed)
1. **Horizon:** a 360° terrain horizon for any point, at a given panel or mast height above the ground. Built by ray-marching the LOLA DEM with lunar curvature: 20 m `LDEM_80S_20MPP_ADJ` out to at least 7 km, then 240 m `LDEM_60S_240MPP_ADJ` to at least 200 km, marched along great circles. Mast height matters a lot: Gläser et al. 2018 report average illumination of 69.5–82.9 % at the surface and 77.1–88.0 % at 2 m (SCIENCE-SPEC E70a, E70b). There is no atmosphere, so the horizon is terrain or the lunar limb: no refraction, no twilight.
2. **Ephemeris:** Sun and Earth positions in the MOON_ME frame from the NAIF SPICE kernels (DE440, moon_pa, moon frame kernels), read in Rust via the [ANISE](https://crates.io/crates/anise) crate (v0.10.6). Positions are made topocentric for each site, because Earth parallax across the Moon is about 0.26°. Libration comes in through the kernels. Positions are apparent (light time and stellar aberration), with UTC converted to TDB by the library.
3. **Visibility:** the fraction of the Sun's disk above the horizon (the Sun is a 0.53° disk, not a point), and Earth's elevation above the terrain mask with a margin.
   - Two metrics are kept apart: illumination (disc fraction, for power) and visibility (any part of the disc, for comparison with PGDA).
4. **Survival:** a battery state-of-charge simulation for a lander preset (panel height and orientation, battery Wh, load W, minimum direct-to-Earth hours).
   - Outputs: the **operable window** (Sun AND Earth), the longest continuous darkness, the time until the first lethal shadow, and the mission margin.
5. **Uncertainty:** sunset-time bands from the LOLA height-error raster, propagated through the horizon (after Barker 2021), giving bands on every window boundary ("sunset at 14:20 ± 40 min"). Monte Carlo is Stretch.

## Validation (outside judges — the scoreboard on our About page)
| # | Judge | Pass |
|---|-------|------|
| V1 | JPL Horizons API: Sun and Earth az/el at 20 random site/time pairs | ≤ 0.01° (aim ≤ 0.002°) |
| V2 | PGDA `AVGVISIB_85S_060M_201608` (Sun and Earth), visibility metric, 0 m, hourly over 18.6 years, 10 000 stratified pixels | pooled median ≤ 5 points, |mean signed error| ≤ 2, r ≥ 0.9 (stretch: median ≤ 2, p90 ≤ 8). Plus the longest-night check against Barker 2021 Site 1 and Mazarico SP-01. |
| V3 | PGDA map-projected NACs | our shadow IoU within 0.05 of NASA's `_MODEL.TIF` IoU on 3 images, and ≥ 0.75 (Stretch) |
| V4 | IM-1 / IM-2 reconstruction | consistent with the published timeline (no Sun elevation has been published for either) |
| V5 | Red-team edge cases: the pole itself, extreme libration, PSR crater floors (sun must be 0), the year boundary | all pass as automated tests |

Report misses honestly, with the numbers.

## Data plan (runtime ≤ 100 MB, static-first)
- DEM:
  - one 10–20 m near-field tile per Artemis region (about 4–5 MB each)
  - a 240 m far field cropped from `LDEM_60S_240MPP_ADJ` to site + 200 km (about 20 MB raw for ±382 km)
  - tiles are int16 with a per-tile offset at 0.25 m or finer
  - the horizon for a clicked point is computed **in the browser** in Rust→WASM, target < 50 ms
- Precomputed natively in Rust offline:
  - Sun and Earth MOON_ME vectors hourly, f64, for 2026–2030 (about 2.1 MB); interpolated to 10 min in the browser
  - region-wide horizon grids for the site-ranking heatmaps
- **Must scope after the PLAN-DRAFT §4 cuts (Nov 11): two regions (Mons Mouton, Shackleton ridge), no 3D terrain, no ranking heatmaps.** The rest of this list is the Stretch vision.
- Coverage: the 9 Artemis III regions (Peak near Cabeus B, Haworth, Malapert Massif, Mons Mouton Plateau, Mons Mouton, Nobile Rim 1 and 2, de Gerlache Rim 2, Slater Plain), plus the IM-1 and IM-2 sites.

## Scope tiers (better, not bigger)
- **Must:** pick a point and date; panorama with Sun and Earth tracks; a large live readout of Sun and Earth elevation with a countdown to the next sunset and Earthset; illumination and Earth timelines; the survival headline (operable days, time to first lethal shadow) for one preset; V1 and V5 passing; deployed. The survival headline moves up from Should: it is the hook and costs 1.5 hours (SCIENCE-SPEC).
- **Should:** IM-2 replay with the pit model; two site-date pairs side by side (A vs B); V2 sample; ± band on sunset times; A1 check; 3D terrain shaded by our model; heatmap for one region.
- **Stretch:** heatmaps for all nine regions; V3 relative IoU; Monte Carlo; IM-1 replay; share links; JSON API.
- **Cut:** SAM 3.1; DSN complexes; the 10 m DEM; boulder hazards; chatbot.

### If "deployed by Sat 12:00" slips, cut in this order
1. 3D terrain with live shadows → a 2D map coloured by the same model output.
2. Nine regions → two: Mons Mouton (IM-2) and de Gerlache Rim 2 / the Shackleton ridge (SP-01, Site 1, and the V2 area). **Undone by #26 (Nov 15):** all nine 2024 regions ship, centred on NASA's outlines (W16). The IM-2 tile lies in Mons Mouton Plateau, not Mons Mouton, and the Shackleton ridge tile lies in none of the nine, so neither stood in for the region named here.
3. In-browser horizon in WASM → horizons precomputed natively on a 100 m grid for those two regions (720 int16 per point, about 14 MB per 10 × 10 km). The panorama and timelines still work; only free clicking is lost.
4. V2 sample → the A1 table check alone.

## Demo story (2 min)
1. IM-2 lying on its side in shadow. It lasted 1 day.
2. The same site and date in Penumbra: the Sun at about 1° and the shadow edge sweeping in. Regional terrain alone says the site had sunlight. Add a 20 m crater: Penumbra shows how deep it must be to shade a panel at height H, and whether a typical degraded crater is that deep.
3. Drag the pin 300 m, or shift the date by 2 days, and see what the model returns.
4. Rank all 9 Artemis regions for a lander preset.
5. The validation scoreboard: V1–V5 with real numbers.
6. The open API, so the next lander can check before it flies.

## Team split
- **A, science core:** ephemeris pipeline, horizon engine, survival simulation, validation V1–V5.
- **B, app:** 3D terrain and shadows, panorama, planner UI, deploy, demo video, project page.
- Shared: the hook, the IM-2 story, limitations.

## Build roles → subagents (Range crew; each does the field's method, not its vocabulary)
| Role | Job | Method / tool |
|------|-----|---------------|
| Geodesist | DEM ingest, curvature, error propagation, V2 | LOLA PGDA GeoTIFFs, first-order error propagation |
| Astrodynamicist | frames, ephemeris, V1 | ANISE + SPICE kernels, Horizons API diffs |
| Power + RF engineer | survival simulation, operable window | battery state-of-charge time-step simulation, terrain mask + elevation margin |
| Accident investigator | IM-1 / IM-2 reconstruction, V4 | mission reports → timeline → our model replay |
| Red-team reviewer | breaks the model, V5 | adversarial test cases as automated tests |
| Data journalist / cinematographer | demo, scroll story, one-number headline | real-scale shadow sweep, panorama shown at the height of a person or lander mast |

## Decisions (from `/grilling`, Nov 8, confirmed)
- Planner-first; public via IM-2 replay mode. Headline = **operable days + time to first lethal shadow**; average % is secondary.
- Scope: the 9 Artemis III regions + the IM-1/IM-2 sites. Dates 2026–2030 (hourly table, 10-min timelines), plus the replay dates.
- Lander = generic parameters with presets; any value we can't cite is labelled "assumed". Earth contact = Earth above the terrain mask + 2° (De Rosa 2012).
- Must order: panorama + timeline first, then simple 3D. 3D shadows come from our model, not GPU shadow maps.
- V2 = PGDA `AVGVISIB_85S_060M_201608` (Sun and Earth), visibility metric, 0 m, hourly over 18.6 years, 10 000 stratified pixels; pass pooled median ≤ 5 points, |mean signed error| ≤ 2, r ≥ 0.9 (stretch: median ≤ 2, p90 ≤ 8). Plus the longest-night check against Barker 2021 Site 1 and Mazarico SP-01.
- V3 = PGDA map-projected NACs; our shadow IoU within 0.05 of NASA's `_MODEL.TIF` IoU on 3 images, and ≥ 0.75 (Stretch).
- DEM: `LDEM_80S_20MPP_ADJ` only (20 m, 2.6 GB), cropped to region tiles, plus `LDEM_60S_240MPP_ADJ` for the 240 m far field. The 10 m map is used only if there's time.
- Nov 10: science review (docs/SCIENCE-SPEC.md) applied; SAM 3.1 and DSN dropped; 3D terrain moved to Should.
- **Rules:** the FAQ forbids working on the challenge before the event. Before Nov 14: research, planning docs, tooling, and a generic hello-world template only. No project code and no data processing. Email the local lead to confirm. Judging is local first, then nomination to global.
- Team **Fractured Toes**: Qi Cai = owner and science core; Charls (`ambatuCode887`) = app and design. Project name: Penumbra. Only the deploy URL is `penumbra-lunar.pages.dev`.
- No LLM evals: V1–V5 are the evals (no runtime LLM).
- Stack → `docs/STACK.md`.

## Open questions (resolved — kept for the record)
- How do Space Apps rules treat data preprocessing and spikes done before Nov 14?
- Does ANISE build to `wasm32`, or do we ship precomputed vectors only? (Precomputed only is fine.)
- Exact Barker 2023 reference values and where to download the PGDA files.
- Which LROC NAC images have usable shadows and timestamps for V3.
- Lander presets: what can we cite for IM-2's power system?
- Open facts: docs/SCIENCE-SPEC.md §5 (U1–U19)

## Missing access / setup
- Data downloads (PGDA, PDS, NAIF, LROC) are public and need no auth. Sizes: 20 m DEM 2.6 GB, 240 m far-field DEM cropped per site.
