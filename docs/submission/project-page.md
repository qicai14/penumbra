# Project page fields: draft (D7, #35)

Draft text for each field on the Space Apps project page, to paste in after "Edit Details" opens (Sun Nov 15, 09:00). Sources: [IDEA](../IDEA.md), [SUBMISSION](../SUBMISSION.md), `web/public/scoreboard.json`, the About view's dataset table (`web/src/lib/About.svelte`) and `docs/paper/refs.yml`. Numbers are copied from those files, not rounded further.

**Character limits:** the repo does not record the form's limits. Each field below gives its current length; if the form shows a limit, trim to it while pasting.

---

## 1. High-level summary

_Length: about 1 150 characters._

> Penumbra is a survival planner for lunar south-pole landers, built for the "Lunar Landing Site Navigator" challenge in "The Next Frontier". Near the pole the Sun never climbs more than a few degrees above the horizon, so "the interaction of grazing sunlight and large local topography variations leads to extreme illumination conditions" (Gläser et al. 2018, doi 10.1016/j.pss.2017.07.006). The challenge says "Small changes in landing location or date can dramatically affect both power generation potential and communication windows." In our own model output, landing at IM-2's site 2 days early (2026-11-12 instead of 2026-11-14) takes operable time from 15.9 days to 0 h. IM-1 and IM-2 both landed near the pole and ended early.
>
> For any site and landing date, Penumbra tells a planner how many hours of sunlight and Earth contact the lander gets, and when the first shadow arrives that would drain its battery. It builds a 360° terrain horizon from NASA LOLA elevation maps and computes Sun and Earth positions from JPL's DE440 ephemeris. A battery model then turns those into an operable window and a time to first lethal shadow.
>
> One Rust model runs natively for validation and in the browser as WebAssembly. A public scoreboard checks it against outside judges: JPL Horizons (V1, pass), NASA's LOLA illumination map (V2, pass), published longest-night figures (V2b, reported as a fail), the IM-2 reconstruction (V4), and red-team edge cases (V5, 7 of 7).

## 2. Demo link

_D5: a 7-slide deck or the 30-second video (D3)._

> `<D3 Drive link, Viewer access>`

