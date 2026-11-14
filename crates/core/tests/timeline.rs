//! T08 Sun-disc fraction, Earth margin and the timeline: SCIENCE-SPEC §1b, §1d, P1, P14,
//! N2 and N8 (docs/SCIENCE-SPEC-REVIEW.md). Horizons are 720 bins of topocentric ENU elevation, radians.

use penumbra_core::eph::{Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, Site};
use penumbra_core::geometry::AzEl;
use penumbra_core::horizon::N_AZ;
use penumbra_core::timeline::{
    DEFAULT_EARTH_MARGIN_DEG, disc_fraction, earth_margin_rad, earth_visible_any_fraction,
    horizon_el_at_rad, timeline,
};

/// The Sun's angular radius at 1 AU, 0.26657° (asin(696 000 / 149 597 870.7), §1b).
const RHO_RAD: f64 = 0.004_652_4;

fn flat_horizon(el_rad: f64) -> [f64; N_AZ] {
    [el_rad; N_AZ]
}

/// §1b T-1b-1 oracle: fraction of a unit disc above a straight line at height u.
fn flat_oracle(u: f64) -> f64 {
    (u.acos() - u * (1.0 - u * u).sqrt()) / std::f64::consts::PI
}

fn sun(az_deg: f64, el_deg: f64) -> AzEl {
    AzEl {
        az_rad: az_deg.to_radians(),
        el_rad: el_deg.to_radians(),
    }
}

// T-1b-1: a flat horizon at u·ρ above the Sun's centre matches the closed form to 2e-3 with
// 32 strips, also with the Sun at azimuth 0 where the disc straddles the 0/360 seam (N8).
#[test]
fn flat_horizon_matches_the_closed_form_within_2e_3() {
    let centre = sun(0.0, 1.0);
    for az_deg in [0.0, 117.3, 359.9] {
        let c = AzEl {
            az_rad: f64::to_radians(az_deg),
            ..centre
        };
        for u in [-0.95, -0.7, -0.3, 0.0, 0.2, 0.5, 0.8, 0.99] {
            let h = flat_horizon(c.el_rad + u * RHO_RAD);
            let f = disc_fraction(&h, c, RHO_RAD).unwrap();
            let want = flat_oracle(u);
            assert!(
                (f - want).abs() <= 2e-3,
                "az {az_deg}°, u = {u}: f = {f}, closed form {want}"
            );
        }
    }
}

// T-1b-1: u = 0, the horizon through the Sun's centre, gives one half.
#[test]
fn horizon_through_the_centre_gives_one_half() {
    let c = sun(200.0, -0.4);
    let f = disc_fraction(&flat_horizon(c.el_rad), c, RHO_RAD).unwrap();
    assert!((f - 0.5).abs() <= 1e-12, "f = {f}");
}

// T-1b-2: a horizon above the whole disc gives exactly 0, not 1e-9.
#[test]
#[allow(clippy::float_cmp)] // exact by design (T-1b-2)
fn hidden_disc_is_exactly_zero() {
    let c = sun(45.0, 2.0);
    let h = flat_horizon(c.el_rad + RHO_RAD);
    assert_eq!(disc_fraction(&h, c, RHO_RAD).unwrap(), 0.0);
}

// N2: a horizon below the whole disc gives exactly 1 (normalised by Σ 2w, not πρ²).
#[test]
#[allow(clippy::float_cmp)] // exact by design (N2)
fn clear_disc_is_exactly_one() {
    let c = sun(45.0, 2.0);
    let h = flat_horizon(c.el_rad - RHO_RAD);
    assert_eq!(disc_fraction(&h, c, RHO_RAD).unwrap(), 1.0);
}

// The disc radius is an input, not a constant: the same horizon at 0.9 × ρ(1 AU) above the
// centre hides more of a smaller disc (ρ is recomputed per step, §1b).
#[test]
fn fraction_depends_on_the_disc_radius() {
    let c = sun(10.0, 0.0);
    let h = flat_horizon(0.9 * RHO_RAD);
    let small = disc_fraction(&h, c, 0.262_f64.to_radians()).unwrap();
    let large = disc_fraction(&h, c, 0.271_f64.to_radians()).unwrap();
    assert!(
        large > small,
        "ρ 0.271° gives {large}, ρ 0.262° gives {small}"
    );
}

