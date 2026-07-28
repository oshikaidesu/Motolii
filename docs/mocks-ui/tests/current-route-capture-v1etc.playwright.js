import { expect, test } from "@playwright/test";

const CAPTURE_ROUTE = "/#plugin-browser-candidate";
const ARCHIVE_ROUTE = "/#archive/inbox-empty";

const EMPTY_STAGE_SELECTORS = [
  ".scene-copy",
  ".rings",
  ".selection-bounds",
  ".motion-path",
  ".stage-hud",
  ".stage-badge",
];

const PRESERVED_STAGE_SELECTORS = [
  "#stage",
  "#stage .frame",
  "#stage .frame .grid",
  "#stage .frame .effect-veil",
  "#stage .connection-preview",
  ".transport",
  "#play",
  "#time",
];

test.describe("G0-6H-V1ETC current-route-capture empty projection", () => {
  test("projects an empty Inspector and Stage without running the legacy script", async ({
    page,
  }) => {
    await page.goto(CAPTURE_ROUTE, { waitUntil: "domcontentloaded" });
    await page.locator("#stage").waitFor({ state: "visible" });
    await page.locator("#inspector").waitFor({ state: "attached" });

    expect(
      await page.locator(".app").getAttribute("data-parity-ready"),
    ).toBeNull();

    const inspector = page.locator("#inspector");
    await expect(inspector).toHaveCount(1);
    await expect(inspector).toHaveJSProperty("tagName", "ASIDE");
    await expect(inspector).toHaveClass("inspector");
    await expect(inspector).toHaveAttribute("id", "inspector");
    await expect(page.locator("#inspector > *")).toHaveCount(0);

    for (const selector of EMPTY_STAGE_SELECTORS) {
      await expect(page.locator(selector)).toHaveCount(0);
    }
    for (const selector of PRESERVED_STAGE_SELECTORS) {
      await expect(page.locator(selector)).toHaveCount(1);
    }
    await expect(page.locator("#stage")).toHaveAttribute(
      "data-mode",
      "installed",
    );
  });

  test("leaves a normal archive route unchanged in the same mode", async ({
    page,
  }) => {
    await page.goto(ARCHIVE_ROUTE, { waitUntil: "domcontentloaded" });
    await page
      .locator('.app[data-parity-ready="true"]')
      .waitFor({ state: "visible" });

    expect(await page.locator("#inspector > *").count()).toBeGreaterThan(0);

    for (const selector of EMPTY_STAGE_SELECTORS) {
      await expect(page.locator(selector)).toHaveCount(1);
    }
  });
});

test.describe("G0-6H-V1ETB current-route-capture Starter Media projection", () => {
  test("renders four development media tiles on the browser candidate route", async ({
    page,
  }) => {
    await page.goto(CAPTURE_ROUTE, { waitUntil: "domcontentloaded" });
    await page.locator("#project-browser").waitFor({ state: "visible" });

    await expect(page.locator(".browser-tabs button")).toHaveCount(3);
    await expect(page.locator('[data-tab="project"]')).toHaveClass(/(?:^|\s)on(?:\s|$)/);
    await expect(page.locator('[data-tab="effects"]')).not.toHaveClass(/(?:^|\s)on(?:\s|$)/);
    await expect(page.locator('[data-tab="create"]')).not.toHaveClass(/(?:^|\s)on(?:\s|$)/);
    await expect(page.locator("#project-browser")).toBeVisible();
    await expect(page.locator("#file-scope-toggle")).toHaveCount(1);
    await expect(page.locator("#asset-source-title")).toHaveText("All Media");
    await expect(page.locator("#asset-scope-label")).toHaveCount(1);
    await expect(page.locator("#asset-scope-label")).toHaveText("");
    await expect(page.locator("#asset-count")).toHaveText("4 ITEMS");
    await expect(page.locator("#asset-selection-count")).toHaveText("0 selected");

    const tiles = page.locator(".candidate-asset-results .asset-tile");
    await expect(tiles).toHaveCount(4);

    const expectedTiles = [
      { asset: "starter-clip.mp4", label: "starter-clip.mp4 · video/mp4", preview: "video" },
      { asset: "starter-mark.svg", label: "starter-mark.svg · image/svg+xml", preview: "logo" },
      { asset: "starter-still.png", label: "starter-still.png · image/png", preview: "texture" },
      { asset: "starter-tone.wav", label: "starter-tone.wav · audio/wav", preview: "audio" },
    ];

    for (let i = 0; i < expectedTiles.length; i += 1) {
      const tile = tiles.nth(i);
      const expected = expectedTiles[i];
      await expect(tile).toHaveAttribute("data-asset", expected.asset);
      await expect(tile).toHaveAttribute("aria-label", expected.label);
      await expect(tile.locator(".asset-preview")).toHaveClass(`asset-preview ${expected.preview}`);
    }
  });

  test("omits legacy browser chrome and selection affordances from the media results", async ({
    page,
  }) => {
    await page.goto(CAPTURE_ROUTE, { waitUntil: "domcontentloaded" });
    await page.locator("#project-browser").waitFor({ state: "visible" });

    const resultsScope = page.locator(".candidate-asset-results");
    await expect(resultsScope.locator(".is-selected")).toHaveCount(0);
    await expect(resultsScope.locator('[aria-pressed="true"]')).toHaveCount(0);
    await expect(resultsScope.locator("[data-asset-origin]")).toHaveCount(0);
    await expect(resultsScope.locator("[data-pack]")).toHaveCount(0);

    const absentSelectors = [
      "#vism-browser",
      "#candidate-pack-scope",
      "#candidate-save-sheet",
      "#add-file-root",
      "#add-media-tag",
      "#media-file-hierarchy",
      '[data-asset-source="project"]',
      "[data-media-recent]",
      "[data-file-root-select]",
      "[data-media-collection]",
      "[data-pack-select]",
    ];

    for (const selector of absentSelectors) {
      await expect(page.locator(selector)).toHaveCount(0);
    }
  });
});
