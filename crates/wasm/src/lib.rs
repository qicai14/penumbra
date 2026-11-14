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

/// The page's one engine: loaded ephemeris segments (near and far tiles land in later tickets).
#[wasm_bindgen]
#[derive(Default)]
pub struct Engine {
    eph: penumbra_core::eph::Ephemeris,
}

fn js_err(e: penumbra_core::eph::EphError) -> JsError {
    JsError::new(&format!("{}: {e:?}", e.code()))
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
}
