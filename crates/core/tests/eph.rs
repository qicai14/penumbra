//! T02 ephemeris lookup and Sun/Earth topocentric track: SCIENCE-SPEC §1c steps 3–4, P16, V1 tracer pair.
//! Row numbers refer to the test table in docs/briefs/T02-ephemeris-tracer.md §5.

use penumbra_core::eph::{
    EphError, Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, R_EARTH_KM, R_SUN_KM, Site,
    sun_earth_track,
};

/// Table of brief row 1: t0 = 1000 s, step 3600 s, three rows.
fn three_row_table() -> Ephemeris {
    let mut eph = Ephemeris::default();
    eph.add(
        1000.0,
        3600.0,
        vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, //
            3.0, 6.0, 9.0, 12.0, 15.0, 18.0, //
            0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
        ],
    )
    .unwrap();
    eph
}

// Row 1: u = 900/3600 = 0.25 between rows 0 and 1 (step 10).
#[test]
fn at_interpolates_linearly_between_rows() {
    let got = three_row_table().at(1900.0).unwrap();
    let want = [1.5, 3.0, 4.5, 6.0, 7.5, 9.0];
    for (g, w) in got.iter().zip(want) {
        assert!((g - w).abs() <= 1e-12, "got {got:?}, want {want:?}");
    }
}

