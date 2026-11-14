//! T07 terrain horizon (SCIENCE-SPEC §1a, ARCHITECTURE §2.2) on synthetic tiles.
//! T-1a-1, T-1a-3 (P10), P13, N11, V5 case 6 (P5). Expected values from the spec closed forms
//! and the T07 design (TS reference marcher as a differential oracle).
//! Scenes without a middle tile pass the far tile as the middle tier: MID_RANGE_M = 7 000 + 240 × 100,
//! so the march samples the same 240 m grid it would with no middle tier (#48).

use penumbra_core::dem::{DemError, DemTile, TileMeta};
use penumbra_core::geometry::{
    EnuFrame, LatLon, MOON_RADIUS_M, MapXy, elevation_closed_form_rad, flat_sphere_dip_rad,
    latlon_to_unit, latlon_to_xy, unit_to_xy, xy_to_latlon, xy_to_unit,
};
use penumbra_core::horizon::{Horizon, HorizonOptions, MID_RANGE_M, N_AZ, horizon};

const R: f64 = MOON_RADIUS_M;
const SCALE_M: f64 = 0.25;

fn site(lat_deg: f64, lon_east_deg: f64) -> LatLon {
    LatLon {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_east_deg.to_radians(),
    }
}

fn xy_of(s: LatLon) -> MapXy {
    latlon_to_xy(s).unwrap()
}

