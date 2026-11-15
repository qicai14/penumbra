//! Region point grid (T24, #25): the browser's site chain (`penumbra_core::site`) at every 100 m
//! cell of a near tile's usable area, for a few landing dates. Ships derived hours only, never
//! horizons (SCIENCE-SPEC §4 compute notes).

use crate::eph::Result;
use penumbra_core::dem::{DemError, DemTile, TileMeta};
use penumbra_core::eph::Ephemeris;
use penumbra_core::geometry::{MapXy, xy_to_latlon};
use penumbra_core::power::{Lethal, Preset};
use penumbra_core::site::{site_horizon, site_survival};

/// Grid pixel, map metres (#25 Cut A).
pub const GRID_PIXEL_M: f64 = 100.0;
/// Cell outside the usable area, or no data under its horizon.
pub const NODATA_U16: u16 = 65535;
/// Lethal value when no lethal shadow falls in the span.
pub const NO_LETHAL_U16: u16 = 65534;

/// A grid of square cells in south polar stereographic map metres, row 0 at the top like the
/// tiles: `(x0_m, y0_m)` is the upper-left corner, cell (i, j) centre is
/// `(x0_m + (i + 0.5)·pixel_m, y0_m − (j + 0.5)·pixel_m)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GridSpec {
    pub x0_m: f64,
    pub y0_m: f64,
    pub width: u32,
    pub height: u32,
    pub pixel_m: f64,
}

impl GridSpec {
    /// Every cell whose centre lies at least `border_m` inside the tile `near`, the block of
    /// centres placed in the middle of that usable square. The grid passes the site picker's P5
    /// border, `penumbra_core::horizon::pick_border_m(near)`.
    pub fn usable(near: &TileMeta, pixel_m: f64, border_m: f64) -> Result<GridSpec> {
        let axis = |n_px: u32| -> Result<(u32, f64)> {
            let span_m = f64::from(n_px) * near.pixel_m - 2.0 * border_m;
            if span_m.is_nan() || span_m < 0.0 || pixel_m.is_nan() || pixel_m <= 0.0 {
                return Err("tile smaller than its usable border, or a bad pixel".into());
            }
            // 1e-9 keeps an exact multiple from flooring one short
            let gaps = (span_m / pixel_m + 1e-9).floor();
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            // whole, ≥ 0 and below 2^32 (a 1536 px tile gives 167)
            let n = gaps as u32 + 1;
            // map metres from the tile edge to the grid edge: border + (span − gaps·pixel)/2 −
            // pixel/2 with the border cancelled, so a non-round border still gives round corners
            let lead_m = (f64::from(n_px) * near.pixel_m - gaps * pixel_m) / 2.0 - pixel_m / 2.0;
            Ok((n, lead_m))
        };
        let (width, lead_x_m) = axis(near.width)?;
        let (height, lead_y_m) = axis(near.height)?;
        Ok(GridSpec {
            x0_m: near.x0_m + lead_x_m,
            y0_m: near.y0_m - lead_y_m,
            width,
            height,
            pixel_m,
        })
    }

    pub fn cells(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Centre of row-major cell `k` (row `k / width`, column `k % width`).
    pub fn centre(&self, k: usize) -> MapXy {
        let w = self.width as usize;
        #[allow(clippy::cast_precision_loss)] // cell indices are far below 2^52
        let (i, j) = ((k % w) as f64, (k / w) as f64);
        MapXy {
            x_m: self.x0_m + (i + 0.5) * self.pixel_m,
            y_m: self.y0_m - (j + 0.5) * self.pixel_m,
        }
    }
}

/// Survival numbers for one cell and landing date, hours after landing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellSummary {
    pub operable_h: f64,
    /// `None`: no lethal shadow in the span.
    pub lethal_h: Option<f64>,
    /// Σ illumination × Δt over the whole span (for the AVGVISIB comparison).
    pub sunlit_h: f64,
    pub contact_h: f64,
}

/// One cell's answer: the numbers, or the named state that its horizon has no data (a near or
/// middle gap or nodata pixel, `OUT_OF_COVERAGE` / `NO_DATA`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Done(CellSummary),
    NoData,
}

/// Everything a grid run holds fixed besides the tiles and dates.
#[derive(Debug, Clone, Copy)]
pub struct RunInputs {
    /// Observer height above the near tile's ground, metres.
    pub mast_m: f64,
    /// Timeline step, seconds (the browser's 600 s fixture step).
    pub step_s: f64,
    pub count: u32,
    /// Earth contact margin m_min, degrees (§1d).
    pub earth_margin_deg: f64,
    pub preset: Preset,
}

