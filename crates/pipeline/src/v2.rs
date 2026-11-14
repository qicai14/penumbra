//! T20 / V2 sample against PGDA `AVGVISIB_85S_060M_201608` and its `_EARTH` twin (SCIENCE-SPEC §3
//! V2 details; P1, P2, P7, P14, U2, U3, N5). Native and offline: the 18.6-year vectors are never shipped.
//!
//! Ours, per pixel: the share (points, 0–100) of hourly UTC steps from 2010-01-01T00:00:00Z with any
//! part of the disc above our terrain horizon (P1, W1; Earth "any fraction", P14), observer at the
//! ground (H = 0, P2), terrain = the shipped 20 m near tiles averaged 3 × 3 onto the AVGVISIB 60 m
//! lattice to 7 km, then the shipped 80 m middle tiles to 31 km and the shipped 240 m far tile (#48).
//! Sun and Earth come from one ANISE table (UTC grid, ANISE converts to TDB, P4); angles are
//! topocentric ENU, map coordinates south polar stereographic.

use crate::dem::{FloatWindow, GeoTiff, ShippedTiles};
use crate::eph::{Result, build_table, load_almanac};
use crate::psr::{START_UNIX_S, STEPS, pick_spread};
use penumbra_core::dem::{DemTile, Terrain};
use penumbra_core::eph::{Ephemeris, Site, sun_earth_track};
use penumbra_core::geometry::{AzEl, MapXy, xy_to_latlon};
use penumbra_core::horizon::{Horizon, HorizonOptions, N_AZ, ground_height_m, horizon};
use penumbra_core::timeline::disc_fraction;
use std::collections::BTreeMap;
use std::path::Path;

/// AVGVISIB grid (W3c, read from the file, DATA-NOTES): upper-left corner of pixel (0, 0), map metres.
pub const REF_X0_M: f64 = -151_740.0;
pub const REF_Y0_M: f64 = 151_740.0;
/// AVGVISIB pixel size, map metres, and side, pixels.
pub const REF_PX_M: f64 = 60.0;
pub const REF_SIDE_PX: u32 = 5058;
/// int16 count → points (percent of time): scale 0.00004 × 100 (W3c).
pub const POINTS_PER_COUNT: f64 = 0.004;
/// AVGVISIB no-data count (GDAL_NODATA "-32768"; none present, DATA-NOTES).
pub const REF_NODATA_COUNT: f32 = -32_768.0;
/// Our near tiles are 20 m; three of them make one AVGVISIB pixel (151.74 km ÷ 20 m = 7587, §3).
const NEAR_PX_M: f64 = 20.0;
const FACTOR: u32 = 3;
/// A candidate's centre is this many 60 m cells (7.2 km) inside its averaged tile, so the 7 km near
/// march never leaves it.
pub const MARGIN_CELLS: u32 = 120;
/// Sun and Earth draw: quota per bin of the reference value (§3 V2 details "Sampling").
pub const QUOTA: usize = 2000;
/// Seed of the order-independent draw (splitmix64 of seed ^ (row << 32 | col)).
pub const SEED: u64 = 0x5045_4e55_4d42_5241;
/// P2 / U2 subsample per Sun bin.
pub const SUB_PER_BIN: usize = 100;
/// P2 observer height for the subsample, metres above ground.
pub const P2_MAST_M: f64 = 2.0;
/// U2: start shift, hourly steps: 4.65 y × 365.25 d × 24 h = 40 761.9 → 40 762.
pub const SHIFT_STEPS: usize = 40_762;
/// U3 split latitude, degrees (MOON_ME).
pub const U3_LAT_DEG: f64 = -88.0;
/// N5: a pixel whose critical obstacle is closer than this, metres, is reported apart.
pub const N5_CRIT_M: f64 = 180.0;
/// P7 dense patch: 64 × 64 contiguous cells centred on the Shackleton rim, map metres.
pub const PATCH_SIDE: u32 = 64;
pub const PATCH_CENTRE_M: (f64, f64) = (-11_000.0, -12_000.0);
/// P7: a fitted shift larger than this, metres, is flagged.
pub const SHIFT_FLAG_M: f64 = 15.0;
const STEP_S: f64 = 3600.0;
const BINS: usize = 5;

/// One pixel of the reference raster that may be drawn: AVGVISIB (col, row) and its value, points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    pub col: u32,
    pub row: u32,
    pub ref_pct: f64,
}

/// Bin of a reference value in points: min(4, floor(pct / 20)). Errors outside [0, 100] or NaN.
pub fn bin_of(pct: f64) -> Result<usize> {
    if !(0.0..=100.0).contains(&pct) {
        return Err(format!("reference value {pct} is not in 0..=100 points").into());
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // floor of 0..=5, checked
    let b = (pct / 20.0).floor() as usize;
    Ok(b.min(BINS - 1))
}

fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9e37_79b9_7f4a_7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

/// Stratified draw: per bin, the `quota` candidates first in splitmix64(seed ^ (row << 32 | col))
/// order, or all of a bin with fewer. Each bin comes back in that hash order, so the result does
/// not depend on the input order.
pub fn stratified_draw(
    cands: &[Candidate],
    seed: u64,
    quota: usize,
) -> Result<[Vec<Candidate>; BINS]> {
    let mut bins: [Vec<Candidate>; BINS] = Default::default();
    for c in cands {
        bins[bin_of(c.ref_pct)?].push(*c);
    }
    for b in &mut bins {
        b.sort_by_key(|c| {
            let key = (u64::from(c.row) << 32) | u64::from(c.col);
            (splitmix64(seed ^ key), c.row, c.col)
        });
        b.truncate(quota);
    }
    Ok(bins)
}

/// Error statistics of e = ours − reference, points.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub n: usize,
    /// Median of |e| (mean of the two middle values when n is even).
    pub median_abs: f64,
    /// 90th percentile of |e|, nearest rank ⌈0.9 n⌉.
    pub p90_abs: f64,
    /// Mean signed error.
    pub mean: f64,
    /// Pearson r of ours against the reference.
    pub r: f64,
}

