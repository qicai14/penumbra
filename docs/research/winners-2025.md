# NASA Space Apps 2025 Global Winners: analysis

Sources: NASA announcement (2025-12-18), each team's spaceappschallenge.org project page (read via browser pane; the pages only render when reached by clicking through from the nasa.gov article, direct loads redirect to the 2026 site), live-site checks (HTTP + title, 2026-11-08), GitHub inspection via `gh`. 114,000+ participants, 551 events, 11,500+ submissions, 10 winners.
Limits: sites were only fetched (HTTP status + title), not visually walked through. Demo videos were not watched (most are private Google Drive links). Repo skims are shallow.

## Summary table

| Award | Project (team, country) | Challenge | NASA data | Stack | Live? | Repo |
|---|---|---|---|---|---|---|
| Best Use of Science | SpaceGenes+ (Germany, solo: Saloni T.) | Build a Space Biology Knowledge Engine | GeneLab OSD/GLDS-288 (mouse spleen RNA-seq) | Framer, Bolt (no-code dashboard), Python/Pandas/NumPy, Figma | Yes (Framer site, 442 KB) | None found |
| Best Use of Data | Resonant Exoplanets (US) | A World Away: Hunting Exoplanets with AI | Kepler, TESS, JWST + Hubble via MAST | FastAPI, XGBoost, Lightkurve, Astroquery, React/TS, ReportLab, custom "Modulus" solver on Qwen | Yes (Vercel SPA) | sahilssan/resonant-worlds-explorer (0 stars, 875 KB, Python/TS) |
| Best Use of Technology | Twisters / SkySense (Mexico) | Will It Rain On My Parade? | NASA POWER API, Earthdata, Giovanni/GES DISC | React, Firebase, Google APIs + calendar, Gemini chatbot, Open-Meteo, OSM, Figma | **No**, skysense.surf is now a Porkbun "domain for sale" page | None found |
| Galactic Impact | Astro Sweepers (Universal Event) | Commercializing LEO | NASA SPARK 2022, DAS, ORDEM 3.2, LEGEND, ORSAT, NASA-STD-8719.14 | Vercel web portal; Python scripts for DCRA/ASRI scoring, DAS automation; small CNN | Yes (Vercel, debris tracking dashboard) | No team repo; lead's later "Navigator" repo only |
| Best Mission Concept | PureFlow (Brazil) | Your Home in Space: Habitat Layout Creator | DONKI space-weather API; NASA inflatable-habitat review paper | Vanilla HTML/JS + 3D; ElevenLabs audio | Yes (GitHub Pages) | larasdiniz/PureFlow-2025 (HTML only, 15 MB, 2 stars) |
| Most Inspirational | Photonics Odyssey / AakashNet (India) | Commercializing LEO | NASA Image & Video Library only; ISRO ISTRAC | HTML/Tailwind/JS mockup; MATLAB + OpenEMS RF/link-budget | Drive folder + Canva deck only (no live app) | None |
| Best Storytelling | HerCode Space, "A Solar Tale" (Universal Event) | Stellar Stories: Space Weather | NASA/NOAA SWPC space-weather pages | React, SCSS, react-scrollama, Netlify; AI art (Nano Banana, Pixaverse), Narakeet | Yes (Netlify) | None found |
| Global Connection | Gaia+LEO (US) | Commercializing LEO | NASA POWER (irradiance, temp, humidity) | Python, mixed-integer optimization, Streamlit/Dash, Three.js | Repo is the "project" (no app) | stayingfoolish/gaia-LEO (2 stars, notebooks + `mip_model.py`) |
| Art & Technology | Zumorroda-X, "Henry's Farm Adventures" (Egypt) | NASA Farm Navigators | Landsat before/after, NDVI, US Drought Monitor, IMERG, SEDAC CROPCLIM/food-supply | Phaser 3 (JS) game, Python crop ML model, Piskel sprites | Yes (Vercel) | esralsayed/test_farmy (194 KB, 4 stars) |
| Local Impact | QUEÑARIS (Peru) | BloomWatch (flowering phenology) | MODIS/LP DAAC vegetation indices (NDVI, EVI, NDMI...), FIRMS, LAADS via earthaccess | Django 5 + DRF + PostGIS/GeoDjango, Celery/Redis, React 18/TS/Vite/Tailwind/shadcn, Gemini 2.5, Rasterio/GeoPandas/GDAL, DJI Matrice 4 drone | Yes (Vercel) | None found |

## Per-winner sections