/// `[date][cell]` results: one horizon per cell, then one timeline and survival per landing
/// (`landings_unix_s`, Unix s UTC, each the timeline's first step). Cells run on every core.
pub fn run_grid(
    tiles: (&DemTile, &DemTile, &DemTile),
    eph: &Ephemeris,
    spec: &GridSpec,
    landings_unix_s: &[f64],
    inp: &RunInputs,
) -> Result<Vec<Vec<Cell>>> {
    let (near, mid, far) = tiles;
    let ks: Vec<usize> = (0..spec.cells()).collect();
    let per_cell = crate::v2::par_map(&ks, |&k| {
        let ll = xy_to_latlon(spec.centre(k)).map_err(|e| format!("cell {k}: {e:?}"))?;
        let sh = match site_horizon(
            near,
            mid,
            far,
            ll.lat_rad.to_degrees(),
            ll.lon_east_rad.to_degrees(),
            inp.mast_m,
        ) {
            Ok(sh) => sh,
            Err(DemError::OutOfCoverage | DemError::NoData) => {
                return Ok(vec![Cell::NoData; landings_unix_s.len()]);
            }
            Err(e) => return Err(format!("cell {k}: {e:?}")),
        };
        landings_unix_s
            .iter()
            .map(|&t0| {
                let s = site_survival(
                    eph,
                    &sh,
                    t0,
                    inp.step_s,
                    inp.count,
                    inp.earth_margin_deg,
                    &inp.preset,
                )
                .map_err(|e| format!("cell {k} landing {t0}: {e}"))?;
                Ok(Cell::Done(CellSummary {
                    operable_h: s.operable_h,
                    lethal_h: match s.lethal {
                        Lethal::At {
                            after_landing_h, ..
                        } => Some(after_landing_h),
                        Lethal::NoneInSpan => None,
                    },
                    sunlit_h: s.sunlit_h,
                    contact_h: s.contact_h,
                }))
            })
            .collect::<std::result::Result<Vec<_>, String>>()
    })?;
    Ok((0..landings_unix_s.len())
        .map(|d| per_cell.iter().map(|c| c[d]).collect())
        .collect())
}

/// Whole hours as u16; a value that does not fit below the two reserved codes is an error.
pub fn hours_u16(h: f64) -> Result<u16> {
    let r = h.round();
    if !(0.0..f64::from(NO_LETHAL_U16)).contains(&r) {
        return Err(format!("{h} h does not fit the grid's u16 hours").into());
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // whole, 0..65534, checked
    Ok(r as u16)
}

/// The shipped layout (#25): little-endian u16, per date in order `operable_h[cells]` then
/// `lethal_h[cells]`; `NODATA_U16` for a no-data cell, `NO_LETHAL_U16` for no lethal shadow.
pub fn encode(results: &[Vec<Cell>]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    for date in results {
        for c in date {
            let v = match c {
                Cell::Done(s) => hours_u16(s.operable_h)?,
                Cell::NoData => NODATA_U16,
            };
            out.extend(v.to_le_bytes());
        }
        for c in date {
            let v = match c {
                Cell::Done(CellSummary {
                    lethal_h: Some(h), ..
                }) => hours_u16(*h)?,
                Cell::Done(CellSummary { lethal_h: None, .. }) => NO_LETHAL_U16,
                Cell::NoData => NODATA_U16,
            };
            out.extend(v.to_le_bytes());
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use penumbra_core::horizon::pick_border_m;

    fn mons_mouton_near() -> TileMeta {
        // the shipped near tile's geometry (manifest.json, #48)
        TileMeta {
            width: 1536,
            height: 1536,
            x0_m: 61_740.0,
            y0_m: 153_360.0,
            pixel_m: 20.0,
            scale_m: 1.0,
            offset_m: 0.0,
            nodata: -32768,
        }
    }

    #[test]
    fn usable_grid_is_167_cells_whose_corner_centres_sit_inside_the_7_029_km_pick_border() {
        // Border 7 028.59 m (#26 S6): 30 720 − 2 × 7 028.59 = 16 662.82 m of centres; 167 at
        // 100 m span 16 600 m, centred, so the grid corner is 7 010 m in, as with the old 7 050 m
        // border and the shipped grid (manifest.json grids, #25).
        let near = mons_mouton_near();
        let g = GridSpec::usable(&near, GRID_PIXEL_M, pick_border_m(&near));
        assert_eq!(
            g.ok(),
            Some(GridSpec {
                x0_m: 68_750.0,
                y0_m: 146_350.0,
                width: 167,
                height: 167,
                pixel_m: 100.0
            })
        );
    }

    #[test]
    fn shackleton_ridge_grid_matches_the_shipped_grid() {
        // Border 7 010.84 m (#26 S6); the shipped grid (manifest.json, #25) starts at
        // (−19 350, −3 650) m with 167 × 167 cells.
        let near = TileMeta {
            x0_m: -26_360.0,
            y0_m: 3_360.0,
            ..mons_mouton_near()
        };
        let g = GridSpec::usable(&near, GRID_PIXEL_M, pick_border_m(&near));
        assert_eq!(
            g.ok(),
            Some(GridSpec {
                x0_m: -19_350.0,
                y0_m: -3_650.0,
                width: 167,
                height: 167,
                pixel_m: 100.0
            })
        );
    }

    #[test]
    fn centre_of_cell_k_is_row_major_with_row_0_at_the_top() {
        let g = GridSpec {
            x0_m: 1000.0,
            y0_m: 5000.0,
            width: 3,
            height: 2,
            pixel_m: 100.0,
        };
        // k = 4 is column 1 of row 1
        assert_eq!(
            g.centre(4),
            MapXy {
                x_m: 1150.0,
                y_m: 4850.0
            }
        );
    }

    #[test]
    fn encode_writes_operable_then_lethal_per_date_with_the_reserved_codes() {
        let done = |op: f64, le: Option<f64>| {
            Cell::Done(CellSummary {
                operable_h: op,
                lethal_h: le,
                sunlit_h: 0.0,
                contact_h: 0.0,
            })
        };
        let results = vec![
            vec![done(119.5, Some(130.4)), Cell::NoData],
            vec![done(720.0, None), done(0.2, Some(0.2))],
        ];
        let words: Vec<u16> = encode(&results)
            .map(|b| {
                b.chunks(2)
                    .map(|w| u16::from_le_bytes([w[0], w[1]]))
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(words, [120, 65535, 130, 65535, 720, 0, 65534, 0]);
    }

    #[test]
    fn hours_u16_refuses_a_negative_or_non_finite_value() {
        assert!(hours_u16(-1.0).is_err() && hours_u16(f64::NAN).is_err());
    }
}
