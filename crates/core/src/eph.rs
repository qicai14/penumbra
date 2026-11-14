//! Ephemeris lookup and the Sun/Earth topocentric track (SCIENCE-SPEC §1c steps 3–4, ARCHITECTURE §2.4).
//!
//! The table rows are apparent (light time + stellar aberration) Moon-centred positions in km,
//! MOON_ME axes, on a UTC grid of Unix seconds (ADR 0002). Lookup is linear, not renormalised.

use crate::geometry::{EnuFrame, GeometryError, LatLon, MOON_RADIUS_M, Vec3, latlon_to_unit};

/// Hourly main segment shipped in `web/public/data/eph-main.<hash6>.bin`: 2026-01-01T00:00:00Z, Unix s UTC.
pub const MAIN_T0_UNIX_S: f64 = 1_767_225_600.0;
/// Row spacing of the main segment, seconds of UTC.
pub const MAIN_STEP_S: f64 = 3600.0;
/// Rows of the main segment; the last row is 2031-01-01T00:00:00Z.
pub const MAIN_ROWS: usize = 43_825;

/// f64 per row: Sun x, y, z then Earth x, y, z, km, MOON_ME.
const ROW_LEN: usize = 6;

/// Why an ephemeris call failed. `code()` gives the API error code (docs/API.md).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EphError {
    /// No segment has been added.
    NotLoaded,
    /// The time is outside every loaded segment (P16: no extrapolation).
    OutOfTable,
    /// A non-finite or malformed argument.
    BadInput(&'static str),
}

impl EphError {
    pub fn code(&self) -> &'static str {
        match self {
            EphError::NotLoaded => "NOT_LOADED",
            EphError::OutOfTable => "OUT_OF_TABLE",
            EphError::BadInput(_) => "BAD_INPUT",
        }
    }
}

/// One evenly spaced run of rows. Valid for t0 ≤ t ≤ t0 + (rows − 1)·step, the last row included.
#[derive(Debug, Clone)]
struct Segment {
    t0_unix_s: f64,
    step_s: f64,
    data: Vec<f64>,
}

/// Loaded ephemeris segments (the main table, later replay segments).
#[derive(Debug, Clone, Default)]
pub struct Ephemeris {
    segments: Vec<Segment>,
}

impl Ephemeris {
    /// Add a segment starting at `t0_unix_s` (Unix s UTC) with rows every `step_s` seconds.
    /// `data` is row-major, 6 per row, km, MOON_ME. BadInput unless at least two rows,
    /// a whole number of rows, step_s > 0 and every value finite.
    pub fn add(&mut self, t0_unix_s: f64, step_s: f64, data: Vec<f64>) -> Result<(), EphError> {
        if !t0_unix_s.is_finite() || !step_s.is_finite() || step_s <= 0.0 {
            return Err(EphError::BadInput("t0 must be finite and step positive"));
        }
        if !data.len().is_multiple_of(ROW_LEN) || data.len() < 2 * ROW_LEN {
            return Err(EphError::BadInput(
                "ephemeris data must be at least two rows of 6 values",
            ));
        }
        if !data.iter().all(|v| v.is_finite()) {
            return Err(EphError::BadInput("ephemeris data must be finite"));
        }
        self.segments.push(Segment {
            t0_unix_s,
            step_s,
            data,
        });
        Ok(())
    }

    /// [sun_x, sun_y, sun_z, earth_x, earth_y, earth_z] in km, MOON_ME, Moon-centred, apparent,
    /// at `t_unix_s` (Unix s UTC). Linear between rows; exact at a row time.
    pub fn at(&self, t_unix_s: f64) -> Result<[f64; 6], EphError> {
        if !t_unix_s.is_finite() {
            return Err(EphError::BadInput("time must be finite"));
        }
        if self.segments.is_empty() {
            return Err(EphError::NotLoaded);
        }
        self.segments
            .iter()
            .find_map(|s| s.at(t_unix_s))
            .ok_or(EphError::OutOfTable)
    }
}

impl Segment {
    fn rows(&self) -> usize {
        self.data.len() / ROW_LEN
    }

    fn row(&self, k: usize) -> &[f64] {
        &self.data[k * ROW_LEN..(k + 1) * ROW_LEN]
    }

