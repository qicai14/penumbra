# T19 / V2b longest-night table check: method brief (issue #20)

From a science-hard method pass. Corpus quotes are in the gitignored `docs/research/lunar-landing/corpus/` of the main checkout.

## Definitions
- **Barker:** LCIP is the longest uninterrupted run with any fraction of the Sun visible (lit means f > 0, so a partial disc counts as lit). LCSP is the longest run with none of the Sun visible (f == 0). Sources: b5251ef3:1106–1110, E123.
- **Mazarico, total night:** f == 0. Pass metric: 2.75 d at 10 m (7b941912:2294–2296).
- **Mazarico, partial night:** f < 1. Reference 7.4 d at 10 m. Report only.
- **Mazarico, day lengths:** f > 0 and f == 1. Report only; there is no 10 m reference value.
- `disc_fraction` returns exactly 0.0 or 1.0 for a hidden or clear disc, so exact comparisons are safe.

## Observer height
`mast_m` above `ground_height_m(near, ll)` at each pixel: 5.0 for Barker, 10.0 for Mazarico. Pass the same value to `horizon(.., mast_m, ..)` and to `Site { ground_m, mast_m }`. `psr.rs` shows the pattern.

## Barker averaging
1. Compute LCIP and LCSP per pixel.
2. Take the arithmetic mean over the RoI pixels.

Do not run on a mean series. Case A against case C proves this order (b5251ef3:549, 1207–1209, 682 vs 698).

RoIs 1–7. These are also in `crates/pipeline/tests/dem_tiles.rs:75`.

| RoI | X (km) | Y (km) | Area (m²) |
|---|---|---|---|
| 1 | −11.5825 | −12.4625 | 8625 |
| 2 | −11.4875 | −12.5575 | 7550 |
| 3 | −11.3525 | −12.5025 | 3825 |
| 4 | −11.2875 | −12.1675 | 13300 |
| 5 | −11.1025 | −11.9425 | 10075 |
| 6 | −10.8025 | −11.7375 | 10000 |
| 7 | −10.2275 | −11.5525 | 7075 |

Radius = √(Area/π). Select pixels with d² ≤ r², inclusive.

Expected pixel counts:
- RoIs 1–7: 20, 18, 9, 35, 26, 26, 19.
- SP-01 at (−11.31, −12.26) km within 150 m: 180. Six pixel centres sit exactly on 150 m, so a strict `<` gives 174.

Grid pixel centres are at −303 990 + 20k m (watch the half-pixel offset, P7).

## Windows (hourly, end exclusive)
| Run | t0 | Steps | Span |
|---|---|---|---|
| Barker | 1 704 067 200 | 17 544 | 2024-01-01T00Z, 2 years |
| SP-01 | 1 577 836 800 | `psr::STEPS` = 163 048 | 2020-01-01T00Z, 18.6 years, assumed (U20) |

A run cut off by the window counts at its length inside the window and is not extended. Flag whenever the longest run touches step 0 or step n−1; for SP-01 such a value is a lower bound. Stay hourly.

Estimated run time: about 25 s single-core. Measured: one 163k-step `timeline()` takes 88 ms; one horizon takes 22 ms.

## Reuse
- `ShippedTiles::load(web/public/data)` with `.near("shackleton-ridge")` and `.far`.
- `geometry::{MapXy, xy_to_latlon}`, `horizon::{horizon, ground_height_m, HorizonOptions::default()}`.
- `pipeline::eph::{load_almanac, build_table}` with `Ephemeris::add`, kernels from `PENUMBRA_DATA` (as `psr::run` does).
- `timeline::timeline(.., DEFAULT_EARTH_MARGIN_DEG.to_radians())`, using its `illumination` column.
- `power::longest_run`: make it `pub`; do not copy it.

## Files
- New `crates/pipeline/src/table_check.rs` and `pub mod table_check;` in `lib.rs`.
- A `v2b <data_dir>` arm in `main.rs` that prints JSON, like `v4-im2`.
- `crates/pipeline/tests/v2b_table.rs` with the tests below, plus an `#[ignore]` real run that panics without `PENUMBRA_DATA`, like `v5_psr.rs`.

## Tests
1. **Thresholds.** f = [1,1,0.5,0,0,0,0,0.2,1,1,1,1,0,0] at 1 h steps gives:
   - LCIP (f > 0): 5 h
   - LCSP (f == 0): 4 h
   - full day (f == 1): 4 h
   - partial night (f < 1): 6 h
2. **Averaging order.** Pixel A = 1×4, 0×4, 1×4 and pixel B = 0×4, 1×4, 0×4. The RoI mean gives LCIP 4 h and LCSP 4 h. A mean series would wrongly give 12 h and 0 h.
3. **Edge.** [0,0,0,1,1,1,1,1,0] gives LCSP 3 h with touches_edge = true.
4. **Selector.** The pixel counts above.

## Scoreboard row (`web/public/scoreboard.json`, same keys as existing rows)
- `id`: "V2b"
- `judge`: "Barker 2021 Table 2 (Site 1, case A); Mazarico 2011 SP-01"
- `metric`: Barker RoIs passing LCSP ≤ LCSP-99 + 1 d and LCIP ≥ LCIP-1 − 10 d (of 7); SP-01 2.75 d inside the pixel range
- `threshold`: 6
- `value`: number of RoIs passing
- `pass`: Barker passes ≥ 6 AND 2.75 ∈ [min, max] of SP-01 total night
- `sample_size`: 7
- `notes` must contain:
  - "One-sided: catches a model that is too dark, not one too bright."
  - The DEM each reference used (P12): Barker 5/80/240 m (b5251ef3:1032–1034); Mazarico 240 m (E06); ours 20 m to 7 km, then 240 m.
  - Per RoI: ours against the reference, with min and max pixel LCSP.
  - Mazarico "assumed (U20)": total night min..max; day f > 0 max; day f = 1 max; partial night f < 1 (ref 7.4); edge flags.
  - The command that produced the numbers.

Caveats for the notes:
- Our 20 m DEM is coarser than Barker's 5 m and biases bright (E71).
- The equal-area disc can take in darker flank pixels (risk of a false miss).
- Mazarico's 6 h step gives about ±0.25 d.
