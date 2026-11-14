//! T17 sunset ± band: SCIENCE-SPEC §1f step 1, docs/briefs/T17-sunset-band.md test table.
//! Timelines are synthetic: Unix s UTC every 600 s, Sun az/el in topocentric ENU degrees.

use penumbra_core::band::{ASSUMED_SIGMA_Z_M, MAX_HALF_WIDTH_MIN, sigma_el_rad, window_bands};
use penumbra_core::horizon::{Horizon, N_AZ};
use penumbra_core::timeline::Timeline;

const STEP_S: f64 = 600.0;
const T0_UNIX_S: f64 = 1_800_000_000.0;

/// A 10 × n API timeline (T08 layout) with only the Sun columns filled; the band reads no others.
fn tl(sun_az_deg: &[f64], sun_el_deg: &[f64]) -> Vec<f64> {
    let n = sun_el_deg.len();
    #[allow(clippy::cast_precision_loss)] // a handful of steps
    let t: Vec<f64> = (0..n).map(|k| T0_UNIX_S + STEP_S * k as f64).collect();
    let zero = vec![0.0; n];
    Timeline {
        t_unix_s: t,
        sun_az_deg: sun_az_deg.to_vec(),
        sun_el_deg: sun_el_deg.to_vec(),
        illumination: zero.clone(),
        visibility: zero.clone(),
        earth_az_deg: zero.clone(),
        earth_el_deg: zero.clone(),
        earth_margin_deg: zero.clone(),
        contact: zero.clone(),
        sun_dist_km: zero,
    }
    .to_api()
}

/// Horizon at `el_deg` (topocentric ENU) with every obstacle `dist_m` away.
fn flat(el_deg: f64, dist_m: f64) -> Horizon {
    Horizon {
        el_rad: [el_deg.to_radians(); N_AZ],
        dist_m: [dist_m; N_AZ],
    }
}

fn bands(az_deg: &[f64], el_deg: &[f64], h: &Horizon) -> Vec<f64> {
    window_bands(&tl(az_deg, el_deg), h, ASSUMED_SIGMA_Z_M).unwrap()
}

fn assert_close(got: f64, want: f64, tol: f64) {
    assert!((got - want).abs() <= tol, "got {got}, want {want}");
}

fn assert_deg(got_rad: f64, want_deg: f64, tol_deg: f64) {
    let got_deg = got_rad.to_degrees();
    assert!(
        (got_deg - want_deg).abs() < tol_deg,
        "got {got_deg}°, want {want_deg}°"
    );
}

// Row 1, D4: the spec's "0.5 m on an obstacle 1 km away is 0.03°, 0.003° at 10 km" is the
// obstacle error alone (0.5/1000 rad = 0.028648°); with both ends at 0.5 m it is √2 larger.
#[test]
fn sigma_el_matches_the_spec_numbers() {
    assert_deg(sigma_el_rad(0.5, 0.0, 1_000.0).unwrap(), 0.028_648, 1e-6);
    assert_deg(sigma_el_rad(0.5, 0.0, 10_000.0).unwrap(), 0.002_864_8, 1e-6);
    assert_deg(sigma_el_rad(0.5, 0.5, 1_000.0).unwrap(), 0.040_514, 1e-6);
    assert_deg(sigma_el_rad(0.5, 0.5, 2_000.0).unwrap(), 0.020_257, 1e-6);
}

#[test]
fn sigma_el_rejects_a_zero_distance() {
    assert!(sigma_el_rad(0.5, 0.5, 0.0).is_err());
}

// Row 2: m falls 0.0084667° in 600 s across a flat 1° horizon 2 km away. σ = 0.020257° (row 1),
// so the half width is 0.020257 / 0.0084667 × 10 min = 23.93 min, the sunset at mid-step.
#[test]
fn sunset_band_is_sigma_over_the_central_difference_rate() {
    let b = bands(
        &[100.0, 100.0],
        &[1.004_233, 0.995_767],
        &flat(1.0, 2_000.0),
    );
    assert_eq!(b.len(), 3);
    assert_close(b[0], T0_UNIX_S + 300.0, 1e-3);
    assert_close(b[1], -1.0, 0.0);
    assert_close(b[2], 23.93, 0.01);
}

// Row 3: the same crossing reversed is a sunrise with the same band.
#[test]
fn sunrise_band_matches_the_reversed_sunset() {
    let b = bands(
        &[100.0, 100.0],
        &[0.995_767, 1.004_233],
        &flat(1.0, 2_000.0),
    );
    assert_close(b[0], T0_UNIX_S + 300.0, 1e-3);
    assert_close(b[1], 1.0, 0.0);
    assert_close(b[2], 23.93, 0.01);
}

// Row 4: σ_el scales as 1/D, so 10 km gives 23.93 × 2 000 / 10 000 = 4.785 min.
#[test]
fn band_shrinks_with_obstacle_distance() {
    let b = bands(
        &[100.0, 100.0],
        &[1.004_233, 0.995_767],
        &flat(1.0, 10_000.0),
    );
    assert_close(b[2], 4.785, 0.01);
}

// Row 5: no sign change, no event.
#[test]
fn sun_always_above_the_horizon_gives_no_band() {
    assert!(bands(&[100.0; 3], &[1.5, 1.4, 1.3], &flat(1.0, 2_000.0)).is_empty());
}

