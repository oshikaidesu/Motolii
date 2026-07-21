import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { expect, test } from "@playwright/test";

test("G0-9 dense Web UI evidence", async ({ page, request, browserName }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Motolii G0-9 isolated Web UI spike" })).toBeVisible();

  const initialRows = await page.locator("[data-asset-id]").count();
  expect(initialRows).toBeLessThan(64);
  await page.getByTestId("browser").evaluate((element) => {
    element.scrollTop = element.scrollHeight;
    element.dispatchEvent(new Event("scroll"));
  });
  await expect(page.locator('[data-asset-id="asset-09999"]')).toBeVisible();
  await page.locator('[data-asset-id="asset-09999"]').click();
  await expect(page.getByTestId("browser-selection")).toHaveText("asset-09999");
  const browser = await page.evaluate(() => window.g09.measureBrowser(120));
  expect(browser.domRows).toBeLessThan(64);
  await expect(page.getByTestId("browser-selection")).toHaveText("asset-09999");

  const canvas = await page.evaluate(() => window.g09.measureTimeline("canvas2d", 120));
  expect(canvas.visibleKeys).toBeGreaterThan(10_000);

  const dynamicSceneInit = await page.evaluate(() => window.g09.initializeDynamicScenes());
  expect(dynamicSceneInit.visibleKeys).toBe(20_000);
  expect(dynamicSceneInit.pixiNonBackgroundPixels).toBeGreaterThan(10_000);
  const dynamicDrag = await page.evaluate(() => window.g09.measureDynamicScenes(90));
  for (const result of [...dynamicDrag.pixi, ...dynamicDrag.konva]) {
    expect(result.semanticWritesDuringMove).toBe(0);
    expect(result.cancelRestored).toBe(true);
    expect(result.commitsOnRelease).toBe(1);
    expect(Number.isFinite(result.p95Ms)).toBe(true);
  }

  const webgpuInit = await page.evaluate(() => window.g09.initializeWebGpu());
  const webgpu = webgpuInit.available
    ? await page.evaluate(() => window.g09.measureTimeline("webgpu", 120))
    : null;

  const revisionBefore = Number(await page.getByTestId("hmr-revision").textContent());
  const hmrStarted = Date.now();
  const response = await request.get("/__g0_9_hmr_tick");
  expect(response.ok()).toBeTruthy();
  await expect(page.getByTestId("hmr-revision")).toHaveText(String(revisionBefore + 1));
  const hmrMs = Date.now() - hmrStarted;

  const report = {
    ticket: "G0-9",
    capturedAt: new Date().toISOString(),
    environment: {
      platform: os.platform(),
      release: os.release(),
      arch: os.arch(),
      browser: browserName,
      userAgent: await page.evaluate(() => navigator.userAgent),
    },
    fixture: await page.evaluate(() => window.g09.fixture),
    browserVirtualization: browser,
    timelineCanvas2d: canvas,
    dynamicSceneInit,
    dynamicDrag,
    browserWebGpu: {
      ...webgpuInit,
      measurement: webgpu,
    },
    hmr: {
      transport: "Vite accepted virtual-module update without Rust process restart",
      elapsedMs: hmrMs,
      revisionBefore,
      revisionAfter: revisionBefore + 1,
    },
    boundaries: {
      nativeWgpuTextureSharedWithBrowser: false,
      productWebViewEmbedded: false,
      pluginSandboxValidated: false,
    },
  };

  const evidence = process.env.G0_9_EVIDENCE;
  if (evidence) {
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(evidence, `${JSON.stringify(report, null, 2)}\n`);
  }
});
