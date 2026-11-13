# NASA Space Apps Challenge 2026: challenge research

> **STATUS (checked 2026-11-08): the 2026 event has NOT happened yet.** The hackathon runs **Nov 14-15, 2026**, six days after today. Registration stays open until the hackathon closes on Nov 15. Project submission opens Sat Nov 14 09:00 and closes Sun Nov 15 23:59 (local time of your event). You can still register, join or form a team, and pick a challenge.

Sources: spaceappschallenge.org/2026/, /2026/challenges/, the 14 challenge pages, /resources/participant-faqs/. WebFetch failed with a TLS certificate error, so I read the JS-rendered pages with the built-in browser. **Per-challenge "Resources" tabs all say "Resources will be available on Friday, November 13."** So there are no official dataset lists yet. The "data" notes below come only from the Details text. Dataset URLs are left blank on purpose rather than guessed.

## 1. Event facts and rules

| Item | Value |
|---|---|
| Theme | "The Next Frontier" |
| Dates | Nov 14-15, 2026 (Local Events plus a virtual asynchronous Universal Event) |
| Submission window | Opens Sat Nov 14, 9:00 AM; closes Sun Nov 15, 11:59 PM local time. **NSAKL26 (our event) closes at 22:00 (Telegram, Nov 14).** FAQ: submit "according to the ending time of your Local Event" |
| Team size | 1-6 people. The Universal Event needs at least 2 people to be eligible for global judging. A person can be on one team, and a team on one challenge |
| Registration | Required for global judging. Choose a Local Event or the Universal Event. Teams can be changed until Nov 15 |
| Submission | Through your team's Project tab on spaceappschallenge.org. The Project page holds a demo link, a project link, a detailed description, a data list and an SI/AI-use statement. This is based on 2025 team pages seen in the browser, not on a 2026 guide |
| Demo video / deployment rules | **Not published yet.** The Project Submission Guide and the Judging and Awards Guide are due Nov 13. The 2025 guide URL returns 404. The 2025 team pages I saw had a demo video link (Drive) and a live project URL (Vercel), so plan for both |
| Judging criteria | **Not published yet** (Judging and Awards Guide, Nov 13). Judges come from NASA, space agency partners and industry. Outcomes are Honorable Mention, Global Finalist and Global Winner |
| SI/AI use | Allowed. The submission **must describe how SI tools were used**. The site uses the term "SI" (Super Intelligence) in place of "AI" |
| Under-18s | Special rules on likeness and parental registration |
| Global collaborators | GoDaddy, Google, Meteomatics, Microsoft, Miro (free "Global Offers", available Nov 13) |
| Reward | Certificate for every participant who submits |

Practical implications for a solo build:
- Ship a live, public URL and a short demo video. The judging rubric is unknown until Nov 13.
- Keep a written log of how SI tools were used, since the submission requires it.
- A solo entrant must register for a Local Event, because the Universal Event requires at least 2 people.
- Several challenges say a browser-playable tool with no login reaches judges best.

## 2. Scoring table (1-5; 5 is best)

Columns: Sci = scientific depth, Data = NASA data richness, Paper = paper-based grounding potential, Real = real/recent problem, Vis = 3D/AR/visual impact, Solo = solo-feasible in about 48h with AI agents.

