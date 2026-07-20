import { expect, test } from "@playwright/test";

const candidateUrl =
  `${process.env.MOTOLII_MOCK_URL ?? "http://127.0.0.1:5173/"}#graph-view-candidate`;

test.beforeEach(async ({ page }) => {
  await page.goto(candidateUrl);
  await expect(
    page.locator('[data-react-surface="graph-view"]'),
  ).toBeVisible();
});

test("実時間×実値のprimary curveとcontext curveを別surfaceで表示する", async ({
  page,
}) => {
  const graph = page.locator('[data-react-surface="graph-view"]');
  await expect(graph).toHaveAttribute("data-view-time", "52-56");
  await expect(graph).toHaveAttribute("data-view-value", "0-100");
  await expect(
    page.getByRole("complementary", { name: "Animated parameters" }),
  ).toBeVisible();
  await expect(page.locator(".graph-primary-curve")).toHaveCount(1);
  await expect(page.locator(".graph-context-curve")).toHaveCount(2);
  await expect(
    page.getByRole("button", { name: "Open Interval Easing Editor" }),
  ).toBeVisible();
});

test("snapshotとFrame Selectedは明示操作でview rangeを変えない", async ({
  page,
}) => {
  const graph = page.locator('[data-react-surface="graph-view"]');
  const rangeBefore = await graph.evaluate((element) => ({
    time: element.dataset.viewTime,
    value: element.dataset.viewValue,
  }));

  await page.getByRole("button", { name: "Curve snapshot" }).click();
  await expect(page.locator(".graph-snapshot-curve")).toBeVisible();
  await expect(page.getByRole("status")).toContainText("view only");

  await page.getByRole("button", { name: "Frame selected" }).click();
  await expect(page.getByRole("status")).toContainText("Frame selected");
  await expect(graph).toHaveAttribute("data-view-time", rangeBefore.time);
  await expect(graph).toHaveAttribute("data-view-value", rangeBefore.value);
});

test("curve上へのkey追加は表示shapeを維持する", async ({ page }) => {
  const curve = page.locator('[data-testid="active-curve"]');
  const keysBefore = await page.locator(".graph-key").count();
  const samplesBefore = await curve.evaluate((path) => {
    const length = path.getTotalLength();
    return Array.from({ length: 31 }, (_, index) => {
      const point = path.getPointAtLength((length * index) / 30);
      return [point.x, point.y];
    });
  });

  await curve.evaluate((path) => {
    const point = path.getPointAtLength(path.getTotalLength() * 0.52);
    const svg = path.ownerSVGElement;
    const svgPoint = svg.createSVGPoint();
    svgPoint.x = point.x;
    svgPoint.y = point.y;
    const screen = svgPoint.matrixTransform(svg.getScreenCTM());
    path.dispatchEvent(
      new MouseEvent("dblclick", {
        bubbles: true,
        clientX: screen.x,
        clientY: screen.y,
      }),
    );
  });

  await expect(page.locator(".graph-key")).toHaveCount(keysBefore + 1);
  await expect(page.getByRole("status")).toContainText("curve preserved");
  const samplesAfter = await curve.evaluate((path) => {
    const length = path.getTotalLength();
    return Array.from({ length: 31 }, (_, index) => {
      const point = path.getPointAtLength((length * index) / 30);
      return [point.x, point.y];
    });
  });
  samplesAfter.forEach((point, index) => {
    expect(Math.abs(point[0] - samplesBefore[index][0])).toBeLessThan(0.25);
    expect(Math.abs(point[1] - samplesBefore[index][1])).toBeLessThan(0.25);
  });
});

test("handle drag中にauto-fitせずreleaseで1 Undoになる", async ({
  page,
}) => {
  const graph = page.locator('[data-react-surface="graph-view"]');
  const handle = page.locator('[data-handle="i1-out"]');
  const box = await handle.boundingBox();
  const rangeBefore = await graph.getAttribute("data-view-value");

  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + 70, box.y - 45, { steps: 4 });
  await expect(graph).toHaveAttribute("data-view-value", rangeBefore);
  await page.mouse.up();

  await expect(page.getByRole("status")).toContainText("Undo 1");
});

test("Shiftは反対tangentを動かさずEscはgesture開始時へ戻す", async ({
  page,
}) => {
  const incoming = page.locator('[data-handle="i1-in"]');
  const outgoing = page.locator('[data-handle="i1-out"]');
  const incomingBefore = {
    x: await incoming.getAttribute("cx"),
    y: await incoming.getAttribute("cy"),
  };
  const outgoingBefore = {
    x: await outgoing.getAttribute("cx"),
    y: await outgoing.getAttribute("cy"),
  };
  const box = await outgoing.boundingBox();

  await page.keyboard.down("Shift");
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + 55, box.y - 36, { steps: 4 });
  await page.keyboard.up("Shift");

  await expect(incoming).toHaveAttribute("cx", incomingBefore.x);
  await expect(incoming).toHaveAttribute("cy", incomingBefore.y);
  await expect(outgoing).not.toHaveAttribute("cx", outgoingBefore.x);

  await page.keyboard.press("Escape");
  await expect(outgoing).toHaveAttribute("cx", outgoingBefore.x);
  await expect(outgoing).toHaveAttribute("cy", outgoingBefore.y);
  await expect(page.getByRole("status")).toContainText("no changes");
});
