# NotebookLM notebooks (Phase 2)

CLI: `notebooklm ask "<q>" -n <id>` · each notebook also holds `sources.md` as "Curated source index".

| Track | Notebook ID | Sources | Folder |
|-------|-------------|---------|--------|
| Lunar Landing Site Navigator | `5d807093-69f3-410b-bd6a-898489882eeb` | 62 (Nov 14: + CSA Moon Exploration from the Resources tab; QuickMap, Moon Trek and Horizons were already in) | `lunar-landing/` |
| MODIS/VIIRS Hot Spots | `4e0c1818-18a4-4ad2-84bd-39b731fb4554` | 55/59 | `fire-hotspots/` |
| Earth Analogs (Moon/Mars) | `e35b0dac-a864-401a-addc-22f8e157d276` | 60/60 | `earth-analogs/` |
| Earth System Trend Detective | `928a3f30-a9a1-4df2-bc05-3cf9125e5b17` | 54/60 | `trend-detective/` |

## Gaps (upload PDFs by hand if needed)
- Hot Spots: RSE 2025 global FRP evaluation (10.1016/j.rse.2025.115136) · GCB 2026 Canada wildfire extremes (10.1111/gcb.70891) · GRL 2024 Canada aridity (10.1029/2024GL114492) · Xu & Wooster 2023 SLSTR FRP (10.1016/j.srs.2023.100087). 4 sources in this notebook show status "error" (failed imports) — safe to delete in the UI.
- Trend Detective: Cleveland 1990 STL paper, Nature Sust. 2019 greening (s41893-019-0220-7), Nature 2010 (nature09396), Nat. Comms 2018 (s41467-018-03732-9), Climate Reanalyzer homepage, VU repository PDF — see `trend-detective/notebook.log`.

## Prior art to check (rival 2026 entries found on GitHub)
- Lunar: CLPS-MOON-BROWSER, lunarwindow · Analogs: terrestrial-analog-finder · Hot Spots: several hackathon repos (see `fire-hotspots/sources.md`).

## News hooks
- Suomi NPP removed from FIRMS on 2 Nov 2026 (Earthdata) — continuity problem is live.
- Artemis III site down-select 13 → 9 regions (NTRS 2025); IM-1/IM-2 lighting/slope landing failures.
