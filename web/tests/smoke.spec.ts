import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Runs against `vite preview` locally and the deployed URL in CI (BASE_URL).
// Site shows the inputs, not a "Computed for" line; Planner keeps it, so read it there and come back.
async function expectComputed(page: import("@playwright/test").Page, text: string) {
  const nav = page.getByRole("navigation", { name: "View" });
  await nav.getByRole("button", { name: "Planner", exact: true }).click();
  await expect(page.getByTestId("computed-for")).toContainText(text);
  await nav.getByRole("button", { name: "Site", exact: true }).click();
}

test("panorama loads the fixture (api_version checked against WASM) and shows the readout", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("readout")).toBeVisible();
  await expect(page.getByLabel(/^Horizon panorama/)).toBeVisible();
  // The horizon is the real DEM (T07 #8), so the panorama badge is off.
  await expect(page.getByTestId("fake-badge")).toHaveCount(0);
  // Survival is the WASM model (T09 #10), so its badges are off too.
  await expect(page.getByTestId("fake-badge-survival")).toHaveCount(0);
  await expect(page.getByTestId("fake-badge-timelines")).toHaveCount(0);
  // The panorama runs on the landing date (#13), so the old sample-dates note is gone.
  await expect(page.getByTestId("track-dates")).toHaveCount(0);
});

test("keyboard focus reads the horizon elevation and obstacle distance", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel(/^Horizon panorama/).focus();
  await page.keyboard.press("ArrowRight");
  await expect(page.getByTestId("readout")).toHaveText(/Az [\d.]+° · horizon [+−][\d.]+° · obstacle [\d.]+ k?m/);
});

// Native values, IM-2, 2026-11-14T00:00:00Z (T02 brief row 14).
test("tracer shows the native Sun and Earth az/el, no FIXTURE badge", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Validation", exact: true }).click();
  await expect(page.getByTestId("tracer").locator("summary")).toContainText(/^Live V1 check: Sun az 97\.5° \/ el 0\.88°$/);
  await page.getByTestId("tracer").locator("summary").click(); // the tracer is one collapsed line under the scoreboard
  const want = { "sun-az": 97.531671689975, "sun-el": 0.878280821, "earth-az": 329.676179426621, "earth-el": -0.153898865224 };
  for (const [id, v] of Object.entries(want)) {
    await expect(page.getByTestId(id)).toBeVisible();
    const got = Number(await page.getByTestId(id).getAttribute("data-value"));
    expect(Math.abs(got - v), id).toBeLessThan(1e-9);
  }
  await expect(page.getByTestId("dip")).toContainText("Horizons check point");
  await expect(page.getByText("FIXTURE")).toHaveCount(0);
});

const cursorX = (page: import("@playwright/test").Page) =>
  page.locator('[data-testid="timelines"] .u-cursor-x').first().evaluate((el) => new DOMMatrix(getComputedStyle(el).transform).m41);

test("timelines render labelled series and follow the panorama time slider", async ({ page }) => {
  await page.goto("/");
  const tl = page.getByTestId("timelines");
  await expect(tl.locator("canvas")).toHaveCount(3);
  // the uPlot live legend is off; the series names and the threshold are in the .now readout under the charts
  const now = page.getByTestId("timelines-now");
  await expect(now).toContainText("illumination");
  await expect(now).toContainText("visibility");
  await expect(now).toContainText("2° (assumed)");
  const slider = page.getByLabel("Time within the span");
  const max = Number(await slider.getAttribute("max"));
  await slider.fill(String(Math.round(max * 0.25)));
  const a = await cursorX(page);
  const textA = await page.getByTestId("timelines-now").textContent();
  await slider.fill(String(Math.round(max * 0.75)));
  await expect.poll(() => cursorX(page)).toBeGreaterThan(a + 20);
  expect(await page.getByTestId("timelines-now").textContent()).not.toBe(textA);
  // the uPlot legend is off, so a key line names every series on screen
  for (const n of ["Illumination", "Visibility", "Earth margin", "Earth contact", "Battery", "Lethal charge"]) await expect(page.getByTestId("series-key")).toContainText(n);
});

test("zoom buttons: + narrows, - widens, Reset shows the full span, and the time cursor stays put (T81 #81)", async ({ page }) => {
  await page.goto("/");
  const zoom = page.getByTestId("zoom");
  await expect(page.getByTestId("timelines").locator("canvas")).toHaveCount(3);
  const span = async () => Number(await zoom.getAttribute("data-span"));
  const full = await span();
  await page.getByTestId("zoom-in").click();
  const half = await span();
  expect(half).toBeCloseTo(full / 2, -1);
  await page.getByTestId("zoom-in").click();
  const quarter = await span();
  expect(quarter).toBeLessThan(half);
  await page.getByTestId("zoom-out").click();
  expect(await span()).toBeGreaterThan(quarter);
  await page.getByTestId("zoom-reset").click();
  expect(await span()).toBe(full);
  await expect(page.getByTestId("timelines-now")).toBeVisible();
});

test("hovering the chart moves the panorama time", async ({ page }) => {
  await page.goto("/");
  const before = await page.getByTestId("now").textContent();
  const over = page.getByTestId("timelines").locator(".u-over").first();
  await over.scrollIntoViewIfNeeded();
  const box = (await over.boundingBox())!;
  await page.mouse.move(box.x + box.width * 0.6, box.y + box.height / 2);
  await expect.poll(() => page.getByTestId("now").textContent()).not.toBe(before);
});

