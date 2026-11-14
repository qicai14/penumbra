//! Battery state of charge, operable hours and the first lethal shadow (SCIENCE-SPEC §1e,
//! ARCHITECTURE §2.5). Energy Wh, power W, Δt in hours inside the balance; times Unix s UTC;
//! Sun az/el in the site's topocentric ENU frame, degrees, as the `Timeline` carries them.

use crate::eph::EphError;
use crate::timeline::Timeline;

/// Solar constant S₀ at 1 AU, W/m² (E57).
pub const SOLAR_CONSTANT_W_M2: f64 = 1366.1;
/// 1 AU in km (IAU 2012 Resolution B2; W8).
pub const AU_KM: f64 = 149_597_870.7;

/// Panel orientation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Panel {
    /// Always faces the Sun: n̂·ŝ = 1.
    SunTracking,
    /// Fixed unit normal in the site's topocentric ENU frame. A vertical panel facing azimuth a is
    /// (sin a, cos a, 0), which gives n̂·ŝ = cos(el)·cos(az − a).
    Fixed { east: f64, north: f64, up: f64 },
}

/// Lander preset: the battery and array fields `survival` consumes (docs/API.md `preset`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Preset {
    pub capacity_wh: f64,
    /// Charge at landing.
    pub start_wh: f64,
    /// Lethal charge E_min: the run dies at the first step with E ≤ this.
    pub min_wh: f64,
    pub panel_area_m2: f64,
    /// Array efficiency η, 0–1.
    pub efficiency: f64,
    pub panel: Panel,
    pub load_w: f64,
}

/// The one cited preset: the rover paper's battery, panel and driving load (E58a–d). Only the
/// fields in `ROVER_CITED_FIELDS` are cited; start charge (full), E_min (0, the §1e default) and
/// Sun tracking are assumed (U10).
pub const ROVER_PRESET: Preset = Preset {
    capacity_wh: 10_000.0, // E58a
    start_wh: 10_000.0,
    min_wh: 0.0,
    panel_area_m2: 1.5, // E58b
    efficiency: 0.30,   // E58c
    panel: Panel::SunTracking,
    load_w: 220.0, // E58d
};

/// Fields of `ROVER_PRESET` that are cited, by API key; every other field is "assumed" (U10).
pub const ROVER_CITED_FIELDS: [&str; 4] = ["capacity_wh", "panel_area_m2", "efficiency", "load_w"];

/// When the battery first reaches E_min, or that it never does in the span.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lethal {
    /// Interpolated from the unclamped value: t_k + Δt·(E_k − E_min)/(E_k − E*), Unix s UTC,
    /// and the same moment in hours after landing.
    At { t_unix_s: f64, after_landing_h: f64 },
    /// No lethal shadow before the timeline ends.
    NoneInSpan,
}

/// One survival run (docs/API.md `survival`). Hour totals cover the whole timeline span, each
/// step counting Δt, whether or not the battery died first (§1e).
#[derive(Debug, Clone, PartialEq)]
pub struct Survival {
    /// Σ (illumination > 0 and contact) × Δt.
    pub operable_h: f64,
    pub lethal: Lethal,
    /// Longest run with illumination = 0: "no part of the Sun is visible" (E123).
    pub longest_dark_h: f64,
    /// Longest run with illumination > 0 (E123).
    pub longest_any_sun_h: f64,
    /// Σ illumination × Δt.
    pub sunlit_h: f64,
    /// Σ contact × Δt.
    pub contact_h: f64,
    /// Charge at t_0, t_1, … up to the last step simulated: all steps, or through t_k of the lethal
    /// step, since the run stops there.
    pub soc_wh: Vec<f64>,
}

fn bad(m: &'static str) -> EphError {
    EphError::BadInput(m)
}

fn check(p: &Preset) -> Result<(), EphError> {
    let finite = [
        p.capacity_wh,
        p.start_wh,
        p.min_wh,
        p.panel_area_m2,
        p.efficiency,
        p.load_w,
    ]
    .iter()
    .all(|v| v.is_finite());
    if !finite
        || p.capacity_wh <= 0.0
        || !(0.0..=p.capacity_wh).contains(&p.min_wh)
        || !(0.0..=p.capacity_wh).contains(&p.start_wh)
        || p.panel_area_m2 < 0.0
        || !(0.0..=1.0).contains(&p.efficiency)
        || p.load_w < 0.0
    {
        return Err(bad(
            "preset must be finite, 0 ≤ min, start ≤ capacity, efficiency 0–1, area and load ≥ 0",
        ));
    }
    if let Panel::Fixed { east, north, up } = p.panel {
        let norm = (east * east + north * north + up * up).sqrt();
        if norm.is_nan() || (norm - 1.0).abs() > 1e-6 {
            return Err(bad("fixed panel normal must be a unit vector"));
        }
    }
    Ok(())
}

