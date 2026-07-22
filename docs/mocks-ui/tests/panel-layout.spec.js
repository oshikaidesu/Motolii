import { expect, test } from "@playwright/test";

const REACT_URL = "http://127.0.0.1:5173/";

async function openCurrentCandidate(page) {
  await page.goto(`${REACT_URL}#plugin-browser-candidate`);
  await expect(
    page.locator('.app[data-resizable-layout="true"]'),
  ).toBeVisible();
}

test.describe("React panel layout", () => {
  test("uses the React candidate as the default and isolates legacy routes", async ({
    page,
  }) => {
    await page.goto(REACT_URL);
    await expect(
      page.getByRole("region", {
        name: "Plugin discovery / browser candidate",
        exact: true,
      }),
    ).toBeVisible();

    await page.goto(`${REACT_URL}#all-surfaces`);
    await expect(page.getByText("未登録のfixtureです: all-surfaces")).toBeVisible();
    await expect(page.locator(".app")).toHaveCount(0);

    await page.goto(`${REACT_URL}#archive/all-surfaces`);
    await expect(page.locator(".app")).toBeVisible();
    await expect(
      page.locator('.app[data-resizable-layout="true"]'),
    ).toHaveCount(0);
  });

  test("resizes Browser, Inspector, and Timeline independently", async ({
    page,
  }) => {
    await openCurrentCandidate(page);

    const browser = page.getByRole("separator", {
      name: "Browserのサイズを変更",
    });
    const inspector = page.getByRole("separator", {
      name: "Inspectorのサイズを変更",
    });
    const timeline = page.getByRole("separator", {
      name: "Timelineのサイズを変更",
    });

    await expect(browser).toHaveAttribute("aria-valuenow", "284");
    await expect(inspector).toHaveAttribute("aria-valuenow", "326");
    await expect(timeline).toHaveAttribute("aria-valuenow", "270");

    await browser.press("ArrowRight");
    await inspector.press("ArrowLeft");
    await timeline.press("ArrowUp");

    await expect(browser).toHaveAttribute("aria-valuenow", "300");
    await expect(inspector).toHaveAttribute("aria-valuenow", "342");
    await expect(timeline).toHaveAttribute("aria-valuenow", "286");

    await browser.dblclick();
    await inspector.press("Home");
    await timeline.dblclick();

    await expect(browser).toHaveAttribute("aria-valuenow", "284");
    await expect(inspector).toHaveAttribute("aria-valuenow", "326");
    await expect(timeline).toHaveAttribute("aria-valuenow", "270");
  });

  test("supports pointer drag while preserving the Stage minimum", async ({
    page,
  }) => {
    await openCurrentCandidate(page);
    const browser = page.getByRole("separator", {
      name: "Browserのサイズを変更",
    });
    const box = await browser.boundingBox();
    expect(box).not.toBeNull();

    await page.mouse.move(box.x + box.width / 2, box.y + 120);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2 + 72, box.y + 120, {
      steps: 5,
    });
    await page.mouse.up();

    await expect(browser).toHaveAttribute("aria-valuenow", "356");
    const stageWidth = await page
      .locator(".stage-shell")
      .evaluate((stage) => stage.getBoundingClientRect().width);
    expect(stageWidth).toBeGreaterThanOrEqual(440);
  });
});
