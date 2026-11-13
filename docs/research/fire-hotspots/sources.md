# Curated sources: Harmonization of MODIS and VIIRS Hot Spots (NASA Space Apps 2026)

URLs checked 2026-11-08 with curl -sIL. '403 bot block' = publisher blocks curl but page is public; confirm on NotebookLM import. Citations from OpenAlex where known, else n/a.


## 0. Challenge

| Title | Year | URL | Citations | Takeaway | Implements? |
|---|---|---|---|---|---|
| NASA Space Apps 2026: Harmonization of MODIS and VIIRS Hot Spots (challenge page) | 2026 | https://www.spaceappschallenge.org/2026/challenges/modis-and-viirs-hot-spots/ | - | The brief itself; curl got a TLS cert error in this sandbox (user-supplied URL, not independently verified) | N |

## 1. Peer-reviewed algorithms, intercomparison, harmonization

| Title | Year | URL | Citations | Takeaway | Implements? |
|---|---|---|---|---|---|
| The Collection 6 MODIS active fire detection algorithm and fire products (Giglio et al.) | 2016 | https://www.sciencedirect.com/science/article/pii/S0034425716300827 | 1763 | C6 contextual algorithm, thresholds, FRP, confidence; CC-BY-NC-ND (403 bot block) | Y |
| The new VIIRS 375 m active fire detection data product (Schroeder et al.) | 2014 | https://zenodo.org/records/14790602 | n/a | 375 m algorithm; ~3x more daytime and 25x more night detections than VIIRS 750 m (403 bot block) | Y |
| Comparison of FRP estimates from VIIRS and MODIS (Li et al.) | 2018 | https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2017JD027823 | n/a | VIIRS FRP sum ~11% above MODIS; view-zenith dependence; basis for scaling (403 bot block) | Y |
| Intercomparison of MODIS Aqua and VIIRS I-band fires and emissions, Punjab (Remote Sensing) | 2018 | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC6108185/ | n/a | VIIRS counts ~4.8x combined MODIS 2012-16; 15-min overpass gap changes detections | Y |
| Fire detection and FRP in Northeast Asia: MODIS versus VIIRS (Remote Sensing) | 2020 | https://www.mdpi.com/2072-4292/12/18/2870 | n/a | Paired FRP r=0.85; VIIRS FRP lower per matched fire, higher in total (403 bot block) | Y |
| Fire Radiative Power Correction and Spatiotemporal Fusion Based on MYD14 and VNP14IMG (Remote Sensing) | 2026 | https://doi.org/10.3390/rs18101650 | n/a | Correction-before-fusion with VZA-dependent FRP correction: closest prior art (403 bot block) | Y |
| Comprehensive global FRP evaluation by minimizing detection bias, extended triple collocation (RSE) | 2025 | https://www.sciencedirect.com/science/article/abs/pii/S0034425725005401 | n/a | MODIS/VIIRS/SLSTR FRP biases after aligning detections; abstract page, may be paywalled (403 bot block) | Y |
| Observation operator and detection limits for MODIS and VIIRS FRP products (GMD) | 2026 | https://gmd.copernicus.org/articles/19/10123/2026/ | n/a | Aqua 13-14h vs SNPP 12-13:30 overpass implies ~20% FRP difference (0-50%); detection limits | Y |
| Systematic absence of low-confidence nighttime detections in VIIRS active fire product (arXiv) | 2025 | https://arxiv.org/pdf/2510.26816 | n/a | Undocumented filtering; a continuity trap for night counts | Y |
| Sentinel-3 SLSTR active fire detection and FRP daytime product, intercomparison to MODIS/VIIRS/Landsat | 2023 | https://www.sciencedirect.com/science/article/pii/S2666017223000123 | n/a | Per-fire FRP slope 0.97 vs MODIS; extra detections are low-FRP; Terra morning successor (403 bot block) | Y |
| Siberian forest fires: anomalies and trends from satellite data 2000-2019 (arXiv) | 2022 | https://arxiv.org/pdf/2206.09240 | n/a | Worked example of anomaly vs multi-year mean from MODIS fire counts | Y |

## 2. Fire seasonality, diurnal cycle, climatology, reviews