// Row 2: each row is returned exactly at its own time, the last row included (P16).
#[test]
#[allow(clippy::float_cmp)] // exact by design: a = 0 at a row time
fn at_returns_rows_exactly_at_their_times_including_the_last() {
    let eph = three_row_table();
    assert_eq!(eph.at(1000.0).unwrap(), [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    assert_eq!(eph.at(4600.0).unwrap(), [3.0, 6.0, 9.0, 12.0, 15.0, 18.0]);
    assert_eq!(eph.at(8200.0).unwrap(), [0.0; 6]);
}

// Row 3 (P16): one second either side of the table, and NaN.
#[test]
fn at_refuses_times_outside_the_table_and_nan() {
    let eph = three_row_table();
    assert_eq!(eph.at(999.0), Err(EphError::OutOfTable));
    assert_eq!(eph.at(8201.0), Err(EphError::OutOfTable));
    assert!(matches!(eph.at(f64::NAN), Err(EphError::BadInput(_))));
    assert!(matches!(eph.at(f64::INFINITY), Err(EphError::BadInput(_))));
}

// Row 4: nothing loaded; bad segment shapes.
#[test]
fn empty_ephemeris_is_not_loaded_and_bad_segments_are_refused() {
    let mut eph = Ephemeris::default();
    assert_eq!(eph.at(0.0), Err(EphError::NotLoaded));
    assert!(matches!(
        eph.add(0.0, 3600.0, vec![0.0; 7]),
        Err(EphError::BadInput(_))
    ));
    assert!(matches!(
        eph.add(0.0, 0.0, vec![0.0; 12]),
        Err(EphError::BadInput(_))
    ));
    assert!(matches!(
        eph.add(0.0, 3600.0, vec![0.0; 6]),
        Err(EphError::BadInput(_))
    ));
    let mut nan_row = vec![0.0; 12];
    nan_row[3] = f64::NAN;
    assert!(matches!(
        eph.add(0.0, 3600.0, nan_row),
        Err(EphError::BadInput(_))
    ));
    assert_eq!(eph.at(0.0), Err(EphError::NotLoaded));
}

// Row 5: two segments; a time in the gap between them is out of the table.
#[test]
#[allow(clippy::float_cmp)] // exact: a constant segment interpolates to itself
fn second_segment_is_found_and_the_gap_is_out_of_table() {
    let mut eph = three_row_table();
    eph.add(100_000.0, 3600.0, vec![7.0; 12]).unwrap();
    assert_eq!(eph.at(101_000.0).unwrap(), [7.0; 6]);
    assert_eq!(eph.at(50_000.0), Err(EphError::OutOfTable));
}

#[test]
fn error_codes_match_the_api() {
    assert_eq!(EphError::NotLoaded.code(), "NOT_LOADED");
    assert_eq!(EphError::OutOfTable.code(), "OUT_OF_TABLE");
    assert_eq!(EphError::BadInput("x").code(), "BAD_INPUT");
}

// Row 6 ([S] C1): the main segment's grid on calendar UTC. 2026-01-01 → 2026-11-14 is 317 days;
// 2026-11-14T00Z = 1 794 614 400 (JD 2461358.5) is row 7608; 2027-01-01T00Z = 1 798 761 600 is row 8760;
// 2031-01-01T00Z = 1 924 992 000 is the last row, 43 824.
#[test]
#[allow(clippy::float_cmp)] // whole seconds, exact in f64
fn main_segment_grid_matches_calendar_utc() {
    assert_eq!(MAIN_T0_UNIX_S, 1_767_225_600.0);
    assert_eq!(MAIN_STEP_S, 3600.0);
    assert_eq!(1_794_614_400.0 - MAIN_T0_UNIX_S, 317.0 * 86_400.0);
    assert_eq!((1_794_614_400.0 - MAIN_T0_UNIX_S) / MAIN_STEP_S, 7608.0);
    assert_eq!((1_798_761_600.0 - MAIN_T0_UNIX_S) / MAIN_STEP_S, 8760.0);
    assert_eq!((1_924_992_000.0 - MAIN_T0_UNIX_S) / MAIN_STEP_S, 43_824.0);
    assert_eq!(MAIN_ROWS, 43_825);
}

fn assert_close(label: &str, got: f64, want: f64, tol: f64) {
    assert!(
        (got - want).abs() <= tol,
        "{label}: got {got}, want {want} ± {tol}"
    );
}

const AU_KM: f64 = 149_597_870.7;

/// Brief row 7: two equal rows, Sun at 1 AU toward (lat −1°, lon 30°), Earth at 384 400 km on +X.
fn constant_table() -> (Ephemeris, f64) {
    let (lat, lon) = ((-1.0f64).to_radians(), 30.0f64.to_radians());
    let sun = [
        AU_KM * lat.cos() * lon.cos(),
        AU_KM * lat.cos() * lon.sin(),
        AU_KM * lat.sin(),
    ];
    let row = [sun[0], sun[1], sun[2], 384_400.0, 0.0, 0.0];
    let mut eph = Ephemeris::default();
    let t0 = 1_000_000.0;
    eph.add(t0, 3600.0, [row, row].concat()).unwrap();
    (eph, t0)
}

/// Brief rows 7–9 site: 85°S, 0°E, ground 0 m, mast 0 m.
fn site_85s() -> Site {
    Site {
        lat_rad: (-85.0f64).to_radians(),
        lon_east_rad: 0.0,
        ground_m: 0.0,
        mast_m: 0.0,
    }
}

// Rows 7–8: values re-derived by science-hard in the T02 design (brief row 7, [R] review 1c, [S] B3, B4).
#[test]
#[allow(clippy::float_cmp)] // the time column is copied, exact
fn sun_earth_track_on_a_constant_table_gives_known_az_el_radius_and_distance() {
    let (eph, t0) = constant_table();
    let n = 3;
    let out = sun_earth_track(&eph, &site_85s(), t0, 1200.0, 3).unwrap();
    assert_eq!(out.len(), 9 * n);
    assert_eq!(&out[0..3], &[t0, t0 + 1200.0, t0 + 2400.0]);
    let col = |c: usize| &out[c * n..(c + 1) * n];
    for i in 0..n {
        assert_close("sun_az_deg", col(1)[i], 30.138_569, 1e-5);
        assert_close("sun_el_deg", col(2)[i], 5.327_131, 1e-5);
        assert_close("sun_radius_deg", col(3)[i], 0.266_568, 1e-5);
        assert_close("sun_dist_km", col(4)[i], 149_597_709.386, 0.1);
        // az 0 may come back as 0 or just under 360; compare on the circle
        let earth_az = col(5)[i];
        assert_close("earth_az_deg", earth_az.min(360.0 - earth_az), 0.0, 1e-5);
        assert_close("earth_el_deg", col(6)[i], 4.741_922, 1e-5);
        assert_close("earth_radius_deg", col(7)[i], 0.951_086, 1e-5);
        assert_close("earth_dist_km", col(8)[i], 384_252.473_6, 0.01);
    }
}

// Row 9: zero count is a bad input; one second before the table fails the whole track (P16).
#[test]
fn sun_earth_track_refuses_zero_count_and_times_outside_the_table() {
    let (eph, t0) = constant_table();
    assert!(matches!(
        sun_earth_track(&eph, &site_85s(), t0, 1200.0, 0),
        Err(EphError::BadInput(_))
    ));
    assert_eq!(
        sun_earth_track(&eph, &site_85s(), t0 - 1.0, 1200.0, 3),
        Err(EphError::OutOfTable)
    );
    // the last sample past the table's end (t0 + 3600) fails the whole call
    assert_eq!(
        sun_earth_track(&eph, &site_85s(), t0, 1801.0, 3),
        Err(EphError::OutOfTable)
    );
    // a huge count is refused before the 9 × count allocation (would trap in wasm)
    assert_eq!(
        sun_earth_track(&eph, &site_85s(), t0, 1.0, u32::MAX),
        Err(EphError::OutOfTable)
    );
    assert!(matches!(
        sun_earth_track(&eph, &site_85s(), t0, 0.0, 3),
        Err(EphError::BadInput(_))
    ));
    let mut bad_site = site_85s();
    bad_site.lat_rad = f64::NAN;
    assert!(matches!(
        sun_earth_track(&eph, &bad_site, t0, 1200.0, 3),
        Err(EphError::BadInput(_))
    ));
}

// Row 10 ([S] D3; spec §1b): the Sun's angular radius at 1 AU.
#[test]
fn solar_radius_at_one_au_is_0_26657_deg() {
    assert_close(
        "asin(R_sun / 1 AU)",
        (R_SUN_KM / AU_KM).asin().to_degrees(),
        0.26657,
        1e-5,
    );
    assert_close("R_EARTH_KM", R_EARTH_KM, 6378.137, 0.0);
}

/// The committed main table, `web/public/data/eph-main.<hash6>.bin`, loaded as the main segment.
fn shipped_main_table() -> Vec<f64> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    let mut bins: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let name = p.file_name().unwrap().to_str().unwrap();
            name.starts_with("eph-main.") && name.ends_with(".bin")
        })
        .collect();
    assert_eq!(bins.len(), 1, "exactly one eph-main.*.bin in {dir:?}");
    let bytes = std::fs::read(bins.pop().unwrap()).unwrap();
    assert_eq!(bytes.len(), 6 * 8 * MAIN_ROWS, "2 103 600 bytes");
    let (words, _) = bytes.as_chunks::<8>();
    words.iter().map(|w| f64::from_le_bytes(*w)).collect()
}

