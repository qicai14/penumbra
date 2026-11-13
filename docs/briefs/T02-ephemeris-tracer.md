# T02 — Tracer: real Sun and Earth az/el for one site and date on the deployed page

Ticket: [PLAN-DRAFT T02](../PLAN-DRAFT.md). Owner: Qi Cai. 1.5 h. hard-science. Blocked by T00, T01 and the three kernels in `data/`. Must be merged and deployed by Sat 12:00.

## 1. Goal and done-criteria (from the ticket)

The deployed page shows the Sun's and the Earth's azimuth and elevation for the IM-2 site (84.7906°S, 29.1957°E, mast height 2 m) on 2026-11-14, computed in the browser by `core` from the real hourly table. Ground height is 0 m and the horizon is the flat-sphere dip, both labelled "no terrain yet".

- [ ] First step: confirm `LT_S` exists in ANISE 0.10.6 (U14). If it does not, stop and take the fallback to `science-hard`.
- [ ] The table is on whole UTC hours built with hifitime; positions are apparent (light time and stellar aberration), Moon-centred, translated in J2000 and then rotated to MOON_ME (§1c steps 1–2).
- [ ] T-1c-1: table + interpolation against native ANISE at 1000 random instants, max error under 0.001°.
- [ ] One Horizons pair for this site and date agrees within 0.01° for both bodies, azimuth compared as Δaz·cos(el). The request text is stored with the fixture (P15).
- [ ] A time outside the table returns an error (P16).
- [ ] The deployed URL shows the same four numbers as the native run, and the FIXTURE badge is off for this view. If T03 has not merged, a plain text readout is enough.

## 2. Files and signatures

`crates/core/src/eph.rs` and `crates/core/tests/eph.rs` (new) · `crates/pipeline/src/` (table builder, T-1c-1) · `crates/wasm/src/lib.rs` (`Engine`, `load_ephemeris`, `sun_earth_track`) · `web/public/data/eph-main.<hash>.bin` and `web/public/data/manifest.json` · one switch in `web/src/lib/engine.ts` · `crates/core/tests/fixtures/horizons/tracer.json`.

```rust
// core/src/eph.rs
pub const R_SUN_KM: f64 = 696_000.0;        // E117
pub const R_EARTH_KM: f64 = 6378.137;       // E98

pub struct Segment { pub t0_unix_s: f64, pub step_s: f64, pub data: Vec<f64> }   // 6 per row
pub struct Ephemeris { segments: Vec<Segment> }
impl Ephemeris {
    pub fn new() -> Self;
    /// BadInput unless data.len() is a multiple of 6, at least 12, step_s > 0 and all finite.
    pub fn add(&mut self, t0_unix_s: f64, step_s: f64, data: Vec<f64>) -> Result<(), Error>;
    /// [sun_x, sun_y, sun_z, earth_x, earth_y, earth_z] in km, MOON_ME, Moon-centred, apparent.
    pub fn at(&self, t_unix_s: f64) -> Result<[f64; 6], Error>;   // NotLoaded, OutOfTable, BadInput
}

pub struct Site { pub lat: f64, pub lon: f64, pub ground_m: f64, pub mast_m: f64 }   // radians, metres

/// 9 × count, column-major, in this order and these units (ARCHITECTURE §3.2):
/// t_unix_s, sun_az_deg, sun_el_deg, sun_radius_deg, sun_dist_km,
/// earth_az_deg, earth_el_deg, earth_radius_deg, earth_dist_km
pub fn sun_earth_track(eph: &Ephemeris, site: &Site, start_unix_s: f64, step_s: f64, count: u32)
    -> Result<Vec<f64>, Error>;
```

WASM, frozen:

```rust
pub fn load_ephemeris(&mut self, t0_unix_s: f64, step_s: f64, data: Box<[f64]>) -> Result<(), JsError>;
pub fn sun_earth_track(&self, lat_deg: f64, lon_deg: f64, ground_m: f64, mast_m: f64,
                       start_unix_s: f64, step_s: f64, count: u32) -> Result<Vec<f64>, JsError>;
```

