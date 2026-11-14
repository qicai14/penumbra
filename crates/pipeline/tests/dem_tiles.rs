//! T06 shipped DEM tiles in web/public/data, read through manifest.json and the core sampler.
//! The `--ignored` tests also need PENUMBRA_DATA (round trip vs source, PSR margin).
//! T-1a-2 (data part, N9), P5, P11, tile margins. Runs in CI: no NASA data needed.

use penumbra_core::dem::{DemError, DemTile, TileMeta};
use penumbra_core::geometry::{LatLon, MapXy, latlon_to_xy};
use penumbra_pipeline::dem::NEAR_SCALES_M;
use penumbra_pipeline::eph::hash6;
use serde_json::Value;
use std::path::PathBuf;

fn web_public() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public")
}

fn manifest() -> Value {
    let text = std::fs::read_to_string(web_public().join("data/manifest.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

/// Loads the tile a manifest entry points to; checks its file name carries its content hash.
fn load(entry: &Value) -> DemTile {
    let url = entry["url"].as_str().unwrap();
    let bytes = std::fs::read(web_public().join(url)).unwrap();
    assert!(
        url.ends_with(&format!(".{}.bin", hash6(&bytes))),
        "{url} does not carry its content hash"
    );
    let n = |k: &str| entry[k].as_f64().unwrap_or_else(|| panic!("{k} missing"));
    let u = |k: &str| u32::try_from(entry[k].as_u64().unwrap()).unwrap();
    let meta = TileMeta {
        width: u("width"),
        height: u("height"),
        x0_m: n("x0_m"),
        y0_m: n("y0_m"),
        pixel_m: n("pixel_m"),
        scale_m: n("scale_m"),
        offset_m: n("offset_m"),
        nodata: i16::try_from(entry["nodata"].as_i64().unwrap()).unwrap(),
    };
    let (words, rest) = bytes.as_chunks::<2>();
    assert!(rest.is_empty());
    DemTile::new(meta, words.iter().map(|w| i16::from_le_bytes(*w)).collect()).unwrap()
}

fn near(id: &str) -> DemTile {
    let m = manifest();
    let region = m["regions"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["id"] == id)
        .unwrap_or_else(|| panic!("region {id} missing"));
    load(&region["near"])
}

fn deg(lat_deg: f64, lon_east_deg: f64) -> MapXy {
    latlon_to_xy(LatLon {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_east_deg.to_radians(),
    })
    .unwrap()
}

/// SP-01 (E85), SCIENCE-SPEC §1a.
fn sp01() -> MapXy {
    deg(-89.45, 222.69)
}

/// IM-2 site, SCIENCE-SPEC T-1a-2.
fn im2() -> MapXy {
    deg(-84.7906, 29.1957)
}

/// Barker 2021 Table 2, Site 1 RoI 1–7 centroids, X and Y km (b5251ef3:555–561, 563–569; E129, E130).
const ROI_KM: [(f64, f64); 7] = [
    (-11.5825, -12.4625),
    (-11.4875, -12.5575),
    (-11.3525, -12.5025),
    (-11.2875, -12.1675),
    (-11.1025, -11.9425),
    (-10.8025, -11.7375),
    (-10.2275, -11.5525),
];

fn rois() -> impl Iterator<Item = MapXy> {
    ROI_KM.iter().map(|&(x, y)| MapXy {
        x_m: x * 1000.0,
        y_m: y * 1000.0,
    })
}

/// Mean height on a 200 m ring (72 points) around `c`, map metres.
fn ring_mean_m(t: &DemTile, c: MapXy) -> f64 {
    let sum: f64 = (0..72)
        .map(|k| {
            let a = f64::from(k) * 5f64.to_radians();
            t.height_m(MapXy {
                x_m: c.x_m + 200.0 * a.sin(),
                y_m: c.y_m + 200.0 * a.cos(),
            })
            .unwrap()
        })
        .sum();
    sum / 72.0
}

// T-1a-2 / N9: SP-01 sits on a ridge crest, so it is above the mean of a 200 m ring around it.
// A swapped x/y or a flipped longitude puts it elsewhere (P7).
#[test]
fn t_1a_2_sp01_is_on_a_ridge_crest() {
    let t = near("shackleton-ridge");
    let (h, ring) = (t.height_m(sp01()).unwrap(), ring_mean_m(&t, sp01()));
    println!("SP-01 h {h:.2} m, 200 m ring mean {ring:.2} m");
    assert!(h > ring);
}

// T-1a-2 / N9: all seven Barker Site 1 RoI centroids are above the mean of a 200 m ring.
#[test]
fn t_1a_2_all_seven_site1_rois_are_above_their_200m_ring() {
    let t = near("shackleton-ridge");
    for (i, c) in rois().enumerate() {
        let (h, ring) = (t.height_m(c).unwrap(), ring_mean_m(&t, c));
        println!("RoI {} h {h:.2} m, ring mean {ring:.2} m", i + 1);
        assert!(h > ring, "RoI {} is not above its ring", i + 1);
    }
}

/// Distance from `p` to the nearest tile edge, map metres (negative outside).
fn inner_margin_m(m: &TileMeta, p: MapXy) -> f64 {
    let x1 = m.x0_m + f64::from(m.width) * m.pixel_m;
    let y1 = m.y0_m - f64::from(m.height) * m.pixel_m;
    [p.x_m - m.x0_m, x1 - p.x_m, m.y0_m - p.y_m, p.y_m - y1]
        .into_iter()
        .fold(f64::INFINITY, f64::min)
}

/// 7 km of ground at the site in map metres: stereographic scale k = 1 + (x² + y²)/(4R²) (§1a).
fn seven_km_in_map_m(p: MapXy) -> f64 {
    let r = penumbra_core::geometry::MOON_RADIUS_M;
    7000.0 * (1.0 + (p.x_m * p.x_m + p.y_m * p.y_m) / (4.0 * r * r))
}

// Issue #7: IM-2, SP-01 and the seven RoIs are each ≥ 7 km inside their near tile [est].
#[test]
fn sites_are_at_least_7km_inside_their_near_tile() {
    let cases = [
        ("mons-mouton", vec![im2()]),
        (
            "shackleton-ridge",
            std::iter::once(sp01()).chain(rois()).collect(),
        ),
    ];
    for (id, pts) in cases {
        let t = near(id);
        for p in pts {
            let margin = inner_margin_m(t.meta(), p);
            println!("{id} {p:?}: margin {margin:.0} m");
            // the sampler's footprint ends half a pixel inside the tile edge
            assert!(
                margin >= seven_km_in_map_m(p) + t.meta().pixel_m / 2.0,
                "{id} {p:?}"
            );
        }
    }
}

// Every near site's 7 km march disc has heights (no no-data inside the pickable square).
#[test]
fn near_tiles_have_no_nodata() {
    for id in ["mons-mouton", "shackleton-ridge"] {
        let t = near(id);
        assert!(!t.counts().contains(&t.meta().nodata), "{id}");
    }
}

// P11: near scale is in {0.0625, 0.125, 0.25} m and the next finer scale would clip the tile.
#[test]
fn near_scale_is_the_finest_that_fits_the_tile() {
    for id in ["mons-mouton", "shackleton-ridge"] {
        let t = near(id);
        let m = t.meta();
        assert!(
            NEAR_SCALES_M.iter().any(|s| (s - m.scale_m).abs() < 1e-12),
            "{id}"
        );
        let (lo, hi) = t
            .counts()
            .iter()
            .filter(|&&c| c != m.nodata)
            .fold((i16::MAX, i16::MIN), |(a, b), &c| (a.min(c), b.max(c)));
        let range_m = f64::from(i32::from(hi) - i32::from(lo)) * m.scale_m;
        println!("{id}: relief {range_m:.1} m at {} m", m.scale_m);
        if m.scale_m > NEAR_SCALES_M[0] {
            assert!(
                range_m / (m.scale_m / 2.0) > 2.0 * f64::from(i16::MAX),
                "{id}"
            );
        }
        assert!((m.pixel_m - 20.0).abs() < 1e-12 && m.width == 1536 && m.height == 1536);
    }
}

// Far tile: 240 m, ±382.08 km about the pole, 0.5 m; the site + 200 km march stays inside it.
#[test]
fn far_tile_is_240m_to_382km_at_half_metre() {
    let far = load(&manifest()["far"]);
    let m = far.meta();
    assert!((m.pixel_m - 240.0).abs() < 1e-12 && (m.scale_m - 0.5).abs() < 1e-12);
    assert!((m.x0_m + 382_080.0).abs() < 1e-6 && (m.y0_m - 382_080.0).abs() < 1e-6);
    assert_eq!((m.width, m.height), (3184, 3184));
    assert!(!far.counts().contains(&m.nodata));
    assert!(inner_margin_m(m, im2()) > 200_000.0);
}

// The engine finds h_max at load (ARCHITECTURE §3.2); the manifest must not carry it.
#[test]
fn manifest_has_no_h_max() {
    assert!(
        !std::fs::read_to_string(web_public().join("data/manifest.json"))
            .unwrap()
            .contains("h_max")
    );
}

// The manifest still lists the ephemeris next to the tiles (the dem step merges, never replaces).
#[test]
fn manifest_keeps_the_ephemeris() {
    assert_eq!(manifest()["ephemeris"][0]["id"], "main");
}

// P5: the shipped tile refuses a point off its edge instead of returning 0 m.
#[test]
fn shipped_near_tile_refuses_points_outside_coverage() {
    let t = near("mons-mouton");
    let far_off = MapXy { x_m: 0.0, y_m: 0.0 };
    assert_eq!(t.height_m(far_off), Err(DemError::OutOfCoverage));
}

fn source(name: &str) -> PathBuf {
    PathBuf::from(std::env::var("PENUMBRA_DATA").expect("set PENUMBRA_DATA to the download folder"))
        .join(name)
}

/// splitmix64 (Steele, Lea & Flood 2014): fixed seed, no `rand` dependency. Uniform below `n`.
fn below(state: &mut u64, n: u32) -> u32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    u32::try_from((z ^ (z >> 31)) % u64::from(n)).unwrap()
}

/// Source pixel index of a tile corner offset given in pixels; it must be a whole, non-negative number.
fn grid_px(v: f64) -> u32 {
    let r = v.round();
    assert!(
        (v - r).abs() < 1e-9 && (0.0..1e6).contains(&r),
        "tile is not on the source grid"
    );
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // whole, in [0, 1e6)
    let px = r as u32;
    px
}

/// Issue #7 round trip: at 1000 random pixels the shipped tile, sampled at the pixel centre through
/// the core sampler, is within half a scale step of the source Float32. The source pixel is read with
/// raw `tiff::read_chunk`, not the pipeline's window reader, so a crop offset cannot hide.
fn round_trip(tile: &DemTile, src_name: &str) {
    use tiff::decoder::{Decoder, DecodingResult};
    let g = penumbra_pipeline::dem::GeoTiff::open(&source(src_name)).unwrap();
    let mut d = Decoder::new(std::io::BufReader::new(
        std::fs::File::open(source(src_name)).unwrap(),
    ))
    .unwrap();
    let (tw, th) = d.chunk_dimensions();
    let across = g.width.div_ceil(tw);
    let m = *tile.meta();
    let (col0, row0) = (
        grid_px((m.x0_m - g.x0_m) / g.pixel_m),
        grid_px((g.y0_m - m.y0_m) / g.pixel_m),
    );
    let mut chunks = std::collections::HashMap::new();
    let mut seed = 7u64;
    let mut worst = 0f64;
    for _ in 0..1000 {
        // the last row and column have no bilinear cell of their own
        let (c, r) = (
            below(&mut seed, m.width - 1),
            below(&mut seed, m.height - 1),
        );
        let (col, row) = (col0 + c, row0 + r);
        let index = (row / th) * across + col / tw;
        let px = chunks.entry(index).or_insert_with(|| {
            let DecodingResult::F32(px) = d.read_chunk(index).unwrap() else {
                panic!("Float32")
            };
            px
        });
        let dw = d.chunk_data_dimensions(index).0;
        let want = f64::from(px[((row % th) * dw + col % tw) as usize]);
        let got = tile
            .height_m(MapXy {
                x_m: m.x0_m + (f64::from(c) + 0.5) * m.pixel_m,
                y_m: m.y0_m - (f64::from(r) + 0.5) * m.pixel_m,
            })
            .unwrap();
        worst = worst.max((got - want).abs());
        assert!(
            (got - want).abs() <= m.scale_m / 2.0 + 1e-9,
            "({c}, {r}): {got} vs {want}"
        );
    }
    println!(
        "{src_name}: worst |decoded − source| {worst:.4} m at scale {} m",
        m.scale_m
    );
}

#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
fn round_trip_near_tiles_within_half_a_step_at_1000_pixels() {
    for id in ["mons-mouton", "shackleton-ridge"] {
        round_trip(&near(id), "LDEM_80S_20MPP_ADJ.TIF");
    }
}

#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
fn round_trip_far_tile_within_half_a_step_at_1000_pixels() {
    round_trip(&load(&manifest()["far"]), "LDEM_60S_240MPP_ADJ.TIF");
}

// Issue #7: ≥ 20 PSR-floor pixels (LPSR = 1, same 20 m grid as the DEM) lie ≥ 7 km inside each near tile.
#[test]
#[ignore = "needs local GeoTIFFs in PENUMBRA_DATA"]
fn at_least_20_psr_pixels_are_7km_inside_each_near_tile() {
    let mut psr = penumbra_pipeline::dem::GeoTiff::open(&source("LPSR_80S_20MPP_ADJ.TIF")).unwrap();
    for id in ["mons-mouton", "shackleton-ridge"] {
        let m = *near(id).meta();
        let border_px = 350; // 7 km at 20 m
        let col = grid_px((m.x0_m - psr.x0_m) / psr.pixel_m) + border_px;
        let row = grid_px((psr.y0_m - m.y0_m) / psr.pixel_m) + border_px;
        let side = m.width - 2 * border_px;
        let mask = psr.read_window(col, row, side, side).unwrap();
        let n = mask.iter().filter(|&&v| v > 0.5).count();
        println!("{id}: {n} PSR pixels in the inner {side}² px");
        assert!(n >= 20, "{id}: {n}");
    }
}
