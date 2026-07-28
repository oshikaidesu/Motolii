import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createServer } from "vite";
import test, { describe } from "node:test";
import assert from "node:assert/strict";

import {
  CURRENT_ROUTE_CAPTURE_PLAN,
  CURRENT_ROUTE_SOURCE_ENTRIES,
  assertCaptureFont,
  assertCaptureLocale,
  captureCurrentRouteBundle,
  isAllowedCaptureRequest,
} from "../scripts/current-route-capture.mjs";
import {
  REFERENCE_TRANSFORM_VERSION,
  REFERENCE_VARIANTS,
  decodedPixels,
} from "../scripts/reference-transform.mjs";
import {
  CurrentRouteGenerationError,
  validateCurrentRouteManifest,
} from "../scripts/current-route-generation.mjs";
import {
  CurrentRouteProvenanceError,
  verifyCurrentRouteSourceClosure,
} from "../scripts/current-route-provenance.mjs";
import { repositoryFingerprint } from "../scripts/repository-fingerprint.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "../../..");
const PROVENANCE_PATH = path.join(
  REPO_ROOT,
  "docs/mocks-ui/current-route-provenance.json",
);

async function readCurrentRouteProvenanceBytes() {
  return readFile(PROVENANCE_PATH);
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function expectGenerationCode(code, run) {
  return assert.rejects(
    Promise.resolve().then(run),
    (error) => {
      assert.ok(error instanceof CurrentRouteGenerationError);
      assert.equal(error.code, code);
      return true;
    },
  );
}

function expectProvenanceCode(code, run) {
  return assert.rejects(
    Promise.resolve().then(run),
    (error) => {
      assert.ok(error instanceof CurrentRouteProvenanceError);
      assert.equal(error.code, code);
      return true;
    },
  );
}

async function readFontManifest() {
  return JSON.parse(await readFile(PROVENANCE_PATH, "utf8"));
}

let bundlePromise;
async function getBundle() {
  if (!bundlePromise) {
    bundlePromise = captureCurrentRouteBundle({ repoRoot: REPO_ROOT });
  }
  return bundlePromise;
}

describe("current-route capture", { concurrency: false }, () => {
  test(
    "captures canonical 30 variants with valid schema and complete manifest",
    { timeout: 240000 },
    async () => {
      const before = await repositoryFingerprint(REPO_ROOT);
      const bundle = await getBundle();
      const after = await repositoryFingerprint(REPO_ROOT);

      assert.equal(after, before);
      assert.equal(bundle.repositoryFingerprint, before);
      assert.equal(bundle.captures.size, 30);

      const expectedPaths = CURRENT_ROUTE_CAPTURE_PLAN.flatMap((entry) =>
        REFERENCE_VARIANTS.map((variant) => `captures/${entry.screen}.${variant}.png`),
      );
      assert.deepEqual([...bundle.captures.keys()], expectedPaths);
      for (const capturePath of expectedPaths) {
        const bytes = bundle.captures.get(capturePath);
        assert.ok(bytes instanceof Uint8Array);
        const decoded = decodedPixels(bytes);
        assert.ok(decoded.length > 0);
      }

      const manifest = bundle.manifest;
      validateCurrentRouteManifest(manifest, {
        expectedVariants: REFERENCE_VARIANTS,
      });

      const expectedModes = [
        "current-route-capture",
        "development",
        "development",
        "development",
        "development",
      ];
      assert.equal(manifest.screens.length, 5);
      for (let i = 0; i < 5; i++) {
        const screen = manifest.screens[i];
        assert.equal(screen.screen, CURRENT_ROUTE_CAPTURE_PLAN[i].screen);
        assert.equal(screen.mode, expectedModes[i]);
      }

      assert.equal(manifest.environment.locale, "ja-JP");
      assert.equal(manifest.environment.timezone, "UTC");
      assert.equal(manifest.environment.theme, "dark");
      assert.equal(manifest.environment.reducedMotion, "reduce");
      assert.equal(manifest.environment.scale, 1);
      assert.equal(manifest.environment.viewport.width, 1440);
      assert.equal(manifest.environment.viewport.height, 900);
      assert.match(manifest.environment.browserVersion, /^\d+\.\d+\.\d+\.\d+$/);
      assert.match(manifest.environment.browserRevision, /^\d+$/);

      assert.equal(
        manifest.environment.fontFixture.computedFamily
          .split(",")[0]
          .trim()
          .replace(/^['"]|['"]$/g, ""),
        "Inter",
      );
      assert.equal(manifest.environment.fontFixture.files.length, 2);
      assert.deepEqual(
        manifest.environment.fontFixture.files,
        [
          {
            path: "docs/mocks-ui/public/reference-fonts/inter-latin-400-normal.woff2",
            sha256: sha256(
              await readFile(
                path.join(
                  REPO_ROOT,
                  "docs/mocks-ui/public/reference-fonts/inter-latin-400-normal.woff2",
                ),
              ),
            ),
            weight: 400,
          },
          {
            path: "docs/mocks-ui/public/reference-fonts/inter-latin-600-normal.woff2",
            sha256: sha256(
              await readFile(
                path.join(
                  REPO_ROOT,
                  "docs/mocks-ui/public/reference-fonts/inter-latin-600-normal.woff2",
                ),
              ),
            ),
            weight: 600,
          },
        ],
      );

      const manifestText = await readCurrentRouteProvenanceBytes();
      assert.equal(
        manifest.sourceManifestSha256,
        sha256(manifestText),
      );
      assert.equal(manifest.transformVersion, REFERENCE_TRANSFORM_VERSION);
      assert.equal(manifest.schemaVersion, 2);
      assert.deepEqual(
        manifest.screens.map(({ screen, mode }) => [screen, mode]),
        [
          ["empty-browser", "current-route-capture"],
          ["mixed-timeline", "development"],
          ["parameter-easing", "development"],
          ["stage-frame-tools", "development"],
          ["shared-effect-relative", "development"],
        ],
      );
    },
  );

  test("verifies closure from committed provenance", { timeout: 240000 }, async () => {
    const provenanceText = await readCurrentRouteProvenanceBytes();
    const provenance = JSON.parse(provenanceText);
    await verifyCurrentRouteSourceClosure({
      repoRoot: REPO_ROOT,
      provenance,
      entries: CURRENT_ROUTE_SOURCE_ENTRIES,
    });
  });

  test("rejects forbidden network requests and accepts data/blob", async () => {
    const vite = await createServer({
      root: path.join(REPO_ROOT, "docs/mocks-ui"),
      logLevel: "error",
      server: { host: "127.0.0.1", port: 0, strictPort: false },
    });
    try {
      await vite.listen();
      const origin = new URL(vite.resolvedUrls.local[0]).origin;
      const allowed = new Set([origin]);
      assert.equal(isAllowedCaptureRequest("data:text/plain,a", allowed), true);
      assert.equal(isAllowedCaptureRequest("blob:null/foo", allowed), true);
      assert.equal(isAllowedCaptureRequest(`${origin}/x`, allowed), true);
      assert.equal(isAllowedCaptureRequest("https://fonts.googleapis.com/x", allowed), false);
      assert.equal(isAllowedCaptureRequest("http://127.0.0.1:9999/x", allowed), false);
    } finally {
      await vite.close();
    }
  });

  test("assertCaptureLocale rejects mismatches", () => {
    assert.throws(() => assertCaptureLocale({ locale: "en-US", timeZone: "UTC" }));
    assert.throws(() => assertCaptureLocale({ locale: "ja-JP", timeZone: "Asia/Tokyo" }));
  });

  test("assertCaptureFont rejects missing checks or fallback family", () => {
    assert.throws(() =>
      assertCaptureFont({
        loadOk: false,
        check400: true,
        check600: true,
        computedFamily: "Inter, sans-serif",
      }),
    );
    assert.throws(() =>
      assertCaptureFont({
        loadOk: true,
        check400: false,
        check600: true,
        computedFamily: "Inter, sans-serif",
      }),
    );
    assert.throws(() =>
      assertCaptureFont({
        loadOk: true,
        check400: true,
        check600: false,
        computedFamily: "Inter, sans-serif",
      }),
    );
    assert.throws(() =>
      assertCaptureFont({
        loadOk: true,
        check400: true,
        check600: true,
        computedFamily: '"ui-sans-serif", sans-serif',
      }),
    );
  });

  test("manifest mutation is rejected", async () => {
    const bundle = await getBundle();
    const manifest = structuredClone(bundle.manifest);

    const swapped = structuredClone(manifest);
    const tmp = swapped.screens[0];
    swapped.screens[0] = swapped.screens[1];
    swapped.screens[1] = tmp;
    await expectGenerationCode("CR2-SCHEMA", () =>
      validateCurrentRouteManifest(swapped, {
        expectedVariants: REFERENCE_VARIANTS,
      }),
    );

    const extraField = structuredClone(manifest);
    extraField.extra = true;
    await expectGenerationCode("CR2-SCHEMA", () =>
      validateCurrentRouteManifest(extraField, {
        expectedVariants: REFERENCE_VARIANTS,
      }),
    );

    const captureMissing = structuredClone(manifest);
    captureMissing.captures.pop();
    await expectGenerationCode("CR2-SCHEMA", () =>
      validateCurrentRouteManifest(captureMissing, {
        expectedVariants: REFERENCE_VARIANTS,
      }),
    );

    const wrongVariant = structuredClone(manifest);
    wrongVariant.captures[0] = {
      ...wrongVariant.captures[0],
      variant: "ultraviolet",
      path: wrongVariant.captures[0].path,
      screen: wrongVariant.screens[0].screen,
    };
    await expectGenerationCode("CR2-SCHEMA", () =>
      validateCurrentRouteManifest(wrongVariant, {
        expectedVariants: REFERENCE_VARIANTS,
      }),
    );
  });

  test("provenance mutation is rejected", async () => {
    const baseline = await readFontManifest();

    const hashZero = structuredClone(baseline);
    hashZero.assets[0] = {
      ...hashZero.assets[0],
      sha256: "0".repeat(64),
    };
    await expectProvenanceCode("CRP-HASH", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: REPO_ROOT,
        provenance: hashZero,
        entries: CURRENT_ROUTE_SOURCE_ENTRIES,
      }),
    );

    const missing = structuredClone(baseline);
    missing.assets = missing.assets.slice(1);
    await expectProvenanceCode("CRP-MISSING", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: REPO_ROOT,
        provenance: missing,
        entries: CURRENT_ROUTE_SOURCE_ENTRIES,
      }),
    );

    const extra = structuredClone(baseline);
    extra.assets.push({
      path: "docs/mocks-ui/src/nonexistent.js",
      sha256: "0".repeat(64),
    });
    extra.assets.sort((left, right) =>
      left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
    );
    await expectProvenanceCode("CRP-EXTRA", () =>
      verifyCurrentRouteSourceClosure({
        repoRoot: REPO_ROOT,
        provenance: extra,
        entries: CURRENT_ROUTE_SOURCE_ENTRIES,
      }),
    );
  });

  test("no publication artifacts are emitted", async () => {
    await getBundle();
    assert.equal(
      existsSync(path.join(REPO_ROOT, "docs/mocks-ui/current-route-output")),
      false,
    );
    assert.equal(existsSync(path.join(REPO_ROOT, "docs/mocks-ui/CURRENT")), false,
    );
  });
});
