//! V5 case 3 (SCIENCE-SPEC §3, U15, E132, E139): 20 pixels well inside the `LPSR_80S_20MPP_ADJ.TIF`
//! mask see no sunlight at any hourly step of 18.6 years. The vectors are generated natively and not shipped.
//! Full run (release; needs the kernels and GeoTIFFs in PENUMBRA_DATA):
//! PENUMBRA_DATA=/path/to/penumbra-resource cargo test --release -p penumbra-pipeline --test v5_psr -- --ignored --nocapture

use penumbra_pipeline::psr::{interior_points, pick_spread, run};

/// 7 × 7 mask: a 5 × 5 block of 1s (cols 1..=5, rows 1..=5) in a ring of 0s.
fn mask() -> Vec<f32> {
    (0..49)
        .map(|k| {
            let (c, r) = (k % 7, k / 7);
            f32::from(u8::from((1..=5).contains(&c) && (1..=5).contains(&r)))
        })
        .collect()
}

#[test]
fn interior_points_keeps_only_pixels_whose_whole_square_is_flagged() {
    // half 1 = 3 × 3 square: only the middle of the 5 × 5 block (3, 3) and its 8 neighbours' squares
    // still inside the block, i.e. centres 2..=4 on both axes; stride 1 returns all nine.
    let p = interior_points(&mask(), 7, 7, 1, 1);
    assert_eq!(p.len(), 9);
    assert!(
        p.iter()
            .all(|&(c, r)| (2..=4).contains(&c) && (2..=4).contains(&r))
    );
    // half 2: only the centre (3, 3) has a 5 × 5 square of 1s.
    assert_eq!(interior_points(&mask(), 7, 7, 2, 1), vec![(3, 3)]);
    // half 3: no 7 × 7 square of 1s exists.
    assert!(interior_points(&mask(), 7, 7, 3, 1).is_empty());
}

#[test]
fn interior_points_honours_the_stride_and_never_reads_off_the_raster() {
    // The grid starts at half = 1, so stride 3 visits 1 and 4 on each axis: only (4, 4) is inside the block.
    let p = interior_points(&mask(), 7, 7, 1, 3);
    assert_eq!(p, vec![(4, 4)]);
}

#[test]
fn a_nan_pixel_is_not_flagged() {
    let mut m = mask();
    m[3 * 7 + 3] = f32::NAN;
    assert!(interior_points(&m, 7, 7, 1, 1).is_empty());
}

#[test]
fn pick_spread_takes_evenly_spaced_entries_in_order() {
    let all: Vec<u32> = (0..100).collect();
    assert_eq!(pick_spread(&all, 4), vec![0, 25, 50, 75]);
    assert_eq!(pick_spread(&all[..3], 20), vec![0, 1, 2]);
}

// V5 case 3, the real run. Panics when PENUMBRA_DATA is unset so --ignored never passes empty.
#[test]
#[ignore = "needs the kernels and LPSR/LDEM GeoTIFFs in PENUMBRA_DATA (not in CI); run with --release"]
fn v5_case3_twenty_well_inside_psr_pixels_are_dark_for_18_6_years() {
    let dir = std::env::var("PENUMBRA_DATA").expect("set PENUMBRA_DATA to the download folder");
    let r = run(std::path::Path::new(&dir)).unwrap();
    println!(
        "V5 case 3: {} sites x {} hourly steps from {} (unix s UTC)",
        r.sites.len(),
        r.steps,
        r.start_unix_s
    );
    for s in &r.sites {
        println!(
            "  px ({}, {}) lat {:.4} lon {:.4} ground {:.1} m: lit steps {}, max illumination {:.3e}, closest Sun-limb approach to horizon {:+.4} deg",
            s.col,
            s.row,
            s.lat_deg,
            s.lon_east_deg,
            s.ground_m,
            s.lit_steps,
            s.max_illumination,
            s.min_gap_deg
        );
    }
    let lit: usize = r.sites.iter().map(|s| s.lit_steps).sum();
    println!("V5 case 3: total lit steps {lit}");
    assert_eq!(r.sites.len(), 20);
    assert_eq!(lit, 0, "visibility must be exactly 0 at every step");
}
