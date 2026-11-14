<script lang="ts">
  const limits = [
    "No limb darkening.",
    "Terrain smaller than a 20 m pixel is not in the DEM.",
    "The Earth margin and σ_z are assumed.",
    "No charge or discharge efficiency.",
    "Lander values are assumed unless marked cited.",
    "The IM-2 pit depth is assumed (U9): the ESA degraded-crater model, not a measured depth.",
    "The IM-2 pit view keeps the regional far horizon: inside a pit the panel sits H below regional ground, under 2 m of observer height at kilometres of distance.",
  ];

  // Dataset names: docs/SUBMISSION.md rubric item 4. Roles tie to the V1-V5 judges in scoreboard.json.
  // Each row links its original sources (URLs from docs/research/lunar-landing or the ledger's W rows).
  // The ids are SCIENCE-SPEC §6 evidence-ledger rows, which hold the exact quote taken from each source.
  const pgda90 = { label: "PGDA product 90", url: "https://pgda.gsfc.nasa.gov/products/90" };
  const datasets = [
    { name: "LOLA DEMs (20 m and 240 m)", role: "Terrain input to the horizon and shadow model", cite: "E133, E89, E135, E34b",
      sources: [pgda90, { label: "Barker et al. 2023, PSJ", url: "https://iopscience.iop.org/article/10.3847/PSJ/acf3e1" }] },
    { name: "PGDA AVGVISIB_85S_060M_201608", role: "Outside judge for V2 (visibility)", cite: "W1, W7",
      sources: [{ label: "AVGVISIB label (.LBL)", url: "https://imbrium.mit.edu/EXTRAS/ILLUMINATION/IMG/AVGVISIB_85S_060M_201608.LBL" }, { label: "PGDA product 69", url: "https://pgda.gsfc.nasa.gov/products/69" }] },
    { name: "NAIF kernels (DE440, lunar frame kernel)", role: "Ephemeris and lunar frame kernels: Sun and Earth positions in the model", cite: "E141, E40, E44",
      sources: [
        { label: "DE440/DE441 description (PDF)", url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440_and_de441.pdf" },
        { label: "NAIF tutorial 23 (PDF)", url: "https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/Tutorials/pdf/individual_docs/23_lunar-earth_pck-fk.pdf" },
        { label: "moon_pa_de440_200625.cmt", url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/pck/moon_pa_de440_200625.cmt" },
      ] },
    { name: "JPL Horizons", role: "Outside judge for V1 (az/el)", cite: "W8, E46",
      sources: [{ label: "Horizons API docs", url: "https://ssd-api.jpl.nasa.gov/doc/horizons.html" }, { label: "Horizons manual", url: "https://ssd.jpl.nasa.gov/horizons/manual.html" }] },
    { name: "LROC NAC (PGDA map-projected)", role: "Outside judge for V3 (illumination: shadow IoU)", cite: "E95, E94", sources: [pgda90] },
    { name: "IM-1 / IM-2 mission reports", role: "Outside judge for V4: published landing reports and the LROC crater image", cite: "E72a, E72b, E75a–c, E73, E74, W6",
      sources: [
        { label: "IM-1 NDL paper, NASA NTRS (PDF)", url: "https://ntrs.nasa.gov/api/citations/20250007036/downloads/aasimndl.pdf" },
        { label: "SpaceNews, IM-1", url: "https://spacenews.com/intuitive-machines-and-nasa-call-im-1-lunar-lander-a-success-as-mission-winds-down/" },
        { label: "CBS News, IM-2", url: "https://www.cbsnews.com/news/athena-moon-lander-tips-over-intuitive-machines-mission/" },
        { label: "Space.com, IM-2", url: "https://www.space.com/astronomy/moon/heres-why-the-private-athena-lunar-lander-toppled-over-on-the-moon" },
        { label: "LROC image 1408", url: "https://lroc.im-ldi.com/images/1408" },
      ] },
  ];
</script>

<section aria-labelledby="about-h">
  <h2 id="about-h">About</h2>

  <h3>Limitations</h3>
  <ul data-testid="limitations">
    {#each limits as l (l)}<li>{l}</li>{/each}
  </ul>

  <h3>Datasets</h3>
  <div class="scroll">
    <table data-testid="datasets">
      <caption>Every dataset, its role and its citation.</caption>
      <thead><tr><th scope="col">Dataset</th><th scope="col">Role</th><th scope="col">Citation</th></tr></thead>
      <tbody>
        {#each datasets as d (d.name)}
          <tr><th scope="row">{d.name}</th><td>{d.role}</td><td data-testid="citation">
            <ul class="sources">
              {#each d.sources as s (s.url)}<li><a href={s.url} target="_blank" rel="noopener noreferrer">{s.label}<span class="sr-only"> (opens in a new tab)</span></a></li>{/each}
            </ul>
            <span class="ids" data-testid="ids">SCIENCE-SPEC §6: {d.cite}</span>
          </td></tr>
        {/each}
      </tbody>
    </table>
  </div>
</section>

<style>
  section { margin-top: var(--section-gap); }
  h2 { margin: 0 0 var(--spacing-16); font: var(--font-weight-bold) var(--text-heading)/var(--leading-heading) var(--font-mono); }
  h3 { margin: var(--spacing-24) 0 var(--spacing-8); font: var(--font-weight-bold) var(--text-base)/var(--leading-base) var(--font-mono); }
  ul { margin: 0; padding-left: var(--spacing-24); }
  .scroll { overflow-x: auto; }
  table { width: 100%; border-collapse: collapse; font-size: var(--text-sm); line-height: var(--leading-sm); }
  caption { text-align: left; color: var(--color-ash); padding-bottom: var(--spacing-8); font-size: var(--text-caption); }
  th, td { border: 1px solid var(--color-iron); padding: var(--spacing-8); text-align: left; vertical-align: top; }
  thead th { color: var(--color-fog); font-weight: var(--font-weight-regular); background: var(--surface-carbon); }
  .sources { padding-left: var(--spacing-16); }
  .ids { display: block; margin-top: var(--spacing-8); color: var(--color-ash); font-size: var(--text-caption); }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }
  a { color: var(--color-paper); text-underline-offset: 2px; }
  a:visited { color: var(--color-paper); }
  a:focus-visible { outline: 2px solid var(--color-paper); outline-offset: 2px; }
</style>
