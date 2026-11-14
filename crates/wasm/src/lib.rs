//! wasm-bindgen wrapper over penumbra-core. Logic stays in core.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    penumbra_core::add(a, b)
}

/// Frozen API version (ARCHITECTURE §3.1); the adapter refuses a fixture or manifest that differs.
#[wasm_bindgen]
pub fn api_version() -> u32 {
    penumbra_core::API_VERSION
}

use penumbra_core::dem::{DemError, DemTile, TileMeta};
use penumbra_core::geometry::LatLon;

/// The page's one engine: loaded ephemeris segments, the current region's near tile and the far tile.
#[wasm_bindgen]
#[derive(Default)]
pub struct Engine {
    eph: penumbra_core::eph::Ephemeris,
    near: Option<DemTile>,
    far: Option<DemTile>,
}

fn js_err(e: penumbra_core::eph::EphError) -> JsError {
    JsError::new(&format!("{}: {e:?}", e.code()))
}

fn dem_err(e: DemError) -> JsError {
    JsError::new(&format!("{}: {e:?}", e.code()))
}

/// MOON_ME latitude and east longitude in degrees → radians.
fn site_deg(lat_deg: f64, lon_deg: f64) -> LatLon {
    LatLon {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_deg.to_radians(),
    }
}

/// A tile from the manifest's eight numbers plus its row-major int16 counts (docs/API.md).
#[allow(clippy::too_many_arguments)] // mirrors the frozen load_near / load_far arguments
fn tile(
    data: Box<[i16]>,
    width: u32,
    height: u32,
    x0_m: f64,
    y0_m: f64,
    pixel_m: f64,
    scale_m: f64,
    offset_m: f64,
    nodata: i16,
) -> Result<DemTile, JsError> {
    let meta = TileMeta {
        width,
        height,
        x0_m,
        y0_m,
        pixel_m,
        scale_m,
        offset_m,
        nodata,
    };
    DemTile::new(meta, data.into_vec()).map_err(dem_err)
}

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine {
        Engine::default()
    }

    /// Add one segment: `t0_unix_s` Unix s UTC, rows every `step_s` s, `data` row-major
    /// Sun x, y, z, Earth x, y, z in km, MOON_ME (docs/API.md).
    pub fn load_ephemeris(
        &mut self,
        t0_unix_s: f64,
        step_s: f64,
        data: Box<[f64]>,
    ) -> Result<(), JsError> {
        self.eph
            .add(t0_unix_s, step_s, data.into_vec())
            .map_err(js_err)
    }

    /// 9 × count, column-major (docs/API.md). Site in degrees (MOON_ME latitude, east longitude),
    /// heights in metres, times Unix s UTC; az/el in the site's topocentric ENU frame, degrees.
    #[allow(clippy::too_many_arguments)] // frozen API signature (docs/API.md)
    pub fn sun_earth_track(
        &self,
        lat_deg: f64,
        lon_deg: f64,
        ground_m: f64,
        mast_m: f64,
        start_unix_s: f64,
        step_s: f64,
        count: u32,
    ) -> Result<Vec<f64>, JsError> {
        let site = penumbra_core::eph::Site {
            lat_rad: lat_deg.to_radians(),
            lon_east_rad: lon_deg.to_radians(),
            ground_m,
            mast_m,
        };
        penumbra_core::eph::sun_earth_track(&self.eph, &site, start_unix_s, step_s, count)
            .map_err(js_err)
    }

    /// Replace the near (20 m) tile: one region at a time. Map metres, south polar stereographic;
    /// `(x0_m, y0_m)` is the upper-left corner of pixel (0, 0); height = offset_m + scale_m × count.
    #[allow(clippy::too_many_arguments)] // frozen API signature (docs/API.md)
    pub fn load_near(
        &mut self,
        data: Box<[i16]>,
        width: u32,
        height: u32,
        x0_m: f64,
        y0_m: f64,
        pixel_m: f64,
        scale_m: f64,
        offset_m: f64,
        nodata: i16,
    ) -> Result<(), JsError> {
        self.near = Some(tile(
            data, width, height, x0_m, y0_m, pixel_m, scale_m, offset_m, nodata,
        )?);
        Ok(())
    }

    /// Replace the far (240 m) tile; its h_max for the early exit is found here (docs/API.md).
    #[allow(clippy::too_many_arguments)] // frozen API signature (docs/API.md)
    pub fn load_far(
        &mut self,
        data: Box<[i16]>,
        width: u32,
        height: u32,
        x0_m: f64,
        y0_m: f64,
        pixel_m: f64,
        scale_m: f64,
        offset_m: f64,
        nodata: i16,
    ) -> Result<(), JsError> {
        self.far = Some(tile(
            data, width, height, x0_m, y0_m, pixel_m, scale_m, offset_m, nodata,
        )?);
        Ok(())
    }

    /// Ground height at the site (MOON_ME degrees, east longitude), metres above the 1 737 400 m sphere.
    pub fn ground_height(&self, lat_deg: f64, lon_deg: f64) -> Result<f64, JsError> {
        let near = self
            .near
            .as_ref()
            .ok_or_else(|| JsError::new("NOT_LOADED: near tile"))?;
        penumbra_core::horizon::ground_height_m(near, site_deg(lat_deg, lon_deg)).map_err(dem_err)
    }

    /// 1 440 values: `el_deg[720]` (topocentric ENU) then `dist_m[720]`; bin i is azimuth i × 0.5°
    /// clockwise from north (docs/API.md). `mast_m` is H above the near tile's ground.
    pub fn horizon(&self, lat_deg: f64, lon_deg: f64, mast_m: f64) -> Result<Vec<f64>, JsError> {
        let near = self
            .near
            .as_ref()
            .ok_or_else(|| JsError::new("NOT_LOADED: near tile"))?;
        let far = self
            .far
            .as_ref()
            .ok_or_else(|| JsError::new("NOT_LOADED: far tile"))?;
        penumbra_core::horizon::horizon(
            near,
            far,
            site_deg(lat_deg, lon_deg),
            mast_m,
            &penumbra_core::horizon::HorizonOptions::default(),
        )
        .map(|h| h.to_api())
        .map_err(dem_err)
    }

    /// 10 × count, column-major (docs/API.md): t_unix_s, sun_az_deg, sun_el_deg, illumination,
    /// visibility, earth_az_deg, earth_el_deg, earth_margin_deg, contact, sun_dist_km. `horizon` is the
    /// 1 440-value layout `horizon` returns (topocentric ENU degrees, then metres); site in MOON_ME
    /// degrees, heights metres, times Unix s UTC. Contact is margin ≥ `earth_margin_deg` (§1d).
    #[allow(clippy::too_many_arguments)] // frozen API signature (docs/API.md)
    pub fn timeline(
        &self,
        horizon: &[f64],
        lat_deg: f64,
        lon_deg: f64,
        ground_m: f64,
        mast_m: f64,
        start_unix_s: f64,
        step_s: f64,
        count: u32,
        earth_margin_deg: f64,
    ) -> Result<Vec<f64>, JsError> {
        let n_az = penumbra_core::horizon::N_AZ;
        if horizon.len() != 2 * n_az {
            return Err(JsError::new("BAD_INPUT: horizon must have 1440 values"));
        }
        let el_hor_rad: [f64; penumbra_core::horizon::N_AZ] =
            std::array::from_fn(|i| horizon[i].to_radians());
        let site = penumbra_core::eph::Site {
            lat_rad: lat_deg.to_radians(),
            lon_east_rad: lon_deg.to_radians(),
            ground_m,
            mast_m,
        };
        penumbra_core::timeline::timeline(
            &self.eph,
            &el_hor_rad,
            &site,
            start_unix_s,
            step_s,
            count,
            earth_margin_deg.to_radians(),
        )
        .map(|t| t.to_api())
        .map_err(js_err)
    }
}

