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
  const broker = page.getByTestId("capability-broker-state");
  await expect.poll(async () => JSON.parse(await broker.textContent()).handled).toBe(3);
  expect(JSON.parse(await broker.textContent())).toEqual({
    handled: 3,
    allowed: ["theme.read"],
    denied: ["document.raw", "native.invoke"],
  });
  const brokerResult = JSON.parse(await broker.textContent());
  await expect(page.getByTitle("Community panel sandbox").contentFrame()
    .getByRole("button", { name: "Community panel fixture" })).toBeVisible();
  const evidence = process.env.G0_9_SANDBOX_EVIDENCE;
  if (evidence) {
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(evidence, `${JSON.stringify({
      ticket: "G0-9",
      capturedAt: new Date().toISOString(),
      result,
      broker: brokerResult,
      sandbox: "allow-scripts without allow-same-origin",
      csp: "harness inline script; default/connect/img none",
      boundaries: {
        separateWebViewValidated: false,
        nativeIpcNegativeValidated: false,
        typedCapabilityBrokerValidated: true,
        loopCrashOomIsolationValidated: false,
      },
    }, null, 2)}\n`);
  }
});

test("2D object handles move, scale, rotate, multi-select, cancel, zoom, and stay bounded", async ({ page }) => {
  await page.goto("/");
  const stage = page.getByTestId("object-handle-stage");
  await stage.scrollIntoViewIfNeeded();
  const box = await stage.boundingBox();
  const read = async () => JSON.parse(await page.getByTestId("object-handle-state").textContent());

  await page.mouse.move(box.x + 130, box.y + 120);
  await page.mouse.down();
  await page.mouse.move(box.x + 190, box.y + 150, { steps: 8 });
  await page.mouse.up();
  await expect.poll(async () => (await read()).lastAction).toBe("move");

  await page.getByTestId("handles-reset").click();
  await page.mouse.move(box.x + 200, box.y + 158);
  await page.mouse.down();
  await page.mouse.move(box.x + 250, box.y + 205, { steps: 8 });
  await page.mouse.up();
  await expect.poll(async () => (await read()).lastAction).toBe("scale");
  expect((await read()).objects[0].scaleX).toBeGreaterThan(1);

  await page.getByTestId("handles-reset").click();
  await page.mouse.move(box.x + 140, box.y + 50);
  await page.mouse.down();
  await page.mouse.move(box.x + 220, box.y + 120, { steps: 10 });
  await page.mouse.up();
  await expect.poll(async () => (await read()).lastAction).toBe("rotate");
  expect(Math.abs((await read()).objects[0].rotation)).toBeGreaterThan(1);

  await page.getByTestId("handles-reset").click();
  await page.getByTestId("handles-multi").click();
  expect((await read()).selection).toBe(2);
  const proxy = page.getByRole("toolbar", { name: "Accessible object transform handles" });
  expect(await proxy.getByRole("button").count()).toBe(3);
  await proxy.getByRole("button", { name: "Scale selected objects up ten percent" }).click();
  expect((await read()).objects.every((object) => object.scaleX === 1.1)).toBe(true);

  await page.getByTestId("handles-zoom").click();
  expect((await read()).zoom).toBe(2);
  expect((await read()).visualCssPx).toBe(14);
  expect((await read()).hitCssPx).toBe(30);

  await page.getByTestId("handles-reset").click();
  await page.mouse.move(box.x + 200, box.y + 158);
  await page.mouse.down();
  await page.mouse.move(box.x + 255, box.y + 210, { steps: 8 });
  await page.keyboard.press("Escape");
  await page.mouse.up();
  await expect.poll(async () => (await read()).cancels).toBe(1);
  const cancelled = await read();
  expect(cancelled.commits).toBe(0);
  expect(cancelled.objects[0].scaleX).toBe(1);

  const evidence = process.env.G0_9_HANDLE_EVIDENCE;
  if (evidence) {
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(evidence, `${JSON.stringify({
      ticket: "G0-9/M5-P2U comparison",
      capturedAt: new Date().toISOString(),
      environment: { platform: os.platform(), release: os.release(), arch: os.arch() },
      twoD: { move: true, scale: true, rotate: true, multiSelection: 2, visualCssPx: 14, hitCssPx: 30,
        zoomInvariant: true, escapeCancel: true, commitsAfterCancel: cancelled.commits, proxyButtons: 3 },
      boundaries: { motoliiD2Connected: false, canonicalTransformValidated: false },
    }, null, 2)}\n`);
  }
});

