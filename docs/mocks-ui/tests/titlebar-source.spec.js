import { expect, test } from "@playwright/test";

const SKELETON_URL = "http://127.0.0.1:5173/#skeleton";

test.describe("fixed React titlebar source", () => {
  test("preserves structure, accessible actions, focus order, and layout role", async ({
    page,
  }) => {
    await page.goto(SKELETON_URL);

    const titlebar = page.locator(".mock-titlebar");
    await expect(titlebar).toHaveCount(1);
    await expect(titlebar.locator(".wordmark")).toHaveText("MOTOLII");
    await expect(titlebar.locator(":scope > span").nth(0)).toHaveText(
      "night_drive.mv",
    );

    const actions = titlebar.getByRole("button");
    await expect(actions).toHaveText(["Settings", "Export"]);
    await expect(actions.nth(0)).toHaveAttribute("type", "button");
    await expect(actions.nth(1)).toHaveAttribute("type", "button");

    await actions.nth(0).focus();
    await expect(actions.nth(0)).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(actions.nth(1)).toBeFocused();

    await expect(titlebar).toHaveCSS("display", "flex");
    const spacerMargin = await titlebar
      .locator(".mock-grow")
      .evaluate((element) => Number.parseFloat(getComputedStyle(element).marginLeft));
    expect(spacerMargin).toBeGreaterThan(0);
  });
});
