const ALLOWED_MEDIA_TYPES = new Set([
  "image/png",
  "image/svg+xml",
  "video/mp4",
  "audio/wav",
]);

const ROOT_KEYS = ["media"];
const ENTRY_KEYS = ["path", "mediaType"];

function fail(code, owner, detail) {
  throw new TypeError(`${code}: ${detail} at ${owner}`);
}

function hasPlainObject(value) {
  return (
    value !== null &&
    typeof value === "object" &&
    !Array.isArray(value)
  );
}

function assertKeySet(obj, expectedKeys, owner, code) {
  const keys = Object.keys(obj);
  if (keys.length !== expectedKeys.length) {
    fail(code, owner, "wrong key count");
  }
  for (const key of expectedKeys) {
    if (!Object.hasOwn(obj, key)) {
      fail(code, owner, `missing key ${key}`);
    }
  }
}

function validatePath(path, owner) {
  if (path.includes("\\")) {
    fail("SMP5", owner, "path contains backslash");
  }
  const segments = path.split("/");
  for (const segment of segments) {
    if (segment === "" || segment === "." || segment === "..") {
      fail("SMP5", owner, "invalid path segment");
    }
  }
}

function decodeStarterMediaProjection(input) {
  if (!hasPlainObject(input)) {
    fail("SMP1", "input", "expected object");
  }

  assertKeySet(input, ROOT_KEYS, "input", "SMP2");

  if (!Array.isArray(input.media)) {
    fail("SMP1", "input.media", "expected array");
  }

  if (input.media.length !== 4) {
    fail("SMP4", "input.media", "length must be 4");
  }

  const normalized = [];
  for (let i = 0; i < input.media.length; i += 1) {
    const entry = input.media[i];
    if (!hasPlainObject(entry)) {
      fail("SMP1", `input.media[${i}]`, "expected object");
    }

    assertKeySet(entry, ENTRY_KEYS, `input.media[${i}]`, "SMP2");

    if (typeof entry.path !== "string") {
      fail("SMP3", `input.media[${i}].path`, "path must be string");
    }

    if (typeof entry.mediaType !== "string") {
      fail("SMP3", `input.media[${i}].mediaType`, "mediaType must be string");
    }

    validatePath(entry.path, `input.media[${i}].path`);

    if (!ALLOWED_MEDIA_TYPES.has(entry.mediaType)) {
      fail("SMP7", `input.media[${i}].mediaType`, "unsupported mediaType");
    }

    const segments = entry.path.split("/");
    const name = segments[segments.length - 1];
    normalized.push({
      path: entry.path,
      mediaType: entry.mediaType,
      name,
    });
  }

  const seenPaths = new Set();
  const seenMediaTypes = new Set();
  for (let i = 0; i < normalized.length; i += 1) {
    const normalizedEntry = normalized[i];
    if (seenPaths.has(normalizedEntry.path)) {
      fail("SMP6", `input.media[${i}].path`, "duplicate path");
    }
    if (seenMediaTypes.has(normalizedEntry.mediaType)) {
      fail("SMP6", `input.media[${i}].mediaType`, "duplicate mediaType");
    }
    seenPaths.add(normalizedEntry.path);
    seenMediaTypes.add(normalizedEntry.mediaType);
  }

  return {
    media: normalized,
  };
}

export { decodeStarterMediaProjection };
