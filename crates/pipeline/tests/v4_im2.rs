//! T15 V4 statement 3 (SCIENCE-SPEC V4 details, §1f A3, N10) on the shipped tiles and replay
//! segment in web/public/data (runs in CI: no NASA data needed).

use penumbra_pipeline::v4::im2_statement3;
use std::path::PathBuf;

fn statement() -> serde_json::Value {
    let data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../web/public/data");
    im2_statement3(&data).unwrap()
}

fn num(v: &serde_json::Value, k: &str) -> f64 {
    v[k].as_f64()
        .unwrap_or_else(|| panic!("{k} missing in {v}"))
}

// Inputs are the spec's: touchdown 2025-03-06T17:28:00Z marked assumed (U8), 84.7906°S 29.1957°E,
// D_c = 20 m, d_c = 1.94 m (E112), a row for each H = 0, 1, 1.44, 2 m.
#[test]
fn statement3_reports_the_assumed_touchdown_and_one_row_per_panel_height() {
    let s = statement();
    assert_eq!(s["touchdown_utc"], "2025-03-06T17:28:00Z");
    assert_eq!(s["touchdown_status"], "assumed (U8)");
    assert!((num(&s, "touchdown_unix_s") - 1_741_282_080.0).abs() < 1e-9);
    assert!((num(&s, "pit_diameter_m") - 20.0).abs() < 1e-12);
    assert!((num(&s, "degraded_crater_depth_m") - 1.94).abs() < 0.005);
    let rows = s["rows"].as_array().unwrap();
    let hs: Vec<f64> = rows.iter().map(|r| num(r, "panel_height_m")).collect();
    assert_eq!(hs, [0.0, 1.0, 1.44, 2.0]);
}

// §1f (E104): el_sun at IM-2 cannot exceed 6.75°; el_pit per H is the spec's 11.0 / 5.4 / 2.9 / −0.3°.
#[test]
fn statement3_rows_respect_the_spec_bounds() {
    let s = statement();
    for (r, el_pit_deg) in s["rows"]
        .as_array()
        .unwrap()
        .iter()
        .zip([11.0, 5.4, 2.9, -0.3])
    {
        assert!(num(r, "sun_el_deg") <= 6.75, "{r}");
        assert!((num(r, "el_pit_deg") - el_pit_deg).abs() < 0.05, "{r}");
        let d_star = num(r, "depth_to_hide_m");
        assert_eq!(
            r["degraded_crater_exceeds_depth_to_hide"].as_bool(),
            Some(num(&s, "degraded_crater_depth_m") > d_star),
            "{r}"
        );
    }
}

// N10: worded "consistent with reports", never "validated", never a statement of what happened.
#[test]
fn statement3_wording_is_consistent_with_reports_and_never_validated() {
    let text = statement().to_string().to_lowercase();
    assert!(text.contains("consistent with reports"), "{text}");
    assert!(!text.contains("validat"), "{text}");
    for banned in ["caused", "was shaded", "proves", "confirmed"] {
        assert!(
            !text.contains(banned),
            "wording contains {banned:?}: {text}"
        );
    }
}
