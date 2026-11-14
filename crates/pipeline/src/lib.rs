//! Native offline pipeline: builds the runtime data files from the NASA kernels and DEMs.

pub mod dem;
pub mod eph;
pub mod psr;
pub mod v4;

use std::path::Path;

/// Set `entries` in `out_dir/manifest.json`, keeping every other key, so the `eph` and `dem`
/// steps can run in either order. A new manifest starts with the format and API versions.
pub fn merge_manifest(out_dir: &Path, entries: Vec<(&str, serde_json::Value)>) -> eph::Result<()> {
    let path = out_dir.join("manifest.json");
    let mut m: serde_json::Map<String, serde_json::Value> = match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => serde_json::Map::new(),
        Err(e) => return Err(e.into()),
    };
    m.insert("format_version".into(), 1.into());
    m.insert("api_version".into(), penumbra_core::API_VERSION.into());
    for (k, v) in entries {
        m.insert(k.into(), v);
    }
    std::fs::write(path, serde_json::to_string(&m)? + "\n")?;
    Ok(())
}