test("scoreboard lists V1-V5 plus V2b and shows 'not run' for checks that have not run", async ({ page, request }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Validation", exact: true }).click();
  const table = page.getByRole("table", { name: /One row per check/ });
  await expect(table.getByTestId("score-row")).toHaveCount(6);
  const rows = (await (await request.get("scoreboard.json")).json()) as { pass: boolean | null }[];
  const notRun = rows.filter((r) => r.pass === null).length;
  await expect(table.getByRole("cell", { name: "not run" })).toHaveCount(notRun);
  // limitations explain the checks but live on About
  await page.getByRole("button", { name: "About", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Limitations" })).toBeVisible();
});

// T13 (#14): every dataset carries its role and an evidence-ledger citation, none left pending (rubric 4 and 10).
// Each citation links to its original source, and every link is one the research lists or a ledger W row already name.
test("every dataset row has a role, SCIENCE-SPEC §6 ids and links to the original sources", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "About", exact: true }).click();
  const rows = page.getByTestId("datasets").locator("tbody tr");
  await expect(rows).toHaveCount(6);
  await expect(page.getByTestId("datasets")).not.toContainText("pending");
  // Every cited id must be a real row of the ledger, so a typo such as E999 fails.
  const ledger = readFileSync(fileURLToPath(new URL("../../docs/SCIENCE-SPEC.md", import.meta.url)), "utf8");
  const known = new Set([...ledger.matchAll(/^\| ([EW]\d+[a-z]?) \|/gm)].map((m) => m[1]));
  const research = (f: string) => readFileSync(fileURLToPath(new URL(`../../docs/research/lunar-landing/${f}`, import.meta.url)), "utf8");
  const bare = (u: string) => u.replace(/^https?:\/\//, ""); // the W1 row says http://, the page links https://
  const listed = bare([ledger, research("sources.txt"), research("index-addendum.txt")].join("\n"));
  const cells = await page.getByTestId("citation").all();
  expect(cells).toHaveLength(6);
  for (const cell of cells) {
    const links = await cell.locator("a").all();
    expect(links.length).toBeGreaterThan(0);
    for (const a of links) {
      const href = (await a.getAttribute("href")) ?? "";
      expect(href, "absolute https link").toMatch(/^https:\/\//);
      expect(href, "not the project's own repo").not.toContain("github.com");
      expect(listed.includes(bare(href)), `${href} in sources.txt, index-addendum.txt or a §6 W row`).toBe(true);
      await expect(a).toHaveAttribute("target", "_blank");
      await expect(a).toHaveAttribute("rel", "noopener noreferrer");
    }
    const ids = ((await cell.getByTestId("ids").textContent()) ?? "").replace(/^.*§6:\s*/, "").split(/,\s*/);
    expect(ids.length).toBeGreaterThan(0);
    for (const id of ids) {
      const range = id.match(/^([EW]\d+)([a-z])–([a-z])$/); // "E75a–c" stands for E75a, E75b, E75c
      const expanded = range ? [...Array(range[3].charCodeAt(0) - range[2].charCodeAt(0) + 1)].map((_, i) => range[1] + String.fromCharCode(range[2].charCodeAt(0) + i)) : [id];
      for (const one of expanded) expect(known.has(one), `${one} in SCIENCE-SPEC §6`).toBe(true);
    }
  }
  await page.getByRole("button", { name: "Validation", exact: true }).click(); // the scoreboard table lives on Validation
  await expect(page.getByRole("cell", { name: /^illumination: shadow IoU/ })).toBeVisible(); // V3 names its metric (P1)
});

test("picker refuses an edge click and an out-of-table date, and recomputes on a good change", async ({ page }) => {
  await page.goto("/");
  const refusal = page.getByTestId("refusal");
  const canvas = page.getByLabel(/^Greyscale height image/); // by label: the panorama canvas can render before the picker's
  await expect(canvas).toBeVisible();
  const box = (await canvas.boundingBox())!;
  await canvas.click({ position: { x: 4, y: box.height / 2 } }); // inside the 7 km border
  await expect(refusal).toContainText("7 km");

  await page.getByTestId("date").fill("2031-03-01");
  await page.getByTestId("date").blur();
  await expect(refusal).toContainText("ephemeris");

  await canvas.click({ position: { x: box.width * 0.4, y: box.height * 0.4 } });
  await expect(refusal).toHaveCount(0);
  const lat = (await page.getByTestId("lat").inputValue());
  await expectComputed(page, Number(lat).toFixed(4));

  await page.getByTestId("mast").fill("6");
  await expectComputed(page, "mast 6.0 m");
});

test("a new landing date and a shorter span recompute without losing the time cursor (#13)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await page.getByTestId("date").fill("2027-03-01");
  await page.getByTestId("date").blur();
  await expectComputed(page, "landing 2027-03-01 · 30 d");
  const slider = page.getByLabel("Time within the span");
  await slider.fill("4000"); // past the end of a 14-day grid (2016 steps)
  await page.getByRole("button", { name: "14 d" }).click();
  await expectComputed(page, "· 14 d");
  await expect(slider).toHaveValue("2015");
  expect(errors).toEqual([]);
});

