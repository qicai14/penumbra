// Curated landing sites for the picker (#82). Every coordinate cites a SCIENCE-SPEC §6 ledger row; none is guessed.
import { checkPick } from "./site";
import { latlonToXy } from "./projection";
import type { NearMeta } from "./engine";

export type CuratedSite = { id: string; name: string; lat: number; lon: number; src: string };

export const SITES: readonly CuratedSite[] = [
  { id: "im1", name: "IM-1 Odysseus", lat: -80.1276, lon: 1.4367, src: "E72a" },
  { id: "im2", name: "IM-2 Athena", lat: -84.7906, lon: 29.1957, src: "W9" },
  { id: "sp01", name: "SP-01 (Mazarico 2011)", lat: -89.45, lon: 222.69, src: "E16b, E16c, E85" },
  // Derived [D]: Barker RoI 1 centroid (E129, E130) through the §1 inverse; sites.test.ts recomputes it.
  { id: "barker1", name: "Barker 2021 Site 1", lat: -89.4389, lon: 222.904, src: "E129, E130, D" },
  { id: "a11", name: "Apollo 11", lat: 0.6875, lon: 23.4333, src: "W10" },
  { id: "a12", name: "Apollo 12", lat: -3.1975, lon: -23.3856, src: "W11" },
  { id: "a14", name: "Apollo 14", lat: -3.6733, lon: -17.4653, src: "W12" },
  { id: "a15", name: "Apollo 15", lat: 26.1008, lon: 3.6527, src: "W13" },
  { id: "a16", name: "Apollo 16", lat: -8.9913, lon: 15.5144, src: "W14" },
  { id: "a17", name: "Apollo 17", lat: 20.1653, lon: 30.7658, src: "W15" },
];

/** The region whose near tile covers the site with the picker's 7 km border (checkPick), or null: OUT_OF_COVERAGE. */
export function coverageRegion(lat: number, lon: number, regions: readonly { id: string; near: NearMeta }[]): string | null {
  const [x, y] = latlonToXy(lat, lon);
  return regions.find((r) => checkPick(x, y, r.near).ok)?.id ?? null;
}

// shortcut: the links go to the tools' base pages, not to the site. QuickMap drops prjExtent/extent on load and Moon Trek
// hash parameters are unverified; upgrade to real deep links once a parameter is confirmed in the tool itself.
export const QUICKMAP_URL = "https://quickmap.lroc.im-ldi.com";
export const MOON_TREK_URL = "https://trek.nasa.gov/moon/";
export const latLonText = (lat: number, lon: number) => `${lat.toFixed(4)}, ${lon.toFixed(4)}`;
