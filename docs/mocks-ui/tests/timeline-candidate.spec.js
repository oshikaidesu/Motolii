import { expect, test } from "@playwright/test";

const candidateUrl =
  `${process.env.MOTOLII_MOCK_URL ?? "http://127.0.0.1:5173/"}#plugin-browser-candidate`;

test.beforeEach(async ({ page }) => {
  await page.goto(candidateUrl);
  await expect(
    page.locator('[data-react-surface="timeline"]'),
  ).toBeVisible();
});

test("初期z=0を扇状展開せず件数付きstackで表示する", async ({
  page,
}) => {
  await page
    .getByRole("button", { name: "Depth Railを開く", exact: true })
    .click();

  const stack = page.getByRole("button", {
    name: /Depth 0 に4 Object/,
  });
  await expect(stack).toBeVisible();
  await expect(stack).toHaveText("0 × 4");
  await expect(
    page.locator(".candidate-depth-marker:not(.camera-marker)"),
  ).toHaveCount(1);

  await stack.hover();
  await expect(
    page.locator(".candidate-depth-marker:not(.camera-marker)"),
  ).toHaveCount(1);
  await expect(stack).toHaveAttribute("data-depth-count", "4");
});

test("Timeline選択は同期しDepth iconだけがRailを開く", async ({
  page,
}) => {
  const timeline = page.locator('[data-react-surface="timeline"]');
  const textBar = page.locator(
    '.candidate-time-bar[data-object-id="night-drive"]',
  );

  await textBar.evaluate((element) => {
    element.dispatchEvent(new MouseEvent("click", { bubbles: true }));
  });
  await expect(textBar).toHaveAttribute("data-selected", "true");
  await expect(timeline).not.toHaveClass(/depth-open/);

  await page
    .getByRole("button", { name: "NIGHT DRIVEのDepth Railを開く" })
    .click();
  await expect(timeline).toHaveClass(/depth-open/);
  await expect(
    page.getByRole("button", {
      name: /Depth 0 に4 Object。focus: NIGHT DRIVE/,
    }),
  ).toBeVisible();
});

test("同じparentの選択Objectを指定区間へLayer Orderで配布する", async ({
  page,
}) => {
  for (const id of ["night-drive", "city-loop", "traffic-pass"]) {
    await page
      .locator(`.candidate-time-bar[data-object-id="${id}"]`)
      .evaluate((element) => {
        element.dispatchEvent(
          new MouseEvent("click", {
            bubbles: true,
            shiftKey: true,
          }),
        );
      });
  }

  await page
    .getByRole("button", { name: "Depth Railを開く", exact: true })
    .click();
  const distribute = page.getByRole("button", {
    name: "Layer Order Distributeを開く",
  });
  await expect(distribute).toBeEnabled();
  await distribute.click();

  await expect(
    page.getByRole("slider", { name: "Depth配布の奥端" }),
  ).toHaveValue("-0.25");
  await expect(
    page.getByRole("slider", { name: "Depth配布の手前端" }),
  ).toHaveValue("0.25");
  await expect(
    page.locator(".candidate-depth-marker.is-preview"),
  ).toHaveCount(4);

  await page
    .getByRole("button", { name: "Depth配布順を反転" })
    .click();
  await page.getByRole("button", { name: "Depth配布を適用" }).click();

  await expect(
    page.locator(".candidate-depth-marker:not(.camera-marker)"),
  ).toHaveCount(4);
  await expect(
    page.locator(
      '.candidate-time-bar[data-object-id="pulse-rings"] .candidate-depth-value',
    ),
  ).toHaveText("z −.25");
  await expect(
    page.locator(
      '.candidate-time-bar[data-object-id="traffic-pass"] .candidate-depth-value',
    ),
  ).toHaveText("z +.25");
});

