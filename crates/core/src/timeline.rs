//! Sun-disc fraction, Earth margin and the per-step timeline (SCIENCE-SPEC §1b, §1d, ARCHITECTURE §2.3).
//! Horizons are `[f64; N_AZ]` of topocentric ENU elevation in radians, bin i at azimuth i × 0.5°
//! clockwise from north; times are Unix s UTC.

use std::f64::consts::{FRAC_PI_2, TAU};

use crate::eph::{EphError, Ephemeris, Site, sun_earth_track};
use crate::geometry::{AzEl, GeometryError};
use crate::horizon::N_AZ;

/// Default Earth contact margin above the terrain, degrees. Assumed: De Rosa's "2° minimum" (E62);
/// the UI labels it "assumed" and the preset may change it (§1d).
pub const DEFAULT_EARTH_MARGIN_DEG: f64 = 2.0;

/// Midpoint strips across the disc (§1b).
const STRIPS: usize = 32;

/// Horizon elevation, radians, at any azimuth `az_rad` (topocentric ENU, clockwise from north):
/// linear between the two neighbouring bins, wrapping 719 → 0 at 360° (E05, N8).
pub fn horizon_el_at_rad(el_hor_rad: &[f64; N_AZ], az_rad: f64) -> Result<f64, GeometryError> {
    if !az_rad.is_finite() {
        return Err(GeometryError::BadInput("azimuth must be finite"));
    }
    #[allow(clippy::cast_precision_loss)] // N_AZ = 720, exact
    let pos = az_rad.rem_euclid(TAU) * (N_AZ as f64) / TAU;
    let k0 = pos.floor();
    let a = pos - k0;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // pos is in [0, 720], so floor fits a usize; the modulo maps an exact 720 back to bin 0
    let k = (k0 as usize) % N_AZ;
    Ok((1.0 - a) * el_hor_rad[k] + a * el_hor_rad[(k + 1) % N_AZ])
}

/// Geometric fraction of a disc of angular radius `radius_rad`, centred at `centre` (topocentric
/// ENU), that is above the horizon, in [0, 1]. 32 midpoint strips normalised by their own Σ 2w, so
/// a clear disc is exactly 1 and a hidden one exactly 0 (§1b, N2, T-1b-2). No limb darkening.
pub fn disc_fraction(
    el_hor_rad: &[f64; N_AZ],
    centre: AzEl,
    radius_rad: f64,
) -> Result<f64, GeometryError> {
    if !(radius_rad > 0.0 && radius_rad < FRAC_PI_2) || !centre.el_rad.is_finite() {
        return Err(GeometryError::BadInput(
            "disc radius must be in (0, 90°) and elevation finite",
        ));
    }
    // shortcut: one step with the disc at the zenith fails the whole timeline; fine at the south pole (Sun el < 7°), handle per step before any low-latitude site.
    if centre.el_rad.abs() + radius_rad >= FRAC_PI_2 {
        return Err(GeometryError::BadInput("disc reaches the zenith"));
    }
    let cos_el = centre.el_rad.cos();
    let (mut seen, mut total) = (0.0, 0.0);
    for i in 0..STRIPS {
        #[allow(clippy::cast_precision_loss)] // i < 32, exact
        let x = -1.0 + (2.0 * i as f64 + 1.0) / (STRIPS as f64);
        let w = radius_rad * (1.0 - x * x).sqrt();
        let g =
            horizon_el_at_rad(el_hor_rad, centre.az_rad + radius_rad * x / cos_el)? - centre.el_rad;
        seen += (w - g).clamp(0.0, 2.0 * w);
        total += 2.0 * w;
    }
    Ok(seen / total)
}

/// Earth elevation margin, radians: el_earth − el_hor(az_earth) (§1d). Contact is margin ≥ m_min.
pub fn earth_margin_rad(
    el_hor_rad: &[f64; N_AZ],
    earth_az_rad: f64,
    earth_el_rad: f64,
) -> Result<f64, GeometryError> {
    Ok(earth_el_rad - horizon_el_at_rad(el_hor_rad, earth_az_rad)?)
}

/// True when any fraction of the Earth's disc is above the horizon: the PGDA / Mazarico definition
/// used for V2-Earth (E12, W1b). Not comms contact, which is centre + margin (P14).
pub fn earth_visible_any_fraction(
    el_hor_rad: &[f64; N_AZ],
    earth: AzEl,
    earth_radius_rad: f64,
) -> Result<bool, GeometryError> {
    Ok(disc_fraction(el_hor_rad, earth, earth_radius_rad)? > 0.0)
}

