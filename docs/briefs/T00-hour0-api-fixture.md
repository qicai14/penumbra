# T00 — Hour 0: API freeze and fixture (science half)

Ticket: [PLAN-DRAFT T00](../PLAN-DRAFT.md). Owners: both. 0.5 h each. Blocked by nothing. This brief covers what the science side must hand Charls by 09:30: the frozen API document, the error type, `api_version`, and a fixture with the final shapes. The repo export, secrets, downloads and issue filing are in PLAN-DRAFT §5 and are not repeated here.

## 1. Goal and done-criteria (from the ticket; the science-side lines)

After 30 minutes both people can work without waiting for each other. Charls has a fixture with the final shapes; Qi Cai has every download running.

- [ ] `docs/API.md` holds the §1 API table, approved by both in the PR. After this PR the API changes only with both names on the change.
- [ ] The fixture has a fake horizon (720 + 720 values), a fake 30-day timeline at 10 minutes (4320 rows × 10), a fake survival result and a sample scoreboard, and says "fake" inside the file.
- [ ] `api_version` is present in the engine, the fixture and the manifest, and the adapter's `validate()` refuses a mismatch. The fixture is at `web/public/fixture/scenario.json` with the keys of ARCHITECTURE §3.5.
- [ ] Downloads running, small files first (kernels, PSR, far field, AVGVISIB pair, then the 2.6 GB DEM; not `_ERR`). The DEM's ETA is written in the issue.

## 2. Files and signatures

| File | What | Who |
|---|---|---|
| `docs/API.md` | ARCHITECTURE §3.1 (conventions) and §3.2 (signatures, array layouts, ownership) copied verbatim, plus the five [D] decisions from [README](README.md) as an "Agreed at freeze" list | Qi Cai |
| `crates/core/src/error.rs` | `Error` and `code()` | Qi Cai |
| `crates/core/src/lib.rs` | `pub const API_VERSION: u32 = 1;` and `pub mod error;`. Remove the placeholder `add`. | Qi Cai |
| `crates/wasm/src/lib.rs` | `api_version()`; an empty `Engine` with `new()`; the `JsError` conversion | Qi Cai |
| `scripts/make-fixture.ts` | writes the fixture (§3) | Qi Cai |
| `web/public/fixture/scenario.json` | the fixture | generated |
| `web/public/data/manifest.json` | `{ "format_version": 1, "api_version": 1, "ephemeris": [], "far": null, "regions": [] }` until T02 and T06 fill it | Qi Cai |
| `web/src/lib/engine.ts` | adapter, types, switches, `validate()` | Charls (ARCHITECTURE §3.5) |

```rust
// core/src/error.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { NotLoaded, OutOfCoverage, NoData, OutOfTable, BadInput }
impl Error {
    pub fn code(self) -> &'static str;   // "NOT_LOADED", "OUT_OF_COVERAGE", "NO_DATA", "OUT_OF_TABLE", "BAD_INPUT"
}

// wasm/src/lib.rs
#[wasm_bindgen] pub fn api_version() -> u32;                 // core::API_VERSION
fn js_err(e: penumbra_core::error::Error, detail: &str) -> JsError;   // JsError::new("<CODE>: <detail>")
#[wasm_bindgen] pub struct Engine { /* filled by T02, T06 */ }
```

Every other export is added by the ticket that implements it, with the frozen signature. Until then the adapter's switch for that function stays on "fixture".

## 3. The fixture: exact recipe

Fake by construction: sine waves, no lunar model. The numbers are chosen so that every axis range survives the flip to real data (ARCHITECTURE §3.5) and so that Charls sees a sunset mid-step, an azimuth wrap, and a lethal time inside the span.

Scalars: `fake: true`, `note: "FAKE DATA for layout only. Not a model result."`, `api_version: 1`, `site: {lat_deg: −84.7906, lon_deg: 29.1957, ground_m: 0, mast_m: 2}`, `start_unix_s: 1794614400`, `step_s: 600`, `count: 4320`, `preset` as in ARCHITECTURE §3.5.

With i = 0 … 719, k = 0 … 4319, angles of sin and cos in radians:

1. `horizon.el_deg[i] = 1.5 + 2.0·sin(2π·i/720) + 1.0·sin(2π·5·i/720 + 1.0)` (between −1.5 and 4.5).
2. `horizon.dist_m[i] = 5000 + 4000·sin(2π·3·i/720)` (between 1 000 and 9 000).
3. `t_unix_s[k] = 1794614400 + 600·k`.
4. `sun_az_deg[k] = (200 − (0.508/6)·k) mod 360`, kept in [0, 360). It falls 0.508°/h, as the real Sun does in the south, and wraps once.
5. `sun_el_deg[k] = 1.0 + 3.0·sin(2π·k/4252)` (a 29.5-day wave between −2 and 4).
6. `sun_radius_deg[k] = 0.2666`, `sun_dist_km[k] = 148 000 000`.
7. `earth_az_deg[k] = (360 + 8·sin(2π·k/3924)) mod 360` (it swings across the 0/360 wrap).
8. `earth_el_deg[k] = 2.0 + 5.5·cos(2π·k/3924)` (between −3.5 and 7.5).
9. `earth_radius_deg[k] = 0.95`, `earth_dist_km[k] = 384 400`.
10. `hor(az)` = linear interpolation of `horizon.el_deg` at 0.5° with wrap.
11. `illumination[k] = clamp(0.5 + (sun_el_deg[k] − hor(sun_az_deg[k]))/(2·0.2666), 0, 1)`; `visibility[k] = illumination[k] > 0 ? 1 : 0`.
12. `earth_margin_deg[k] = earth_el_deg[k] − hor(earth_az_deg[k])`; `contact[k] = earth_margin_deg[k] ≥ 2 ? 1 : 0`.
13. `track` repeats columns 3–9 under its own keys; `timeline` holds its ten keys (with `sun_dist_km`).
14. `survival`: `operable_h` = (number of k with illumination > 0 and contact = 1)/6; `sunlit_h` = Σ illumination/6; `contact_h` = Σ contact/6; `longest_dark_h` and `longest_any_sun_h` from runs of illumination = 0 and visibility = 1, in hours; `lethal_h = 301.2`; `soc_wh[k] = max(0, 10000·(1 − (k/6)/301.2))`. The battery line is a straight fake; it is not the T09 recurrence.
15. `scoreboard`: the one V1 entry of ARCHITECTURE §3.5 with `value: null`, `pass: null`, `notes: "not run"`.

