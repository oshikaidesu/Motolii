import { createHash, randomUUID } from "node:crypto";
import {
  mkdir,
  open,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import path from "node:path";
import { PNG } from "pngjs";

const MANIFEST_FIELDS = new Set([
  "schemaVersion",
  "generation",
  "browserVersion",
  "sourceManifestSha256",
  "screens",
  "captures",
]);
const CAPTURE_FIELDS = new Set([
  "path",
  "screen",
  "variant",
  "sha256",
]);
const SHA256 = /^[0-9a-f]{64}$/;
const GENERATION = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;

export class ReferenceGenerationError extends Error {
  constructor(code, message, options) {
    super(`${code}: ${message}`, options);
    this.name = "ReferenceGenerationError";
    this.code = code;
  }
}

function reject(code, message, cause) {
  throw new ReferenceGenerationError(code, message, cause ? { cause } : undefined);
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function pixelSha256(bytes, capturePath) {
  let png;
  try {
    png = PNG.sync.read(bytes);
  } catch (cause) {
    reject("RG3-CAPTURE", `${capturePath} is not a valid PNG`, cause);
  }
  return sha256(
    Buffer.concat([
      Buffer.from(`${png.width}x${png.height}\0`),
      Buffer.from(png.data),
    ]),
  );
}

function exactFields(value, fields, owner) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    reject("RG3-SCHEMA", `${owner} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...fields].sort();
  if (actual.join("\0") !== expected.join("\0")) {
    reject("RG3-SCHEMA", `${owner} fields are outside the closed schema`);
  }
}

function safeGeneration(value) {
  if (typeof value !== "string" || !GENERATION.test(value) || value === "." || value === "..") {
    reject("RG3-SCHEMA", `invalid generation ${JSON.stringify(value)}`);
  }
  return value;
}

function safeCapturePath(value) {
  if (typeof value !== "string" || value.includes("\\")) {
    reject("RG3-SCHEMA", "capture path must use forward slashes");
  }
  const normalized = path.posix.normalize(value);
  if (
    normalized !== value ||
    !normalized.startsWith("captures/") ||
    normalized === "captures/" ||
    normalized.includes("/../") ||
    path.posix.isAbsolute(normalized)
  ) {
    reject("RG3-SCHEMA", `unsafe capture path ${JSON.stringify(value)}`);
  }
  return normalized;
}

function byteMap(captures) {
  const entries = captures instanceof Map ? [...captures] : Object.entries(captures ?? {});
  const result = new Map();
  for (const [capturePath, value] of entries) {
    const safePath = safeCapturePath(capturePath);
    if (result.has(safePath)) reject("RG3-SCHEMA", `duplicate capture ${safePath}`);
    if (!Buffer.isBuffer(value) && !(value instanceof Uint8Array)) {
      reject("RG3-SCHEMA", `${safePath} is not byte-like`);
    }
    result.set(safePath, Buffer.from(value));
  }
  return result;
}

function expectedSet(values, owner) {
  if (!Array.isArray(values) || values.length === 0) {
    reject("RG3-SCHEMA", `${owner} must be a non-empty array`);
  }
  const result = new Set(values);
  if (result.size !== values.length || values.some((value) => typeof value !== "string" || value.length === 0)) {
    reject("RG3-SCHEMA", `${owner} must contain unique non-empty strings`);
  }
  return result;
}

export function validateReferenceGeneration(
  manifest,
  captures,
  {
    browserVersion,
    sourceManifestSha256,
    expectedScreens,
    expectedVariants,
  } = {},
) {
  requireClosedValidation({
    browserVersion,
    sourceManifestSha256,
    expectedScreens,
    expectedVariants,
  });
  exactFields(manifest, MANIFEST_FIELDS, "manifest");
  if (manifest.schemaVersion !== 1) reject("RG3-SCHEMA", "schemaVersion must be 1");
  safeGeneration(manifest.generation);
  if (typeof manifest.browserVersion !== "string" || manifest.browserVersion.length === 0) {
    reject("RG3-SCHEMA", "browserVersion is required");
  }
  if (!SHA256.test(manifest.sourceManifestSha256 ?? "")) {
    reject("RG3-SCHEMA", "sourceManifestSha256 must be a SHA-256");
  }
  if (browserVersion !== undefined && manifest.browserVersion !== browserVersion) {
    reject("RG3-BROWSER", `browser ${browserVersion} does not match ${manifest.browserVersion}`);
  }
  if (
    sourceManifestSha256 !== undefined &&
    manifest.sourceManifestSha256 !== sourceManifestSha256
  ) {
    reject("RG3-SOURCE", "source manifest SHA-256 does not match the generation");
  }

  const screens = expectedSet(manifest.screens, "screens");
  if (expectedScreens) {
    const expected = expectedSet(expectedScreens, "expectedScreens");
    if (manifest.screens.join("\0") !== expectedScreens.join("\0")) {
      reject("RG3-SCREEN", "generation screens do not match the expected closed set");
    }
  }
  const variants = expectedVariants
    ? expectedSet(expectedVariants, "expectedVariants")
    : null;
  if (!Array.isArray(manifest.captures) || manifest.captures.length === 0) {
    reject("RG3-SCHEMA", "captures must be a non-empty array");
  }

  const bytesByPath = byteMap(captures);
  const declaredPaths = new Set();
  const variantsByScreen = new Map([...screens].map((screen) => [screen, new Map()]));
  for (const [index, capture] of manifest.captures.entries()) {
    exactFields(capture, CAPTURE_FIELDS, `captures[${index}]`);
    const capturePath = safeCapturePath(capture.path);
    if (declaredPaths.has(capturePath)) reject("RG3-SCHEMA", `duplicate capture ${capturePath}`);
    declaredPaths.add(capturePath);
    if (!screens.has(capture.screen)) reject("RG3-SCREEN", `unknown screen ${capture.screen}`);
    if (typeof capture.variant !== "string" || capture.variant.length === 0) {
      reject("RG3-SCHEMA", `${capturePath} has an invalid variant`);
    }
    if (variants && !variants.has(capture.variant)) {
      reject("RG3-VARIANT", `unknown variant ${capture.variant}`);
    }
    if (!SHA256.test(capture.sha256 ?? "")) {
      reject("RG3-SCHEMA", `${capturePath} has an invalid SHA-256`);
    }
    const bytes = bytesByPath.get(capturePath);
    if (!bytes || sha256(bytes) !== capture.sha256) {
      reject("RG3-CAPTURE", `${capturePath} bytes do not match the manifest`);
    }
    const screenVariants = variantsByScreen.get(capture.screen);
    if (screenVariants.has(capture.variant)) {
      reject("RG3-SCHEMA", `duplicate ${capture.screen}/${capture.variant}`);
    }
    screenVariants.set(capture.variant, {
      byteSha256: capture.sha256,
      pixelSha256: pixelSha256(bytes, capturePath),
    });
  }
  if (declaredPaths.size !== bytesByPath.size) {
    reject("RG3-CAPTURE", "capture directory contains undeclared bytes");
  }

  if (expectedScreens && expectedVariants) {
    const expectedOrder = expectedScreens.flatMap((screen) =>
      expectedVariants.map((variant) => `${screen}\0${variant}`),
    );
    const actualOrder = manifest.captures.map(
      (capture) => `${capture.screen}\0${capture.variant}`,
    );
    if (actualOrder.join("\n") !== expectedOrder.join("\n")) {
      reject("RG3-VARIANT", "capture screen/variant order is not canonical");
    }
  }

  const normalPixels = new Map();
  for (const [screen, screenVariants] of variantsByScreen) {
    const normal = screenVariants.get("normal");
    if (!normal) reject("RG3-VARIANT", `${screen} has no normal capture`);
    if (variants && [...variants].some((variant) => !screenVariants.has(variant))) {
      reject("RG3-VARIANT", `${screen} does not cover every expected variant`);
    }
    for (const [variant, digest] of screenVariants) {
      if (variant !== "normal" && digest.pixelSha256 === normal.pixelSha256) {
        reject("RG3-DERIVED", `${screen}/${variant} has pixels identical to normal`);
      }
    }
    if (normalPixels.has(normal.pixelSha256)) {
      reject(
        "RG3-SCREEN",
        `${screen} normal pixels duplicate ${normalPixels.get(normal.pixelSha256)}`,
      );
    }
    normalPixels.set(normal.pixelSha256, screen);
  }
  return { generation: manifest.generation, captures: bytesByPath };
}

async function syncFile(filename) {
  const handle = await open(filename, "r");
  try {
    await handle.sync();
  } finally {
    await handle.close();
  }
}

async function perform(inject, checkpoint, operation) {
  await inject?.(checkpoint);
  return operation();
}

function requireClosedValidation(validation) {
  if (
    !validation ||
    typeof validation.browserVersion !== "string" ||
    !SHA256.test(validation.sourceManifestSha256 ?? "") ||
    !Array.isArray(validation.expectedScreens) ||
    !Array.isArray(validation.expectedVariants)
  ) {
    reject(
      "RG3-SCHEMA",
      "publication requires browser, source SHA, screen, and variant expectations",
    );
  }
}

export async function publishReferenceGeneration({
  root,
  manifest,
  captures,
  inject,
  validation,
}) {
  root = path.resolve(root);
  requireClosedValidation(validation);
  const validated = validateReferenceGeneration(manifest, captures, validation);
  const manifestBytes = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`);
  const generations = path.join(root, "generations");
  const nonce = randomUUID();
  const stage = path.join(generations, `.stage-${manifest.generation}-${nonce}`);
  const destination = path.join(generations, manifest.generation);
  const currentTemp = path.join(root, `.CURRENT-${nonce}`);
  let stageMoved = false;
  try {
    await perform(inject, "mkdir-generations", () => mkdir(generations, { recursive: true }));
    await perform(inject, "mkdir-stage", () => mkdir(stage));
    const directories = new Set([path.join(stage, "captures")]);
    for (const capturePath of validated.captures.keys()) {
      directories.add(path.dirname(path.join(stage, capturePath)));
    }
    for (const directory of [...directories].sort()) {
      await perform(inject, `mkdir-capture-directory:${path.relative(stage, directory)}`, () =>
        mkdir(directory, { recursive: true }),
      );
    }
    for (const [capturePath, bytes] of [...validated.captures].sort(([left], [right]) => left.localeCompare(right))) {
      const filename = path.join(stage, capturePath);
      await perform(inject, `write-capture:${capturePath}`, () => writeFile(filename, bytes, { flag: "wx" }));
      await perform(inject, `sync-capture:${capturePath}`, () => syncFile(filename));
    }
    for (const directory of [...directories].sort().reverse()) {
      await perform(
        inject,
        `sync-capture-directory:${path.relative(stage, directory)}`,
        () => syncFile(directory),
      );
    }
    const manifestPath = path.join(stage, "manifest.json");
    await perform(inject, "write-manifest", () => writeFile(manifestPath, manifestBytes, { flag: "wx" }));
    await perform(inject, "sync-manifest", () => syncFile(manifestPath));
    await perform(inject, "sync-stage-directory", () => syncFile(stage));
    await perform(inject, "rename-stage", () => rename(stage, destination));
    stageMoved = true;
    await perform(inject, "sync-generations-directory", () => syncFile(generations));
    await perform(inject, "write-current-temp", () =>
      writeFile(currentTemp, `${manifest.generation}\n`, { flag: "wx" }),
    );
    await perform(inject, "sync-current-temp", () => syncFile(currentTemp));
    await perform(inject, "rename-current", () => rename(currentTemp, path.join(root, "CURRENT")));
    await perform(inject, "sync-root-directory", () => syncFile(root));
  } catch (cause) {
    await rm(currentTemp, { force: true }).catch(() => {});
    if (!stageMoved) await rm(stage, { recursive: true, force: true }).catch(() => {});
    if (cause instanceof ReferenceGenerationError) throw cause;
    reject("RG3-PUBLISH", `publication failed: ${cause.message}`, cause);
  }
}

async function listCaptureFiles(directory, prefix = "captures") {
  const result = [];
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const relative = `${prefix}/${entry.name}`;
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...(await listCaptureFiles(filename, relative)));
    else if (entry.isFile()) result.push(relative);
    else reject("RG3-CAPTURE", `non-file capture entry ${relative}`);
  }
  return result;
}

