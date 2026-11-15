//! #26: the region list the `dem` step reads (crates/pipeline/regions.json).

use penumbra_pipeline::regions::{Region, parse_regions, regions};

#[test]
fn the_checked_in_list_holds_the_shipped_regions_in_order() {
    let r = regions().unwrap();
    let ids: Vec<&str> = r.iter().map(|g| g.id.as_str()).collect();
    assert_eq!(&ids[..2], ["mons-mouton", "shackleton-ridge"]);
    assert_eq!(ids.len(), 10);
    assert!(r.iter().all(|g| !g.evidence.is_empty()));
}

// T-1a-2: the IM-2 site (84.7906°S, 29.1957°E) maps to (+77.1, +138.0) km.
#[test]
fn a_lat_lon_centre_is_projected_to_map_metres() {
    let mm: Region = regions().unwrap().remove(0);
    assert!(
        (mm.centre.x_m / 1000.0 - 77.1).abs() < 0.05,
        "{}",
        mm.centre.x_m
    );
    assert!(
        (mm.centre.y_m / 1000.0 - 138.0).abs() < 0.05,
        "{}",
        mm.centre.y_m
    );
}

#[test]
fn a_map_centre_is_kept_as_given() {
    let sr = &regions().unwrap()[1];
    assert_eq!((sr.centre.x_m, sr.centre.y_m), (-11_000.0, -12_000.0));
}

#[test]
fn a_centre_with_both_or_neither_form_is_refused() {
    let entry = |centre: &str| {
        format!(
            r#"[{{"id":"a","name":"A","tier":1,"evidence":["E116"],"label":"",
            "centre":{centre}}}]"#
        )
    };
    assert!(
        parse_regions(&entry(
            r#"{"x_m":0,"y_m":0,"lat_deg":-89,"lon_east_deg":0}"#
        ))
        .is_err()
    );
    assert!(parse_regions(&entry(r#"{}"#)).is_err());
    assert!(parse_regions(&entry(r#"{"x_m":1000,"y_m":0}"#)).is_ok());
}

#[test]
fn a_bad_tier_or_a_repeated_id_is_refused() {
    let one = |id: &str, tier: u32| {
        format!(
            r#"{{"id":"{id}","name":"A","tier":{tier},"evidence":["E116"],"label":"",
            "centre":{{"x_m":1000,"y_m":0}}}}"#
        )
    };
    assert!(parse_regions(&format!("[{}]", one("a", 4))).is_err());
    assert!(parse_regions(&format!("[{},{}]", one("a", 1), one("a", 2))).is_err());
}

/// W16 outline of the region named `name`, projected to map metres (closing vertex dropped).
fn outline(name: &str) -> Vec<penumbra_core::geometry::MapXy> {
    use penumbra_core::geometry::{LatLon, latlon_to_xy};
    let gj: serde_json::Value = serde_json::from_str(include_str!(
        "../data/A3_9regions-lonlat-simplified.geojson"
    ))
    .unwrap();
    let f = gj["features"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| {
            f["properties"]["Region"]
                .as_str()
                .unwrap()
                .eq_ignore_ascii_case(name)
        })
        .unwrap_or_else(|| panic!("no W16 outline named {name}"));
    let mut ring: Vec<_> = f["geometry"]["coordinates"][0]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let (lon, lat) = (v[0].as_f64().unwrap(), v[1].as_f64().unwrap());
            latlon_to_xy(LatLon {
                lat_rad: lat.to_radians(),
                lon_east_rad: lon.to_radians(),
            })
            .unwrap()
        })
        .collect();
    if ring.first() == ring.last() {
        ring.pop();
    }
    ring
}

fn inside(p: &penumbra_core::geometry::MapXy, ring: &[penumbra_core::geometry::MapXy]) -> bool {
    let mut c = false;
    for i in 0..ring.len() {
        let (a, b) = (&ring[i], &ring[(i + ring.len() - 1) % ring.len()]);
        if (a.y_m > p.y_m) != (b.y_m > p.y_m)
            && p.x_m < (b.x_m - a.x_m) * (p.y_m - a.y_m) / (b.y_m - a.y_m) + a.x_m
        {
            c = !c;
        }
    }
    c
}

// #26: a tier-1 centre is the vertex mean of its W16 outline (within 4-decimal rounding), and
// the IM-2 tile (tier 2) sits inside the Mons Mouton Plateau outline.
#[test]
fn tier_one_centres_are_the_mean_of_their_nasa_outline() {
    for r in regions().unwrap() {
        match r.tier {
            1 => {
                let ring = outline(&r.name);
                let n = ring.len() as f64;
                let (mx, my) = ring
                    .iter()
                    .fold((0.0, 0.0), |(x, y), p| (x + p.x_m / n, y + p.y_m / n));
                let d = (r.centre.x_m - mx).hypot(r.centre.y_m - my);
                assert!(d < 10.0, "{}: centre {d:.1} m from the outline mean", r.id);
                assert!(inside(&r.centre, &ring), "{}", r.id);
            }
            2 => assert!(
                inside(&r.centre, &outline("Mons Mouton Plateau")),
                "{}",
                r.id
            ),
            _ => {
                let gj: serde_json::Value = serde_json::from_str(include_str!(
                    "../data/A3_9regions-lonlat-simplified.geojson"
                ))
                .unwrap();
                for f in gj["features"].as_array().unwrap() {
                    let name = f["properties"]["Region"].as_str().unwrap();
                    assert!(!inside(&r.centre, &outline(name)), "{} is in {name}", r.id);
                }
            }
        }
    }
}
