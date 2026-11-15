# Penumbra

Penumbra tells lunar landing planners how many hours of sunlight and Earth contact a lander gets at a south-pole site and date, and when the first lethal shadow arrives. One Rust model computes it natively for validation and as WebAssembly in the browser, and is checked against JPL Horizons and NASA LOLA illumination maps.

**Live app:** https://penumbra-lunar.pages.dev · NASA Space Apps Challenge 2026 · Team Fractured Toes

<!-- TODO(human): add a screenshot of the Site view, e.g. docs/screenshot.png -->

## What it does

- **Site:** pick a point and landing date in the Mons Mouton or Shackleton-ridge region. You get the horizon panorama with Sun and Earth tracks, a countdown to the next sunset and Earthset, illumination and Earth-contact timelines, and the survival headline (operable days, time to the first lethal shadow). A region heatmap shows operable days on a 100 m grid, and a readout names the nearest better site within 1 km.
- **Compare A vs B:** two site and date pairs side by side.
- **IM-2 replay:** Athena's site and date, with a local-pit model.
- **About:** the validation scoreboard (V1 to V5, misses included), the limitations and the datasets.

## Run it

Rust stable with the `wasm32-unknown-unknown` target, the `wasm-bindgen` CLI and Bun. `scripts/bootstrap.sh` checks and installs what it can.

```bash
bash scripts/bootstrap.sh
bun install
bun run dev        # builds the WASM engine, then starts the Vite dev server
bun run test       # Rust tests (cargo test --workspace)
bun run --cwd web test   # web unit tests, including the grid-vs-engine equality check
bun run paper      # project document, docs/paper/penumbra.pdf (needs typst)
```

The terrain and ephemeris files the app needs are already in `web/public/data`. Rebuilding them needs the raw NASA data and the native pipeline (`crates/pipeline`); see `docs/STACK.md`.

## Layout

- `crates/core`: the model (horizon, ephemeris, visibility, survival). Pure maths, no I/O. Its tests are in `crates/core/tests` (for example `horizon.rs` and `survival.rs`).
- `crates/pipeline`: native, offline tools that cut the data and run the validation.
- `crates/wasm`: browser bindings to the same `core`.
- `web/`: Svelte 5 app (Three.js, uPlot).
- `docs/`: science spec, evidence and the project document.

## Data sources

- LOLA south-pole DEMs (LDEM_80S_20MPP_ADJ, LDEM_60S_240MPP_ADJ), NASA PGDA.
- Lunar south-pole illumination maps (AVGVISIB_85S_060M_201608 and its Earth twin), NASA PGDA, used as the V2 judge.
- NAIF generic kernels: de440.bsp, moon_pa_de440_200625.bpc, moon_de440_250416.tf, naif0012.tls.
- JPL Horizons, used as the V1 judge.

## References

- Mazarico et al. 2011, Illumination conditions of the lunar polar regions using LOLA topography, *Icarus*.
- Barker et al. 2021, Improved LOLA elevation maps for south pole landing sites: error estimates and their impact on illumination conditions, *Planetary and Space Science*.
- Barker et al. 2023, A New View of the Lunar South Pole from the Lunar Orbiter Laser Altimeter (LOLA), *The Planetary Science Journal*.
- Park et al. 2021, The JPL Planetary and Lunar Ephemerides DE440 and DE441, *The Astronomical Journal*.
- De Rosa 2012, Characterisation of potential landing sites for the European Space Agency's Lunar Lander project, *Planetary and Space Science*.
- Gläser et al. 2018, Illumination conditions at the lunar poles: implications for future exploration, *Planetary and Space Science*.

Full entries: `docs/paper/refs.yml`.

## License

Apache-2.0, see `LICENSE`.