| # | Challenge | Diff | Sci | Data | Paper | Real | Vis | Solo | **Total /30** |
|---|---|---|---|---|---|---|---|---|---|
| 11 | Lunar Landing Site Navigator | Adv/Int | 5 | 4 | 4 | 5 | 5 | 4 | **27** |
| 8 | Harmonization of MODIS and VIIRS Hot Spots | Adv/Int | 4 | 5 | 4 | 5 | 3 | 4 | **25** |
| 2 | Earth System Trend Detective | Adv | 4 | 5 | 4 | 4 | 3 | 4 | **24** |
| 9 | Earth Analogs of Moon Base / Mars | Adv/Int/Beg | 4 | 5 | 4 | 3 | 4 | 3 | **23** |
| 6 | Field Shift: Adapting Farms | Adv/Int | 4 | 5 | 4 | 4 | 3 | 3 | **23** |
| 5 | Dancing with the SARs (NISAR) | Adv/Int | 4 | 4 | 4 | 4 | 4 | 3 | **23** |
| 12 | Planet X and SPHEREx | Adv | 4 | 4 | 4 | 4 | 4 | 3 | **23** |
| 7 | Flame in Freefall (microgravity combustion) | Adv/Int | 4 | 3 | 5 | 4 | 3 | 3 | **22** |
| 10 | Interplanetary Survival Guide: Martian Map | Int | 3 | 5 | 3 | 3 | 5 | 3 | **22** |
| 3 | Junior Astronaut Mission Trainer | Int/Beg | 2 | 3 | 2 | 2 | 3 | 4 | **16** |
| 4 | Health Monitoring Software for Astronauts | Int | 2 | 2 | 3 | 3 | 2 | 4 | **16** |
| 13 | Space Mission Design Game | Int/Beg | 3 | 2 | 2 | 2 | 3 | 4 | **16** |
| 1 | Abandoned but not Forgotten (storytelling) | Int/Beg | 1 | 2 | 1 | 2 | 3 | 4 | **13** |
| 14 | The Earth Information Jukebox | Int/Beg | 1 | 2 | 1 | 2 | 3 | 3 | **12** |

Scores are my judgment from the Details text only. Resource lists were not yet published.

## 3. Per-challenge detail

All URLs are under `https://www.spaceappschallenge.org/2026/challenges/<slug>/`. Event is the 2026 NASA Space Apps Challenge. Every challenge page says "refer to the Resources tab. More resources may be added before the hackathon," and that tab opens Nov 13. The **data resources** line lists only datasets named in the Details text.

### 1. Abandoned but not Forgotten: Storytelling about NASA's Discarded Equipment on the Moon and Mars
- URL: `.../nasas-discarded-equipment-storytelling/`
- Difficulty: Intermediate, Beginner/Youth
- Tags: Astrophysics, Planets & Moons, Space Exploration
- Summary: Tell the story of NASA hardware left on the Moon, Mars and in deep space, such as rovers, descent stages, instruments and probes. The audience is school-age space enthusiasts. The priority is turning historical and scientific data into an engaging narrative. Projects could be an interactive map of the equipment or another storytelling medium. Accuracy is required, but the judging emphasis is narrative.
- Data resources: none named (Resources tab opens Nov 13).
- Potential considerations: Focus on narrative and show how the missions operated. Link the hardware's engineering to the science gained. Decide the target age group and the accessibility of the format. Choose between specific missions and a broader story. Choose which perspectives to highlight.

### 2. Be An Earth System Trend Detective!
- URL: `.../earth-system-trend-detective/`
- Difficulty: Advanced
- Tags: Earth Science, Software
- Summary: Build a tool that selects variables from Earth science missions or models and visualizes how they change over time. It must determine what is changing, where, and by how much. It must also say whether the change is statistically "significant". The same process can drive opposite trends in different regions. Variable selection and the significance test are the two hard parts.
- Data resources: unspecified "variables measured by Earth science missions or models". CUAHSI hydrology plotting tools are mentioned as outside tools.
- Potential considerations: The audience is high school and college students and scientifically curious adults. Give simple variable definitions. Use a guided question flow plus demo examples. SI may help find data likely to show trends, drawing on news and papers or on regional characteristics (desert, rainforest, estuary, city atmosphere). Daily and monthly data are easiest. A weak trend is still interesting. Trends can be non-linear, and the seasonal cycle should be removed. Linear trend detection is the expected baseline.

