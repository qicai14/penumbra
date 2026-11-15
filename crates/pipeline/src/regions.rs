//! Region list for the `dem` step (#26): one near and one middle tile per entry of
//! `crates/pipeline/regions.json`, built in file order.
//!
//! Each entry: `id` (tile and manifest key), `name`, `centre`, `tier`, `evidence` (SCIENCE-SPEC
//! §6 ledger ids for the centre and the region), `label` (a short note for the UI). `centre` is
//! exactly one of `{"lat_deg", "lon_east_deg"}` (MOON_ME planetocentric, degrees, east-positive)
//! or `{"x_m", "y_m"}` (south polar stereographic map metres, SCIENCE-SPEC §1). `tier` says how
//! the centre is anchored: 1 = the vertex mean of NASA's 2024 region outline (W16), 2 = a
//! published site inside one of those outlines, 3 = not a 2024 region (validation tile).

use crate::eph::Result;
use penumbra_core::geometry::{LatLon, MapXy, latlon_to_xy};
use serde_json::Value;

/// One region, its centre resolved to map metres.
#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub id: String,
    pub name: String,
    /// Tile centre, south polar stereographic map metres.
    pub centre: MapXy,
    pub tier: u8,
    pub evidence: Vec<String>,
    pub label: String,
}

/// The checked-in list, `crates/pipeline/regions.json`.
pub fn regions() -> Result<Vec<Region>> {
    parse_regions(include_str!("../regions.json"))
}

/// Parse a region list; refuses a missing field, a centre with both or neither form, a tier
/// outside 1..=3 and a repeated id.
pub fn parse_regions(json: &str) -> Result<Vec<Region>> {
    let entries: Vec<Value> = serde_json::from_str(json)?;
    let mut out: Vec<Region> = Vec::new();
    for e in &entries {
        let text = |k: &str| -> Result<String> {
            Ok(e[k]
                .as_str()
                .ok_or(format!("region: no text `{k}`"))?
                .to_owned())
        };
        let id = text("id")?;
        if out.iter().any(|r| r.id == id) {
            return Err(format!("region {id}: id repeated").into());
        }
        let tier = e["tier"]
            .as_u64()
            .and_then(|t| u8::try_from(t).ok())
            .filter(|t| (1..=3).contains(t))
            .ok_or(format!("region {id}: tier must be 1, 2 or 3"))?;
        let evidence = e["evidence"]
            .as_array()
            .ok_or(format!("region {id}: no `evidence` list"))?
            .iter()
            .map(|v| v.as_str().map(str::to_owned))
            .collect::<Option<Vec<_>>>()
            .ok_or(format!("region {id}: evidence ids must be text"))?;
        let c = &e["centre"];
        let num = |k: &str| c[k].as_f64();
        let centre = match (num("lat_deg"), num("lon_east_deg"), num("x_m"), num("y_m")) {
            (Some(lat_deg), Some(lon_east_deg), None, None) => latlon_to_xy(LatLon {
                lat_rad: lat_deg.to_radians(),
                lon_east_rad: lon_east_deg.to_radians(),
            })
            .map_err(|err| format!("region {id}: {err:?}"))?,
            (None, None, Some(x_m), Some(y_m)) => MapXy { x_m, y_m },
            _ => {
                return Err(format!(
                    "region {id}: centre needs lat_deg + lon_east_deg or x_m + y_m, not both"
                )
                .into());
            }
        };
        out.push(Region {
            name: text("name")?,
            label: text("label")?,
            id,
            centre,
            tier,
            evidence,
        });
    }
    Ok(out)
}
