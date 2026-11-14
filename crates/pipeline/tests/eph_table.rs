//! T02 ephemeris table against native ANISE (SCIENCE-SPEC T-1c-1) and the shipped file against a rebuild.
//! Run: PENUMBRA_DATA=/path/to/penumbra-resource cargo test -p penumbra-pipeline --release --test eph_table -- --ignored --nocapture

use anise::constants::frames::{MOON_ME_DE440_ME421_FRAME, MOON_PA_DE440_FRAME};
use anise::prelude::Aberration;
use penumbra_core::eph::{
    Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, REPLAY_IM2_ROWS, REPLAY_IM2_STEP_S,
    REPLAY_IM2_T0_UNIX_S,
};
use penumbra_pipeline::eph::{
    MAIN_FILE_PREFIX, REPLAY_IM2_FILE_PREFIX, build_table, load_almanac, moon_me_row, moon_row,
    read_shipped_table,
};
use std::path::PathBuf;

fn kernels() -> PathBuf {
    PathBuf::from(std::env::var("PENUMBRA_DATA").expect("set PENUMBRA_DATA to the download folder"))
}

fn shipped_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data")
}

/// Angle between two 3-vectors, degrees (atan2 form, accurate at small angles).
fn angle_deg(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross = [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ];
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    (cross.iter().map(|c| c * c).sum::<f64>().sqrt())
        .atan2(dot)
        .to_degrees()
}

/// splitmix64 (Steele, Lea & Flood 2014): fixed seed, no `rand` dependency. Uniform in [0, 1).
fn uniform(state: &mut u64) -> f64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    #[allow(clippy::cast_precision_loss)] // 53-bit mantissa draw, exact
    let u = ((z ^ (z >> 31)) >> 11) as f64 / (1u64 << 53) as f64;
    u
}

/// T-1c-1 on one segment: hourly table + linear interpolation vs direct ANISE at 1000 seeded
/// instants, max < 0.001°, Moon-centred and topocentric from IM-2 (84.7906°S, 29.1957°E, 2 m mast).
fn t_1c_1(label: &str, t0_unix_s: f64, step_s: f64, rows: usize) {
    let almanac = load_almanac(&kernels()).unwrap();
    let mut eph = Ephemeris::default();
    eph.add(
        t0_unix_s,
        step_s,
        build_table(&almanac, t0_unix_s, step_s, rows).unwrap(),
    )
    .unwrap();

    let (lat, lon) = ((-84.7906f64).to_radians(), 29.1957f64.to_radians());
    let site_km = [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()].map(|u| u * 1737.402);
    let topo = |v: &[f64]| [v[0] - site_km[0], v[1] - site_km[1], v[2] - site_km[2]];

    #[allow(clippy::cast_precision_loss)] // ≤ 43 824, exact
    let span_s = (rows - 1) as f64 * step_s;
    let mut seed = 0x0050_454E_554D_4252u64; // "PENUMBR"
    let mut worst = [0.0f64; 4]; // sun, earth Moon-centred; sun, earth topocentric
    for _ in 0..1000 {
        let t_unix_s = t0_unix_s + uniform(&mut seed) * span_s;
        let got = eph.at(t_unix_s).unwrap();
        let want = moon_me_row(&almanac, t_unix_s).unwrap();
        for (b, r) in [(0usize, 0..3), (1, 3..6)] {
            let (g, w) = (&got[r.clone()], &want[r]);
            worst[b] = worst[b].max(angle_deg([g[0], g[1], g[2]], [w[0], w[1], w[2]]));
            worst[b + 2] = worst[b + 2].max(angle_deg(topo(g), topo(w)));
        }
    }
    println!(
        "T-1c-1 {label} max error, deg: Moon-centred Sun {:.3e} Earth {:.3e}; topocentric IM-2 Sun {:.3e} Earth {:.3e}",
        worst[0], worst[1], worst[2], worst[3]
    );
    for w in worst {
        assert!(w < 0.001, "T-1c-1 {label} max error {w}° ≥ 0.001°");
    }
}

#[test]
#[ignore = "needs local kernels in PENUMBRA_DATA"]
fn t_1c_1_table_interpolation_vs_anise_1000_instants() {
    t_1c_1("main", MAIN_T0_UNIX_S, MAIN_STEP_S, MAIN_ROWS);
}

// T-1c-1 on the IM-2 replay segment (T15), 2025-02-20 to 2025-04-06 UTC.
#[test]
#[ignore = "needs local kernels in PENUMBRA_DATA"]
fn t_1c_1_replay_im2_table_interpolation_vs_anise_1000_instants() {
    t_1c_1(
        "replay-im2",
        REPLAY_IM2_T0_UNIX_S,
        REPLAY_IM2_STEP_S,
        REPLAY_IM2_ROWS,
    );
}

