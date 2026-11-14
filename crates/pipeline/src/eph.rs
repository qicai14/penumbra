//! Ephemeris table builder (SCIENCE-SPEC §1c steps 1–2): apparent Sun and Earth, Moon-centred, MOON_ME, km.
//!
//! Each row: translate in J2000 with light time + stellar aberration (`LT_S`, P3), then rotate to
//! MOON_ME_DE440_ME421 at the same epoch. Rows are built from Unix seconds UTC, so the grid stays on
//! whole UTC hours; ANISE converts to TDB itself (P4, ADR 0002).

use anise::constants::frames::{EARTH_J2000, MOON_J2000, MOON_ME_DE440_ME421_FRAME, SUN_J2000};
use anise::prelude::{Aberration, Almanac, Epoch};
use std::error::Error;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

/// Kernels loaded, in order (E44: the binary PCK needs the frame kernel). Other files are not needed.
pub const KERNELS: [&str; 3] = ["de440.bsp", "moon_pa_de440_200625.bpc", "moon_fk_de440.epa"];

/// Prefix and suffix of the shipped table file, `eph-main.<hash6>.bin`.
pub const MAIN_FILE_PREFIX: &str = "eph-main.";
pub const MAIN_FILE_SUFFIX: &str = ".bin";

/// Load DE440, the DE440 Moon PA kernel and the Moon frame kernel from `dir`.
pub fn load_almanac(dir: &Path) -> Result<Almanac> {
    let mut almanac = Almanac::default();
    for name in KERNELS {
        let path = dir.join(name);
        let path = path.to_str().ok_or("kernel path is not UTF-8")?;
        almanac = almanac.load(path)?;
    }
    Ok(almanac)
}

/// [sun_x, sun_y, sun_z, earth_x, earth_y, earth_z], km, MOON_ME, Moon-centred, apparent, at `t_unix_s` (UTC).
pub fn moon_me_row(almanac: &Almanac, t_unix_s: f64) -> Result<[f64; 6]> {
    let epoch = Epoch::from_unix_seconds(t_unix_s);
    let dcm = almanac.rotate(MOON_J2000, MOON_ME_DE440_ME421_FRAME, epoch)?;
    let mut row = [0.0; 6];
    for (i, body) in [SUN_J2000, EARTH_J2000].into_iter().enumerate() {
        let state = almanac.translate(body, MOON_J2000, epoch, Aberration::LT_S)?;
        let v = dcm * state.radius_km;
        row[3 * i..3 * i + 3].copy_from_slice(&[v.x, v.y, v.z]);
    }
    Ok(row)
}

/// `rows` rows at `t0_unix_s + k·step_s` (UTC), row-major, 6 per row.
pub fn build_table(
    almanac: &Almanac,
    t0_unix_s: f64,
    step_s: f64,
    rows: usize,
) -> Result<Vec<f64>> {
    let mut data = Vec::with_capacity(6 * rows);
    for k in 0..rows {
        #[allow(clippy::cast_precision_loss)] // k < 2^52, exact
        let t_unix_s = t0_unix_s + k as f64 * step_s;
        data.extend_from_slice(&moon_me_row(almanac, t_unix_s)?);
    }
    Ok(data)
}

/// Little-endian f64 bytes of the table, no header (ARCHITECTURE §3.5).
pub fn to_le_bytes(data: &[f64]) -> Vec<u8> {
    data.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// First 6 hex characters of the FNV-1a-64 hash of `bytes`, for the cache-busting file name.
pub fn hash6(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    format!("{h:016x}")[..6].to_string()
}

/// Every `eph-main.*.bin` in `dir`.
pub fn shipped_table_paths(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.starts_with(MAIN_FILE_PREFIX) && name.ends_with(MAIN_FILE_SUFFIX) {
            paths.push(path);
        }
    }
    Ok(paths)
}

/// The single shipped main table in `dir`, decoded. Errors unless exactly one file is there.
pub fn read_shipped_table(dir: &Path) -> Result<Vec<f64>> {
    let paths = shipped_table_paths(dir)?;
    let [path] = paths.as_slice() else {
        return Err(format!(
            "expected one eph-main.*.bin in {}, found {}",
            dir.display(),
            paths.len()
        )
        .into());
    };
    let bytes = std::fs::read(path)?;
    let (words, rest) = bytes.as_chunks::<8>();
    if !rest.is_empty() {
        return Err("table length is not a whole number of f64".into());
    }
    Ok(words.iter().map(|w| f64::from_le_bytes(*w)).collect())
}
