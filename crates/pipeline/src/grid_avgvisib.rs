//! #25 acceptance: the grid's chain against AVGVISIB on the cells it shares.
//!
//! A grid cell stores a 30-day number, AVGVISIB is a share over 18.6 years, so the two cannot be
//! compared directly. This runs the grid's own chain (`site_horizon`: the shipped 20 m near, 80 m
//! middle and 240 m far tiers, at the cell centre) with the grid's timeline over V2's span (hourly
//! from 2010-01-01, 163 048 steps) and compares the Sun-visibility share (f > 0, P1) with the
//! AVGVISIB pixel holding the cell centre, under V2's stratified draw and pass rule. It is run at
//! the grid's 2 m mast and at AVGVISIB's declared 0 m (P2): the 2 m run moves the mean by the P2 amount.

use crate::dem::ShippedTiles;
use crate::eph::{Result, build_table, load_almanac};
use crate::grid::{GRID_PIXEL_M, GridSpec};
use crate::psr::{START_UNIX_S, STEPS};
use crate::v2::{Candidate, RefRaster, bin_of, cell_of, par_map, passes, stats, stratified_draw};
use penumbra_core::eph::Ephemeris;
use penumbra_core::geometry::xy_to_latlon;
use penumbra_core::site::site_horizon;
use penumbra_core::timeline::{DEFAULT_EARTH_MARGIN_DEG, timeline};
use std::path::Path;

/// Cells drawn per fifth of the AVGVISIB value (V2 draws 2 000; a region has far fewer cells).
pub const QUOTA: usize = 500;

fn stats_json(ours: &[f64], reference: &[f64]) -> serde_json::Value {
    match stats(ours, reference) {
        Ok(s) => serde_json::json!({
            "n": s.n, "median_abs": s.median_abs, "p90_abs": s.p90_abs,
            "mean_signed": s.mean, "r": s.r,
        }),
        Err(e) => serde_json::json!({"n": ours.len(), "error": e.to_string()}),
    }
}

/// `data_dir` is `web/public/data`, `resource_dir` is PENUMBRA_DATA (kernels and AVGVISIB).
pub fn run(data_dir: &Path, resource_dir: &Path, region: &str) -> Result<serde_json::Value> {
    let started = std::time::Instant::now();
    let tiles = ShippedTiles::load(data_dir)?;
    let (near, mid) = (tiles.near(region)?, tiles.mid(region)?);
    let spec = GridSpec::usable(
        near.meta(),
        GRID_PIXEL_M,
        penumbra_core::horizon::pick_border_m(near.meta()),
    )?;
    let sun_ref = RefRaster::open(&resource_dir.join("AVGVISIB_85S_060M_201608.TIF"))?;

    // Shared cells: those whose centre falls in a valid AVGVISIB pixel (distinct cells, distinct pixels: 100 m > 60 m).
    let mut cells: std::collections::BTreeMap<(i64, i64), (usize, f64)> = Default::default();
    for k in 0..spec.cells() {
        let (col, row) = cell_of(spec.centre(k))?;
        if let Ok(v) = sun_ref.at_cell(col, row) {
            cells.insert((row, col), (k, v));
        }
    }
    let cands = cells
        .iter()
        .map(|(&(row, col), &(_, ref_pct))| {
            Ok(Candidate {
                col: u32::try_from(col)?,
                row: u32::try_from(row)?,
                ref_pct,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let mut avail = [0usize; 5];
    for c in &cands {
        avail[bin_of(c.ref_pct)?] += 1;
    }
    let bins = stratified_draw(&cands, crate::v2::SEED, QUOTA)?;
    let drawn: Vec<usize> = bins.iter().map(Vec::len).collect();
    let picks: Vec<(usize, usize, f64)> = bins
        .iter()
        .enumerate()
        .flat_map(|(b, v)| {
            v.iter()
                .map(|c| {
                    let (k, r) = cells[&(i64::from(c.row), i64::from(c.col))];
                    (b, k, r)
                })
                .collect::<Vec<_>>()
        })
        .collect();
    eprintln!(
        "grid-v2: {} shared cells, avail {avail:?}, drawn {drawn:?}",
        cands.len()
    );

    let almanac = load_almanac(resource_dir)?;
    let mut eph = Ephemeris::default();
    eph.add(
        START_UNIX_S,
        3600.0,
        build_table(&almanac, START_UNIX_S, 3600.0, STEPS)?,
    )
    .map_err(|e| format!("{e}"))?;
    eprintln!(
        "grid-v2: table built ({:.0} s)",
        started.elapsed().as_secs_f64()
    );

    let count = u32::try_from(STEPS)?;
    let mut out = serde_json::Map::new();
    for mast_m in [0.0, 2.0] {
        let ours = par_map(&picks, |&(_, k, _)| {
            let ll = xy_to_latlon(spec.centre(k)).map_err(|e| format!("cell {k}: {e:?}"))?;
            let sh = site_horizon(
                near,
                mid,
                &tiles.far,
                ll.lat_rad.to_degrees(),
                ll.lon_east_rad.to_degrees(),
                mast_m,
            )
            .map_err(|e| format!("cell {k}: {e:?}"))?;
            let tl = timeline(
                &eph,
                &sh.el_hor_rad,
                &sh.site,
                START_UNIX_S,
                3600.0,
                count,
                DEFAULT_EARTH_MARGIN_DEG.to_radians(),
            )
            .map_err(|e| format!("cell {k}: {e}"))?;
            #[allow(clippy::cast_precision_loss)] // 163 048
            Ok(100.0 * tl.visibility.iter().sum::<f64>() / tl.visibility.len() as f64)
        })?;
        let refs: Vec<f64> = picks.iter().map(|p| p.2).collect();
        let per_bin: Vec<_> = (0..5)
            .map(|b| {
                let (o, r): (Vec<f64>, Vec<f64>) = picks
                    .iter()
                    .zip(&ours)
                    .filter(|(p, _)| p.0 == b)
                    .map(|(p, v)| (*v, p.2))
                    .unzip();
                let mut j = stats_json(&o, &r);
                j["bin"] = format!("{}-{}", 20 * b, 20 * (b + 1)).into();
                j
            })
            .collect();
        let max_abs = ours
            .iter()
            .zip(&refs)
            .map(|(o, r)| (o - r).abs())
            .fold(0.0, f64::max);
        let mut pooled = stats_json(&ours, &refs);
        pooled["pass"] = stats(&ours, &refs).is_ok_and(|s| passes(&s)).into();
        out.insert(
            format!("mast_{mast_m}m"),
            serde_json::json!({"pooled": pooled, "per_bin": per_bin, "max_abs_error": max_abs}),
        );
        eprintln!(
            "grid-v2: mast {mast_m} m done ({:.0} s)",
            started.elapsed().as_secs_f64()
        );
    }
    Ok(serde_json::json!({
        "metric": "Sun visibility (any part of the disc, f > 0), points; e = ours - AVGVISIB",
        "region": region,
        "chain": "penumbra_core::site::site_horizon (shipped 20/80/240 m tiers) + core timeline, hourly from 2010-01-01T00Z, 163048 steps",
        "shared_cells": cands.len(),
        "available_per_bin": avail,
        "drawn_per_bin": drawn,
        "seed": format!("{:#x}", crate::v2::SEED),
        "results": out,
        "run_time_s": started.elapsed().as_secs_f64(),
    }))
}
