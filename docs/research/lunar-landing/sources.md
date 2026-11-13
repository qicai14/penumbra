# Lunar Landing Site Navigator: curated NotebookLM sources

Citation counts from OpenAlex (Nov 2026); n/a = not a journal article or not looked up.


## A. Challenge and Artemis/CLPS site context

| # | Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|---|
| 1 | Lunar Landing Site Navigator (NASA Space Apps 2026 challenge) | 2026 | https://www.spaceappschallenge.org/2026/challenges/lunar-landing-site-navigator/ | n/a | Official brief; curl could not reach this host from the research machine (000), so unverified | N |
| 2 | Artemis III Site Selection Update: Downselection From Thirteen to Nine Candidate Regions | 2025 | https://ntrs.nasa.gov/citations/20250011660 | n/a | Criteria and rationale for the 13 to 9 region cut; defines the sites to compare | Y |
| 3 | NASA Provides Update on Artemis III Moon Landing Regions | 2024 | https://www.nasa.gov/news-release/nasa-provides-update-on-artemis-iii-moon-landing-regions/ | n/a | Official list of nine regions (Haworth, Malapert, Mons Mouton, Nobile, de Gerlache Rim 2, Slater Plain, Peak near Cabeus B) | Y |
| 4 | NASA Identifies Candidate Regions for Landing Next Americans on Moon | 2022 | https://www.nasa.gov/news-release/nasa-identifies-candidate-regions-for-landing-next-americans-on-moon/ | n/a | Original 13 regions, about 15x15 km each, ~100 m landing-site radius, selection criteria | Y |
| 5 | Evaluating potential landing sites for the Artemis III mission using a multi-criteria decision making approach | 2024 | https://arxiv.org/abs/2406.19863 | n/a | TOPSIS ranking of candidate regions on illumination, Earth visibility, slope; template for a scoring metric | Y |
| 6 | ARTEMIS III Candidate Landing Regions: Assessing the Likelihood of Spa-Derived Materials | 2025 | https://ntrs.nasa.gov/citations/20250002110 | n/a | Science-side view of the candidate regions (compositional) | N |
| 7 | Dynamic Secondary Illumination in Permanent Shadows within Artemis III Candidate Landing Regions | 2024 | https://ntrs.nasa.gov/citations/88786892701355 | n/a | Secondary (scattered) illumination inside PSRs for the candidate regions | N |
| 8 | Enhanced Topography Models for Selected Lunar South Pole Regions with Shape-from-Shading | 2026 | https://ntrs.nasa.gov/citations/20260004665 | n/a | Sub-LOLA-resolution DEMs for candidate regions; terrain-accuracy caveat for horizons | Y |
| 9 | Artemis III EVA Mission Capability for de Gerlache-Shackleton Ridge | 2022 | https://ntrs.nasa.gov/citations/20210026296 | n/a | EVA and traverse planning constraints tied to lighting and comm | N |

## B. Illumination, horizon and Earth-visibility modeling

| # | Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|---|
| 10 | Illumination Conditions of the Lunar Polar Regions Using LOLA Topography (Mazarico et al.) | 2011 | https://ntrs.nasa.gov/citations/20120010094 | 393 | Canonical horizon-map illumination algorithm on LOLA DEMs; illumination %, longest night, Earth visibility | Y |
| 11 | Illumination Conditions at the Lunar Poles: Implications for Future Exploration | 2017 | https://ntrs.nasa.gov/citations/20170007365 | 118 | High-res LOLA DTM illumination, compared with LROC results (Speyerer & Robinson 2013) | Y |
| 12 | Lunar Pole Illumination and Communications Maps Computed from GSSR Elevation Data | 2009 | https://ntrs.nasa.gov/citations/20150014706 | n/a | Early joint Sun + Earth visibility method (illumination and direct-to-Earth comm statistics) | Y |
| 13 | Lunar South Pole Illumination: Review, Reassessment, and Power System Implications | 2007 | https://ntrs.nasa.gov/citations/20070034951 | n/a | Links illumination statistics to solar power system design | Y |
| 14 | Numerical Simulation of Illumination and Thermal Conditions at the Lunar Poles Using LOLA DTMs | 2017 | https://ntrs.nasa.gov/citations/20170009130 | n/a | Ray-traced illumination plus thermal on LOLA DTMs | Y |
| 15 | Effective Crewed Surface Science near the Lunar South Pole: Some Illumination Considerations | 2020 | https://ntrs.nasa.gov/citations/20200003145 | n/a | Sunlight and shadow constraints on crewed EVA planning | N |
| 16 | Improved LOLA Elevation Maps for South Pole Landing Sites: Error Estimates and Impact on Illumination (Barker et al.) | 2020 | https://ntrs.nasa.gov/citations/20205009660 | 139 | DEM error propagates into illumination uncertainty; ~5 m south-pole maps | Y |
| 17 | A New View of the Lunar South Pole from LOLA (Barker et al.) | 2023 | https://iopscience.iop.org/article/10.3847/PSJ/acf3e1 | n/a | Current LOLA south-pole DEM (LDEM_80S_20MPP_ADJ) and its error products | Y |
| 18 | Towers on the Peaks of Eternal Light: Quantifying the Available Solar Power | 2021 | https://arxiv.org/abs/2102.11766 | n/a | Power vs tower height with illumination; quantifies solar generation potential | Y |
| 19 | Earthshine as an Illumination Source at the Moon | 2019 | https://arxiv.org/abs/1904.00236 | n/a | Earthshine radiance model for PSRs; ties to Earth-visibility geometry | Y |
| 20 | The Spectral Radiance of Indirectly Illuminated Surfaces in Regions of Permanent Shadow on the Moon | 2020 | https://ntrs.nasa.gov/citations/20205011608 | n/a | Indirect illumination radiometry in PSRs | N |
| 21 | Recovery Policies for Safe Exploration of Lunar PSRs by a Solar-Powered Rover | 2023 | https://arxiv.org/abs/2307.16786 | n/a | Uses illumination time series for rover energy planning | N |
| 22 | Safe Mission-Level Path Planning for Exploration of Lunar Shadowed Regions by a Solar-Powered Rover | 2024 | https://arxiv.org/abs/2401.08558 | n/a | Time-dependent sun visibility as a planning constraint | N |

## C. Topography and mapped data (LOLA, LRO)

| # | Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|---|
| 23 | LOLA South Pole DEM products, PGDA (LDEM_80S_20MPP_ADJ) | 2023 | https://pgda.gsfc.nasa.gov/products/90 | n/a | Download page, formats, frame (MOON_ME / DE421) and resolution of the 20 m south-pole DEM | Y |
| 24 | PDS Geosciences Node: LRO LOLA data | 2024 | https://pds-geosciences.wustl.edu/missions/lro/lola.htm | n/a | Archive entry for all LOLA products | Y |
| 25 | LRO LOLA Gridded Data Record Shape Map (GDRDEM) help page, ODE | 2024 | https://ode.rsl.wustl.edu/moon/pagehelp/Content/Missions_Instruments/LRO/LOLA/GDR/GDRDEM.htm | n/a | File layout and polar stereographic grid definitions for GDR DEMs | Y |
| 26 | Summary of the Results from LOLA after Seven Years in Lunar Orbit | 2016 | https://ntrs.nasa.gov/citations/20160008718 | n/a | Instrument performance, topography and accuracy summary | N |
| 27 | A New Lunar Digital Elevation Model from LOLA and SELENE Terrain Camera (Barker et al.) | 2016 | https://ntrs.nasa.gov/citations/20170002253 | 598 | Merged DEM, error budget, global context | Y |

## D. Volatiles, PSRs and landing-site science

| # | Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|---|
| 28 | Micro cold traps on the Moon (Hayne et al.) | 2020 | https://arxiv.org/abs/2005.05369 | 133 | Sub-km cold-trap area from roughness; why local horizons matter | Y |
| 29 | Direct evidence of surface exposed water ice in the lunar polar regions (Li et al.) | 2018 | https://pmc.ncbi.nlm.nih.gov/articles/PMC6130389/ | n/a | Shows where ice is exposed in PSRs | N |
| 30 | Characterisation of potential landing sites for the ESA Lunar Lander project | 2012 | https://arxiv.org/abs/1208.5587 | 117 | Landing-site screening on illumination, Earth visibility and slopes with LOLA; close methodological precedent | Y |
| 31 | Evidence for Widespread Hydrogen Sequestration within the Moon's South Polar Cold Traps | 2023 | https://arxiv.org/abs/2303.03911 | n/a | Cold-trap volatile evidence, context for PSR targets | N |

