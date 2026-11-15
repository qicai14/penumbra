//! Terrain horizon: a 720-azimuth great-circle ray march over a near, an optional middle and a far
//! DEM tier (SCIENCE-SPEC §1a, ARCHITECTURE §2.2). MOON_ME, sphere R = 1 737 400 m, radians inside.

use crate::dem::{DemError, DemTile, Terrain};
use crate::geometry::{
    EnuFrame, LatLon, MOON_RADIUS_M, MapXy, Vec3, latlon_to_unit, latlon_to_xy, unit_to_xy,
    xy_to_unit,
};

/// Azimuth bins, 0.5° each; bin i is azimuth i × 0.5° clockwise from north (§1a, E04).
pub const N_AZ: usize = 720;
/// The near (20 m) tier is marched out to this ground distance, metres (§1a tiers).
pub const NEAR_RANGE_M: f64 = 7_000.0;
/// The middle (80 m) tier is marched out to this ground distance, metres (§1a tiers, T-1a-4, #48).
/// 31 000 = 7 000 + 240 × 100, so the far samples beyond fall where the two-tier march puts them.
pub const MID_RANGE_M: f64 = 31_000.0;
/// The far tile must reach site + 200 km (§1a range, E66); a tile edge closer than this is an error.
pub const REQUIRED_RANGE_M: f64 = 200_000.0;
/// An obstacle closer than 3 near pixels (60 m) is flagged as a possible DEM spike (P13).
pub const NEAR_OBSTACLE_M: f64 = 60.0;

/// Pick border of a near tile, south polar stereographic map metres (P5, #26 S6): the closest a
/// site may sit to the tile edge and still march `NEAR_RANGE_M` of ground inside the tile.
/// Map length = ground length × k, k = 1 + r²/4R² (§1), largest at the tile corner farthest from
/// the pole; plus half a pixel, as the outermost pixel centre sits that far inside the edge.
/// Same rule as `pickBorderM` in web/src/lib/site.ts.
pub fn pick_border_m(near: &crate::dem::TileMeta) -> f64 {
    let x1_m = near.x0_m + f64::from(near.width) * near.pixel_m;
    let y1_m = near.y0_m - f64::from(near.height) * near.pixel_m;
    // the farthest corner of an axis-aligned tile takes the largest |x| and the largest |y|
    let x_m = near.x0_m.abs().max(x1_m.abs());
    let y_m = near.y0_m.abs().max(y1_m.abs());
    let k = 1.0 + (x_m * x_m + y_m * y_m) / (4.0 * MOON_RADIUS_M * MOON_RADIUS_M);
    NEAR_RANGE_M * k + near.pixel_m / 2.0
}

/// March settings. `Default` is the shipped behaviour.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HorizonOptions {
    /// Stop an azimuth when the far tile's highest point can no longer beat the current maximum
    /// (only armed when that point is above the observer, N11). Never changes the result.
    pub early_exit: bool,
    /// Ground distance, metres, after which the march switches from the near to the middle tile.
    pub near_range_m: f64,
    /// Ground distance, metres, after which the march switches from the middle to the far tile.
    pub mid_range_m: f64,
    /// Ground distance, metres, beyond which no sample is taken; `INFINITY` marches to the tile edge.
    pub max_range_m: f64,
}

impl Default for HorizonOptions {
    fn default() -> Self {
        HorizonOptions {
            early_exit: true,
            near_range_m: NEAR_RANGE_M,
            mid_range_m: MID_RANGE_M,
            max_range_m: f64::INFINITY,
        }
    }
}

/// Horizon of one site: elevation (topocentric ENU, radians above the local horizontal) and the
/// ground distance in metres of the sample that set it, per azimuth bin.
#[derive(Debug, Clone, PartialEq)]
pub struct Horizon {
    pub el_rad: [f64; N_AZ],
    pub dist_m: [f64; N_AZ],
}

impl Horizon {
    /// True when any bin's horizon is set by a sample under `NEAR_OBSTACLE_M` away (P13).
    pub fn has_near_obstacle(&self) -> bool {
        self.dist_m.iter().any(|&d| d < NEAR_OBSTACLE_M)
    }

    /// The API layout (docs/API.md): `el_deg[720]` then `dist_m[720]`.
    pub fn to_api(&self) -> Vec<f64> {
        self.el_rad
            .iter()
            .map(|e| e.to_degrees())
            .chain(self.dist_m.iter().copied())
            .collect()
    }
}

/// Ground height at `site` (MOON_ME lat, east lon), metres above the reference sphere, from the
/// near tile. `horizon` uses the same call, so the two cannot disagree (docs/API.md).
pub fn ground_height_m<N: Terrain>(near: &N, site: LatLon) -> Result<f64, DemError> {
    near.height_m(latlon_to_xy(site)?)
}

/// One step of the march: ground distance s, sin θ and versine 1 − cos θ, θ = s/R.
#[derive(Clone, Copy)]
struct Step {
    s_m: f64,
    sin: f64,
    vers: f64,
}

