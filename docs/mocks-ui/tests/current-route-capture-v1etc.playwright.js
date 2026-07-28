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
