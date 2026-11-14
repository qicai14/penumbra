// Fetches the 20 V1 site/time pairs from JPL Horizons (SCIENCE-SPEC §1c request, §3 "V1 details").
// Libration-extreme times are picked from the shipped table web/public/data/eph-main.<hash6>.bin
// (rows: Sun xyz, Earth xyz, km, MOON_ME, hourly UTC from 2026-01-01T00:00:00Z).
// Each pair stores the request URL with the raw response (P15), so tests run offline.
// Run: bun scripts/fetch-horizons-v1.ts  → crates/core/tests/fixtures/horizons/v1-pairs.json
import { readdirSync, readFileSync, writeFileSync } from "fs";

const OUT = "crates/core/tests/fixtures/horizons/v1-pairs.json";
const DATA = "web/public/data";
const T0_UNIX_S = 1_767_225_600; // 2026-01-01T00:00:00Z, core MAIN_T0_UNIX_S
const STEP_S = 3600;

// Sub-Earth latitude and longitude (deg, MOON_ME) of every row; the libration extremes are their max and min.
const bin = readdirSync(DATA).filter((n) => n.startsWith("eph-main.") && n.endsWith(".bin"));
if (bin.length !== 1) throw new Error(`expected one eph-main.*.bin in ${DATA}, found ${bin.length}`);
const buf = readFileSync(`${DATA}/${bin[0]}`);
const rows = new Float64Array(buf.buffer, buf.byteOffset, buf.byteLength / 8);
const ext = { latMax: [-Infinity, 0], latMin: [Infinity, 0], lonMax: [-Infinity, 0], lonMin: [Infinity, 0] };
for (let k = 0; k * 6 < rows.length; k++) {
  const [x, y, z] = [rows[6 * k + 3], rows[6 * k + 4], rows[6 * k + 5]];
  const lat = (Math.asin(z / Math.hypot(x, y, z)) * 180) / Math.PI;
  const lon = (Math.atan2(y, x) * 180) / Math.PI;
  if (lat > ext.latMax[0]) ext.latMax = [lat, k];
  if (lat < ext.latMin[0]) ext.latMin = [lat, k];
  if (lon > ext.lonMax[0]) ext.lonMax = [lon, k];
  if (lon < ext.lonMin[0]) ext.lonMin = [lon, k];
}
const rowTime = (k: number) => iso(T0_UNIX_S + k * STEP_S);
function iso(unixS: number) {
  return new Date(unixS * 1000).toISOString().replace("T", " ").slice(0, 19); // UTC
}

// Sites: lat (deg, south negative), east lon (deg, [0, 360)), height above the 1737.4 km sphere (km).
const POLE = [-90, 0], NEAR_POLE = [-89.9, 0];
const SP01 = [-89.45, 222.69]; // Mazarico 2011 (E16b, E16c)
const IM1 = [-80.1276, 1.4367]; // E72a
const IM2 = [-84.7906, 29.1957]; // ARCHITECTURE F20 / tracer
const FAR60 = [-60, 180], FAR85 = [-85, 180]; // T-1d-1, T-1d-1b far-side sites
// De Rosa 2012 Table 2, "Centre of Region of Interest" (114cd0dd:1288–1331), west lon + 360
const CR1 = [-89.4679, 360 - 137.411], GR1 = [-88.6952, 360 - 68.2846], LP1 = [-85.4596, 31.78053];
const MP1 = [-85.9756, 360 - 2.11241], SR1 = [-89.7755, 360 - 156.448];