/// Value at nearest rank ⌈p·n⌉ of an ascending, non-empty slice.
fn nearest_rank(sorted: &[f64], p: f64) -> f64 {
    #[allow(clippy::cast_precision_loss)] // n < 2^52
    let n = sorted.len() as f64;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 1 ≤ rank ≤ n
    let rank = ((p * n).ceil() as usize).clamp(1, sorted.len());
    sorted[rank - 1]
}

/// Statistics of `ours` against `reference` (both points). Errors on unequal lengths, fewer than
/// two values, a non-finite value, or zero variance in either series (r undefined, never NaN).
pub fn stats(ours: &[f64], reference: &[f64]) -> Result<Stats> {
    if ours.len() != reference.len() || ours.len() < 2 {
        return Err("stats need two series of equal length ≥ 2".into());
    }
    if !ours.iter().chain(reference).all(|v| v.is_finite()) {
        return Err("stats need finite values".into());
    }
    let n = ours.len();
    #[allow(clippy::cast_precision_loss)] // n < 2^52
    let nf = n as f64;
    let e: Vec<f64> = ours.iter().zip(reference).map(|(o, r)| o - r).collect();
    let mut abs: Vec<f64> = e.iter().map(|v| v.abs()).collect();
    abs.sort_by(f64::total_cmp);
    let median_abs = if n % 2 == 1 {
        abs[n / 2]
    } else {
        (abs[n / 2 - 1] + abs[n / 2]) / 2.0
    };
    let mean = e.iter().sum::<f64>() / nf;
    let (mo, mr) = (
        ours.iter().sum::<f64>() / nf,
        reference.iter().sum::<f64>() / nf,
    );
    let (mut sxy, mut sxx, mut syy) = (0.0, 0.0, 0.0);
    for (o, r) in ours.iter().zip(reference) {
        let (dx, dy) = (o - mo, r - mr);
        sxy += dx * dy;
        sxx += dx * dx;
        syy += dy * dy;
    }
    if sxx <= 0.0 || syy <= 0.0 {
        return Err("Pearson r is undefined: a series has zero variance".into());
    }
    Ok(Stats {
        n,
        median_abs,
        p90_abs: nearest_rank(&abs, 0.9),
        mean,
        r: sxy / (sxx * syy).sqrt(),
    })
}

/// The V2 pass (SCIENCE-SPEC §3, inclusive): pooled median |e| ≤ 5, |mean e| ≤ 2, r ≥ 0.9.
pub fn passes(s: &Stats) -> bool {
    s.median_abs <= 5.0 && s.mean.abs() <= 2.0 && s.r >= 0.9
}

/// The V2 stretch target (SCIENCE-SPEC §3): median |e| ≤ 2 and p90 |e| ≤ 8.
pub fn stretch(s: &Stats) -> bool {
    s.median_abs <= 2.0 && s.p90_abs <= 8.0
}

fn whole(v: f64, what: &str) -> Result<i64> {
    if (v - v.round()).abs() > 1e-6 || v.abs() > 1e9 {
        return Err(format!("{what} {v} is not a whole number of pixels").into());
    }
    #[allow(clippy::cast_possible_truncation)] // whole and below 1e9, checked above
    Ok(v.round() as i64)
}

/// Average a 20 m tile 3 × 3 onto the AVGVISIB 60 m lattice (pixel edges at −151 740 + 60k m on x,
/// 151 740 − 60k m on y): the DEM is averaged, not nine results (§3 V2 details). Heights decode as
/// offset + scale × count and average in f64; a cell touching nodata is NaN (NoData when sampled).
/// Errors unless the tile is 20 m and on the 20 m lattice that nests in the 60 m one.
pub fn average_3x3(tile: &DemTile) -> Result<FloatWindow> {
    let m = *tile.meta();
    if (m.pixel_m - NEAR_PX_M).abs() > 1e-9 {
        return Err(format!("tile pixel is {} m, not 20 m", m.pixel_m).into());
    }
    let kx = whole((m.x0_m - REF_X0_M) / NEAR_PX_M, "tile x0 offset")?;
    let ky = whole((REF_Y0_M - m.y0_m) / NEAR_PX_M, "tile y0 offset")?;
    let f = i64::from(FACTOR);
    let (c0, r0) = (
        u32::try_from((f - kx.rem_euclid(f)) % f)?,
        u32::try_from((f - ky.rem_euclid(f)) % f)?,
    );
    let (w, h) = (
        m.width.saturating_sub(c0) / FACTOR,
        m.height.saturating_sub(r0) / FACTOR,
    );
    let counts = tile.counts();
    let mut out = Vec::with_capacity(w as usize * h as usize);
    for j in 0..h {
        for i in 0..w {
            let mut sum = 0.0;
            let mut ok = true;
            for dr in 0..FACTOR {
                for dc in 0..FACTOR {
                    let (col, row) = (c0 + FACTOR * i + dc, r0 + FACTOR * j + dr);
                    let c = counts[row as usize * m.width as usize + col as usize];
                    if c == m.nodata {
                        ok = false;
                    }
                    sum += m.offset_m + m.scale_m * f64::from(c);
                }
            }
            #[allow(clippy::cast_possible_truncation)] // metres, f32 keeps ~0.5 mm at 10 km (P11)
            out.push(if ok { (sum / 9.0) as f32 } else { f32::NAN });
        }
    }
    FloatWindow::from_heights(
        m.x0_m + f64::from(c0) * NEAR_PX_M,
        m.y0_m - f64::from(r0) * NEAR_PX_M,
        NEAR_PX_M * f64::from(FACTOR),
        w,
        h,
        out,
    )
}

