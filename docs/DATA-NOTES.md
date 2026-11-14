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
