import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";

const VIEWPORT = { width: 1440, height: 900 };
const LEGACY_SOURCE_SHA256 =
  "a20bd36d6b8b241ef4ec21adda6b045e470bf89fbc78e2788207aa6d0875ec68";
const LEGACY_URL =
  "http://127.0.0.1:5174/docs/mocks/m3-vism-host-boundary.html#inbox-empty";
const REACT_ARCHIVE_URL = "http://127.0.0.1:5173/#archive/inbox-empty";
const REACT_CANDIDATE_URL = "http://127.0.0.1:5173/#plugin-browser-candidate";
const LEGACY_SOURCE_FETCH_URL =
  "http://127.0.0.1:5174/docs/mocks/m3-vism-host-boundary.html";
const CROSS_PAGE_MODES = ["installed", "discover", "blocked", "missing"];

const STYLE_PROPERTIES = [
  "display",
  "position",
  "flex-direction",
  "align-items",
  "justify-content",
  "gap",
  "padding-top",
  "padding-right",
  "padding-bottom",
  "padding-left",
  "margin-top",
  "margin-right",
  "margin-bottom",
  "margin-left",
  "border-top-width",
  "border-right-width",
  "border-bottom-width",
  "border-left-width",
  "border-top-color",
  "border-right-color",
  "border-bottom-color",
  "border-left-color",
  "border-top-style",
  "border-right-style",
  "border-bottom-style",
  "border-left-style",
  "background-color",
  "color",
  "opacity",
  "font-size",
  "font-weight",
  "font-family",
  "line-height",
  "letter-spacing",
  "overflow",
  "overflow-x",
  "overflow-y",
  "width",
  "height",
  "min-width",
  "min-height",
  "max-width",
  "max-height",
  "box-sizing",
];

const STYLE_SELECTOR_LIST = [
  "#inspector",
  "#inspector > .panel-head",
  "#inspector > .section",
  "#inspector .section-title",
  "#inspector .identity",
  "#inspector .identity .icon",
  "#inspector .row",
  "#inspector label",
  "#inspector .value",
  "#inspector .tag",
  "#inspector .axis-pack",
  "#inspector .automation-mark",
  "#inspector .scrub",
  "#inspector .scrub-dial",
  "#inspector output",
  "#inspector .scrub-hint",
  "#inspector .color-chip",
  "#inspector .segmented",
  "#inspector .link-value",
  "#inspector .driver-mini",
  "#inspector .applied-plugin",
  "#inspector .notice",
  "#inspector .lifecycle",
  "#inspector .action-strip",
  "#inspector .action-strip .btn",
  "#inspector > details.dev-info",
  "#inspector .effect-parameter-description",
];

const ARIA_SELECTOR_LIST = [
  ...STYLE_SELECTOR_LIST,
  "#inspector button",
  "#inspector summary",
  "#inspector [aria-hidden]",
  "#inspector [tabindex]",
  "#inspector button:disabled",
  "#inspector [disabled]",
  "#inspector [hidden]",
];

const INSTALLED_PRESENCE_EXCLUSIONS = new Set([
  "#inspector .effect-parameter-description",
  "#inspector .notice",
  "#inspector .action-strip",
  "#inspector .action-strip .btn",
]);

const EXPECTED_COUNTS = {
  installed: {
    panelHead: 1,
    sections: 7,
    appliedPluginSections: 1,
    scrubs: 2,
    automationMarks: 2,
    objectAutomation: 5,
    colorChips: 2,
    linkTarget: 1,
    effectDescription: 0,
    errorNotice: 0,
    devInfo: 1,
  },
  discover: {
    panelHead: 1,
    sections: 1,
    appliedPluginSections: 0,
    scrubs: 0,
    automationMarks: 0,
    objectAutomation: 0,
    colorChips: 0,
    linkTarget: 0,
    effectDescription: 0,
    errorNotice: 0,
    devInfo: 1,
  },
  blocked: {
    panelHead: 1,
    sections: 1,
    appliedPluginSections: 0,
    scrubs: 0,
    automationMarks: 0,
    objectAutomation: 0,
    colorChips: 0,
    linkTarget: 0,
    effectDescription: 0,
    errorNotice: 1,
    devInfo: 1,
  },
  missing: {
    panelHead: 1,
    sections: 2,
    appliedPluginSections: 0,
    scrubs: 0,
    automationMarks: 0,
    objectAutomation: 0,
    colorChips: 0,
    linkTarget: 0,
    effectDescription: 0,
    errorNotice: 1,
    devInfo: 1,
  },
};