File: `f64` little-endian, row-major, 6 per row, no header. Main segment: t0 = 1 767 225 600 (2026-01-01T00:00:00Z), step 3600 s, 43 825 rows, 2 103 600 bytes. Manifest entry as in ARCHITECTURE §3.5.

## 3. Algorithm

### 3a. Table builder (pipeline, native; SCIENCE-SPEC §1c steps 1–2)

Library facts below were read from docs.rs for ANISE 0.10.6 on Nov 11. They were not compiled. Each [U] must be checked in the first 20 minutes.

1. Add `anise = { version = "0.10.6", default-features = false }` to `crates/pipeline`. The default features (`analysis`, `metaload`) pull in a `ureq` downloader, `rayon` and more; none is needed. [U: that it builds on Windows without them.]
2. U14: `anise::astro::Aberration::LT_S` exists in 0.10.6. Its type is `Option<Aberration>`; the doc line is "Unconverged light time correction in reception mode with stellar aberration". This settles the ticket's first step unless the build disagrees. `CN_S` (converged) also exists; the difference is far below 1e-6° here, so keep `LT_S` as the spec says.
3. Load the three kernels into one `Almanac`: `de440.bsp`, `moon_pa_de440_200625.bpc`, `moon_fk_de440.epa` (E44: the PCK needs the frame kernel). `Almanac::load(self, path: &str)` guesses the file type; `with_euler_parameters` attaches a parsed `.epa`. [U: that `load` accepts the `.epa`.]
4. For row k = 0 … rows − 1: `epoch = Epoch::from_unix_seconds(t0 + 3600·k)`. Building each epoch from Unix seconds keeps rows on whole UTC hours even across a leap second (U12, P4, N3). Never add 69 s by hand.
5. `sun = almanac.translate(SUN_J2000, MOON_J2000, epoch, Aberration::LT_S)?` and the same with `EARTH_J2000`. Argument order is (target, observer, epoch, correction). The result is a Cartesian state in km in J2000 axes. Frame constants are in `anise::constants::frames`.
6. `dcm = almanac.rotate(MOON_J2000, MOON_ME_DE440_ME421_FRAME, epoch)?` (E40, E43), then apply its 3 × 3 rotation to both position vectors. [U: the exact way to apply a `DCM` to a 3-vector; read `anise::math::rotation::DCM`. U: whether the generic `MOON_ME_FRAME` is the same frame with this `.epa`.] Rotating as a separate step at the observer's epoch is the spec's rule (U14). Do not use `IAU_MOON_FRAME` (E42a) or a PA frame (P8).
7. Write the six numbers of each row as little-endian f64. Name the file with the first 6 hex characters of its SHA-256 and write the manifest entry.
8. Sanity checks on the finished table, before any Horizons comparison (they localise a frame or sign error; all [D] from E101, E102, E104):
   - |Sun| is between 146.6e6 and 152.6e6 km; |Earth| is between 356 000 and 407 000 km.
   - Sub-Earth point: latitude asin(E_z/|E|) within ±6.9°, longitude atan2(E_y, E_x) within ±8.2°, on every row. This proves the frame is ME with +X toward the Earth.
   - Sub-solar latitude asin(S_z/|S|) within ±1.6° on every row.
   - Sub-solar longitude atan2(S_y, S_x) decreases by 0.49–0.53° per row (E107).

### 3b. Lookup and interpolation (core; ARCHITECTURE §2.4)

9. Find the segment with `t0 ≤ t ≤ t0 + (rows − 1)·step` (linear scan; three segments at most). None: `OutOfTable`. No segment loaded: `NotLoaded`. Non-finite t: `BadInput`.
10. `u = (t − t0)/step`, `k = floor(u)`, `a = u − k`. If `k == rows − 1` return row k (a is 0 there). Otherwise `r = (1 − a)·row[k] + a·row[k + 1]`, component by component.

