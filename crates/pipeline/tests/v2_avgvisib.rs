//! T20 / V2 sample against `AVGVISIB_85S_060M_201608` and its `_EARTH` twin (SCIENCE-SPEC §3 V2
//! details, P1, P2, P7, P14, U2, U3, N5). Tests 1–8 run in CI without NASA data. Test 9 reads the
//! real rasters (local only):
//! PENUMBRA_DATA=/path/to/penumbra-resource cargo test --release -p penumbra-pipeline --test v2_avgvisib -- --ignored --nocapture
//! The full run is `PENUMBRA_DATA=… cargo run --release -p penumbra-pipeline -- v2 web/public/data`.
// Exact float equality is intended where the expected value is exactly representable.
#![allow(clippy::float_cmp)]

use penumbra_core::dem::{DemTile, Terrain, TileMeta};
use penumbra_core::eph::{Ephemeris, MAIN_STEP_S, MAIN_T0_UNIX_S, Site};
use penumbra_core::geometry::MapXy;
use penumbra_core::horizon::{Horizon, N_AZ};
use penumbra_core::timeline::{DEFAULT_EARTH_MARGIN_DEG, timeline};
use penumbra_pipeline::eph::{MAIN_FILE_PREFIX, read_shipped_table};
use penumbra_pipeline::v2::{
    Candidate, average_3x3, bin_of, critical_obstacle_m, passes, shares, shift_fit, stats,
    stratified_draw,
};
use std::path::PathBuf;

// Test 1: bins of the reference value, b = min(4, floor(pct / 20)) (§3 V2 details "Sampling").
#[test]
fn bin_of_splits_the_value_range_into_fifths() {
    let got: Vec<usize> = [0.0, 19.996, 20.0, 99.99, 100.0]
        .iter()
        .map(|&p| bin_of(p).unwrap())
        .collect();
    assert_eq!(got, vec![0, 0, 1, 4, 4]);
}

#[test]
fn bin_of_rejects_negative_and_nan() {
    assert!(bin_of(-0.004).is_err());
    assert!(bin_of(f64::NAN).is_err());
}

fn candidates(per_bin: [usize; 5]) -> Vec<Candidate> {
    let mut out = Vec::new();
    for (b, &n) in per_bin.iter().enumerate() {
        for k in 0..n {
            let k = u32::try_from(k).unwrap();
            let b32 = u32::try_from(b).unwrap();
            out.push(Candidate {
                col: k % 100,
                row: b32 * 1000 + k / 100,
                #[allow(clippy::cast_precision_loss)] // b < 5
                ref_pct: 20.0 * b as f64 + 10.0,
            });
        }
    }
    out
}

fn keys(bins: &[Vec<Candidate>; 5]) -> Vec<(u32, u32)> {
    let mut k: Vec<(u32, u32)> = bins.iter().flatten().map(|c| (c.col, c.row)).collect();
    k.sort_unstable();
    k
}

// Test 2: quota 2 000 per bin; a bin with fewer takes all of them; no duplicates.
#[test]
fn stratified_draw_takes_the_quota_or_all_of_a_short_bin() {
    let bins = stratified_draw(&candidates([5000, 3000, 2000, 1999, 10]), 7, 2000).unwrap();
    let n: Vec<usize> = bins.iter().map(Vec::len).collect();
    assert_eq!(n, vec![2000, 2000, 2000, 1999, 10]);
    let k = keys(&bins);
    let mut dedup = k.clone();
    dedup.dedup();
    assert_eq!(dedup.len(), k.len());
}

#[test]
fn stratified_draw_does_not_depend_on_input_order_but_on_the_seed() {
    let c = candidates([5000, 3000, 0, 0, 0]);
    let mut shuffled = c.clone();
    shuffled.reverse();
    shuffled.rotate_left(1234);
    let a = keys(&stratified_draw(&c, 7, 2000).unwrap());
    assert_eq!(a, keys(&stratified_draw(&shuffled, 7, 2000).unwrap()));
    assert_ne!(a, keys(&stratified_draw(&c, 8, 2000).unwrap()));
}

