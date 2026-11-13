# T09 — Survival: state of charge, operable window, first lethal shadow

Ticket: [PLAN-DRAFT T09](../PLAN-DRAFT.md). Owner: Qi Cai. 1.5 h. Blocked by T08 for real data only. The `core` part is pure and needs no DEM: under gate G1 (DEM late) do it before T06.

## 1. Goal and done-criteria (from the ticket)

The survival headline for one lander preset: operable days and time to first lethal shadow.

- [ ] T-1e-1: no generation, 100 W load, 705 Wh start, E_min = 0 gives death at 7.050 h ± 1e-9, interpolated from the unclamped value.
- [ ] T-1e-2: energy is conserved when no clamp is hit.
- [ ] Solar flux is 1366.1 W/m² scaled by (1 AU / d)². A Sun-tracking panel and a fixed panel both work; a vertical panel facing azimuth a gives cos(el)·cos(az − a).
- [ ] The simulation stops at the first lethal event. "No lethal shadow in the span" is its own result, not a large number.
- [ ] Operable window = illuminated AND in Earth contact. Longest darkness (f = 0) and longest any-part-visible period follow the E123 definitions.
- [ ] One preset uses the cited values (10 000 Wh, 1.5 m², 30 %, 220 W) marked "cited"; every other value is marked "assumed" (U10).
- [ ] The deployed headline for the IM-2 site reads real data.

## 2. Files and signatures

`crates/core/src/power.rs` and `crates/core/tests/power.rs` (new) · `crates/wasm/src/lib.rs` (`survival`, a free function) · one switch in `web/src/lib/engine.ts` · the preset list lives in `web/` (it is data with tags, not maths).

```rust
// core/src/power.rs
pub const S0_W_M2: f64 = 1366.1;            // E57
pub const AU_KM: f64 = 149_597_870.7;       // standard value, U17

/// timeline: 10 × n column-major, the layout timeline() returns.
/// preset: 12 values, see below. landing_unix_s: must lie in [t_0, t_{n−1}].
/// Returns 8 + n values: header, then soc_wh[n].
pub fn survival(timeline: &[f64], preset: &[f64], landing_unix_s: f64) -> Result<Vec<f64>, Error>;
```

WASM, frozen: `survival(timeline: &[f64], preset: &[f64], landing_unix_s: f64) -> Result<Vec<f64>, JsError>`; a pass-through.

`preset[12]` (ARCHITECTURE §3.2): 0 `capacity_wh`, 1 `start_wh`, 2 `min_wh`, 3 `panel_area_m2`, 4 `efficiency` (0–1), 5 `panel_mode` (0 = Sun-tracking, 1 = fixed), 6–8 fixed-panel normal `east`, `north`, `up`, 9 `load_w`, 10–11 reserved zeros.

Result: 0 `operable_h`, 1 `lethal_h` (hours after landing; NaN = no lethal shadow in the span), 2 `longest_dark_h`, 3 `longest_any_sun_h`, 4 `sunlit_h`, 5 `contact_h`, 6 reserved 0, 7 `n`, then `soc_wh[k]` = stored energy at time t_k.

Timeline columns used: 0 `t_unix_s`, 1 `sun_az_deg`, 2 `sun_el_deg`, 3 `illumination`, 4 `visibility`, 8 `contact`, 9 `sun_dist_km`.

## 3. Algorithm (SCIENCE-SPEC §1e with F7; ARCHITECTURE §2.5)