### 3c. Topocentric az/el (core; SCIENCE-SPEC §1c steps 3–4)

11. `û = latlon_to_unit(lat, lon)`, `frame = frame_at(û)`, site vector `s = ((R + ground_m + mast_m)/1000)·û` in km.
12. For each body: `(az, el) = az_el(&frame, s, r_body)`, `dist = |r_body − s|`, `radius = asin(R_body_km / dist)`. No refraction (E53).
13. Fill the nine columns, converting angles to degrees. `count == 0`, `step_s ≤ 0` or a non-finite argument is `BadInput`. Any time outside the table fails the whole call with `OutOfTable` (P16).

### 3d. Tracer page

14. `Engine::load_ephemeris` copies the array into an `Ephemeris`. The adapter fetches the manifest, then the table (`new Float64Array(buffer)`; little-endian platforms only, one comment).
15. Call `sun_earth_track(−84.7906, 29.1957, 0, 2, 1794614400, 3600, 24)` and show row 0 as four numbers, plus `flat_dip(2)` = −0.0869° labelled "no terrain yet".
16. Fetch one Horizons pair for this site and time with the request of [T05 §3](T05-v1-horizons.md) (`SITE_COORD='29.1957,-84.7906,0.002'`, `TLIST='2461358.5'`), store the request text and the response in the fixture, and assert agreement within 0.01°.

## 4. Edge cases and pitfalls