Demo-video links: SpaceGenes+ Drive; Resonant Exoplanets https://www.youtube.com/watch?v=5fBB5YJpfwU; Twisters Drive; Astro Sweepers Drive; PureFlow https://youtu.be/hHUpo7bTVuc; Photonics Odyssey (Canva deck); HerCode https://youtu.be/1C9ueho9ero; Gaia+LEO Drive; Zumorroda-X Drive; QUEÑARIS https://youtu.be/QvbLhHqKULc. Drive links are often not public, so YouTube is safer.

### 1. SpaceGenes+ (Best Use of Science)
- Page: https://www.spaceappschallenge.org/2025/find-a-team/spacegenes/ ; site https://spacegenes-plus.framer.website/ ; video on Drive.
- A single-person dashboard (Saloni T., Germany) showing "synergy genes" (TP53, ATM, GADD45A): genes whose fold-change under radiation plus microgravity exceeds the sum of single-stressor effects. One real GeneLab dataset (OSD-288), a pandas synergy score, a Bolt-generated dashboard, a Framer front page.
- Heavy AI use is declared (Bolt, Lovable, ChatGPT, Gemini, Claude), but the core idea is a scientifically framed non-additivity metric.
- Why it won: a specific scientific insight ("most tools treat stressors in isolation"), a real dataset, clear users (biologists, drug developers, mission planners) and a polished site. Weak points: one dataset, "future" filters, no repo and no validation. It shows judges reward a sharp scientific question over engineering size.

### 2. Resonant Exoplanets (Best Use of Data)
- Page: .../resonant-exoplanets/ ; site https://resonant-planet.vercel.app ; repo https://github.com/sahilssan/resonant-worlds-explorer
- A full pipeline: fetch Kepler/TESS/JWST data, run transit detection (Lightkurve, XGBoost, CNN embeddings), validate with physics, run a custom "Modulus" exact-arithmetic chemistry solver to separate real biosignatures (O2 + CH4) from abiotic false positives, then output plots and PDF reports.
- Repo: Python + TS, FastAPI, React, Docker. The root has dozens of agent-style status documents (`FINAL_HANDOFF`, `MISSION_COMPLETE`, `BIOSIGNATURE_DETECTION`, `CONTROL_VALIDATION_REPORT`). It reads as AI-agent-heavy but also shows control and validation reports.
- Live site works (title "Exoplanet AI - Hunting for New Worlds").
- Why it won: the broadest real-data use (four missions), an end-to-end pipeline, a novel angle (false-positive reduction), a YouTube demo and a research paper. The "exact math transformer" claim is bold and hard for judges to verify, but the ambition and depth carried it.

### 3. Twisters / SkySense (Best Use of Technology)
- Page: .../twisters/ ; formerly https://skysense.surf/ (now a domain-for-sale parking page).
- A weather-risk web app: NASA POWER + Open-Meteo data, comfort-based risk model, safest time windows, Google Calendar sync, a Gemini chat assistant ("TwisterBot"), Firebase auth. React + Figma glass UI.
- Team of 5 from Monterrey. Declares that they wrote the core code and used Lovable and Builder.io only to speed things up.
- Why it won: product completeness and integration (auth, calendar, notifications, chat) that answered the challenge brief directly. It is technology-forward with little new science. The site is now dead, so reviewing it later is impossible. Lesson: keep the demo alive and record it.

### 4. Astro Sweepers (Galactic Impact)
- Page: .../astro-sweepers-we-catch-what-space-leaves-behind/ ; site https://astro-sweepers.vercel.app/ ; no team repo.
- Automates NASA's Debris Assessment Software workflow (ODAR/EOMP reports) and computes a custom "ASRI" risk index from four weighted components (legacy, compliance, risk burden, sustainability). It includes a delta-v model, a worked case study (CZ-2C rocket body: 22 m/s burn moves it from non-compliant to compliant), 2025 to 2100 population scenarios, an eight-tier SaaS pricing model and financials.
- Uses real NASA tools (DAS, ORDEM, SPARK 2022, NASA-STD-8719.14) and Space-Track.
- Why it won: a concrete regulated customer pain (compliance paperwork), engineering numbers, and a business model, which is what the LEO challenge asked for. The "ML" part is admitted as exploratory. Some page text reads like leftover prompts to the writer, so the writing could be tighter.

