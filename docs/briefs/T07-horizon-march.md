# T07 — Terrain horizon in the browser

Ticket: [PLAN-DRAFT T07](../PLAN-DRAFT.md). Owner: Qi Cai. 1.75 h. hard-science. Blocked by T06 for the real tiles; every test in §5 runs on analytic terrain and needs no data, so the kernel can be written on Saturday morning for the benchmark (ARCHITECTURE §5 risk 1).

## 1. Goal and done-criteria (from the ticket)

Pick a site and see its real 720-azimuth skyline in the panorama, computed in the browser from the shipped tiles.

- [ ] T-1a-1: flat sphere, H = 2 m gives −0.0869° at every azimuth.
- [ ] T-1a-3: a 1000 m cone at 50 km, at an azimuth off the map axes, matches the closed form to 0.002° (P10: the march follows the great circle).
- [ ] N11: at the highest point of a tile, the horizon with the early exit on equals the horizon with it off.
- [ ] The observer's own pixel is skipped; an obstacle under 3 pixels away is reported through `dist_m`; the caller tests `dist_m < 60` (P13; ARCHITECTURE §3 dropped the separate flag).
- [ ] A site 5 km inside the far-field boundary returns a coverage error (V5 case 6).
- [ ] T-1a-4 report (native): shipped tiles against the full 20 m DEM at 5 points, at H = 0 and H = 2 m. Above 0.05°, open a follow-up for an 80 m tier or a wider near tile.
- [ ] T-1a-5 report: SP-01, H = 10 m, range 150 km against the tile edge.
- [ ] U18: the time of one native horizon and one in-browser horizon is written down. 50 ms is the target; over 2 s in the browser fires slip cut 3.
- [ ] The deployed panorama shows the real skyline at the IM-2 site, and the FIXTURE badge is off for the horizon.

## 2. Files and signatures

