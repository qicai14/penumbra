//! T05 V1: 20 JPL Horizons site/time pairs through the shipped table and interpolation (SCIENCE-SPEC §1c,
//! §3 "V1 details", N3, N8, U13, P15), plus the Earth and Sun sanity checks P4, P9, T-1d-1 and T-1d-1b.
//! Fixtures: `tests/fixtures/horizons/v1-pairs.json`, written by `bun scripts/fetch-horizons-v1.ts`.

use penumbra_core::eph::{
    Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, Site, sun_earth_track,
};
use serde_json::Value;

/// The committed main table, `web/public/data/eph-main.<hash6>.bin`, as the only loaded segment.
fn shipped_ephemeris() -> Ephemeris {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    let bins: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let name = p.file_name().unwrap().to_str().unwrap();
            name.starts_with("eph-main.") && name.ends_with(".bin")
        })
        .collect();
    assert_eq!(bins.len(), 1, "exactly one eph-main.*.bin in {dir:?}");
    let bytes = std::fs::read(&bins[0]).unwrap();
    let (words, _) = bytes.as_chunks::<8>();
    let mut eph = Ephemeris::default();
    eph.add(
        MAIN_T0_UNIX_S,
        MAIN_STEP_S,
        words.iter().map(|w| f64::from_le_bytes(*w)).collect(),
    )
    .unwrap();
    eph
}