impl Step {
    fn at(s_m: f64) -> Step {
        let th = s_m / MOON_RADIUS_M;
        let half = (th / 2.0).sin();
        Step {
            s_m,
            sin: th.sin(),
            vers: 2.0 * half * half,
        }
    }
}

/// Elevation of the sample at step `st` along MOON_ME direction `d` from unit site `up`, as the
/// fraction (num, den) with tan el = num / den, den > 0; observer radius `r_o` metres (§1a closed form).
fn sample<T: Terrain>(
    t: &T,
    up: Vec3,
    d: Vec3,
    r_o: f64,
    st: &Step,
) -> Result<(f64, f64), DemError> {
    let u = up.scale(1.0 - st.vers).plus(d.scale(st.sin));
    let r_t = MOON_RADIUS_M + t.height_m(unit_to_xy(u)?)?;
    Ok(((r_t - r_o) - r_t * st.vers, r_t * st.sin))
}

/// Point-Sun shadow mask over a near tile at one instant (T21, the SCIENCE-SPEC §3 V3 method).
/// `lit` is row-major, `cols × rows`; entry (i, j) is the tile's pixel centre (col i·stride,
/// row j·stride). This is a point Sun, not illumination: no disc fraction.
#[derive(Debug, Clone, PartialEq)]
pub struct PointSunMask {
    pub cols: u32,
    pub rows: u32,
    pub lit: Vec<bool>,
}

/// Point-Sun mask: each grid point (see `PointSunMask`) is lit when the Sun's centre, at MOON_ME
/// position `sun_moon_me_m` (metres), is above every near-tile sample on one great-circle ray along
/// the Sun's topocentric ENU azimuth (the §1a closed form along one azimuth, not 720). The observer
/// is `mast_m` above the near tile's ground. Pixel centres of the last column and row lie outside
/// the bilinear footprint, so the grid is `ceil((width − 1) / stride)` × `ceil((height − 1) / stride)`.
/// Near-tile nodata on a ray is an error, never 0 m (P5).
// shortcut: only the near tile is marched (ARCHITECTURE §2.6), so a point whose ray leaves the tile before the blocking terrain reads lit; add a far-field horizon at the tile centre if that matters.
pub fn point_sun_mask(
    near: &DemTile,
    stride: u32,
    mast_m: f64,
    sun_moon_me_m: Vec3,
) -> Result<PointSunMask, DemError> {
    if stride == 0 {
        return Err(DemError::BadInput("stride must be at least 1"));
    }
    if !mast_m.is_finite() || mast_m < 0.0 {
        return Err(DemError::BadInput("mast height must be finite and ≥ 0"));
    }
    let m = *near.meta();
    let (cols, rows) = (
        (m.width - 1).div_ceil(stride),
        (m.height - 1).div_ceil(stride),
    );
    let half_circle_m = std::f64::consts::PI * MOON_RADIUS_M;
    let mut lit = Vec::new();
    for j in 0..rows {
        for i in 0..cols {
            let xy = MapXy {
                x_m: m.x0_m + m.pixel_m * (f64::from(i * stride) + 0.5),
                y_m: m.y0_m - m.pixel_m * (f64::from(j * stride) + 0.5),
            };
            let up = xy_to_unit(xy)?;
            let frame = EnuFrame::at(up)?;
            let r_o = MOON_RADIUS_M + near.height_m(xy)? + mast_m;
            let sun = frame.az_el(up.scale(r_o), sun_moon_me_m)?;
            let (sa, ca) = sun.az_rad.sin_cos();
            let d = frame.north.scale(ca).plus(frame.east.scale(sa));
            let (sin_el, cos_el) = sun.el_rad.sin_cos();
            // below the bare-sphere limb seen from the mast: dark even if the ray leaves the tile at once
            let dip = ((r_o - mast_m) / r_o).acos();
            let mut point_lit = sun.el_rad >= -dip;
            for k in 1u32.. {
                if !point_lit {
                    break;
                }
                let st = Step::at(m.pixel_m * f64::from(k));
                if st.s_m > half_circle_m {
                    break;
                }
                let (num, den) = match sample(near, up, d, r_o, &st) {
                    Ok(v) => v,
                    Err(DemError::OutOfCoverage) => break,
                    Err(e) => return Err(e),
                };
                // sample el > Sun el; den > 0 and cos el ≥ 0, so no division is needed
                if num * cos_el > sin_el * den {
                    point_lit = false;
                    break;
                }
            }
            lit.push(point_lit);
        }
    }
    Ok(PointSunMask { cols, rows, lit })
}

