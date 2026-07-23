import { createHash } from "node:crypto";
import { mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import assert from "node:assert/strict";
import { PNG } from "pngjs";
import {
  ReferenceGenerationError,
  publishReferenceGeneration,
  readCurrentReferenceGeneration,
  validateReferenceGeneration,
} from "../scripts/reference-generation.mjs";

const BROWSER = "Chromium/149.0.7827.55";
const SOURCE_SHA = "a".repeat(64);
const TRANSFORM_VERSION = "reference-transform-test-v1";
const SCREENS = ["screen-a"];
const VARIANTS = ["normal", "grayscale"];
const VALIDATION = {
  browserVersion: BROWSER,
  sourceManifestSha256: SOURCE_SHA,
  transformVersion: TRANSFORM_VERSION,
  expectedScreens: SCREENS,
  expectedVariants: VARIANTS,
};

function digest(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function png(red, green, blue) {
  const image = new PNG({ width: 1, height: 1 });
  image.data.set([red, green, blue, 255]);
  return PNG.sync.write(image);
}

function bundle(generation, marker) {
  const base = createHash("sha256").update(marker).digest()[0];
  const captures = new Map([
    ["captures/screen-a.normal.png", png(base, 20, 30)],
    ["captures/screen-a.grayscale.png", png(base, base, base)],
  ]);
  const manifest = {
    schemaVersion: 1,
    generation,
    browserVersion: BROWSER,
    sourceManifestSha256: SOURCE_SHA,
    transformVersion: TRANSFORM_VERSION,
    screens: [...SCREENS],
    captures: [
      {
        path: "captures/screen-a.normal.png",
        screen: "screen-a",
        variant: "normal",
        sha256: digest(captures.get("captures/screen-a.normal.png")),
      },
      {
        path: "captures/screen-a.grayscale.png",
        screen: "screen-a",
        variant: "grayscale",
        sha256: digest(captures.get("captures/screen-a.grayscale.png")),
      },
    ],
  };
  return { manifest, captures };
}

async function temporaryRoot(run) {
  const root = await mkdtemp(path.join(tmpdir(), "motolii-reference-generation-"));
  try {
    await run(root);
  } finally {
    await rm(root, { recursive: true, force: true });
  }
}

function expectCode(code, run) {
  assert.throws(run, (error) => {
    assert.ok(error instanceof ReferenceGenerationError);
    assert.equal(error.code, code);
    return true;
  });
}

test("validates browser, source, schema, screen, variant, and derived independence", () => {
  const current = bundle("generation-1", "one");
  validateReferenceGeneration(current.manifest, current.captures, VALIDATION);

  expectCode("RG3-BROWSER", () =>
    validateReferenceGeneration(current.manifest, current.captures, {
      ...VALIDATION,
      browserVersion: "Chromium/other",
    }),
  );
  expectCode("RG3-SOURCE", () =>
    validateReferenceGeneration(current.manifest, current.captures, {
      ...VALIDATION,
      sourceManifestSha256: "b".repeat(64),
    }),
  );

  const extraField = structuredClone(current.manifest);
  extraField.unowned = true;
  expectCode("RG3-SCHEMA", () =>
    validateReferenceGeneration(extraField, current.captures, VALIDATION),
  );

  const unknownScreen = structuredClone(current.manifest);
  unknownScreen.captures[0].screen = "screen-unknown";
  expectCode("RG3-SCREEN", () =>
    validateReferenceGeneration(unknownScreen, current.captures, VALIDATION),
  );

  const unknownVariant = structuredClone(current.manifest);
  unknownVariant.captures[1].variant = "invented";
  expectCode("RG3-VARIANT", () =>
    validateReferenceGeneration(unknownVariant, current.captures, VALIDATION),
  );

  const identical = bundle("generation-2", "two");
  const normal = identical.captures.get("captures/screen-a.normal.png");
  identical.captures.set("captures/screen-a.grayscale.png", normal);
  identical.manifest.captures[1].sha256 = digest(normal);
  expectCode("RG3-DERIVED", () =>
    validateReferenceGeneration(identical.manifest, identical.captures, VALIDATION),
  );
});

test("rejects order, undeclared bytes, byte mismatch, invalid PNG, and duplicate normal pixels", () => {
  const current = bundle("generation-negative", "negative");

  const reordered = structuredClone(current.manifest);
  reordered.captures.reverse();
  expectCode("RG3-VARIANT", () =>
    validateReferenceGeneration(reordered, current.captures, VALIDATION),
  );

  const undeclared = new Map(current.captures);
  undeclared.set("captures/extra.png", png(1, 2, 3));
  expectCode("RG3-CAPTURE", () =>
    validateReferenceGeneration(current.manifest, undeclared, VALIDATION),
  );

  const mismatched = new Map(current.captures);
  mismatched.set("captures/screen-a.normal.png", png(4, 5, 6));
  expectCode("RG3-CAPTURE", () =>
    validateReferenceGeneration(current.manifest, mismatched, VALIDATION),
  );

  const invalidPng = new Map(current.captures);
  const broken = Buffer.from("not-png");
  invalidPng.set("captures/screen-a.normal.png", broken);
  const invalidManifest = structuredClone(current.manifest);
  invalidManifest.captures[0].sha256 = digest(broken);
  expectCode("RG3-CAPTURE", () =>
    validateReferenceGeneration(invalidManifest, invalidPng, VALIDATION),
  );

  const duplicateNormal = new Map(current.captures);
  duplicateNormal.set(
    "captures/screen-b.normal.png",
    current.captures.get("captures/screen-a.normal.png"),
  );
  duplicateNormal.set("captures/screen-b.grayscale.png", png(7, 7, 7));
  const duplicateManifest = structuredClone(current.manifest);
  duplicateManifest.screens.push("screen-b");
  duplicateManifest.captures.push(
    {
      path: "captures/screen-b.normal.png",
      screen: "screen-b",
      variant: "normal",
      sha256: digest(duplicateNormal.get("captures/screen-b.normal.png")),
    },
    {
      path: "captures/screen-b.grayscale.png",
      screen: "screen-b",
      variant: "grayscale",
      sha256: digest(duplicateNormal.get("captures/screen-b.grayscale.png")),
    },
  );
  expectCode("RG3-SCREEN", () =>
    validateReferenceGeneration(duplicateManifest, duplicateNormal, {
      ...VALIDATION,
      expectedScreens: ["screen-a", "screen-b"],
    }),
  );
});

test("publishes manifest and captures through one immutable generation pointer", async () => {
  await temporaryRoot(async (root) => {
    const first = bundle("generation-1", "one");
    await publishReferenceGeneration({ root, ...first, validation: VALIDATION });
    const read = await readCurrentReferenceGeneration({ root, validation: VALIDATION });
    assert.equal(read.generation, "generation-1");
    assert.deepEqual(
      read.captures.get("captures/screen-a.normal.png"),
      first.captures.get("captures/screen-a.normal.png"),
    );
  });
});

test("publisher refuses an open-ended validation contract", async () => {
  await temporaryRoot(async (root) => {
    const current = bundle("generation-open", "open");
    await assert.rejects(
      publishReferenceGeneration({ root, ...current }),
      (error) => error instanceof ReferenceGenerationError && error.code === "RG3-SCHEMA",
    );
  });
});

test("reader rejects generation-root extras and a generation name cannot be overwritten", async () => {
  await temporaryRoot(async (root) => {
    const current = bundle("generation-immutable", "immutable");
    await publishReferenceGeneration({ root, ...current, validation: VALIDATION });
    await assert.rejects(
      publishReferenceGeneration({ root, ...current, validation: VALIDATION }),
      (error) => error instanceof ReferenceGenerationError && error.code === "RG3-PUBLISH",
    );
    await writeFile(
      path.join(root, "generations", current.manifest.generation, "extra"),
      "undeclared",
    );
    await assert.rejects(
      readCurrentReferenceGeneration({ root, validation: VALIDATION }),
      (error) => error instanceof ReferenceGenerationError && error.code === "RG3-READ",
    );
  });
});

test("every publication I/O failure leaves exactly the old or new complete generation", async (suite) => {
  const checkpoints = [];
  await temporaryRoot(async (root) => {
    const probe = bundle("generation-probe", "probe");
    await publishReferenceGeneration({
      root,
      ...probe,
      validation: VALIDATION,
      inject(checkpoint) {
        checkpoints.push(checkpoint);
      },
    });
  });
  assert.ok(checkpoints.length > 10);
  assert.equal(new Set(checkpoints).size, checkpoints.length);

  for (const [index, failedCheckpoint] of checkpoints.entries()) {
    await suite.test(failedCheckpoint, async () => {
      await temporaryRoot(async (root) => {
        const oldBundle = bundle(`generation-old-${index}`, "old");
        const newBundle = bundle(`generation-new-${index}`, "new");
        await publishReferenceGeneration({
          root,
          ...oldBundle,
          validation: VALIDATION,
        });
        await assert.rejects(
          publishReferenceGeneration({
            root,
            ...newBundle,
            validation: VALIDATION,
            inject(checkpoint) {
              if (checkpoint === failedCheckpoint) throw new Error("injected I/O failure");
            },
          }),
          (error) => error instanceof ReferenceGenerationError && error.code === "RG3-PUBLISH",
        );
        const read = await readCurrentReferenceGeneration({ root, validation: VALIDATION });
        const expected =
          failedCheckpoint === "sync-root-directory" ? newBundle : oldBundle;
        assert.equal(read.generation, expected.manifest.generation);
        assert.deepEqual(read.manifest, expected.manifest);
        assert.equal(read.captures.size, expected.captures.size);
        for (const [capturePath, bytes] of expected.captures) {
          assert.deepEqual(read.captures.get(capturePath), bytes);
        }
        assert.equal(read.manifest.generation, read.generation);
      });
    });
  }
});
