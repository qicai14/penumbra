import { expect, test } from "@playwright/test";

test("renders the Rust/WASM sum", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByTestId("sum")).toHaveText("2 + 3 = 5 (from Rust/WASM)");
});