### 3. Build a Junior Astronaut Mission Trainer
- URL: `.../junior-astronaut-mission-trainer/`
- Difficulty: Intermediate, Beginner/Youth
- Tags: Games, Planets & Moons, Software, Space Exploration, Sun
- Summary: Design a game or app where students run a lunar or Martian outpost. They balance life support, radiation shielding, power and food production. The aim is to make real engineering trade-offs tangible for kids aged 7-12. Real data should ground the game, such as space weather and radiation readings.
- Data resources named: DONKI (Space Weather Database), OSDR (Open Science Data Repository, astronaut dosimetry and biology), Artemis and Mars surface mission environmental data.
- Potential considerations: Audience is ages 7-12 and should also work for families and classrooms. Use plain language. A browser-playable game with no download or account reaches the most judges. Pull from NASA APIs, or embed a curated real dataset and do not present static data as live. Cite sources in the tool. Every choice needs a real cost. Do not let polish replace mechanical accuracy. Track only two or three core resources. Standout features: link shielding decisions to current space weather, replayability, and a short note on which datasets were used and why.

### 4. Create Health Monitoring Software for Astronauts on Space Missions
- URL: `.../health-monitoring-software/`
- Difficulty: Intermediate
- Tags: Human Exploration, Software, Space Exploration
- Summary: Build a visual tool that lets astronauts assess their own health on long missions and gives guidance on treatment and improvement. Hazards come from NASA's Human Research Program: radiation, isolation and confinement, distance from Earth, gravity fields and hostile closed environments. Candidate indicators are blood pressure, heart rate, blood sugar, body temperature, sleep and mood.
- Data resources: none named.
- Potential considerations: Support different environments (spacecraft, station, rover, surface base). Prompt astronauts to take measurements as part of mission routine. Features could include gamification, dynamic charts over time, and colour coding of indicator status. Do not assume astronauts know how to assess their health, so give simple instructions.

### 5. Dancing with the SARs
- URL: `.../dancing-with-the-sars/`
- Difficulty: Advanced, Intermediate
- Tags: Earth Science
- Summary: Build an interactive application using NASA-ISRO NISAR radar (SAR) data to track and visualize one or more types of surface change anywhere in the world. Examples are wetland loss, wildfires, earthquakes, farming activity and glacier movement. SAR works day and night and through clouds. Radar is most sensitive to surface roughness and moisture.
- Data resources named: NISAR (SAR) data, other-sensor SAR for frequency comparison, ancillary datasets (population, vegetation type).
- Potential considerations: Compare polarizations or frequencies. Build time series from several dates. Add ancillary context layers. Show human drivers (agriculture, industry). Strong storytelling through visualizations in any digital medium. Personal local context is welcome. Interactive elements such as image sliders and web maps.

### 6. Field Shift: Adapting Farms with NASA Data
- URL: `.../adapting-farms-with-nasa-data/`
- Difficulty: Advanced, Intermediate
- Tags: Arts & Multimedia, Earth Science, Software
- Summary: Build a decision-support tool combining NASA Earth observations, local soil information, crop characteristics and farmer priorities. It should help farmers explore crop-rotation strategies that improve soil health and resilience to heat, drought, rainfall change and extreme weather. SI is encouraged for turning the data into explainable insights.
- Data resources named: SMAP, ECOSTRESS, GPM, Landsat, MODIS, NASA POWER, GRACE/GRACE-FO, SRTM and other NASA elevation products, GEDI biomass, soil datasets.
- Potential considerations: Explain evapotranspiration, soil-moisture anomalies, NDVI, frost risk and water balance. Compare satellite and ground measurements. Compare SI and statistical approaches. Express model uncertainty and say where local expertise is needed. Harmonize datasets of different resolutions onto a common grid. Compare rotation scenarios on nutrient cycling, erosion, organic matter, pest cycles, water use, forage and market factors. Map rotations to farmer priorities. Explain nitrogen-fixing, cover, deep-rooting and drought-tolerant crops. Interactive maps of long-term trends and field-level vulnerabilities.

