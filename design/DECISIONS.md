# Design decisions and open questions

For Qi Cai's review on Nov 13. Newest first.

## 2026-11-14 · Screen 6 lo-fi pick: Compare A vs B (Charls + Claude review)

Method:
- The three pages in `wireframes-v2/compare/` were checked in the browser.
- They were scored against IDEA.md "What the challenge page stresses" and PLAN-DRAFT T27 and Q10 (1 h budget).
- None needs a redo.

| Screen | Pick | Why | Fix before hi-fi |
|---|---|---|---|
| Compare A vs B | **C** Difference as summary (B − A sentence moved to the bottom, Charls) | A and B and the shared timeline come first, then one sentence sums up B − A for planners and the public. It answers the schedule-slip question ("compare sites and dates quickly") and builds from the same pieces as A plus one sentence, so it fits T27's 1 h. | 1. The sentence must handle a site change, a date change, both, B being better, and no difference, not only "3 days later costs…". 2. Say which moment the clocks show: one shared time cursor, or each side's own calendar date. 3. Add a "custom coordinates" entry to the site picker (IDEA asks for curated sites plus custom coordinates). 4. Label the shared axis with calendar dates, not only "+3d". |

**Not picked:**
- **A, Two columns:** sound, but the reader has to work out B − A, and the timeline sits far from the numbers it explains. Keep it as C's fallback layout.
- **B, One timeline with two lanes:** needs a new two-lane chart (over the 1 h budget), and its clocks are too small for the challenge's "time left before sunset or Earthset". If reused, fix:
  - the oversized SVG labels that overlap the lanes;
  - the clipped "+33d" tick;
  - the table that scrolls sideways on phones.

**Open question for Qi Cai:** should the clocks follow one shared time cursor, or each side's own landing time?

## 2026-11-14 · Round-2 picks against the Resources tab, plus a screen-6 redo (Qi Cai + Claude review)

Method:
- All 15 round-2 pages were rendered at 1440 px in headless Chrome and checked by eye.
- They were scored against what the challenge page stresses (IDEA.md "What the challenge page stresses"): Sun and Earth relative to the horizon, time left before sunset or Earthset, fast comparison of sites and dates, why small changes matter, and the planner, educator and public audiences.
- Charls can override any pick.

| Screen | Pick | Why | Fix before hi-fi |
|---|---|---|---|
| Site | **v2 B** Countdown first | It is the challenge's "time remaining before sunset or Earthset" and the Must T26 readout. The "None" state is already designed. | Borrow A's eye-height selector into the header. |
| Planner | **v2 B** What changes what | The challenge asks why small location or date changes have a large effect, and the "which input matters" bars answer it. | Add a "pin moved 300 m" bar next to the ±2-day bars. |
| Terrain | **v2 C** Cross-section to the Sun | It shows how terrain sets the horizon (what Moon Trek is listed for), is cheap, and survives the 3D cut. | The Sun ray is drawn through the ridge while the panel says "clears the ray by +31 m". Draw the ray grazing above the peak, or make the sign agree. |
| IM-2 replay | **v2 A** Pit model explained | The demo's key moment. | The ray starts on the left crater wall, not at the panel: start it at the panel top so it meets the right rim. Add B's sourced verdict table (Validity criterion). |
| Scoreboard | **v2 C** One-line verdict | Readable for the public and slide-ready, with the data and citation table on the same page. A's cards sit one click away. | Headline says "3 of 4 checks pass" above 5 boxes. Use "3 of the 4 checks we ran pass; V3 not run". |

