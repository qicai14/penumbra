//! Native offline pipeline. `penumbra-pipeline eph <out_dir>` builds the hourly ephemeris table
//! from the kernels in PENUMBRA_DATA, writes `eph-main.<hash6>.bin`, the IM-2 replay segment
//! `eph-replay-im2.<hash6>.bin` (2025-02-20 to 2025-04-06 UTC) and `manifest.json` into
//! `out_dir`, and prints the tracer row (IM-2, 2026-11-14T00:00:00Z).
//! `penumbra-pipeline dem <out_dir>` crops the two near tiles, their 80 m middle tiles (#48) and the
//! far tile (T06) and merges
//! them into the same `manifest.json`.
//! `penumbra-pipeline horizon <data_dir>` (release build) prints the T07 reports: T-1a-4 tier
//! ablation against the float 20 m DEM in PENUMBRA_DATA, T-1a-5 range check and U18 native timing.
//! `penumbra-pipeline v4-im2 <data_dir>` prints V4 statement 3 (IM-2 at its assumed touchdown,
//! pit depth to hide the Sun per panel height) as JSON from the shipped files; no kernels needed.
//! `penumbra-pipeline v2b <data_dir>` (release) prints the V2b longest-night table check (T19) as
//! JSON from the shipped tiles, with an hourly ephemeris built from the kernels in PENUMBRA_DATA.
//! `penumbra-pipeline v2 <data_dir>` (release, ~10 min on 8 cores) prints the V2 sample against
//! AVGVISIB and its Earth twin (T20) as JSON; kernels and both AVGVISIB rasters from PENUMBRA_DATA.

use penumbra_core::eph::{
    Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, REPLAY_IM2_ROWS, REPLAY_IM2_STEP_S,
    REPLAY_IM2_T0_UNIX_S, Site, sun_earth_track,
};
use penumbra_pipeline::eph::{
    MAIN_FILE_PREFIX, MAIN_FILE_SUFFIX, REPLAY_IM2_FILE_PREFIX, Result, build_table, hash6,
    load_almanac, shipped_table_paths, to_le_bytes,
};
use std::path::{Path, PathBuf};

/// Tracer instant: 2026-11-14T00:00:00Z, Unix s UTC.
const TRACER_UNIX_S: f64 = 1_794_614_400.0;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_, cmd, out_dir] if cmd == "eph" => eph(Path::new(out_dir)),
        [_, cmd, out_dir] if cmd == "dem" => dem(Path::new(out_dir)),
        [_, cmd, data_dir] if cmd == "horizon" => penumbra_pipeline::horizon_report::run(Path::new(data_dir)),
        [_, cmd, data_dir] if cmd == "v2b" => {
            let kernels =
                PathBuf::from(std::env::var("PENUMBRA_DATA").map_err(|_| "set PENUMBRA_DATA")?);
            let r = penumbra_pipeline::table_check::run(Path::new(data_dir), &kernels)?;
            println!("{}", serde_json::to_string_pretty(&r)?);
            Ok(())
        }
        [_, cmd, data_dir] if cmd == "v2" => {
            let resource =
                PathBuf::from(std::env::var("PENUMBRA_DATA").map_err(|_| "set PENUMBRA_DATA")?);
            let r = penumbra_pipeline::v2::run(Path::new(data_dir), &resource)?;
            println!("{}", serde_json::to_string_pretty(&r)?);
            Ok(())
        }
        [_, cmd, data_dir] if cmd == "v4-im2" => {
            let s = penumbra_pipeline::v4::im2_statement3(Path::new(data_dir))?;
            println!("{}", serde_json::to_string_pretty(&s)?);
            Ok(())
        }
        _ => Err(
            "usage: penumbra-pipeline eph|dem|horizon|v4-im2|v2b|v2 <dir>  (kernels and DEMs from PENUMBRA_DATA)"
                .into(),
        ),
    }
}