### 7. Flame in Freefall: SI-Powered Fire Safety Insights from Microgravity Combustion Data
- URL: `.../flame-in-freefall/`
- Difficulty: Advanced, Intermediate
- Tags: Space Exploration
- Summary: Build an interactive SI-powered dashboard that summarizes, ranks and interprets NASA microgravity combustion experiments (ISS and others) in the Physical Science Informatics (PSI) database. The goal is fire-safety insight for spacecraft, Moon bases and pressurized rovers. Lunar and Mars habitats are expected to use 30% or more oxygen at reduced pressure, which makes materials much more flammable. There is currently no partial-gravity combustion data in PSI.
- Data resources named: NASA PSI database; investigations BASS and BASS-II, Saffire-I/II/III, SAME and SAME-R, DAFT and DAFT-2, FLEX and FLEX-2, CFI and CFI-G, ACME (six experiments including BRE and E-FIELD Flames), SPICE, SLICE. Hardware: CIR, MDCA, MSG.
- Potential considerations: Audience is professors, students, researchers and aerospace engineers. Summarize objectives, fuels (shape and size), test conditions (oxygen concentration, airflow), variables, sensors and measurements. Compare investigations. Extract data that supports predictive flammability modelling. Note oxygen levels and the spacesuit-versus-habitat pressure context. Partial-gravity behaviour can be bounded by microgravity and normal-gravity results. The dashboard could also cover Earth applications (water-mist extinguishers, smoke detection, firefighter gear).

### 8. Harmonization of MODIS and VIIRS Hot Spots
- URL: `.../modis-and-viirs-hot-spots/`
- Difficulty: Advanced, Intermediate
- Tags: Earth Science, Software
- Summary: Build a web app that merges the more than 20-year MODIS and VIIRS active-fire hotspot records, which are not directly comparable, into a burning-activity calendar. Users pick an area of interest. They can see historical patterns, anomalies and critical periods. Target users are early-warning and emergency responders, scientists, land managers and firefighters.
- Data resources named: MODIS and VIIRS active fire hotspots (specific product list on the Resources tab, Nov 13).
- Potential considerations: Interactive map with a drawn area of interest or a lat/lon entry. Accessible colour ramps. A "burning calendar". Key indices: historical comparisons and anomalies against multi-year averages, number of fires detected and reported.

### 9. Identify Earth Locations that Analog the Permanent Moon Base Locations and Mars
- URL: `.../permanent-moon-base-locations-and-mars/`
- Difficulty: Advanced, Intermediate, Beginner/Youth
- Tags: Earth Science, Planets & Moons, Software, Space Exploration
- Summary: Use open Earth, Moon and Mars data to identify and characterize new terrestrial analogs for future bases and landing sites (deserts, polar regions, volcanic and arid terrain, caves). The goal is not to rediscover Atacama or Haughton or to find one place that merely looks similar. Present the result as a map, visualization or tool. Combine datasets and explain what is similar, how similar, and what differs.
- Data resources named: open Earth, Moon and Mars datasets from NASA and other agencies (terrain, temperature, geology, vegetation, water and ice, aerosols). Lists open Nov 13.
- Potential considerations: Think beyond visual similarity. Use several dimensions (geology and mineralogy, terrain and slope, temperature and illumination, water and ice, dust and wind, biology and habitability, caves and lava tubes, operational conditions). Do not require a full match. Look at processes as well as places. Look beyond famous sites (Antarctica, Atacama, Iceland, Hawaii). Cover below-surface environments. Combine datasets. Consider scale.

