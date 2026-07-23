import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "@playwright/test";
import { createServer } from "vite";
import { clamp } from "../src/candidates/easing-graph-model.js";

export const REFERENCE_SCREENS = [
  "empty-browser",
  "mixed-timeline",
  "parameter-easing",
  "stage-frame-tools",
  "shared-effect-relative",
];

export const REFERENCE_SEMANTICS = Object.freeze({
  "empty-browser": ["empty-project", "asset-browser", "transport", "context-explanation"],
  "mixed-timeline": ["video", "audio", "shape", "text", "group", "selection", "mute", "keyframe", "bake-cache"],
  "parameter-easing": ["selected-parameter", "keyframe", "easing-popup", "focus", "warning", "disabled"],
  "stage-frame-tools": ["stage", "output-frame", "inside-object", "outside-object", "scrim", "selection", "camera", "hand"],
  "shared-effect-relative": ["shared-definition", "three-nonadjacent-uses", "stack-position", "connection-gutter", "from-out", "use-in", "fold-count", "normal-drag", "relative-hud"],
});

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const EXPECTED_BROWSER_VERSION = "149.0.7827.55";
const EXPECTED_BROWSER_REVISION = "1228";

async function readBrowserDescriptor() {
  const packagePath = fileURLToPath(import.meta.resolve("playwright-core/package.json"));
  const descriptorPath = path.join(path.dirname(packagePath), "browsers.json");
  const descriptors = JSON.parse(await readFile(descriptorPath, "utf8"));
  const descriptor = descriptors.browsers.find(
    (entry) => entry.name === "chromium-headless-shell",
  );
  if (
    descriptor?.browserVersion !== EXPECTED_BROWSER_VERSION ||
    descriptor?.revision !== EXPECTED_BROWSER_REVISION
  ) {
    throw new Error(
      `reference browser descriptor mismatch: ${descriptor?.browserVersion}/${descriptor?.revision}`,
    );
  }
  return descriptor;
}

async function readFixtures(fixturePaths) {
  const fixtures = {};
  for (const id of ["document", "scenes", "tokens"]) {
    const filename = fixturePaths[id];
    if (typeof filename !== "string") throw new TypeError(`missing ${id} fixture path`);
    const value = JSON.parse(await readFile(filename, "utf8"));
    if (!value || typeof value !== "object" || Array.isArray(value)) {
      throw new TypeError(`${id} fixture root must be an object`);
    }
    fixtures[id] = value;
  }
  return fixtures;
}

function assertSemanticSet(screenId, ids) {
  const expected = REFERENCE_SEMANTICS[screenId];
  const actual = [...ids].sort();
  if (
    actual.length !== new Set(actual).size ||
    actual.join("\0") !== [...expected].sort().join("\0")
  ) {
    throw new Error(`${screenId} semantic IDs differ from the closed set`);
  }
}

