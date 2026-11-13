# T10 — V5 red-team suite

Ticket: [PLAN-DRAFT T10](../PLAN-DRAFT.md). Owner: Qi Cai. 1 h. Blocked by T05, T07, T08, T09. Must is complete when this merges. Case 3 needs `data/` (the PSR file, the kernels); the other six run in CI.

## 1. Goal and done-criteria (from the ticket)

The V5 row of the scoreboard.

- [ ] Case 1: at the pole no output is NaN, and the flat-sphere horizon is −0.0869° at H = 2 m at every azimuth.
- [ ] Case 2: flat pole, 2026–2030: Sun elevation within ±1.6°, Earth-centre elevation within ±7.2°.
- [ ] Case 3: 20 pixels well inside regions flagged in `LPSR_80S_20MPP_ADJ.TIF`: visibility is exactly 0 at every hourly step of 18.6 years (vectors generated natively, not shipped). No edge pixels (U15).
- [ ] Case 4: year boundary and both table ends (P4, P16).
- [ ] Case 5: far side at 60°S, Earth never visible (T-1d-1).
- [ ] Case 6: DEM edge returns a coverage error (P5).
- [ ] Case 7: T-1e-1 and T-1e-2.
- [ ] `scoreboard.json` lists each case as pass or miss.

## 2. Files and signatures

`crates/pipeline/src/v5.rs` (new: the seven cases and the `v5` subcommand) · `crates/pipeline/tests/v5.rs` (calls the same seven functions) · `web/public/data/scoreboard.json`. No new function in `core`: this ticket only calls what T01–T09 built.

```rust
// pipeline/src/v5.rs
pub struct CaseResult { pub id: &'static str, pub metric: String, pub threshold: f64, pub value: f64,
                        pub pass: Option<bool>, pub sample_size: u64, pub notes: String }
pub fn case_1(eph: &Ephemeris) -> CaseResult;      // … case_7
pub fn case_3(data_dir: &Path) -> CaseResult;      // pass = None and notes = "not run: data missing" without data/
```

One implementation per case. `cargo test -p penumbra-pipeline` asserts `pass == Some(true)` for cases 1, 2, 4, 5, 6, 7 using the shipped table in `web/public/data/`. `cargo run -p penumbra-pipeline --release -- v5` runs all seven and upserts seven entries, ids `V5.1` … `V5.7`, with the keys of ARCHITECTURE §3.5 (`judge`: "red-team case, SCIENCE-SPEC §3"). A case that did not run is written with `pass: null`, `notes: "not run"`; the page shows "not run", never a pass.

## 3. Algorithm, case by case

1. **Pole.** Site (−90°, 0°), ground 0, mast 2.
   - `horizon` with flat near and far terrain and exit `None`: all 720 values equal −0.086937° and are finite.
   - `sun_earth_track` over every table row (start t0, step 3600, count 43 825): all 9 × 43 825 values finite.
   - `timeline` on that flat horizon over the same span: all values finite, illumination in [0, 1].
   - value = the count of non-finite values; threshold 0.
2. **Flat pole ranges.** From case 1's track with mast 0: value = max(|Sun el|) and, in the notes, min and max Earth-centre el. Pass if Sun |el| ≤ 1.6° (E104, E99) and Earth |el| ≤ 7.2° (E102 plus 0.26° parallax). Expected from geometry [S]: the Earth between −7.13° and +6.61° if the latitude libration reaches ±6.87° at the mean distance; the asymmetry is the parallax.
3. **PSR floor** (native, `--release`).
   1. From `LPSR_80S_20MPP_ADJ.TIF` (E132) pick 20 flagged pixels that are at least 5 pixels from any unflagged pixel and inside the Shackleton ridge tile's pickable square (T06 step 6). Write their (x, y) in `docs/DATA-NOTES.md`. [U: the file's flag value; read it in T04.]
   2. Build an in-memory `Ephemeris` segment with the T02 builder: t0 = 1 262 304 000 (2010-01-01T00:00:00Z, U2), step 3600, 163 048 rows (18.6 × 365.25 × 24, rounded up; E36 rounds it to 165 000). Not written to `web/`.
   3. For each pixel centre: `horizon` at mast 0 (U15), then `timeline` with count 163 048. value = Σ of the visibility column over all 20; pass if it is exactly 0. Put the largest illumination seen (must also be 0) and the lowest Sun margin in the notes.
   4. `sample_size` = 20 × 163 048 = 3 260 960.
4. **Year boundary and table ends.**
   - Directions at 1 798 761 540 and 1 798 761 660 (2026-12-31T23:59 and 2027-01-01T00:01): Sun and Earth each move less than 0.02°, and the Sun's move equals its move over 23:57–23:59 within 0.0005° (P4).
   - `at(t0)` and `at(1 924 992 000)` succeed; `at(t0 − 1)` and `at(1 924 992 001)` are `OutOfTable`; `sun_earth_track` with a count that runs one step past the end is `OutOfTable` as a whole (P16).
