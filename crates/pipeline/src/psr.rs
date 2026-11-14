//! V5 case 3 (SCIENCE-SPEC §3, U15, E132, E139): sites well inside the LOLA PSR mask must see no
//! sunlight at any hourly step of 18.6 years. Native and offline: the vectors are never shipped.

use crate::dem::{FAR_SCALES_M, GeoTiff, NEAR_SCALES_M, build_tile};
use crate::eph::{Result, build_table, load_almanac};
use penumbra_core::eph::{Ephemeris, R_SUN_KM, Site};
use penumbra_core::geometry::{MapXy, xy_to_latlon};
use penumbra_core::horizon::{HorizonOptions, ground_height_m, horizon};
use penumbra_core::timeline::{DEFAULT_EARTH_MARGIN_DEG, horizon_el_at_rad, timeline};
use std::path::Path;

/// Half-width of the all-flagged square around a chosen pixel: 25 px = 500 m at 20 m (U15: no edge pixels).
pub const HALF_PX: u32 = 25;
/// Candidate grid step, pixels.
const STRIDE_PX: u32 = 25;
/// Scan blocks are `BLOCK_PX` square; `BLOCKS` of them across, centred on the pole (about ±143 km).
const BLOCK_PX: u32 = 1024;
const BLOCKS: u32 = 14;
/// Candidates farther from the pole than this are dropped: the 240 m far tile must reach site + 200 km.
const MAX_POLE_DISTANCE_M: f64 = 140_000.0;
/// Near tile side, pixels (as `penumbra-pipeline dem`), and far tile side at 240 m.
const NEAR_PX: u32 = 1536;
const FAR_PX: u32 = 3184;
/// Sites checked.
pub const SITES: usize = 20;
/// Span: 18.6 years hourly from 2010-01-01T00:00:00Z (§3 V2 details): ceil(18.6 × 365.25 × 24) rows.
/// The start is arbitrary (V2's start reused); V5 only asks for 18.6 years.
pub const START_UNIX_S: f64 = 1_262_304_000.0;
pub const STEPS: usize = 163_048;
const STEP_S: f64 = 3600.0;

/// Pixels `(col, row)`, on a `stride` grid starting at `half`, whose whole (2·half+1)² square lies
/// on the raster and holds only 1.0 (NaN counts as not flagged). Row-major order.
pub fn interior_points(mask: &[f32], w: u32, h: u32, half: u32, stride: u32) -> Vec<(u32, u32)> {
    let (wu, side) = (w as usize, 2 * half + 1);
    let mut out = Vec::new();
    if w < side || h < side {
        return out;
    }
    for r in (half..h - half).step_by(stride as usize) {
        for c in (half..w - half).step_by(stride as usize) {
            let all_flagged = (r - half..=r + half).all(|rr| {
                let row = rr as usize * wu;
                mask[row + (c - half) as usize..=row + (c + half) as usize]
                    .iter()
                    .all(|&v| (v - 1.0).abs() < 1e-6)
            });
            if all_flagged {
                out.push((c, r));
            }
        }
    }
    out
}

/// `n` entries at evenly spaced positions of `all`, in order (all of them when there are fewer).
pub fn pick_spread<T: Copy>(all: &[T], n: usize) -> Vec<T> {
    if all.len() <= n {
        return all.to_vec();
    }
    (0..n).map(|i| all[i * all.len() / n]).collect()
}

/// One checked site.
#[derive(Debug, Clone)]
pub struct PsrSite {
    pub col: u32,
    pub row: u32,
    pub lat_deg: f64,
    pub lon_east_deg: f64,
    pub ground_m: f64,
    /// Hourly steps with any part of the Sun's disc above the horizon; the V5 pass needs 0.
    pub lit_steps: usize,
    pub max_illumination: f64,
    /// Closest approach over all steps: max of (Sun's upper limb elevation − horizon elevation at the
    /// Sun's azimuth), degrees. Negative means the whole disc stayed below the horizon.
    pub min_gap_deg: f64,
}

#[derive(Debug, Clone)]
pub struct PsrReport {
    pub start_unix_s: f64,
    pub steps: usize,
    pub sites: Vec<PsrSite>,
}

