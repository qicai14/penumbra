//! Sunset ± band: first-order propagation of DEM height error onto each sunrise and sunset
//! (SCIENCE-SPEC §1f step 1, docs/briefs/T17-sunset-band.md).

use crate::eph::EphError;
use crate::geometry::GeometryError;
use crate::horizon::Horizon;
use crate::timeline::horizon_el_at_rad;

/// DEM height error at both the observer and the obstacle, metres. Assumed: the 0.30–0.50 m RMS
/// of the 5 m site DEMs (E23a); the `_ERR` raster is not used. The UI labels it "assumed".
pub const ASSUMED_SIGMA_Z_M: f64 = 0.5;

/// Elevation uncertainty, radians (topocentric ENU), of an obstacle `obstacle_dist_m` metres away:
/// √(σ_obstacle² + σ_observer²) / D (§1f step 1).
pub fn sigma_el_rad(
    sigma_obstacle_m: f64,
    sigma_observer_m: f64,
    obstacle_dist_m: f64,
) -> Result<f64, EphError> {
    if !(obstacle_dist_m > 0.0 && obstacle_dist_m.is_finite())
        || !(sigma_obstacle_m >= 0.0 && sigma_observer_m >= 0.0)
        || !(sigma_obstacle_m.is_finite() && sigma_observer_m.is_finite())
    {
        return Err(EphError::BadInput(
            "σ_z must be finite ≥ 0 and the obstacle distance finite > 0",
        ));
    }
    Ok(sigma_obstacle_m.hypot(sigma_observer_m) / obstacle_dist_m)
}

/// Upper bound on a half width, minutes: one 29.5-day synodic month (length from E118). A design
/// choice [D], not in §1f: a grazing crossing (rate → 0) is ill-conditioned, and past a month the
/// Sun's geometry repeats, so a wider band says nothing more. Keeps every band finite, never inf.
/// Callers (T18) must read `half_width_min == MAX_HALF_WIDTH_MIN` as "unbounded", not a real width.
pub const MAX_HALF_WIDTH_MIN: f64 = 29.5 * 24.0 * 60.0;

/// Columns of the API timeline (docs/API.md); the band reads t (0), Sun az (1) and Sun el (2).
const COLS: usize = 10;

/// Sunrise and sunset ± bands for `timeline` (the 10 × n T08 layout: Unix s UTC, Sun az/el in
/// topocentric ENU degrees) against `horizon`, with height error `sigma_z_m` at both the observer
/// and the obstacle (`ASSUMED_SIGMA_Z_M` by default). Returns 3 × k, column-major: `t_unix_s`,
/// `kind` (+1 sunrise, −1 sunset), `half_width_min`; k may be 0 (§1f step 1, docs/API.md).
///
/// An event is the Sun's centre crossing m(t) = el_sun − el_hor(az_sun) = 0 between two samples
/// (m_k < 0 ≤ m_{k+1} sunrise, m_k ≥ 0 > m_{k+1} sunset). Its time is linear in m; the rate is
/// the difference of m across that step, never 0.5°/h × slope. σ_el uses D of the bin under the
/// Sun at the crossing. Half width = σ_el / |dm/dt|, capped at `MAX_HALF_WIDTH_MIN`.
pub fn window_bands(
    timeline: &[f64],
    horizon: &Horizon,
    sigma_z_m: f64,
) -> Result<Vec<f64>, EphError> {
    if !timeline.len().is_multiple_of(COLS) {
        return Err(EphError::BadInput("timeline must be 10 × n"));
    }
    // σ_z > 0: zero would give a single time, which §1f forbids; NaN horizons would skip events.
    if !(sigma_z_m > 0.0 && sigma_z_m.is_finite()) {
        return Err(EphError::BadInput("σ_z must be finite > 0"));
    }
    if horizon
        .el_rad
        .iter()
        .chain(&horizon.dist_m)
        .any(|v| !v.is_finite())
    {
        return Err(EphError::BadInput("horizon values must be finite"));
    }
    let n = timeline.len() / COLS;
    let col = |c: usize| &timeline[c * n..(c + 1) * n];
    let (t, az_deg, el_deg) = (col(0), col(1), col(2));
    if t.iter().chain(az_deg).chain(el_deg).any(|v| !v.is_finite()) {
        return Err(EphError::BadInput("timeline Sun columns must be finite"));
    }
    let m_rad =
        (0..n)
            .map(|k| {
                Ok(el_deg[k].to_radians()
                    - horizon_el_at_rad(&horizon.el_rad, az_deg[k].to_radians())?)
            })
            .collect::<Result<Vec<f64>, GeometryError>>()
            .map_err(|GeometryError::BadInput(why)| EphError::BadInput(why))?;
    let (mut ts, mut kinds, mut widths) = (Vec::new(), Vec::new(), Vec::new());
    for k in 0..n.saturating_sub(1) {
        let (m0, m1) = (m_rad[k], m_rad[k + 1]);
        let kind = if m0 < 0.0 && m1 >= 0.0 {
            1.0
        } else if m0 >= 0.0 && m1 < 0.0 {
            -1.0
        } else {
            continue;
        };
        let dt_s = t[k + 1] - t[k];
        if dt_s <= 0.0 {
            return Err(EphError::BadInput("timeline times must increase"));
        }
        // m changed sign, so m1 − m0 ≠ 0.
        let frac = -m0 / (m1 - m0);
        // wrap the step to [−180°, 180°) so a crossing near north stays on the near side (N8)
        let daz_deg = (az_deg[k + 1] - az_deg[k] + 180.0).rem_euclid(360.0) - 180.0;
        let az = az_deg[k] + frac * daz_deg;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // round then rem_euclid gives an exact bin index in [0, 720), N_AZ bins of 0.5°
        let bin = (az / 0.5).round().rem_euclid(720.0) as usize;
        let sigma_rad = sigma_el_rad(sigma_z_m, sigma_z_m, horizon.dist_m[bin])?;
        // σ·Δt/|Δm| rather than σ/rate: no division by an underflowed rate.
        let half_width_min = (sigma_rad * dt_s / (m1 - m0).abs() / 60.0).min(MAX_HALF_WIDTH_MIN);
        ts.push(t[k] + frac * dt_s);
        kinds.push(kind);
        widths.push(half_width_min);
    }
    Ok([ts, kinds, widths].concat())
}
