// D1 project document (docs/SUBMISSION.md). Build: bun run paper -> docs/paper/penumbra.pdf
// Skeleton only until Nov 14: headings, placeholders and one figure slot per V-test. No results.

#set document(title: "Penumbra", author: ("Qi Cai", "Charls"))
#set page(paper: "a4", margin: 2cm, numbering: "1")
#set text(size: 10.5pt)
#set heading(numbering: "1.1")
#set math.equation(numbering: "(1)")

#let todo(body) = text(fill: rgb("#b45309"))[[TODO: #body]]

// A figure slot: grey box until the V-test result exists. Swap the rect for image(...) on the day.
#let slot(id, what) = figure(
  rect(width: 100%, height: 5cm, fill: luma(240), stroke: luma(180))[
    #align(center + horizon, text(fill: luma(110))[#id figure slot: #what])
  ],
  caption: [#id — #todo[caption, judge, threshold, result from `scoreboard.json`]],
)

#align(center)[
  #text(size: 20pt, weight: "bold")[Penumbra] \
  #text(size: 12pt)[Sunlight, Earth contact and the first lethal shadow at lunar south-pole landing sites] \
  #v(4pt)
  Team Fractured Toes: Qi Cai, Charls · NASA Space Apps Challenge 2026, Kuala Lumpur \
  #link("https://penumbra-lunar.pages.dev")
]

= Introduction
#todo[the planner's question; IM-1 and IM-2 as motivation; the "Next Frontier" theme; what Penumbra answers per site and date]

= Methodology
== Data
#todo[one row per dataset with its role] @pgda-lola-spole @pgda-illumination @naif-kernels @horizons

== Ephemeris and frames
#todo[MOON_ME via DE440 + Moon PA PCK + frame kernel, apparent positions, UTC to TDB] @park2021 @anise

== Terrain horizon
#todo[great-circle march over the 20 m near field and 240 m far field] @barker2023

== Solar and Earth disc visibility
#todo[disc fraction against the horizon; visibility vs illumination metric] @mazarico2011

== Survival model
#todo[state of charge and time to first lethal shadow; Earth contact threshold 2°] @derosa2012

= Results
== V1 — Ephemeris against JPL Horizons
#slot("V1", "Sun and Earth az/el error, 20 site/time pairs")

== V2 — Visibility against PGDA AVGVISIB
#slot("V2", "residuals per value bin, Sun and Earth")

== V3 — Shadows against LROC NAC
#slot("V3", "IoU of ours vs NASA model vs NAC")

== V4 — IM-1 and IM-2 reconstruction
#slot("V4", "horizon and Sun track at touchdown")

== V5 — Red-team cases
#slot("V5", "pass/fail table of edge cases")

= Discussion
#todo[limits: DEM resolution, observer height, unknowns U1–U20; comparison with Moon Trek, QuickMap, Horizons; commercialization] @barker2021 @glaser2018

= Conclusion
#todo[]

= Use of AI
#todo[from the D7 draft]

#bibliography("refs.yml", title: "References", style: "ieee")
