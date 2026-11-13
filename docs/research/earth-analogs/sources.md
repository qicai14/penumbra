# Earth analog sources for NASA Space Apps 2026

Curl check: HTTP 200 unless noted (403 = publisher bot-block, pages are OA; 000 = host unreachable from curl). Citations from OpenAlex; n/a = not retrieved.


## 1. Analog catalogs, reviews and similarity methods

| Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|
| Planetary habitability: lessons learned from terrestrial analogues (Preston & Dartnell) | 2014 | http://lewisdartnell.com/en-gb/wp-content/uploads/2014/01/18-Preston-Dartnell2014_IJA.pdf | 166 | Reviews analog sites by Mars/Europa/Titan environment; calls for an open global analog database. | N |
| Definition and use of functional analogues in planetary exploration (Foucher et al.) | 2021 | https://hal.science/hal-03130725/document | n/a | Formal split of general vs functional analogues; defines which properties must match for each use. | Y |
| Earth as a Tool for Astrobiology - A European Perspective | 2017 | https://link.springer.com/article/10.1007/s11214-017-0369-1 | n/a | Summarizes the ESA analog-site catalogue (37 sites; 2-21 are Mars analogs). | N |
| Lava tubes on Earth, Moon and Mars: size and morphology (Sauro et al.) | 2020 | https://cris.unibo.it/bitstream/11585/769982/5/2020%2bSauro%2bet%2bal%2bESR%2bLava%2btubes.pdf | 213 | Compares tube dimensions across bodies; gravity scaling explains why lunar tubes are huge (curl 403, OA). | Y |
| Fundamental Science and Engineering Questions in Planetary Cave Exploration | 2022 | https://doi.org/10.1029/2022JE007194 | 44 | Community roadmap for cave science, hazards and ops (curl 403, OA). | N |
| Radar Observation of the Lava Tubes on the Moon and Mars | 2023 | https://www.mdpi.com/2072-4292/15/11/2850 | 38 | Radar detection of subsurface voids; lets Earth GPR surveys serve as analog test (curl 403, OA). | Y |
| Mars Global Cave Candidate Catalog (MGC3), Cushing/USGS | 2017 | https://astrogeology.usgs.gov/search/map/mars_global_cave_candidate_catalog_v1_cushing | n/a | 1000+ Mars cave entrances; downloadable table for cave-density comparison. | Y |
| Deep learning object detection for mapping cave candidates on Mars (MGC3 build-up) | 2021 | https://www.hou.usra.edu/meetings/lpsc2021/pdf/1316.pdf | n/a | Method and training data for detecting pits/skylights. | Y |
| Lunar Pit Atlas (LROC) | 2021 | https://www.lroc.asu.edu/atlases/pits | n/a | Catalog of ~280 lunar pits with sizes; pair with MGC3 and Earth lava-tube skylight inventories. | Y |
| New candidate cave entrances on the Moon found using deep learning (Le Corre et al.) | 2023 | https://kar.kent.ac.uk/110353/1/New%20candidate%20cave%20entrances%20on%20the%20Moon%20found%20using%20deep%20learning.pdf | n/a | ML pit detector on LRO images with open weights. | Y |
| Deep learning selection of analogues for Mars landing sites in the Qaidam Basin | 2025 | https://arxiv.org/pdf/2501.08584 | n/a | CNN feature-vector Euclidean distance between Earth and Mars imagery as analog score. | Y |
| Remote and in-situ characterization of Mars analogs: coupling scales (Harris et al.) | 2022 | https://www.frontiersin.org/articles/10.3389/fspas.2022.849078/full | n/a | Compares 5 Mars-analog lakes on environmental, mineral and biological parameters; scale coupling. | Y |
| Planetary analog sites in the Indian subcontinent and Indian Ocean: underexplored | 2025 | https://www.frontiersin.org/articles/10.3389/fspas.2025.1712191/full | 1 | Recent new-analog-site proposals outside the usual list. | N |
| Europlanet TA1 Planetary Field Analogue sites (Rio Tinto, Danakil, Makgadikgadi, Puna, Kangerlussuaq) | 2024 | https://www.europlanet-society.org/europlanet-2024-ri/ta1-pfa | n/a | Established site list to exclude or benchmark against; includes lesser-known ones. | N |
| Analog Sites for Mars Missions II (LPI contribution) | 2003 | https://www.lpi.usra.edu/lpi/contribution_docs/LPI-001738.pdf | n/a | Classic workshop criteria for mission-relevant analogs. | N |
| Principles for Integrating Mars Analog Science, Operations, and Technology Research | 2003 | https://ntrs.nasa.gov/citations/20030065746 | n/a | Operations-fidelity framework for analog selection. | N |
| Using Earth-based Operational Field Tests as High-Fidelity Analogs for Planetary Surface Exploration | 2021 | https://ntrs.nasa.gov/citations/20210014301 | n/a | Ops dimension: what makes a field test a faithful analog. | N |
| Living at the Extremes: Extremophiles and the Limits of Life in a Planetary Context | 2019 | https://www.frontiersin.org/articles/10.3389/fmicb.2019.00780/full | 707 | Habitability envelope (T, water activity, radiation) to score habitability dimension. | Y |

