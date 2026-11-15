//! DEM crop and int16 encode (T06; ARCHITECTURE §2.1, SCIENCE-SPEC §1a, P5, P11).

use crate::eph::Result;

/// Floor of the near and middle tile scale, metres per count: a level tile still gets a positive
/// scale, and 1 mm is far below the DEM's own 0.30–0.50 m error (E23a). [D]
pub const MIN_SCALE_M: f64 = 0.001;
/// Coarsest near and middle tile scale, metres per count (SCIENCE-SPEC §1a tile encoding, P11).
pub const MAX_SCALE_M: f64 = 0.25;
/// Far-tile scale: 0.25 m spans only ±8 191 m, which the far field may exceed (ARCHITECTURE §2.1).
pub const FAR_SCALES_M: [f64; 1] = [0.5];
/// Count written for a source NaN (the GeoTIFF no-data value).
pub const NODATA: i16 = i16::MIN;
/// Middle tile side, 80 m pixels (4 × 4 means of the 20 m source), same centre as the near tile:
/// ±39.68 km covers the 8.31 km half-width of the pickable area plus the 31 km mid range at map
/// scale ≤ 1.0032 (#48, SCIENCE-SPEC §1a tiers).
pub const MID_PX: u32 = 992;
/// Source pixels per middle-tile pixel side: 4 × 20 m = 80 m.
pub const MID_BLOCK_PX: u32 = 4;

/// A tile's counts and the scale and offset that decode them: h = offset_m + scale_m × count.
#[derive(Debug, Clone)]
pub struct Encoded {
    pub counts: Vec<i16>,
    pub scale_m: f64,
    pub offset_m: f64,
}

/// Encode heights (metres, NaN = no data) with the first of `scales_m` whose ±32 767 counts
/// hold every height around the mid-range offset. Errors if every scale clips or nothing is finite.
pub fn encode(heights_m: &[f32], scales_m: &[f64]) -> Result<Encoded> {
    encode_snapped(heights_m, scales_m, None)
}

/// `encode`, with the offset moved by under half a count onto `anchor_m` + k·scale when given,
/// so a resized tile keeps an earlier tile's quantisation grid. Needs a single scale.
fn encode_snapped(heights_m: &[f32], scales_m: &[f64], anchor_m: Option<f64>) -> Result<Encoded> {
    let (lo, hi) = heights_m
        .iter()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(f64::from(v)), hi.max(f64::from(v)))
        });
    if lo > hi {
        return Err("tile has no finite height".into());
    }
    let mid_m = (lo + hi) / 2.0;
    // counts are rounded relative to `base_m`, then shifted by `k_counts`, so a snapped tile rounds
    // ties exactly as a tile whose offset was the anchor did (f64::round is half away from zero)
    let (offset_m, base_m, k_counts) = match (anchor_m, scales_m) {
        (None, _) => (mid_m, mid_m, 0.0),
        (Some(a), [s]) => {
            let k = ((mid_m - a) / s).round();
            (a + k * s, a, k)
        }
        (Some(_), _) => return Err("a snapped offset needs exactly one scale".into()),
    };
    let limit = f64::from(i16::MAX);
    let scale_m = *scales_m
        .iter()
        .find(|&&s| {
            ((hi - offset_m) / s).round() <= limit && ((lo - offset_m) / s).round() >= -limit
        })
        .ok_or_else(|| format!("heights {lo} .. {hi} m clip at every scale {scales_m:?}"))?;
    let counts = heights_m
        .iter()
        .map(|&v| {
            if v.is_finite() {
                #[allow(clippy::cast_possible_truncation)]
                // within ±32 767, checked by the scale choice
                let c = (((f64::from(v) - base_m) / scale_m).round() - k_counts) as i16;
                c
            } else {
                NODATA
            }
        })
        .collect();
    Ok(Encoded {
        counts,
        scale_m,
        offset_m,
    })
}