/// One AVGVISIB raster held in memory as points (count × 0.004; nodata → NaN), on the fixed grid above.
#[derive(Debug)]
pub struct RefRaster {
    values_pct: Vec<f32>,
}

impl RefRaster {
    /// Read the whole raster (5058² int16, about 100 MB as f32) after checking its grid facts.
    pub fn open(path: &Path) -> Result<RefRaster> {
        let mut g = GeoTiff::open(path)?;
        let grid_ok = g.width == REF_SIDE_PX
            && g.height == REF_SIDE_PX
            && (g.pixel_m - REF_PX_M).abs() < 1e-9
            && (g.x0_m - REF_X0_M).abs() < 1e-6
            && (g.y0_m - REF_Y0_M).abs() < 1e-6;
        if !grid_ok || !g.pixel_is_area {
            return Err(format!(
                "{}: not the AVGVISIB 60 m pixel-is-area grid",
                path.display()
            )
            .into());
        }
        if g.sphere_radius_m != Some(penumbra_core::geometry::MOON_RADIUS_M) {
            return Err(format!("{}: not on the 1 737 400 m sphere", path.display()).into());
        }
        let counts = g.read_window(0, 0, g.width, g.height)?;
        #[allow(clippy::cast_possible_truncation)]
        // points ≤ 131.07, f32 is exact enough (0.004 steps)
        let values_pct = counts
            .into_iter()
            .map(|c| {
                if c == REF_NODATA_COUNT {
                    f32::NAN
                } else {
                    (f64::from(c) * POINTS_PER_COUNT) as f32
                }
            })
            .collect();
        Ok(RefRaster { values_pct })
    }

    /// Value at (col, row), points. Errors outside the raster or on nodata.
    pub fn at_cell(&self, col: i64, row: i64) -> Result<f64> {
        let side = i64::from(REF_SIDE_PX);
        if !(0..side).contains(&col) || !(0..side).contains(&row) {
            return Err(format!("cell ({col}, {row}) is outside AVGVISIB").into());
        }
        let v = self.values_pct[usize::try_from(row * side + col)?];
        if v.is_nan() {
            return Err(format!("cell ({col}, {row}) is nodata").into());
        }
        Ok(f64::from(v))
    }

    /// Value of the pixel holding map point `xy`, points.
    pub fn at(&self, xy: MapXy) -> Result<f64> {
        let (c, r) = cell_of(xy)?;
        self.at_cell(c, r)
    }

    /// Central-difference gradient (∂/∂x, ∂/∂y), points per map metre, y up (row − 1).
    pub fn grad(&self, col: i64, row: i64) -> Result<(f64, f64)> {
        let gx = (self.at_cell(col + 1, row)? - self.at_cell(col - 1, row)?) / (2.0 * REF_PX_M);
        let gy = (self.at_cell(col, row - 1)? - self.at_cell(col, row + 1)?) / (2.0 * REF_PX_M);
        Ok((gx, gy))
    }
}

/// AVGVISIB (col, row) of the pixel holding map point `xy`.
pub fn cell_of(xy: MapXy) -> Result<(i64, i64)> {
    if !xy.x_m.is_finite() || !xy.y_m.is_finite() {
        return Err("map point must be finite".into());
    }
    let c = ((xy.x_m - REF_X0_M) / REF_PX_M).floor();
    let r = ((REF_Y0_M - xy.y_m) / REF_PX_M).floor();
    Ok((whole(c, "col")?, whole(r, "row")?))
}

/// Centre of AVGVISIB pixel (col, row), map metres.
pub fn cell_centre(col: i64, row: i64) -> MapXy {
    #[allow(clippy::cast_precision_loss)] // |col|, |row| < 2^31
    MapXy {
        x_m: REF_X0_M + (col as f64 + 0.5) * REF_PX_M,
        y_m: REF_Y0_M - (row as f64 + 0.5) * REF_PX_M,
    }
}

