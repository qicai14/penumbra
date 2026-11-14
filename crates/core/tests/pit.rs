//! T15 local pit (SCIENCE-SPEC §1f "Local pit (A3)", V4 statement 3). Expected values are the
//! spec's own: d_c = 1.94 m for D_c = 20 m (E112, 11° slope); el_pit 11.0° / 5.4° / 2.9° / −0.3°
//! at H = 0 / 1 / 1.44 / 2 m; the Sun's highest elevation at IM-2's latitude is 6.75° (E104).

use penumbra_core::horizon::N_AZ;
use penumbra_core::pit::{
    degraded_crater_depth_m, depth_to_hide_m, max_sun_elevation_rad, pit_elevation_rad,
    with_pit_rad,
};

fn assert_close(label: &str, got: f64, want: f64, tol: f64) {
    assert!(
        (got - want).abs() <= tol,
        "{label}: got {got}, want {want} ± {tol}"
    );
}

#[test]
fn degraded_crater_of_20_m_is_1_94_m_deep() {
    assert_close("d_c", degraded_crater_depth_m(20.0).unwrap(), 1.94, 0.005);
}

#[test]
fn pit_elevation_for_20_m_degraded_crater_matches_spec_at_each_panel_height() {
    let d_c = degraded_crater_depth_m(20.0).unwrap();
    for (h_m, want_deg) in [(0.0, 11.0), (1.0, 5.4), (1.44, 2.9), (2.0, -0.3)] {
        let got = pit_elevation_rad(d_c, 20.0, h_m).unwrap().to_degrees();
        assert_close(&format!("el_pit at H = {h_m} m"), got, want_deg, 0.05);
    }
}

#[test]
fn pit_raises_only_bins_below_the_pit_elevation() {
    let mut el_hor = [0.0f64; N_AZ];
    el_hor[10] = 0.3;
    let out = with_pit_rad(&el_hor, 0.1).unwrap();
    assert!((out[0] - 0.1).abs() < 1e-15, "low bin raised to el_pit");
    assert!((out[10] - 0.3).abs() < 1e-15, "higher bin kept");
}

#[test]
fn max_sun_elevation_at_im2_latitude_is_6_75_deg() {
    let got = max_sun_elevation_rad((-84.7906f64).to_radians())
        .unwrap()
        .to_degrees();
    assert_close("max el_sun at 84.7906°S", got, 6.75, 0.005);
}

// A pit exactly as deep as the degraded crater hides a disc whose upper limb is at 11°
// from a panel on the floor: d_c* = 0 + 10·tan 11° = 1.94 m (§1f).
#[test]
fn depth_to_hide_a_disc_with_upper_limb_at_11_deg_from_the_floor_is_1_94_m() {
    let rho = 0.25f64.to_radians();
    let got = depth_to_hide_m(20.0, 0.0, 11f64.to_radians() - rho, rho).unwrap();
    assert_close("d_c*", got, 1.94, 0.005);
}

// The upper limb on the geometric horizon: the pit must reach exactly the panel height.
#[test]
fn depth_to_hide_with_upper_limb_on_the_horizon_is_the_panel_height() {
    let rho = 0.27f64.to_radians();
    let got = depth_to_hide_m(20.0, 1.44, -rho, rho).unwrap();
    assert_close("d_c*", got, 1.44, 1e-12);
}

#[test]
fn pit_model_refuses_non_finite_and_non_positive_inputs() {
    assert!(degraded_crater_depth_m(0.0).is_err());
    assert!(degraded_crater_depth_m(f64::NAN).is_err());
    assert!(pit_elevation_rad(1.9, -20.0, 0.0).is_err());
    assert!(pit_elevation_rad(f64::NAN, 20.0, 0.0).is_err());
    assert!(with_pit_rad(&[0.0; N_AZ], f64::INFINITY).is_err());
    assert!(max_sun_elevation_rad(f64::NAN).is_err());
    assert!(
        depth_to_hide_m(20.0, 0.0, 1.0, 0.0).is_err(),
        "ρ must be > 0"
    );
    assert!(
        depth_to_hide_m(20.0, 0.0, 1.6, 0.01).is_err(),
        "limb past zenith"
    );
}