/// A whole number of seconds as an integer, so the manifest reads `3600`, not `3600.0`.
fn whole_s(t_s: f64) -> Result<i64> {
    if t_s.fract() != 0.0 || t_s.abs() > 9.0e15 {
        return Err(format!("{t_s} s is not a whole number of seconds").into());
    }
    #[allow(clippy::cast_possible_truncation)] // whole and below 2^53, checked above
    Ok(t_s as i64)
}

fn eph(out_dir: &Path) -> Result<()> {
    let kernels = PathBuf::from(std::env::var("PENUMBRA_DATA").map_err(|_| "set PENUMBRA_DATA")?);
    let almanac = load_almanac(&kernels)?;
    std::fs::create_dir_all(out_dir)?;
    let segments = [
        (
            "main",
            MAIN_FILE_PREFIX,
            MAIN_T0_UNIX_S,
            MAIN_STEP_S,
            MAIN_ROWS,
        ),
        (
            "replay-im2",
            REPLAY_IM2_FILE_PREFIX,
            REPLAY_IM2_T0_UNIX_S,
            REPLAY_IM2_STEP_S,
            REPLAY_IM2_ROWS,
        ),
    ];
    let mut entries = Vec::new();
    let mut written = Vec::new();
    let mut main_data = Vec::new();
    for (id, prefix, t0_unix_s, step_s, rows) in segments {
        let data = build_table(&almanac, t0_unix_s, step_s, rows)?;
        let bytes = to_le_bytes(&data);
        let name = format!("{prefix}{}{MAIN_FILE_SUFFIX}", hash6(&bytes));
        std::fs::write(out_dir.join(&name), &bytes)?;
        println!(
            "wrote {} ({} bytes)",
            out_dir.join(&name).display(),
            bytes.len()
        );
        // main stays first: the app loads ephemeris[0] as the main table
        entries.push(serde_json::json!({"id": id, "url": format!("data/{name}"),
            "t0_unix_s": whole_s(t0_unix_s)?, "step_s": whole_s(step_s)?, "rows": rows}));
        written.push((prefix, name));
        if id == "main" {
            main_data = data;
        }
    }
    penumbra_pipeline::merge_manifest(
        out_dir,
        vec![("ephemeris", serde_json::Value::Array(entries))],
    )?;
    // remove older tables only after the new tables and manifest are on disk
    for (prefix, name) in &written {
        for old in shipped_table_paths(out_dir, prefix)? {
            if old.file_name() != Some(std::ffi::OsStr::new(name)) {
                std::fs::remove_file(old)?;
            }
        }
    }
    println!("wrote manifest.json");

    let data = main_data;
    let mut eph = Ephemeris::default();
    eph.add(MAIN_T0_UNIX_S, MAIN_STEP_S, data)?;
    let im2 = Site {
        lat_rad: (-84.7906f64).to_radians(),
        lon_east_rad: 29.1957f64.to_radians(),
        ground_m: 0.0,
        mast_m: 2.0,
    };
    let t = sun_earth_track(&eph, &im2, TRACER_UNIX_S, MAIN_STEP_S, 1)?;
    println!("tracer IM-2 2026-11-14T00:00:00Z, ground 0 m, mast 2 m (topocentric ENU, deg):");
    println!("  sun   az {:.12} el {:.12}", t[1], t[2]);
    println!("  earth az {:.12} el {:.12}", t[5], t[6]);
    Ok(())
}

/// Near tile side, pixels: 30.72 km at 20 m, so a site at the centre has 15.36 km of margin (ARCHITECTURE §2.1).
const NEAR_PX: u32 = 1536;
/// Middle tile side, 80 m pixels (4 × 4 means of the 20 m source), same centre as the near tile:
/// ±39.68 km covers the 8.31 km half-width of the pickable area plus the 31 km mid range at map
/// scale ≤ 1.0032 (#48, SCIENCE-SPEC §1a tiers).
const MID_PX: u32 = 992;
/// Source pixels per middle-tile pixel side: 4 × 20 m = 80 m.
const MID_BLOCK_PX: u32 = 4;
/// Far tile side, pixels: ±382.08 km at 240 m about the pole (ARCHITECTURE §2.1).
const FAR_PX: u32 = 3184;