// The manifest the browser reads must describe the committed table with the core constants (ARCHITECTURE §3.5).
#[test]
fn shipped_manifest_matches_table_and_constants() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    let m = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    for field in [
        format!("\"t0_unix_s\":{MAIN_T0_UNIX_S}"),
        format!("\"step_s\":{MAIN_STEP_S}"),
        format!("\"rows\":{MAIN_ROWS}"),
    ] {
        assert!(m.contains(&field), "manifest.json lacks {field}: {m}");
    }
    let url = m
        .split("\"url\":\"data/")
        .nth(1)
        .unwrap()
        .split('"')
        .next()
        .unwrap();
    assert!(
        dir.join(url).is_file(),
        "manifest url data/{url} is not a shipped file"
    );
    shipped_main_table(); // and it is the single eph-main.*.bin, of the right length
}

fn shipped_ephemeris() -> Ephemeris {
    let mut eph = Ephemeris::default();
    eph.add(MAIN_T0_UNIX_S, MAIN_STEP_S, shipped_main_table())
        .unwrap();
    eph
}

// Brief row 11 (step 8): frame and sign checks on every row of the shipped table.
// Bounds [D] from E101, E102, E104, E107: distances, ME frame with +X toward the Earth, sub-solar motion.
#[test]
fn shipped_table_sanity_bounds() {
    let data = shipped_main_table();
    let norm = |v: &[f64]| v.iter().map(|c| c * c).sum::<f64>().sqrt();
    let mut prev_subsolar_lon_deg: Option<f64> = None;
    for (k, row) in data.chunks(6).enumerate() {
        let (sun, earth) = (&row[0..3], &row[3..6]);
        let (s, e) = (norm(sun), norm(earth));
        assert!((146.6e6..=152.6e6).contains(&s), "row {k}: |Sun| {s} km");
        assert!(
            (356_000.0..=407_000.0).contains(&e),
            "row {k}: |Earth| {e} km"
        );
        let sub_earth_lat = (earth[2] / e).asin().to_degrees();
        let sub_earth_lon = earth[1].atan2(earth[0]).to_degrees();
        assert!(
            sub_earth_lat.abs() <= 6.9,
            "row {k}: sub-Earth lat {sub_earth_lat}"
        );
        assert!(
            sub_earth_lon.abs() <= 8.2,
            "row {k}: sub-Earth lon {sub_earth_lon}"
        );
        let sub_solar_lat = (sun[2] / s).asin().to_degrees();
        assert!(
            sub_solar_lat.abs() <= 1.6,
            "row {k}: sub-solar lat {sub_solar_lat}"
        );
        let lon = sun[1].atan2(sun[0]).to_degrees();
        if let Some(prev) = prev_subsolar_lon_deg {
            let d = (lon - prev + 540.0).rem_euclid(360.0) - 180.0;
            assert!(
                (-0.53..=-0.49).contains(&d),
                "row {k}: sub-solar dlon {d}°/h"
            );
        }
        prev_subsolar_lon_deg = Some(lon);
    }
}