#[test]
fn bad_disc_inputs_are_refused() {
    let h = flat_horizon(0.0);
    assert!(disc_fraction(&h, sun(0.0, 0.0), 0.0).is_err());
    assert!(disc_fraction(&h, sun(0.0, 0.0), f64::NAN).is_err());
    assert!(disc_fraction(&h, sun(f64::NAN, 0.0), RHO_RAD).is_err());
    assert!(disc_fraction(&h, sun(0.0, 89.9), RHO_RAD).is_err()); // disc reaches the zenith
}

/// Horizon with bin 719 at 1° and bin 0 at 2°, all others 3°: a step across the 0/360 seam.
fn seam_horizon() -> [f64; N_AZ] {
    let mut h = flat_horizon(3_f64.to_radians());
    h[N_AZ - 1] = 1_f64.to_radians();
    h[0] = 2_f64.to_radians();
    h
}

// N8: the lookup wraps at 720 bins: half way between bin 719 (359.5°) and bin 0 (360°).
#[test]
fn horizon_lookup_wraps_between_bin_719_and_bin_0() {
    let h = seam_horizon();
    let el = horizon_el_at_rad(&h, 359.75_f64.to_radians()).unwrap();
    assert!(
        (el.to_degrees() - 1.5).abs() <= 1e-9,
        "el = {}°",
        el.to_degrees()
    );
    let el = horizon_el_at_rad(&h, -0.25_f64.to_radians()).unwrap();
    assert!(
        (el.to_degrees() - 1.5).abs() <= 1e-9,
        "el = {}°",
        el.to_degrees()
    );
}

// N8: the Earth margin is continuous as the azimuth crosses 0/360.
#[test]
fn earth_margin_is_continuous_across_azimuth_zero() {
    let h = seam_horizon();
    let el_earth = 4_f64.to_radians();
    let eps = 1e-9;
    let before = earth_margin_rad(&h, std::f64::consts::TAU - eps, el_earth).unwrap();
    let after = earth_margin_rad(&h, eps, el_earth).unwrap();
    assert!((before - after).abs() < 1e-6, "{before} vs {after}");
    assert!(
        (after.to_degrees() - 2.0).abs() < 1e-6,
        "margin {}°",
        after.to_degrees()
    );
}

// §1d: margin = el_earth − el_hor(az_earth), linear between bins.
#[test]
fn earth_margin_is_elevation_minus_interpolated_horizon() {
    let mut h = flat_horizon(0.0);
    h[20] = 1_f64.to_radians(); // 10.0°
    h[21] = 2_f64.to_radians(); // 10.5°
    let m = earth_margin_rad(&h, 10.25_f64.to_radians(), 5_f64.to_radians()).unwrap();
    assert!(
        (m.to_degrees() - 3.5).abs() <= 1e-9,
        "margin {}°",
        m.to_degrees()
    );
}

// P14: "any fraction of the Earth visible" is its own function, and differs from contact:
// centre 0.5° below a flat horizon, Earth radius 0.95° → upper limb is visible, no contact.
#[test]
fn any_fraction_visible_differs_from_centre_contact() {
    let h = flat_horizon(0.0);
    let earth = sun(0.0, -0.5);
    assert!(earth_visible_any_fraction(&h, earth, 0.95_f64.to_radians()).unwrap());
    let m = earth_margin_rad(&h, earth.az_rad, earth.el_rad).unwrap();
    assert!(m.to_degrees() < DEFAULT_EARTH_MARGIN_DEG);
}

// §1d: the default comms margin is 2° (De Rosa E62, labelled assumed).
#[test]
#[allow(clippy::float_cmp)] // a literal constant
fn default_earth_margin_is_two_degrees() {
    assert_eq!(DEFAULT_EARTH_MARGIN_DEG, 2.0);
}

/// The committed main table, `web/public/data/eph-main.<hash6>.bin`.
fn shipped_ephemeris() -> Ephemeris {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    let bin = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            let name = p.file_name().unwrap().to_str().unwrap();
            name.starts_with("eph-main.") && name.ends_with(".bin")
        })
        .unwrap();
    let bytes = std::fs::read(bin).unwrap();
    assert_eq!(bytes.len(), 6 * 8 * MAIN_ROWS);
    let (words, _) = bytes.as_chunks::<8>();
    let mut eph = Ephemeris::default();
    eph.add(
        MAIN_T0_UNIX_S,
        MAIN_STEP_S,
        words.iter().map(|w| f64::from_le_bytes(*w)).collect(),
    )
    .unwrap();
    eph
}

