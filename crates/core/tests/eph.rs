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
