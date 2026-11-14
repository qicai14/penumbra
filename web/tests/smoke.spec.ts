import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

// Runs against `vite preview` locally and the deployed URL in CI (BASE_URL).
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
  await page.getByRole("button", { name: "About", exact: true }).click();
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
  await expect(tl).toContainText("Illumination");
  await expect(tl).toContainText("Visibility");
  await expect(tl).toContainText("2° (assumed)");
  const slider = page.getByLabel("Time within the span");
  const max = Number(await slider.getAttribute("max"));
  await slider.fill(String(Math.round(max * 0.25)));
  const a = await cursorX(page);
  const textA = await page.getByTestId("timelines-now").textContent();
  await slider.fill(String(Math.round(max * 0.75)));
  await expect.poll(() => cursorX(page)).toBeGreaterThan(a + 20);
  expect(await page.getByTestId("timelines-now").textContent()).not.toBe(textA);
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
  await page.getByRole("button", { name: "About", exact: true }).click();
  const table = page.getByRole("table", { name: /One row per check/ });
  await expect(table.getByTestId("score-row")).toHaveCount(6);
  const rows = (await (await request.get("scoreboard.json")).json()) as { pass: boolean | null }[];
  const notRun = rows.filter((r) => r.pass === null).length;
  await expect(table.getByRole("cell", { name: "not run" })).toHaveCount(notRun);
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
  await expect(page.getByTestId("computed-for")).toContainText(Number(lat).toFixed(4));

  await page.getByTestId("mast").fill("6");
  await expect(page.getByTestId("computed-for")).toContainText("mast 6.0 m");
});

test("a new landing date and a shorter span recompute without losing the time cursor (#13)", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await page.goto("/");
  await page.getByTestId("date").fill("2027-03-01");
  await page.getByTestId("date").blur();
  await expect(page.getByTestId("computed-for")).toContainText("landing 2027-03-01 · 30 d");
  const slider = page.getByLabel("Time within the span");
  await slider.fill("4000"); // past the end of a 14-day grid (2016 steps)
  await page.getByRole("button", { name: "14 d" }).click();
  await expect(page.getByTestId("computed-for")).toContainText("· 14 d");
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
  const when = /^(Sunset|Earthset) in (\d+d \d+h|\d+m)$|^no (sunset|Earthset|sunrise|Earthrise) (in [\d.]+ (h|days?)|before the span ends)$/;
  await expect(page.getByTestId("sunset")).toHaveText(when);
  await expect(page.getByTestId("earthset")).toHaveText(when);
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
  await expect(page.getByTestId("computed-for")).toContainText(Number(await page.getByTestId("lat").inputValue()).toFixed(4));
  await expect(page.getByTestId("refusal")).toHaveCount(0);
});

test("survival headline, charge trace and preset form", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("headline-operable")).toContainText(/days?$/);
  await expect(page.getByTestId("headline-lethal")).toContainText(/days?$|no lethal shadow in \d+ days?/);
  await expect(page.getByTestId("soc-chart").locator("canvas")).toHaveCount(1);
  await expect(page.getByTestId("soc-chart")).toContainText("Battery charge");
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
  await expect(page.getByTestId("computed-for")).toContainText("mast 6.0 m");
  await expect(page.getByTestId("computed-for")).toContainText("· 14 d");
  await page.getByRole("button", { name: "Compare A vs B" }).click();
  await expect(page.getByTestId("col-b")).toContainText("landing 2026-12-05");
  await expect(page).toHaveURL(/b=-84\.7906,29\.1957@2026-12-05&mast=6&span=14/);
});

test("the Compare button opens the compare view from the site view, and a bad URL falls back", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: "Compare A vs B" }).click();
  await expect(page.getByTestId("col-a")).toBeVisible();
  await expect(page).toHaveURL(/a=-84\.7906,29\.1957@2026-11-14&b=-84\.7906,29\.1957@2026-11-14/);
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
  await expect(legend).toContainText("Point-Sun mask");
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
  await expect(t3).toBeVisible();
  await expect(page.locator("canvas[aria-label^='3D height mesh']")).toBeVisible();
  await expect(t3).toContainText("Point-Sun mask");
  await expect(t3).toContainText("not illumination");
  await expect(t3).toHaveAttribute("data-exaggeration", "1");
  await expect(t3).toContainText("1× (true scale)");
  await expect(t3).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  await expect(site).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  // one mask feeds both views: same time, one vertex per mask point
  expect(await t3.getAttribute("data-t")).toBe(await site.getAttribute("data-t"));
  expect(Number(await t3.getAttribute("data-vertices"))).toBe(192 * 192);
  await page.getByTestId("t3-exaggeration").selectOption("2");
  await expect(t3).toContainText("Heights drawn ×2");
  await page.getByTestId("t3-exaggeration").selectOption("1");
  // scrubbing moves the 3D view's time with the picker's
  const slider = page.getByLabel("Time within the span");
  const max = Number(await slider.getAttribute("max"));
  await slider.fill(String(Math.round(max / 2)));
  await expect(t3).toHaveAttribute("data-stride", "8", { timeout: 60000 });
  await expect.poll(async () => (await t3.getAttribute("data-t")) === (await site.getAttribute("data-t")), { timeout: 60000 }).toBe(true);
});

test("four views, a keyboard-reachable GitHub link, and a verdict strip built from scoreboard.json (T79 #79)", async ({ page, request }) => {
  await page.goto("/");
  const nav = page.getByRole("navigation", { name: "View" });
  for (const name of ["Site", "Compare A vs B", "IM-2 replay", "About"]) await expect(nav.getByRole("button", { name })).toBeVisible();
  // Scoreboard, About and Tracer live in the About view, not on the Site scroll.
  await expect(page.getByRole("table", { name: /One row per check/ })).toHaveCount(0);
  await expect(page.getByTestId("sun-az")).toHaveCount(0);
  const gh = page.getByRole("link", { name: "GitHub" });
  await expect(gh).toHaveAttribute("href", "https://github.com/qicai14/penumbra");
  await gh.focus();
  await expect(gh).toBeFocused();
  // The strip text is rebuilt from the data file: every id with its status.
  const rows = (await (await request.get("scoreboard.json")).json()) as { id: string; pass: boolean | null }[];
  const ids = (s: boolean | null) => rows.filter((r) => r.pass === s).map((r) => r.id).join(", ");
  const want = ([[true, "pass"], [false, "fails"], [null, "not run"]] as const).filter(([s]) => ids(s)).map(([s, w]) => `${ids(s)} ${w}`).join(" · ");
  await expect(page.getByTestId("verdict-strip")).toHaveText(want);
  await page.getByRole("button", { name: "Validation details" }).click();
  await expect(nav.getByRole("button", { name: "About" })).toHaveAttribute("aria-pressed", "true");
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
