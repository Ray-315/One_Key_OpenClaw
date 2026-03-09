import { test, expect } from "@playwright/test";

test.describe("App Navigation", () => {
  test("should load dashboard page", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("text=One Key OpenClaw")).toBeVisible();
  });

  test("sidebar navigation links are visible", async ({ page }) => {
    await page.goto("/");
    const sidebar = page.locator("aside");
    await expect(sidebar).toBeVisible();
    await expect(sidebar.locator("nav a")).toHaveCount(6);
  });

  test("navigate to environment check page", async ({ page }) => {
    await page.goto("/");
    await page.click('a[href="/env"]');
    await expect(page).toHaveURL("/env");
  });

  test("navigate to deploy page", async ({ page }) => {
    await page.goto("/");
    await page.click('a[href="/deploy"]');
    await expect(page).toHaveURL("/deploy");
  });

  test("navigate to recipe page", async ({ page }) => {
    await page.goto("/");
    await page.click('a[href="/recipe"]');
    await expect(page).toHaveURL("/recipe");
  });

  test("navigate to task flow page", async ({ page }) => {
    await page.goto("/");
    await page.click('a[href="/flow"]');
    await expect(page).toHaveURL("/flow");
  });

  test("navigate to log page", async ({ page }) => {
    await page.goto("/");
    await page.click('a[href="/log"]');
    await expect(page).toHaveURL("/log");
  });
});

test.describe("Language Switching", () => {
  test("language toggle button exists in sidebar", async ({ page }) => {
    await page.goto("/");
    const langBtn = page.locator("aside button", { hasText: /English|中文/ });
    await expect(langBtn).toBeVisible();
  });

  test("clicking language toggle switches language", async ({ page }) => {
    await page.goto("/");
    const langBtn = page.locator("aside button", { hasText: /English|中文/ });
    const initialText = await langBtn.textContent();
    await langBtn.click();
    const newText = await langBtn.textContent();
    expect(newText).not.toBe(initialText);
  });
});