export async function renderReferenceNormals({ fixturePaths }) {
  if (clamp(2, 0, 1) !== 1) throw new Error("source model closure is unavailable");
  await readBrowserDescriptor();
  const fixtures = await readFixtures(fixturePaths);
  const vite = await createServer({
    root: ROOT,
    logLevel: "error",
    server: { host: "127.0.0.1", port: 0, strictPort: false },
  });
  let browser;
  try {
    await vite.listen();
    const baseUrl = vite.resolvedUrls?.local?.[0];
    if (!baseUrl) throw new Error("Vite did not expose a local reference URL");
    const allowedOrigin = new URL(baseUrl).origin;
    browser = await chromium.launch({ headless: true });
    if (browser.version() !== EXPECTED_BROWSER_VERSION) {
      throw new Error(
        `reference browser version mismatch: ${browser.version()}`,
      );
    }
    const context = await browser.newContext({
      viewport: { width: 1440, height: 900 },
      deviceScaleFactor: 1,
      locale: "en-US",
      timezoneId: "UTC",
      colorScheme: "dark",
      reducedMotion: "reduce",
    });
    await context.addInitScript((value) => {
      globalThis.__MOTOLII_REFERENCE_FIXTURES__ = value;
    }, fixtures);
    const page = await context.newPage();
    const rejected = [];
    await page.route("**/*", async (route) => {
      const url = new URL(route.request().url());
      if (url.origin === allowedOrigin || ["data:", "blob:"].includes(url.protocol)) {
        await route.continue();
      } else {
        rejected.push(url.href);
        await route.abort("blockedbyclient");
      }
    });
    const captures = new Map();
    for (const screenId of REFERENCE_SCREENS) {
      await page.goto(`${baseUrl}#reference/${screenId}`, { waitUntil: "networkidle" });
      await page.locator(`[data-reference-screen="${screenId}"]`).waitFor();
      await page.addStyleTag({
        content: "*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}",
      });
      const fonts = await page.evaluate(async () => {
        await document.fonts.ready;
        const regular = await document.fonts.load('400 12px "MotoliiReferenceInter"');
        const semibold = await document.fonts.load('600 12px "MotoliiReferenceInter"');
        const referenceRoot = document.querySelector(".motolii-mock-app");
        return {
          regular: regular.length,
          semibold: semibold.length,
          readyRegular: document.fonts.check('400 12px "MotoliiReferenceInter"'),
          readySemibold: document.fonts.check('600 12px "MotoliiReferenceInter"'),
          interfaceRole: getComputedStyle(referenceRoot).fontFamily,
          technicalRole: getComputedStyle(referenceRoot)
            .getPropertyValue("--mock-role-font-technical"),
        };
      });
      if (
        !fonts.regular ||
        !fonts.semibold ||
        !fonts.readyRegular ||
        !fonts.readySemibold ||
        !fonts.interfaceRole.includes("MotoliiReferenceInter") ||
        !fonts.technicalRole.includes("MotoliiReferenceInter")
      ) {
        throw new Error(`reference font fallback detected on ${screenId}`);
      }
      const semanticIds = await page
        .locator("[data-semantic-id]")
        .evaluateAll((elements) => elements.map((element) => element.dataset.semanticId));
      assertSemanticSet(screenId, semanticIds);
      if (["mixed-timeline", "shared-effect-relative"].includes(screenId)) {
        const bars = await page.locator(".time-bar").evaluateAll((elements) =>
          elements.map((element) => {
            const rect = element.getBoundingClientRect();
            return {
              background: getComputedStyle(element).backgroundColor,
              bottom: rect.bottom,
              left: rect.left,
              right: rect.right,
              top: rect.top,
              visible: rect.width > 0 && rect.height > 0,
              viewportHeight: window.innerHeight,
              viewportWidth: window.innerWidth,
            };
          }),
        );
        if (
          bars.length !== 6 ||
          bars.some(
            (bar) =>
              !bar.visible ||
              bar.left < 0 ||
              bar.top < 0 ||
              bar.right > bar.viewportWidth ||
              bar.bottom > bar.viewportHeight ||
              bar.background === "rgba(0, 0, 0, 0)" ||
              bar.background === "transparent",
          )
        ) {
          throw new Error(`reference timeline bars are clipped or transparent on ${screenId}`);
        }
      }
      captures.set(
        screenId,
        await page.screenshot({
          animations: "disabled",
          caret: "hide",
          fullPage: false,
          type: "png",
        }),
      );
    }
    if (rejected.length > 0) {
      throw new Error(`reference capture attempted external network: ${rejected.join(", ")}`);
    }
    await context.close();
    return captures;
  } finally {
    await browser?.close();
    await vite.close();
  }
}

export const REFERENCE_BROWSER = Object.freeze({
  name: "Chromium Headless Shell",
  version: EXPECTED_BROWSER_VERSION,
  revision: EXPECTED_BROWSER_REVISION,
  viewport: Object.freeze({ width: 1440, height: 900 }),
  deviceScaleFactor: 1,
  locale: "en-US",
  timezoneId: "UTC",
  colorScheme: "dark",
  reducedMotion: "reduce",
});
