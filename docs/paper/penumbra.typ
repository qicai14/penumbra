// D1 project document (docs/SUBMISSION.md). Build: bun run paper -> docs/paper/penumbra.pdf
// Every number below is copied from web/public/scoreboard.json (commits shown in the Results table),
// docs/SCIENCE-SPEC.md, docs/DATA-NOTES.md or docs/IDEA.md. The table in section 3 reads the JSON at build time
// (the build uses --root .); the prose in Results is typed by hand and must be re-checked when the scoreboard changes.

#set document(title: "Penumbra", author: ("Qi Cai", "Charls"))
#set page(paper: "a4", margin: 2cm, numbering: "1")
#set text(size: 10.5pt)
#set par(justify: false, leading: 0.62em)
#set heading(numbering: "1.1")
#set math.equation(numbering: none)
#show table: set text(size: 9pt)
#show figure.caption: set text(size: 9pt)

// A tagged display equation: the tag is the SCIENCE-SPEC section id (1a to 1f).
#let eq(tag, body) = grid(
  columns: (1fr, auto),
  align: (center + horizon, right + horizon),
  body, [(#tag)],
)

// Architecture diagram box.
#let node(title, sub, fill: luma(245)) = rect(
  width: 100%, inset: 7pt, radius: 3pt, fill: fill, stroke: luma(120),
  align(center)[#text(weight: "bold")[#title] \ #text(size: 8.5pt)[#sub]],
)
#let arrow(label) = align(center + horizon, text(size: 8.5pt)[#label])

#align(center)[
  #text(size: 22pt, weight: "bold")[Penumbra] \
  #text(size: 12pt)[Sunlight, Earth contact and the first lethal shadow at lunar south-pole landing sites] \
  #v(4pt)
  Team Fractured Toes: Qi Cai, Charls · NASA Space Apps Challenge 2026, Kuala Lumpur \
  #link("https://penumbra-lunar.pages.dev")
]

#v(6pt)
#figure(
  rect(width: 100%, height: 6.5cm, fill: luma(240), stroke: luma(180))[
    #align(center + horizon, text(fill: luma(100))[
      *Team poster slot (deliverable D2).* \
      No poster image exists in the repository yet. \
      This box holds the poster, with the project name Penumbra and the team name Fractured Toes.
    ])
  ],
  caption: [Project name with team poster (guidebook outline item 1). Placeholder: the poster file is made separately.],
)

= Introduction

Two private landers reached the Moon's south polar region, in February 2024 and March 2025. Both met the same problem: the Sun sits very low there. At the pole itself the Sun's centre stays within 1.54° of the horizon. A 10 m hill casts a shadow hundreds of metres long. Moving a site by 100 m, or a landing date by 2 days, can decide whether a lander lives.

IM-1 Odysseus landed on 22 February 2024 on a 12° slope and ran for about 6 days. IM-2 Athena landed on 6 March 2025 near Mons Mouton. It came to rest on its side in a small crater, where a low Sun angle and the panel orientation stopped charging. It lasted about a day (this duration is unconfirmed).

A landing planner needs one answer for a chosen site and date: *how many hours of sunlight and Earth contact does the lander get, and when does the first shadow arrive that kills it?* Existing tools show one layer each. Penumbra answers the whole question. It is our contribution to the theme "The Next Frontier": the south pole is where the next landings will go, and it is also where landing planning is hardest.

*What it does.* For a south-pole site, a date and a lander preset, Penumbra computes the terrain horizon, the Sun and Earth tracks, the hours of sunlight and Earth contact, the operable window (Sun and Earth together), and the time to the first lethal shadow. The terrain is NASA LOLA topography and the positions come from DE440 ephemerides. We checked it against JPL Horizons and NASA's published LOLA illumination maps. Section 3 lists what passed, what failed and what we did not run.

= Methodology

== Architecture

One Rust model computes everything. The pure-maths crate `crates/core` has no file or network access. It is used twice: natively by `crates/pipeline` (offline, builds the data files and runs the validation) and as WebAssembly through `crates/wasm` in the browser (`web/`). The two cannot drift because they are the same code. There is no server: the app is static and the horizon for a clicked point is computed in the page.