### 5. PureFlow (Best Mission Concept)
- Page: .../pureflow2/ ; site https://larasdiniz.github.io/PureFlow-2025/ ; repo https://github.com/larasdiniz/PureFlow-2025
- A habitat design tool: 3D layout with live mass, power and life-support readouts, and a link to NASA DONKI so real solar storms hit the simulated habitat. All plain HTML/JS (the repo is 15 MB, mostly assets).
- All students from a CS course (6 members). The UI is a story ("Mission Architect") with strong copy: "Would your habitat survive a solar storm today?"
- Why it won: it went beyond the layout tool by tying live space-weather data to survivability; the question in the title doubles as a hook. The repo is simple, so presentation and concept beat code size.

### 6. Photonics Odyssey / AakashNet (Most Inspirational)
- Page: .../photonic-force/ ; deliverables are a Drive folder and a Canva deck.
- A sovereign Indian LEO broadband concept: phased-array terminal (about 1,280 elements, Ku/Ka), laser-linked mesh, ISRO/HAL/BHEL manufacturing, a rupee cost cap per terminal (23,000), and a target of 700M unserved people. The software is a no-backend HTML/Tailwind mockup. RF performance was modelled in MATLAB and OpenEMS.
- NASA data use is nominal (image library). Inspirational, not scientific.
- Why it won: a national-scale story and real hardware engineering with numbers. It proves a strong narrative plus a physical-design analysis can win without a working app.

### 7. HerCode Space, "A Solar Tale" (Best Storytelling)
- Page: .../hercode-space/ ; site https://a-solar-tale.netlify.app/ ; video https://youtu.be/1C9ueho9ero
- A scrolling children's story narrated by the Sun in five acts (myth, Galileo, flares, a girl's curiosity). React + react-scrollama + SCSS, AI-assisted illustrations and narration. Data: NASA and NOAA SWPC explainers.
- Why it won: the challenge itself was a story, and they delivered exactly that with a consistent voice, clear science per act, and classroom use cases. Minimal tech, high polish.

### 8. Gaia+LEO (Global Connection)
- Page: .../gaialeo/ ; repo https://github.com/stayingfoolish/gaia-LEO
- A mixed-integer optimization (`mip_model.py`) that places 40 MW orbital data-center nodes and ground sites to minimize CapEx + OpEx under latency, bandwidth, power and thermal constraints. Streamlit/Dash dashboard, a Three.js visual (`main.js`), and notebooks for antenna sizing and constellation analysis. Data: NASA POWER (2 years), US EIA energy prices/CO2, a Starcloud white paper.
- Repo is small (about 1 MB) but substantive and readable.
- Why it won: genuine operations-research modelling that answers "when does orbital compute beat terrestrial?", a timely topic (AI energy and water), and physics constraints. It is the clearest case of "real model, not a dashboard."

### 9. Zumorroda-X, Henry's Farm Adventures (Art & Technology)
- Page: .../zumorroda-x/ ; site https://henrysfarmadventures.vercel.app/ ; repo https://github.com/esralsayed/test_farmy
- A Phaser 2D educational game: Henry travels to India (heat), Pakistan (2022 floods, Landsat before/after slider), Egypt (salinization), the US (drought map), Brazil (CO2 and crop-survival ML model). Pixel art made in Piskel. Static site, no build step.
- Four female CS students from the German University in Cairo. Humorous team identity ("residents of Planet Zumorroda").
- Why it won: the challenge asked for a game, and they used five real datasets inside a story with named farmers and real events; the game is playable. The repo is named `test_farmy` and has clutter files (`force-refresh.html`, `test-simple.html`), so polish was in the product, not the repo.

### 10. QUEÑARIS (Local Impact)
- Page: .../watana-project/ ; site https://quenaris.vercel.app ; video https://youtu.be/QvbLhHqKULc
- Smart reforestation of queñua (Polylepis) forests for Arequipa's water supply: drone orthomosaics (DJI Matrice 4, Metashape, QGIS), MODIS/LP DAAC vegetation indices via earthaccess, Gemini analysis, mycorrhizal seedlings (5,000 already grown, over 80% target survival), and a first planting scheduled for December. Backend Django/GeoDjango/Celery, frontend React/TS.
- Real stakes: spring flow dropped from 14 L/s to 0, and the page quantifies water use shares by sector (agriculture, energy, mining).
- Why it won: a real local problem, field hardware, physical seedlings already prepared, a dated next step, and a full-stack geospatial platform. The challenge was loosely matched (BloomWatch) but local impact is what the award measures.

## Cross-cutting patterns

