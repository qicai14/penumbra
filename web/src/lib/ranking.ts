// Pure ordering of the region ranking in the site picker. No model maths: it only sorts values the grids already hold.
export type Rankable = { b: { operableH: number } | null; lethalH?: number };

/**
 * Most operable hours first; a later first lethal shadow breaks a tie (Infinity = none in span, undefined = unknown, last);
 * a region with no best cell (its grid lacks the date) goes last. Stable: full ties keep input order.
 */
export function rankBests<T extends Rankable>(items: T[]): T[] {
  return [...items].sort((p, q) => (q.b ? 1 : 0) - (p.b ? 1 : 0) || (q.b?.operableH ?? 0) - (p.b?.operableH ?? 0) || (q.lethalH ?? -1) - (p.lethalH ?? -1));
}

/** AVGVISIB covers 85°S to the pole; a tile centred north of that has no LOLA check. */
export const AVGVISIB_NORTH_LIMIT_DEG = -85;
export const outsideAvgvisib = (centreLatDeg: number) => centreLatDeg > AVGVISIB_NORTH_LIMIT_DEG;