test("Sun and Earth elevation and the sunset and Earthset countdown follow the time marker (T26 #27)", async ({ page }) => {
  await page.goto("/");
  const cd = page.getByTestId("countdown");
  await expect(cd).toBeVisible();
  const el = /^[+−]\d+\.\d°$/; // one decimal
  await expect(page.getByTestId("cd-sun-el")).toHaveText(el);
  await expect(page.getByTestId("cd-earth-el")).toHaveText(el);
  const dh = String.raw`(\d+d \d+h|\d+m)`;
  const none = String.raw`no (sunset|Earthset|sunrise|Earthrise) (in [\d.]+ (h|days?)|before the span ends)`;
  const when = new RegExp(`^((Sunrise|Earth contact) in ${dh} · then )?((Sunset|Earthset|sunset) in ${dh}|${none})$`);
  await expect(page.getByTestId("sunset")).toHaveText(when);
  await expect(page.getByTestId("earthset")).toHaveText(when);
  // IM-2 from 2026-11-14 00:00 UTC starts in shadow: the next event is the sunrise on the 15th (first limb at ~03:00,
  // band list 06:11 for the centre), not the sunset 17 days out (#107).
  await expect(page.getByTestId("sunset")).toHaveText(/^Sunrise in 1d \d+h · then sunset in 17d /);
  const before = await cd.textContent();
  const slider = page.getByLabel("Time within the span");
  await slider.fill(String(Math.round(Number(await slider.getAttribute("max")) * 0.6)));
  await expect.poll(() => cd.textContent()).not.toBe(before);
  await expect(cd.locator(".pill")).toHaveCount(0); // no fake badge on it
});

test("switching region moves the site and refreshes the lat/lon boxes", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("lat")).toHaveValue("-84.7906");
  await page.getByLabel("Region").selectOption("shackleton-ridge");
  await expect(page.getByTestId("lat")).not.toHaveValue("-84.7906");
  await expectComputed(page, Number(await page.getByTestId("lat").inputValue()).toFixed(4));
  await expect(page.getByTestId("refusal")).toHaveCount(0);
});

test("survival headline, charge trace and preset form", async ({ page }) => {
  await page.goto("/");
  // the charge trace is a Site timeline; the headline and preset live on Planner (wireframes-v2)
  await expect(page.getByTestId("soc-chart").locator("canvas")).toHaveCount(1);
  await expect(page.getByTestId("soc-chart")).toHaveAttribute("aria-label", /Battery state of charge/); // the uPlot live legend is off
  await page.getByRole("button", { name: "Planner", exact: true }).click();
  await expect(page.getByTestId("headline-operable")).toContainText(/days?$/);
  await expect(page.getByTestId("headline-lethal")).toContainText(/days?$|no lethal shadow in \d+ days?/);
  // sunlight and Earth contact hours come from the model's survival sums, whole hours
  await expect(page.getByTestId("headline-sun-hours")).toHaveText(/^\d+ h$/);
  await expect(page.getByTestId("headline-earth-hours")).toHaveText(/^\d+ h$/);
  // the preset reads as one line; Edit reveals the inputs
  await expect(page.getByTestId("preset-summary")).toContainText("Min charge 0 Wh · Panel 1.5 m² · Cells 30 % · Load 220 W · Earth margin 2 ° · Sun-tracking");
  await page.getByTestId("preset-edit").locator("summary").click();
  // U10: every field says cited or assumed; an edit recomputes and the field then reads assumed.
  const tags = page.locator('[data-testid^="tag-"]');
  await expect(tags.first()).toBeVisible();
  expect(await tags.count()).toBeGreaterThan(5);
  for (const t of await tags.allTextContents()) expect(["cited", "assumed"]).toContain(t);
  await expect(page.getByTestId("tag-load_w")).toHaveText("cited");
  await page.getByLabel(/^Load/).fill("300");
  await page.getByLabel(/^Load/).blur();
  await expect(page.getByTestId("tag-load_w")).toHaveText("assumed");
  await expect(page.getByLabel(/^Load/)).toHaveValue("300");
});

test("compare view shows A and B with their own countdowns, the B - A row and sentence, and the pairs live in the URL (T27 #28)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/?a=-84.7906,29.1957@2026-11-14&b=-84.7906,29.1957@2026-11-17&preset=load_w:300");
  await expect(page.getByTestId("col-a")).toBeVisible();
  await expect(page.getByTestId("col-b")).toBeVisible();
  const el = /^[+−]\d+\.\d°$/;
  for (const s of ["a", "b"]) {
    await expect(page.getByTestId(`${s}-operable`)).toContainText(/days?$|h$/);
    await expect(page.getByTestId(`${s}-lethal`)).toContainText(/days?$|h$|no lethal shadow/);
    await expect(page.getByTestId(`${s}-cd-sun-el`)).toHaveText(el);
    await expect(page.getByTestId(`${s}-cd-earth-el`)).toHaveText(el);
  }
  await expect(page.getByTestId("diff-operable")).toHaveText(/^([+−].+|no difference)$/);
  await expect(page.getByTestId("compare-sentence")).toContainText("3 days later");
  await expect(page.getByTestId("cmp-preset-load_w")).toHaveValue("300"); // the preset came from the URL
  // The sentence is the last thing on the page.
  expect(await page.locator("main > *").last().getAttribute("data-testid")).toBe("compare-sentence");
  // Changing B's date rewrites the URL and the sentence.
  await page.getByTestId("b-date").fill("2026-12-01");
  await expect(page).toHaveURL(/b=-84\.7906,29\.1957@2026-12-01/);
  await expect(page.getByTestId("compare-sentence")).toContainText("17 days later");
  // A bad B latitude is refused.
  await page.getByTestId("b-lat").fill("-95");
  await page.getByTestId("b-lat").press("Enter");
  await expect(page.getByTestId("b-refusal")).toBeVisible();
  await expect(page.getByText(/fixture/i)).toHaveCount(0);
  expect(errors).toEqual([]);
});

