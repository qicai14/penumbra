// D1 project document (docs/SUBMISSION.md). Build: bun run paper -> docs/paper/penumbra.pdf
// Every number has a source in docs/submission/facts.md (copied word-identical), web/public/scoreboard.json,
// docs/DATA-NOTES.md, docs/EVIDENCE.md or refs.yml. The equations are docs/SCIENCE-SPEC.md section 1.

#set document(title: "Penumbra", author: ("Charls Pua Vui Hau", "Lim Qi Cai"))
#set page(paper: "a4", margin: 2cm, numbering: "1")
#set text(size: 10.5pt)
#set par(justify: false, leading: 0.62em)
#set heading(numbering: "1.1")
#set math.equation(numbering: none)
#show table: set text(size: 8.5pt)
#show figure.caption: set text(size: 9pt)
#show figure.where(kind: table): set figure.caption(position: top)
#set table.cell(breakable: false)
#show link: set text(fill: rgb("#9a4a00"))

// A tagged display equation: the tag is the SCIENCE-SPEC section id (1a to 1e).
#let eq(tag, body) = grid(
  columns: (1fr, auto),
  align: (center + horizon, right + horizon),
  body, [(#tag)],
)

// Architecture diagram box and arrow.
#let node(title, sub, fill: luma(245)) = rect(
  width: 100%, height: 2.6cm, inset: 6pt, radius: 3pt, fill: fill, stroke: luma(110),
  align(center + horizon)[#text(size: 9pt, weight: "bold")[#title] \ #v(2pt) #text(size: 7.5pt)[#sub]],
)
#let arr = align(center + horizon, text(size: 16pt, fill: luma(90))[→])

#align(center)[
  #text(size: 22pt, weight: "bold")[Penumbra] \
  #text(size: 12pt)[Sunlight, Earth contact and the first lethal shadow at lunar south-pole landing sites] \
  #v(4pt)
  Team Fractured Toes: Charls Pua Vui Hau, Lim Qi Cai · NASA Space Apps Challenge 2026, Kuala Lumpur \
  Live app: #link("https://penumbra-lunar.pages.dev") · Code: #link("https://github.com/qicai14/penumbra")
]

#v(4pt)
// Team poster (guidebook outline item 1).
#figure(
  image("poster.png", height: 19.5cm),
  caption: [Team poster: Penumbra by Team Fractured Toes (Charls Pua Vui Hau, Lim Qi Cai). Challenge: Lunar Landing Site Navigator.],
)
#pagebreak()

= Introduction

The theme of this challenge is "The Next Frontier", and for landers the next frontier is the lunar south pole. The Sun sits low there, so a small hill can shade a site for days, and the date matters as much as the place. Two private landers have already met this problem.

Intuitive Machines' Odysseus (IM-1) landed on 2024-02-22 at 80.1276° S, 1.4367° E @nasa-lro-im1 on a 12° slope and came to rest about 30° off the surface. It ran 144 hours and ended "as the sun moved out of view of the one illuminated solar panel" @spacenews-im1. Athena (IM-2) reached Mons Mouton on 2025-03-06 at 84.7906° S, 29.1957° E. It "ended up on its side within a 20-meter diameter crater" @lroc-im2, and NASA got "some data" before its batteries were depleted @nasa-im2-end. A low Sun angle and the panels' orientation stopped charging.

Both landers tipped, and no planning tool can fix a lander lying on its side. Penumbra answers a narrower question: what the site and the date gave an upright lander. That part a planner controls before launch. The demand is growing. NASA cut its Artemis III candidate list from thirteen to nine south-pole regions @artemis-nine, and the first crewed south-pole landing is now Artemis IV in 2028 @nasa-artemis-iv. Blue Moon MK1 lands on the Shackleton connecting ridge in 2027 @nasa-blue-moon, and NASA lists south-pole deliveries for Intuitive Machines, Firefly and Astrobotic @nasa-clps-deliveries.

For a site, a landing date and a mast height, Penumbra answers three questions (@tab-questions). It ships all nine Artemis III regions plus the Shackleton ridge.