/// Square tile, offset 0, scale 0.25 m, with `(x0_m, y0_m)` the upper-left pixel corner.
fn tile(n: u32, pixel_m: f64, x0_m: f64, y0_m: f64, counts: Vec<i16>) -> DemTile {
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

/// Flat (h = 0) square tile with its middle pixel corner on `centre`.
fn flat(n: u32, pixel_m: f64, centre: MapXy) -> DemTile {
    let half = f64::from(n) / 2.0 * pixel_m;
    let len = usize::try_from(n * n).unwrap();
    tile(
        n,
        pixel_m,
        centre.x_m - half,
        centre.y_m + half,
        vec![0; len],
    )
}

fn opts(early_exit: bool, max_range_m: f64) -> HorizonOptions {
    HorizonOptions {
        early_exit,
        max_range_m,
        ..HorizonOptions::default()
    }
}

/// MOON_ME direction of azimuth `az_deg` (topocentric ENU, clockwise from north) at `s`.
fn dir(s: LatLon, az_deg: f64) -> penumbra_core::geometry::Vec3 {
    let f = EnuFrame::at(latlon_to_unit(s).unwrap()).unwrap();
    let (sa, ca) = az_deg.to_radians().sin_cos();
    f.north.scale(ca).plus(f.east.scale(sa))
}

/// Map point at ground distance `dist_m` along azimuth `az_deg` on the great circle from `s`.
fn along(s: LatLon, az_deg: f64, dist_m: f64) -> MapXy {
    let th = dist_m / R;
    let u = latlon_to_unit(s)
        .unwrap()
        .scale(th.cos())
        .plus(dir(s, az_deg).scale(th.sin()));
    unit_to_xy(u).unwrap()
}

fn bits(h: &Horizon) -> (Vec<u64>, Vec<u64>) {
    (
        h.el_rad.iter().map(|v| v.to_bits()).collect(),
        h.dist_m.iter().map(|v| v.to_bits()).collect(),
    )
}

// T-1a-1: flat sphere, H = 2 m → −acos(R/(R + H)) = −0.0869366° at every azimuth; the best
// 20 m sample is 2640 m (geometric horizon √(2RH) = 2636.4 m). At the pole too, no NaN (P9).
#[test]
fn flat_sphere_gives_the_mast_dip_at_every_azimuth() {
    let dip = flat_sphere_dip_rad(2.0).unwrap();
    for s in [site(-85.0, 30.0), site(-90.0, 0.0)] {
        let c = xy_of(s);
        let h = horizon(
            &flat(760, 20.0, c),
            &flat(200, 240.0, c),
            &flat(200, 240.0, c),
            s,
            2.0,
            &opts(true, 20_000.0),
        )
        .unwrap();
        for i in 0..N_AZ {
            assert!(
                (h.el_rad[i] - dip).abs() < 1e-8,
                "{s:?} bin {i}: {}",
                h.el_rad[i]
            );
            assert!((h.dist_m[i] - 2640.0).abs() < 1e-9, "{s:?} bin {i}");
        }
    }
}

// P13 own pixel: on a flat sphere at H = 0 the first sample is s = 20 m (one pixel out, the
// observer's own cell skipped), el = −θ/2 = −10/R rad.
#[test]
fn own_pixel_is_skipped() {
    let s = site(-85.0, 30.0);
    let c = xy_of(s);
    let h = horizon(
        &flat(760, 20.0, c),
        &flat(200, 240.0, c),
        &flat(200, 240.0, c),
        s,
        0.0,
        &opts(true, 20_000.0),
    )
    .unwrap();
    for i in 0..N_AZ {
        assert!((h.el_rad[i] + 10.0 / R).abs() < 1e-12, "bin {i}");
        assert!((h.dist_m[i] - 20.0).abs() < 1e-9, "bin {i}");
    }
}

/// Pole site on the centre of near pixel (379, 379): x0 = −7590, y0 = +7590 at 20 m; optional
/// 10 m spike at row 379, col 381, i.e. map (+40, 0) m: due east (east = +Y = map +x at the pole).
fn pole_spike_horizon(spike: bool) -> Horizon {
    let s = site(-90.0, 0.0);
    let mut counts = vec![0i16; 760 * 760];
    if spike {
        counts[379 * 760 + 381] = 40; // 10 m at 0.25 m per count
    }
    let near = tile(760, 20.0, -7590.0, 7590.0, counts);
    horizon(
        &near,
        &flat(200, 240.0, xy_of(s)),
        &flat(200, 240.0, xy_of(s)),
        s,
        2.0,
        &opts(true, 20_000.0),
    )
    .unwrap()
}

// P13 flag: a 10 m spike 40 m due east sets bin 180 (az 90°) to the closed form
// el(R + 2, R + 10, 40 m) = 11.3092349° and raises the near-obstacle flag; bin 540 (west) stays flat.
#[test]
fn spike_two_pixels_away_sets_the_horizon_and_the_flag() {
    let h = pole_spike_horizon(true);
    let want = elevation_closed_form_rad(R + 2.0, R + 10.0, 40.0).unwrap();
    assert!((want.to_degrees() - 11.309_234_9).abs() < 1e-6);
    assert!(
        (h.el_rad[180] - want).abs() < 1e-8,
        "{}",
        h.el_rad[180].to_degrees()
    );
    assert!((h.dist_m[180] - 40.0).abs() < 1e-9);
    assert!((h.el_rad[540] - flat_sphere_dip_rad(2.0).unwrap()).abs() < 1e-8);
    assert!(h.has_near_obstacle());
}

#[test]
fn flat_ground_raises_no_near_obstacle_flag() {
    assert!(!pole_spike_horizon(false).has_near_obstacle());
}

/// Cone apex ground distance in T-1a-3: 7000 + 240·179 m, a far-tier sample.
const CONE_S_M: f64 = 49_960.0;

/// T-1a-3 scene: site (−85°, 30°E), a 1000 m cone (slope 0.5 in map metres) whose apex lies on the
/// great circle at azimuth 67.5° (bin 135, off both map axes) and ground distance 49 960 m, on the
/// centre of a far pixel. Far 560² at 240 m around the site; near flat.
fn cone_scene() -> (DemTile, DemTile, LatLon, MapXy) {
    let s = site(-85.0, 30.0);
    let c = xy_of(s);
    let apex = along(s, 67.5, CONE_S_M);
    let n_c = ((apex.x_m - c.x_m) / 240.0).floor() + 280.0;
    let n_r = 280.0 - ((apex.y_m - c.y_m) / 240.0).floor();
    let (x0, y0) = (
        apex.x_m - 240.0 * (n_c + 0.5),
        apex.y_m + 240.0 * (n_r + 0.5),
    );
    let mut counts = Vec::with_capacity(560 * 560);
    for row in 0..560u32 {
        for col in 0..560u32 {
            let px = x0 + 240.0 * (f64::from(col) + 0.5);
            let py = y0 - 240.0 * (f64::from(row) + 0.5);
            let d_map = (px - apex.x_m).hypot(py - apex.y_m);
            let h = (1000.0 - 0.5 * d_map).max(0.0);
            #[allow(clippy::cast_possible_truncation)] // h / 0.25 ≤ 4000, whole after round
            counts.push((h / SCALE_M).round() as i16);
        }
    }
    (
        flat(760, 20.0, c),
        tile(560, 240.0, x0, y0, counts),
        s,
        apex,
    )
}

// T-1a-3 / P10: closed form el = atan2((R+1000)cos θ − (R+2), (R+1000) sin θ), θ = 49 960/R
// = 0.005588386517 rad = 0.3201910°; the marcher must match it to 0.002° at bin 135.
// A straight map-line march gives 0.2964°, so this fails if the march leaves the great circle.
#[test]
fn cone_off_the_map_axes_matches_the_closed_form() {
    let (near, far, s, _) = cone_scene();
    let want_rad = 0.005_588_386_517;
    let cf = elevation_closed_form_rad(R + 2.0, R + 1000.0, CONE_S_M).unwrap();
    assert!((cf - want_rad).abs() < 1e-12, "closed form {cf}");
    let h = horizon(&near, &far, &far, s, 2.0, &opts(true, 60_000.0)).unwrap();
    assert!(
        (h.el_rad[135] - want_rad).abs().to_degrees() < 0.002,
        "{}°",
        h.el_rad[135].to_degrees()
    );
    assert!((h.dist_m[135] - CONE_S_M).abs() < 1e-9, "{}", h.dist_m[135]);
}

// The apex seen through EnuFrame::az_el is at az 67.5° and the closed-form el: the scene is
// built in the same frame the Sun's azimuth uses.
#[test]
fn cone_apex_is_at_bin_135_in_the_enu_frame() {
    let (_, _, s, apex) = cone_scene();
    let up = latlon_to_unit(s).unwrap();
    let ae = EnuFrame::at(up)
        .unwrap()
        .az_el(
            up.scale(R + 2.0),
            xy_to_unit(apex).unwrap().scale(R + 1000.0),
        )
        .unwrap();
    assert!((ae.az_rad.to_degrees() - 67.5).abs() < 1e-9);
    assert!((ae.el_rad - 0.005_588_386_517).abs() < 1e-11);
}

// N11 armed path: r_max = R + 1000 > r_o, so the exit can fire; it must not change any bin.
#[test]
fn early_exit_changes_nothing_when_armed() {
    let (near, far, s, _) = cone_scene();
    let on = horizon(&near, &far, &far, s, 2.0, &opts(true, 60_000.0)).unwrap();
    let off = horizon(&near, &far, &far, s, 2.0, &opts(false, 60_000.0)).unwrap();
    assert_eq!(bits(&on), bits(&off));
}

// N11 guard path: observer at H = 100 m above flat ground with an 88 m near block 1000 m due east;
// the far tile's h_max (0 m) is below the observer, so the exit must stay off. The east bin then
// reaches the bare-sphere horizon: −acos(R/(R+100)) = −0.6147325° at 7000 + 240·49 = 18 760 m.
// An unguarded exit stops at the block: −0.6782° at 1040 m.
#[test]
fn early_exit_stays_off_when_the_far_tile_is_below_the_observer() {
    let s = site(-85.0, 30.0);
    let c = xy_of(s);
    let (x0, y0) = (c.x_m - 379.5 * 20.0, c.y_m + 379.5 * 20.0);
    let e = along(s, 90.0, 1000.0);
    let (col, row) = (((e.x_m - x0) / 20.0).floor(), ((y0 - e.y_m) / 20.0).floor());
    let mut counts = vec![0i16; 760 * 760];
    for dr in -2..=2 {
        for dc in -2..=2 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            // whole pixel indices inside the 760² tile (block is ~50 px from the centre)
            let k = (row + f64::from(dr)) as usize * 760 + (col + f64::from(dc)) as usize;
            counts[k] = 352; // 88 m
        }
    }
    let near = tile(760, 20.0, x0, y0, counts);
    let far = flat(400, 240.0, c);
    let on = horizon(&near, &far, &far, s, 100.0, &opts(true, 40_000.0)).unwrap();
    let off = horizon(&near, &far, &far, s, 100.0, &opts(false, 40_000.0)).unwrap();
    assert_eq!(bits(&on), bits(&off));
    assert!((on.el_rad[180].to_degrees() + 0.614_732_5).abs() < 1e-6);
    assert!((on.dist_m[180] - 18_760.0).abs() < 1e-9);
}