**Not sufficient as drawn (do not build from these):**
- **Planner A:** the four step cards are empty boxes taking half the screen.
- **Terrain A:** a hatched placeholder with no terrain or shadow content, so there is nothing to judge.
- **Site C:** shows no Sun or Earth track (the challenge's core visual) and has an empty band under the clocks.
- **Replay C:** its 1 km search ring isn't centred on the IM-2 pin. Fix that if it is reused.

**Redo: screen 6, Compare A vs B (new, Should).**
- **Why a new screen:** none of the 15 round-2 options compares two site-date pairs, yet the challenge page asks for it twice ("compare landing sites and dates quickly", "side-by-side or sequential comparison"). The schedule-slip use case depends on it.
- **What it shows:**
  - two columns, A and B, each with site, date, the 4 countdown clocks, operable days and first lethal shadow
  - a B − A difference row
  - both timelines on one shared time axis
  - a curated site picker: Artemis regions, IM-1, IM-2, and Apollo/other sites marked `OUT_OF_COVERAGE`
  - "open in QuickMap / Moon Trek" links
- **Where to start:** the round-2 whole-app take `wireframes-v2/flows/c` (compare and rank).
- **Ticket:** PLAN-DRAFT T27, promoted to Should (Q10).
- **Owner:** Charls, via `/design-pass`, three directions as before.

Round-2 table below is filled from this entry.

## 2026-11-12 · One orange: ember everywhere (Charls)

- **Decision:** ember `#da5c2c` is Penumbra's only warm colour. It is used for the Sun (charts, slides), the Sun in the wordmark and the favicon mark, the primary button, log bars and the callout border. Amber `#f4a740` is retired.
- **Why:**
  - Amber plus ember on one page read as two near-identical oranges.
  - Charls compared both on `brand/orange-options.html` (now deleted) and picked ember, the pure Axiom look.
  - The old objection (ember too close to lethal red) no longer applies, because lethal has no colour now.
- **Costs and fixes:**
  - Ember is 5.5:1 on black (amber was 10.4:1): fine for lines, fills and labels.
  - Axiom's white text on ember is 3.3:1, which fails AA at 16px. The primary button label is black instead (5.5:1).
- **Rule:** on any chart that shows the Sun, nothing else is ember, so orange always means Sun.
- **Files:**
  - `DESIGN.md` and `tokens.css` updated (`--color-sun` is now an alias of `--color-ember`).
  - `brand/assets/sun-mark.svg` and `sun-mark-16.svg` recoloured. The 16px file also had three SVGs pasted into one; it is now one.
  - `brand/index.html` updated.

## 2026-11-12 · Brand: full migration to Axiom (team)

- **Decision:** Penumbra now uses the Axiom style for everything:
  - surfaces `#000` → `#111` → `#191919` → `#202020`, and the grey ramp
  - monospace type throughout
  - 2px corners, flat, no gradients or shadows
  - `~/` prompt labels, `>` arrow field, log bars
  - status shown by symbol, not colour
- **Who and why:** asked by big back in the team chat on Nov 12; Charls and still water agreed. Our Low Sun tokens were "too wordy and not detailed"; the team prefers Axiom's look and font.
- **Kept from Low Sun:**
  - Sun colour amber `#f4a740`.
  - The PENUMBRA wordmark, unchanged (Archivo italic, amber Sun mark).
- **Changes to Axiom:**
  - **Blue:** Axiom's `#2a7fff` → NASA blue `#0032A0`. Fills only, because it is 2.0:1 on black. Lines and text use the tint `#4f7fd9` (5.4:1).
  - **Font:** BerkeleyMono (paid) → IBM Plex Mono, one of Axiom's listed substitutes. Inter stays for fine print.
  - **Ember `#da5c2c`:** kept as the UI accent (primary button, log bars, card left border), never for the Sun. This settles the contradiction in the Axiom source.
  - **Lethal:** no colour. Shown as ✕ in white plus words. `#ff6b5e` is retired, which also removes the ember-vs-lethal clash noted below.
- **Files:**
  - `DESIGN.md` and `tokens.css` rewritten.
  - Sources saved as `brand/axiom-source.md` and `brand/axiom-tokens.json`.
  - Token names changed, so `slides/index.html` and `brand/ember-options.html` lose their colours until rebuilt or removed.
- **Next:**
  - Brand page in the new style.
  - Update the Canva "Axiom test" deck (Sun → `#f4a740`, blue → `#0032A0` / `#4f7fd9`, font → Plex Mono).
  - Remove the Low Sun pages.

## 2026-11-12 · Brand colour: keep amber (Charls) · SUPERSEDED by the Axiom migration above (the Sun amber survives)

- **Decision: A · keep Low Sun amber `#f4a740`** for the Sun, the mark and the wordmark. Ember `#da5c2c` was not adopted. Charls picked it on Nov 12 from `design/brand/ember-options.html`.
- **Why ember was set aside** (all measured):
  - It is only 12° of hue from lethal `#ff6b5e`, and almost identical to it in the protanopia simulation, so sunlight and the lethal shadow could be confused.
  - It is 4.0:1 on the panel colour, too low for small text.
  - Adopting it would also have meant moving lethal to violet-magenta.
- **Taken from the Axiom reference:** only the restraint (one warm accent, used rarely, on near-black). This matches DESIGN.md's "Sun Means Sun Rule". Axiom's typeface, mark and layout are not used.
- Not decided: switching to Axiom-style neutral grey surfaces (`#191919` instead of `#1f2630`). The page has a toggle for it.

## 2026-11-12 · Provisional DESIGN.md and shared tokens (Charls + Claude)

### Made
- **`design/DESIGN.md`** (provisional), written with `/impeccable document` from the brand A work that already exists. Nothing new was invented.
  - Frontmatter holds the tokens: palette, type roles, radius, spacing and three components.
  - The body gives the rules, with measured contrast. For example, Earthshine is only 4.4:1 on surface, so use it only for charts or text of 24px and up there.
  - North Star: "Grazing Light". Final values come in `/design-pass 3`.
- **`design/tokens.css`** mirrors the frontmatter as CSS variables. Change a value in DESIGN.md first, then here.
- **`design/brand/assets/`**: `sun-mark.svg` (the lockup starburst) and `sun-mark-16.svg` (the 9-ray small size). They are made from the same geometry as the brand pages.
- **Slides now point at these files** instead of copying values from `brand/index.html`.
  - They link `../tokens.css`, with no colour values left in the slide file.
  - The lockup Sun is the SVG asset; the JavaScript that redrew it is gone.
  - Re-rendered: slides 1, 2, 5, 6 and 7 match the Canva plates pixel for pixel. Slide 3's panorama background moved from #05070a to pure black. Slide 4 differs only by anti-aliasing.
  - The detector now checks the slides against DESIGN.md. One finding is left, "cramped padding" on the full-bleed title slide, which is a false positive.

### Open
- The wordmark is still live Archivo text, so there's no lockup SVG yet. An outlined wordmark is still needed (Nov 11 open question).
- The brand pages (`brand/index.html`, `low-sun-marks.html`) and the wireframes still carry their own copies of the values. Point them at `tokens.css` in `/design-pass 3`.
- The impeccable sidecar (`.impeccable/design.json`) hasn't been written yet.

## 2026-11-12 · D5 slide deck template, issue #4 (Charls + Claude)

### Made
- **7-slide deck template** in `design/slides/index.html` (16:9, 1920×1080), in brand set A Low Sun. It covers D5's required content:
  1. title with team and member names
  2. the challenge
  3. a project image
  4. the user experience
  5. how NASA and JPL data make it possible
  6. validation
  7. what's next
  - Arrow keys move between slides, and P prints the deck to PDF for submission.
  - Results are `PLACEHOLDER` and the panorama is labelled FAKE, because there are no real results before Nov 14.
- **Built with two skills:** `slides` (through `/design`) for the deck, and `canvas-design` for the title artwork.
  - The artwork's design philosophy is "Grazing Light" (`design/slides/philosophy.md`).
  - The artwork itself is `design/slides/title-art.png`: a half-risen starburst Sun on a ridge-line horizon, the Sun and Earth paths, a 0–2° scale and long-shadow hatching. It is labelled "ILLUSTRATIVE".
- **Brand used as tokens:**
  - Ground `#000000`, surface `#1f2630`, text `#ebe7df`, muted `#a7b0bc`.
  - Sun `#f4a740` and Earth `#3592c4`. Lethal `#ff6b5e` appears only with the words "lethal shadow".
  - Montserrat Medium for text and IBM Plex Mono for numbers. The real starburst lockup is on the title slide.
  - A footer horizon on each slide carries a small Sun that rises as the deck advances.
- **Canva version (first try):** colour edits only, left unsaved; replaced by the image version below.
- **Checked:**
  - Every slide was rendered at 1920×1080 in headless Chrome.
  - Fixed a physically wrong shadow diagram: the ray now hits the ridge.
  - Fixed a Sun path that never dipped behind the horizon, an oversized placeholder, and an empty closing slide.

- **impeccable critique, then fixes (same day):**
  - Score 21/32. Detector: 0 findings in the file; 1 in the browser, a widowed heading on slide 6, now fixed.
  - Charls chose to fix only the P0 and P1 issues:
    - **Slide 2:** the claim "the Sun never climbs more than about 1.5°" was wrong for 84.6° S, where it reaches about 7°. It now reads "skims the horizon, a few degrees up at most". The unsourced "about a day" for IM-2 is gone.
    - **Slide 1:** the wordmark is now measured on the loaded Archivo and scaled evenly, not stretched with `textLength`. The title art's Sun is larger (K 3.5 → 5.5), so it reads as the scene and the lockup as the signature.
    - **Slide 4:** now has a payoff. It shows FAKE values (8.5 operable days, first lethal shadow at T + 8 d 12 h at SP-01) and a timeline where the Sun sets behind the ridge (Charls asked for the sunset).
    - **Slide 6:** has FAKE results and a PASS/MISS status column, written in words. Targets come from SCIENCE-SPEC.
  - Kept, per Charls's scope: the amber eyebrow labels (impeccable treats these as a ban), the colour overload, and the rubric gaps (commercial story, Next Frontier).
- **Canva version (replaces the earlier one):** the Canva design "Penumbra Hackathon" (DAHaocIUy18) now holds 7 full-bleed slide images rendered from the HTML in the brand fonts, with alt text.
  - The 12 result values on slides 4 and 6 are editable Canva text, in Canva's default font because the connector can't set fonts.
  - To refresh the images after Nov 14, open `index.html?plate#N`, capture it at 1920×1080 and swap the page background.

### Open
- Still to do for issue #4: D2 team poster (a good fit for `canvas-design`), the brand kit and the D3/D4 video storyboard.
- The wordmark still uses `textLength` here too (see the Nov 11 open question).

## 2026-11-12 · Lo-fi wireframes, round 2 (Charls + Claude)

### Made
- **5 screens × 3 new directions** in `design/wireframes-v2/<screen>/{a,b,c}.html`, compared one row per screen in `design/wireframes-v2/index.html` (same format as round 1). Built from `docs/IDEA.md`, `docs/SCIENCE-SPEC.md` and `docs/ARCHITECTURE.md`; each card names the doc section it comes from. Round 1 (`design/wireframes/`) is kept.
  - **Site:** A lander's-eye view (Sun-disc close-up, eye-height slider) · B countdown first (four live clocks with ±) · C which ridge casts the shadow (horizon shaded by distance, range chart).
  - **Planner:** A step by step (site → date → lander → answer) · B what changes what (each input shows what it reruns; bar chart of which input matters most) · C landing-date calendar 2026–2030.
  - **Terrain:** A shadow sweep (3D + day film strip) · B region heatmap ranking (100 m cells, best cell within 1 km of IM-2) · C cross-section from the pin to the Sun.
  - **IM-2 replay:** A pit model explained (crater cross-section, depth slider) · B timeline check for V4 (published vs model events) · C could it have survived? (nearest better site and date).
  - **Scoreboard:** A evidence per check · B error plots (V1 scatter, V2 scatter and histogram) · C one-line verdict with the miss explained.
  - Some options build on a round-1 idea (Planner B, Terrain B, Replay B and C, Scoreboard B); the index says which.
- **One FAKE model drives every chart and number** (`design/wireframes-v2/v2.js`), shaped like the engine's real outputs (horizon, timeline columns, survival header), so a picture can't disagree with its number. It is not lunar maths. Example: IM-2 site terrain only = 8.5 operable days; with the 20 m crater = 1.2 days (lethal T+1d 4h).
- An earlier whole-app take on round 2 (A planner console, B story first, C compare and rank, plus a states page: loading, `OUT_OF_COVERAGE`, no lethal shadow, `NOT_LOADED`, slip fallbacks, phone width) is kept as reference in `design/wireframes-v2/flows/`.
- Checked every page at 1440 px and 375 px: text contrast AA, no horizontal overflow, chart labels ≥ 11 px on screen, no script errors. Planner C draws about 900 fake runs and takes about 1.5 s.

### Open: Charls to pick (round 2)
| Screen | Pick (round 1 or 2, A / B / C or mix) | Notes |
|---|---|---|
| Site (panorama + timelines) | v2 B (Nov 14) | + A's eye-height selector |
| Planner | v2 B (Nov 14) | + "pin moved 300 m" bar |
| Terrain | v2 C (Nov 14) | fix the ray/ridge drawing |
| IM-2 replay | v2 A (Nov 14) | fix the ray origin; + B's verdict table |
| Scoreboard | v2 C (Nov 14) | fix the "3 of 4" headline |
| Compare A vs B (new) | to design | screen 6, see the Nov 14 entry |

## 2026-11-11 · Lo-fi wireframes and brand directions (Charls + Claude)

### Made
- **Mark page refinements** (Charls, Nov 11, in `design/brand/low-sun-marks.html`):
  - Primary ground changed from `#0d1117` to pure black `#000000`, also used as the ink on light grounds.
  - Supporting typography (body, headings, tags) changed to **Montserrat Medium (500)**. The wordmark stays wide italic Archivo 800.
  - The Grazing Sun is drawn as a **starburst**, not a plain half-disc: a small core ringed by 15 thin rays of uneven length, upper half only. At 16 px it drops to 9 thicker rays so they don't blur.
  - The Sun moved closer to the wordmark and now sits over the "B" and "R" (centre x 462 → 295, nudged three times). The horizon line was shortened to end just past it, and the empty space on the right of the lockup was cropped (viewBox 540 → 362 wide, same wordmark size).
  - Carried into `design/brand/index.html` set A the same day (lockup, type panel, sample, palette). The half-disc Sun was replaced with the same starburst (lockups and all icon sizes).
- **Five screens wireframed, three structural directions each**, in `design/wireframes/<screen>/{a,b,c}.html`; compare them in `design/wireframes/index.html`. Screens follow CHARLS-START F1: panorama + timelines, planner (pin/date + survival headline), 3D terrain, IM-2 replay, validation scoreboard.
  - The region heatmap (Should) has no screen of its own. It lives in Terrain C (2D map coloured by operable days), which is also the fallback for cut order 1.
- **Three brand directions** in `design/brand/index.html`: A Low Sun (recommended), B Almanac, C Instrument. Every text and chart colour passes WCAG AA against its ground, measured live on the page and with a Bun check.
- A's Earth colour was darkened from `#5ec4e6` to `#3592c4`. The two were almost equal in lightness to the Sun amber, so the tracks would shimmer where they overlap.
- Brand process: method from the `brand-identity` skill (mechanism per set, axes, AI-default reflex test, competitor read, contrast gate). Its Python pipeline was **not run**: 21k lines of low-trust scripts, and the repo has a no-Python rule. As a result, there are no outlined SVG wordmarks yet; wordmarks are live Google Fonts text.

- **Low Sun v2** (`design/brand/low-sun-v2.html`): **not adopted.** Charls kept the original set A page (`design/brand/index.html`) on Nov 11. v2 was made because Charls felt the first page looked "a bit AI slop". It was a refined version alongside the original, not a replacement. The concept is the same (Grazing Sun mark, amber on blue-black, Archivo + IBM Plex Mono). The craft changes:
  - The wordmark is measured on the loaded italic Archivo instead of stretched to a fixed width. v1 used `textLength`, which distorted the letters.
  - A construction drawing with cap-height units, clear space and Sun radius.
  - The hero is a live panorama: the Sun glides along illustrative ridges and the readout shows the disc fraction it computes.
  - Colour is shown as a strip proportional to how much each colour is used.
  - Pill tags, eyebrow labels, swatch cards and the big-number tile are gone. The tool answer is set as a sentence.
  - Three misuse rules specific to this mark.
  - Themed selection, focus and scrollbar.

### Open: Charls to pick
| Screen | Pick (A / B / C or mix) | Notes |
|---|---|---|
| Panorama + timelines | | |
| Planner | | |
| 3D terrain | | |
| IM-2 replay | | |
| Scoreboard | | |
| **Brand set** | **A · Low Sun** (Nov 11) | Charls liked A but not the disc beside the name. The mark moves to the right of the wordmark with a long line back over it (reference: a Lockheed Martin-style lockup). Three mark options are in `design/brand/low-sun-marks.html`: 1 Grazing Sun, 2 First Light (star glint), 3 Long Shadow. **Mark: 1 Grazing Sun** (Charls, Nov 11): a half-risen starburst Sun over the "B" and "R" and a hairline horizon over the tops of the letters, with a wide italic Archivo wordmark. Ground is pure black `#000000`; supporting text is Montserrat Medium (500), numbers IBM Plex Mono. On light grounds the amber darkens to `#b0600a`. |

### Open questions
- The wordmark is still stretched to a fixed width with `textLength`, which distorts the letters. Fix in `/design-pass 3`: measure it on the loaded Archivo font (as v2 did), or outline it to SVG paths.
- Sun and Earth tracks: separate them by colour only, or also by dash pattern as in the wireframes? (Recommended: both, for colour-blind users.)
- Set B needs a dark mode for terrain views, which means two full themes. Is that worth it for a two-person team?
- Lethal colour plus text label everywhere; never colour alone.
