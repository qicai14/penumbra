# Penumbra API (frozen)

The `core` → WASM boundary, frozen at Hour 0 (T00, issue #1). This file is canonical; [ARCHITECTURE.md §3.1–§3.2](ARCHITECTURE.md) mirrors it. The adapter types, manifest and fixture shapes are in [§3.5](ARCHITECTURE.md).

**Change rule.** From the merge of the T00 PR, this file changes only in a PR approved by both Qi Cai and Charls, and every change bumps `api_version` in `crates/core` (`API_VERSION`), the fixture (`web/public/fixture/scenario.json`) and the manifest together. The adapter's `validate()` refuses a mismatch.

**Manifest.** `manifest.json` (written by `pipeline` from T06) carries `"api_version": 3` next to `format_version`.

## Conventions (frozen with the API)

- **Angles** are degrees at the boundary and radians inside `core`. **Azimuth** is clockwise from north, 0 ≤ az < 360. **Longitude** is east-positive; outputs are in [0, 360), inputs accept any value.
- **Time** is Unix seconds UTC, f64. **Lengths** are metres, except ephemeris distances in km. **Energy** is Wh, power W.
- **Heights**: `ground_m` is metres above the 1 737 400 m sphere; `mast_m` is the mast height H above the ground.
- Every number crossing the boundary is f64. There is no f32 anywhere in the API (N7).
- **Errors** are thrown as a JS `Error` whose message starts with a code and a colon: `NOT_LOADED`, `OUT_OF_COVERAGE`, `NO_DATA`, `OUT_OF_TABLE`, `BAD_INPUT`. A function never returns a default in place of an error (P5, P16).
- `api_version()` returns 3 (2 from T15, when the pit functions began to throw BAD_INPUT; 3 from #58, which dropped `latlon_to_xy` and `xy_to_latlon`: the web projects in TS, held to `core::geometry` within 1 mm by `web/src/lib/projection.test.ts`). The fixture and the manifest carry the same number; the adapter refuses a mismatch.

## Functions and array layouts

Rust signatures in `crates/wasm`. wasm-bindgen generates the `.d.ts` from them.

```rust
#[wasm_bindgen] pub fn api_version() -> u32;                                   // 3

#[wasm_bindgen] pub struct Engine { /* ephemeris segments, near tile, far tile */ }

#[wasm_bindgen]
impl Engine {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Engine;

    // Loading. Each call copies the array into the engine once; the engine owns it from then on.
    pub fn load_ephemeris(&mut self, t0_unix_s: f64, step_s: f64, data: Box<[f64]>) -> Result<(), JsError>;
    pub fn load_near(&mut self, data: Box<[i16]>, width: u32, height: u32, x0_m: f64, y0_m: f64,
                     pixel_m: f64, scale_m: f64, offset_m: f64, nodata: i16) -> Result<(), JsError>;
    pub fn load_far(&mut self, data: Box<[i16]>, width: u32, height: u32, x0_m: f64, y0_m: f64,
                    pixel_m: f64, scale_m: f64, offset_m: f64, nodata: i16) -> Result<(), JsError>;

    // Model. Arguments are scalars or small arrays; results are new arrays owned by JS.
    pub fn ground_height(&self, lat_deg: f64, lon_deg: f64) -> Result<f64, JsError>;
    pub fn sun_earth_track(&self, lat_deg: f64, lon_deg: f64, ground_m: f64, mast_m: f64,
                           start_unix_s: f64, step_s: f64, count: u32) -> Result<Vec<f64>, JsError>;
    pub fn horizon(&self, lat_deg: f64, lon_deg: f64, mast_m: f64) -> Result<Vec<f64>, JsError>;
    // Point-Sun mask over the near tile (T21), not illumination: 1 lit, 0 shadowed, row-major,
    // cols = ceil((width − 1) / stride), rows = ceil((height − 1) / stride).
    pub fn point_sun_mask(&self, t_unix_s: f64, mast_m: f64, stride: u32) -> Result<Vec<u8>, JsError>;
    pub fn timeline(&self, horizon: &[f64], lat_deg: f64, lon_deg: f64, ground_m: f64, mast_m: f64,
                    start_unix_s: f64, step_s: f64, count: u32, earth_margin_deg: f64)
                    -> Result<Vec<f64>, JsError>;
}

#[wasm_bindgen] pub fn survival(timeline: &[f64], preset: &[f64], landing_unix_s: f64)
                    -> Result<Vec<f64>, JsError>;

// Should tier. Names frozen now; built in T15 and T17.
#[wasm_bindgen] pub fn pit_horizon(horizon: &[f64], pit_diameter_m: f64, pit_depth_m: f64,
                                   panel_height_m: f64) -> Result<Vec<f64>, JsError>; // same layout as horizon; throws BAD_INPUT
#[wasm_bindgen] pub fn pit_depth_to_hide(pit_diameter_m: f64, panel_height_m: f64,
                                         sun_el_deg: f64, sun_radius_deg: f64) -> Result<f64, JsError>; // d_c* in m; throws BAD_INPUT
#[wasm_bindgen] pub fn window_bands(timeline: &[f64], horizon: &[f64], sigma_z_m: f64)
                    -> Result<Vec<f64>, JsError>; // 3 × k; throws BAD_INPUT
```

**Array layouts.** `n` is `count`. Column c of a series result is `result.subarray(c·n, (c+1)·n)`.

| Function | Length | Layout |
|---|---|---|
| `load_ephemeris` `data` | 6 × rows | row-major, as in the file: Sun x, y, z, Earth x, y, z in km, MOON_ME. Call once per segment. |
| `load_near`, `load_far` `data` | width × height | row-major int16, row 0 = largest y. `load_near` replaces the current near tile (one region at a time). `load_far` also finds h_max for the early exit, so the manifest doesn't carry it. |
| `sun_earth_track` | 9 × n | `t_unix_s`, `sun_az_deg`, `sun_el_deg`, `sun_radius_deg`, `sun_dist_km`, `earth_az_deg`, `earth_el_deg`, `earth_radius_deg`, `earth_dist_km` |
| `horizon` | 1 440 | `el_deg[720]` then `dist_m[720]`; index i is azimuth i × 0.5° |
| `timeline` | 10 × n | `t_unix_s`, `sun_az_deg`, `sun_el_deg`, `illumination` (0–1), `visibility` (0 or 1), `earth_az_deg`, `earth_el_deg`, `earth_margin_deg`, `contact` (0 or 1), `sun_dist_km` |
| `survival` `preset` | 12 | `capacity_wh`, `start_wh`, `min_wh`, `panel_area_m2`, `efficiency` (0–1), `panel_mode` (0 = Sun-tracking, 1 = fixed), fixed-panel normal `east`, `north`, `up`, `load_w`, then two reserved zeros |
| `survival` result | 8 + n | header: `operable_h`, `lethal_h` (hours after landing; NaN = no lethal shadow in the span), `longest_dark_h`, `longest_any_sun_h`, `sunlit_h`, `contact_h`, reserved 0, `n`; then `soc_wh[n]` |
| `window_bands` | 3 × k | `t_unix_s`, `kind` (+1 sunrise, −1 sunset), `half_width_min` |

Notes:
- `horizon` reads the ground height itself with the same code as `ground_height`, so the two can't disagree.
- `timeline` takes the horizon as an argument (in the layout `horizon` returns), not from hidden state. That lets `pit_horizon` feed it a raised horizon for the IM-2 replay.
- `survival` and the `pit_*` functions need no loaded data, so they are free functions and can be tested and used before any file loads. T09's core part can then land before the DEM does, as the plan wants.
- The lander preset's Earth margin and mast height are fields of the preset in the glossary, but they are consumed by `timeline` and `horizon`. The adapter passes them there.
- The step is taken from `t[1] − t[0]`, so `survival` has no step argument to get wrong.

**Ownership.**
1. One `Engine` per page, created at start-up and never freed.
2. Arrays passed in are copied at the call. JS may drop or reuse them afterwards.
3. Every result is a new `Float64Array` owned by JS. No result needs `free()`.
4. Nothing in `web/` may hold a view into `wasm.memory.buffer`. Growing WASM memory (which loading a tile does) detaches every such view without an error.