/// One pixel's shares over the hourly window, points (0–100).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shares {
    /// Sun visibility: steps with disc fraction f > 0 (P1, W1; what AVGVISIB averages).
    pub sun_vis_pct: f64,
    /// Mean Sun disc fraction (illumination); never above `sun_vis_pct`.
    pub sun_illum_pct: f64,
    /// Earth "any fraction visible" (P14, `earth_visible_any_fraction`); not comms contact.
    pub earth_vis_pct: f64,
    /// Sun visibility over the window started `shift` steps later (U2), when asked for.
    pub sun_vis_shifted_pct: Option<f64>,
    /// Sun centre elevation range and largest Sun angular radius over the window, degrees (topocentric ENU).
    pub sun_el_min_deg: f64,
    pub sun_el_max_deg: f64,
    pub sun_rho_max_deg: f64,
}

/// Disc fraction above the horizon, with an exact shortcut: a disc wholly above the highest bin
/// is 1 and wholly below the lowest is 0, as `disc_fraction` gives for those cases (N2).
fn fraction(
    el_hor_rad: &[f64; N_AZ],
    (lo_rad, hi_rad): (f64, f64),
    az_deg: f64,
    el_deg: f64,
    radius_deg: f64,
) -> Result<f64> {
    let (el, rho) = (el_deg.to_radians(), radius_deg.to_radians());
    if el - rho > hi_rad {
        return Ok(1.0);
    }
    if el + rho < lo_rad {
        return Ok(0.0);
    }
    let c = AzEl {
        az_rad: az_deg.to_radians(),
        el_rad: el,
    };
    disc_fraction(el_hor_rad, c, rho).map_err(|e| format!("{e:?}").into())
}

/// Shares at `site` against horizon `el_hor_rad` (topocentric ENU, radians, bin i at i × 0.5°) for
/// `steps` hourly steps from `start_unix_s` (Unix s UTC). With `shift = Some(s)` one track of
/// `steps + s` rows also gives the Sun visibility of the window `[s, s + steps)` (U2).
pub fn shares(
    eph: &Ephemeris,
    el_hor_rad: &[f64; N_AZ],
    site: &Site,
    start_unix_s: f64,
    steps: usize,
    shift: Option<usize>,
) -> Result<Shares> {
    if steps == 0 {
        return Err("shares need at least one step".into());
    }
    // with a shift s the one track spans both windows, [0, steps) and [s, s + steps)
    let rows = match shift {
        Some(s) => s + steps,
        None => steps,
    };
    let track = sun_earth_track(eph, site, start_unix_s, STEP_S, u32::try_from(rows)?)?;
    // sun_earth_track columns (docs/API.md): t, sun az/el/radius/dist, earth az/el/radius/dist.
    let col = |c: usize| &track[c * rows..(c + 1) * rows];
    let (sun_az, sun_el, sun_r) = (col(1), col(2), col(3));
    let (earth_az, earth_el, earth_r) = (col(5), col(6), col(7));
    let bounds = el_hor_rad
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &e| {
            (lo.min(e), hi.max(e))
        });
    let (mut vis, mut illum, mut earth, mut shifted) = (0.0, 0.0, 0.0, 0.0);
    let (mut el_lo, mut el_hi, mut rho_hi) = (f64::INFINITY, f64::NEG_INFINITY, 0.0f64);
    for k in 0..rows {
        let f = fraction(el_hor_rad, bounds, sun_az[k], sun_el[k], sun_r[k])?;
        let v = f64::from(u8::from(f > 0.0));
        if k < steps {
            vis += v;
            illum += f;
            let e = fraction(el_hor_rad, bounds, earth_az[k], earth_el[k], earth_r[k])?;
            earth += f64::from(u8::from(e > 0.0));
            el_lo = el_lo.min(sun_el[k]);
            el_hi = el_hi.max(sun_el[k]);
            rho_hi = rho_hi.max(sun_r[k]);
        }
        if shift.is_some_and(|s| k >= s) {
            shifted += v;
        }
    }
    #[allow(clippy::cast_precision_loss)] // steps < 2^32
    let n = steps as f64;
    Ok(Shares {
        sun_vis_pct: 100.0 * vis / n,
        sun_illum_pct: 100.0 * illum / n,
        earth_vis_pct: 100.0 * earth / n,
        sun_vis_shifted_pct: shift.map(|_| 100.0 * shifted / n),
        sun_el_min_deg: el_lo,
        sun_el_max_deg: el_hi,
        sun_rho_max_deg: rho_hi,
    })
}

/// N5 critical obstacle, metres: the nearest `dist_m` among azimuth bins whose horizon elevation
/// (degrees, topocentric ENU) lies in `[band_lo_deg, band_hi_deg]`, the band the Sun's disc crosses
/// (min el_sun − ρ_max .. max el_sun + ρ_max). None when no bin lies in the band.
pub fn critical_obstacle_m(hor: &Horizon, band_lo_deg: f64, band_hi_deg: f64) -> Option<f64> {
    hor.el_rad
        .iter()
        .zip(&hor.dist_m)
        .filter(|(e, _)| (band_lo_deg..=band_hi_deg).contains(&e.to_degrees()))
        .map(|(_, &d)| d)
        .reduce(f64::min)
}

