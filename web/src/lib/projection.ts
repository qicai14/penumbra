// Map projection for the web, kept in TS by decision on #58 (docs/API.md no longer lists latlon_to_xy /
// xy_to_latlon). Polar stereographic, sphere R = 1 737 400 m, ARCHITECTURE §2.1: the same maths as
// core::geometry, held to it within 1 mm by projection.test.ts against projection-ref.json at the
// reference points (all 1 km or more from the pole; asin(uz) loses about 3 cm within a metre of it).
// Longitude 0 is along +y, 90 deg E along +x; output longitude is in [0, 360) like core.
// Used to place a picker click on a tile and to pick the near-tile region for a site (engine.ts).
const R = 1737400;
const D = Math.PI / 180;

export function latlonToXy(lat_deg: number, lon_deg: number): [number, number] {
  const c = Math.cos(lat_deg * D), uz = Math.sin(lat_deg * D);
  const k = (2 * R) / (1 - uz);
  return [k * c * Math.sin(lon_deg * D), k * c * Math.cos(lon_deg * D)];
}

export function xyToLatlon(x_m: number, y_m: number): [number, number] {
  const t = (x_m * x_m + y_m * y_m) / (4 * R * R);
  const ux = y_m / R / (1 + t), uy = x_m / R / (1 + t), uz = -(1 - t) / (1 + t);
  const lon = Math.atan2(uy, ux) / D;
  const wrapped = lon < 0 ? lon + 360 : lon;
  return [Math.asin(uz) / D, wrapped >= 360 ? 0 : wrapped]; // a tiny negative lon + 360 rounds to 360, as in core's wrap_tau
}