/// The committed `<prefix><hash6>.bin` is the table the kernels give today (1e-6 km).
fn shipped_matches_rebuild(prefix: &str, t0_unix_s: f64, step_s: f64, rows: usize) {
    let almanac = load_almanac(&kernels()).unwrap();
    let rebuilt = build_table(&almanac, t0_unix_s, step_s, rows).unwrap();
    let shipped = read_shipped_table(&shipped_data_dir(), prefix).unwrap();
    assert_eq!(shipped.len(), rebuilt.len());
    let worst_km = shipped
        .iter()
        .zip(&rebuilt)
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f64, f64::max);
    println!("{prefix}*.bin shipped vs rebuild max |Δ| {worst_km:.3e} km");
    assert!(worst_km <= 1e-6, "shipped table differs by {worst_km} km");
}

#[test]
#[ignore = "needs local kernels in PENUMBRA_DATA"]
fn shipped_table_matches_rebuild() {
    shipped_matches_rebuild(MAIN_FILE_PREFIX, MAIN_T0_UNIX_S, MAIN_STEP_S, MAIN_ROWS);
}

#[test]
#[ignore = "needs local kernels in PENUMBRA_DATA"]
fn shipped_replay_im2_table_matches_rebuild() {
    shipped_matches_rebuild(
        REPLAY_IM2_FILE_PREFIX,
        REPLAY_IM2_T0_UNIX_S,
        REPLAY_IM2_STEP_S,
        REPLAY_IM2_ROWS,
    );
}

/// (site unit × (1737.4 km + h), t_unix_s UTC) of the 20 V1 pairs in the core Horizons fixture.
fn v1_sites_and_times() -> Vec<([f64; 3], f64)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../core/tests/fixtures/horizons/v1-pairs.json");
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let f = |p: &serde_json::Value, k: &str| p[k].as_f64().unwrap();
    v.as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let (lat, lon) = (
                f(p, "lat_deg").to_radians(),
                f(p, "lon_east_deg").to_radians(),
            );
            let r_km = 1737.4 + f(p, "h_km");
            let site = [lat.cos() * lon.cos(), lat.cos() * lon.sin(), lat.sin()].map(|u| u * r_km);
            (site, f(p, "t_unix_s"))
        })
        .collect()
}

/// Largest and smallest angle (deg) between the topocentric Sun (b = 0) or Earth (b = 1) of the
/// shipped choice (MOON_ME, LT_S) and of `other`, over the 20 V1 sites and times.
fn v1_spread_deg(
    other: impl Fn(&anise::prelude::Almanac, f64) -> [f64; 6],
    b: usize,
) -> (f64, f64) {
    let almanac = load_almanac(&kernels()).unwrap();
    let (mut lo, mut hi) = (f64::INFINITY, 0.0f64);
    for (site, t_unix_s) in v1_sites_and_times() {
        let (me, alt) = (
            moon_me_row(&almanac, t_unix_s).unwrap(),
            other(&almanac, t_unix_s),
        );
        let topo = |r: &[f64; 6]| [0, 1, 2].map(|i| r[3 * b + i] - site[i]);
        let a = angle_deg(topo(&me), topo(&alt));
        (lo, hi) = (lo.min(a), hi.max(a));
    }
    (lo, hi)
}

// P3: with stellar aberration off (light time only) the Sun moves by 0.005–0.006° at every V1 pair
// (v/c ≈ 1.0e-4 rad, §1c), so the V1 fixtures can see the correction.
#[test]
#[ignore = "needs local kernels in PENUMBRA_DATA"]
fn p3_turning_aberration_off_moves_the_sun_by_0_005_to_0_006_deg() {
    let (lo, hi) = v1_spread_deg(
        |a, t| moon_row(a, t, MOON_ME_DE440_ME421_FRAME, Aberration::LT).unwrap(),
        0,
    );
    println!("P3 Sun LT vs LT_S over the V1 pairs: {lo:.5}° to {hi:.5}°");
    assert!(lo >= 0.005 && hi <= 0.006, "Sun moves {lo}..{hi}°");
}

// P8: computed in MOON_PA instead of MOON_ME, the Sun and the Earth differ by more than 0.01° at
// every V1 pair, so V1 would catch the wrong frame.
#[test]
#[ignore = "needs local kernels in PENUMBRA_DATA"]
fn p8_moon_pa_instead_of_me_differs_by_more_than_0_01_deg() {
    for (b, body) in [(0, "Sun"), (1, "Earth")] {
        let (lo, hi) = v1_spread_deg(
            |a, t| moon_row(a, t, MOON_PA_DE440_FRAME, Aberration::LT_S).unwrap(),
            b,
        );
        println!("P8 {body} PA vs ME over the V1 pairs: {lo:.5}° to {hi:.5}°");
        assert!(lo > 0.01, "{body} PA vs ME only {lo}°");
    }
}