/// Encode near or middle tile heights at the finest scale whose ±32 767 counts hold the tile:
/// relief / 65 534, floored at `MIN_SCALE_M` (P11, #48). Errors if nothing is finite or the
/// relief needs a scale coarser than `MAX_SCALE_M` (16 383.5 m).
pub fn encode_finest(heights_m: &[f32]) -> Result<Encoded> {
    let (lo, hi) = heights_m
        .iter()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(f64::from(v)), hi.max(f64::from(v)))
        });
    if lo > hi {
        return Err("tile has no finite height".into());
    }
    // 1e-12 keeps the extreme counts at ±32 767 under f64 rounding
    let scale_m = ((hi - lo) / (2.0 * f64::from(i16::MAX)) * (1.0 + 1e-12)).max(MIN_SCALE_M);
    if scale_m > MAX_SCALE_M {
        return Err(format!("relief {lo} .. {hi} m needs a scale above {MAX_SCALE_M} m").into());
    }
    encode(heights_m, &[scale_m])
}

/// Offset of the ±382 km far tile shipped before #26. The ±396 km tile snaps its offset onto this
/// grid (anchor + k × 0.5 m) and rounds its counts relative to the anchor, so a height the old tile
/// held decodes to the same value, rounding ties included (before that tie fix, 1 pixel of the
/// 10 137 856 shared ones differed, measured Nov 15).
pub const FAR_OFFSET_ANCHOR_M: f64 = -345.260_742_187_5;

/// The far tile's encoder: the first of `FAR_SCALES_M` that holds it, offset on the
/// `FAR_OFFSET_ANCHOR_M` grid.
pub fn encode_far(heights_m: &[f32]) -> Result<Encoded> {
    encode_snapped(heights_m, &FAR_SCALES_M, Some(FAR_OFFSET_ANCHOR_M))
}

/// One Float32 GeoTIFF band, read window by window (never the whole raster, N6), with the
/// GeoTIFF facts T06 must record before any crop (U5, U6).
#[derive(Debug)]
pub struct GeoTiff {
    decoder: tiff::decoder::Decoder<std::io::BufReader<std::fs::File>>,
    pub width: u32,
    pub height: u32,
    /// Upper-left corner of pixel (0, 0), map metres (ModelTiepointTag).
    pub x0_m: f64,
    pub y0_m: f64,
    /// Pixel size, map metres (ModelPixelScaleTag; x and y must agree).
    pub pixel_m: f64,
    /// GTRasterTypeGeoKey (1025) = 1.
    pub pixel_is_area: bool,
    /// GDAL_NODATA tag text.
    pub nodata: Option<String>,
    /// GeogSemiMajorAxisGeoKey (2057), metres: the datum the heights are measured from (U6).
    pub sphere_radius_m: Option<f64>,
    /// GeoAsciiParamsTag text (datum and projection names).
    pub citation: Option<String>,
    /// SampleFormat (339) = 3: IEEE float samples, heights in metres. False for int16 counts
    /// (AVGVISIB), which `read_window` returns raw and the height readers refuse.
    pub float_samples: bool,
}

impl GeoTiff {
    pub fn open(path: &std::path::Path) -> Result<GeoTiff> {
        use tiff::tags::Tag;
        let file = std::fs::File::open(path)?;
        let mut d = tiff::decoder::Decoder::new(std::io::BufReader::new(file))?;
        let (width, height) = d.dimensions()?;
        let tie = d.get_tag(Tag::ModelTiepointTag)?.into_f64_vec()?;
        let scale = d.get_tag(Tag::ModelPixelScaleTag)?.into_f64_vec()?;
        let ([0.0, 0.0, _, x0_m, y0_m, ..], [sx, sy, ..]) = (tie.as_slice(), scale.as_slice())
        else {
            return Err(format!("tiepoint {tie:?} is not at pixel (0, 0)").into());
        };
        if (sx - sy).abs() > 1e-9 {
            return Err(format!("pixels are not square: {sx} × {sy} m").into());
        }
        let keys = d.get_tag(Tag::GeoKeyDirectoryTag)?.into_u16_vec()?;
        let doubles = match d.find_tag(Tag::GeoDoubleParamsTag)? {
            Some(v) => v.into_f64_vec()?,
            None => Vec::new(),
        };
        // header (4 shorts), then (key, location, count, value) per key
        let entries = keys.get(4..).ok_or("GeoKeyDirectory has no header")?;
        let key = |id: u16| {
            entries
                .as_chunks::<4>()
                .0
                .iter()
                .find(|k| k[0] == id)
                .map(|k| (k[1], k[3]))
        };
        let pixel_is_area = key(1025) == Some((0, 1));
        let sphere_radius_m = match key(2057) {
            Some((34736, i)) => doubles.get(usize::from(i)).copied(),
            _ => None,
        };
        let text = |d: &mut tiff::decoder::Decoder<_>, tag| -> Result<Option<String>> {
            Ok(match d.find_tag(tag)? {
                Some(v) => Some(v.into_string()?.trim_end_matches(['\0', '|']).to_string()),
                None => None,
            })
        };
        // absent SampleFormat means unsigned integer (TIFF 6.0)
        let float_samples = match d.find_tag(Tag::SampleFormat)? {
            Some(v) => v.into_u16_vec()? == [3],
            None => false,
        };
        let nodata = text(&mut d, Tag::GdalNodata)?;
        let citation = text(&mut d, Tag::GeoAsciiParamsTag)?;
        Ok(GeoTiff {
            decoder: d,
            width,
            height,
            x0_m: *x0_m,
            y0_m: *y0_m,
            pixel_m: *sx,
            pixel_is_area,
            nodata,
            sphere_radius_m,
            citation,
            float_samples,
        })
    }