// Test 3: e = ours − ref; median and p90 (nearest rank) of |e|, mean of e, Pearson r.
#[test]
fn stats_of_a_hand_worked_error_vector() {
    let reference = [10.0, 20.0, 30.0, 40.0, 50.0];
    let ours = [7.0, 19.0, 30.0, 42.0, 60.0];
    let s = stats(&ours, &reference).unwrap();
    assert_eq!(s.n, 5);
    assert_eq!(s.median_abs, 2.0);
    assert_eq!(s.p90_abs, 10.0);
    assert!((s.mean - 1.6).abs() < 1e-12);
}

#[test]
fn pearson_r_is_one_for_a_linear_map_and_minus_one_when_negated() {
    let reference = [0.0, 10.0, 25.0, 60.0, 90.0];
    let up: Vec<f64> = reference.iter().map(|r| 2.0 * r + 1.0).collect();
    let down: Vec<f64> = reference.iter().map(|r| -r).collect();
    assert!((stats(&up, &reference).unwrap().r - 1.0).abs() < 1e-12);
    assert!((stats(&down, &reference).unwrap().r + 1.0).abs() < 1e-12);
}

#[test]
fn pearson_r_of_a_constant_series_is_an_error_not_nan() {
    assert!(stats(&[5.0; 4], &[1.0, 2.0, 3.0, 4.0]).is_err());
    assert!(stats(&[1.0, 2.0, 3.0, 4.0], &[5.0; 4]).is_err());
}

#[test]
fn pass_rule_is_inclusive_at_median_5_mean_2_r_0_9() {
    let mut s = stats(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]).unwrap();
    (s.median_abs, s.mean, s.r) = (5.0, 2.0, 0.9);
    assert!(passes(&s));
    s.mean = -2.0;
    assert!(passes(&s));
    s.median_abs = 5.0001;
    assert!(!passes(&s));
    (s.median_abs, s.mean) = (5.0, 2.0001);
    assert!(!passes(&s));
    (s.mean, s.r) = (2.0, 0.8999);
    assert!(!passes(&s));
}

/// A 20 m tile with `counts(col, row)`, scale 1 m, offset 0 m.
fn tile(x0_m: f64, y0_m: f64, w: u32, h: u32, counts: impl Fn(u32, u32) -> i16) -> DemTile {
    let meta = TileMeta {
        width: w,
        height: h,
        x0_m,
        y0_m,
        pixel_m: 20.0,
        scale_m: 1.0,
        offset_m: 0.0,
        nodata: i16::MIN,
    };
    let c = (0..h).flat_map(|r| (0..w).map(move |c| (c, r)));
    DemTile::new(meta, c.map(|(c, r)| counts(c, r)).collect()).unwrap()
}

/// AVGVISIB pixel edges sit at −151 740 + 60k m on x and 151 740 − 60k m on y (W3c, pixel-is-area).
const X_ONE_PX_OFF_M: f64 = -151_740.0 + 6000.0 - 20.0;
const Y_ON_LATTICE_M: f64 = 151_740.0 - 6000.0;

fn centre(x0_m: f64, y0_m: f64, i: u32, j: u32) -> MapXy {
    MapXy {
        x_m: x0_m + 60.0 * (f64::from(i) + 0.5),
        y_m: y0_m - 60.0 * (f64::from(j) + 0.5),
    }
}

// Test 4: a tile whose left edge is one 20 m pixel west of the 60 m lattice starts at col 1.
#[test]
fn average_3x3_snaps_to_the_avgvisib_lattice() {
    let t = tile(X_ONE_PX_OFF_M, Y_ON_LATTICE_M, 10, 9, |_, _| 0);
    let w = average_3x3(&t).unwrap();
    assert_eq!(
        w.grid(),
        (3, 3, X_ONE_PX_OFF_M + 20.0, Y_ON_LATTICE_M, 60.0)
    );
}

