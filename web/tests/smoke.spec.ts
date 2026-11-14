import { expect, test } from "@playwright/test";

// Runs against `vite preview` locally and the deployed URL in CI (BASE_URL).
test("panorama loads the fixture (api_version checked against WASM) and shows the readout", async ({ page, request }) => {
  await page.goto("/");
  await expect(page.getByTestId("readout")).toBeVisible();
  await expect(page.getByLabel(/^Horizon panorama/)).toBeVisible();
  // The badge follows the data's own `fake` flag.
  const res = await request.get("fixture/scenario.json");
  const { fake } = await res.json();
  await expect(page.getByTestId("fake-badge")).toHaveCount(fake ? 1 : 0);
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
