//! T07 native reports (SCIENCE-SPEC T-1a-4, T-1a-5, U18), printed for docs/DATA-NOTES.md.

use crate::dem::{FloatWindow, GeoTiff, ShippedTiles};
use crate::eph::Result;
use penumbra_core::dem::DemTile;
use penumbra_core::geometry::{LatLon, MapXy, latlon_to_xy, xy_to_latlon};
use penumbra_core::horizon::{
    Horizon, HorizonOptions, N_AZ, NEAR_OBSTACLE_M, ground_height_m, horizon,
};
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Reference near range: 7000 + 240·388 m, so its far samples beyond fall on the shipped ones.
const REF_NEAR_RANGE_M: f64 = 100_120.0;
/// Half-width of the float reference window around the site, map metres (> REF_NEAR_RANGE_M).
const REF_HALF_M: f64 = 101_000.0;
/// T-1a-4 pass mark on bins whose obstacle is ≥ 60 m away in both runs (SCIENCE-SPEC §1a).
const PASS_DEG: f64 = 0.05;
/// Border of the near tile's pickable area for the extreme-pixel points. DATA-NOTES T06 uses
/// 350 px (7 km), but map distance exceeds ground distance by 1 + r²/4R² (~0.2 % at Mons
/// Mouton), so a 7 km march from the 350 px edge leaves the tile; 360 px (7.2 km) does not.
const BORDER_PX: usize = 360;

fn deg(lat_deg: f64, lon_east_deg: f64) -> LatLon {
    LatLon {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_east_deg.to_radians(),
    }
}

/// Bin index → azimuth, degrees clockwise from north.
fn az_deg(i: usize) -> f64 {
    #[allow(clippy::cast_precision_loss)] // i < 720
    let a = i as f64 * 0.5;
    a
}

/// Site on the centre of the highest (`high`) or lowest pixel of the tile's pickable area.
fn extreme_pixel(t: &DemTile, high: bool) -> Result<LatLon> {
    let m = t.meta();
    let w = usize::try_from(m.width)?;
    let h = usize::try_from(m.height)?;
    let mut best: Option<(i16, usize, usize)> = None;
    for r in BORDER_PX..h - BORDER_PX {
        for c in BORDER_PX..w - BORDER_PX {
            let v = t.counts()[r * w + c];
            if v == m.nodata {
                continue;
            }
            let better = best.is_none_or(|(b, _, _)| if high { v > b } else { v < b });
            if better {
                best = Some((v, r, c));
            }
        }
    }
    let (_, r, c) = best.ok_or("pickable area has no height")?;
    let xy = MapXy {
        x_m: m.x0_m + (f64::from(u32::try_from(c)?) + 0.5) * m.pixel_m,
        y_m: m.y0_m - (f64::from(u32::try_from(r)?) + 0.5) * m.pixel_m,
    };
    Ok(xy_to_latlon(xy).map_err(|e| format!("{e:?}"))?)
}

fn el_dist(h: &Horizon, i: usize) -> String {
    format!("{:.4}° @ {:.0} m", h.el_rad[i].to_degrees(), h.dist_m[i])
}