What separated winners:
1. A sharp question or number, not a topic. "Non-additive gene synergy", "ASRI index", "when does orbital compute beat Earth", "would your habitat survive today's solar storm". Each has a one-sentence hook.
2. A named user or customer with a real pain. Operators filing DAS reports, biologists, farmers named Bilal and Ramesh, Arequipa's water users, insurers.
3. Real datasets used for what they actually measure, and more than one. Resonant (four missions), Zumorroda-X (five datasets), QUEÑARIS (MODIS + drone). Only SpaceGenes+ and Gaia+LEO relied on one dataset, and both compensated with a real calculation.
4. An actual computation beyond display: a synergy formula, a MIP solver, a risk index with weights and sensitivity, delta-v math, link budgets, live DONKI feed into a simulation. Pure dashboards did not win.
5. Challenge fit plus a twist. PureFlow added live space weather, Astro Sweepers added a business model and compliance automation, Zumorroda-X made a game with real maps.
6. Storytelling and presentation: personas, named case studies, dated next steps, humour (Zumorroda-X), a children's narrator (HerCode). Most pages carry a pitch, not a feature list.
7. Real-world proof where possible: seedlings grown, drone flown, hardware cost capped, financials, a worked case study.
8. The three LEO winners (Astro Sweepers, Photonics Odyssey, Gaia+LEO) show the "concept plus business model" challenge type wins on analysis depth, not apps. Three of ten winners came from one challenge, which suggests business/concept challenges are less crowded.

Stacks: React or plain JS front ends (about 6 of 10), Vercel/Netlify/GitHub Pages/Framer hosting, Python for any real calculation (FastAPI, Streamlit, pandas, XGBoost), Lovable/Bolt/Builder.io used for scaffolding, Phaser for games, Django/GeoDjango for the one heavy platform. Nobody needed a complex stack; Three.js/3D appeared in two projects (PureFlow, Gaia+LEO), AR/VR in none. Every team declared AI use openly, and agent-assisted builds clearly did not hurt.

Anti-patterns and risks visible in the set:
- Dead demos: SkySense's domain lapsed within a year. Unreviewable demos lose later value; use a platform that will outlive the event, plus a public video.
- Private Google Drive demo links (7 of 10); some judges may not open them.
- Overclaiming: Astro Sweepers lists ML as "exploratory" and has leftover instruction-style text; Resonant's exact-arithmetic solver claim is unverified from the outside.
- Repos hard to find or messy (`test_farmy`, a README-only repo, no repo at all for five projects). Winners did not need a clean repo, but nobody wins because of a bad one.
- Winners avoided: a generic dashboard over one CSV with no insight, a chatbot wrapper with no domain model, and a feature list with no user.

## Checklist to win in 2026 (solo, with AI coding agents)

1. Choose the challenge that is least crowded and rewards analysis (concept/business/modelling), then write one sentence: "For [named user], I show [specific number/insight] using [dataset]." Do this before any code.
2. Pick one real model to build and own it: a score with explicit weights, an optimization, a physics or statistics calculation. Have the agent write it, then verify it by hand on one worked example and put that example on the page.
3. Use at least two real NASA datasets (from the challenge resource list), and one non-NASA source for context. State on the page what each dataset contributes. Prefer APIs (POWER, DONKI, earthaccess/MODIS, GeneLab, MAST) so data stays live.
4. Name a persona and one concrete scenario (a specific place, event, or case study with numbers, e.g. a 2022 flood or one satellite).
5. Add one validation or sanity check: compare against a known result, a before/after, a sensitivity run, or a control case. Winners with this looked rigorous.
6. Add one live-feed or real-world hook (a live API in the simulation, a dated next step, a hardware cost cap, a pilot plan).
7. Deploy on Vercel/Netlify/GitHub Pages before the second night, then confirm it opens in a private window. Do not rely on a domain you must renew; keep it live past the awards (about 4 to 6 weeks).
8. Make a 30-second to 2-minute demo: hook, persona, the one insight, the model result, the next step. Upload it to YouTube as public or unlisted, not Drive. Add captions.
9. Write the project page like a pitch: summary first (what, for whom, why), then how it works, a worked example, data list, AI disclosure, limitations, next steps. Delete agent boilerplate and prompt leftovers before submitting.
10. Publish a tidy public repo (README with screenshot, run steps, data sources, limitations, one test of the core model), link it from the page, and be honest about AI use. Use agents for scaffolding, UI and docs, and spend the saved human time on steps 1, 2 and 5.
