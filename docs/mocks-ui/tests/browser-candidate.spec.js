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
    const firstCard = page.locator(".candidate-plugin-card").first();
    const defaultBox = await firstCard.boundingBox();
    expect(Math.abs(defaultBox.width - defaultBox.height)).toBeLessThan(1);

    await page.getByRole("button", { name: "Settings" }).click();
    await page
      .getByRole("dialog", { name: "Settings" })
      .getByRole("button", { name: "Browser" })
      .click();
    const thumbnailSize = page.locator("#plugin-thumb-size");
    await expect(thumbnailSize).toBeVisible();
    await thumbnailSize.evaluate((control) => {
      control.value = "64";
      control.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await page.getByRole("button", { name: "Done" }).click();
    const smallBox = await firstCard.boundingBox();
    expect(Math.abs(smallBox.width - smallBox.height)).toBeLessThan(1);

    await thumbnailSize.evaluate((control) => {
      control.value = "160";
      control.dispatchEvent(new Event("input", { bubbles: true }));
    });
    const largeBox = await firstCard.boundingBox();
    expect(Math.abs(largeBox.width - largeBox.height)).toBeLessThan(1);
    expect(largeBox.width).toBeGreaterThan(smallBox.width + 20);

    await thumbnailSize.evaluate((control) => {
      control.value = "80";
      control.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await page
      .getByRole("button", { name: "Thumbnail and name view" })
      .click();
    const namedThumbnailBox = await firstCard.locator(".plugin-thumb").boundingBox();
    expect(namedThumbnailBox.width / namedThumbnailBox.height).toBeGreaterThan(1.7);

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

  test("uses list taxonomy as contextual result navigation", async ({
    page,
  }) => {
    await openCandidate(page);
    await page.getByRole("button", { name: "List view" }).click();

    const echoType = page.getByRole("navigation", { name: "Echo Bloom type" });
    await expect(echoType).toBeVisible();
    await expect(echoType).toContainText("Effect");
    await expect(echoType).toContainText("Light");

    await echoType.getByRole("button", { name: "Light" }).click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(1);
    await expect(page.locator("#plugin-result-count")).toHaveText("1");
    await expect(page.locator("#plugin-taxonomy-clear")).toHaveText("Light ×");

    await page.locator("#plugin-taxonomy-clear").click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(4);
    await expect(page.locator("#plugin-result-count")).toHaveText("4");

    const glyphType = page.getByRole("navigation", {
      name: "Glyph Current type",
    });
    await glyphType.getByRole("button", { name: "Generator" }).click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(2);
    await page.locator('[data-plugin-source="all"]').click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(4);
    await expect(page.locator("#plugin-taxonomy-clear")).toBeHidden();
  });

  test("uses the same search, source rail, and result grid for Project and Files", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(page.locator(".browser-tabs .browser-tab")).toHaveText([
      "Assets",
      "Files",
      "Plugins",
    ]);
    await page.getByRole("button", { name: "Assets" }).click();
    await expect(page.locator(".candidate-project-browser")).toBeVisible();
    await expect(page.getByRole("searchbox", { name: "Search assets" })).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Asset sources" })).toBeVisible();
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(4);
    await expect(page.locator("[data-asset-source]")).toHaveCount(0);

    await page.getByRole("button", { name: "Files" }).click();
    await expect(page.locator("[data-file-root-select]")).toHaveCount(3);
    await expect(page.locator("#asset-path")).toContainText("City Source");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(2);
    await expect(page.locator("#place-asset")).toHaveText("OPEN");
    const hierarchyRows = page.locator("#file-tree button");
    await expect(hierarchyRows).toHaveCount(2);
    await expect(hierarchyRows.nth(0)).toContainText("L0");
    await expect(hierarchyRows.nth(1)).toContainText("L1");
    const rootRow = await hierarchyRows.nth(0).boundingBox();
    const childRow = await hierarchyRows.nth(1).boundingBox();
    expect(Math.abs(rootRow.x - childRow.x)).toBeLessThan(1);

    await hierarchyRows.nth(1).click();
    await expect(page.locator("#asset-path")).toContainText("MV");
    await expect(page.locator("#file-tree button.current")).toContainText("MV");
    await expect(page.locator("#file-tree")).toContainText("night_drive");

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
