//! T06 int16 tile encoder (ARCHITECTURE §2.1, SCIENCE-SPEC §1a tile encoding, P5, P11). No NASA data.

use penumbra_core::dem::{DemTile, TileMeta};
use penumbra_core::geometry::MapXy;
use penumbra_pipeline::dem::{FAR_SCALES_M, MIN_SCALE_M, NODATA, encode, encode_finest};

/// Heights spanning exactly `range_m`, from −range/2 + 100 to +range/2 + 100.
fn span(range_m: f32) -> Vec<f32> {
    vec![100.0 - range_m / 2.0, 100.0, 100.0 + range_m / 2.0]
}

// P11, #48: the near scale is the finest that holds the tile, so the lowest and highest heights
// land on −32 767 and +32 767 (65 534 steps), and the far scale stays 0.5 m.
#[test]
fn near_scale_is_the_finest_that_fits() {
    for range_m in [4000.0, 5000.0, 10_000.0] {
        let e = encode_finest(&span(range_m)).unwrap();
        assert_eq!(e.counts, vec![-32_767, 0, 32_767], "{range_m} m");
    }
}

// SCIENCE-SPEC §1a: 0.25 m or finer, so 17 000 m (68 000 counts at 0.25 m) fails instead of
// taking a coarser scale.
#[test]
fn near_encoder_fails_when_the_coarsest_scale_clips() {
    assert!(encode_finest(&span(17_000.0)).is_err());
}

// A level tile keeps a positive scale (the 1 mm floor) and decodes to its height exactly.
#[test]
fn level_tile_takes_the_floor_scale() {
    let e = encode_finest(&[812.5; 4]).unwrap();
    assert!((e.scale_m - MIN_SCALE_M).abs() < 1e-15);
    assert!(e.counts.iter().all(|&c| c == 0) && (e.offset_m - 812.5).abs() < 1e-12);
}

// Far tile is 0.5 m: 32 000 m fits (64 000 steps), 33 000 m clips (66 000).
#[test]
fn far_encoder_uses_half_metre_and_fails_when_it_clips() {
    let e = encode(&span(32_000.0), &FAR_SCALES_M).unwrap();
    assert!((e.scale_m - 0.5).abs() < 1e-12);
    assert!(encode(&span(33_000.0), &FAR_SCALES_M).is_err());
}

#[test]
fn nan_source_pixels_become_the_nodata_count() {
    let e = encode_finest(&[1.0, f32::NAN, 3.0]).unwrap();
    assert_eq!(e.counts[1], NODATA);
    assert!(e.counts[0] != NODATA && e.counts[2] != NODATA);
}

#[test]
fn encoder_fails_when_no_pixel_has_a_height() {
    assert!(encode_finest(&[f32::NAN, f32::NAN]).is_err());
}

// Decoded through the core sampler at pixel centres, every height is within half a scale step.
#[test]
fn round_trip_through_the_sampler_is_within_half_a_step() {
    let (w, h) = (40u32, 30u32);
    // A deterministic bumpy surface over 7.9 km of relief → 7 900 / 65 534 ≈ 0.1205 m.
    let src: Vec<f32> = (0..w * h)
        .map(|k| {
            let (c, r) = (f64::from(k % w), f64::from(k / w));
            #[allow(clippy::cast_possible_truncation)] // test heights are within f32 range
            let v = (3950.0 * (0.37 * c).sin() * (0.23 * r).cos() - 812.3) as f32;
            v
        })
        .collect();
    let e = encode_finest(&src).unwrap();
    assert!(e.scale_m > 0.12 && e.scale_m < 0.121, "{}", e.scale_m);
    let meta = TileMeta {
        width: w,
        height: h,
        x0_m: -1000.0,
        y0_m: 500.0,
        pixel_m: 20.0,
        scale_m: e.scale_m,
        offset_m: e.offset_m,
        nodata: NODATA,
    };
    let tile = DemTile::new(meta, e.counts).unwrap();
    for r in 0..h - 1 {
        for c in 0..w - 1 {
            let xy = MapXy {
                x_m: -1000.0 + (f64::from(c) + 0.5) * 20.0,
                y_m: 500.0 - (f64::from(r) + 0.5) * 20.0,
            };
            let got = tile.height_m(xy).unwrap();
            let want = f64::from(src[(r * w + c) as usize]);
            assert!(
                (got - want).abs() <= e.scale_m / 2.0 + 1e-9,
                "({c}, {r}): {got} vs {want}"
            );
        }
    }
}
