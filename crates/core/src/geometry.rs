//! Site geometry without data (SCIENCE-SPEC §1a): south polar stereographic map, MOON_ME unit
//! vectors, the local east-north-up frame and azimuth/elevation. All f64 (N7), radians inside.

use std::f64::consts::{FRAC_PI_2, TAU};

/// Radius of the DEM reference sphere and of the stereographic projection, metres (E30, E31a).
pub const MOON_RADIUS_M: f64 = 1_737_400.0;

/// Why a geometry input was refused. Maps to the API's `BAD_INPUT` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    /// The input is not finite, or the projection cannot represent it (the north pole).
    BadInput(&'static str),
}

/// Selenographic position in MOON_ME: latitude (south negative) and east-positive longitude, radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatLon {
    pub lat_rad: f64,
    pub lon_east_rad: f64,
}

/// South polar stereographic map coordinates, metres, origin at the south pole.
/// Longitude 0 lies along +y and 90°E along +x (§1a, checked against SP-01).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MapXy {
    pub x_m: f64,
    pub y_m: f64,
}

/// Forward projection from (lat, east lon): r = 2R·tan((90° + φ)/2), x = r·sin λ, y = r·cos λ (§1a, E31b).
pub fn latlon_to_xy(site: LatLon) -> Result<MapXy, GeometryError> {
    if !site.lat_rad.is_finite() || !site.lon_east_rad.is_finite() {
        return Err(GeometryError::BadInput("lat/lon must be finite"));
    }
    if !(-FRAC_PI_2..FRAC_PI_2).contains(&site.lat_rad) {
        return Err(GeometryError::BadInput(
            "lat must be in [-90°, 90°): the south polar map cannot show the north pole",
        ));
    }
    let r_m = 2.0 * MOON_RADIUS_M * ((FRAC_PI_2 + site.lat_rad) / 2.0).tan();
    let (sin_lon, cos_lon) = site.lon_east_rad.sin_cos();
    Ok(MapXy {
        x_m: r_m * sin_lon,
        y_m: r_m * cos_lon,
    })
}

/// Inverse projection: colatitude from the south pole c = 2·arctan(r / 2R) (E31b), lon = atan2(x, y).
/// Angle into [0, 2π): `rem_euclid` alone rounds a tiny negative angle up to exactly 2π.
fn wrap_tau(a: f64) -> f64 {
    let w = a.rem_euclid(TAU);
    if w >= TAU { 0.0 } else { w }
}

/// Longitude is returned in [0, 2π); at the pole itself it is 0.
pub fn xy_to_latlon(xy: MapXy) -> Result<LatLon, GeometryError> {
    check_finite_xy(xy)?;
    let r_m = xy.x_m.hypot(xy.y_m);
    Ok(LatLon {
        lat_rad: 2.0 * (r_m / (2.0 * MOON_RADIUS_M)).atan() - FRAC_PI_2,
        lon_east_rad: wrap_tau(xy.x_m.atan2(xy.y_m)),
    })
}

/// A 3-vector in MOON_ME (+X toward lon 0 on the equator, +Z the body north pole). Unit vectors
/// are dimensionless; positions are metres from the Moon's centre (`_m` on the variable).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    pub fn plus(self, o: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + o.x,
            y: self.y + o.y,
            z: self.z + o.z,
        }
    }
    pub fn minus(self, o: Vec3) -> Vec3 {
        self.plus(o.scale(-1.0))
    }
    pub fn scale(self, k: f64) -> Vec3 {
        Vec3 {
            x: self.x * k,
            y: self.y * k,
            z: self.z * k,
        }
    }
    pub fn dot(self, o: Vec3) -> f64 {
        self.x * o.x + self.y * o.y + self.z * o.z
    }
    pub fn cross(self, o: Vec3) -> Vec3 {
        Vec3 {
            x: self.y * o.z - self.z * o.y,
            y: self.z * o.x - self.x * o.z,
            z: self.x * o.y - self.y * o.x,
        }
    }
    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
    fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

/// MOON_ME unit vector of a selenographic position: (cos φ cos λ, cos φ sin λ, sin φ).
pub fn latlon_to_unit(site: LatLon) -> Result<Vec3, GeometryError> {
    if !site.lat_rad.is_finite() || !site.lon_east_rad.is_finite() {
        return Err(GeometryError::BadInput("lat/lon must be finite"));
    }
    let (sin_lat, cos_lat) = site.lat_rad.sin_cos();
    let (sin_lon, cos_lon) = site.lon_east_rad.sin_cos();
    Ok(Vec3 {
        x: cos_lat * cos_lon,
        y: cos_lat * sin_lon,
        z: sin_lat,
    })
}

/// Trig-free inverse projection to a MOON_ME unit vector (§1a):
/// t = (x² + y²)/(4R²), û = (y/R, x/R, −(1 − t))/(1 + t).
pub fn xy_to_unit(xy: MapXy) -> Result<Vec3, GeometryError> {
    check_finite_xy(xy)?;
    let t = (xy.x_m * xy.x_m + xy.y_m * xy.y_m) / (4.0 * MOON_RADIUS_M * MOON_RADIUS_M);
    Ok(Vec3 {
        x: xy.y_m / MOON_RADIUS_M,
        y: xy.x_m / MOON_RADIUS_M,
        z: -(1.0 - t),
    }
    .scale(1.0 / (1.0 + t)))
}

