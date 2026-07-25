import { expect, test } from "@playwright/test";

const LEGACY_URL =
  "http://127.0.0.1:5174/docs/mocks/m3-vism-host-boundary.html#inbox-empty";
const REACT_ARCHIVE_URL =
  "http://127.0.0.1:5173/#archive/inbox-empty";
const REACT_CANDIDATE_URL =
  "http://127.0.0.1:5173/#plugin-browser-candidate";

const EXPECTED_COUNTS = {
  installed: {
    sections: 7,
    scrubs: 2,
    objectAutomation: 5,
    colorChips: 2,
  },
  discover: {
    sections: 1,
    scrubs: 0,
    objectAutomation: 0,
    colorChips: 0,
  },
  blocked: {
    sections: 1,
    scrubs: 0,
    objectAutomation: 0,
    colorChips: 0,
  },
  missing: {
    sections: 2,
    scrubs: 0,
    objectAutomation: 0,
    colorChips: 0,
  },
};

const EFFECT_FOCUSED_COUNTS = {
  sections: 2,
  scrubs: 2,
  objectAutomation: 0,
  colorChips: 0,
};

async function settle(page, url, { parserBridge = false, waitInboxEmpty = true } = {}) {
  await page.goto(url, { waitUntil: "domcontentloaded" });
  await page.locator(".app").waitFor({ state: "visible" });
  if (parserBridge) {
    await page
      .locator('.app[data-parity-ready="true"]')
      .waitFor({ state: "visible" });
  }
  if (waitInboxEmpty) {
    await page.locator("#inbox.empty").waitFor({ state: "visible" });
  }
  await page.evaluate(async () => {
    await document.fonts.ready;
    await new Promise((resolve) =>
      requestAnimationFrame(() => requestAnimationFrame(resolve)),
    );
  });
}

async function selectMode(page, mode) {
  await page.locator(`.vism[data-mode="${mode}"]`).first().click();
  await expect(page.locator("#stage")).toHaveAttribute("data-mode", mode);
}

async function readCounts(page) {
  return page.locator("#inspector").evaluate((root) => ({
    sections: root.querySelectorAll(":scope > .section").length,
    scrubs: root.querySelectorAll(".scrub[data-param]").length,
    objectAutomation: root.querySelectorAll("[data-object-automation]").length,
    colorChips: root.querySelectorAll(".color-chip").length,
  }));
}

async function readInspectorStructure(page) {
  return page.locator("#inspector").evaluate((root) => {
    function normalizeClass(value) {
      return value.trim().split(/\s+/).filter(Boolean).sort().join(" ");
    }
    function normalizeStyleDeclaration(value) {
      return value
        .split(";")
        .map((part) => part.trim())
        .filter(Boolean)
        .map((part) => {
          const colon = part.indexOf(":");
          if (colon === -1) return part;
          return `${part.slice(0, colon).trim()}:${part.slice(colon + 1).trim()}`;
        })
        .sort()
        .join(";");
    }
    function serialize(node) {
      if (node.nodeType === Node.TEXT_NODE) {
        const text = node.textContent.replace(/\s+/g, " ").trim();
        return text ? { text } : null;
      }
      if (node.nodeType !== Node.ELEMENT_NODE) return null;
      const attrs = {};
      for (const attr of node.attributes) {
        if (attr.name === "class") {
          attrs.class = normalizeClass(attr.value);
        } else if (attr.name === "style") {
          attrs.style = normalizeStyleDeclaration(attr.value);
        } else {
          attrs[attr.name] = attr.value;
        }
      }
      const children = [...node.childNodes].map(serialize).filter(Boolean);
      return {
        tag: node.tagName.toLowerCase(),
        ...(Object.keys(attrs).length ? { attrs } : {}),
        ...(children.length ? { children } : {}),
      };
    }
    return serialize(root);
  });
}

test.beforeEach(async ({ page }) => {
  await settle(page, REACT_ARCHIVE_URL);
});

