import { randomUUID } from "node:crypto";
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

const GENERATION = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;

function safeGeneration(value, fail) {
  if (
    typeof value !== "string" ||
    value === "." ||
    value === ".." ||
    !GENERATION.test(value)
  ) {
    fail("schema", `invalid generation ${JSON.stringify(value)}`);
  }
  return value;
}

function safeCapturePath(value, fail) {
  if (typeof value !== "string" || value.includes("\\") || value.includes("\0")) {
    fail("schema", "capture path must be a forward-slash relative path");
  }
  const normalized = path.posix.normalize(value);
  if (
    path.posix.isAbsolute(normalized) ||
    normalized === "." ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    normalized.includes("/../") ||
    normalized !== value ||
    !normalized.startsWith("captures/") ||
    normalized === "captures/"
  ) {
    fail("schema", `unsafe capture path ${JSON.stringify(value)}`);
  }
  return normalized;
}

function ensureCaptureMap(captures, fail, nonByteLikeKind = "capture") {
  const entries = captures instanceof Map ? [...captures] : Object.entries(captures ?? {});
  const result = new Map();
  for (const [capturePath, value] of entries) {
    const safePath = safeCapturePath(capturePath, fail);
    if (result.has(safePath)) {
      fail("schema", `duplicate capture ${safePath}`);
    }
    if (!Buffer.isBuffer(value) && !(value instanceof Uint8Array)) {
      fail(nonByteLikeKind, `${safePath} is not byte-like`);
    }
    result.set(safePath, Buffer.from(value));
  }
  return result;
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

async function listCaptureFiles(directory, prefix, fail, filesystemKind) {
  let entries;
  try {
    entries = await readdir(directory, { withFileTypes: true });
  } catch (cause) {
    fail(filesystemKind, `cannot read captures: ${cause.message}`, cause);
  }
  const result = [];
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const relative = `${prefix}/${entry.name}`;
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const nested = await listCaptureFiles(filename, relative, fail, filesystemKind);
      result.push(...nested);
    } else if (entry.isFile()) {
      result.push(relative);
    } else {
      fail("capture", `non-file capture entry ${relative}`);
    }
  }
  return result;
}

function ensureValidatedPayload(payload, fail) {
  if (!payload || typeof payload !== "object" || Array.isArray(payload)) {
    fail("schema", "publication validation did not return a validation payload");
  }
  if (typeof payload.generation !== "string") {
    fail("schema", "publication validation did not return a generation");
  }
  if (!(payload.captures instanceof Map)) {
    fail("schema", "publication validation did not return capture bytes map");
  }
  return { generation: payload.generation, captures: payload.captures };
}

