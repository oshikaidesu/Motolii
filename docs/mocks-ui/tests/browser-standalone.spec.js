import { expect, test } from "@playwright/test";

const URL = "http://127.0.0.1:5173/browser-standalone.html";

test("product Browser mounts without the legacy Host shell", async ({ page }) => {
  const pageErrors = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await page.goto(URL);

  await expect(page.locator(".browser-candidate")).toHaveCount(1);
  await expect(page.locator(".app")).toHaveCount(0);
  await expect(page.locator(".inspector, .timeline")).toHaveCount(0);
  await expect(page.locator("#vism-browser")).toBeVisible();
  await expect(page.locator("#elements-browser")).toBeHidden();
  await expect(page.locator("#project-browser")).toBeHidden();

  await page.locator('[data-tab="create"]').click();
  await expect(page.locator("#vism-browser")).toBeHidden();
  await expect(page.locator("#elements-browser")).toBeVisible();
  await expect(page.locator("#project-browser")).toBeHidden();

  await page.locator('[data-tab="project"]').click();
  await expect(page.locator("#vism-browser")).toBeHidden();
  await expect(page.locator("#elements-browser")).toBeHidden();
  await expect(page.locator("#project-browser")).toBeVisible();
  expect(pageErrors).toEqual([]);
});