#figure(
  grid(
    columns: (1fr, 1.4cm, 1fr, 1.4cm, 1fr),
    row-gutter: 4pt,
    node("NASA data", "LOLA DEMs, PGDA maps, NAIF kernels, Horizons"), [], [], [], node("web/", "Svelte 5, Three.js, uPlot: draws what the model returns"),
    arrow([↓ read]), [], [], [], arrow([↑ calls]),
    node("crates/pipeline", "native, offline: builds data, runs V1 to V5"), arrow([← uses →]), node("crates/core", "one Rust model, pure maths", fill: rgb("#fde8c8")), arrow([← uses →]), node("crates/wasm", "wasm-bindgen wrapper, no maths"),
    arrow([↓ writes]), [], [], [], arrow([↑ loads]),
    grid.cell(colspan: 5, node("Runtime data files", "int16 terrain tiles (20 m, 80 m, 240 m), hourly Sun and Earth table, region grids, manifest.json, scoreboard.json")),
  ),
  caption: [Architecture. The same `core` runs natively for validation and in the browser as WASM.],
)

== Data

Every dataset is NASA's. @tab-data gives the role and the citation of each. All data is public and needs no login.

#figure(
  table(
    columns: (2.3fr, 3fr, 1.6fr),
    align: left,
    stroke: 0.5pt + luma(150),
    table.header([*Dataset*], [*Role in Penumbra*], [*Citation*]),
    [LOLA south pole DEMs: `LDEM_80S_20MPP_ADJ` (20 m), `LDEM_60S_240MPP_ADJ` (240 m), `LPSR_80S_20MPP_ADJ`], [Terrain input to the horizon and shadow model; the PSR mask is used by red-team case 3 (V5)], [@pgda-lola-spole, @barker2023],
    [PGDA `AVGVISIB_85S_060M_201608` (Sun) and `_EARTH`], [Outside judge for V2: share of time any part of the Sun (or Earth) is visible], [@pgda-illumination, @mazarico2011],
    [NAIF kernels: `de440.bsp`, `moon_pa_de440_200625.bpc`, `moon_de440_250416.tf`, `naif0012.tls`], [Sun and Earth positions and the lunar frame; read with ANISE and turned into the hourly table], [@naif-kernels, @park2021, @anise],
    [JPL Horizons], [Outside judge for V1: Sun and Earth azimuth and elevation], [@horizons],
    [LROC NAC (PGDA map-projected)], [Planned outside judge for V3 (shadow IoU). *Not run:* no NAC image was downloaded], [@pgda-lola-spole],
    [IM-1 and IM-2 mission reports and the LROC image of the IM-2 crater], [Outside judge for V4: landing times, site and outcome], [Sources linked on the About page of the app],
  ),
  caption: [Datasets, their role and their citation. The same table is on the app's About page, which links each original source.],
) <tab-data>

The pipeline cuts the DEMs into int16 tiles with a per-tile height offset. The scale is the finest that holds the tile (for example 0.037 m for the Mons Mouton near tile), because the 0.5 m step of the PDS product would give a 1.4° slope error between neighbouring 20 m pixels. The terrain payload is 33.6 MB for the two regions, and the hourly ephemeris table is 2.1 MB (43 825 rows).

== Ephemeris and frames <sec-eph>

We use DE440 @park2021, not DE441, and the lunar frame `MOON_ME`, in which the LOLA DEMs and the Horizons lunar sites are defined, so no rotation is needed between the terrain and the sky. ANISE @anise reads the NAIF kernels @naif-kernels and gives the _apparent_ Sun and Earth positions (light time plus stellar aberration) as seen from the Moon's centre. UTC is converted to TDB by the library, never by hand. The browser receives an hourly table in UTC and interpolates it linearly. The Sun turns about 0.5° per hour, so the interpolation error is about 0.0001°.

== Equations

The model has six parts. Their equations are SCIENCE-SPEC section 1 (tags 1a to 1f). Angles are in radians inside `core`. Here $R = 1737.4$ km is the DEM reference sphere, $h$ the terrain height, and $H$ the mast or panel height above the ground.

*1a. Horizon with curvature.* A DEM point $(x, y)$ in the south-polar stereographic map becomes a unit vector $hat(u)$ in `MOON_ME`:

