# T06 — DEM tiles for two regions and the far field

Ticket: [PLAN-DRAFT T06](../PLAN-DRAFT.md). Owner: Qi Cai. 1.5 h. Blocked by T01, T04 and the DEM download. The sampler and the encoder (§5 rows 1–13) need no data; write them while the download runs.

## 1. Goal and done-criteria (from the ticket)

The shipped terrain for Mons Mouton and the Shackleton ridge, in the §1 formats, with a sampler that fails loudly outside coverage.

- [ ] U5 and U6 are read from `LDEM_80S_20MPP_ADJ.TIF` and written down before any crop: tiepoint, pixel scale, pixel-is-area or point, no-data value, height reference.
- [ ] T-1a-2 (data part): SP-01 lands on a ridge crest, and all seven Barker Site 1 RoI centroids are higher than the mean height on a 200 m ring around them (N9).
- [ ] Two near tiles (20 m) and one far tile (240 m, ±382 km) are written. Near `scale` is the finest of 0.0625, 0.125 and 0.25 m that fits the tile (P11). The encoder fails if any height clips.
- [ ] The manifest has no `h_max` (the engine computes it at load). Data file names carry a content hash.
- [ ] Round trip: decoded heights are within half a `scale` step of the source at 1000 random pixels.
- [ ] The sampler returns an error for a point outside coverage or on no-data, never 0 m (P5).
- [ ] Tile centres are chosen so that the IM-2 site, SP-01, the seven RoIs and at least 20 PSR-floor pixels are each at least 7 km inside a tile edge.

## 2. Files and signatures

`crates/core/src/dem.rs` and `crates/core/tests/dem.rs` (new) · `crates/pipeline/src/` (GeoTIFF reader from T04, crop, encode) · `crates/wasm/src/lib.rs` (`load_near`, `load_far`, `ground_height`) · `web/public/data/near-<region>.<hash>.bin`, `far-240m.<hash>.bin`, `manifest.json` · `docs/DATA-NOTES.md`.

```rust
// core/src/dem.rs
pub trait Dem { fn height(&self, x: f64, y: f64) -> Result<f64, Error>; }   // map metres → metres above R

pub struct Tile {
    pub data: Vec<i16>,                 // row-major, row 0 = largest y
    pub width: usize, pub height: usize,
    pub x0_m: f64, pub y0_m: f64,       // upper-left CORNER of pixel (0, 0)
    pub pixel_m: f64, pub scale_m: f64, pub offset_m: f64, pub nodata: i16,
}
impl Tile {
    /// BadInput unless data.len() == width·height, width ≥ 2, height ≥ 2, pixel_m > 0, scale_m > 0.
    pub fn new(data: Vec<i16>, width: usize, height: usize, x0_m: f64, y0_m: f64,
               pixel_m: f64, scale_m: f64, offset_m: f64, nodata: i16) -> Result<Tile, Error>;
    /// Largest decoded height over valid pixels; None if every pixel is nodata. Used by load_far.
    pub fn max_height(&self) -> Option<f64>;
}
impl Dem for Tile { /* bilinear, §3 step 9 */ }

// pipeline (pure functions, unit-tested without data)
/// Finest candidate scale whose counts fit ±32767 for both ends; Err if none fits.
fn choose_scale(min_m: f64, max_m: f64, candidates: &[f64]) -> Result<(f64 /*scale*/, f64 /*offset*/), String>;
fn encode(h: f64, scale: f64, offset: f64) -> i16;          // round((h − offset)/scale)
fn crop_origin(centre_x: f64, centre_y: f64, pixel: f64, size_px: usize) -> (f64 /*x0*/, f64 /*y0*/);
```

WASM, frozen: `load_near` and `load_far` take `(data: Box<[i16]>, width: u32, height: u32, x0_m, y0_m, pixel_m, scale_m, offset_m, nodata: i16)`; `ground_height(lat_deg, lon_deg) -> Result<f64, JsError>` is `near.height(latlon_to_xy(lat, lon))`, `NOT_LOADED` without a near tile. `load_far` stores `max_height()`.