#figure(
  table(
    columns: (3.4cm, 1fr),
    align: left,
    stroke: 0.5pt + luma(150),
    table.header([*Question*], [*What the app returns*]),
    [How many hours of sunlight?], [Hours with any part of the Sun's disc above the real terrain horizon, from the 20 m LOLA DEM out to 7 km, 80 m to 31 km, 240 m beyond],
    [How many hours of Earth contact?], [Hours with Earth above the horizon by a 2° margin (assumed: a preset value, taken from one paper's comms margin @derosa2012, not a mission requirement)],
    [When does the first lethal shadow arrive?], [The first moment stored battery energy falls to the preset's minimum (survival model)],
  ),
  caption: [The three questions Penumbra answers.],
) <tab-questions>

= Methodology

== Architecture

One Rust model computes everything. The pure-maths crate `crates/core` has no file or network access. The native pipeline (`crates/pipeline`) runs it offline to build the data files and the validation scoreboard, and `crates/wasm` compiles the same code to WebAssembly for the browser. The scoreboard numbers and the planner's numbers come from the same code, so they cannot drift. There is no backend: the app is static files on Cloudflare Pages, and the horizon for a clicked point is computed in the page (@fig-arch).

#figure(
  grid(
    columns: (1fr, 0.55cm, 1fr, 0.55cm, 1fr, 0.55cm, 1fr, 0.55cm, 1fr),
    node("NASA data", "LOLA DEMs, PGDA maps, NAIF kernels, Horizons"),
    arr,
    node("Native Rust pipeline", "crates/pipeline + crates/core, offline; runs V1 to V5"),
    arr,
    node("Static files on Cloudflare Pages", "int16 terrain tiles, hourly Sun and Earth table, scoreboard", fill: rgb("#eef3fb")),
    arr,
    node("Same Rust model as WASM", "crates/core via crates/wasm, in the browser", fill: rgb("#fde8c8")),
    arr,
    node("Svelte UI", "Three.js terrain, uPlot timelines, horizon panorama"),
  ),
  caption: [Architecture. No backend: 0 servers and 0 runtime API calls. The same `crates/core` runs natively for validation and in the browser as WASM.],
) <fig-arch>

== Data

All terrain, illumination and ephemeris data is NASA's (@tab-data). The pipeline cuts the DEMs into int16 tiles. The NAIF kernels are 133 MB, so the Sun and Earth vectors ship as an hourly table of about 2.1 MB.

#figure(
  table(
    columns: (4.6cm, 3.6cm, 1fr),
    align: left,
    stroke: 0.5pt + luma(150),
    table.header([*Data*], [*Used for*], [*Citation*]),
    [LOLA south-pole DEMs `LDEM_80S_20MPP_ADJ` (20 m), `LDEM_60S_240MPP_ADJ` (240 m), PSR mask `LPSR_80S_20MPP_ADJ`], [Terrain horizon, slope, V5 shadowed sites], [NASA GSFC PGDA, doi 10.60903/gsfcpgda-lola-spole @pgda-lola-spole; Barker et al. 2023 @barker2023],
    [PGDA illumination maps `AVGVISIB_85S_060M_201608` and `_EARTH`], [V2 judge], [@pgda-illumination; 18.6 years hourly],
    [NAIF kernels `de440.bsp`, `moon_pa_de440_200625.bpc`, `moon_fk_de440.epa`, `moon_de440_250416.tf`], [Sun and Earth vectors in the Moon frame, via ANISE], [@naif-kernels; DE440, Park et al. 2021 @park2021],
    [JPL Horizons API], [V1 judge], [@horizons],
    [LROC], [IM-2 landing site and crater; Artemis III region outlines via the QuickMap layer], [@lroc-im2 @quickmap],
    [Published PGDA-based studies], [V2b reference values], [Barker et al. 2021 @barker2021, Mazarico et al. 2011 @mazarico2011],
  ),
  caption: [NASA data used, its role and its citation.],
) <tab-data>

== Frames and the hourly ephemeris table <sec-eph>

The LOLA DEMs and the Horizons lunar sites are both defined in the Moon's mean-Earth frame, `MOON_ME`, so the model works in that frame and needs no rotation between terrain and sky. ANISE @anise, a pure-Rust SPICE reader, loads the DE440 kernels @park2021 @naif-kernels with the `MOON_PA` orientation and the `MOON_PA` to `MOON_ME` frame definition. It returns the apparent Sun and Earth positions (light time plus stellar aberration) as seen from the Moon's centre. The library converts UTC to TDB; we never do it by hand. The pipeline stores these vectors on a fixed grid of whole UTC hours, and the browser interpolates the table linearly, so it never handles time scales or kernels.

== Equations

The model has six parts and a local-pit option, each taken from `docs/SCIENCE-SPEC.md` section 1 (tags 1a to 1f). Angles are in radians inside `core`. $R = 1737.4$ km is the DEM reference sphere, $h$ the terrain height and $H$ the mast or panel height above the ground.

*1a. Horizon with curvature.* A DEM point $(x, y)$ in the south-polar stereographic map becomes a unit vector $hat(u)$ in `MOON_ME`:

#eq("1a", $ t = frac(x^2 + y^2, 4 R^2), quad hat(u)(x, y) = frac((y slash R, thin x slash R, thin -(1 - t)), 1 + t) $)

For each of 720 azimuths the model marches outward along the great circle and keeps the largest elevation. For a terrain point at radius $r_t = R + h$ and ground distance $s$, seen from an observer at radius $r_o = R + h_s + H$, with $theta = s slash R$:

#eq("1a", $ tan("el") = frac(r_t cos theta - r_o, r_t sin theta) $)

The march uses three terrain tiers: the 20 m LOLA DEM out to 7 km, 80 m to 31 km, and 240 m beyond. Each step is one pixel of the tier in use. The Moon has no atmosphere, so there is no refraction.

*1b. Visible fraction of the Sun's disc.* The Sun is a disc of angular radius $rho = arcsin(R_("sun") slash d_("sun"))$. In the tangent plane at the Sun, with $xi$ across the disc and $w = sqrt(rho^2 - xi^2)$, the horizon is $g(xi)$ relative to the Sun's centre. The lit fraction is

#eq("1b", $ f = frac(1, pi rho^2) integral_(-rho)^(rho) "clamp"(w - g(xi), 0, 2 w) thin d xi $)

The model keeps two outputs apart. _Illumination_ is $f$ and drives power. _Visibility_ is 1 when $f > 0$, which is what the PGDA maps average and what V2 compares.

*1c. Sun and Earth direction from the site.* With the site vector $bold(s) = (R + h + H) hat(u)_s$ and the apparent body position $bold(r)$ from the table, the direction is $bold(d) = bold(r) - bold(s)$. In the local up, east and north frame:

#eq("1c", $ "el" = arcsin frac(bold(d) dot bold("up"), |bold(d)|), quad "az" = op("atan2")(bold(d) dot bold("east"), bold(d) dot bold("north")) $)

*1d. Earth visibility margin.* Earth must clear the terrain horizon by a margin $m_("min")$, 2° by default (assumed, following De Rosa @derosa2012):

#eq("1d", $ "margin"(t) = "el"_("earth")(t) - "el"_("hor")("az"_("earth")(t)), quad "contact" = ("margin" >= m_("min")) $)

*1e. Battery state of charge.* A plain energy balance with a fixed time step $Delta t$:

#eq("1e", $ P_("gen") = S_0 (1 "AU" slash d_("sun"))^2 thin A thin eta thin max(0, hat(n) dot hat(s)) thin f, quad E_(k+1) = "clamp"(E_k + (P_("gen") - P_("load")) Delta t, 0, E_("max")) $)

Here $A$ is the panel area, $eta$ the cell efficiency, $hat(n)$ the panel normal and $hat(s)$ the unit vector to the Sun. *Lethal shadow* is the first moment in a scenario when stored energy $E$ falls to the preset's minimum $E_("min")$; the survival run stops there. Time to first lethal shadow is that moment minus the landing time. The operable window is the set of steps with $f > 0$ and Earth contact, counted up to the lethal time.

*1f. Uncertainty from DEM error.* LOLA heights carry an error $sigma_z$, and an error on a near obstacle moves the horizon more than one far away. For azimuth bin $i$ with the obstacle at distance $D_i$, the first-order propagation (SCIENCE-SPEC 1f, step 1) is

#eq("1f", $ sigma_("el", i) = frac(sqrt(sigma_(z, "obstacle")^2 + sigma_(z, "observer")^2), D_i), quad sigma_t = frac(sigma_("el"), |d("el"_("sun") - "el"_("hor")) slash d t|) $)

evaluated at each sunrise and sunset. The timelines draw this as a ± band. Where the Sun skims a flat horizon the rate in the denominator is near zero and the band runs to hours, so the app writes "ill-conditioned" instead of a time (`ILL_CONDITIONED_BAND_H` in `web/src/lib/timelines.ts`).

*Local pit.* IM-2 ended in a crater about one DEM pixel wide @lroc-im2, and terrain that small is not in the DEM. For a lander at the centre of a crater of diameter $D_c$ and depth $d_c$, with the panel at height $H$ above the floor, the pit raises the horizon floor at every azimuth (SCIENCE-SPEC 1f, "Local pit"):

#eq("pit", $ "el"_("pit") = arctan frac(d_c - H, D_c slash 2), quad "el"'_("hor", i) = max("el"_("hor", i), "el"_("pit")), quad d_c^* = H + frac(D_c, 2) tan("el"_("sun") + rho) $)

The default depth follows ESA's degraded-crater model, an 11° slope from floor to rim (SCIENCE-SPEC E112), so $d_c = (D_c slash 2) tan 11 degree$. This depth is assumed, not measured. The model reports $d_c^*$, the depth that would hide the whole Sun disc, and leaves the reader to compare it with the assumed depth.

== The application

The app is one page at #link("https://penumbra-lunar.pages.dev"). A planner picks a region, a point, a landing date, a span and a mast height. The Site view then shows where the Sun and Earth are at the chosen moment, the horizon panorama (the skyline the lander sees all the way round, with the Sun and Earth tracks across it), and timelines of illumination, Earth margin, Earth contact and battery charge, with the first lethal shadow marked where the battery reaches its minimum (@fig-site). The page also gives operable days for the next 30 landing dates (\#108), the slope at the pin from the 20 m DEM (\#112), and a 3D terrain view coloured by the model. Other views compare two sites or dates side by side, replay IM-2, and show the validation scoreboard with its misses. Every number on the page comes from the WASM build of `crates/core`, the same code that produced the scoreboard in @tab-score.

// Two crops of docs/screenshots/critique-after/site-1440.png, made with
// ffmpeg -vf crop=1440:621:0:1123 (horizon) and crop=1440:777:0:1771 (timelines).
#figure(
  stack(
    spacing: 4pt,
    rect(inset: 0pt, stroke: 0.5pt + luma(150), image("/docs/screenshots/paper/site-horizon.png", width: 16cm)),
    rect(inset: 0pt, stroke: 0.5pt + luma(150), image("/docs/screenshots/paper/site-timelines.png", width: 16cm)),
  ),
  caption: [The Site view at the IM-2 site (84.7906° S, 29.1957° E), mast 2 m, 30-day span from 2026-11-14. Top: the horizon panorama with the Sun (orange) and Earth (blue) tracks. Bottom: the timelines of illumination, Earth margin, Earth contact and battery charge, with the first lethal shadow marked.],
) <fig-site>