/// One survival run (docs/API.md). `timeline` is the 10 × n layout `Engine.timeline` returns (Unix
/// s UTC, topocentric ENU degrees); `preset` is the 12-value layout: capacity_wh, start_wh, min_wh,
/// panel_area_m2, efficiency (0–1), panel_mode (0 = Sun-tracking, 1 = fixed), fixed normal east,
/// north, up (ENU), load_w, two reserved zeros. `landing_unix_s` must be the timeline's first step.
/// Result: operable_h, lethal_h (hours after landing; NaN = no lethal shadow in the span),
/// longest_dark_h, longest_any_sun_h, sunlit_h, contact_h, 0, m; then soc_wh[m], where m is the
/// number of steps simulated (fewer than n when the run stops at a lethal shadow).
#[wasm_bindgen]
pub fn survival(
    timeline: &[f64],
    preset: &[f64],
    landing_unix_s: f64,
) -> Result<Vec<f64>, JsError> {
    use penumbra_core::power::{Lethal, Panel, Preset};
    if !timeline.len().is_multiple_of(10) || preset.len() != 12 {
        return Err(JsError::new(
            "BAD_INPUT: timeline must be 10 × n and preset 12 values",
        ));
    }
    let n = timeline.len() / 10;
    let col = |c: usize| timeline[c * n..(c + 1) * n].to_vec();
    let tl = penumbra_core::timeline::Timeline {
        t_unix_s: col(0),
        sun_az_deg: col(1),
        sun_el_deg: col(2),
        illumination: col(3),
        visibility: col(4),
        earth_az_deg: col(5),
        earth_el_deg: col(6),
        earth_margin_deg: col(7),
        contact: col(8),
        sun_dist_km: col(9),
    };
    #[allow(clippy::float_cmp)] // the mode and reserved slots are exact small integers
    let panel = match preset[5] {
        0.0 => Panel::SunTracking,
        1.0 => Panel::Fixed {
            east: preset[6],
            north: preset[7],
            up: preset[8],
        },
        _ => return Err(JsError::new("BAD_INPUT: panel_mode must be 0 or 1")),
    };
    #[allow(clippy::float_cmp)]
    if preset[10] != 0.0 || preset[11] != 0.0 {
        return Err(JsError::new("BAD_INPUT: preset reserved values must be 0"));
    }
    let p = Preset {
        capacity_wh: preset[0],
        start_wh: preset[1],
        min_wh: preset[2],
        panel_area_m2: preset[3],
        efficiency: preset[4],
        panel,
        load_w: preset[9],
    };
    let s = penumbra_core::power::survival(&tl, &p, landing_unix_s).map_err(js_err)?;
    let lethal_h = match s.lethal {
        Lethal::At {
            after_landing_h, ..
        } => after_landing_h,
        Lethal::NoneInSpan => f64::NAN,
    };
    #[allow(clippy::cast_precision_loss)] // step counts are far below 2^52
    let m = s.soc_wh.len() as f64;
    let mut out = vec![
        s.operable_h,
        lethal_h,
        s.longest_dark_h,
        s.longest_any_sun_h,
        s.sunlit_h,
        s.contact_h,
        0.0,
        m,
    ];
    out.extend(s.soc_wh);
    Ok(out)
}

