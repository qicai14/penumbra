// Fetches the T02 tracer pair from JPL Horizons (P15: the request URL is stored with each fixture).
// Site IM-2: 29.1957°E, 84.7906°S, altitude 0.002 km (2 m mast) on the MEAN_ME sphere; 2026-11-14T00:00 UTC.
// Run: bun scripts/fetch-horizons.ts  → crates/core/tests/fixtures/horizons/tracer-{sun,earth}.json
import { mkdirSync, writeFileSync } from "fs";

const OUT = "crates/core/tests/fixtures/horizons";
const params = (command: string) => ({
  format: "json", COMMAND: `'${command}'`, OBJ_DATA: "'NO'", MAKE_EPHEM: "'YES'", EPHEM_TYPE: "'OBSERVER'",
  CENTER: "'coord@301'", COORD_TYPE: "'GEODETIC'", SITE_COORD: "'29.1957,-84.7906,0.002'",
  TLIST: "'2026-11-14 00:00:00'", TIME_TYPE: "'UT'", QUANTITIES: "'4'", ANG_FORMAT: "'DEG'",
  EXTRA_PREC: "'YES'", APPARENT: "'AIRLESS'", CSV_FORMAT: "'YES'",
});

mkdirSync(OUT, { recursive: true });
for (const [name, command] of [["sun", "10"], ["earth", "399"]]) {
  const request = `https://ssd.jpl.nasa.gov/api/horizons.api?${new URLSearchParams(params(command))}`;
  const res = await fetch(request);
  if (!res.ok) throw new Error(`${name}: HTTP ${res.status}`);
  const response = await res.json();
  if (!String(response.result).includes("$$SOE")) throw new Error(`${name}: no $$SOE in response`);
  writeFileSync(`${OUT}/tracer-${name}.json`, JSON.stringify({ request, response }, null, 2) + "\n");
  console.log(`${name}: wrote ${OUT}/tracer-${name}.json`);
}
