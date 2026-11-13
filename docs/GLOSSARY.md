# Penumbra

Penumbra answers one planner question for the lunar south pole: at this site and date, does this lander get enough sunlight and Earth contact to survive, and when does the first shadow arrive that would kill it?

## Where and what

**Site**:
One point on the lunar surface (latitude, longitude) inside DEM coverage, where a lander would sit.
_Avoid_: location, pixel, spot

**Region**:
One of the nine Artemis III candidate areas, or the IM-1 / IM-2 landing area, that a site belongs to.
_Avoid_: zone, area

**Lander preset**:
A named set of lander parameters: panel height, panel orientation, panel area and efficiency, battery capacity, load, and minimum direct-to-Earth hours. Each value is marked cited or assumed.
_Avoid_: config, lander profile, vehicle settings

**Mast height**:
The height above the local ground of the point whose view is computed (panel or antenna), written H.
_Avoid_: altitude, elevation (elevation is an angle here)

**Scenario**:
One site, one landing date and one lander preset together: the full input to a single survival run.
_Avoid_: case, run, query

## What the terrain hides

**Horizon**:
The elevation angle of the terrain skyline seen from a site at a given mast height, for every azimuth around 360°, with the distance to the obstacle that sets it.
_Avoid_: skyline, horizon mask (when the margin is meant, say terrain mask)

**Terrain mask**:
The horizon used as a visibility cut-off for Earth contact, raised by the elevation margin.
_Avoid_: horizon (when the margin is included)

## What the lander gets

**Illumination**:
The fraction of the Sun's disc above the horizon at a moment, from 0 to 1; it drives power.
_Avoid_: sunlight percentage, visibility (visibility is any part of the disc)

**Visibility**:
Whether any part of a body's disc is above the horizon; the metric the published NASA maps use.
_Avoid_: illumination

**Operable window**:
A continuous period when the site has both illumination above zero and Earth above the terrain mask.
_Avoid_: comms window, sunlit window, uptime

**Lethal shadow**:
The first moment in a scenario when stored battery energy falls to the preset's minimum; the survival run stops there. Time to first lethal shadow is that moment minus landing time.
_Avoid_: death, blackout, eclipse