function manifestBytes(manifest) {
  return Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`);
}

export async function publishImmutableGeneration({
  root,
  manifest,
  captures,
  inject,
  validate,
  fail,
  policy = {},
}) {
  root = path.resolve(root);

  let validated;
  try {
    validated = await Promise.resolve(validate(manifest, captures));
  } catch (cause) {
    throw cause;
  }

  const payload = ensureValidatedPayload(validated, fail);
  const generation = safeGeneration(payload.generation, fail);
  const validatedCaptures = ensureCaptureMap(
    payload.captures,
    fail,
    policy.nonByteLikeKind ?? "capture",
  );

  const manifestText = manifestBytes(manifest);
  const generations = path.join(root, "generations");
  const nonce = randomUUID();
  const stage = path.join(generations, `.stage-${generation}-${nonce}`);
  const destination = path.join(generations, generation);
  const currentTemp = path.join(root, `.CURRENT-${nonce}`);
  let stageMoved = false;

  try {
    await perform(inject, "mkdir-generations", () => mkdir(generations, { recursive: true }));
    await perform(inject, "mkdir-stage", () => mkdir(stage));

    const directories = new Set([path.join(stage, "captures")]);
    for (const capturePath of validatedCaptures.keys()) {
      directories.add(path.dirname(path.join(stage, capturePath)));
    }

    for (const directory of [...directories].sort()) {
      await perform(
        inject,
        `mkdir-capture-directory:${path.relative(stage, directory)}`,
        () => mkdir(directory, { recursive: true }),
      );
    }

    for (const [capturePath, bytes] of [...validatedCaptures].sort(([left], [right]) =>
      left.localeCompare(right),
    )) {
      const filename = path.join(stage, capturePath);
      await perform(
        inject,
        `write-capture:${capturePath}`,
        () => writeFile(filename, bytes, { flag: "wx" }),
      );
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
    await perform(inject, "write-manifest", () =>
      writeFile(manifestPath, manifestText, { flag: "wx" })
    );
    await perform(inject, "sync-manifest", () => syncFile(manifestPath));
    await perform(inject, "sync-stage-directory", () => syncFile(stage));
    await perform(inject, "rename-stage", () => rename(stage, destination));
    stageMoved = true;

    await perform(inject, "sync-generations-directory", () => syncFile(generations));
    await perform(
      inject,
      "write-current-temp",
      () => writeFile(currentTemp, `${generation}\n`, { flag: "wx" }),
    );
    await perform(inject, "sync-current-temp", () => syncFile(currentTemp));
    await perform(inject, "rename-current", () => rename(currentTemp, path.join(root, "CURRENT")));
    await perform(inject, "sync-root-directory", () => syncFile(root));
  } catch (cause) {
    await rm(currentTemp, { force: true }).catch(() => {});
    if (!stageMoved) {
      await rm(stage, { recursive: true, force: true }).catch(() => {});
    }
    if (policy.rethrowDomainError?.(cause)) throw cause;
    fail("publish", `publication failed: ${String(cause?.message ?? cause)}`, cause);
  }
}

export async function readImmutableGeneration({ root, validate, fail, policy = {} }) {
  root = path.resolve(root);

  let generation;
  try {
    generation = (await readFile(path.join(root, "CURRENT"), "utf8")).trim();
  } catch (cause) {
    fail("read", `cannot read CURRENT: ${cause.message}`, cause);
  }

  safeGeneration(generation, (kind, message, cause) =>
    fail(policy.invalidGenerationKind ?? "read", message, cause)
  );

  const directory = path.join(root, "generations", generation);
  let manifest;
  try {
    manifest = JSON.parse(await readFile(path.join(directory, "manifest.json"), "utf8"));
  } catch (cause) {
    fail("read", `cannot read manifest for ${generation}: ${cause.message}`, cause);
  }

  if (manifest?.generation !== generation) {
    fail("read", "CURRENT and manifest name different generations");
  }

  let generationEntries;
  try {
    generationEntries = await readdir(directory, { withFileTypes: true });
  } catch (cause) {
    fail("read", `cannot read generation directory: ${cause.message}`, cause);
  }

  const entryShape = generationEntries
    .map((entry) => `${entry.name}:${entry.isDirectory() ? "directory" : entry.isFile() ? "file" : "other"}`)
    .sort();

  if (entryShape.join("\0") !== ["captures:directory", "manifest.json:file"].join("\0")) {
    fail("read", `generation ${generation} contains an undeclared root entry`);
  }

  const captures = new Map();
  const captureFilesystemKind = policy.captureFilesystemKind ?? "capture";
  for (const capturePath of await listCaptureFiles(
    path.join(directory, "captures"),
    "captures",
    fail,
    captureFilesystemKind,
  )) {
    try {
      captures.set(
        capturePath,
        await readFile(path.join(directory, capturePath)),
      );
    } catch (cause) {
      fail(
        captureFilesystemKind,
        `cannot read captures for ${generation}: ${cause.message}`,
        cause,
      );
    }
  }

  const normalizedCaptures = ensureCaptureMap(captures, (kind, message, cause) =>
    fail(kind, message, cause),
    policy.nonByteLikeKind ?? "capture",
  );

  let validated;
  try {
    validated = await Promise.resolve(validate(manifest, normalizedCaptures));
  } catch (cause) {
    throw cause;
  }

  const payload = ensureValidatedPayload(validated, fail);
  const validatedCaptures = ensureCaptureMap(
    payload.captures,
    (kind, message, cause) => fail("schema", message, cause),
  );

  if (validated.generation !== generation) {
    fail("read", "CURRENT and manifest name different generations");
  }
  if (validatedCaptures.size !== normalizedCaptures.size) {
    fail("capture", "capture set does not match manifest");
  }

  return {
    generation,
    manifest,
    captures: validatedCaptures,
  };
}
