import path from "node:path";

export const CURRENT_ROUTE_OUTPUT_ROOT = "current-route-output";
export const CURRENT_ROUTE_PROVENANCE_PATH = "current-route-provenance.json";
export const CURRENT_ROUTE_COMMANDS = Object.freeze({
  generate: "generate-current-route",
  check: "check-current-route",
});
export const CURRENT_ROUTE_MANIFEST_SCHEMA_VERSION = 2;
export const CURRENT_ROUTE_SCREENS = Object.freeze([
  "empty-browser",
  "mixed-timeline",
  "parameter-easing",
  "stage-frame-tools",
  "shared-effect-relative",
]);

const SHA256 = /^[0-9a-f]{64}$/;
const GENERATION = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;
const CAPTURE_FIELDS = new Set(["path", "screen", "variant", "sha256"]);

export class CurrentRouteGenerationError extends Error {
  constructor(code, message, options) {
    super(`${code}: ${message}`, options);
    this.name = "CurrentRouteGenerationError";
    this.code = code;
  }
}

function reject(code, message, cause) {
  throw new CurrentRouteGenerationError(code, message, cause ? { cause } : undefined);
}

function exactFields(value, fields, owner) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    reject("CR2-SCHEMA", `${owner} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...fields].sort();
  if (actual.join("\0") !== expected.join("\0")) {
    reject("CR2-SCHEMA", `${owner} fields are outside the closed schema`);
  }
}

function nonEmptyString(value, owner) {
  if (typeof value !== "string" || value.length === 0) {
    reject("CR2-SCHEMA", `${owner} must be a non-empty string`);
  }
}

function validateGeneration(value) {
  if (
    typeof value !== "string" ||
    value === "." ||
    value === ".." ||
    !GENERATION.test(value)
  ) {
    reject("CR2-GENERATION", `invalid generation ${JSON.stringify(value)}`);
  }
}

function safeRelativePosix(value, owner) {
  if (typeof value !== "string" || value.length === 0) {
    reject("CR2-SCHEMA", `${owner} must be a non-empty relative path`);
  }
  if (value.includes("\\") || value.includes("\0")) {
    reject("CR2-SCHEMA", `invalid ${owner} path: ${JSON.stringify(value)}`);
  }
  if (/^[A-Za-z]:\//.test(value)) {
    reject("CR2-SCHEMA", `invalid ${owner} path: ${JSON.stringify(value)}`);
  }
  if (value.startsWith("./")) {
    reject("CR2-SCHEMA", `${owner} must not start with ./: ${JSON.stringify(value)}`);
  }
  const normalized = path.posix.normalize(value);
  if (
    path.posix.isAbsolute(normalized) ||
    normalized === "." ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    normalized.includes("/../") ||
    normalized !== value
  ) {
    reject("CR2-SCHEMA", `invalid ${owner} path: ${JSON.stringify(value)}`);
  }
  return normalized;
}

function safeCapturePath(value) {
  const safe = safeRelativePosix(value, "capture path");
  if (!safe.startsWith("captures/") || safe === "captures/") {
    reject("CR2-SCHEMA", `invalid capture path: ${JSON.stringify(value)}`);
  }
  return safe;
}

function exactArray(values, owner) {
  if (!Array.isArray(values) || values.length === 0) {
    reject("CR2-SCHEMA", `${owner} must be a non-empty array`);
  }
  const seen = new Set();
  for (const value of values) {
    if (typeof value !== "string" || value.length === 0) {
      reject("CR2-SCHEMA", `${owner} entries must be non-empty strings`);
    }
    seen.add(value);
  }
  if (seen.size !== values.length) {
    reject("CR2-SCHEMA", `${owner} must contain unique values`);
  }
  return [...values];
}

function validateInteger(value, owner) {
  if (!Number.isInteger(value) || value <= 0) {
    reject("CR2-SCHEMA", `${owner} must be a positive integer`);
  }
}

function validateScale(value) {
  if (typeof value !== "number" || !Number.isFinite(value) || value <= 0) {
    reject("CR2-SCHEMA", "scale must be a positive finite number");
  }
}

function validateFontFiles(fontFiles) {
  if (!Array.isArray(fontFiles) || fontFiles.length === 0) {
    reject("CR2-SCHEMA", "fontFixture.files must be a non-empty array");
  }
  const seen = [];
  for (const [index, entry] of fontFiles.entries()) {
    exactFields(entry, new Set(["path", "sha256", "weight"]), `environment.fontFixture.files[${index}]`);
    const filePath = safeRelativePosix(entry.path, `environment.fontFixture.files[${index}].path`);
    if (!SHA256.test(entry.sha256 ?? "")) {
      reject("CR2-SCHEMA", `${filePath} must be a SHA-256`);
    }
    validateInteger(entry.weight, `${filePath}.weight`);
    seen.push(filePath);
  }
  for (let index = 1; index < seen.length; index += 1) {
    if (seen[index - 1] === seen[index]) {
      reject("CR2-SCHEMA", `duplicate font file path ${seen[index]}`);
    }
  }
  for (let index = 0; index < seen.length; index += 1) {
    if (index > 0 && seen[index - 1] > seen[index]) {
      reject("CR2-SCHEMA", "fontFixture.files must be in strict path order");
    }
  }
}

function validateEnvironment(environment) {
  exactFields(
    environment,
    new Set([
      "viewport",
      "scale",
      "locale",
      "timezone",
      "theme",
      "reducedMotion",
      "browserVersion",
      "browserRevision",
      "fontFixture",
    ]),
    "environment",
  );
  exactFields(environment.viewport, new Set(["width", "height"]), "environment.viewport");
  validateInteger(environment.viewport.width, "environment.viewport.width");
  validateInteger(environment.viewport.height, "environment.viewport.height");
  validateScale(environment.scale);
  nonEmptyString(environment.locale, "environment.locale");
  nonEmptyString(environment.timezone, "environment.timezone");
  nonEmptyString(environment.theme, "environment.theme");
  nonEmptyString(environment.reducedMotion, "environment.reducedMotion");
  nonEmptyString(environment.browserVersion, "environment.browserVersion");
  nonEmptyString(environment.browserRevision, "environment.browserRevision");
  exactFields(environment.fontFixture, new Set(["files", "computedFamily"]), "environment.fontFixture");
  validateFontFiles(environment.fontFixture.files);
  nonEmptyString(environment.fontFixture.computedFamily, "environment.fontFixture.computedFamily");
}

export function validateCurrentRouteManifest(manifest, { expectedVariants } = {}) {
  exactFields(
    manifest,
    new Set([
      "schemaVersion",
      "generation",
      "sourceManifestSha256",
      "transformVersion",
      "environment",
      "screens",
      "captures",
    ]),
    "manifest",
  );
  if (manifest.schemaVersion !== CURRENT_ROUTE_MANIFEST_SCHEMA_VERSION) {
    reject("CR2-SCHEMA", "schemaVersion must be 2");
  }
  validateGeneration(manifest.generation);
  if (!SHA256.test(manifest.sourceManifestSha256 ?? "")) {
    reject("CR2-SCHEMA", "sourceManifestSha256 must be a SHA-256");
  }
  nonEmptyString(manifest.transformVersion, "transformVersion");
  if (!manifest.transformVersion) {
    reject("CR2-SCHEMA", "transformVersion is required");
  }

  const variants = exactArray(expectedVariants, "expectedVariants");

  if (!Array.isArray(manifest.screens) || manifest.screens.length !== CURRENT_ROUTE_SCREENS.length) {
    reject("CR2-SCHEMA", "screens has invalid cardinality");
  }
  for (const [index, screen] of manifest.screens.entries()) {
    exactFields(screen, new Set(["screen", "mode"]), `screens[${index}]`);
    if (screen.screen !== CURRENT_ROUTE_SCREENS[index]) {
      reject("CR2-SCHEMA", "screens must match canonical screen order");
    }
    nonEmptyString(screen.mode, `screens[${index}].mode`);
  }

  if (!Array.isArray(manifest.captures)) {
    reject("CR2-SCHEMA", "captures must be an array");
  }
  const expectedPathOrder = CURRENT_ROUTE_SCREENS.flatMap((screen) =>
    variants.map((variant) => `captures/${screen}.${variant}.png`),
  );
  if (manifest.captures.length !== expectedPathOrder.length) {
    reject("CR2-SCHEMA", "captures must be the full screen x variant cartesian set");
  }

  const seenPaths = new Set();
  for (const [index, capture] of manifest.captures.entries()) {
    exactFields(capture, CAPTURE_FIELDS, `captures[${index}]`);
    const capturePath = safeCapturePath(capture.path);
    if (seenPaths.has(capturePath)) {
      reject("CR2-SCHEMA", `duplicate capture ${capturePath}`);
    }
    if (capture.screen !== CURRENT_ROUTE_SCREENS[Math.floor(index / variants.length)]) {
      reject("CR2-VARIANT", `${capturePath} screen does not match declared screen order`);
    }
    const match = capturePath.match(/^captures\/([^/.]+(?:\/.+)?)\.([^/.]+)\.png$/);
    if (!match || match[1] !== capture.screen || !match[2]) {
      reject("CR2-SCHEMA", `${capturePath} is not a canonical capture path`);
    }
    if (!variants.includes(capture.variant)) {
      reject("CR2-SCHEMA", `${capture.variant} is not in expectedVariants`);
    }
    if (capture.variant !== variants[index % variants.length]) {
      reject("CR2-SCHEMA", "capture variant does not match its canonical slot");
    }
    if (capturePath !== expectedPathOrder[index]) {
      reject("CR2-VARIANT", "capture screen/variant order is not canonical");
    }
    if (!SHA256.test(capture.sha256 ?? "")) {
      reject("CR2-SCHEMA", `${capturePath} has an invalid SHA-256`);
    }
    seenPaths.add(capturePath);
  }

  for (const screen of CURRENT_ROUTE_SCREENS) {
    const variantsByScreen = new Set(
      manifest.captures
        .filter((capture) => capture.screen === screen)
        .map((capture) => capture.variant),
    );
    for (const variant of variants) {
      if (!variantsByScreen.has(variant)) {
        reject("CR2-VARIANT", `${screen} does not cover every expected variant`);
      }
    }
  }

  validateEnvironment(manifest.environment);
  return JSON.parse(JSON.stringify(manifest));
}