/// T-1a-4 statistics of `ship` against `reference`.
fn compare(label: &str, ship: &Horizon, reference: &Horizon) -> f64 {
    let d: Vec<f64> = (0..N_AZ)
        .map(|i| (ship.el_rad[i] - reference.el_rad[i]).to_degrees().abs())
        .collect();
    let worst = |keep: &dyn Fn(usize) -> bool| {
        (0..N_AZ)
            .filter(|&i| keep(i))
            .max_by(|&a, &b| d[a].total_cmp(&d[b]))
    };
    let far_enough =
        |i: usize| ship.dist_m[i] >= NEAR_OBSTACLE_M && reference.dist_m[i] >= NEAR_OBSTACLE_M;
    let rms = |keep: &dyn Fn(usize) -> bool| {
        let v: Vec<f64> = (0..N_AZ)
            .filter(|&i| keep(i))
            .map(|i| d[i] * d[i])
            .collect();
        #[allow(clippy::cast_precision_loss)] // ≤ 720 bins
        let n = v.len() as f64;
        (v.iter().sum::<f64>() / n).sqrt()
    };
    println!("  {label}");
    if let Some(i) = worst(&|_| true) {
        println!(
            "    max |Δel| all bins      {:.4}° at az {:.1}° (shipped {}, ref {})",
            d[i],
            az_deg(i),
            el_dist(ship, i),
            el_dist(reference, i)
        );
    }
    let n60 = (0..N_AZ).filter(|&i| far_enough(i)).count();
    let mut max60 = 0.0;
    if let Some(i) = worst(&far_enough) {
        max60 = d[i];
        println!(
            "    max |Δel| dist ≥ 60 m   {:.4}° at az {:.1}° (shipped {}, ref {})  [{n60} bins]",
            d[i],
            az_deg(i),
            el_dist(ship, i),
            el_dist(reference, i)
        );
    }
    let over = (0..N_AZ)
        .filter(|&i| far_enough(i) && d[i] > PASS_DEG)
        .count();
    println!(
        "    RMS all {:.4}°, dist ≥ 60 m {:.4}°; bins > {PASS_DEG}° with dist ≥ 60 m: {over}",
        rms(&|_| true),
        rms(&far_enough)
    );
    let bands = [
        ("<60 m", 0.0, 60.0),
        ("60 m–7 km", 60.0, 7_000.0),
        ("7–15 km", 7_000.0, 15_000.0),
        ("15–100 km", 15_000.0, 100_000.0),
        (">100 km", 100_000.0, f64::INFINITY),
    ];
    let text: Vec<String> = bands
        .iter()
        .map(|&(name, lo, hi)| {
            let in_band = |i: usize| reference.dist_m[i] >= lo && reference.dist_m[i] < hi;
            let n = (0..N_AZ).filter(|&i| in_band(i)).count();
            match worst(&in_band) {
                Some(i) => format!("{name} {:.4}° ({n})", d[i]),
                None => format!("{name} – (0)"),
            }
        })
        .collect();
    println!("    by ref distance: {}", text.join(" · "));
    max60
}

fn timing(label: &str, t: &ShippedTiles, region: &str, site: LatLon, mast_m: f64) -> Result<()> {
    let o = HorizonOptions::default();
    t.horizon(region, site, mast_m, &o)?;
    let mut ms: Vec<f64> = (0..20)
        .map(|_| {
            let t0 = Instant::now();
            let h = t.horizon(region, site, mast_m, &o);
            let dt = t0.elapsed().as_secs_f64() * 1000.0;
            h.map(|_| dt)
        })
        .collect::<Result<_>>()?;
    ms.sort_by(f64::total_cmp);
    println!(
        "  {label}: median {:.2} ms, max {:.2} ms (1 warm-up + 20 calls)",
        (ms[9] + ms[10]) / 2.0,
        ms[19]
    );
    Ok(())
}

/// The T07 reports for `data_dir` (shipped tiles); the T-1a-4 reference DEM comes from PENUMBRA_DATA.
pub fn run(data_dir: &Path) -> Result<()> {
    let t = ShippedTiles::load(data_dir)?;
    let im2 = deg(-84.7906, 29.1957);
    let sp01 = deg(-89.45, 222.69);

    println!("U18 native horizon time (release, near + mid + far):");
    timing("IM-2 H = 2 m", &t, "mons-mouton", im2, 2.0)?;
    timing("SP-01 H = 10 m", &t, "shackleton-ridge", sp01, 10.0)?;

    println!("\nT-1a-5 SP-01, H = 10 m: tile edge vs max range 150 km");
    let edge = t.horizon("shackleton-ridge", sp01, 10.0, &HorizonOptions::default())?;
    let cut = HorizonOptions {
        max_range_m: 150_000.0,
        ..HorizonOptions::default()
    };
    let near150 = t.horizon("shackleton-ridge", sp01, 10.0, &cut)?;
    let differ: Vec<usize> = (0..N_AZ)
        .filter(|&i| edge.el_rad[i].to_bits() != near150.el_rad[i].to_bits())
        .collect();
    println!("  bins that differ: {}", differ.len());
    if let Some(&i) = differ.iter().max_by(|&&a, &&b| {
        let da = (edge.el_rad[a] - near150.el_rad[a]).abs();
        let db = (edge.el_rad[b] - near150.el_rad[b]).abs();
        da.total_cmp(&db)
    }) {
        println!(
            "  max |Δel| {:.4}° at az {:.1}° (edge {}, 150 km {})",
            (edge.el_rad[i] - near150.el_rad[i]).to_degrees().abs(),
            az_deg(i),
            el_dist(&edge, i),
            el_dist(&near150, i)
        );
    }
    let far_i = (0..N_AZ)
        .max_by(|&a, &b| edge.dist_m[a].total_cmp(&edge.dist_m[b]))
        .ok_or("no bins")?;
    println!(
        "  farthest obstacle (tile edge run): {:.0} m at az {:.1}°",
        edge.dist_m[far_i],
        az_deg(far_i)
    );
    println!("  longest-night change: pending T08 (#9)");

    let src_dir = PathBuf::from(std::env::var("PENUMBRA_DATA").map_err(|_| "set PENUMBRA_DATA")?);
    let mut src = GeoTiff::open(&src_dir.join("LDEM_80S_20MPP_ADJ.TIF"))?;
    let rows = tier_ablation(&t, &mut src)?;
    let worst = rows.iter().map(|r| r.max_dev_deg).fold(0.0, f64::max);
    println!(
        "\nT-1a-4 worst max |Δel| over bins with dist ≥ 60 m: {worst:.4}° ({})",
        if worst > PASS_DEG {
            "ABOVE 0.05°"
        } else {
            "within 0.05°"
        }
    );
    Ok(())
}