const EFFECT_FOCUSED_COUNTS = {
  panelHead: 1,
  sections: 2,
  appliedPluginSections: 0,
  scrubs: 2,
  automationMarks: 2,
  objectAutomation: 0,
  colorChips: 0,
  linkTarget: 0,
  effectDescription: 1,
  errorNotice: 0,
  devInfo: 1,
};

async function settle(
  page,
  url,
  { parserBridge = false, waitInboxEmpty = true } = {},
) {
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
  await page
    .locator(`.vism[data-mode="${mode}"]`)
    .first()
    .click();
  await page.locator("#stage").waitFor({ state: "visible" });
  await expect(page.locator("#stage")).toHaveAttribute("data-mode", mode);
}

async function readInspectorOracle(page) {
  return page.locator("#inspector").evaluate(
    (root, { ariaSelectors, styleSelectors, styleProperties }) => {
      function normalizeStyleDeclaration(styleAttr) {
        if (!styleAttr) return "";
        return styleAttr
          .split(";")
          .map((part) => part.trim())
          .filter(Boolean)
          .map((decl) => {
            const splitAt = decl.indexOf(":");
            if (splitAt === -1) return decl.trim();
            const property = decl.slice(0, splitAt).trim().toLowerCase();
            const value = decl.slice(splitAt + 1).trim();
            return `${property}:${value}`;
          })
          .sort()
          .join(";");
      }

      function inspectorStructure(node) {
        function serialize(child) {
          if (child.nodeType === Node.TEXT_NODE) {
            const text = child.textContent.replace(/\s+/g, " ").trim();
            return text ? { text } : null;
          }
          if (child.nodeType !== Node.ELEMENT_NODE) return null;
          const attrs = {};
          for (const attr of [...child.attributes].sort((a, b) =>
            a.name.localeCompare(b.name),
          )) {
            if (attr.name === "class") {
              attrs.class = attr.value
                .split(/\s+/)
                .filter(Boolean)
                .sort()
                .join(" ");
            } else if (attr.name === "style") {
              attrs.style = normalizeStyleDeclaration(attr.value);
            } else {
              attrs[attr.name] = attr.value;
            }
          }
          const children = [...child.childNodes].map(serialize).filter(Boolean);
          return {
            tag: child.tagName.toLowerCase(),
            ...(Object.keys(attrs).length ? { attrs } : {}),
            ...(children.length ? { children } : {}),
          };
        }
        return serialize(node);
      }

      function inspectorAria(selectors) {
        const projection = {};
        for (const selector of selectors) {
          root.querySelectorAll(selector).forEach((element, index) => {
            const entry = {};
            if (element.hasAttribute("role")) {
              entry.role = element.getAttribute("role");
            }
            for (const attr of element.attributes) {
              if (attr.name.startsWith("aria-")) {
                entry[attr.name] = attr.value;
              }
            }
            if (element.hasAttribute("tabindex")) {
              entry.tabindex = element.getAttribute("tabindex");
            }
            if (
              element instanceof HTMLButtonElement ||
              element instanceof HTMLInputElement ||
              element instanceof HTMLSelectElement ||
              element instanceof HTMLTextAreaElement ||
              element instanceof HTMLOptionElement ||
              element instanceof HTMLFieldSetElement
            ) {
              if (element.disabled) {
                entry.disabled = true;
              }
            } else if (element.hasAttribute("disabled")) {
              entry.disabled = true;
            }
            if (element.hidden) {
              entry.hidden = true;
            }
            projection[`${selector}@${index}`] = entry;
          });
        }
        return projection;
      }

      function inspectorStyles(selectors, properties) {
        const styles = {};
        for (const selector of selectors) {
          root.querySelectorAll(selector).forEach((element, index) => {
            const computed = getComputedStyle(element);
            const entry = {};
            for (const property of properties) {
              entry[property] = computed.getPropertyValue(property);
            }
            styles[`${selector}@${index}`] = entry;
          });
        }
        return styles;
      }

      function inspectorPresence(selectors) {
        const presence = {};
        for (const selector of selectors) {
          let matches = root.querySelectorAll(selector).length;
          if (matches === 0 && root.matches(selector)) {
            matches = 1;
          }
          presence[selector] = matches > 0;
        }
        return presence;
      }

      function inspectorCounts() {
        return {
          panelHead: root.querySelectorAll(":scope > .panel-head").length,
          sections: root.querySelectorAll(":scope > .section").length,
          appliedPluginSections: root.querySelectorAll(
            ":scope > .section:has(.applied-plugin)",
          ).length,
          scrubs: root.querySelectorAll(".scrub[data-param]").length,
          automationMarks: root.querySelectorAll(
            ".automation-mark[data-automation]",
          ).length,
          objectAutomation: root.querySelectorAll(
            ".automation-mark[data-object-automation]",
          ).length,
          colorChips: root.querySelectorAll(".color-chip").length,
          linkTarget: root.querySelectorAll("#link-target").length,
          effectDescription: root.querySelectorAll(
            "#effect-parameter-description",
          ).length,
          errorNotice: root.querySelectorAll(".notice.error").length,
          devInfo: root.querySelectorAll(":scope > details.dev-info").length,
        };
      }

      return {
        structure: inspectorStructure(root),
        aria: inspectorAria(ariaSelectors),
        styles: inspectorStyles(styleSelectors, styleProperties),
        presence: inspectorPresence(styleSelectors),
        counts: inspectorCounts(),
      };
    },
    {
      ariaSelectors: ARIA_SELECTOR_LIST,
      styleSelectors: STYLE_SELECTOR_LIST,
      styleProperties: STYLE_PROPERTIES,
    },
  );
}

