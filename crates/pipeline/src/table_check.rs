//! T19 / V2b longest-night table check (SCIENCE-SPEC §3 V2b, U20, P12; brief docs/briefs/T19-table-check.md).
//! Barker 2021 Table 2 (Site 1, case A, Δz = 5 m) and Mazarico 2011 SP-01 (H = 10 m), against our
//! per-pixel hourly disc fraction f from the shipped tiles. Native and offline.

use crate::dem::ShippedTiles;
use crate::eph::{Result, build_table, load_almanac};
use penumbra_core::dem::DemTile;
use penumbra_core::eph::{Ephemeris, Site};
use penumbra_core::geometry::{MapXy, xy_to_latlon};
use penumbra_core::horizon::{HorizonOptions, ground_height_m, horizon};
use penumbra_core::power::longest_run;
use penumbra_core::timeline::{DEFAULT_EARTH_MARGIN_DEG, timeline};
use std::path::Path;

/// LDEM_80S_20MPP grid: pixel centres sit at −303 990 + 20k m
/// on both map axes (polar stereographic, MOON_ME), half a pixel inside the edge (P7).
const GRID_ORIGIN_M: f64 = -303_990.0;
const GRID_PX_M: f64 = 20.0;

/// One Barker 2021 Table 2 Site 1 region of interest.
#[derive(Debug, Clone, Copy)]
pub struct Roi {
    /// Centroid, polar stereographic map km (b5251ef3:555–569; E129, E130).
    pub x_km: f64,
    pub y_km: f64,
    pub area_m2: f64,
    /// Case A, published, days (b5251ef3:682–688): LCIP, 1st percentile over the RoI.
    pub lcip_1_d: f64,
    /// Case A, published, days (b5251ef3:707–713): LCSP, 99th percentile over the RoI.
    pub lcsp_99_d: f64,
}

impl Roi {
    pub fn centre(&self) -> MapXy {
        MapXy {
            x_m: self.x_km * 1000.0,
            y_m: self.y_km * 1000.0,
        }
    }
    /// Radius of the equal-area disc, √(Area/π), metres.
    pub fn radius_m(&self) -> f64 {
        (self.area_m2 / std::f64::consts::PI).sqrt()
    }
}

const fn roi(x_km: f64, y_km: f64, area_m2: f64, lcip_1_d: f64, lcsp_99_d: f64) -> Roi {
    Roi {
        x_km,
        y_km,
        area_m2,
        lcip_1_d,
        lcsp_99_d,
    }
}

/// Barker 2021 Table 2, Site 1 RoIs 1–7 (SCIENCE-SPEC §3 V2b).
pub const SITE1_ROIS: [Roi; 7] = [
    roi(-11.5825, -12.4625, 8625.0, 215.79, 5.29),
    roi(-11.4875, -12.5575, 7550.0, 216.28, 7.31),
    roi(-11.3525, -12.5025, 3825.0, 216.89, 8.41),
    roi(-11.2875, -12.1675, 13300.0, 231.20, 4.00),
    roi(-11.1025, -11.9425, 10075.0, 218.98, 4.52),
    roi(-10.8025, -11.7375, 10000.0, 220.98, 4.16),
    roi(-10.2275, -11.5525, 7075.0, 119.07, 5.42),
];
/// Barker pass margins, days (SCIENCE-SPEC §3 V2b): LCSP ≤ LCSP-99 + 1, LCIP ≥ LCIP-1 − 10.
const LCSP_MARGIN_D: f64 = 1.0;
const LCIP_MARGIN_D: f64 = 10.0;
/// RoIs of 7 that must pass.
pub const BARKER_PASS_MIN: usize = 6;
/// Barker observer height Δz, m above ground.
const BARKER_MAST_M: f64 = 5.0;
/// 2024-01-01T00:00:00Z, Unix s UTC; 2 years hourly, end exclusive (731 d × 24).
const BARKER_T0_UNIX_S: f64 = 1_704_067_200.0;
const BARKER_STEPS: usize = 17_544;