#eq("1a", $ t = frac(x^2 + y^2, 4 R^2), quad hat(u)(x, y) = frac((y slash R, thin x slash R, thin -(1 - t)), 1 + t) $)

For each of 720 azimuths (0.5° apart) the model marches outward along the great circle and keeps the largest elevation. For a terrain point at radius $r_t = R + h$ and ground distance $s$, seen from an observer at radius $r_o = R + h_s + H$, with $theta = s slash R$:

#eq("1a", $ tan("el") = frac(r_t cos theta - r_o, r_t sin theta) $)

The march uses three terrain tiers: 20 m out to 7 km, 80 m to 31 km, then 240 m. Each step is one pixel of the tier in use. There is no atmosphere, so there is no refraction.

*1b. Fraction of the Sun's disc.* The Sun is a disc of angular radius $rho = arcsin(R_("sun") slash d_("sun"))$, about 0.26°. In the tangent plane at the Sun, with $xi$ across the disc and $w = sqrt(rho^2 - xi^2)$, the horizon is $g(xi)$ relative to the Sun's centre. The lit fraction is

#eq("1b", $ f = frac(1, pi rho^2) integral_(-rho)^(rho) "clamp"(w - g(xi), 0, 2 w) thin d xi $)

evaluated with 32 midpoint strips. Two outputs are kept apart: _illumination_ is $f$ (used for power), and _visibility_ is 1 when $f > 0$ (this is what the PGDA maps average).

*1c. Sun and Earth direction from the site.* With the site vector $bold(s) = (R + h + H) hat(u)_s$ and the apparent body position $bold(r)$, the direction is $bold(d) = bold(r) - bold(s)$. In the local frame (up, east, north):

#eq("1c", $ "el" = arcsin frac(bold(d) dot bold("up"), |bold(d)|), quad "az" = op("atan2")(bold(d) dot bold("east"), bold(d) dot bold("north")) $)

This is the topocentric shift. It moves the Earth by about 0.26° and the Sun by 0.0007°.

*1d. Earth contact.* Earth must clear the terrain by a margin. The default is 2°, taken from De Rosa @derosa2012 and shown as "assumed" in the app:

#eq("1d", $ "margin"(t) = "el"_("earth")(t) - "el"_("hor")("az"_("earth")(t)), quad "contact" = ("margin" >= m_("min")) $)

*1e. Battery state of charge.* A plain energy balance with time step $Delta t$ = 10 min:

#eq("1e", $ P_("gen") = S_0 (1 "AU" slash d_("sun"))^2 thin A thin eta thin max(0, hat(n) dot hat(s)) thin f, quad E_(k+1) = "clamp"(E_k + (P_("gen") - P_("load")) Delta t, 0, E_("max")) $)

The first lethal event is the first step with $E <= E_("min")$. The time to first lethal shadow is that time minus the landing time. The operable window is the set of steps with $f > 0$ and Earth contact.

*1f. Uncertainty from DEM error.* For azimuth bin $i$ with the obstacle at distance $D_i$:

#eq("1f", $ sigma_("el", i) = frac(sqrt(sigma_(z, "obstacle")^2 + sigma_(z, "observer")^2), D_i), quad sigma_t = frac(sigma_("el"), |d("el"_("sun") - "el"_("hor")) slash d t|) $)

This gives a plus-or-minus band on each sunrise and sunset. We use $sigma_z = 0.5$ m, which is assumed. Where the Sun skims a flat horizon the band is hours long, and the app writes "ill-conditioned" instead of a time.

*Local pit.* IM-2 stopped in a crater about one DEM pixel wide. A crater of diameter $D_c$ and depth $d_c$ raises the horizon floor to $"el"_("pit") = arctan((d_c - H) slash (D_c slash 2))$. The default depth follows ESA's degraded-crater model (an 11° slope). The depth is assumed, not measured.

== The application

The app is one page (`https://penumbra-lunar.pages.dev`) with four views and a link to the GitHub repository.