async function referencePages(browser) {
  const context = await browser.newContext({
    viewport: VIEWPORT,
    deviceScaleFactor: 1,
    colorScheme: "dark",
    locale: "ja-JP",
    reducedMotion: "reduce",
  });
  return {
    context,
    legacyPage: await context.newPage(),
    reactPage: await context.newPage(),
  };
}

function assertCountsMatch(mode, counts) {
  const expected = EXPECTED_COUNTS[mode];
  for (const [key, value] of Object.entries(expected)) {
    expect(
      counts[key],
      `${mode} ${key}`,
    ).toBe(value);
  }
}

function assertNoPlainInstalledControls(counts) {
  expect(counts.objectAutomation).toBe(0);
  expect(counts.colorChips).toBe(0);
  expect(counts.linkTarget).toBe(0);
}

function assertInstalledPresence(presence) {
  for (const selector of STYLE_SELECTOR_LIST) {
    if (INSTALLED_PRESENCE_EXCLUSIONS.has(selector)) {
      continue;
    }
    expect(presence[selector], `installed presence ${selector}`).toBe(true);
  }
}

function assertBlockedPresence(presence) {
  expect(presence["#inspector .notice"]).toBe(true);
  expect(presence["#inspector .action-strip .btn"]).toBe(true);
}

