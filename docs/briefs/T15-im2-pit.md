# T15 — IM-2 replay: replay segment, local pit, V4 statement 3

Ticket: [PLAN-DRAFT T15](../PLAN-DRAFT.md). Owner: Qi Cai. 1.5 h. Should tier, on the critical path to T16 and the video. Blocked by T08. The pit functions are pure; the replay segment needs the kernels.

## 1. Goal and done-criteria (from the ticket)

The IM-2 site on its landing date, with two answers: what regional terrain says, and how deep a 20 m pit must be to hide the Sun from a panel at height H.

- [ ] A replay segment of the ephemeris table covers 2025-03-06 with margin; T-1c-1 holds on it.
- [ ] Pit model (§1f): a 20 m pit with an 11° slope is 1.94 m deep; el_pit is 11.0° at H = 0, 5.4° at 1 m, 2.9° at 1.44 m and −0.3° at 2 m.
- [ ] The Sun's elevation at the IM-2 latitude never exceeds 6.75°.
- [ ] V4 statement 3: el_sun at touchdown (2025-03-06 17:28 UTC, marked "assumed", U8) from regional terrain; d_c* = H + (D_c/2)·tan(el_sun + ρ); and whether the 1.9 m degraded-crater depth exceeds d_c*.
- [ ] The result is worded "consistent with reports". It is never called validated and never stated as what happened (N10).
- [ ] IM-1 (statements 1 and 2) is not attempted here: its far field is out of range.

## 2. Files and signatures

`crates/core/src/pit.rs` and `crates/core/tests/pit.rs` (new) · `crates/pipeline/src/` (second segment through the T02 builder) · `web/public/data/eph-im2.<hash>.bin` and a second entry in the manifest's `ephemeris` list · `crates/wasm/src/lib.rs` · `web/public/data/scoreboard.json` (V4).

```rust
// core/src/pit.rs — radians, metres
pub const DEGRADED_SLOPE: f64 = 11.0_f64.to_radians();     // E112 (write the literal 0.19198621771937624)
pub fn default_depth(diameter_m: f64) -> f64;              // (D/2)·tan 11°
pub fn pit_el(diameter_m: f64, depth_m: f64, panel_height_m: f64) -> f64;
pub fn raise(h: &Horizon, diameter_m: f64, depth_m: f64, panel_height_m: f64) -> Horizon;
pub fn depth_to_hide(diameter_m: f64, panel_height_m: f64, sun_el: f64, sun_radius: f64) -> f64;
```

WASM, frozen (no `Result` in either):

```rust
pub fn pit_horizon(horizon: &[f64], pit_diameter_m: f64, pit_depth_m: f64, panel_height_m: f64) -> Vec<f64>;
pub fn pit_depth_to_hide(pit_diameter_m: f64, panel_height_m: f64, sun_el_deg: f64, sun_radius_deg: f64) -> f64;
```

`pit_horizon` returns the 1440 layout; for a bad horizon array or a non-positive diameter it returns an empty array, which `validate()` rejects. `pit_depth_to_hide` returns NaN for bad input.

## 3. Algorithm (SCIENCE-SPEC §1f "Local pit")

1. `default_depth(D) = (D/2)·tan(11°)` (E112). For D = 20 m: 1.9438 m.
2. `pit_el = atan((depth − H)/(D/2))`, for a lander at the centre of the pit with its panel H above the floor.
3. `raise`: for every azimuth, if `pit_el > el[i]` then `el[i] = pit_el` and `dist[i] = D/2`; otherwise leave both.
4. `depth_to_hide = H + (D/2)·tan(sun_el + sun_radius)`: the depth at which the rim covers the Sun's upper limb.
5. **Replay segment**: the T02 builder with t0 = 1 739 491 200 (2025-02-14T00:00:00Z), 1081 rows, to 2025-03-31T00:00:00Z. Run T-1c-1 on it (100 random times is enough). The adapter calls `load_ephemeris` once per manifest segment.
6. **V4 statement 3** (pipeline, real tiles): at the IM-2 site (84.7906°S, 29.1957°E; U8), mast H, t = 1 741 282 080 (2025-03-06T17:28:00Z, "assumed", U8):
   1. `horizon` → `timeline` for one step: regional answer = illumination and the Sun's margin above terrain.
   2. `d* = depth_to_hide(20, H, el_sun, ρ)` with D_c = 20 m (W6).
   3. Sentence: "From regional terrain the Sun was X° up with Y % of its disc clear. A 20 m crater would need to be d* m deep to hide the Sun from a panel H m above its floor; a typical degraded crater that wide is 1.9 m deep." Then "deeper than" or "shallower than". Report for H = 0.5, 1 and 2 m; H is "assumed" (U10).
   4. Scoreboard entry `V4.3`: `judge` "IM-2 mission reports (E73, E74, W6)", `metric` "reconstruction, consistent with reports", `pass: null` unless Qi Cai decides a rule; the numbers go in `notes`.
