//! T10 V5 red-team suite, cases 1, 2 and 4–7 (SCIENCE-SPEC §3 "V5 cases"; case 3, the PSR floor,
//! is `crates/pipeline/tests/v5_psr.rs` because it needs the raw GeoTIFFs). Each case prints its
//! measured value; `cargo test -p penumbra-core --test v5_red_team -- --nocapture` shows them.
//! Times are Unix s UTC, angles degrees in the site's topocentric ENU frame, heights metres.

use penumbra_core::dem::{DemError, DemTile, TileMeta};
use penumbra_core::eph::{
    Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, Site, sun_earth_track,
};
use penumbra_core::geometry::{
    AzEl, LatLon, MapXy, flat_sphere_dip_rad, latlon_to_xy, xy_to_latlon,
};
use penumbra_core::horizon::{HorizonOptions, N_AZ, horizon};
use penumbra_core::power::{AU_KM, Lethal, Panel, Preset, survival};
use penumbra_core::timeline::{Timeline, earth_visible_any_fraction, timeline};

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

fn site(lat_deg: f64, lon_east_deg: f64, mast_m: f64) -> Site {
    Site {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_east_deg.to_radians(),
        ground_m: 0.0,
        mast_m,
    }
}

/// Flat (h = 0) square tile, scale 0.25 m, pixel-is-area; `(x0_m, y0_m)` is the upper-left corner.
fn flat_tile(n: u32, pixel_m: f64, x0_m: f64, y0_m: f64) -> DemTile {
    let meta = TileMeta {
        width: n,
        height: n,
        x0_m,
        y0_m,
        pixel_m,
        scale_m: 0.25,
        offset_m: 0.0,
        nodata: i16::MIN,
    };
    DemTile::new(meta, vec![0; usize::try_from(n * n).unwrap()]).unwrap()
}

/// Flat tile with its middle on `c`.
fn flat_around(n: u32, pixel_m: f64, c: MapXy) -> DemTile {
    let half = f64::from(n) / 2.0 * pixel_m;
    flat_tile(n, pixel_m, c.x_m - half, c.y_m + half)
}

fn all_finite(label: &str, values: &[f64]) {
    let bad = values.iter().filter(|v| !v.is_finite()).count();
    assert_eq!(bad, 0, "{label}: {bad} non-finite values");
}

// Case 1: at the pole (−90°) nothing is NaN, and on a flat sphere the horizon is −0.0869° at
// H = 2 m at every azimuth. Checked through the horizon, a 5-year hourly timeline and its API array.
#[test]
fn case1_pole_has_no_nan_and_flat_sphere_horizon_is_minus_0_0869_deg_at_2_m() {
    let pole = LatLon {
        lat_rad: -std::f64::consts::FRAC_PI_2,
        lon_east_rad: 0.0,
    };
    let c = latlon_to_xy(pole).unwrap();
    assert!(c.x_m.abs() < 1e-9 && c.y_m.abs() < 1e-9);
    let opts = HorizonOptions {
        max_range_m: 20_000.0,
        ..HorizonOptions::default()
    };
    let h = horizon(
        &flat_around(760, 20.0, c),
        &flat_around(200, 240.0, c),
        &flat_around(200, 240.0, c),
        pole,
        2.0,
        &opts,
    )
    .unwrap();
    let el_deg: Vec<f64> = h.el_rad.iter().map(|e| e.to_degrees()).collect();
    let (lo, hi) = el_deg
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(l, u), &e| {
            (l.min(e), u.max(e))
        });
    println!(
        "case 1: pole horizon at H = 2 m, {N_AZ} azimuths: min {lo:.7}°, max {hi:.7}° (spec −0.0869°)"
    );
    all_finite("horizon el", &h.el_rad);
    all_finite("horizon dist", &h.dist_m);
    for (i, e) in el_deg.iter().enumerate() {
        assert!((e + 0.0869).abs() < 5e-5, "bin {i}: {e}°");
    }
    let dip = flat_sphere_dip_rad(2.0).unwrap().to_degrees();
    assert!((lo - dip).abs() < 1e-6 && (hi - dip).abs() < 1e-6);

    let eph = shipped_ephemeris();
    let s = site(-90.0, 0.0, 2.0);
    let n = u32::try_from(MAIN_ROWS).unwrap();
    let tl = timeline(
        &eph,
        &h.el_rad,
        &s,
        MAIN_T0_UNIX_S,
        MAIN_STEP_S,
        n,
        2f64.to_radians(),
    )
    .unwrap();
    all_finite("timeline API array", &tl.to_api());
    let track = sun_earth_track(&eph, &s, MAIN_T0_UNIX_S, MAIN_STEP_S, n).unwrap();
    all_finite("track", &track);
    println!(
        "case 1: {} hourly timeline and track values, none non-finite",
        tl.to_api().len() + track.len()
    );
}