5. **Far side.** Site (−60°, 180°), flat, ground 0, mast 0, every table row: value = max(Earth el + Earth radius); pass if < −20°. Expected about −22° [S]: the centre peaks at −23.37° for latitude libration alone and −23.07° with longitude libration at perigee, and the radius adds 0.95°–1.02°.
6. **DEM edge.** The synthetic case of T07 row 9 (far terrain ends 5 km from the site, exit off): `Err(OutOfCoverage)`. Also, with the real tiles loaded, `ground_height` and `horizon` at (−77.6158°, 90°), which is map point (377 000, 0), 5 km inside the far tile's edge: both `OutOfCoverage`. Never a horizon.
7. **Energy.** T09 rows 1 and 8, called through `core::survival`: value = |lethal_h − 7.050|, threshold 1e-9.

## 4. Edge cases and pitfalls

- **N1.** The far-side site is 60°S, not 85°S. At (−85°, 180°) the Earth really rises to +1.6°; that test is T-1d-1b in T05.
- **U15.** Barker's PSR map doesn't say whether the Sun is a disc or a point, or the observer's height. Hence "well inside" (5 px = 100 m from the edge) and mast 0. If a deep-interior pixel shows visibility, that is a finding about our model, the DEM vintage or the pixel choice: report it as a miss with the numbers (which pixel, how many steps, the Sun margin), then try 10 px. Don't quietly pick other pixels.
- **N2.** Case 3 needs `disc_fraction` to return exactly 0 (T08 row 3). A 1e-9 there turns into thousands of "visible" steps.
- **Release build.** Case 3 is 3.3 M timeline steps and 20 horizons: seconds in `--release`, minutes in debug (ARCHITECTURE §4).
- **Slip lever c** (PLAN-DRAFT §4): 5 PSR pixels instead of 20 if machine time is short. Say so in the notes; `sample_size` changes.
- **Honest rows.** A miss is written with its value. A case that can't run is "not run".
- Cases 1, 2, 4, 5 read the shipped table through the manifest (N3), not a table rebuilt in the test.

## 5. Test table

| # | Case | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | 1: flat horizon at the pole, H = 2 | 720 × −0.086937° | 1e-5° | [S] G8 |
| 2 | 1: non-finite values in track and timeline at the pole, all rows | 0 | exact | spec V5 case 1 |
| 3 | 2: Sun el at the flat pole, all rows | within ±1.6° | — | [U] E104; needs the table |
| 4 | 2: Earth-centre el at the flat pole | within ±7.2°; geometry gives −7.13° to +6.61° | — | [S] I4 for the bounds; [U] on the table |
| 5 | Geometry check for row 4, no table: pole observer on the sphere, Earth 384 400 km away at body latitude −6.87°, then +6.87° | el = +6.6128°, then −7.1270° | 1e-4° | [S] I4. A body south of the equator is above the south pole's horizon |
| 6 | 3: Σ visibility, 20 PSR pixels × 163 048 steps | 0 | exact | [U] needs data; E132 |
| 7 | 3: calendar → Unix for 2010-01-01T00:00Z; rows | 1 262 304 000; 163 048 | exact | [S] |
| 8 | 4: `at(1 767 225 600)`, `at(1 924 992 000)`; `at` one second outside each | Ok, Ok; `OutOfTable` × 2 | exact | [S] C1; P16 |
| 9 | 4: Sun's move over 23:59 → 00:01 | 0.015°–0.019°, and within 0.0005° of the previous 2 minutes | — | [U]; 0.508°/h × 2 min = 0.0169° [D] |
| 10 | 5: max(Earth el + radius) at (−60°, 180°) | < −20°; about −22° | — | [S] I1, I2 for the geometry; [U] on the table |
| 11 | 5: geometry check, no table: observer (−60°, 180°), Earth 384 400 km away at body latitude −6.87°, longitude 0° | centre el = −23.37° | 0.01° | [S] I1 |
| 12 | 6: synthetic edge | `Err(OutOfCoverage)` | exact | T07 row 9 |
| 13 | 6: `xy_to_latlon(377 000, 0)` | (−77.6158°, 90°) | 1e-4° | [S] A6 |
| 14 | 7: lethal time | 7.050 h | 1e-9 | [S] E4 |

## 6. Red/green order for /tdd

1. Rows 5, 7, 11, 13: plain geometry asserts, green at once if T01 is right.
2. Cases 7, 6 (synthetic), 1, 4: they reuse earlier tests; write the `CaseResult` wrapper and the scoreboard upsert with them.
3. Cases 2 and 5 on the shipped table.
4. Case 3 last, in `--release`, with the "not run" path tested first by pointing it at an empty directory.
5. Run `v5`, commit `scoreboard.json`, check the scoreboard page shows seven rows.

**Stop and escalate if** case 2 or 5 fails on the real table while rows 5 and 11 pass (the table or frame is wrong, not the geometry: go back to T02's step 8 checks); or case 3 shows visibility at more than one pixel after the 10 px retry (hand to `science-hard` with the per-pixel report; meanwhile write the miss). Don't widen ±1.6°, ±7.2° or −20°.
