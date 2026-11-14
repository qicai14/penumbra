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
| `near-mons-mouton` | IM-2 site (77.1, 138.0) | 1536² at 20 m | (61 740, 153 360) | 0.0625 m | 2 431 m |
| `near-shackleton-ridge` | (−11.0, −12.0), between SP-01 and the Site 1 RoIs | 1536² at 20 m | (−26 360, 3 360) | 0.125 m | 4 361 m |
| `far-240m` | the pole | 3184² at 240 m (±382.08 km) | (−382 080, 382 080) | 0.5 m | |

- Tiles snap to the source pixel grid, so no resampling. The offset is the middle of the tile's height range. No tile has a no-data pixel.
- Round trip, 1 000 seeded pixels per tile, raw `read_chunk` against the core sampler at pixel centres: worst errors 0.031 m, 0.0625 m and 0.25 m, at most half a step each.
- Inside the inner 836² px (7 km border) of each near tile: 18 627 (Mons Mouton) and 21 508 (Shackleton ridge) LPSR = 1 pixels.
- Native build of all three tiles: about 1 s in release mode (file on HDD, Windows desktop).
