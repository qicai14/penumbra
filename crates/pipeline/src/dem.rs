//! DEM crop and int16 encode (T06; ARCHITECTURE §2.1, SCIENCE-SPEC §1a, P5, P11).

use crate::eph::Result;

/// Near-tile scales, finest first: the encoder takes the first that holds the tile (P11).
pub const NEAR_SCALES_M: [f64; 3] = [0.0625, 0.125, 0.25];
/// Far-tile scale: 0.25 m spans only ±8 191 m, which the far field may exceed (ARCHITECTURE §2.1).
pub const FAR_SCALES_M: [f64; 1] = [0.5];
/// Count written for a source NaN (the GeoTIFF no-data value).
pub const NODATA: i16 = i16::MIN;

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
    let (lo, hi) = heights_m
        .iter()
        .filter(|v| v.is_finite())
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
            (lo.min(f64::from(v)), hi.max(f64::from(v)))
        });
    if lo > hi {
        return Err("tile has no finite height".into());
    }
    let offset_m = (lo + hi) / 2.0;
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
                let c = ((f64::from(v) - offset_m) / scale_m).round() as i16;
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
        })
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
                let DecodingResult::F32(px) = self.decoder.read_chunk(index)? else {
                    return Err("expected Float32 samples".into());
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

/// Crop a `size_px` square from `src`, centred on map point `centre` and snapped to its pixel grid,
/// and encode it with the first of `scales_m` that fits. Errors if the square leaves the raster.
pub fn build_tile(
    src: &mut GeoTiff,
    centre: penumbra_core::geometry::MapXy,
    size_px: u32,
    scales_m: &[f64],
) -> Result<penumbra_core::dem::DemTile> {
    // U5/U6: a point-registered or non-sphere source would shift or bias every tile (P7).
    if !src.pixel_is_area || src.sphere_radius_m != Some(penumbra_core::geometry::MOON_RADIUS_M) {
        return Err("source DEM is not pixel-is-area on the 1737.4 km sphere".into());
    }
    let half = f64::from(size_px) / 2.0;
    let first = |offset_m: f64, limit: u32| -> Result<u32> {
        let v = (offset_m / src.pixel_m - half).round();
        if v < 0.0 || v + f64::from(size_px) > f64::from(limit) {
            return Err(format!("tile at {centre:?} leaves the {limit} px source").into());
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // whole, in [0, limit]
        let v = v as u32;
        Ok(v)
    };
    let col0 = first(centre.x_m - src.x0_m, src.width)?;
    let row0 = first(src.y0_m - centre.y_m, src.height)?;
    let heights = src.read_window(col0, row0, size_px, size_px)?;
    let e = encode(&heights, scales_m)?;
    let meta = penumbra_core::dem::TileMeta {
        width: size_px,
        height: size_px,
        x0_m: src.x0_m + f64::from(col0) * src.pixel_m,
        y0_m: src.y0_m - f64::from(row0) * src.pixel_m,
        pixel_m: src.pixel_m,
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
}

impl ShippedTiles {
    /// Load every tile listed in `data_dir/manifest.json`; urls are relative to `data_dir/..`.
    pub fn load(data_dir: &std::path::Path) -> Result<ShippedTiles> {
        let text = std::fs::read_to_string(data_dir.join("manifest.json"))?;
        let m: serde_json::Value = serde_json::from_str(&text)?;
        let root = data_dir.parent().ok_or("data dir has no parent")?;
        let far = load_entry(root, &m["far"])?;
        let regions = m["regions"]
            .as_array()
            .ok_or("manifest has no regions")?
            .iter()
            .map(|r| {
                let id = r["id"].as_str().ok_or("region without id")?.to_string();
                Ok((id, load_entry(root, &r["near"])?))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(ShippedTiles { far, regions })
    }

    pub fn near(&self, id: &str) -> Result<&penumbra_core::dem::DemTile> {
        self.regions
            .iter()
            .find(|(r, _)| r == id)
            .map(|(_, t)| t)
            .ok_or_else(|| format!("region {id} not in manifest").into())
    }
}

fn load_entry(
    root: &std::path::Path,
    e: &serde_json::Value,
) -> Result<penumbra_core::dem::DemTile> {
    let url = e["url"].as_str().ok_or("tile entry without url")?;
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
    /// Read the source pixels whose centres lie within `half_m` (map metres) of `centre` in x and y.
    pub fn read(
        src: &mut GeoTiff,
        centre: penumbra_core::geometry::MapXy,
        half_m: f64,
    ) -> Result<FloatWindow> {
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