## 2. Moon/Mars base-site requirements and resources

| Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|
| Evaluating potential landing sites for Artemis III using a multi-criteria decision approach | 2024 | https://doi.org/10.1016/j.actaastro.2024.10.049 | 28 | Weighted MCDM scoring of south-pole sites; template for our weights. | Y |
| Analysis of Illumination Conditions in the Lunar South Polar Region Using Multi-source DEMs | 2023 | https://www.mdpi.com/2072-4292/15/24/5691 | 24 | Illumination fraction method from DEM horizon masks (curl 403, OA). | Y |
| Regions of interest (ROI) for future exploration missions to the lunar South Pole | 2019 | https://doi.org/10.1016/j.pss.2019.104750 | 103 | ROI criteria: slope, illumination, Earth visibility, PSR access. | Y |
| Improved LOLA elevation maps for south pole landing sites | 2020 | https://doi.org/10.1016/j.pss.2020.105119 | 139 | Gridded DEM error estimates and slope/roughness at landing scale. | Y |
| Characterisation of Potential Landing Sites for ESA's Lunar Lander Project | 2012 | https://arxiv.org/abs/1208.5587 | 117 | Slope and illumination methodology with explicit thresholds. | Y |
| Artemis III Science Definition Team Report Briefing | 2021 | https://ntrs.nasa.gov/citations/20210009687 | n/a | Science and site objectives for first south-pole landing. | N |
| Artemis Internal Science Team: Mission Planning Update | 2023 | https://ntrs.nasa.gov/citations/20230000192 | n/a | Updated landing region priorities. | N |
| Lunar resources (review) | 2015 | https://arxiv.org/abs/1410.6865 | 430 | Resource inventory (ice, regolith, metals) to define what a base needs. | N |
| Human Mars Landing Site and Impacts on Mars Surface Operations | 2016 | https://ntrs.nasa.gov/citations/20150013826 | 25 | Exploration Zone (EZ) criteria: ~100 km radius, ice, science, ops. | Y |
| Mars Surface Systems Common Capabilities and Challenges for Human Missions | 2016 | https://ntrs.nasa.gov/citations/20150019647 | 1 | Surface ops constraints (power, dust, thermal) driving site needs. | N |
| A Lunar Water ISRU System Study for Human-Scale Propellant Production | 2021 | https://ntrs.nasa.gov/citations/20210016820 | n/a | Ice grade and quantity thresholds for ISRU. | Y |
| Refined Mapping of Subsurface Water Ice on Mars to Support Future Missions (SWIM) | 2025 | https://iopscience.iop.org/article/10.3847/PSJ/ad9b24 | n/a | Ice-table depth map 60N-60S; core Mars water layer. | Y |
| Thermal Detectability of Subsurface Water Ice on Mars (SWIM) | 2025 | https://iopscience.iop.org/article/10.3847/PSJ/ae7b38 | n/a | SWIM23 TES ice map compared with earlier products. | Y |
| Subsurface Water Ice Mapping on Mars: A Probabilistic Approach | 2025 | https://iopscience.iop.org/article/10.3847/PSJ/ae8904 | n/a | Bayesian probability maps; uncertainty handling we can mirror. | Y |
| The Martian mid-latitude subsurface ice is the remnant of a past ice sheet | 2026 | https://arxiv.org/abs/2602.18847 | n/a | Genesis of mid-latitude ice; points to glacial/periglacial Earth analogs. | N |
| Global regolith thermophysical properties of the Moon from Diviner (Hayne et al.) | 2017 | https://arxiv.org/abs/1711.00977 | 393 | Rock abundance, regolith density and conductivity; thermal-inertia analog dimension. | Y |