`crates/core/src/horizon.rs` (new), `crates/core/tests/horizon.rs` (new), `crates/core/src/dem.rs` (the `Dem` trait; create it here if T06 hasn't), `crates/pipeline/src/` (ablation run), `crates/wasm/src/lib.rs` (`Engine::horizon`), one switch in `web/src/lib/engine.ts`, `docs/DATA-NOTES.md`.

```rust
// dem.rs
pub trait Dem { fn height(&self, x: f64, y: f64) -> Result<f64, Error>; }   // metres above R; map metres in

// horizon.rs
pub const N_AZ: usize = 720;                       // 0.5° (E04, E24b, E64)
pub const NEAR_STEP_M: f64 = 20.0;                 // one pixel of the near tile
pub const NEAR_STEPS: usize = 350;                 // to 7 000 m (E35, E92)
pub const FAR_STEP_M: f64 = 240.0;
pub const FAR_STEPS: usize = 1562;                 // to 381 880 m
pub const MIN_RANGE_M: f64 = 200_000.0;            // E66, E100; decision D1 in README

pub struct Horizon { pub el: [f64; N_AZ], pub dist: [f64; N_AZ] }     // radians, metres
impl Horizon {
    pub fn to_api(&self) -> Vec<f64>;                        // 1440: el_deg[720] then dist_m[720]
    pub fn from_api(a: &[f64]) -> Result<Horizon, Error>;    // BadInput unless len == 1440 and all finite
}

/// h_max_far: the largest height in the far tile, or None to switch the early exit off.
pub fn horizon<N: Dem, F: Dem>(near: &N, far: &F, h_max_far: Option<f64>,
                               lat: f64, lon: f64, mast_m: f64) -> Result<Horizon, Error>;
```

Index i is azimuth i × 0.5°, clockwise from north. Generic parameters (not `dyn`) so the sampler inlines into the loop.

WASM, frozen: `Engine::horizon(&self, lat_deg, lon_deg, mast_m) -> Result<Vec<f64>, JsError>`. It calls `core::horizon` with the loaded tiles and `Some(h_max)` found in `load_far`, then `to_api()`. `NOT_LOADED` if either tile is missing.

## 3. Algorithm (ARCHITECTURE §2.2, SCIENCE-SPEC §1a)

Setup, once per call:

1. `û_s = latlon_to_unit(lat, lon)`, `(x_s, y_s) = latlon_to_xy(lat, lon)`, `frame = frame_at(û_s)` (T01).
2. `h_s = near.height(x_s, y_s)?`, `r_o = R + h_s + mast_m`. This is the same code `ground_height` uses.
3. Step table, 1912 entries: `s_j = 20·j` for j = 1…350, then `s_j = 7000 + 240·j` for j = 1…1562. For each: `θ = s/R`, `sinθ`, `versθ = 2·sin²(θ/2)`. Building it per call costs about 4000 trig calls, which is nothing; don't cache it.
4. `r_max = R + h_max_far`. The early exit is allowed only if `h_max_far` is `Some` and `r_max > r_o` (F1).

Per azimuth i:

5. `d̂ = north·cos(az) + east·sin(az)`, az = i × 0.5°.
6. For each step j, near then far:
   1. Far steps only, if the exit is allowed and a best sample exists: `bn = (r_max − r_o) − r_max·versθ_j`, `bd = r_max·sinθ_j`. If `bn·best_den < best_num·bd`, stop this azimuth.
   2. `û = û_s·(1 − versθ_j) + d̂·sinθ_j`. This is the great circle (P10, E02).
   3. `(x, y) = unit_to_xy(û)`.
   4. `h = near.height(x, y)?` for near steps. For far steps `far.height(x, y)`: on `OutOfCoverage` with `s_j ≥ MIN_RANGE_M` stop this azimuth normally; with `s_j < MIN_RANGE_M` return the error (P5, D1). `NoData` is always returned as an error.
   5. `r_t = R + h`, `num = (r_t − r_o) − r_t·versθ_j`, `den = r_t·sinθ_j`. This is `r_t·cos θ − r_o` without cancellation.
   6. If there is no best yet, or `num·best_den > best_num·den`: keep `(num, den, s_j)`. Both `den` are positive, so cross-multiplying compares tan(el).
7. `el[i] = atan2(best_num, best_den)`, `dist[i] = best s`.

The first sample is at 20 m, so the observer's own pixel is never an obstacle. No trig, square root or `atan` inside step 6.

## 4. Edge cases and pitfalls

- **P10.** Never step along a straight map line. Row 4 fails by 0.01° if you do.
- **Small-angle form.** `(h − h_s − H)/s − s/2R` is 0.0006° off at 50 km. Use step 6.5.
- **F1 / N11.** The exit is skipped when `r_max ≤ r_o`. The exit bound uses the far tile's maximum, so it is applied to far steps only; near-tile heights can exceed it.
- **P5.** The near march can't leave coverage if the site is 7 km inside the near tile. If it does, the `?` returns `OutOfCoverage`; never substitute 0 m.
- **D1.** A ray that reaches the far tile's edge after 200 km ends normally. Before 200 km it is an error. From the pole the edge is 382 km away; from the IM-2 site the north edge is 244 km away.
- **P13.** Don't filter near obstacles. `dist < 60` is the flag and the UI shows it.
- **N7.** f64 only.
- **Cancellation.** `r_t·cos θ − r_o` with `cos` computed directly loses about 6 digits at 20 m. Use `versθ`.
- **Performance.** `opt-level = 3` is already set. If the browser time is over 50 ms, follow the ladder in ARCHITECTURE §2.2 "If it misses", in order.

## 5. Test table

Analytic terrain is a closure wrapped in a small test-only `Dem`. "gc distance" is R times the angle between the sample's unit vector (`xy_to_unit(x, y)`) and a reference unit vector. Site for rows 1–8 unless stated: (−85°, 45°E), ground 0.

| # | Terrain; site; H; exit | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | Flat (h = 0); H = 2; exit None | every el = −0.086937°; every dist = 2 640 m | 1e-5°; dist ± 20 m | [S] G1; T-1a-1. The tangent point is √(2RH) = 2 636 m |
| 2 | Flat; H = 0.1; None | every el = −0.019442°; dist 580 m | 1e-5°; ± 20 m | [S] G2; N7. The continuous dip is −0.019440° |
| 3 | Flat; H = 0; None | every el = −0.00032978°; dist 20 m | 1e-7° | [S] G3. The first sample wins: el = −s/2R |
| 4 | Cone: 1000 m high, base radius 2 000 m, peak at gc distance 50 200 m along azimuth 37° (bin 74; far step j = 180). h = max(0, 1000·(1 − D/2000)), D = gc distance to the peak. H = 2; None | el[74] = 0.310766°, dist[74] = 50 200 m; el[0] = −0.086937° | 0.002° (ticket); the script agrees to 1e-5° | [S] G4, B6; T-1a-3 |
| 5 | Row 4 with `Some(1000.0)` | identical to row 4 at all 720 azimuths | 1e-12 rad | early exit never changes a result |
| 6 | Mesa: h = 100 where gc distance from the site ≥ 990 m, else 0; H = 2; None | every el = 5.58046°, dist 1 000 m | 1e-4° | [S] G6, B7. Near tier and `atan2` |
| 7 | Summit: h = max(0, 1000·(1 − D/5000)), D = gc distance from the site (ground 1000 m); H = 2. Run with `Some(1000.0)` and with None | every el = −1.94544°; dist 59 080 m; the two runs equal | 1e-4°; dist ± 240 m; equal to 1e-12 rad | [S] G7, B8; N11. Here r_max < r_o, so the exit must stay off |
| 8 | Flat; site (−90°, 0°); H = 2; None | every el = −0.086937°; nothing NaN | 1e-5° | [S] G8; V5 case 1 |
| 9 | Near: flat. Far: flat inside \|x − x_s\| ≤ 5 000 and \|y − y_s\| ≤ 5 000, `OutOfCoverage` outside; H = 2; None | `Err(OutOfCoverage)` | exact | V5 case 6; D1. With H = 2 and h_max = 0 the exit can't hide the edge |
| 10 | Row 9 with the far box at ± 250 000 m | `Ok`, every el = −0.086937° | 1e-5° | D1: leaving after 200 km is normal |
| 11 | Far returns `NoData` at one sample 30 km out on azimuth 0 | `Err(NoData)` | exact | P5 |
| 12 | `Horizon::from_api(to_api(h))` on row 4's result; `from_api` of 1439 values | round trip to 1e-12°; `Err(BadInput)` | — | layout: `el_deg[720]` then `dist_m[720]` |

Row 4, why it bites: a marcher that walks a straight map line from this site passes 19 m beside the peak and returns el[74] = 0.29222°, which is 0.0185° low [S G5]. The neighbours el[73] and el[75] are both 0.061025° [S]; they are not asserted, but a large asymmetry between them means the azimuth is off.

## 6. Red/green order for /tdd

1. Row 1 red. Write setup and the near loop only (no far tier, no exit). Rows 1–3 and 6 go green.
2. Row 4 red. Add the far loop. Rows 4 and 8 green.
3. Rows 9–11 red. Add the coverage rule D1 and error propagation.
4. Rows 5 and 7 red (the exit doesn't exist yet, so `Some` is ignored and they pass trivially: write the exit, then confirm they still pass, then break the `r_max > r_o` guard on purpose and confirm that row 7 stays green or goes red; either way note it in the PR).
5. Row 12, then `Engine::horizon`, then flip the adapter switch.
6. Native timing: `cargo run -p penumbra-pipeline --release` on one site, 20 repeats, median. Browser timing: `performance.now()` around `engine.horizon` on the deployed page, 20 sites. Write both in `docs/DATA-NOTES.md` (U18).
7. Reports (native, need `data/`):
   - **T-1a-4**: 5 points (the IM-2 site, SP-01, Barker RoI 4, one PSR-floor pixel, one mid-slope point), H = 0 and H = 2. Reference: the same march with 20 m steps out to 50 km from an f32 crop of `LDEM_80S_20MPP_ADJ.TIF` (5000² px, 100 MB), then the far tile. Report the max |Δel| over 720 azimuths per point.
   - **T-1a-5**: SP-01, H = 10 m. Run once as is and once with the far march cut at 150 km. Report the max |Δel| and, after T08, the change in longest night.

**Stop and escalate if** rows 1–4 pass natively but the deployed skyline at the IM-2 site is flat or all NaN (a tile metadata problem: go back to T06, don't touch the marcher); or T-1a-4 is above 0.05° (open the 80 m tier follow-up, don't tune); or the browser time is over 2 s after `opt-level = 3` (gate G2, slip cut 3). After two failed attempts at row 4, hand to `science-hard` with this brief.