fn geo_err(e: penumbra_core::geometry::GeometryError) -> JsError {
    let penumbra_core::geometry::GeometryError::BadInput(why) = e;
    JsError::new(&format!("BAD_INPUT: {why}"))
}

/// `horizon` (the 1 440-value layout `Engine.horizon` returns: topocentric ENU el_deg[720] then
/// dist_m[720]) raised to the rim of a pit around the lander: el′ = max(el, el_pit) with
/// el_pit = atan((d_c − H)/(D_c/2)) (SCIENCE-SPEC §1f A3). A raised bin's distance becomes the rim
/// distance D_c/2. Throws BAD_INPUT instead of returning NaN (docs/API.md errors rule).
#[wasm_bindgen]
pub fn pit_horizon(
    horizon: &[f64],
    pit_diameter_m: f64,
    pit_depth_m: f64,
    panel_height_m: f64,
) -> Result<Vec<f64>, JsError> {
    use penumbra_core::horizon::N_AZ;
    if horizon.len() != 2 * N_AZ {
        return Err(JsError::new("BAD_INPUT: horizon must have 1440 values"));
    }
    if horizon.iter().any(|v| !v.is_finite()) {
        return Err(JsError::new("BAD_INPUT: horizon values must be finite"));
    }
    let el_pit_rad =
        penumbra_core::pit::pit_elevation_rad(pit_depth_m, pit_diameter_m, panel_height_m)
            .map_err(geo_err)?;
    let el_hor_rad: [f64; N_AZ] = std::array::from_fn(|i| horizon[i].to_radians());
    let raised = penumbra_core::pit::with_pit_rad(&el_hor_rad, el_pit_rad).map_err(geo_err)?;
    let mut out: Vec<f64> = raised.iter().map(|e| e.to_degrees()).collect();
    out.extend((0..N_AZ).map(|i| {
        if raised[i] > el_hor_rad[i] {
            pit_diameter_m / 2.0
        } else {
            horizon[N_AZ + i]
        }
    }));
    Ok(out)
}

/// Pit depth d_c* = H + (D_c/2)·tan(el_sun + ρ), metres, that hides the whole solar disc from a
/// panel at `panel_height_m` above the pit floor (SCIENCE-SPEC §1f, V4 statement 3). `sun_el_deg`
/// and `sun_radius_deg` are the topocentric ENU Sun-centre elevation and angular radius, degrees,
/// as `sun_earth_track` returns them.
#[wasm_bindgen]
pub fn pit_depth_to_hide(
    pit_diameter_m: f64,
    panel_height_m: f64,
    sun_el_deg: f64,
    sun_radius_deg: f64,
) -> Result<f64, JsError> {
    penumbra_core::pit::depth_to_hide_m(
        pit_diameter_m,
        panel_height_m,
        sun_el_deg.to_radians(),
        sun_radius_deg.to_radians(),
    )
    .map_err(geo_err)
}