| Title | Year | URL | Citations | Takeaway | Implements? |
|---|---|---|---|---|---|
| Global distribution and seasonality of active fires observed with Terra and Aqua MODIS (Giglio et al.) | 2006 | https://dataserver-coids.inpe.br/queimadas/queimadas/Publicacoes-Impacto/material3os/2006_Giglio_etal_GlobalDistribuiton_JGR_DE3os.pdf | n/a | Terra vs Aqua peak-month and season-length agreement; strong tropical diurnal bias | Y |
| Satellite remote sensing of active fires: history and current status (Wooster et al.) | 2021 | https://www.fs.usda.gov/rm/pubs_journals/2021/rmrs_2021_wooster_m001.pdf | n/a | Review: counts are an imperfect proxy; LEO sampling limits diurnal cycle (403 bot block) | N |
| Decadal (2012-2023) variability of biomass fires on Indian landmass and their FRP (PMC) | 2025 | https://pmc.ncbi.nlm.nih.gov/articles/PMC12297426 | n/a | Regional monthly VIIRS climatology and FRP as a template | Y |
| Tracking recent extremes and interannual variability of global fire emissions, near-real-time GFED (ESSD) | 2026 | https://essd.copernicus.org/articles/18/7535/2026/ | n/a | GFED5NRT: VIIRS-based consistent extension; anomaly framing | Y |
| The Global Fire Atlas of individual fire size, duration, speed and direction (Andela et al.) | 2019 | https://essd.copernicus.org/articles/11/529/2019/ | n/a | Fire season = 12 months centred on peak burned-area month; competitor/context | Y |
| Global fire emissions estimates during 1997-2016 (van der Werf et al., GFED4s) | 2017 | https://essd.copernicus.org/articles/9/697/2017/ | n/a | Reference monthly fire climatology and emissions | N |
| Multi-decadal trends and variability in burned area from GFED5 (Chen et al.) | 2023 | https://essd.copernicus.org/articles/15/5227/2023/ | 306 | Global burned area -1.2%/yr 2001-2020; baseline for multi-year mean | N |
| State of Wildfires 2023-2024 (ESSD) | 2024 | https://essd.copernicus.org/articles/16/3601/2024/ | n/a | Canada 2023 burned area 6x average; anomaly framing | N |
| State of Wildfires 2024-2025 (ESSD) | 2025 | https://essd.copernicus.org/articles/17/5377/2025/ | n/a | 2.2 Pg C emissions; extreme episodes in Americas; LA fatalities | N |
| Drivers and impacts of the record-breaking 2023 wildfire season in Canada (Nature Comms, PMC) | 2024 | https://www.ncbi.nlm.nih.gov/pmc/articles/PMC11335882/ | n/a | Record-season case study motivating anomaly detection | N |
| Persistent 2023-2025 wildfire extremes in Canada (Global Change Biology) | 2026 | https://onlinelibrary.wiley.com/doi/10.1111/gcb.70891 | n/a | Active fire counts ~6 SD above 2002-2022 mean in 2023: anomaly example (403 bot block) | Y |
| Increased atmospheric aridity drives the 2023 extreme wildfire season in Canada (GRL) | 2025 | https://agupubs.onlinelibrary.wiley.com/doi/full/10.1029/2024GL114492 | n/a | Anomaly = 2023 minus 2001-2022 mean method (403 bot block) | Y |

## 3. NASA/NOAA data documentation and APIs