- *Site.* A verdict strip at the top reads the scoreboard and says, for example, which checks pass, which fail and which did not run, with a button to the validation details. Pick a region, a point on the terrain map, a landing date, a span and a mast height. The page shows the survival headline (operable days and time to first lethal shadow) for a lander preset, a countdown to sunset and Earthset, the horizon panorama with the Sun and Earth tracks, a 3D terrain view shaded by our model, and timelines of sunlight, Earth contact and battery charge with the plus-or-minus bands.
- *Compare A vs B.* Two site-and-date pairs with one lander preset, side by side, with the difference. The pairs live in the page URL.
- *IM-2 replay.* The IM-2 site and date, answered twice: with regional terrain only, and with a pit of a chosen size.
- *About.* The scoreboard, the IM-2 tracer check, the limitations and the dataset table with their sources.

For both built regions (Mons Mouton and the Shackleton ridge) the Site view also has a *region heatmap* and a *nearest-better-site readout*, and the site picker shows the best cell of each region side by side, with a button that moves the site there. The heatmap colours each 100 m cell of the region by operable days, for three landing dates, with a contour at 5 operable days. The readout takes the pinned site and names the nearest grid point within 1 km that gains at least a day of operable time, with the gain, the distance and the direction on the map (or says that none does). The grid was computed natively with the same `core` functions the browser runs. It is built for the default preset, a 2 m mast and a 30 day span: if the planner changes any of these, the readout asks for a reset instead of showing a number that no longer matches the headline.

= Results

All results are copied from the validation scoreboard (`web/public/scoreboard.json`, run on 14 and 15 November 2026). We report the misses with the passes. Only checks that ran are claimed.

#let sb = json("/web/public/scoreboard.json")
#let row(id) = sb.find(r => r.id == id)
// Number with a unit, taken from the scoreboard (an empty value prints "none").
#let val(id, key, unit: "") = {
  let v = row(id).at(key)
  if v == none { "none" } else { str(v) + unit }
}
#let verdict(id, ok, bad, none-text) = {
  let p = row(id).pass
  if p == none { none-text } else if p { ok } else { bad }
}
#let commits = sb.filter(r => r.commit != none).map(r => r.commit).dedup()