= Results

All validation values come from `web/public/scoreboard.json`, read on 2026-11-15. We report the miss and the check we did not run beside the passes.

#figure(
  table(
    columns: (1.6cm, 3.3cm, 1fr),
    align: left,
    stroke: 0.5pt + luma(150),
    table.header([*Test*], [*Field values*], [*Plain statement*]),
    [V1, JPL Horizons], [`value: 0.00011`, `threshold: 0.002`, `pass: true`, `sample_size: 20`], [Worst Sun or Earth azimuth/elevation difference over 20 site-time pairs, 2026–2030: 0.00011° (Earth); the Sun is 0.000013°. That is 18 times inside the 0.002° threshold.],
    [V2, LOLA AVGVISIB], [`value: 0.48`, `threshold: 5`, `pass: true`, `sample_size: 6123`], [Pooled median visibility error 0.48 points against NASA's 18.6-year average-visibility map @pgda-illumination, over 6 123 pixels; mean signed error +0.72, Pearson r 0.955. Earth twin: median 0.18, r 0.990.],
    [V2b, published sites], [`value: 7`, `threshold: 6`, `pass: false`, `sample_size: 7`], [*Reported as a fail.* Barker 2021 Site 1 @barker2021: 7 of 7 regions pass, above the threshold of 6. The fail comes from the second check, the SP-01 range check: Mazarico 2011's @mazarico2011 2.75-day longest night is 0.21 days below our shortest pixel (2.96 days). That gap is inside Mazarico's 6-hour step (about ±0.25 days), but the agreed rule is the strict range, so `pass` is false.],
    [V3, LROC NAC images], [`value: null`, `pass: null`, `notes: "not run"`], [*Not run.* See Future work.],
    [V4, IM-2 replay], [`pass: true`], [*A reconstruction, not a validation:* at the assumed touchdown the Sun stands 2.46° up, and a 1.94 m-deep pit hides it from panels at 0–1.44 m. Fragile: at H = 1.44 m the margin is only 2.7 cm, a panel at H = 2 m is not hidden ($d_c^*$ = 2.48 m), and a few minutes of error in the assumed touchdown time can flip the result.],
    [V5, red team], [`value: 7`, `threshold: 7`, `pass: true`, `sample_size: 7`], [All 7 edge cases pass (pole singularity, flat-pole bounds, 20 permanently shadowed sites that stay dark for 18.6 years).],
  ),
  caption: [The validation scoreboard. Each test has an outside judge; V3 has no result and claims none.],
) <tab-score>

