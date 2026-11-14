# T08 — Sun-disc fraction, Earth margin and timelines

Ticket: [PLAN-DRAFT T08](../PLAN-DRAFT.md). Owner: Qi Cai. 1.25 h. Blocked by T02 and T07. Every test in §5 except row 16 runs without data.

## 1. Goal and done-criteria (from the ticket)

The 30-day, 10-minute timeline for the picked site from real data: illumination, visibility, Earth margin and contact.

- [ ] T-1b-1: a flat horizon at u·ρ gives f = (acos u − u·√(1 − u²)) / π within 2e-3 using 32 midpoint strips; u = 0 gives 0.5.
- [ ] T-1b-2: a hidden disc gives exactly 0. N2: a clear disc gives exactly 1 (normalised by Σ 2w).
- [ ] ρ is recomputed at each step from the Sun's distance and stays within 0.261°–0.272° (Moon-centred; widened Nov 14). **(See decision D3: assert 0.2614°–0.2719°.)**
- [ ] P1: visibility ≥ illumination at every step, and the two outputs have those names.
- [ ] N8: the margin is continuous as the Sun crosses azimuth 0/360.
- [ ] Earth: margin = el_earth − el_hor(az_earth); contact = margin ≥ 2° by default, labelled "assumed". "Any fraction of the Earth visible" is a separate, differently named function (P14).
- [ ] The deployed timelines for the IM-2 site read real data.

## 2. Files and signatures

`crates/core/src/disc.rs`, `crates/core/src/timeline.rs`, `crates/core/tests/disc.rs`, `crates/core/tests/timeline.rs` (new) · `crates/wasm/src/lib.rs` (`Engine::timeline`) · one switch in `web/src/lib/engine.ts`.

```rust
// disc.rs — radians
pub const N_STRIPS: usize = 32;
/// Horizon elevation at any azimuth: linear between the 720 nodes, wrapping at 2π (E05, N8).
pub fn horizon_at(h: &Horizon, az: f64) -> f64;
/// Fraction of a disc of angular radius rho, centred at (az, el), above the horizon. In [0, 1];
/// exactly 0 when hidden, exactly 1 when clear.
pub fn disc_fraction(h: &Horizon, az: f64, el: f64, rho: f64) -> f64;

// timeline.rs
/// P14: the PGDA "any fraction of the Earth disc" rule, for V2-Earth only. Not contact.
pub fn earth_any_visible(h: &Horizon, az: f64, el: f64, rho_earth: f64) -> bool;   // disc_fraction > 0

/// 10 × count, column-major, in this order and these units (ARCHITECTURE §3.2):
/// t_unix_s, sun_az_deg, sun_el_deg, illumination, visibility, earth_az_deg, earth_el_deg,
/// earth_margin_deg, contact, sun_dist_km
pub fn timeline(eph: &Ephemeris, horizon: &Horizon, site: &Site, start_unix_s: f64, step_s: f64,
                count: u32, earth_margin: f64 /* radians */) -> Result<Vec<f64>, Error>;
```

WASM, frozen: `Engine::timeline(&self, horizon: &[f64], lat_deg, lon_deg, ground_m, mast_m, start_unix_s, step_s, count: u32, earth_margin_deg) -> Result<Vec<f64>, JsError>`. It runs `Horizon::from_api(horizon)?` (so a 1439-value array is `BAD_INPUT`) and converts the margin to radians. The horizon is an argument, never engine state, so T15 can pass a raised one.

Column meanings: `illumination` = f in [0, 1]; `visibility` = 1 if f > 0 else 0; `earth_margin_deg` = el_earth − el_hor(az_earth), the margin itself and not the threshold; `contact` = 1 if that margin ≥ the threshold.

## 3. Algorithm

### 3a. `horizon_at`

1. `a = az.rem_euclid(2π) / (π/360)` (the bin width is 0.5°). `i = floor(a)`, `f = a − i`. Return `(1 − f)·el[i % 720] + f·el[(i + 1) % 720]`. The `% 720` on `i` matters: `rem_euclid` can return exactly 2π for a tiny negative input.