## 3. Data and tool documentation

| Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|
| The Lunar Reconnaissance Orbiter Diviner Lunar Radiometer Experiment | 2009 | https://link.springer.com/article/10.1007/s11214-009-9529-2 | 479 | Instrument and temperature product definitions. | N |
| NASA's Solar System Treks: Online Portals for Planetary Mapping and Modeling | 2018 | https://ntrs.nasa.gov/citations/20180000384 | 2 | Describes Trek layers and web services for programmatic access. | Y |
| Moon Trek | 2024 | https://trek.nasa.gov/moon/ | n/a | Browse/download LOLA, Diviner, illumination layers. | Y |
| Mars Trek | 2024 | https://trek.nasa.gov/mars/ | n/a | MOLA, CTX, CRISM, THEMIS layers. | Y |
| Diviner Lunar Radiometer (UCLA) data portal | 2024 | https://diviner.ucla.edu/ | n/a | Surface temperature, rock abundance products. | Y |
| PDS Geosciences: LOLA | 2024 | https://pds-geosciences.wustl.edu/missions/lro/lola.htm | n/a | Raw/gridded lunar topography archive. | Y |
| PDS Geosciences: CRISM | 2024 | https://pds-geosciences.wustl.edu/missions/mro/crism.htm | n/a | Mars mineralogy cubes and summary parameters. | Y |
| THEMIS (Mars Odyssey) thermal inertia and imaging | 2024 | https://themis.asu.edu/ | n/a | Thermal inertia mosaics for dust/rock proxy. | Y |
| USGS Astropedia: LRO LOLA DEM 118 m | 2024 | https://astrogeology.usgs.gov/search/map/moon_lro_lola_dem_118m | n/a | Ready lunar DEM for slope/roughness. | Y |
| USGS Astropedia: MOLA-HRSC blended DEM 200 m | 2024 | https://astrogeology.usgs.gov/search/map/mars_mgs_mola_mex_hrsc_blended_dem_global_200m | n/a | Mars global DEM. | Y |
| USGS/JPL ASTER Spectral Library (ECOSTRESS speclib) | 2024 | https://speclib.jpl.nasa.gov/ | n/a | Lab emissivity/reflectance spectra for mineral matching. | Y |
| MODIS Land Surface Temperature MOD11A1 v061 | 2024 | https://lpdaac.usgs.gov/products/mod11a1v061/ | n/a | Daily Earth LST for thermal amplitude analogs. | Y |
| ERA5 hourly single levels (Copernicus CDS) | 2024 | https://cds.climate.copernicus.eu/datasets/reanalysis-era5-single-levels | n/a | Wind, pressure, temperature for dust/wind dimension. | Y |
| WorldClim 2.1 climate surfaces | 2020 | https://www.worldclim.org/data/worldclim21.html | n/a | 1 km temperature/precip climatologies. | Y |
| Present and future Koppen-Geiger climate classification maps at 1 km (Beck et al.) | 2018 | https://www.nature.com/articles/sdata2018214 | n/a | Climate class layer for pre-filtering cold/arid Earth regions. | Y |
| Copernicus DEM (GLO-30) description | 2024 | https://dataspace.copernicus.eu/explore-data/data-collections/copernicus-contributing-missions/collections-description/COP-DEM | n/a | 30 m global DEM for Earth slope/roughness. | Y |
| Google Earth Engine data catalog | 2024 | https://developers.google.com/earth-engine/datasets | n/a | One place to pull Earth rasters (SRTM, MODIS, ERA5). | Y |

