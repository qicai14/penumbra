# Data notes

Facts about the downloaded NASA files that the code relies on. Raw files live in `PENUMBRA_DATA` (see `scripts/bootstrap.sh`).

## PGDA GeoTIFFs (T04, #5)

Header facts from `stack_preflight::tiff_reads_dem_headers` (Nov 12), all three DEMs: BigTIFF, Deflate (8), floating-point predictor (3), Float32, 512² tiles, tiepoint upper-left (−304 000, 304 000) m at 20 m (240 m file: ±931 200), GTRasterType = 1 pixel-is-area, no-data "nan".

Tile decode, from `stack_preflight::tiff_decodes_one_psr_tile` (asserted unless marked *measured*; measured Nov 14 on a Windows desktop, `tiff` 0.11.3, debug test build, file on HDD):

- `read_chunk(index)` decodes one tile as `DecodingResult::F32` from `LPSR_80S_20MPP_ADJ.TIF` and `LDEM_80S_20MPP_ADJ.TIF`. Only that tile is read; the whole raster would be 30 400² × 4 B = 3.7 GB (N6). No GDAL conversion needed. *Measured:* 25–35 ms per LPSR tile, 90 ms per LDEM tile.
- Both 20 m files: 30 400 × 30 400 px, 60 × 60 = 3 600 tiles. Tile index = tile_row × 60 + tile_col: TIFF 6.0 §15 orders tiles left to right, then top to bottom (not checked by the test; the centre tile is on the diagonal).
- **Edge tiles are not padded.** The last tile (3599) holds 192 × 192 px and `read_chunk` returns 36 864 samples, not 512². Index edge-tile pixels with `chunk_data_dimensions(index).0` as the row stride.
- **LPSR is a PSR mask, not heights.** The centre tile (row 29, col 29; pixel 15 200 is x = 0 by the tiepoint, so it holds the pole) has only 0.0 and 1.0 and no NaN. *Measured:* 19 076 of 262 144 pixels are 1. Heights come from `LDEM_*` only.
- LDEM centre tile: all finite and within ±10 km (predictor 3 decodes to plausible metres). *Measured:* −1 219 .. 1 552 m.

## DEM tiles (T06, #7)

Read Nov 14 from the files themselves, before any crop. Asserted by `dem_source::u5_u6_tags_of_the_20m_dem` and `…_240m_dem` (local-only, `--ignored`).

**U5, `LDEM_80S_20MPP_ADJ.TIF`:**
- ModelTiepointTag: pixel (0, 0) ↔ (−304 000, +304 000) m, the upper-left corner. ModelPixelScaleTag: 20 m, square. 30 400 × 30 400 px.
- GTRasterTypeGeoKey (1025) = 1, pixel-is-area. So pixel (col, row) has its centre at x = −304 000 + (col + 0.5)·20, y = 304 000 − (row + 0.5)·20 (SCIENCE-SPEC §1a). The GMT metadata in the file agrees: `node_offset = 1` (pixel registration).
- No-data: `GDAL_NODATA = "nan"`, `_FillValue = nan`, Float32.
- Lon-0 axis: confirmed along +y on decoded heights. With x = r·sin λ, y = r·cos λ, SP-01 is 4.6 m above the mean of its 200 m ring, and all seven Barker Site 1 RoI centroids are 7.3 to 10.8 m above theirs (`dem_tiles::t_1a_2_*`, run on the shipped tile).
- The GMT `x#actual_range` says {−310, 310} km, but the tiepoint and size give ±304 km. The code uses the TIFF tags; the GMT text is a leftover of the source grid.
- `LDEM_60S_240MPP_ADJ.TIF`: same facts at 240 m, tiepoint (−931 200, +931 200) m, 7 760 × 7 760 px.

**U6, height reference:** heights are metres above a sphere of radius 1 737 400 m. GeogSemiMajorAxisGeoKey (2057) = 1 737 400.0 in both DEMs, and GeoAsciiParams says `Datum = Moon (2015) - Sphere`, `Ellipsoid = Moon (2015) - Sphere`. That matches R in SCIENCE-SPEC §1 (E30). So U6 is settled as "yes" for these files.