    /// Errors unless the samples are floats: int16 counts are not heights in metres (#77).
    fn require_heights(&self) -> Result<()> {
        if self.float_samples {
            Ok(())
        } else {
            Err("source samples are integer counts, not Float32 heights in metres".into())
        }
    }

    /// Pixels `col0..col0 + w` × `row0..row0 + h`, row-major, decoding only the tiles they touch.
    pub fn read_window(&mut self, col0: u32, row0: u32, w: u32, h: u32) -> Result<Vec<f32>> {
        use tiff::decoder::DecodingResult;
        if col0 + w > self.width || row0 + h > self.height || w == 0 || h == 0 {
            return Err(format!("window {col0}+{w} × {row0}+{h} is outside the raster").into());
        }
        let (tw, th) = self.decoder.chunk_dimensions();
        let across = self.width.div_ceil(tw);
        let mut out = vec![f32::NAN; w as usize * h as usize];
        for trow in row0 / th..=(row0 + h - 1) / th {
            for tcol in col0 / tw..=(col0 + w - 1) / tw {
                let index = trow * across + tcol;
                // Float32 DEMs, or int16 counts (AVGVISIB, T20) returned as their raw count values
                let px: Vec<f32> = match self.decoder.read_chunk(index)? {
                    DecodingResult::F32(px) => px,
                    DecodingResult::I16(px) => px.into_iter().map(f32::from).collect(),
                    _ => return Err("expected Float32 or Int16 samples".into()),
                };
                // edge tiles are not padded: the row stride is the tile's data width (DATA-NOTES)
                let (dw, dh) = self.decoder.chunk_data_dimensions(index);
                let (c_lo, c_hi) = ((tcol * tw).max(col0), (tcol * tw + dw).min(col0 + w));
                let (r_lo, r_hi) = ((trow * th).max(row0), (trow * th + dh).min(row0 + h));
                for row in r_lo..r_hi {
                    let src = ((row - trow * th) * dw + (c_lo - tcol * tw)) as usize;
                    let dst = ((row - row0) * w + (c_lo - col0)) as usize;
                    let n = (c_hi - c_lo) as usize;
                    out[dst..dst + n].copy_from_slice(&px[src..src + n]);
                }
            }
        }
        Ok(out)
    }
}