*V1* runs through the shipped hourly table and its linear interpolation, so it tests what the browser uses. *V2* passes on the pooled data, and the pooled figure needs care: per-bin correlation is below 0.9 in every fifth (0.62 to 0.89), and the pooled pass rests on the 0–60 % bins, because the brightest rims are under-sampled. *V2b* is one-sided. It catches a model that is too dark and misses one that is too bright. It has two parts. The Barker part uses the threshold rule: 7 of 7 regions pass against a threshold of 6. The SP-01 part uses the strict range rule, and there our longest night is too long, which is the too-dark direction. Two caveats from the V2b `notes` apply: the equal-area disc around each centre can take in darker flank pixels, which risks a false miss, and Mazarico's 6-hour step gives about ±0.25 d, so the 0.21 d gap is inside his step resolution. *V3* needs LROC NAC images. None were downloaded, so the shadow check has no input and no V3 result is claimed. *V4* rests on an assumed touchdown time (2025-03-06T17:28Z) and an assumed pit depth. At a panel height of 1.44 m the pit clears the Sun by only 2.7 cm, a panel at 2 m is not hidden ($d_c^*$ = 2.48 m), and the Sun moves about 0.5° per hour, so a few minutes of error in the touchdown time can flip the result (scoreboard.json V4 `notes`). It shows the model is consistent with the IM-2 reports and proves nothing more.