| Title | Year | URL | Citations | Takeaway | Implements? |
|---|---|---|---|---|---|
| MODIS Collection 6 and 6.1 Active Fire Product User's Guide v1.0 (MCD14ML) | 2021 | https://modis-fire.umd.edu/files/MODIS_C6_C6.1_Fire_User_Guide_1.0.pdf | - | Column definitions, confidence, FRP, daynight, MCD14ML format | Y |
| VIIRS Collection 2 375 m Active Fire User Guide v1.2 (VNP14IMGML, VJ114IMGML, VJ214IMGML) | 2025 | https://www.earthdata.nasa.gov/s3fs-public/2025-06/VIIRS_C2_AF-375m_User_Guide_1.2.pdf | - | Authoritative fields/processing for SNPP, NOAA-20, NOAA-21 | Y |
| VIIRS Active Fire Product User's Guide v1.4 (SNPP, VNP14) | 2018 | https://viirsland.gsfc.nasa.gov/PDF/VIIRS_activefire_User_Guide.pdf | - | Legacy C1 guide; ASCII files mimic MCD14ML | Y |
| VIIRS 375 m Active Fire Science Processing Algorithm (VFIRE375_SPA) | 2019 | https://directreadout.sci.gsfc.nasa.gov/users_guides_documents/VFIRE375_3.0.0_SPA_1.9.pdf | - | Implementation-level algorithm description | Y |
| NOAA ATBD: VIIRS I-band active fires | 2016 | https://www.star.nesdis.noaa.gov/jpss/documents/ATBD/ATBD_Iband_ActiveFires_v1.0.pdf | - | Equations and thresholds for 375 m detection | Y |
| NOAA ATBD: NDE active fire (VIIRS 750 m M-band) | 2020 | https://www.star.nesdis.noaa.gov/jpss/documents/ATBD/ATBD_NDE_AF_v2.6.pdf | - | 750 m algorithm for continuity comparisons | Y |
| FIRMS API landing page | 2025 | https://firms.modaps.eosdis.nasa.gov/api/ | - | Entry to area, data_availability, missing_data services | Y |
| FIRMS API: get a MAP_KEY | 2025 | https://firms.modaps.eosdis.nasa.gov/api/map_key/ | - | Rate limit 5000 transactions per 10 min | Y |
| FIRMS API: area endpoint | 2025 | https://firms.modaps.eosdis.nasa.gov/api/area/ | - | /api/area/csv/KEY/SOURCE/bbox/days[/date]; 1-5 days per call | Y |
| FIRMS API: data availability | 2025 | https://firms.modaps.eosdis.nasa.gov/api/data_availability/ | - | Date coverage per product, SP vs NRT | Y |
| How to use the FIRMS API in Python (Fire Data Academy) | 2025 | https://firms.modaps.eosdis.nasa.gov/content/academy/data_api/firms_api_use.html | - | Working code for chunked bbox downloads | Y |
| FIRMS Download Archived Data | 2025 | https://firms.modaps.eosdis.nasa.gov/download/ | - | Bulk per-sensor archive CSVs (MODIS 2000+, VIIRS 2012+) | Y |
| FIRMS Active Fire Data (24h/48h/7d downloads) | 2025 | https://firms.modaps.eosdis.nasa.gov/active_fire/ | - | NRT files; standard product replaces NRT after 2-3 months | N |
| Copernicus C3S Fire radiative power and active fire pixels 2020-present (SLSTR) | 2024 | https://cds.climate.copernicus.eu/datasets/satellite-fire-radiative-power?tab=overview | - | Open SLSTR gridded FRP for cross-checks | Y |

## 4. MODIS end-of-life, orbit drift, continuity (2020-2026)

| Title | Year | URL | Citations | Takeaway | Implements? |
|---|---|---|---|---|---|
| Transitioning from Suomi NPP VIIRS and OMPS: what FIRMS users need to know (Earthdata) | 2026 | https://www.earthdata.nasa.gov/news/blog/transitioning-from-suomi-npp-viirs-omps-what-firms-users-need-know | - | SNPP FIRMS production stops 2 Nov 2026; Terra is the only morning overpass | N |
| VIIRS instruments become more essential as Terra and Aqua drift (Earthdata) | 2023 | https://www.earthdata.nasa.gov/news/feature-articles/viirs-instruments-become-more-essential-terra-aqua-drift-traditional-orbits | - | Drift consequences and VIIRS role | N |
| Transition from MODIS to VIIRS (Earthdata alert) | 2025 | https://www.earthdata.nasa.gov/data/alerts-outages/transition-from-modis-viirs | - | Official shutdown framing: late 2026 / early 2027 | N |
| MODIS to VIIRS Transition (LAADS DAAC) | 2025 | https://ladsweb.modaps.eosdis.nasa.gov/learn/modis-to-viirs-transition | - | Dates differ between versions (Aqua Aug 2026 vs Sep 2027) | N |
| Drifting away from MODIS in the CAMS Global Fire Assimilation System | 2025 | https://atmosphere.copernicus.eu/drifting-away-modis-cams-global-fire-assimilation-system | - | CAMS began adding VIIRS 4 Dec 2025; Terra ends Jan 2027 | N |
| Terra begins to drift in time (NASA Terra) | 2020 | https://terra.nasa.gov/news/terra-begins-to-drift-in-time | - | Last inclination maneuver Feb 2020; 10:30 crossing time lost | N |
| Terra: the end of an era (NASA Science) | 2025 | https://science.nasa.gov/science-research/earth-science/terra-the-end-of-an-era/ | - | End of science projected Feb 2027 | N |
| NASA Fire Products: current status and future recommendations (Giglio, LCLUC) | 2022 | https://lcluc.umd.edu/sites/default/files/12%20Giglio_LCLUC_NASA_Fire_202210.pdf | - | Fire team view: no morning VIIRS; SLSTR pilot for Terra continuity | N |