// Case 2: flat pole, every hour of 2026–2030: Sun elevation within ±1.6° (E104, E99), Earth-centre
// elevation within ±7.2° (E102: 6.87° latitude libration + 0.26° topocentric shift).
#[test]
fn case2_flat_pole_sun_within_1_6_deg_and_earth_within_7_2_deg() {
    let n = MAIN_ROWS;
    let t = sun_earth_track(
        &shipped_ephemeris(),
        &site(-90.0, 0.0, 0.0),
        MAIN_T0_UNIX_S,
        MAIN_STEP_S,
        u32::try_from(n).unwrap(),
    )
    .unwrap();
    let worst = |col: usize| {
        t[col * n..(col + 1) * n]
            .iter()
            .fold(0.0f64, |m, v| m.max(v.abs()))
    };
    let (sun, earth) = (worst(2), worst(6));
    println!(
        "case 2: pole {n} hourly steps: max |Sun el| {sun:.4}° (bound 1.6°), max |Earth el| {earth:.4}° (bound 7.2°)"
    );
    assert!(sun <= 1.6, "Sun el {sun}°");
    assert!(earth <= 7.2, "Earth el {earth}°");
}

// Case 4 (P4): positions are continuous from 2026-12-31T23:59 to 2027-01-01T00:01 UTC, so a kink or
// misaligned row at the boundary shows as unequal minute steps; the constant 69 s offset is V1's job (P4).
#[test]
fn case4_year_boundary_is_continuous() {
    let start = 1_798_761_600.0 - 60.0; // 2026-12-31T23:59:00Z
    let t = sun_earth_track(
        &shipped_ephemeris(),
        &site(-84.7906, 29.1957, 2.0),
        start,
        60.0,
        3,
    )
    .unwrap();
    for (name, c) in [
        ("sun_az", 1),
        ("sun_el", 2),
        ("earth_az", 5),
        ("earth_el", 6),
    ] {
        let col = &t[3 * c..3 * c + 3];
        let (s1, s2) = (col[1] - col[0], col[2] - col[1]);
        println!("case 4: {name} steps {s1:+.6e} then {s2:+.6e} deg/min across 2027-01-01T00:00Z");
        assert!((s1 - s2).abs() < 1e-4, "{name} jumps: {s1} then {s2}");
    }
}

// Case 4 (P16): the first and last table rows are valid; one second outside either end is an error
// that names the table (OUT_OF_TABLE), for a single instant and for a span that runs off the end.
#[test]
fn case4_table_ends_are_valid_and_one_second_beyond_is_an_error() {
    let eph = shipped_ephemeris();
    let s = site(-85.0, 0.0, 0.0);
    #[allow(clippy::cast_precision_loss)] // 43 824 is exact in f64
    let last = MAIN_T0_UNIX_S + (MAIN_ROWS - 1) as f64 * MAIN_STEP_S;
    assert!(
        (last - 1_924_992_000.0).abs() < 1e-6,
        "last row is 2031-01-01T00:00:00Z"
    );
    assert!(eph.at(MAIN_T0_UNIX_S).is_ok() && eph.at(last).is_ok());
    for t in [MAIN_T0_UNIX_S - 1.0, last + 1.0] {
        assert_eq!(eph.at(t).unwrap_err().code(), "OUT_OF_TABLE", "t = {t}");
        assert_eq!(
            sun_earth_track(&eph, &s, t, MAIN_STEP_S, 1)
                .unwrap_err()
                .code(),
            "OUT_OF_TABLE"
        );
    }
    // a span whose last step is the first second past the end fails whole, with no partial result
    let err = sun_earth_track(&eph, &s, last - 3599.0, MAIN_STEP_S, 2).unwrap_err();
    assert_eq!(err.code(), "OUT_OF_TABLE");
    println!(
        "case 4: rows 2026-01-01T00:00Z..2031-01-01T00:00Z valid; t0-1 s, last+1 s and an overrunning span all OUT_OF_TABLE"
    );
}

// Case 5 (T-1d-1): far side at (−60°, 180°), flat sphere: the Earth never reaches the horizon.
// Its upper limb stays below −20° and no step has any fraction of the disc above it.
#[test]
fn case5_far_side_60s_earth_never_visible() {
    let n = MAIN_ROWS;
    let t = sun_earth_track(
        &shipped_ephemeris(),
        &site(-60.0, 180.0, 0.0),
        MAIN_T0_UNIX_S,
        MAIN_STEP_S,
        u32::try_from(n).unwrap(),
    )
    .unwrap();
    let flat = [0.0; N_AZ];
    let (mut top, mut visible) = (f64::NEG_INFINITY, 0usize);
    for i in 0..n {
        let earth = AzEl {
            az_rad: t[5 * n + i].to_radians(),
            el_rad: t[6 * n + i].to_radians(),
        };
        top = top.max(t[6 * n + i] + t[7 * n + i]);
        visible += usize::from(
            earth_visible_any_fraction(&flat, earth, t[7 * n + i].to_radians()).unwrap(),
        );
    }
    println!(
        "case 5: (−60°, 180°) {n} steps: Earth upper limb max {top:.3}° (bound −20°), steps with any Earth visible {visible}"
    );
    assert!(top < -20.0, "upper limb {top}°");
    assert_eq!(visible, 0);
}