test("Group childはparent-local scopeへ切り替えてrootと混在させない", async ({
  page,
}) => {
  await page
    .getByRole("button", { name: "City gridのDepth Railを開く" })
    .click();

  await expect(
    page.getByRole("button", {
      name: "Depth scope: ROOT / Pulse rings",
    }),
  ).toBeVisible();
  await expect(
    page.locator(".candidate-depth-marker:not(.camera-marker)"),
  ).toHaveCount(1);
  await expect(
    page.getByRole("button", { name: "City grid · Depth 0" }),
  ).toBeVisible();
  await expect(page.locator(".camera-marker")).toHaveCount(0);
});

test("Timeline dock内でGraph Viewへ切り替え、同じ場所へ戻れる", async ({
  page,
}) => {
  const timeline = page.locator('[data-react-surface="timeline"]');
  const openGraph = page.getByRole("button", { name: "Open Graph View" });
  const openTimeline = page.getByRole("button", { name: "Open Timeline" });

  await expect(openTimeline).toHaveAttribute("aria-pressed", "true");
  await openGraph.click();

  const graph = timeline.locator('[data-react-surface="graph-view"]');
  await expect(graph).toBeVisible();
  await expect(graph).toHaveAttribute("data-docked", "true");
  await expect(openGraph).toHaveAttribute("aria-pressed", "true");
  await expect(timeline.locator(".candidate-timeline-body")).toHaveCount(0);
  const geometry = await graph.evaluate((element) => {
    const svg = element.querySelector(".graph-canvas");
    const key = element.querySelector(".graph-key");
    const svgRect = svg.getBoundingClientRect();
    const keyRect = key.getBoundingClientRect();
    const viewBox = svg.viewBox.baseVal;
    return {
      aspectDelta: Math.abs(
        svgRect.width / svgRect.height - viewBox.width / viewBox.height,
      ),
      keyRatio: keyRect.width / keyRect.height,
    };
  });
  expect(geometry.aspectDelta).toBeLessThan(0.02);
  expect(geometry.keyRatio).toBeGreaterThan(0.95);
  expect(geometry.keyRatio).toBeLessThan(1.05);

  await openTimeline.click();
  await expect(graph).toHaveCount(0);
  await expect(timeline.locator(".candidate-timeline-body")).toBeVisible();
  await expect(openTimeline).toHaveAttribute("aria-pressed", "true");
});

test("focus中のAutomation区間からInterval Easing Editorを開ける", async ({
  page,
}) => {
  const graphButton = page.getByRole("button", {
    name: "Pulse rings · IntensityのInterval Easing Editorを開く",
  });
  await expect(graphButton).toBeEnabled();

  await graphButton.click();

  const graph = page.getByRole("complementary", {
    name: "Interval Easing Editor",
  });
  await expect(graph).toBeVisible();
  await expect(graph).toHaveAttribute("aria-hidden", "false");
  await expect(graph.locator("#easing-target")).toHaveText(
    "Pulse rings · Intensity",
  );
});

