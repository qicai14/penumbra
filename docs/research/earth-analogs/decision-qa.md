# Decision Q&A (earth-analogs)

## Q: What is the single hardest scientific or computational problem in this challenge, and which sources give the method or equations to solve it? Answer in under 150 words.

Answer:
The single hardest computational problem in this challenge is **simulating 
lunar polar illumination and visibility** by constructing horizon databases 
from high-resolution DEMs, due to exponential processing costs over 
multi-temporal solar ephemerides [1, 2]. Scientifically, **quantifying 
cross-body terrain similarity** across mismatched spatial scales and incomplete
datasets presents a parallel challenge [3, 4].

Primary methods and equations are provided by:
- **Lunar Illumination & Shadow Height Error**: *"Analysis of Illumination 
Conditions in the Lunar South Polar Region..."* details Lambertian solar 
reflectance (\\(I = \rho(\cos i \cos \alpha + \sin i \sin \alpha \cos(\Phi - 
\beta))\\)) and elevation error (\\(\Delta h = \Delta d \tan \alpha + h_B - 
h_A\\)) [5, 6].
- **Terrain Similarity Scoring**: The *terrestrial-analog-finder* repository 
defines robust weighted distance (\\(D = \sqrt{\frac{\sum w_i d_i^2 + \sum w_i 
P^2}{\sum w_i}}\\)) and similarity score (\\(S = 100 e^{-D}\\)) [7].
- **Multi-Criteria Site Selection**: *"Evaluating potential landing sites for 
Artemis III..."* applies TOPSIS relative closeness (\\(R_i = \frac{d_i^-}{d_i^+
+ d_i^-}\\)) [8].

🌋 Would you like to explore how these terrain similarity algorithms perform 
when evaluating specific Earth analogs like Devon Island or Mauna Kea?

Resumed conversation: f3474ed4-1896-438a-99e8-3f10dd4bc289

## Q: Which NASA datasets are needed, roughly how large are they, and can they be preprocessed to under 1 GB for a static web app? Answer in under 150 words.

Answer:
The primary NASA and partner datasets required include:

- **Lunar Topography**: **LRO LOLA DEM (118m)** (~8 GB) [1, 2] and **5 m polar 
DTMs** [3, 4].
- **Martian Topography**: **MGS MOLA - MEX HRSC Blended DEM (200m)** (~11 GB) 
[5, 6] and **MRO CTX DTMs** (~20 m) [4].
- **Earth Reference**: **Copernicus DEM (GLO-30)** [4, 7].

**Preprocessing to < 1 GB**:
**Yes**, these multi-gigabyte datasets can easily be preprocessed for a static 
web app. By cropping targeted **12 km × 12 km regional windows** [8, 9], 
reprojecting them to a uniform **30 m grid** [9, 10], and pre-computing scalar 
terrain metrics (relief, slope distribution, roughness, hypsometry) [9, 11], 
the raw rasters compress into lightweight JSON records and hillshade tiles 
totaling **~250 MB** [12, 13].

🌐 Would you like to inspect how the offline Python pipeline extracts these 30 
m terrain metrics from cloud-optimized GeoTIFFs?

Resumed conversation: f3474ed4-1896-438a-99e8-3f10dd4bc289

## Q: What do existing tools and prior projects fail to do that users in this field need? Name the gap concretely. Answer in under 150 words.

Answer:
Existing tools and prior projects suffer from **siloed, single-dimension 
analysis**:

1. **Visualization Portals (e.g., NASA Solar System Treks)**: Provide image 
layers and browsing tools but lack automated, quantitative cross-body 
similarity ranking algorithms [1, 2].
2. **Terrain & Image-Based Finders**: Evaluate isolated dimensions — strictly 
elevation geometry (relief, slope, roughness) or 2D visual textures — while 
ignoring **thermal inertia, mineralogy, subsurface ice, and operational 
feasibility** [3-6].

**The Concrete Gap**: Existing tools fail to provide a **unified, 
multi-dimensional evaluation model** that synthesizes terrain geometry, 
thermophysical properties, mineralogy, subsurface ice, and operational 
constraints (such as illumination and communications) across mismatched spatial
scales, while carrying dataset uncertainties into the similarity score [4, 6, 
7].

🗺️ Would you like to see how we can define the weights and feature components 
for a multi-dimensional similarity vector across these axes?

Resumed conversation: f3474ed4-1896-438a-99e8-3f10dd4bc289

## Q: In a 48-hour hackathon, what would a convincing validation that the results are correct look like (reference values, known events, published numbers to reproduce)? Answer in under 150 words.

Answer:
A convincing 48-hour hackathon validation requires reproducing established 
literature benchmark values, ground-truth catalogs, and statistical stability 
metrics:

1. **Replicate Literature Analog Rankings**: Verify similarity scores against 
published baselines—such as Qaidam Basin–Mars feature distances (<0.5) [1], 
Atacama hyper-aridity matches [2], or Askja Caldera/Nevada Desert lunar terrain
distances (~0.022) [3].
2. **Benchmark Against Official Catalogs**: Confirm feature extraction against 
verified PDS catalogs, such as the **LROC Lunar Pit Atlas** (281 pits) [4] or 
**USGS Mars Global Cave Catalog (MGC3)** (>1,000 pits) [5].
3. **Reproduce Ground-Truth Mission Metrics**: Match SWIM project ice-table 
depths at spot-check sites (e.g., Viking 2 / Phoenix lander ice depth of ~11–12
cm) [6] and check DEM height uncertainties against LOLA benchmarks (RMS height 
error ~0.3–0.5 m) [7].
4. **Statistical Stability**: Demonstrate top-rank stability across candidate 
sites using **bootstrap resampling** [8].

🏆 Would you like to review a sample validation report template to present 
these benchmark comparisons to hackathon judges?

Resumed conversation: f3474ed4-1896-438a-99e8-3f10dd4bc289
