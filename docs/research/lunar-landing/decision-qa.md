# Decision Q&A (lunar-landing)

## Q: What is the single hardest scientific or computational problem in this challenge, and which sources give the method or equations to solve it? Answer in under 150 words.

Answer:
The single hardest computational problem in this challenge is **calculating 
local terrain horizon profiles and time-dependent Sun/Earth visibility** over 
high-resolution Digital Elevation Models (DEMs) [1-4]. Because solar elevation 
angles near the lunar south pole are extremely low (~1–2°), minor topographic 
obstacles cast kilometer-long shadows, making brute-force 3D ray tracing 
computationally prohibitive across long time horizons [1, 2, 5].

Key sources providing methods and equations:
- **Mazarico et al. (2011)** [4] and **Barker et al. (2021/2023)** [6, 7]: 
Detail the **horizon-map precomputation method**, storing horizon elevation 
angles across discrete azimuths to reduce illumination queries to fast spatial 
lookups.
- **HAZAKIKO (2026)** [5, 8]: Formulates ray-marching algorithms, shadow 
geometry (\\(L_{\text{shadow}} \approx 38h\\) at \\(1.5^\circ\\) solar 
elevation), and horizon lookup tables.
- **JPL DE440 / SPICE Toolkit (Acton 1996; Park et al. 2021)** [1, 9, 10]: 
Provides the ephemeris vector equations (\\(LT+S\\)) and body-fixed frame 
transformations (`MOON_ME`/`MOON_PA`) needed to determine exact solar and 
terrestrial coordinates relative to local horizons.

🌕 **Would you like me to create an interactive tool or Python script to 
compute horizon profiles and solar visibility windows for candidate Artemis 
landing sites?**

Resumed conversation: 289a358e-7bbd-4559-bc11-2ddb949aa221

## Q: Which NASA datasets are needed, roughly how large are they, and can they be preprocessed to under 1 GB for a static web app? Answer in under 150 words.

Answer:
The primary NASA datasets required are:

1. **LRO LOLA Digital Elevation Models (DEMs)** (e.g., `LDEM_80S_80M`, `20M`, 
`10M` from NASA PDS) [1-3]. Raw polar DEM files range from ~570 MB for regional
tiles up to 2.6–4.8 GB for uncompressed high-resolution grids [4-6].
2. **NASA/JPL SPICE Ephemeris Kernels** (DE440 ephemeris and `MOON_ME` frames 
from NAIF), totaling ~120–570 MB [4, 7, 8].

**Yes, they can easily be preprocessed under 1 GB.**

By precomputing 360° terrain horizon profiles (azimuth elevation angles) for 
target landing sites, block-averaging DEM grids (e.g., to 400 m), and 
interpolating hourly solar/terrestrial vectors offline [4, 9], the client 
runtime bundle shrinks to just **~10.5 MB** (or under 100 MB for full regional 
maps) [4]. This enables fast, serverless performance in a static web app [4].

🚀 **Would you like me to write a Python script demonstrating how to compress a
LOLA DEM into precomputed horizon lookup tables for web deployment?**

Resumed conversation: 289a358e-7bbd-4559-bc11-2ddb949aa221

## Q: What do existing tools and prior projects fail to do that users in this field need? Name the gap concretely. Answer in under 150 words.

Answer:
Existing tools such as NASA’s Moon Trek, LROC QuickMap, and JPL Horizons are 
designed for detailed, single-layer spatial analysis or raw ephemeris 
extraction, creating a barrier to rapid scenario comparison [1].

**The concrete gap is the lack of an integrated workflow for dynamic, 
side-by-side site and date trade-off evaluations** [1, 2]. Specifically, 
existing projects fail to:

1. **Calculate joint Sun and Earth visibility** against local 3D terrain 
horizons to assess solar power potential and Direct-To-Earth (DTE) 
communication windows simultaneously [1, 3, 4].
2. **Provide flexible temporal comparison** showing how operational windows 
shift across changing launch dates or candidate landing locations without 
navigating fragmented interfaces [1, 5].
3. **Translate complex ephemeris and DEM data into intuitive, real-time visual 
metrics** accessible to both mission planners and non-specialists [1, 5].

🛠️ **Would you like me to outline a software architecture or data pipeline to 
bridge this gap for your challenge submission?**

Resumed conversation: 289a358e-7bbd-4559-bc11-2ddb949aa221

## Q: In a 48-hour hackathon, what would a convincing validation that the results are correct look like (reference values, known events, published numbers to reproduce)? Answer in under 150 words.

Answer:
A convincing 48-hour hackathon validation relies on three quantitative 
benchmarks:

1. **Ephemeris Verification**: Cross-check computed Sun and Earth 
azimuth/elevation against **JPL Horizons API** query outputs (DE440/DE441 
ephemerides) [1, 2]. Achieving directional agreement within **<0.01′** 
(<0.006°) confirms correct `MOON_ME` frame transformations [1, 2].
2. **Published Baseline Reproduction**: Match established literature values 
from **Mazarico et al. (2011)** and **Barker et al. (2023)**—specifically 
reproducing **~82–92% annual illumination** at Shackleton crater rim points and
a total PSR area of **~21,000 km²** poleward of 80°S [3-5].
3. **Historical Event Back-Testing**: Model lighting conditions for real lunar 
missions, such as **IM-1 Odysseus** (Feb 2024 touchdown) [6], **IM-2 Athena** 
(March 2025 low-angle crater shadow tipping) [7], or **Firefly Blue Ghost** 
(March 2025 sunrise at Mare Crisium) [8].

🛠️ **Would you like me to generate a Python test script that queries the JPL 
Horizons API to automatically validate your position vectors?**

Resumed conversation: 289a358e-7bbd-4559-bc11-2ddb949aa221
