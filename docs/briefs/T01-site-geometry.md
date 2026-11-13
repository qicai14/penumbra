# T01 — Site geometry without data

Ticket: [PLAN-DRAFT T01](../PLAN-DRAFT.md). Owner: Qi Cai. 1 h. Blocked by T00. No DEM, no ephemeris, no dependencies.

## 1. Goal and done-criteria (from the ticket)

Convert between lat/lon, map x/y and MOON_ME unit vectors; build the local east-north-up frame at a site; return azimuth and elevation of any 3D point seen from the site.

- [ ] T-1a-2 (no-data part): forward then inverse projection returns lat/lon to 1e-9 rad. SP-01 (222.69°E, 89.45°S) maps to (−11.31, −12.26) km. The IM-2 site (84.7906°S, 29.1957°E) maps to (+77.1, +138.0) km.
- [ ] The vector form and the closed form of elevation agree; the curvature drop is 0.7 m at 1.6 km, 720 m at 50 km and 6.5 km at 150 km.
- [ ] Flat-sphere dip is −0.0869° at H = 2 m and −0.01944° at H = 0.1 m; everything is f64 (N7).
- [ ] At the pole the frame is north = +X, east = +Y and no output is NaN.
- [ ] P9: from a site at 85°S, 0°E a point toward the body's north pole has azimuth 0° and a point to the east has azimuth 90°.

## 2. Files and signatures

`crates/core/src/geom.rs` (new), `crates/core/src/error.rs` (from T00), `crates/core/tests/geom.rs` (new), `crates/wasm/src/lib.rs` (two exports).

Core, radians and metres:

```rust
pub const R_M: f64 = 1_737_400.0;

pub fn latlon_to_xy(lat: f64, lon: f64) -> [f64; 2];      // [x_m, y_m]
pub fn xy_to_latlon(x: f64, y: f64) -> [f64; 2];          // [lat, lon], lon in [0, 2π)
pub fn latlon_to_unit(lat: f64, lon: f64) -> [f64; 3];    // MOON_ME unit vector
pub fn xy_to_unit(x: f64, y: f64) -> [f64; 3];            // no trig
pub fn unit_to_xy(u: [f64; 3]) -> [f64; 2];               // no trig; runs inside the march

pub struct Frame { pub up: [f64; 3], pub east: [f64; 3], pub north: [f64; 3] }
pub fn frame_at(u: [f64; 3]) -> Frame;

/// Azimuth in [0, 2π) clockwise from north, and elevation, of point p seen from o.
/// o and p are MOON_ME vectors in the same unit (m or km).
pub fn az_el(frame: &Frame, o: [f64; 3], p: [f64; 3]) -> (f64, f64);

/// tan(elevation) of terrain at radius r_t seen from radius r_o across central angle theta.
pub fn tan_el(r_o: f64, r_t: f64, theta: f64) -> f64;
/// Flat-sphere horizon for an eye at height h above the sphere: −acos(R / (R + h)).
pub fn flat_dip(h: f64) -> f64;
```

WASM, frozen (ARCHITECTURE §3.2), degrees:

```rust
#[wasm_bindgen] pub fn latlon_to_xy(lat_deg: f64, lon_deg: f64) -> Vec<f64>;   // [x_m, y_m]
#[wasm_bindgen] pub fn xy_to_latlon(x_m: f64, y_m: f64) -> Vec<f64>;           // [lat_deg, lon_deg]
```

These two have no `Result` in the frozen API. A NaN in gives NaN out; the adapter's `validate()` catches it.

## 3. Algorithm

1. **Forward, lat/lon → map** (SCIENCE-SPEC §1a; E30, E31a, E31b): `r = 2R·tan((π/2 + lat)/2)`, `x = r·sin(lon)`, `y = r·cos(lon)`. Longitude 0 is along +y, 90°E along +x.
2. **Inverse, map → lat/lon**: `lat = −π/2 + 2·atan(hypot(x, y) / (2R))`, `lon = atan2(x, y)` wrapped into [0, 2π) with `rem_euclid`. At (0, 0) `atan2` gives 0, which is the convention.
3. **Map → unit vector**, no trig: `t = (x² + y²)/(4R²)`, `û = (y/R, x/R, −(1 − t)) / (1 + t)`.
4. **Unit vector → map**, no trig: `x = 2R·u_y/(1 − u_z)`, `y = 2R·u_x/(1 − u_z)`.
5. **Lat/lon → unit vector**: `(cos lat·cos lon, cos lat·sin lon, sin lat)`. It must equal step 1 followed by step 3 to 1e-12.
6. **Frame** (E51): `up = û`. `e = ẑ × û = (−u_y, u_x, 0)`. If `hypot(u_x, u_y) < 1e-12` (the pole): `north = (1, 0, 0)`, `east = (0, 1, 0)` for the south pole. Otherwise `east = e/|e|`, `north = û × east`.
7. **az/el** (E50, E51): `d = p − o`, `el = asin(d·up / |d|)`, `az = atan2(d·east, d·north)` wrapped into [0, 2π).
8. **Closed form** [D in the spec]: `tan_el = (r_t·cos θ − r_o)/(r_t·sin θ)`. Don't use the small-angle form: it is 0.0006° off in the test vector below.
9. **Dip** (E68): `−acos(R/(R + h))`.

