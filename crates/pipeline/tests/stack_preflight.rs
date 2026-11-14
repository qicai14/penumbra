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

#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
#[expect(clippy::float_cmp, reason = "the mask holds exact 0.0 and 1.0")]
fn tiff_decodes_one_psr_tile() {
    use tiff::decoder::{Decoder, DecodingResult};
    fn tile(name: &str, index: Option<u32>) -> (u32, Vec<f32>, (u32, u32)) {
        let mut d = Decoder::new(std::io::BufReader::new(File::open(data(name)).unwrap())).unwrap();
        assert_eq!(d.dimensions().unwrap(), (30400, 30400));
        assert_eq!(d.chunk_dimensions(), (512, 512));
        assert_eq!(d.tile_count().unwrap(), 60 * 60);
        // Tile holding the centre pixel (15 200, 15 200) unless told otherwise.
        let index = index.unwrap_or(29 * 60 + 29);
        let t = std::time::Instant::now();
        // read_chunk reads only this tile's bytes, never the 3.7 GB raster.
        let DecodingResult::F32(px) = d.read_chunk(index).unwrap() else {
            panic!("expected Float32 samples")
        };
        println!("{name} tile {index}: {} ms", t.elapsed().as_millis());
        (index, px, d.chunk_data_dimensions(index))
    }

    let (_, px, dims) = tile("LPSR_80S_20MPP_ADJ.TIF", None);
    assert_eq!((px.len(), dims), (512 * 512, (512, 512)));
    assert!(px.iter().all(|v| v.is_finite()), "centre tile has no-data");
    // A wrong floating-point predictor (3) undo would scramble bytes into non-mask floats.
    assert!(
        px.iter().all(|&v| v == 0.0 || v == 1.0),
        "PSR tile is not a 0/1 mask"
    );
    println!(
        "  {} of {} in PSR",
        px.iter().filter(|&&v| v == 1.0).count(),
        px.len()
    );

    // Edge tile: 30 400 − 59 × 512 = 192 px of data each way.
    let (_, px, dims) = tile("LPSR_80S_20MPP_ADJ.TIF", Some(3599));
    assert_eq!(dims, (192, 192));
    println!("  last tile buffer {} samples for {dims:?} data", px.len());

    // Same codec on the heights file T06 needs: values must be plausible metres.
    let (_, px, _) = tile("LDEM_80S_20MPP_ADJ.TIF", None);
    let (lo, hi) = px
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    println!("  LDEM centre tile heights {lo} .. {hi} m");
    assert!(px.iter().all(|v| v.is_finite()) && lo > -10_000.0 && hi < 10_000.0);
}
