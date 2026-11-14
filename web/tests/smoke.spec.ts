import { expect, test } from "@playwright/test";

test("renders the Rust/WASM sum", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("sum")).toHaveText("2 + 3 = 5 (from Rust/WASM)");
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
