//! T07 horizon on the shipped tiles in web/public/data (runs in CI: no NASA data needed).
//! IM-2 cross-check values: two-tier from the T07 design (TS reference marcher), three-tier from #48.

use penumbra_core::geometry::LatLon;
use penumbra_core::horizon::{HorizonOptions, N_AZ, NEAR_RANGE_M, ground_height_m};
use penumbra_pipeline::dem::ShippedTiles;
use std::path::PathBuf;

fn shipped() -> ShippedTiles {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    ShippedTiles::load(&data).unwrap()
}

/// IM-2 site (84.7906°S, 29.1957°E), SCIENCE-SPEC T-1a-2.
fn im2() -> LatLon {
    LatLon {
        lat_rad: (-84.7906f64).to_radians(),
        lon_east_rad: 29.1957f64.to_radians(),
    }
}

// IM-2, H = 2 m on Mons Mouton near + mid + far (#48): h_s = 5321.92 m; el max 4.0617° at az 117.5°
// (16 520 m); el min −1.4703° at az 328.0° (15 160 m); no bin under 60 m (P13); exit on == off (N11).
// [U] The two-tier values (4.0427° at 16 600 m, −1.4750° at az 328.5°, 14 920 m) matched the T07 TS
// reference marcher; these three-tier values are this marcher's own (no independent oracle yet),
// bounded only by T-1a-4 (0.05°).
#[test]
fn im2_horizon_on_the_shipped_tiles_matches_the_reference() {
    let t = shipped();
    let near = t.near("mons-mouton").unwrap();
    let hs = ground_height_m(near, im2()).unwrap();
    assert!((hs - 5321.92).abs() < 0.01, "h_s {hs}");

    let h = t
        .horizon("mons-mouton", im2(), 2.0, &HorizonOptions::default())
        .unwrap();
    let el_deg: Vec<f64> = h.el_rad.iter().map(|e| e.to_degrees()).collect();
    let imax = (0..N_AZ)
        .max_by(|&a, &b| el_deg[a].total_cmp(&el_deg[b]))
        .unwrap();
    let imin = (0..N_AZ)
        .min_by(|&a, &b| el_deg[a].total_cmp(&el_deg[b]))
        .unwrap();
    assert_eq!(imax, 235, "az of el max");
    assert!((el_deg[imax] - 4.0617).abs() < 1e-4, "{}", el_deg[imax]);
    assert!((h.dist_m[imax] - 16_520.0).abs() < 1e-6);
    assert_eq!(imin, 656, "az of el min");
    assert!((el_deg[imin] + 1.4703).abs() < 1e-4, "{}", el_deg[imin]);
    assert!((h.dist_m[imin] - 15_160.0).abs() < 1e-6);
    assert!(!h.has_near_obstacle());

    let off = HorizonOptions {
        early_exit: false,
        ..HorizonOptions::default()
    };
    let h_off = t.horizon("mons-mouton", im2(), 2.0, &off).unwrap();
    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&h.el_rad), bits(&h_off.el_rad));
    assert_eq!(bits(&h.dist_m), bits(&h_off.dist_m));
}

// N11 at the highest point of a tile: the highest pixel of the Mons Mouton pickable area
// (360 px border, as in the T-1a-4 run) sits above most of the far tile, so the exit is armed
// often; it must not change any bin.
#[test]
fn early_exit_changes_nothing_at_the_highest_pickable_pixel() {
    use penumbra_core::geometry::{MapXy, xy_to_latlon};
    let t = shipped();
    let near = t.near("mons-mouton").unwrap();
    let m = near.meta();
    let (w, b) = (usize::try_from(m.width).unwrap(), 360usize);
    let (mut best, mut at) = (i16::MIN, (0usize, 0usize));
    for r in b..w - b {
        for c in b..w - b {
            let v = near.counts()[r * w + c];
            if v != m.nodata && v > best {
                (best, at) = (v, (r, c));
            }
        }
    }
    let px = |i: usize| f64::from(u32::try_from(i).unwrap()) + 0.5;
    let site = xy_to_latlon(MapXy {
        x_m: m.x0_m + px(at.1) * m.pixel_m,
        y_m: m.y0_m - px(at.0) * m.pixel_m,
    })
    .unwrap();
    let on = t
        .horizon("mons-mouton", site, 2.0, &HorizonOptions::default())
        .unwrap();
    let off = HorizonOptions {
        early_exit: false,
        ..HorizonOptions::default()
    };
    let off = t.horizon("mons-mouton", site, 2.0, &off).unwrap();
    let bits = |v: &[f64]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!(bits(&on.el_rad), bits(&off.el_rad));
    assert_eq!(bits(&on.dist_m), bits(&off.dist_m));
}

