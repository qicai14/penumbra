# T17 — Sunset ± band

Ticket: [PLAN-DRAFT T17](../PLAN-DRAFT.md). Owner: Qi Cai. 1 h. Should tier. Blocked by T08. Pure: no data needed for any test but row 9.

## 1. Goal and done-criteria (from the ticket)

A ± time on every sunrise and sunset in the timeline, from first-order propagation of DEM height error (§1f step 1).

- [ ] σ_el = √(σ_obstacle² + σ_observer²) / D, with σ_z = 0.5 m labelled "assumed". The `_ERR` raster is not downloaded.
- [ ] The rate is the central difference of el_sun − el_hor(az_sun) across the crossing in the 10-minute timeline, not 0.5°/h times a slope.
- [ ] Checks: a 0.5 m error on an obstacle 1 km away is 0.03°, and 0.003° at 10 km. **(Decision D4: these two numbers are the obstacle error alone.)**
- [ ] At H = 5 m the band is small (E28).
- [ ] A near-flat horizon returns a wide band. It never returns a single time and never divides by zero.

## 2. Files and signatures

`crates/core/src/band.rs` and `crates/core/tests/band.rs` (new) · `crates/wasm/src/lib.rs`.

```rust
// core/src/band.rs
/// Elevation uncertainty in radians of an obstacle d metres away.
pub fn sigma_el(sigma_obstacle_m: f64, sigma_observer_m: f64, d_m: f64) -> f64;
/// timeline: 10 × n (T08 layout). horizon: 1440 (T07 layout). Returns 3 × k, column-major:
/// t_unix_s, kind (+1 sunrise, −1 sunset), half_width_min. k may be 0.
pub fn window_bands(timeline: &[f64], horizon: &Horizon, sigma_z_m: f64) -> Result<Vec<f64>, Error>;
```

WASM, frozen: `window_bands(timeline: &[f64], horizon: &[f64], sigma_z_m: f64) -> Vec<f64>`. No `Result`: on bad input return an empty array.

## 3. Algorithm (SCIENCE-SPEC §1f step 1, with F8)

1. `sigma_el = sqrt(σ_obstacle² + σ_observer²)/d`.
2. For every sample k: `m_k = sun_el_deg[k] − hor(sun_az_deg[k])`, with `hor` = `horizon_at` (T08; it wraps at 0/360).
3. An event lies between k and k + 1 when the sign of m changes (D5): `m_k < 0 ≤ m_{k+1}` is a sunrise (+1); `m_k ≥ 0 > m_{k+1}` is a sunset (−1). This is the Sun's centre crossing the horizon.
4. `frac = −m_k/(m_{k+1} − m_k)`; `t = t_k + frac·Δt`. The denominator can't be zero when the sign changes.
5. `rate = (m_{k+1} − m_k)/Δt`, degrees per second. This is the difference across the crossing (F8); it includes the Sun's own elevation rate, up to 0.046°/h at 85°S.
6. Obstacle distance: `az = sun_az_deg[k] + frac·wrap180(sun_az_deg[k+1] − sun_az_deg[k])`; `D = dist[round(az/0.5°) mod 720]`, with a Euclidean mod (`rem_euclid`): az can come out a hair below 0 or at 360.
7. `σ = sigma_el(sigma_z, sigma_z, D)` in degrees (observer and obstacle share σ_z = 0.5 m, E23a, "assumed").
8. `half_width_min = σ/|rate|/60`. Append `(t, kind, half_width_min)`.

## 4. Edge cases and pitfalls

- **D4.** The ticket's "0.03° at 1 km" is 0.5 m/1000 m with the observer error at zero. With both ends at 0.5 m the function returns 0.0405°. Row 1 tests the formula both ways; the band uses both ends.
- **F8.** Never compute the rate as 0.5°/h × horizon slope.
- **Near-flat horizon.** A grazing crossing has a tiny rate and a band of hours or days. Return it as is, finite; T18 writes "ill-conditioned" above 6 h. A Sun that skims within σ of the horizon without m changing sign produces no event: a known limit, stated in the About text.
- **P13.** If D < 60 m the obstacle is within 3 pixels and the band is huge (σ/D). That is the honest answer; don't clamp D.
- **N8.** Step 6 must wrap the azimuth difference, or a crossing near north reads a distance from the wrong side of the sky.
- The band covers LOLA height error only, not terrain smaller than a pixel (E114).

## 5. Test table

Synthetic timelines: Δt = 600 s, `sun_az_deg` constant 100° unless stated; horizon flat at 1.0°, dist 2 000 m everywhere unless stated; σ_z = 0.5.

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | `sigma_el(0.5, 0.5, 2000)`; `(0.5, 0, 1000)`; `(0.5, 0, 10 000)`; `(0.5, 0.5, 1000)` | 0.020257°; 0.028648°; 0.0028648°; 0.040514° | 1e-6° | [S] H3; [R] review 1f; D4. The last is √2 × 0.028648 [S] |
| 2 | Two samples, `sun_el_deg` = 1.004233, 0.995767 (m falls 0.0084667° in 600 s = 0.0508°/h) | one event: t = t_0 + 300 s, kind −1, half width 23.93 min | 0.01 min | [S] H3; [R] review 1f |
| 3 | Row 2 reversed (0.995767, 1.004233) | t_0 + 300 s, kind +1, 23.93 min | 0.01 min | symmetry |
| 4 | Row 2 with dist 10 000 m | 4.785 min | 0.01 min | 23.93 × 2000/10 000 |
| 5 | Three samples all above the horizon (1.5, 1.4, 1.3) | empty result | exact | step 3 |
| 6 | m = +1e-7°, −1e-7° (near-flat) | one event; half width finite and above 6 h (about 16 880 h) | — | ticket: wide band, no division by zero. 0.020257° ÷ (2e-7° per 600 s = 1.2e-6°/h) [S] |
| 7 | m = 0 exactly at k, negative at k + 1; and negative at k, 0 at k + 1 | one sunset at t_k; one sunrise at t_{k+1} | exact | step 3: each event counted once |
| 8 | Wrap: `sun_az_deg` = 0.2, 359.8 across a sunset; dist = 2 000 for bins 719, 0, 1 and 90 000 for the rest | half width as row 2 (uses D = 2 000) | 0.01 min | N8 |
| 9 | Real: the IM-2 site, mast 5 m, 30 days | bands reported; most under 60 min | — | [U] E28; needs tiles |
| 10 | Layout: row 2's result; bad timeline length | `[t, −1, 23.93]` (length 3); `Err(BadInput)` in core, empty array from WASM | — | ARCHITECTURE §3.2 |

## 6. Red/green order for /tdd

1. Row 1, then step 1.
2. Rows 2–5 red, then steps 2–8 without the wrap. Rows 6–7.
3. Row 8 red, then the wrap in step 6.
4. Row 10 and the WASM wrapper. Row 9 on the deployed page, by eye, noted in the PR.

**Stop and escalate if** row 2 gives 16.9 min (σ computed with one end only), or 12.0 or 47.9 min (the rate is off by a factor of 2): fix the formula. If Qi Cai overrules D4 or D5, only step 3 or step 7 changes.