test("pair edits survive a trip to the site view, and mast and span ride in the URL", async ({ page }) => {
  await page.goto("/?a=-84.7906,29.1957@2026-11-14&b=-84.7906,29.1957@2026-11-20&mast=6&span=14");
  await expect(page.getByTestId("col-b")).toContainText("landing 2026-11-20");
  await page.getByTestId("b-date").fill("2026-12-05");
  await expect(page.getByTestId("col-b")).toContainText("landing 2026-12-05");
  await page.getByRole("button", { name: "Site", exact: true }).click();
  await expectComputed(page, "mast 6.0 m");
  await expectComputed(page, "· 14 d");
  await page.getByRole("button", { name: "Compare", exact: true }).click();
  await expect(page.getByTestId("col-b")).toContainText("landing 2026-12-05");
  await expect(page).toHaveURL(/b=-84\.7906,29\.1957@2026-12-05&mast=6&span=14/);
});

test("the Compare button opens the compare view from the site view, and a bad URL falls back", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Compare", exact: true }).click();
  await expect(page.getByTestId("col-a")).toBeVisible();
  await expect(page).toHaveURL(/a=-84\.7906,29\.1957@2026-11-14&b=/);
  // C4-3 (#109): B is never a copy of A, says why, and B − A is a real difference on the default IM-2 site.
  await expect(page.getByTestId("compare-why")).toContainText(/^B is the /);
  await expect(page.getByTestId("diff-operable")).toHaveText(/^[+−]/);
  await expect(page.getByTestId("compare-sentence")).not.toContainText("nothing to compare");
  await page.goto("/?a=garbage&b=-1,2@1999-01-01");
  await expect(page.getByTestId("col-a")).toContainText("-84.7906");
  await expect(page.getByTestId("col-b")).toContainText("-84.7906");
});

// T16 (#17): IM-2 replay. One click, two answers side by side, tagged assumptions, editable pit.
test("IM-2 replay shows the regional and pit answers with their tags and refuses a bad pit", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "IM-2 replay" }).click();
  for (const tag of ["assumed · U8", "assumed · U10", "unconfirmed · U19"]) await expect(page.getByTestId("replay-assumptions")).toContainText(tag);
  await page.getByTestId("replay-load").click();
  await expect(page.getByTestId("answer-regional")).toContainText("Regional terrain says");
  await expect(page.getByTestId("answer-pit")).toContainText("A pit of 20 × 1.94 m says");
  await expect(page.getByTestId("regional-operable")).toHaveText(/\S/);
  await expect(page.getByTestId("pit-operable")).toHaveText(/\S/);
  await expect(page.getByTestId("dstar-sentence")).toContainText(/would need to be [\d.]+ m deep to hide the Sun/);
  await expect(page.getByTestId("pano-pair")).toContainText("Regional terrain");
  await expect(page.getByTestId("pano-pair")).toContainText("Pit rim");
  await expect(page.getByTestId("pit-depth")).toHaveValue("1.94");
  await expect(page.getByTestId("pit-depth-tag")).toContainText("U9");
  await page.getByTestId("pit-depth").fill("3");
  await page.getByTestId("pit-depth").dispatchEvent("change");
  await expect(page.getByTestId("answer-pit")).toContainText("20 × 3 m");
  await expect(page.getByTestId("pit-depth-tag")).toContainText("assumed");
  await page.getByTestId("pit-diameter").fill("0");
  await page.getByTestId("pit-diameter").dispatchEvent("change");
  await expect(page.getByTestId("pit-refusal")).toContainText("diameter");
});

// T18 (#19): sunrise and sunset ± bands from the WASM window_bands, σ_z visible as assumed.
test("timelines show the ± band list and the assumed σ_z", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("band-key")).toContainText("σ_z = 0.5 m (assumed)");
  await expect(page.getByTestId("band-list").locator("li").first()).toHaveText(/^(sunrise|sunset) \d{4}-\d{2}-\d{2} \d{2}:\d{2} (± (<1|\d+) min|ill-conditioned)$/);
});