/// The UI pick border (web/src/lib/site.ts PICK_BORDER_M): a 7 km march needs ~7 014 m of map
/// at Mons Mouton (scale 1 + r²/4R²), so the picker insets each near tile by 7 050 m.
const PICK_BORDER_M: f64 = 7_050.0;

// The four corners of the 7 050 m-inset area of both shipped near tiles get a horizon (P5 margin);
// the march stops at the near range, so this checks the near tile alone.
#[test]
fn corners_of_the_pick_area_get_a_horizon() {
    use penumbra_core::geometry::{MapXy, xy_to_latlon};
    let near_only = HorizonOptions {
        max_range_m: NEAR_RANGE_M,
        ..HorizonOptions::default()
    };
    let t = shipped();
    for (id, near) in &t.regions {
        let m = near.meta();
        let (x_lo, y_hi) = (m.x0_m + PICK_BORDER_M, m.y0_m - PICK_BORDER_M);
        let x_hi = m.x0_m + f64::from(m.width) * m.pixel_m - PICK_BORDER_M;
        let y_lo = m.y0_m - f64::from(m.height) * m.pixel_m + PICK_BORDER_M;
        for (x_m, y_m) in [(x_lo, y_hi), (x_hi, y_hi), (x_lo, y_lo), (x_hi, y_lo)] {
            let site = xy_to_latlon(MapXy { x_m, y_m }).unwrap();
            let r = t.horizon(id, site, 2.0, &near_only);
            assert!(r.is_ok(), "{id} corner ({x_m}, {y_m}): {:?}", r.err());
        }
    }
}

// T-1a-4 (SCIENCE-SPEC §1a, #48): at IM-2, SP-01, Shackleton lowest and Mons Mouton highest and
// lowest, H = 0 and 2 m, the shipped horizon is within 0.05° of the float 20 m DEM on every bin
// whose obstacle is ≥ 60 m away. Release build advised: ten 101 km float windows.
#[test]
#[ignore = "needs LDEM_80S_20MPP_ADJ.TIF in PENUMBRA_DATA"]
fn t_1a_4_shipped_horizon_is_within_0_05_deg_of_the_float_20m_dem() {
    let src_dir = PathBuf::from(std::env::var("PENUMBRA_DATA").unwrap());
    let mut src =
        penumbra_pipeline::dem::GeoTiff::open(&src_dir.join("LDEM_80S_20MPP_ADJ.TIF")).unwrap();
    let rows = penumbra_pipeline::horizon_report::tier_ablation(&shipped(), &mut src).unwrap();
    assert_eq!(rows.len(), 10, "5 points × H = 0, 2 m");
    let over: Vec<_> = rows.iter().filter(|r| r.max_dev_deg > 0.05).collect();
    assert!(over.is_empty(), "above 0.05°: {over:?}");
}

// #48: the middle 80 m tile covers the 31 km mid range from every corner of the pick area (P5).
#[test]
fn corners_of_the_pick_area_get_a_horizon_through_the_mid_tier() {
    use penumbra_core::geometry::{MapXy, xy_to_latlon};
    let t = shipped();
    for (id, near) in &t.regions {
        let m = near.meta();
        let (x_lo, y_hi) = (m.x0_m + PICK_BORDER_M, m.y0_m - PICK_BORDER_M);
        let x_hi = m.x0_m + f64::from(m.width) * m.pixel_m - PICK_BORDER_M;
        let y_lo = m.y0_m - f64::from(m.height) * m.pixel_m + PICK_BORDER_M;
        for (x_m, y_m) in [(x_lo, y_hi), (x_hi, y_hi), (x_lo, y_lo), (x_hi, y_lo)] {
            let site = xy_to_latlon(MapXy { x_m, y_m }).unwrap();
            let r = t.horizon(id, site, 2.0, &HorizonOptions::default());
            assert!(r.is_ok(), "{id} corner ({x_m}, {y_m}): {:?}", r.err());
        }
    }
}

// #48: each region ships an 80 m middle tile next to its 20 m near tile.
#[test]
fn every_region_ships_an_80m_mid_tile() {
    let t = shipped();
    assert_eq!(t.mids.len(), t.regions.len());
    for (id, _) in &t.regions {
        let m = t.mid(id).unwrap().meta();
        assert!((m.pixel_m - 80.0).abs() < 1e-12, "{id}: {} m", m.pixel_m);
    }
}