test("固定Object列を作らずKey Toolsと一枚のpacking時間面を維持する", async ({
  page,
}) => {
  await expect(page.locator(".candidate-object-rail")).toHaveCount(0);
  await expect(page.getByText("OBJECTS", { exact: true })).toHaveCount(0);
  await expect(page.locator("#inbox")).not.toBeVisible();
  await expect(
    page.getByRole("complementary", { name: /Inbox/ }),
  ).toHaveCount(0);
  await expect(
    page.getByRole("complementary", { name: "Key Tools" }),
  ).toBeVisible();
  await expect(page.locator(".candidate-band-action-rail")).toBeVisible();
  await expect(page.locator(".candidate-band-action-row")).toHaveCount(5);
  await expect(page.locator(".candidate-pack-plane")).toHaveCount(1);
  await expect(page.locator(".candidate-pack-guides")).toBeVisible();
  await expect(page.locator(".candidate-pack-handle")).toHaveCount(0);
  await expect(
    page.getByRole("slider", { name: "barの太さ" }),
  ).toHaveCount(0);

  const controls = page.locator(".candidate-object-state");
  const count = await controls.count();
  expect(count).toBeGreaterThan(0);
  for (let index = 0; index < count; index += 1) {
    await expect(controls.nth(index).locator("xpath=..")).toHaveClass(
      /time-bar/,
    );
  }

  const alignment = await page.evaluate(() => {
    const rail = document.querySelector(".candidate-band-action-rail");
    const viewport = document.querySelector(".candidate-time-viewport");
    const head = document.querySelector(".candidate-band-action-head");
    const ruler = document.querySelector(".candidate-beat-ruler");
    const rows = [
      ...document.querySelectorAll(".candidate-band-action-row"),
    ];
    const guides = [
      ...document.querySelectorAll(".candidate-pack-guides i"),
    ];
    return {
      railRight: rail.getBoundingClientRect().right,
      viewportLeft: viewport.getBoundingClientRect().left,
      headBottom: head.getBoundingClientRect().bottom,
      rulerBottom: ruler.getBoundingClientRect().bottom,
      boundaries: rows.map((row, index) => ({
        rail: row.getBoundingClientRect().bottom,
        time: guides[index].getBoundingClientRect().bottom,
      })),
    };
  });
  expect(alignment.viewportLeft).toBe(alignment.railRight);
  expect(alignment.rulerBottom).toBe(alignment.headBottom);
  alignment.boundaries.forEach(({ rail, time }) => {
    expect(time).toBe(rail);
  });
});

test("各境界のresize領域が対応するpacking帯だけを更新する", async ({
  page,
}) => {
  const handles = page.getByRole("slider", {
    name: /packingレーン\d+の高さを調整/,
  });
  await expect(handles).toHaveCount(5);
  const handle = page.getByRole("slider", {
    name: "packingレーン2の高さを調整",
  });
  const rail = page.locator(".candidate-band-action-rail");
  const bars = page.locator(".candidate-time-bar");
  const handleBox = await handle.boundingBox();
  const railBox = await rail.boundingBox();
  const firstHeightBefore = await bars.nth(0).evaluate(
    (element) => element.getBoundingClientRect().height,
  );
  const thirdTopBefore = await bars.nth(2).evaluate(
    (element) => element.getBoundingClientRect().top,
  );
  const secondGuide = page.locator(".candidate-pack-guides i").nth(1);

  expect(handleBox).not.toBeNull();
  expect(railBox).not.toBeNull();
  expect(handleBox.width).toBeGreaterThanOrEqual(50);
  expect(handleBox.height).toBeGreaterThanOrEqual(12);
  expect(handleBox.x).toBeGreaterThanOrEqual(railBox.x);
  expect(handleBox.x + handleBox.width).toBeLessThanOrEqual(
    railBox.x + railBox.width,
  );

  await handle.hover();
  await expect(secondGuide).toHaveAttribute("data-active", "true");

  await page.mouse.move(
    handleBox.x + handleBox.width / 2,
    handleBox.y + handleBox.height / 2,
  );
  await page.mouse.down();
  await page.mouse.move(
    handleBox.x + handleBox.width / 2,
    handleBox.y + handleBox.height / 2 + 12,
  );
  await page.mouse.up();

  await expect(handle).toHaveAttribute("aria-valuenow", "46");
  await expect(bars.nth(1)).toHaveCSS("height", "40px");
  await expect(bars.nth(0)).toHaveCSS(
    "height",
    `${firstHeightBefore}px`,
  );
  const thirdTopAfter = await bars.nth(2).evaluate(
    (element) => element.getBoundingClientRect().top,
  );
  expect(thirdTopAfter - thirdTopBefore).toBe(12);

  const cityLoopTop = await page
    .locator('.candidate-time-bar[data-object-id="city-loop"]')
    .evaluate((element) => element.getBoundingClientRect().top);
  const trafficTop = await page
    .locator('.candidate-time-bar[data-object-id="traffic-pass"]')
    .evaluate((element) => element.getBoundingClientRect().top);
  expect(trafficTop).toBe(cityLoopTop);
});

