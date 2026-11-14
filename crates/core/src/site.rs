//! The browser's chain for one site, in one place so the native grid (#25) and the WASM bindings
//! cannot drift: ground height, three-tier horizon through its API degrees, timeline, survival.
//! The browser's horizon crosses the API in degrees (`Horizon::to_api`) and comes back in radians,
//! so this chain takes the same round trip; skipping it could move a sunrise by one step.

use crate::dem::{DemError, DemTile};
use crate::eph::{EphError, Ephemeris, Site};
use crate::geometry::LatLon;
use crate::horizon::{HorizonOptions, N_AZ, ground_height_m, horizon};
use crate::power::{Preset, Survival, survival};
use crate::timeline::timeline;

/// MOON_ME latitude and east longitude, degrees, as the API takes them → radians.
pub fn latlon_from_deg(lat_deg: f64, lon_east_deg: f64) -> LatLon {
    LatLon {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_east_deg.to_radians(),
    }
}

/// The topocentric ENU horizon elevations, radians, from the 1 440-value API layout
/// (`el_deg[720]` then `dist_m[720]`, docs/API.md). BAD_INPUT for any other length.
pub fn el_hor_rad_from_api(api: &[f64]) -> Result<[f64; N_AZ], EphError> {
    if api.len() != 2 * N_AZ {
        return Err(EphError::BadInput("horizon must have 1440 values"));
    }
    Ok(std::array::from_fn(|i| api[i].to_radians()))
}

/// A site ready for timelines: the API `Site` (ground from the near tile, `mast_m` above it) and
/// its horizon, topocentric ENU radians, after the API degree round trip.
#[derive(Debug, Clone)]
pub struct SiteHorizon {
    pub site: Site,
    pub el_hor_rad: [f64; N_AZ],
}

/// What `Engine.ground_height` then `Engine.horizon` give the browser for `(lat_deg, lon_east_deg)`
/// (MOON_ME degrees) and `mast_m`: near, middle and far tiers, default options.
pub fn site_horizon(
    near: &DemTile,
    mid: &DemTile,
    far: &DemTile,
    lat_deg: f64,
    lon_east_deg: f64,
    mast_m: f64,
) -> Result<SiteHorizon, DemError> {
    let ll = latlon_from_deg(lat_deg, lon_east_deg);
    let ground_m = ground_height_m(near, ll)?;
    let api = horizon(near, mid, far, ll, mast_m, &HorizonOptions::default())?.to_api();
    let el_hor_rad = el_hor_rad_from_api(&api)
        .map_err(|_| DemError::BadInput("horizon layout is not 1440 values"))?;
    Ok(SiteHorizon {
        site: Site {
            lat_rad: ll.lat_rad,
            lon_east_rad: ll.lon_east_rad,
            ground_m,
            mast_m,
        },
        el_hor_rad,
    })
}

/// What `Engine.timeline` then `survival` give the browser: `count` steps of `step_s` from
/// `landing_unix_s` (Unix s UTC; landing is the first step), contact at margin ≥
/// `earth_margin_deg` (§1d).
pub fn site_survival(
    eph: &Ephemeris,
    sh: &SiteHorizon,
    landing_unix_s: f64,
    step_s: f64,
    count: u32,
    earth_margin_deg: f64,
    preset: &Preset,
) -> Result<Survival, EphError> {
    let tl = timeline(
        eph,
        &sh.el_hor_rad,
        &sh.site,
        landing_unix_s,
        step_s,
        count,
        earth_margin_deg.to_radians(),
    )?;
    survival(&tl, preset, landing_unix_s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_hor_rad_from_api_refuses_a_layout_that_is_not_1440_values() {
        assert_eq!(
            el_hor_rad_from_api(&[0.0; 720]).map_err(|e| e.code()),
            Err("BAD_INPUT")
        );
    }

    #[test]
    fn el_hor_rad_from_api_reads_bin_i_as_degrees_from_the_first_half() {
        let mut api = vec![0.0; 1440];
        api[1] = 90.0; // bin 1 elevation, degrees
        api[721] = 5000.0; // bin 1 distance, metres: must not be read as an elevation
        let el = el_hor_rad_from_api(&api).map_err(|e| e.code());
        assert_eq!(el.map(|e| e[1]), Ok(std::f64::consts::FRAC_PI_2));
    }
}