- **P4 / N3.** The grid is UTC. A table built on TDB epochs and read as UTC is 69 s off, 0.010° of Sun azimuth, and native tests would still pass. Step 4 prevents it; the Horizons pair in step 16 detects it because it goes through the table.
- **P3.** Light time without stellar aberration (or the reverse) is a 0.006° error on the Earth; neither is 0.0057° on the Sun. `LT_S` applies both.
- **P8.** PA instead of ME moves directions by 0.02–0.03° [U, from memory of the DE440 PA→ME angles, about 68″ and 79″]. The sub-Earth check in step 8 doesn't see it; the Horizons pair does.
- **P16.** The last row is valid at exactly its own time and nowhere after it.
- **P9.** With T01 right, the Earth at the IM-2 site sits near azimuth 331° (the Earth is over longitude 0, west of the site's meridian).
- **U12.** If `from_unix_seconds` is missing from the re-exported hifitime, use `Epoch::from_gregorian_utc(y, m, d, h, 0, 0, 0)`; don't add hours as a `Duration` across years without checking a leap second.
- The table holds Moon-centred vectors. The parallax (0.26° for the Earth) comes from subtracting the site vector in step 12; never normalise the table vectors before that.
- Linear interpolation shortens |r| by 1e-5 mid-interval. That changes `sun_dist_km` by 1500 km and the flux by 2e-5: harmless, don't "fix" it.

## 5. Test table

Rows 1–10 need no data and run in CI. Site for rows 7–9: (−85°, 0°), ground 0, mast 0.

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | Table t0 = 1000, step 3600, rows (1,2,3,4,5,6), (3,6,9,12,15,18), (0,0,0,0,0,0). `at(1900)` | (1.5, 3, 4.5, 6, 7.5, 9) | 1e-12 | step 10, a = 0.25 |
| 2 | Same table, `at(1000)`, `at(4600)`, `at(8200)` | row 0, row 1, row 2 exactly | 0 | step 10; last row valid at its own time |
| 3 | Same table, `at(999)`, `at(8201)`, `at(NaN)` | `OutOfTable`, `OutOfTable`, `BadInput` | exact | P16 |
| 4 | Empty `Ephemeris`, `at(0)`; `add(0, 3600, vec![0.0; 7])`; `add(0, 0, vec![0.0; 12])` | `NotLoaded`; `BadInput`; `BadInput` | exact | step 9 |
| 5 | Two segments: the row-1 table at t0 = 1000 and a second at t0 = 100 000 whose rows are all (7,7,7,7,7,7). `at(101 000)`; `at(50 000)` | (7,…); `OutOfTable` | exact | replay segments (T15) |
| 6 | Calendar → Unix: 2026-01-01T00:00Z; 2026-11-14T00:00Z; 2027-01-01T00:00Z; 2031-01-01T00:00Z; 2025-03-06T17:28Z | 1 767 225 600; 1 794 614 400 (row 7608; JD 2461358.5); 1 798 761 600 (row 8760); 1 924 992 000 (row 43 824, so 43 825 rows); 1 741 282 080 | exact | [S] C1 |
| 7 | Constant table (two equal rows): Sun = 149 597 870.7 km × (cos(−1°)cos 30°, cos(−1°)sin 30°, sin(−1°)); Earth = (384 400, 0, 0). `sun_earth_track(…, count = 3)` | Sun az 30.13857°, el 5.32713°, radius 0.266568°, dist 149 597 709.4 km; Earth az 0°, el 4.74192°, radius 0.951086°, dist 384 252.47 km; the same at all 3 times | 1e-5°; 0.1 km (Sun), 0.01 km (Earth) | [R] review 1c; [S] B3, B4 |
| 8 | Row 7's result | length 27; `[0..3]` = start, start + step, start + 2·step; `[3..6]` Sun az; … `[24..27]` Earth dist | exact | ARCHITECTURE §3.2 layout |
| 9 | Row 7 with `count = 0`; with a start one second before the table | `BadInput`; `OutOfTable` | exact | step 13 |
| 10 | asin(696 000 / 149 597 870.7) | 0.26657° | 1e-5° | [S] D3; spec §1b |
| 11 | Real table, every row: the four checks of step 8 | all hold | as stated | [U] needs the kernels; bounds from E101, E102, E104, E107 |
| 12 | T-1c-1: 1000 seeded random times in [t0, t0 + 43 824·3600]. Angle between the interpolated and the direct ANISE direction, Sun and Earth | max < 0.001°; expect about 0.0001° (F4) | — | [U] needs the kernels |
| 13 | Tracer pair, IM-2 site, mast 2 m, t = 1 794 614 400, through the table | Horizons az and el within 0.01°, Δaz·cos(el), both bodies | 0.01° | [U] needs the fixture. Plausible ranges from geometry [S]: Sun el within ±6.75°; Earth el between −3.1° and +11.6°; Earth az between 322° and 340° (sub-Earth point within ±6.9° latitude, ±8.2° longitude, at 384 400 km) |
| 14 | The native run and the deployed page, same call | the same four numbers | 1e-9° | ARCHITECTURE §4 (maths libraries differ) |

## 6. Red/green order for /tdd

1. Twenty-minute spike first, no tests: steps 1–6 for one epoch, print both vectors, run the step 8 checks by eye. This retires ARCHITECTURE risk 2.
2. Rows 1–5 red, then steps 9–10.
3. Row 6 as plain asserts in the test file.
4. Rows 7–10 red, then steps 11–13.
5. Table builder to a file (steps 4–7), then rows 11–12 as `pipeline` tests that print "kernels missing, skipped" when `data/` is empty.
6. WASM exports, manifest, adapter switch, readout. Then row 13 (fetch the fixture once with `scripts/`, commit it) and row 14.

**Stop and escalate if** the spike's sub-Earth point is outside ±9° (wrong frame or wrong body order: hand to `science-hard` with the printed vectors); or ANISE doesn't build on Windows after 20 minutes (build the table on the other laptop or in CI, as ARCHITECTURE risk 2 says); or row 13 misses by 0.005–0.011° (that size is aberration or the UTC grid: recheck steps 4–5 before anything else); or row 13 misses by 0.02–0.03° (PA frame, step 6).