#figure(
  {
    let rows = (
      ("0 to 20 %", "2000", 0.43, "6.76", "+1.95", "0.62"),
      ("20 to 40 %", "2000", 0.47, "3.39", "+0.28", "0.89"),
      ("40 to 60 %", "2000", 0.48, "2.83", "+0.15", "0.83"),
      ("60 to 80 %", "118", 3.12, "9.46", "-2.74", "0.70"),
      ("80 to 100 %", "5", 1.86, "9.17", "-1.95", "0.63"),
    )
    table(
      columns: (2.2cm, 1.4cm, 1.5cm, 1.4cm, 1.4cm, 1.2cm, 1fr),
      align: (left, right, right, right, right, right, left),
      stroke: 0.5pt + luma(150),
      table.header([*Reference bin*], [*Pixels*], [*Median*], [*p90*], [*Mean*], [*r*], [*Median against the limit of 5*]),
      ..rows.map(r => (
        r.at(0), r.at(1), str(r.at(2)), r.at(3), r.at(4), r.at(5),
        box(width: 100%, height: 0.32cm, stroke: 0.4pt + luma(170),
          align(left, rect(width: r.at(2) / 5 * 100%, height: 100%, fill: rgb("#d97706"), stroke: none))),
      )).flatten(),
    )
  },
  caption: [V2 per fifth of the PGDA map (errors in points, ours minus reference). Source: `web/public/scoreboard.json`, V2 `notes`, "Per fifth (n, median, p90, mean, r)". The two brightest bins hold only 118 and 5 pixels in our tiles (\#75).],
) <tab-v2>

#figure(
  table(
    columns: (1cm, 1.3cm, 2.3cm, 2.3cm, 2.3cm, 2.3cm, 1fr),
    align: (left, right, right, right, right, right, right),
    stroke: 0.5pt + luma(150),
    table.header([*RoI*], [*Pixels*], [*Longest day, ours (d)*], [*Longest day, Barker (d)*], [*Longest night, ours (d)*], [*Longest night, Barker (d)*], [*Our night, pixel min to max (d)*]),
    [1], [20], [225.80], [215.79], [4.43], [5.29], [3.71 to 8.63],
    [2], [18], [222.20], [216.28], [6.63], [7.31], [5.33 to 10.79],
    [3], [9], [229.75], [216.89], [7.07], [8.41], [5.63 to 13.88],
    [4], [35], [233.07], [231.20], [4.11], [4.00], [3.46 to 6.29],
    [5], [26], [217.63], [218.98], [4.46], [4.52], [3.92 to 5.83],
    [6], [26], [220.72], [220.98], [3.95], [4.16], [3.54 to 4.71],
    [7], [19], [138.47], [119.07], [5.26], [5.42], [5.08 to 5.54],
  ),
  caption: [V2b, Barker 2021 Site 1 @barker2021: mean over the 20 m pixels near each RoI centroid, observer 5 m above ground, hourly 2024 and 2025. Source: `web/public/scoreboard.json`, V2b `notes` (LCIP = longest day, LCSP = longest night). Our 20 m DEM is coarser than Barker's 5 m and biases bright.],
) <tab-v2b>

In @tab-v2b our mean longest night is shorter than Barker's at six of the seven RoIs: against Barker's 5 m DEM our model leans bright. At SP-01 it leans the other way, because Mazarico's DEM is 240 m and ours is finer near the site.

== Speed, first load and hosting

The horizon is the costly step: 720 azimuths marched over the near, middle and far tiles. We timed it at two sites, IM-2 at a 2 m mast (high horizon, early exit) and SP-01 at a 10 m mast (low horizon, marches about 190 km), on a Windows 11 Pro desktop with an AMD Zen 2 CPU, 8 logical CPUs and 16 GB RAM. The headline numbers are the native release medians from the quieter run: *17.20 ms* at IM-2 and *33.66 ms* at SP-01, against a target of 50 ms. @tab-speed shows the re-runs of 2026-11-15 on a shared, busy machine. They are load-dependent, and 5–10 ms gaps between them are run-to-run noise. Every median in every run is under the 50 ms target; the single worst call, native SP-01 at 51.02 ms, was 1 ms over.

#figure(
  table(
    columns: (4.2cm, 1fr, 1fr),
    align: left,
    stroke: 0.5pt + luma(150),
    table.header([*Where*], [*IM-2, H = 2 m*], [*SP-01, H = 10 m*]),
    [Native, release], [median 18.94 ms, max 23.87 ms], [median 41.91 ms, max 51.02 ms],
    [WASM in Bun 1.4.4], [single-call median 12.55 ms, max 14.97 ms], [single-call median 28.51 ms, max 36.61 ms],
    [Browser, live app (Chromium 156)], [single-call median 18.15 ms, max 18.60 ms], [single-call median 34.25 ms, max 35.50 ms],
  ),
  caption: [Horizon computation time, re-runs measured 2026-11-15. Headline (quieter run): native median 17.20 ms at IM-2 and 33.66 ms at SP-01.],
) <tab-speed>

