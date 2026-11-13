# T05 — V1: 20 Horizons pairs through the table path

Ticket: [PLAN-DRAFT T05](../PLAN-DRAFT.md). Owner: Qi Cai. 1 h. hard-science. Blocked by T02. Needs the network once (fixture fetch); tests never call it.

## 1. Goal and done-criteria (from the ticket)

The V1 row of the scoreboard, with a real worst-case error.

- [ ] 20 site/time pairs as in §3 "V1 details": the pole, a point 0.1° from it, SP-01, IM-1, IM-2, a far-side point, region centres; times across 2026–2030 including each libration extreme found in the table and both sides of a year boundary; two sites at +5 km and −3 km altitude (P15).
- [ ] Requests follow §1c (`coord@301`, geodetic east longitude, latitude, height in km, quantity 4, UT, degrees, extra precision). The first response header confirms the 1737.400 km radius (U13).
- [ ] **V1: worst case ≤ 0.002° over the 20 pairs**, Sun and Earth, azimuth as Δaz·cos(el) with the 0/360 wrap handled (N8). The comparison goes through the shipped table and interpolation (N3).
- [ ] P3: turning aberration off moves the Sun by 0.005–0.006°. P4: positions are continuous from 2026-12-31T23:59 to 2027-01-01T00:01 UTC. P8: PA instead of ME differs by more than 0.01°.
- [ ] P9: at 85°S, 0°E the Earth is within 9° of azimuth 0, and the Sun's azimuth decreases at about 0.5°/h.
- [ ] T-1d-1: at (−60°, 180°) on a flat sphere the Earth's upper limb stays below −20° through 2026–2030. T-1d-1b: at (−85°, 180°) the Earth-centre elevation peaks between +1.0° and +1.7° and is negative more than half the time.
- [ ] `scoreboard.json` has the V1 entry. A miss is written as a miss with its number.

## 2. Files and signatures

`scripts/fetch-horizons.ts` (Bun; the only code that touches the network) · `crates/core/tests/fixtures/horizons/*.json` · `crates/core/tests/v1.rs` and `crates/core/tests/earth_sanity.rs` · `crates/pipeline/src/` (P3 and P8, which need ANISE) · `web/public/data/scoreboard.json`. `serde_json` becomes a dev-dependency of `core`.

No new public function in `core`. Test helpers (in the test file):

```rust
fn wrap180(deg: f64) -> f64;                    // into [−180, 180)
/// max(|Δel|, |wrap180(Δaz)·cos(el_ref)|) in degrees
fn v1_error(az: f64, el: f64, az_ref: f64, el_ref: f64) -> f64;
```

Fixture file, one per site and body (20 files):

```json
{ "site": {"lat_deg": -84.7906, "lon_deg": 29.1957, "h_km": 0.002},
  "body": "sun",
  "request": "<the full URL as sent>",
  "response": "<the raw result text>",
  "rows": [ {"t_unix_s": 1794614400, "az_deg": 0.0, "el_deg": 0.0} ] }
```

`h_km` is ground + mast in km. The test calls `sun_earth_track(lat, lon, ground_m = h_km·1000, mast_m = 0, t, 3600, 1)` on the **shipped table file** (N3).

## 3. Algorithm

1. **Find the libration extremes** (pipeline or a test with `--nocapture`): over all 43 825 rows compute the sub-Earth latitude asin(E_z/|E|) and longitude atan2(E_y, E_x). Take the four rows of max and min latitude and max and min longitude, and add 1800 s to each so the comparison falls mid-interval. Write the four Unix times in `docs/DATA-NOTES.md`.
2. **The 20 pairs.** Every fixed time is off the whole hour on purpose: the table is exact on the hour, and mid-interval is where interpolation error peaks.

   | Site | lat, E-lon, h (km) | Times |
   |---|---|---|
   | Pole | −90, 0, 0 | A, L1 |
   | Near pole | −89.9, 45, 0 | E |
   | SP-01 (E85) | −89.45, 222.69, 0 | B, L2 |
   | IM-1 (E72a) | −80.1276, 1.4367, 0 | F, L3 |
   | IM-2 (U8) | −84.7906, 29.1957, 0.002 | B (the tracer pair), C, D, L4 |
   | Far side | −60, 180, 0 | G, E |
   | Barker RoI 1 (E129, E130) | −89.438926, 222.9040, 0.005 | H, F |
   | 85°S, lon 0 | −85, 0, 0 | G |
   | 85°S, lon 180 | −85, 180, 0 | H, L1 |
   | High site (P15) | −85, 0, +5 | E |
   | Low site (P15) | −84.7906, 29.1957, −3 | G |

   | Time | UTC | Unix s |
   |---|---|---|
   | A | 2026-01-01 00:30 | 1 767 227 400 |
   | B | 2026-11-14 00:00 | 1 794 614 400 |
   | C | 2026-12-31 23:30 | 1 798 759 800 |
   | D | 2027-01-01 00:30 | 1 798 763 400 |
   | E | 2027-07-01 12:30 | 1 814 445 000 |
   | F | 2028-02-29 06:30 | 1 835 418 600 |
   | G | 2029-06-15 18:30 | 1 876 242 600 |
   | H | 2030-12-31 22:30 | 1 924 986 600 |
   | L1–L4 | from step 1: max lat, min lat, max lon, min lon, each + 1800 s | — |