/// Run V5 case 3 with the kernels and the three GeoTIFFs in `dir` (the `PENUMBRA_DATA` layout).
/// Observer height 0 m, as V2 declares (P2).
pub fn run(dir: &Path) -> Result<PsrReport> {
    let err = |e: &dyn std::fmt::Debug| format!("{e:?}");
    let mut mask_src = GeoTiff::open(&dir.join("LPSR_80S_20MPP_ADJ.TIF"))?;
    let mut near_src = GeoTiff::open(&dir.join("LDEM_80S_20MPP_ADJ.TIF"))?;
    let grid = |g: &GeoTiff| (g.width, g.height, g.pixel_m, g.x0_m, g.y0_m);
    if grid(&mask_src) != grid(&near_src) {
        return Err("LPSR and LDEM_80S are not on the same grid".into());
    }
    let px = mask_src.pixel_m;
    let (pole_col, pole_row) = (-mask_src.x0_m / px, mask_src.y0_m / px);
    let first = |pole: f64| -> Result<u32> {
        let v = (pole - f64::from(BLOCKS * BLOCK_PX) / 2.0).round();
        if v < 0.0 {
            return Err("scan area leaves the raster".into());
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // whole and ≥ 0, checked
        Ok(v as u32)
    };
    let (c_first, r_first) = (first(pole_col)?, first(pole_row)?);

    // Scan blocks around the pole for pixels with a 500 m all-flagged square.
    let mut cands = Vec::new();
    for br in 0..BLOCKS {
        for bc in 0..BLOCKS {
            let (c0, r0) = (c_first + bc * BLOCK_PX, r_first + br * BLOCK_PX);
            let m = mask_src.read_window(c0, r0, BLOCK_PX, BLOCK_PX)?;
            for (c, r) in interior_points(&m, BLOCK_PX, BLOCK_PX, HALF_PX, STRIDE_PX) {
                let xy = MapXy {
                    x_m: mask_src.x0_m + (f64::from(c0 + c) + 0.5) * px,
                    y_m: mask_src.y0_m - (f64::from(r0 + r) + 0.5) * px,
                };
                if xy.x_m.hypot(xy.y_m) <= MAX_POLE_DISTANCE_M {
                    cands.push((c0 + c, r0 + r, xy));
                }
            }
        }
    }
    println!(
        "{} candidate pixels with a 500 m all-flagged square",
        cands.len()
    );
    let chosen = pick_spread(&cands, SITES);

    let mut far_src = GeoTiff::open(&dir.join("LDEM_60S_240MPP_ADJ.TIF"))?;
    let far = build_tile(
        &mut far_src,
        MapXy { x_m: 0.0, y_m: 0.0 },
        FAR_PX,
        &FAR_SCALES_M,
    )?;
    let almanac = load_almanac(dir)?;
    let mut eph = Ephemeris::default();
    eph.add(
        START_UNIX_S,
        STEP_S,
        build_table(&almanac, START_UNIX_S, STEP_S, STEPS)?,
    )
    .map_err(|e| err(&e))?;

    let mut sites = Vec::new();
    for (col, row, xy) in chosen {
        let near = build_tile(&mut near_src, xy, NEAR_PX, &NEAR_SCALES_M)?;
        let ll = xy_to_latlon(xy).map_err(|e| err(&e))?;
        let hor = horizon(&near, &far, ll, 0.0, &HorizonOptions::default()).map_err(|e| err(&e))?;
        let ground_m = ground_height_m(&near, ll).map_err(|e| err(&e))?;
        let site = Site {
            lat_rad: ll.lat_rad,
            lon_east_rad: ll.lon_east_rad,
            ground_m,
            mast_m: 0.0,
        };
        #[allow(clippy::cast_possible_truncation)] // 163 048 fits u32
        let tl = timeline(
            &eph,
            &hor.el_rad,
            &site,
            START_UNIX_S,
            STEP_S,
            STEPS as u32,
            DEFAULT_EARTH_MARGIN_DEG.to_radians(),
        )
        .map_err(|e| err(&e))?;
        let mut gap = f64::NEG_INFINITY;
        for i in 0..STEPS {
            let rho_deg = (R_SUN_KM / tl.sun_dist_km[i]).asin().to_degrees();
            let hor_deg = horizon_el_at_rad(&hor.el_rad, tl.sun_az_deg[i].to_radians())
                .map_err(|e| err(&e))?
                .to_degrees();
            gap = gap.max(tl.sun_el_deg[i] + rho_deg - hor_deg);
        }
        sites.push(PsrSite {
            col,
            row,
            lat_deg: ll.lat_rad.to_degrees(),
            lon_east_deg: ll.lon_east_rad.to_degrees(),
            ground_m,
            lit_steps: tl.visibility.iter().filter(|&&v| v > 0.0).count(),
            max_illumination: tl.illumination.iter().copied().fold(0.0, f64::max),
            min_gap_deg: gap,
        });
    }
    Ok(PsrReport {
        start_unix_s: START_UNIX_S,
        steps: STEPS,
        sites,
    })
}