/// IM-2 site (84.7906°S, 29.1957°E), 2 m mast, ground on the sphere.
fn im2() -> Site {
    Site {
        lat_rad: -84.7906_f64.to_radians(),
        lon_east_rad: 29.1957_f64.to_radians(),
        ground_m: 0.0,
        mast_m: 2.0,
    }
}

/// A rolling synthetic horizon, ±1.5° with 3 cycles round the sky, so the Sun crosses it often.
fn rolling_horizon() -> [f64; N_AZ] {
    std::array::from_fn(|i| {
        #[allow(clippy::cast_precision_loss)] // i < 720, exact
        let az = i as f64 * std::f64::consts::PI / 360.0;
        1.5_f64.to_radians() * (3.0 * az).sin()
    })
}

/// 2026-11-14T00:00:00Z, Unix s UTC.
const T0_UNIX_S: f64 = 1_794_614_400.0;

// P1: over the 30-day, 10-minute timeline visibility ≥ illumination at every step, visibility is
// 0 or 1, and partial steps exist so the check is not vacuous.
#[test]
#[allow(clippy::float_cmp)] // visibility is exactly 0 or 1 by definition
fn visibility_is_at_least_illumination_at_every_step() {
    let tl = timeline(
        &shipped_ephemeris(),
        &rolling_horizon(),
        &im2(),
        T0_UNIX_S,
        600.0,
        4320,
        DEFAULT_EARTH_MARGIN_DEG.to_radians(),
    )
    .unwrap();
    assert_eq!(tl.illumination.len(), 4320);
    let mut partial = 0;
    for (i, (&ill, &vis)) in tl.illumination.iter().zip(&tl.visibility).enumerate() {
        assert!((0.0..=1.0).contains(&ill), "step {i}: illumination {ill}");
        assert!(vis == 0.0 || vis == 1.0, "step {i}: visibility {vis}");
        assert!(
            vis >= ill,
            "step {i}: visibility {vis} < illumination {ill}"
        );
        assert_eq!(vis == 1.0, ill > 0.0, "step {i}");
        if ill > 0.0 && ill < 1.0 {
            partial += 1;
        }
    }
    assert!(partial > 0, "no partial-disc step in 30 days");
}

// §1d: contact = margin ≥ the given minimum, and the margin column is el_earth − el_hor(az_earth).
#[test]
#[allow(clippy::float_cmp)] // contact is exactly 0 or 1 by definition
fn contact_is_margin_at_least_the_minimum() {
    let h = rolling_horizon();
    let tl = timeline(
        &shipped_ephemeris(),
        &h,
        &im2(),
        T0_UNIX_S,
        600.0,
        4320,
        2_f64.to_radians(),
    )
    .unwrap();
    let (mut on, mut off) = (0, 0);
    for i in 0..4320 {
        let m = tl.earth_margin_deg[i];
        let want = earth_margin_rad(
            &h,
            tl.earth_az_deg[i].to_radians(),
            tl.earth_el_deg[i].to_radians(),
        )
        .unwrap()
        .to_degrees();
        assert!(
            (m - want).abs() <= 1e-9,
            "step {i}: margin {m}, want {want}"
        );
        assert_eq!(tl.contact[i] == 1.0, m >= 2.0, "step {i}: margin {m}");
        if tl.contact[i] == 1.0 {
            on += 1
        } else {
            off += 1
        }
    }
    assert!(
        on > 0 && off > 0,
        "contact never changes ({on} on, {off} off)"
    );
}

// docs/API.md: the timeline is 10 × n column-major in the frozen column order.
#[test]
#[allow(clippy::float_cmp)] // the API copies the same values
fn api_layout_is_ten_columns_in_the_frozen_order() {
    let tl = timeline(
        &shipped_ephemeris(),
        &rolling_horizon(),
        &im2(),
        T0_UNIX_S,
        600.0,
        3,
        DEFAULT_EARTH_MARGIN_DEG.to_radians(),
    )
    .unwrap();
    let api = tl.to_api();
    assert_eq!(api.len(), 30);
    let cols = [
        &tl.t_unix_s,
        &tl.sun_az_deg,
        &tl.sun_el_deg,
        &tl.illumination,
        &tl.visibility,
        &tl.earth_az_deg,
        &tl.earth_el_deg,
        &tl.earth_margin_deg,
        &tl.contact,
        &tl.sun_dist_km,
    ];
    for (c, col) in cols.iter().enumerate() {
        assert_eq!(&api[c * 3..(c + 1) * 3], col.as_slice(), "column {c}");
    }
    assert_eq!(tl.t_unix_s[1], T0_UNIX_S + 600.0);
}

