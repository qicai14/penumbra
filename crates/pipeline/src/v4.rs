//! V4 statement 3 (SCIENCE-SPEC V4 details, §1f A3): the IM-2 site at its assumed touchdown time,
//! from regional terrain (shipped Mons Mouton + far tiles) and the shipped replay segment, with the
//! pit depth d_c* that would hide the whole solar disc from a panel at each height H.
//! Wording follows N10: "consistent with reports", never a statement of what happened.

use crate::dem::ShippedTiles;
use crate::eph::{REPLAY_IM2_FILE_PREFIX, Result, read_shipped_table};
use penumbra_core::eph::{
    Ephemeris, IM2_TOUCHDOWN_UNIX_S, REPLAY_IM2_STEP_S, REPLAY_IM2_T0_UNIX_S, Site, sun_earth_track,
};
use penumbra_core::geometry::{AzEl, LatLon};
use penumbra_core::horizon::{HorizonOptions, ground_height_m, horizon};
use penumbra_core::pit::{
    degraded_crater_depth_m, depth_to_hide_m, pit_elevation_rad, with_pit_rad,
};
use penumbra_core::timeline::{disc_fraction, horizon_el_at_rad};
use std::path::Path;

/// IM-2 site, MOON_ME degrees (U8: LROC featured image 1408).
const IM2_LAT_DEG: f64 = -84.7906;
const IM2_LON_EAST_DEG: f64 = 29.1957;
/// IM-2's crater, about 20 m wide (W6).
const PIT_DIAMETER_M: f64 = 20.0;
/// Panel heights above the floor; the spec gives no single one, so one row each (§1f).
const PANEL_HEIGHTS_M: [f64; 4] = [0.0, 1.0, 1.44, 2.0];

/// Statement 3 as JSON for the validator to copy into scoreboard.json. `data_dir` is
/// `web/public/data` (manifest, tiles and `eph-replay-im2.*.bin`).
pub fn im2_statement3(data_dir: &Path) -> Result<serde_json::Value> {
    let err = |e: &dyn std::fmt::Debug| format!("{e:?}");
    let tiles = ShippedTiles::load(data_dir)?;
    let near = tiles.near("mons-mouton")?;
    let mut eph = Ephemeris::default();
    eph.add(
        REPLAY_IM2_T0_UNIX_S,
        REPLAY_IM2_STEP_S,
        read_shipped_table(data_dir, REPLAY_IM2_FILE_PREFIX)?,
    )
    .map_err(|e| err(&e))?;
    let site = LatLon {
        lat_rad: IM2_LAT_DEG.to_radians(),
        lon_east_rad: IM2_LON_EAST_DEG.to_radians(),
    };
    let ground_m = ground_height_m(near, site).map_err(|e| err(&e))?;
    let d_c_m = degraded_crater_depth_m(PIT_DIAMETER_M).map_err(|e| err(&e))?;

    let mut rows = Vec::new();
    for h_m in PANEL_HEIGHTS_M {
        let hor = horizon(near, &tiles.far, site, h_m, &HorizonOptions::default())
            .map_err(|e| err(&e))?;
        let track = sun_earth_track(
            &eph,
            &Site {
                lat_rad: site.lat_rad,
                lon_east_rad: site.lon_east_rad,
                ground_m,
                mast_m: h_m,
            },
            IM2_TOUCHDOWN_UNIX_S,
            REPLAY_IM2_STEP_S,
            1,
        )
        .map_err(|e| err(&e))?;
        // sun_earth_track columns (docs/API.md), count 1: t, sun az, el, radius, dist, ...
        let (sun_az_deg, sun_el_deg, sun_radius_deg) = (track[1], track[2], track[3]);
        let sun = AzEl {
            az_rad: sun_az_deg.to_radians(),
            el_rad: sun_el_deg.to_radians(),
        };
        let rho_rad = sun_radius_deg.to_radians();
        let el_pit_rad = pit_elevation_rad(d_c_m, PIT_DIAMETER_M, h_m).map_err(|e| err(&e))?;
        let pitted = with_pit_rad(&hor.el_rad, el_pit_rad).map_err(|e| err(&e))?;
        let d_star_m =
            depth_to_hide_m(PIT_DIAMETER_M, h_m, sun.el_rad, rho_rad).map_err(|e| err(&e))?;
        rows.push(serde_json::json!({
            "panel_height_m": h_m,
            "sun_az_deg": sun_az_deg,
            "sun_el_deg": sun_el_deg,
            "sun_radius_deg": sun_radius_deg,
            "regional_horizon_el_at_sun_az_deg":
                horizon_el_at_rad(&hor.el_rad, sun.az_rad).map_err(|e| err(&e))?.to_degrees(),
            "illumination_regional": disc_fraction(&hor.el_rad, sun, rho_rad).map_err(|e| err(&e))?,
            "el_pit_deg": el_pit_rad.to_degrees(),
            "illumination_with_pit": disc_fraction(&pitted, sun, rho_rad).map_err(|e| err(&e))?,
            "depth_to_hide_m": d_star_m,
            "degraded_crater_exceeds_depth_to_hide": d_c_m > d_star_m,
        }));
    }
    Ok(serde_json::json!({
        "statement": 3,
        "site": {"lat_deg": IM2_LAT_DEG, "lon_east_deg": IM2_LON_EAST_DEG, "ground_m": ground_m},
        "touchdown_utc": "2025-03-06T17:28:00Z",
        "touchdown_unix_s": IM2_TOUCHDOWN_UNIX_S,
        "touchdown_status": "assumed (U8)",
        "pit_diameter_m": PIT_DIAMETER_M,
        "degraded_crater_depth_m": d_c_m,
        "formula": "d_c* = H + (D_c/2)·tan(el_sun + ρ), ρ = asin(R_sun / d_sun)",
        "rows": rows,
        "wording": "Model output at an assumed touchdown time. Where the 11° degraded-crater depth exceeds d_c*, a pit shadow on the panel is consistent with reports (E73, E74). Terrain below the 20 m DEM pixel is not in the DEM (P6); the pit is a stated model.",
    }))
}