/// Far tile 834² flat at 240 m on the pole (footprint ends at ±100 080 m), flat 20 m near tile
/// centred on the site: the P5 scene of `tests/horizon.rs`, rebuilt here so this suite stands alone.
fn edge_case(x_m: f64, y_m: f64) -> Result<(), DemError> {
    let s = xy_to_latlon(MapXy { x_m, y_m }).unwrap();
    let near = flat_around(760, 20.0, MapXy { x_m, y_m });
    let far = flat_tile(834, 240.0, -100_080.0, 100_080.0);
    let opts = HorizonOptions {
        max_range_m: 50_000.0,
        ..HorizonOptions::default()
    };
    horizon(&near, &far, &far, s, 2.0, &opts).map(|_| ())
}

// Case 6 (P5): a site 5 km inside the far tile's edge returns a coverage error, never a horizon from
// invented 0 m terrain. A site well inside the same tile is accepted, so the error is the edge's doing.
#[test]
fn case6_dem_edge_returns_a_coverage_error() {
    assert_eq!(edge_case(95_080.0, 0.0), Err(DemError::OutOfCoverage));
    assert_eq!(edge_case(0.0, 95_080.0), Err(DemError::OutOfCoverage));
    assert!(edge_case(20_000.0, 30_000.0).is_ok());
    println!(
        "case 6: sites 5 km inside the far tile edge (x and y) -> OUT_OF_COVERAGE; site 36 km from the pole -> horizon"
    );
}

fn tl_no_sun(n: usize) -> Timeline {
    #[allow(clippy::cast_precision_loss)] // small test counts
    let t = (0..n).map(|i| 1_794_614_400.0 + 600.0 * i as f64).collect();
    Timeline {
        t_unix_s: t,
        sun_az_deg: vec![0.0; n],
        sun_el_deg: vec![0.0; n],
        illumination: vec![0.0; n],
        visibility: vec![0.0; n],
        earth_az_deg: vec![0.0; n],
        earth_el_deg: vec![5.0; n],
        earth_margin_deg: vec![5.0; n],
        contact: vec![1.0; n],
        sun_dist_km: vec![AU_KM; n],
    }
}

fn preset(start_wh: f64, area_m2: f64, eff: f64, load_w: f64) -> Preset {
    Preset {
        capacity_wh: 10_000.0,
        start_wh,
        min_wh: 0.0,
        panel_area_m2: area_m2,
        efficiency: eff,
        panel: Panel::SunTracking,
        load_w,
    }
}

// Case 7 (T-1e-1): P_gen = 0, P_load = 100 W, E0 = 705 Wh, E_min = 0 → dead at 7.050 h ± 1e-9.
#[test]
fn case7_t_1e_1_705_wh_at_100_w_dies_at_7_050_h() {
    let tl = tl_no_sun(60);
    let s = survival(&tl, &preset(705.0, 1.5, 0.3, 100.0), tl.t_unix_s[0]).unwrap();
    let Lethal::At {
        after_landing_h, ..
    } = s.lethal
    else {
        panic!("expected a lethal shadow, got {:?}", s.lethal);
    };
    println!("case 7 T-1e-1: lethal at {after_landing_h:.12} h (spec 7.050 ± 1e-9)");
    assert!((after_landing_h - 7.050).abs() <= 1e-9, "{after_landing_h}");
}

// Case 7 (T-1e-2): with no clamp hit, Σ(P_gen − P_load)·Δt = E_end − E_start. 1 m² at 10 % at 1 AU
// gives 136.61 W × f; f = 1, ½, 1, ½, 1, ½ over six 10-minute steps and 100 W load:
// (614.745 − 600) / 6 = 2.4575 Wh, computed here from the inputs, not from the model.
#[test]
fn case7_t_1e_2_energy_is_conserved_without_clamps() {
    let mut tl = tl_no_sun(7);
    tl.illumination = vec![1.0, 0.5, 1.0, 0.5, 1.0, 0.5, 1.0];
    let s = survival(&tl, &preset(5_000.0, 1.0, 0.1, 100.0), tl.t_unix_s[0]).unwrap();
    assert_eq!(s.lethal, Lethal::NoneInSpan);
    let gen_w: f64 = tl.illumination[..6].iter().map(|f| 1366.1 * 0.1 * f).sum();
    let expected = (gen_w - 6.0 * 100.0) / 6.0;
    let got = s.soc_wh[6] - s.soc_wh[0];
    println!(
        "case 7 T-1e-2: E_end − E_start = {got:.9} Wh, Σ(P_gen − P_load)·Δt = {expected:.9} Wh"
    );
    assert!((got - expected).abs() < 1e-9, "{got} vs {expected}");
}