Files: `i16` little-endian, row-major, no header. Near 1536 × 1536 (4 718 592 B). Far 3184 × 3184 (20 275 712 B). Manifest keys as in ARCHITECTURE §3.5.

## 3. Algorithm

### 3a. Read the source (pipeline)

Library facts are from docs.rs for `tiff` 0.11.3, read Nov 11, not compiled.

1. Open with `Decoder::new(BufReader::new(File::open(path)?))`. Read with named tags: `Tag::ModelPixelScaleTag` (33550) and `Tag::ModelTiepointTag` (33922) through `get_tag_f64_vec`; `Tag::GeoKeyDirectoryTag` (34735) through `get_tag_u16_vec`; `Tag::GdalNodata` (42113) through `get_tag_ascii_string`; `Tag::Compression`, `Tag::Predictor`, `Tag::TileWidth`, `Tag::TileLength`. `find_tag` returns `Ok(None)` for a missing tag.
2. Write in `docs/DATA-NOTES.md` (U5, U6): image size, tile size, compression, predictor, sample format; the tiepoint (i, j, k, X, Y, Z) and the pixel scale; the raster type from the geokey directory (key 1025: 1 = pixel-is-area, 2 = pixel-is-point); the no-data string; and the product page's statement on the height reference (E133).
3. Upper-left corner of pixel (0, 0): for pixel-is-area it is the tiepoint (X, Y) when (i, j) = (0, 0). For pixel-is-point subtract half a pixel in x and add half a pixel in y. Expected for the 20 m product [U5]: 30 400 × 30 400 px, corner (−304 000, +304 000), pixel 20 m (E135).
4. Read only the chunks a crop needs: `chunk_dimensions()` gives the tile size, `read_chunk(index)` returns `DecodingResult::F32(Vec<f32>)` for float data. Chunk index = tile_row × tiles_across + tile_col [U: row-major order; check by reading the chunk that holds a known pixel]. Never decode the whole raster (N6): 30 400² × 4 B is 3.7 GB.

### 3b. Crop (pipeline)

