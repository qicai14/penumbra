---
status: accepted
---

# Runtime data is raw little-endian arrays plus one manifest

Everything the browser loads is a headerless little-endian array (int16 DEM tiles, f64 ephemeris rows, u16 heatmap grids) described by a single `manifest.json`. DEM heights are int16 with a per-tile scale and offset: 0.25 m or finer for the 20 m near tiles, 0.5 m for the 240 m far tile. We chose this over cloud-optimized GeoTIFF, PMTiles or Parquet because a typed array over the fetched bytes is the whole parser, and the engine wants exactly that array.

## Considered options

- **Ship GeoTIFF / COG and decode in the browser.** Needs a TIFF decoder in WASM or JavaScript and brings projection metadata we already fix in code. It only helps generic GIS clients, which we don't serve.
- **float32 tiles.** No quantisation, but twice the size: nine regions would be 127 MB against a 100 MB budget, and the horizon march's cache footprint doubles.
- **The PDS int16 convention (0.5 m everywhere).** Too coarse for the near field: 0.5 m between adjacent 20 m pixels is a 1.4° slope error.

## Consequences

- The manifest is the single source of shapes, scales and offsets. The structs behind it are defined once in `core`; the pipeline writes them and the engine reads them.
- The encoder must fail on clipping. The far tile uses 0.5 m because 0.25 m spans only ±8 191 m.
- Data file names carry a content hash and only the manifest is fetched by a fixed name, so a stale cached tile can't be paired with a new manifest.
- The files are binary in git: they need `*.bin binary` in `.gitattributes`, and their folder must not match an ignore pattern.
- Pixel registration (pixel-is-area) and the height datum (above the 1 737 400 m sphere) are assumptions baked into the tiles; they are confirmed from the GeoTIFF tags before the first crop.
