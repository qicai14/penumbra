// `eph` and `dem` each merge into manifest.json; a rerun of one must keep the other's keys.
#[test]
fn merge_keeps_keys_from_both_steps() {
    let dir = std::env::temp_dir().join(format!("penumbra-manifest-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let _ = std::fs::remove_file(dir.join("manifest.json"));
    penumbra_pipeline::merge_manifest(&dir, vec![("regions", serde_json::json!([1]))]).unwrap();
    penumbra_pipeline::merge_manifest(&dir, vec![("ephemeris", serde_json::json!("e"))]).unwrap();
    let text = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    let m: serde_json::Value = serde_json::from_str(&text).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(m["regions"], serde_json::json!([1]));
    assert_eq!(m["ephemeris"], "e");
    assert_eq!(m["format_version"], 1);
}
