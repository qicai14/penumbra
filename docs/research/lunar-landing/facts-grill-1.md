# Facts for grill-1 (gathered 2026-11-08; facts only, no decisions)

Legend: [V] verified from the source fetched this session; [U] unverified / secondary / inferred.

## 1. Space Apps 2026 rules on pre-event work

- [V] FAQ: "Teams are not allowed to begin working on the challenges prior to the hackathon." (challenge summaries Sep 17, statements Oct 28, resources Nov 13). https://www.spaceappschallenge.org/resources/participant-faqs/
- [V] Hackathon starts Nov 14, 2026 (or another day that week approved in writing); registration through Nov 15. https://www.spaceappschallenge.org/legal/ and FAQ above.
- [V] Legal page (Participant Agreement): non-NASA materials ("dataset or code that does not originate from NASA or other space agency partners") must be freely/publicly available without restriction, or OSI-licensed. Licenses of Solution Materials must be identified in submitted documentation; NASA will not post Solutions lacking license documentation; "You must cite every source". Original content is freely available or OSI-licensed; default Apache 2.0 if none specified. https://www.spaceappschallenge.org/legal/
- [V] FAQ: "Super Intelligence" (=AI; incl. code generation tools) may be used; submission must describe how SI tools were used. https://www.spaceappschallenge.org/resources/participant-faqs/
- [U] No explicit sentence found on "pre-existing code" or "code must be written during the event". The only operative rule is the no-pre-hackathon-work-on-challenges FAQ line. The Project Submission Guide (due Nov 13) and Judging guide are not yet published/visible, so unchecked. The site is an SPA; the legal page text was read via the in-app browser (JS-rendered). WebFetch not used.
- Nothing found distinguishing generic preprocessing/tooling from challenge work; interpretation is open (ask Local Lead / contact-us https://www.spaceappschallenge.org/contact-us/).

## 2. ANISE (nyx-space/anise)

- [V] crates.io latest default: 0.10.6 (2026-08-12); GitHub pre-releases 1.0.0-alpha (2026-10-04) and 1.0.0-beta (2026-10-19; workspace Cargo.toml version "1.0.0-beta"). https://github.com/nyx-space/anise/releases , https://crates.io/crates/anise
- [V] No wasm/JS bindings in repo: workspace members are anise, anise-cli, anise-gui, anise-py, anise/fuzz, anise-cpp. No "wasm" file in the 429-path repo tree; CI workflows: rust, python, cpp, gui, benchmarks, gemini-review (none wasm). GitHub issue search for "wasm" returns only #482 (conda-forge, irrelevant). https://github.com/nyx-space/anise
- [V] anise/Cargo.toml: hard (non-optional) deps include memmap2, rayon only under optional features, `sofars`, `hifitime`, `nalgebra`, `der`, `zerocopy`. Default features = ["metaload","analysis"]; metaload pulls ureq/rustls/platform-dirs/regex/serde_dhall; analysis pulls rayon/csv/hyperdual. Workspace dep uses `default-features = false`. Almanac::load uses std::fs::read. https://github.com/nyx-space/anise/blob/master/anise/Cargo.toml
- [U] Whether `--no-default-features` builds for wasm32-unknown-unknown is UNTESTED (no cargo/rustup on this machine). memmap2 being a mandatory dep is a possible blocker (unverified). `Almanac::load_from_bytes(BytesMut)` exists (src/almanac/mod.rs) so loading from in-memory bytes is possible API-wise.
- [V] Moon frames: constants MOON_ME=31001, MOON_PA=31000, MOON_PA_DE440=31008 (src/constants.rs). README recommends moon_fk_de440.epa + moon_pa_de440_200625.bpc + de440.bsp for accurate lunar work; data files moon_pa_de440_200625.bpc, moon_fk_de440.epa in repo `data/`; http://public-data.nyxspace.com/anise/v0.7/moon_fk_de440.epa . https://github.com/nyx-space/anise (README)
- API sketch from repo llms.txt (https://github.com/nyx-space/anise/blob/master/llms.txt) [V for calls shown]:
  ```rust
  let almanac = Almanac::default().with_spk(SPK::load("de440.bsp")?).with_bpc(BPC::load("moon_pa_de440_200625.bpc")?).load("moon_fk_de440.epa")?;
  let state = almanac.transform(SUN /*Frame*/, MOON_ME_frame, epoch, Aberration::NONE)?; // Sun in Moon-ME
  ```
  Almanac methods present (src/almanac/): transform, transform_to, state_of, unit_vector, sun_unit_vector, earth_sun_unit_vector, line_of_sight_obstructed, occultation, solar_eclipsing, local_solar_time, apparent_solar_time. Exact `transform` signature and whether `.epa` frame kernel loads via `.load()` for MOON_ME not confirmed [U]. Rust docs: https://docs.rs/anise . ctx7 CLI not run (context7 MCP needs auth; no time).

## 3. LOLA south-pole DEMs

PGDA Barker 2023 page: https://pgda.gsfc.nasa.gov/products/90 (data DOI 10.60903/gsfcpgda-lola-spole). Files are cloud-optimized GeoTIFF, south polar stereographic X/Y in metres, MOON_ME frame of DE421 (page text, [V]). Download base: `https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/`
- LDEM_83S_10MPP_ADJ.TIF: 10 m/px, 83-90S; Content-Length 5,116,709,568 B (4.8 GB) [V HEAD request]. Extent: Barker 2023 Table 1 gives stereographic range +/-216.4 km (new) vs +/-151.68 (old LDEM_85S_10M). Companion: _ERR 7.5G, _EFFRES 7.5G, _HILL 2.5G, LDSM_83S_10MPP_ADJ 7.1G, LDEC 415M.
- LDEM_80S_20MPP_ADJ.TIF: 20 m/px, 80-90S, +/-304 km; 2,696,082,051 B (2.6 GB) [V]. Companions: _ERR 3.7G, _EFFRES 3.8G, _HILL 1.3G, LDSM 3.6G, LDEC 1.3G. Also LPSR_80S_20MPP_ADJ.TIF (148M, PSR map) + .SHP/.CSV.
- Heights are in metres (LDEM TIF) relative to the 1737.4 km sphere [U for TIF offset; PDS IMG version uses radius with offset 1737400 m]. Table 1 source: https://iopscience.iop.org/article/10.3847/PSJ/acf3e1 (PDF fetched).
- The requested "LDEM_87S_5M" does not exist by that name. Actual PDS product: LDEM_875S_5M (87.5-90S, 5 m/px, 30336 x 30336, int16 LSB, SCALING_FACTOR 0.5, OFFSET 1737400 m, 1,840,545,792 B, from older/preliminary LOLA data) at https://imbrium.mit.edu/DATA/LOLA_GDR/POLAR/IMG/LDEM_875S_5M.IMG (+ .LBL) [V]. PDS Geosciences mirror of same family: http://pds-geosciences.wustl.edu/lro/lro-l-lola-3-rdr-v1/ (not fetched [U]). Other IMG polar products (80S_20M 1.85 GB, 85S_10M 1.84 GB, etc.) are in the same directory. Note the old 85S_10M is superseded by 83S_10MPP_ADJ.
- Barker 2021 adjusted 5 m site DEMs (Sites 1,4,6,7,11,20,20v2,23): file pattern Site01_final_adj_5mpp_surf.tif (+_ldec, _slp, _toterr, _slperr), 16x16 km each; listed at https://pgda.gsfc.nasa.gov/products/78 (direct URL path not captured [U]).
- Precomputed illumination (PGDA, Mazarico 2011 method; 2016 vintage, time span 2009-2013 label dates; avg over hourly steps for 18.6 yr per page) https://pgda.gsfc.nasa.gov/products/69 :
  - Average sun illumination: AVGVISIB_85S_060M_201608.TIF (34.0 MB; 60 m/px; IMG 51.2 MB; 5058x5058 int16, scale 0.00004), AVGVISIB_75S_120M_201608.TIF (83 MB), AVGVISIB_65S_240M_201608.TIF (60 MB). Base: `https://pgda.gsfc.nasa.gov/data/MoonIllumination/` (also _COG.TIF variants); PDS IMG at http://imbrium.mit.edu/EXTRAS/ILLUMINATION/IMG/ .
  - Earth visibility: AVGVISIB_85S_060M_201608_EARTH.TIF (24.9 MB), AVGVISIB_75S_120M_201608_EARTH.TIF. 65S Earth file name not captured [U].
  - Permanent shadow: LPSR_85S_060M_201608.TIF, LPSR_75S_120M_201608.TIF. Sky visibility: 65S 240 m only.
  - Caveat from page: Earth counted visible if any part of disc unobstructed. Doc: README_ILLUMINATION_2016.TXT in the EXTRAS/ILLUMINATION dir.

## 4. Published illumination validation numbers

- [V] Barker et al. 2023 (PSJ 4:183) is a PSR/topography/roughness paper; text has NO named-point illumination percentages. It states PSR map method (horizon-based, nested DEMs, sun at min declination, 720 time steps), total PSR area poleward of 80S ~20,800 km2 (>=1 px) and 506,349 PSRs. https://iopscience.iop.org/article/10.3847/PSJ/acf3e1
- [V] Barker et al. 2021 (PSS 203, 105119) Table 2 (Site 1, Jan 1 2024 - Jan 1 2026, 100 clones, 1 m above surface) is the closest to point targets: RoI centroids (X,Y km in polar stereographic): RoI1 (-11.5825,-12.4625), RoI2 (-11.4875,-12.5575), RoI3 (-11.3525,-12.5025), RoI4 (-11.2875,-12.1675), RoI5 (-11.1025,-11.9425), RoI6 (-10.8025,-11.7375), RoI7 (-10.2275,-11.5525). 1st-percentile avg illumination at z=1 m, case A: 68.03, 70.32, 67.51, 69.61, 70.38, 71.15, 67.36 %. Case C (best pixel per hour): 85.84, 85.75, 77.31, 86.81, 84.47, 86.74, 83.44 %. Text: at 5 m, 1st pct avg illumination 77% (RoI 3) to 87% (RoI 4, 6). Source PDF (NTRS): https://ntrs.nasa.gov/api/citations/20205009660/downloads/Mike%20Barker_%20Paper_%20Improved%20Lola.pdf . These are regional statistics over 2024-2026, not lat/lon points.
- [U] Mazarico et al. 2011 (Icarus 211:1066) full text/table NOT obtainable (paywalled; not on NTRS as PDF; Unpaywall: closed). Search snippets (secondary, unverified): site near Shackleton rim sunlit continuously ~240 days, longest total darkness ~1.5 days; Glaser et al. 2018 (PSS) 2 m-above-surface values 77.1-88.0 % (max on ridge Shackleton-de Gerlache), surface 69.5-82.9 %. ADS: https://ui.adsabs.harvard.edu/abs/2011Icar..211.1066M ; Glaser 2018 doi:10.1016/j.pss.2017.07.006. Exact lat/lon tables must be pulled by hand from the paper.

## 5. LROC NAC for shadow validation

- [V] PGDA Barker 2023 page lists NAC images map-projected onto the 20 m LDEM plus simulated images: `M*E_20MPP.TIF` (I/F, map-projected NAC; names like M1188426614LE_20MPP.TIF / ...RE_20MPP.TIF, 1.4-3.1 MB) and `M*_20MPP_MODEL.TIF` (simulated, W/m^2; e.g. M1188426614_20MPP_MODEL.TIF 30 MB, M1311409938_20MPP_MODEL.TIF 69 MB, M1312308500_20MPP_MODEL.TIF 85 MB, M139356343_20MPP_MODEL.TIF 73 MB). Path presumably under https://pgda.gsfc.nasa.gov/data/LOLA_20mpp/ [U]. https://pgda.gsfc.nasa.gov/products/90 . Paper figures A12 etc. show such pairs (M139729686, M1188426614) with sub-solar longitude noted.
- [V] LROC NAC ROI controlled mosaics (RDR, PDS LROLRC_2001): NAC_ROI_SHKLTNPKLO1_P888S1247 ("Peak near Shackleton", polar stereographic, centre 88.8S 124.7E, 1 m/px, 16-bit GeoTIFF, 113 NACs/251 segments, 4 sub-solar-longitude bins, abs. accuracy 5-20 m). README: https://pds.mcp.nasa.gov/data/store/img/lunar_reconnaissance_orbiter/pds4/lroc/lro-l-lroc-5-rdr/LROLRC_2001/DATA/BDR/NAC_ROI/SHKLTNPKLO1/NAC_ROI_SHKLTNPKLO1_P888S1247_README.TXT . Naming convention (in same README): NAC_ROI_<site 8 chars><LO|HI><v>_<A=E|P><lat><N|S><lon>. Mosaics mask by lighting bins, so single acquisition time is not one value (per-segment time via the individual NAC EDR/RDR). Other 12 Artemis III region mosaic IDs not enumerated [U]; Shackleton-de Gerlache ridge curated downloads: https://lroc.im-ldi.com/images/downloads and https://lroc.im-ldi.com/images/1247 .
- Image time/geometry: individual NAC EDR/CDR labels carry START_TIME, sub-solar/incidence/emission; image ID M<digits> is the spacecraft clock count (M1463440322L = IM-1 frame, taken 2024-02-24 18:57 UTC per NASA). Label-field names and the PDS cumulative index path (LROLRC_0xxx) not verified this session [U]. QuickMap (https://quickmap.lroc.asu.edu) lists NAC frames with time.

## 6. IM-1 Odysseus and IM-2 Athena

- IM-1 [V NASA]: landed 2024-02-22 6:24 p.m. EST (23:24 UTC per NASA; Wikipedia/aggregators say 23:23:53 UTC [U]); 80.13 S, 1.44 E, elevation 2,579 m, inside degraded ~1 km crater, ~12 deg local slope; tipped on landing (~30 deg per secondary sources [U]); LRO LROC NAC frame M1463440322L on 2024-02-24 1:57 p.m. EST (= 18:57 UTC) at ~90 km altitude. Operated ~1 week, last contact 2024-02-29 [U secondary]. https://www.nasa.gov/missions/lro/nasas-lro-images-intuitive-machines-odysseus-lander/ ; https://en.wikipedia.org/wiki/IM-1
- IM-2 [U, secondary only]: landed 2025-03-06 17:28:50 UTC on Mons Mouton at 84.7906 S, 29.1957 E (Wikipedia, citing LROC); ended on its side in ~20 m crater, ~250 m (IM) / >400 m (NASA) off target; altimeter failure; last contact 2025-03-07 06:15 UTC; power insufficient then depleted. https://en.wikipedia.org/wiki/IM-2 ; LROC page https://lroc.im-ldi.com/images/1401 (pre-landing) and https://lroc.im-ldi.com/images/1408 (oblique Athena). Primary NASA/IM press release and LROC post-landing coordinate page were not fetched [U].
- No published sun elevation or tilt-vs-power numbers found for either mission in the sources read [U].

## Fact check (Nov 10, Haiku)

### 1. IM-2 Athena landing coordinates and touchdown time

- Value: 84.7906 S, 29.1957 E (LROC page). Touchdown date 06 March 2025 (LROC page). Touchdown time: not found in a primary source.
- Quote (LROC): "ended up on its side within a 20-meter diameter crater" ; "reached the surface of the Moon on 06 March 2025."
- URL: https://lroc.im-ldi.com/images/1408 (LROC Featured Image, "IM-2 Athena Seen Obliquely"; hosted on lroc.im-ldi.com, not lroc.asu.edu)
- Tag: [V] for coordinates and date; [U] for time.
- Caveat: the coordinates came through the WebFetch summary, not a raw copy of the page. Re-open the page and confirm the digits by eye. Touchdown time: Wikipedia gives 17:28:50 UTC and trackers say ~17:28 UTC, but neither is primary. The Intuitive Machines investor release returned HTTP 403 to both WebFetch and curl. The NASA blog post was not located. Where it would be: the IM investor release or the NASA LRO/IM-2 blog post.

### 2. PGDA AVGVISIB_85S_060M_201608

- Value: time span 18.6 years, hourly time step (matches product 69 caption for the 85S-90S, 60 m/px illumination and Earth-visibility maps). Observer height: not stated.
- Quote (product 69): "averaged over hourly timesteps spanning 18.6 years, for the south polar region." ; "height above the surface (for potential rovers or habitats)."
- URL: https://pgda.gsfc.nasa.gov/products/69
- Tag: time span and step [V]; observer height [U].
- Caveat: the page never names AVGVISIB_85S_060M_201608. The caption ties the 18.6-year and hourly values to the 85S-90S, 60 m/px maps but does not explicitly tie them to this file. Observer height is absent from the page. Where it would be: the .LBL label of the AVGVISIB file on the PDS LOLA node (not fetched).

### 3. LROC NAC PDS label fields (time and projection)

- Value: START_TIME = UTC time at start of image acquisition. MAP_PROJECTION_TYPE = projection characteristic of a map. Example NAC label value: PRODUCT_ID M102658937LE, START_TIME = 2009-07-19T16:07:50.004.
- Quote (archive SIS): "START_TIME UTC time when first image line of target was acquired." ; "MAP_PROJECTION_TYPE Identifies the projection characteristic of a given map."
- URLs: https://pds.mcp.nasa.gov/data/store/img/lunar_reconnaissance_orbiter/pds4/lroc/lro-l-lroc-3-cdr/LROLRC_1066A/DOCUMENT/ARCHSIS.PDF (archive SIS, v1.14 volume) ; https://pds.mcp.nasa.gov/data/store/img/lunar_reconnaissance_orbiter/pds4/lroc/lro-l-lroc-2-edr/LROLRC_0044B/DOCUMENT/LROCSIS.PDF (EDR SIS, example label)
- Quote (EDR SIS): "The UTC time and date at the start of the image acquisition."
- Tag: [V] for START_TIME and the NAC example; [V] for the MAP_PROJECTION_TYPE definition only (no example value fetched).
- PGDA map-projected products (M*E_20MPP.TIF, M*_20MPP_MODEL.TIF): projection and units are stated on https://pgda.gsfc.nasa.gov/products/90. Quote: "All TIF files are cloud-optimized GeoTiffs (COGs) with south polar stereographic X/Y coordinates in meters". Timestamp: [U], the page says nothing about acquisition time. Where it would be: the PDS label of each product, or the LROC observation record for the source frame M1188426614.
- Caveat: the EDR SIS sample label is for an EDR, not a map product. The M1188426614E_20MPP.TIF filename did not appear in the PGDA listing; only _MODEL.TIF, LE and RE variants did.

### 4. IM-1 Odysseus and IM-2 Athena power figures

- Value: Athena battery 25 Ah on a 28 VDC bus (Nova-C generic spec). Solar array: 200 W combined maximum on the lunar surface (generic spec). Wh: not stated. Post-landing ops: about 13 hours (Intuitive Machines statement via news). IM-1 figures: not found.
- Quote (eoPortal): "A 25 amp-hour battery supplies power to a 28 VDC bus for use by the spacecraft when power generation lags consumption."
- URL: https://www.eoportal.org/satellite-missions/intuitive-machines
- Tag: [U] for all Athena figures. eoPortal is a secondary catalog and the spec is Nova-C generic, not Athena-specific.
- Caveat: no primary source for Athena power was reachable. The IM investor release (commissioning) returned 403. NTRS search returned only general south-pole power papers (e.g. https://ntrs.nasa.gov/citations/20090015377), not IM-2. Where it would be: the Intuitive Machines IM-2 mission press kit or a NASA PRIME-1 / IM-2 report.

### Summary

1. IM-2 coordinates: [V] (LROC page, 84.7906 S, 29.1957 E, via fetch summary; re-check by eye).
2. IM-2 touchdown time: [U] (17:28:50 UTC is Wikipedia only; primary IM release returned 403).
3. PGDA AVGVISIB time span and step: [V] (hourly, 18.6 years, product 69 caption); observer height: [U].
4. NAC START_TIME and MAP_PROJECTION_TYPE definitions: [V] (archive SIS and EDR SIS); PGDA map-product timestamp: [U].
5. IM-1 Odysseus power and ops duration: [U] (no primary source found).
6. IM-2 Athena battery 25 Ah and 200 W array: [U] (eoPortal, generic Nova-C spec; Wh and Athena-specific figures missing).
Caveats: the LROC coordinate page is on lroc.im-ldi.com, not lroc.asu.edu; several primary pages (IM release, NASA blog) returned 403 or redirect loops; PGDA observer height and map timestamps are absent from the documentation I could reach.