/// Longest run of consecutive steps where `pred` holds.
fn longest_run(xs: &[f64], pred: impl Fn(f64) -> bool) -> usize {
    let (mut best, mut cur) = (0, 0);
    for &x in xs {
        cur = if pred(x) { cur + 1 } else { 0 };
        best = best.max(cur);
    }
    best
}

/// Survive the timeline `tl` from `landing_unix_s` (Unix s UTC, which must be `tl.t_unix_s[0]`)
/// with preset `p`. Δt = t[1] − t[0]; the steps must be even. Power at step k is held over
/// [t_k, t_k + Δt] (explicit Euler, §1e); the run stops at the first lethal event.
#[allow(clippy::float_cmp)] // landing is the same f64 as t[0]; contact is exactly 0 or 1 (timeline)
pub fn survival(tl: &Timeline, p: &Preset, landing_unix_s: f64) -> Result<Survival, EphError> {
    check(p)?;
    let t = &tl.t_unix_s;
    let n = t.len();
    let cols = [
        &tl.sun_az_deg,
        &tl.sun_el_deg,
        &tl.illumination,
        &tl.contact,
        &tl.sun_dist_km,
    ];
    if n < 2 || cols.iter().any(|c| c.len() != n) {
        return Err(bad("timeline needs at least 2 steps and equal columns"));
    }
    if landing_unix_s != t[0] {
        return Err(bad("landing time must be the timeline's first step"));
    }
    let step_s = t[1] - t[0];
    if !step_s.is_finite()
        || step_s <= 0.0
        || t.windows(2)
            .any(|w| ((w[1] - w[0]) - step_s).abs() > 1e-6 * step_s)
    {
        return Err(bad("timeline steps must be even and increasing"));
    }
    if tl.illumination.iter().any(|f| !(0.0..=1.0).contains(f))
        || tl.contact.iter().any(|&c| c != 0.0 && c != 1.0)
        || tl.sun_dist_km.iter().any(|d| !(d.is_finite() && *d > 0.0))
        || tl
            .sun_az_deg
            .iter()
            .chain(&tl.sun_el_deg)
            .any(|a| !a.is_finite())
    {
        return Err(bad(
            "illumination must be 0–1, contact 0 or 1, Sun distance > 0, angles finite",
        ));
    }
    let dt_h = step_s / 3600.0;

    let mut soc_wh = vec![p.start_wh];
    let mut lethal = Lethal::NoneInSpan;
    if p.start_wh <= p.min_wh {
        lethal = Lethal::At {
            t_unix_s: t[0],
            after_landing_h: 0.0,
        };
    } else {
        for k in 0..n - 1 {
            let (az, el) = (tl.sun_az_deg[k].to_radians(), tl.sun_el_deg[k].to_radians());
            let cos_inc = match p.panel {
                Panel::SunTracking => 1.0,
                Panel::Fixed { east, north, up } => {
                    east * el.cos() * az.sin() + north * el.cos() * az.cos() + up * el.sin()
                }
            };
            let s_w_m2 = SOLAR_CONSTANT_W_M2 * (AU_KM / tl.sun_dist_km[k]).powi(2);
            let p_gen_w =
                s_w_m2 * p.panel_area_m2 * p.efficiency * cos_inc.max(0.0) * tl.illumination[k];
            let e_k = soc_wh[k];
            let e_star = e_k + (p_gen_w - p.load_w) * dt_h;
            if e_star <= p.min_wh {
                let t_lethal = t[k] + step_s * (e_k - p.min_wh) / (e_k - e_star);
                lethal = Lethal::At {
                    t_unix_s: t_lethal,
                    after_landing_h: (t_lethal - landing_unix_s) / 3600.0,
                };
                break;
            }
            soc_wh.push(e_star.min(p.capacity_wh));
        }
    }

    let illum = &tl.illumination;
    #[allow(clippy::cast_precision_loss)] // step counts are far below 2^52
    let hours = |steps: usize| steps as f64 * dt_h;
    Ok(Survival {
        operable_h: hours(
            illum
                .iter()
                .zip(&tl.contact)
                .filter(|&(&f, &c)| f > 0.0 && c == 1.0)
                .count(),
        ),
        lethal,
        longest_dark_h: hours(longest_run(illum, |f| f == 0.0)),
        longest_any_sun_h: hours(longest_run(illum, |f| f > 0.0)),
        sunlit_h: illum.iter().sum::<f64>() * dt_h,
        contact_h: tl.contact.iter().sum::<f64>() * dt_h,
        soc_wh,
    })
}