3. **Request**, one per site and body, sequentially (SCIENCE-SPEC §1c; Horizons API doc de84f202:213–236, 404–421, 482–496, 1227–1261):

   ```
   https://ssd.jpl.nasa.gov/api/horizons.api?format=json
     &COMMAND='10'            ('399' for the Earth)
     &OBJ_DATA='NO'&MAKE_EPHEM='YES'&EPHEM_TYPE='OBSERVER'
     &CENTER='coord@301'&COORD_TYPE='GEODETIC'
     &SITE_COORD='29.1957,-84.7906,0.002'      E-lon deg, lat deg, h km (E48a, E48b)
     &TLIST='2026-11-14 00:00:00' '2026-12-31 23:30:00' …
     &TIME_TYPE='UT'                           (E54)
     &QUANTITIES='4'&ANG_FORMAT='DEG'&EXTRA_PREC='YES'&APPARENT='AIRLESS'&CSV_FORMAT='YES'
   ```

   URL-encode the quotes and spaces (`%27`, `%20`). Calendar strings avoid any Julian-date rounding. [U: the exact CSV column order; read the header line above `$$SOE` in the first response.]
4. **Parse**: the `result` string, lines between `$$SOE` and `$$EOE` (de84f202:1426–1442). Keep the raw text in the fixture.
5. **First response**: find the Moon's radius in the header and confirm 1737.400 km (E97, U13). Count the decimals of az/el and write the count in the scoreboard notes. If it prints 4 decimals, the judge's own rounding is 0.00005°, still 40 times inside the threshold.
6. **Compare** each row: ours through `sun_earth_track` on the shipped table; `v1_error` as in §2. V1 value = the maximum over the 20 pairs and both bodies. Pass if ≤ 0.002°.
7. **Sensitivity tests** (pipeline, need the kernels; each proves the fixture can see a bug):
   - P3: at pairs B, E and G, the angle between the Sun's direction with `Aberration::LT_S` and with `Aberration::LT` is between 0.005° and 0.006° (expect 0.0054–0.0059° [D]: 20.5″ scaled by the Moon's speed, 29–31 km/s).
   - P8: at the same pairs, the Earth's direction rotated with `MOON_PA_DE440_FRAME` differs from the ME one by more than 0.01° (expect 0.02–0.03° [U, from memory of the PA→ME angles]).
8. **Table tests** (core, on the shipped table, in CI):
   - P4: Sun and Earth directions at 1 798 761 540 and 1 798 761 660 (23:59 and 00:01) differ by less than 0.02° each, and the Sun's 2-minute change equals its change one hour earlier within 0.0005°.
   - P9: at (−85°, 0°), flat, on every row: Earth az within 9° of 0 (the review found extremes of 8.11° and 8.28°). Over any 24 rows the Sun's azimuth, unwrapped, falls by 11.0–13.5° (0.46–0.56°/h).
   - T-1d-1: at (−60°, 180°), every row: Earth el + Earth radius < −20°.
   - T-1d-1b: at (−85°, 180°): max Earth-centre el in [1.0°, 1.7°]; el < 0 on more than half the rows.
9. **Scoreboard**: one entry, keys as in ARCHITECTURE §3.5: `id` "V1", `judge` "JPL Horizons", `metric` "worst |Δel| or |Δaz·cos el| in degrees, Sun and Earth, through the shipped table", `threshold` 0.002, `value`, `pass`, `sample_size` 20, `notes` (decimals, which pair was worst), `commit`, `date`.