/// Crop a `size_px` square of `block_px` × `block_px` source-pixel means from `src`, centred on map
/// point `centre` and snapped to the block grid (source pixel edges at multiples of `block_px`), and
/// encode it with `encoder`. `block_px` = 1 keeps the source pixels; a block holding a NaN is NaN
/// (no data). Errors if the square leaves the raster or the source is not Float32 heights.
pub fn build_tile(
    src: &mut GeoTiff,
    centre: penumbra_core::geometry::MapXy,
    size_px: u32,
    block_px: u32,
    encoder: fn(&[f32]) -> Result<Encoded>,
) -> Result<penumbra_core::dem::DemTile> {
    // U5/U6: a point-registered or non-sphere source would shift or bias every tile (P7).
    if !src.pixel_is_area || src.sphere_radius_m != Some(penumbra_core::geometry::MOON_RADIUS_M) {
        return Err("source DEM is not pixel-is-area on the 1737.4 km sphere".into());
    }
    src.require_heights()?;
    if block_px == 0 {
        return Err("block must be at least one source pixel".into());
    }
    let span = size_px
        .checked_mul(block_px)
        .ok_or("tile span overflows u32")?;
    let block_m = src.pixel_m * f64::from(block_px);
    let half = f64::from(size_px) / 2.0;
    let first = |offset_m: f64, limit: u32| -> Result<u32> {
        let v = (offset_m / block_m - half).round() * f64::from(block_px);
        if v < 0.0 || v + f64::from(span) > f64::from(limit) {
            return Err(format!("tile at {centre:?} leaves the {limit} px source").into());
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // whole, in [0, limit]
        let v = v as u32;
        Ok(v)
    };
    let col0 = first(centre.x_m - src.x0_m, src.width)?;
    let row0 = first(src.y0_m - centre.y_m, src.height)?;
    let raw = src.read_window(col0, row0, span, span)?;
    let heights = if block_px == 1 {
        raw
    } else {
        let (n, b, w) = (size_px as usize, block_px as usize, span as usize);
        let cells = f64::from(block_px * block_px);
        (0..n * n)
            .map(|k| {
                let (r, c) = (k / n, k % n);
                let sum: f64 = (0..b * b)
                    .map(|q| f64::from(raw[(r * b + q / b) * w + c * b + q % b]))
                    .sum();
                // NaN in, NaN out; a mean of f32 heights is within f32 range
                #[allow(clippy::cast_possible_truncation)]
                let mean = (sum / cells) as f32;
                mean
            })
            .collect()
    };
    let e = encoder(&heights)?;
    let meta = penumbra_core::dem::TileMeta {
        width: size_px,
        height: size_px,
        x0_m: src.x0_m + f64::from(col0) * src.pixel_m,
        y0_m: src.y0_m - f64::from(row0) * src.pixel_m,
        pixel_m: block_m,
        scale_m: e.scale_m,
        offset_m: e.offset_m,
        nodata: NODATA,
    };
    penumbra_core::dem::DemTile::new(meta, e.counts).map_err(|e| format!("{e:?}").into())
}

/// Little-endian int16 bytes of a tile's counts, no header (ARCHITECTURE §3.5).
pub fn to_le_bytes(tile: &penumbra_core::dem::DemTile) -> Vec<u8> {
    tile.counts().iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// Manifest entry for a tile at `url` (the eight numbers of ARCHITECTURE §2.1; no h_max).
pub fn tile_json(url: &str, m: &penumbra_core::dem::TileMeta) -> serde_json::Value {
    serde_json::json!({
        "url": url, "width": m.width, "height": m.height, "x0_m": m.x0_m, "y0_m": m.y0_m,
        "pixel_m": m.pixel_m, "scale_m": m.scale_m, "offset_m": m.offset_m, "nodata": m.nodata,
    })
}

/// The tiles the browser ships, read back through `manifest.json` (ARCHITECTURE §3.5).
#[derive(Debug)]
pub struct ShippedTiles {
    pub far: penumbra_core::dem::DemTile,
    /// (region id, near tile), in manifest order.
    pub regions: Vec<(String, penumbra_core::dem::DemTile)>,
    /// (region id, middle 80 m tile), in manifest order (#48).
    pub mids: Vec<(String, penumbra_core::dem::DemTile)>,
}

impl ShippedTiles {
    /// Load every tile listed in `data_dir/manifest.json`; urls are relative to `data_dir/..`.
    pub fn load(data_dir: &std::path::Path) -> Result<ShippedTiles> {
        let text = std::fs::read_to_string(data_dir.join("manifest.json"))?;
        let m: serde_json::Value = serde_json::from_str(&text)?;
        let root = data_dir.parent().ok_or("data dir has no parent")?;
        let far = load_entry(root, &m["far"])?;
        let list = m["regions"].as_array().ok_or("manifest has no regions")?;
        let tiles = |key: &str| {
            list.iter()
                .map(|r| {
                    let id = r["id"].as_str().ok_or("region without id")?.to_string();
                    Ok((id, load_entry(root, &r[key])?))
                })
                .collect::<Result<Vec<_>>>()
        };
        Ok(ShippedTiles {
            far,
            regions: tiles("near")?,
            mids: tiles("mid")?,
        })
    }

    pub fn near(&self, id: &str) -> Result<&penumbra_core::dem::DemTile> {
        find(&self.regions, id)
    }

    pub fn mid(&self, id: &str) -> Result<&penumbra_core::dem::DemTile> {
        find(&self.mids, id)
    }

    /// The shipped horizon (what the browser computes): region `id`'s near and middle tiles, then
    /// the far tile. Site in MOON_ME lat / east lon, `mast_m` above the near tile's ground.
    pub fn horizon(
        &self,
        id: &str,
        site: penumbra_core::geometry::LatLon,
        mast_m: f64,
        opts: &penumbra_core::horizon::HorizonOptions,
    ) -> Result<penumbra_core::horizon::Horizon> {
        let (near, mid) = (self.near(id)?, self.mid(id)?);
        penumbra_core::horizon::horizon(near, mid, &self.far, site, mast_m, opts)
            .map_err(|e| format!("{id}: {e:?}").into())
    }
}

fn find<'a>(
    tiles: &'a [(String, penumbra_core::dem::DemTile)],
    id: &str,
) -> Result<&'a penumbra_core::dem::DemTile> {
    tiles
        .iter()
        .find(|(r, _)| r == id)
        .map(|(_, t)| t)
        .ok_or_else(|| format!("region {id} not in manifest").into())
}

