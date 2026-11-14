//! T01 site geometry without data: SCIENCE-SPEC §1a, T-1a-2 (no-data part), P7, P9, N7, V5 case 1.

use penumbra_core::geometry::{
    EnuFrame, LatLon, MOON_RADIUS_M, MapXy, Vec3, curvature_drop_m, elevation_closed_form_rad,
    flat_sphere_dip_rad, latlon_to_unit, latlon_to_xy, unit_to_xy, xy_to_latlon, xy_to_unit,
};

fn assert_close(label: &str, got: f64, want: f64, tol: f64) {
    assert!(
        (got - want).abs() <= tol,
        "{label}: got {got}, want {want} ± {tol}"
    );
}

fn site_deg(lat_deg: f64, lon_east_deg: f64) -> LatLon {
    LatLon {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_east_deg.to_radians(),
    }
}

// T-1a-2 / P7: SP-01 at 222.69°E, 89.45°S (E85) maps to (−11.31, −12.26) km. Tolerance is half the last printed digit.
#[test]
fn t_1a_2_sp01_maps_to_published_map_xy() {
    let xy = latlon_to_xy(site_deg(-89.45, 222.69)).unwrap();
    assert_close("SP-01 x_km", xy.x_m / 1000.0, -11.31, 0.005);
    assert_close("SP-01 y_km", xy.y_m / 1000.0, -12.26, 0.005);
}

// T-1a-2 / P7: IM-2 at 84.7906°S, 29.1957°E maps to (+77.1, +138.0) km; a swap moves it 86 km.
#[test]
fn t_1a_2_im2_maps_to_published_map_xy() {
    let xy = latlon_to_xy(site_deg(-84.7906, 29.1957)).unwrap();
    assert_close("IM-2 x_km", xy.x_m / 1000.0, 77.1, 0.05);
    assert_close("IM-2 y_km", xy.y_m / 1000.0, 138.0, 0.05);
}

// T-1a-2: forward then inverse returns (lat, lon) to 1e-9 rad.
#[test]
fn t_1a_2_projection_round_trips_latlon_to_1e9_rad() {
    let sites = [
        (-89.45, 222.69),
        (-84.7906, 29.1957),
        (-85.0, 0.0),
        (-80.13, 1.44),
        (-60.0, 300.0),
        (-89.999, 135.0),
    ];
    for (lat_deg, lon_deg) in sites {
        let site = site_deg(lat_deg, lon_deg);
        let back = xy_to_latlon(latlon_to_xy(site).unwrap()).unwrap();
        assert_close("lat_rad", back.lat_rad, site.lat_rad, 1e-9);
        assert_close("lon_east_rad", back.lon_east_rad, site.lon_east_rad, 1e-9);
    }
}

// Inputs a projection cannot represent are errors, never a number (no silent fallback).
#[test]
fn projection_rejects_non_finite_and_north_pole() {
    assert!(latlon_to_xy(site_deg(90.0, 0.0)).is_err());
    assert!(latlon_to_xy(site_deg(f64::NAN, 0.0)).is_err());
    assert!(
        xy_to_latlon(MapXy {
            x_m: f64::INFINITY,
            y_m: 0.0
        })
        .is_err()
    );
}

fn assert_vec_close(label: &str, got: Vec3, want: Vec3, tol: f64) {
    assert_close(&format!("{label}.x"), got.x, want.x, tol);
    assert_close(&format!("{label}.y"), got.y, want.y, tol);
    assert_close(&format!("{label}.z"), got.z, want.z, tol);
}

const X_HAT: Vec3 = Vec3 {
    x: 1.0,
    y: 0.0,
    z: 0.0,
};
const Y_HAT: Vec3 = Vec3 {
    x: 0.0,
    y: 1.0,
    z: 0.0,
};
const Z_HAT: Vec3 = Vec3 {
    x: 0.0,
    y: 0.0,
    z: 1.0,
};

// V5 case 1 (frame part): at the pole north = +X (toward lon 0), east = +Y (§1a), whether the
// site comes from the map origin (exactly −Z) or from lat −90° (−Z plus a 6e-17 rounding residue).
#[test]
fn v5_case1_pole_frame_is_north_plus_x_east_plus_y() {
    let from_map = xy_to_unit(MapXy { x_m: 0.0, y_m: 0.0 }).unwrap();
    let from_latlon = latlon_to_unit(site_deg(-90.0, 0.0)).unwrap();
    for up in [from_map, from_latlon] {
        let frame = EnuFrame::at(up).unwrap();
        assert_vec_close("north", frame.north, X_HAT, 1e-12);
        assert_vec_close("east", frame.east, Y_HAT, 1e-12);
        assert_vec_close("up", frame.up, Z_HAT.scale(-1.0), 1e-12);
    }
}