test("five render states publish inspector sink and render non-empty subtree", async ({
  page,
}) => {
  const sinkReady = await page.evaluate((symbolKey) => {
    const sink = window[Symbol.for(symbolKey)];
    return Boolean(sink?.publish);
  }, "motolii.legacyHostBoundary.inspector");
  expect(sinkReady).toBe(true);

  for (const mode of ["installed", "discover", "blocked", "missing"]) {
    await selectMode(page, mode);
    await expect(page.locator("#inspector")).not.toBeEmpty();
    await expect(page.locator("#inspector .panel-head")).toHaveText("Inspector");
  }

  await page.goto(REACT_CANDIDATE_URL, { waitUntil: "domcontentloaded" });
  await page.locator(".app").waitFor({ state: "visible" });
  await page
    .locator('.app[data-parity-ready="true"]')
    .waitFor({ state: "visible" });
  await expect(page.locator("#vism-browser.candidate-plugin-browser")).toHaveCount(1);
  await expect(page.locator("#inspector")).not.toBeEmpty();
  const effectCounts = await readCounts(page);
  expect(effectCounts.sections).toBe(EFFECT_FOCUSED_COUNTS.sections);
  expect(effectCounts.scrubs).toBe(EFFECT_FOCUSED_COUNTS.scrubs);
  expect(effectCounts.objectAutomation).toBe(EFFECT_FOCUSED_COUNTS.objectAutomation);
  expect(effectCounts.colorChips).toBe(EFFECT_FOCUSED_COUNTS.colorChips);
  await expect(page.locator("#inspector .lifecycle")).toHaveCount(0);
});

test("object automation toggles once per click", async ({ page }) => {
  await selectMode(page, "installed");
  const mark = page.locator(
    '.automation-mark[data-object-automation="scale"]',
  );
  await mark.click();
  await expect(mark).toHaveAttribute("aria-pressed", "true");
  await expect(mark).toHaveClass(/on/);
  await expect(page.locator("#scale-object-state i").first()).toHaveText(
    "AUTO ON",
  );
  await expect(page.locator("#undo-state")).toHaveText("Undo 1 · scale automation");
});

test("effect automation updates timeline channel visibility", async ({
  page,
}) => {
  await selectMode(page, "installed");
  const intensityMark = page.locator(
    '.automation-mark[data-automation="intensity"]',
  );
  const intensityKeys = page.locator('#vism-clip .inline-key[data-channel="Intensity"]');
  await expect(intensityMark).toHaveAttribute("aria-pressed", "true");
  await expect(intensityKeys.first()).not.toHaveClass(/channel-hidden/);
  await intensityMark.click();
  await expect(intensityMark).toHaveAttribute("aria-pressed", "false");
  await expect(intensityKeys.first()).toHaveClass(/channel-hidden/);
  await expect(page.locator("#undo-state")).toHaveText("Undo 1 · intensity automation");
});

test("scrub pointer and keyboard paths update undo and esc cancels drag", async ({
  page,
}) => {
  await selectMode(page, "installed");
  const scrub = page.locator("#intensity");
  const box = await scrub.boundingBox();
  expect(box).not.toBeNull();

  await scrub.evaluate((el, startX) => {
    el.dispatchEvent(
      new PointerEvent("pointerdown", {
        bubbles: true,
        clientX: startX,
        pointerId: 1,
        button: 0,
        pointerType: "mouse",
      }),
    );
    el.dispatchEvent(
      new PointerEvent("pointermove", {
        bubbles: true,
        clientX: startX + 60,
        pointerId: 1,
        pointerType: "mouse",
      }),
    );
    el.dispatchEvent(
      new PointerEvent("pointerup", {
        bubbles: true,
        pointerId: 1,
        pointerType: "mouse",
      }),
    );
  }, box.x + 20);
  await expect(page.locator("#intensity-read")).not.toHaveText("64%");
  await expect(page.locator("#undo-state")).toHaveText("Undo 1 · Intensity");

  await scrub.dispatchEvent("pointerdown", {
    bubbles: true,
    clientX: box.x + 20,
    pointerId: 2,
    button: 0,
  });
  await scrub.dispatchEvent("pointerup", {
    bubbles: true,
    pointerId: 2,
  });
  await expect(page.locator("#undo-state")).toHaveText("Undo 1 · Intensity");

  await scrub.dispatchEvent("pointerdown", {
    bubbles: true,
    clientX: box.x + 20,
    pointerId: 3,
    button: 0,
  });
  await scrub.dispatchEvent("pointermove", {
    bubbles: true,
    clientX: box.x + 100,
    pointerId: 3,
  });
  await page.keyboard.press("Escape");
  await expect(scrub).not.toHaveClass(/dragging/);
});

