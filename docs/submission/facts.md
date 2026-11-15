# Penumbra facts

The single source for every number in the README, the D1 paper, the D7 project page and the video script. Copy from here; if a number changes, change it here first. Each row names its source: a repo file and field, an evidence id (`docs/EVIDENCE.md` C-ids, `docs/SCIENCE-SPEC.md` E/W-ids), a `docs/paper/refs.yml` key, or a public URL. Measured rows give the command and the date. Written 2026-11-15.

## 1. Pitch

Penumbra tells a lunar landing planner how many hours of sunlight and Earth contact a lander gets at a south-pole site and date, and when the first lethal shadow arrives. One Rust model runs natively for validation and as WASM in the browser, checked against JPL Horizons and NASA LOLA illumination maps.

It answers three questions for a site, a landing date and a mast height:

| Question | What the app returns | Source |
|---|---|---|
| How many hours of sunlight? | Hours with any part of the Sun's disc above the real terrain horizon, from the 20 m LOLA DEM out to 7 km, 80 m to 31 km, 240 m beyond | `docs/DATA-NOTES.md` "Shipped tiles"; three-tier horizon in `crates/core/src/horizon.rs` (#48) |
| How many hours of Earth contact? | Hours with Earth above the horizon by a 2° margin (assumed: a preset value, taken from one paper's comms margin, not a mission requirement) | `web/src/lib/engine.ts` preset `earth_margin_deg: 2`; SCIENCE-SPEC E62 |
| When does the first lethal shadow arrive? | The first moment stored battery energy falls to the preset's minimum (survival model) | `docs/GLOSSARY.md` "Lethal shadow"; `crates/core/src/power.rs` (#10, T09) |

## 2. Penumbra beside the tools planners already use

Cells: yes, partial, no, or "not found in docs" where the tool's public docs were searched and nothing turned up. Footnotes are the tools' public pages. Lookups on 2026-11-15.

| Capability | Penumbra | NASA Moon Trek | LROC QuickMap | JPL Horizons |
|---|---|---|---|---|
| Sunlight hours at a site and date, terrain-shadowed | yes (§1) | partial: a "Surface Lighting" tool, no per-site sunlit-hours total [T1][T3] | partial: Terrain Shadows over a UTC range, no sunlit-hours total [Q1] | partial, no terrain: Sun rise and set from a site, against a horizon that allows dip "but not local topography" [H1] |
| Earth contact over time | yes (§1) | not found in docs (a Line of Sight tool exists) [T3] | partial: SUN+EARTH and EARTH shadow sources, no contact-hours output [Q1] | partial: rise and set of target bodies from a site [H1] |
| First lethal shadow | yes (#10) | not found in docs | not found in docs | not found in docs |
| 30-date landing window | yes, operable days for the next 30 landing dates (#108) | not found in docs | partial: start, end and delta-hours inputs, no sweep summary [Q1][Q3] | partial: raw start/stop/step tables only [H2] |
| Slope at a point | yes, from the 20 m DEM (#112) | partial: Slope tool, registered account needed [T1][T3] | partial: elevation at a point and along a path; slope not found in docs [Q2] | not offered (ephemeris service) |
| Validated against Horizons and LOLA | yes: V1 and V2 in §3 | not found in docs [T2] | not found in docs | it is the reference: Horizons serves DE441 [H1] |
| Runs with no backend | yes: static files only (§3) | no (server-based) [T2] | no (server-based) [Q1] | no (server-based) [H1] |
| Cost | $0, no account (§3) | free to browse; lighting and slope need a registered account [T1] | price not found in docs; "Contact ACT" for Terrain Shadows [Q1] | free; "No account or password is required." [H1] |

Where the other tools are better:

- **Moon Trek** covers the whole Moon in one portal, with lighting, slope, Sun-angle and line-of-sight tools [T1][T3]. Penumbra covers ten south-pole regions.
- **QuickMap** carries the full LROC data catalogue, a Sun+Earth terrain-shadow tool over any time range, and CSV export [Q2]. Penumbra ships only the layers its model needs.
- **Horizons** is the ephemeris authority (DE441), free, scriptable through an API [H1][H2]. Penumbra's V1 test uses Horizons as its judge.

SUNLOCK (https://github.com/nasa/spaceapps/discussions/2170) and Lunar Window (https://github.com/nasa/spaceapps/discussions/2092) also address this challenge; we did not compare features.

Footnotes:

- [T1] https://trek.nasa.gov/moon/ — tools list ("Calculate Sun Angle", "Surface Lighting"); "Advanced tools … (e.g., lighting, slope analysis) require a registered individual or group account."
- [T2] Moon Trek paper, https://moontrek.jpl.nasa.gov — a browser portal on "back end server infrastructure"; slope for hazard assessment; no validation against ephemerides or LOLA found.
- [T3] https://meetingorganizer.copernicus.org/EGU23/EGU23-969.html (EGU23 abstract on Moon Trek, re-read 2026-11-15) — "lighting analysis, electrostatic surface potential analysis, slope analysis"; "line of sight analysis (facilitating communications planning and detailed studies of solar illumination)".
- [Q1] https://qts.quickmap.io/fcgi-bin/fprovweb.exe?_xtype=text%2Fhtml&cmd_script=qts_js.msh — TerrainShadows with SUN, SUN+EARTH, EARTH sources and START/END UTC, DELTA TIME HOURS; Traverse Mode is a server request.
- [Q2] https://docs.quickmap.io/toolbar/2-query — elevation along a path, CSV export. QuickMap lives at https://quickmap.lroc.im-ldi.com.
- [Q3] https://docs.quickmap.io/map/0-time — the time panel applies to time-dependent layers only.
- [H1] https://ssd.jpl.nasa.gov/horizons/manual.html (re-read 2026-11-15) — TVH "Allows for horizon dip effect and atmospheric refraction, but not local topography."; rise and set "are always with respect to the true-visual-horizon reference plane (TVH)"; interpolates "previously defined ephemerides, such as DE441"; "No account or password is required."
- [H2] https://ssd-api.jpl.nasa.gov/doc/horizons.html — START_TIME, STOP_TIME, STEP_SIZE.

## 3. Metrics

### Horizon computation time (measured 2026-11-15)

One horizon is 720 azimuths marched over the near, middle and far tiles. Two sites: IM-2 at a 2 m mast (high horizon, early exit) and SP-01 at a 10 m mast (low horizon, marches about 190 km). Machine: Windows 11 Pro desktop, AMD Zen 2, 8 logical CPUs, 16 GB RAM (`docs/DATA-NOTES.md` T07). Target 50 ms (ARCHITECTURE §2.2).

| Where | Command | IM-2, H = 2 m | SP-01, H = 10 m |
|---|---|---|---|
| Native, release | `cargo build --release -p penumbra-pipeline && target/release/penumbra-pipeline horizon web/public/data` (10:41 UTC; 1 warm-up + 20 calls) | median 18.94 ms, max 23.87 ms | median 41.91 ms, max 51.02 ms |
| WASM in Bun 1.4.4 | `bun run wasm`, then a bench script (not in the repo yet) that loads far, near and mid from `manifest.json` and calls `engine.horizon()` (10:42 UTC; 1 warm-up, 3 runs of 20 calls, then 20 single timed calls) | 12.47–12.77 ms per call across runs; single-call median 12.55 ms, max 14.97 ms | 26.67–33.81 ms per call; single-call median 28.51 ms, max 36.61 ms |
| Browser, live app | In https://penumbra-lunar.pages.dev (Chromium 156 in a fresh browser profile): import the deployed `/assets/penumbra_wasm-08dzdBv0.js`, load the shipped tiles, `performance.now()` around `engine.horizon()`, same method as the Bun row | 18.52, 18.88, 18.54 ms per call; single-call median 18.15 ms, max 18.60 ms | 35.08, 34.88, 34.99 ms per call; single-call median 34.25 ms, max 35.50 ms |

**Headline numbers (quote these):** native release, median **17.20 ms** at IM-2 and **33.66 ms** at SP-01, from the quieter U18 run in `docs/DATA-NOTES.md` (Nov 15, same machine and method). The three rows above are today's re-runs on a shared, busy machine; they are load-dependent, and 5–10 ms gaps between them are run-to-run noise. Every median in every run is under the 50 ms target; the single worst call today, native SP-01 at 51.02 ms, was 1 ms over. The bench script is not in the repo yet (see Future work).

### Validation (from `web/public/scoreboard.json`, read 2026-11-15)

| Test | Field values | Plain statement |
|---|---|---|
| V1, JPL Horizons | `id: "V1"`, `value: 0.00011`, `threshold: 0.002`, `pass: true`, `sample_size: 20`, `commit: "5ff00a1"` | Worst Sun or Earth azimuth/elevation difference over 20 site-time pairs, 2026–2030: 0.00011° (Earth); the Sun is 0.000013°. That is 18 times inside the 0.002° threshold. |
| V2, LOLA AVGVISIB | `id: "V2"`, `value: 0.48`, `threshold: 5`, `pass: true`, `sample_size: 6123`, `commit: "909a833"` | Pooled median visibility error 0.48 points against NASA's 18.6-year average-visibility map, over 6 123 pixels; mean signed error +0.72, Pearson r 0.955 (`notes`). Earth twin: median 0.18, r 0.990. |
| V2b, published sites | `id: "V2b"`, `value: 7`, `threshold: 6`, `pass: false`, `sample_size: 7` | Reported as a fail. `value: 7` counts Barker 2021 Site 1 regions that pass: 7 of 7, above the threshold of 6. The fail comes from the second check, the SP-01 range check: Mazarico 2011's 2.75-day longest night is 0.21 days below our shortest pixel (2.96 days). That gap is inside Mazarico's 6-hour step (about ±0.25 days), but the agreed rule is the strict range, so `pass` is false (`notes`; refs `barker2021`, `mazarico2011`). |
| V3, LROC NAC images | `id: "V3"`, `value: null`, `pass: null`, `notes: "not run"` | Not run. See Future work. |
| V4, IM-2 replay | `id: "V4"`, `pass: true` | A reconstruction, not a validation: at the assumed touchdown the Sun stands 2.46° up, and a 1.94 m-deep pit hides it from panels at 0–1.44 m (`notes`). |
| V5, red team | `id: "V5"`, `value: 7`, `threshold: 7`, `pass: true`, `sample_size: 7` | All 7 edge cases pass (pole singularity, flat-pole bounds, 20 permanently shadowed sites that stay dark for 18.6 years). |

Per-bin V2 correlation is below 0.9 in every fifth (0.62 to 0.89); the pooled pass rests on the 0–60 % bins (#75, `notes`). Say "pooled" when quoting r.

### First load (measured 2026-11-15)

Cold load of https://penumbra-lunar.pages.dev/?cold=1 in Chromium 156 with a fresh browser profile, Resource Timing summed over the navigation and 21 own-origin resources after 8 s idle:

| Measure | Value |
|---|---|
| Transferred (bytes on the wire, headers included) | 34 269 727 B = **34.3 MB** |
| Encoded body | 34 263 909 B |
| Decoded | 34 966 301 B |
| Largest file | `far-240m.ebf340.bin`, 21 780 000 B |
| Load event | 740 ms (data files keep arriving until about 2.9 s) |

The `.bin` tiles travel uncompressed (encoded = decoded), so they are 98 % of the bytes. The JS bundle is 225 KB on the wire (780 KB decoded), WASM 52 KB (136 KB decoded). Choosing another region adds its near and mid tiles (6.7 MB, `docs/DATA-NOTES.md`).

### Hosting and runtime calls

| Claim | Value | Source |
|---|---|---|
| Hosting cost | **$0/month**: Cloudflare Pages, static assets only | "On both free and paid plans, requests to static assets are free and unlimited." https://developers.cloudflare.com/pages/functions/pricing/ |
| Servers we run | **0** | `docs/STACK.md` Backend row: "none at runtime" |
| Runtime API calls | **0**: no API, no key, no database | network log of the cold load above: every data and code request is a GET to `penumbra-lunar.pages.dev` |
| Third-party requests | **1 host family**: Google Fonts (`fonts.googleapis.com` CSS, font files from `fonts.gstatic.com`) | measured on the cold load; the three font links in `web/index.html`. See Future work. |

So "0 runtime API calls" holds; "only its own origin" does not yet, because of the fonts.

## 4. Stack and why not the default

| Choice | Instead of | Why | Source |
|---|---|---|---|
| One Rust model, native and WASM | a Python or TS model plus a browser port | The scoreboard numbers and the planner's numbers come from the same code, so they cannot drift | `docs/adr/0001-one-rust-model-native-and-wasm.md`; `docs/STACK.md` |
| ANISE (native only) + NAIF kernels | CSPICE, SpiceyPy | Pure-Rust SPICE reader with MOON_ME/MOON_PA frames; the kernels are 133 MB, so vectors ship as an hourly table of about 2.1 MB | `docs/STACK.md`; `docs/adr/0002-ephemeris-table-on-a-utc-grid.md`; ref `anise` |
| Svelte 5 + Vite | React / Next.js | Compiled, small bundle; one heavy canvas plus panels needs no React and no SSR | `docs/STACK.md` |
| Plain Three.js, colours from the model | Threlte, GPU shadow maps | Shadow maps are unreliable at 1° Sun elevation; the picture must equal the validated numbers | `docs/STACK.md`; `docs/adr/0006-illumination-from-the-model-not-shadow-maps.md` |
| uPlot | Chart.js, Plotly, ECharts | 260 k points per series stays smooth at about 50 KB | `docs/STACK.md` |
| Bun + TypeScript | Python, npm | One non-Rust language; Bun runs TS directly | `docs/STACK.md`; AGENTS.md |
| Typst for the D1 paper | Word, Canva | Real equations, numbers pulled from repo files, plain text that diffs in PRs | `docs/SUBMISSION.md` "Typst for D1" |
| Cloudflare Pages | Vercel, Netlify | Free, global, static; nothing to go down after judging | `docs/STACK.md` Hosting and Backend rows |

## 5. The problem

- **IM-1 tipped over.** Intuitive Machines' Odysseus landed on 2024-02-22 (E72b) at 80.1276° S, 1.4367° E (E72a), near Malapert A [unverified in EVIDENCE], on a 12° slope and came to rest about 30° off the surface; it ran 144 hours and ended "as the sun moved out of view of the one illuminated solar panel" (SCIENCE-SPEC E75a–c). NASA LRO imaged it on 2024-02-24 [unverified in EVIDENCE]: https://www.nasa.gov/missions/lro/nasas-lro-images-intuitive-machines-odysseus-lander/
- **IM-2 ended on its side in shadow.** Athena reached Mons Mouton on 2025-03-06 at 84.7906° S, 29.1957° E (SCIENCE-SPEC W9). It "ended up on its side within a 20-meter diameter crater" (LROC, https://lroc.im-ldi.com/images/1408, W6). NASA got "some data" before its batteries were depleted (C2, https://www.nasa.gov/news-release/nasa-receives-some-data-before-intuitive-machines-ends-lunar-mission/). A low Sun angle and the panels' orientation stopped charging (E73, E74).
- **Artemis is going to the south pole.** NASA cut its Artemis III candidate list from thirteen to nine south-pole regions (https://ntrs.nasa.gov/citations/20250011660, "Downselection From Thirteen to Nine Candidate Regions"; the nine are listed in SCIENCE-SPEC E116; outlines W16, NASA release of 28 Oct 2024). The first crewed south-pole landing is now Artemis IV in 2028 (C4). Penumbra ships all nine regions plus the Shackleton ridge (#26).
- **More landers are coming.** Blue Moon MK1 lands on the Shackleton connecting ridge in 2027 (C5); NASA lists south-pole deliveries for Intuitive Machines, Firefly and Astrobotic (C6).

## 6. Commercialization

Condensed from `docs/submission/commercialization.md`; every figure is a C-id in `docs/EVIDENCE.md`.

- **What failure costs.** CLPS task orders (as of Feb 2024): IM-1 $122.0 M, IM-2 $50.0 M, Astrobotic Griffin $322.8 M; the VIPER rover was $505.4 M (C1). CLPS is capped at $2.6 B through November 2028 (C3).
- **Buyers.** CLPS lander vendors choosing sites and dates; NASA and partner landing-site teams for the crewed south-pole landing; rover and ISRU teams sizing power and comms at a stop; universities teaching mission design.
- **Model.** Open core: the app and the Rust model stay free and Apache-2.0, so anyone can rerun V1 and V2. Paid: custom sites and dates, finer DEMs and a signed sign-off report. Price anchor: **our own illustrative number, not a sourced figure**: $25 000 per site-and-date study, which is 0.05 % of a $50 M task order (C1). Never quote it as a market price.
- **Why now.** Artemis IV in 2028 (C4), Blue Moon MK1 in 2027 (C5), four more listed south-pole deliveries (C6), China's Chang'e-7 planned for the second half of 2026 (C7).

## 7. Limitations, written as Future work

| Limitation | Future work, with its path |
|---|---|
| Runtime data uses 94.2 of the 100 MB budget | #104: ten regions at 7.02 MB each plus the 21.78 MB far tile leave 5.8 MB. Before an eleventh region, state the rule in ARCHITECTURE §2.1: raise the budget, drop the mid tile where #48 allows, or shrink tiles. |
| The landing-window heatmap has only 3 grid dates (2025-03-06, 2026-05-31, 2026-11-14) | `web/public/data/manifest.json` `grids[].dates`; each grid costs 0.33 MB and about 3 minutes to build (#103 notes). Add dates with `penumbra-pipeline grid <data_dir> <region>` inside the #104 budget. The per-site 30-date window (#108) is computed live for any date. |
| V3 (LROC NAC shadow check) not run | `scoreboard.json` V3 `notes: "not run"`. Steps in SCIENCE-SPEC "V3 details": read START_TIME from each NAC's PDS label, threshold the NAC and NASA's `_MODEL.TIF` with Otsu, compare IoU; pass if within 0.05 of NASA's own model. |
| Sites work only inside the ten shipped tiles | #103: a design note sizing on-demand tiles, a tile server, or a coarser any-site mode with its error stated. Building a tile takes seconds; shipping it is the limit. |
| First load is 34.3 MB (measured, §3) | The `.bin` tiles are served uncompressed (encoded = decoded on the cold load). Check Brotli on `.bin` at Cloudflare (STACK "compression unverified"), and lazy-load the 21.78 MB far tile and the ten grids until a site is picked. |
| Region outlines come from QuickMap's file host | #106: compare the USGS grids (doi 10.5066/P1MEQ6UK) with W16 and record the result as a ledger row. |
| Fonts load from Google, so the app is not single-origin | Self-host IBM Plex Mono and Archivo as woff2 under `web/public/` and drop the three font links in `web/index.html`. |
| The WASM timing bench is not in the repo | Commit it as `scripts/bench-horizon.ts` (method in §3) so the WASM row can be re-run with one `bun` command. |

## 8. NASA data used

| Data | Used for | Citation |
|---|---|---|
| LOLA south-pole DEMs `LDEM_80S_20MPP_ADJ` (20 m), `LDEM_60S_240MPP_ADJ` (240 m), PSR mask `LPSR_80S_20MPP_ADJ` | Terrain horizon, slope, V5 shadowed sites | NASA GSFC PGDA, doi 10.60903/gsfcpgda-lola-spole, https://pgda.gsfc.nasa.gov/products/90 (ref `pgda-lola-spole`); Barker et al. 2023 (ref `barker2023`) |
| PGDA illumination maps `AVGVISIB_85S_060M_201608` and `_EARTH` | V2 judge | https://pgda.gsfc.nasa.gov/products/69 (ref `pgda-illumination`); 18.6 years hourly (W7) |
| NAIF kernels `de440.bsp`, `moon_pa_de440_200625.bpc`, `moon_fk_de440.epa`, `moon_de440_250416.tf` | Sun and Earth vectors in the Moon frame, via ANISE | https://naif.jpl.nasa.gov/pub/naif/generic_kernels/ (ref `naif-kernels`); DE440, Park et al. 2021 (ref `park2021`) |
| JPL Horizons API | V1 judge | https://ssd-api.jpl.nasa.gov/doc/horizons.html (ref `horizons`) |
| LROC | IM-2 landing site and crater (W6, W9); Artemis III region outlines via the QuickMap layer (W16) | https://lroc.im-ldi.com/images/1408; https://quickmap.lroc.im-ldi.com |
| Published PGDA-based studies | V2b reference values | Barker et al. 2021 (ref `barker2021`), Mazarico et al. 2011 (ref `mazarico2011`) |