fn dem(out_dir: &Path) -> Result<()> {
    use penumbra_core::geometry::{LatLon, MapXy, latlon_to_xy};
    use penumbra_pipeline::dem::{
        GeoTiff, build_tile, encode_far, encode_finest, tile_json, to_le_bytes,
    };
    let src_dir = PathBuf::from(std::env::var("PENUMBRA_DATA").map_err(|_| "set PENUMBRA_DATA")?);
    let im2 = latlon_to_xy(LatLon {
        lat_rad: (-84.7906f64).to_radians(),
        lon_east_rad: 29.1957f64.to_radians(),
    })
    .map_err(|e| format!("{e:?}"))?;
    // Mons Mouton: centred on the IM-2 site. Shackleton ridge: between SP-01 (−11.31, −12.26) km
    // and Barker's Site 1 RoIs (−11.58 .. −10.23, −12.56 .. −11.55) km, so all sit ~15 km inside.
    let regions = [
        ("mons-mouton", "Mons Mouton", im2),
        (
            "shackleton-ridge",
            "Shackleton ridge",
            MapXy {
                x_m: -11_000.0,
                y_m: -12_000.0,
            },
        ),
    ];

    std::fs::create_dir_all(out_dir)?;
    let mut written = Vec::new();
    let mut write = |prefix: &str, tile: &penumbra_core::dem::DemTile| -> Result<String> {
        let bytes = to_le_bytes(tile);
        let name = format!("{prefix}.{}.bin", hash6(&bytes));
        std::fs::write(out_dir.join(&name), &bytes)?;
        let m = tile.meta();
        println!(
            "wrote {name}: {}² px at {} m, x0 {} y0 {} m, scale {} m, offset {} m",
            m.width, m.pixel_m, m.x0_m, m.y0_m, m.scale_m, m.offset_m
        );
        written.push(name.clone());
        Ok(format!("data/{name}"))
    };

    let mut near_src = GeoTiff::open(&src_dir.join("LDEM_80S_20MPP_ADJ.TIF"))?;
    let mut region_json = Vec::new();
    for (id, name, centre) in regions {
        let tile = build_tile(&mut near_src, centre, NEAR_PX, 1, encode_finest)?;
        let url = write(&format!("near-{id}"), &tile)?;
        let mid = build_tile(&mut near_src, centre, MID_PX, MID_BLOCK_PX, encode_finest)?;
        let mid_url = write(&format!("mid-{id}"), &mid)?;
        region_json.push(serde_json::json!({"id": id, "name": name,
            "near": tile_json(&url, tile.meta()), "mid": tile_json(&mid_url, mid.meta())}));
    }
    let mut far_src = GeoTiff::open(&src_dir.join("LDEM_60S_240MPP_ADJ.TIF"))?;
    let far = build_tile(
        &mut far_src,
        MapXy { x_m: 0.0, y_m: 0.0 },
        FAR_PX,
        1,
        encode_far,
    )?;
    let far_url = write("far-240m", &far)?;

    penumbra_pipeline::merge_manifest(
        out_dir,
        vec![
            ("far", tile_json(&far_url, far.meta())),
            ("regions", serde_json::Value::Array(region_json)),
        ],
    )?;
    // remove older tiles only after the new tiles and manifest are on disk
    for entry in std::fs::read_dir(out_dir)? {
        let name = entry?.file_name().to_string_lossy().into_owned();
        let is_tile = ["near-", "mid-", "far-"]
            .iter()
            .any(|p| name.starts_with(p))
            && name.ends_with(".bin");
        if is_tile && !written.contains(&name) {
            std::fs::remove_file(out_dir.join(name))?;
        }
    }
    Ok(())
}
