---
status: accepted
---

# Light and shadow on screen come from the model, not GPU shadow maps

Every lit or shaded thing the app draws (the panorama, the timelines, a shaded map, 3D terrain colours) is drawn from numbers the Rust model returned. The 3D view uses plain Three.js with per-vertex colours computed by the model and no light-casting shadows. We chose this because GPU shadow maps are unreliable at a 1° Sun elevation, where a 10 m hill throws a shadow hundreds of metres long, and because a picture that disagrees with the validated numbers would undermine the scoreboard next to it.

## Consequences

- `web/` contains no science: it never computes an angle or a shadow, and the standard Three.js shadow pipeline stays off.
- Shading a whole tile costs model time, not GPU time. A full march for every pixel of a 256 × 256 mask is about 76 million samples, so a shaded map needs either a near-tile-only march or a worker.
- React Three Fiber and Threlte add nothing here; the scene is one mesh with a colour attribute.