### 3b. `disc_fraction` (SCIENCE-SPEC §1b with F18; ARCHITECTURE §2.3)

2. `half = rho / cos(el)`, the disc's half-width in azimuth (0.27° for the Sun, under one bin). If `cos(el) < 1e-6`, use `half = π`.
3. Quick test over the nodes the disc can touch: for node index n from `floor((az − half)/bin)` to `ceil((az + half)/bin)` (at most 720 of them, each taken `rem_euclid(720)`), find `lo` and `hi` of `el[n]`. If `el − rho ≥ hi` return 1.0. If `el + rho ≤ lo` return 0.0. This is exact, because the interpolated horizon never leaves the range of its nodes.
4. Otherwise, for i = 0 … 31: `x = −1 + (2i + 1)/32`, `w = rho·sqrt(1 − x²)`, `g = horizon_at(az + rho·x/cos(el)) − el`, `num += clamp(w − g, 0, 2w)`, `den += 2w`. Return `num/den`.

The normaliser is the same quadrature (Σ 2w), not πρ². With πρ² a clear Sun reads 1.0017 (F18, N2).

### 3c. `timeline`

5. Validate: `count ≥ 1`, `step_s > 0`, all arguments finite, else `BadInput`. Build `û`, the frame and the site vector in km once (as T02 step 11).
6. For k = 0 … count − 1, `t = start + k·step`:
   1. `row = eph.at(t)?` (`OutOfTable` fails the whole call, P16).
   2. Sun: `(az, el)`, `dist = |r_sun − s|`, `rho = asin(R_SUN_KM / dist)` (E117; recomputed every step). `f = disc_fraction(horizon, az, el, rho)`.
   3. Earth: `(az_e, el_e)`. `margin = el_e − horizon_at(horizon, az_e)` (SCIENCE-SPEC §1d). `contact = margin ≥ earth_margin`.
   4. Write the ten columns; angles in degrees.

The default threshold, 2° (E62), is the adapter's and the preset's business; `core` has no default.

## 4. Edge cases and pitfalls