/// P7 shift fit: ordinary least squares of e ≈ a + δx·gx + δy·gy, with g the reference gradient
/// (points per metre). Returns (a points, δx m, δy m); ours(p) ≈ ref(p + δ). Errors when singular.
pub fn shift_fit(e: &[f64], gx: &[f64], gy: &[f64]) -> Result<(f64, f64, f64)> {
    if e.len() != gx.len() || e.len() != gy.len() || e.len() < 3 {
        return Err("shift fit needs three equal series of ≥ 3 values".into());
    }
    let mut m = [[0.0f64; 3]; 3];
    let mut v = [0.0f64; 3];
    for ((&ek, &x), &y) in e.iter().zip(gx).zip(gy) {
        let row = [1.0, x, y];
        for i in 0..3 {
            v[i] += row[i] * ek;
            for j in 0..3 {
                m[i][j] += row[i] * row[j];
            }
        }
    }
    let det3 = |a: &[[f64; 3]; 3]| {
        a[0][0] * (a[1][1] * a[2][2] - a[1][2] * a[2][1])
            - a[0][1] * (a[1][0] * a[2][2] - a[1][2] * a[2][0])
            + a[0][2] * (a[1][0] * a[2][1] - a[1][1] * a[2][0])
    };
    let d = det3(&m);
    if !d.is_finite() || d.abs() < 1e-300 {
        return Err("shift fit is singular (no gradient)".into());
    }
    // Cramer's rule
    let solve = |k: usize| {
        let mut a = m;
        for i in 0..3 {
            a[i][k] = v[i];
        }
        det3(&a) / d
    };
    Ok((solve(0), solve(1), solve(2)))
}

/// Map `f` over `items` on every core; results in input order. Items are dealt round-robin so
/// slow neighbours spread across threads. The first error stops every thread before its next item.
pub fn par_map<T: Sync, R: Send>(
    items: &[T],
    f: impl Fn(&T) -> std::result::Result<R, String> + Sync,
) -> Result<Vec<R>> {
    let threads = std::thread::available_parallelism()?.get();
    let done = std::sync::atomic::AtomicUsize::new(0);
    let stop = std::sync::atomic::AtomicBool::new(false);
    let parts = std::thread::scope(|s| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let (f, done, stop) = (&f, &done, &stop);
                s.spawn(move || {
                    let mut out = Vec::new();
                    for (i, item) in items.iter().enumerate().skip(t).step_by(threads) {
                        if stop.load(std::sync::atomic::Ordering::Relaxed) {
                            break;
                        }
                        let r = f(item).inspect_err(|_| {
                            stop.store(true, std::sync::atomic::Ordering::Relaxed);
                        })?;
                        out.push((i, r));
                        let k = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
                        if k % 1000 == 0 {
                            eprintln!("  {k} / {}", items.len());
                        }
                    }
                    Ok::<_, String>(out)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().map_err(|_| "worker thread panicked".to_string())?)
            .collect::<std::result::Result<Vec<_>, String>>()
    })?;
    let mut slots: Vec<Option<R>> = (0..items.len()).map(|_| None).collect();
    for (i, r) in parts.into_iter().flatten() {
        slots[i] = Some(r);
    }
    slots
        .into_iter()
        .map(|r| r.ok_or_else(|| "a result is missing".into()))
        .collect()
}

/// A drawn pixel: region index, AVGVISIB cell and centre, and both reference values (points).
#[derive(Debug, Clone, Copy)]
struct Pixel {
    region: usize,
    col: i64,
    row: i64,
    xy: MapXy,
    sun_ref: f64,
    earth_ref: f64,
}

/// One evaluated pixel.
#[derive(Debug, Clone, Copy)]
struct Out {
    s: Shares,
    crit_m: Option<f64>,
}

fn eval(
    near: &FloatWindow,
    mid: &DemTile,
    far: &DemTile,
    eph: &Ephemeris,
    xy: MapXy,
    mast_m: f64,
    shift: Option<usize>,
) -> Result<Out> {
    let err = |e: &dyn std::fmt::Debug| format!("{e:?}");
    let ll = xy_to_latlon(xy).map_err(|e| err(&e))?;
    let hor =
        horizon(near, mid, far, ll, mast_m, &HorizonOptions::default()).map_err(|e| err(&e))?;
    let site = Site {
        lat_rad: ll.lat_rad,
        lon_east_rad: ll.lon_east_rad,
        ground_m: ground_height_m(near, ll).map_err(|e| err(&e))?,
        mast_m,
    };
    let s = shares(eph, &hor.el_rad, &site, START_UNIX_S, STEPS, shift)?;
    if s.sun_vis_pct < s.sun_illum_pct {
        return Err(format!("visibility below illumination at {xy:?}").into());
    }
    let crit_m = critical_obstacle_m(
        &hor,
        s.sun_el_min_deg - s.sun_rho_max_deg,
        s.sun_el_max_deg + s.sun_rho_max_deg,
    );
    Ok(Out { s, crit_m })
}

fn stats_json(ours: &[f64], reference: &[f64]) -> serde_json::Value {
    match stats(ours, reference) {
        Ok(s) => serde_json::json!({
            "n": s.n, "median_abs": s.median_abs, "p90_abs": s.p90_abs,
            "mean_signed": s.mean, "r": s.r,
        }),
        Err(e) => serde_json::json!({"n": ours.len(), "error": e.to_string()}),
    }
}

/// Region ids of the shipped near tiles, in the order the draw uses.
const REGIONS: [&str; 2] = ["shackleton-ridge", "mons-mouton"];

type Key = (i64, i64);
/// A map-coordinate transform for the P7 orientation check.
type MapTransform = fn(MapXy) -> MapXy;

