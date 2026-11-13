# SCIENCE-SPEC-REVIEW — independent referee check of docs/SCIENCE-SPEC.md

Progress: COMPLETE (Nov 10) · check 1 re-derivations done and script-verified · check 2 ledger audit done (173/173 grep, all rows read in context, 6 of 10 W-rows re-fetched) · check 3 thresholds done · check 4 pitfalls done · check 5 scope done · verdict written

Reviewer: Fable 5.2 (second session). Documents only; SCIENCE-SPEC.md not edited. Bulk reading by sonnet subagents; every quote cited below was re-checked with grep or sed by the reviewer.

## Verdict

**Spec usable after fixes.** No section needs redoing. Every equation in §1 survived independent re-derivation and a numeric check. The errors are one wrong Must test, one data-path decision, the horizon range, one numerical recipe that fails its own test, three overclaims in the validation story, and a missing hand-off plan.

Counts: **BLOCKER 0 · MAJOR 7 · MINOR 15**

| ID | Sev. | Where | One line |
|----|------|-------|----------|
| F6 | MAJOR | §1d T-1d-1, V5 case 5 | Test is false: the Earth reaches +1.6° at 85°S, 180° |
| F3 | MAJOR | §1c shipped table, P4 | TDB grid forces a hand-coded 69 s offset in the browser; V1 does not cover that path |
| F2 | MAJOR | §1a range, IDEA data plan | 150 km misses terrain that sets the longest night at ridge sites |
| F9 | MAJOR | §1f pit, V4 statement 3, demo step 2 | Pit model gives no shadow for a panel 2 m up; "dark below the rim" is false |
| F14 | MAJOR | §0 A1, §3 V2b | Longest-night benchmark is not like-for-like and has no pass rule |
| F15 | MAJOR | §3 V2 row | "2-point median is realistic" has no supporting evidence |
| F17 | MAJOR | §4 | Zero slack for A; B blocked on A for 8 h; 3.7 GB error raster not budgeted |
| F18 | MINOR | §1b, T-1b-1 | 32 midpoint strips miss the 1e-3 tolerance and give f = 1.0017 for a clear Sun |
| F1 | MINOR | §1a early exit | Bound is not monotonic for a summit observer |
| F4 | MINOR | §1c interpolation | Wrong reason, right conclusion |
| F5 | MINOR | §1c V1 request | E55 is about RA/DEC; add azimuth wrap |
| F7 | MINOR | §1e | E_min undefined; crossing interpolated from the clamped value |
| F8 | MINOR | §1f prose | Sun elevation rate is not negligible at 85°S |
| F10 | MINOR | E131 → [U] | Table 3 comes from the 1970–2044 run, not the 2020 cycle |
| F11 | MINOR | E60/E109 | Source's reasoning is numerically wrong; 6.5° maximum is [D] |
| F12 | MINOR | E63 | Source rejects its own 3σ model in the next sentence |
| F13 | MINOR | E19a | Abstract's 240 d / 1.5 d appear nowhere in the body |
| F16 | MINOR | §3 V3 | Geometric mask vs Lambertian render is not like-for-like |
| F19 | MINOR | §1a, STACK | DEM is a cloud-optimized GeoTIFF; decode path untested |
| F20 | MINOR | T-1a-2 | SP-01 lies on the map diagonal, the worst landmark for an x/y swap |
| F21 | MINOR | E33b, E03, E23c | Quotes truncated or weaker than the claim |
| F22 | MINOR | §4 diff list, docs/IDEA.md | "Not applied" is stale; one garbled row |

Top three fixes: F6, F3, F2. Then F17 (hour-0 API freeze) before anyone writes code.

Finding format below: **severity** · location · what is wrong · evidence · fix (replacement text).

---

## 1. Re-derivations (check 1)

All numbers in this section were recomputed by script (f64) after being worked by hand.

### 1a Horizon elevation with curvature — formulas CONFIRMED
Re-derived independently: inverse and forward stereographic (û = (y/R, x/R, −(1−t))/(1+t) follows from sin c = 2T/(1+T²), cos c = (1−T²)/(1+T²), T = r/2R); scale k = 1+t (1.00765 at 80°S); ENU frame; pole limit (north → +X, east → +Y approaching along lon 0); tan(el) closed form; small-angle form; drops 0.74 m / 719 m / 6475 m; dip 0.08694° at H = 2 m; map-line curvature r/(2R²) → 283 m at 150 km from 5° colatitude; SP-01 → (−11.308, −12.259) km. All agree with the spec.

