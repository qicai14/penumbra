//! #58 drop path: `web/src/lib/projection-ref.json` holds `core::geometry` answers at SP-01, IM-2 and
//! the four corners of every near tile in the shipped manifest. This test pins the file to Rust;
//! `web/src/lib/projection.test.ts` pins the TS projection to the same file. Each side is held to
//! 0.5 mm, so TS and Rust agree to 1 mm. Regenerate after a tile or projection change with
//! `PROJECTION_REF_WRITE=1 cargo test -p penumbra-pipeline --test projection_ref`.

use penumbra_core::geometry::{LatLon, MapXy, latlon_to_xy, xy_to_latlon};
use serde_json::{Value, json};
use std::path::PathBuf;

const R_M: f64 = 1_737_400.0;
const TOL_M: f64 = 0.0005;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn num(v: &Value, k: &str) -> f64 {
    v[k].as_f64()
        .unwrap_or_else(|| panic!("{k} missing in {v}"))
}

fn from_latlon(name: &str, lat_deg: f64, lon_deg: f64) -> Value {
    let xy = latlon_to_xy(LatLon {
        lat_rad: lat_deg.to_radians(),
        lon_east_rad: lon_deg.to_radians(),
    })
    .unwrap();
    json!({ "name": name, "lat_deg": lat_deg, "lon_deg": lon_deg, "x_m": xy.x_m, "y_m": xy.y_m })
}

fn from_xy(name: &str, x_m: f64, y_m: f64) -> Value {
    let ll = xy_to_latlon(MapXy { x_m, y_m }).unwrap();
    json!({ "name": name, "lat_deg": ll.lat_rad.to_degrees(), "lon_deg": ll.lon_east_rad.to_degrees(), "x_m": x_m, "y_m": y_m })
}

/// SP-01 and IM-2 (T01 brief rows 1–2) plus the four corners of each near tile in the manifest.
fn expected() -> Vec<Value> {
    let text = std::fs::read_to_string(root().join("web/public/data/manifest.json")).unwrap();
    let m: Value = serde_json::from_str(&text).unwrap();
    let mut pts = vec![
        from_latlon("SP-01", -89.45, 222.69),
        from_latlon("IM-2", -84.7906, 29.1957),
    ];
    for r in m["regions"].as_array().unwrap() {
        let n = &r["near"];
        let (x0, y0, p) = (num(n, "x0_m"), num(n, "y0_m"), num(n, "pixel_m"));
        let (x1, y1) = (x0 + num(n, "width") * p, y0 - num(n, "height") * p);
        let id = r["id"].as_str().unwrap();
        for (corner, x, y) in [
            ("NW", x0, y0),
            ("NE", x1, y0),
            ("SW", x0, y1),
            ("SE", x1, y1),
        ] {
            pts.push(from_xy(&format!("{id} {corner}"), x, y));
        }
    }
    pts
}

/// Surface distance in metres between two (lat, lon) pairs, small-angle form; lon wraps mod 360.
fn ground_m(a_lat: f64, a_lon: f64, b_lat: f64, b_lon: f64) -> f64 {
    let dlon = ((a_lon - b_lon + 540.0) % 360.0) - 180.0;
    let dn = (a_lat - b_lat).to_radians() * R_M;
    let de = dlon.to_radians() * R_M * a_lat.to_radians().cos();
    dn.hypot(de)
}

#[test]
fn reference_file_matches_core_geometry_to_half_a_millimetre() {
    let path = root().join("web/src/lib/projection-ref.json");
    let want = expected();
    if std::env::var_os("PROJECTION_REF_WRITE").is_some() {
        std::fs::write(&path, serde_json::to_string_pretty(&want).unwrap() + "\n").unwrap();
    }
    let text = std::fs::read_to_string(&path).unwrap();
    let got: Vec<Value> = serde_json::from_str(&text).unwrap();
    assert_eq!(
        got.len(),
        want.len(),
        "point count; regenerate (see the file header)"
    );
    for (g, w) in got.iter().zip(&want) {
        assert_eq!(g["name"], w["name"]);
        let dxy = (num(g, "x_m") - num(w, "x_m")).hypot(num(g, "y_m") - num(w, "y_m"));
        let dll = ground_m(
            num(g, "lat_deg"),
            num(g, "lon_deg"),
            num(w, "lat_deg"),
            num(w, "lon_deg"),
        );
        assert!(
            dxy <= TOL_M && dll <= TOL_M,
            "{}: xy off {dxy} m, latlon off {dll} m",
            w["name"]
        );
    }
}
