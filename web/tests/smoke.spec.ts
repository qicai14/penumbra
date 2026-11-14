import { expect, test } from "@playwright/test";

// Runs against `vite preview` locally and the deployed URL in CI (BASE_URL).
test("panorama loads the fixture (api_version checked against WASM) and shows the readout", async ({ page, request }) => {
  await page.goto("/");
  await expect(page.getByTestId("readout")).toBeVisible();
  await expect(page.getByLabel(/^Horizon panorama/)).toBeVisible();
  // The badge follows the data's own `fake` flag.
  const res = await request.get("fixture/scenario.json");
  const { fake } = await res.json();
  // The horizon is the real DEM (T07 #8), so the panorama badge is off; the fixture-dates note follows `fake`.
  await expect(page.getByTestId("fake-badge")).toHaveCount(0);
  // Survival is the WASM model (T09 #10), so its badges are off too.
  await expect(page.getByTestId("fake-badge-survival")).toHaveCount(0);
  await expect(page.getByTestId("fake-badge-timelines")).toHaveCount(0);
  await expect(page.getByTestId("track-dates")).toHaveCount(fake ? 1 : 0);
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
  const want = { "sun-az": 97.531671689975, "sun-el": 0.878280821, "earth-az": 329.676179426621, "earth-el": -0.153898865224 };
  for (const [id, v] of Object.entries(want)) {
    await expect(page.getByTestId(id)).toBeVisible();
    const got = Number(await page.getByTestId(id).getAttribute("data-value"));
    expect(Math.abs(got - v), id).toBeLessThan(1e-9);
  }
  await expect(page.getByTestId("dip")).toContainText("no terrain yet");
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

test("scoreboard lists V1-V5 and shows 'not run' for checks that have not run", async ({ page, request }) => {
  await page.goto("/");
  const table = page.getByRole("table", { name: /One row per check/ });
  await expect(table.getByTestId("score-row")).toHaveCount(5);
  const rows = (await (await request.get("scoreboard.json")).json()) as { pass: boolean | null }[];
  const notRun = rows.filter((r) => r.pass === null).length;
  await expect(table.getByRole("cell", { name: "not run" })).toHaveCount(notRun);
  await expect(page.getByRole("heading", { name: "Limitations" })).toBeVisible();
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