#[test]
fn average_3x3_of_a_plane_is_the_centre_value() {
    // count = col + 2·row; 60 m cell (i, j) covers cols 1+3i..=3+3i, rows 3j..=3j+2: centre (2+3i, 1+3j).
    let t = tile(X_ONE_PX_OFF_M, Y_ON_LATTICE_M, 10, 9, |c, r| {
        i16::try_from(c + 2 * r).unwrap()
    });
    let w = average_3x3(&t).unwrap();
    let x0 = X_ONE_PX_OFF_M + 20.0;
    for (i, j) in [(0, 0), (1, 1), (1, 0)] {
        let want = f64::from(2 + 3 * i) + 2.0 * f64::from(1 + 3 * j);
        let got = w.height_m(centre(x0, Y_ON_LATTICE_M, i, j)).unwrap();
        assert!(
            (got - want).abs() < 1e-4,
            "cell ({i}, {j}): {got} vs {want}"
        );
    }
}

#[test]
fn average_3x3_of_a_curved_surface_is_the_mean_not_the_centre() {
    // count = col²: cell (0, 0) covers cols 1, 2, 3 → (1 + 4 + 9) / 3 = 14/3, not the centre's 4.
    let t = tile(X_ONE_PX_OFF_M, Y_ON_LATTICE_M, 10, 9, |c, _| {
        i16::try_from(c * c).unwrap()
    });
    let w = average_3x3(&t).unwrap();
    let got = w
        .height_m(centre(X_ONE_PX_OFF_M + 20.0, Y_ON_LATTICE_M, 0, 0))
        .unwrap();
    assert!((got - 14.0 / 3.0).abs() < 1e-4, "{got}");
}

#[test]
fn average_3x3_rejects_a_tile_off_the_20m_lattice() {
    let t = tile(X_ONE_PX_OFF_M + 10.0, Y_ON_LATTICE_M, 10, 9, |_, _| 0);
    assert!(average_3x3(&t).is_err());
}

fn shipped_eph() -> Ephemeris {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    let mut eph = Ephemeris::default();
    eph.add(
        MAIN_T0_UNIX_S,
        MAIN_STEP_S,
        read_shipped_table(&dir, MAIN_FILE_PREFIX).unwrap(),
    )
    .unwrap();
    eph
}

fn site() -> Site {
    Site {
        lat_rad: (-89.5f64).to_radians(),
        lon_east_rad: 0.0,
        ground_m: 0.0,
        mast_m: 0.0,
    }
}

fn flat(el_deg: f64) -> [f64; N_AZ] {
    [el_deg.to_radians(); N_AZ]
}

/// 0° for azimuths 0..180°, 3° for 180..360°.
fn step() -> [f64; N_AZ] {
    let mut h = [0.0; N_AZ];
    for (i, v) in h.iter_mut().enumerate() {
        if i >= N_AZ / 2 {
            *v = 3f64.to_radians();
        }
    }
    h
}

/// Every whole row of the shipped main table but the last: 2026-01-01 to 2030-12-31 hourly.
const STEPS: usize = 43_824;

// Test 5 (P1): visibility counts a step when any of the disc is up, so it is ≥ the mean fraction,
// and it is exactly the mean of timeline().visibility.
#[test]
fn sun_visibility_is_the_mean_of_timeline_visibility_and_not_below_illumination() {
    let eph = shipped_eph();
    for el in [flat(-1.0), flat(0.0), step()] {
        let s = shares(&eph, &el, &site(), MAIN_T0_UNIX_S, STEPS, None).unwrap();
        let tl = timeline(
            &eph,
            &el,
            &site(),
            MAIN_T0_UNIX_S,
            MAIN_STEP_S,
            u32::try_from(STEPS).unwrap(),
            DEFAULT_EARTH_MARGIN_DEG.to_radians(),
        )
        .unwrap();
        #[allow(clippy::cast_precision_loss)] // 43 824 steps, exact
        let n = STEPS as f64;
        assert_eq!(s.sun_vis_pct, 100.0 * tl.visibility.iter().sum::<f64>() / n);
        assert!(s.sun_vis_pct >= s.sun_illum_pct);
        assert!(s.sun_vis_pct > 0.0);
    }
}

