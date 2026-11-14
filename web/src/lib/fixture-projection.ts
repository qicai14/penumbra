// FIXTURE STAND-IN for the WASM latlon_to_xy / xy_to_latlon (T01 #2). Delete this file when #2 merges and
// point engine.ts at the WASM functions. Polar stereographic, sphere R = 1 737 400 m, ARCHITECTURE §2.1.
// Longitude 0 is along +y, 90 deg E along +x. Used only to place a picker click on the fixture tile.
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
  return [Math.asin(uz) / D, Math.atan2(uy, ux) / D];
}
