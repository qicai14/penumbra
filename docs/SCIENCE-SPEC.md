# SCIENCE-SPEC — Penumbra

Progress: §0 done · §1 done · §2 done · §3 done · §4 done · §5 done · §6 done (ledger audited by grep, Nov 10). Review complete.

Nov 10: independent review (docs/SCIENCE-SPEC-REVIEW.md) applied, F1–F22.

Written Nov 10 2026 by the Fable science review. `science-core` and `validator` implement from this on Nov 14.

## How to read this
- **[V]** = a verbatim quote exists in `docs/research/lunar-landing/corpus/`. Each is a row in the §6 ledger with an ID (E04), the file (first 8 characters of the corpus filename), the line and the quote. Check with `grep -nF "quote" docs/research/lunar-landing/corpus/<file8>*.txt`.
- **[W]** = a verbatim quote from a web document that is not in the corpus (W-rows in §6, with URL). Counted separately; a reviewer cannot grep these.
- **[U]** = nothing quotable. Each is numbered (U1…) in §5 with the source that would settle it.
- **[D]** = derived here from [V] inputs by geometry or arithmetic. The derivation is shown. It is not a quoted claim and is not counted.
- Quotes are single-line fragments because the corpus text is wrapped at about 200 characters.

Sections: 0 Ambition check · 1 Equations · 2 Pitfalls · 3 V1–V5 thresholds · 4 Scope · 5 Open facts · 6 Evidence ledger and count

---

## 0. Ambition check

The plan already aims at the right outcome (a survival answer, with outside judges). Three changes make it better without making it bigger. Each one replaces something.

| # | Proposal | Outside judge | Hours | Replaces |
|---|----------|---------------|-------|----------|
| A1 | **Benchmark the headline metric, not only the average.** The headline is the longest night and time to first shadow, so check that against published numbers. **Barker Site 1**, Δz = 5 m, 2024-01-01 to 2026-01-01, hourly. For each RoI average our 20 m pixels within sqrt(Area/π) of the centroid. Published case-A values, RoI 1–7: LCIP-1 = 215.79, 216.28, 216.89, 231.20, 218.98, 220.98, 119.07 days (b5251ef3:682–688); LCSP-99 = 5.29, 7.31, 8.41, 4.00, 4.52, 4.16, 5.42 days (b5251ef3:706–712). Pass if LCSP ≤ LCSP-99 + 1 day and LCIP ≥ LCIP-1 − 10 days for at least 6 of 7 RoIs. This is one-sided: it catches a model that is too dark, not one that is too bright. **Mazarico SP-01**, H = 10 m, one cycle from 2020 (assumed): evaluate every 20 m pixel within 150 m of (−11.31, −12.26) km; pass if 2.75 days lies inside the range of longest total night (f = 0) over those pixels. Report both the f > 0 and the f = 1 day lengths. An average can pass while every sunset is hours wrong. | Barker 2021 Table 2; Mazarico 2011 SP-01 | 1 | DSN-complex visibility (Should) |
| A2 | **Score V3 against NASA's own simulation instead of an absolute IoU.** PGDA ships a simulated image for each map-projected NAC (E94, E95). Compute IoU(NAC, ours) and IoU(NAC, NASA model) with the same threshold; pass if ours is within 0.05 of NASA's. Both rasters are already on the 20 m DEM grid, so a plain threshold is enough. | Barker 2023 NAC + `_MODEL.TIF` pairs | 3 | SAM 3.1 and the Meta API key (all of it) |
| A3 | **Say what the DEM cannot see.** IM-2 ended in a crater about one DEM pixel wide (W6). ESA found the same limit: features "smaller than the LOLA resolution (here 40 m) are not captured" (E114). Add a user-set "local pit" (diameter, depth) that raises the horizon floor, and show both answers: regional terrain says X, a pit says Y. A planner trusts a tool that states its resolution limit. | The IM-2 outcome (E73, E74) | 1 | "Our number matches what happened" as a demo claim |

Why V1 is no longer a differentiator: a competing repo already reports "largest error 0.006°" against Horizons (E77), another logs a Fincannon reproduction (E78), and a third integrates the solar disc by strips against a terrain horizon (E120). The challenge statement itself asks for Sun and Earth positions "relative to the horizon" (E79b) and suggests real topography and libration (E79c, E79d). So V1 and a terrain horizon are table stakes. V2 (pixel-wise against PGDA), A1 and A3 are where an outside judge can separate us.

---

## 1. Equations

Symbols used throughout:
- R = 1 737 400 m, the DEM reference sphere (E30, E31a).
- Body frame = MOON_ME. With the DE440 kernels this is `MOON_ME_DE440_ME421` (E40, E43), which "has been defined to align closely with the DE421 ME reference frame" (E41). The DEM is in "the MOON_ME frame of the JPL ephemeris DE421" (E32, E32b), accurate to "a few meters" (E33b). Horizons places lunar sites in the "mean Earth of DE421" (E46). So the DEM, our ephemeris and the V1 judge share one frame. No rotation between them is needed.
- (x, y) = DEM map coordinates in metres, origin at the south pole. h = DEM height in metres (E133) above R [U6]. H = mast or panel height above the ground.
- All angles in radians inside `core`; degrees only at the API edge.

### 1a. Horizon elevation with curvature

There is no atmosphere, so the horizon is terrain or the lunar limb, with no refraction (E53) and no twilight.

**Projection (south polar stereographic, true scale at the pole, sphere R).** [E30, E31a, E31b]

Inverse, map → unit vector in MOON_ME, with no trigonometry:

    t = (x² + y²) / (4R²)
    û(x, y) = ( y/R , x/R , −(1 − t) ) / (1 + t)

Forward, unit vector → map, also with no trigonometry:

    x = 2R · u_y / (1 − u_z) ,   y = 2R · u_x / (1 − u_z)

Forward from (lat φ, east lon λ), for the API edge:

    r = 2R · tan( (90° + φ) / 2 )        (φ negative in the south)
    x = r · sin λ ,  y = r · cos λ

- [D] Derived from the colatitude relation c = 2·arctan(r / 2R) (E31b), with longitude 0 along +y and 90°E along +x.
- Orientation check [D]: Mazarico's SP-01 at 222.69°E, 89.45°S (E85) maps to (x, y) = (−11.31, −12.26) km with these formulas. Barker 2021's Site 1 points on the same ridge are at X = −11.58 km, Y = −12.46 km (E129, E130). They agree, so this orientation is right for the PGDA products. The sibling illumination raster also states `CENTER_LONGITUDE = 0` (W3b). Still read the GeoTIFF tags on Nov 14 [U5].
- Local scale factor k = 1 + t: 1.0000 at the pole, 1.0077 at 80°S [D]. Map distance is not ground distance. Working in 3D vectors removes the issue.
- Pixel convention [U5]: GeoTIFF is normally pixel-is-area, so pixel (col, row) has its centre at x = x₀ + (col + 0.5)·Δ, y = y₀ − (row + 0.5)·Δ, where (x₀, y₀) is the upper-left corner from the tiepoint tag. Read `GTRasterTypeGeoKey`; do not assume.
- Extent: the 20 m product covers ± 304 km (E135). A 240 m product to 60°S exists for the far field (E89), and an 80 m one if a middle tier is needed.

**Positions.**

    ground point:  p = (R + h(x, y)) · û(x, y)
    observer:      o = (R + h(x_s, y_s) + H) · û_s          (h bilinear at the site)

**Local frame at the observer** (spherical normal, which is what "plane perpendicular to local zenith" means on a sphere, E51):

    up    = û_s
    east  = ẑ × û_s / |ẑ × û_s|        ẑ = (0, 0, 1), the body north pole
    north = û_s × east

At the pole itself ẑ × û_s = 0. Convention there: north = +X (toward lon 0), east = +Y. V5 tests this.

**Elevation and azimuth of any point p (terrain, Sun or Earth)** from the observer:

    d  = p − o
    el = asin( d·up / |d| )
    az = atan2( d·east , d·north )      clockwise from north (E50)

Curvature is exact in this form because both points are 3D vectors. Mazarico states that curvature cannot be ignored for the lunar poles (E01, E84). Barker 2021 also works "in Cartesian coordinates" (E27).

**Closed form along one azimuth** [D]: observer radius r_o = R + h_s + H, terrain radius r_t = R + h, central angle θ = s / R for ground distance s:

    tan(el) = ( r_t·cos θ − r_o ) / ( r_t·sin θ )
    el ≈ (h − h_s − H) / s − s / (2R)      (small angles)

The drop s²/2R is 0.7 m at 1.6 km, 720 m at 50 km and 6.5 km at 150 km [D].

**Horizon profile.** For each of N_az = 720 azimuths at 0.5° (E04, E24b, E64), march outward along the great circle and keep the maximum el. Store el_hor[i] and the distance D[i] of the obstacle that set it (needed in 1f and for the UI). Between azimuths interpolate linearly (E05). Mazarico found 36 azimuths gave visible artefacts (E122).

- March along the great circle, not along a straight map line: û(θ) = û_s·cos θ + (north·cos az + east·sin az)·sin θ, then the trig-free forward projection gives (x, y) for the DEM lookup, and the closed form gives el. Reason [D]: a straight line on a stereographic map is not a great circle. Its curvature is r/(2R²), so at 5° from the pole it ends 280 m (0.11° of azimuth) off the great circle after 150 km, and twice that at 80°S. Mazarico used a gnomonic projection to avoid this (E02).
- Range: no fixed cut-off. March until the early-exit bound fires or the far-field tile ends. Crop the far field to site + 200 km at least (Bryant, E66/E100); Barker 2023 uses 310 km (E93).
- Early exit: stop an azimuth at distance s when B(s) < current max, where B(s) = atan((r_max·cos θ − r_o)/(r_max·sin θ)), r_max = R + h_max, θ = s/R, **and only if r_max > r_o**. If r_max ≤ r_o, do not exit early.
- Resolution tiers: Barker 2021 uses 5 m within 5 km, 80 m for 5–100 km and 240 m beyond (E25). Barker 2023 starts "with 20 m pixel" out to 7 km and coarsens to 480 m (E35, E92). ESA samples every 20 m within 3 km and every 2 km beyond 75 km (E108). Ours: 20 m near field out to at least 7 km, 240 m beyond. Step = one pixel of the tier in use. DEM sampled bilinearly. Test T-1a-4 decides whether an 80 m middle tier is needed.
- Skip the first pixel so the observer's own cell is not an obstacle.
- Tile encoding [D]: int16 at 0.5 m steps (the PDS convention, E31c) is too coarse for the near field. A 0.5 m step between neighbouring 20 m pixels is a 1.4° slope error. Use a per-tile offset and a scale of 0.25 m or finer. That keeps quantisation (0.07 m RMS) below the DEM's own error (E23a).
- Read the DEM tile by tile; never decode the whole raster. At hour 0, open `LPSR_80S_20MPP_ADJ.TIF` (148 MB, same grid) with the `tiff` crate to prove the compression and predictor are supported before the 2.6 GB download finishes. If not, convert once with `gdal_translate -co COMPRESS=NONE` on any machine that has GDAL.