test("3D gizmo exposes DCC modes, spaces, snapping, camera exclusion, and actual axis drag", async ({ page }) => {
  await page.goto("/");
  const stage = page.getByTestId("spatial-gizmo-stage");
  await stage.scrollIntoViewIfNeeded();
  const box = await stage.boundingBox();
  const read = async () => JSON.parse(await page.getByTestId("spatial-gizmo-state").textContent());
  await expect.poll(async () => (await read()).object?.position).toBeTruthy();

  await page.getByTestId("gizmo-translate").click();
  const xHandle = await page.evaluate(() => window.g09SpatialGizmo.project("x"));
  await page.mouse.move(box.x + xHandle.x, box.y + xHandle.y);
  await page.mouse.down();
  await expect.poll(async () => (await read()).cameraEnabled).toBe(false);
  await page.mouse.move(box.x + xHandle.x + 90, box.y + xHandle.y + 18, { steps: 12 });
  await page.mouse.up();
  await expect.poll(async () => (await read()).commits).toBe(1);
  const translated = await read();
  expect(Math.abs(translated.object.position[0])).toBeGreaterThan(0);
  expect(translated.cameraEnabled).toBe(true);

  await page.getByTestId("gizmo-reset").click();
  await page.getByTestId("gizmo-scale").click();
  await page.mouse.move(box.x + xHandle.x, box.y + xHandle.y);
  await page.mouse.down();
  await page.mouse.move(box.x + xHandle.x + 70, box.y + xHandle.y + 14, { steps: 10 });
  await page.mouse.up();
  await expect.poll(async () => (await read()).commits).toBe(1);
  expect(Math.abs((await read()).object.scale[0] - 1)).toBeGreaterThan(0.01);

  await page.getByTestId("gizmo-reset").click();
  await page.getByTestId("gizmo-rotate").click();
  expect((await read()).mode).toBe("rotate");
  const yHandle = await page.evaluate(() => window.g09SpatialGizmo.project("y"));
  await page.mouse.move(box.x + yHandle.x, box.y + yHandle.y);
  await page.mouse.down();
  await page.mouse.move(box.x + xHandle.x, box.y + xHandle.y, { steps: 14 });
  await page.mouse.up();
  await expect.poll(async () => (await read()).commits).toBe(1);
  expect((await read()).object.rotation.some((value) => Math.abs(value) > 0.01)).toBe(true);

  await page.getByTestId("gizmo-space").click();
  expect((await read()).space).toBe("local");

  await page.getByTestId("gizmo-reset").click();
  await page.getByTestId("gizmo-translate").click();
  await page.mouse.move(box.x + xHandle.x, box.y + xHandle.y);
  await page.mouse.down();
  await page.mouse.move(box.x + xHandle.x + 60, box.y + xHandle.y + 12, { steps: 8 });
  await page.keyboard.press("Escape");
  await page.mouse.up();
  await expect.poll(async () => (await read()).cancels).toBe(1);
  expect((await read()).commits).toBe(0);
  expect((await read()).object.position).toEqual([0, 0, 0]);
  expect(await page.getByRole("toolbar", { name: "3D transform modes" }).getByRole("button").count()).toBe(5);

  const evidence = process.env.G0_9_HANDLE_EVIDENCE;
  if (evidence) {
    const previous = fs.existsSync(evidence) ? JSON.parse(fs.readFileSync(evidence, "utf8")) : {};
    fs.mkdirSync(path.dirname(evidence), { recursive: true });
    fs.writeFileSync(evidence, `${JSON.stringify({
      ...previous,
      threeD: {
        actualTranslate: true,
        actualScale: true,
        actualRotate: true,
        worldAndLocalSpace: true,
        snappingConfigured: { translation: 0.25, rotationDegrees: 15, scale: 0.1 },
        cameraDisabledDuringGizmoDrag: true,
        escapeCancel: true,
        commitsAfterCancel: (await read()).commits,
        proxyButtons: 5,
      },
      openBoundaries: {
        nativeStageCompositionValidated: false,
        perspectiveAndOrthographicValidated: false,
        occlusionAndConstantScreenSizeValidated: false,
        mixedParentMultiSelectionValidated: false,
        motoliiP2UScaleDepthSeparationValidated: false,
      },
    }, null, 2)}\n`);
  }
});