// P16: a timeline past the table fails with OUT_OF_TABLE; a non-finite margin is refused.
#[test]
fn out_of_table_and_bad_margin_are_errors() {
    let eph = shipped_ephemeris();
    let h = flat_horizon(0.0);
    let e = timeline(&eph, &h, &im2(), 0.0, 600.0, 3, 0.03).unwrap_err();
    assert_eq!(e.code(), "OUT_OF_TABLE");
    let e = timeline(&eph, &h, &im2(), T0_UNIX_S, 600.0, 3, f64::NAN).unwrap_err();
    assert_eq!(e.code(), "BAD_INPUT");
}

// §1b and the issue: over the 30-day, 10-minute IM-2 timeline from 2026-11-14, ρ = asin(R_sun / d_sun)
// per step stays within 0.261°–0.272° and is not one constant. The range includes the Moon-orbit term:
// the table is Moon-centred, so the Moon's ±384 400 km widens the Earth–Sun yearly range; the 30-day run
// reaches 0.271002° (2026-12-05) and a full 2026 run 0.2715°.
#[test]
fn sun_radius_stays_in_range_over_the_30_day_timeline() {
    let tl = timeline(
        &shipped_ephemeris(),
        &flat_horizon(0.0),
        &im2(),
        T0_UNIX_S,
        600.0,
        4320,
        DEFAULT_EARTH_MARGIN_DEG.to_radians(),
    )
    .unwrap();
    let rho: Vec<f64> = tl
        .sun_dist_km
        .iter()
        .map(|d| (penumbra_core::eph::R_SUN_KM / d).asin().to_degrees())
        .collect();
    for (i, r) in rho.iter().enumerate() {
        assert!((0.261..=0.272).contains(r), "step {i}: ρ = {r}°");
    }
    assert!(
        rho.windows(2).any(|w| (w[0] - w[1]).abs() > 1e-9),
        "ρ is constant"
    );
}

// Review Should 1 (brief rows 9–10): a tilted straight horizon y = b + s·x across the disc, centred
// on the 0/360 seam at el 60°, matches the flat oracle at the line's distance u = b / √(1 + s²).
// Catches a missing 1/cos(el) in the strip azimuth and a broken wrap. (A mirrored ξ sign cannot be
// seen: the disc is symmetric, so the fraction of a reflected horizon is the same.)
#[test]
fn tilted_horizon_across_the_seam_matches_the_closed_form() {
    let (az_c, el_c) = (0.1_f64, 60.0_f64);
    let (b, s) = (0.3 * RHO_RAD, 1.0);
    let h: [f64; N_AZ] = std::array::from_fn(|i| {
        #[allow(clippy::cast_precision_loss)] // i < 720, exact
        let d = (i as f64 * 0.5 - az_c + 180.0).rem_euclid(360.0) - 180.0;
        el_c.to_radians() + b + s * el_c.to_radians().cos() * d.to_radians()
    });
    let f = disc_fraction(&h, sun(az_c, el_c), RHO_RAD).unwrap();
    let want = flat_oracle(0.3 / (1.0 + s * s).sqrt());
    assert!((f - want).abs() < 2e-3, "f = {f}, closed form {want}");
}

// Review Should 2 (brief row 13): the timeline uses this step's ρ = asin(R_sun / d_sun), not a
// constant: each step equals disc_fraction at that step's own radius.
#[test]
fn timeline_uses_the_per_step_sun_radius() {
    let h = rolling_horizon();
    let tl = timeline(
        &shipped_ephemeris(),
        &h,
        &im2(),
        T0_UNIX_S,
        600.0,
        4320,
        DEFAULT_EARTH_MARGIN_DEG.to_radians(),
    )
    .unwrap();
    let mut differs = 0;
    for i in 0..tl.illumination.len() {
        let c = sun(tl.sun_az_deg[i], tl.sun_el_deg[i]);
        let rho = (penumbra_core::eph::R_SUN_KM / tl.sun_dist_km[i]).asin();
        let own = disc_fraction(&h, c, rho).unwrap();
        assert!(
            (tl.illumination[i] - own).abs() < 1e-9,
            "step {i}: {} vs {own}",
            tl.illumination[i]
        );
        if (disc_fraction(&h, c, RHO_RAD).unwrap() - own).abs() > 1e-6 {
            differs += 1;
        }
    }
    assert!(differs > 0, "no step where ρ matters; the test is vacuous");
}
