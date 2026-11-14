//! T19 / V2b longest-night table check (SCIENCE-SPEC §3 V2b, U20, P12; brief docs/briefs/T19-table-check.md).
//! Tests 1–4 of the brief run in CI (no NASA data). The real run (release; kernels in PENUMBRA_DATA):
//! PENUMBRA_DATA=/path/to/penumbra-resource cargo test --release -p penumbra-pipeline --test v2b_table -- --ignored --nocapture

use penumbra_core::geometry::MapXy;
use penumbra_pipeline::table_check::{
    SITE1_ROIS, SP01_RADIUS_M, SP01_XY_KM, pixel_runs, pixels_in_disc, roi_mean,
};

// Brief test 1: lit means f > 0 (Barker LCIP), dark f == 0 (LCSP), full day f == 1, partial night f < 1.
#[test]
fn thresholds_split_partial_disc_steps_as_the_definitions_say() {
    let f = [
        1.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.2, 1.0, 1.0, 1.0, 1.0, 0.0, 0.0,
    ];
    let r = pixel_runs(&f, 1.0);
    assert_eq!(
        (
            r.lit_h.length_h,
            r.dark_h.length_h,
            r.full_day_h.length_h,
            r.partial_night_h.length_h
        ),
        (5.0, 4.0, 4.0, 6.0)
    );
}

// Brief test 2: average per-pixel run lengths, never take runs on a mean series (b5251ef3:549, 1207–1209).
// A mean series of these two pixels is 0.5 everywhere: 12 h lit, 0 h dark, which is wrong.
#[test]
fn roi_mean_averages_per_pixel_runs_not_a_mean_series() {
    let a: Vec<f64> = [1.0, 0.0, 1.0].iter().flat_map(|&v| [v; 4]).collect();
    let b: Vec<f64> = [0.0, 1.0, 0.0].iter().flat_map(|&v| [v; 4]).collect();
    let m = roi_mean(&[pixel_runs(&a, 1.0), pixel_runs(&b, 1.0)]).unwrap();
    assert_eq!((m.lcip_h, m.lcsp_h), (4.0, 4.0));
}

#[test]
fn roi_mean_of_no_pixels_is_an_error() {
    assert!(roi_mean(&[]).is_err());
}

// Brief test 3: a run cut off by the window counts at its length inside the window and is flagged.
#[test]
fn a_run_touching_the_window_edge_is_flagged() {
    let f = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0];
    let r = pixel_runs(&f, 1.0);
    assert_eq!(r.dark_h.length_h, 3.0);
    assert!(r.dark_h.touches_edge);
}

#[test]
fn an_interior_run_is_not_flagged() {
    let f = [1.0, 0.0, 0.0, 1.0];
    assert!(!pixel_runs(&f, 1.0).dark_h.touches_edge);
}

// Brief test 4: pixel counts on the 20 m grid (centres at −303 990 + 20k m), d² ≤ r² inclusive.
#[test]
fn site1_roi_discs_hold_the_expected_pixel_counts() {
    let counts: Vec<usize> = SITE1_ROIS
        .iter()
        .map(|roi| pixels_in_disc(roi.centre(), roi.radius_m()).len())
        .collect();
    assert_eq!(counts, vec![20, 18, 9, 35, 26, 26, 19]);
}

#[test]
fn sp01_150m_disc_holds_180_pixels_including_the_six_on_the_rim() {
    let c = MapXy {
        x_m: SP01_XY_KM.0 * 1000.0,
        y_m: SP01_XY_KM.1 * 1000.0,
    };
    assert_eq!(pixels_in_disc(c, SP01_RADIUS_M).len(), 180);
}

// V2b, the real run. Panics when PENUMBRA_DATA is unset so --ignored never passes empty.
#[test]
#[ignore = "needs the kernels in PENUMBRA_DATA (not in CI); run with --release"]
fn v2b_real_run() {
    let dir = std::env::var("PENUMBRA_DATA").expect("set PENUMBRA_DATA to the download folder");
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    let r = penumbra_pipeline::table_check::run(&data, std::path::Path::new(&dir)).unwrap();
    println!("{}", serde_json::to_string_pretty(&r).unwrap());
}