// Row 6: a near-flat crossing (m = ±1e-7°) is a wide band, finite, never a single time: about
// 16 880 h uncapped, so the cap of one 29.5-day synodic month (E118) applies.
#[test]
fn near_flat_crossing_gives_a_wide_finite_band() {
    let b = bands(
        &[100.0, 100.0],
        &[1.0 + 1e-7, 1.0 - 1e-7],
        &flat(1.0, 2_000.0),
    );
    assert_eq!(b.len(), 3);
    assert!(
        b[2].is_finite() && b[2] > 6.0 * 60.0,
        "half width {} min",
        b[2]
    );
    assert_close(b[2], MAX_HALF_WIDTH_MIN, 1e-9);
}

// The cap is one synodic month, 29.5 d (E118), in minutes.
#[test]
fn half_width_cap_is_one_synodic_month() {
    assert_close(MAX_HALF_WIDTH_MIN, 29.5 * 24.0 * 60.0, 1e-9);
}

// A crossing of a few ulps still gives the finite cap, never inf or NaN.
#[test]
fn subnormal_rate_never_divides_to_infinity() {
    let b = bands(
        &[100.0, 100.0],
        &[f64::MIN_POSITIVE, -f64::MIN_POSITIVE],
        &flat(0.0, 2_000.0),
    );
    assert_close(b[2], MAX_HALF_WIDTH_MIN, 1e-9);
}

// Row 7: m = 0 exactly on a sample counts once: a sunset at t_k, or a sunrise at t_{k+1} (D5).
#[test]
fn zero_margin_on_a_sample_is_one_event_at_that_sample() {
    let set = bands(&[100.0, 100.0], &[0.0, -0.01], &flat(0.0, 2_000.0));
    assert_eq!((set.len(), set[0], set[1]), (3, T0_UNIX_S, -1.0));
    let rise = bands(&[100.0, 100.0], &[-0.01, 0.0], &flat(0.0, 2_000.0));
    assert_eq!((rise.len(), rise[0], rise[1]), (3, T0_UNIX_S + STEP_S, 1.0));
}

// Row 8, N8: a sunset as the Sun crosses north (0.2° → 359.8°) reads D at 0°, not at 180°.
#[test]
fn crossing_north_reads_the_distance_on_the_near_side() {
    let mut h = flat(1.0, 90_000.0);
    for i in [N_AZ - 1, 0, 1] {
        h.dist_m[i] = 2_000.0;
    }
    let b = bands(&[0.2, 359.8], &[1.004_233, 0.995_767], &h);
    assert_close(b[2], 23.93, 0.01);
}

// §1f worked example only; NOT the E28 check (that needs a real IM-2 horizon, brief row 9):
// the Sun moving 0.5°/h in azimuth (E107) at constant elevation over a horizon falling 0.1° per
// degree gives dm/dt = 0.05°/h. With D = 1 km, σ_el = 0.040514° (row 1), so the half width is
// 0.040514 / 0.05 h = 48.62 min: small, under an hour (brief row 9).
#[test]
fn sloping_horizon_band_follows_the_spec_worked_example() {
    let mut h = flat(0.0, 1_000.0);
    #[allow(clippy::cast_precision_loss)] // i < 720
    for (i, el) in h.el_rad.iter_mut().enumerate() {
        *el = (-0.1 * (i as f64 * 0.5 - 100.0)).to_radians();
    }
    let step_deg = 0.5 * STEP_S / 3_600.0;
    let az: Vec<f64> = (0..4).map(|k| 99.9 + step_deg * f64::from(k)).collect();
    let b = bands(&az, &[0.0; 4], &h);
    assert_eq!((b.len(), b[1]), (3, 1.0));
    assert_close(b[2], 48.62, 0.01);
    assert!(b[2] < 60.0);
}

// Row 10: one band is [t, kind, half_width_min]; a timeline not 10 × n is BAD_INPUT.
#[test]
fn bad_timeline_length_is_bad_input() {
    let mut t = tl(&[100.0, 100.0], &[1.004_233, 0.995_767]);
    t.pop();
    let e = window_bands(&t, &flat(1.0, 2_000.0), ASSUMED_SIGMA_Z_M).unwrap_err();
    assert_eq!(e.code(), "BAD_INPUT");
}

// Review fix: σ_z ≤ 0 would give a zero-width band (a single time), NaN would silently skip events.
#[test]
fn non_positive_or_nan_sigma_z_is_bad_input() {
    let t = tl(&[100.0, 100.0], &[1.004_233, 0.995_767]);
    for s in [0.0, -0.5, f64::NAN] {
        let e = window_bands(&t, &flat(1.0, 2_000.0), s).unwrap_err();
        assert_eq!(e.code(), "BAD_INPUT", "σ_z = {s}");
    }
}

// Review fix: a NaN in the horizon makes m NaN, which no sign test catches, so it must throw.
#[test]
fn non_finite_horizon_is_bad_input() {
    let t = tl(&[100.0, 100.0], &[1.004_233, 0.995_767]);
    let mut h = flat(1.0, 2_000.0);
    h.el_rad[200] = f64::NAN;
    assert_eq!(
        window_bands(&t, &h, ASSUMED_SIGMA_Z_M).unwrap_err().code(),
        "BAD_INPUT"
    );
    let mut h = flat(1.0, 2_000.0);
    h.dist_m[5] = f64::INFINITY;
    assert_eq!(
        window_bands(&t, &h, ASSUMED_SIGMA_Z_M).unwrap_err().code(),
        "BAD_INPUT"
    );
}
