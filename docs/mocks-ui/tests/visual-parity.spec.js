import { expect, test } from "@playwright/test";
import pixelmatch from "pixelmatch";
import { PNG } from "pngjs";

const VIEWPORT = { width: 1440, height: 900 };
const LEGACY_URL =
  "http://127.0.0.1:5174/docs/mocks/m3-vism-host-boundary.html";
const REACT_URL = "http://127.0.0.1:5173/";
const HASH_FIXTURES = [
  "asset-explorer",
  "inbox-empty",
  "color-book",
  "z-rail",
  "easing-interval",
  "settings",
];
const FIXTURE_STATE_SELECTORS = {
  "all-surfaces": [
    '.browser-tab[data-tab="project"].on',
    '.asset-source-tab[data-asset-source="files"].on',
    '#color-book-drawer.open[aria-hidden="false"]',
    "#timeline.depth-open",
    '#depth-rail[aria-hidden="false"]',
  ],
  "asset-explorer": [
    '.browser-tab[data-tab="project"].on',
    '.asset-source-tab[data-asset-source="files"].on',
  ],
  "inbox-empty": ["#inbox.empty"],
  "color-book": ['#color-book-drawer.open[aria-hidden="false"]'],
  "z-rail": ["#timeline.depth-open", '#depth-rail[aria-hidden="false"]'],
  "easing-interval": ['#easing-panel.open[aria-hidden="false"]'],
  settings: ["#settings-sheet.open"],
};

// 同じChrome・同じviewportでの再構成なので、面積差1%は構造回帰として扱う。
const MAX_MISMATCH_RATIO = 0.01;

async function settle(page, url, { parserBridge = false } = {}) {
  // ViteのHMR socketは接続を保つため、networkidleを完了条件にしない。
  await page.goto(url, { waitUntil: "domcontentloaded" });
  await page.locator(".app").waitFor({ state: "visible" });
  if (parserBridge) {
    await page
      .locator('.app[data-parity-ready="true"]')
      .waitFor({ state: "visible" });
  }
  // markerだけで済ませず、旧scriptがhash fixtureを適用した結果も審判する。
  const fixture = new URL(url).hash.slice(1) || "all-surfaces";
  await Promise.all(
    (FIXTURE_STATE_SELECTORS[fixture] ?? []).map((selector) =>
      page.locator(selector).waitFor({ state: "visible" }),
    ),
  );
  await page.evaluate(async () => {
    await document.fonts.ready;
    await new Promise((resolve) =>
      requestAnimationFrame(() => requestAnimationFrame(resolve)),
    );
  });
}

async function capture(page, url, options) {
  await settle(page, url, options);
  return page.screenshot({
    animations: "disabled",
    caret: "hide",
    fullPage: false,
    scale: "css",
  });
}

function comparePng(actualBuffer, expectedBuffer) {
  const actual = PNG.sync.read(actualBuffer);
  const expected = PNG.sync.read(expectedBuffer);

  expect(
    { width: actual.width, height: actual.height },
    "React fixture and legacy golden must render at the same dimensions",
  ).toEqual({ width: expected.width, height: expected.height });

  const diff = new PNG({ width: actual.width, height: actual.height });
  const mismatchedPixels = pixelmatch(
    expected.data,
    actual.data,
    diff.data,
    actual.width,
    actual.height,
    {
      threshold: 0.1,
      includeAA: false,
      diffColor: [255, 68, 68],
      diffColorAlt: [68, 160, 255],
      alpha: 0.65,
    },
  );

  return {
    diff: PNG.sync.write(diff),
    mismatchedPixels,
    mismatchRatio: mismatchedPixels / (actual.width * actual.height),
  };
}