fn load_entry(
    root: &std::path::Path,
    e: &serde_json::Value,
) -> Result<penumbra_core::dem::DemTile> {
    let url = e["url"]
        .as_str()
        .ok_or("tile entry missing or without url")?;
    let num = |k: &str| e[k].as_f64().ok_or_else(|| format!("{url}: {k} missing"));
    let int = |k: &str| e[k].as_i64().ok_or_else(|| format!("{url}: {k} missing"));
    let meta = penumbra_core::dem::TileMeta {
        width: u32::try_from(int("width")?)?,
        height: u32::try_from(int("height")?)?,
        x0_m: num("x0_m")?,
        y0_m: num("y0_m")?,
        pixel_m: num("pixel_m")?,
        scale_m: num("scale_m")?,
        offset_m: num("offset_m")?,
        nodata: i16::try_from(int("nodata")?)?,
    };
    let bytes = std::fs::read(root.join(url))?;
    let (words, rest) = bytes.as_chunks::<2>();
    if !rest.is_empty() {
        return Err(format!("{url}: odd byte count").into());
    }
    let counts = words.iter().map(|w| i16::from_le_bytes(*w)).collect();
    penumbra_core::dem::DemTile::new(meta, counts).map_err(|e| format!("{url}: {e:?}").into())
}

/// A float window of the source DEM, sampled like a tile but without int16 quantisation: the
/// T-1a-4 reference. Same bilinear rule and pixel-is-area convention as `DemTile`; NaN → `NoData`.
#[derive(Debug)]
pub struct FloatWindow {
    width: u32,
    height: u32,
    /// Upper-left corner of pixel (0, 0), map metres.
    x0_m: f64,
    y0_m: f64,
    pixel_m: f64,
    heights_m: Vec<f32>,
    max_m: f64,
}

impl FloatWindow {
    /// A window over `heights_m` (row-major, `width × height`, metres above the 1 737 400 m sphere,
    /// NaN = no data); `(x0_m, y0_m)` is the upper-left corner of pixel (0, 0), map metres.
    pub fn from_heights(
        x0_m: f64,
        y0_m: f64,
        pixel_m: f64,
        width: u32,
        height: u32,
        heights_m: Vec<f32>,
    ) -> Result<FloatWindow> {
        if width < 2 || height < 2 || heights_m.len() != width as usize * height as usize {
            return Err("window needs ≥ 2 × 2 pixels and width × height heights".into());
        }
        if !(x0_m.is_finite() && y0_m.is_finite() && pixel_m.is_finite() && pixel_m > 0.0) {
            return Err("window geometry must be finite with a positive pixel".into());
        }
        let max_m = heights_m
            .iter()
            .filter(|v| v.is_finite())
            .fold(f64::NEG_INFINITY, |m, &v| m.max(f64::from(v)));
        if !max_m.is_finite() {
            return Err("window has no finite height".into());
        }
        Ok(FloatWindow {
            width,
            height,
            x0_m,
            y0_m,
            pixel_m,
            heights_m,
            max_m,
        })
    }