**Test vector 1a.** h_s = 0, H = 2 m, obstacle top h = 1000 m at ground distance s = 50 km. θ = s/R = 0.02877863 rad, r_o = 1 737 402 m, r_t = 1 738 400 m. r_t·cos θ − r_o = 278.17 m; r_t·sin θ = 50 021.87 m. **Expected el = 0.31862°.** The small-angle form gives 0.31918°, so tests must use the tan form. Second vector: flat sphere, H = 0.1 m → **−0.01944°**.

**F1 · MINOR · §1a "Early exit"** — called exact, but for a = h_max − h_s − H < 0 the bound a/s − s/2R rises until s = sqrt(2R|a|) (83 km for |a| = 2 km), so an early stop drops real horizon. It also uses the small-angle form.
Fix: "Early exit: stop an azimuth at distance s when B(s) < current max, where B(s) = atan((r_max·cos θ − r_o)/(r_max·sin θ)), r_max = R + h_max, θ = s/R, **and only if r_max > r_o**. If r_max ≤ r_o, do not exit early."

**F2 · MAJOR · §1a "Range"; docs/IDEA.md Science model 1 and Data plan** — 150 km is too short for the sites the headline depends on.
- De Rosa's stated reason does not hold: a 7 km peak at 150 km sits at **+0.19°**, not "lower than the minimum Sun elevation (e.g. -6.5° at -85° latitude)" (114cd0dd:267–268).
- The ledger's own Bryant quote gives the bound for the pole: "at 200 km from the pole, a mountain must be over 7000 m tall in order to block the winter Sun" (018af83e:489–490). Check: 7 km at 200 km is at −1.30°; the midwinter upper limb is at −1.27°.
- For an elevated observer it is further. From h_s = 1.9 km (Shackleton ridge), a 6 km massif at 170 km is at **−1.43°**, inside the midwinter disc (limbs −1.81° to −1.27°); a 7 km peak still reaches the lower limb at 198 km. Mons Mouton is about 180 km from SP-01. The longest night (A1) happens exactly then.
- Mazarico states no maximum distance; "150km away" (E07) is a colour-scale label in a caption (7b941912:2223). His DEM was 915 × 915 km with 5° of padding (7b941912:213–217, 2229–2231).
Fix, §1a: "Range: no fixed cut-off. March until the early-exit bound fires or the far-field tile ends. Crop the far field to site + 200 km at least (Bryant, E66/E100); Barker 2023 uses 310 km (E93)." Fix, docs/IDEA.md: "to 150 km" → "to at least 200 km"; "site + 150 km (about 16 MB raw for ±336 km)" → "site + 200 km (about 20 MB raw for ±382 km)". New test T-1a-5: "SP-01, H = 10 m: horizon with range 150 km vs tile edge; report the largest el difference and the change in longest night."

Note, no finding: int16 at 0.25 m spans ±8191 m; the far-field tile's relief can approach that, so it cannot go "finer" there. Use 0.25 m near field, 0.5 m far field.

**F19 · MINOR · §1a, STACK "DEM I/O"** — the page says "All TIF files are cloud-optimized GeoTiffs" (583c4107:22). So the 2.6 GB DEM (583c4107:167–169) is tiled and compressed; decoded whole it is 30 400² × 4 B = 3.7 GB. Data type and nodata are not stated anywhere in the corpus (U5).
Fix, add to §1a: "Read the DEM tile by tile; never decode the whole raster. At hour 0, open `LPSR_80S_20MPP_ADJ.TIF` (148 MB, same grid) with the `tiff` crate to prove the compression and predictor are supported before the 2.6 GB download finishes. If not, convert once with `gdal_translate -co COMPRESS=NONE` on any machine that has GDAL."

**F20 · MINOR · §1a orientation check, T-1a-2** — SP-01 is at (−11.3, −12.3): almost on the diagonal, so an x/y swap moves it only 1.3 km, onto the same ridge system. The LOLA description gives no X/Y ↔ longitude rule (ad6021a1:86–91 has only the colatitude formula).
Fix, add to T-1a-2: "All seven Barker RoI centroids (X/Y cells at b5251ef3:555–561 and 563–569) must be higher than the mean DEM height on a 200 m ring around them. Also assert that the IM-2 site (84.7906°S, 29.1957°E) maps to (+77.1, +138.0) km; a swap moves it 86 km."

