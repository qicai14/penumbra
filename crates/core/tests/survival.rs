//! T09 battery state of charge, operable window and first lethal shadow: SCIENCE-SPEC §1e, §2.5
//! (ARCHITECTURE), E123, U10. Hand-built timelines, no DEM: times Unix s UTC, angles topocentric
//! ENU degrees, energy Wh, power W.

use penumbra_core::power::{Lethal, Panel, Preset, ROVER_CITED_FIELDS, ROVER_PRESET, survival};
use penumbra_core::timeline::Timeline;

/// 1 AU in km (IAU 2012 Resolution B2), the distance at which S = S₀.
const AU_KM: f64 = 149_597_870.7;
const T0: f64 = 1_794_614_400.0; // 2026-11-14T00:00:00Z
const STEP_S: f64 = 600.0; // Δt = 10 min (§1e)

/// A timeline of `illum.len()` steps every 10 min from T0, Sun at (az, el), full contact unless given.
fn tl(
    illum: &[f64],
    contact: &[f64],
    sun_az_deg: f64,
    sun_el_deg: f64,
    sun_dist_km: f64,
) -> Timeline {
    let n = illum.len();
    #[allow(clippy::cast_precision_loss)] // test step counts are small
    let t = (0..n).map(|i| T0 + STEP_S * i as f64).collect();
    Timeline {
        t_unix_s: t,
        sun_az_deg: vec![sun_az_deg; n],
        sun_el_deg: vec![sun_el_deg; n],
        illumination: illum.to_vec(),
        visibility: illum
            .iter()
            .map(|&f| f64::from(u8::from(f > 0.0)))
            .collect(),
        earth_az_deg: vec![0.0; n],
        earth_el_deg: vec![5.0; n],
        earth_margin_deg: vec![5.0; n],
        contact: contact.to_vec(),
        sun_dist_km: vec![sun_dist_km; n],
    }
}

fn preset(start_wh: f64, area_m2: f64, eff: f64, panel: Panel, load_w: f64) -> Preset {
    Preset {
        capacity_wh: 10_000.0,
        start_wh,
        min_wh: 0.0,
        panel_area_m2: area_m2,
        efficiency: eff,
        panel,
        load_w,
    }
}

#[test]
fn t_1e_1_no_generation_100_w_from_705_wh_dies_at_7_050_h() {
    // 60 steps = 10 h, longer than the 7.05 h to death; illumination 0 so P_gen = 0.
    let t = tl(&[0.0; 60], &[1.0; 60], 0.0, 0.0, AU_KM);
    let p = preset(705.0, 1.5, 0.3, Panel::SunTracking, 100.0);
    let s = survival(&t, &p, T0).expect("valid inputs");
    let Lethal::At {
        after_landing_h, ..
    } = s.lethal
    else {
        panic!("expected a lethal shadow, got {:?}", s.lethal);
    };
    assert!((after_landing_h - 7.050).abs() <= 1e-9, "{after_landing_h}");
}

#[test]
fn lethal_run_stops_at_the_lethal_step() {
    // Death falls in step 42 (7.0–7.1667 h): soc holds t_0..t_42 and nothing after.
    let t = tl(&[0.0; 60], &[1.0; 60], 0.0, 0.0, AU_KM);
    let s = survival(&t, &preset(705.0, 1.5, 0.3, Panel::SunTracking, 100.0), T0).unwrap();
    assert_eq!(s.soc_wh.len(), 43);
    assert!((s.soc_wh[42] - 5.0).abs() < 1e-9, "{}", s.soc_wh[42]);
}

#[test]
fn t_1e_2_energy_is_conserved_without_clamps() {
    // Sun-tracking, 1 m² at 10 %, 1 AU: P_gen = 136.61 W × f. Six steps of f = 1, ½, 1, ½, 1, ½
    // give Σ P_gen = 614.745 W·step, minus 6 × 100 W, × 1/6 h = +2.4575 Wh.
    let f = [1.0, 0.5, 1.0, 0.5, 1.0, 0.5, 1.0];
    let t = tl(&f, &[1.0; 7], 0.0, 3.0, AU_KM);
    let s = survival(
        &t,
        &preset(5_000.0, 1.0, 0.1, Panel::SunTracking, 100.0),
        T0,
    )
    .unwrap();
    assert_eq!(s.lethal, Lethal::NoneInSpan);
    assert_eq!(s.soc_wh.len(), 7);
    let end = s.soc_wh[6];
    assert!((end - 5_002.457_5).abs() < 1e-9, "{end}");
}

#[test]
fn flux_scales_with_inverse_square_of_sun_distance() {
    // At 2 AU, S = 1366.1 / 4 = 341.525 W/m²; 1 m², η = 1, tracking, f = 1, no load: +56.920833 Wh per step.
    let t = tl(&[1.0, 1.0], &[1.0, 1.0], 0.0, 3.0, 2.0 * AU_KM);
    let s = survival(&t, &preset(1_000.0, 1.0, 1.0, Panel::SunTracking, 0.0), T0).unwrap();
    assert!(
        (s.soc_wh[1] - (1_000.0 + 341.525 / 6.0)).abs() < 1e-9,
        "{}",
        s.soc_wh[1]
    );
}