**Shipped tiles** (`penumbra-pipeline dem web/public/data`; the manifest gets `far` and `regions`, and keeps `ephemeris`):

| Tile | Centre (map km) | Size | Upper-left (m) | Scale | Relief |
|---|---|---|---|---|---|
| `near-mons-mouton` | IM-2 site (77.1, 138.0) | 1536² at 20 m | (61 740, 153 360) | 0.03709 m | 2 431 m |
| `near-shackleton-ridge` | (−11.0, −12.0), between SP-01 and the Site 1 RoIs | 1536² at 20 m | (−26 360, 3 360) | 0.06655 m | 4 361 m |
| `mid-mons-mouton` (#48) | same as its near tile, ±20 m | 992² at 80 m (4 × 4 means of the 20 m source) | (37 440, 177 680) | 0.1284 m | |
| `mid-shackleton-ridge` (#48) | same as its near tile, ±20 m | 992² at 80 m | (−50 640, 27 680) | 0.0761 m | |
| `far-240m` | the pole | 3184² at 240 m (±382.08 km) | (−382 080, 382 080) | 0.5 m | |

- Near tiles snap to the source pixel grid, middle tiles to its 4-pixel blocks, so no resampling beyond the 4 × 4 mean. The offset is the middle of the tile's height range. No tile has a no-data pixel.
- Scale (#48): near and middle tiles take the finest scale that holds them, relief / 65 534 (lowest pixel −32 767, highest +32 767); before #48 the near tiles were 0.0625 and 0.125 m. The far tile is unchanged (same bytes, same hash `5b21e3`).
- Payload (#48): near tiles 4 718 592 B each (unchanged size), middle tiles 1 968 128 B each, far 20 275 712 B. Terrain total 33 649 152 B (33.6 MB), up 3 936 256 B (+13 %) from 29.7 MB. A region costs 6.69 MB instead of 4.72 MB.
- Round trip, 1 000 seeded pixels per tile, raw `read_chunk` against the core sampler at pixel centres: worst errors 0.0185 m, 0.0333 m and 0.25 m, at most half a step each (0.031 m and 0.0625 m before #48).
- Inside the inner 836² px (7 km border) of each near tile: 18 627 (Mons Mouton) and 21 508 (Shackleton ridge) LPSR = 1 pixels.
- Native build of all three tiles: about 1 s in release mode (file on HDD, Windows desktop).

## Terrain horizon (T07, #8)

Run Nov 14: `PENUMBRA_DATA=D:\DownloadHDD\penumbra-resource target/release/penumbra-pipeline horizon web/public/data` (release, LTO). Machine: Windows 11 Pro desktop, AMD Zen 2 (AMD64 Family 23 Model 113), 8 logical CPUs, 16 GB RAM, DEM on HDD. The output is deterministic; two runs gave the same angles.

**Shipped-tile cross-check, IM-2, H = 2 m** (`horizon_shipped::im2_*`, runs in CI): h_s = 5321.92 m. Near + mid + far (#48): the highest horizon is 4.0617° at az 117.5° (obstacle 16 520 m away), the lowest −1.4703° at az 328.0° (15 160 m). No bin has its obstacle under 60 m, and early exit on and off give bitwise-equal results. Before #48 (near + far only) it was 4.0427° at 16 600 m and −1.4750° at az 328.5° (14 920 m), which matched the TS reference marcher in the T07 design; the three-tier values have no outside reference beyond T-1a-4 (0.05° of the float 20 m DEM).

### U18: time for one horizon

| Where | IM-2, H = 2 m | SP-01, H = 10 m |
|---|---|---|
| Native, release, 1 warm-up + 20 calls, near + mid + far (#48, Nov 15) | median 17.20 ms, max 18.36 ms | median 33.66 ms, max 36.64 ms |
| Native, release, 1 warm-up + 20 calls, two tiers (before #48) | median 9.78 ms, max 10.62 ms | median 22.06 ms, max 22.98 ms |
| WASM release build, near + mid + far (#48, Nov 15), Bun 1.4.4 (JavaScriptCore), 1 warm-up + 20 `engine.horizon()` calls, 3 runs | median of run medians 14.61 ms (runs 14.92, 14.61, 14.39), worst max 18.89 ms | median of run medians 31.82 ms (runs 34.34, 30.48, 31.82), worst max 61.83 ms (one call, other runs 35.20 and 48.61) |
| WASM release build, two tiers (before #48), same method, one run | median 9.57 ms, max 11.03 ms | median 20.88 ms, max 23.29 ms |
| Browser, near + mid + far (#48, Nov 15): Claude desktop app pane (Chromium, Windows 11, same 8-logical-CPU Zen 2 machine), Vite dev server, `performance.now()` around 20 `engine.horizon()` calls / 20, 3 runs after 1 warm-up | 14.66, 14.20, 13.28 ms per call (median 14.20) | 26.98, 26.03, 25.92 ms per call (median 26.03) |
| Browser, two tiers (before #48), same method | 9.87, 10.29, 11.18 ms per call (median 10.29) | 23.48, 23.50, 27.22 ms per call (median 23.50) |

The WASM and browser rows are re-measured with the middle tier (Nov 15): IM-2 9.57 → 14.61 ms in WASM and 10.29 → 14.20 ms in the browser, SP-01 20.88 → 31.82 ms and 23.50 → 26.03 ms (before → after). The benchmark is a throwaway script (load far, near and mid for the region from `manifest.json`, one warm-up, 20 timed calls at IM-2 H = 2 m and SP-01 H = 10 m); it is not in the repo. Bun and native rows were taken minutes apart on a shared machine, so the 1–3 ms gaps between them are within run-to-run noise. The middle tier adds 300 samples per azimuth, so native time rose by 1.5–1.8×. Every time with the middle tier, native, WASM and browser, is below the 50 ms target (ARCHITECTURE §2.2) and far below the 2 s threshold that fires slip cut 3. No cut is needed. The SP-01 run takes longer because its horizon is low (−0.86° at best), so the early exit fires late and most azimuths march to about 190 km.

### T-1a-4: tier ablation

**Re-run for #48 (Nov 15).** Shipped run: int16 20 m near tile out to 7 km, int16 80 m middle tile out to 31 km, then the 240 m far tile, all at the scales above. Same reference, points and filter as below. Test: `horizon_shipped::t_1a_4_shipped_horizon_is_within_0_05_deg_of_the_float_20m_dem` (`--ignored`, PENUMBRA_DATA, ~16 s in release).

| Point | H | Max \|Δel\|, dist ≥ 60 m (before #48) | where (shipped / ref) | RMS ≥ 60 m | bins > 0.05° | Max \|Δel\| by ref distance: <60 m · 60 m–7 km · 7–15 km · 15–100 km · >100 km |
|---|---|---|---|---|---|---|
| IM-2 | 0 | 0.0140° (0.0749°) | az 290.0°: 0.8550° @ 9 880 m / 0.8690° @ 9 900 m | 0.0041° | 0 | 0.0143 · 0.0048 · 0.0140 · 0.0098 · – |
| IM-2 | 2 | 0.0140° (0.0751°) | az 290.0°: 0.8434° @ 9 880 m / 0.8574° @ 9 900 m | 0.0037° | 0 | – · 0.0032 · 0.0140 · 0.0098 · – |
| SP-01 | 0 | 0.0334° (0.0663°) | az 278.0°: −2.1567° @ 60 m / −2.1233° @ 60 m | 0.0108° | 0 | 0.1177 · 0.0334 · – · 0.0126 · – |
| SP-01 | 2 | **0.0382°** (0.0669°) | az 218.5°: −1.6902° @ 60 m / −1.6521° @ 60 m | 0.0118° | 0 | 0.0401 · 0.0382 · – · 0.0126 · – |
| Mons Mouton highest | 0 | 0.0094° (0.0382°) | az 6.5°: −1.1946° @ 14 840 m / −1.1852° @ 14 840 m | 0.0030° | 0 | 0.0819 · 0.0065 · 0.0094 · 0.0068 · – |
| Mons Mouton highest | 2 | 0.0094° (0.0397°) | az 6.5°: −1.2023° @ 14 840 m / −1.1929° @ 14 840 m | 0.0028° | 0 | – · 0.0064 · 0.0094 · 0.0068 · – |
| Mons Mouton lowest | 0 | 0.0039° (0.0083°) | az 308.0°: 9.1358° @ 220 m / 9.1397° @ 220 m | 0.0013° | 0 | – · 0.0039 · – · – · – |
| Mons Mouton lowest | 2 | 0.0039° (0.0068°) | az 308.0°: 8.6288° @ 220 m / 8.6327° @ 220 m | 0.0012° | 0 | – · 0.0039 · – · – · – |
| Shackleton lowest | 0 | 0.0281° (0.0570°) | az 307.5°: 1.7472° @ 38 920 m / 1.7754° @ 38 900 m | 0.0054° | 0 | – · 0.0140 · 0.0208 · 0.0281 · 0.0043 |
| Shackleton lowest | 2 | 0.0281° (0.0570°) | az 307.5°: 1.7443° @ 38 920 m / 1.7724° @ 38 900 m | 0.0052° | 0 | – · 0.0011 · 0.0208 · 0.0281 · 0.0043 |

**Result: T-1a-4 passes. The worst bin with dist ≥ 60 m is 0.0382° (SP-01, H = 2 m, an obstacle 60 m away).** Without the 60 m filter the largest difference is 0.1177° (SP-01, H = 0, az 202.5°, the first pixel at 20 m, P11/P13). Shipped against float h_s: 5321.918/5321.906, 1955.023/1955.010, 5807.831/5807.813, 4832.325/4832.324, −575.991/−575.982 m.

**Options measured for #48** (max \|Δel\| with dist ≥ 60 m over the five points and both H; same reference). The worst bin of each:

| Option | Worst | Fails at | Terrain payload |
|---|---|---|---|
| Shipped before #48 (0.0625/0.125 m, 20 m to 7 km, 240 m beyond) | 0.0751° | IM-2, SP-01, Shackleton lowest | 29.7 MB |
| Finest scale only (20 m to 7 km, 240 m beyond) | 0.0751° | IM-2 (7–15 km), SP-01 H = 2 (0.0609° at 29 km), Shackleton lowest | 29.7 MB |
| 20 m to 15.16 km on 2352² tiles (same pick area), finest scale | 0.0609° | SP-01 H = 2: obstacle at 29 km, 240 m tier | 42.4 MB (+12.7) |
| 80 m to 15.16 km, finest scale | 0.0609° | same bin | 31.1 MB [est] |
| **80 m to 31 km, finest scale (chosen)** | **0.0382°** | none | **33.6 MB (+3.9)** |
| 80 m to 55 km or 100 km | 0.0382° | none (Shackleton lowest 0.0168°) | 40–59 MB [est] |

The option runs used a composite sampler that read the 80 m tile at 20 m steps; the chosen layout was then re-run with 80 m steps (the table above). A 20 m tile to 31 km would be 3 850² px (29.6 MB per region), so the 80 m tier wins on payload and the 20 m widening fails at 15 km. The finest scale is what brings SP-01's 60 m bins from 0.0669° to 0.0382°: at 0.125 m a 60 m obstacle carries up to 0.06° of quantisation.

#### Before #48 (Nov 14)

Shipped run: int16 20 m tile out to 7 km, then the 240 m far tile. Reference: the float LDEM_80S_20MPP window (site ± 101 km, NaN treated as no data) out to 100 120 m, then the same 240 m far tile. 100 120 m = 7000 + 240 × 388 m, so beyond that distance both runs take identical far samples. The 0.05° rule is applied only to bins whose obstacle is at least 60 m away in both runs: at 20–40 m, int16 quantisation dominates at H = 0 (P11, P13).

| Point (map km) | H | Max \|Δel\|, dist ≥ 60 m | where (shipped / ref) | RMS ≥ 60 m | bins > 0.05° | Max \|Δel\| by ref distance: <60 m · 60 m–7 km · 7–15 km · 15–100 km · >100 km |
|---|---|---|---|---|---|---|
| IM-2 (77.11, 137.99) | 0 | **0.0749°** | az 324.5°: −1.2521° @ 9 160 m / −1.1772° @ 9 300 m | 0.0199° | 7 | 0.1019 · 0.0047 · 0.0749 · 0.0429 · – |
| IM-2 | 2 | **0.0751°** | az 324.5°: −1.2645° @ 9 160 m / −1.1894° @ 9 300 m | 0.0176° | 7 | – · 0.0047 · 0.0751 · 0.0429 · – |
| SP-01 (−11.31, −12.26) | 0 | **0.0663°** | az 36.5°: 0.7630° @ 60 m / 0.8294° @ 60 m | 0.0234° | 27 | 0.0396 · 0.0663 · – · 0.0126 · – |
| SP-01 | 2 | **0.0669°** | az 218.5°: −1.7189° @ 60 m / −1.6521° @ 60 m | 0.0209° | 37 | 0.0416 · 0.0669 · – · 0.0609 · – |
| Mons Mouton highest (85.25, 129.85), h_s 5 807.8 m | 0 | 0.0382° | az 334.0°: −1.2629° @ 16 120 m / −1.2247° @ 16 260 m | 0.0141° | 0 | 0.0393 · 0.0068 · 0.0320 · 0.0382 · – |
| Mons Mouton highest | 2 | 0.0397° | az 195.0°: 2.0417° @ 12 280 m / 2.0814° @ 12 200 m | 0.0109° | 0 | – · 0.0075 · 0.0397 · 0.0383 · – |
| Mons Mouton lowest (80.23, 145.71), h_s 4 832.3 m | 0 | 0.0083° | az 323.0°: 9.7432° @ 140 m / 9.7349° @ 140 m | 0.0023° | 0 | – · 0.0083 · – · – · – |
| Mons Mouton lowest | 2 | 0.0068° | az 283.0°: 8.1257° @ 240 m / 8.1189° @ 240 m | 0.0020° | 0 | – · 0.0068 · – · – · – |
| Shackleton lowest (−19.15, −20.15), h_s −576.0 m | 0 | **0.0570°** | az 177.5°: 15.8319° @ 7 240 m / 15.8889° @ 7 280 m | 0.0089° | 2 | – · 0.0268 · 0.0570 · 0.0293 · 0.0043 |
| Shackleton lowest | 2 | **0.0570°** | az 177.5°: 15.8173° @ 7 240 m / 15.8744° @ 7 280 m | 0.0086° | 2 | – · 0.0017 · 0.0570 · 0.0293 · 0.0043 |

Without the 60 m filter, the largest difference over all bins is 0.1019°, at IM-2, H = 0, az 226.5°: the first pixel at 20 m, int16 against float (P11). The ground heights agree to within 0.03 m in every row: shipped against float h_s is 5321.919/5321.906, 1955.034/1955.010, 5807.821/5807.813, 4832.321/4832.324 and −575.976/−575.982 m.

**Result: T-1a-4 misses the 0.05° rule. The worst bin with dist ≥ 60 m is 0.0751° (IM-2, H = 2 m).** Four of the five points have bins above 0.05°, and the misses come from two separate causes:
1. **The 240 m tier from 7 to 15 km.** At IM-2 and Shackleton-lowest, the worst bins are ridge crests 7.2–9.3 km away, which the 240 m far tile smooths. The cheapest remedy test is a 1536² int16 20 m tile centred on the site with a 15 km near range, measured against the same float reference. At IM-2 it lowers the worst bin to 0.0440° (H = 0) and 0.0440° (H = 2), with 0 bins above 0.05°; the 7–15 km band drops from 0.075° to 0.0003°.
2. **The near field at 60 m (SP-01).** The worst bins are obstacles at exactly 60 m, where int16 (0.125 m steps on the Shackleton tile) against float is still visible. A wider tile does not fix this. With the remedy, SP-01 is 0.0278° at H = 0 (0 bins above) but 0.0531° at H = 2 (4 bins above, all at 60 m). Every 20–60 m obstacle here is also flagged by P13.

T07 did not change the tile widths; #48 fixed both causes (80 m middle tier to 31 km, finest per-tile scale; table above).

**Picker margin.** The highest pixel inside the T06 pickable area (836² px, 7 km border) of Mons Mouton is (85.45, 129.65) km, on its last column, and the horizon there returns `OUT_OF_COVERAGE`. Map distance is ground distance × (1 + r²/4R²), about 0.2 % more at Mons Mouton, so a 7 000 m march needs about 7 014 m of map, while only 6 990 m (between pixel centres) is covered. The error is correct (P5), but the UI picker needs a border of at least about 7.05 km. The table above uses a 360 px (7.2 km) border to pick the extreme pixels.

### T-1a-5: SP-01, H = 10 m, march to the tile edge against a 150 km cap

- 46 of 720 bins differ.
- Largest difference: 0.4396° at az 106.5°. Marching to the tile edge gives −2.1894° (obstacle 195 880 m away); the 150 km cap gives −2.6290° (91 720 m).
- Farthest obstacle when marching to the tile edge: 198 280 m, at az 108.0°.
- Change in the longest night: pending T08 (#9).

So a cap at 150 km would bias the low-horizon azimuths at SP-01 downward by up to 0.44°. The march stays uncapped (§1a), so the far tile has to reach site + 200 km.

## AVGVISIB rasters (T20, #21)

**Header facts** (`v2_avgvisib::avgvisib_headers_and_spot_counts`, local-only, `--ignored`), both `AVGVISIB_85S_060M_201608.TIF` and `_EARTH`: 5058 × 5058 px at 60 m, tiepoint upper-left (−151 740, +151 740) m, pixel-is-area, sphere 1 737 400 m, `GDAL_NODATA = -32768` (none in the sampled area). int16 counts; points = count × 0.004 (scale 0.00004, W3c). BigTIFF, Deflate, predictor 2, 256² tiles: `tiff` 0.11.3 decodes it as `DecodingResult::I16`, and the spot counts match (Sun (2340, 2732) = 21 605, (2340, 2733) = 0, (2529, 2529) = 6 880, max 22 111 at (2349, 2724) = 88.44 %; Earth (2340, 2732) = 14 485, (2529, 2529) = 7 872). The grids nest: AVGVISIB pixel edges sit at −151 740 + 60k m, so our 20 m tiles average 3 × 3 onto it exactly (Shackleton from col 1, row 0 → 511 × 512 cells; Mons Mouton from col 0, row 0 → 512 × 512).

**Orientation:** the same map frame as ours (x = r·sin λ, y = r·cos λ, row 0 = largest y). Pearson r of our visibility against the reference read at (x, y) is 0.955; at (−x, y) 0.19, (x, −y) −0.17, (y, x) 0.04.

**Reference speckle:** the Sun raster has isolated exact zeros inside lit terrain. Of 271 sampled zeros, 10 have all eight neighbours ≥ 20 %. These give large positive residuals in the 0–20 % bin, by construction.

**Sample shortfall:** our two near tiles (Shackleton ridge and Mons Mouton) hold only 118 candidate pixels in 60–80 % and 5 in 80–100 % (147 696 candidates ≥ 7.2 km inside the tiles: 35 274 / 84 811 / 27 488 / 118 / 5 per fifth). With the agreed rule (quota 2 000, a short bin takes all), the Sun sample is 6 123 pixels, not 10 000, and it is a two-region sample. The bright tail is undersampled; follow-up [#75](https://github.com/qicai14/penumbra/issues/75).

**Run** (Nov 15, `PENUMBRA_DATA=… cargo run --release -p penumbra-pipeline -- v2 web/public/data`, 524 s on 8 threads, three-tier horizon (#48; 546 s before), 14 545 pixels at H = 0 plus 405 at H = 2 m). Visibility = share of hourly UTC steps from 2010-01-01T00Z (163 048 steps) with any part of the disc up (P1); H = 0; e = ours − ref, points. Seed 0x50454e554d425241.

| Sun, per fifth | n | median \|e\| | p90 \|e\| | mean e | r |
|---|---|---|---|---|---|
| 0–20 | 2000 | 0.43 | 6.76 | +1.95 | 0.62 |
| 20–40 | 2000 | 0.47 | 3.39 | +0.28 | 0.89 |
| 40–60 | 2000 | 0.48 | 2.83 | +0.15 | 0.83 |
| 60–80 | 118 | 3.12 | 9.46 | −2.74 | 0.70 |
| 80–100 | 5 | 1.86 | 9.17 | −1.95 | 0.63 |
| **pooled** | **6123** | **0.48** | **3.98** | **+0.72** | **0.955** |

**Verdict: V2 passes** (median ≤ 5, |mean| ≤ 2, r ≥ 0.9), and the stretch target is met too (median ≤ 2, p90 ≤ 8). The pass rests on the 0–60 % bins: the two bright bins are small and biased dark (ours below theirs). The pooled r is lifted by the spread between bins: inside every bin r is below 0.9 (0.62, 0.89, 0.83, 0.70, 0.63), and the 60–80 % bin's mean error (−2.74) would miss the |mean| ≤ 2 rule if bins were judged alone. Per-bin values are reported, not judged. The bright-tail shortfall is #75.

- **Earth twin** (any fraction of the disc, P14; own 10 000-pixel draw, 2 000 per fifth; report only): median 0.18, p90 3.16, mean +0.39, r 0.990.
- **P2** (405-pixel subsample, 100 per Sun fifth [U: our choice]): H = 0 median 0.74, mean −0.23, r 0.981; H = 2 m median 1.65, mean +3.76, r 0.978. Raising the observer 2 m adds 3.98 points on average, so H = 0 is the height that matches.
- **U2:** starting 4.65 y later (40 762 h) changes visibility by mean 0.028, p90 0.058, max 0.18 points. Under 0.5: the undocumented start date does not matter.
- **U3:** lat ≤ −88° (in our tiles this is the Shackleton tile; Mons Mouton is all north of −88°): median 0.59, p90 5.95, mean +1.59, r 0.935 (n 2973); rest median 0.42, p90 2.41, mean −0.09, r 0.986 (n 3150). The region split and the latitude split are the same split here.
- **N5** (critical obstacle < 180 m, among bins whose horizon lies in the band the Sun's disc crosses): median 0.73, p90 4.50, mean +0.64 (n 4126); rest median 0.27, p90 2.03, mean +0.89 (n 1997).
- **P7:** fitted shift e ≈ a + δ·∇ref: pooled δ = (1.1, 0.7) m, patch δ = (4.1, 1.9) m, both well under the 15 m flag [U: our choice, a quarter of a 60 m pixel]. The 64 × 64 patch at (−11.0, −12.0) km (the Shackleton tile centre, not SP-01 at (−11.31, −12.26)) has median 1.15, p90 8.12, mean +2.40, r 0.71. The map below (threshold ±10 points [U: our choice]) has 330 '+' (ours brighter, mostly single cells, consistent with reference pits) and 17 '−'. About ten of the '−' form one diagonal run from about (row 26, col 39) to (39, 26), about 1 km long, each next to a '+'. That run is the only paired bright and dark feature; reading it is visual, while the shift fit (δ ≈ 4 m) is the measurement that rules out a registration shift.

P7 residual map, 64 × 64 cells of 60 m centred at (−11.0, −12.0) km; row 0 = north (largest y), col 0 = west; '+' e > 10, '-' e < −10, '.' else:

```
......+...........+.......++......................++............
..+......+......................................................
.+++...+......+.............++.+................................
..+....+......++...............+..................+....+....+...
+.............+..........+................+.................+...
.........+......................................................
..+.......................+.....................................
.........+...........+........................++............+...
...........................................+........+.......+...
................................................................
.................................+.+............................
.......................................................+....+...
.....+..................+.....+..................+.....+....+...
....................+.........+.........++......................
..........................................................+.....
..........................................+..+.................+
.....................................++......................+..
...........+......................++......+........+.........++.
.....................+................+.........................
..............................................+...+.+...........
....................+.....+............................+..+.....
.........................+......................................
...................+++..................++......................
...................++++...+.....................................
...+................++........+...-................+............
....................++........+..+...+...+......................
..+....................................--.-.....................
............++..+.............+......-..+.........+..+..........
............++...................-......+...++..+.....+...+.....
.............+.........+..+...+..-............+.+.....++........
.......................++.+.......+.......+......-..............
..........+..+.................-..+............+...+.....+......
....+.....+.....+.+...+....+...................+................
..............+...+........-...............+...+.++.............
......................++.....-.............+..........+.........
...+......................+..-..................................
...........+...............+.............+......................
...........................+...........+..........+.............
.++.....................-.+.....+...............................
.....................+++..-....................+................
.+...++.+.........+.......+...++................+....+.....+....
.....+..........................................+..........+....
...+.....+............+++......++.......+................+......
......+............++.++................+++....+...........+....
.....+.............+..+........+......+..++............+.....+..
.+..+..++++....+...+.........+.........++...............+....+..
++.++..++..+.......++..............................++...........
+......+.+..+......+...................................+........
+....+.............++.+....................+........+...........
.......+....+......+++...............+..................+.......
..................+........++.................+....+............
+......++..........+.......+......++.....+......................
+............................+.....................+.......+....
...+.....++...........+.......++...........................+....
.....+........++...+......+..........+..........+...............
...................++..+........................................
...................+..+++..-....................+........+......
.......................+..................................+.....
......................++.....+........++............+..+........
........+........+...........+....+....+........................
.+..++...........+.....................+........................
...+..+.........-++........+..+.......+.........................
......+....................+....................+...............
...+..+..........++.........-+.....+.........+..............+...
```

Known gaps (stated, not fixed): AVGVISIB is not the Mazarico 2011 run (its max 88.44 % is below SP-01's 92.66 % (E16e), so it is not cross-checked against E16); beyond 7 km our field is 80 m to 31 km and 240 m beyond, against their 60 m (not measured; the nearest number we have is T-1a-4's up to 0.038° (0.075° before #48) for 240 m against a 20 m reference); 163 048 steps against Barker's 165 000 (E93). Measured cost is about 0.28 core-seconds per pixel (524 s × 8 threads ÷ 14 950 evaluations), about 20× the spec's 15 ms/pixel line (U18).

## Validator re-run on the three-tier horizon (#48, Nov 15)

All four checks re-run on 909a833 with the shipped near, middle and far tiles; no threshold or tolerance changed, no pass/fail flipped. Runs are seeded and deterministic, so these are exact differences, not noise. V2 is above (524 s against 546 s). Before → after:

| Check | Metric | Before | After | Verdict |
|---|---|---|---|---|
| V2 | pooled median \|e\| / mean e / p90 / r (n 6123) | 0.46 / +0.75 / 3.98 / 0.955 | 0.48 / +0.72 / 3.98 / 0.955 | pass (≤ 5), unchanged |
| V2b | Barker RoIs passing (of 7) | 7 | 7 | pass |
| V2b | SP-01 total night, pixel min..max (ref 2.75 d) | 2.96..16.75 d | 2.96..16.71 d (min 2.958) | fail, unchanged (0.21 d below the pixel range) |
| V4 | IM-2 depth to hide, H = 0 / 1 / 1.44 / 2 m; pit 1.944 m | 0.48 / 1.48 / 1.92 / 2.48 m | 0.48 / 1.48 / 1.92 / 2.48 m (margin at 1.44 m 2.65 cm) | consistent, unchanged |
| V5 case 3 (PSR) | 20 sites × 163 048 steps, lit steps; closest Sun limb to horizon | 0; −0.378° | 0; −0.378° (−0.3777°, same pixel 17398, 10380) | pass |

V2b ran beside V2, so its 97 s is not a clean timing. The V4 note used to say the Sun was "0.81 deg above the regional horizon"; 0.81° was the horizon itself (main at 7a257e8 prints 0.813°). The note now reads 1.63° above a 0.84° horizon (three-tier run).
