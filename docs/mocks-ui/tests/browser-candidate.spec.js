import { expect, test } from "@playwright/test";

const CANDIDATE_URL =
  "http://127.0.0.1:5173/#plugin-browser-candidate";

async function openCandidate(page) {
  await page.goto(CANDIDATE_URL, { waitUntil: "domcontentloaded" });
  await page
    .locator('.app[data-parity-ready="true"]')
    .waitFor({ state: "visible" });
}

test.describe("shared discovery Browser candidate", () => {
  test("uses a React easing surface with AM interval actions and channel context", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(
      page.locator('[data-react-surface="easing-graph"]'),
    ).toHaveCount(1);
    await expect(
      page.locator('.inline-key[data-key-context="current"]'),
    ).toHaveCount(3);
    await expect(
      page.locator('.inline-key[data-key-context="context"]'),
    ).toHaveCount(2);

    await page
      .getByRole("button", {
        name: "Pulse rings · IntensityのInterval Easing Editorを開く",
      })
      .click();
    const panel = page.getByRole("complementary", {
      name: "Interval Easing Editor",
    });
    await expect(panel).toBeVisible();
    // bridgeのlegacy scriptが旧座標系で同じDOMを書き換えても、候補側は
    // click後にReact stateと473×499の座標系を必ず取り戻す。
    await expect(panel.locator("#ease-x1")).toHaveText("0.40");
    await expect(panel.locator("#ease-y1")).toHaveText("0.00");
    await expect(panel.locator("#ease-x2")).toHaveText("0.20");
    await expect(panel.locator("#ease-y2")).toHaveText("1.00");
    await expect(panel.locator("#graph-curve")).toHaveAttribute(
      "d",
      /^M30\.0,[\d.]+ C195\.2,[\d.]+ 112\.6,[\d.]+ 443\.0,[\d.]+$/,
    );
    await panel.getByRole("button", { name: "Ease In", exact: true }).click();
    await expect(panel.locator("#ease-x1")).toHaveText("0.42");
    await expect(panel.locator("#ease-y1")).toHaveText("0.00");
    await expect(panel.locator("#ease-x2")).toHaveText("1.00");
    await expect(panel.locator("#ease-y2")).toHaveText("1.00");
    await panel.getByRole("button", { name: "Easing actions" }).click();
    await expect(
      panel.getByRole("button", { name: "Overshoot mode" }),
    ).toHaveAttribute("aria-pressed", "false");
    await panel.getByRole("button", { name: "Easing actions" }).click();
    for (const name of [
      "Bounce",
      "Elastic",
      "Cyclic · Sine",
      "Random",
      "Steps",
      "Elastic Steps",
    ]) {
      await expect(
        panel.getByRole("button", { name, exact: true }),
      ).toBeVisible();
    }
    const confirmedHandles = [
      ["Bounce", [["button", "FIRST DIP handle"]]],
      [
        "Elastic",
        [
          ["slider", "OVERSHOOT LIMIT handle"],
          ["button", "WAVE SIZE handle"],
        ],
      ],
      [
        "Cyclic · Sine",
        [
          ["slider", "PERIOD handle"],
          ["slider", "PEAK POSITION handle"],
          ["slider", "SMOOTHNESS handle"],
          ["slider", "LOWER LIMIT handle"],
        ],
      ],
      [
        "Random",
        [
          ["slider", "SEED handle"],
          ["slider", "SIZE handle"],
          ["button", "HEIGHT handle"],
          ["slider", "CURVE BIAS handle"],
        ],
      ],
      [
        "Steps",
        [
          ["slider", "STEP WIDTH handle"],
          ["slider", "SMOOTHNESS handle"],
        ],
      ],
      [
        "Elastic Steps",
        [
          ["slider", "STEP WIDTH handle"],
          ["slider", "ELASTICITY handle"],
        ],
      ],
    ];
    for (const [curve, handles] of confirmedHandles) {
      await panel
        .getByRole("button", { name: curve, exact: true })
        .click();
      for (const [role, handle] of handles) {
        await expect(
          panel.getByRole(role, { name: handle, exact: true }),
        ).toBeVisible();
      }
    }

    const keyShapeBefore = await page
      .locator(".inline-key")
      .evaluateAll((keys) =>
        keys.map((key) => ({
          channel: key.dataset.channel,
          left: key.style.left,
          filled: key.classList.contains("filled"),
        })),
      );
    const intervalStart = page.locator(
      ".time-bar.selected .inline-key.key-selected",
    ).first();
    await panel
      .getByRole("button", { name: "Elastic", exact: true })
      .click();
    await expect(
      panel.getByRole("button", { name: "Overshoot mode" }),
    ).toHaveAttribute("aria-pressed", "true");
    const waveSizeHandle = panel.getByRole("button", {
      name: "WAVE SIZE handle",
      exact: true,
    });
    const elasticPathBefore = await panel
      .locator("#graph-curve")
      .getAttribute("d");
    const elasticParamsBefore = await intervalStart.getAttribute(
      "data-preview-params",
    );
    const amplitudeBox = await waveSizeHandle.boundingBox();
    await page.mouse.move(
      amplitudeBox.x + amplitudeBox.width / 2,
      amplitudeBox.y + amplitudeBox.height / 2,
    );
    await page.mouse.down();
    await page.mouse.move(
      amplitudeBox.x + amplitudeBox.width / 2,
      amplitudeBox.y - 28,
      { steps: 4 },
    );
    await page.mouse.up();
    await expect(intervalStart).not.toHaveAttribute(
      "data-preview-params",
      elasticParamsBefore,
    );
    // handleは曲線上のanchorに載るので、parameter確定後は位置も追従する。
    const movedBox = await waveSizeHandle.boundingBox();
    expect(Math.abs(movedBox.y - amplitudeBox.y)).toBeGreaterThan(4);
    const elasticPathAfter = await panel
      .locator("#graph-curve")
      .getAttribute("d");
    expect(elasticPathAfter).not.toEqual(elasticPathBefore);
    await expect(page.locator("#undo-state")).toContainText(
      "Undo 1 · Interval Easing",
    );
    await panel
      .getByRole("button", { name: "Cyclic · Sine", exact: true })
      .click();
    const sinePath = await panel.locator("#graph-curve").getAttribute("d");
    await panel
      .getByRole("button", { name: "Random", exact: true })
      .click();
    // 非overshoot型へ切り替えるとovershootは明示OFFへ戻り、curveは0〜1に留まる。
    await expect(
      panel.getByRole("button", { name: "Overshoot mode" }),
    ).toHaveAttribute("aria-pressed", "false");
    const randomPath = await panel.locator("#graph-curve").getAttribute("d");
    expect(randomPath).not.toEqual(sinePath);
    await panel
      .getByRole("button", { name: "Steps", exact: true })
      .click();
    const keyShapeAfter = await page
      .locator(".inline-key")
      .evaluateAll((keys) =>
        keys.map((key) => ({
          channel: key.dataset.channel,
          left: key.style.left,
          filled: key.classList.contains("filled"),
        })),
      );
    expect(keyShapeAfter).toEqual(keyShapeBefore);
    await expect(intervalStart).toHaveAttribute("data-easing", "Steps");
    await expect(intervalStart).toHaveAttribute(
      "data-curve",
      "interval:Steps",
    );
    await expect(page.locator("#undo-state")).toContainText(
      "Undo 1 · Interval Easing",
    );
    await panel
      .getByRole("button", { name: /Smooth/ })
      .click();
    await expect(intervalStart).not.toHaveAttribute(
      "data-interpolation-kind",
    );
    const overshootToggle = panel.locator(
      'button[aria-label="Overshoot mode"]',
    );
    // Steps(非overshoot型)選択の時点で明示OFFへ戻っている。
    await expect(overshootToggle).toHaveAttribute("aria-pressed", "false");
    await overshootToggle.click();
    await expect(overshootToggle).toHaveAttribute("aria-pressed", "true");
    await overshootToggle.click();
    await expect(overshootToggle).toHaveAttribute("aria-pressed", "false");

    await panel.getByRole("button", { name: "Easing actions" }).click();
    const actions = panel.getByRole("menu", { name: "Easing actions" });
    const pasteCurrent = actions.getByRole("menuitem", {
      name: "Paste Curve",
    });
    const pasteAll = actions.getByRole("menuitem", {
      name: "Paste to all in current channel · 2",
    });
    await expect(pasteCurrent).toBeDisabled();
    await expect(pasteAll).toBeDisabled();

    await actions.getByRole("menuitem", { name: "Copy Curve" }).click();
    await expect(pasteCurrent).toBeEnabled();
    await expect(pasteAll).toBeEnabled();
    await pasteAll.click();
    await expect(page.locator("#undo-state")).toContainText(
      "Undo 1 · Paste Easing to channel",
    );
  });

  test("keeps a fixed graph view for each Overshoot mode", async ({ page }) => {
    await openCandidate(page);
    const panel = page.locator('[data-react-surface="easing-graph"]');
    const graph = panel.locator(".easing-graph");
    const overshootToggle = panel.locator(
      'button[aria-label="Overshoot mode"]',
    );
    await expect(graph).toHaveAttribute("data-view-top", "1.35");
    await expect(graph).toHaveAttribute("data-view-bottom", "-0.35");
    await expect(overshootToggle).toHaveAttribute("aria-pressed", "false");
    const offIcon = await overshootToggle
      .locator(".candidate-overshoot-curve")
      .getAttribute("d");

    await overshootToggle.evaluate((button) => button.click());
    await expect(overshootToggle).toHaveAttribute("aria-pressed", "true");
    await expect(graph).toHaveAttribute("data-view-top", "2.20");
    await expect(graph).toHaveAttribute("data-view-bottom", "-0.50");
    const onIcon = await overshootToggle
      .locator(".candidate-overshoot-curve")
      .getAttribute("d");
    expect(onIcon).not.toEqual(offIcon);

    const before = await graph.evaluate((element) => ({
      top: element.dataset.viewTop,
      bottom: element.dataset.viewBottom,
      guides: [...element.querySelectorAll(".graph-guide")].map((guide) => [
        guide.getAttribute("y1"),
        guide.getAttribute("y2"),
      ]),
    }));

    await panel
      .locator('[data-curve="Ease In"]')
      .evaluate((button) => button.click());

    const after = await graph.evaluate((element) => ({
      top: element.dataset.viewTop,
      bottom: element.dataset.viewBottom,
      guides: [...element.querySelectorAll(".graph-guide")].map((guide) => [
        guide.getAttribute("y1"),
        guide.getAttribute("y2"),
      ]),
    }));
    expect(after).toEqual(before);
  });

  test("keeps plugin results visual and reserves state labels for deviations", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(page.locator(".candidate-plugin-card")).toHaveCount(3);
    await expect(page.locator(".candidate-plugin-card .thumb-state")).toHaveText([
      "Unavailable",
    ]);
    await expect(page.getByText("READY", { exact: true })).toHaveCount(0);
    await expect(page.getByText("AVAILABLE", { exact: true })).toHaveCount(0);
    await expect(page.locator("#vism-browser")).not.toContainText(
      "Layered light pulses",
    );
    await expect(page.getByRole("button", { name: "Apply" })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Filters" })).toHaveCount(0);
    await expect(page.locator("#plugin-filter-panel")).toHaveCount(0);
    await expect(
      page.locator(".candidate-plugin-card.is-selected"),
    ).toHaveCount(1);

    const namedCardLayout = await page
      .locator(".candidate-plugin-card")
      .evaluateAll((cards) =>
        cards.map((card) => {
          const outer = card.getBoundingClientRect();
          const main = card
            .querySelector(".candidate-plugin-card-main")
            .getBoundingClientRect();
          return {
            unusedHeight: outer.height - main.height,
            selectedOutline:
              getComputedStyle(card).boxShadow !== "none",
          };
        }),
      );
    expect(
      namedCardLayout.every(({ unusedHeight }) =>
        Math.abs(unusedHeight) < 1,
      ),
    ).toBe(true);
    expect(
      namedCardLayout.filter(({ selectedOutline }) => selectedOutline),
    ).toHaveLength(1);

    const echoName = page.locator(
      '.candidate-plugin-card[data-plugin-name="Echo Bloom"] .candidate-card-name b',
    );
    await expect(echoName).toHaveText("Echo Bloom");
    expect(
      await echoName.evaluate(
        (element) => element.scrollWidth <= element.clientWidth,
      ),
    ).toBe(true);

    await page.getByRole("button", { name: "Thumbnail-only view" }).click();
    await expect(page.locator("#vism-browser")).toHaveAttribute(
      "data-view",
      "visual",
    );
    await expect(page.locator(".candidate-card-name:visible")).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Echo Bloom" })).toBeVisible();
    const firstCard = page.locator(".candidate-plugin-card").first();
    const defaultBox = await firstCard.boundingBox();
    expect(Math.abs(defaultBox.width - defaultBox.height)).toBeLessThan(1);

    await page.getByRole("button", { name: "Settings" }).click();
    await page
      .getByRole("dialog", { name: "Settings" })
      .getByRole("button", { name: "Browser" })
      .click();
    const thumbnailSize = page.locator("#plugin-thumb-size");
    await expect(thumbnailSize).toBeVisible();
    await thumbnailSize.evaluate((control) => {
      control.value = "64";
      control.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await page.getByRole("button", { name: "Done" }).click();
    const smallBox = await firstCard.boundingBox();
    expect(Math.abs(smallBox.width - smallBox.height)).toBeLessThan(1);

    await thumbnailSize.evaluate((control) => {
      control.value = "160";
      control.dispatchEvent(new Event("input", { bubbles: true }));
    });
    const largeBox = await firstCard.boundingBox();
    expect(Math.abs(largeBox.width - largeBox.height)).toBeLessThan(1);
    expect(largeBox.width).toBeGreaterThan(smallBox.width + 20);

    await thumbnailSize.evaluate((control) => {
      control.value = "80";
      control.dispatchEvent(new Event("input", { bubbles: true }));
    });
    await page
      .getByRole("button", { name: "Thumbnail and name view" })
      .click();
    const namedThumbnailBox = await firstCard.locator(".plugin-thumb").boundingBox();
    expect(namedThumbnailBox.width / namedThumbnailBox.height).toBeGreaterThan(1.7);

    await expect(page.locator('[data-plugin-source="issues"]')).toHaveCount(0);

    await page.getByRole("searchbox", { name: "Search effects" }).fill("fold");
    await expect(
      page.locator(".candidate-plugin-card:visible"),
    ).toHaveCount(1);
    await expect(page.locator("#plugin-result-count")).toHaveText("1");
  });

  test("applies a usable effect by drag and drop or double click", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(
      page.locator("#inspector #effect-parameter-description"),
    ).toContainText(
      "Adjust Intensity and Spread",
    );
    await expect(page.locator("#inspector")).not.toContainText("TRANSFORM");
    await expect(page.locator("#inspector")).toContainText("Pulse rings · Effect");
    const echo = page.locator('.candidate-plugin-card[data-plugin-name="Echo Bloom"]');
    await echo.dblclick();
    await expect(page.locator("#undo-state")).toContainText("Add Echo Bloom");

    await page.getByRole("button", { name: "Create", exact: true }).click();
    const glyph = page.locator('.candidate-element-card[data-element="glyph-current"]');
    await glyph.dragTo(page.locator("#stage .scene-copy"));
    await expect(page.locator("#undo-state")).toContainText("Add Glyph Current");
    await expect(page.locator("#stage .selection-bounds")).toHaveClass(/handoff/);

    await page.getByRole("button", { name: "↶ Undo" }).click();
    await expect(page.locator("#undo-state")).toContainText(
      "Redo 1 · Add Glyph Current",
    );
    await page.getByRole("button", { name: "↷ Redo" }).click();
    await expect(page.locator("#undo-state")).toContainText(
      "Undo 1 · Add Glyph Current",
    );

    await page.getByRole("button", { name: "Effects", exact: true }).click();
    const unavailable = page.locator('.candidate-plugin-card[data-mode="blocked"]');
    await unavailable.click();
    await expect(page.locator(".drag-context-tip")).toContainText(
      "Capability unavailable",
    );
  });

  test("shares the Explorer shell and commit grammar across Plugins and Media", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(
      page.locator("#vism-browser > .candidate-search-row"),
    ).toHaveCount(1);
    await expect(
      page.locator("#vism-browser > .candidate-browser-layout > .candidate-browser-nav"),
    ).toHaveCount(1);

    await page.getByRole("button", { name: "Media" }).click();
    await expect(
      page.locator("#project-browser > .candidate-search-row"),
    ).toHaveCount(1);
    await expect(
      page.locator("#project-browser > .candidate-browser-layout > .candidate-browser-nav"),
    ).toHaveCount(1);

    const mediaCandidates = page.locator(".asset-tile");
    await expect(mediaCandidates).not.toHaveCount(0);
    expect(
      await page.locator('.asset-tile[draggable="true"]').count(),
    ).toBe(await mediaCandidates.count());

    const firstProjectAsset = page.locator(
      '.asset-tile[data-asset-source-view="all"][data-asset="night_drive.wav"]',
    );
    await firstProjectAsset.dblclick();
    await expect(page.locator("#undo-state")).toContainText(
      "Place night_drive.wav",
    );

    await page.locator('[data-file-root-select="audio"]').click();
    const externalAsset = page.locator(
      '.asset-tile[data-asset-source-view="files"][data-file-root="audio"][data-file-path=""][data-asset="impact_04.wav"]',
    );
    await externalAsset.dragTo(page.locator("#stage .scene-copy"));
    await expect(page.locator("#undo-state")).toContainText(
      "Import and place impact_04.wav",
    );

    const sourceRow = await page
      .locator('[data-file-root-select="audio"]')
      .boundingBox();
    expect(sourceRow.height).toBeLessThanOrEqual(24);
  });

  test("resizes, closes by drag, and restores the Browser folder hierarchy without changing results", async ({
    page,
  }) => {
    await openCandidate(page);
    await page.getByRole("button", { name: "Media" }).click();

    const browser = page.locator(".browser-candidate");
    const hierarchy = page
      .locator("#project-browser")
      .getByRole("navigation", { name: "Media sources" });
    const results = page.locator("#project-browser .candidate-results");
    const widthBefore = await results.evaluate(
      (element) => element.getBoundingClientRect().width,
    );
    const separator = page.getByRole("separator", {
      name: "ブラウザのフォルダ階層の横幅を変更",
    });
    const separatorBefore = await separator.boundingBox();
    expect(separatorBefore).not.toBeNull();

    await page.mouse.move(
      separatorBefore.x + separatorBefore.width / 2,
      separatorBefore.y + separatorBefore.height / 2,
    );
    await page.mouse.down();
    await page.mouse.move(separatorBefore.x + 50, separatorBefore.y + 10);
    await page.mouse.up();

    await expect(separator).toHaveAttribute("aria-valuenow", "152");
    const widthExpanded = await results.evaluate(
      (element) => element.getBoundingClientRect().width,
    );
    expect(widthExpanded).toBeLessThan(widthBefore);

    const separatorExpanded = await separator.boundingBox();
    expect(separatorExpanded).not.toBeNull();
    await page.mouse.move(
      separatorExpanded.x + separatorExpanded.width / 2,
      separatorExpanded.y + separatorExpanded.height / 2,
    );
    await page.mouse.down();
    await page.mouse.move(1, separatorExpanded.y + 10);
    await page.mouse.up();

    await expect(browser).toHaveAttribute("data-hierarchy-hidden", "true");
    await expect(hierarchy).toBeHidden();
    await expect(
      page.locator("#project-browser .asset-tile:not([hidden])"),
    ).toHaveCount(6);
    const widthHidden = await results.evaluate(
      (element) => element.getBoundingClientRect().width,
    );
    expect(widthHidden).toBeGreaterThan(widthBefore);

    await page
      .getByRole("button", {
        name: "ブラウザのフォルダ階層を表示",
      })
      .click();
    await expect(browser).toHaveAttribute("data-hierarchy-hidden", "false");
    await expect(hierarchy).toBeVisible();
    await expect(
      page.getByRole("separator", {
        name: "ブラウザのフォルダ階層の横幅を変更",
      }),
    ).toHaveAttribute("aria-valuenow", "106");
  });

  test("classifies Effects and Create items with tags and shares Browser thumbnail size", async ({
    page,
  }) => {
    await openCandidate(page);

    const browser = page.locator(".browser-candidate");
    const effectTags = page.getByRole("group", { name: "Effect tags" });
    const effectKinetic = effectTags.getByRole("button", { name: /Kinetic/ });
    const echoBloom = page.locator(
      '.candidate-plugin-card[data-browser-item="echo-bloom"]',
    );
    await echoBloom.dragTo(effectKinetic);
    await expect(echoBloom).toHaveAttribute(
      "data-tags",
      /(?:^| )kinetic(?: |$)/,
    );
    await expect(effectKinetic.locator("b")).toHaveText("2");

    await effectTags.getByRole("button", { name: /Review/ }).click();
    await expect(
      page.locator(".candidate-plugin-card:visible"),
    ).toHaveCount(1);
    await expect(
      page.locator(
        '.candidate-plugin-card[data-browser-item="fold-field"]',
      ),
    ).toBeVisible();

    await page.getByRole("button", { name: "Settings" }).click();
    await page.getByRole("button", { name: "Browser" }).click();
    const thumbnailSize = page.getByRole("slider", {
      name: "Browser thumbnail size",
    });
    await thumbnailSize.fill("160");
    await expect(browser).toHaveAttribute(
      "data-browser-thumbnail-size",
      "160",
    );
    await page.getByRole("button", { name: "Done" }).click();

    await page.getByRole("button", { name: "Create" }).click();
    await page
      .getByRole("button", { name: "Create thumbnail-only view" })
      .click();
    await expect(page.locator("#elements-browser")).toHaveAttribute(
      "data-view",
      "visual",
    );
    await expect(page.locator("#status-body")).toHaveText(
      "Thumbnail only",
    );
    await expect(
      page.locator("#elements-browser .candidate-element-name:visible"),
    ).toHaveCount(0);
    const createVisualCard = page
      .locator("#elements-browser .candidate-element-card")
      .first();
    const createVisualBox = await createVisualCard.boundingBox();
    expect(
      Math.abs(createVisualBox.width - createVisualBox.height),
    ).toBeLessThan(1);
    await page
      .getByRole("button", {
        name: "Create thumbnail and name view",
      })
      .click();

    const createTags = page.getByRole("group", { name: "Create tags" });
    const createBrand = createTags.getByRole("button", { name: /Brand kit/ });
    const particleField = page.locator(
      '.candidate-element-card[data-browser-item="particle-field"]',
    );
    await particleField.dragTo(createBrand);
    await expect(particleField).toHaveAttribute(
      "data-tags",
      /(?:^| )brand-kit(?: |$)/,
    );
    await expect(createBrand.locator("b")).toHaveText("4");

    await createBrand.click();
    await expect(
      page.locator(".candidate-element-card:visible"),
    ).toHaveCount(4);

    await page.getByRole("button", { name: "Media" }).click();
    await expect(browser).toHaveAttribute(
      "data-browser-thumbnail-size",
      "160",
    );
  });

  test("keeps Browser labels on one line and leaves detail to the lower tips", async ({
    page,
  }) => {
    await openCandidate(page);
    await page.getByRole("button", { name: "Create" }).click();

    const separator = page.getByRole("separator", {
      name: "ブラウザのフォルダ階層の横幅を変更",
    });
    await separator.focus();
    await separator.press("ArrowLeft");
    await separator.press("ArrowLeft");

    const resultHeader = page.locator(
      "#element-results .candidate-results-head",
    );
    const headerMetrics = await resultHeader.evaluate((element) => ({
      clientHeight: element.clientHeight,
      scrollHeight: element.scrollHeight,
      whiteSpace: getComputedStyle(element).whiteSpace,
    }));
    expect(headerMetrics.whiteSpace).toBe("nowrap");
    expect(headerMetrics.scrollHeight).toBeLessThanOrEqual(
      headerMetrics.clientHeight,
    );

    const labelsStaySingleLine = await page
      .locator(
        "#elements-browser .candidate-nav-title, #elements-browser .candidate-nav-group button span",
      )
      .evaluateAll((elements) =>
        elements.every(
          (element) => getComputedStyle(element).whiteSpace === "nowrap",
        ),
      );
    expect(labelsStaySingleLine).toBe(true);

    await page.locator("#elements-browser").hover();
    await expect(page.locator("#status-title")).toHaveText("Create Browser");
    await expect(page.locator("#status-body")).toContainText(
      "Browse every registered item",
    );
  });

  test("shows typed drag rejection and keeps Stage and Timeline as explicit targets", async ({
    page,
  }) => {
    await openCandidate(page);
    await page.getByRole("button", { name: "Media" }).click();
    await page.locator('[data-file-root-select="city"]').click();

    const folder = page.locator(
      '.asset-tile[data-asset-source-view="files"][data-file-root="city"][data-file-directory="MV"]',
    );
    await folder.dragTo(page.locator("#stage .scene-copy"));
    await expect(page.locator("#status-body")).toContainText(
      "Folder cannot be placed",
    );
    await expect(page.locator("#undo-state")).toHaveText("");

    const file = page.locator(
      '.asset-tile[data-asset-source-view="files"][data-file-root="city"][data-file-path=""][data-asset="logo.svg"]',
    );
    await file.dragTo(page.locator(".time-plane"));
    await expect(page.locator("#undo-state")).toContainText(
      "Import and place logo.svg",
    );
    await expect(
      page.locator(".time-plane > .time-bar.handoff"),
    ).toContainText("logo.svg");
  });

  test("bulk-tags files and folders and switches between folder and recursive file views", async ({
    page,
  }) => {
    await openCandidate(page);
    await page.getByRole("button", { name: "Media" }).click();
    await page.locator('[data-file-root-select="city"]').click();

    const folder = page.locator(
      '.asset-tile[data-asset-source-view="files"][data-file-root="city"][data-file-directory="MV"]',
    );
    const rootFile = page.locator(
      '.asset-tile[data-asset-source-view="files"][data-file-root="city"][data-file-path=""][data-asset="logo.svg"]',
    );
    await page
      .locator("#project-browser")
      .getByRole("button", { name: "Select", exact: true })
      .click();
    await folder.click();
    await rootFile.click();
    await expect(page.locator("#asset-selection-count")).toHaveText("2 selected");

    await page.locator("#add-media-tag").click();
    await page.getByRole("textbox", { name: "New tag name" }).fill("Client");
    await page.locator("#create-media-tag").click();
    const clientTagBox = page.locator('[data-media-tag-box="client"]');
    await expect(clientTagBox).toContainText("Client");
    await folder.dragTo(clientTagBox);
    await expect(folder).toHaveAttribute("data-tags", "client");
    await expect(rootFile).toHaveAttribute("data-tags", "client");
    await expect(folder.locator(".asset-tags")).toHaveText("client");
    await expect(rootFile.locator(".asset-tags")).toHaveText("client");
    await expect(clientTagBox.locator("b")).toHaveText("2");
    await expect(page.locator("#undo-state")).not.toContainText("tag");

    await page.getByRole("button", { name: "All files view" }).click();
    await expect(page.locator("#asset-scope-label")).toHaveText("ALL FILES");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(7);
    await expect(
      page.locator(".candidate-asset-grid .asset-tile[data-file-directory]:visible"),
    ).toHaveCount(0);

    await page.getByRole("button", { name: "Browse folders" }).click();
    await expect(page.locator("#asset-scope-label")).toHaveText("REGISTERED ROOT");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(2);
  });

  test("uses list taxonomy as contextual result navigation", async ({
    page,
  }) => {
    await openCandidate(page);
    await page.getByRole("button", { name: "List view" }).click();

    const echoType = page.getByRole("navigation", { name: "Echo Bloom type" });
    await expect(echoType).toBeVisible();
    await expect(echoType).toContainText("Effect");
    await expect(echoType).toContainText("Light");

    await echoType.getByRole("button", { name: "Light" }).click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(1);
    await expect(page.locator("#plugin-result-count")).toHaveText("1");
    await expect(page.locator("#plugin-taxonomy-clear")).toHaveText("Light ×");

    await page.locator("#plugin-taxonomy-clear").click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(3);
    await expect(page.locator("#plugin-result-count")).toHaveText("3");

    const typePulse = page.getByRole("navigation", {
      name: "Type Pulse type",
    });
    await typePulse.getByRole("button", { name: "Typography" }).click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(1);
    await page.locator('[data-plugin-source="all"]').click();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(3);
    await expect(page.locator("#plugin-taxonomy-clear")).toBeHidden();
  });

  test("uses one Media browser for Project and registered folders", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(page.locator(".browser-tabs .browser-tab")).toHaveText([
      "Media",
      "Effects",
      "Create",
    ]);
    await page.getByRole("button", { name: "Media" }).click();
    await expect(page.locator(".candidate-project-browser")).toBeVisible();
    await expect(page.getByRole("searchbox", { name: "Search media" })).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Media sources" })).toBeVisible();
    await expect(page.locator("#asset-source-title")).toHaveText("All Media");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(6);
    await expect(page.locator('[data-asset-source="all"]')).toHaveClass(/on/);
    await expect(page.locator("#project-browser")).toHaveAttribute("data-view", "visual");
    await expect(page.locator(".candidate-asset-grid .asset-name:visible")).toHaveCount(0);
    await expect(
      page.locator('.asset-tile[data-asset-source-view="all"][data-asset="logo.svg"]'),
    ).toHaveCount(1);
    const mediaTile = page.locator(".candidate-asset-grid .asset-tile:visible").first();
    const visualBox = await mediaTile.boundingBox();
    expect(visualBox.width / visualBox.height).toBeCloseTo(16 / 9, 1);

    const mediaPreview = mediaTile.locator(".asset-preview");
    await expect(mediaPreview).toHaveCSS("aspect-ratio", "16 / 9");
    await mediaPreview.evaluate((preview) => {
      const image = document.createElement("img");
      image.alt = "";
      preview.append(image);
    });
    await expect(mediaPreview.locator("img")).toHaveCSS("object-fit", "contain");

    await page.getByRole("button", { name: "Media thumbnail and name view" }).click();
    await expect(page.locator("#project-browser")).toHaveAttribute("data-view", "grid");
    await expect(mediaTile.locator(".asset-name")).toBeVisible();
    const namedPreviewBox = await mediaPreview.boundingBox();
    expect(namedPreviewBox.width / namedPreviewBox.height).toBeCloseTo(16 / 9, 1);

    await page.locator('[data-asset-source="project"]').click();
    await expect(page.locator("#asset-source-title")).toHaveText("Project");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(4);

    await expect(page.locator("[data-file-root-select]")).toHaveCount(3);
    await page.locator('[data-file-root-select="city"]').click();
    await expect(page.locator("#asset-path")).toContainText("City Source");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(2);
    await expect(page.locator("#project-browser #place-asset")).toHaveCount(0);
    await page.getByRole("button", { name: "Media thumbnail-only view" }).click();
    await expect(
      page.locator(
        '.asset-tile[data-file-directory="MV"] .asset-name',
      ),
    ).toBeVisible();
    await expect(
      page.locator(
        '.asset-tile[data-file-root="city"][data-asset="logo.svg"] .asset-name',
      ),
    ).toBeHidden();
    const folderBox = await page
      .locator('.asset-tile[data-file-directory="MV"]')
      .boundingBox();
    expect(folderBox.width / folderBox.height).toBeCloseTo(16 / 9, 1);
    const hierarchyRows = page.locator("#file-tree button");
    await expect(hierarchyRows).toHaveCount(2);
    await expect(hierarchyRows.nth(0)).toContainText("L0");
    await expect(hierarchyRows.nth(1)).toContainText("L1");
    const rootRow = await hierarchyRows.nth(0).boundingBox();
    const childRow = await hierarchyRows.nth(1).boundingBox();
    expect(Math.abs(rootRow.x - childRow.x)).toBeLessThan(1);

    await hierarchyRows.nth(1).click();
    await expect(page.locator("#asset-path")).toContainText("MV");
    await expect(page.locator("#file-tree button.current")).toContainText("MV");
    await expect(page.locator("#file-tree")).toContainText("night_drive");

    await page.locator('[data-file-root-select="audio"]').click();
    await expect(page.locator("#asset-path")).toContainText("Audio Library");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(3);

    await page
      .locator('.asset-tile[data-file-root="audio"][data-file-directory="Hits"]')
      .dblclick();
    await expect(page.locator("#asset-path")).toContainText("Hits");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(2);
    await expect(page.locator("#asset-scope-label")).toHaveText("FOLDER");

    await page.locator("#file-parent").click();
    await expect(page.locator("#asset-path")).not.toContainText("Hits");
    await page.locator("#add-file-root").click();
    await expect(page.locator("#status-body")).toContainText(
      "Choose another base folder",
    );

    await page.getByRole("button", { name: "Media list view" }).click();
    await expect(page.locator("#project-browser")).toHaveAttribute("data-view", "list");
    const listPreviewBox = await page
      .locator(".candidate-asset-grid .asset-tile:visible .asset-preview")
      .first()
      .boundingBox();
    expect(listPreviewBox.width / listPreviewBox.height).toBeCloseTo(16 / 9, 1);
    await page.getByRole("searchbox", { name: "Search media" }).fill("impact");
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(1);
  });

  test("keeps one Pack manageable while projecting its items into Media, Create, and Effects", async ({
    page,
  }) => {
    await openCandidate(page);

    await expect(page.locator(".browser-tabs .browser-tab")).toHaveText([
      "Media",
      "Effects",
      "Create",
    ]);
    await page.getByRole("button", { name: "Create", exact: true }).click();
    await expect(page.getByRole("navigation", { name: "Create sources" })).toBeVisible();
    await expect(page.locator(".candidate-element-card:visible")).toHaveCount(8);
    await expect(page.getByText("Built-in", { exact: true })).not.toHaveCount(0);
    await expect(page.getByText("Orbit Forge", { exact: true })).not.toHaveCount(0);
    await expect(page.getByText("Motion Kit", { exact: true })).not.toHaveCount(0);

    await page.getByRole("button", { name: "Generators" }).click();
    await expect(page.locator(".candidate-element-card:visible")).toHaveCount(3);
    await expect(
      page.getByRole("button", {
        name: "Particle Field · Generator · Orbit Forge",
      }),
    ).toBeVisible();

    await page.locator('[data-element-filter="all"]').click();
    const rectangle = page.getByRole("button", {
      name: "Rectangle · Shape · Built-in",
    });
    await rectangle.dblclick();
    await expect(page.locator("#undo-state")).toContainText("Add Rectangle");

    const particles = page.getByRole("button", {
      name: "Particle Field · Generator · Orbit Forge",
    });
    await particles.click({ button: "right" });
    const contextMenu = page.getByRole("menu", { name: "Create item commands" });
    await expect(contextMenu).toBeVisible();
    await contextMenu.getByRole("button", { name: "Show provider" }).click();
    await expect(page.locator(".candidate-element-card:visible")).toHaveCount(1);
    await expect(page.locator("#element-result-title")).toHaveText("Orbit Forge");

    await page.locator('[data-pack-select="motion-kit-alpha"]').first().click();
    const packScope = page.locator("#candidate-pack-scope");
    await expect(packScope).toBeVisible();
    await expect(packScope).toContainText("ONE PACK · THREE USES");
    await expect(page.locator(".candidate-element-card:visible")).toHaveCount(3);
    await expect(
      page.locator('[data-item-identity="motion-kit.type-pulse"]:visible'),
    ).toHaveCount(1);
    await packScope.getByRole("button", { name: "Effects 2" }).click();
    await expect(page.getByRole("searchbox", { name: "Search effects" })).toBeVisible();
    await expect(page.locator(".candidate-plugin-card:visible")).toHaveCount(2);
    await packScope.getByRole("button", { name: "Media 2" }).click();
    await expect(page.locator(".candidate-asset-grid .asset-tile:visible")).toHaveCount(2);
    await page.getByRole("button", { name: "Clear pack scope" }).click();

    await page.locator("#stage .frame").click({ button: "right", position: { x: 8, y: 8 } });
    const addMenu = page.getByRole("menu", { name: "Add element" });
    await expect(addMenu).toBeVisible();
    await addMenu.getByRole("button", { name: "T Text" }).click();
    await expect(page.locator("#undo-state")).toContainText("Add Text");
  });

  test("generates a poster and optional motion preview without changing the Document", async ({
    page,
  }) => {
    await openCandidate(page);

    const typePulse = page.locator(
      '.candidate-plugin-card[data-item-identity="motion-kit.type-pulse"]',
    );
    await expect(typePulse).toHaveAttribute("data-preview", "motion");
    await expect(typePulse.locator(".candidate-motion-mark")).toHaveText("▶");
    await expect(typePulse.locator(".candidate-impact")).toHaveText("◆ 12 KEYS");

    await page.getByRole("button", { name: "＋ Save current…" }).click();
    const saveSheet = page.getByRole("dialog", { name: "Save to Browser" });
    await expect(saveSheet).toBeVisible();
    await expect(saveSheet.locator("#candidate-preview-copy")).toHaveText(
      "Auto · 2s around playhead",
    );
    await expect(
      saveSheet.getByRole("slider", { name: "Preview in" }),
    ).toHaveValue("25");
    await expect(
      saveSheet.getByRole("slider", { name: "Preview out" }),
    ).toHaveValue("75");
    await expect(saveSheet.locator("#candidate-range-duration")).toHaveText(
      "2.0s",
    );

    await saveSheet.getByRole("button", { name: "GIF / Video" }).click();
    await expect(saveSheet.locator("#candidate-preview-copy")).toContainText(
      "imported preview",
    );
    await saveSheet
      .getByRole("slider", { name: "Preview in" })
      .evaluate((input) => {
        input.value = "35";
        input.dispatchEvent(new Event("input", { bubbles: true }));
      });
    await expect(saveSheet.locator("#candidate-range-duration")).toHaveText(
      "1.6s",
    );
    await saveSheet
      .getByRole("button", { name: "Generate preview" })
      .click();
    await expect(saveSheet.locator("#candidate-preview-copy")).toHaveText(
      "Ready · 1.6s rendered",
    );
    await expect(page.locator("#status-body")).toContainText(
      "Document unchanged",
    );
    await saveSheet.locator("#candidate-save-name").fill("My kinetic title");
    await saveSheet.getByRole("button", { name: "Save", exact: true }).click();
    await expect(saveSheet).toBeHidden();
    await expect(page.locator("#status-body")).toContainText(
      "My kinetic title",
    );
    await expect(page.locator("#status-body")).toContainText(
      "Document unchanged",
    );
    await expect(page.locator("#undo-state")).toHaveText("");
  });
});