// T22 (#23): the site picker's height image carries the point-Sun mask for the time marker.
test("scrubbing the time marker redraws the point-Sun mask off the main thread, and the site readout agrees with the Sun's elevation (T22 #23)", async ({ page }) => {
  await page.goto("/");
  const site = page.getByTestId("mask-site");
  const legend = page.getByTestId("mask-legend");
  await expect(legend).toContainText("Sun-centre map");
  await expect(legend).toContainText("not illumination");
  // wait for the fine mask of the first time
  await expect(site).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  const slider = page.getByLabel("Time within the span");
  const max = Number(await slider.getAttribute("max"));
  // page stays responsive while a scrub storm is computed: rAF keeps ticking and every input lands
  await page.evaluate(() => {
    const w = window as any; w.__frames = 0; w.__gap = 0; let last = performance.now();
    const f = (now: number) => { w.__frames++; w.__gap = Math.max(w.__gap, now - last); last = now; requestAnimationFrame(f); };
    requestAnimationFrame(f);
  });
  const t0 = await site.getAttribute("data-t");
  for (let k = 1; k <= 40; k++) await slider.fill(String(Math.round((max * k) / 40)));
  // a mask takes 0.1 to 1 s to compute; on the page thread that would show as a gap that long between frames
  expect(await page.evaluate(() => (window as any).__frames)).toBeGreaterThan(0);
  expect(await page.evaluate(() => (window as any).__gap), "largest gap between animation frames, ms").toBeLessThan(250);
  await expect(site).not.toHaveAttribute("data-t", t0!);
  await expect(site).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  // the mask's time is the marker's time, and its site verdict never contradicts a Sun below the flat horizon
  for (const frac of [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9]) {
    await slider.fill(String(Math.round(max * frac)));
    const el = Number(((await page.getByTestId("cd-sun-el").textContent()) ?? "").replace("−", "-").match(/-?[\d.]+/)![0]);
    // the mask follows the marker's time, then settles on the fine grid
    if (el < -1) await expect(site).toHaveAttribute("data-lit", "0", { timeout: 60000 });
  }
  await expect(page.locator("canvas[aria-label^='Greyscale height image']")).toBeVisible();
});

// T23 (#24): the 3D terrain paints the same point-Sun mask as the picker, at the same time marker.
test("the 3D terrain mounts, shows the picker's mask and time, labels vertical exaggeration (T23 #24)", async ({ page }) => {
  await page.goto("/");
  const t3 = page.getByTestId("terrain-3d");
  const site = page.getByTestId("mask-site");
  const nav = page.getByRole("navigation", { name: "View" });
  // the picker's mask is on Site and the 3D view on Terrain; read the picker's time first
  await expect(site).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  const siteT = await site.getAttribute("data-t");
  await nav.getByRole("button", { name: "Terrain" }).click();
  await expect(t3).toBeVisible();
  await expect(page.locator("canvas[aria-label^='3D height mesh']")).toBeVisible();
  // the full mask legend lives in the picker; the 3D view keeps the Sun readout and one caveat line
  await expect(t3).toContainText("Sun (model): azimuth");
  await expect(page.getByTestId("t3-caveat")).toHaveText("Point-Sun mask, not illumination; points near the tile edge can read lit.");
  await expect(t3).toHaveAttribute("data-exaggeration", "1");
  await expect(t3).toContainText("1× (true scale)");
  await expect(t3).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  // one mask feeds both views: same time, one vertex per mask point
  expect(await t3.getAttribute("data-t")).toBe(siteT);
  expect(Number(await t3.getAttribute("data-vertices"))).toBe(192 * 192);
  await page.getByTestId("t3-exaggeration").selectOption("2");
  await expect(t3).toContainText("Heights drawn ×2");
  await page.getByTestId("t3-exaggeration").selectOption("1");
  // scrubbing on Site moves the 3D view's time with the picker's
  await nav.getByRole("button", { name: "Site" }).click();
  const slider = page.getByLabel("Time within the span");
  const max = Number(await slider.getAttribute("max"));
  await slider.fill(String(Math.round(max / 2)));
  await expect.poll(async () => (await site.getAttribute("data-t")) !== siteT, { timeout: 60000 }).toBe(true);
  await expect(site).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  const midT = await site.getAttribute("data-t");
  await nav.getByRole("button", { name: "Terrain" }).click();
  await expect(t3).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  await expect.poll(async () => await t3.getAttribute("data-t"), { timeout: 60000 }).toBe(midT);
});

test("seven views in the wireframes-v2 order, a keyboard-reachable GitHub link, and the scoreboard on Validation only (T79 #79)", async ({ page }) => {
  await page.goto("/");
  const nav = page.getByRole("navigation", { name: "View" });
  await expect(nav.getByRole("button")).toHaveText(["Site", "Planner", "Terrain", "Compare", "IM-2 replay", "Validation", "About"]);
  // Scoreboard and Tracer live in the Validation view, not on the Site scroll.
  await expect(page.getByRole("table", { name: /One row per check/ })).toHaveCount(0);
  await expect(page.getByTestId("sun-az")).toHaveCount(0);
  const gh = page.getByRole("link", { name: "GitHub" });
  await expect(gh).toHaveAttribute("href", "https://github.com/qicai14/penumbra");
  await gh.focus();
  await expect(gh).toBeFocused();
  await nav.getByRole("button", { name: "Validation" }).click();
  await expect(nav.getByRole("button", { name: "Validation" })).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("table", { name: /One row per check/ })).toBeVisible();
});

test("a curated site outside the tiles shows OUT_OF_COVERAGE, not an error; both tool links are present (#82)", async ({ page }) => {
  await page.goto("/");
  const curated = page.getByTestId("curated");
  await expect(curated).toBeEnabled();
  await curated.selectOption("a11");
  await expect(page.getByTestId("out-of-coverage")).toContainText("OUT_OF_COVERAGE");
  await expect(page.getByTestId("refusal")).toHaveCount(0);
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(page.getByTestId("latlon-text")).toHaveText("0.6875, 23.4333");
  const qm = page.getByRole("link", { name: "Open in QuickMap (opens in a new tab)" });
  await expect(qm).toHaveAttribute("href", "https://quickmap.lroc.im-ldi.com");
  await expect(qm).toHaveAttribute("target", "_blank");
  await expect(qm).toHaveAttribute("rel", /noopener/);
  await expect(page.getByRole("link", { name: "Open in Moon Trek (opens in a new tab)" })).toHaveAttribute("href", "https://trek.nasa.gov/moon/");
  // a covered site moves the site and clears the state
  await curated.selectOption("sp01");
  await expect(page.getByTestId("out-of-coverage")).toHaveCount(0);
  await expect(page.getByTestId("lat")).toHaveValue("-89.45");
});