/// Trig-free forward projection of a MOON_ME unit vector (§1a): x = 2R·u_y/(1 − u_z), y = 2R·u_x/(1 − u_z).
/// The body north pole (u_z = 1) has no map position and is refused.
pub fn unit_to_xy(u: Vec3) -> Result<MapXy, GeometryError> {
    if !u.is_finite() {
        return Err(GeometryError::BadInput("unit vector must be finite"));
    }
    let denom = 1.0 - u.z;
    if denom <= f64::EPSILON {
        return Err(GeometryError::BadInput(
            "the body north pole has no south polar map position",
        ));
    }
    Ok(MapXy {
        x_m: 2.0 * MOON_RADIUS_M * u.y / denom,
        y_m: 2.0 * MOON_RADIUS_M * u.x / denom,
    })
}

/// Azimuth and elevation in the topocentric ENU frame of a site, radians.
/// `az_rad` is clockwise from north in [0, 2π) (E50); `el_rad` is above the local horizontal plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AzEl {
    pub az_rad: f64,
    pub el_rad: f64,
}

/// Distance from the body axis below which a site counts as on the pole, as |ẑ × û| (= cos lat).
/// 1e-12 is about 2 µm of ground; inside it ẑ × û has no usable direction.
const POLE_AXIS_DISTANCE: f64 = 1e-12;

/// Topocentric east-north-up frame at a site, as MOON_ME unit vectors (§1a, spherical normal E51).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnuFrame {
    pub east: Vec3,
    pub north: Vec3,
    pub up: Vec3,
}

impl EnuFrame {
    /// Frame at the site whose MOON_ME unit vector is `up`: east = ẑ × up / |ẑ × up|, north = up × east.
    /// On the pole itself east = +Y, which gives north = +X (toward lon 0) at the south pole (§1a, V5).
    pub fn at(up: Vec3) -> Result<EnuFrame, GeometryError> {
        if !up.is_finite() || (up.norm() - 1.0).abs() > 1e-9 {
            return Err(GeometryError::BadInput("site must be a finite unit vector"));
        }
        let z_cross_up = Vec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
        .cross(up);
        let axis_distance = z_cross_up.norm();
        let east = if axis_distance < POLE_AXIS_DISTANCE {
            Vec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            }
        } else {
            z_cross_up.scale(1.0 / axis_distance)
        };
        Ok(EnuFrame {
            east,
            north: up.cross(east),
            up,
        })
    }

    /// Azimuth and elevation of `target_m` seen from `observer_m`, both MOON_ME positions in metres (§1a):
    /// d = p − o, el = asin(d·up/|d|), az = atan2(d·east, d·north).
    pub fn az_el(&self, observer_m: Vec3, target_m: Vec3) -> Result<AzEl, GeometryError> {
        let d_m = target_m.minus(observer_m);
        let range_m = d_m.norm();
        if !range_m.is_finite() || range_m <= 0.0 {
            return Err(GeometryError::BadInput(
                "target must be finite and apart from the observer",
            ));
        }
        Ok(AzEl {
            az_rad: wrap_tau(d_m.dot(self.east).atan2(d_m.dot(self.north))),
            el_rad: (d_m.dot(self.up) / range_m).clamp(-1.0, 1.0).asin(),
        })
    }
}

/// Closed-form elevation along one azimuth (§1a): tan(el) = (r_t·cos θ − r_o)/(r_t·sin θ), θ = s/R.
/// Radii are metres from the Moon's centre (observer r_o = R + h_s + H, target r_t = R + h);
/// `ground_distance_m` is s, measured on the R sphere, and must be positive.
pub fn elevation_closed_form_rad(
    observer_radius_m: f64,
    target_radius_m: f64,
    ground_distance_m: f64,
) -> Result<f64, GeometryError> {
    if !(observer_radius_m.is_finite() && target_radius_m.is_finite() && ground_distance_m > 0.0)
        || !ground_distance_m.is_finite()
    {
        return Err(GeometryError::BadInput(
            "radii must be finite and ground distance positive",
        ));
    }
    let theta = ground_distance_m / MOON_RADIUS_M;
    let (sin_t, cos_t) = theta.sin_cos();
    Ok((target_radius_m * cos_t - observer_radius_m).atan2(target_radius_m * sin_t))
}

/// Curvature drop s²/(2R) in metres at ground distance `ground_distance_m` (§1a).
pub fn curvature_drop_m(ground_distance_m: f64) -> f64 {
    ground_distance_m * ground_distance_m / (2.0 * MOON_RADIUS_M)
}

/// Horizon dip of a bare sphere seen from mast height `mast_m` ≥ 0 metres: −acos(R/(R + H)) (T-1a-1).
pub fn flat_sphere_dip_rad(mast_m: f64) -> Result<f64, GeometryError> {
    if !mast_m.is_finite() || mast_m < 0.0 {
        return Err(GeometryError::BadInput(
            "mast height must be finite and ≥ 0",
        ));
    }
    Ok(-(MOON_RADIUS_M / (MOON_RADIUS_M + mast_m)).acos())
}

fn check_finite_xy(xy: MapXy) -> Result<(), GeometryError> {
    if xy.x_m.is_finite() && xy.y_m.is_finite() {
        Ok(())
    } else {
        Err(GeometryError::BadInput("map x/y must be finite"))
    }
}
