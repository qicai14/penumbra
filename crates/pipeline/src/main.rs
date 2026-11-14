//! Native offline pipeline. `penumbra-pipeline eph <out_dir>` builds the hourly ephemeris table
//! from the kernels in PENUMBRA_DATA, writes `eph-main.<hash6>.bin` and `manifest.json` into
//! `out_dir`, and prints the tracer row (IM-2, 2026-11-14T00:00:00Z).

use penumbra_core::eph::{
    Ephemeris, MAIN_ROWS, MAIN_STEP_S, MAIN_T0_UNIX_S, Site, sun_earth_track,
};
use penumbra_pipeline::eph::{
    MAIN_FILE_PREFIX, MAIN_FILE_SUFFIX, Result, build_table, hash6, load_almanac,
    shipped_table_paths, to_le_bytes,
};
use std::path::{Path, PathBuf};

/// Tracer instant: 2026-11-14T00:00:00Z, Unix s UTC.
const TRACER_UNIX_S: f64 = 1_794_614_400.0;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    match args.as_slice() {
        [_, cmd, out_dir] if cmd == "eph" => eph(Path::new(out_dir)),
        _ => Err("usage: penumbra-pipeline eph <out_dir>  (kernels from PENUMBRA_DATA)".into()),
    }
}

fn eph(out_dir: &Path) -> Result<()> {
    let kernels = PathBuf::from(std::env::var("PENUMBRA_DATA").map_err(|_| "set PENUMBRA_DATA")?);
    let almanac = load_almanac(&kernels)?;
    let data = build_table(&almanac, MAIN_T0_UNIX_S, MAIN_STEP_S, MAIN_ROWS)?;
    let bytes = to_le_bytes(&data);
    let name = format!("{MAIN_FILE_PREFIX}{}{MAIN_FILE_SUFFIX}", hash6(&bytes));

    std::fs::create_dir_all(out_dir)?;
    for old in shipped_table_paths(out_dir)? {
        std::fs::remove_file(old)?;
    }
    std::fs::write(out_dir.join(&name), &bytes)?;
    let manifest = format!(
        "{{\"format_version\":1,\"api_version\":{},\"ephemeris\":[{{\"id\":\"main\",\"url\":\"data/{name}\",\"t0_unix_s\":{MAIN_T0_UNIX_S},\"step_s\":{MAIN_STEP_S},\"rows\":{MAIN_ROWS}}}]}}\n",
        penumbra_core::API_VERSION
    );
    std::fs::write(out_dir.join("manifest.json"), manifest)?;
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