## 4. Recent news (2023-2026)

| Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|
| NASA Moon Base (formerly Artemis Base Camp) page | 2026 | https://www.nasa.gov/moon-base/ | n/a | Official base program overview. | N |
| NASA lays out moon base plans with landers, buggies and drones (NPR) | 2026 | https://www.npr.org/2026/05/27/nx-s1-5836209/nasa-moon-base-plans | n/a | Three-phase plan, south-pole siting, Shackleton connecting ridge. | N |
| Artemis moon base will cover hundreds of square miles (Space.com) | 2026 | https://www.space.com/astronomy/moon/artemis-moon-base-will-cover-hundreds-of-square-miles-with-hopping-drones-and-new-lunar-rovers-nasa-says | n/a | Base footprint scale: argues analog regions must be large. | N |
| NASA's second CHAPEA crew completes 378-day Mars simulation | 2026 | https://newspaceeconomy.ca/2026/11/02/nasas-second-chapea-crew-completes-a-378-day-mars-simulation/ | n/a | Mission end and next Moon-and-Mars analog from 2027. | N |
| HI-SEAS IV microbiome dynamics and implications for future crewed missions | 2021 | https://microbiomejournal.biomedcentral.com/articles/10.1186/s40168-020-00959-x | 37 | Habitat analog operations and habitability finding. | N |

## 5. Existing tools / prior art to differentiate from

| Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|
| NASA Space Apps 2026 challenge page: Permanent Moon Base Locations and Mars | 2026 | https://www.spaceappschallenge.org/2026/challenges/permanent-moon-base-locations-and-mars/ | n/a | Official brief (curl cannot reach host; user-supplied URL). | N |
| terrestrial-analog-finder (GitHub, Space Apps 2026 entry) | 2026 | https://github.com/shishir-kuet/terrestrial-analog-finder | n/a | Terrain-only similarity (relief, slope, roughness, hypsometry); our multi-dimensional approach should exceed it. | Y |
| Finding Earth's Best Testbeds for Moon and Mars Rovers: A Quantitative Terrain Comparison | 2026 | https://nhsjs.com/2026/finding-earths-best-testbeds-for-moon-and-mars-rovers-a-quantitative-terrain-comparison/ | n/a | Image-texture similarity only; no thermal, mineral or ops dimensions. | Y |
| The Planetary Terrestrial Analogues Library (PTAL) | 2021 | https://www.sciencedirect.com/science/article/pii/S0032063321001781 | n/a | Rock/regolith sample database for Mawrth Vallis and Oxia Planum (curl 403, OA). | N |

## Key open problems / differentiation angles

- Existing entries (terrain-only DEM similarity, CNN image features) ignore thermal, mineral and water axes; build a per-dimension similarity vector with explicit "similar / how much / differs" output.
- No published analog score uses uncertainty: carry SWIM-style probabilities and DEM error (LOLA/Copernicus) through to the score.
- Scale mismatch: Earth gravity changes lava-tube size, dust/wind scaling and thermal amplitude; add process-analog and gravity-scaled terms, not just surface lookalikes.
- Subsurface analogs (lava tubes, ground ice, permafrost) are rarely scored; join MGC3, Lunar Pit Atlas and Earth skylight/permafrost inventories.
- Operations dimension (accessibility, power from illumination, comms, logistics) is absent from prior scores; derive it from Artemis ROI and Mars EZ criteria and rank new, non-classic sites.