The 30-second video (D3, #31). Before submitting, set it to "Anyone with the link: Viewer" and open it logged-out. The video also goes on YouTube.

## 3. Final project link

> https://penumbra-lunar.pages.dev

Source code: https://github.com/qicai14/penumbra. If the form has one link only, use the app URL. Open it logged-out once before submitting.

## 4. Details: what it does, how it works, tools and languages

_Length: about 1 950 characters._

> **What it does.** Pick a south-pole site, a landing date and a mast height, or a curated site: IM-2, SP-01, Barker Site 1; Apollo and IM-1 sites are listed as out of coverage. Penumbra shows:
> - the operable hours (Sun and Earth contact at once);
> - the time to the first lethal shadow;
> - a landing window: operable days for the next 30 landing dates (#108);
> - the slope at the pin, from the 20 m DEM (#112);
> - a horizon panorama with the Sun and Earth tracks;
> - timelines with uncertainty bands on each sunrise and sunset;
> - a 3D terrain view;
> - a side-by-side comparison of two sites or dates;
> - a replay of IM-2's landing site and date.
> - First-visit tour: three coach marks introduce the app on a first visit.
>
> A region heatmap over Mons Mouton shows the nearest nearby spot that would have kept the lander running longer.
>
> **How it works.**
> 1. **Horizon:** ray-marching the LOLA terrain with lunar curvature, in three tiers (20 m out to 7 km, 80 m to 31 km, 240 m beyond).
> 2. **Sun and Earth:** topocentric positions in the Moon's mean-Earth frame from the NAIF DE440 kernels, including light time and aberration.
> 3. **Visibility:** the fraction of the Sun's disc above the horizon, kept apart from "any part visible", which is the metric NASA's maps use.
> 4. **Battery:** a state-of-charge model for a lander preset.
> 5. **Uncertainty:** sunset and sunrise bands from the LOLA height-error map.
>
> **Validation.** Every check runs against a source we did not build, and misses are reported, not tuned away. V3 (shadow comparison with LROC NAC images) was not run, because the images were not downloaded.
>
> **Tools and languages.**
> - Rust (core model, offline pipeline, WebAssembly via wasm-bindgen) and the ANISE crate for SPICE kernels.
> - TypeScript with Svelte 5 and Vite for the app, Three.js for 3D and uPlot for timelines.
> - Bun for scripts and tests, and Playwright for browser tests.
> - Typst for the project document.
> - Deployed on Cloudflare Pages.
>
> **Team and contributions.** Team Fractured Toes. Counts from the git history on `main`, as of 2026-11-15.
> - Lim Qi Cai (team lead; 107 commits, 49 merged PRs): idea and research, planning, repo scaffolding, CI and Cloudflare deploy, the science model and data pipeline (`crates/core`, `crates/pipeline`, WASM), validation (V1–V5), web app features, the D1 paper and submission docs.
> - Charls Pua Vui Hau (60 commits, 21 merged PRs): web app UI and tests (`web/src`, `web/tests`), design and v2 wireframes (`design/`), screenshots, the D2 poster and the D3/D4 videos.

## 5. NASA data and partner data (with citations)

_Every dataset the shipped app or its validation uses. From the About view's table and `docs/paper/refs.yml`._

| Data | Role in Penumbra | Citation |
|---|---|---|
| LOLA south-pole DEMs `LDEM_80S_20MPP_ADJ`, `LDEM_60S_240MPP_ADJ`, `LPSR_80S_20MPP_ADJ` (NASA GSFC PGDA) | Terrain for the horizon and shadow model; height-error map for the uncertainty bands; PSR mask for edge cases | Barker et al. 2023, *The Planetary Science Journal*, doi:10.3847/PSJ/acf3e1; PGDA product 90, doi:10.60903/gsfcpgda-lola-spole, https://pgda.gsfc.nasa.gov/products/90 |
| LOLA illumination maps `AVGVISIB_85S_060M_201608` and `_EARTH` (NASA GSFC PGDA) | Outside judge for V2 (visibility) | Mazarico et al. 2011, *Icarus*, https://ntrs.nasa.gov/citations/20120010094; PGDA product 69, https://pgda.gsfc.nasa.gov/products/69 |
| NAIF generic kernels: `de440.bsp`, `moon_pa_de440_200625.bpc`, `moon_fk_de440.epa`, `moon_de440_250416.tf` (NASA JPL) | Sun and Earth positions in the Moon's frame | Park et al. 2021, *The Astronomical Journal*, doi:10.3847/1538-3881/abd414; https://naif.jpl.nasa.gov/pub/naif/generic_kernels/ |
| JPL Horizons System and API (NASA JPL) | Outside judge for V1 (Sun and Earth azimuth and elevation) | https://ssd-api.jpl.nasa.gov/doc/horizons.html |
| LROC image of the IM-2 site (NASA / ASU) | IM-2 landing coordinates and crater, for V4 | https://lroc.im-ldi.com/images/1408 |
| Apollo landing-site coordinates (Apollo Lunar Surface Journal) | Curated site list | https://www.apollojournals.org/alsj/alsjcoords.html |
| Published south-pole illumination studies | Longest-night check (V2b) and model choices | Barker et al. 2021, *Planetary and Space Science*, https://ntrs.nasa.gov/citations/20205009660; Gläser et al. 2018, https://ntrs.nasa.gov/citations/20170007365; De Rosa et al. 2012, https://arxiv.org/abs/1208.5587 |
| IM-1 / IM-2 mission reports | Outside judge for V4 | IM-1 NDL paper, NASA NTRS, https://ntrs.nasa.gov/api/citations/20250007036/downloads/aasimndl.pdf; news reports listed in the app's About view |

Partner (non-NASA space agency) data: none used. The CSA Moon Exploration portal and Moon Trek / QuickMap are linked from the app as tools, not used as inputs.

## 6. References (every outside resource: code, fonts, images)

_The form has a checkbox confirming this list is complete. Tick it only after checking `web/package.json`, `Cargo.lock` and the D2/D3/D4 assets._

- **Data and papers:** everything in section 5.
- **Rust crates:**
  - ANISE 0.10.6 (https://github.com/nyx-space/anise)
  - wasm-bindgen 0.2.130
  - tiff 0.11.3
  - serde_json 1.0.154
- **JavaScript libraries:**
  - Svelte 5.57.3
  - Vite 8.3.7
  - Three.js 0.187.0
  - uPlot 1.6.32
  - Playwright (tests)
  - TypeScript
  - svelte-check
- **Tools:**
  - Bun
  - Typst (project document)
  - Cloudflare Pages (hosting)
  - Canva (poster, slides, video editing)
- **Fonts:** IBM Plex Mono and Archivo (Google Fonts).
- **Images:**
  - NASA Space Apps Challenge logo (poster): NASA Space Apps Challenge, https://www.spaceappschallenge.org
  - Lunar background on the poster: source to confirm.
  - Team photos: our own.
- **Use of AI:** no separate write-up (SUBMISSION D7); answer only if the form asks.

---

## Before submitting (from #35)
- [ ] Lock the page while editing, "Save Draft" often, then "Publish", then "Submit for Judging".
- [ ] Every Drive link is "Anyone with the link: Viewer" and opened once logged-out; videos also on YouTube.
- [ ] Both members are on the team page's Members tab.
- [ ] Submit by **Sun Nov 15, 22:00** (NSAKL26).