// Test 5 (P14): the Earth twin is "any fraction visible", which is at least the share with the
// centre above the horizon, which is at least the comms share with the 2° margin.
#[test]
fn earth_any_fraction_share_is_at_least_the_centre_and_the_margin_shares() {
    let eph = shipped_eph();
    for el in [flat(-1.0), flat(0.0), step()] {
        let s = shares(&eph, &el, &site(), MAIN_T0_UNIX_S, STEPS, None).unwrap();
        let tl = timeline(
            &eph,
            &el,
            &site(),
            MAIN_T0_UNIX_S,
            MAIN_STEP_S,
            u32::try_from(STEPS).unwrap(),
            DEFAULT_EARTH_MARGIN_DEG.to_radians(),
        )
        .unwrap();
        #[allow(clippy::cast_precision_loss)] // counts < 2^52
        let pct = |k: usize| 100.0 * k as f64 / STEPS as f64;
        let centre_up = pct(tl.earth_margin_deg.iter().filter(|&&m| m >= 0.0).count());
        let contact = pct(tl.contact.iter().filter(|&&c| c > 0.0).count());
        // Strict: Earth's limb crosses these horizons, so a centre-above-horizon twin would fail here.
        assert!(
            s.earth_vis_pct > centre_up,
            "{} <= {centre_up}",
            s.earth_vis_pct
        );
        assert!(centre_up >= contact);
        assert!(
            s.earth_vis_pct > contact,
            "any fraction should beat a 2° margin"
        );
    }
}

#[test]
fn a_10_degree_wall_hides_both_sun_and_earth_at_89_5_s() {
    let s = shares(
        &shipped_eph(),
        &flat(10.0),
        &site(),
        MAIN_T0_UNIX_S,
        STEPS,
        None,
    )
    .unwrap();
    assert_eq!((s.sun_vis_pct, s.earth_vis_pct), (0.0, 0.0));
}

fn horizon(el_deg: f64, dist_m: f64) -> Horizon {
    Horizon {
        el_rad: [el_deg.to_radians(); N_AZ],
        dist_m: [dist_m; N_AZ],
    }
}

// Test 6 (N5): the critical obstacle is the nearest one among bins whose horizon lies in the
// band the Sun's disc crosses; a near bin outside the band does not count.
#[test]
fn critical_obstacle_is_the_nearest_bin_inside_the_band() {
    let mut h = horizon(1.0, 500.0);
    h.dist_m[7] = 60.0;
    assert_eq!(critical_obstacle_m(&h, -2.0, 3.0), Some(60.0));
}

#[test]
fn a_near_obstacle_outside_the_band_is_ignored() {
    let mut h = horizon(1.0, 500.0);
    (h.el_rad[7], h.dist_m[7]) = (10f64.to_radians(), 60.0);
    assert_eq!(critical_obstacle_m(&h, -2.0, 3.0), Some(500.0));
}

#[test]
fn no_bin_in_the_band_gives_none() {
    assert_eq!(critical_obstacle_m(&horizon(10.0, 60.0), -2.0, 3.0), None);
}

// Test 7 (P7): a map shifted by δ leaves e ≈ δ · ∇ref; OLS on the reference gradient recovers δ.
fn edge(x_m: f64, y_m: f64) -> f64 {
    50.0 + 40.0 * (x_m / 200.0).tanh() + 5.0 * (y_m / 300.0).sin()
}

fn edge_grad(x_m: f64, y_m: f64) -> (f64, f64) {
    let c = (x_m / 200.0).cosh();
    (40.0 / 200.0 / (c * c), 5.0 / 300.0 * (y_m / 300.0).cos())
}