// V5 case 1 / P9: no output at the pole is NaN; a point at lon 0 is due north, one at 90°E due east.
#[test]
fn v5_case1_pole_az_el_is_finite_and_follows_the_convention() {
    let up = xy_to_unit(MapXy { x_m: 0.0, y_m: 0.0 }).unwrap();
    let frame = EnuFrame::at(up).unwrap();
    let observer_m = up.scale(MOON_RADIUS_M + 2.0);
    for (lon_deg, want_az_deg) in [(0.0, 0.0), (90.0, 90.0), (180.0, 180.0), (270.0, 270.0)] {
        let target_m = latlon_to_unit(site_deg(-89.0, lon_deg))
            .unwrap()
            .scale(MOON_RADIUS_M);
        let az_el = frame.az_el(observer_m, target_m).unwrap();
        assert!(az_el.az_rad.is_finite() && az_el.el_rad.is_finite());
        assert_close("az_deg", az_el.az_rad.to_degrees(), want_az_deg, 1e-9);
    }
}

// P9: from 85°S, 0°E the body's north pole is at azimuth 0°.
#[test]
fn p9_body_north_pole_is_azimuth_zero_from_85s() {
    let up = latlon_to_unit(site_deg(-85.0, 0.0)).unwrap();
    let frame = EnuFrame::at(up).unwrap();
    let observer_m = up.scale(MOON_RADIUS_M);
    let az_el = frame.az_el(observer_m, Z_HAT.scale(MOON_RADIUS_M)).unwrap();
    assert_close("az_deg", az_el.az_rad.to_degrees(), 0.0, 1e-9);
}

// P9: from 85°S, 0°E a point displaced along +Y (the direction of increasing east longitude at
// lon 0, dû/dλ = (−sin λ, cos λ, 0)) is at azimuth 90°: clockwise from north, atan2(east, north).
#[test]
fn p9_point_to_the_east_is_azimuth_90_from_85s() {
    let up = latlon_to_unit(site_deg(-85.0, 0.0)).unwrap();
    let frame = EnuFrame::at(up).unwrap();
    let observer_m = up.scale(MOON_RADIUS_M);
    let target_m = observer_m.plus(Y_HAT.scale(1000.0));
    let az_el = frame.az_el(observer_m, target_m).unwrap();
    assert_close("az_deg", az_el.az_rad.to_degrees(), 90.0, 1e-9);
}

// Observer and target in the same place have no direction: an error, not NaN.
#[test]
fn az_el_rejects_coincident_points() {
    let up = latlon_to_unit(site_deg(-85.0, 0.0)).unwrap();
    let frame = EnuFrame::at(up).unwrap();
    let p = up.scale(MOON_RADIUS_M);
    assert!(frame.az_el(p, p).is_err());
}

// The trig-free inverse û(x, y) agrees with the lat/lon unit vector, and the trig-free
// forward û → (x, y) inverts it (§1a, both formulas).
#[test]
fn t_1a_2_unit_vector_and_map_agree() {
    for (lat_deg, lon_deg) in [(-89.45, 222.69), (-84.7906, 29.1957), (-70.0, 123.0)] {
        let site = site_deg(lat_deg, lon_deg);
        let xy = latlon_to_xy(site).unwrap();
        let u_map = xy_to_unit(xy).unwrap();
        assert_vec_close("u", u_map, latlon_to_unit(site).unwrap(), 1e-12);
        let back = unit_to_xy(u_map).unwrap();
        assert_close("x_m", back.x_m, xy.x_m, 1e-6);
        assert_close("y_m", back.y_m, xy.y_m, 1e-6);
    }
    assert!(unit_to_xy(Z_HAT).is_err());
}

