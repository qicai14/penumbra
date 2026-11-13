---
status: accepted
---

# One Rust model, run natively and as WASM

The science model lives once, in `crates/core`: pure functions over slices, no I/O, no dependencies. `crates/pipeline` runs it natively for precompute and validation, and `crates/wasm` exposes it to the browser. We chose this over a Python or TypeScript model with a separate browser port because the numbers on the validation scoreboard and the numbers the planner sees must come from the same code, and two implementations drift.

## Considered options

- **Python/NumPy offline plus a TypeScript port in the browser.** Fastest to start, but every equation exists twice and V1–V5 would validate the copy the user never runs.
- **All TypeScript.** One language, but the horizon march (about a million DEM samples per click) and the 18.6-year validation runs want native speed, and there is no SPICE reader.
- **Rust everywhere, including ephemeris in the browser.** ANISE 0.10.6 does compile for wasm32 (checked Nov 12), but the browser would have to fetch the 133 MB of kernels and load them from bytes, against a 2.1 MB table.

## Consequences

- ANISE and the `tiff` crate are dependencies of `pipeline` only. `core` must never import them, and `pipeline` must never be a dependency of `wasm`.
- Whatever the browser needs from a native-only library is precomputed into a data file. The ephemeris is the first case (ADR 0002).
- Native and WASM builds use different maths libraries for sin, atan2 and asin, so their results can differ in the last digits. Compare them with a tolerance, never bit for bit.
