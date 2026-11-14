//! T06 checks against the NASA GeoTIFFs: U5/U6 tags and the window reader. Round trip and PSR margin are in dem_tiles.rs.
//! Run: PENUMBRA_DATA=/path/to/penumbra-resource cargo test -p penumbra-pipeline --release --test dem_source -- --ignored --nocapture

use penumbra_pipeline::dem::GeoTiff;
use std::path::PathBuf;

fn data(name: &str) -> PathBuf {
    PathBuf::from(std::env::var("PENUMBRA_DATA").expect("set PENUMBRA_DATA to the download folder"))
        .join(name)
}

// U5 (SCIENCE-SPEC §5, read Nov 12): tiepoint UL (−304 000, +304 000) m, 20 m, pixel-is-area,
// no-data NaN. U6: the datum is a sphere of 1737.4 km (E30), so heights are above that sphere.
#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
fn u5_u6_tags_of_the_20m_dem() {
    let g = GeoTiff::open(&data("LDEM_80S_20MPP_ADJ.TIF")).unwrap();
    println!(
        "{:?} {:?} {:?}",
        (g.x0_m, g.y0_m, g.pixel_m, g.pixel_is_area),
        g.sphere_radius_m,
        g.citation
    );
    assert_eq!((g.width, g.height), (30_400, 30_400));
    assert!((g.x0_m + 304_000.0).abs() < 1e-6 && (g.y0_m - 304_000.0).abs() < 1e-6);
    assert!((g.pixel_m - 20.0).abs() < 1e-9);
    assert!(g.pixel_is_area);
    assert_eq!(g.nodata.as_deref(), Some("nan"));
    assert_eq!(g.sphere_radius_m, Some(1_737_400.0));
}

// Far source: ±931.2 km at 240 m (DATA-NOTES), same datum.
#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
fn u5_u6_tags_of_the_240m_dem() {
    let g = GeoTiff::open(&data("LDEM_60S_240MPP_ADJ.TIF")).unwrap();
    println!(
        "{:?} {:?} {:?}",
        (g.x0_m, g.y0_m, g.pixel_m, g.pixel_is_area),
        g.sphere_radius_m,
        g.citation
    );
    assert!((g.x0_m + 931_200.0).abs() < 1e-6 && (g.y0_m - 931_200.0).abs() < 1e-6);
    assert!((g.pixel_m - 240.0).abs() < 1e-9);
    assert!(g.pixel_is_area);
    assert_eq!(g.sphere_radius_m, Some(1_737_400.0));
}

// A window across tile boundaries equals the same pixels read one raw `read_chunk` at a time.
#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
fn window_matches_raw_chunks_across_tile_seams() {
    use tiff::decoder::{Decoder, DecodingResult};
    let path = data("LDEM_80S_20MPP_ADJ.TIF");
    let mut g = GeoTiff::open(&path).unwrap();
    // 510..515 straddles the 512-px seam both ways.
    let win = g.read_window(14_846, 14_846, 6, 6).unwrap();
    let mut d = Decoder::new(std::io::BufReader::new(std::fs::File::open(&path).unwrap())).unwrap();
    for r in 0..6u32 {
        for c in 0..6u32 {
            let (col, row) = (14_846 + c, 14_846 + r);
            let index = (row / 512) * 60 + col / 512;
            let DecodingResult::F32(px) = d.read_chunk(index).unwrap() else {
                panic!("Float32")
            };
            let want = px[((row % 512) * 512 + col % 512) as usize];
            assert_eq!(
                win[(r * 6 + c) as usize].to_bits(),
                want.to_bits(),
                "({col}, {row})"
            );
        }
    }
}
