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

test("actual pointer drag, snap, cancel, marquee, and IME boundary", async ({ page }) => {
  await page.goto("/");
  const stage = page.getByTestId("interaction-stage");
  await stage.scrollIntoViewIfNeeded();
  const box = await stage.boundingBox();
  expect(box).not.toBeNull();

  const state = async () => JSON.parse(await page.getByTestId("interaction-state").textContent());
  await page.mouse.move(box.x + 45, box.y + 53);
  await page.mouse.down();
  await page.mouse.move(box.x + 118, box.y + 96, { steps: 8 });
  await page.mouse.up();
  await expect.poll(async () => (await state()).commits).toBe(1);
  expect((await state()).x % 10).toBe(0);
  expect((await state()).y % 10).toBe(0);

  await page.mouse.move(box.x + 115, box.y + 95);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width + 80, box.y + 180, { steps: 8 });
  await page.keyboard.press("Escape");
  await page.mouse.up();
  await expect.poll(async () => (await state()).cancels).toBe(1);
  expect((await state()).commits).toBe(1);

  await page.mouse.move(box.x + 20, box.y + 20);
  await page.mouse.down();
  await page.mouse.move(box.x + 250, box.y + 160, { steps: 8 });
  await page.mouse.up();
  const selection = await state();
  expect(selection.selected.length).toBeGreaterThan(0);
  await expect(page.getByRole("list", { name: "Selected keyframes" }).getByRole("button").first()).toBeVisible();
  expect(await page.getByTestId("selection-proxy").getByRole("button").count()).toBe(1);

  const captureProbe = page.getByTestId("capture-probe");
  const captureBox = await captureProbe.boundingBox();
  await page.mouse.move(captureBox.x + 20, captureBox.y + 20);
  await page.mouse.down();
  await page.mouse.move(captureBox.x + captureBox.width + 200, captureBox.y - 80, { steps: 6 });
  await page.mouse.up();
  await expect.poll(async () => JSON.parse(await page.getByTestId("capture-state").textContent()).captured).toBe(false);
  const capture = JSON.parse(await page.getByTestId("capture-state").textContent());
  expect(capture.moves).toBeGreaterThan(0);
  expect(capture.captured).toBe(false);

  const input = page.getByTestId("ime-input");
  await input.focus();
  await input.dispatchEvent("compositionstart", { data: "に" });
  await input.dispatchEvent("keydown", { key: "k", metaKey: true, isComposing: true });
  await input.dispatchEvent("compositionupdate", { data: "日本" });
  await input.dispatchEvent("compositionend", { data: "日本" });
  let ime = JSON.parse(await page.getByTestId("ime-state").textContent());
  expect(ime.shortcutCount).toBe(0);
  expect(ime.events).toEqual(["compositionstart", "compositionupdate", "compositionend"]);
  await input.press("Meta+k");
  ime = JSON.parse(await page.getByTestId("ime-state").textContent());
  expect(ime.shortcutCount).toBe(1);

  const evidence = process.env.G0_9_INTERACTION_EVIDENCE;
  if (evidence) {
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(evidence, `${JSON.stringify({
      ticket: "G0-9",
      capturedAt: new Date().toISOString(),
      environment: { platform: os.platform(), release: os.release(), arch: os.arch() },
      actualMouse: {
        konvaDrag: true,
        snappedCssPx: { x: selection.x, y: selection.y, grid: 10 },
        escapedAfterOffSurfaceMove: selection.cancels === 1,
        adapterCommits: selection.commits,
        marqueeSelected: selection.selected.length,
      },
      pointerCapture: { surfaceExitMoves: capture.moves, released: !capture.captured },
      accessibilityProxy: { domButtons: 1, selectedCount: selection.selected.length },
      syntheticCompositionGate: ime,
      boundaries: {
        motoliiD2Connected: false,
        rationalTimeSnapValidated: false,
        physicalImeValidated: false,
        voiceOverValidated: false,
        penValidated: false,
      },
    }, null, 2)}\n`);
  }
});

test("opaque community iframe exposes only the explicit message path", async ({ page }) => {
  await page.goto("/");
  const state = page.getByTestId("sandbox-state");
  await expect.poll(async () => JSON.parse(await state.textContent()).status).toBe("received");
  const result = JSON.parse(await state.textContent());
  expect(result).toEqual({
    status: "received",
    origin: "null",
    parentDocument: true,
    storage: true,
    network: true,
    nativeBridge: true,
  });
  await expect(page.getByTitle("Community panel sandbox").contentFrame()
    .getByRole("button", { name: "Community panel fixture" })).toBeVisible();
  const evidence = process.env.G0_9_SANDBOX_EVIDENCE;
  if (evidence) {
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(evidence, `${JSON.stringify({
      ticket: "G0-9",
      capturedAt: new Date().toISOString(),
      result,
      sandbox: "allow-scripts without allow-same-origin",
      csp: "harness inline script; default/connect/img none",
      boundaries: {
        separateWebViewValidated: false,
        nativeIpcNegativeValidated: false,
        loopCrashOomIsolationValidated: false,
      },
    }, null, 2)}\n`);
  }
});