/// One timeline (docs/API.md `timeline`), a column per field, `n` rows. Angles are degrees in the
/// site's topocentric ENU frame, times Unix s UTC.
#[derive(Debug, Clone, PartialEq)]
pub struct Timeline {
    pub t_unix_s: Vec<f64>,
    pub sun_az_deg: Vec<f64>,
    pub sun_el_deg: Vec<f64>,
    /// Sun-disc fraction above the horizon, 0–1. Used for power (P1).
    pub illumination: Vec<f64>,
    /// 1 when any part of the Sun's disc is visible (illumination > 0), else 0; what AVGVISIB averages (P1, W1).
    pub visibility: Vec<f64>,
    pub earth_az_deg: Vec<f64>,
    pub earth_el_deg: Vec<f64>,
    /// el_earth − el_hor(az_earth), degrees (§1d).
    pub earth_margin_deg: Vec<f64>,
    /// 1 when earth_margin_deg ≥ the minimum margin (comms contact, centre + margin, P14), else 0.
    pub contact: Vec<f64>,
    pub sun_dist_km: Vec<f64>,
}

impl Timeline {
    /// The API layout: 10 × n, column-major, in the field order above.
    pub fn to_api(&self) -> Vec<f64> {
        [
            &self.t_unix_s,
            &self.sun_az_deg,
            &self.sun_el_deg,
            &self.illumination,
            &self.visibility,
            &self.earth_az_deg,
            &self.earth_el_deg,
            &self.earth_margin_deg,
            &self.contact,
            &self.sun_dist_km,
        ]
        .iter()
        .flat_map(|c| c.iter().copied())
        .collect()
    }
}

/// Timeline at `site` for `count` steps from `start_unix_s` every `step_s` seconds (Unix s UTC),
/// against horizon `el_hor_rad`. Sun and Earth come from the interpolated table (§1c); ρ is
/// recomputed per step as asin(R_sun / d_sun) (§1b). `min_margin_rad` is the Earth contact margin.
pub fn timeline(
    eph: &Ephemeris,
    el_hor_rad: &[f64; N_AZ],
    site: &Site,
    start_unix_s: f64,
    step_s: f64,
    count: u32,
    min_margin_rad: f64,
) -> Result<Timeline, EphError> {
    if !min_margin_rad.is_finite() {
        return Err(EphError::BadInput("Earth margin must be finite"));
    }
    if !el_hor_rad.iter().all(|e| e.is_finite()) {
        return Err(EphError::BadInput("horizon must be finite"));
    }
    let track = sun_earth_track(eph, site, start_unix_s, step_s, count)?;
    let n = count as usize;
    // sun_earth_track columns (docs/API.md): t, sun az/el/radius/dist, earth az/el/radius/dist.
    let col = |c: usize| &track[c * n..(c + 1) * n];
    let (sun_az, sun_el, sun_r) = (col(1), col(2), col(3));
    let (earth_az, earth_el) = (col(5), col(6));
    let mut illumination = Vec::with_capacity(n);
    let mut earth_margin_deg = Vec::with_capacity(n);
    let mut contact = Vec::with_capacity(n);
    for i in 0..n {
        let centre = AzEl {
            az_rad: sun_az[i].to_radians(),
            el_rad: sun_el[i].to_radians(),
        };
        illumination.push(disc_fraction(el_hor_rad, centre, sun_r[i].to_radians())?);
        let m = earth_margin_rad(
            el_hor_rad,
            earth_az[i].to_radians(),
            earth_el[i].to_radians(),
        )?;
        earth_margin_deg.push(m.to_degrees());
        contact.push(f64::from(u8::from(m >= min_margin_rad)));
    }
    Ok(Timeline {
        t_unix_s: col(0).to_vec(),
        sun_az_deg: sun_az.to_vec(),
        sun_el_deg: sun_el.to_vec(),
        visibility: illumination
            .iter()
            .map(|&f| f64::from(u8::from(f > 0.0)))
            .collect(),
        illumination,
        earth_az_deg: earth_az.to_vec(),
        earth_el_deg: earth_el.to_vec(),
        contact,
        earth_margin_deg,
        sun_dist_km: col(4).to_vec(),
    })
}