### 10. Interplanetary Survival Guide: Martian Map
- URL: `.../interplanetary-survival-guide-martian-map/`
- Difficulty: Intermediate
- Tags: Human Exploration, Mars, Planets & Moons, Software, Space Exploration
- Summary: Create a layered, integrated view of a Martian location or route that pulls data from several NASA missions. It should help a human explorer plan a "Marswalk" safely while doing science. Existing tools such as JPL Mars Trek and the USGS-NASA geologic mapping program are built for regional and global scales. This view targets a specific mission.
- Data resources named: NASA Mars mission data (topography, geology, surface imagery, current conditions); JPL Mars Trek and USGS-NASA Planetary Geologic Mapping as reference tools.
- Potential considerations: Format could be a spacecraft display, a tablet app or a helmet-visor HUD. It can be a static snapshot, interactive, or a map generator. Look at maps from missions to other destinations for inspiration. Choose a past, future or timeless data frame. Use surface imagery as point-of-view context. Consider rover support and astronauts working in pairs.

### 11. Lunar Landing Site Navigator
- URL: `.../lunar-landing-site-navigator/`
- Difficulty: Advanced, Intermediate
- Tags: Human Exploration, Moon, Software, Space Exploration
- Summary: Build an intuitive tool for comparing south-pole landing sites and dates by visualizing Sun and Earth positions against the local horizon. It should show power-generation potential and direct-to-Earth communication windows. The extreme lighting at the south pole and the CLPS program make this operationally relevant. Existing ephemeris tools work but are hard to use.
- Data resources named: JPL Horizons System (ephemeris), LOLA (Lunar Orbiter Laser Altimeter) topography, pre-computed ephemeris datasets.
- Potential considerations: Use ephemerides from Horizons or pre-computed data. Use LOLA to build realistic horizons. There is no atmosphere, so the horizon comes from terrain and the lunar limb. Show solar elevation prominently. Support quick scenario comparison across site-date pairs. Show time remaining before sunset or Earthset. Make the lunar-versus-terrestrial differences vivid. Explain why small location or date changes matter so much. Account for the real topography, libration and orbital variations. Handle steep slopes and deep craters at the pole. Help planners coordinate science and engineering and react to schedule changes.

### 12. Planet X and SPHEREx
- URL: `.../planet-x-and-spherex/`
- Difficulty: Advanced
- Tags: Astrophysics, Planets & Moons, Software
- Summary: Build a public web tool that displays sky images from the SPHEREx mission (all-sky every six months, 102 near-infrared bands, more than a billion objects, since 2025). Users should quickly see how the sky changes over time. Comets, asteroids, stars, brown dwarfs and even planets reveal themselves by moving between images. The project follows the tradition of citizen-science discoveries.
- Data resources named: SPHEREx images, Gaia and WISE as overlay data, and Astropy and Aladin (CDS) as tools. Exact SPHEREx access details come on the Resources tab.
- Potential considerations: New data arrives fast, so make the tool easy to update. Display the 102 bands creatively and co-add the six detector arrays for signal-to-noise. On-the-fly spectra are a bonus. Offer add and subtract of images from different times. Let users choose the field of view. Offer overlays from Gaia and WISE. Support coordinates in the URL for embedding. Display metadata such as contamination flags.

### 13. Space Mission Design Game
- URL: `.../space-mission-design-game/`
- Difficulty: Intermediate, Beginner/Youth
- Tags: Games
- Summary: Create an interactive game where players design, manage and simulate a complete space mission. They face constraints on budget, mass, power, communications, orbit and mission lifetime. Every decision should have consequences. The game should show why a mission succeeds or fails.
- Data resources: none named (the text lists no NASA data).
- Potential considerations: Make it accessible to different knowledge levels. Single-player and team play (up to 4 roles, such as planning, spacecraft design, science, communications and resources). Clear objectives and feedback. Re-runnable simulation. A scoring system for success, science value, efficiency and decisions. Difficulty levels and scenarios. Start by defining a mission objective. Give every major decision a consequence. Show there is no single perfect solution. Include realistic constraints. Spacecraft components and instruments with mass, power, cost and data attributes. The page text was truncated in my capture after the spacecraft and payload section.