test("what changes what: four labelled signed bars, and the 300 m bars say so when the grid cannot answer (T83 #83)", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Planner", exact: true }).click(); // What changes what lives on Planner
  await expect(page.getByTestId("what-changes")).toBeVisible();
  for (const [id, label] of [["date-minus", "Landing 2 days earlier"], ["date-plus", "Landing 2 days later"], ["pin-best", "Pin moved 300 m: best cell"], ["pin-worst", "Pin moved 300 m: worst cell"]]) {
    await expect(page.getByTestId(`wc-${id}`)).toContainText(label);
    await expect(page.getByTestId(`wc-${id}-value`)).toHaveText(/^[+−]?\d+\.\d d$/);
  }
  expect(Number((await page.getByTestId("wc-pin-best-value").textContent())!.replace("−", "-").replace(" d", ""))).toBeGreaterThanOrEqual(0);
  // A landing date the grid has no layer for: the date bars still run, the 300 m bars say why they are missing.
  // The date is set on Site, so go there and come back.
  await page.getByRole("button", { name: "Site", exact: true }).click();
  await page.getByTestId("date").fill("2026-11-20");
  await page.getByTestId("date").blur();
  await page.getByRole("button", { name: "Planner", exact: true }).click();
  await expect(page.getByTestId("wc-pin-best-note")).toContainText("no layer for 2026-11-20");
  await expect(page.getByTestId("wc-date-plus-value")).toBeVisible();
});