Tests:
- T-1a-1 Flat sphere (h = 0), H = 2 m: el_hor = −acos(R / (R + H)) = −0.0869° at every azimuth [D]. Bryant prints the same dip formula (E68).
- T-1a-2 Round trip: forward then inverse projection returns (φ, λ) to 1e-9 rad. SP-01 must map to (−11.31, −12.26) km and land on a ridge crest in the DEM, not in a crater. This catches a swapped x/y or a flipped longitude. All seven Barker RoI centroids (X/Y cells at b5251ef3:555–561 and 563–569) must be higher than the mean DEM height on a 200 m ring around them. Also assert that the IM-2 site (84.7906°S, 29.1957°E) maps to (+77.1, +138.0) km; a swap moves it 86 km.
- T-1a-3 Synthetic cone of height 1000 m at 50 km, at an azimuth that is not along a map axis: the marcher must match the closed form to 0.002°.
- T-1a-4 Tier ablation, native only: horizon from the shipped tiles (two tiers, int16) against the full 20 m float DEM at 5 points, at H = 0 and H = 2 m. Report the max difference. If it is above 0.05°, add the 80 m tier or widen the 20 m tile.
- T-1a-5 SP-01, H = 10 m: horizon with range 150 km vs tile edge; report the largest el difference and the change in longest night.

### 1b. Visible fraction of the Sun's disk

Sun angular radius ρ = asin(R_sun / d_sun) with R_sun = 696 000 km (E117). At 1 AU this gives a diameter of 0.533°, which matches the corpus values 0.53° (E56a, E105) and 0.533° (E61). ρ varies from 0.261° to 0.272° over the year [D]: the Earth–Sun range 147.1–152.1 million km, widened by the Moon's ±384 400 km because the table is Moon-centred (0.2615°–0.2718°); recompute it at each step, as Mazarico does.

Work in the tangent plane centred on the Sun: ξ = (az − az_sun)·cos(el_sun), η = el − el_sun. For a strip at ξ the disk spans η ∈ [−w, +w] with w = sqrt(ρ² − ξ²). The horizon there is g(ξ) = el_hor(az_sun + ξ / cos el_sun) − el_sun.

    f = (1 / πρ²) · ∫ from −ρ to ρ of  clamp( w − g(ξ) , 0 , 2w ) dξ

- Evaluate with 32 midpoint strips. The disk is about one azimuth bin wide, so the horizon inside it is one or two straight segments (E05). Normalise by the same quadrature: f = Σ clamp(w_i − g_i, 0, 2w_i) / Σ 2w_i. This gives exactly 1 for a clear disc and exactly 0 for a hidden one.
- f is the geometric fraction, as in Mazarico: "the ratio of the solar disc" occulted by the horizon (E08). Barker 2021 adds "limb darkening at 550" nm (E26). We do not; state it under Limitations. Simpler models exist and are weaker: ESA's ICAT uses only the vertical fraction (E61), Fincannon only the horizontal one (E119, E56b), and a point Sun is "roughly equivalent to having a 50% threshold" (E110, E111).
- Two outputs per time step, and they must not be confused (§2 P1). Mazarico defines both (E81, E82):
  - **illumination** = f, in [0, 1]. Used for power.
  - **visibility** = 1 if f > 0, else 0. This is what the PGDA `AVGVISIB` rasters average (W1).
  - At SP-01 they differ by 3.65 points: 89.01 % illumination, 92.66 % visibility (E16d, E16e).

Tests:
- T-1b-1 Flat horizon at height u·ρ relative to the Sun's centre: f = ( acos(u) − u·sqrt(1 − u²) ) / π [D]. u = 0 gives 0.5; u = −1 gives 1; u = +1 gives 0. Strip integration must match to 2e-3 with 32 strips.
- T-1b-2 Horizon above the whole disk gives exactly 0 (not 1e-9). The PSR tests in V5 rely on this.

### 1c. Topocentric Sun and Earth az/el in MOON_ME

Use DE440, not DE441: `DE441 is less accurate than DE440 for the current` era (E141).

For each UTC instant t:

1. Convert UTC → TDB with hifitime. Never hand-code the offset [U12].
2. From ANISE, with observer = Moon centre, get the **apparent** position (light time plus stellar aberration) of the Sun, r_sun(t), and of the Earth, r_earth(t), in km, in J2000 axes. ANISE has this as `Aberration::LT_S` (W5). Then rotate J2000 → MOON_ME at time t. Doing the rotation as a separate step avoids any doubt about which epoch a body-fixed transform uses [U14]. Horizons' quantity 4 is "Adjusted for light-time," gravitational deflection and stellar aberration (E49, E52).
3. Site vector s = (R + h + H)·û(φ, λ), in km.
4. d = r_body − s, then az/el from the 1a formulas. No refraction: "No refraction effects are modeled for non-Earth sites" (E53).

Why each correction is needed [D, using physical constants that are not in the corpus, U17]:
- Stellar aberration on the Sun is v/c ≈ 30 km/s ÷ 299 792 km/s = 1.0e-4 rad = 0.0057°. Skipping it uses up most of the V1 budget.
- For the Earth, light time (1.3 s × 30 km/s = 38 km of Earth motion, 0.0057°) and stellar aberration (0.0057°) nearly cancel, leaving about 0.0002°. Apply both or neither. Applying one alone gives a 0.006° error.
- UTC versus TDB: the offset is about 69 s. The Sun moves its own width in about an hour (E107), which is 0.53°/h, so 69 s is 0.010° of azimuth. That alone is the whole V1 threshold.
- Evaluating aberration at the Moon's centre instead of at the site is fine: the site adds under 5 m/s and 6 ms.
- Gravitational deflection is below 0.0005° here. Ignore it.
- Topocentric shift of the Earth: 1737.4 / 384 400 rad = 0.26°. Of the Sun: 0.0007°. Both come out of step 4.

MOON_PA → MOON_ME: the kernels do it. Load the binary PCK together with the lunar frame kernel (E44); the generic names "are simply MOON_PA and MOON_ME" (E43), and the frames are specific to one ephemeris (E45d). The ME frame is a fixed rotation from the PA frame (E96). The three angles and the size of the offset are not in the corpus [U4]. Do not use IAU_MOON: "cannot be used to reference high-accuracy lunar orientation data" (E42a, E42b). The PCK covers 1549 onward (E45e), so 2026–2030 and the 18.6-year V2 span are inside it.

**Shipped table.** Store Moon-centred apparent r_sun and r_earth in MOON_ME, f64 km, on a fixed **UTC** grid: rows are whole UTC hours from 2026-01-01T00:00:00Z, built natively with hifitime. The browser never converts time scales. The browser interpolates linearly and applies steps 3–4.
- [D] Size: 5 years hourly = 43 825 rows × 48 bytes = 2.1 MB. At 10 minutes it is 12.6 MB, not the 3 MB in docs/IDEA.md.
- [D] Hourly is enough: the Sun's direction turns about 0.51° per hour (E107), so the direction error of linear interpolation is about 0.0001° (libration acceleration ≈ 1e-12 rad/s² × (3600 s)²/8); the chord shortens |r| by 1e-5, which is harmless. Timelines are still evaluated every 10 minutes from the interpolated vectors.
- Test T-1c-1: table + interpolation against native ANISE at 1000 random instants, max error under 0.001°.