    /// (width, height, x0_m, y0_m, pixel_m): pixels, and the upper-left corner and pixel size in map metres.
    pub fn grid(&self) -> (u32, u32, f64, f64, f64) {
        (self.width, self.height, self.x0_m, self.y0_m, self.pixel_m)
    }

    /// Read the source pixels whose centres lie within `half_m` (map metres) of `centre` in x and y.
    /// Errors if the source is not Float32 heights.
    pub fn read(
        src: &mut GeoTiff,
        centre: penumbra_core::geometry::MapXy,
        half_m: f64,
    ) -> Result<FloatWindow> {
        src.require_heights()?;
        let px = src.pixel_m;
        let lo_c = ((centre.x_m - half_m - src.x0_m) / px).floor();
        let lo_r = ((src.y0_m - centre.y_m - half_m) / px).floor();
        let n = (2.0 * half_m / px).ceil() + 2.0;
        let fits = |lo: f64, limit: u32| lo >= 0.0 && lo + n <= f64::from(limit);
        if !fits(lo_c, src.width) || !fits(lo_r, src.height) {
            return Err(format!("window ±{half_m} m at {centre:?} leaves the source").into());
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // whole numbers inside the raster, checked above
        let (col0, row0, size) = (lo_c as u32, lo_r as u32, n as u32);
        let heights_m = src.read_window(col0, row0, size, size)?;
        let max_m = heights_m
            .iter()
            .filter(|v| v.is_finite())
            .fold(f64::NEG_INFINITY, |m, &v| m.max(f64::from(v)));
        if !max_m.is_finite() {
            return Err("window has no finite height".into());
        }
        Ok(FloatWindow {
            width: size,
            height: size,
            x0_m: src.x0_m + f64::from(col0) * px,
            y0_m: src.y0_m - f64::from(row0) * px,
            pixel_m: px,
            heights_m,
            max_m,
        })
    }
}

impl penumbra_core::dem::Terrain for FloatWindow {
    fn pixel_m(&self) -> f64 {
        self.pixel_m
    }
    fn max_height_m(&self) -> f64 {
        self.max_m
    }
    fn height_m(
        &self,
        xy: penumbra_core::geometry::MapXy,
    ) -> std::result::Result<f64, penumbra_core::dem::DemError> {
        use penumbra_core::dem::DemError;
        if !xy.x_m.is_finite() || !xy.y_m.is_finite() {
            return Err(DemError::BadInput("map x/y must be finite"));
        }
        let fc = (xy.x_m - self.x0_m) / self.pixel_m - 0.5;
        let fr = (self.y0_m - xy.y_m) / self.pixel_m - 0.5;
        let (ic, ir) = (fc.floor(), fr.floor());
        if ic < 0.0
            || ir < 0.0
            || ic >= f64::from(self.width - 1)
            || ir >= f64::from(self.height - 1)
        {
            return Err(DemError::OutOfCoverage);
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // range checked above: 0 ≤ ic < width − 1 and 0 ≤ ir < height − 1, whole numbers
        let (i, j) = (ic as usize, ir as usize);
        let w = self.width as usize;
        let at = |k: usize| {
            let v = self.heights_m[k];
            if v.is_finite() {
                Ok(f64::from(v))
            } else {
                Err(DemError::NoData)
            }
        };
        let k = j * w + i;
        let (v00, v01, v10, v11) = (at(k)?, at(k + 1)?, at(k + w)?, at(k + w + 1)?);
        let (a, b) = (fc - ic, fr - ir);
        let top = v00 + a * (v01 - v00);
        let bottom = v10 + a * (v11 - v10);
        Ok(top + b * (bottom - top))
    }
}