### 14. The Earth Information Jukebox
- URL: `.../the-earth-information-jukebox/`
- Difficulty: Intermediate, Beginner/Youth
- Tags: Arts & Multimedia, Earth Science, Software
- Summary: Build an "Earth Jukebox" that pairs NASA Earth Information Center (EIC) visualization frames with sonifications generated in real time. The goal is accessibility for blind and low-vision users and a multi-sensory way to experience Earth data. It balances direct data translation against musicality and artistic expression.
- Data resources named: NASA EIC visualizations (frames from animated sequences).
- Potential considerations: Define sonification terms and extract frames from EIC animations. The submission can be a coded interactive UI or a curated gallery or playlist of generated audio. Prioritize UX and accessibility (screen-reader support, tactile feedback, descriptive text). Map colour, brightness, shapes and pixel density to pitch, rhythm, tempo, volume and instrumentation. Weigh strict translation against musical scales and harmonies. Computer vision, ML, synthesizers and DAWs are all fair game. Document the "why" and the algorithms. Convey emotional impact.

## 4. Top 4 shortlist

1. **Lunar Landing Site Navigator (27).** It is the most physics-heavy and the most visual. The core model is real: ephemeris (JPL Horizons or SPICE-style geometry), a LOLA elevation grid, horizon ray-casting without atmosphere, and solar elevation giving power and Earth visibility giving communication windows. It maps to well-defined published methods (lunar illumination and Earth-visibility studies). It fits CLPS and Artemis, which is current and operational. A 3D south-pole view with a time scrubber and a site/date comparison is a strong demo, and one person can build it because the data volume is manageable (a pole-region DEM tile plus ephemeris).

2. **Harmonization of MODIS and VIIRS Hot Spots (25).** The data is rich and the scientific problem is real: two sensors with different resolution and detection characteristics must be made comparable to produce a consistent 20+ year record. You can ground it in the literature on MODIS/VIIRS active-fire cross-sensor comparison, calibrate an overlap-period adjustment with an explicit error estimate, and compute anomalies against multi-year baselines. The calendar plus an area-of-interest map is easy to deploy and relevant to responders. Visuals are good but not 3D unless you add a globe.

3. **Earth System Trend Detective (24).** It has the strongest statistical core: deseasonalization, trend tests (for example Mann-Kendall with a Sen slope) and significance with uncertainty, across many NASA variables. It lets you cite methods papers and show regional sign reversals, which is the interesting science. Risk: it can look like a generic dashboard, so the demo needs three or four strong worked cases. It is the safest to deliver solo.

4. **Earth Analogs of Moon Base / Mars (23).** Pick this over the other 23s for the multi-dimensional similarity idea: terrain (slope, roughness), thermal, mineralogy and ice proxies from Earth datasets compared with LOLA, Mars and Moon products. You can ground it in the planetary-analog literature and rank sites with an explainable per-dimension similarity score. A 3D globe with analog heatmaps has good visual impact. It is harder than 1-3 because it spans three planetary data sources, so keep to two or three dimensions.

Close runners-up (all 23): Field Shift (rich data, many integrations, risk of a shallow model), Dancing with the SARs (NISAR processing is heavy for one person in 48h), Planet X and SPHEREx (great science but needs careful image-service work and, as of now, unknown data access).

### Avoid
- **The Earth Information Jukebox (12).** Sonification of existing visualizations. Little scientific content, no model, no papers. This is the example you flagged.
- **Abandoned but not Forgotten (13).** Storytelling from general knowledge, with no data analysis.
- **Space Mission Design Game (16) and Junior Astronaut Mission Trainer (16).** Education games with simplified trade-offs. They do not use NASA data in any depth, so they would feel like "pure visuals / general knowledge". Health Monitoring Software (16) has weak data and is a UI exercise.

## 5. Caveats
- Dataset resource lists and the judging and submission guides appear only on **Nov 13**, so re-check them then (in particular whether a deployed demo and a video are mandatory, and the rubric weights).
- My difficulty labels come from the list page. Scores are subjective.
- WebFetch could not be used (certificate error), and the browser tab sometimes redirected to 2025 pages when left idle, so I used a second tab.
