//! T77: the GeoTIFF read paths on tiny synthetic files written here, so CI covers them without
//! PENUMBRA_DATA. The NASA layout these mimic: Deflate (8) + horizontal predictor (2), pixel-is-area,
//! on the 1 737 400 m sphere (DATA-NOTES). Int16 = AVGVISIB counts; Float32 = LDEM heights in metres.

use penumbra_core::geometry::{MOON_RADIUS_M, MapXy};
use penumbra_pipeline::dem::{FloatWindow, GeoTiff, build_tile, encode_finest};
use tiff::encoder::{Compression, DeflateLevel, Predictor, TiffEncoder, colortype};
use tiff::tags::Tag;

const W: u32 = 5;
const H: u32 = 7;
const PX_M: f64 = 20.0;
/// Upper-left corner of pixel (0, 0), map metres: the raster spans x −50..50, y −70..70.
const X0_M: f64 = -50.0;
const Y0_M: f64 = 70.0;

/// Row-major int16 counts with sign changes and the int16 extremes, so a predictor-2 delta wraps.
fn counts() -> Vec<i16> {
    (0..W * H)
        .map(|k| match k {
            3 => i16::MIN,
            4 => i16::MAX,
            _ => i16::try_from(i64::from(k) * 997 - 17_000).unwrap(),
        })
        .collect()
}

fn path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("penumbra-t77-{}-{name}.tif", std::process::id()))
}

/// Small counts (0..35) that would encode as a valid tile if read as metres.
fn small_counts() -> Vec<i16> {
    (0..W * H).map(|k| i16::try_from(k).unwrap()).collect()
}

/// Write a W × H GeoTIFF of `data` in strips of 3 rows (so the last strip is short, 1 row).
/// `predictor` is Horizontal (2) for int16; the encoder has no float predictor (3), so None for f32.
fn write<C: colortype::ColorType>(
    name: &str,
    data: &[C::Inner],
    predictor: Predictor,
) -> std::path::PathBuf
where
    [C::Inner]: tiff::encoder::TiffValue,
{
    let p = path(name);
    let file = std::fs::File::create(&p).unwrap();
    let mut enc = TiffEncoder::new(file)
        .unwrap()
        .with_compression(Compression::Deflate(DeflateLevel::Balanced))
        .with_predictor(predictor);
    let mut img = enc.new_image::<C>(W, H).unwrap();
    img.rows_per_strip(3).unwrap();
    let d = img.encoder();
    d.write_tag(Tag::ModelPixelScaleTag, &[PX_M, PX_M, 0.0][..])
        .unwrap();
    d.write_tag(Tag::ModelTiepointTag, &[0.0, 0.0, 0.0, X0_M, Y0_M, 0.0][..])
        .unwrap();
    // header (version 1.1.0, 2 keys); 1025 RasterPixelIsArea; 2057 semi-major axis → doubles[0]
    let keys: [u16; 12] = [1, 1, 0, 2, 1025, 0, 1, 1, 2057, 34736, 1, 0];
    d.write_tag(Tag::GeoKeyDirectoryTag, &keys[..]).unwrap();
    d.write_tag(Tag::GeoDoubleParamsTag, &[MOON_RADIUS_M][..])
        .unwrap();
    img.write_data(data).unwrap();
    p
}

#[test]
fn synthetic_int16_file_is_deflate_with_predictor_2() {
    let p = write::<colortype::GrayI16>("layout", &counts(), Predictor::Horizontal);
    let mut d = tiff::decoder::Decoder::new(std::fs::File::open(&p).unwrap()).unwrap();
    let tag = |d: &mut tiff::decoder::Decoder<_>, t| d.get_tag(t).unwrap().into_u16().unwrap();
    assert_eq!(
        (
            tag(&mut d, Tag::Compression),
            tag(&mut d, Tag::Predictor),
            tag(&mut d, Tag::SampleFormat)
        ),
        (8, 2, 2)
    );
}

#[test]
fn int16_window_across_strips_returns_the_raw_counts() {
    let mut g = GeoTiff::open(&write::<colortype::GrayI16>(
        "window",
        &counts(),
        Predictor::Horizontal,
    ))
    .unwrap();
    // cols 1..4, rows 2..7: crosses strips 0|1|2, the last strip is 1 row (short)
    let got = g.read_window(1, 2, 3, 5).unwrap();
    let all = counts();
    let want: Vec<f32> = (2..7)
        .flat_map(|r| (1..4).map(move |c| (r, c)))
        .map(|(r, c)| f32::from(all[r * W as usize + c]))
        .collect();
    assert_eq!(got, want);
}

#[test]
fn int16_extremes_survive_the_predictor() {
    let mut g = GeoTiff::open(&write::<colortype::GrayI16>(
        "extremes",
        &counts(),
        Predictor::Horizontal,
    ))
    .unwrap();
    assert_eq!(
        g.read_window(3, 0, 2, 1).unwrap(),
        [f32::from(i16::MIN), f32::from(i16::MAX)]
    );
}

#[test]
fn int16_source_is_rejected_as_a_dem_tile() {
    let mut g = GeoTiff::open(&write::<colortype::GrayI16>(
        "tile",
        &small_counts(),
        Predictor::Horizontal,
    ))
    .unwrap();
    let r = build_tile(&mut g, MapXy { x_m: 0.0, y_m: 0.0 }, 4, 1, encode_finest);
    assert!(r.is_err(), "int16 counts were read as heights in metres");
}

#[test]
fn int16_source_is_rejected_as_a_float_window() {
    let mut g = GeoTiff::open(&write::<colortype::GrayI16>(
        "float",
        &small_counts(),
        Predictor::Horizontal,
    ))
    .unwrap();
    let r = FloatWindow::read(&mut g, MapXy { x_m: 0.0, y_m: 0.0 }, 20.0);
    assert!(r.is_err(), "int16 counts were read as heights in metres");
}

#[test]
fn float32_source_still_reads_as_a_float_window() {
    #[allow(clippy::cast_precision_loss)] // small whole numbers, exact in f32
    let heights_m: Vec<f32> = (0..W * H).map(|k| k as f32 * 0.5 - 3.0).collect();
    let mut g = GeoTiff::open(&write::<colortype::Gray32Float>(
        "f32",
        &heights_m,
        Predictor::None,
    ))
    .unwrap();
    let win = FloatWindow::read(&mut g, MapXy { x_m: 0.0, y_m: 0.0 }, 20.0).unwrap();
    // ±20 m at 20 m pixels: ceil(40 / 20) + 2 = 4 px a side from col floor(30 / 20) = 1,
    // row floor(50 / 20) = 2, so the window corner is (−50 + 20, 70 − 40) m
    assert_eq!(win.grid(), (4, 4, -30.0, 30.0, 20.0));
}