## 4. Edge cases and pitfalls

- **P7 / N9, x/y swap.** SP-01 is almost on the map diagonal, so a swap moves it only 1.3 km. The IM-2 row catches a swap: it moves that site 86 km. Keep both rows.
- **P9, azimuth sense.** `atan2(east, north)`, not `atan2(north, east)`. Rows 9–11 pin it.
- **Pole.** `ẑ × û` is zero there. Row 12 checks no NaN and the +X/+Y convention. The frame is east × north = up; check it once.
- **N7.** No `f32` anywhere, including literals.
- `(1 − u_z)` is about 2 in the south, so step 4 is well conditioned. Don't add a guard for the north pole; nothing calls it there.
- Map distance is not ground distance (scale k = 1 + t, 1.0077 at 80°S). Nothing in this ticket measures distance on the map.

## 5. Test table

Tolerances are absolute. Angles in the table are degrees; convert in the test.

| # | Input | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | `latlon_to_xy(−89.45°, 222.69°)` (SP-01) | x = −11 308.2 m, y = −12 258.9 m | 0.5 m | [S] A1; spec §1a |
| 2 | `latlon_to_xy(−84.7906°, 29.1957°)` (IM-2) | x = +77 108.2 m, y = +137 993.3 m | 0.5 m | [S] A2; T-1a-2 |
| 3 | `latlon_to_xy(−85°, 0°)` | (0, 151 713.0) m | 0.1 m | [S] A3 |
| 4 | `latlon_to_xy(−80°, 90°)` | (304 005.6, 0) m | 0.1 m | [S] A4; the 20 m DEM's ±304 km edge (E135) |
| 5 | `xy_to_latlon(−11 582.5, −12 462.5)` (Barker RoI 1, E129, E130) | lat −89.438926°, lon 222.9040° | 1e-5° | [S] A5 |
| 6 | Round trip of row 2, and of 1000 random (lat ∈ [−90°, −60°], lon ∈ [0°, 360°)) | same lat/lon | 1e-9 rad; compare lon·cos(lat) | T-1a-2 |
| 7 | `latlon_to_unit(−85°, 0°)`; `frame_at` of it | û = (0.0871557, 0, −0.9961947); east = (0, 1, 0); north = (0.9961947, 0, 0.0871557) | 1e-7 | [R] review 1c |
| 8 | `xy_to_unit(latlon_to_xy(φ, λ))` vs `latlon_to_unit(φ, λ)`, and `unit_to_xy` back, at rows 1–4 | equal | 1e-12; 1e-6 m | steps 3–5 |
| 9 | Observer at (−85°, 0°) radius R; target on the sphere at (−84°, 0°) | az 0°, el −0.5° | 1e-9° | [S] B1; equal radii give el = −θ/2 |
| 10 | Same observer; target on the sphere at (−85°, 1°E) | az 90.49810°, el −0.0435773° | 1e-5° | [S] B2 |
| 11 | Same observer; targets `o + north` and `o + east` | az 0° and 90°, el 0° | 1e-9° | P9 |
| 12 | `frame_at(latlon_to_unit(−90°, 0°))` and `az_el` of `o + north`, `o + east` | north = (1, 0, 0), east = (0, 1, 0), up = (0, 0, −1); az 0° and 90°; nothing NaN | 1e-12 | spec §1a; V5 case 1 |
| 13 | Observer (−85°, 0°), radius 1737.4 km; body at (384 400, 0, 0) km | az 0.00000°, el 4.74192° | 1e-5° | [R] review 1c |
| 14 | Same observer; body 149 597 870.7 km from the Moon's centre toward lat −1°, lon 30°E | az 30.13857°, el 5.32713° | 1e-5° | [R] review 1c |
| 15 | `flat_dip(2.0)`, `flat_dip(0.1)` | −0.086937°, −0.019440° | 1e-6° | [S] B5; T-1a-1, N7 |
| 16 | `atan(tan_el(R + 2, R + 1000, 50 000/R))` | 0.31862° | 1e-5° | [R] review 1a |
| 17 | Row 16 again through `az_el`: observer (−85°, 45°E) at radius R + 2; target at radius R + 1000, 50 000 m along azimuth 37° (`û_s·cos θ + (north·cos az + east·sin az)·sin θ`) | el equals row 16; az 37° | 1e-9° | vector form = closed form |
| 18 | s²/(2R) at 1 600, 50 000, 150 000 m | 0.7367, 719.47, 6 475.2 m | 0.01 m | [S] B9 |

## 6. Red/green order for /tdd

1. Rows 1–4 red, then steps 1 and 5 green.
2. Rows 5–6, then step 2.
3. Rows 7–8, then steps 3, 4, 6.
4. Rows 9–12, then step 7 and the pole branch.
5. Rows 13–17, then steps 8–9. Row 18 is one assert on a formula in the test file.
6. Add the two WASM exports (degree conversion only). Run `bun run check` and the smoke test.

**Stop and escalate if** row 2 gives (+138.0, +77.1) km or a negative sign: the orientation is wrong, and every later ticket inherits it. Fix the formula, never the expected value. Also stop if rows 13–14 miss by more than 1e-4°: T02 and V1 can't pass on top of that.