## E. Ephemerides, SPICE and Horizons (data docs)

| # | Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|---|
| 32 | The JPL Planetary and Lunar Ephemerides DE440 and DE441 (Park et al.) | 2021 | https://iopscience.iop.org/article/10.3847/1538-3881/abd414 | n/a | DE440 accuracy and time span (1550-2650); source of lunar libration | Y |
| 33 | Horizons API documentation | 2024 | https://ssd-api.jpl.nasa.gov/doc/horizons.html | n/a | REST parameters (COMMAND, CENTER, QUANTITIES, EPHEM_TYPE) for Sun/Earth az-el from Moon sites | Y |
| 34 | Horizons System Manual | 2024 | https://ssd.jpl.nasa.gov/horizons/manual.html | n/a | Site-on-body observer tables, output quantities (azimuth/elevation, libration) | Y |
| 35 | NAIF lunar kernels overview (lunar_kernels.txt) | 2024 | https://naif.jpl.nasa.gov/naif/lunar_kernels.txt | n/a | Which PCK/FK/SPK to load for lunar work | Y |
| 36 | NAIF generic_kernels FK/satellites readme (moon_de440 frame kernel) | 2025 | https://naif.jpl.nasa.gov/pub/naif/generic_kernels/fk/satellites/aareadme.txt | n/a | Current DE440 lunar FK pairing; MOON_PA vs MOON_ME frames | Y |
| 37 | moon_pa_de440_200625 binary PCK comments | 2020 | https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/moon_pa_de440_200625.cmt | n/a | Coverage (1549-2650) and how to load moon_pa orientation | Y |
| 38 | NAIF tutorial: Lunar and Earth PCK/FK high-accuracy orientation | 2016 | https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/Tutorials/pdf/individual_docs/23_lunar-earth_pck-fk.pdf | n/a | Step-by-step: build lunar body-fixed to J2000 transforms | Y |
| 39 | DE440 and DE441 summary (NAIF, PDF) | 2021 | https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440_and_de441.pdf | n/a | Which SPK to use and accuracy | Y |
| 40 | DE440 technical comments | 2020 | https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440_tech-comments.txt | n/a | Lunar libration and parameters in DE440 | Y |
| 41 | SpiceyPy documentation | 2024 | https://spiceypy.readthedocs.io/en/stable/ | n/a | Python SPICE API (spkpos, azlcpo, furnsh) for implementation | Y |
| 42 | Regionalized Lunar South Pole Surface Navigation System Analysis | 2008 | https://ntrs.nasa.gov/citations/20080018575 | n/a | Surface comm/nav coverage analysis at the south pole with terrain | N |

## F. Existing tools and open-source code (differentiation)

| # | Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|---|
| 43 | NASA Moon Trek | 2024 | https://trek.nasa.gov/moon/ | n/a | Web portal; lighting/line-of-sight tools need an account, no multi-site date comparison | N |
| 44 | LROC QuickMap | 2024 | https://quickmap.lroc.im-ldi.com | n/a | Layers, TerrainShadows synthetic images; map-centric, not horizon-profile comparison | N |
| 45 | Moon Trek: An Interactive Web Portal for Current and Future Lunar Missions | 2017 | https://ntrs.nasa.gov/citations/20170008884 | n/a | Design/capability description of Moon Trek | N |
| 46 | Lunar QuickMap: Meeting the Challenge of Polar Exploration | 2022 | https://www.hou.usra.edu/meetings/lunarpolar2022/pdf/5028.pdf | n/a | QuickMap polar illumination and terrain-shadow features | N |
| 47 | LROC curated downloads (polar illumination maps) | 2024 | https://lroc.im-ldi.com/images/downloads | n/a | Time-weighted and percent illumination polar maps for validation | Y |
| 48 | localizedprojects/CLPS-MOON-BROWSER (Lunar Horizon) | 2026 | https://github.com/localizedprojects/CLPS-MOON-BROWSER | n/a | Competing Space Apps entry for the same brief: DE440, horizons, scoring | Y |
| 49 | HAZAKIKO/lunar-shadow-algorithms | 2026 | https://github.com/HAZAKIKO/lunar-shadow-algorithms | n/a | NumPy DEM ray-march + horizon-map shadow code with self-checks | Y |
| 50 | npbassett/shapes | 2023 | https://github.com/npbassett/shapes | n/a | Horizon calculator for Earth/Moon sites using LOLA | Y |
| 51 | Hamzaslam1996/lunarwindow | 2026 | https://github.com/Hamzaslam1996/lunarwindow | n/a | Space Apps 2026 entry: SPICE DE440 + LOLA horizon mask, illumination, DTE windows | Y |
| 52 | pythonhack000/lunar-navigator | 2026 | https://github.com/pythonhack000/lunar-navigator | n/a | 720-azimuth great-circle horizon march (1-300 km) | Y |