/// V5 case 6 scene: far 834² flat at 240 m on the pole (x0 = −100 080, y0 = +100 080), so its
/// bilinear footprint ends at ±99 960 m; a flat near tile centred `near_shift_x_m` east of the site.
fn edge_case(
    x_m: f64,
    y_m: f64,
    near_shift_x_m: f64,
    far_nodata_at: Option<MapXy>,
    o: &HorizonOptions,
) -> Result<Horizon, DemError> {
    let xy = MapXy { x_m, y_m };
    let s = xy_to_latlon(xy).unwrap();
    let near = flat(
        760,
        20.0,
        MapXy {
            x_m: x_m + near_shift_x_m,
            y_m,
        },
    );
    let mut counts = vec![0i16; 834 * 834];
    if let Some(p) = far_nodata_at {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        // the point is inside the tile, so both indices are whole and in [0, 834)
        let (col, row) = (
            ((p.x_m + 100_080.0) / 240.0).floor() as usize,
            ((100_080.0 - p.y_m) / 240.0).floor() as usize,
        );
        counts[row * 834 + col] = i16::MIN;
    }
    let far = tile(834, 240.0, -100_080.0, 100_080.0, counts);
    horizon(&near, &far, &far, s, 2.0, o)
}

// V5 case 6 / P5: the far tile must cover the march; a site 5 km inside its edge is an error.
#[test]
fn site_well_inside_the_far_tile_gets_a_horizon() {
    assert!(edge_case(20_000.0, 30_000.0, 0.0, None, &opts(true, 50_000.0)).is_ok());
    assert!(edge_case(45_080.0, 0.0, 0.0, None, &opts(true, 50_000.0)).is_ok());
}