/// SP-01 centre, polar stereographic map km (SCIENCE-SPEC §3 V2b), and the pixel disc radius.
pub const SP01_XY_KM: (f64, f64) = (-11.31, -12.26);
pub const SP01_RADIUS_M: f64 = 150.0;
/// Mazarico observer height H, m above ground.
const SP01_MAST_M: f64 = 10.0;
/// One precession cycle from 2020-01-01T00:00:00Z, Unix s UTC, assumed (U20); hourly.
const SP01_T0_UNIX_S: f64 = 1_577_836_800.0;
const SP01_STEPS: usize = crate::psr::STEPS;
/// Mazarico 2011 SP-01 longest total night (f = 0) at 10 m, days (7b941912:2294–2296).
pub const SP01_TOTAL_NIGHT_D: f64 = 2.75;
/// Mazarico 2011 SP-01 longest partial night (f < 1) at 10 m, days (7b941912:2294–2296): report only.
const SP01_PARTIAL_NIGHT_D: f64 = 7.4;

const STEP_S: f64 = 3600.0;

/// 20 m pixel centres within `radius_m` of `centre` (map metres), d² ≤ r² inclusive.
pub fn pixels_in_disc(centre: MapXy, radius_m: f64) -> Vec<MapXy> {
    let k_range = |c: f64| {
        let lo = ((c - radius_m - GRID_ORIGIN_M) / GRID_PX_M).ceil();
        let hi = ((c + radius_m - GRID_ORIGIN_M) / GRID_PX_M).floor();
        #[allow(clippy::cast_possible_truncation)] // whole numbers of pixels, |k| < 2^31
        (lo as i64..=hi as i64)
    };
    #[allow(clippy::cast_precision_loss)] // |k| < 2^31, exact in f64
    let at = |k: i64| GRID_ORIGIN_M + GRID_PX_M * k as f64;
    let mut out = Vec::new();
    for ky in k_range(centre.y_m) {
        for kx in k_range(centre.x_m) {
            let p = MapXy {
                x_m: at(kx),
                y_m: at(ky),
            };
            let (dx, dy) = (p.x_m - centre.x_m, p.y_m - centre.y_m);
            if dx * dx + dy * dy <= radius_m * radius_m {
                out.push(p);
            }
        }
    }
    out
}

/// Longest run in a window, and whether it touches step 0 or step n−1 (then a lower bound).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Run {
    pub length_h: f64,
    pub touches_edge: bool,
}

/// The four longest runs of one pixel's disc-fraction series f (hourly or `step_h` hours).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PixelRuns {
    /// f > 0: any of the Sun visible (Barker LCIP, Mazarico day f > 0).
    pub lit_h: Run,
    /// f == 0: none of the Sun visible (Barker LCSP, Mazarico total night).
    pub dark_h: Run,
    /// f == 1: whole disc visible (Mazarico day f = 1).
    pub full_day_h: Run,
    /// f < 1: part of the disc hidden (Mazarico partial night).
    pub partial_night_h: Run,
}

fn run_of(f: &[f64], step_h: f64, pred: impl Fn(f64) -> bool + Copy) -> Run {
    let best = longest_run(f, pred);
    let lead = f.iter().take_while(|&&x| pred(x)).count();
    let trail = f.iter().rev().take_while(|&&x| pred(x)).count();
    #[allow(clippy::cast_precision_loss)] // step counts < 2^52
    Run {
        length_h: best as f64 * step_h,
        touches_edge: best > 0 && (lead == best || trail == best),
    }
}