/// Azimuth and elevation (degrees) of the single $$SOE row of a stored Horizons response,
/// after asserting the request parameters the comparison depends on (P15).
fn horizons_az_el(fixture: &str) -> (f64, f64) {
    for header in [
        "MEAN_ME",
        "1737.4, 1737.4, 1737.4",
        "29.1957, -84.7906, .002",
        "AIRLESS",
    ] {
        assert!(fixture.contains(header), "fixture header lacks {header:?}");
    }
    assert!(fixture.contains("\"request\": \"https://ssd.jpl.nasa.gov/api/horizons.api?"));
    let soe = fixture.split("$$SOE").nth(1).unwrap();
    let fields: Vec<&str> = soe.split("$$EOE").next().unwrap().split(',').collect();
    assert!(fields[0].contains("2026-Nov-14 00:00:00.000"));
    (
        fields[3].trim().parse().unwrap(),
        fields[4].trim().parse().unwrap(),
    )
}

// V1 tracer pair (brief row 13, T02 design): IM-2, mast 2 m, 2026-11-14T00Z through the shipped table
// vs JPL Horizons, Δaz·cos(el) and Δel within 0.002° (tight enough to see P3 at 0.0056° and P4 at 0.0097°).
#[test]
fn v1_tracer_im2_matches_horizons_through_shipped_table() {
    let im2 = Site {
        lat_rad: (-84.7906f64).to_radians(),
        lon_east_rad: 29.1957f64.to_radians(),
        ground_m: 0.0,
        mast_m: 2.0,
    };
    let t = sun_earth_track(&shipped_ephemeris(), &im2, 1_794_614_400.0, 3600.0, 1).unwrap();
    for (body, az_col) in [("sun", 1), ("earth", 5)] {
        let fixture = std::fs::read_to_string(format!(
            "{}/tests/fixtures/horizons/tracer-{body}.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap();
        let (hz_az, hz_el) = horizons_az_el(&fixture);
        let (az, el) = (t[az_col], t[az_col + 1]);
        let d_az = ((az - hz_az + 540.0).rem_euclid(360.0) - 180.0) * hz_el.to_radians().cos();
        let d_el = el - hz_el;
        println!(
            "{body}: ours az {az:.9} el {el:.9}; Horizons {hz_az} {hz_el}; Δaz·cos(el) {d_az:+.2e}° Δel {d_el:+.2e}°"
        );
        assert!(d_az.abs() <= 0.002, "{body} Δaz·cos(el) {d_az}°");
        assert!(d_el.abs() <= 0.002, "{body} Δel {d_el}°");
    }
}