#[test]
fn site_5_km_inside_the_far_edge_is_out_of_coverage() {
    assert_eq!(
        edge_case(95_080.0, 0.0, 0.0, None, &opts(true, 50_000.0)),
        Err(DemError::OutOfCoverage)
    );
}

// With the default (unlimited) range the far tile must reach 200 km (§1a, E66); here it ends at ~70 km.
#[test]
fn far_tile_ending_before_200_km_is_out_of_coverage() {
    assert_eq!(
        edge_case(20_000.0, 30_000.0, 0.0, None, &HorizonOptions::default()),
        Err(DemError::OutOfCoverage)
    );
}

#[test]
fn site_5_km_inside_the_near_edge_is_out_of_coverage() {
    assert_eq!(
        edge_case(20_000.0, 30_000.0, 2_600.0, None, &opts(true, 50_000.0)),
        Err(DemError::OutOfCoverage)
    );
}

#[test]
fn far_nodata_on_a_ray_is_an_error_not_zero() {
    let hole = MapXy {
        x_m: 20_000.0,
        y_m: 50_000.0,
    };
    assert_eq!(
        edge_case(20_000.0, 30_000.0, 0.0, Some(hole), &opts(true, 50_000.0)),
        Err(DemError::NoData)
    );
}

#[test]
fn bad_options_are_refused() {
    for (mast, o) in [
        (-1.0, HorizonOptions::default()),
        (f64::NAN, HorizonOptions::default()),
        (2.0, opts(true, f64::NAN)),
        (2.0, opts(true, 0.0)),
        (
            2.0,
            HorizonOptions {
                near_range_m: 10.0,
                ..HorizonOptions::default()
            },
        ),
    ] {
        let s = site(-85.0, 30.0);
        let c = xy_of(s);
        let r = horizon(
            &flat(760, 20.0, c),
            &flat(200, 240.0, c),
            &flat(200, 240.0, c),
            s,
            mast,
            &o,
        );
        assert!(matches!(r, Err(DemError::BadInput(_))), "{mast} {o:?}");
    }
}

// API layout (docs/API.md): el_deg[720] then dist_m[720].
#[test]
fn api_layout_is_degrees_then_metres() {
    let v = pole_spike_horizon(true).to_api();
    assert_eq!(v.len(), 1440);
    assert!((v[180] - 11.309_234_9).abs() < 1e-6);
    assert!((v[720 + 180] - 40.0).abs() < 1e-9);
}

