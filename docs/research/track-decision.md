# Track decision (Nov 8)

From 4 questions asked to each NotebookLM notebook (raw answers: `docs/research/<track>/decision-qa.md`).

| | Lunar Landing | Hot Spots | Earth Analogs | Trend Detective |
|---|---|---|---|---|
| Hard core | Horizon maps from LOLA + DE440/SPICE Sun/Earth vectors (libration, MOON_ME frame) | Cross-sensor count/FRP harmonization: footprint, bow-tie, VZA, orbital drift | Multi-dimensional similarity across mismatched scales | Autocorrelation-corrected MK/Sen + FDR, deseasonalization, breakpoints |
| Data → web | LOLA 0.5–5 GB + kernels → **~10–100 MB** precomputed | FIRMS archive tens–hundreds GB → 100–500 MB gridded | ~20 GB DEMs → ~250 MB | Subset via Harmony/AppEEARS → small |
| Gap | No tool gives **joint Sun + Earth windows** vs real horizon, side-by-side site × date | No sensor-invariant AOI burn calendar with provenance; naive concatenation fakes trends | No unified score with thermal/mineral/ice/ops + uncertainty | No tool pairs exploration with honest field significance |
| Validation in 48 h | **Horizons cross-check <0.006°**; reproduce Mazarico/Barker (Shackleton 82–92% illum., PSR ~21,000 km²); back-test IM-2 tipping | Reproduce 2023 Canada (~6 SD); MODIS↔VIIRS parity within +3.6% | Reproduce published analog distances; catalogs (Pit Atlas, MGC3) | COVID NO₂ −48%; Antarctic ice 2014–17 drop; GRACE Greenland ~279 Gt/yr |
| Visual | 3D south pole, horizon + Sun/Earth tracks | 2D map + calendar | 3D globe | maps + charts |
| Prior art risk | rivals exist (CLPS-MOON-BROWSER, lunarwindow, "HAZAKIKO 2026") | several hackathon repos | **terrestrial-analog-finder already defines the score formula** — notebook leans on it | Giovanni/Worldview strong |

## Recommendation: **Lunar Landing Site Navigator**
- Cleanest, most objective validation: an independent oracle (JPL Horizons) plus published numbers plus a real failure (IM-2) to back-test.
- Smallest runtime data → static deploy, fast 3D, Rust/WASM horizon engine fits.
- Clear two-person split: science core (ephemeris + horizon engine + validation) / app (3D, comparison UI, demo).
- Differentiate vs rivals with: joint Sun∧Earth "operable window" metric, DEM-error uncertainty bands (Barker 2020), SAM-segmented boulder/shadow hazards, IM-2 back-test as the story hook.

**Runner-up: Hot Spots** — most timely (Suomi NPP dropped from FIRMS Nov 2, 2026) with strong validation, but heavier data pipeline and weaker visuals.
**Drop: Earth Analogs** (rival already owns the core formula), **Trend Detective** (dashboard risk, least novel).

Next: verify the rival repos + "HAZAKIKO 2026" before committing, then Opus 5.5 high → `/be-ambitious`.
