---
status: accepted
---

# The WASM boundary is a stateful engine that copies results out

The browser talks to the model through one `Engine` object that owns the loaded data files, plus pure functions for everything that needs no data. Per-call arguments are scalars or small arrays. Every result is a new flat, column-major `Float64Array` owned by JavaScript, in degrees, metres and Unix seconds UTC. `web/` never holds a view into WASM memory. We chose copies over zero-copy views because the largest per-call result is 346 kB, and a view into WASM memory is silently detached whenever that memory grows.

## Considered options

- **Stateless functions that take the tiles on every call** (the first draft of the Hour 0 API). Simple to describe, but wasm-bindgen copies slice arguments, so each horizon would copy 25 MB.
- **Zero-copy views of results** (`Float64Array::view`). Saves a copy worth about 0.2 % of the horizon budget, breaks when a tile load grows memory, and can't be transferred out of a worker.
- **Rich JS objects through serde.** Friendlier types, but a new dependency and a serialisation layer for what is a handful of numeric arrays. The TypeScript adapter gives the same types with `subarray`.
- **Row-major results.** Matches a table on paper, but uPlot takes one array per series, so every chart would need a de-interleaving loop.

## Consequences

- The only state is the loaded files. Every result is a pure function of them and the arguments; the horizon is passed into `timeline` explicitly, which is also what lets the pit model substitute a raised horizon.
- `web/src/lib/engine.ts` is the only module that imports the generated WASM or the fixture. Its functions are async from the start, so a worker can be put behind it later without changing a caller.
- Column order and units are part of the frozen API. The fixture uses the same key names as the adapter's types, and an `api_version` number is checked across engine, fixture and manifest.
- Errors are thrown with a leading code (`OUT_OF_COVERAGE`, `NO_DATA`, `OUT_OF_TABLE`, `NOT_LOADED`, `BAD_INPUT`); no function returns a default value in place of one.
