# How to review the wireframes and brand directions

About 30–40 minutes. You pick one layout per screen and one brand set, then write the picks in `DECISIONS.md`.

## 0. Open the pages (2 min)

From the repo root, start the server:

```bash
bunx serve design -l 4173
```

Then open these in a browser. **Keep the trailing slash**, or the styles won't load.
- Wireframes: http://localhost:4173/wireframes/
- Brand: http://localhost:4173/brand/

Keep `design/DECISIONS.md` open next to them to write picks as you go.

## 1. Wireframes: one screen at a time (about 4 min per screen)

Do this for each of the 5 screens on the comparison page.

1. Read the three captions and the "+ / −" lines under the thumbnails.
2. Click **Open ↗** on each of A, B and C and look at it full size.
3. Ask the questions for that screen (below).
4. Write a letter in the table in `DECISIONS.md`. Mixing is fine, for example "B, with the event table from A". Add one line on why.

Judge **layout and flow only**. Colour, fonts and polish come later, and all the numbers are fake.

### Questions per screen

**Panorama + timelines (Must, built first)**
- In 5 seconds, can you tell *when* the first lethal shadow arrives?
- Does the panorama make it obvious that the Sun is skimming the horizon?
- Can you match a point on the panorama to a point on the timeline?

**Planner (Must, this is the hook)**
- Is "operable days + time to first lethal shadow" the first thing you see?
- Can you do demo step 3 (move the pin 300 m or shift the date 2 days) and see the change?
- Where would a CLPS planner look for the lander inputs (panel height, battery, load)?

**3D terrain (Should)**
- If 3D gets cut on Saturday, does this layout still work as a 2D map? (Cut order 1)
- Does it link back to the numbers, or is it just pretty?
- Will it read well in the 30-second video?

**IM-2 replay (Should)**
- Does it tell the demo story in order: lander on its side → regional map says sun → add the crater → move or wait → check before you fly?
- Is it clear which parts are the published record and which are our model? Are unconfirmed facts marked [U]?
- Would a judge get the point without someone narrating?

**Validation scoreboard (Must: V1, V5)**
- Can you see pass and miss at a glance, with the real number and its limit?
- Are misses shown honestly, not hidden?
- Will it fit on one slide for the deck?

## 2. Brand directions (about 10 min)

Scroll through A, B and C on the brand page. For each set, check:

1. **Logo at small sizes.** Look at the 16 and 32 px tiles. Does it still read as a mark, or turn into a blob?
2. **Samples on terrain.** Can you read the headline number and the "lethal shadow" line on the dark sample?
3. **Chart colours.** In the three timeline lanes, are Sun, Earth and Operable easy to tell apart? Squint, or check it in greyscale (in Chrome DevTools: Rendering → Emulate vision deficiencies).
4. **Contrast.** Every swatch should say "AA ✓". Anything in red has failed.
5. **Fit.** Does it look like a tool for landing planners, or like a marketing site? Would it hold up on the poster and the 7-slide deck?
6. **Workload.** Set B needs both a light and a dark theme. Is that worth it for two people?

Pick one set, or name the parts to mix (for example "A's logo and palette with B's type").

## 3. Record and hand off (5 min)

1. Fill in the "Open: Charls to pick" table in `design/DECISIONS.md`: a letter per screen and the brand set, each with a one-line reason.
2. Answer or strike out the open questions under it.
3. Tell Claude your picks. The next step is the hi-fi prototype in `design/prototype/` plus `design/DESIGN.md` (tokens).
4. For Qi Cai's review on Nov 13: commit on a branch and open a PR, or ask Claude to do it.

## Red flags to call out

- Any screen where the survival headline isn't visible without scrolling.
- Colour as the only signal for "lethal" or for pass/fail (there must always be a text label too).
- Real-looking numbers without a FAKE label.
- A layout that depends on a Stretch feature (Monte Carlo, all nine regions, IM-1).