/// Horizon at `site` (MOON_ME lat, east lon) for an observer `mast_m` metres above the near tile's
/// ground, over the three tiers (#48). Each azimuth follows the great circle (P10), skips the
/// observer's own pixel (P13), samples every near pixel to `near_range_m`, every `mid` pixel to
/// `mid_range_m` and every far pixel beyond. Near- or middle-tile gaps or nodata, and a far tile
/// ending inside `REQUIRED_RANGE_M`, are errors, never 0 m (P5).
pub fn horizon<N: Terrain, M: Terrain, F: Terrain>(
    near: &N,
    mid: &M,
    far: &F,
    site: LatLon,
    mast_m: f64,
    opts: &HorizonOptions,
) -> Result<Horizon, DemError> {
    let (p_n, p_f) = (near.pixel_m(), far.pixel_m());
    if !mast_m.is_finite() || mast_m < 0.0 {
        return Err(DemError::BadInput("mast height must be finite and ≥ 0"));
    }
    if !opts.near_range_m.is_finite() || opts.near_range_m < p_n {
        return Err(DemError::BadInput(
            "near range must be finite and at least one near pixel",
        ));
    }
    if opts.max_range_m.is_nan() || opts.max_range_m < p_n {
        return Err(DemError::BadInput(
            "max range must be at least one near pixel",
        ));
    }
    let up = latlon_to_unit(site)?;
    let frame = EnuFrame::at(up)?;
    let r_o = MOON_RADIUS_M + ground_height_m(near, site)? + mast_m;
    let r_max = MOON_RADIUS_M + far.max_height_m();
    let exit_armed = opts.early_exit && r_max > r_o;

    // j = 1 is the first pixel out: the observer's own cell is never a sample (P13).
    // 1e-9 keeps an exact multiple (7000 / 20) from flooring one short.
    let n_near = (opts.near_range_m / p_n + 1e-9).floor();
    let near_end_m = n_near * p_n;
    let near_steps: Vec<Step> = (1..)
        .map(|j| Step::at(p_n * f64::from(j)))
        .take_while(|st| st.s_m <= near_end_m.min(opts.max_range_m))
        .collect();
    // Middle steps continue from the near range; the far march starts where they end.
    let p_m = mid.pixel_m();
    let n_mid = ((opts.mid_range_m - near_end_m) / p_m + 1e-9).floor();
    if !n_mid.is_finite() || n_mid < 1.0 {
        return Err(DemError::BadInput(
            "mid range must be finite and at least one mid pixel past the near range",
        ));
    }
    let far_start_m = near_end_m + n_mid * p_m;
    let mid_steps: Vec<Step> = (1..)
        .map(|k| Step::at(near_end_m + p_m * f64::from(k)))
        .take_while(|st| st.s_m <= far_start_m.min(opts.max_range_m))
        .collect();
    // Far steps are shared by all azimuths and grown on demand: the tile edge sets the length.
    let mut far_steps: Vec<Step> = Vec::new();
    // Half way round the Moon every great circle has left any tile: a hard stop for the loop.
    let half_circle_m = std::f64::consts::PI * MOON_RADIUS_M;

    let mut out = Horizon {
        el_rad: [0.0; N_AZ],
        dist_m: [0.0; N_AZ],
    };
    for i in 0..N_AZ {
        #[allow(clippy::cast_precision_loss)] // i < 720, exact in f64
        let az_rad = (i as f64) * std::f64::consts::PI / 360.0;
        let (sa, ca) = az_rad.sin_cos();
        let d = frame.north.scale(ca).plus(frame.east.scale(sa));
        let (mut best_num, mut best_den, mut best_s) = (-1.0, 0.0, 0.0);
        for st in &near_steps {
            let (num, den) = sample(near, up, d, r_o, st)?;
            if num * best_den > best_num * den {
                (best_num, best_den, best_s) = (num, den, st.s_m);
            }
        }
        for st in &mid_steps {
            let (num, den) = sample(mid, up, d, r_o, st)?;
            if num * best_den > best_num * den {
                (best_num, best_den, best_s) = (num, den, st.s_m);
            }
        }
        for j in 0.. {
            if far_steps.len() <= j {
                let k =
                    u32::try_from(j + 1).map_err(|_| DemError::BadInput("far march too long"))?;
                far_steps.push(Step::at(far_start_m + p_f * f64::from(k)));
            }
            let st = far_steps[j];
            if st.s_m > opts.max_range_m || st.s_m > half_circle_m {
                break;
            }
            if exit_armed {
                // B(s): the elevation of the far tile's highest point at s (§1a early exit, N11).
                let (bn, bd) = ((r_max - r_o) - r_max * st.vers, r_max * st.sin);
                if bn * best_den < best_num * bd {
                    break;
                }
            }
            let (num, den) = match sample(far, up, d, r_o, &st) {
                Ok(v) => v,
                Err(DemError::OutOfCoverage) if st.s_m >= REQUIRED_RANGE_M => break,
                Err(e) => return Err(e),
            };
            if num * best_den > best_num * den {
                (best_num, best_den, best_s) = (num, den, st.s_m);
            }
        }
        out.el_rad[i] = f64::atan2(best_num, best_den);
        out.dist_m[i] = best_s;
    }
    Ok(out)
}