export async function readCurrentReferenceGeneration({ root, validation } = {}) {
  root = path.resolve(root);
  requireClosedValidation(validation);
  let generation;
  try {
    generation = (await readFile(path.join(root, "CURRENT"), "utf8")).trim();
  } catch (cause) {
    reject("RG3-READ", `cannot read CURRENT: ${cause.message}`, cause);
  }
  safeGeneration(generation);
  const directory = path.join(root, "generations", generation);
  let manifest;
  try {
    manifest = JSON.parse(await readFile(path.join(directory, "manifest.json"), "utf8"));
  } catch (cause) {
    reject("RG3-READ", `cannot read manifest for ${generation}: ${cause.message}`, cause);
  }
  if (manifest.generation !== generation) {
    reject("RG3-READ", "CURRENT and manifest name different generations");
  }
  const generationEntries = await readdir(directory, { withFileTypes: true });
  const entryShape = generationEntries
    .map((entry) => `${entry.name}:${entry.isDirectory() ? "directory" : entry.isFile() ? "file" : "other"}`)
    .sort();
  if (entryShape.join("\0") !== ["captures:directory", "manifest.json:file"].join("\0")) {
    reject("RG3-READ", `generation ${generation} contains an undeclared root entry`);
  }
  const captures = new Map();
  try {
    for (const capturePath of await listCaptureFiles(path.join(directory, "captures"))) {
      captures.set(capturePath, await readFile(path.join(directory, capturePath)));
    }
  } catch (cause) {
    if (cause instanceof ReferenceGenerationError) throw cause;
    reject("RG3-READ", `cannot read captures for ${generation}: ${cause.message}`, cause);
  }
  validateReferenceGeneration(manifest, captures, validation);
  return { generation, manifest, captures };
}