test("Object bar内のSoloとMuteをpressed状態とbar投影へ反映する", async ({
  page,
}) => {
  const solo = page.getByRole("button", { name: "Pulse ringsをSolo" });
  const mute = page.getByRole("button", { name: "NIGHT DRIVEをMute" });
  const pulseBar = page.locator(
    '.candidate-time-bar[data-object-id="pulse-rings"]',
  );
  const textBar = page.locator(
    '.candidate-time-bar[data-object-id="night-drive"]',
  );

  await solo.click();
  await expect(solo).toHaveAttribute("aria-pressed", "true");
  await expect(pulseBar).toHaveAttribute("data-audible", "true");
  await expect(textBar).toHaveAttribute("data-audible", "false");

  await mute.click();
  await expect(mute).toHaveAttribute("aria-pressed", "true");
  await expect(textBar).toHaveAttribute("data-muted", "true");
});

test("帯のM/Sは帯状態を持たず現在の全Objectへ一括適用する", async ({
  page,
}) => {
  const bandMute = page.getByRole("button", {
    name: "帯5上の全ObjectをMute",
  });
  const cityMute = page.getByRole("button", {
    name: "neon_reflection.mp4をMute",
  });
  const trafficMute = page.getByRole("button", {
    name: "traffic_pass.mp4をMute",
  });

  await cityMute.click();
  await expect(bandMute).toHaveAttribute("aria-pressed", "mixed");

  await bandMute.click();
  await expect(bandMute).toHaveAttribute("aria-pressed", "true");
  await expect(cityMute).toHaveAttribute("aria-pressed", "true");
  await expect(trafficMute).toHaveAttribute("aria-pressed", "true");

  await bandMute.click();
  await expect(bandMute).toHaveAttribute("aria-pressed", "false");
  await expect(cityMute).toHaveAttribute("aria-pressed", "false");
  await expect(trafficMute).toHaveAttribute("aria-pressed", "false");
});

test("Groupを同じ時間面で折り畳み展開しM/Sをchildへ複製しない", async ({
  page,
}) => {
  const timeline = page.locator('[data-react-surface="timeline"]');
  const group = page.locator(
    '.candidate-time-bar[data-object-id="pulse-rings"]',
  );
  const child = page.locator(
    '.candidate-time-bar[data-object-id="city-grid"]',
  );
  const childMute = page.getByRole("button", {
    name: "City gridをMute",
  });
  const childSolo = page.getByRole("button", {
    name: "City gridをSolo",
  });
  const groupMute = page.getByRole("button", {
    name: "Pulse ringsをMute",
  });
  const groupSolo = page.getByRole("button", {
    name: "Pulse ringsをSolo",
  });
  const fold = page.getByRole("button", {
    name: "Pulse ringsを折り畳む",
  });
  const initialTimelineBox = await timeline.boundingBox();
  const initialHash = await page.evaluate(() => location.hash);

  await expect(group).toHaveAttribute("data-group-expanded", "true");
  await expect(child).toBeVisible();
  await expect(page.locator(".candidate-group-lane-bg")).toHaveCount(1);
  await expect(page.locator(".candidate-group-guide")).toHaveCount(1);
  expect(
    await group.evaluate((element) => getComputedStyle(element).backgroundColor),
  ).not.toBe(
    await child.evaluate((element) => getComputedStyle(element).backgroundColor),
  );

  await groupSolo.click();
  await expect(groupSolo).toHaveAttribute("aria-pressed", "true");
  await expect(childSolo).toHaveAttribute("aria-pressed", "false");
  await expect(child).toHaveAttribute("data-audible", "true");
  await groupSolo.click();

  await childSolo.click();
  await expect(childSolo).toHaveAttribute("aria-pressed", "true");
  await expect(groupSolo).toHaveAttribute("aria-pressed", "false");
  await expect(group).toHaveAttribute("data-audible", "true");
  await expect(child).toHaveAttribute("data-audible", "true");
  await childSolo.click();
  await childMute.click();

  await fold.click();
  await expect(group).toHaveAttribute("data-group-expanded", "false");
  await expect(child).toHaveCount(0);
  await expect(page.locator(".candidate-band-action-row")).toHaveCount(4);
  await expect(page.locator(".candidate-group-lane-bg")).toHaveCount(0);
  await expect(page.locator(".candidate-group-guide")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Pulse ringsを展開する" }),
  ).toBeVisible();

  await groupMute.click();
  await group.dblclick({ position: { x: 300, y: 12 } });
  await expect(group).toHaveAttribute("data-group-expanded", "true");
  await expect(child).toBeVisible();
  await expect(childMute).toHaveAttribute("aria-pressed", "true");
  await expect(groupMute).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator(".candidate-band-action-row")).toHaveCount(5);
  expect(await page.evaluate(() => location.hash)).toBe(initialHash);
  expect(await timeline.boundingBox()).toEqual(initialTimelineBox);
});

