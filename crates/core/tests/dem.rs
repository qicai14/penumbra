//! T06 DEM sampler (ARCHITECTURE §2.1, SCIENCE-SPEC §1a pixel convention [U5], P5).

use penumbra_core::dem::{DemError, DemTile, TileMeta};
use penumbra_core::geometry::MapXy;

/// 4 × 3 tile, counts = col + 10·row, so heights are a plane and bilinear is exact.
fn plane_tile(nodata_at: Option<usize>) -> DemTile {
    let meta = TileMeta {
        width: 4,
        height: 3,
        x0_m: 1000.0,
        y0_m: 2000.0,
        pixel_m: 20.0,
        scale_m: 0.25,
        offset_m: 100.0,
        nodata: i16::MIN,
    };
    let mut counts: Vec<i16> = (0..3i16)
        .flat_map(|row| (0..4i16).map(move |col| col + 10 * row))
        .collect();
    if let Some(i) = nodata_at {
        counts[i] = i16::MIN;
    }
    DemTile::new(meta, counts).unwrap()
}

fn xy(x_m: f64, y_m: f64) -> MapXy {
    MapXy { x_m, y_m }
}

// Pixel-is-area [U5]: centre of (col, row) is x0 + (col + 0.5)·Δ, y0 − (row + 0.5)·Δ.
// (1035, 1980) is col 1.25, row 0.5 → h = 100 + 0.25·(1.25 + 10·0.5) = 101.5625 m.
#[test]
fn sampler_is_bilinear_on_pixel_centres() {
    let h = plane_tile(None).height_m(xy(1035.0, 1980.0)).unwrap();
    assert!((h - 101.5625).abs() < 1e-9, "got {h}");
}

// Pixel (2, 1) centre is (1050, 1970): h = 100 + 0.25·12 = 103 m.
#[test]
fn sampler_returns_the_pixel_value_at_its_centre() {
    let h = plane_tile(None).height_m(xy(1050.0, 1970.0)).unwrap();
    assert!((h - 103.0).abs() < 1e-9, "got {h}");
}

// P5: outside coverage is an error, never 0 m. The last valid x is the last pixel centre (1070).
#[test]
fn sampler_refuses_points_outside_coverage() {
    let t = plane_tile(None);
    for p in [
        xy(1070.0, 1980.0),
        xy(1009.9, 1980.0),
        xy(1035.0, 1990.1),
        xy(1035.0, 1950.0),
        xy(-5.0e5, 0.0),
    ] {
        assert_eq!(t.height_m(p), Err(DemError::OutOfCoverage), "{p:?}");
    }
}

// P5: a no-data neighbour is an error, never 0 m and never a partial average.
#[test]
fn sampler_refuses_cells_touching_nodata() {
    // counts index 6 = (col 2, row 1)
    let t = plane_tile(Some(6));
    assert_eq!(t.height_m(xy(1035.0, 1980.0)), Err(DemError::NoData));
    // cell (0, 0)..(1, 1) does not touch it
    assert!(t.height_m(xy(1015.0, 1985.0)).is_ok());
}

#[test]
fn sampler_refuses_non_finite_points() {
    assert!(matches!(
        plane_tile(None).height_m(xy(f64::NAN, 1980.0)),
        Err(DemError::BadInput(_))
    ));
}

#[test]
fn tile_refuses_counts_that_do_not_match_its_size() {
    let meta = *plane_tile(None).meta();
    assert!(matches!(
        DemTile::new(meta, vec![0; 11]),
        Err(DemError::BadInput(_))
    ));
}

#[test]
fn error_codes_match_the_api() {
    assert_eq!(DemError::OutOfCoverage.code(), "OUT_OF_COVERAGE");
    assert_eq!(DemError::NoData.code(), "NO_DATA");
    assert_eq!(DemError::BadInput("x").code(), "BAD_INPUT");
}

// Early-exit bound (docs/API.md load_far finds h_max): highest count 23 (col 3, row 2) →
// 100 + 0.25·23 = 105.75 m; with that pixel nodata the next is 22 → 105.5 m.
#[test]
fn max_height_skips_nodata() {
    assert!((plane_tile(None).max_height_m() - 105.75).abs() < 1e-12);
    assert!((plane_tile(Some(11)).max_height_m() - 105.5).abs() < 1e-12);
}

#[test]
fn all_nodata_tile_is_refused() {
    let meta = *plane_tile(None).meta();
    assert!(matches!(
        DemTile::new(meta, vec![i16::MIN; 12]),
        Err(DemError::BadInput(_))
    ));
}
