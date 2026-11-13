# crates/

- `core` is pure maths: no I/O, deterministic, builds for both native and wasm32. Logic lives here.
- `pipeline` is native only (ANISE, `tiff`); it never compiles to wasm32.
- `wasm` is a thin wasm-bindgen wrapper over `core`.
- Test-first with `cargo test`; test names reference the SCIENCE-SPEC test id they cover.
- Pre-commit and CI run `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings`. Workspace lints (root `Cargo.toml`) flag exact float compares and lossy int casts; an intended cast gets `#[allow(clippy::<lint>)]` with a reason.
- Tests are the oracle: add tests, never weaken an existing one without its id, reason and cited source.
- Boundary: degrees, metres, Unix seconds UTC, f64; flat column-major arrays. Radians inside `core`.