5. Near tile, 1536 px at 20 m around a centre (cx, cy): `x0 = floor((cx − 15 360)/20)·20`, `y0 = ceil((cy + 15 360)/20)·20`. Source column of the tile's first pixel = (x0 − X_corner)/20, source row = (Y_corner − y0)/20. Both must be whole numbers; if not, stop (the grid isn't what U5 assumed). Never resample.
6. Centres:
   - **Mons Mouton**: the IM-2 site, (77 108, 137 993) m → x0 = 61 740, y0 = 153 360; source column 18 287, row 7 532.
   - **Shackleton ridge**: (−4 500, −10 000) m → x0 = −19 860, y0 = 5 360; source column 14 207, row 14 932. Pickable square (7 040 m from the tile edge, §4): x from −12 820 to +3 820, y from −18 320 to −1 680. It holds SP-01 (−11 308, −12 259) and the seven RoIs (x −11 583 to −10 228, y −12 558 to −11 553). [U: that it holds 20 PSR-floor pixels of Shackleton's interior; count them in `LPSR_80S_20MPP_ADJ.TIF` at least 5 px inside the flagged region. If fewer than 20, move the centre east by up to 1 000 m.]
7. Far tile: 3184 px at 240 m from `LDEM_60S_240MPP_ADJ.TIF` (E89), x0 = −382 080, y0 = +382 080. The same whole-number check with that file's own corner and pixel scale.

### 3c. Encode (pipeline; ARCHITECTURE §2.1)

8. Over the cropped heights find min and max (skip source no-data; count it and print the count). `offset = round((min + max)/2)` to a whole metre. Try scales in order: near 0.0625, 0.125, 0.25 m (P11); far 0.5 m only. A scale fits if `|round((min − offset)/scale)| ≤ 32 767` and the same for max. None fits: fail with the range printed. Counts: `round((h − offset)/scale)`; source no-data → −32 768. Write little-endian. File name: `<stem>.<first 6 hex of SHA-256>.bin`.

### 3d. Sample (core; ARCHITECTURE §2.1)

9. `height(x, y)`:
   ```
   fc = (x − x0_m)/pixel_m − 0.5     i = floor(fc)   a = fc − i
   fr = (y0_m − y)/pixel_m − 0.5     j = floor(fr)   b = fr − j
   if not (0 ≤ i < width − 1 and 0 ≤ j < height − 1): OutOfCoverage
   v00 = data[j·width + i], v10 = data[j·width + i + 1], v01 = data[(j+1)·width + i], v11 = data[(j+1)·width + i + 1]
   if any equals nodata: NoData
   v = (1−b)·((1−a)·v00 + a·v10) + b·((1−a)·v01 + a·v11)
   return offset_m + scale_m·v
   ```
   Interpolate the counts, scale once. A non-finite x or y is `BadInput`.

### 3e. Check and ship

10. Round trip: 1000 seeded random pixels per tile, `|decoded − source| ≤ scale/2 + 1e-3`.
11. T-1a-2 data part: `height` at SP-01 and at each RoI centroid must exceed the mean of 16 samples on a 200 m ring around it (N9). Centroids in km (b5251ef3:555–561, 563–569): (−11.5825, −12.4625), (−11.4875, −12.5575), (−11.3525, −12.5025), (−11.2875, −12.1675), (−11.1025, −11.9425), (−10.8025, −11.7375), (−10.2275, −11.5525).
12. Write the manifest: `format_version`, the far entry and one region entry per near tile, with the eight tile numbers and the hashed URL. No `h_max`.

## 4. Edge cases and pitfalls

- **Floating-point predictor.** `tiff` 0.11.3 documents predictor 3 as "Not currently supported", and float GeoTIFFs written by GDAL often use it. T04 finds out. If `Predictor` is 3: convert once with `gdal_translate -co PREDICTOR=1` (or `-co COMPRESS=NONE`) on the machine named for the fallback, and say so in the issue. Don't write a predictor decoder.
- **`Limits`.** The default decoding buffer is 256 MiB, fine for single chunks. If opening fails on the tile-offset arrays, raise `ifd_value_size` through `Decoder::with_limits` [U].
- **P7, half-pixel.** x0, y0 are the corner, and pixel centres are at +0.5. A pixel-is-point file read as area shifts everything 10 m: step 3.
- **P5.** `OutOfCoverage` and `NoData` are errors all the way to the UI. No default height.
- **The valid region is one half-pixel smaller than the tile on every side**, and the last pixel centre itself is outside (rows 4–5). T11 must keep the pin 7 040 m, not 7 000 m, from the tile edge, so the 7 km near march stays inside. Tell Charls.
- **P11.** Don't ship the near field at 0.5 m: one count between neighbouring 20 m pixels is a 1.4° slope.
- **Scale and offset come from the manifest only.** The decoder has no constants.
- `.gitignore` is already anchored to `/data/` and `*.bin` is marked binary (ARCHITECTURE §6.2 rows 1–2). Check `git status` shows the three `.bin` files before committing.
- Cloudflare's per-file limit against the 20.3 MB far tile is unverified (ARCHITECTURE §5 risk 5). If the deploy rejects it, split the far tile into two files of 1592 rows and concatenate in the adapter.

## 5. Test table

Tile T: 3 × 3, x0 = 1000, y0 = 2000, pixel 20, scale 0.25, offset 100, nodata −32 768, counts by row [0, 4, 8], [12, 16, 20], [24, 28, 32]. Pixel centres are at x = 1010, 1030, 1050 and y = 1990, 1970, 1950.

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | T.height(1030, 1970) (a pixel centre) | 104.0 | 1e-12 | [S] F1: count 16 |
| 2 | T.height(1020, 1975) | 102.75 | 1e-12 | [S] F1: a = 0.5, b = 0.75, count 11 |
| 3 | Tile 2 × 2, counts [0, 10], [20, 50], same metadata; height(1015, 1980) | 103.75 | 1e-12 | [S] F1: a = 0.25, b = 0.5, count 15 |
| 4 | T.height(1005, 1970); T.height(999, 1970); T.height(1030, 2001) | `OutOfCoverage` × 3 | exact | step 9: the outer half-pixel is outside |
| 5 | T.height(1050, 1970) (the last pixel centre) | `OutOfCoverage` | exact | step 9: i = width − 1 |
| 6 | T with count [1][1] = −32 768; height(1020, 1975); height(NaN, 0) | `NoData`; `BadInput` | exact | P5 |
| 7 | `Tile::new` with 8 counts for 3 × 3; with pixel_m = 0 | `BadInput` × 2 | exact | §2 |
| 8 | T.max_height(); T with [2][2] = −32 768 | 108.0; 107.0 | 1e-12 | 100 + 0.25 × 32; × 28 |
| 9 | `choose_scale(−2000.3, 1500.7, [0.0625, 0.125, 0.25])`; `encode` of both ends | (0.0625, −250); +28 011 and −28 005 | exact | [S] F4 |
| 10 | `choose_scale(−1000, 4000, near)`; `choose_scale(−4000, 5000, near)` | (0.125, 1500), max count 20 000; (0.25, 500), max count 18 000 | exact | [S] F3: spans 4 095.875, 8 191.75, 16 383.5 m |
| 11 | `choose_scale(−8500, 8500, near)`; `choose_scale(−8500, 8500, [0.5])` | Err; (0.5, 0), counts ±17 000 | exact | [S] F3: 0.5 m spans 32 767 m |
| 12 | `crop_origin(77 108, 137 993, 20, 1536)`; `crop_origin(−4500, −10 000, 20, 1536)` | (61 740, 153 360); (−19 860, 5 360) | exact | [S] F2 |
| 13 | Encode then decode 1000 random heights in [−3000, 3000] at scale 0.125, offset 0 | error ≤ 0.0625 | 1e-9 | round-trip rule |
| 14 | Real DEM: tags | 30 400 × 30 400, corner (−304 000, 304 000), 20 m, pixel-is-area | — | [U] U5, U6 |
| 15 | Real DEM: source column and row of the two near tiles | 18 287, 7 532 and 14 207, 14 932 | exact | [S] F2 if row 14 holds; else recompute |
| 16 | Real tiles: step 11 | SP-01 and 7 of 7 RoIs above their ring mean | — | [U] T-1a-2, N9 |
| 17 | Real tiles: `ground_height` at SP-01 and at the IM-2 site | finite; SP-01 near +1.9 km (review F2) | — | [U] |
| 18 | Real tiles: `ground_height(−60°, 0°)` | `OUT_OF_COVERAGE` | exact | P5 |

## 6. Red/green order for /tdd

1. Rows 1–3 red, then step 9. Rows 4–8.
2. Rows 9–13 red, then step 8 and `crop_origin`.
3. With the DEM on disk: steps 1–3, write DATA-NOTES, row 14. Stop here if row 14 differs and recompute step 6 before cropping.
4. Steps 4–8 for Mons Mouton, then step 10. Then the Shackleton ridge, then the far tile.
5. Rows 16–18, the manifest, the WASM loaders and `ground_height`. Deploy and load the page once to see all three files arrive.

**Stop and escalate if** row 16 fails for most RoIs (the orientation or the half-pixel is wrong: recheck T01 row 2 and step 3; don't move the centroids); or the far tile clips at 0.5 m (print the range and ask before changing the scale, because ARCHITECTURE §2.1 fixes it); or the predictor is 3 and no machine has GDAL (ask Qi Cai at once; T06 must not wait more than 30 minutes).