#figure(
  table(
    columns: (0.9cm, 3.3cm, 3.0cm, 1.8cm, 2.9cm, 3.0cm),
    align: (left, left, left, left, left, left),
    stroke: 0.5pt + luma(150),
    table.header([*ID*], [*Outside judge*], [*Metric*], [*Threshold*], [*Result*], [*Status*]),
    [V1], [JPL Horizons], [worst az/el difference, #val("V1", "sample_size") site/time pairs], [≤ #val("V1", "threshold", unit: "°")], [#val("V1", "value", unit: "°")], [#verdict("V1", [Pass], [*Fail*], [*Not run*])],
    [V2], [PGDA #raw("AVGVISIB_85S_ 060M_201608")], [pooled median visibility error, #{let s = str(row("V2").sample_size); s.slice(0, -3) + " " + s.slice(-3)} pixels], [≤ #val("V2", "threshold") points], [#val("V2", "value") points], [#verdict("V2", [Pass; rests on the 0 to 60 % bins], [*Fail*], [*Not run*])],
    [V2b], [Barker 2021 Site 1; Mazarico 2011 SP-01], [longest-night ranges, #val("V2b", "sample_size") RoIs plus SP-01], [all inside], [Barker #val("V2b", "value") of #val("V2b", "sample_size"); SP-01 outside by 0.21 d], [#verdict("V2b", [Pass], [*Fail*, stated], [*Not run*])],
    [V3], [LROC NAC (shadow IoU)], [IoU gap to NASA model], [≤ #val("V3", "threshold")], [#val("V3", "value")], [#verdict("V3", [Pass], [*Fail*], [*Not run*])],
    [V4], [IM-2 reports], [consistent with reports], [none set], [one statement, IM-2], [#verdict("V4", [Reconstruction, not validation], [*Fail*], [*Not run*])],
    [V5], [Red-team cases], [cases passing], [#val("V5", "threshold") of #val("V5", "sample_size")], [#val("V5", "value") of #val("V5", "sample_size")], [#verdict("V5", [Pass], [*Fail*], [*Not run*])],
  ),
  caption: [The validation scoreboard, read from `scoreboard.json` at build time (commits #commits.map(c => raw(c)).join(", ")). V1 ran on 14 November; V2, V2b, V4 and V5 on 15 November.],
) <tab-score>

== V1: ephemeris against JPL Horizons

We compared our Sun and Earth azimuth and elevation with Horizons at 20 site and time pairs. They include the pole, a point 0.1° from the pole, the SP-01 site, the IM-1 and IM-2 sites, two far-side points, five De Rosa region centres, the four libration extremes of the table, and both sides of 1 January 2027, with 5 km and -3 km height offsets. The test goes through the shipped hourly table and linear interpolation. The worst difference is *0.00011°* (Earth), against a threshold of 0.002°. The Sun's worst difference is 0.000013°. Turning stellar aberration off would move the Sun by 0.0055 to 0.0060°, so the check is sensitive to the corrections in @sec-eph. Horizons ran with DE441 and we use DE440, so the two use different ephemerides.

== V2: visibility against the NASA PGDA illumination map

The judge is the PGDA `AVGVISIB_85S_060M_201608` map. We simulated hourly from 1 January 2010 for 163 048 steps (18.6 years) at a 0 m observer height, and compared with the map at 6 123 pixels drawn from the Shackleton ridge and Mons Mouton tiles in five brightness bins. The pooled median error is *0.48 points* (threshold 5), the mean signed error is +0.72 points (limit 2), and Pearson's $r$ is 0.955 (limit 0.9). The stretch targets (median 2, 90th percentile 8) were also met: the 90th percentile is 3.98.

#figure(
  {
    let rows = (
      ("0 to 20 %", "2 000", 0.43, "6.76", "+1.95", "0.62"),
      ("20 to 40 %", "2 000", 0.47, "3.39", "+0.28", "0.89"),
      ("40 to 60 %", "2 000", 0.48, "2.83", "+0.15", "0.83"),
      ("60 to 80 %", "118", 3.12, "9.46", "-2.74", "0.70"),
      ("80 to 100 %", "5", 1.86, "9.17", "-1.95", "0.63"),
    )
    table(
      columns: (2.2cm, 1.4cm, 1.7cm, 1.5cm, 1.5cm, 1.2cm, 5.2cm),
      align: (left, right, right, right, right, right, left),
      stroke: 0.5pt + luma(150),
      table.header([*Reference bin*], [*Pixels*], [*Median*], [*p90*], [*Mean*], [*r*], [*Median against the limit of 5*]),
      ..rows.map(r => (
        r.at(0), r.at(1), str(r.at(2)), r.at(3), r.at(4), r.at(5),
        box(width: 100%, height: 0.35cm, stroke: 0.4pt + luma(170),
          align(left, rect(width: r.at(2) / 5 * 100%, height: 100%, fill: rgb("#d97706"), stroke: none))),
      )).flatten(),
    )
  },
  caption: [V2 per brightness bin of the PGDA map (errors in points, ours minus reference). The bar is the median as a share of the limit.],
) <tab-v2>

*Read this pass with care.* The per-bin $r$ is below 0.9 in every bin. Our tiles hold only 118 pixels in the 60 to 80 % bin and 5 in the 80 to 100 % bin, so the brightest rims are under-sampled. The pass rests on the 0 to 60 % bins. We accept this as a stated limit. The Earth twin of the same test (any part of Earth's disc visible, report only) gave a median error of 0.18 points and $r$ = 0.990. AVGVISIB is not the Mazarico 2011 run, and beyond 7 km its far field is 60 m against our 80 m to 31 km and 240 m beyond. A cross-check of the Mons Mouton grid (1 500 cell centres, 2 m mast, report only) gave a median error of 0.73 points and $r$ = 0.973.

== V2b: longest night against Barker 2021 and Mazarico 2011 (a stated fail)

This check asks whether our longest continuous night and longest continuous day fall near published values. It is one-sided: it catches a model that is too dark, not one that is too bright. Two references were used.

- *Barker 2021 Site 1 @barker2021:* passes *7 of 7* regions of interest.
- *Mazarico 2011 SP-01 @mazarico2011:* *fails.* Their longest total night is 2.75 d. Our shortest pixel among the 180 pixels within 150 m of SP-01 is 2.96 d, which is 0.21 d too long.

The 0.21 d gap is inside the reference's own 6 hour time step (about 0.25 d) and in the direction expected from our finer 20 m near field (their DEM is 240 m). Nothing was tuned. We still report it as a fail, because the agreed rule is the strict range. The scoreboard marks V2b as failed.

#figure(
  table(
    columns: (1.1cm, 1.4cm, 2.1cm, 2.1cm, 2.1cm, 2.1cm, 2.6cm),
    align: (left, right, right, right, right, right, right),
    stroke: 0.5pt + luma(150),
    table.header([*RoI*], [*Pixels*], [*Longest day, ours (d)*], [*Longest day, Barker (d)*], [*Longest night, ours (d)*], [*Longest night, Barker (d)*], [*Our night, min to max (d)*]),
    [1], [20], [225.80], [215.79], [4.43], [5.29], [3.71 to 8.63],
    [2], [18], [222.20], [216.28], [6.63], [7.31], [5.33 to 10.79],
    [3], [9], [229.75], [216.89], [7.07], [8.41], [5.63 to 13.88],
    [4], [35], [233.07], [231.20], [4.11], [4.00], [3.46 to 6.29],
    [5], [26], [217.63], [218.98], [4.46], [4.52], [3.92 to 5.83],
    [6], [26], [220.72], [220.98], [3.95], [4.16], [3.54 to 4.71],
    [7], [19], [138.47], [119.07], [5.26], [5.42], [5.08 to 5.54],
  ),
  caption: [V2b, Barker 2021 Site 1: mean over the pixels near each RoI centroid, observer 5 m above ground, hourly 2024 and 2025. Our 20 m terrain is coarser than Barker's 5 m and biases bright.],
) <tab-v2b>

== V3: shadows against LROC NAC images (not run)

V3 was meant to compare our shadow shapes with NASA's map-projected NAC images and its own shadow model, as an intersection over union (IoU) gap of at most 0.05 on three images. *We did not run it.* No NAC image was downloaded, so the check has no input. No V3 result is claimed anywhere in this document. It is future work.

== V4: IM-2 reconstruction (consistent with reports, not validated)

We reconstructed one statement, about IM-2. At an assumed touchdown of 2025-03-06 17:28 UTC (the touchdown time is not in a primary source), the model puts the Sun at 2.46° elevation, 1.63° above the regional horizon. A 20 m crater with the default depth (1.94 m) hides the whole Sun disc for panel heights of 0, 1 and 1.44 m, and not for 2 m. At 1.44 m the margin is only 2.7 cm, and the Sun moves about 0.5° per hour, so a few minutes of error in the assumed time can flip this row. The crater is below the DEM's resolution, so it is a stated model, not terrain. The IM-1 statements were dropped. The scoreboard says "consistent with reports; not validated", and so do we.

== V5: red-team cases

All 7 edge cases pass as automated tests. Examples: at the pole nothing is non-finite and the flat-sphere horizon at 2 m is -0.0869° at all 720 azimuths; over 2026 to 2030 the Sun at the pole never rises above 1.5874° (bound 1.6°); in 20 permanently shadowed crater pixels the Sun is never above the horizon over 18.6 years (closest approach 0.378° below it); and positions are continuous across 1 January 2027. One case needs the 148 MB shadow mask and is not run in CI. It was re-run locally.

== Speed

The horizon is the costly step. Medians are 13 to 34 ms across native, WASM and browser at the IM-2 and SP-01 sites (docs/DATA-NOTES.md, U18), under our 50 ms target (one WASM call at SP-01 took 62 ms). The V2 run (14 950 horizon-and-timeline evaluations) took 524 s on 8 threads.

= Discussion

== Limitations

These are the limitations stated on the app's About page.

- No limb darkening.
- Terrain smaller than a 20 m pixel is not in the DEM.
- The Earth margin and $sigma_z$ are assumed.
- No charge or discharge efficiency.
- Lander values are assumed unless marked cited.
- The IM-2 pit depth is assumed (U9): the ESA degraded-crater model, not a measured depth.
- The IM-2 pit view keeps the regional far horizon: inside a pit the panel sits $H$ below regional ground, under 2 m of observer height at kilometres of distance.

The scoreboard adds checks that did not pass cleanly, listed in section 3: V2b failed on SP-01, V3 was not run, V2 rests on the 0 to 60 % bins, and V4 is a reconstruction. Only two regions (Mons Mouton and the Shackleton ridge) are built, and both have the region heatmap. The other seven Artemis III regions are not in the app: we have no cited centre for them.

Mast height matters. Gläser et al. @glaser2018 report average illumination of 69.5 to 82.9 % at the surface and 77.1 to 88.0 % at 2 m, so the app shows the observer height and does not hide it.

== Comparison with existing tools

@tab-compare compares Penumbra with the three tools the challenge page lists. It is our reading of their published descriptions, not a test of the live tools, and it shows only what we can support.

#figure(
  table(
    columns: (3.1cm, 3.6cm, 3.4cm, 3.1cm, 3.1cm),
    align: left,
    stroke: 0.5pt + luma(150),
    table.header([], [*NASA Moon Trek*], [*LROC QuickMap*], [*JPL Horizons*], [*Penumbra*]),
    [Main form], [Web portal; lighting and line-of-sight tools need an account], [Layers and maps; synthetic terrain-shadow images], [Ephemeris service and API], [Static web app, no login],
    [Horizon profile and Sun and Earth tracks for a site], [Map and raster centred], [Map centred], [Directions only, no terrain], [Yes, with mast height],
    [Side-by-side site and date comparison], [No multi-site date comparison found], [Not a horizon-profile comparison], [No], [Yes (Compare A vs B)],
    [Joint Sun and Earth operable window and time to first lethal shadow], [Not found], [Not found], [No], [Yes, for a lander preset],
    [Checked against outside judges], [Not applicable], [Not applicable], [It is our judge for V1], [V1, V2, V2b, V4, V5 (V3 not run)],
  ),
  caption: [Comparison. Sources: the tool descriptions in `docs/research/lunar-landing/sources.md` and `docs/IDEA.md`. "Not found" means we did not find it in those descriptions.],
) <tab-compare>

Penumbra does not replace these tools. It uses Horizons as its judge.

== Commercialization

*Who pays.* The users are the teams that choose where and when to land: CLPS vendors, Artemis operations planners and rover teams. A schedule slip forces them to re-plan a site and date, which is the case the Compare view serves.

*Model.* The model and the app are open (Apache-2.0 code, public NASA data), so any team can check a site before it flies. We propose to add an open programmatic API, and a paid mission-planning service on top of it: custom lander presets with cited values, more regions and dates, validation reports for a given mission, and support for the planning team. This is a proposal. The app today is a free static page, and no paid service or hosted API exists yet.

*Value.* A lander that loses power early loses the mission. IM-1 operated for 144 hours (six days) after it tipped over, and IM-2 for about a day. We have no cited price for a lost lander or for a planning tool, so we give no dollar figure and make no cost comparison. The value we can state is that the check takes minutes in a browser, at a site and date, before the lander is built or launched.

== Future work

Run V3 with LROC NAC images. Add the other seven Artemis III regions once each has a cited centre. Add the Monte Carlo version of the uncertainty bands. Replace the assumed lander values with cited ones.

= Conclusion

Penumbra shows landing planners the hours of sunlight and Earth contact at a south-pole site and date, and the time to the first lethal shadow. One Rust model runs natively for validation and in the browser as WebAssembly. It agrees with JPL Horizons to 0.00011° (V1) and with the NASA PGDA illumination map to a median of 0.48 points (V2, on the 0 to 60 % bins). It passes all 7 red-team cases (V5). It misses the SP-01 night length by 0.21 d (V2b), the IM-2 result is a reconstruction (V4), and V3 was not run. We state these as they are. The next frontier needs landers that survive low Sun and long shadows, and a planner can now ask this question before the landing.

= Use of AI

We used an AI coding assistant (Anthropic's Claude, in Claude Code) to help write Rust and web code, tests and documents. The team set the scope and chose the checks. Every number in this document comes from the test outputs and the scoreboard, and the equations come from the cited papers and our own derivations. The team reviewed and ran all code. The app has no AI model at runtime.

#bibliography("refs.yml", title: "References", style: "ieee")
