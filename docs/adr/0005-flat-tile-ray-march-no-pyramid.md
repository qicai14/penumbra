---
status: accepted
---

# The horizon is a fixed-step march over flat tiles, with no pyramid

The horizon is computed by marching 720 great circles outward from the site, one pixel per step: the 20 m near tile to 7 km, then the 240 m far tile until an early-exit bound from the tile's highest point fires or the tile ends. The tiles are flat arrays; there is no max-mipmap or quadtree. We chose this because the arithmetic (0.7–1.4 million samples per horizon, 60 ns each inside a 50 ms budget) says the simple loop is fast enough, and a pyramid's failure mode is a wrong bound that hides a peak without failing any test.

## Considered options

- **Max-mipmap or quadtree over the far tile.** Skips empty space, most where the near horizon is high. It adds a third of the far tile in memory and a curvature-aware bound per node, and taking the maximum over a cell raises the horizon, which biases the comparison with NASA's illumination maps toward dark.
- **Straight lines in the map plane.** Cheaper per step, but a straight map line leaves the great circle by about 280 m after 150 km, twice that at 80°S.
- **Step size growing with distance.** Fewer samples, but only safe on top of a max-pyramid, so it inherits that option's cost.
- **A fixed range cut-off at 150 km.** Too short: a 7 km peak at 150 km still stands at +0.19° (science review F2).

## Consequences

- The inner loop has no trigonometry: the great-circle point comes from a step table of sinθ and 1 − cosθ shared by every azimuth, the projection is one division, and elevations are compared as tangents.
- The early exit is valid only while the tile's highest point is above the observer; a site on that highest point must give the same horizon with the exit on and off.
- If the Saturday benchmark misses the budget, the next steps are, in order: `opt-level = 3`, a coarse 16 × 16 px skip grid (79 kB, never changes a result), a worker. A full pyramid is not on that list.
- The 7 km near/far switch and the int16 scales are checked against the full 20 m float DEM (T-1a-4); a difference above 0.05° reopens the tier layout, not this decision.