test("Automation済みchannelを縦一覧へ展開し選択KeyをKeystone型操作面で扱う", async ({
  page,
}) => {
  const pulseTrigger = page.getByRole("button", {
    name: "Pulse ringsのAutomationを開く · 3 channel",
  });
  const pulseStack = page.locator(
    '.candidate-automation-stack[data-object-id="pulse-rings"]',
  );

  await pulseTrigger.click();
  await expect(pulseStack).toBeVisible();
  await expect(pulseStack.locator(".candidate-automation-row")).toHaveCount(3);
  await expect(pulseStack.getByText("Intensity", { exact: true })).toBeVisible();
  await expect(pulseStack.getByText("Spread", { exact: true })).toBeVisible();
  await expect(pulseStack.getByText("Depth", { exact: true })).toBeVisible();
  await expect(pulseStack.getByText("Scale", { exact: true })).toHaveCount(0);
  await expect(
    page.locator(".candidate-automation-parameter-row"),
  ).toHaveCount(0);

  const intensityKey = page.getByRole("button", {
    name: "Pulse rings · Intensity · Key 1",
  });
  const spreadKey = page.getByRole("button", {
    name: "Pulse rings · Spread · Key 1",
  });
  const intensityEasing = await intensityKey.getAttribute("data-easing");
  const spreadEasing = await spreadKey.getAttribute("data-easing");
  await intensityKey.click();
  await spreadKey.click();

  const tools = page.getByRole("complementary", { name: "Key Tools" });
  await expect(tools).toBeVisible();
  await expect(tools.getByText("◆ 2")).toBeVisible();
  await tools.getByRole("button", { name: "全選択" }).click();
  await tools.getByRole("button", { name: "開始へ整列" }).click();
  const intensityLeft = await intensityKey.evaluate(
    (element) => element.style.left,
  );
  const spreadLeft = await spreadKey.evaluate(
    (element) => element.style.left,
  );
  expect(intensityLeft).toBe(spreadLeft);
  expect(await intensityKey.getAttribute("data-easing")).toBe(intensityEasing);
  expect(await spreadKey.getAttribute("data-easing")).toBe(spreadEasing);

  await pulseTrigger.click();
  await expect(pulseStack).toHaveCount(0);

  const cityTrigger = page.getByRole("button", {
    name: "neon_reflection.mp4のAutomationを開く · 0 channel",
  });
  await cityTrigger.click();
  const cityMenu = page.getByRole("dialog", {
    name: "neon_reflection.mp4のAutomation",
  });
  const search = cityMenu.getByRole("searchbox", {
    name: "Automation channelを検索",
  });
  await expect(search).toBeVisible();
  await expect(
    cityMenu.getByRole("button", { name: /Scale/ }),
  ).toHaveCount(0);

  await search.fill("scale");
  await cityMenu.getByRole("button", { name: /Scale/ }).click();
  await expect(
    page.getByRole("button", {
      name: "neon_reflection.mp4のAutomationを開く · 1 channel",
    }),
  ).toBeVisible();
  await expect(
    page.locator(
      '.candidate-automation-stack[data-object-id="city-loop"] .candidate-automation-row[data-channel="Scale"]',
    ),
  ).toHaveCount(1);
});

