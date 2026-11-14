//! Local pit around the lander (SCIENCE-SPEC §1f "Local pit (A3)", V4 statement 3, P6).
//!
//! A lander at the centre of a crater of diameter D_c and depth d_c, panel at height H above the
//! floor, sees the rim at el_pit = atan((d_c − H) / (D_c/2)) at every azimuth. Angles are
//! topocentric ENU elevations in radians; lengths in metres. Terrain below the 20 m DEM pixel is
//! not in the DEM (P6), so this is a stated model, not a measurement.

use std::f64::consts::FRAC_PI_2;

use crate::geometry::GeometryError;
use crate::horizon::N_AZ;

/// Degraded-crater wall slope, degrees: an "11° slope from bottom to top of" the rim (E112).
pub const DEGRADED_CRATER_SLOPE_DEG: f64 = 11.0;
/// Most the Sun rises above the colatitude, degrees: 90° − |φ| + 1.54° (E104).
pub const SUN_MAX_ABOVE_COLATITUDE_DEG: f64 = 1.54;

fn diameter_ok(diameter_m: f64) -> Result<(), GeometryError> {
    if diameter_m.is_finite() && diameter_m > 0.0 {
        Ok(())
    } else {
        Err(GeometryError::BadInput(
            "pit diameter must be finite and > 0",
        ))
    }
}

/// ESA degraded-crater depth d_c = (D_c/2)·tan 11°, metres (E112). 1.94 m for D_c = 20 m.
pub fn degraded_crater_depth_m(diameter_m: f64) -> Result<f64, GeometryError> {
    diameter_ok(diameter_m)?;
    Ok(diameter_m / 2.0 * DEGRADED_CRATER_SLOPE_DEG.to_radians().tan())
}

/// Rim elevation seen from the panel, el_pit = atan((d_c − H)/(D_c/2)), radians, topocentric ENU.
/// Negative when the panel is above the rim.
pub fn pit_elevation_rad(
    depth_m: f64,
    diameter_m: f64,
    panel_height_m: f64,
) -> Result<f64, GeometryError> {
    diameter_ok(diameter_m)?;
    if !depth_m.is_finite() || !panel_height_m.is_finite() {
        return Err(GeometryError::BadInput(
            "pit depth and panel height must be finite",
        ));
    }
    Ok(((depth_m - panel_height_m) / (diameter_m / 2.0)).atan())
}

/// el_hor′[i] = max(el_hor[i], el_pit) for every azimuth bin, radians, topocentric ENU.
pub fn with_pit_rad(
    el_hor_rad: &[f64; N_AZ],
    el_pit_rad: f64,
) -> Result<[f64; N_AZ], GeometryError> {
    if !el_pit_rad.is_finite() {
        return Err(GeometryError::BadInput("pit elevation must be finite"));
    }
    Ok(el_hor_rad.map(|e| e.max(el_pit_rad)))
}

/// The Sun's highest possible centre elevation at selenographic latitude `lat_rad` (MOON_ME),
/// radians: 90° − |φ| + 1.54° (E104). 6.75° at IM-2's 84.7906°S.
pub fn max_sun_elevation_rad(lat_rad: f64) -> Result<f64, GeometryError> {
    if !lat_rad.is_finite() {
        return Err(GeometryError::BadInput("latitude must be finite"));
    }
    Ok(FRAC_PI_2 - lat_rad.abs() + SUN_MAX_ABOVE_COLATITUDE_DEG.to_radians())
}

/// Pit depth that hides the whole solar disc from a panel at `panel_height_m` above the floor:
/// d_c* = H + (D_c/2)·tan(el_sun + ρ), metres (§1f, V4 statement 3). `sun_el_rad` is the Sun
/// centre's topocentric ENU elevation, `sun_radius_rad` its angular radius ρ = asin(R_sun/d_sun).
pub fn depth_to_hide_m(
    diameter_m: f64,
    panel_height_m: f64,
    sun_el_rad: f64,
    sun_radius_rad: f64,
) -> Result<f64, GeometryError> {
    diameter_ok(diameter_m)?;
    if !panel_height_m.is_finite() || !sun_el_rad.is_finite() {
        return Err(GeometryError::BadInput(
            "panel height and Sun elevation must be finite",
        ));
    }
    if !(sun_radius_rad > 0.0 && sun_radius_rad < FRAC_PI_2) {
        return Err(GeometryError::BadInput("Sun radius must be in (0, 90°)"));
    }
    let limb_rad = sun_el_rad + sun_radius_rad;
    if limb_rad.abs() >= FRAC_PI_2 {
        return Err(GeometryError::BadInput(
            "Sun's upper limb at or past the zenith",
        ));
    }
    Ok(panel_height_m + diameter_m / 2.0 * limb_rad.tan())
}