/// Runs of `f` (disc fraction, 0–1; `disc_fraction` gives exactly 0.0 or 1.0 for a hidden or clear
/// disc, so the exact comparisons are intended).
#[allow(clippy::float_cmp)] // f is exactly 0.0 or 1.0 at the ends (timeline::disc_fraction)
pub fn pixel_runs(f: &[f64], step_h: f64) -> PixelRuns {
    PixelRuns {
        lit_h: run_of(f, step_h, |x| x > 0.0),
        dark_h: run_of(f, step_h, |x| x == 0.0),
        full_day_h: run_of(f, step_h, |x| x == 1.0),
        partial_night_h: run_of(f, step_h, |x| x < 1.0),
    }
}

/// Barker RoI value: arithmetic mean over pixels of each pixel's LCIP and LCSP, hours.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RoiMean {
    pub lcip_h: f64,
    pub lcsp_h: f64,
}

/// Mean of per-pixel runs (never runs on a mean series, b5251ef3:549, 1207–1209).
pub fn roi_mean(pixels: &[PixelRuns]) -> Result<RoiMean> {
    if pixels.is_empty() {
        return Err("RoI holds no pixels".into());
    }
    #[allow(clippy::cast_precision_loss)] // pixel counts are tiny
    let n = pixels.len() as f64;
    Ok(RoiMean {
        lcip_h: pixels.iter().map(|p| p.lit_h.length_h).sum::<f64>() / n,
        lcsp_h: pixels.iter().map(|p| p.dark_h.length_h).sum::<f64>() / n,
    })
}

/// Hourly disc fraction f at a pixel centre `xy` (map metres), observer `mast_m` above ground.
fn pixel_f(
    near: &DemTile,
    far: &DemTile,
    eph: &Ephemeris,
    xy: MapXy,
    mast_m: f64,
    t0_unix_s: f64,
    steps: usize,
) -> Result<Vec<f64>> {
    let err = |e: &dyn std::fmt::Debug| format!("{e:?}");
    let ll = xy_to_latlon(xy).map_err(|e| err(&e))?;
    let hor = horizon(near, far, ll, mast_m, &HorizonOptions::default()).map_err(|e| err(&e))?;
    let site = Site {
        lat_rad: ll.lat_rad,
        lon_east_rad: ll.lon_east_rad,
        ground_m: ground_height_m(near, ll).map_err(|e| err(&e))?,
        mast_m,
    };
    let tl = timeline(
        eph,
        &hor.el_rad,
        &site,
        t0_unix_s,
        STEP_S,
        u32::try_from(steps)?,
        DEFAULT_EARTH_MARGIN_DEG.to_radians(),
    )
    .map_err(|e| err(&e))?;
    Ok(tl.illumination)
}

fn min_max(xs: impl Iterator<Item = f64>) -> (f64, f64) {
    xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
        (lo.min(x), hi.max(x))
    })
}

const H_PER_D: f64 = 24.0;

