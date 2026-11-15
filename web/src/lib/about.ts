// About view content. Every limitation carries the id of the issue, scoreboard row or SCIENCE-SPEC row it traces to.
// Wording of V2, V2b and V3 follows the #29 comment; #48 follows docs/DATA-NOTES.md T-1a-4.
export type Limit = { id: string; text: string };

export const limits: Limit[] = [
  { id: "V2b", text: "V2b fails, and we say so. It is one-sided: it catches a model that is too dark, not one too bright. Barker Site 1 passes 7 of 7, but Mazarico's SP-01 total night of 2.75 d lies 0.21 d below our shortest pixel (2.96 d). That is inside his 6 h step (about ±0.25 d) and in the direction expected from our finer 20 m near field, and nothing was tuned, but the agreed rule is the strict range, so it is reported as a fail." },
  { id: "V3", text: "V3 was not run: no LROC NAC images were downloaded, so the shadow IoU check has no input. Future work." },
  { id: "#75", text: "V2 says little about the brightest rims: only 123 sampled pixels above 60 % in the two original tiles the V2 scoreboard covers (118 in 60–80 %, 5 in 80–100 %). The pass rests on the 0–60 % bins. This is the terrain, not our sampling: AVGVISIB peaks at 88.44 % and about 19 pixels on the whole cap exceed 80 %. The V2 scoreboard covers those two original tiles only. Six of the new regions passed a V2-rule grid comparison against AVGVISIB (r 0.976–0.988 at a 2 m mast, 0.946–0.981 at 0 m; V2 notes); Peak near Cabeus B and Nobile Rim 2 lie north of AVGVISIB's 85°S coverage and were not checked." },
  { id: "#26", text: "Overlapping tiles (Mons Mouton Plateau and Mons Mouton, Nobile Rim 1 and Nobile Rim 2) may give slightly different numbers for the same point, because each tile has its own height scale and edges. The app uses the selected region's tile." },
  { id: "#48", text: "Horizon accuracy is checked against our own float 20 m DEM, not an independent source. The shipped three-tier horizon (20 m to 7 km, 80 m to 31 km, 240 m beyond) is within 0.0382° of it at five test points (rule: 0.05°, obstacles 60 m or more away). Closer than 60 m the largest difference is 0.1177°. The IM-2 highest horizon moved from 4.0427° to 4.0617° with the middle tier." },
  { id: "E26", text: "No limb darkening." },
  { id: "P6", text: "Terrain smaller than a 20 m pixel is not in the DEM." },
  { id: "§1d", text: "The Earth contact margin (2°) and the DEM height error σ_z (0.5 m) are assumed, not cited (SCIENCE-SPEC §1d, §1f)." },
  { id: "U10", text: "No charge or discharge efficiency. Lander battery and array values are assumed unless marked cited." },
  { id: "U9", text: "The IM-2 pit depth is assumed (U9): the ESA degraded-crater model, not a measured depth." },
  { id: "V4", text: "The IM-2 pit view keeps the regional far horizon: inside a pit the panel sits H below regional ground, under 2 m of observer height at kilometres of distance." },
];

// D3 (30 s demo) and D4 (5 min video). Empty = not shipped yet; a slot without a URL renders nothing.
export const VIDEO_DEMO_URL = "";
export const VIDEO_FULL_URL = "";

export const videoSlots = [
  { id: "d3", label: "Watch the 30 s demo", url: VIDEO_DEMO_URL },
  { id: "d4", label: "Watch the 5 min video", url: VIDEO_FULL_URL },
];

export const shownVideos = (slots: { id: string; label: string; url: string }[]) =>
  slots.filter((s) => s.url.trim() !== "");