/// One T-1a-4 result: the max |Δel| (degrees) between shipped and reference over the bins whose
/// obstacle is ≥ 60 m away in both runs, at one point and mast height H (metres).
#[derive(Debug, Clone, PartialEq)]
pub struct AblationRow {
    pub point: &'static str,
    pub mast_m: f64,
    pub max_dev_deg: f64,
}

/// T-1a-4 (SCIENCE-SPEC §1a): the shipped horizon against the float 20 m DEM `src`
/// (LDEM_80S_20MPP_ADJ) at the five points, H = 0 and 2 m. Prints the detail for DATA-NOTES.
pub fn tier_ablation(t: &ShippedTiles, src: &mut GeoTiff) -> Result<Vec<AblationRow>> {
    let mm = t.near("mons-mouton")?;
    let sr = t.near("shackleton-ridge")?;
    let err = |e: penumbra_core::dem::DemError| format!("{e:?}");
    let points = [
        ("IM-2", "mons-mouton", deg(-84.7906, 29.1957)),
        ("SP-01", "shackleton-ridge", deg(-89.45, 222.69)),
        (
            "Mons Mouton highest",
            "mons-mouton",
            extreme_pixel(mm, true)?,
        ),
        (
            "Mons Mouton lowest",
            "mons-mouton",
            extreme_pixel(mm, false)?,
        ),
        (
            "Shackleton lowest",
            "shackleton-ridge",
            extreme_pixel(sr, false)?,
        ),
    ];
    println!(
        "\nT-1a-4 shipped (int16 20 m to 7 km, 80 m to 31 km, then 240 m) vs float 20 m to {REF_NEAR_RANGE_M} m + same 240 m"
    );
    let ref_opts = HorizonOptions {
        near_range_m: REF_NEAR_RANGE_M,
        ..HorizonOptions::default()
    };
    let mut rows = Vec::new();
    for (name, region, site) in points {
        let near = t.near(region)?;
        let xy = latlon_to_xy(site).map_err(|e| format!("{e:?}"))?;
        let t0 = Instant::now();
        let win = FloatWindow::read(src, xy, REF_HALF_M)?;
        println!(
            "{name} ({:.4}°, {:.4}°E) map ({:.1}, {:.1}) m [window read {:.1} s]",
            site.lat_rad.to_degrees(),
            site.lon_east_rad.to_degrees(),
            xy.x_m,
            xy.y_m,
            t0.elapsed().as_secs_f64()
        );
        println!(
            "  h_s shipped {:.3} m, float {:.3} m",
            ground_height_m(near, site).map_err(err)?,
            ground_height_m(&win, site).map_err(err)?
        );
        for mast_m in [0.0, 2.0] {
            let ship = t.horizon(region, site, mast_m, &HorizonOptions::default())?;
            let reference = horizon(&win, &t.far, site, mast_m, &ref_opts).map_err(err)?;
            let max_dev_deg = compare(&format!("H = {mast_m} m"), &ship, &reference);
            rows.push(AblationRow {
                point: name,
                mast_m,
                max_dev_deg,
            });
        }
    }
    Ok(rows)
}