## 4. Edge cases and pitfalls

- **N3.** Comparing ANISE directly with Horizons proves nothing about the app. The test must read the `.bin`.
- **N8 / F5.** Wrap Δaz into [−180, 180) before multiplying by cos(el). Rows 1–3.
- **Pole.** At −90° the azimuth depends on the pole convention (north = +X, toward longitude 0). Horizons may print a different azimuth origin there, or refuse the site. If the pole pair's elevation agrees and only azimuth differs by a constant, report elevation for that pair and say so in the notes; the 0.1° site covers azimuth near the pole.
- **P15.** Height is km, longitude is east-positive, the centre is `coord@301` and not `500@301`. The ±5 km and −3 km sites exist to catch metres-for-km: a 1000× height error is a visible parallax change on the Earth.
- **P3.** 0.006° is what an aberration bug looks like (E77). A worst case near 0.0057° is that bug, not noise.
- **P4.** A worst case near 0.010° on Sun azimuth only is the UTC/TDB grid.
- **A miss stays a miss.** Don't drop the worst pair. Write the number.
- Horizons allows one request at a time. Fetch sequentially and commit the fixtures; rerunning tests must not refetch.

## 5. Test table

Rows 1–6 need no data. Rows 7–12 need the fixtures or the shipped table and can't be precomputed.

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | `v1_error(az 0.001°, el 60°, az_ref 359.999°, el_ref 60°)` | 0.001° | 1e-9° | [S] N8: Δaz = +0.002°, × cos 60° |
| 2 | `wrap180(359.5)`, `wrap180(−180)`, `wrap180(180)`, `wrap180(540.25)` | −0.5, −180, −180, −179.75 | 1e-12 | definition [−180, 180) |
| 3 | `v1_error(10°, 5.0010°, 10°, 5.0000°)`; `v1_error(10.0020°, 89°, 10°, 89°)` | 0.0010°; 0.0000349° | 1e-7° | [S] cos 89° = 0.0174524 |
| 4 | Unix → the eight times of §3 step 2 | as in the table | exact | [S] |
| 5 | SITE_COORD strings for IM-2 with a 2 m mast, the high site and the low site | `29.1957,-84.7906,0.002`, `0,-85,5`, `29.1957,-84.7906,-3` | exact | E48a–c, P15 |
| 6 | Parse a saved `result` text with two rows between `$$SOE` and `$$EOE` | two (t, az, el) rows; a text without `$$SOE` is an error | — | de84f202:1426–1442 |
| 7 | V1 over 20 pairs × 2 bodies | max ≤ 0.002°; expect about 0.0002° | — | [U] spec §3 V1 |
| 8 | P3 at B, E, G | 0.005°–0.006° | — | [U] spec P3 |
| 9 | P8 at B, E, G | > 0.01° | — | [U] spec P8 |
| 10 | P4 across 2027-01-01 | < 0.02°, smooth to 0.0005° | — | [U]; the Sun moves 0.017° in 2 minutes [D] |
| 11 | P9 at (−85°, 0°) | Earth az within 9° of 0; Sun az falls 11.0–13.5° per 24 h | — | [U]; review 1d |
| 12 | T-1d-1 and T-1d-1b | upper limb < −20°; peak in [1.0°, 1.7°], negative > half | — | [S] for one geometry: sub-Earth latitude −6.87° gives −23.37° at (−60°, 180°) and +1.61° at (−85°, 180°); [U] over the real table |

## 6. Red/green order for /tdd

1. Rows 1–3 (helpers), then rows 4–6 (the fetch script's pure parts: time formatting, SITE_COORD, parsing).
2. Step 1 (libration extremes), then fetch and commit the 20 fixtures. Do step 5 on the first response before fetching the rest.
3. Row 7 red or green on first run; this is the measurement. Write the scoreboard entry either way.
4. Rows 10–12 on the shipped table.
5. Rows 8–9 in `pipeline`.

**Stop and escalate if** row 7's worst case is above 0.002° and matches a signature in §4 (0.0057°: aberration; 0.010° in Sun azimuth: time scale; 0.02–0.03°: frame). Fix T02's builder, rebuild the table, rerun. If it is above 0.002° with no signature after 30 minutes, hand to `science-hard` with the per-pair table of Δaz·cos(el) and Δel. Never loosen 0.002°; a miss is reported as a miss.
