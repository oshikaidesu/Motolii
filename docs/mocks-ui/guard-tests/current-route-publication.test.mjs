import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import {
  mkdtemp,
  readFile,
  readdir,
  rm,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";
import test from "node:test";
import assert from "node:assert/strict";

import { CurrentRouteGenerationError } from "../scripts/current-route-generation.mjs";
import {
  CURRENT_ROUTE_OUTPUT_ROOT,
  CURRENT_ROUTE_SCREENS,
} from "../scripts/current-route-generation.mjs";
import {
  publishCurrentRouteGeneration,
  readPublishedCurrentRouteGeneration,
  validateCurrentRoutePublication,
} from "../scripts/current-route-publication.mjs";
import { currentRouteGenerationAction } from "../scripts/current-route-cli.mjs";
import {
  REFERENCE_TRANSFORM_VERSION,
  REFERENCE_VARIANTS,
} from "../scripts/reference-transform.mjs";
import { repositoryFingerprint } from "../scripts/repository-fingerprint.mjs";
import {
  ReferenceGenerationError,
  publishReferenceGeneration,
  readCurrentReferenceGeneration,
} from "../scripts/reference-generation.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "../../..");
const PROVENANCE_PATH = path.join(
  REPO_ROOT,
  "docs/mocks-ui/current-route-provenance.json",
);
const REFERENCE_OUTPUT_ROOT = path.join(REPO_ROOT, "docs/mocks-ui/reference-output");
const CURRENT_OUTPUT_ROOT = path.join(
  REPO_ROOT,
  "docs/mocks-ui",
  CURRENT_ROUTE_OUTPUT_ROOT,
);
const execFileAsync = promisify(execFile);

const EXPECTED_SCREEN_COUNT = CURRENT_ROUTE_SCREENS.length;
const EXPECTED_CAPTURE_COUNT = EXPECTED_SCREEN_COUNT * REFERENCE_VARIANTS.length;

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function expectCode(code, run, messagePattern) {
  return assert.rejects(
    Promise.resolve().then(run),
    (error) => {
      assert.ok(error instanceof CurrentRouteGenerationError);
      assert.equal(error.code, code);
      if (messagePattern) assert.match(error.message, messagePattern);
      return true;
    },
  );
}

function png(value) {
  const image = new PNG({ width: 1, height: 1 });
  image.data.set([value, value, value, 255]);
  return PNG.sync.write(image);
}

function canonicalScreens() {
  return CURRENT_ROUTE_SCREENS.map((screen, index) => ({
    screen,
    mode: index === 0 ? "current-route-capture" : "development",
  }));
}

async function sourceManifestSha() {
  return sha256(await readFile(PROVENANCE_PATH));
}

function bundlePath(root, generation) {
  return path.join(root, "generations", generation);
}

function currentText(root) {
  return readFile(path.join(root, "CURRENT"), "utf8");
}

async function temporaryRoot(run) {
  const root = await mkdtemp(path.join(tmpdir(), "motolii-current-route-publication-"));
  try {
    await run(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

async function buildBundle(generation, sourceManifestSha) {
  const captures = new Map();
  const manifestCaptures = [];

  for (const screen of CURRENT_ROUTE_SCREENS) {
    for (const [index, variant] of REFERENCE_VARIANTS.entries()) {
      const value = createHash("sha256").update(`${generation}-${screen}-${variant}`).digest()[0];
      const capturePath = `captures/${screen}.${variant}.png`;
      const bytes = png(value);

      captures.set(capturePath, bytes);
      manifestCaptures.push({
        path: capturePath,
        screen,
        variant,
        sha256: sha256(bytes),
      });
    }
  }

  return {
    manifest: {
      schemaVersion: 2,
      generation,
      sourceManifestSha256: sourceManifestSha,
      transformVersion: REFERENCE_TRANSFORM_VERSION,
      environment: {
        viewport: { width: 1, height: 1 },
        scale: 1,
        locale: "locale-x",
        timezone: "timezone-x",
        theme: "theme-x",
        reducedMotion: "motion-x",
        browserVersion: "browser-x",
        browserRevision: "revision-x",
        fontFixture: {
          files: [
            {
              path: "fixtures/font-a.woff2",
              sha256: "a".repeat(64),
              weight: 400,
            },
            {
              path: "fixtures/font-b.woff2",
              sha256: "b".repeat(64),
              weight: 600,
            },
          ],
          computedFamily: "family-x",
        },
      },
      screens: canonicalScreens(),
      captures: manifestCaptures,
    },
    captures,
  };
}

function provenanceSuccessor(bundle, generation, sourceManifestSha256) {
  return {
    manifest: {
      ...structuredClone(bundle.manifest),
      generation,
      sourceManifestSha256,
    },
    captures: new Map(
      [...bundle.captures].map(([capturePath, bytes]) => [
        capturePath,
        Buffer.from(bytes),
      ]),
    ),
  };
}

function buildReferenceBundle(generation = "reference-parity") {
  const screen = "screen-a";
  const variants = ["normal", "variant-a"];
  const captures = new Map();
  const manifestCaptures = variants.map((variant, index) => {
    const capturePath = `captures/${screen}.${variant}.png`;
    const bytes = png(index + 20);
    captures.set(capturePath, bytes);
    return {
      path: capturePath,
      screen,
      variant,
      sha256: sha256(bytes),
    };
  });
  const validation = {
    browserVersion: "browser-v1-x",
    sourceManifestSha256: "c".repeat(64),
    transformVersion: "transform-x",
    expectedScreens: [screen],
    expectedVariants: variants,
  };
  return {
    manifest: {
      schemaVersion: 1,
      generation,
      browserVersion: validation.browserVersion,
      sourceManifestSha256: validation.sourceManifestSha256,
      transformVersion: validation.transformVersion,
      screens: [screen],
      captures: manifestCaptures,
    },
    captures,
    validation,
  };
}

test("shared publication preserves legacy RG3 failure semantics", async (suite) => {
  await suite.test("manifest schema wins before non-byte capture", async () => {
    await temporaryRoot(async (root) => {
      const bundle = buildReferenceBundle("reference-schema-first");
      const malformed = { ...bundle.manifest, schemaVersion: 2 };
      const invalidCaptures = new Map(bundle.captures);
      invalidCaptures.set([...invalidCaptures.keys()][0], "not-bytes");
      await assert.rejects(
        publishReferenceGeneration({
          root,
          manifest: malformed,
          captures: invalidCaptures,
          validation: bundle.validation,
        }),
        (error) => error instanceof ReferenceGenerationError && error.code === "RG3-SCHEMA",
      );
    });
  });

  await suite.test("invalid CURRENT remains RG3-SCHEMA", async () => {
    await temporaryRoot(async (root) => {
      const bundle = buildReferenceBundle("reference-current");
      await publishReferenceGeneration({ root, ...bundle });
      await writeFile(path.join(root, "CURRENT"), "../outside\n");
      await assert.rejects(
        readCurrentReferenceGeneration({ root, validation: bundle.validation }),
        (error) => error instanceof ReferenceGenerationError && error.code === "RG3-SCHEMA",
      );
    });
  });

  await suite.test("missing captures directory remains RG3-READ", async () => {
    await temporaryRoot(async (root) => {
      const bundle = buildReferenceBundle("reference-missing");
      await publishReferenceGeneration({ root, ...bundle });
      await rm(
        path.join(bundlePath(root, bundle.manifest.generation), "captures"),
        { recursive: true },
      );
      await assert.rejects(
        readCurrentReferenceGeneration({ root, validation: bundle.validation }),
        (error) => error instanceof ReferenceGenerationError && error.code === "RG3-READ",
      );
    });
  });

  await suite.test("typed inject error is rethrown by identity", async () => {
    await temporaryRoot(async (root) => {
      const bundle = buildReferenceBundle("reference-typed-inject");
      const injected = new ReferenceGenerationError("RG3-SOURCE", "typed injection");
      await assert.rejects(
        publishReferenceGeneration({
          root,
          ...bundle,
          inject(checkpoint) {
            if (checkpoint === "mkdir-stage") throw injected;
          },
        }),
        (error) => error === injected,
      );
    });
  });

  await suite.test("generic inject remains RG3-PUBLISH with legacy prefix", async () => {
    await temporaryRoot(async (root) => {
      const bundle = buildReferenceBundle("reference-generic-inject");
      await assert.rejects(
        publishReferenceGeneration({
          root,
          ...bundle,
          inject(checkpoint) {
            if (checkpoint === "mkdir-stage") throw new Error("generic injection");
          },
        }),
        (error) =>
          error instanceof ReferenceGenerationError &&
          error.code === "RG3-PUBLISH" &&
          /publication failed: generic injection/.test(error.message),
      );
    });
  });
});

test("round trips manifest and captures through immutable publication", async () => {
  await temporaryRoot(async (root) => {
    const manifestSource = await sourceManifestSha();
    const bundle = await buildBundle("generation-roundtrip", manifestSource);

    await publishCurrentRouteGeneration({
      root,
      manifest: bundle.manifest,
      captures: bundle.captures,
    });

    const current = await readPublishedCurrentRouteGeneration({ root });
    assert.equal(current.generation, bundle.manifest.generation);
    assert.deepEqual(current.manifest, bundle.manifest);
    assert.equal(current.captures.size, bundle.captures.size);
    for (const [capturePath, expectedBytes] of bundle.captures) {
      assert.deepEqual(current.captures.get(capturePath), expectedBytes);
    }

    assert.equal(
      await currentText(root),
      `${bundle.manifest.generation}\n`,
    );
  });
});

test("repository published generation validates v2 manifest shape and bytes", async () => {
    const manifestSource = await sourceManifestSha();
    const current = await readPublishedCurrentRouteGeneration({
      root: CURRENT_OUTPUT_ROOT,
    });

    assert.equal(current.manifest.schemaVersion, 2);
    assert.deepEqual(
      Object.keys(current.manifest),
      [
        "schemaVersion",
        "generation",
        "sourceManifestSha256",
        "transformVersion",
        "environment",
        "screens",
        "captures",
      ],
    );
    assert.deepEqual(current.manifest.screens, canonicalScreens());
    assert.equal(current.manifest.screens.length, EXPECTED_SCREEN_COUNT);
    assert.equal(current.manifest.captures.length, EXPECTED_CAPTURE_COUNT);
    assert.equal(current.manifest.sourceManifestSha256, manifestSource);
    assert.equal(current.manifest.transformVersion, REFERENCE_TRANSFORM_VERSION);

    for (const capture of current.manifest.captures) {
      assert.equal(sha256(current.captures.get(capture.path)), capture.sha256);
    }

    const environmentKeys = [
      "viewport",
      "scale",
      "locale",
      "timezone",
      "theme",
      "reducedMotion",
      "browserVersion",
      "browserRevision",
      "fontFixture",
    ];
    assert.deepEqual(Object.keys(current.manifest.environment).sort(), environmentKeys.sort());
});

test("CLI check path is immutable and does not change repository fingerprint", async () => {
    const before = await repositoryFingerprint(REPO_ROOT);
    const { stdout } = await execFileAsync(
      process.execPath,
      ["scripts/current-route-cli.mjs", "check"],
      {
        cwd: path.join(REPO_ROOT, "docs/mocks-ui"),
        maxBuffer: 1024 * 1024,
      },
    );
    const after = await repositoryFingerprint(REPO_ROOT);

    assert.match(
      stdout,
      /^current-route generation OK: [A-Za-z0-9._-]+ \(30 PNGs\)\n$/,
    );
    assert.equal(after, before);
});

test("generate decision allows only an exact provenance successor", async () => {
  const manifestSource = await sourceManifestSha();
  const current = await buildBundle("generation-action-old", manifestSource);
  const identical = provenanceSuccessor(
    current,
    current.manifest.generation,
    current.manifest.sourceManifestSha256,
  );
  const provenanceOnly = provenanceSuccessor(
    current,
    "generation-action-new",
    "d".repeat(64),
  );

  assert.equal(
    currentRouteGenerationAction(current, identical),
    "already-published",
  );
  assert.equal(
    currentRouteGenerationAction(current, provenanceOnly),
    "publish-provenance-successor",
  );

  const changedCapture = provenanceSuccessor(
    current,
    "generation-action-capture",
    "e".repeat(64),
  );
  const firstCapture = [...changedCapture.captures.keys()][0];
  const changedBytes = png(251);
  changedCapture.captures.set(firstCapture, changedBytes);
  changedCapture.manifest.captures[0].sha256 = sha256(changedBytes);
  assert.equal(
    currentRouteGenerationAction(current, changedCapture),
    "reject-capture-drift",
  );

  const changedEnvironment = provenanceSuccessor(
    current,
    "generation-action-environment",
    "f".repeat(64),
  );
  changedEnvironment.manifest.environment.locale = "locale-y";
  assert.equal(
    currentRouteGenerationAction(current, changedEnvironment),
    "reject-capture-drift",
  );
});

test("source relaxation still validates schema and capture bytes", async () => {
  const manifestSource = await sourceManifestSha();
  const current = await buildBundle("generation-source-relaxation", manifestSource);
  const stale = provenanceSuccessor(
    current,
    "generation-source-relaxation-stale",
    "0".repeat(64),
  );

  assert.throws(
    () => validateCurrentRoutePublication(stale.manifest, stale.captures),
    (error) =>
      error instanceof CurrentRouteGenerationError &&
      error.code === "CR2-SCHEMA",
  );
  assert.doesNotThrow(() =>
    validateCurrentRoutePublication(stale.manifest, stale.captures, {
      requireCurrentSource: false,
    }),
  );

  const firstCapture = [...stale.captures.keys()][0];
  const damagedCaptures = new Map(stale.captures);
  const damagedBytes = Buffer.from(damagedCaptures.get(firstCapture));
  damagedBytes[0] ^= 0xff;
  damagedCaptures.set(firstCapture, damagedBytes);
  assert.throws(
    () =>
      validateCurrentRoutePublication(stale.manifest, damagedCaptures, {
        requireCurrentSource: false,
      }),
    (error) =>
      error instanceof CurrentRouteGenerationError &&
      error.code === "CR2-CAPTURE",
  );

  assert.throws(
    () =>
      validateCurrentRoutePublication(
        { ...stale.manifest, schemaVersion: 1 },
        stale.captures,
        { requireCurrentSource: false },
      ),
    (error) =>
      error instanceof CurrentRouteGenerationError &&
      error.code === "CR2-SCHEMA",
  );
});

test("every publication checkpoint failure keeps only old or new complete generation", async (suite) => {
  const checkpoints = [];
  await temporaryRoot(async (root) => {
    const manifestSource = await sourceManifestSha();
    const probe = await buildBundle("generation-probe", manifestSource);
    await publishCurrentRouteGeneration({
      root,
      manifest: probe.manifest,
      captures: probe.captures,
      inject(checkpoint) {
        checkpoints.push(checkpoint);
      },
    });
  });
  assert.ok(checkpoints.length > 10);
  assert.equal(new Set(checkpoints).size, checkpoints.length);

  for (const checkpoint of checkpoints) {
    await suite.test(checkpoint, async () => {
      await temporaryRoot(async (root) => {
        const manifestSource = await sourceManifestSha();
        const oldBundle = await buildBundle("generation-old", manifestSource);
        const newBundle = await buildBundle("generation-new", manifestSource);

        await publishCurrentRouteGeneration({
          root,
          manifest: oldBundle.manifest,
          captures: oldBundle.captures,
        });

        let observedCheckpoint = false;
        await expectCode("CR2-PUBLISH", () =>
          publishCurrentRouteGeneration({
            root,
            manifest: newBundle.manifest,
            captures: newBundle.captures,
            inject(checkpointName) {
              if (checkpointName === checkpoint) {
                observedCheckpoint = true;
                throw new Error("injected failure");
              }
            },
          }),
        );
        assert.equal(observedCheckpoint, true);

        const current = await readPublishedCurrentRouteGeneration({ root });
        const expected =
          checkpoint === "sync-root-directory" ? newBundle : oldBundle;

        assert.equal(current.generation, expected.manifest.generation);
        assert.equal(current.captures.size, expected.captures.size);
        assert.equal(current.manifest.generation, expected.manifest.generation);
        for (const [capturePath, expectedBytes] of expected.captures) {
          assert.deepEqual(current.captures.get(capturePath), expectedBytes);
        }

        const entryText = await currentText(root);
        assert.equal(entryText.trim(), expected.manifest.generation);

        const rootEntries = await readdir(root);
        assert.equal(rootEntries.some((entry) => entry.startsWith(".CURRENT-")), false);
        const generationEntries = await readdir(path.join(root, "generations"));
        assert.equal(generationEntries.some((entry) => entry.startsWith(".stage-")), false);
      });
    });
  }
});

test("publishing a distinct successor advances CURRENT while republishing it is rejected", async () => {
  await temporaryRoot(async (root) => {
    const manifestSource = await sourceManifestSha();
    const first = await buildBundle("generation-diff-old", manifestSource);
    const second = await buildBundle("generation-diff-new", manifestSource);

    await publishCurrentRouteGeneration({
      root,
      manifest: first.manifest,
      captures: first.captures,
    });

    await publishCurrentRouteGeneration({
      root,
      manifest: second.manifest,
      captures: second.captures,
    });

    await expectCode("CR2-PUBLISH", () =>
      publishCurrentRouteGeneration({
        root,
        manifest: second.manifest,
        captures: second.captures,
      }),
    );

    const current = await readPublishedCurrentRouteGeneration({ root });
    assert.equal(current.generation, second.manifest.generation);
    assert.deepEqual(
      JSON.parse(await readFile(path.join(bundlePath(root, first.manifest.generation), "manifest.json"), "utf8")),
      first.manifest,
    );
  });
});

test("read rejects independent manifest/capture tampering and stray entries", async (suite) => {
  const cases = [
    {
      name: "flipped capture byte",
      code: "CR2-CAPTURE",
      message: /capture|PNG|SHA-256/i,
      async mutate({ generationDir, firstCapture, bundle }) {
        const flipped = Buffer.from(bundle.captures.get(firstCapture));
        flipped[0] ^= 0xff;
        await writeFile(path.join(generationDir, firstCapture), flipped);
      },
    },
    {
      name: "manifest capture digest",
      code: "CR2-CAPTURE",
      message: /capture|SHA-256/i,
      async mutate({ generationDir, bundle }) {
        const changed = JSON.parse(JSON.stringify(bundle.manifest));
        changed.captures[0].sha256 = "0".repeat(64);
        await writeFile(path.join(generationDir, "manifest.json"), `${JSON.stringify(changed)}\n`);
      },
    },
    {
      name: "missing capture",
      code: "CR2-CAPTURE",
      message: /capture/i,
      async mutate({ generationDir, firstCapture }) {
        await rm(path.join(generationDir, firstCapture));
      },
    },
    {
      name: "undeclared capture",
      code: "CR2-CAPTURE",
      message: /capture/i,
      async mutate({ generationDir }) {
        await writeFile(path.join(generationDir, "captures", "extra.png"), png(9));
      },
    },
    {
      name: "generation root entry",
      code: "CR2-READ",
      message: /undeclared root entry/i,
      async mutate({ generationDir }) {
        await writeFile(path.join(generationDir, "extra-entry"), "bad");
      },
    },
  ];

  for (const scenario of cases) {
    await suite.test(scenario.name, async () => temporaryRoot(async (root) => {
    const manifestSource = await sourceManifestSha();
    const bundle = await buildBundle("generation-tamper", manifestSource);
    await publishCurrentRouteGeneration({
      root,
      manifest: bundle.manifest,
      captures: bundle.captures,
    });

    const firstCapture = [...bundle.captures.keys()][0];
    const generationDir = bundlePath(root, bundle.manifest.generation);
      await scenario.mutate({ generationDir, firstCapture, bundle });
      await expectCode(
        scenario.code,
        () => readPublishedCurrentRouteGeneration({ root }),
        scenario.message,
      );
    }));
  }
});

test("rejects invalid CURRENT and source provenance mismatch", async () => {
  await temporaryRoot(async (root) => {
  const manifestSource = await sourceManifestSha();
  const bundle = await buildBundle("generation-mismatch", manifestSource);

    await publishCurrentRouteGeneration({
      root,
      manifest: bundle.manifest,
      captures: bundle.captures,
    });

    await rm(path.join(root, "CURRENT"));
    await expectCode("CR2-READ", () => readPublishedCurrentRouteGeneration({ root }));

    await writeFile(path.join(root, "CURRENT"), "\n");
    await expectCode("CR2-READ", () => readPublishedCurrentRouteGeneration({ root }));

    await writeFile(path.join(root, "CURRENT"), "../outside\n");
    await expectCode("CR2-READ", () => readPublishedCurrentRouteGeneration({ root }));

    await writeFile(path.join(root, "CURRENT"), "other-generation\n");
    await expectCode("CR2-READ", () => readPublishedCurrentRouteGeneration({ root }));

    const invalidSource = { ...bundle.manifest, sourceManifestSha256: "0".repeat(64) };
    const sourceRoot = await mkdtemp(path.join(tmpdir(), "motolii-current-route-publication-source-"));
    await expectCode("CR2-SCHEMA", () =>
      publishCurrentRouteGeneration({
        root: sourceRoot,
        manifest: invalidSource,
        captures: bundle.captures,
      }),
    ).finally(() => rm(sourceRoot, { recursive: true, force: true }));
  });
});

test("rejects v1-or-legacy schema and unsafe capture paths", async () => {
  await temporaryRoot(async (root) => {
    const manifestSource = await sourceManifestSha();

    const base = await buildBundle("generation-schema-v", manifestSource);
    const oldVersion = { ...base.manifest, schemaVersion: 1 };
    await expectCode("CR2-SCHEMA", () =>
      publishCurrentRouteGeneration({ root, manifest: oldVersion, captures: base.captures }),
    );

    const extraField = { ...base.manifest, browserVersion: "Chromium/extra" };
    await expectCode("CR2-SCHEMA", () =>
      publishCurrentRouteGeneration({ root, manifest: extraField, captures: base.captures }),
    );

    const absolute = await buildBundle("generation-absolute", manifestSource);
    const absolutePath = "/tmp/evil.png";
    const absoluteCaptures = new Map(absolute.captures);
    absoluteCaptures.set(absolutePath, png(7));
    absolute.manifest = { ...absolute.manifest, generation: "generation-absolute" };
    absolute.manifest.captures = [
      { ...absolute.manifest.captures[0], path: absolutePath },
      ...absolute.manifest.captures.slice(1),
    ];
    await expectCode("CR2-SCHEMA", () =>
      publishCurrentRouteGeneration({
        root,
        manifest: absolute.manifest,
        captures: absoluteCaptures,
      }),
    );

    const traversal = await buildBundle("generation-traversal", manifestSource);
    const traversalPath = "captures/../../evil.png";
    const traversalCaptures = new Map(traversal.captures);
    traversalCaptures.delete("captures/empty-browser.normal.png");
    traversalCaptures.set(traversalPath, png(8));
    traversal.manifest = { ...traversal.manifest, generation: "generation-traversal" };
    traversal.manifest.captures = [
      { ...traversal.manifest.captures[0], path: traversalPath },
      ...traversal.manifest.captures.slice(1),
    ];
    await expectCode("CR2-SCHEMA", () =>
      publishCurrentRouteGeneration({
        root,
        manifest: traversal.manifest,
        captures: traversalCaptures,
      }),
    );

    const windows = await buildBundle("generation-windows", manifestSource);
    const windowsPath = "captures\\evil.variant-a.png";
    const windowsCaptures = new Map(windows.captures);
    windowsCaptures.delete("captures/empty-browser.normal.png");
    windowsCaptures.set(windowsPath, png(9));
    windows.manifest = { ...windows.manifest, generation: "generation-windows" };
    windows.manifest.captures = [
      { ...windows.manifest.captures[0], path: windowsPath },
      ...windows.manifest.captures.slice(1),
    ];
    await expectCode("CR2-SCHEMA", () =>
      publishCurrentRouteGeneration({
        root,
        manifest: windows.manifest,
        captures: windowsCaptures,
      }),
    );
  });
});

test("old reference output hashes remain unchanged", async () => {
  const currentTextValue = (await readFile(path.join(REFERENCE_OUTPUT_ROOT, "CURRENT"), "utf8")).trim();
  const manifestBytes = await readFile(
    path.join(REFERENCE_OUTPUT_ROOT, "generations", currentTextValue, "manifest.json"),
  );
  const provenanceBytes = await readFile(
    path.join(REPO_ROOT, "docs/mocks-ui/reference-provenance.json"),
  );

  const currentBytes = await readFile(path.join(REFERENCE_OUTPUT_ROOT, "CURRENT"));

  const capturesDir = path.join(
    REFERENCE_OUTPUT_ROOT,
    "generations",
    currentTextValue,
    "captures",
  );
  const captureFiles = (await readdir(capturesDir)).sort();
    const digest = createHash("sha256");

    assert.equal(sha256(currentBytes), "a2eb48d4b93fa5eeae88d7c9de877e2791e49971c3b678da40e97d18a3e3021e");
    assert.equal(sha256(manifestBytes), "feabc97a8791d05a337d3f1365b09a2e1fb652b583163466dfec90320c5536f5");
    assert.equal(sha256(provenanceBytes), "08f96cbd77545e1734cc285970137ba20e1b9f31f3fac8f4e3704c467daa64a4");

    assert.equal(captureFiles.length, EXPECTED_CAPTURE_COUNT);
    for (const filename of captureFiles) {
      const fileDigest = sha256(await readFile(path.join(capturesDir, filename)));
      digest.update(`${fileDigest}  ${filename}\n`);
    }
  assert.equal(
    digest.digest("hex"),
    "131bfa2676918b5129fb7be8692265430cbccfcf481c0bfed0b4d348e5a74493",
  );
});
