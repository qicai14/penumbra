import { expect, test } from "@playwright/test";

// Runs against `vite preview` locally and the deployed URL in CI (BASE_URL).
test("panorama loads the fixture (api_version checked against WASM) and shows the readout", async ({ page, request }) => {
  await page.goto("/");
  await expect(page.getByTestId("readout")).toBeVisible();
  await expect(page.locator("canvas")).toBeVisible();
  // The badge follows the data's own `fake` flag.
  const res = await request.get("fixture/scenario.json");
  const { fake } = await res.json();
  await expect(page.getByTestId("fake-badge")).toHaveCount(fake ? 1 : 0);
});

test("keyboard focus reads the horizon elevation and obstacle distance", async ({ page }) => {
  await page.goto("/");
  await page.locator("canvas").focus();
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
