//! Pre-event stack checks: can our crates open the NASA files at all?
//! Format level only: no positions, angles or heights are computed here.
//! Run: PENUMBRA_DATA=/path/to/penumbra-resource cargo test -p penumbra-pipeline --test stack_preflight -- --ignored --nocapture

use std::{fs::File, io::Read, path::PathBuf};

fn data(name: &str) -> String {
    let dir = std::env::var("PENUMBRA_DATA").expect("set PENUMBRA_DATA to the download folder");
    PathBuf::from(dir).join(name).to_string_lossy().into_owned()
}

#[test]
#[ignore = "needs local kernels in PENUMBRA_DATA"]
fn anise_loads_the_three_kernels() {
    use anise::prelude::Almanac;
    let almanac = Almanac::default()
        .load(&data("de440.bsp"))
        .and_then(|a| a.load(&data("moon_pa_de440_200625.bpc")))
        .and_then(|a| a.load(&data("moon_fk_de440.epa")))
        .expect("all three kernels load with default-features = false");

    assert_eq!(almanac.num_loaded_spk(), 1);
    assert_eq!(almanac.num_loaded_bpc(), 1);
    let mut spk: Vec<_> = almanac.spk_domains().unwrap().into_iter().collect();
    spk.sort_by_key(|(id, _)| *id);
    for (id, (start, end)) in spk {
        println!("SPK body {id}: {start} .. {end}");
    }
    for (id, (start, end)) in almanac.bpc_domains().unwrap() {
        println!("BPC frame {id}: {start} .. {end}");
    }
    almanac.describe(None, None, None, None, Some(true), None, None, None);
}

#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
fn tiff_reads_dem_headers() {
    use tiff::{decoder::Decoder, tags::Tag};
    for name in [
        "LPSR_80S_20MPP_ADJ.TIF",
        "LDEM_80S_20MPP_ADJ.TIF",
        "LDEM_60S_240MPP_ADJ.TIF",
    ] {
        let path = data(name);
        let mut magic = [0u8; 4];
        File::open(&path).unwrap().read_exact(&mut magic).unwrap();
        // 42 = classic TIFF, 43 = BigTIFF (little-endian "II")
        println!("\n{name}: magic {magic:?} bigtiff={}", magic[2] == 43);

        let mut d = Decoder::new(std::io::BufReader::new(File::open(&path).unwrap()))
            .expect("tiff crate opens the file");
        println!(
            "  dimensions {:?} colortype {:?}",
            d.dimensions().unwrap(),
            d.colortype()
        );
        println!("  chunk {:?}", d.chunk_dimensions());
        for tag in [
            Tag::Compression,
            Tag::Predictor,
            Tag::BitsPerSample,
            Tag::SampleFormat,
            Tag::PlanarConfiguration,
            Tag::TileWidth,
            Tag::TileLength,
            Tag::RowsPerStrip,
            Tag::ModelPixelScaleTag,
            Tag::ModelTiepointTag,
            Tag::GeoKeyDirectoryTag,
            Tag::Unknown(42113), // GDAL_NODATA
        ] {
            println!("  {tag:?}: {:?}", d.find_tag(tag).ok().flatten());
        }
    }
}
