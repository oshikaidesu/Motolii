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

test.describe("G0-6H-V1ETT current-route-capture Timeline projection", () => {
  test("renders empty projection on timeline and keeps project/automation content hidden", async ({
    page,
  }) => {
    const pageErrors = [];
    page.on("pageerror", (error) => {
      pageErrors.push(error);
    });

    await page.goto(CAPTURE_ROUTE, { waitUntil: "domcontentloaded" });
    await expect(page.locator("#timeline")).toBeVisible();
    expect(pageErrors).toHaveLength(0);

    const emptyTimelineSelectors = [
      ".candidate-time-bar",
      "#vism-clip",
      "[data-object-id]",
      ".candidate-object-state",
      ".candidate-sm",
      ".candidate-group-fold",
      ".candidate-group-lane-bg",
      ".candidate-group-guide",
      ".candidate-automation-trigger",
      ".candidate-automation-stack",
      ".candidate-automation-row",
      ".candidate-automation-key",
      ".candidate-automation-add-row",
      ".candidate-automation-menu",
      ".candidate-depth-open",
      ".candidate-depth-value",
      "#depth-rail",
      ".z-rail",
      "#depth-key",
      "#z-axis",
      "#z-readout",
      ".candidate-depth-scope",
      ".candidate-band-action-rail",
      ".candidate-band-action-row",
      ".candidate-band-sm",
      ".candidate-pack-resize-zone",
      '[data-selected="true"]',
    ];

    for (const selector of emptyTimelineSelectors) {
      await expect(page.locator(selector)).toHaveCount(0);
    }

    const timelineText = await page.locator("#timeline").innerText();
    expect(timelineText).not.toContain("Pulse rings");
    expect(timelineText).not.toContain("NIGHT DRIVE");
    expect(timelineText).not.toContain("City grid");
    expect(timelineText).not.toContain("night_drive.wav");
    expect(timelineText).not.toContain("neon_reflection.mp4");
    expect(timelineText).not.toContain("traffic_pass.mp4");
    expect(pageErrors).toHaveLength(0);
  });

  test("preserves chrome and keyboard switching for timeline keys/layers controls", async ({
    page,
  }) => {
    const pageErrors = [];
    page.on("pageerror", (error) => {
      pageErrors.push(error);
    });

    await page.goto(CAPTURE_ROUTE, { waitUntil: "domcontentloaded" });
    await expect(page.locator("#timeline")).toHaveCount(1);
    expect(pageErrors).toHaveLength(0);

    await expect(page.locator("#timeline[data-react-surface='timeline']")).toHaveCount(
      1,
    );
    await expect(page.locator(".candidate-timeline-head")).toHaveCount(1);
    await expect(page.locator("#depth-toggle")).toHaveCount(1);
    await expect(page.locator(".candidate-timeline-view-switch")).toHaveCount(1);
    await expect(page.locator(".candidate-beat-ruler")).toHaveCount(1);
    await expect(page.locator(".candidate-beat-ruler span")).toHaveCount(10);
    await expect(page.locator("#playhead")).toHaveCount(1);
    await expect(page.locator("#playhead")).toHaveAttribute(
      "style",
      /left:\s*62.5%/,
    );
    await expect(page.locator(".candidate-pack-guides")).toHaveCount(1);
    await expect(page.locator(".candidate-pack-guides i")).toHaveCount(5);
    await expect(page.locator(".candidate-key-tools")).toHaveCount(1);

    await expect(
      page.locator(".candidate-key-mode button[aria-pressed='true']"),
    ).toHaveText("KEYS");
    await expect(page.locator(".candidate-key-tools-head b")).toHaveText("◆ 0");
    await expect(
      page.locator(".candidate-key-scope button[aria-pressed='true']"),
    ).toHaveAttribute("aria-label", "Object別");

    const layers = page
      .locator(".candidate-key-mode")
      .locator("button")
      .filter({ hasText: "LAYERS" });
    await expect(layers).toHaveCount(1);
    await layers.focus();
    await expect(layers).toBeFocused();
    await layers.press("Enter");

    await expect(
      page.locator(".candidate-key-mode button[aria-pressed='true']"),
    ).toHaveText("LAYERS");
    await expect(page.locator(".candidate-key-tools-head b")).toHaveText("▤ 0");
    expect(pageErrors).toHaveLength(0);
  });
});