## 5. News and motivation (2023-2026)

| Title | Year | URL | Citations | Takeaway | Implements? |
|---|---|---|---|---|---|
| A record-breaking boreal wildfire season (CAMS) | 2023 | https://atmosphere.copernicus.eu/record-breaking-boreal-wildfire-season | - | Canada 2023 emissions record | N |
| 2025 sees intense wildfire year in the Northern Hemisphere (CAMS) | 2025 | https://atmosphere.copernicus.eu/2025-sees-intense-wildfire-year-northern-hemisphere | - | Canada and Europe 2025 emissions | N |
| 2025 was EU's most destructive wildfire season on record (JRC) | 2026 | https://joint-research-centre.ec.europa.eu/jrc-news-and-updates/2025-was-eus-most-destructive-wildfire-season-record-2026-03-31_en | - | 1.08 Mha burned in the EU | N |
| Remote sensing during the 2025 Los Angeles wildfires (AGU Advances) | 2026 | https://agupubs.onlinelibrary.wiley.com/doi/10.1029/2025AV002092 | - | Responders used twice-daily VIIRS detections when aircraft grounded (403 bot block) | N |

## 6. Existing tools and code to differentiate from

| Title | Year | URL | Citations | Takeaway | Implements? |
|---|---|---|---|---|---|
| GWIS Global Wildfire Information System | 2025 | https://gwis.jrc.ec.europa.eu | - | Country profiles, seasonality, current-season charts; burned-area based | N |
| Global Fire Emissions Database (GFED) data page | 2025 | https://www.globalfiredata.org/data.html | - | Monthly burned area/emissions; no active-fire calendar | N |
| Global Fire Atlas (Zenodo update, 2002-2024) | 2024 | https://zenodo.org/records/11400062 | - | Per-fire size/duration atlas (403 bot block) | N |
| GitHub: Tanishkax/hotspot_mvp_prototype (HotSpot Sync) | 2026 | https://github.com/Tanishkax/hotspot_mvp_prototype | - | Per-latitude-band ratio scaling; held-out error +3.6% vs -82% raw MODIS (README claim) | Y |
| GitHub: the-aris-club/flare-chronos | 2026 | https://github.com/the-aris-club/flare-chronos | - | 26-year harmonized calendar on FIRMS with ingestion cache | Y |
| GitHub: bhavishyaone1/FireChronicle | 2026 | https://github.com/bhavishyaone1/FireChronicle | - | 11-stage pipeline methodology doc | Y |
| GitHub: tanishqkumar87919-dot/Harmonisation-of-MODIS-and-VIIRS-Hotspots- | 2026 | https://github.com/tanishqkumar87919-dot/Harmonisation-of-MODIS-and-VIIRS-Hotspots- | - | One-to-many pixel geometry (1 MODIS pixel ~7 VIIRS) | Y |
| GitHub: istiakahamedshimul/fire-harmonix | 2026 | https://github.com/istiakahamedshimul/fire-harmonix | - | Harmonized Fire Activity Index on 0.05 deg grid | Y |
| GitHub: harishk2086/FireHarmonize | 2026 | https://github.com/harishk2086/FireHarmonize | - | MATCHED_EVENT / MODIS_ONLY / VIIRS_ONLY labelling | Y |

## Key open problems / differentiation angles

- Overpass-time and diurnal-cycle correction: Terra drifts earlier than 10:30, Aqua later than 13:30, VIIRS fixed near 13:30; no published region-wise diurnal-aware correction of counts.
- Count and FRP harmonization: scale per land-cover/latitude band using overlap years (2012-present) with ratio-of-sums, and report uncertainty instead of one global factor.
- Terra-gap handling: no morning VIIRS; make the calendar degrade gracefully (optional SLSTR or geostationary morning inputs) after the 2026-27 shutdowns.
- Sensor-aware anomaly baseline: multi-year mean and SD on the harmonized series, with change-point tests at sensor transitions (2012 SNPP, 2018 NOAA-20, 2023 NOAA-21, Terra/Aqua end) so anomalies are not artefacts.
- Responder-first UX gap: FIRMS shows points, GWIS/GFED show burned area; none offers a user-drawn-polygon burning calendar with critical-period flags and per-sensor provenance.
