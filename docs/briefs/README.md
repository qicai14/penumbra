# Implementation briefs

Progress: COMPLETE (Nov 11) · 11 briefs (T00, T01, T02, T05, T06, T07, T08, T09, T10, T15, T17) · two script passes done and applied: every [S] value was recomputed, and §3 of T00, T06, T08, T09, T15 and T17 was implemented literally in a scratch script and run against §5 · what is still [U] needs real data on Nov 14 or a compile.

One brief per science ticket on the critical path of [PLAN-DRAFT](../PLAN-DRAFT.md). Each is written so that Opus at medium effort can run the ticket test-first on Nov 14 without Fable. Documents only: there is no code here, and every signature inside `crates/core` is a proposal. Only [ARCHITECTURE §3.2](../ARCHITECTURE.md) is frozen.

## How to use a brief

1. `/ticket <n>`, then read the brief for that ticket before the spec.
2. Section 5 of each brief is the test table. Write those tests first, in the order of section 6.
3. If a "stop and escalate" line fires, stop. Don't change a tolerance to get green.

Tags in the test tables:
- **[S]** = recomputed by an independent script on Nov 11 (the script is not in the repo).
- **[R]** = taken from SCIENCE-SPEC-REVIEW §1, which script-checked it on Nov 10.
- **[U]** = unverified: from memory, or it needs real data on Nov 14.
- **[D]** = a decision this brief made where the documents are silent. All are listed below for Qi Cai.

## Order

| Order | Brief | Ticket | Needs data? | Blocks |
|---|---|---|---|---|
| 1 | [T00-hour0-api-fixture](T00-hour0-api-fixture.md) | T00 | no | everything |
| 2 | [T01-site-geometry](T01-site-geometry.md) | T01 | no | T02, T06, T07 |
| 3 | [T02-ephemeris-tracer](T02-ephemeris-tracer.md) | T02 | kernels | T05, T08 |
| 4 | [T05-v1-horizons](T05-v1-horizons.md) | T05 | Horizons fixtures | T10 |
| 5 | [T06-dem-tiles](T06-dem-tiles.md) | T06 | DEM | T07 |
| 6 | [T07-horizon-march](T07-horizon-march.md) | T07 | tiles (tests need none) | T08, T10 |
| 7 | [T08-disc-fraction-timeline](T08-disc-fraction-timeline.md) | T08 | no (tests) | T09, T15, T17 |
| 8 | [T09-survival](T09-survival.md) | T09 | no | T10 |
| 9 | [T10-v5-red-team](T10-v5-red-team.md) | T10 | PSR file for case 3 | Must complete |
| 10 | [T15-im2-pit](T15-im2-pit.md) | T15 | replay segment | T16 |
| 11 | [T17-sunset-band](T17-sunset-band.md) | T17 | no | T18 |

Gate G1 (DEM late at Sat 14:00): do brief 8 (T09, pure) before brief 5.

## What Charls's tickets wait for

Charls builds every view on the fixture and flips one adapter switch per science merge.

| Charls's ticket | Fixture from | Flips to real when | The brief that defines the shape |
|---|---|---|---|
| T03 panorama | T00 | T02 (tracks), T07 (skyline) | T00 §3, T02 §2, T07 §2 |
| T11 site and date controls | T00 | T06 (near tile for the height image, `ground_height`) | T06 §2 |
| T12 timelines | T00 | T08 | T08 §2 |
| T13 scoreboard | T00 (sample row) | T05 (V1 row), T10 (V5 rows) | T05 §3 step 8, T10 §3 |
| T14 headline and preset form | T00 | T09 | T09 §2 |
| T16 IM-2 replay view | — | T15 | T15 §2 |
| T18 band display | — | T17 | T17 §2 |

Nothing of Charls's waits for T01, T05's tests or T10's tests; they change no shape.

## Shared conventions (every brief assumes these)