#[test]
fn vertical_panel_facing_azimuth_a_gets_cos_el_cos_az_minus_a() {
    // Panel faces az 30° (normal (sin 30°, cos 30°, 0)); Sun at az 90°, el 0°: cos 0 · cos 60° = ½.
    // 1 m², η = 1, 1 AU, f = 1: 683.05 W for 1/6 h = +113.841667 Wh.
    let a = 30_f64.to_radians();
    let panel = Panel::Fixed {
        east: a.sin(),
        north: a.cos(),
        up: 0.0,
    };
    let t = tl(&[1.0, 1.0], &[1.0, 1.0], 90.0, 0.0, AU_KM);
    let s = survival(&t, &preset(1_000.0, 1.0, 1.0, panel, 0.0), T0).unwrap();
    assert!(
        (s.soc_wh[1] - (1_000.0 + 683.05 / 6.0)).abs() < 1e-9,
        "{}",
        s.soc_wh[1]
    );
}

#[test]
fn fixed_panel_facing_away_from_the_sun_generates_nothing() {
    // Normal north, Sun due south: cos(180°) < 0 clamps to 0, so a 100 W load drains 100/6 Wh.
    let panel = Panel::Fixed {
        east: 0.0,
        north: 1.0,
        up: 0.0,
    };
    let t = tl(&[1.0, 1.0], &[1.0, 1.0], 180.0, 2.0, AU_KM);
    let s = survival(&t, &preset(1_000.0, 1.0, 1.0, panel, 100.0), T0).unwrap();
    assert!(
        (s.soc_wh[1] - (1_000.0 - 100.0 / 6.0)).abs() < 1e-9,
        "{}",
        s.soc_wh[1]
    );
}

#[test]
fn charge_clamps_at_capacity() {
    let t = tl(&[1.0, 1.0], &[1.0, 1.0], 0.0, 3.0, AU_KM);
    let s = survival(&t, &preset(10_000.0, 1.0, 1.0, Panel::SunTracking, 0.0), T0).unwrap();
    assert_eq!(s.soc_wh[1], 10_000.0);
}

#[test]
fn hours_follow_the_e123_and_operable_definitions() {
    // Steps: f = 1, 0, 0, 0, 0.3, 1, 0; contact = 1, 1, 0, 0, 0, 1, 1.
    // Longest f = 0 run: 3 steps = 0.5 h. Longest f > 0 run: 2 steps = 1/3 h.
    // Sunlit Σ f·Δt = 2.3/6 h; contact 4/6 h; operable (f > 0 and contact) steps 0 and 5 = 2/6 h.
    let f = [1.0, 0.0, 0.0, 0.0, 0.3, 1.0, 0.0];
    let c = [1.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.0];
    let t = tl(&f, &c, 0.0, 3.0, AU_KM);
    let s = survival(&t, &preset(5_000.0, 1.0, 0.1, Panel::SunTracking, 0.0), T0).unwrap();
    assert!((s.longest_dark_h - 0.5).abs() < 1e-12);
    assert!((s.longest_any_sun_h - 2.0 / 6.0).abs() < 1e-12);
    assert!((s.sunlit_h - 2.3 / 6.0).abs() < 1e-12);
    assert!((s.contact_h - 4.0 / 6.0).abs() < 1e-12);
    assert!((s.operable_h - 2.0 / 6.0).abs() < 1e-12);
}

#[test]
fn the_cited_preset_carries_the_rover_paper_values() {
    // E58a–d: 10 000 Wh, 1.5 m², 30 %, 220 W.
    assert_eq!(ROVER_PRESET.capacity_wh, 10_000.0);
    assert_eq!(ROVER_PRESET.panel_area_m2, 1.5);
    assert_eq!(ROVER_PRESET.efficiency, 0.30);
    assert_eq!(ROVER_PRESET.load_w, 220.0);
    assert_eq!(
        ROVER_CITED_FIELDS,
        ["capacity_wh", "panel_area_m2", "efficiency", "load_w"]
    );
}

#[test]
fn landing_must_be_the_timeline_start() {
    let t = tl(&[0.0; 3], &[1.0; 3], 0.0, 0.0, AU_KM);
    assert!(survival(&t, &ROVER_PRESET, T0 + 1.0).is_err());
}

#[test]
fn a_non_unit_fixed_panel_normal_is_refused() {
    let panel = Panel::Fixed {
        east: 1.0,
        north: 1.0,
        up: 0.0,
    };
    let t = tl(&[1.0; 3], &[1.0; 3], 0.0, 0.0, AU_KM);
    assert!(survival(&t, &preset(1_000.0, 1.0, 1.0, panel, 0.0), T0).is_err());
}