**V1 Horizons request** (fixtures saved as JSON; tests never call the network):
- `CENTER='coord@301'`, `COORD_TYPE='GEODETIC'`, `SITE_COORD='<E-lon deg>,<lat deg>,<h km>'` (E48a, E48b; the manual's own lunar example is E48c).
- `COMMAND='10'` and `'399'`, `QUANTITIES='4'`, `TIME_TYPE='UT'` ("After 1962, UT means UTC", E54), `ANG_FORMAT='DEG'`, `EXTRA_PREC='YES'` (E55, cited only for the parameter's existence; U13 covers the rest).
- h is in km "above reference ellipsoid". An example header prints the Moon's radius as 1737.400 km (E97); confirm it in the first fixture's header [U13]. Send h = (DEM height + H) / 1000.
- "The lunar PA frame is not currently available in Horizons" (E47), so there is no frame setting to get wrong on their side.

### 1d. Earth elevation margin

    margin(t) = el_earth(t) − el_hor( az_earth(t) )
    contact   = margin(t) ≥ m_min

- m_min = 2° by default. De Rosa uses the same: "imposing a 2° minimum" elevation (E62), and found it cost at most 5 % of availability. Label it "assumed" in the UI and make it a preset field.
- Earth's angular radius ρ_e = asin(6378.137 km / d) ≈ 0.95° (E98, E103). Three definitions exist and they differ by up to 1.9° of elevation:
  - PGDA and Mazarico: visible "if any fraction of" the disc is (E12, W1b). Use this one for V2-Earth.
  - Bryant's strict form needs the whole disc clear (E67).
  - Ours for comms: centre + m_min.
- Expected behaviour, for sanity checks: at the poles the Earth's centre swings about ±6.5° in elevation each month (E106), within libration limits of 8.16° in longitude and 6.87° in latitude (E101, E102). Contact windows near 85°S are about 14 days, up to about 19 (E113).
- DSN-complex visibility is cut (§0 A1, §4).

T-1d-1: at a far-side site (lat −60°, lon 180°) on a flat sphere the Earth's upper limb stays below −20° at all times in 2026–2030 [D: centre peaks at −23.4°]. T-1d-1b: at (lat −85°, lon 180°) the Earth's centre elevation peaks between +1.0° and +1.7° and is negative more than half the time.

### 1e. Battery state of charge

No source in the corpus prints an update equation (the rover papers define the state of charge only as stored energy). This is a plain energy balance [D]:

    S(t)      = S₀ · (1 AU / d_sun)²                          W/m²
    P_gen(t)  = S(t) · A · η · max(0, n̂·ŝ(t)) · f(t)           W
    E_{k+1}   = clamp( E_k + (P_gen − P_load)·Δt , 0 , E_max )  Wh, Δt in hours

- S₀ = 1366.1 W/m² (E57; another paper uses 1,367, E57b). f is the illumination fraction from 1b. ŝ is the unit vector to the Sun in the local east-north-up frame. n̂ is the panel normal in the same frame; a tipped lander is a rotated n̂.
- Sun-tracking panel: n̂·ŝ = 1. Vertical body-mounted panel facing azimuth a: n̂·ŝ = cos(el)·cos(az − a).
- Lethal event: the first step with E ≤ E_min. E_min is a preset field (default 0 Wh). Lethal time = t_k + Δt·(E_k − E_min)/(E_k − E*_{k+1}), where E* is the value before clamping. The simulation stops at the first lethal event. "Time to first lethal shadow" = that time minus landing time.
- Limitations: no charge or discharge efficiency, no separate night heater load.
- Δt = 10 min. Explicit Euler is exact when power is held constant over a step.
- Outputs to report beside it, with the published definitions: the longest continuous period when "no part of the Sun is visible" and the longest when any part is (E123).
- Hours are sums over the timeline steps, each step counting Δt: sunlit hours = Σ illumination × Δt (a half-hidden Sun counts half); contact hours = Σ contact × Δt; operable hours = Σ (illumination > 0 and contact) × Δt, the glossary's operable window (ARCHITECTURE §2.5). Operable hours stop at the lethal time (the lethal step counted only up to it; the full span when there is none), since a dead lander is not operable; sunlit, contact, longest-dark and longest-any-Sun hours stay full-span site statistics (E123), comparable to PGDA in V2.
- Presets. Cited: the rover paper's 10,000 Wh battery, 1.5 m² panel at 30 %, 220 W driving load (E58a–d); cell efficiency about 15 % for silicon (E59b). Not cited: any IM-1 or IM-2 value [U10]. Show uncited values as "assumed".
- Why darkness length is the design driver: Fincannon sizes storage from "73 to 117 hours" for the worst lunar day at a Shackleton rim site (E59).

Tests:
- T-1e-1: P_gen = 0, P_load = 100 W, E₀ = 705 Wh, E_min = 0: dead at 7.050 h ± 1e-9.
- T-1e-2 Energy is conserved: Σ(P_gen − P_load)·Δt equals E_end − E_start when no clamp is hit.

### 1f. Uncertainty from DEM error

What Barker 2021 does (E20–E22): smooth the DEM, add "artificial random fractal terrain" below 200 m baselines, resample at LOLA points "randomly perturbed" by the track errors, re-grid, and repeat for 100 clones. That needs the raw LOLA tracks and is out of reach in 16 hours.

Ours, in two steps [D]:

**Step 1, first-order propagation (Should).** For azimuth bin i with obstacle distance D[i]:

    σ_el[i] = sqrt( σ_z(obstacle)² + σ_z(observer)² ) / D[i]
    σ_t     = σ_el / | d(el_sun − el_hor)/dt |      at each sunrise or sunset

σ_z comes from `LDEM_80S_20MPP_ADJ_ERR.TIF`, the "surface height error (in meters)" (E34a). Typical sizes: 0.30–0.50 m RMS on the 5 m site DEMs (E23a); track errors 1.00/1.00/0.15 m for the 2023 maps (E33a). A 0.5 m error on an obstacle 1 km away is 0.03°, and at 10 km it is 0.003° [D]. At the pole the Sun moves about 0.5°/h in azimuth (E107). Compute the rate by central difference of m(t) = el_sun(t) − el_hor(az_sun(t)) across the crossing in the 10-minute timeline. Do not approximate it by 0.5°/h × slope: at 85°S the Sun's own elevation rate is up to 0.046°/h. On a horizon sloping 0.1° per degree, 0.03° is 35 minutes. On a flat horizon the time is ill-conditioned and the band is hours. Show the band, not a single time.

**Step 2, Monte Carlo on the horizon, not on the DEM (Stretch).** 100 clones (E22). Per clone: draw one observer-height error and one error per distinct obstacle (adjacent bins with D within 20 m share a draw); perturb el_hor[i] by (z_obs − z_site)/D[i]; rerun the timeline; report the 5th–95th percentile of each window boundary.

- Sanity check against Barker: at 5 m mast height the DEM "uncertainties have an almost negligible effect on the illumination" (E28). Our band at H = 5 m must be small too.
- Honest limit: this captures LOLA height error only. Terrain smaller than a 20 m pixel is not in the DEM or its error file (E114), and the DEM still contains a few spurious "flagpoles" and "potholes" (E91). De Rosa describes a 3σ worst case (E63) and notes it is too pessimistic for a 2 m array; offer it only as an optional upper bound.

**Local pit (A3)** [D]: for a lander at the centre of a crater of diameter D_c and depth d_c, with panel height H above the floor:

    el_pit = atan( (d_c − H) / (D_c / 2) )
    el_hor′[i] = max( el_hor[i] , el_pit )      for every azimuth

Default depth from ESA's degraded-crater model, an "11° slope from bottom to top of" the rim (E112): d_c = (D_c/2)·tan 11°. For D_c = 20 m that is 1.9 m. The Sun's highest possible elevation at latitude φ is 90° − |φ| + 1.54° (E104), which is 6.75° at IM-2's latitude [D]. Whether a panel is shaded depends on H: for D_c = 20 m, d_c = 1.94 m, el_pit is 11.0° at H = 0, 5.4° at H = 1 m, 2.9° at H = 1.44 m and −0.3° at H = 2 m. So report the pit depth d_c* = H + (D_c/2)·tan(el_sun + ρ) that would hide the whole disc, not an assertion of shadow.

---

## 2. Pitfalls, ranked

Ranked by how likely each is to fail a V-check, times how hard it is to notice. Each has the test that catches it.

| # | Pitfall | Fails | Size | Test that catches it |
|---|---------|-------|------|----------------------|
| P1 | **Comparing the wrong metric to AVGVISIB.** The PGDA raster is the share of time steps with any part of the disc visible (W1). Our natural output is the mean disc fraction. | V2 | 3.65 points at SP-01 (E16d, E16e), more than the ±2 point threshold | V2 uses `visibility`. Unit test: visibility ≥ illumination at every pixel. Scoreboard prints which metric was compared. |
| P2 | **Observer height.** AVGVISIB's height is not documented [U1]. Mazarico's maps "were obtained at surface level" (E10). Going to 10 m moves SP-01 from 89.01 % to 93.10 % (E16d, E16f). | V2 | 4 points for 10 m; 2 m is in between | Run V2 at H = 0 as declared. Run a 500-pixel subsample at H = 2 m as well and print both residuals. |
| P3 | **Missing stellar aberration, or light time without it.** | V1 | 0.0057° on the Sun; 0.006° on the Earth if only one is applied [D] | V1 at ≤ 0.002°. A second test asserts that turning aberration off moves the Sun by 0.005–0.006°, so the fixture is known to be sensitive. Note that a competitor's reported worst error is 0.006° (E77), which is the size of this effect. |
| P4 | **UTC treated as TDB.** | V1 | 0.010° of Sun azimuth [D] | V1. Plus a V5 year-boundary test: positions are continuous from 2026-12-31T23:59 to 2027-01-01T00:01 UTC. |
| P5 | **DEM coverage runs out.** The 20 m product ends at ± 304 km (E135). IM-1 at 80.13°S (E72a) is 300 km from the pole [D], so its northern horizon, where the Sun is, has 4 km of data. Missing terrain reads as "always sunlit". | V4, V5 | Total | Loader asserts that site + march range is inside the far-field tile and that no sample is nodata; otherwise return an error, never 0 m. Far field from `LDEM_60S_240MPP_ADJ` (E89). |
| P6 | **Terrain below DEM resolution.** IM-2's crater is about 20 m (W6); the DEM pixel is 20 m and effective resolution is about that at best (E39, E34b). | V4 | Decides the outcome | No test can catch it. Mitigation is A3 and a stated limit. |
| P7 | **Projection orientation, x/y swap, or half-pixel offset.** A mirrored map still looks plausible. | V2, V3, all | 10 m to hundreds of km | T-1a-2 (SP-01 lands on Barker's Site 1 ridge). V2 residual map: a half-pixel shift shows as paired bright/dark fringes along every shadow edge. |
| P8 | **Wrong frame: IAU_MOON or MOON_PA instead of MOON_ME.** (E42a, E47) | V1 | Above the V1 threshold; exact size [U4] | V1. Plus: compute once in PA and assert the result differs from ME by more than 0.01°, so the test proves it can see the difference. |
| P9 | **Azimuth sign or atan2 argument order; the frame at the pole.** | V1, V5 | 180° or mirror | At 85°S, 0°E the Earth must sit within 9° of azimuth 0 (E101). The Sun's azimuth must decrease with time in the south, at about 0.5°/h (E107) [D]. At the pole no output is NaN. |
| P10 | **Marching straight map lines instead of great circles.** | V2, V3 | 0.1–0.2° of azimuth at 150 km [D] | T-1a-3 at an off-axis azimuth. |
| P11 | **int16 tiles at 0.5 m, or a two-tier DEM seam at 7 km.** | V2 at H = 0 | Up to 1.4° between neighbouring pixels [D] | T-1a-4. |
| P12 | **Coarse or smoothed DEM biases illumination high.** At 40 m "illumination is generally even higher due to the smoothing of the horizon" (E71). Mazarico's table used a 240 m DEM (E06). | A1, V2 | Several points at H = 0 | Compare to Mazarico at 10 m, not at the surface. State the DEM each reference used. |
| P13 | **DEM noise spikes** ("flagpoles", E91) set a false horizon next to the observer. | V2, demo | Local | UI shows the obstacle distance per azimuth. V5 flags any site whose horizon comes from an obstacle under 3 pixels away. |
| P14 | **Earth-visibility definition** (any fraction, centre, or whole disc). | V2-Earth | Up to 1.9° of elevation (E103) | V2-Earth uses any-fraction (W1b); comms uses centre + 2°. Named differently in code. |
| P15 | **Horizons inputs**: altitude in metres instead of km (E48b), west-positive longitude, geocentric instead of site. | V1 | Large | Fixtures include one site at +5 km and one at −3 km altitude, and the request text is stored with each fixture. |
| P16 | **Extrapolating the ephemeris table** outside 2026–2030 or past the last row. | V5 | Silent | Out-of-range time returns an error. |

Libration needs no special handling: it is inside the kernels. The test for it is P9's Earth check plus the V5 bound that the Earth's centre, seen from the flat pole, never exceeds 6.87° + 0.26° in elevation (E102).

---

## 3. Are the V1–V5 thresholds reachable?

| # | docs/IDEA.md threshold | Reachable? | Honest target |
|---|-------------------|------------|---------------|
| V1 | ≤ 0.01°, aim ≤ 0.006° | Yes, with margin, if P3, P4 and P8 are handled. Frames agree to "a few meters" (E33b), which is about 1e-6 rad. | **≤ 0.002° worst case over 20 pairs.** Change the aim: 0.006° is what an aberration bug looks like. |
| V2 | median ≤ 2 points, p90 ≤ 5 points | Median yes. p90 is doubtful at the surface with a different DEM. | Report per fifth and pooled: median, p90, mean signed error, Pearson r. **Pass: pooled median ≤ 5 points, |mean signed error| ≤ 2, r ≥ 0.9. Stretch: median ≤ 2, p90 ≤ 8.** These are targets: no published pixel-wise comparison at 0 m exists to calibrate them. |
| V3 | IoU ≥ 0.85 on ≥ 3 images | Unknown. NASA's own render has about 30 % rms radiance error (E90) and misses sub-resolution features (E114). | **IoU within 0.05 of NASA's `_MODEL.TIF` IoU on 3 images, and ≥ 0.75.** Stretch tier. |
| V4 | Sun elevation and hours-to-shadow agree with mission reports | No. No report in the corpus gives a Sun elevation for either mission [U11]. | **Rename to "reconstruction".** Pass = the three statements below hold. |
| V5 | All red-team cases pass | Yes. | Same. Cases listed below. |

### V1 details
- 20 site/time pairs: the pole, one point 0.1° from it, SP-01, IM-1, IM-2, a far-side point, and region centres; times spread over 2026–2030, including one at each libration extreme found in the table and one on each side of a year boundary. Two sites with non-zero altitude (P15).
- Compare both azimuth and elevation, for the Sun and for the Earth. Azimuth error is compared as Δaz·cos(el).

### V2 details: what must match, and the cost
- **Second reference method:** a published simulation synthetically illuminates a 400 × 400 km, 20 m LOLA DTM with the horizon method (E136, E137). Use it to read our V2 misses against, not as a pass rule. SLDEM2015 (LOLA + Kaguya TC) covers only ±60° (E138), so it cannot replace the polar LDEMs.
- **Product:** `AVGVISIB_85S_060M_201608` and its `_EARTH` twin. 5058 × 5058 pixels at 60 m, int16 × 0.00004 (W3c).
- **Metric:** share of time steps with any part of the disc visible (W1, W1b; E81).
- **Span and step:** hourly over 18.6 years (W7). Barker 2023 describes the same recipe for this group: hourly "over the full lunar precession cycle of 18.6 yr", 165,000 steps (E93, E36). Mazarico's paper itself used 6 h over 1970–2044 (E09), so the PGDA product is a later, finer run. The start date is not documented [U2]. Use 2010-01-01 + 18.6 years and check on a subsample that shifting the start by 4.65 years changes the average by under 0.5 points.
- **Height:** 0 m, declared (P2) [U1].
- **DEM:** theirs is `LDEM_85S_060M`, a 60 m map from 2016 (W4). Ours is the 2023 20 m map. For V2 only, average our near field 3 × 3 to 60 m. The grids nest: 151.74 km ÷ 20 m = 7587 pixels exactly [D]. Whether their horizon used terrain beyond the 85°S cap is unknown [U3], so report pixels poleward of 88°S separately from the rest.
- **Sampling:** 10 000 pixels inside our tiles, five bins of the value range (0–20 %, …, 80–100 %); if a bin has fewer than 2 000 pixels take all of them and print the count. Without stratifying, permanently shadowed and flat pixels dominate and the median is trivially good.
- **Cost** [D, unmeasured, U18]: a horizon is 720 azimuths × about 700 samples ≈ 0.5 M bilinear lookups; ARCHITECTURE §2.2 budgets 0.68–1.38 M samples, 12–33 ms typical [est], against a 50 ms target. The timeline is 163 000 steps of one interpolation and one comparison, about 5 ms; the disc integral runs only when the Sun is within ρ of the horizon. About 15 ms per pixel per core, so 10 000 pixels take about 3 minutes on one core. The full raster is 25.6 M pixels, about 4 core-days: do not attempt it. The 18.6-year Sun and Earth vectors are 163 000 rows from ANISE, generated natively and not shipped.
- **How good can agreement be?** ESA's two independent tools agreed "always less than 4%" on accumulated illumination at their five best points (E65, E121). Mazarico saw about 3 % of pixels change when he coarsened the azimuth step (E15b). His own validation against images was visual (E15a). So ±2 points at every pixel is tighter than professionals report between their own tools, and that agreement was on the easiest points. No published pixel-wise comparison at 0 m exists, so the targets above are targets, not forecasts.
- **V2b, table check (A1):** **Barker Site 1**, Δz = 5 m, 2024-01-01 to 2026-01-01, hourly. For each RoI average our 20 m pixels within sqrt(Area/π) of the centroid. Published case-A values, RoI 1–7: LCIP-1 = 215.79, 216.28, 216.89, 231.20, 218.98, 220.98, 119.07 days (b5251ef3:682–688); LCSP-99 = 5.29, 7.31, 8.41, 4.00, 4.52, 4.16, 5.42 days (b5251ef3:706–712). Pass if LCSP ≤ LCSP-99 + 1 day and LCIP ≥ LCIP-1 − 10 days for at least 6 of 7 RoIs. This is one-sided: it catches a model that is too dark, not one that is too bright. **Mazarico SP-01**, H = 10 m, one cycle from 2020 (assumed): evaluate every 20 m pixel within 150 m of (−11.31, −12.26) km; pass if 2.75 days lies inside the range of longest total night (f = 0) over those pixels. Report both the f > 0 and the f = 1 day lengths.
- Mazarico's Table 3 averages come from the four-cycle 1970–2044 run (E09). The SP-01 day/night numbers appear to use one precession cycle from 2020 (E131, stated for Fig. 13 only) [U20].

### V3 details
- Inputs: four model images are listed (M1188426614, M1311409938, M1312308500, M139356343) with map-projected NAC partners (E94, E95). Barker reports "good overall correspondence" (E38), not a number.
- Blocker: the acquisition time of each image is not on the product page [U7]. Get START_TIME from each NAC's PDS label on Nov 14.
- Method: for V3 only, test each pixel with a single ray toward the Sun (the 1a closed form along one azimuth). This is 720 times cheaper than a full horizon. Threshold the NAC with Otsu's method; apply the same rule to NASA's model image. Render ours the same way: I = f · max(0, n̂·ŝ), n̂ from the DEM gradient, then Otsu. Compare the IoU of the dark class for all three.
- ESA measured about 5 m RMS shadow-edge registration against NACs with 40 m data (E115), so edges within one 20 m pixel are the expected floor.

### V4 details
What the sources do say:
- IM-1: landed at −80.1276°, 1.4367°E on Feb 22 2024 at 23:23:00 UTC (E72a, E72b), on a 12° slope, resting about 30° off (E75a). It operated 144 hours (E75b) and ended "as the sun moved out of view of the one illuminated solar panel" (E75c).
- IM-2: low Sun angle plus panel orientation (E73, E74), about 820 feet off target (E76), in a crater about 20 m wide (W6). Touchdown time is not in a primary source [U8].

Reconstruction statements we can test:
1. IM-1: the Sun's whole disc is above our terrain horizon at touchdown.
2. IM-1: over the next 144 hours the Sun stays above the terrain horizon while its azimuth moves by about 73° [D: 144 h × 0.51°/h]. That is consistent with a fixed panel losing the Sun, and it shows terrain shadow was not the cause.
3. IM-2: report el_sun at touchdown from regional terrain, and the pit depth d_c* = H + (D_c/2)·tan(el_sun + ρ) that would hide the whole disc from a panel at height H. State whether the ESA degraded-crater depth (1.9 m for 20 m) exceeds d_c*.

Label these "consistent with reports", not "validated". IM-1 needs the 60°S far field (P5); if that is not built, drop IM-1 and keep IM-2.

### V5 cases
1. The pole (−90°): no NaN; flat-sphere horizon is −0.0869° at H = 2 m at every azimuth.
2. Flat pole, 2026–2030: Sun elevation stays within ±1.6° (E104, E99); Earth-centre elevation within ±7.2° (E102).
3. A permanently shadowed floor: pick 20 pixels well inside regions flagged in `LPSR_80S_20MPP_ADJ.TIF` (E132); visibility must be exactly 0 at every hourly step of 18.6 years. Whether that map treats the Sun as a disc or a point is not stated [U15], so avoid edge pixels. This is sunlight only: earthshine is the dominant natural light during lunar night (E139) and is out of scope.
4. Year boundary and table ends (P4, P16).
5. Far side at 60°S: Earth never visible (T-1d-1).
6. DEM edge: a site 5 km inside the far-field boundary returns a coverage error, not a horizon (P5).
7. Energy tests T-1e-1 and T-1e-2.

---

## 4. Scope for 2 people × about 16 working hours

Score = judge impact (1–5) × confidence (0–1) ÷ hours. Hours are estimates for one person with Claude Pro doing test-first work.

**Hour 0 (both, 30 min).** Start all downloads in the background (DEM 2.6 GB, far field 210 MB, PSR 148 MB, AVGVISIB pair). Freeze the `core` → WASM API (names, units, array layouts) and commit one JSON fixture with a fake horizon and a fake 30-day timeline; B builds against it until A's engine lands. **Order for A:** projection + T-1a tests (no data) → ephemeris table + V1 fixtures (no DEM) → DEM crop → horizon + disc + timeline → state of charge → WASM → V5. **Error raster:** do not download `_ERR` (3.7 GB) for the Should tier; use σ_z = 0.5 m (E23a) labelled 'assumed', and the raster only in Stretch.

**Science core (person A, 16 h)**

| Rank | Item | Hours | Impact | Conf. | Score | Tier |
|------|------|-------|--------|-------|-------|------|
| 1 | Survival: state of charge, operable window, time to first lethal shadow (1e) | 1.5 | 5 | 0.85 | 2.8 | Must |
| 2 | V5 red-team tests | 1 | 3 | 0.9 | 2.7 | Must |
| 3 | Uncertainty step 1: ± band on each sunset (1f) | 1 | 3 | 0.7 | 2.1 | Should |
| 4 | IM-2 replay with the local-pit model (A3) | 1.5 | 5 | 0.6 | 2.0 | Should |
| 5 | Ephemeris table, topocentric az/el, V1 fixtures (1c) | 2.5 | 5 | 0.9 | 1.8 | Must |
| 6 | A1: Barker Site 1 and SP-01 longest-night check | 1 | 3 | 0.6 | 1.8 | Should |
| 7 | DEM crop to tiles, projection, tests T-1a (1a) | 2.5 | 5 | 0.8 | 1.6 | Must |
| 8 | Horizon engine, disc fraction, timelines; native + WASM (1a, 1b) | 3 | 5 | 0.8 | 1.3 | Must |
| 9 | V2 sample against AVGVISIB | 2 | 4 | 0.6 | 1.2 | Should |
| 10 | Site × date heatmap, one region | 2 | 4 | 0.5 | 1.0 | Should |
| 11 | JSON API + dataset | 1.5 | 2 | 0.7 | 0.9 | Stretch |
| 12 | Monte Carlo bands (1f step 2) | 1.5 | 2 | 0.6 | 0.8 | Stretch |
| 13 | IM-1 replay (needs the 60°S far field) | 1.5 | 2 | 0.5 | 0.7 | Stretch |
| 14 | DSN-complex visibility | 1.5 | 1 | 0.8 | 0.5 | Cut |
| 15 | V3 relative IoU (A2) | 3 | 3 | 0.4 | 0.4 | Stretch |
| 16 | SAM 3.1 shadow segmentation | 3 | 1 | 0.3 | 0.1 | Cut |

Items 5, 7 and 8 score lower only because they are large; everything else depends on them, so they go first. Must is 10.5 h. Items 3 and 4 bring person A to 13 h; the remaining 3 h are slack. Items 6 and 9 start only after Must is deployed. Items 10–13 and 15 happen only if something finishes early.

**App (person B, 16 h):** panorama with Sun and Earth tracks and timelines (4), pin and date controls with the survival headline (2), 3D terrain coloured by our model (4), deploy plus scoreboard page (2), demo video and project page (3).

**Re-ranked tiers**
- **Must:** pick a point and date; panorama with Sun and Earth tracks; a large live readout of Sun and Earth elevation with a countdown to the next sunset and Earthset; illumination and Earth timelines; the survival headline (operable days, time to first lethal shadow) for one preset; V1 and V5 passing; deployed. The survival headline moves up from Should: it is the hook and costs 1.5 hours.
- **Should:** IM-2 replay with the pit model; two site-date pairs side by side (A vs B); V2 sample; ± band on sunset times; A1 check; 3D terrain shaded by our model; heatmap for one region.
- **Stretch:** heatmaps for all nine regions; V3 relative IoU; Monte Carlo; IM-1 replay; share links; JSON API.
- **Cut:** SAM 3.1; DSN complexes; the 10 m DEM; boulder hazards; chatbot.

**If "deployed by Sat 12:00" slips, cut in this order**
1. 3D terrain with live shadows → a 2D map coloured by the same model output.
2. Nine regions → two: Mons Mouton (IM-2) and de Gerlache Rim 2 / the Shackleton ridge (SP-01, Site 1, and the V2 area).
3. In-browser horizon in WASM → horizons precomputed natively on a 100 m grid for those two regions (720 int16 per point, about 14 MB per 10 × 10 km). The panorama and timelines still work; only free clicking is lost.
4. V2 sample → the A1 table check alone.

**Compute notes** [D, unmeasured, U18]: a region heatmap at 100 m over 15 × 15 km is 22 500 points, about 6 core-minutes for horizons. Do not store horizons for heatmaps; store the derived numbers.

### docs/IDEA.md diff list (applied Nov 10)

1. **Hook:** "then checked against JPL Horizons and real LROC shadow images" → "then checked against JPL Horizons and NASA's published LOLA illumination maps". V3 is Stretch, so the hook must not depend on it.
2. **Problem, bullet 1:** "never rises more than about 1.5°" → "at the pole itself the Sun's centre stays within 1.54° of the horizon; at 85°S it reaches about 6.5°" (E104, E109).
3. **Problem, bullet 2:** "in a crater where rim shadow stopped battery charging" → "in a small crater, where a low Sun angle and the panels' orientation stopped charging" (E73, E74). "lasted about 1 day" → mark as unconfirmed [U19].
4. **Science model 1:** replace "10 m/px LOLA (Barker 2023 `LDEM_83S_10MPP_ADJ`) for the near field and coarser LOLA tiles out to about 150 km" → "20 m `LDEM_80S_20MPP_ADJ` out to at least 7 km, then 240 m `LDEM_60S_240MPP_ADJ` to at least 200 km, marched along great circles". This also removes the conflict with the Decisions section.
5. **Science model 1:** "Barker reports 82.9% at the surface versus 88.0% at 2 m on reference rims" → "Gläser et al. 2018 report average illumination of 69.5–82.9 % at the surface and 77.1–88.0 % at 2 m" (E70a, E70b). The numbers are not Barker's.
6. **Science model 2:** add "apparent positions (light time and stellar aberration), UTC converted to TDB by the library".
7. **Science model 3:** add "Two metrics are kept apart: illumination (disc fraction, for power) and visibility (any part of the disc, for comparison with PGDA)".
8. **Science model 5:** "Monte Carlo over the LOLA DEM error from Barker 2020/2023" → "sunset-time bands from the LOLA height-error raster, propagated through the horizon (after Barker 2021); Monte Carlo is Stretch".
9. **V1 row:** "aim ≤ 0.006°" → "aim ≤ 0.002°".
10. **V2 row:** → "PGDA `AVGVISIB_85S_060M_201608` (Sun and Earth), visibility metric, 0 m, hourly over 18.6 years, 10 000 stratified pixels: pooled median ≤ 5 points, |mean signed error| ≤ 2, r ≥ 0.9 (stretch: median ≤ 2, p90 ≤ 8). Plus the longest-night check against Barker 2021 Site 1 and Mazarico SP-01."
11. **V3 row:** → "PGDA map-projected NACs: our shadow IoU within 0.05 of NASA's `_MODEL.TIF` IoU on 3 images, and ≥ 0.75 (Stretch)". Remove SAM 3.1 here and in Decisions, Scope, Build roles and Missing access.
12. **V4 row:** → "IM-1 / IM-2 reconstruction: consistent with the published timeline (no Sun elevation has been published for either)".
13. **Data plan:** "Sun and Earth MOON_ME vectors every 10 min for 2026–2030 (about 3 MB)" → "hourly, f64, about 2.1 MB; interpolated to 10 min in the browser". "a 240 m far-field polar cap (about 12 MB)" → "a 240 m far field cropped from `LDEM_60S_240MPP_ADJ` to site + 200 km (about 20 MB raw for ±382 km)". Add "tiles are int16 with a per-tile offset at 0.25 m or finer".
14. **Scope tiers:** replace with the re-ranked tiers above.
15. **Demo story 2:** "Our number matches what happened" → "Regional terrain alone says the site had sunlight. Add the 20 m crater the map cannot see, and the panels are in shadow." **Step 3:** remove "get 9 operable days"; show whatever the model returns.
16. **Decisions:** "Dates 2026–2030 at 10-min steps" → "hourly table, 10-min timelines". V2 line → as item 10. V3 line → as item 11. DEM line: add `LDEM_60S_240MPP_ADJ` for the far field. Earth contact: keep 2°, cite De Rosa 2012. Remove "DSN visibility (Should)".
17. **Build roles:** Geodesist: "Monte Carlo, IoU against LROC NAC" → "error propagation, V2". Power + RF: drop "DSN visibility".
18. **Missing access:** remove the Meta Model API key line.
19. **Follow-up outside docs/IDEA.md:** `docs/STACK.md` mentions SAM 3.1 calls and int16 tiles; update both to match items 11 and 13.

---

## 5. Open facts

Each is [U]. None blocks the build; each has a default.

| # | Open fact | Default until settled | Source that would settle it |
|---|-----------|----------------------|-----------------------------|
| U1 | Observer height used for `AVGVISIB_85S_060M_201608` | 0 m | Not in `README_ILLUMINATION_2016.TXT`, the `.LBL`, or PGDA product 69 (all checked). Ask the PGDA contact (E. Mazarico), or infer from the H = 0 vs H = 2 m residuals. |
| U2 | Start date of AVGVISIB's 18.6-year span | 2010-01-01 | Same contact. The label's 2009–2013 dates (W2) look like the LOLA data span. |
| U3 | Whether AVGVISIB's horizon used terrain outside the 85°S map | Unknown; report ≥ 88°S separately | Same contact. |
| U4 | The three PA→ME rotation angles and the offset in metres | Not needed in code; kernels apply it | NAIF frame kernel `moon_de440_*.tf`, comment block. |
| U5 | GeoTIFF tags of `LDEM_80S_20MPP_ADJ.TIF`: tiepoint, pixel scale, pixel-is-area or point, nodata | Pixel-is-area, lon 0 along +y | **Read Nov 12** (all three DEMs): tiepoint upper-left (−304 000, +304 000) m, pixel 20 m, GTRasterType = pixel-is-area, no-data `nan`, Float32. Lon-0 axis still to confirm on the first decoded tile. |
| U6 | That TIF heights are relative to the 1737.4 km sphere | Yes | **Likely yes, unconfirmed (Nov 12):** Barker 2021 calls the heights "elevation above the reference sphere" (corpus 56dae622, l.448) and LOLA uses 1737.4 km as that radius (Smith 2010 via corpus 293cfb1e l.2718; Barker 2016 corpus 6b5bb6cc l.254); no source states it for this file. Low impact either way: a uniform datum offset of even 250 m changes R + h by 0.014 %, which moves a 200 km horizon angle by about 0.0005° and Earth parallax by under 0.0001°. Note the datum in DATA-NOTES during T06; no test needed. |
| U7 | Acquisition times of the four NAC images with model renders | None; V3 blocked | PDS label `START_TIME` of each NAC (LROC archive). |
| U8 | IM-2 touchdown time (UTC) and coordinates from a primary source | 2025-03-06, 84.7906°S 29.1957°E; time 17:28 UTC, marked assumed | Intuitive Machines release or NASA; LROC featured image 1408 for the coordinates (re-read the digits by eye). |
| U9 | IM-2 crater depth | ESA degraded-crater model (E112) | LROC NAC DTM or LROC post-landing analysis. |
| U10 | Battery and array values for IM-1 and IM-2 | Shown as "assumed" | Intuitive Machines press kits. |
| U11 | Sun elevation at touchdown for IM-1 and IM-2 | Our computed value, shown as a prediction | No published value found. |
| U12 | TDB − UTC and leap seconds | Library handles it | NAIF leap-seconds kernel; IERS Bulletin C. |
| U13 | Decimal places of Horizons az/el with `EXTRA_PREC`; the lunar radius used for sites | 1737.4 km | The header of the first V1 response. |
| U14 | ANISE 0.10.6: that `LT_S` exists in that version and how body-fixed transforms pick the epoch | Translate in J2000, rotate separately | **Closed Nov 12:** `Aberration::LT_S` exists (`astro/aberration.rs:73`). `Almanac::transform` translates with the aberration, then rotates at the same observation epoch, so translate-then-rotate matches it. |
| U15 | Whether Barker's PSR map treats the Sun as a disc or a point, and its observer height | Avoid PSR edge pixels in V5 | Barker 2023, PSR section (not stated in the text). |
| U16 | Depth-to-diameter ratio of small fresh craters | 11° degraded slope (E112) | Micro Cold Traps paper (the values are garbled in the corpus text; read the PDF). |
| U17 | Physical constants used in [D] estimates: c, Earth's orbital speed, Earth–Moon distance | Standard values | Any constants table; they affect estimates only, not code. |
| U18 | Per-pixel compute time (15 ms) and everything derived from it | Estimate | Measure the first native horizon on Nov 14. |
| U19 | How long IM-2 operated | "under a day", marked unconfirmed | Intuitive Machines mission summary. |
| U20 | Span of the SP-01 day/night numbers in Mazarico's Table 3 (E131 is stated for Fig. 13 only; Table 3 averages come from the 1970–2044 run) | One precession cycle from 2020, marked assumed | Mazarico 2011, Section 3.3 and Fig. 13 text, read by eye. |

## 6. Evidence ledger

Row format: `| ID | file8:line | quote | what it supports |`. Quotes are in backticks and are exact single-line substrings.

File key: 7b941912 Mazarico 2011 · b5251ef3 Barker 2021 · 56dae622 Barker 2023 · 583c4107 PGDA product page (Barker 2023 data) · ad6021a1 LOLA GDRDEM description · 7cae519b NAIF tutorial 23 lunar-earth PCK/FK · 0aca6ab2 moon_pa_de440_200625.cmt · ddf062ee NAIF lunar_kernels.txt · 7b1b2250 Park 2021 DE440 · 01433f98 Horizons manual · e1760284 Fincannon · 30275b36 Ross, Towers on the Peaks · a9f70387 Recovery Policies (rover) · 114cd0dd De Rosa 2012 · 018af83e Bryant 2009 · da3c1d6c Illumination Conditions at the Lunar Poles (Gläser 2018) · 94de7ee6 IM-1 NDL paper · b715c6e9 CBS (IM-2) · 09052fd5 Space.com (IM-2) · ce06497a SpaceNews (IM-1) · e4f8733d, e6dbbf4a, a97be361 competitor repos · dd988b75 challenge statement · 74310de4 LOLA DTM illumination and thermal simulation · 6b5bb6cc SLDEM2015 (Barker 2016) · 653692c5 Earthshine as an illumination source · 5c98fb8b Artemis III TOPSIS site ranking · c6ab698c DE440/DE441 description

| ID | file8:line | quote | supports |
|----|-----------|-------|----------|
| E01 | 7b941912:296 | `curvature of the planet could be ignored` | the flat-Earth shortcut (Dozier) ignores curvature; see E84 |
| E02 | 7b941912:181 | `polar gnomonic projection on which great circle` | Mazarico marches on a gnomonic map |
| E03 | 7b941912:207 | `reference radius of 1737.4 km` | reference radius of Mazarico's gnomonic projection, not of the heights |
| E04 | 7b941912:306 | `(Nth = 720, e.g. a spacing of 0.5` | 720 azimuths |
| E05 | 7b941912:400 | `straight segment between the maps at hi and hi+1` | linear interpolation between azimuths |
| E06 | 7b941912:182 | `A resolution of 240 m was used as a` | Mazarico DEM resolution |
| E07 | 7b941912:2223 | `to 150km away` | obstacle distance |
| E08 | 7b941912:398 | `as an extended source, and calculate the ratio of the solar disc` | Sun as extended disc |
| E09 | 7b941912:854 | `cles from 1970 to 2044, with a time step of 6 h` | Mazarico span and step |
| E10 | 7b941912:2239 | `they were obtained at surface level.` | average maps at 0 m |
| E11 | 7b941912:1336 | `and 10 m above the surface. Due to this height increase, their rank can change.` | Table 3 heights |
| E12 | 7b941912:884 | `we consider the Earth visible if any fraction of` | Earth visibility definition |
| E13 | 7b941912:404 | `from the DE421 ephemeris (Williams et al., 2008) and the lunar` | Mazarico ephemeris |
| E14 | 7b941912:405 | `orientation model by Seidelmann et al. (2007). We use the SPICE` | Mazarico orientation model |
| E15a | 7b941912:807 | `The agreement is visually excel` | Mazarico validation is visual |
| E15b | 7b941912:840 | `about 3% of the grid elements over the whole region` | azimuth-step sensitivity |
| E15c | 7b941912:821 | `showed good agreement` | vs ray tracing |
| E16a | 7b941912:1335 | `List of the 50 most illuminated spots in each pole.` | Table 3 caption |
| E16b | 7b941912:1749 | `222.69` | SP-01 longitude |
| E16c | 7b941912:1750 | `−89.45` | SP-01 latitude |
| E16d | 7b941912:1752 | `89.01` | SP-01 illumination %, surface |
| E16e | 7b941912:1753 | `92.66` | SP-01 visibility %, surface |
| E16f | 7b941912:1755 | `93.10` | SP-01 illumination %, 10 m |
| E16g | 7b941912:1756 | `95.83` | SP-01 visibility %, 10 m |
| E17 | 7b941912:2270 | `202 ± 7 days of total, uninterrupted sunlight, and 339 sunlit days` | SP-01 longest day |
| E18 | 7b941912:2295 | `total night at the surface is 5.88 days, but only 2.75 days at 10 m` | SP-01 longest night |
| E19a | 7b941912:53 | `Shackleton crater rim in the south polar region is sunlit continuously for 240 days` | abstract; its 240 d / 1.5 d appear nowhere in the body, so not a benchmark |
| E19b | 7b941912:996 | `The total PSR area is 12,866 km2 in the north and 16,055 km2 in` | PSR area |
| E20 | b5251ef3:819 | `artificial random fractal terrain (H ¼ 1) at baselines ≲ 200 m. This` | clone method |
| E21 | b5251ef3:823 | `sampled at the LOLA point locations, which are randomly perturbed by` | clone method |
| E22 | b5251ef3:831 | `In total, 100 such clones are` | 100 clones |
| E23a | b5251ef3:54 | `typical RMS values of ~0.30–0.50 m and ~1.5–2.5` | height and slope RMS |
| E23b | b5251ef3:332 | `~12–14 cm horizontally and ~2 cm vertically for most (~68%) of the` | track errors after adjustment |
| E23c | b5251ef3:406 | `The median RMS slope error is 1.90 deg while the MADb is` | slope error, site not stated |
| E24a | b5251ef3:1030 | `horizon method outlined in Mazarico et al. (2011)` | Barker 2021 method |
| E24b | b5251ef3:1031 | `720 sight lines` | 720 azimuths |
| E25 | b5251ef3:1034 | `km, the 80 m/pix original LDEM for distances 5–100 km, and the 240` | resolution tiers |
| E26 | b5251ef3:1038 | `treated as a 2-dimensional extended source with limb darkening at 550` | limb darkening |
| E27 | b5251ef3:1037 | `tions are performed in Cartesian coordinates at 1-h intervals. The Sun is` | Cartesian, hourly |
| E28 | b5251ef3:1245 | `uncertainties have an almost negligible effect on the illumination con-` | DEM error at 5 m |
| E29a | b5251ef3:547 | `Summary of illumination conditions for all 100 clones at Site 1 RoIs between Jan. 1, 2024 and Jan. 1, 2026` | Table 2 span |
| E29b | b5251ef3:1181 | `illumination ranges from 67% (RoI 7) to 71% (RoI 6), the 1st percentile` | Site 1 at 1 m |
| E30 | 56dae622:288 | `Scale is true at the pole, with a projection radius of 1737.4 km` | projection |
| E31a | ad6021a1:87 | `meters at a radius of 1,737,400 m, true at the pole` | projection radius |
| E31b | ad6021a1:90 | `2arctan(0.5r/1,737,400)` | colatitude formula |
| E31c | ad6021a1:274 | `HEIGHT = (DN * SCALING_FACTOR).` | PDS IMG scaling |
| E31d | ad6021a1:277 | `where OFFSET is the radius of a reference sphere` | PDS IMG offset |
| E32 | 56dae622:275 | `stereographic projection centered at the pole in the MOON_ME frame of the JPL ephemeris DE421` | DEM frame |
| E32b | 583c4107:25 | `south polar stereographic X/Y coordinates in meters and in the MOON_ME reference frame of the JPL DE421 ephemeris` | DEM frame, product page |
| E33a | 56dae622:448 | `we consider 1.00/1.00/0.15 m as a conservative estimate of the final track geolocation errors` | track errors |
| E33b | 56dae622:281 | `Coordinates in the MOON_ME frame of DE421 have an overall uncertainty of a few meters` | ME frame uncertainty relative to PA |
| E34a | 583c4107:34 | `LDEM_80S_20MPP_ADJ_ERR.TIF - LDEM surface height error (in meters)` | error raster |
| E34b | 583c4107:33 | `LDEM_80S_20MPP_ADJ_EFFRES.TIF - LDEM effective resolution (in meters)` | effective-resolution raster |
| E35 | 56dae622:851 | `starting with 20 m pixel` | nested tiers |
| E36 | 56dae622:863 | `simulation of southern summer in just 720 time steps instead of the 165,000` | 18.6 yr hourly = 165,000 steps |
| E37 | 56dae622:384 | `The surface is assumed here to be Lambertian with broadband visual albedo of 0.1` | how the NASA model images are rendered |
| E38 | 56dae622:422 | `good overall correspondence between the NAC- and LDEM-rendered images` | NAC vs model |
| E39 | 56dae622:271 | `Much of the Artemis zone` | effective resolution near 20 m |
| E40 | 7cae519b:448 | `Special case for DE440: ME frame is MOON_ME_DE440_ME421, which is` | DE440 ME frame name |
| E41 | 0aca6ab2:77 | `has been defined to align closely with the DE421 ME reference frame.` | DE440 ME ≈ DE421 ME |
| E42a | 7cae519b:110 | `CAUTION: ‘IAU_MOON’ cannot be used to reference high-accuracy lunar orientation data` | no IAU_MOON |
| E42b | ddf062ee:131 | `of the rather imprecise IAU_MOON frame.` | no IAU_MOON |
| E43 | 7cae519b:462 | `The generic frame names are simply MOON_PA and MOON_ME.` | aliases |
| E44 | 0aca6ab2:49 | `Applications using this file must also load the NAIF lunar frame kernel` | kernel pairing |
| E45a | 7b1b2250:467 | `Most of lunar cartographic products are defined relative to the DE421 mean-Earth/mean-rotation (MER) frame.` | cartography uses DE421 ME |
| E45b | ddf062ee:17 | `the Principal Axes (PA) frame, and the Mean Earth/Polar Axis (ME) frame.` | two frames |
| E45d | 7cae519b:445 | `These frames are unique to a particular JPL-produced planetary and lunar` | frames are DE-specific |
| E45e | 0aca6ab2:97 | `1549 DEC 31 00:00:00.000` | PCK coverage start |
| E46 | 01433f98:834 | `(or mean Earth of DE421 for the Moon)` | Horizons lunar site frame |
| E47 | 01433f98:1105 | `The lunar PA frame is not currently available in Horizons.` | Horizons has no PA |
| E48a | 01433f98:821 | `E-long - Geodetic east longitude (DEGREES)` | site input |
| E48b | 01433f98:823 | `- Altitude above reference ellipsoid (km)` | site input |
| E48c | 01433f98:551 | `348.8 degrees east longitude, -43.3 degrees latitude (south), and zero km altitude` | lunar site example |
| E49 | 01433f98:1845 | `4. Apparent azimuth & elevation (AZ-EL) :` | quantity 4 |
| E50 | 01433f98:1849 | `Azimuth is measured clockwise from north:` | azimuth convention |
| E51 | 01433f98:1853 | `Elevation angle is with respect to plane perpendicular to local zenith` | elevation convention |
| E52 | 01433f98:1846 | `Adjusted for light-time,` | corrections |
| E53 | 01433f98:865 | `No refraction effects are modeled for non-Earth sites.` | no refraction |
| E54 | 01433f98:994 | `After 1962, UT means UTC` | time scale |
| E55 | 01433f98:1678 | `Yes (~ 10^-5 arcsec` | EXTRA_PREC |
| E56a | e1760284:517 | `The algorithm allows for a disk modeled Sun with 0.53° angular width.` | Sun diameter |
| E56b | e1760284:521 | `vertical partial blockage was not implemented at this time.` | Fincannon limit |
| E57 | 30275b36:817 | `This value is 1366.1 W/m2, which is equivalent to` | solar constant |
| E57b | a9f70387:1350 | `constant total solar flux of 1,367 W/m2` | solar constant |
| E58a | a9f70387:1719 | `10,000 Wh` | rover battery capacity (Table 1) |
| E58b | a9f70387:1705 | `1.5 m2` | rover panel area (Table 1) |
| E58c | a9f70387:1707 | `30 %` | rover panel efficiency (Table 1) |
| E58d | a9f70387:1711 | `220 W` | rover driving load (Table 1) |
| E59 | e1760284:165 | `have, for the worse case lunar day, a 0.71 average illumination fraction and 73 to 117 hours required for` | storage sized by darkness |
| E59b | 30275b36:821 | `The average efficiency for present day silicon solar cells is ~15%` | cell efficiency |
| E60 | 114cd0dd:267 | `is 150 km, which has been estimated as the range at which the highest terrain point in the region` | horizon range (its reasoning is numerically wrong, see F2; not used for the range) |
| E61 | 114cd0dd:280 | `by assuming a Sun disc with an angular diameter of 0.533°` | Sun diameter |
| E62 | 114cd0dd:487 | `The effect of local terrain were also taken into account by imposing a 2° minimum` | 2° comms margin |
| E63 | 114cd0dd:395 | `A simple error model consists in adding 3 times the standard deviation` | worst-case DEM error |
| E64 | 114cd0dd:257 | `A 0.5° spacing between azimuth lines` | 0.5° azimuth |
| E65 | 114cd0dd:509 | `with a difference of always less than 4% of the total` | tool cross-validation |
| E66 | 018af83e:484 | `scanned along lines of azimuth for at least 200 km` | horizon range |
| E67 | 018af83e:876 | `the entire Earth disk, up to Earth’s South Pole, to not count as visible to the lunar terrain.` | strict Earth visibility |
| E68 | 018af83e:2663 | `arccosbR / (R + h) l (5)` | horizon dip formula (as printed) |
| E69 | 018af83e:2630 | `Solar illumination and Earth visibility metrics increase with increased altitude above the` | height effect |
| E70a | da3c1d6c:36 | `illumination levels range from 69.5% to 82.9%, with the highest illumination` | surface range |
| E70b | da3c1d6c:41 | `levels range from 77.1% to 88.0%` | 2 m range |
| E71 | da3c1d6c:281 | `culated illumination is generally even higher due to the smoothing of the horizon` | coarser DEM biases high |
| E72a | 94de7ee6:153 | `at the location -80.1276 deg N, 1.4367 deg E` | IM-1 coordinates |
| E72b | 94de7ee6:154 | `ary 22nd, 2024 at 23:23:00 UTC` | IM-1 touchdown time |
| E73 | b715c6e9:196 | `Given a low sun angle and the orientation of the lander's solar cells` | IM-2 power loss |
| E74 | 09052fd5:555 | `Athena's solar panels couldn't harvest enough sunlight in this orientation` | IM-2 power loss |
| E75a | ce06497a:113 | `The landing took place on a 12-degree slope, causing the lander to rest at an angle of about 30 degrees off the surface` | IM-1 slope and tilt |
| E75b | ce06497a:120 | `a total operational time on the surface of 144 hours, or six days` | IM-1 duration |
| E75c | ce06497a:120 | `as the sun moved out of view of the one illuminated solar panel` | IM-1 end cause |
| E76 | b715c6e9:208 | `some 820 feet from the intended target` | IM-2 miss distance |
| E77 | e4f8733d:497 | `Sun and Earth positions checked against NASA/JPL Horizons (DE441): largest error 0.006° over two years at four sites` | competitor V1 |
| E78 | e6dbbf4a:286 | `docs(validation): log Fincannon 2007 Shackleton-rim reproduction` | competitor validation |
| E79a | dd988b75:86 | `Your challenge is to create a tool or application that enables rapid evaluation of lunar landing site options.` | challenge objective |
| E79b | dd988b75:92 | `Display the positions of the Sun and Earth relative to the horizon at a selected landing site on a chosen date.` | challenge objective |
| E79c | dd988b75:115 | `Consider accounting for the Moon's actual topography, as terrain elevation can significantly affect horizon positions and visibility windows.` | challenge consideration |
| E79d | dd988b75:116 | `Consider whether your tool will account for factors like Earth's libration (wobble) or the Moon's orbital variations.` | challenge consideration |
| E80 | a97be361:510 | `For each site it works out when the Sun and the Earth are above the real terrain horizon.` | competitor scope |
| E81 | 7b941912:856 | `visibility (mean of the boolean Sun visible/not visible), average` | Mazarico metric 1: visibility |
| E82 | 7b941912:857 | `Sun illumination (mean of the normalized solar disc area visible),` | Mazarico metric 2: illumination |
| E83 | 7b941912:851 | `node precession cycles (each 18.6 years). We used the DE421` | 18.6-year cycle |
| E84 | 7b941912:297 | `in the case of (large) polar regions of the Moon, additional com-` | curvature is necessary at the poles |
| E85 | 7b941912:2267 | `Moon (SP-01: 222.69 E, 89.45 S). Fig. 14a–c shows the time series` | SP-01 coordinates |
| E86 | 7b941912:2292 | `to 204 ± 8 days, and there are 350 sunlit days. However, in Fig. 14c,` | SP-01 at 10 m |
| E87 | 7b941912:2296 | `while the longest partial nights are respectively 10.5 and 7.4 days.` | SP-01 partial nights |
| E88 | 56dae622:296 | `LDEM_80S_20MPP_ADJ 80` | Table 1 row: 80S product, range ± 304 km |
| E89 | 583c4107:41 | `LDEM_60S_240MPP_ADJ.TIF` | 240 m DEM to 60°S exists (210M) |
| E90 | 56dae622:428 | `has a typical rms error of ∼30%, which is reasonable given the assumed constant-albedo Lambertian surface.` | NASA model vs NAC radiance error |
| E91 | 56dae622:428 | `This figure also has examples of spurious noise points that remain in the new maps and that can appear as “flagpoles” (labeled A and B) or “potholes” (labeled C).` | DEM artefacts |
| E92 | 56dae622:853 | `out to a distance of 7 km and ending with 480 m pixel` | Barker 2023 tiers |
| E93 | 56dae622:855 | `at a distance of 310 km. The next step involves computing the illumination conditions at hourly time steps over the full lunar precession cycle of 18.6 yr at 20 m pixel` | horizon to 310 km; hourly over 18.6 yr |
| E94 | 583c4107:368 | `M*_20MPP_MODEL.TIF - Simulated image using the 20MPP LDEM (units of W/m^2).` | NASA model images |
| E95 | 583c4107:369 | `M*E_20MPP.TIF - 20MPP NAC image map-projected onto the 20MPP LDEM (units of I/F).` | map-projected NACs |
| E96 | 01433f98:1105 | `is a rotation by a fixed offet from the DE440 principal axis (or PA) system` | Horizons ME frame derives from DE440 PA |
| E97 | de84f202:1410 | `MOON (R_eq= 1737.400) km` | Horizons Moon radius (printed in an example header) |
| E98 | de84f202:1487 | `6378.137, 6356.752 km {Equator_a, b, pole_c}` | Earth radii |
| E99 | 018af83e:487 | `an elevation angle of –1.54 deg as seen from the lunar south pole. The tallest peak near the` | Sun declination limit |
| E100 | 018af83e:490 | `mountain must be over 7000 m tall in order to block the winter Sun. This gave a practical` | why 200 km is enough |
| E101 | 018af83e:587 | `maximum east–west longitude libration of 8.16 deg, and a north–south latitude libration of` | libration in longitude |
| E102 | 018af83e:588 | `6.87 deg.4 These libration values are the maximum possible librations, not the typical aver-` | libration in latitude |
| E103 | 018af83e:590 | `Earth’s disk subtends an angle of 1.9 deg as` | Earth angular diameter |
| E104 | 018af83e:596 | `The Sun moves across all degrees of longitude, but is confined to latitudes between 1.54 deg` | subsolar latitude range |
| E105 | 018af83e:598 | `disk subtends an angle of 0.53 deg as seen from the Moon. The motion of the northern and` | Sun diameter |
| E106 | 114cd0dd:111 | `due to the fact that the Earth centre follows a pattern of approximately ±6.5° in elevation every` | Earth elevation swing at the poles |
| E107 | 114cd0dd:258 | `fact that the Sun takes approximately 1 hour (the simulation step) to cover its own size, which in` | Sun azimuth rate; hourly step |
| E108 | 114cd0dd:260 | `i.e. points within 3 km are sampled every 20 m, points further than 75 km are sampled every 2 km` | ESA sampling along azimuth |
| E109 | 114cd0dd:268 | `has an elevation lower than the minimum Sun elevation (e.g. -6.5° at -85° latitude). Once the` | minimum Sun elevation at 85°S only; the 6.5° maximum is [D] (5° + 1.54°) |
| E110 | 114cd0dd:306 | `The biggest limitation of STK is that the Sun is simulated only as a point source. However, this is` | point-source tool |
| E111 | 114cd0dd:307 | `roughly equivalent to having a 50% threshold for the illumination/darkness transition, and the` | point Sun ≈ 50 % threshold |
| E112 | 114cd0dd:478 | `when considering: the low Sun elevation; a degraded crater model (11° slope from bottom to top of` | degraded crater slope |
| E113 | 114cd0dd:484 | `generally follow a regular pattern of ~14 days. As expected, areas around -85° latitude show a` | Earth windows |
| E114 | 114cd0dd:518 | `smaller than the LOLA resolution (here 40 m) are not captured (craters at (Xstereo, Ystereo) = (-11.1,` | sub-resolution craters missed |
| E115 | 114cd0dd:520 | `estimated in the range of 5 m (RMS), by systematically measuring on the NAC images the distance` | NAC registration error |
| E116 | 96f9d5d4:6 | `Peak near Cabeus B, Haworth, Malapert Massif, Mons Mouton Plateau, Mons Mouton, Nobile Rim 1, Nobile Rim 2, de Gerlache Rim 2, and Slater Plain` | the nine Artemis III regions |
| E117 | 0a1ca447:1889 | `equatorial radius (696,000 km)` | Sun radius |
| E118 | 018af83e:50 | `tion is exposed to the Sun for half of the 29.5-day synodic month, but a tall peak near the` | synodic month |
| E119 | e1760284:520 | `horizontal partial blockage of Sun, there may also be a vertical component of the blockage. Calculation of` | Fincannon disc model limit |
| E120 | a97be361:852 | `The visible fraction of each disk is integrated over 16 strips against the jagged horizon` | a competitor already integrates the disc by strips |
| E121 | 114cd0dd:505 | `distance between the two points returned by the different tools of only 690 m. For the five most` | ESA two-tool comparison |
| E122 | 7b941912:299 | `Secondly, the interpolation errors arising from the low number of` | 36 azimuths gave artefacts |
| E123 | b5251ef3:782 | `LCSP (bottom row) is the longest uninterrupted period when no part of the Sun is visible.` | longest-shadow definition |
| E124 | b5251ef3:1183 | `of LCIP ranges from 63 days (RoI 1) to 142 days (RoI 5), and the 99th` | Site 1 at 1 m: longest lit period |
| E125 | b5251ef3:1184 | `percentile of LCSP ranges from 13 days (RoI 4 and 6) to 22 days (RoI 3).` | Site 1 at 1 m: longest shadow |
| E126 | b5251ef3:1188 | `ranges from 83% (RoI 7) to 89% (RoI 1), the 1st percentile of LCIP ranges` | Site 1 at 5 m: average illumination |
| E127 | b5251ef3:1189 | `from 119 days (RoI 7) to 231 days (RoI 4), and the 99th percentile of` | Site 1 at 5 m: longest lit period |
| E128 | b5251ef3:1190 | `LCSP ranges from 4 days (RoI 4–6) to 8 days (RoI 3). The squares in` | Site 1 at 5 m: longest shadow |
| E129 | b5251ef3:555 | `−11.5825` | Site 1 RoI 1 X (km), Table 2 cell |
| E130 | b5251ef3:563 | `−12.4625` | Site 1 RoI 1 Y (km), Table 2 cell |
| E131 | 7b941912:2255 | `were done on one precession cycle starting in 2020. The surface-le-` | re-tagged [U20]: stated for Fig. 13 only, not the span of Table 3 |
| E132 | 583c4107:331 | `LPSR_80S_20MPP_ADJ.TIF` | PSR map for V5 |
| E133 | 583c4107:32 | `LDEM_80S_20MPP_ADJ.TIF - LOLA Digital Elevation Model (LDEM) giving surface height (in meters)` | DEM units |
| E134 | 583c4107:125 | `LDEM_80S_80MPP_ADJ.TIF` | 80 m product for a middle tier |
| E135 | 56dae622:296 | `± 304 20 LDEM_80S_40M` | Table 1: 20 m product range ± 304 km |
| E136 | 74310de4:33 | `simulated by synthetically illuminating the LOLA DTMs using the horizon` | an independent horizon-method illumination simulation (V2 reference) |
| E137 | 74310de4:67 | `A 400 x 400 km LOLA DTM with 20 m/pixel resolution` | that simulation used a 20 m DTM over 400 × 400 km |
| E138 | 6b5bb6cc:57 | `covering latitudes within ±60` | SLDEM2015 stops at ±60°, so it is not a polar DEM option |
| E139 | 653692c5:76 | `night it is by far the dominant source of natural illumination` | earthshine lights the night side; V5 PSR = 0 is sunlight only |
| E140 | 5c98fb8b:22 | `TOPSIS algorithm. By integrating topographic, illumination, and mineralogy data` | prior Artemis III site ranking (multi-criteria), the closest prior art to our ranking |
| E141 | c6ab698c:53 | `DE441 is less accurate than DE440 for the current` | DE440 is the right ephemeris for 2026–2030 |

Web rows (NASA documents fetched Nov 10 by a subagent with curl; not in the corpus, so not grep-checkable there):

| ID | URL | quote | supports |
|----|-----|-------|----------|
| W1 | http://imbrium.mit.edu/EXTRAS/ILLUMINATION/IMG/AVGVISIB_85S_060M_201608.LBL | `indicate the percentage of timesteps where each pixel was sunlit (by any fraction of the solar disc).` | AVGVISIB is a binary any-fraction average |
| W1b | …/AVGVISIB_85S_060M_201608_EARTH.LBL | `(by any fraction of the Earth disc).` | Earth variant |
| W2 | same .LBL | `START_TIME = 2009-07-13T17:33:17.246` / `STOP_TIME = 2013-07-18T14:12:13` | label dates (probably the LOLA data span, not the simulated span) |
| W3a | same .LBL | `MAP_PROJECTION_TYPE = "POLAR STEREOGRAPHIC"` | projection |
| W3b | same .LBL | `CENTER_LATITUDE = -90 <deg>` / `CENTER_LONGITUDE = 0 <deg>` | projection centre |
| W3c | same .LBL | `A_AXIS_RADIUS = 1737.4 <km>` · `MAP_SCALE = 60 <m/pix>` · `LINES = 5058` · `SAMPLE_BITS = 16` · `SCALING_FACTOR = 0.00004` | raster layout |
| W4 | same .LBL | `The calculations are based on the LOLA elevation map LDEM_85S_060M` | the DEM behind AVGVISIB |
| W5 | https://raw.githubusercontent.com/nyx-space/anise/master/anise/src/astro/aberration.rs | `pub const LT_S: Option<Self> = Some(Self {` | ANISE has light time + stellar aberration (master branch, read Nov 10) |
| W6 | https://lroc.im-ldi.com/images/1408 | `ended up on its side within a 20-meter diameter crater` | IM-2 crater size (from facts-grill-1.md; fetched through a summarising tool, re-read by eye) |
| W7 | https://pgda.gsfc.nasa.gov/products/69 | `averaged over hourly timesteps spanning 18.6 years, for the south polar region.` | AVGVISIB span and step (from facts-grill-1.md) |
| W8 | https://ssd-api.jpl.nasa.gov/doc/horizons.html (header saved in `crates/core/tests/fixtures/horizons/tracer-sun.json`, Nov 13) | `1 au= 149597870.700 km` | AU used to scale solar flux (§1e) |

Negative results (looked for, not found):
- Observer height of AVGVISIB: not in README_ILLUMINATION_2016.TXT, not in the .LBL, not on PGDA product 69.
- The three PA→ME rotation angles: not in any NAIF file in the corpus.
- An IM-1 or IM-2 Sun elevation, battery or array value: not in any corpus file.
- A state-of-charge update equation: not printed in any corpus file.

- A Sun elevation for IM-1 or IM-2 in any mission report: not found.
- `Within SPICE their names are MOON_PA_DExxx…` (a subagent quote): not in the file, dropped.

### Audit and count
- Audit, Nov 10: every E-row quote was checked with `grep -nF` against its corpus file by script. 173 of 173 were found at the listed line. Four subagent quotes failed the first pass; three were re-cut to a single line and one was dropped.
- W-rows are quotes from web documents fetched by subagents or copied from `facts-grill-1.md`. They are not in the corpus and were not re-fetched by this review.
- [D] items are derivations and are not counted.

**Count: 178 claims [V] · 11 claims [W] (web, not grep-checkable) · 20 claims [U].** (E131 re-tagged [V]→[U] by the independent review.)
