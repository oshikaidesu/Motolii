import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { chromium } from "@playwright/test";
import { createServer } from "vite";

import {
  CURRENT_ROUTE_SCREENS,
  CURRENT_ROUTE_MANIFEST_SCHEMA_VERSION,
  validateCurrentRouteManifest,
} from "./current-route-generation.mjs";
import {
  currentRouteSourceManifestSha256,
  verifyCurrentRouteSourceClosure,
} from "./current-route-provenance.mjs";
import { repositoryFingerprint } from "./repository-fingerprint.mjs";
import {
  REFERENCE_TRANSFORM_VERSION,
  REFERENCE_VARIANTS,
  deriveReferencePng,
} from "./reference-transform.mjs";

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function capturePathFor(screen, variant) {
  return `captures/${screen}.${variant}.png`;
}

function normalizeLocaleTimeZone(value) {
  return {
    locale: value.locale,
    timeZone: value.timeZone,
  };
}

function firstFontToken(family) {
  if (typeof family !== "string") return "";
  const first = family.split(",")[0] ?? "";
  return first.trim().replace(/^['"]|['"]$/g, "");
}

export const CURRENT_ROUTE_SOURCE_ENTRIES = Object.freeze([
  "docs/mocks-ui/src/main.jsx",
  "docs/mocks-ui/public/reference-fonts/inter-latin-400-normal.woff2",
  "docs/mocks-ui/public/reference-fonts/inter-latin-600-normal.woff2",
  "docs/mocks-ui/scripts/reference-transform.mjs",
  "docs/mocks-ui/package-lock.json",
]);

export const CURRENT_ROUTE_CAPTURE_PLAN = Object.freeze([
  {
    screen: CURRENT_ROUTE_SCREENS[0],
    modeKind: "screen-1",
    operation: async () => {},
    oracle: async (page) => {
      await page.locator('#root[data-current-route-capture-ready="true"]').waitFor({
        state: "attached",
      });
    },
  },
  {
    screen: CURRENT_ROUTE_SCREENS[1],
    modeKind: "normal",
    operation: async () => {},
    oracle: async (page) => {
      await page.locator(".app[data-parity-ready='true']").waitFor({ state: "visible" });
    },
  },
  {
    screen: CURRENT_ROUTE_SCREENS[2],
    modeKind: "normal",
    operation: async (page) => {
      await page.locator(".app[data-parity-ready='true']").waitFor({
        state: "visible",
      });
      await page
        .getByRole("button", {
          name: "Pulse rings · IntensityのInterval Easing Editorを開く",
        })
        .click();
    },
    oracle: async (page) => {
      const panel = page.getByRole("complementary", {
        name: "Interval Easing Editor",
      });
      await panel.waitFor({ state: "visible" });
      const hidden = await panel.getAttribute("aria-hidden");
      if (hidden !== "false") {
        throw new Error("parameter easing panel is not visible");
      }
    },
  },
  {
    screen: CURRENT_ROUTE_SCREENS[3],
    modeKind: "normal",
    operation: async (page) => {
      await page.locator(".app[data-parity-ready='true']").waitFor({
        state: "visible",
      });
      await page.locator('[aria-label="Hand"]').click();
    },
    oracle: async (page) => {
      const handButton = page.locator('[aria-label="Hand"]');
      const isActive = await handButton.evaluate((button) =>
        button.classList.contains("active"),
      );
      const stageClass = await page.locator("#stage").getAttribute("class");
      if (!isActive || !stageClass?.split(/\s+/).includes("interaction-hand")) {
        throw new Error("hand tool interaction state is not ready");
      }
    },
  },
  {
    screen: CURRENT_ROUTE_SCREENS[4],
    modeKind: "normal",
    operation: async (page) => {
      await page.locator(".app[data-parity-ready='true']").waitFor({
        state: "visible",
      });
      await page.locator('[aria-label="Relative Move"]').click();
    },
    oracle: async (page) => {
      const relativeButton = page.locator('[aria-label="Relative Move"]');
      const isActive = await relativeButton.evaluate((button) =>
        button.classList.contains("active"),
      );
      const stageClass = await page.locator("#stage").getAttribute("class");
      await page.locator('[aria-label="Relative Move motion path"]').waitFor({
        state: "visible",
      });
      if (
        !isActive ||
        !stageClass?.split(/\s+/).includes("interaction-relative")
      ) {
        throw new Error("relative move interaction state is not ready");
      }
    },
  },
]);

export function isAllowedCaptureRequest(url, allowedOrigins) {
  const allowed = allowedOrigins instanceof Set ? allowedOrigins : new Set(allowedOrigins);
  const parsed = new URL(url);
  if (parsed.protocol === "data:" || parsed.protocol === "blob:") return true;
  return allowed.has(parsed.origin);
}

export function assertCaptureLocale(observed) {
  const observedLocale = normalizeLocaleTimeZone(observed);
  if (observedLocale.locale !== "ja-JP" || observedLocale.timeZone !== "UTC") {
    throw new Error("locale/timeZone mismatch for current-route capture");
  }
}

export function assertCaptureFont(observed) {
  if (
    !observed ||
    observed.loadOk !== true ||
    observed.check400 !== true ||
    observed.check600 !== true
  ) {
    throw new Error("required Inter font faces are not ready");
  }
  if (firstFontToken(observed.computedFamily) !== "Inter") {
    throw new Error("computed font family is not Inter");
  }
}

function readSourceClosureManifestBytes(repoRoot) {
  return readFile(path.join(repoRoot, "docs/mocks-ui/current-route-provenance.json"), "utf8");
}

async function readChromiumDescriptor() {
  const packagePath = fileURLToPath(import.meta.resolve("playwright-core/package.json"));
  const browsersPath = path.join(path.dirname(packagePath), "browsers.json");
  const content = await readFile(browsersPath, "utf8");
  const descriptor = JSON.parse(content);
  const browser = descriptor.browsers?.find(
    (entry) => entry?.name === "chromium-headless-shell",
  );
  if (!browser) {
    throw new Error("chromium-headless-shell descriptor missing");
  }
  return browser;
}

async function readFontFixtureRecords(repoRoot) {
  const entries = [
    {
      path: "docs/mocks-ui/public/reference-fonts/inter-latin-400-normal.woff2",
      weight: 400,
    },
    {
      path: "docs/mocks-ui/public/reference-fonts/inter-latin-600-normal.woff2",
      weight: 600,
    },
  ].sort((left, right) => left.path.localeCompare(right.path));
  return Promise.all(
    entries.map(async ({ path: fontPath, weight }) => ({
      path: fontPath,
      sha256: sha256(await readFile(path.join(repoRoot, fontPath))),
      weight,
    })),
  );
}

async function captureScreen({ page, origin, allowedOrigins, plan, fontFixtureObservations }) {
  const allowed = new Set(allowedOrigins);
  const blocked = [];

  await page.route("**/*", (route) => {
    const requestUrl = route.request().url();
    if (isAllowedCaptureRequest(requestUrl, allowed)) {
      route.continue();
      return;
    }
    blocked.push(requestUrl);
    route.abort("blockedbyclient");
  });

  await page.goto(`${origin}#plugin-browser-candidate`, {
    waitUntil: "domcontentloaded",
  });

  await plan.operation(page);
  await plan.oracle(page);

  const observation = await page.evaluate(async (fonts) => {
    const locale = Intl.DateTimeFormat().resolvedOptions();
    const app = document.querySelector(".app");
    let check400 = false;
    let check600 = false;
    try {
      const regular = new FontFace("Inter", fonts.inter400, { weight: "400" });
      const semibold = new FontFace("Inter", fonts.inter600, { weight: "600" });
      await Promise.all([regular.load(), semibold.load()]);
      document.fonts.add(regular);
      document.fonts.add(semibold);
      await document.fonts.ready;
      check400 = document.fonts.check("400 12px Inter");
      check600 = document.fonts.check("600 12px Inter");
    } catch {
      return {
        locale: {
          locale: locale.locale,
          timeZone: locale.timeZone,
        },
        loadOk: false,
        check400: false,
        check600: false,
        computedFamily: app ? getComputedStyle(app).fontFamily : "",
      };
    }
    return {
      locale: {
        locale: locale.locale,
        timeZone: locale.timeZone,
      },
      loadOk: true,
      check400,
      check600,
      computedFamily: app ? getComputedStyle(app).fontFamily : "",
    };
  }, fontFixtureObservations);

  assertCaptureLocale(observation.locale);
  assertCaptureFont(observation);

  const capture = await page.screenshot({
    animations: "disabled",
    caret: "hide",
    fullPage: false,
    type: "png",
  });
  if (blocked.length > 0) {
    throw new Error(`capture attempted external network: ${blocked.join(", ")}`);
  }

  return {
    capture,
    computedFamily: observation.computedFamily,
  };
}

export async function captureCurrentRouteBundle({ repoRoot }) {
  const before = await repositoryFingerprint(repoRoot);
  const sourceManifestText = await readSourceClosureManifestBytes(repoRoot);
  const sourceManifest = JSON.parse(sourceManifestText);
  const sourceManifestSha256 = currentRouteSourceManifestSha256(
    Buffer.from(sourceManifestText, "utf8"),
  );

  await verifyCurrentRouteSourceClosure({
    repoRoot,
    provenance: sourceManifest,
    entries: CURRENT_ROUTE_SOURCE_ENTRIES,
  });

  const viteNoMode = await createServer({
    root: path.join(repoRoot, "docs/mocks-ui"),
    logLevel: "error",
    server: { host: "127.0.0.1", port: 0, strictPort: false },
  });
  const viteCapture = await createServer({
    root: path.join(repoRoot, "docs/mocks-ui"),
    mode: "current-route-capture",
    logLevel: "error",
    server: { host: "127.0.0.1", port: 0, strictPort: false },
  });

  const descriptor = await readChromiumDescriptor();
  const fontFiles = await readFontFixtureRecords(repoRoot);

  let browser;
  try {
    await Promise.all([viteNoMode.listen(), viteCapture.listen()]);

    const noModeUrl = viteNoMode.resolvedUrls?.local?.[0];
    const captureUrl = viteCapture.resolvedUrls?.local?.[0];
    if (!noModeUrl || !captureUrl) {
      throw new Error("Vite URL is unavailable");
    }

    const normalMode = viteNoMode.config.mode;
    const currentRouteMode = viteCapture.config.mode;
    if (normalMode !== "development" || currentRouteMode !== "current-route-capture") {
      throw new Error("unexpected vite mode configuration");
    }

    const modeByKind = {
      "screen-1": currentRouteMode,
      normal: normalMode,
    };

    browser = await chromium.launch({
      headless: true,
    });

    const browserVersion = browser.version();
    if (browserVersion !== descriptor.browserVersion) {
      throw new Error(
        `chromium headless shell mismatch: ${browserVersion}/${descriptor.browserVersion}`,
      );
    }

    const captures = new Map();
    let observedFamily;

    for (const entry of CURRENT_ROUTE_CAPTURE_PLAN) {
      const serverUrl =
        entry.modeKind === "screen-1" ? captureUrl : noModeUrl;
      const allowedOrigin = new URL(serverUrl).origin;
      const context = await browser.newContext({
        viewport: { width: 1440, height: 900 },
        deviceScaleFactor: 1,
        locale: "ja-JP",
        timezoneId: "UTC",
        colorScheme: "dark",
        reducedMotion: "reduce",
      });
      const page = await context.newPage();
      try {
        const { capture, computedFamily } = await captureScreen({
          page,
          origin: serverUrl,
          allowedOrigins: [allowedOrigin],
          plan: entry,
          fontFixtureObservations: {
            inter400: `url(${new URL(
              "reference-fonts/inter-latin-400-normal.woff2",
              serverUrl,
            )})`,
            inter600: `url(${new URL(
              "reference-fonts/inter-latin-600-normal.woff2",
              serverUrl,
            )})`,
          },
        });
        if (observedFamily === undefined) {
          observedFamily = computedFamily;
        } else if (observedFamily !== computedFamily) {
          throw new Error("computed font family differs between capture screens");
        }

        for (const variant of REFERENCE_VARIANTS) {
          const pathId = capturePathFor(entry.screen, variant);
          captures.set(
            pathId,
            variant === "normal" ? capture : deriveReferencePng(capture, variant),
          );
        }
      } finally {
        await context.close();
      }
    }

    if (observedFamily === undefined) {
      throw new Error("missing computed font family observation");
    }

    const hash = createHash("sha256");
    for (const screen of CURRENT_ROUTE_CAPTURE_PLAN.map(({ screen }) => screen)) {
      for (const variant of REFERENCE_VARIANTS) {
        const capturePath = capturePathFor(screen, variant);
        const bytes = captures.get(capturePath);
        if (!bytes) {
          throw new Error(`missing capture ${capturePath}`);
        }
        hash.update(Buffer.from(`${capturePath}\0`));
        hash.update(bytes);
      }
    }

    const aggregate = hash.digest("hex");

    const manifest = {
      schemaVersion: CURRENT_ROUTE_MANIFEST_SCHEMA_VERSION,
      generation: `${sourceManifestSha256.slice(0, 12)}-${aggregate.slice(0, 12)}`,
      sourceManifestSha256,
      transformVersion: REFERENCE_TRANSFORM_VERSION,
      environment: {
        viewport: {
          width: 1440,
          height: 900,
        },
        scale: 1,
        locale: "ja-JP",
        timezone: "UTC",
        theme: "dark",
        reducedMotion: "reduce",
        browserVersion,
        browserRevision: String(descriptor.revision),
        fontFixture: {
          files: fontFiles,
          computedFamily: observedFamily,
        },
      },
      screens: CURRENT_ROUTE_CAPTURE_PLAN.map((entry) => ({
        screen: entry.screen,
        mode: modeByKind[entry.modeKind],
      })),
      captures: CURRENT_ROUTE_CAPTURE_PLAN.flatMap((entry) =>
        REFERENCE_VARIANTS.map((variant) => ({
          path: capturePathFor(entry.screen, variant),
          screen: entry.screen,
          variant,
          sha256: sha256(captures.get(capturePathFor(entry.screen, variant))),
        })),
      ),
    };

    validateCurrentRouteManifest(manifest, { expectedVariants: REFERENCE_VARIANTS });

    const after = await repositoryFingerprint(repoRoot);
    if (before !== after) {
      throw new Error("repository fingerprint changed while capturing");
    }

    return {
      manifest,
      captures,
      repositoryFingerprint: after,
    };
  } finally {
    await browser?.close();
    await Promise.all([viteNoMode.close(), viteCapture.close()]);
  }
}
