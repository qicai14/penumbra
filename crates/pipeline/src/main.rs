//! Native offline pipeline. `penumbra-pipeline eph <out_dir>` builds the hourly ephemeris table
//! from the kernels in PENUMBRA_DATA, writes `eph-main.<hash6>.bin` and `manifest.json` into
//! `out_dir`, and prints the tracer row (IM-2, 2026-11-14T00:00:00Z).
//! `penumbra-pipeline dem <out_dir>` crops the two near tiles and the far tile (T06) and merges
//! them into the same `manifest.json`.
//! `penumbra-pipeline horizon <data_dir>` (release build) prints the T07 reports: T-1a-4 tier
//! ablation against the float 20 m DEM in PENUMBRA_DATA, T-1a-5 range check and U18 native timing.

use penumbra_core::eph::{
    Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, Site, sun_earth_track,
};
use penumbra_pipeline::eph::{
    MAIN_FILE_PREFIX, MAIN_FILE_SUFFIX, Result, build_table, hash6, load_almanac,
    shipped_table_paths, to_le_bytes,
};
use std::path::{Path, PathBuf};

mod horizon_report;

/// Tracer instant: 2026-11-14T00:00:00Z, Unix s UTC.
const TRACER_UNIX_S: f64 = 1_794_614_400.0;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_, cmd, out_dir] if cmd == "eph" => eph(Path::new(out_dir)),
        [_, cmd, out_dir] if cmd == "dem" => dem(Path::new(out_dir)),
        [_, cmd, data_dir] if cmd == "horizon" => horizon_report::run(Path::new(data_dir)),
        _ => Err(
            "usage: penumbra-pipeline eph|dem|horizon <dir>  (kernels and DEMs from PENUMBRA_DATA)"
                .into(),
        ),
    }
}

fn eph(out_dir: &Path) -> Result<()> {
    let kernels = PathBuf::from(std::env::var("PENUMBRA_DATA").map_err(|_| "set PENUMBRA_DATA")?);
    let almanac = load_almanac(&kernels)?;
    let data = build_table(&almanac, MAIN_T0_UNIX_S, MAIN_STEP_S, MAIN_ROWS)?;
    let bytes = to_le_bytes(&data);
    let name = format!("{MAIN_FILE_PREFIX}{}{MAIN_FILE_SUFFIX}", hash6(&bytes));

    std::fs::create_dir_all(out_dir)?;
    std::fs::write(out_dir.join(&name), &bytes)?;
    penumbra_pipeline::merge_manifest(
        out_dir,
        vec![(
            "ephemeris",
            serde_json::json!([{"id": "main", "url": format!("data/{name}"), "t0_unix_s": MAIN_T0_UNIX_S,
                "step_s": MAIN_STEP_S, "rows": MAIN_ROWS}]),
        )],
    )?;
    // remove older tables only after the new table and manifest are on disk
    for old in shipped_table_paths(out_dir)? {
        if old.file_name() != Some(std::ffi::OsStr::new(&name)) {
            std::fs::remove_file(old)?;
        }
    }
    println!(
        "wrote {} ({} bytes) and manifest.json",
        out_dir.join(&name).display(),
        bytes.len()
    );

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
/// Far tile side, pixels: ±382.08 km at 240 m about the pole (ARCHITECTURE §2.1).
const FAR_PX: u32 = 3184;

fn dem(out_dir: &Path) -> Result<()> {
    use penumbra_core::geometry::{LatLon, MapXy, latlon_to_xy};
    use penumbra_pipeline::dem::{
        FAR_SCALES_M, GeoTiff, NEAR_SCALES_M, build_tile, tile_json, to_le_bytes,
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
        let tile = build_tile(&mut near_src, centre, NEAR_PX, &NEAR_SCALES_M)?;
        let url = write(&format!("near-{id}"), &tile)?;
        region_json.push(
            serde_json::json!({"id": id, "name": name, "near": tile_json(&url, tile.meta())}),
        );
    }
    let mut far_src = GeoTiff::open(&src_dir.join("LDEM_60S_240MPP_ADJ.TIF"))?;
    let far = build_tile(
        &mut far_src,
        MapXy { x_m: 0.0, y_m: 0.0 },
        FAR_PX,
        &FAR_SCALES_M,
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
        let is_tile =
            (name.starts_with("near-") || name.starts_with("far-")) && name.ends_with(".bin");
        if is_tile && !written.contains(&name) {
            std::fs::remove_file(out_dir.join(name))?;
        }
    }
    Ok(())
}