test("color apply refreshes chip without inspector structure drift", async ({
  browser,
}) => {
  test.setTimeout(60_000);
  const context = await browser.newContext();
  const legacyPage = await context.newPage();
  const reactPage = await context.newPage();
  try {
    await settle(legacyPage, LEGACY_URL);
    await settle(reactPage, REACT_ARCHIVE_URL, { parserBridge: true });
    await selectMode(legacyPage, "installed");
    await selectMode(reactPage, "installed");

    const runColorSequence = async (page) => {
      await page.locator('.color-chip[data-color-channel="Fill"]').click();
      const labelBefore = await page
        .locator('.color-chip[data-color-channel="Fill"]')
        .getAttribute("data-label");
      await page.locator(".book-swatch:not([hidden])").first().click();
      await page.locator("#apply-color").click();
      const chip = page.locator('.color-chip[data-color-channel="Fill"]');
      const labelAfter = await chip.getAttribute("data-label");
      expect(labelAfter).toMatch(/\S+/);
      expect(labelAfter).not.toBe(labelBefore);
      const chipColor = await chip.evaluate((el) =>
        el.style.getPropertyValue("--chip"),
      );
      expect(chipColor).not.toBe("");
      const structureAfter = await readInspectorStructure(page);
      return { labelAfter, chipColor, structureAfter };
    };

    const legacyResult = await runColorSequence(legacyPage);
    const reactResult = await runColorSequence(reactPage);
    expect(reactResult.labelAfter).toBe(legacyResult.labelAfter);
    expect(reactResult.chipColor).toBe(legacyResult.chipColor);
    expect(reactResult.structureAfter).toEqual(legacyResult.structureAfter);
  } finally {
    await context.close();
  }
});

test("mode round trip preserves inspector counts", async ({ page }) => {
  const sequence = ["installed", "discover", "blocked", "missing", "installed"];
  for (const mode of sequence) {
    await selectMode(page, mode);
    const counts = await readCounts(page);
    const expected = EXPECTED_COUNTS[mode];
    expect(counts.sections).toBe(expected.sections);
    expect(counts.scrubs).toBe(expected.scrubs);
    expect(counts.objectAutomation).toBe(expected.objectAutomation);
    expect(counts.colorChips).toBe(expected.colorChips);
  }
});

test("add-vism updates plugin history and history item changes stage mode", async ({
  page,
}) => {
  await selectMode(page, "discover");
  await page.locator("#add-vism").click();
  const historyItems = page.locator("#plugin-history .plugin-history-item");
  await expect(historyItems).toHaveCount(3);
  await historyItems.filter({ hasText: "Echo Bloom" }).click();
  await expect(page.locator("#stage")).toHaveAttribute("data-mode", "installed");
});

test("review recovery opens recovery surface", async ({ page }) => {
  await selectMode(page, "missing");
  await expect(page.locator("#recovery.open")).toHaveCount(0);
  await page.locator("#review-recovery").click();
  await expect(page.locator("#recovery.open")).toHaveCount(1);
});

test("inspector sink symbol is published on window", async ({ page }) => {
  const published = await page.evaluate(() => {
    const sink = window[Symbol.for("motolii.legacyHostBoundary.inspector")];
    return typeof sink?.publish === "function" && typeof sink?.refresh === "function";
  });
  expect(published).toBe(true);
});