1. **Validate**, else `BadInput`: `timeline.len()` is a multiple of 10 with n ≥ 2; `preset.len() == 12`; everything finite; `capacity > 0`; `min ≥ 0`; `min < start ≤ capacity`; `area ≥ 0`; `0 ≤ efficiency ≤ 1`; `load ≥ 0`; `panel_mode` is 0 or 1; for mode 1 the normal's length is above 1e-9 (normalise it); `t_0 ≤ landing ≤ t_{n−1}`.
2. `Δt_h = (t_1 − t_0)/3600` (the step is read from the timeline, never passed). `k0` = the first k with `t_k ≥ landing` (D2).
3. `soc[k] = start` for k ≤ k0. `E = start`.
4. For k = k0 … n − 1 (sample k stands for [t_k, t_k + Δt)):
   1. `S = S0·(AU_KM / sun_dist_km[k])²`.
   2. Incidence: tracking → 1. Fixed → `max(0, n̂·ŝ)` with `ŝ = (cos el·sin az, cos el·cos az, sin el)` in (east, north, up) from columns 1–2.
   3. `P_gen = S·area·efficiency·incidence·illumination[k]`.
   4. `E* = E + (P_gen − load)·Δt_h`.
   5. If `E* ≤ min`: `t_lethal = t_k + Δt_s·(E − min)/(E − E*)`; `k_L = k`; set `soc[j] = min` for every j > k; stop the loop. The crossing uses E* **before** clamping (F7).
   6. Else `E = min(E*, capacity)`; if k + 1 < n, `soc[k + 1] = E`.
