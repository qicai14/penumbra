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