- R = 1 737 400 m (E30, E31a). Body frame MOON_ME. f64 only (N7).
- Inside `core`: radians, metres, km for ephemeris vectors, Unix seconds UTC. The functions that return a frozen array layout (`sun_earth_track`, `timeline`, `survival`, `Horizon::to_api`) return **degrees**, exactly as ARCHITECTURE §3.2 lists them, so `wasm` passes them through with no maths and native tests see the browser's numbers.
- Azimuth is clockwise from north, in [0, 360). Longitude is east-positive, output in [0, 360).
- Errors: one enum in `crates/core/src/error.rs`, `Error { NotLoaded, OutOfCoverage, NoData, OutOfTable, BadInput }`, with `code()` returning `NOT_LOADED`, `OUT_OF_COVERAGE`, `NO_DATA`, `OUT_OF_TABLE`, `BAD_INPUT`. `wasm` turns it into `JsError::new("<CODE>: <detail>")`. A NaN or infinite argument is `BadInput`.
- A timeline sample k stands for the interval [t_k, t_k + Δt). All n samples count in sums, so 4320 samples at 600 s are exactly 30 days.
- Proposed files in `crates/core/src/`: `error.rs`, `geom.rs` (T01), `eph.rs` (T02), `dem.rs` (T06), `horizon.rs` (T07), `disc.rs` and `timeline.rs` (T08), `power.rs` (T09), `pit.rs` (T15), `band.rs` (T17). Tests go in `crates/core/tests/<same name>.rs`.
- Native and WASM maths libraries differ in the last digits. No test compares bit for bit except where a brief says "exactly".

## Decisions made here where the documents are silent [D]

Confirm or overrule on Nov 13. Each brief marks where it uses one.

| # | Decision | Used in | Why it had to be made |
|---|---|---|---|
| D1 | The far march ends without error when the ray leaves the far tile at 200 km or more. It returns `OUT_OF_COVERAGE` only when the tile ends before 200 km and the early exit has not fired. | T07, T10 case 6 | SCIENCE-SPEC §1a says "march until … the far-field tile ends"; V5 case 6 wants an error 5 km from the edge. Both hold only with a minimum range. 200 km is the spec's own crop rule (E66, E100). |
| D2 | `survival` starts at the first sample with t ≥ landing. `operable_h`, `sunlit_h` and `contact_h` are summed from landing to the lethal time (or the end). `longest_dark_h` and `longest_any_sun_h` are over the whole span from landing, because they describe the site. | T09 | ARCHITECTURE §2.5 defines the sums but not their span. |
| D3 | The Sun-radius check in T08 is 0.2614°–0.2719°, not the ticket's 0.262°–0.271°. | T08 | The ticket's band is for the Earth's distance from the Sun. The Moon is up to 406 700 km nearer or farther, which gives 0.26149°–0.27185°. Correct code would fail the ticket's band by 0.0005°–0.001°. |
| D4 | T17's checks "0.5 m at 1 km is 0.03°" use the obstacle error alone (observer error 0). With both ends at 0.5 m the value is 0.0405°. | T17 | The ticket's number and the ticket's formula differ by √2. |
| D5 | A sunrise or sunset in T17 is a sign change of m = el_sun − el_hor(az_sun) between two samples (Sun's centre on the horizon). | T17 | SCIENCE-SPEC §1f names m(t) but not the event. |

## Findings for the plan

Found while writing the briefs. Each is written into the brief named; none is fixed in PLAN-DRAFT or the spec.

- **`tiff` may not read the DEM.** `tiff` 0.11.3 documents floating-point predictor 3 as "Not currently supported". If the PGDA GeoTIFFs use it, T04 fails and the GDAL fallback is needed ([T06 §4](T06-dem-tiles.md)). Check one file's predictor tag first on Nov 14.
- **T08's Sun-radius band fails correct code.** The ticket says 0.262°–0.271°; from the Moon it is 0.26149°–0.27185° (D3).
- **T17's check and formula differ by √2.** "0.5 m at 1 km is 0.03°" holds for the obstacle error alone (D4).
- **ANISE needs `default-features = false`.** Its default features pull in a `ureq` downloader. `Aberration::LT_S` exists per docs.rs for 0.10.6; not compiled here (U14, [T02 §3a](T02-ephemeris-tracer.md)).
- **Pin margin is 7 040 m, not 7 000 m**, from the near-tile edge, so the 7 km march stays inside the valid region ([T06 §4](T06-dem-tiles.md)). T11 needs this number.