fn shift_case(dx_m: f64) -> (f64, f64, f64) {
    let (mut e, mut gx, mut gy) = (Vec::new(), Vec::new(), Vec::new());
    for i in -30..=30 {
        for j in -30..=30 {
            let (x, y) = (f64::from(i) * 20.0, f64::from(j) * 40.0);
            e.push(edge(x + dx_m, y) - edge(x, y));
            let g = edge_grad(x, y);
            gx.push(g.0);
            gy.push(g.1);
        }
    }
    shift_fit(&e, &gx, &gy).unwrap()
}

#[test]
fn shift_fit_recovers_a_30m_east_shift() {
    let (_, dx, dy) = shift_case(30.0);
    assert!((dx - 30.0).abs() < 3.0, "δx {dx}");
    assert!(dy.abs() < 3.0, "δy {dy}");
}

#[test]
fn shift_fit_of_an_unshifted_map_is_zero() {
    let (_, dx, dy) = shift_case(0.0);
    assert!(dx.abs() < 1e-9 && dy.abs() < 1e-9, "{dx} {dy}");
}

// Test 8 (U2): both windows from one long track equal two separate calls.
#[test]
fn shifted_window_from_one_track_equals_a_separate_call() {
    let eph = shipped_eph();
    let (n, shift) = (2000, 700);
    let both = shares(&eph, &step(), &site(), MAIN_T0_UNIX_S, n, Some(shift)).unwrap();
    let first = shares(&eph, &step(), &site(), MAIN_T0_UNIX_S, n, None).unwrap();
    #[allow(clippy::cast_precision_loss)] // 700 steps
    let later_t0 = MAIN_T0_UNIX_S + shift as f64 * MAIN_STEP_S;
    let later = shares(&eph, &step(), &site(), later_t0, n, None).unwrap();
    assert_eq!(both.sun_vis_pct, first.sun_vis_pct);
    assert_eq!(both.earth_vis_pct, first.earth_vis_pct);
    assert_eq!(both.sun_vis_shifted_pct, Some(later.sun_vis_pct));
    assert_ne!(first.sun_vis_pct, later.sun_vis_pct, "windows must differ");
}

// Test 9: header and spot values of the real rasters, read Nov 15 with gdalinfo-free tiff 0.11.3.
// Panics when PENUMBRA_DATA is unset so --ignored never passes empty.
#[test]
#[ignore = "needs AVGVISIB_85S_060M_201608*.TIF in PENUMBRA_DATA (not in CI)"]
fn avgvisib_headers_and_spot_counts() {
    use penumbra_pipeline::dem::GeoTiff;
    let dir = PathBuf::from(std::env::var("PENUMBRA_DATA").expect("set PENUMBRA_DATA"));
    let mut sun = GeoTiff::open(&dir.join("AVGVISIB_85S_060M_201608.TIF")).unwrap();
    assert_eq!((sun.width, sun.height, sun.pixel_m), (5058, 5058, 60.0));
    assert_eq!((sun.x0_m, sun.y0_m), (-151_740.0, 151_740.0));
    assert!(sun.pixel_is_area);
    assert_eq!(sun.sphere_radius_m, Some(1_737_400.0));
    assert_eq!(sun.nodata.as_deref(), Some("-32768"));
    let at = |g: &mut GeoTiff, c, r| g.read_window(c, r, 1, 1).unwrap()[0];
    assert_eq!(at(&mut sun, 2340, 2732), 21_605.0);
    assert_eq!(at(&mut sun, 2340, 2733), 0.0);
    assert_eq!(at(&mut sun, 2529, 2529), 6_880.0);
    assert_eq!(at(&mut sun, 2349, 2724), 22_111.0);
    let mut earth = GeoTiff::open(&dir.join("AVGVISIB_85S_060M_201608_EARTH.TIF")).unwrap();
    assert_eq!((earth.width, earth.height), (5058, 5058));
    assert_eq!(at(&mut earth, 2340, 2732), 14_485.0);
    assert_eq!(at(&mut earth, 2529, 2529), 7_872.0);
}