/// Run V2: tiles from `data_dir` (`web/public/data`), kernels and both AVGVISIB rasters from
/// `resource_dir` (PENUMBRA_DATA). Returns the report as JSON for the scoreboard row.
#[allow(clippy::too_many_lines)] // one report, read top to bottom
pub fn run(data_dir: &Path, resource_dir: &Path) -> Result<serde_json::Value> {
    let started = std::time::Instant::now();
    let tiles = ShippedTiles::load(data_dir)?;
    let near: Vec<FloatWindow> = REGIONS
        .iter()
        .map(|id| average_3x3(tiles.near(id)?))
        .collect::<Result<_>>()?;
    let mid: Vec<&DemTile> = REGIONS
        .iter()
        .map(|id| tiles.mid(id))
        .collect::<Result<_>>()?;
    let sun_ref = RefRaster::open(&resource_dir.join("AVGVISIB_85S_060M_201608.TIF"))?;
    let earth_ref = RefRaster::open(&resource_dir.join("AVGVISIB_85S_060M_201608_EARTH.TIF"))?;

    // Candidates: every AVGVISIB cell ≥ 7.2 km inside an averaged tile, both regions.
    let mut pixels: BTreeMap<Key, Pixel> = BTreeMap::new();
    let mut skipped = 0usize;
    for (region, w) in near.iter().enumerate() {
        let (wi, hi, x0, y0, _) = w.grid();
        let c0 = whole((x0 - REF_X0_M) / REF_PX_M, "window col")?;
        let r0 = whole((REF_Y0_M - y0) / REF_PX_M, "window row")?;
        for j in MARGIN_CELLS..hi.saturating_sub(MARGIN_CELLS) {
            for i in MARGIN_CELLS..wi.saturating_sub(MARGIN_CELLS) {
                let (col, row) = (c0 + i64::from(i), r0 + i64::from(j));
                let xy = cell_centre(col, row);
                match (
                    sun_ref.at_cell(col, row),
                    earth_ref.at_cell(col, row),
                    w.height_m(xy),
                ) {
                    (Ok(sun), Ok(earth), Ok(_)) => {
                        pixels.insert(
                            (row, col),
                            Pixel {
                                region,
                                col,
                                row,
                                xy,
                                sun_ref: sun,
                                earth_ref: earth,
                            },
                        );
                    }
                    _ => skipped += 1,
                }
            }
        }
    }
    let cand = |get: fn(&Pixel) -> f64| -> Result<Vec<Candidate>> {
        pixels
            .values()
            .map(|p| {
                Ok(Candidate {
                    col: u32::try_from(p.col)?,
                    row: u32::try_from(p.row)?,
                    ref_pct: get(p),
                })
            })
            .collect()
    };
    let (sun_cands, earth_cands) = (cand(|p| p.sun_ref)?, cand(|p| p.earth_ref)?);
    let avail = |c: &[Candidate]| -> Result<Vec<usize>> {
        let mut n = vec![0; BINS];
        for x in c {
            n[bin_of(x.ref_pct)?] += 1;
        }
        Ok(n)
    };
    let (sun_avail, earth_avail) = (avail(&sun_cands)?, avail(&earth_cands)?);
    // Same seed for both draws: each is still a uniform draw per bin, and the overlap saves work.
    let sun_bins = stratified_draw(&sun_cands, SEED, QUOTA)?;
    let earth_bins = stratified_draw(&earth_cands, SEED, QUOTA)?;
    let counts = |b: &[Vec<Candidate>; BINS]| b.iter().map(Vec::len).collect::<Vec<_>>();
    eprintln!(
        "V2: {} candidates ({skipped} skipped); Sun available per bin {sun_avail:?}, drawn {:?}; Earth drawn {:?}; seed {SEED:#x}",
        pixels.len(),
        counts(&sun_bins),
        counts(&earth_bins)
    );
    let key = |c: &Candidate| (i64::from(c.row), i64::from(c.col));
    let sub: Vec<Key> = sun_bins
        .iter()
        .flat_map(|b| pick_spread(b, SUB_PER_BIN))
        .map(|c| key(&c))
        .collect();
    let (pc, pr) = cell_of(MapXy {
        x_m: PATCH_CENTRE_M.0,
        y_m: PATCH_CENTRE_M.1,
    })?;
    let half = i64::from(PATCH_SIDE / 2);
    let patch: Vec<Key> = (pr - half..pr + half)
        .flat_map(|r| (pc - half..pc + half).map(move |c| (r, c)))
        .collect();

    // Every pixel once at H = 0; the subsample also carries the shifted window (U2).
    let mut tasks: BTreeMap<Key, bool> = BTreeMap::new();
    for k in sun_bins.iter().chain(&earth_bins).flatten().map(key) {
        tasks.insert(k, false);
    }
    for k in &patch {
        tasks.insert(*k, false);
    }
    for k in &sub {
        tasks.insert(*k, true);
    }
    let task_list: Vec<(Pixel, bool)> = tasks
        .iter()
        .map(|(k, &s)| {
            pixels
                .get(k)
                .map(|p| (*p, s))
                .ok_or_else(|| format!("pixel {k:?} is not a candidate").into())
        })
        .collect::<Result<_>>()?;

    let almanac = load_almanac(resource_dir)?;
    let mut eph = Ephemeris::default();
    eph.add(
        START_UNIX_S,
        STEP_S,
        build_table(&almanac, START_UNIX_S, STEP_S, STEPS + SHIFT_STEPS)?,
    )?;
    eprintln!(
        "V2: table built ({:.0} s); {} pixels at H = 0 ({} with the U2 shift), {} at H = {P2_MAST_M} m",
        started.elapsed().as_secs_f64(),
        task_list.len(),
        sub.len(),
        sub.len()
    );
    let h0 = par_map(&task_list, |(p, s)| {
        eval(
            &near[p.region],
            mid[p.region],
            &tiles.far,
            &eph,
            p.xy,
            0.0,
            s.then_some(SHIFT_STEPS),
        )
        .map_err(|e| e.to_string())
    })?;
    let ours: BTreeMap<Key, Out> = tasks.keys().copied().zip(h0).collect();
    let sub_px: Vec<Pixel> = sub.iter().map(|k| pixels[k]).collect();
    let h2 = par_map(&sub_px, |p| {
        eval(
            &near[p.region],
            mid[p.region],
            &tiles.far,
            &eph,
            p.xy,
            P2_MAST_M,
            None,
        )
        .map_err(|e| e.to_string())
    })?;

    // Sun: per bin and pooled.
    let sun_all: Vec<Key> = sun_bins.iter().flatten().map(key).collect();
    let series = |ks: &[Key], o: fn(&Out) -> f64, r: fn(&Pixel) -> f64| {
        let a: Vec<f64> = ks.iter().map(|k| o(&ours[k])).collect();
        let b: Vec<f64> = ks.iter().map(|k| r(&pixels[k])).collect();
        (a, b)
    };
    let sun_o = |o: &Out| o.s.sun_vis_pct;
    let sun_r = |p: &Pixel| p.sun_ref;
    let (so, sr) = series(&sun_all, sun_o, sun_r);
    let pooled = stats(&so, &sr)?;
    let per_bin = |bins: &[Vec<Candidate>; BINS], o: fn(&Out) -> f64, r: fn(&Pixel) -> f64| {
        bins.iter()
            .enumerate()
            .map(|(b, bin)| {
                let ks: Vec<_> = bin.iter().map(key).collect();
                let (a, c) = series(&ks, o, r);
                let mut j = stats_json(&a, &c);
                j["bin"] = format!("{}-{}", 20 * b, 20 * (b + 1)).into();
                j
            })
            .collect::<Vec<_>>()
    };
    let earth_all: Vec<Key> = earth_bins.iter().flatten().map(key).collect();
    let earth_o = |o: &Out| o.s.earth_vis_pct;
    let earth_r = |p: &Pixel| p.earth_ref;
    let (eo, er) = series(&earth_all, earth_o, earth_r);
    // Each metric on the other draw too (report only): "evaluate both metrics on the union".
    let (so_e, sr_e) = series(&earth_all, sun_o, sun_r);
    let (eo_s, er_s) = series(&sun_all, earth_o, earth_r);

    // P2: the subsample at H = 0 and H = 2 m.
    let (s0, sref) = series(&sub, sun_o, sun_r);
    let s2: Vec<f64> = h2.iter().map(|o| o.s.sun_vis_pct).collect();
    #[allow(clippy::cast_precision_loss)] // ~500 pixels
    let p2_mean_change = s2.iter().zip(&s0).map(|(a, b)| a - b).sum::<f64>() / s0.len() as f64;

    // U2: shifted start on the subsample.
    let mut du: Vec<f64> = sub
        .iter()
        .map(|k| {
            let s = ours[k].s;
            s.sun_vis_shifted_pct
                .map(|v| (v - s.sun_vis_pct).abs())
                .ok_or_else(|| format!("pixel {k:?} has no shifted window").into())
        })
        .collect::<Result<_>>()?;
    du.sort_by(f64::total_cmp);
    #[allow(clippy::cast_precision_loss)] // ~500 pixels
    let u2_mean = du.iter().sum::<f64>() / du.len() as f64;

    // U3 and N5 splits of the Sun draw.
    let split = |pred: &dyn Fn(&Key) -> Result<bool>| -> Result<serde_json::Value> {
        let (mut yes, mut no) = (Vec::new(), Vec::new());
        for k in &sun_all {
            if pred(k)? {
                yes.push(*k);
            } else {
                no.push(*k);
            }
        }
        let (a, b) = series(&yes, sun_o, sun_r);
        let (c, d) = series(&no, sun_o, sun_r);
        Ok(serde_json::json!({"flagged": stats_json(&a, &b), "rest": stats_json(&c, &d)}))
    };
    let u3 = split(&|k| {
        let ll = xy_to_latlon(pixels[k].xy).map_err(|e| format!("{e:?}"))?;
        Ok(ll.lat_rad.to_degrees() <= U3_LAT_DEG)
    })?;
    let n5 = split(&|k| Ok(ours[k].crit_m.is_some_and(|d| d < N5_CRIT_M)))?;

    // P7 (a): orientation. Ours against the reference read at transformed map points.
    let transforms: [(&str, MapTransform); 4] = [
        ("identity (x, y)", |p| p),
        ("mirror x (-x, y)", |p| MapXy {
            x_m: -p.x_m,
            y_m: p.y_m,
        }),
        ("mirror y (x, -y)", |p| MapXy {
            x_m: p.x_m,
            y_m: -p.y_m,
        }),
        ("transpose (y, x)", |p| MapXy {
            x_m: p.y_m,
            y_m: p.x_m,
        }),
    ];
    let mut orientation = Vec::new();
    for (name, t) in transforms {
        let r: Vec<f64> = sun_all
            .iter()
            .map(|k| sun_ref.at(t(pixels[k].xy)))
            .collect::<Result<_>>()?;
        orientation.push(serde_json::json!({"transform": name, "r": stats(&so, &r)?.r}));
    }
    // P7 (b): shift fit, pooled and on the patch.
    let fit = |ks: &[Key]| -> Result<serde_json::Value> {
        let (mut e, mut gx, mut gy) = (Vec::new(), Vec::new(), Vec::new());
        for k in ks {
            let p = pixels[k];
            let g = sun_ref.grad(p.col, p.row)?;
            e.push(ours[k].s.sun_vis_pct - p.sun_ref);
            gx.push(g.0);
            gy.push(g.1);
        }
        let (a, dx, dy) = shift_fit(&e, &gx, &gy)?;
        Ok(serde_json::json!({
            "n": e.len(), "intercept_points": a, "dx_m": dx, "dy_m": dy,
            "flag": dx.hypot(dy) > SHIFT_FLAG_M,
        }))
    };
    // P7 (c): ASCII residual map of the patch, row 0 = north (largest y).
    let mut map = Vec::new();
    for r in pr - half..pr + half {
        let line: String = (pc - half..pc + half)
            .map(|c| {
                let e = ours[&(r, c)].s.sun_vis_pct - pixels[&(r, c)].sun_ref;
                if e > 10.0 {
                    '+'
                } else if e < -10.0 {
                    '-'
                } else {
                    '.'
                }
            })
            .collect();
        map.push(line);
    }
    let (po, pref) = series(&patch, sun_o, sun_r);

    // Reference speckle: sampled zeros whose 8 neighbours are all ≥ 20 points.
    let is_zero = |k: &&Key| pixels[*k].sun_ref <= 0.0;
    let isolated_zeros = sun_all
        .iter()
        .filter(is_zero)
        .filter(|k| {
            let p = pixels[*k];
            (-1..=1).all(|dr| {
                (-1..=1).all(|dc| {
                    (dr == 0 && dc == 0)
                        || sun_ref
                            .at_cell(p.col + dc, p.row + dr)
                            .is_ok_and(|v| v >= 20.0)
                })
            })
        })
        .count();
    let zeros = sun_all.iter().filter(is_zero).count();

    Ok(serde_json::json!({
        "metric": "visibility (any fraction of disc, f > 0), points; e = ours - ref",
        "reference": "AVGVISIB_85S_060M_201608 (+ _EARTH), count x 0.004",
        "mast_m": 0.0,
        "start_unix_s": START_UNIX_S,
        "steps": STEPS,
        "step_s": STEP_S,
        "dem": "shipped 20 m near tiles averaged 3x3 to the AVGVISIB 60 m lattice (7 km near tier), shipped 80 m middle tiles to 31 km, shipped 240 m far tile beyond",
        "regions": REGIONS,
        "seed": format!("{SEED:#x}"),
        "candidates": pixels.len(),
        "candidates_skipped": skipped,
        "sun": {
            "available_per_bin": sun_avail,
            "drawn_per_bin": counts(&sun_bins),
            "pooled": stats_json(&so, &sr),
            "per_bin": per_bin(&sun_bins, sun_o, sun_r),
            "pass": passes(&pooled),
            "stretch": stretch(&pooled),
            "on_earth_draw": stats_json(&so_e, &sr_e),
            "zeros_sampled": zeros,
            "isolated_zeros_sampled": isolated_zeros,
        },
        "earth": {
            "metric": "any fraction of Earth's disc visible (P14), report only",
            "available_per_bin": earth_avail,
            "drawn_per_bin": counts(&earth_bins),
            "pooled": stats_json(&eo, &er),
            "per_bin": per_bin(&earth_bins, earth_o, earth_r),
            "on_sun_draw": stats_json(&eo_s, &er_s),
        },
        "p2": {
            "n": sub.len(),
            "h0": stats_json(&s0, &sref),
            "h2": stats_json(&s2, &sref),
            "mean_change_h2_minus_h0": p2_mean_change,
        },
        "u2": {
            "shift_steps": SHIFT_STEPS,
            "n": du.len(),
            "mean_abs_change": u2_mean,
            "p90_abs_change": nearest_rank(&du, 0.9),
            "max_abs_change": du.last().copied(),
            "pass": u2_mean < 0.5,
        },
        "u3": {"split": "lat <= -88 deg (flagged) vs rest; in our tiles this is Shackleton vs Mons Mouton", "stats": u3},
        "n5": {"split": "critical obstacle < 180 m (flagged) vs rest", "stats": n5},
        "p7": {
            "orientation_r": orientation,
            "shift_fit_pooled": fit(&sun_all)?,
            "shift_fit_patch": fit(&patch)?,
            "patch": {
                "centre_m": [PATCH_CENTRE_M.0, PATCH_CENTRE_M.1],
                "side_cells": PATCH_SIDE,
                "stats": stats_json(&po, &pref),
                "map": map,
                "legend": "'+' e > 10, '-' e < -10, '.' else; row 0 = north (largest map y), col 0 = west",
            },
        },
        "run_time_s": started.elapsed().as_secs_f64(),
    }))
}