## G. Recent news and real-world failures (2024-2026)

| # | Title | Year | URL | Cites | Takeaway | Implements? |
|---|---|---|---|---|---|---|
| 53 | Athena moon mission comes to early end after lander tips over (CBS) | 2025 | https://www.cbsnews.com/news/athena-moon-lander-tips-over-intuitive-machines-mission/ | n/a | IM-2 tipped in a crater near Mons Mouton; low sun left it unable to recharge | N |
| 54 | Here's why the private Athena lunar lander toppled over (Space.com) | 2025 | https://www.space.com/astronomy/moon/heres-why-the-private-athena-lunar-lander-toppled-over-on-the-moon | n/a | Postmortem: altimeter interference, terrain/lighting, crater-recognition mismatch | N |
| 55 | Intuitive Machines and NASA call IM-1 a success as mission winds down (SpaceNews) | 2024 | https://spacenews.com/intuitive-machines-and-nasa-call-im-1-lunar-lander-a-success-as-mission-winds-down/ | n/a | IM-1 tipped on a ~12 degree slope; ended as lunar night/power ran out | N |
| 56 | Navigation Doppler Lidar Performance Assessment and Trajectory Reconstruction of IM-1 | 2025 | https://ntrs.nasa.gov/citations/20250007036 | n/a | Engineering reconstruction of the IM-1 descent | N |
| 57 | Touchdown! Carrying NASA Science, Firefly's Blue Ghost Lands on Moon | 2025 | https://www.nasa.gov/news-release/touchdown-carrying-nasa-science-fireflys-blue-ghost-lands-on-moon/ | n/a | Successful CLPS landing timed to lunar sunrise (Mare Crisium) | N |
| 58 | Firefly Blue Ghost Mission One Press Kit (NASA CLPS) | 2025 | https://www.nasa.gov/wp-content/uploads/2025/01/np-2025-01-001-jsc-clps-firefly-press-kit-screen-508-1-10-25-final.pdf | n/a | Mission timeline built around Sun geometry; day/night budget | N |
| 59 | NASA selects Blue Origin to take the once-canceled VIPER rover to the moon (Phys.org) | 2025 | https://phys.org/news/2025-09-nasa-blue-canceled-viper-rover.html | n/a | VIPER revived for a late-2027 CLPS delivery after 2024 cancellation | N |
| 60 | Commercial Lunar Payload Services (NASA) | 2025 | https://www.nasa.gov/clps | n/a | Official CLPS program and delivery list | N |

## Key open problems / differentiation angles

- Per-site horizon profiles (azimuth vs elevation) with Sun and Earth tracks overlaid are rare; Moon Trek and QuickMap are map/raster-centric and the polar illumination papers publish maps, not site-by-date comparisons.
- Landing-date selection (time to sunset and Earthset over the ~6.5-day surface window the Artemis criteria use) is rarely a first-class output; existing tools give illumination % over a year.
- DEM error matters: Barker 2020/2023 show LOLA error shifts illumination; propagate DEM uncertainty and show confidence bands, and flag sub-LOLA features (shape-from-shading, micro cold traps).
- Earth visibility needs libration-aware DE440 / moon_pa transforms; some hobby codes ignore libration or use a flat horizon. A joint Sun AND Earth window (power plus comm) is the key metric.
- IM-1/IM-2 show the real failure is low-angle lighting plus slope; add shadow timing and slope context to the horizon view. Artemis III was reportedly restructured (Feb 2026) into a LEO docking test with the landing on Artemis IV (~2028), so the nine-region list stays the reference but dates are in flux (not verified against a primary NASA page). Rival Space Apps 2026 repos already exist (CLPS-MOON-BROWSER, lunarwindow).