7. Check on the segment: max Sun el at the IM-2 site, flat horizon, over the 1081 rows ≤ 6.75° (90° − 84.7906° + 1.54° = 6.7494°, E104).

## 4. Edge cases and pitfalls

- **N10 / F9.** The pit model does not say the lander was in shadow. At H = 2 m the default pit's rim is below the panel (−0.3°). Report d*, never "the panels were dark".
- **The horizon passed in was computed at mast height H above regional ground.** Inside a pit the panel is H above the floor, which is `depth` below that ground. The model ignores the difference for the far horizon (under 2 m of observer height at kilometres of distance). Say so in the About text; don't correct for it.
- **Two segments.** `Ephemeris::at` picks the segment by time; a time between segments is `OutOfTable` (T02 row 5). The date control must refuse 2025 dates outside the replay segment.
- **U8, U9, U10, U19.** Touchdown time, crater depth, panel height and "lasted about a day" are tagged "assumed" or "unconfirmed" in every output.
- No IM-1: its site is 300 km from the pole and needs a far field to 500 km.

## 5. Test table

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | `default_depth(20)` | 1.9438 m | 1e-4 | [S] H1; E112 |
| 2 | `pit_el(20, 1.9438, H)` for H = 0, 1, 1.44, 2 | 11.000°, 5.3916°, 2.8841°, −0.3220° | 1e-3° | [S] H1; ticket |
| 3 | `raise` of a flat 1° horizon (dist 5000), pit 20 × 1.9438, H = 1 | every el 5.3916°, every dist 10 | 1e-3° | step 3 |
| 4 | Row 3 with H = 2 | unchanged: el 1°, dist 5000 | exact | pit_el < horizon |
| 5 | Horizon 8° for bins 0–359 and 2° for 360–719 (dist 5000); pit as row 3 | bins 0–359 unchanged; bins 360–719: 5.3916°, dist 10 | 1e-3° | step 3 |
| 6 | `depth_to_hide(20, 2, 5°, 0.2666°)` | 2.92179 m | 1e-5 | [S] H2 |
| 7 | `depth_to_hide(20, 0, 0, 0)`; `depth_to_hide(20, 2, −0.2666°, 0.2666°)` | 0; 2.0 | 1e-12 | step 4 |
| 8 | Consistency with T08: flat horizon at `pit_el(20, depth_to_hide(20, 2, 5°, ρ), 2)`; `disc_fraction` of a Sun at el 5°, ρ = 0.2666°. Then the same with the depth 0.05 m less | exactly 0; then > 0 | `==` | steps 2 and 4 are inverses |
| 9 | Calendar → Unix: 2025-02-14T00:00Z; 2025-03-31T00:00Z; 2025-03-06T17:28Z; rows | 1 739 491 200; 1 743 379 200; 1 741 282 080; 1081 | exact | [S] C1 and pass 2 |
| 10 | 90 − 84.7906 + 1.54 | 6.7494° | 1e-9 | [S] H4 |
| 11 | Replay segment, the IM-2 site, flat: max Sun el | ≤ 6.75° | — | [U] needs the kernels |
| 12 | `pit_horizon` with 1439 values; with diameter 0 | empty array × 2 | exact | §2 |

## 6. Red/green order for /tdd

1. Rows 1–2, then steps 1–2. Rows 3–5, then step 3. Rows 6–8, then step 4. Row 12 with the WASM wrappers.
2. Row 9 as asserts; build the segment; row 11; T-1c-1 on the segment.
3. Step 6 in `pipeline`; write the V4.3 entry and paste the sentence into the issue for T16.

**Stop and escalate if** the regional answer at touchdown is "Sun fully hidden by terrain" (surprising against E73's "low sun angle": check the site coordinates and the time first, then report it as found; don't adjust anything to make the story work).
