# Submission research, 2026-11-15

Read-only lookups for checkpoint 5 (#131). Each finding has its URL. [V] = checked verbatim; [U] = relayed or summarised.

## a) Space Apps discussions

Method: read-only `gh api graphql` over nasa/spaceapps, 469 discussions with bodies and comments. [V] means checked against the thread text. [U] means relayed by a commenter, or the rule text wasn't read at source. Many answers come from community Navigators, not the GO Team.

### Submission pitfalls
- **Demo is one of: 7 slides max, or a video of 30 s or less.** Pick one primary demo. [V, quoted by asker] https://github.com/nasa/spaceapps/discussions/2226
- **No login-gated links.** The demo must not require a password, permission or registration; test it logged out. [V] https://github.com/nasa/spaceapps/discussions/2226
- **Final project link can be a website.** Source code isn't required; the link must be public with no login. [V] https://github.com/nasa/spaceapps/discussions/2202
- **"Check back later" on the project page.** Use the blue Edit button on the Team page, or log out and back in. [V] https://github.com/nasa/spaceapps/discussions/2213 · https://github.com/nasa/spaceapps/discussions/2239
- **Deadline and time zone.** Local events use their own time zone. NSAKL26 closes at 22:00 (docs/SUBMISSION.md). [V] https://github.com/nasa/spaceapps/discussions/2228 · https://github.com/nasa/spaceapps/discussions/2216
- **AI-generated images and video need a visible label on each image.** A credits page isn't enough. Unmodified NASA imagery and hand-made visuals don't need one. **Check the poster and the 30-s video.** [V/U] https://github.com/nasa/spaceapps/discussions/2195 · https://github.com/nasa/spaceapps/discussions/2163 · https://github.com/nasa/spaceapps/discussions/2225
- **Mission identifiers and branding.** Avoid NASA logos and mission patches. Describing Artemis III candidate regions as data is distinct from using official branding. [V/U] https://github.com/nasa/spaceapps/discussions/2017
- **Data citation.** At least one NASA data source is required for Global Awards (GO Team). List all data, code, libraries and images, and label modelled values. [V] https://github.com/nasa/spaceapps/discussions/1849 · https://github.com/nasa/spaceapps/discussions/2092
- **Work must be built in the hackathon window.** [U] https://github.com/nasa/spaceapps/discussions/2092
- **Free-domain OTP failures.** Use a pages.dev address (we already do). [V] https://github.com/nasa/spaceapps/discussions/2231
- **At least 2 registered members for global judging.** Members are set before the deadline. [V/U] https://github.com/nasa/spaceapps/discussions/2201
- Not found: anything on late-submission acceptance or Drive permission problems.

### Lunar and south-pole threads
- **Challenge scope (Lunar Landing Site Navigator).** "Rapid evaluation of lunar landing site options": compare sites and dates, with Sun and Earth positions against the horizon. Matches Penumbra. [V] https://github.com/nasa/spaceapps/discussions/2135
- **Format is open.** A web app is accepted. [V] https://github.com/nasa/spaceapps/discussions/2160
- **Data.** Other teams found JPL Horizons, Moon Trek and LROC QuickMap useful. [V] https://github.com/nasa/spaceapps/discussions/2129 · https://github.com/nasa/spaceapps/discussions/2092
- **Same-challenge peers: SUNLOCK** (south-pole sunlight and Earth visibility, LOLA 80 m, live shadows) https://github.com/nasa/spaceapps/discussions/2170 **and "Lunar Window"** (Taipei) https://github.com/nasa/spaceapps/discussions/2092. Penumbra's differentiators: validation against Horizons and LOLA AVGVISIB, the first lethal shadow, the 30-date window, and the 20 m DEM.
- **Scope note.** Separate transfer and descent from surface conditions; Penumbra covers surface illumination and Earth contact only. [V] https://github.com/nasa/spaceapps/discussions/2135

## b) Challenge page and NSAKL26 guidebook → Penumbra

Challenge: Lunar Landing Site Navigator, https://www.spaceappschallenge.org/2026/challenges/lunar-landing-site-navigator/ (read 2026-11-15 in a browser). Item names are the page's headings; the rest is paraphrased.

| Item (challenge page) | Where Penumbra answers it | Gap / Future work |
|---|---|---|
| Summary: compare sites and dates quickly; Sun and Earth vs the horizon; power and direct-to-Earth comms windows | Site view headline (operable time, first lethal shadow); Compare tab; Earth contact hours (facts §1) | Earth *visibility* stands in for a DTE link; no link budget |
| Background: Quickmap, Moon Trek and Horizons are hard for non-specialists to use for rapid comparison | README/facts §2 comparison table | — |
| Background: schedule slips force new dates or sites | "Land 2 days early" scenario button; 30-date landing window (#108) | — |
| Objective: interactive site selection and comparison (curated list: Apollo, CLPS, other countries; custom coordinates; side by side) | Curated sites (IM-2, nine Artemis regions + Shackleton ridge), pin inside shipped tiles, Compare A/B | Other countries' sites (#117); click anywhere (#103) |
| Objective: date and time flexibility across lunar days and seasons | Date picker; 30-date window; 3 grid dates for the heatmap | More grid dates (facts §7) |
| Objective: Sun and Earth position vs horizon; time left before sunset or Earthset | Horizon panorama with Sun and Earth tracks; countdown (#107) | — |
| Objective: power and comms metrics (solar elevation, sunlight duration, Earth visibility windows) | Sunlight hours, Earth contact hours, timelines, battery state of charge | — |
| Objective: multiple visualizations (2D, 3D) | 3D terrain (Three.js), 2D panorama, heatmap, uPlot timelines | — |
| Objective: temporal dynamics (animation or timeline) | Play button (#113), timelines | — |
| Objective: accessibility across skill levels (planners, educators, public) | Hero + scenario buttons; "Why the Moon is different" card (#114); plain labels with IDs in tooltips | — |
| Consideration: Horizons or pre-computed ephemeris | DE440 via ANISE, hourly table; V1 against Horizons, 0.00011° | — |
| Consideration: LOLA topography for horizons | 20/80/240 m three-tier horizon | — |
| Consideration: no atmosphere; horizon from terrain and limb | Horizon from terrain and lunar sphere, no refraction (SCIENCE-SPEC §1) | — |
| Consideration: show solar elevation prominently | Panorama + elevation timeline panel (°) | Could add a numeric "Sun elevation now" readout |
| Consideration: lunar vs terrestrial differences | "Why the Moon is different" card | — |
| Consideration: why small changes matter | "Land 2 days early" → operable 0 h; Compare | — |
| Consideration: libration and orbital variation | MOON_PA frame + DE440 (libration included in geometry) | — |
| Consideration: steep slopes and deep craters | Slope from the 20 m DEM (#112); local pit model (V4); PSR mask (V5) | — |
| Engagement: science/engineering coordination; schedule changes; education | Validation tab (shared evidence); landing window; Why-the-Moon card | No multi-user or sharing feature (#102 deep links) |

### NSAKL26 guidebook (local PDF, pages 1–64 read as images; copyrighted, paraphrased)
- p.8: the project document suggests a research-paper outline (Introduction, Methodology, Results, Discussion, Conclusion, References). → D1 follows it (#135).
- p.8: the 30-second pitch covers problem, data, insight, solution, impact and demo. → D3 is done.
- p.6: judges' novelty points include differentiation, comparison and validation. → comparison table + scoreboard.
- p.7: the global award categories include Best Use of Science, Best Use of Data and Best Mission Concept. → the strongest fits.
- p.8: the local standard includes entrepreneurship. → commercialization in README, D1 and D7.
- Pages 65+ were not read.

## c) Moon Trek, QuickMap, JPL Horizons

[V] = quote seen verbatim. [U] = seen only through a summary or snippet; re-check before quoting. The corpus is not public, so each claim cites the public URL.

| # | Question | Moon Trek | LROC QuickMap | JPL Horizons |
|---|---|---|---|---|
| 1 | Sunlight hours at a site and date, terrain-shadowed | Partial: "Surface Lighting" tool, no per-site sunlit-hours output found [1][3] | Partial: Terrain Shadows over a UTC range; no sunlit-hours total found [9] | Partial, no terrain: "lit by the Sun" flag; topography not modelled [14][15] |
| 2 | Earth visibility / contact over time | Not found (Line of Sight tool exists) [3] | Partial: SUN+EARTH and EARTH sources; no contact-hours output [9] | Partial: rise/set of target bodies from a site [16] |
| 3 | Time to first lethal shadow | Not found | Not found | Not found |
| 4 | 30-date landing-window sweep | Not found | Partial: start/end/delta-hours inputs, no sweep summary [9][12] | Raw data only: start/stop/step tables [18] |
| 5 | Slope at a point | Partial: Slope tool, registered account needed [2][7] | Partial: elevation at point and along path; slope not found [11] | No |
| 6 | Validation published against ephemerides or LOLA | Not found in the Moon Trek paper [8] | Not found | Is the ephemeris source (DE441) [20] |
| 7 | Runs with no backend | No: NASA back-end services [4][6] | No: server CGI [9][10] | No: JPL servers [19] |
| 8 | Cost / account | Free to browse; lighting and slope need a registered account [2] | Price not found; "Contact ACT" for Terrain Shadows [9] | Free, "No account or password is required." [19] |

1. https://trek.nasa.gov/moon/ — tools list "Calculate Sun Angle", "Surface Lighting" [V]
2. https://trek.nasa.gov/moon/ — "Advanced tools … (e.g., lighting, slope analysis) require a registered individual or group account." [V]
3. https://meetingorganizer.copernicus.org/EGU23/EGU23-969.html — lighting, slope, line-of-sight analysis [U]
4. Moon Trek paper (https://moontrek.jpl.nasa.gov) — "a browser-based web portal" [V]
6. Same paper — "back end server infrastructure" [V]
7. Same paper — "local hazard assessments including slope" [V]
8. Same paper — no validation against ephemerides or LOLA maps found
9. https://qts.quickmap.io/fcgi-bin/fprovweb.exe?_xtype=text%2Fhtml&cmd_script=qts_js.msh — TerrainShadows; sources SUN, SUN+EARTH, EARTH; START/END UTC, DELTA TIME HOURS [V]
10. Same — Traverse Mode is a server request [V]
11. https://docs.quickmap.io/toolbar/2-query — elevation along a path, CSV export [V]
12. https://docs.quickmap.io/map/0-time — time panel only for time-dependent layers [V]
14. https://ssd.jpl.nasa.gov/horizons/manual.html — surface point "lit by the Sun" [U]
15. Horizons manual — TVH horizon "not local topography" [V]
16. Horizons manual — target-body rise, transit, set [V]
18. https://ssd-api.jpl.nasa.gov/doc/horizons.html — START_TIME, STOP_TIME, STEP_SIZE [V]
19. Horizons manual — "No account or password is required." [V]
20. Horizons manual — interpolates "previously defined ephemerides, such as DE441" [V]

QuickMap has moved to https://quickmap.lroc.im-ldi.com; the old quickmap.lroc.asu.edu address is being retired [V].

**What each does better than a small planner app**
- **Moon Trek:** one portal with lighting, slope, Sun-angle and line-of-sight tools, shareable URL-encoded views, and open data APIs. https://trek.nasa.gov/moon/
- **QuickMap:** a large LROC data catalogue and a terrain-shadow tool with Sun+Earth sources over a time range, plus CSV export. https://docs.quickmap.io/
- **Horizons:** the ephemeris authority (DE441), free with no account, with a scriptable API. https://ssd.jpl.nasa.gov/horizons/manual.html

**Not found:** sunlit-hours totals, Earth-contact hours, first-shadow time or a 30-date sweep in any of the three; validation of Moon Trek or QuickMap against LOLA; a QuickMap price.

## d) README skills and stack-diagram

Report only; nothing installed. Fit for the listed skills is judged from registry names and descriptions, not their SKILL.md.

- **archify** (https://skills.sh/tt-a1i/archify/archify, 192K installs): already in this session's skill list. HTML architecture diagrams with light/dark themes and SVG/PNG export. **Use** if a rendered diagram is wanted; no new dependency.
- **Mermaid in the README**: GitHub renders ```mermaid fences natively, so no image file and no skill is needed. **Use** for the README architecture diagram (stays editable as text). Optional skill: https://skills.sh/softaworks/agent-toolkit/mermaid-diagrams.
- **create-readme** (https://skills.sh/github/awesome-copilot/create-readme, 19.6K): generic README drafter. **Skip**: the README is built from facts.md.
- **readme-optimization** (https://skills.sh/samber/developer-relations-skills/readme-optimization, 13.6K): review pass on an existing README. **Optional**, after the README lands.
- **crafting-effective-readmes** (https://skills.sh/softaworks/agent-toolkit/crafting-effective-readmes, 4.3K): layout guidance. **Skip**: the layout is already set.
- **animated-readme-logo**, **shieldcn-badges**, **readme-hero**: **skip**. They're decorative, plain shields.io badges need no skill, and readme-hero has 1 install.
- **caru-ini/stack-diagram** (https://github.com/caru-ini/stack-diagram; MIT, 2.5K stars, pushed 2026-11-13). Install: `npx skills add caru-ini/stack-diagram`. Makes a stack diagram with official logos as HTML, PNG or React. Its README says it requires Python 3.9+ plus network access for logos, and Playwright for PNG. **Skip**: that conflicts with the repo's no-Python rule, and uPlot logo coverage is unverified. Its "share to gallery" step would publish the stack publicly on stacks.caru.moe.