5. `lethal_h = (t_lethal − landing)/3600`, or NaN if the loop never stopped.
6. **Sums** (D2). Weight `q_k` = 1 for k0 ≤ k < k_L, `(t_lethal − t_k)/Δt_s` for k = k_L, 0 otherwise; with no lethal event, 1 for every k ≥ k0.
   - `sunlit_h = Σ q_k·illumination[k]·Δt_h`
   - `contact_h = Σ q_k·contact[k]·Δt_h`
   - `operable_h = Σ q_k·(illumination[k] > 0 and contact[k] == 1)·Δt_h` (the glossary's operable window)
7. **Runs**, over k0 … n − 1 whether or not the lander died, because they describe the site (D2; definitions E123):
   - `longest_dark_h` = longest run of `illumination[k] == 0` × Δt_h
   - `longest_any_sun_h` = longest run of `visibility[k] == 1` × Δt_h
8. Header slot 6 = 0, slot 7 = n.

Not modelled, and stated on the page: charge and discharge efficiency, a night heater load, panel temperature (SCIENCE-SPEC §1e).

**Preset for the page** (adapter data, not `core`): capacity 10 000 Wh (E58a), panel 1.5 m² (E58b), efficiency 0.30 (E58c), load 220 W (E58d): all four "cited" (a rover paper's values, said so in the tooltip). `start_wh` 10 000, `min_wh` 0, tracking panel, Earth margin 2° (E62), mast 2 m: "assumed" (U10).

## 4. Edge cases and pitfalls

- **F7.** Interpolating from the clamped value returns the step end: 7.167 h in T-1e-1 instead of 7.050 h.
- **"No lethal shadow"** is NaN in slot 1, and the adapter maps NaN to `null`. Never a large number.
- **After death** `soc` holds `min` to the end, so the chart shows a flat dead battery and no NaN reaches uPlot.
- **Flux distance** is column 9 in km, the topocentric distance. 1 AU in metres or a distance in metres gives a flux 10⁶ off; row 4 catches it.
- **Fixed panel azimuth.** `ŝ` is (east, north, up), so a panel facing azimuth a is `(sin a, cos a, 0)`. Swapping sin and cos mirrors it; rows 5–6.
- **P1.** Power uses `illumination`. `visibility` is only for the any-Sun run.
- **Equality.** `E* ≤ min` (not `<`): landing exactly on the minimum is lethal at the step end.
- The lethal check also runs for the last sample (k = n − 1), though its `soc[k + 1]` isn't stored.
- A timeline with a non-uniform step is not supported; steps come from `timeline()` and are uniform.

## 5. Test table

Test timelines are built by a helper: n samples, t_k = 600·k, `sun_dist_km` = 149 597 870.7, `sun_az_deg` = 30, `sun_el_deg` = 5, other columns as stated. Preset P0 = [10000, 10000, 0, 1.5, 0.30, 0, 0, 0, 0, 220, 0, 0]. Landing = 0.

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | T-1e-1: n = 60, illumination 0. Preset capacity 1000, start 705, min 0, load 100 | `lethal_h` = 7.050; death in the 43rd step (k_L = 42); `soc[42]` = 5.0; `soc[43..]` = 0 | 1e-9 h; 1e-9 Wh | [S] E4; spec T-1e-1 |
| 2 | Tracking, illumination 1, 1 AU: generation per step | P_gen = 614.745 W | 1e-9 | [R] review 1e: 1366.1 × 1.5 × 0.30 |
| 3 | n = 7, illumination 1, contact 1. P0 with start 5000 | `soc[6]` = 5 394.745; `lethal_h` NaN | 1e-9 | [R] review 1e; T-1e-2: 6 × (614.745 − 220)/6 |
| 4 | Row 2 with `sun_dist_km` = 147.1e6 | P_gen = 1 412.889 × 0.45 = 635.800 W (from `soc[1] − soc[0]`, start 5000) | 1e-3 W | [S] E2 |
| 5 | Fixed panel, normal (0, 1, 0) (vertical, facing north); Sun az 30°, el 5°, illumination 1 | incidence 0.8627299; P_gen = 530.359 W | 1e-6; 1e-3 W | [S] E3: cos 5° · cos 30° |
| 6 | Fixed panel facing az 120°: (sin 120°, cos 120°, 0); facing 210° | P_gen = 0 for both (cos 90° = 0, then a negative clamped) | 1e-9 W | spec §1e |
| 7 | n = 4, illumination 1. P0 with start 9900 | `soc` = 9900, 9 965.790833, 10 000, 10 000 | 1e-6 | [S] E6: the clamp at capacity |
| 8 | T-1e-2: n = 200, seeded random illumination in [0, 1], P0 with start 5000, capacity 1e9, load 300 | `soc[n−1] − soc[0]` = Σ_{k<n−1} (614.745·f_k − 300)/6, provided no lethal event | 1e-6 Wh | spec T-1e-2 |
| 9 | Sums: n = 12, illumination [1, 1, 0.5, 0, 0, 0, 0.25, 1, 1, 0, 0, 1], visibility = (illumination > 0), contact [1, 1, 1, 1, 0, 0, 0, 1, 1, 1, 0, 0]. P0 with capacity and start 1e6 | `sunlit_h` 0.958333; `contact_h` 1.166667; `operable_h` 0.833333; `longest_dark_h` 0.5; `longest_any_sun_h` 0.5; `lethal_h` NaN; slot 7 = 12 | 1e-6 | [S] E5 |
| 10 | Row 1 again, contact 1 throughout | `contact_h` = 7.050 (42 full steps and 0.3 of the 43rd); `operable_h` 0; `longest_dark_h` = 10.0 (all 60 samples) | 1e-9 | D2: sums stop at death, runs don't |
| 11 | Row 3, then continue with illumination 0: n = 7 + 160, illumination 1 for k < 6 and 0 after | `lethal_h` = 1.0 + 24.521568 = 25.521568 | 1e-5 h | [S] E1: 5 394.745 / 220 |
| 12 | Landing mid-span: row 1's timeline with landing = 6000 s | `soc[0..=10]` = 705; `lethal_h` still 7.050; `longest_dark_h` = 50 × (1/6) = 8.3333 | 1e-9 | D2: k0 = 10 |
| 13 | Bad input: 11 preset values; start > capacity; start = min; `panel_mode` 2; mode 1 with a zero normal; n = 1; landing before t_0 | `BadInput` each | exact | step 1 |
| 14 | Result length and header | length 8 + n; slot 6 = 0; slot 7 = n | exact | ARCHITECTURE §3.2 |

## 6. Red/green order for /tdd

1. Row 13 and row 14 red, then step 1 and the output shell.
2. Row 1 red, then steps 2–5 with tracking only. Rows 2, 3, 7, 8, 11.
3. Row 4 (flux), rows 5–6 (fixed panel).
4. Row 9 red, then steps 6–7. Rows 10 and 12.
5. WASM export, adapter switch, preset with tags. Deployed check: the headline at the IM-2 site shows operable days and either a lethal time or "no lethal shadow in 30 days", and the state-of-charge trace never goes below `min_wh`.

**Stop and escalate if** row 1 gives 7.1667 (clamped interpolation, F7) or 7.0 (the crossing is rounded to a step): fix the crossing formula, never the tolerance. If Qi Cai overrules D2 at the freeze, only steps 6–7 and rows 10 and 12 change.