// §1a: the vector form and the closed form tan(el) = (r_t·cos θ − r_o)/(r_t·sin θ) agree for a
// target placed along the great circle û(θ) = û_s·cos θ + (north·cos az + east·sin az)·sin θ.
// Off-axis azimuth, raised and sunken targets, three ranges.
#[test]
fn t_1a_vector_and_closed_form_elevation_agree() {
    let up = latlon_to_unit(site_deg(-85.0, 30.0)).unwrap();
    let frame = EnuFrame::at(up).unwrap();
    let observer_radius_m = MOON_RADIUS_M + 120.0 + 2.0; // h_s = 120 m, H = 2 m
    let observer_m = up.scale(observer_radius_m);
    let az_rad = 37.0_f64.to_radians();
    let heading = frame
        .north
        .scale(az_rad.cos())
        .plus(frame.east.scale(az_rad.sin()));
    for s_m in [1_600.0, 50_000.0, 150_000.0] {
        for h_m in [-500.0, 0.0, 300.0, 4_000.0] {
            let theta = s_m / MOON_RADIUS_M;
            let target_radius_m = MOON_RADIUS_M + h_m;
            let target_m = up
                .scale(theta.cos())
                .plus(heading.scale(theta.sin()))
                .scale(target_radius_m);
            let vector = frame.az_el(observer_m, target_m).unwrap();
            let closed =
                elevation_closed_form_rad(observer_radius_m, target_radius_m, s_m).unwrap();
            assert_close("el_rad", vector.el_rad, closed, 1e-12);
            assert_close("az_rad", vector.az_rad, az_rad, 1e-9);
        }
    }
}

// §1a: curvature drop s²/2R is 0.7 m at 1.6 km, 720 m at 50 km, 6.5 km at 150 km.
// The spec prints all three to 2 significant figures (s²/2R = 0.737 m, 719.47 m, 6475 m), so the
// tolerance is half a unit in the 2nd figure: ±0.05 m, ±5 m, ±0.05 km.
#[test]
fn t_1a_curvature_drop_matches_spec() {
    assert_close("drop at 1.6 km, m", curvature_drop_m(1_600.0), 0.7, 0.05);
    assert_close("drop at 50 km, m", curvature_drop_m(50_000.0), 720.0, 5.0);
    assert_close(
        "drop at 150 km, km",
        curvature_drop_m(150_000.0) / 1000.0,
        6.5,
        0.05,
    );
}

// §1a small-angle form el ≈ (h − h_s − H)/s − s/2R: on a bare sphere seen from the ground the
// closed form gives tan(el) = −tan(θ/2), so el = −s/(2R) = −drop/s exactly in real arithmetic;
// 1e-12 rad covers the f64 cancellation in cos θ − 1.
#[test]
fn t_1a_closed_form_on_bare_sphere_is_minus_drop_over_distance() {
    for s_m in [1_600.0, 50_000.0, 150_000.0] {
        let el = elevation_closed_form_rad(MOON_RADIUS_M, MOON_RADIUS_M, s_m).unwrap();
        assert_close("el_rad", el, -curvature_drop_m(s_m) / s_m, 1e-12);
    }
}

// T-1a-1 / N7: flat-sphere dip −acos(R/(R + H)) is −0.0869° at H = 2 m and −0.01944° at
// H = 0.1 m; half the last printed digit as tolerance. f64 throughout (f32 would lose H = 0.1 m).
#[test]
fn t_1a_1_flat_sphere_dip_matches_spec() {
    assert_close(
        "dip at 2 m, deg",
        flat_sphere_dip_rad(2.0).unwrap().to_degrees(),
        -0.0869,
        0.00005,
    );
    assert_close(
        "dip at 0.1 m, deg",
        flat_sphere_dip_rad(0.1).unwrap().to_degrees(),
        -0.01944,
        0.000005,
    );
}

// The dip is the elevation of the tangent point on the bare sphere: the vector form must find it.
#[test]
fn t_1a_1_dip_equals_vector_elevation_of_tangent_point() {
    let mast_m = 2.0;
    let up = xy_to_unit(MapXy { x_m: 0.0, y_m: 0.0 }).unwrap();
    let frame = EnuFrame::at(up).unwrap();
    let observer_m = up.scale(MOON_RADIUS_M + mast_m);
    let theta = (MOON_RADIUS_M / (MOON_RADIUS_M + mast_m)).acos();
    let heading = frame.north.scale(0.6).plus(frame.east.scale(0.8));
    let tangent_m = up
        .scale(theta.cos())
        .plus(heading.scale(theta.sin()))
        .scale(MOON_RADIUS_M);
    let el = frame.az_el(observer_m, tangent_m).unwrap().el_rad;
    assert_close("el_rad", el, flat_sphere_dip_rad(mast_m).unwrap(), 1e-9);
}

#[test]
fn closed_form_and_dip_reject_inputs_with_no_answer() {
    assert!(flat_sphere_dip_rad(-1.0).is_err());
    assert!(flat_sphere_dip_rad(f64::NAN).is_err());
    assert!(elevation_closed_form_rad(MOON_RADIUS_M, MOON_RADIUS_M, 0.0).is_err());
}
