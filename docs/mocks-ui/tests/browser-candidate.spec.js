import { expect, test } from "@playwright/test";

const CANDIDATE_URL =
  "http://127.0.0.1:5173/#plugin-browser-candidate";

async function openCandidate(page) {
  await page.goto(CANDIDATE_URL, { waitUntil: "domcontentloaded" });
  await page
    .locator('.app[data-parity-ready="true"]')
    .waitFor({ state: "visible" });
}

test.describe("shared discovery Browser candidate", () => {
  test("keeps plugin results visual and reserves state labels for deviations", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(page.locator(".candidate-plugin-card")).toHaveCount(4);
    await expect(page.locator(".candidate-plugin-card .thumb-state")).toHaveText([
      "Unavailable",
      "Missing",
    ]);
    await expect(page.getByText("READY", { exact: true })).toHaveCount(0);
    await expect(page.getByText("AVAILABLE", { exact: true })).toHaveCount(0);

    await page.locator('[data-plugin-source="issues"]').click();
    await expect(
      page.locator(".candidate-plugin-card:visible"),
    ).toHaveCount(2);
    await expect(page.locator("#plugin-result-count")).toHaveText("2");

    await page.locator("#plugin-filter-toggle").click();
    await expect(page.locator("#plugin-filter-panel")).toBeVisible();
    await page.locator('[data-plugin-label="effect"]').click();
    await expect(
      page.locator(".candidate-plugin-card:visible"),
    ).toHaveCount(1);
    await expect(page.locator("#plugin-result-count")).toHaveText("1");
  });

  test("uses the same search, source rail, and result grid for Project and Files", async ({
    page,
  }) => {
    await openCandidate(page);

    await page.locator('button[data-tab="project"]').click();
    await expect(page.locator(".candidate-project-browser")).toBeVisible();
    await expect(page.getByRole("searchbox", { name: "Search assets" })).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Asset sources" })).toBeVisible();
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(4);

    await page.locator('button[data-asset-source="files"]').click();
    await expect(page.locator("#asset-path")).toContainText("source");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(6);
    await expect(page.locator("#place-asset")).toHaveText("＋ INBOX");
  });
});