/// Run V2b: tiles from `data_dir` (`web/public/data`), kernels from `kernels_dir` (PENUMBRA_DATA).
/// Returns the report as JSON for the scoreboard row.
pub fn run(data_dir: &Path, kernels_dir: &Path) -> Result<serde_json::Value> {
    let started = std::time::Instant::now();
    let tiles = ShippedTiles::load(data_dir)?;
    let near = tiles.near("shackleton-ridge")?;
    let almanac = load_almanac(kernels_dir)?;
    // One table from 2020 covers both windows; 2024-01-01 is a whole number of hours after it.
    let mut eph = Ephemeris::default();
    eph.add(
        SP01_T0_UNIX_S,
        STEP_S,
        build_table(&almanac, SP01_T0_UNIX_S, STEP_S, SP01_STEPS)?,
    )
    .map_err(|e| format!("{e:?}"))?;
    let step_h = STEP_S / 3600.0;

    let mut rois = Vec::new();
    let mut barker_passes = 0;
    for (i, r) in SITE1_ROIS.iter().enumerate() {
        let px = pixels_in_disc(r.centre(), r.radius_m())
            .into_iter()
            .map(|xy| {
                let f = pixel_f(
                    near,
                    &tiles.far,
                    &eph,
                    xy,
                    BARKER_MAST_M,
                    BARKER_T0_UNIX_S,
                    BARKER_STEPS,
                )?;
                Ok(pixel_runs(&f, step_h))
            })
            .collect::<Result<Vec<_>>>()?;
        let m = roi_mean(&px)?;
        let (lcsp_min_h, lcsp_max_h) = min_max(px.iter().map(|p| p.dark_h.length_h));
        let (lcip_d, lcsp_d) = (m.lcip_h / H_PER_D, m.lcsp_h / H_PER_D);
        let pass = lcsp_d <= r.lcsp_99_d + LCSP_MARGIN_D && lcip_d >= r.lcip_1_d - LCIP_MARGIN_D;
        barker_passes += usize::from(pass);
        rois.push(serde_json::json!({
            "roi": i + 1,
            "pixels": px.len(),
            "lcip_mean_d": lcip_d,
            "lcip_1_ref_d": r.lcip_1_d,
            "lcsp_mean_d": lcsp_d,
            "lcsp_99_ref_d": r.lcsp_99_d,
            "lcsp_pixel_min_d": lcsp_min_h / H_PER_D,
            "lcsp_pixel_max_d": lcsp_max_h / H_PER_D,
            "lcip_edge_pixels": px.iter().filter(|p| p.lit_h.touches_edge).count(),
            "lcsp_edge_pixels": px.iter().filter(|p| p.dark_h.touches_edge).count(),
            "pass": pass,
        }));
    }

    let c = MapXy {
        x_m: SP01_XY_KM.0 * 1000.0,
        y_m: SP01_XY_KM.1 * 1000.0,
    };
    let sp = pixels_in_disc(c, SP01_RADIUS_M)
        .into_iter()
        .map(|xy| {
            let f = pixel_f(
                near,
                &tiles.far,
                &eph,
                xy,
                SP01_MAST_M,
                SP01_T0_UNIX_S,
                SP01_STEPS,
            )?;
            Ok(pixel_runs(&f, step_h))
        })
        .collect::<Result<Vec<_>>>()?;
    let range_d = |get: fn(&PixelRuns) -> Run| {
        let (lo, hi) = min_max(sp.iter().map(|p| get(p).length_h));
        let edge = sp.iter().filter(|p| get(p).touches_edge).count();
        serde_json::json!({"min_d": lo / H_PER_D, "max_d": hi / H_PER_D, "edge_pixels": edge})
    };
    let (night_lo_h, night_hi_h) = min_max(sp.iter().map(|p| p.dark_h.length_h));
    let sp01_pass = (night_lo_h / H_PER_D..=night_hi_h / H_PER_D).contains(&SP01_TOTAL_NIGHT_D);

    Ok(serde_json::json!({
        "barker": {
            "mast_m": BARKER_MAST_M,
            "t0_unix_s": BARKER_T0_UNIX_S,
            "steps": BARKER_STEPS,
            "rois": rois,
            "passes": barker_passes,
            "pass": barker_passes >= BARKER_PASS_MIN,
        },
        "sp01": {
            "mast_m": SP01_MAST_M,
            "t0_unix_s": SP01_T0_UNIX_S,
            "steps": SP01_STEPS,
            "pixels": sp.len(),
            "total_night_f0": range_d(|p| p.dark_h),
            "total_night_ref_d": SP01_TOTAL_NIGHT_D,
            "day_f_gt_0": range_d(|p| p.lit_h),
            "day_f_eq_1": range_d(|p| p.full_day_h),
            "partial_night_f_lt_1": range_d(|p| p.partial_night_h),
            "partial_night_ref_d": SP01_PARTIAL_NIGHT_D,
            "pass": sp01_pass,
        },
        "pass": barker_passes >= BARKER_PASS_MIN && sp01_pass,
        "run_time_s": started.elapsed().as_secs_f64(),
    }))
}