test.describe("inspector parity oracle", () => {
  test.describe.configure({ retries: 0 });
  test.use({ viewport: VIEWPORT });

  test("served legacy source SHA256 matches authority", async ({ request }) => {
    const response = await request.get(LEGACY_SOURCE_FETCH_URL);
    expect(response.ok()).toBe(true);
    const body = await response.body();
    const digest = createHash("sha256").update(body).digest("hex");
    expect(digest).toBe(LEGACY_SOURCE_SHA256);
  });

  test("spec self-protection guards", () => {
    const source = readFileSync(fileURLToPath(import.meta.url), "utf8");
    const join = (parts) => parts.join("");
    const forbiddenTokens = [
      join(["toMatch", "Snap", "shot"]),
      join(["update", "Snap", "shot"]),
      join(["test", ".", "skip"]),
      join(["test", ".", "fixme"]),
      join(["eslint", "-", "disable"]),
      join(["@ts", "-", "ignore"]),
      join(["@ts", "-", "expect", "-", "error"]),
      join(["toBeTruthy", "(", ")"]),
      join(["toBeFalsy", "(", ")"]),
      join(["expect", ".", "soft"]),
    ];
    for (const token of forbiddenTokens) {
      expect(source.includes(token), `forbidden token: ${token}`).toBe(false);
    }
  });

  for (const mode of CROSS_PAGE_MODES) {
    test(`cross-page inspector parity: ${mode}`, async ({ browser }) => {
      const { context, legacyPage, reactPage } = await referencePages(browser);
      try {
        await Promise.all([
          settle(legacyPage, LEGACY_URL),
          settle(reactPage, REACT_ARCHIVE_URL, { parserBridge: true }),
        ]);
        await Promise.all([
          selectMode(legacyPage, mode),
          selectMode(reactPage, mode),
        ]);

        const [legacyOracle, reactOracle] = await Promise.all([
          readInspectorOracle(legacyPage),
          readInspectorOracle(reactPage),
        ]);

        assertCountsMatch(mode, legacyOracle.counts);
        assertCountsMatch(mode, reactOracle.counts);
        if (mode === "installed") {
          assertInstalledPresence(legacyOracle.presence);
          assertInstalledPresence(reactOracle.presence);
        }
        if (mode === "blocked") {
          assertBlockedPresence(legacyOracle.presence);
          assertBlockedPresence(reactOracle.presence);
          await expect(
            legacyPage.locator("#inspector .action-strip .btn[disabled]"),
          ).toHaveCount(1);
          await expect(
            reactPage.locator("#inspector .action-strip .btn[disabled]"),
          ).toHaveCount(1);
        }
        if (mode !== "installed") {
          assertNoPlainInstalledControls(legacyOracle.counts);
          assertNoPlainInstalledControls(reactOracle.counts);
          expect(legacyOracle.counts.scrubs).toBe(0);
          expect(reactOracle.counts.scrubs).toBe(0);
          expect(legacyOracle.counts.automationMarks).toBe(0);
          expect(reactOracle.counts.automationMarks).toBe(0);
        }

        expect(reactOracle.structure).toEqual(legacyOracle.structure);
        expect(reactOracle.aria).toEqual(legacyOracle.aria);
        expect(reactOracle.styles).toEqual(legacyOracle.styles);
        expect(reactOracle.presence).toEqual(legacyOracle.presence);
        expect(reactOracle.counts).toEqual(legacyOracle.counts);

        await expect(
          reactPage.locator("#inspector [data-react-surface]"),
        ).toHaveCount(0);
      } finally {
        await context.close();
      }
    });
  }

  test("effect-focused React-only inspector pin", async ({ browser }) => {
    test.setTimeout(60_000);
    const context = await browser.newContext({
      viewport: VIEWPORT,
      deviceScaleFactor: 1,
      colorScheme: "dark",
      locale: "ja-JP",
      reducedMotion: "reduce",
    });
    const candidatePage = await context.newPage();
    const archivePage = await context.newPage();
    try {
      await settle(candidatePage, REACT_CANDIDATE_URL, {
        parserBridge: true,
        waitInboxEmpty: false,
      });
      await expect(
        candidatePage.locator("#vism-browser.candidate-plugin-browser"),
      ).toHaveCount(1);
      await expect(
        candidatePage.locator(
          '.candidate-plugin-card[data-browser-item="echo-bloom"].is-selected',
        ),
      ).toHaveCount(1);
      await expect(candidatePage.locator("#stage")).toHaveAttribute(
        "data-mode",
        "installed",
      );

      const oracle = await readInspectorOracle(candidatePage);
      for (const [key, value] of Object.entries(EFFECT_FOCUSED_COUNTS)) {
        expect(oracle.counts[key], `effect-focused ${key}`).toBe(value);
      }

      await expect(
        candidatePage.locator("#effect-parameter-description"),
      ).toHaveText(
        "Layered light pulses that follow the selected object. Adjust Intensity and Spread while watching the Stage.",
      );

      const sectionTitles = await candidatePage
        .locator("#inspector .section-title")
        .evaluateAll((elements) =>
          elements.map((element) => {
            const span = element.querySelector("span");
            const clone = element.cloneNode(true);
            clone.querySelector("span")?.remove();
            return {
              primary: clone.textContent.replace(/\s+/g, " ").trim(),
              span: span
                ? span.textContent.replace(/\s+/g, " ").trim()
                : "",
            };
          }),
        );
      expect(sectionTitles).toEqual([
        { primary: "EDITING EFFECT", span: "ON OBJECT" },
        { primary: "ECHO BLOOM", span: "HOST PANEL" },
      ]);

      await expect(candidatePage.locator("#intensity")).toHaveAttribute(
        "aria-label",
        "Intensity。無限目盛を左右dragして変更",
      );
      await expect(candidatePage.locator("#spread")).toHaveAttribute(
        "aria-label",
        "Spread。無限目盛を左右dragして変更",
      );
      await expect(
        candidatePage.locator('.automation-mark[data-automation="intensity"]'),
      ).toHaveAttribute("aria-pressed", "true");
      await expect(
        candidatePage.locator('.automation-mark[data-automation="spread"]'),
      ).toHaveAttribute("aria-pressed", "false");

      await settle(archivePage, REACT_ARCHIVE_URL, { parserBridge: true });
      await selectMode(archivePage, "installed");

      const [candidateShell, archiveShell] = await Promise.all([
        candidatePage.locator("#inspector").evaluate((root) => {
          const style = getComputedStyle(root);
          return { display: style.display, overflow: style.overflow };
        }),
        archivePage.locator("#inspector").evaluate((root) => {
          const style = getComputedStyle(root);
          return { display: style.display, overflow: style.overflow };
        }),
      ]);
      expect(candidateShell).toEqual(archiveShell);

      assertNoPlainInstalledControls(oracle.counts);
    } finally {
      await context.close();
    }
  });

  test("installed interaction oracles match across legacy and archive", async ({
    browser,
  }) => {
    test.setTimeout(120_000);
    const { context, legacyPage, reactPage } = await referencePages(browser);

    async function runInteractionSequence(page, url, parserBridge) {
      await settle(page, url, { parserBridge });
      await selectMode(page, "installed");

      const scaleMark = page.locator(
        '.automation-mark[data-object-automation="scale"]',
      );
      await scaleMark.click();
      await expect(scaleMark).toHaveClass(/on/);
      await expect(scaleMark).toHaveAttribute("aria-pressed", "true");
      await expect(scaleMark).toHaveAttribute(
        "aria-label",
        "scale automation on",
      );
      await expect(page.locator("#scale-object-state i").first()).toHaveText(
        "AUTO ON",
      );
      await expect(page.locator("#undo-state")).toHaveText("Undo 1 · scale automation");

      const spreadMark = page.locator(
        '.automation-mark[data-automation="spread"]',
      );
      await spreadMark.click();
      await expect(spreadMark).toHaveAttribute("aria-pressed", "true");
      await expect(page.locator("#spread-auto-state")).toHaveText("AUTO ON");
      await expect(page.locator("#undo-state")).toHaveText(
        "Undo 1 · spread automation",
      );

      const intensityScrub = page.locator("#intensity");
      await intensityScrub.focus();
      await page.keyboard.press("ArrowRight");
      await expect(page.locator("#intensity-read")).toHaveText("65%");
      await expect(intensityScrub).toHaveAttribute("style", /--dial-shift:\s*130/);
      await page.keyboard.press("Shift+ArrowRight");
      await expect(page.locator("#intensity-read")).toHaveText("75%");
      await expect(intensityScrub).toHaveAttribute("style", /--dial-shift:\s*150/);
      await page.keyboard.press("ArrowLeft");
      await expect(page.locator("#intensity-read")).toHaveText("74%");
      await expect(page.locator("#undo-state")).toHaveText("Undo 1 · Intensity");
      const glow = await page
        .locator("#stage")
        .evaluate((stage) => stage.style.getPropertyValue("--glow"));
      expect(glow).toBe("0.74");

      const spreadScrub = page.locator("#spread");
      await spreadScrub.focus();
      for (let step = 0; step < 5; step += 1) {
        await page.keyboard.press("Shift+ArrowLeft");
      }
      await expect(page.locator("#spread-read")).toHaveText("0%");
      await page.keyboard.press("ArrowLeft");
      await expect(page.locator("#spread-read")).toHaveText("0%");

      const structureBeforeFill = (
        await readInspectorOracle(page)
      ).structure;
      await page
        .locator('.color-chip[data-color-channel="Fill"]')
        .click();
      await expect(page.locator("#color-book-drawer.open")).toHaveCount(1);
      await expect(page.locator("#color-book-drawer")).toHaveAttribute(
        "aria-hidden",
        "false",
      );
      await expect(page.locator("#color-target")).toHaveText("FILL · PREVIEW");
      await expect(page.locator("#apply-color")).toHaveText("Apply to Fill");
      const structureAfterFill = (
        await readInspectorOracle(page)
      ).structure;
      expect(structureAfterFill).toEqual(structureBeforeFill);

      await page.locator("#surface-colors").click();
      await expect(page.locator("#color-book-drawer.open")).toHaveCount(0);
      await expect(page.locator("#color-book-drawer")).toHaveAttribute(
        "aria-hidden",
        "true",
      );

      const structureBeforeLink = (
        await readInspectorOracle(page)
      ).structure;
      await page.locator("#link-target").click();
      await expect(page.locator("#stage")).toHaveClass(/interaction-connect/);
      await expect(page.locator("#color-book-drawer.open")).toHaveCount(0);
      const structureAfterLink = (await readInspectorOracle(page)).structure;
      expect(structureAfterLink).toEqual(structureBeforeLink);

      return {
        scalePressed: await scaleMark.getAttribute("aria-pressed"),
        spreadPressed: await spreadMark.getAttribute("aria-pressed"),
        intensityRead: await page.locator("#intensity-read").textContent(),
        spreadRead: await page.locator("#spread-read").textContent(),
        undoLabel: await page.locator("#undo-state").textContent(),
        glow,
        stageClass: await page.locator("#stage").getAttribute("class"),
        colorTarget: await page.locator("#color-target").textContent(),
        applyColor: await page.locator("#apply-color").textContent(),
        drawerOpenAfterLink: await page
          .locator("#color-book-drawer.open")
          .count(),
      };
    }

    try {
      const legacyResult = await runInteractionSequence(
        legacyPage,
        LEGACY_URL,
        false,
      );
      const reactResult = await runInteractionSequence(
        reactPage,
        REACT_ARCHIVE_URL,
        true,
      );
      expect(reactResult).toEqual(legacyResult);
      expect(legacyResult.drawerOpenAfterLink).toBe(0);
      expect(reactResult.drawerOpenAfterLink).toBe(0);
    } finally {
      await context.close();
    }
  });
});
