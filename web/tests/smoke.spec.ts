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