test("what changes what: a region with no grid says so instead of guessing", async ({ page }) => {
  await page.route("**/data/manifest.json", async (route) => {
    const m = await (await route.fetch()).json();
    delete m.grids;
    await route.fulfill({ json: m });
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Planner", exact: true }).click();
  await expect(page.getByTestId("wc-pin-best-note")).toContainText("no grid data for this region");
  await expect(page.getByTestId("wc-pin-worst-note")).toContainText("no grid data for this region");
  await expect(page.getByTestId("wc-date-minus-value")).toBeVisible();
});

// T80 (#80): About content. Limitations carry their issue or scoreboard ids; empty video URLs render nothing.
test("About lists team, user story, limitations with ids, and no video slot while URLs are empty", async ({ page, request }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "About", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Team" })).toBeVisible();
  await expect(page.getByText("Fractured Toes")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Why Penumbra" })).toBeVisible();
  const lim = page.getByTestId("limitations");
  for (const id of ["V2b", "V3", "#75", "#48"]) await expect(lim.locator(`[data-limit="${id}"]`)).toHaveCount(1);
  // Every scoreboard row that is not a pass is named in the limitations.
  const rows = (await (await request.get("scoreboard.json")).json()) as { id: string; pass: boolean | null }[];
  for (const r of rows.filter((r) => r.pass !== true)) await expect(lim.locator(`[data-limit="${r.id}"]`).first()).toBeVisible();
  await expect(lim).toContainText("0.21 d");
  await expect(lim).toContainText("123 sampled pixels above 60 %");
  await expect(page.getByTestId("videos")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Videos" })).toHaveCount(0);
  await expect(page.locator("main video, main iframe")).toHaveCount(0);
});

test("Terrain has its own time slider; moving it redraws the 3D mask and carries back to Site", async ({ page }) => {
  await page.goto("/");
  const nav = page.getByRole("navigation", { name: "View" });
  await nav.getByRole("button", { name: "Terrain" }).click();
  const t3 = page.getByTestId("terrain-3d");
  await expect(t3).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  const before = await t3.getAttribute("data-t");
  await page.getByTestId("t3-time").fill("300");
  await expect(t3).not.toHaveAttribute("data-t", before!, { timeout: 60000 });
  await expect(page.getByTestId("countdown")).toBeVisible(); // Sun and Earth elevation sit above the 3D view too
  // Reset view lights up once the camera leaves home, and goes quiet again when pressed
  const reset = page.getByTestId("t3-reset");
  await expect(reset).toHaveAttribute("data-moved", "false");
  const cv = page.locator("canvas[aria-label^='3D height mesh']");
  await cv.hover(); await page.mouse.wheel(0, -400);
  await expect(reset).toHaveAttribute("data-moved", "true");
  await reset.click();
  await expect(reset).toHaveAttribute("data-moved", "false");
  const after = await t3.getAttribute("data-t");
  await nav.getByRole("button", { name: "Site" }).click();
  await expect(page.getByTestId("mask-site")).toHaveAttribute("data-t", after!, { timeout: 60000 });
});

test("ten regions: each shows its tier and label, and the ranking is numbered and sorted by operable days (#26)", async ({ page }) => {
  await page.goto("/");
  const sel = page.getByLabel("Region");
  await expect(sel.locator("option")).toHaveCount(10);
  for (const id of ["haworth", "slater-plain", "peak-near-cabeus-b"]) {
    await sel.selectOption(id);
    await expect(page.getByTestId("region-label")).toContainText(/Tier [123]/);
    await expect(page.getByTestId("refusal")).toHaveCount(0);
    // the survival headline lives on Planner
    await page.getByRole("navigation", { name: "View" }).getByRole("button", { name: "Planner", exact: true }).click();
    await expect(page.getByTestId("headline-operable")).toHaveText(/\S/);
    await page.getByRole("navigation", { name: "View" }).getByRole("button", { name: "Site", exact: true }).click();
  }
  await expect(page.getByTestId("ranking-title")).toContainText("Region ranking for 2026-11-14");
  const days = (await page.getByTestId("best-cells").locator(".days").allTextContents()).map((t) => parseFloat(t));
  expect(days.length).toBe(10);
  expect([...days].sort((a, b) => b - a)).toEqual(days);
  // one ranked table in a closed dropdown whose summary names the #1 site; open it, rank starts at 1, each row's → moves the site
  await expect(page.getByTestId("ranking-top")).toContainText("best: ");
  await page.getByTestId("ranking").locator("summary").first().click();
  const rows = page.getByTestId("best-cells").locator("tbody tr");
  await expect(rows).toHaveCount(10);
  await expect(rows.first().locator(".n")).toHaveText("1");
  await expect(page.getByTestId("best-cells").getByRole("button", { name: /^Move site to / })).toHaveCount(10);
});

test("off-grid date: one message and three chips, a chip restores the ranking; move pin matches the readout (#111)", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("best-cells")).toBeVisible();
  await page.getByTestId("date").fill("2026-12-20");
  await page.getByTestId("date").blur();
  await expect(page.getByTestId("no-grid-msg")).toHaveCount(1);
  await expect(page.getByTestId("best-cells")).not.toContainText("No grid for");
  await expect(page.getByTestId("date-chip")).toHaveCount(3);
  await expect(page.getByTestId("date-note")).toBeVisible();
  await page.getByTestId("date-chip").filter({ hasText: "2026-11-14" }).click();
  await expect(page.getByTestId("no-grid-msg")).toHaveCount(0);
  await expect(page.getByTestId("ranking-title")).toContainText("Region ranking for 2026-11-14");
  expect(await page.getByTestId("best-cells").locator(".days").count()).toBe(10);
  // Move pin there: the headline then shows the readout's gain over the old figure. Both live on Planner.
  await page.getByRole("navigation", { name: "View" }).getByRole("button", { name: "Planner", exact: true }).click();
  const days = async () => parseFloat((await page.getByTestId("headline-operable").textContent())!);
  const before = await days();
  const move = page.getByTestId("move-pin");
  await expect(move).toHaveCount(1);
  {
    const gain = parseFloat((await page.getByTestId("nearest-better").textContent())!.match(/\+(\d+\.\d) d/)![1]);
    expect((await move.boundingBox())!.height).toBeGreaterThanOrEqual(44);
    await move.click();
    await expect.poll(async () => Math.abs((await days()) - (before + gain))).toBeLessThan(0.15);
  }
});

test("landing window: bar equals headline, clicking a bar sets the date and the headline (#108)", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("navigation", { name: "View" }).getByRole("button", { name: "Planner", exact: true }).click(); // chart, headline and "Computed for" live on Planner
  const table = page.getByTestId("lw-date");
  await expect(table).toHaveCount(30, { timeout: 15000 });
  const headline = async () => (await page.getByTestId("headline-operable").textContent())!;
  // AC1, 2026-11-14 at the IM-2 site: the bar's figure is the headline's.
  await expect(page.getByTestId("computed-for")).toContainText("landing 2026-11-14");
  expect(await table.first().getAttribute("data-operable")).toBe(await headline());
  const ms = await page.evaluate(() => (window as any).__landingWindowMs as number);
  console.info(`landing window: 30 dates in ${ms} ms`);
  expect(ms).toBeLessThan(5000);
  await expect(page.getByTestId("lw-best")).toContainText("Best landing date");
  // The canvas is hidden from screen readers; the table is the keyboard route.
  await expect(page.getByTestId("lw-chart").locator("canvas").first()).toHaveAttribute("aria-hidden", "true");
  // Click the 6th bar on the chart itself.
  const over = page.getByTestId("lw-chart").locator(".u-over");
  await over.scrollIntoViewIfNeeded();
  const box = (await over.boundingBox())!;
  await page.mouse.move(box.x + (box.width * (5 + 0.6)) / 30, box.y + box.height - 3);
  await page.mouse.down(); await page.mouse.up();
  const want = table.nth(5);
  await expect(page.getByTestId("computed-for")).toContainText(`landing ${await want.getAttribute("data-date")}`);
  await expect.poll(headline).toBe(await want.getAttribute("data-operable"));
  const wantDate = (await want.getAttribute("data-date"))!; // read before leaving Planner, where the table lives
  await page.getByRole("navigation", { name: "View" }).getByRole("button", { name: "Site", exact: true }).click(); // the date box lives on Site
  await expect(page.getByTestId("date")).toHaveValue(wantDate);
});

test("play advances the time marker (C4-7 #113)", async ({ page }) => {
  await page.emulateMedia({ reducedMotion: "reduce" }); // no autoplay, but play still works
  await page.goto("/");
  const slider = page.getByLabel("Time within the span");
  await expect(slider).toBeVisible();
  const before = Number(await slider.inputValue());
  await page.waitForTimeout(300);
  expect(Number(await slider.inputValue())).toBe(before);
  await page.getByTestId("play").focus();
  await page.keyboard.press("Space"); // Space toggles when the button has focus
  await page.waitForTimeout(1000);
  expect(Number(await slider.inputValue())).toBeGreaterThan(before);
  await page.getByTestId("play").click(); // pause holds
  const held = await slider.inputValue();
  await page.waitForTimeout(300);
  expect(await slider.inputValue()).toBe(held);
});

// Hero line and three scenario buttons (C4-4 #110). The survival headline lives on Planner.
const navBtn = (page: import("@playwright/test").Page, name: string) => page.getByRole("navigation", { name: "View" }).getByRole("button", { name, exact: true });
async function heroHeadline(page: import("@playwright/test").Page) {
  await navBtn(page, "Planner").click();
  await expect(page.getByTestId("headline-operable")).toHaveText(/\d/);
  await expect(page.getByTestId("headline-lethal")).toHaveText(/\S/);
  return (await page.getByTestId("headline-operable").textContent())!;
}
async function moveAway(page: import("@playwright/test").Page) { // a site and date the hero buttons would change
  await page.getByTestId("lat").fill("-84.9"); await page.getByTestId("lon").fill("29.2");
  await page.getByRole("button", { name: "Set site" }).click();
  await page.getByTestId("date").fill("2026-05-31"); await page.getByTestId("date").blur();
  await expect(page.getByTestId("date")).toHaveValue("2026-05-31");
}

test("hero: line and three buttons are in the first viewport, 44 px, no horizontal scroll (#110)", async ({ page }) => {
  for (const [w, h] of [[1440, 900], [390, 844]]) {
    await page.setViewportSize({ width: w, height: h });
    await page.goto("/");
    await expect(page.getByTestId("hero")).toBeVisible();
    await expect(page.getByTestId("hero-line")).not.toContainText(/btierb|AVGVISIB|\b[WEU]\d/i);
    const line = (await page.getByTestId("hero-line").boundingBox())!;
    expect(line.y + line.height).toBeLessThanOrEqual(h);
    expect(line.x + line.width).toBeLessThanOrEqual(w);
    for (const id of ["hero-im2", "hero-best", "hero-slip"]) {
      const box = (await page.getByTestId(id).boundingBox())!;
      expect(box.height).toBeGreaterThanOrEqual(44);
      expect(box.width).toBeGreaterThanOrEqual(44);
      expect(box.y + box.height).toBeLessThanOrEqual(h);
      expect(box.x + box.width).toBeLessThanOrEqual(w);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  }
});

test("hero: IM-2's site button restores IM-2 and 2026-11-14 from elsewhere (#110)", async ({ page }) => {
  await page.goto("/");
  await moveAway(page);
  await expect(page.getByTestId("lat")).toHaveValue("-84.9");
  await page.getByTestId("hero-im2").click();
  await expect(page.getByTestId("date")).toHaveValue("2026-11-14");
  await expect(page.getByTestId("lat")).toHaveValue("-84.7906");
  await expect(page.getByTestId("lon")).toHaveValue("29.1957");
  await heroHeadline(page);
});

test("hero: best region button moves to the top-ranked region and its best cell (#110)", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("ranking-top")).toContainText("best: ");
  await page.getByTestId("ranking").locator("summary").first().click();
  const first = page.getByTestId("best-cells").locator("tbody tr").first();
  const id = (await first.getAttribute("data-testid"))!.replace("best-", "");
  const [lat, lon] = (await first.getByRole("button").getAttribute("title"))!.split(",").map((s) => parseFloat(s));
  await page.getByTestId("hero-best").click();
  await expect(page.getByLabel("Region")).toHaveValue(id);
  await expect.poll(async () => parseFloat(await page.getByTestId("lat").inputValue())).toBeCloseTo(lat, 3);
  await expect.poll(async () => parseFloat(await page.getByTestId("lon").inputValue())).toBeCloseTo(lon, 3);
  await heroHeadline(page);
});

