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
    await expect(page.locator("#vism-browser")).not.toContainText(
      "Layered light pulses",
    );
    await expect(page.getByRole("button", { name: "Apply" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Filters" })).toHaveCount(0);
    await expect(page.locator("#plugin-filter-panel")).toHaveCount(0);

    await page.getByRole("button", { name: "Thumbnail-only view" }).click();
    await expect(page.locator("#vism-browser")).toHaveAttribute(
      "data-view",
      "visual",
    );
    await expect(page.locator(".candidate-card-name:visible")).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Echo Bloom" })).toBeVisible();
    await page
      .getByRole("button", { name: "Thumbnail and name view" })
      .click();

    await page.locator('[data-plugin-source="issues"]').click();
    await expect(
      page.locator(".candidate-plugin-card:visible"),
    ).toHaveCount(2);
    await expect(page.locator("#plugin-result-count")).toHaveText("2");

    await page.getByRole("searchbox", { name: "Search plugins" }).fill("fold");
    await expect(
      page.locator(".candidate-plugin-card:visible"),
    ).toHaveCount(1);
    await expect(page.locator("#plugin-result-count")).toHaveText("1");
  });

  test("applies a usable effect by drag and drop or double click", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(
      page.locator("#inspector #effect-parameter-description"),
    ).toContainText(
      "Adjust Intensity and Spread",
    );
    await expect(page.locator("#inspector")).not.toContainText("TRANSFORM");
    await expect(page.locator("#inspector")).toContainText("Pulse rings · Effect");
    const echo = page.locator('.candidate-plugin-card[data-mode="installed"]');
    await echo.dblclick();
    await expect(page.locator("#undo-state")).toContainText("Add Echo Bloom");

    const glyph = page.locator('.candidate-plugin-card[data-mode="discover"]');
    await glyph.dragTo(page.locator("#stage"));
    await expect(page.locator("#undo-state")).toContainText("Add Glyph Current");
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
    await expect(page.locator("[data-file-root-select]")).toHaveCount(3);
    await expect(page.locator("#asset-path")).toContainText("City Source");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(2);
    await expect(page.locator("#place-asset")).toHaveText("OPEN");

    await page.locator('[data-file-root-select="audio"]').click();
    await expect(page.locator("#asset-path")).toContainText("Audio Library");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(3);

    await page
      .locator('.asset-tile[data-file-root="audio"][data-file-directory="Hits"]')
      .dblclick();
    await expect(page.locator("#asset-path")).toContainText("Hits");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(2);
    await expect(page.locator("#place-asset")).toHaveText("＋ INBOX");

    await page.locator("#file-parent").click();
    await expect(page.locator("#asset-path")).not.toContainText("Hits");
    await page.locator("#add-file-root").click();
    await expect(page.locator("#status-body")).toContainText(
      "Choose another base folder",
    );
  });
});