fn pairs() -> Vec<Value> {
    let path = format!(
        "{}/tests/fixtures/horizons/v1-pairs.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let v: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    v.as_array().unwrap().clone()
}

fn num(v: &Value, key: &str) -> f64 {
    v[key]
        .as_f64()
        .unwrap_or_else(|| panic!("{key} is not a number"))
}

/// Unix seconds of a proleptic Gregorian UTC date (Hinnant's days_from_civil), no leap seconds,
/// so a fixture's `t_unix_s` is checked against its `t_utc` text independently of the fetch script.
fn unix_s_utc(y: i64, m: i64, d: i64, hh: i64, mm: i64, ss: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (m + if m > 2 { -3 } else { 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    days * 86_400 + hh * 3600 + mm * 60 + ss
}

/// Site of a pair: ground = h_km × 1000 above the 1737.4 km sphere (§1c: h = (DEM + H)/1000), mast 0.
fn site(pair: &Value) -> Site {
    Site {
        lat_rad: num(pair, "lat_deg").to_radians(),
        lon_east_rad: num(pair, "lon_east_deg").to_radians(),
        ground_m: num(pair, "h_km") * 1000.0,
        mast_m: 0.0,
    }
}

/// Horizons az/el (degrees, airless apparent, topocentric) of one stored response, after checking
/// the request follows §1c and the header echoes the 1737.4 km sphere (U13) and this pair's site (P15).
fn horizons_az_el(pair: &Value, body: &str) -> (f64, f64) {
    let id = pair["id"].as_str().unwrap();
    let request = pair[body]["request"].as_str().unwrap();
    for p in [
        "CENTER=%27coord%40301%27",
        "COORD_TYPE=%27GEODETIC%27",
        "QUANTITIES=%274%27",
        "TIME_TYPE=%27UT%27",
        "ANG_FORMAT=%27DEG%27",
        "EXTRA_PREC=%27YES%27",
        "APPARENT=%27AIRLESS%27",
    ] {
        assert!(request.contains(p), "{id} {body}: request lacks {p}");
    }
    let command = if body == "sun" {
        "%2710%27"
    } else {
        "%27399%27"
    };
    assert!(request.contains(&format!("COMMAND={command}")));
    let result = pair[body]["response"]["result"].as_str().unwrap();
    for header in [
        "Center radii    : 1737.4, 1737.4, 1737.4 km",
        "Center pole/equ : MEAN_ME",
    ] {
        assert!(
            result.contains(header),
            "{id} {body}: header lacks {header:?}"
        );
    }
    let geodetic: Vec<f64> = result
        .split("Center geodetic : ")
        .nth(1)
        .unwrap()
        .split('{')
        .next()
        .unwrap()
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();
    let want = [
        num(pair, "lon_east_deg"),
        num(pair, "lat_deg"),
        num(pair, "h_km"),
    ];
    for (g, w) in geodetic.iter().zip(want) {
        assert!(
            (g - w).abs() <= 1e-9,
            "{id} {body}: site echo {geodetic:?} vs {want:?}"
        );
    }

    // t_utc "2026-12-31 23:59:00" is printed by Horizons as "2026-Dec-31 23:59:00.000" (UT = UTC, E54)
    let t_utc = pair["t_utc"].as_str().unwrap();
    let f: Vec<i64> = t_utc
        .split(['-', ' ', ':'])
        .map(|s| s.parse().unwrap())
        .collect();
    #[allow(clippy::cast_precision_loss)] // whole seconds below 2^53
    let t_unix_s = unix_s_utc(f[0], f[1], f[2], f[3], f[4], f[5]) as f64;
    assert!(
        (num(pair, "t_unix_s") - t_unix_s).abs() < 0.5,
        "{id}: t_unix_s"
    );
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let stamp = format!(
        "{}-{}-{} {}.000",
        &t_utc[0..4],
        months[usize::try_from(f[1] - 1).unwrap()],
        &t_utc[8..10],
        &t_utc[11..19]
    );
    let soe = result.split("$$SOE").nth(1).unwrap();
    let fields: Vec<&str> = soe.split("$$EOE").next().unwrap().split(',').collect();
    assert_eq!(fields[0].trim(), stamp, "{id} {body}: time tag");
    (
        fields[3].trim().parse().unwrap(),
        fields[4].trim().parse().unwrap(),
    )
}

/// V1 metric for one body (N8): (Δaz·cos(el), Δel) in degrees, Δaz wrapped into [−180°, 180°).
fn v1_error_deg(az: f64, el: f64, hz_az: f64, hz_el: f64) -> (f64, f64) {
    let d_az = ((az - hz_az + 540.0).rem_euclid(360.0) - 180.0) * hz_el.to_radians().cos();
    (d_az, el - hz_el)
}

// N8: the metric wraps 0/360 and scales by cos(el); values worked by hand.
#[test]
fn v1_metric_wraps_azimuth_and_scales_by_cos_el() {
    let (d_az, d_el) = v1_error_deg(359.999, 60.0, 0.001, 59.999);
    assert!(
        (d_az - (-0.002 * 59.999f64.to_radians().cos())).abs() < 1e-9,
        "{d_az}"
    );
    assert!((d_el - 0.001).abs() < 1e-9, "{d_el}");
    let (d_az, _) = v1_error_deg(0.0005, 0.0, 359.9995, 0.0);
    assert!((d_az - 0.001).abs() < 1e-9, "{d_az}");
}

// V1 (§3, N3): every pair through the shipped hourly table and linear interpolation, worst
// |Δaz·cos(el)| and |Δel| ≤ 0.002° for the Sun and for the Earth.
#[test]
fn v1_twenty_horizons_pairs_through_shipped_table_within_0_002_deg() {
    let eph = shipped_ephemeris();
    let pairs = pairs();
    assert_eq!(pairs.len(), 20);
    let mut worst = [0.0f64; 2]; // sun, earth
    for pair in &pairs {
        let t = sun_earth_track(&eph, &site(pair), num(pair, "t_unix_s"), 3600.0, 1).unwrap();
        for (b, (body, az_col)) in [("sun", 1), ("earth", 5)].into_iter().enumerate() {
            let (hz_az, hz_el) = horizons_az_el(pair, body);
            let (d_az, d_el) = v1_error_deg(t[az_col], t[az_col + 1], hz_az, hz_el);
            println!(
                "{} {body:5} {:>9.5} {:>9.5}  Δaz·cos(el) {d_az:+.2e}  Δel {d_el:+.2e}  {}",
                pair["id"], hz_az, hz_el, pair["why"]
            );
            worst[b] = worst[b].max(d_az.abs()).max(d_el.abs());
        }
    }
    println!(
        "V1 worst, deg: Sun {:.3e}, Earth {:.3e}",
        worst[0], worst[1]
    );
    assert!(worst[0] <= 0.002, "V1 Sun worst {}°", worst[0]);
    assert!(worst[1] <= 0.002, "V1 Earth worst {}°", worst[1]);
}

/// Sub-Earth latitude and longitude, degrees, of every row of the shipped table (MOON_ME, apparent).
fn sub_earth_lat_lon_deg(eph: &Ephemeris) -> Vec<(f64, f64)> {
    (0..MAIN_ROWS)
        .map(|k| {
            #[allow(clippy::cast_precision_loss)] // k < 43 825
            let row = eph.at(MAIN_T0_UNIX_S + k as f64 * MAIN_STEP_S).unwrap();
            let e = (row[3] * row[3] + row[4] * row[4] + row[5] * row[5]).sqrt();
            (
                (row[5] / e).asin().to_degrees(),
                row[4].atan2(row[3]).to_degrees(),
            )
        })
        .collect()
}

// §3 "V1 details": pairs 01–04 sit on the libration extremes found in the table (max/min sub-Earth
// latitude, max/min sub-Earth longitude), inside the E101/E102 limits of 8.16° and 6.87°.
#[test]
fn v1_pairs_include_each_libration_extreme_of_the_table() {
    let ll = sub_earth_lat_lon_deg(&shipped_ephemeris());
    let arg = |f: &dyn Fn(&(f64, f64)) -> f64, max: bool| {
        let mut best = 0;
        for k in 1..ll.len() {
            if (f(&ll[k]) > f(&ll[best])) == max {
                best = k;
            }
        }
        #[allow(clippy::cast_precision_loss)] // k < 43 825
        let t = MAIN_T0_UNIX_S + best as f64 * MAIN_STEP_S;
        (t, f(&ll[best]))
    };
    let extremes = [
        arg(&|p| p.0, true),
        arg(&|p| p.0, false),
        arg(&|p| p.1, true),
        arg(&|p| p.1, false),
    ];
    let pairs = pairs();
    for (pair, (t, value)) in pairs.iter().zip(extremes) {
        println!("{} extreme {value:+.4}° at {t}", pair["id"]);
        assert!(
            (num(pair, "t_unix_s") - t).abs() < 0.5,
            "{}: not the table extreme",
            pair["id"]
        );
    }
    assert!(extremes[0].1 > 6.0 && extremes[0].1 <= 6.87);
    assert!(extremes[1].1 < -6.0 && extremes[1].1 >= -6.87);
    assert!(extremes[2].1 > 7.0 && extremes[2].1 <= 8.16);
    assert!(extremes[3].1 < -7.0 && extremes[3].1 >= -8.16);
}

// P4 (V5 year boundary): at IM-2 (2 m) the track is continuous from 2026-12-31T23:59 to
// 2027-01-01T00:01 UTC: two equal one-minute steps, the Sun's azimuth falling at about 0.5°/h (E107).
#[test]
fn p4_track_is_continuous_across_the_2027_year_boundary() {
    let im2 = Site {
        lat_rad: (-84.7906f64).to_radians(),
        lon_east_rad: 29.1957f64.to_radians(),
        ground_m: 0.0,
        mast_m: 2.0,
    };
    let start_unix_s = 1_798_761_600.0 - 60.0; // 2026-12-31T23:59:00Z
    let t = sun_earth_track(&shipped_ephemeris(), &im2, start_unix_s, 60.0, 3).unwrap();
    let col = |c: usize| &t[3 * c..3 * c + 3];
    for (name, c) in [
        ("sun_az", 1),
        ("sun_el", 2),
        ("earth_az", 5),
        ("earth_el", 6),
    ] {
        let (s1, s2) = (col(c)[1] - col(c)[0], col(c)[2] - col(c)[1]);
        println!("{name}: steps {s1:+.6e} {s2:+.6e} deg/min");
        assert!(
            (s1 - s2).abs() < 1e-4,
            "{name} jumps: {s1} then {s2} deg/min"
        );
    }
    let sun_az_rate_deg_h = (col(1)[2] - col(1)[0]) / 2.0 * 60.0;
    assert!(
        (-0.55..=-0.45).contains(&sun_az_rate_deg_h),
        "Sun az rate {sun_az_rate_deg_h}°/h"
    );
}

/// Every hourly row of the shipped table seen from a bare-sphere site (ground and mast 0 m).
fn hourly_track(lat_deg: f64, lon_east_deg: f64) -> Vec<f64> {
    let s = Site {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_east_deg.to_radians(),
        ground_m: 0.0,
        mast_m: 0.0,
    };
    let n = u32::try_from(MAIN_ROWS).unwrap();
    sun_earth_track(&shipped_ephemeris(), &s, MAIN_T0_UNIX_S, MAIN_STEP_S, n).unwrap()
}

// P9: at 85°S, 0°E the Earth stays within 9° of azimuth 0 (E101) and the Sun's azimuth falls at
// about 0.5°/h (E107) every hour of 2026–2030.
#[test]
fn p9_earth_near_az_0_and_sun_az_falls_half_a_degree_per_hour_at_85s_0e() {
    let t = hourly_track(-85.0, 0.0);
    let n = MAIN_ROWS;
    let (sun_az, earth_az) = (&t[n..2 * n], &t[5 * n..6 * n]);
    let worst_earth = earth_az
        .iter()
        .map(|a| a.min(360.0 - a))
        .fold(0.0f64, f64::max);
    println!("P9 Earth max |az| {worst_earth:.3}°");
    assert!(worst_earth <= 9.0, "Earth az {worst_earth}° from 0");
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for w in sun_az.windows(2) {
        let d = (w[1] - w[0] + 540.0).rem_euclid(360.0) - 180.0;
        lo = lo.min(d);
        hi = hi.max(d);
    }
    println!("P9 Sun az rate {lo:.4} to {hi:.4} °/h");
    assert!(lo >= -0.55 && hi <= -0.45, "Sun az rate {lo}..{hi} °/h");
}

// T-1d-1: at (−60°, 180°) on a flat sphere the Earth's upper limb stays below −20° in 2026–2030.
#[test]
fn t_1d_1_far_side_60s_earth_upper_limb_stays_below_minus_20_deg() {
    let t = hourly_track(-60.0, 180.0);
    let n = MAIN_ROWS;
    let top = (0..n)
        .map(|i| t[6 * n + i] + t[7 * n + i])
        .fold(f64::NEG_INFINITY, f64::max);
    println!("T-1d-1 Earth upper limb max {top:.3}°");
    assert!(top < -20.0, "upper limb {top}°");
}

// T-1d-1b: at (−85°, 180°) the Earth-centre elevation peaks in [+1.0°, +1.7°] and is negative
// more than half the time.
#[test]
fn t_1d_1b_85s_180e_earth_centre_peaks_1_to_1_7_deg_and_is_mostly_below() {
    let t = hourly_track(-85.0, 180.0);
    let n = MAIN_ROWS;
    let el = &t[6 * n..7 * n];
    let peak = el.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    #[allow(clippy::cast_precision_loss)] // counts < 43 825
    let below = el.iter().filter(|e| **e < 0.0).count() as f64 / n as f64;
    println!(
        "T-1d-1b Earth centre peak {peak:.3}°, below {:.1} %",
        below * 100.0
    );
    assert!((1.0..=1.7).contains(&peak), "peak {peak}°");
    assert!(below > 0.5, "negative {below}");
}
