---
status: accepted
---

# The ephemeris ships as an hourly table on a UTC grid

Sun and Earth positions are precomputed natively (ANISE and hifitime) into a table of apparent, Moon-centred MOON_ME vectors, one row per whole UTC hour from 2026-01-01T00:00:00Z, and the browser interpolates it linearly. Every time that crosses a module boundary is Unix seconds, UTC. We put the table on a UTC grid, not the TDB grid an ephemeris naturally uses, so that the browser never converts between time scales.

## Considered options

- **TDB grid, convert in the browser.** The natural output of the library, but UTC − TDB is 69.184 s, which is 0.0098° of Sun azimuth, five times the V1 target. A hand-coded offset in JavaScript is exactly the bug the science review (F3) found: native tests pass and only the shipped app is wrong.
- **Run the ephemeris library in the browser.** Not possible; see ADR 0001.
- **A finer table (10 minutes) with no interpolation.** 12.6 MB instead of 2.1 MB, for no gain: linear interpolation of hourly rows is good to about 0.0001°.

## Consequences

- V1 must be run through the shipped path (table plus interpolation), not through ANISE directly, or it validates the wrong thing.
- Unix time has no leap seconds and neither does a table indexed by whole UTC hours, so a leap second inside the span can't shift the index. It would cost at most 1 s (0.00014°) inside one hour.
- A time outside every loaded segment is an error, never an extrapolation. Dates outside 2026–2030 (the IM-2 replay) need their own segment.
