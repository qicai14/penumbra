//! T21 point-Sun shadow mask over the near tile (SCIENCE-SPEC §3 V3 method, ARCHITECTURE §2.6).
//! Differential oracle: T08's full-horizon Sun-disc fraction (§1b) at the same points, compared
//! only where the disc is wholly clear (1) or wholly hidden (0); partial-disc points are skipped.

use penumbra_core::dem::{DemTile, TileMeta};
use penumbra_core::eph::R_SUN_KM;
use penumbra_core::geometry::{
    EnuFrame, LatLon, MOON_RADIUS_M, MapXy, Vec3, latlon_to_unit, latlon_to_xy, xy_to_latlon,
    xy_to_unit,
};
use penumbra_core::horizon::{HorizonOptions, ground_height_m, horizon, point_sun_mask};
use penumbra_core::timeline::disc_fraction;

const R: f64 = MOON_RADIUS_M;
const SCALE_M: f64 = 0.25;
const NEAR_N: u32 = 1536;
const NEAR_PX_M: f64 = 20.0;
const STRIDE: u32 = 64;
const MAST_M: f64 = 2.0;
/// 1 AU in metres (IAU 2012 B2).
const AU_M: f64 = 149_597_870_700.0;

/// Square tile, offset 0, scale 0.25 m, centred on `c`, heights from `h_m(x_m, y_m)` at pixel centres.
fn tile(n: u32, pixel_m: f64, c: MapXy, h_m: impl Fn(f64, f64) -> f64) -> DemTile {
    let half = f64::from(n) / 2.0 * pixel_m;
    let (x0_m, y0_m) = (c.x_m - half, c.y_m + half);
    let mut counts = Vec::with_capacity(usize::try_from(n * n).unwrap());
    for row in 0..n {
        for col in 0..n {
            let x = x0_m + pixel_m * (f64::from(col) + 0.5);
            let y = y0_m - pixel_m * (f64::from(row) + 0.5);
            #[allow(clippy::cast_possible_truncation)]
            // heights ≤ 300 m → ≤ 1200 counts, whole after round
            counts.push((h_m(x, y) / SCALE_M).round() as i16);
        }
    }
    let meta = TileMeta {
        width: n,
        height: n,
        x0_m,
        y0_m,
        pixel_m,
        scale_m: SCALE_M,
        offset_m: 0.0,
        nodata: i16::MIN,
    };
    DemTile::new(meta, counts).unwrap()
}

/// MOON_ME position, metres, of a Sun at 1 AU seen at (az, el) from the ground at `c`
/// (topocentric ENU of `c`, az clockwise from north).
fn sun_m(c: MapXy, az_deg: f64, el_deg: f64) -> Vec3 {
    let up = xy_to_unit(c).unwrap();
    let f = EnuFrame::at(up).unwrap();
    let (sa, ca) = az_deg.to_radians().sin_cos();
    let (se, ce) = el_deg.to_radians().sin_cos();
    let d = f
        .north
        .scale(ca * ce)
        .plus(f.east.scale(sa * ce))
        .plus(up.scale(se));
    up.scale(R).plus(d.scale(AU_M))
}

// T21 acceptance 1: at 20 grid points the point-Sun mask equals T08's full-horizon visibility
// wherever the whole disc is clear (fraction 1 → lit) or hidden (fraction 0 → shadowed).
// Scene: near 1536² at 20 m around (−89°, 30°E) with three 150 m Gaussian hills inside 4 km;
// far 1800² at 240 m, flat 0 m, so terrain beyond the near tile never sets the horizon.
#[test]
fn point_sun_mask_agrees_with_the_full_horizon_where_the_disc_is_whole() {
    let c = latlon_to_xy(LatLon {
        lat_rad: (-89.0f64).to_radians(),
        lon_east_rad: 30.0f64.to_radians(),
    })
    .unwrap();
    let hills = [(-2500.0, 1500.0), (1800.0, 2600.0), (900.0, -3200.0)];
    let near = tile(NEAR_N, NEAR_PX_M, c, |x, y| {
        hills
            .iter()
            .map(|(dx, dy)| {
                let d2 = (x - c.x_m - dx).powi(2) + (y - c.y_m - dy).powi(2);
                150.0 * (-d2 / (2.0 * 800.0 * 800.0)).exp()
            })
            .sum()
    });
    let far = tile(1800, 240.0, c, |_, _| 0.0);
    let sun = sun_m(c, 100.0, 0.4);

    let mask = point_sun_mask(&near, STRIDE, MAST_M, sun).unwrap();
    // pixel centres 0, 64, …, 1472: the last column and row are outside the bilinear footprint
    assert_eq!((mask.cols, mask.rows), (24, 24), "ceil((1536 − 1) / 64)");
    assert_eq!(mask.lit.len(), 24 * 24);

    let x0 = c.x_m - f64::from(NEAR_N) / 2.0 * NEAR_PX_M;
    let y0 = c.y_m + f64::from(NEAR_N) / 2.0 * NEAR_PX_M;
    let (mut clear, mut hidden, mut checked) = (0, 0, 0);
    for j in [8u32, 10, 13, 16] {
        for i in [8u32, 10, 12, 14, 16] {
            let xy = MapXy {
                x_m: x0 + NEAR_PX_M * (f64::from(i * STRIDE) + 0.5),
                y_m: y0 - NEAR_PX_M * (f64::from(j * STRIDE) + 0.5),
            };
            let site = xy_to_latlon(xy).unwrap();
            let h = horizon(&near, &far, site, MAST_M, &HorizonOptions::default()).unwrap();
            let up = latlon_to_unit(site).unwrap();
            let obs = up.scale(R + ground_height_m(&near, site).unwrap() + MAST_M);
            let ae = EnuFrame::at(up).unwrap().az_el(obs, sun).unwrap();
            let rho_rad = (R_SUN_KM * 1000.0 / sun.minus(obs).norm()).asin();
            let f = disc_fraction(&h.el_rad, ae, rho_rad).unwrap();
            let lit = mask.lit[usize::try_from(j * mask.cols + i).unwrap()];
            checked += 1;
            #[allow(clippy::float_cmp)] // 0 and 1 are exact by design (T-1b-2, N2)
            if f == 1.0 {
                clear += 1;
                assert!(lit, "point ({i}, {j}): disc clear, mask shadowed");
            } else if f == 0.0 {
                hidden += 1;
                assert!(!lit, "point ({i}, {j}): disc hidden, mask lit");
            }
        }
    }
    assert_eq!(checked, 20);
    // Guard against a vacuous pass: the scene must test both classes.
    assert!(clear >= 3 && hidden >= 3, "clear {clear}, hidden {hidden}");
}

// Review fix: a Sun below the limb must shadow every point, including edge points whose ray
// leaves the tile at the first step (they used to read lit at night).
#[test]
fn point_sun_mask_is_dark_everywhere_when_the_sun_is_below_the_limb() {
    let c = latlon_to_xy(LatLon {
        lat_rad: (-89.0f64).to_radians(),
        lon_east_rad: 30.0f64.to_radians(),
    })
    .unwrap();
    let near = tile(256, NEAR_PX_M, c, |_, _| 0.0);
    for az in [0.0, 90.0, 180.0, 270.0] {
        let mask = point_sun_mask(&near, 32, MAST_M, sun_m(c, az, -10.0)).unwrap();
        assert!(
            mask.lit.iter().all(|l| !l),
            "Sun at −10°, az {az}°: some point reads lit"
        );
    }
}