Write numbers with at most 6 decimals to keep the file near 400 kB.

## 4. Edge cases and pitfalls

- **Freeze means freeze.** Names, order and units in `docs/API.md` come from ARCHITECTURE §3.2 character for character. The type keys in `engine.ts`, the fixture keys and the column names are the same strings. A rename after 09:30 needs both names on the PR.
- **The five [D] decisions** in the README change no signature, but D1 and D2 change numbers Charls will see. Get a yes or no on each during the freeze.
- **The fixture is not a model.** The script must not import anything from `crates/` or compute a lunar quantity. If someone asks why the fake Sun sets where it does: it doesn't mean anything.
- **`fetch`, not `import`**, for the fixture, so it stays out of the bundle.
- **`NaN` in JSON.** `lethal_h` is a number in the fixture. A real "no lethal shadow" is NaN from WASM and `null` in the `Survival` type; JSON can't hold NaN, so the fixture can't show that case. Charls should test the `null` branch by hand once.
- **Pin margin for T11**: 7 040 m from the near-tile edge, not 7 000 m ([T06 §4](T06-dem-tiles.md)).
- `.gitignore` anchors `/data/` only; `web/public/data/` and `web/public/fixture/` are tracked. Check with `git status` after generating.

## 5. Test table

These are the checks `validate()` and a small Bun test run on the generated file. Row values follow from §3.

| # | Check on `scenario.json` | Expected | Tol. | Source |
|---|---|---|---|---|
| 1 | `fake`, `api_version`, `count`, `step_s` | `true`, 1, 4320, 600 | exact | ticket; ARCHITECTURE §3.5 |
| 2 | `horizon.el_deg.length`, `horizon.dist_m.length`; every array under `track`, `timeline`; `survival.soc_wh.length` | 720, 720; 4320 each; 4320 | exact | ARCHITECTURE §3.2 |
| 3 | `timeline.t_unix_s[0]`, `[1]`, `[4319]` | 1 794 614 400; 1 794 615 000; 1 797 205 800 | exact | [S] C1 |
| 4 | `horizon.el_deg[0]`, `[180]` | 2.341471 (1.5 + sin 1.0); 3.5 + sin(1.0 + 2.5π) = 4.040302 | 1e-5 | [S] pass 2; sin 1 = 0.841471, cos 1 = 0.540302 |
| 5 | `sun_az_deg[0]`, `[2362]`, `[2363]` | 200; 0.01733; 359.93267 | 1e-4 | [S] pass 2; 0.084667 × 2362 = 199.98267 |
| 6 | Ranges: `horizon.el_deg`; `sun_el_deg`; `earth_el_deg`; every azimuth | within [−2, 15]; [−7, 7]; [−8, 8]; [0, 360) | — | ARCHITECTURE §3.5 |
| 7 | `illumination` | all in [0, 1]; at least one value strictly between; `visibility` = 1 exactly where illumination > 0 | — | ARCHITECTURE §3.5; P1 |
| 8 | `earth_az_deg` | has values below 10 and values above 350 | — | step 7; exercises the wrap in the UI |
| 9 | `survival.lethal_h`; `soc_wh[0]`, `[1807]`, `[1808]` | 301.2; 10 000; 1.1067; 0 | 1e-3 | [S] pass 2; 1807/6 = 301.1667 h |
| 10 | Time is strictly increasing; `contact` and `visibility` are only 0 or 1 | true | — | `validate()` |
| 11 | `validate()` on the fixture with `api_version` set to 2 | refuses, naming the mismatch | — | ticket |
| 12 | `api_version()` from WASM on the deployed page | 1 | exact | ticket |
| 13 | `Error::OutOfTable.code()` and the JS message for it | `"OUT_OF_TABLE"`; starts with `"OUT_OF_TABLE: "` | exact | ARCHITECTURE §3.1 |

## 6. Order (no /tdd cycle; this is a 30-minute setup ticket)

1. 09:03 copy the pre-downloaded data into `data/` and check `MANIFEST.tsv` (PLAN-DRAFT §5).
2. 09:05 draft `docs/API.md` from ARCHITECTURE §3.1–§3.2 plus the [D] list; 09:10 both walk it line by line (the freeze, PLAN-DRAFT §5). Changes now or never.
3. 09:12 `error.rs`, `API_VERSION`, `api_version()`, empty `Engine`. Row 13 as a `cargo test`; row 12 after deploy.
4. 09:15 `scripts/make-fixture.ts`, generate, rows 1–10 as a Bun test.
5. 09:25 the empty manifest; PR with both approvals; deploy; Charls starts T03 on the fixture.

**Stop and escalate if** either person wants a different shape than ARCHITECTURE §3.2 after 09:30 (it costs both people; decide together, in the PR, once), or the fixture and `engine.ts` disagree on a key name (fix the one that differs from ARCHITECTURE §3.2).