A cold load of the live app in Chromium 156 with a fresh profile transfers *34.3 MB* (34 269 727 B on the wire). The largest file is the far terrain tile, `far-240m.ebf340.bin`, at 21 780 000 B. The `.bin` tiles travel uncompressed, so they are 98 % of the bytes; the JS bundle is 225 KB on the wire and the WASM 52 KB. The load event fires at 740 ms, and data files keep arriving until about 2.9 s.

Hosting costs *\$0/month*: Cloudflare Pages serves static assets free and unlimited @cf-pages-pricing. We run *0* servers, and the app makes *0* runtime API calls: no API, no key, no database. Every data and code request on the cold load is a GET to `penumbra-lunar.pages.dev`. The one third-party host family is Google Fonts (`fonts.googleapis.com` and `fonts.gstatic.com`), so the app is not yet single-origin.

= Discussion

== Penumbra beside the tools planners already use

@tab-compare compares Penumbra with the three tools on the challenge page, from their public documentation as read on 2026-11-15. "Not found in docs" means we searched the tool's public docs and found nothing.

#figure(
  table(
    columns: (2.5cm, 2.5cm, 1fr, 1fr, 1fr),
    align: left,
    stroke: 0.5pt + luma(150),
    inset: 4pt,
    table.header([*Capability*], [*Penumbra*], [*NASA Moon Trek*], [*LROC QuickMap*], [*JPL Horizons*]),
    [Sunlight hours at a site and date, terrain-shadowed], [yes], [partial: a "Surface Lighting" tool, no per-site sunlit-hours total @moontrek @moontrek-egu23], [partial: Terrain Shadows over a UTC range, no sunlit-hours total @quickmap-shadows], [partial, no terrain: Sun rise and set from a site, against a horizon that allows dip "but not local topography" @horizons-manual],
    [Earth contact over time], [yes], [not found in docs (a Line of Sight tool exists) @moontrek-egu23], [partial: SUN+EARTH and EARTH shadow sources, no contact-hours output @quickmap-shadows], [partial: rise and set of target bodies from a site @horizons-manual],
    [First lethal shadow], [yes], [not found in docs], [not found in docs], [not found in docs],
    [30-date landing window], [yes, operable days for the next 30 landing dates], [not found in docs], [partial: start, end and delta-hours inputs, no sweep summary @quickmap-shadows @quickmap-time], [partial: raw start/stop/step tables only @horizons],
    [Slope at a point], [yes, from the 20 m DEM], [partial: Slope tool, registered account needed @moontrek @moontrek-egu23], [partial: elevation at a point and along a path; slope not found in docs @quickmap-query], [not offered (ephemeris service)],
    [Validated against Horizons and LOLA], [yes: V1 and V2], [not found in docs @moontrek-paper], [not found in docs], [it is the reference: Horizons serves DE441 @horizons-manual],
    [Runs with no backend], [yes: static files only], [no (server-based) @moontrek-paper], [no (server-based) @quickmap-shadows], [no (server-based) @horizons-manual],
    [Cost], [\$0, no account], [free to browse; lighting and slope need a registered account @moontrek], [price not found in docs; "Contact ACT" for Terrain Shadows @quickmap-shadows], [free; "No account or password is required." @horizons-manual],
  ),
  caption: [Penumbra beside the tools planners already use. Lookups on 2026-11-15.],
) <tab-compare>

