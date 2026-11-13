# Decision Q&A (fire-hotspots)

## Q: What is the single hardest scientific or computational problem in this challenge, and which sources give the method or equations to solve it? Answer in under 150 words.

Answer:
The single hardest problem in this challenge is **cross-sensor harmonization of
active fire counts and Fire Radiative Power (FRP)** across MODIS (~1–9.7 km²) 
and VIIRS (375 m) [1-3]. This discrepancy stems from incommensurable spatial 
footprints, off-nadir scan geometry ("bow-tie" area expansion up to 9.7×), 
detection sensitivity thresholds, and orbital drift [2-4].

Key sources providing methods and equations:
- **Pixel Geometry Equations**: *GMD (Sofiev & Kouznetsov, 2026)* [5] and *Fu 
et al. (2020)* [6] supply trigonometric equations for cross-track (\\(\Delta 
S\\)) and along-track (\\(\Delta T\\)) pixel area expansion as functions of 
scan angle \\(\theta\\).
- **VZA-Dependent FRP Correction**: *Remote Sensing (2026)* [7-9] details an 
empirical quadratic VZA correction model (\\(R(\theta) = a\theta^2 + b\theta + 
c\\)), area-based VIIRS cloud correction, and quality-prioritized fusion.
- **Analytical Observation Operators**: *GMD (Sofiev & Kouznetsov, 2026)* [10, 
11] formulates observation operators (\\(\Xi_{pix}\\)) mapping FRP across 
sensors and pixel scales.

🔥 Would you like a detailed breakdown comparing the quadratic VZA correction 
model against the analytical observation operators?

Resumed conversation: bcbb288b-ccad-44ae-9867-478b401166d0

## Q: Which NASA datasets are needed, roughly how large are they, and can they be preprocessed to under 1 GB for a static web app? Answer in under 150 words.

Answer:
The required NASA datasets are the **MODIS Collection 6.1** (Terra/Aqua, 1 km) 
[1, 2] and **VIIRS 375 m Collection 2** (Suomi-NPP, NOAA-20, NOAA-21) [1, 2] 
active fire products from NASA FIRMS [1, 3].

**Raw Dataset Size**: The complete multi-decadal global archive spans tens to 
hundreds of gigabytes, with VIIRS alone recording over 21.5 million detections 
per year globally [4, 5]. 

**Preprocessing under 1 GB**: **Yes.** By spatially and temporally aggregating 
raw point detections—such as gridding into 0.05°–0.25° cells, pentads, or H3 
index grids, or filtering by a specific region of interest—the historical 
dataset can easily be compressed into static formats (e.g., SQLite, Parquet, or
compressed GeoJSON/CSV) to **100–500 MB** [6-9]. This fits comfortably within 
client-side memory limits for a static web application [6, 7].

🔥 Would you like a sample data pipeline script to aggregate raw FIRMS CSVs 
into a lightweight SQLite or Parquet database?

Resumed conversation: bcbb288b-ccad-44ae-9867-478b401166d0

## Q: What do existing tools and prior projects fail to do that users in this field need? Name the gap concretely. Answer in under 150 words.

Answer:
Existing tools and platforms leave three concrete gaps:

1. **Naive Point Concatenation**: Platforms like NASA FIRMS expose raw thermal 
anomaly points [1, 2]. Simply merging records creates artificial trend jumps 
because one 1-km MODIS pixel circumscribes ~7 VIIRS 375 m pixels [3-5] while 
ignoring scan-geometry "bow-tie" footprint expansion [4, 6] and VZA-dependent 
FRP bias [7].
2. **Uncorrected Orbital Drift**: Existing systems fail to adjust for Terra and
Aqua drifting away from their traditional 10:30/13:30 overpasses, corrupting 
diurnal fire trend comparisons [1, 8-10].
3. **Lack of User-Centric Analytics**: Tools like GWIS and GFED focus on broad,
retrospective burned-area grids [1, 9, 11]. Users lack an interactive, 
**user-defined area-of-interest (AOI) burning activity calendar** that delivers
sensor-invariant anomaly baselines and full record provenance for emergency 
managers [1-3, 12].

🔥 Would you like to review an architectural outline for an interactive AOI 
burning activity calendar pipeline?

Resumed conversation: bcbb288b-ccad-44ae-9867-478b401166d0

## Q: In a 48-hour hackathon, what would a convincing validation that the results are correct look like (reference values, known events, published numbers to reproduce)? Answer in under 150 words.

Answer:
A convincing hackathon validation requires three multi-scale benchmarks:

1. **Benchmark Extreme Events**: Reproduce documented historical extremes, such
as the **2023 Canadian wildfire season** showing active fire counts ~6 SD above
the 2001–2022 baseline [1] and over 9× average carbon emissions [2].
2. **Published Cross-Sensor Parity**: Demonstrate that harmonized MODIS matches
VIIRS totals within +3.6% (reducing raw MODIS omission bias from −82%) [3], 
while daily continental FRP achieves a near 1:1 RMA regression (\\(R \ge 
0.98\\)) [4].
3. **High-Resolution Spatial Validation**: Validate detection reliability 
against 30 m **Landsat reference datasets (GABAM / LFTA)** [5], confirming 
global daytime commission errors stay below 1.2% [6] while significantly 
boosting Probability of Detection (POD) for low-FRP fires [7].

This combines peer-reviewed quantitative metrics, independent high-resolution 
ground truth, and real-world disaster reproduction.

🔥 Would you like a step-by-step validation protocol or test suite script for 
your hackathon submission?

Resumed conversation: bcbb288b-ccad-44ae-9867-478b401166d0
