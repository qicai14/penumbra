//! DEM tiles and the bilinear height sampler (ARCHITECTURE §2.1, SCIENCE-SPEC §1a).
//!
//! A tile is row-major int16 counts on the south polar stereographic map grid, row 0 = largest y.
//! Height above the 1737.4 km reference sphere [U6] is `offset_m + scale_m × count`.

use crate::geometry::MapXy;

/// Why a height could not be given. Maps to the API's error codes; never replaced by 0 m (P5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemError {
    /// The point is not inside the tile's bilinear footprint (between its outer pixel centres).
    OutOfCoverage,
    /// One of the four pixels around the point holds the no-data count.
    NoData,
    /// A non-finite point or a malformed tile.
    BadInput(&'static str),
}

impl DemError {
    pub fn code(&self) -> &'static str {
        match self {
            DemError::OutOfCoverage => "OUT_OF_COVERAGE",
            DemError::NoData => "NO_DATA",
            DemError::BadInput(_) => "BAD_INPUT",
        }
    }
}

/// The eight numbers the manifest carries for a tile (ARCHITECTURE §2.1). Map metres, south polar
/// stereographic (`MapXy`); `(x0_m, y0_m)` is the upper-left corner of pixel (0, 0), pixel-is-area [U5].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TileMeta {
    pub width: u32,
    pub height: u32,
    pub x0_m: f64,
    pub y0_m: f64,
    /// Pixel size, map metres.
    pub pixel_m: f64,
    /// Metres per count.
    pub scale_m: f64,
    /// Height of count 0, metres above the reference sphere.
    pub offset_m: f64,
    /// Count that marks a pixel with no height.
    pub nodata: i16,
}

/// A loaded tile: metadata plus `width × height` counts.
#[derive(Debug, Clone)]
pub struct DemTile {
    meta: TileMeta,
    counts: Vec<i16>,
}

impl DemTile {
    pub fn new(meta: TileMeta, counts: Vec<i16>) -> Result<DemTile, DemError> {
        let n = u64::from(meta.width) * u64::from(meta.height);
        if meta.width < 2 || meta.height < 2 || u64::try_from(counts.len()) != Ok(n) {
            return Err(DemError::BadInput(
                "tile needs ≥ 2 × 2 pixels and width × height counts",
            ));
        }
        let finite = [
            meta.x0_m,
            meta.y0_m,
            meta.pixel_m,
            meta.scale_m,
            meta.offset_m,
        ];
        if !finite.iter().all(|v| v.is_finite()) || meta.pixel_m <= 0.0 || meta.scale_m <= 0.0 {
            return Err(DemError::BadInput(
                "tile geometry must be finite with positive pixel and scale",
            ));
        }
        Ok(DemTile { meta, counts })
    }

    pub fn meta(&self) -> &TileMeta {
        &self.meta
    }

    pub fn counts(&self) -> &[i16] {
        &self.counts
    }

    /// Height in metres above the reference sphere at map point `xy`, bilinear between the four
    /// surrounding pixel centres (counts interpolated, then scaled once).
    pub fn height_m(&self, xy: MapXy) -> Result<f64, DemError> {
        if !xy.x_m.is_finite() || !xy.y_m.is_finite() {
            return Err(DemError::BadInput("map x/y must be finite"));
        }
        let m = &self.meta;
        let fc = (xy.x_m - m.x0_m) / m.pixel_m - 0.5;
        let fr = (m.y0_m - xy.y_m) / m.pixel_m - 0.5;
        let (ic, ir) = (fc.floor(), fr.floor());
        if ic < 0.0 || ir < 0.0 || ic >= f64::from(m.width - 1) || ir >= f64::from(m.height - 1) {
            return Err(DemError::OutOfCoverage);
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // range checked above: 0 ≤ ic < width − 1 and 0 ≤ ir < height − 1, whole numbers
        let (i, j) = (ic as usize, ir as usize);
        let w = m.width as usize;
        let at = |k: usize| -> Result<f64, DemError> {
            let v = self.counts[k];
            if v == m.nodata {
                Err(DemError::NoData)
            } else {
                Ok(f64::from(v))
            }
        };
        let k = j * w + i;
        let (v00, v01, v10, v11) = (at(k)?, at(k + 1)?, at(k + w)?, at(k + w + 1)?);
        let (a, b) = (fc - ic, fr - ir);
        let top = v00 + a * (v01 - v00);
        let bottom = v10 + a * (v11 - v10);
        Ok(m.offset_m + m.scale_m * (top + b * (bottom - top)))
    }
}