### 1b Visible Sun-disk fraction — formula CONFIRMED, recipe fails its test
Tangent-plane integral, clamp and closed form re-derived. ρ at 1 AU = 0.26657°; range 0.2622–0.2711°.

**Test vectors 1b** (flat horizon at u·ρ above the Sun's centre): u = 0.5 → **f = 0.19550**; u = −0.5 → **0.80450**; u = 0.9 → **0.01869**; u = 0.3 → **0.31192**.

**F18 · MINOR · §1b "32 midpoint strips", T-1b-1** — with the spec's normalisation 1/(πρ²), 32 midpoint strips give f = **1.0017** for an unobstructed Sun and are off by 1.6e-3 at u = −0.5. T-1b-1 (1e-3) fails for correct code, f never equals 1 exactly, and power runs 0.17 % high. (n = 16: 4.8e-3; n = 64: 6.0e-4.)
Fix: "Normalise by the same quadrature: f = Σ clamp(w_i − g_i, 0, 2w_i) / Σ 2w_i. This gives exactly 1 for a clear disc and exactly 0 for a hidden one. Tolerance for T-1b-1: 2e-3 with 32 strips."

### 1c Topocentric az/el — physics CONFIRMED
Checked: stellar aberration v/c = 0.00569°; Earth light-time shift 38 km / 384 400 km = 0.0057°, cancelling aberration to (v_M − v_E)/c ≈ 0.0002°; UTC−TDB 69.184 s × 0.508°/h = 0.00976°; site velocity and 6 ms negligible; parallax 0.259° (Earth) and 0.00067° (Sun). "Apparent at Moon centre, rotate at t, subtract the site vector" equals a topocentric apparent position to well under 1e-5°. Frame chain supported: "(or mean Earth of DE421 for the Moon)" (01433f98:834); "a rotation by a fixed offet from the DE440 principal axis" (01433f98:1105); east-positive for the Moon (01433f98:850–852). `TIME_TYPE` is a valid request parameter (de84f202:277).

**Test vectors 1c** (site φ = −85°, λ = 0°, h = H = 0, R = 1737.4 km; û = (0.0871557, 0, −0.9961947); east = (0, 1, 0); north = (0.9961947, 0, 0.0871557)):
- Body at r = (384 400, 0, 0) km in MOON_ME → d·up = 31 765.25, |d| = 384 252.47 → **az 0.00000°, el 4.74192°** (Moon-centre value 5.000°; parallax −0.258°).
- Body at 149 597 870.7 km toward sub-point (lat −1°, lon 30°E) → **az 30.13857°, el 5.32713°**.

**F3 · MAJOR · §1c "Shipped table", §2 P4** — the table is on a TDB grid, so the browser must turn a UTC date into TDB itself. The spec says never to hand-code that offset, hifitime is not in the browser path, and V1 as written can pass on native ANISE while the shipped path is 0.010° and 69 s off.
Fix, §1c: "Store the table on a fixed **UTC** grid: rows are whole UTC hours from 2026-01-01T00:00:00Z, built natively with hifitime. The browser never converts time scales." Fix, §3 V1 details, add: "V1 runs through the shipped table + interpolation code path, not through ANISE directly."

**F4 · MINOR · §1c "[D] Hourly is enough"** — Δθ²/8 is the radial sag of the chord (relative 1e-5), not an angle. The direction error of linear interpolation is third order (about 0.016·Δθ³ ≈ 1e-8 rad); libration acceleration dominates at about 1.6e-6 rad. Conclusion stands.
Fix: "…the direction error of linear interpolation is about 0.0001° (libration acceleration ≈ 1e-12 rad/s² × (3600 s)²/8); the chord shortens |r| by 1e-5, which is harmless."

**F5 · MINOR · §1c V1 request** — E55 sits under "Extra-precision on RA/DEC output" (01433f98:1676–1678); nothing says az/el gain digits. Cite it only for the parameter's existence (U13 already covers the rest). Add to V1 details: "Wrap Δaz to ±180° before multiplying by cos(el)."

### 1d Earth margin — the test is wrong
**F6 · MAJOR · §1d T-1d-1, §3 V5 case 5** — "at lat −85°, lon 180° on a flat sphere the Earth is below the horizon at all times" is false. With the sub-Earth point at latitude −6.87° (E102) the Earth's centre is 88.13° from the site. Script: **el = +1.61°** topocentric (upper limb about +2.6°). Latitude libration comes within a degree of that most months. V5 is a Must, so correct code fails a Must test.
Fix: "T-1d-1: at a far-side site (lat −60°, lon 180°) on a flat sphere the Earth's upper limb stays below −20° at all times in 2026–2030 [D: centre peaks at −23.4°]. T-1d-1b: at (lat −85°, lon 180°) the Earth's centre elevation peaks between +1.0° and +1.7° and is negative more than half the time." V5 case 5 → "Far side at 60°S: Earth never visible (T-1d-1)."
P9's "within 9° of azimuth 0" holds: the extremes are 8.11° and 8.28°.

### 1e Battery state of charge — CONFIRMED
Units: W/m² × m² × 1 × 1 × 1 = W; W × h = Wh. Vertical-panel term cos(el)·cos(az − a) re-derived.

**Test vectors 1e.** S = 1366.1 W/m², A = 1.5 m², η = 0.30, tracking, f = 1 → **P_gen = 614.745 W**. P_load = 220 W, E = 5000 Wh, six 10-min steps → **5394.745 Wh**. Then f = 0: empty after **24.522 h**.

**F7 · MINOR · §1e "Lethal event", T-1e-1** — E_min is never defined, and interpolating the crossing from the clamped E_{k+1} returns the step end. With E₀ = 705 Wh and 100 W the true answer is 7.050 h; the clamped interpolation gives 7.167 h. T-1e-1 (700 Wh, exactly 42 steps) cannot see this.
Fix: "E_min is a preset field (default 0 Wh). Lethal time = t_k + Δt·(E_k − E_min)/(E_k − E*_{k+1}), where E* is the value before clamping. The simulation stops at the first lethal event." Fix test: "T-1e-1: P_gen = 0, P_load = 100 W, E₀ = 705 Wh, E_min = 0: dead at 7.050 h ± 1e-9."
State under Limitations: no charge or discharge efficiency, no separate night heater load.

### 1f Sunset-time band — equation CONFIRMED
**Test vector 1f.** σ_z = 0.5 m at both ends, D = 2 km → **σ_el = 0.02026°**. Rate 0.508°/h × slope 0.1 = 0.0508°/h → **σ_t = 23.9 min**.

**F8 · MINOR · §1f Step 1 prose** — "almost not at all in elevation" is true only at the pole. At IM-2's 84.79°S the Sun's elevation rate reaches **0.046°/h**, as large as the 0.05°/h azimuth term in the spec's own example. Dropping it can halve or double the band.
Fix: "Compute the rate by central difference of m(t) = el_sun(t) − el_hor(az_sun(t)) across the crossing in the 10-minute timeline. Do not approximate it by 0.5°/h × slope: at 85°S the Sun's own elevation rate is up to 0.046°/h."

**F9 · MAJOR · §1f "Local pit", §3 V4 statement 3, docs/IDEA.md demo step 2** — the claim does not follow from the model. With the spec's defaults (D_c = 20 m, d_c = 1.94 m), el_pit = 11.0° at H = 0, 5.4° at H = 1 m, 2.9° at H = 1.44 m (0.5 m below the rim) and **−0.3° at H = 2 m**. So "dark for any panel below the rim" is false whenever the Sun is above a few degrees, and "add the 20 m crater … and the panels are in shadow" is not guaranteed. The spec's own source says such gaps "disappear when increasing the height above the surface to e.g. 5 m" (114cd0dd:480–481), and the reported cause was "a low sun angle and the orientation of the lander's solar cells" (E73).
Fix, V4 statement 3: "IM-2: report el_sun at touchdown from regional terrain, and the pit depth d_c* = H + (D_c/2)·tan(el_sun + ρ) that would hide the whole disc from a panel at height H. State whether the ESA degraded-crater depth (1.9 m for 20 m) exceeds d_c*." Fix, docs/IDEA.md demo step 2: "…Add a 20 m crater: Penumbra shows how deep it must be to shade a panel at height H, and whether a typical degraded crater is that deep."

---

## 2. Evidence audit (check 2)

- **Script:** all 173 E-row quotes are substrings of the stated file at the stated line (173/173). The random 30 % sample (seed 20261110, 51 rows) passes too, since every row does.
- **Meaning:** all 173 rows were read in context: 51 sample rows and 62 formula-critical rows by the reviewer, the other 60 by a sonnet subagent with the reviewer re-reading every row it flagged.
- **W-rows:** W1, W2, W3a, W3b, W3c, W4 and W7 re-fetched from the live label and PGDA product 69, through a summarising fetch tool; all match. The label also has `LINE_PROJECTION_OFFSET = 2528.5`, so the pole sits on a pixel corner. W1b, W5 and W6 were not re-fetched.

### [V] rows that should be [U] or reworded

**F10 · MINOR · E131 as used in §3 V2b → [U]** — the sentence belongs to Fig. 13 (7b941912:2252–2255). Table 3 is tied to the other run: "From the results of the long temporal simulation (Section 3.3, Fig. 6), we identified the 50 grid elements…" (7b941912:1313–1315), which is 1970–2044 at 6 h (7b941912:853–854). The span of the SP-01 day/night numbers is only implied.
Fix, V2b: "Mazarico's Table 3 averages come from the four-cycle 1970–2044 run (E09). The SP-01 day/night numbers appear to use one precession cycle from 2020 (E131, stated for Fig. 13 only) [U20]."

**F11 · MINOR · E60/E109 as used in §1a and docs/IDEA.md Problem bullet 1** — the quote exists but its reasoning is wrong by 6.7° (F2), and E109 gives the minimum Sun elevation at 85°S. The "reaches about 6.5°" maximum is [D] (5° + 1.54°), not E109.

**F12 · MINOR · E63 as used in §1f** — the next sentence reads "this model returns unfeasible illumination conditions for the lander" (114cd0dd:397–398).
Fix: "De Rosa describes a 3σ worst case (E63) and notes it is too pessimistic for a 2 m array; offer it only as an optional upper bound."

**F13 · MINOR · E19a** — the abstract's "sunlit continuously for 240 days" and "about 1.5 days" (7b941912:53–54) match nothing in the body, which gives SP-01 202 ± 7 d at the surface, 204 ± 8 d at 10 m, 5.88 d and 2.75 d. Do not use E19a as a benchmark; F14 names the numbers to use.

**F21 · MINOR · truncated or weak quotes**
- E33b: the sentence ends "…of a few meters relative to the principal axis frame" (56dae622:281). It is the ME-to-PA realisation error, which is what V1 needs, but the ledger cuts the qualifier. Fix the "supports" cell: "ME frame uncertainty relative to PA".
- E03: 1737.4 km is the reference radius of Mazarico's gnomonic projection (7b941912:207), not of the heights. R rests on E30/E31a; drop E03 from that list.
- E23c: the caption (b5251ef3:405–406) does not name Site 1. Relabel "slope error, site not stated".
- E42b and E78 are weaker than their labels ("rather imprecise"; a commit title). Harmless.

Rows checked and sound that carry the most weight: Table 3 column order (7b941912:1338–1346, 1749–1756); E81/E82 metric definitions; E12; E08/E05; E31b; E135; E36; E46/E96/E47; E48a–c; E28; E123; E127/E128; E62; E112; E114.

---

## 3. Thresholds V1–V5 (check 3)

**V1 (≤ 0.002°): honest and reachable** once F3 is fixed. Residuals after the listed corrections are of order 1e-4°.

**F14 · MAJOR · §0 A1, §3 V2b** — the longest-night benchmark is not like-for-like and has no pass rule.
- Barker's numbers are not longest periods. Table 2 gives "the 1st percentile of LCIP" and "the 99th percentile of LCSP" over 100 clones; the ranges quoted in §0 are case A, "computed after averaging the RoI for each clone" (b5251ef3:547–550). X/Y are centroids of regions of 3825–13 300 m² (b5251ef3:570–577), on a 5 m DEM. No median or single-DEM value is printed, and the real DEM "tend[s] to be better than the mean/median of the clones" (b5251ef3:1193–1194).
- Ours is one 20 m DEM at one point, so the comparison is one-sided.
- SP-01 is given to 0.01°, which is ±152 m in latitude. "202 ± 7 days of total" sunlight is the full-disc figure; the any-part figure is "261 days over the precession cycle" (7b941912:2288–2289), height unclear.
Fix, replace the A1 row's benchmark sentence and the V2b bullet with:
"**Barker Site 1**, Δz = 5 m, 2024-01-01 to 2026-01-01, hourly. For each RoI average our 20 m pixels within sqrt(Area/π) of the centroid. Published case-A values, RoI 1–7: LCIP-1 = 215.79, 216.28, 216.89, 231.20, 218.98, 220.98, 119.07 days (b5251ef3:682–688); LCSP-99 = 5.29, 7.31, 8.41, 4.00, 4.52, 4.16, 5.42 days (b5251ef3:706–712). Pass if LCSP ≤ LCSP-99 + 1 day and LCIP ≥ LCIP-1 − 10 days for at least 6 of 7 RoIs. This is one-sided: it catches a model that is too dark, not one that is too bright. **Mazarico SP-01**, H = 10 m, one cycle from 2020 (assumed): evaluate every 20 m pixel within 150 m of (−11.31, −12.26) km; pass if 2.75 days lies inside the range of longest total night (f = 0) over those pixels. Report both the f > 0 and the f = 1 day lengths."
Also delete "a longest-night number cannot" from §0; with a one-sided test it can.

**F15 · MAJOR · §3 V2 row and "How good can agreement be?"** — "a 2-point median is realistic" has no support.
- The only number is ESA's "less than 4% of the total" at "the five most illuminated points" (114cd0dd:505–509): the easiest pixels, both tools on 240 m maps.
- V2 is at H = 0 against a 2016 map that predates the track adjustment, when height differences ran "∼0.4 m at the lowest slopes to ∼4 m at the highest" (56dae622:448). One metre across a 60 m pixel is a 0.95° change in a locally-set horizon, against a Sun that spans ±1.5° to ±6.5°. Mid-range pixels can move by tens of points, and stratifying puts 60 % of the sample there.
- Mazarico does not say how the surface-level horizon treats the observer's own pixel, so the "skip the first pixel" rule is an uncontrolled difference.
Fix, V2 row "Honest target": "Report per fifth and pooled: median, p90, mean signed error, Pearson r. **Pass: pooled median ≤ 5 points, |mean signed error| ≤ 2, r ≥ 0.9. Stretch: median ≤ 2, p90 ≤ 8.** These are targets: no published pixel-wise comparison at 0 m exists to calibrate them." Mirror this in docs/IDEA.md V2 and Decisions.
Same row, sampling: "2 000 from each fifth of the reference value" is ambiguous. Fix: "five bins of the value range (0–20 %, …, 80–100 %); if a bin has fewer than 2 000 pixels take all of them and print the count."

**F16 · MINOR (V3 is Stretch) · §3 V3 method, §0 A2** — NASA's `_MODEL.TIF` is a Lambertian radiance image (E37, E94). Otsu on it marks grazing sunlit slopes dark, as in the NAC; a geometric mask does not, so ours scores lower for a reason that is not an error.
Fix: "Render ours the same way: I = f · max(0, n̂·ŝ), n̂ from the DEM gradient, then Otsu. Compare the IoU of the dark class for all three."

**V4:** the rename to "reconstruction" is honest. Statement 3 must change (F9). Statement 2 is a model prediction with nothing outside to compare; keep "consistent with reports".

**V5:** case 5 must change (F6). Case 3: the PSR map was made with the Sun fixed at minimum declination in 720 steps (56dae622:863); its observer height and Sun model are not stated, so keep "avoid edge pixels" and add observer height to U15.

---

## 4. Pitfalls missing from §2 (check 4), ranked by chance of failing a V-test

| Rank | New pitfall | Fails | Test |
|------|-------------|-------|------|
| N1 | Wrong far-side test: correct code fails (F6) | V5 | Replace T-1d-1 |
| N2 | Strip quadrature normalised by πρ²: f = 1.0017, T-1b-1 fails (F18) | V5, energy | Normalise by Σ 2w; assert f == 1 for a clear disc |
| N3 | TDB table read as UTC in the browser (F3) | V1 in the app, silently | UTC grid; V1 through the table path |
| N4 | Horizon cut at 150 km on high sites (F2) | A1, V2 near the pole | T-1a-5 range ablation |
| N5 | V2 at H = 0: horizon set by the first one or two pixels, so DEM vintage and the first-pixel rule dominate (F15) | V2 | Store obstacle distance per sample pixel; report V2 separately for D < 180 m |
| N6 | DEM decode: whole-raster read (3.7 GB) or unsupported COG compression (F19) | Everything, at hour 1 | Open the 148 MB PSR file first |
| N7 | f32 in positions: 1.7e6 m in f32 resolves 0.125 m, 0.36° over 20 m | V2, V5 at H = 0 | `core` is f64 only; flat sphere at H = 0.1 m gives −0.01944° |
| N8 | Azimuth wrap at 0/360 in el_hor interpolation and Δaz (F5) | V1, timelines | Margin is continuous as the Sun crosses az = 0 |
| N9 | x/y swap survives the SP-01 check (F20) | V2, V3 | Seven-RoI and IM-2 checks in T-1a-2 |
| N10 | Pit model reported as an outcome (F9) | V4, demo | Report d_c* |
| N11 | Early exit on a summit site (F1) | V5 | Highest point of the tile: early exit on and off agree |

---

## 5. Scope (check 5)

Arithmetic: Must = 1.5 + 1 + 2.5 + 2.5 + 3 = 10.5 h; items 3, 4, 6, 9 add 5.5 h = 16.0 h. Person B's list sums to 15 h, not 16.

**F17 · MAJOR · §4** — the plan has no slack and no hand-off.
- Person A is at exactly 16.0 h with nothing for downloads, integration or debugging.
- B's panorama and timelines need the WASM API, which exists only after A's items 5, 7 and 8 (8 h). Nothing freezes the interface or supplies fake data.
- The ± band (item 3, 1 h) needs `LDEM_80S_20MPP_ADJ_ERR.TIF`, which is **3.7 GB** (583c4107:173–175), larger than the DEM (2.6 GB).
- Item 7 (2.5 h) includes decoding a cloud-optimized GeoTIFF with an untested crate (F19).
Fix, insert before the table: "**Hour 0 (both, 30 min).** Start all downloads in the background (DEM 2.6 GB, far field 210 MB, PSR 148 MB, AVGVISIB pair). Freeze the `core` → WASM API (names, units, array layouts) and commit one JSON fixture with a fake horizon and a fake 30-day timeline; B builds against it until A's engine lands. **Order for A:** projection + T-1a tests (no data) → ephemeris table + V1 fixtures (no DEM) → DEM crop → horizon + disc + timeline → state of charge → WASM → V5. **Error raster:** do not download `_ERR` (3.7 GB) for the Should tier; use σ_z = 0.5 m (E23a) labelled 'assumed', and the raster only in Stretch." Change the sentence after the table to: "Must is 10.5 h. Items 3 and 4 bring person A to 13 h; the remaining 3 h are slack. Items 6 and 9 start only after Must is deployed."

**F22 · MINOR · §4 "docs/IDEA.md diff list (not applied)"** — docs/IDEA.md already contains every item (its Decisions say "Nov 10: science review … applied"), so the heading is stale. One row came out garbled: Build roles, Geodesist reads "DEM ingest, curvature, error propagation, V3 | LOLA PDS/PGDA, error propagation, V2". Fix: heading → "docs/IDEA.md diff list (applied Nov 10)"; Geodesist row → "DEM ingest, curvature, error propagation, V2 | LOLA PGDA GeoTIFFs, first-order error propagation". STACK and IDEA say the DEM is 2.7 GB; the page says 2.6G.

Dependency order otherwise sound: the "if deployed by Sat 12:00 slips" cut list is in the right order, and moving the survival headline to Must is justified by its 1.5 h cost.

---

## Open items this review did not close
- W1b, W5 (ANISE `LT_S`) and W6 were not re-fetched; U14 still needs docs.rs for 0.10.6.
- PA→ME angles are not in the corpus (the matrix is missing from the extracted Park 2021 text near 7b1b2250:471). U4 stands; P8's "differs by more than 0.01°" is plausible but unquoted.
- Observer height of AVGVISIB (U1): the label has no height line. PGDA product 69 mentions "height above the surface (for potential rovers or habitats)" in a sentence the fetch tool would not quote in full; read that page by eye on Nov 14.
- The 18.6-year span start (U2) and the horizon extent behind AVGVISIB (U3) remain open.

Tools used: `audit.mjs` (ledger grep and context), `verify.mjs` (test vectors), both in the session scratchpad; not project code.