    /// None when t is outside [t0, t0 + (rows − 1)·step].
    fn at(&self, t_unix_s: f64) -> Option<[f64; 6]> {
        let last = self.rows() - 1;
        let u = (t_unix_s - self.t0_unix_s) / self.step_s;
        #[allow(clippy::cast_precision_loss)] // row counts are far below 2^52
        let last_f = last as f64;
        if !(0.0..=last_f).contains(&u) {
            return None;
        }
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // u is in [0, last] (checked above), so floor fits a usize exactly
        let k = u.floor() as usize;
        let mut out = [0.0; 6];
        if k == last {
            out.copy_from_slice(self.row(k));
            return Some(out);
        }
        #[allow(clippy::cast_precision_loss)] // k ≤ rows, far below 2^52
        let a = u - k as f64;
        let (r0, r1) = (self.row(k), self.row(k + 1));
        for i in 0..ROW_LEN {
            out[i] = (1.0 - a) * r0[i] + a * r1[i];
        }
        Some(out)
    }
}

/// Solar radius, km (E117).
pub const R_SUN_KM: f64 = 696_000.0;
/// Earth equatorial radius, km (E98).
pub const R_EARTH_KM: f64 = 6378.137;

/// Columns of a `sun_earth_track` result (docs/API.md).
const TRACK_COLS: usize = 9;

/// Observer site: selenographic MOON_ME latitude and east-positive longitude in radians;
/// `ground_m` above the 1 737 400 m sphere and `mast_m` above the ground, metres.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Site {
    pub lat_rad: f64,
    pub lon_east_rad: f64,
    pub ground_m: f64,
    pub mast_m: f64,
}

impl From<GeometryError> for EphError {
    fn from(e: GeometryError) -> Self {
        match e {
            GeometryError::BadInput(why) => EphError::BadInput(why),
        }
    }
}

/// Topocentric Sun and Earth seen from `site` at `count` times `start_unix_s + i·step_s` (Unix s UTC).
/// Returns 9 × count, column-major: t_unix_s, sun_az_deg, sun_el_deg, sun_radius_deg, sun_dist_km,
/// earth_az_deg, earth_el_deg, earth_radius_deg, earth_dist_km (docs/API.md). Azimuth is clockwise
/// from north in the site's topocentric ENU frame, elevation airless (no refraction, E53). Any time
/// outside the table fails the whole call with `OutOfTable` (P16).
pub fn sun_earth_track(
    eph: &Ephemeris,
    site: &Site,
    start_unix_s: f64,
    step_s: f64,
    count: u32,
) -> Result<Vec<f64>, EphError> {
    if count == 0 {
        return Err(EphError::BadInput("count must be at least 1"));
    }
    if !start_unix_s.is_finite() || !step_s.is_finite() || step_s <= 0.0 {
        return Err(EphError::BadInput("start must be finite and step positive"));
    }
    if !site.ground_m.is_finite() || !site.mast_m.is_finite() {
        return Err(EphError::BadInput("ground and mast heights must be finite"));
    }
    let up = latlon_to_unit(LatLon {
        lat_rad: site.lat_rad,
        lon_east_rad: site.lon_east_rad,
    })?;
    let frame = EnuFrame::at(up)?;
    let site_m = up.scale(MOON_RADIUS_M + site.ground_m + site.mast_m);

    let n = count as usize;
    let mut out = vec![0.0; TRACK_COLS * n];
    for i in 0..n {
        #[allow(clippy::cast_precision_loss)] // i < 2^32, exact in f64
        let t_unix_s = start_unix_s + i as f64 * step_s;
        let row = eph.at(t_unix_s)?;
        out[i] = t_unix_s;
        for (body_km, r_body_km, col) in [(&row[0..3], R_SUN_KM, 1), (&row[3..6], R_EARTH_KM, 5)] {
            // table rows are km; T01's az_el takes metres
            let body_m = Vec3 {
                x: body_km[0],
                y: body_km[1],
                z: body_km[2],
            }
            .scale(1000.0);
            let ae = frame.az_el(site_m, body_m)?;
            let dist_km = body_m.minus(site_m).norm() / 1000.0;
            out[col * n + i] = ae.az_rad.to_degrees();
            out[(col + 1) * n + i] = ae.el_rad.to_degrees();
            out[(col + 2) * n + i] = (r_body_km / dist_km).asin().to_degrees();
            out[(col + 3) * n + i] = dist_km;
        }
    }
    Ok(out)
}