test("Key ToolsをTimeline左端dockでKEYSとLAYERSへ排他的に切り替える", async ({
  page,
}) => {
  const tools = page.getByRole("complementary", { name: "Key Tools" });
  const rail = page.locator(".candidate-band-action-rail");
  const viewport = page.locator(".candidate-time-viewport");
  const toolsBox = await tools.boundingBox();
  const railBox = await rail.boundingBox();
  const viewportBox = await viewport.boundingBox();
  expect(toolsBox).not.toBeNull();
  expect(railBox).not.toBeNull();
  expect(viewportBox).not.toBeNull();
  expect(toolsBox.x + toolsBox.width).toBe(railBox.x);
  expect(railBox.x + railBox.width).toBe(viewportBox.x);

  const keysMode = tools.getByRole("button", { name: "KEYS" });
  const layersMode = tools.getByRole("button", { name: "LAYERS" });
  await expect(keysMode).toHaveAttribute("aria-pressed", "true");
  await expect(
    tools.getByRole("button", { name: "Align", exact: true }),
  ).toBeVisible();
  await expect(
    tools.getByRole("button", { name: "Layer Align" }),
  ).toHaveCount(0);

  await layersMode.click();
  await expect(layersMode).toHaveAttribute("aria-pressed", "true");
  await expect(
    tools.getByRole("button", { name: "Layer Align" }),
  ).toBeVisible();
  await expect(
    tools.getByRole("button", { name: "Align", exact: true }),
  ).toHaveCount(0);

  const pulseBar = page.locator(
    '.candidate-time-bar[data-object-id="pulse-rings"]',
  );
  const textBar = page.locator(
    '.candidate-time-bar[data-object-id="night-drive"]',
  );
  await page.keyboard.down("Shift");
  await textBar.click({ position: { x: 120, y: 10 } });
  await page.keyboard.up("Shift");
  await expect(pulseBar).toHaveAttribute("data-selected", "true");
  await expect(textBar).toHaveAttribute("data-selected", "true");
  await expect(tools.getByText("▤ 2")).toBeVisible();

  await keysMode.click();
  await expect(keysMode).toHaveAttribute("aria-pressed", "true");
  await expect(pulseBar).toHaveAttribute("data-selected", "true");
  await expect(textBar).toHaveAttribute("data-selected", "true");
});

test("横時間軸は左Key Toolsを固定したままscrollできる", async ({
  page,
}) => {
  const viewport = page.locator(".candidate-time-viewport");
  const tools = page.getByRole("complementary", { name: "Key Tools" });
  const initialToolsX = await tools.evaluate((element) =>
    element.getBoundingClientRect().x,
  );

  await expect
    .poll(() =>
      viewport.evaluate(
        (element) => element.scrollWidth > element.clientWidth,
      ),
    )
    .toBe(true);
  await viewport.evaluate((element) => {
    element.scrollLeft = 320;
    element.dispatchEvent(new Event("scroll"));
  });
  await expect
    .poll(() => viewport.evaluate((element) => element.scrollLeft))
    .toBeGreaterThan(0);
  await expect
    .poll(() =>
      tools.evaluate((element) => element.getBoundingClientRect().x),
    )
    .toBe(initialToolsX);
});