test("hero: best region is disabled with the ranked dates listed when the date has no ranking (#110)", async ({ page }) => {
  await page.goto("/");
  await page.getByTestId("date").fill("2026-12-20"); await page.getByTestId("date").blur();
  await expect(page.getByTestId("hero-best")).toBeDisabled();
  await expect(page.getByTestId("hero-best")).toHaveAttribute("aria-describedby", "hero-best-note");
  await expect(page.getByTestId("hero-best-note")).toContainText("2026-11-14");
});

test("hero: schedule-slips button sets IM-2's site and 2026-11-12; both dates give a headline (#110)", async ({ page }) => {
  await page.goto("/");
  await moveAway(page);
  await page.getByTestId("hero-im2").click();
  const on14 = await heroHeadline(page);
  await navBtn(page, "Site").click();
  await moveAway(page);
  await page.getByTestId("hero-slip").click();
  await expect(page.getByTestId("date")).toHaveValue("2026-11-12");
  await expect(page.getByTestId("hero-operable")).toHaveText(/[0-9]/);
  await expect(page.getByTestId("lat")).toHaveValue("-84.7906");
  await expect(page.getByTestId("lon")).toHaveValue("29.1957");
  const on12 = await heroHeadline(page);
  console.info(`hero slip: operable ${on14} on 2026-11-14, ${on12} on 2026-11-12`);
  expect(on12).not.toBe(on14);
});

test("the pin shows its slope from the 20 m DEM (C4-6 #112)", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("slope")).toHaveText(/^Slope \d+\.\d° \(20 m DEM, 60 m window\)$/);
});