/// Square tile of constant height `h_m` (a multiple of 0.25 m), middle pixel corner on `centre`.
fn level(n: u32, pixel_m: f64, centre: MapXy, h_m: f64) -> DemTile {
    let half = f64::from(n) / 2.0 * pixel_m;
    #[allow(clippy::cast_possible_truncation)] // test heights are whole multiples of 0.25 m
    let count = (h_m / SCALE_M) as i16;
    let len = usize::try_from(n * n).unwrap();
    tile(
        n,
        pixel_m,
        centre.x_m - half,
        centre.y_m + half,
        vec![count; len],
    )
}

// #48 middle tier (§1a tiers): the shipped mid range is 31 000 m = 7 000 + 240 × 100, so the far
// samples beyond it fall where the two-tier march put them.
#[test]
fn mid_range_ends_on_the_two_tier_far_grid() {
    assert!((MID_RANGE_M - 31_000.0).abs() < 1e-9);
    assert!(((MID_RANGE_M - 7_000.0) / 240.0).fract().abs() < 1e-12);
}

// A level 100 m middle tile around a flat near tile, H = 0: the first middle sample, one 80 m
// pixel past the 7 000 m near range, sets the horizon at every azimuth (el falls with s at
// constant h). Expected value: the §1a closed form at 7 080 m.
#[test]
fn the_first_mid_sample_is_one_mid_pixel_past_the_near_range() {
    let s = site(-85.0, 30.0);
    let c = xy_of(s);
    let want = elevation_closed_form_rad(R, R + 100.0, 7_080.0).unwrap();
    let h = horizon(
        &flat(760, 20.0, c),
        &level(800, 80.0, c, 100.0),
        &flat(500, 240.0, c),
        s,
        0.0,
        &opts(true, 50_000.0),
    )
    .unwrap();
    for i in 0..N_AZ {
        assert!(
            (h.el_rad[i] - want).abs() < 1e-9,
            "bin {i}: {}",
            h.el_rad[i]
        );
        assert!(
            (h.dist_m[i] - 7_080.0).abs() < 1e-9,
            "bin {i}: {}",
            h.dist_m[i]
        );
    }
}

// Flat near and middle tiles, a level 1 000 m far tile, H = 0: the first far sample is one 240 m
// pixel past the 31 000 m mid range (31 240 m), not past the near range.
#[test]
fn the_far_march_starts_at_the_mid_range() {
    let s = site(-85.0, 30.0);
    let c = xy_of(s);
    let want = elevation_closed_form_rad(R, R + 1_000.0, 31_240.0).unwrap();
    let h = horizon(
        &flat(760, 20.0, c),
        &flat(800, 80.0, c),
        &level(500, 240.0, c, 1_000.0),
        s,
        0.0,
        &opts(true, 50_000.0),
    )
    .unwrap();
    for i in 0..N_AZ {
        assert!(
            (h.el_rad[i] - want).abs() < 1e-9,
            "bin {i}: {}",
            h.el_rad[i]
        );
        assert!(
            (h.dist_m[i] - 31_240.0).abs() < 1e-9,
            "bin {i}: {}",
            h.dist_m[i]
        );
    }
}

// P5: a middle tile that ends inside the mid range (±16 km here) is an error, never 0 m.
#[test]
fn mid_tile_ending_inside_the_mid_range_is_out_of_coverage() {
    let s = site(-85.0, 30.0);
    let c = xy_of(s);
    let r = horizon(
        &flat(760, 20.0, c),
        &flat(400, 80.0, c),
        &flat(500, 240.0, c),
        s,
        0.0,
        &opts(true, 50_000.0),
    );
    assert_eq!(r, Err(DemError::OutOfCoverage));
}

#[test]
fn mid_range_shorter_than_one_mid_pixel_past_the_near_range_is_refused() {
    let s = site(-85.0, 30.0);
    let c = xy_of(s);
    let o = HorizonOptions {
        mid_range_m: 7_050.0,
        ..opts(true, 50_000.0)
    };
    let r = horizon(
        &flat(760, 20.0, c),
        &flat(800, 80.0, c),
        &flat(500, 240.0, c),
        s,
        0.0,
        &o,
    );
    assert!(matches!(r, Err(DemError::BadInput(_))), "{r:?}");
}