async function landmarkBoxes(page) {
  return page.locator(".app").evaluate((app) => {
    const selectors = [
      ".titlebar",
      ".commandbar",
      ".workspace",
      ".browser",
      ".browser-tabs",
      ".plugin-grid",
      ".stage-shell",
      ".inspector",
      ".timeline",
      ".timeline-body",
      ".inbox",
      ".time-grid",
      ".settings-sheet",
      ".status",
    ];

    return Object.fromEntries(
      selectors.map((selector) => {
        const element = app.querySelector(selector);
        if (!element) return [selector, null];
        const box = element.getBoundingClientRect();
        return [
          selector,
          {
            x: Math.round(box.x),
            y: Math.round(box.y),
            width: Math.round(box.width),
            height: Math.round(box.height),
          },
        ];
      }),
    );
  });
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

test.describe("legacy mock visual parity", () => {
  test.use({ viewport: VIEWPORT });

  test("#all-surfaces preserves the original composition", async ({
    browser,
  }, testInfo) => {
    test.setTimeout(60_000);
    const { context, legacyPage, reactPage } =
      await referencePages(browser);

    try {
      const [legacy, react] = await Promise.all([
        capture(legacyPage, `${LEGACY_URL}#all-surfaces`),
        capture(reactPage, `${REACT_URL}#archive/all-surfaces`, {
          parserBridge: true,
        }),
      ]);
      const comparison = comparePng(react, legacy);

      await Promise.all([
        testInfo.attach("legacy-all-surfaces", {
          body: legacy,
          contentType: "image/png",
        }),
        testInfo.attach("react-all-surfaces", {
          body: react,
          contentType: "image/png",
        }),
        testInfo.attach("visual-diff", {
          body: comparison.diff,
          contentType: "image/png",
        }),
      ]);

      testInfo.annotations.push({
        type: "visual mismatch",
        description: `${(comparison.mismatchRatio * 100).toFixed(3)}%`,
      });

      expect(
        comparison.mismatchRatio,
        `${comparison.mismatchedPixels} pixels differ (${(
          comparison.mismatchRatio * 100
        ).toFixed(3)}%); expected at most ${MAX_MISMATCH_RATIO * 100}%`,
      ).toBeLessThanOrEqual(MAX_MISMATCH_RATIO);

      const [legacyLandmarks, reactLandmarks] = await Promise.all([
        landmarkBoxes(legacyPage),
        landmarkBoxes(reactPage),
      ]);
      expect(reactLandmarks).toEqual(legacyLandmarks);

      await expect(reactPage.locator(".app")).toHaveCount(1);
      await expect(reactPage.locator(".workspace")).toHaveCount(1);
      await expect(reactPage.getByLabel("譜面")).toBeVisible();
      await expect(
        reactPage.getByRole("region", { name: "Color Book", exact: true }),
      ).toBeVisible();
    } finally {
      await context.close();
    }
  });

  for (const fixture of HASH_FIXTURES) {
    test(`#${fixture} preserves the original fixture state`, async ({
      browser,
    }, testInfo) => {
      const { context, legacyPage, reactPage } =
        await referencePages(browser);

      try {
        const [legacy, react] = await Promise.all([
          capture(legacyPage, `${LEGACY_URL}#${fixture}`),
          capture(reactPage, `${REACT_URL}#archive/${fixture}`, {
            parserBridge: true,
          }),
        ]);
        const comparison = comparePng(react, legacy);

        if (comparison.mismatchRatio > MAX_MISMATCH_RATIO) {
          await Promise.all([
            testInfo.attach(`legacy-${fixture}`, {
              body: legacy,
              contentType: "image/png",
            }),
            testInfo.attach(`react-${fixture}`, {
              body: react,
              contentType: "image/png",
            }),
            testInfo.attach(`diff-${fixture}`, {
              body: comparison.diff,
              contentType: "image/png",
            }),
          ]);
        }

        const [legacyLandmarks, reactLandmarks] = await Promise.all([
          landmarkBoxes(legacyPage),
          landmarkBoxes(reactPage),
        ]);
        expect(reactLandmarks).toEqual(legacyLandmarks);

        expect(
          comparison.mismatchRatio,
          `${fixture}: ${comparison.mismatchedPixels} pixels differ (${(
            comparison.mismatchRatio * 100
          ).toFixed(3)}%)`,
        ).toBeLessThanOrEqual(MAX_MISMATCH_RATIO);
      } finally {
        await context.close();
      }
    });
  }
});