- **P1.** Illumination and visibility are different columns and different words (glossary). V2 compares visibility. Never output one under the other's name.
- **N2 / T-1b-2.** "Exactly" means `== 0.0` and `== 1.0`. The PSR test in V5 case 3 relies on exact zeros over 163 000 steps.
- **N8.** All azimuth arithmetic goes through `horizon_at`, which wraps. Don't index `el[]` anywhere else.
- **P14.** `contact` uses the Earth's centre plus the margin. `earth_any_visible` uses the limb. They differ by up to 0.95° + 2°. Keep both names.
- **D3.** The ticket's original ρ band (0.262°–0.271°) was derived for the Earth's distance from the Sun; the ticket now reads 0.261°–0.272° (Moon-centred, widened Nov 14). From the Moon the range is 0.26149°–0.27185° [S]. Assert 0.2614°–0.2719° and print the observed min and max in the PR. As built, `crates/core/tests/timeline.rs` asserts the rounded 0.261..=0.272: the science check (checkpoint Nov 14, S1) found no plausible bug that lands only in the 0.0004° slivers, so the test was left as is.
- **No limb darkening** (E26 is Barker's choice, not ours). f is the geometric fraction (E08). It goes on the Limitations list.
- A sliver of disc between two strip midpoints can read f = 0 while a hair is visible. The threshold error is 0.0005·ρ, or 0.00013°. Leave it.
- Don't test `margin ≥ 2°` with a horizon built as `el − 2°`: the subtraction can land on either side of 2.0. Row 11 uses thresholds on both sides instead.

## 5. Test table

"Flat at X" means all 720 nodes equal X. ρ☉ = 0.2666° unless stated. Rows 5–8 use az = 123.4°, el = 1.0°.

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | Nodes el[719] = 1°, el[0] = 3°, el[1] = 0°, rest 0°. `horizon_at` at 359.75°, −0.25°, 719.75°, 0°, 0.25° | 2.0°, 2.0°, 2.0°, 3.0°, 1.5° | 1e-9° | N8; step 1 |
| 2 | Row 1's horizon, `horizon_at(−1e-18 rad)` | 3.0°, no panic | 1e-9° | `rem_euclid` returning 2π |
| 3 | Flat at el + ρ and at el + 1.5ρ | exactly 0.0 | `==` | T-1b-2 |
| 4 | Flat at el − ρ and at el − 1.5ρ | exactly 1.0 | `==` | N2 |
| 5 | Flat at el (u = 0) | 0.5 | 1e-12 | T-1b-1; the strips are symmetric |
| 6 | Flat at el + u·ρ for u = 0.5, −0.5, 0.9, 0.3 | 0.195309, 0.804691, 0.018711, 0.311403 | 1e-5 | [S] D2, the 32-strip values |
| 7 | Row 6 against the closed form (acos u − u·√(1 − u²))/π | 0.19550, 0.80450, 0.01869, 0.31192; error 1.9e-4, 1.9e-4, 1.8e-5, 5.2e-4 | 2e-3 (ticket) | [R] review 1b; [S] D1, D2 |
| 8 | 1000 seeded random (horizon nodes in [−2°, 4°], az, el in [−2°, 4°]) | 0 ≤ f ≤ 1; visibility ≥ illumination | — | P1 |
| 9 | Sloped horizon through the disc centre: az = 10.25°, el = 1.0°; nodes 19, 20, 21, 22 = −0.5°, 0.5°, 1.5°, 2.5° | 0.5 | 1e-12 | [S] D4a: a line through the centre halves the disc |
| 10 | The same across the wrap: az = 0°, el = 1°; nodes 718, 719, 0, 1, 2 = −1°, 0°, 1°, 2°, 3° | 0.5 | 1e-12 | [S] D4b; N8 |
| 11 | Constant table, Earth at (384 400, 0, 0) km, site (−85°, 0°, 0, 0): flat at 3.0°. Thresholds 1.74° and 1.75° | `earth_el_deg` 4.74192, `earth_margin_deg` 1.74192; contact 1, then 0 | 1e-5° | [R] review 1c; §1d |
| 12 | Row 11's Earth (el 4.74192°), ρ⊕ = 0.951086°: flat at 5.2°; flat at 5.8° | `earth_any_visible` true (limb at 5.693°); false | — | [S] B3; P14 |
| 13 | ρ from distance. Constant table, Sun 147.1e6 km away, then 152.1e6 km, same direction. Read `sun_el_deg` from the result, then rerun with the horizon flat at that el + 0.2690° | near Sun (ρ = 0.27109°): visibility 1, 0 < illumination < 0.01; far Sun (ρ = 0.26218°): illumination exactly 0 | — | [S] pass 2 (near-Sun fraction 4.2e-4); ρ recomputed per step |
| 14 | Row 11's call with count = 3 | length 30; columns in the §2 order; `[0..3]` = the three times; `[27..30]` = Sun distance in km | exact | ARCHITECTURE §3.2 |
| 15 | count = 0; step = 0; a start before the table | `BadInput`; `BadInput`; `OutOfTable` | exact | step 5, P16 |
| 16 | Real table, flat horizon, the IM-2 site, all of 2026–2030 hourly | Sun radius between 0.2614° and 0.2719° on every row; report min and max | — | [U] D3; needs the table |

## 6. Red/green order for /tdd

1. Rows 1–2 red, then step 1.
2. Rows 3–5 red, then steps 2–4. Rows 6–10 should pass with no further change; if row 6 fails but row 7 passes, the strip positions or the normaliser differ from §3b.
3. Rows 11–15 red, then steps 5–6 and `earth_any_visible`.
4. `Engine::timeline`, the adapter switch, row 16 as a test that reads the shipped table.
5. Deployed check: at the IM-2 site the 30-day chart shows illumination moving between 0 and 1 with partial values at sunrise and sunset, and `validate()` passes.

**Stop and escalate if** row 4 gives 1.0017 or row 3 gives 1e-9 (wrong normaliser or a missing quick test: fix, don't loosen `==`); or row 16's range falls outside 0.2614°–0.2719° (then the distance or R☉ is wrong: check `sun_dist_km` is the topocentric distance in km).
