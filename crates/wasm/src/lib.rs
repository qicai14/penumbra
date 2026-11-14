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
}
