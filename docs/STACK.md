# Stack — Penumbra

Each choice states why it beats the mainstream default. Decided Nov 8 (`/grilling`).

| Layer | Choice | Instead of | Why |
|-------|--------|-----------|-----|
| Science core | **Rust** Cargo workspace: `core` (pure maths, no I/O), `pipeline` (native), `wasm` (wasm-bindgen wrapper) | Python/NumPy | One implementation runs offline and in the browser, so the two can't drift. Native speed for horizon ray-marching. Pure `core` is easy to test-drive. |
| Ephemeris | **ANISE** crate (native only) + NAIF kernels `de440.bsp`, `moon_pa_de440_200625.bpc`, `moon_fk_de440.epa` | SpiceyPy, CSPICE | Pure Rust SPICE reader with MOON_ME/MOON_PA frames. It compiles for wasm32 (checked Nov 12) but the kernels are 133 MB, so vectors are precomputed as an hourly table (about 2.1 MB). |
| DEM I/O | pure-Rust **`tiff`** crate | GDAL | No native GIS dependency on Windows or CI. The GeoTIFF is a single known product. It is a cloud-optimized GeoTIFF: read it tile by tile, never whole (3.7 GB decoded), and test the 148 MB PSR file at hour 0 (SCIENCE-SPEC §1a). |
| WASM bindings | **wasm-bindgen** (CLI) directly | wasm-pack | The rustwasm org (wasm-pack) was archived in 2025. wasm-bindgen generates the `.d.ts`. |
| Frontend | **Svelte 5 + Vite**, run with **Bun** | React / Next.js | Compiled framework, small bundle, little state boilerplate. One heavy canvas plus panels doesn't need React. No SSR needed. |
| 3D | **plain Three.js**; per-vertex illumination computed by our WASM model | React Three Fiber / Threlte; GPU shadow maps | Shadow maps are wrong at 1° sun elevation. The visuals must equal our numbers. Threlte's ecosystem is thin. |
| Charts | **uPlot**; panorama on a hand-drawn 2D canvas | Chart.js / Plotly / ECharts | 260k points per series stays smooth at about 50 KB. |
| Data format | **raw little-endian binary** (int16 tiles with a per-tile offset at 0.25 m or finer, horizon grids, f64 vectors) + one JSON manifest; compression unverified for `.bin`, budget 27 MB uncompressed first load | Parquet / COG / PMTiles | Zero-copy typed arrays and WASM, no parser libraries. The GIS formats only help generic GIS clients, which we don't need. |
| Backend | **none at runtime** | Hono / Express / DB | Fully static: no keys to leak, nothing to go down after judging. Add a Hono Worker only if a live LLM feature is ever added. |
| Offline scripts | **Bun + TypeScript** (downloads, Horizons fixture fetches, scoreboard merge; validation compute is in `pipeline`) | Python | One non-Rust language. Bun runs TS directly. |
| Hosting | **Cloudflare Pages** (project `penumbra-lunar`, set by the repo variable `CF_PAGES_PROJECT`), public GitHub repo `penumbra`, **Apache-2.0** | Vercel / Netlify | Free, global, static. Apache-2.0 is the Space Apps default, and the Participant Agreement needs licensed and cited sources. |
| Tests | **`cargo test`** (V1 Horizons fixtures, V5 red-team) via `/tdd`; validation pipeline → `scoreboard.json` (V2–V4); **1 Playwright** smoke test on the deployed URL; Vitest only if TS logic grows; data-heavy checks run as `pipeline --release`; native and WASM are compared with a tolerance | Jest-heavy suite | The science lives in Rust. The scoreboard serves as both test and pitch. |
| Type check | `svelte-check` (stock) | tsc-rs (pingdotgg/ts-rust) | tsc-rs is an experimental type-checker port that only matters on huge codebases. Svelte uses its own checker. |
| Evals | **none** (V1–V5 are the evals) | LLM eval harness | No runtime LLM. |

## Data sources
- LOLA `LDEM_80S_20MPP_ADJ.TIF` (20 m, ±304 km, 2.6 GB) — pgda.gsfc.nasa.gov/data/LOLA_20mpp/
- LOLA `LDEM_60S_240MPP_ADJ` (240 m far field, cropped to site + 200 km)
- PGDA `AVGVISIB_85S_060M_201608.TIF` + `_EARTH` (V2) — pgda.gsfc.nasa.gov/data/MoonIllumination/
- PGDA map-projected NAC `M*E_20MPP.TIF` + `M*_20MPP_MODEL.TIF` (V3)
- NAIF kernels (above); JPL Horizons API (V1)
- Facts + URLs: `docs/research/lunar-landing/facts-grill-1.md`
