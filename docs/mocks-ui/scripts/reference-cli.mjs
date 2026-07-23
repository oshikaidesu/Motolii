import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import {
  publishReferenceGeneration,
  readCurrentReferenceGeneration,
} from "./reference-generation.mjs";
import {
  verifyFixtureCausality,
  verifyReferenceManifest,
} from "./reference-guard.mjs";
import {
  REFERENCE_BROWSER,
  REFERENCE_SCREENS,
  renderReferenceNormals,
} from "./reference-capture.mjs";
import {
  decodedPixels,
  deriveReferencePng,
  REFERENCE_TRANSFORM_VERSION,
  REFERENCE_VARIANTS,
} from "./reference-transform.mjs";

const execFileAsync = promisify(execFile);
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const PROVENANCE_PATH = path.join(ROOT, "reference-provenance.json");
const OUTPUT_ROOT = path.join(ROOT, "reference-output");
const BROWSER_VERSION = `Chromium/${REFERENCE_BROWSER.version}`;

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function loadProvenance() {
  const bytes = await readFile(PROVENANCE_PATH);
  const manifest = JSON.parse(bytes.toString("utf8"));
  await verifyReferenceManifest(ROOT, manifest);
  if (
    manifest.transformVersion !== REFERENCE_TRANSFORM_VERSION ||
    JSON.stringify(manifest.capture) !== JSON.stringify(REFERENCE_BROWSER)
  ) {
    throw new Error("reference provenance capture or transform contract is stale");
  }
  const declaredFiles = [manifest.transformationSource, ...(manifest.fontFiles ?? [])];
  for (const entry of declaredFiles) {
    if (
      typeof entry?.path !== "string" ||
      path.isAbsolute(entry.path) ||
      entry.path.split("/").includes("..") ||
      sha256(await readFile(path.join(ROOT, entry.path))) !== entry.sha256
    ) {
      throw new Error(`reference provenance file is stale: ${entry?.path}`);
    }
  }
  const packageLock = JSON.parse(await readFile(path.join(ROOT, "package-lock.json"), "utf8"));
  for (const tool of Object.values(manifest.toolchain ?? {})) {
    const locked = packageLock.packages?.[`node_modules/${tool.package}`];
    if (locked?.version !== tool.version || locked?.integrity !== tool.integrity) {
      throw new Error(`reference toolchain is stale: ${tool.package}`);
    }
  }
  await verifyFixtureCausality({ root: ROOT, manifest });
  return { manifest, sha256: sha256(bytes) };
}

function fixturePaths(manifest) {
  return Object.fromEntries(
    manifest.fixtureLayers.map((layer) => [layer.id, path.join(ROOT, layer.path)]),
  );
}

async function expectedBundle(provenance) {
  const normals = await renderReferenceNormals({ fixturePaths: fixturePaths(provenance.manifest) });
  const captures = new Map();
  for (const screen of REFERENCE_SCREENS) {
    const normal = normals.get(screen);
    for (const variant of REFERENCE_VARIANTS) {
      const capturePath = `captures/${screen}.${variant}.png`;
      captures.set(
        capturePath,
        variant === "normal" ? normal : deriveReferencePng(normal, variant),
      );
    }
  }
  const aggregate = sha256(
    Buffer.concat([...captures].map(([capturePath, bytes]) =>
      Buffer.concat([Buffer.from(`${capturePath}\0`), Buffer.from(bytes)]),
    )),
  );
  const generation = `u0e2-${provenance.sha256.slice(0, 12)}-${aggregate.slice(0, 12)}`;
  const manifest = {
    schemaVersion: 1,
    generation,
    browserVersion: BROWSER_VERSION,
    sourceManifestSha256: provenance.sha256,
    transformVersion: REFERENCE_TRANSFORM_VERSION,
    screens: [...REFERENCE_SCREENS],
    captures: [...captures].map(([capturePath, bytes]) => {
      const match = capturePath.match(/^captures\/(.+)\.([^.]+)\.png$/);
      return {
        path: capturePath,
        screen: match[1],
        variant: match[2],
        sha256: sha256(bytes),
      };
    }),
  };
  return { manifest, captures };
}

function validation(provenance) {
  return {
    browserVersion: BROWSER_VERSION,
    sourceManifestSha256: provenance.sha256,
    transformVersion: REFERENCE_TRANSFORM_VERSION,
    expectedScreens: REFERENCE_SCREENS,
    expectedVariants: REFERENCE_VARIANTS,
  };
}

async function repositoryFingerprint() {
  const { stdout: names } = await execFileAsync(
    "git",
    ["ls-files", "-co", "--exclude-standard", "-z"],
    { cwd: ROOT, encoding: "buffer", maxBuffer: 32 * 1024 * 1024 },
  );
  const files = names.toString("utf8").split("\0").filter(Boolean).sort();
  const digest = createHash("sha256");
  for (const filename of files) {
    digest.update(filename).update("\0").update(await readFile(path.join(ROOT, filename)));
  }
  const { stdout: status } = await execFileAsync(
    "git",
    ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    { cwd: ROOT, encoding: "buffer", maxBuffer: 32 * 1024 * 1024 },
  );
  return sha256(Buffer.concat([Buffer.from(digest.digest("hex")), status]));
}

async function compareExpected(current, expected) {
  if (JSON.stringify(current.manifest) !== JSON.stringify(expected.manifest)) {
    throw new Error("committed reference manifest differs from the current fixed inputs");
  }
  if (current.captures.size !== expected.captures.size) {
    throw new Error("committed reference capture set is not closed");
  }
  for (const [capturePath, expectedBytes] of expected.captures) {
    const actualBytes = current.captures.get(capturePath);
    if (!actualBytes) throw new Error(`missing committed capture ${capturePath}`);
    if (!decodedPixels(actualBytes).equals(decodedPixels(expectedBytes))) {
      throw new Error(`reference pixels differ: ${capturePath}`);
    }
  }
}

async function generate() {
  const provenance = await loadProvenance();
  const expected = await expectedBundle(provenance);
  let current = null;
  try {
    current = await readCurrentReferenceGeneration({
      root: OUTPUT_ROOT,
      validation: validation(provenance),
    });
  } catch (error) {
    if (
      error?.code !== "RG3-SOURCE" &&
      !(error?.code === "RG3-READ" && String(error.message).includes("cannot read CURRENT"))
    ) {
      throw error;
    }
  }
  if (current) {
    await compareExpected(current, expected);
    console.log(`reference generation already current: ${current.generation}`);
    return;
  }
  await publishReferenceGeneration({
    root: OUTPUT_ROOT,
    ...expected,
    validation: validation(provenance),
  });
  console.log(`reference generation published: ${expected.manifest.generation}`);
}

async function check() {
  const before = await repositoryFingerprint();
  const provenance = await loadProvenance();
  const expected = await expectedBundle(provenance);
  const current = await readCurrentReferenceGeneration({
    root: OUTPUT_ROOT,
    validation: validation(provenance),
  });
  await compareExpected(current, expected);
  const after = await repositoryFingerprint();
  if (before !== after) throw new Error("check-reference changed repository bytes or status");
  console.log(`reference generation OK: ${current.generation} (${current.captures.size} PNGs)`);
}

const command = process.argv[2];
if (command === "generate") await generate();
else if (command === "check") await check();
else throw new Error("usage: node scripts/reference-cli.mjs generate|check");