type Pair = { id: string; why: string; site: number[]; h_km: number; t_utc: string };
const p = (id: string, why: string, site: number[], h_km: number, t_utc: string): Pair => ({ id, why, site, h_km, t_utc });
const PAIRS: Pair[] = [
  p("01", "pole, sub-Earth lat max", POLE, 0, rowTime(ext.latMax[1])),
  p("02", "0.1 deg from pole, sub-Earth lat min", NEAR_POLE, 0, rowTime(ext.latMin[1])),
  p("03", "SP-01, sub-Earth lon max", SP01, 0, rowTime(ext.lonMax[1])),
  p("04", "IM-1, sub-Earth lon min", IM1, 0, rowTime(ext.lonMin[1])),
  p("05", "IM-2 2 m, before the year boundary", IM2, 0.002, "2026-12-31 23:59:00"),
  p("06", "IM-2 2 m, after the year boundary", IM2, 0.002, "2027-01-01 00:01:00"),
  p("07", "far side 60S", FAR60, 0, "2026-03-15 06:30:00"),
  p("08", "far side 85S", FAR85, 0, "2026-07-04 12:00:00"),
  p("09", "CR1 region centre", CR1, 0, "2027-05-20 18:45:00"),
  p("10", "GR1 region centre", GR1, 0, "2027-09-09 03:15:00"),
  p("11", "LP1 region centre, leap day", LP1, 0, "2028-02-29 12:00:00"),
  p("12", "MP1 region centre", MP1, 0, "2028-08-10 21:30:00"),
  p("13", "SR1 region centre", SR1, 0, "2029-01-25 09:10:00"),
  p("14", "IM-2 at +5 km (P15)", IM2, 5, "2029-06-30 15:20:00"),
  p("15", "SP-01 at -3 km (P15)", SP01, -3, "2029-11-11 11:11:00"),
  p("16", "pole", POLE, 0, "2030-04-02 04:40:00"),
  p("17", "MP1 region centre", MP1, 0, "2030-10-17 16:05:00"),
  p("18", "IM-1, near the table end", IM1, 0, "2030-12-31 22:30:00"),
  p("19", "GR1 region centre, near the table start", GR1, 0, "2026-01-01 00:30:00"),
  p("20", "LP1 region centre, before a year boundary", LP1, 0, "2028-12-31 23:30:00"),
];

const params = (command: string, pair: Pair) => ({
  format: "json", COMMAND: `'${command}'`, OBJ_DATA: "'NO'", MAKE_EPHEM: "'YES'", EPHEM_TYPE: "'OBSERVER'",
  CENTER: "'coord@301'", COORD_TYPE: "'GEODETIC'",
  SITE_COORD: `'${pair.site[1]},${pair.site[0]},${pair.h_km}'`, // E-lon deg, lat deg, h km (E48a, E48b)
  TLIST: `'${pair.t_utc}'`, TIME_TYPE: "'UT'", QUANTITIES: "'4'", ANG_FORMAT: "'DEG'",
  EXTRA_PREC: "'YES'", APPARENT: "'AIRLESS'", CSV_FORMAT: "'YES'",
});

async function fetchBody(command: string, pair: Pair) {
  const request = `https://ssd.jpl.nasa.gov/api/horizons.api?${new URLSearchParams(params(command, pair))}`;
  const res = await fetch(request);
  if (!res.ok) throw new Error(`${pair.id}/${command}: HTTP ${res.status}`);
  const response = await res.json();
  if (!String(response.result).includes("$$SOE")) throw new Error(`${pair.id}/${command}: no $$SOE\n${response.result}`);
  return { request, response };
}

const out = [];
for (const pair of PAIRS) {
  const t_unix_s = Date.parse(pair.t_utc.replace(" ", "T") + "Z") / 1000;
  out.push({
    id: pair.id, why: pair.why, lat_deg: pair.site[0], lon_east_deg: pair.site[1], h_km: pair.h_km,
    t_utc: pair.t_utc, t_unix_s,
    sun: await fetchBody("10", pair),
    earth: await fetchBody("399", pair),
  });
  console.log(`${pair.id} ${pair.t_utc} ${pair.why}`);
}
writeFileSync(OUT, JSON.stringify(out, null, 1) + "\n");
console.log(`libration extremes (deg): ${JSON.stringify(ext)}; wrote ${OUT}`);