The other tools are better in several ways. Moon Trek covers the whole Moon in one portal, with lighting, slope, Sun-angle and line-of-sight tools @moontrek @moontrek-egu23; Penumbra covers ten south-pole regions. QuickMap carries the full LROC data catalogue, a Sun+Earth terrain-shadow tool over any time range, and CSV export @quickmap-query; Penumbra ships only the layers its model needs. Horizons is the ephemeris authority (DE441), free and scriptable through an API @horizons-manual @horizons, and Penumbra's V1 test uses it as the judge. SUNLOCK @sunlock and Lunar Window @lunar-window also address this challenge; we did not compare features.

== Limitations and future work

Each limitation in @tab-future has a concrete next step.

#figure(
  table(
    columns: (4.4cm, 1fr),
    align: left,
    stroke: 0.5pt + luma(150),
    table.header([*Limitation*], [*Future work, with its path*]),
    [Runtime data uses 94.2 of the 100 MB budget], [\#104: ten regions at 7.02 MB each plus the 21.78 MB far tile, plus the 2.1 MB ephemeris and manifest, leave 5.8 MB. Before an eleventh region, state the rule in ARCHITECTURE §2.1: raise the budget, drop the mid tile where \#48 allows, or shrink tiles.],
    [The landing-window heatmap has only 3 grid dates (2025-03-06, 2026-05-31, 2026-11-14)], [Each grid costs 0.33 MB and about 3 minutes to build. Add dates with `penumbra-pipeline grid <data_dir> <region>` inside the \#104 budget. The per-site 30-date window (\#108) is computed live for any date.],
    [V3 (LROC NAC shadow check) not run], [Read START_TIME from each NAC's PDS label, threshold the NAC and NASA's `_MODEL.TIF` with Otsu, compare IoU; pass if within 0.05 of NASA's own model.],
    [Sites work only inside the ten shipped tiles], [\#103: a design note sizing on-demand tiles, a tile server, or a coarser any-site mode with its error stated. Building a tile takes seconds; shipping it is the limit.],
    [First load is 34.3 MB], [The `.bin` tiles are served uncompressed. Check Brotli on `.bin` at Cloudflare, and lazy-load the 21.78 MB far tile and the ten grids until a site is picked.],
    [Region outlines come from QuickMap's file host], [\#106: compare the USGS grids (doi 10.5066/P1MEQ6UK) @usgs-artemis-grids with the QuickMap outlines and record the result as a ledger row.],
    [Fonts load from Google, so the app is not single-origin], [Self-host IBM Plex Mono and Archivo as woff2 under `web/public/` and drop the three font links in `web/index.html`.],
    [The WASM timing bench is not in the repo], [Commit it as `scripts/bench-horizon.ts` so the WASM row can be re-run with one `bun` command.],
  ),
  caption: [Limitations, each written as future work.],
) <tab-future>

The model also leaves out some physics on purpose. It has no limb darkening and no charge or discharge efficiency, terrain smaller than a 20 m pixel is not in the DEM, and lander values are assumed unless marked cited. Mast height changes illumination at the pole @glaser2018, so the app shows the observer height in every view.

= Commercialization

A lander that loses power early loses its mission, and these missions are expensive. CLPS task orders (as of Feb 2024) were \$122.0 M for IM-1, \$50.0 M for IM-2 and \$322.8 M for Astrobotic Griffin; the VIPER rover was \$505.4 M @oig-clps. CLPS is capped at \$2.6 B through November 2028 @nasa-clps.

The buyers are CLPS lander vendors choosing sites and dates, NASA and partner landing-site teams for the crewed south-pole landing, rover and ISRU teams sizing power and comms at a stop, and universities teaching mission design. The model is open core. The app and the Rust model stay free and Apache-2.0, so anyone can rerun V1 and V2. Paid work covers custom sites and dates, finer DEMs and a signed sign-off report.

As a price anchor we use *our own illustrative number, not a sourced figure*: \$25 000 per site-and-date study, which is 0.05 % of a \$50 M task order @oig-clps. It is not a market price. The timing is now: Artemis IV lands in 2028 @nasa-artemis-iv, Blue Moon MK1 in 2027 @nasa-blue-moon, four more south-pole deliveries are listed @nasa-clps-deliveries, and China's Chang'e-7 is planned for the second half of 2026 @change7.

= Conclusion

Penumbra tells a planner how many hours of sunlight and Earth contact a lander gets at a south-pole site and date, and when the first lethal shadow arrives. One Rust model runs natively for validation and as WASM in the browser, with no backend and \$0/month hosting. It agrees with JPL Horizons to 0.00011° against a 0.002° threshold (V1) and with NASA's average-visibility map to a pooled median of 0.48 points (V2), and all 7 red-team cases pass (V5). V2b fails on SP-01 by 0.21 days, V3 was not run, and V4 is a reconstruction; the scoreboard says so in the app and in this paper. The next south-pole landers can check a site and a date in a browser before they fly.

#bibliography("refs.yml", title: "References", style: "ieee")

#heading(numbering: none, level: 1)[Software and fonts]

#set text(size: 9pt)
- ANISE, pure-Rust SPICE reader: #link("https://github.com/nyx-space/anise")
- wasm-bindgen, Rust to WebAssembly bindings: #link("https://github.com/rustwasm/wasm-bindgen")
- Svelte 5: #link("https://svelte.dev") · Vite: #link("https://vite.dev")
- Three.js: #link("https://threejs.org") · uPlot: #link("https://github.com/leeoniya/uPlot")
- Bun: #link("https://bun.sh") · Typst: #link("https://typst.app")
- Fonts in the app: IBM Plex Mono (#link("https://github.com/IBM/plex")) and Archivo (#link("https://fonts.google.com/specimen/Archivo")), served by Google Fonts
