import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  CURRENT_ROUTE_MANIFEST_SCHEMA_VERSION,
  CURRENT_ROUTE_PROVENANCE_PATH,
  CURRENT_ROUTE_SCREENS,
  CurrentRouteGenerationError,
  validateCurrentRouteManifest,
} from "./current-route-generation.mjs";
import {
  publishImmutableGeneration,
  readImmutableGeneration,
} from "./immutable-generation.mjs";
import { REFERENCE_VARIANTS } from "./reference-transform.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const CURRENT_ROUTE_DIR = path.resolve(__dirname, "..");
const CURRENT_ROUTE_PROVENANCE_FULL_PATH = path.join(
  CURRENT_ROUTE_DIR,
  CURRENT_ROUTE_PROVENANCE_PATH,
);

function throwRouteError(code, message, cause) {
  throw new CurrentRouteGenerationError(code, message, cause ? { cause } : undefined);
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function validateCurrentManifestSource(manifest) {
  const provenanceBytes = readFileSync(CURRENT_ROUTE_PROVENANCE_FULL_PATH);
  const provenance = JSON.parse(readFileSync(CURRENT_ROUTE_PROVENANCE_FULL_PATH, "utf8"));
  const sourceManifestSha256 = sha256(provenanceBytes);

  if (manifest.sourceManifestSha256 !== sourceManifestSha256) {
    throwRouteError(
      "CR2-SCHEMA",
      "sourceManifestSha256 does not match current provenance manifest",
    );
  }
  if (provenance.schemaVersion !== CURRENT_ROUTE_MANIFEST_SCHEMA_VERSION) {
    throwRouteError("CR2-SCHEMA", "current-route provenance schema version is unexpected");
  }
}

export function validateCurrentRoutePublication(
  manifest,
  captures,
  { requireCurrentSource = true } = {},
) {
  if (requireCurrentSource) {
    validateCurrentManifestSource(manifest);
  }

  const validatedManifest = validateCurrentRouteManifest(manifest, {
    expectedVariants: REFERENCE_VARIANTS,
  });

  const capturesByPath = new Map(
    (validatedManifest.captures ?? []).map((capture) => [capture.path, capture]),
  );

  if (validatedManifest.screens.length !== CURRENT_ROUTE_SCREENS.length) {
    throwRouteError("CR2-SCHEMA", "captures missing one or more screens");
  }

  if (captures.size !== validatedManifest.captures.length) {
    throwRouteError("CR2-CAPTURE", "capture count does not match manifest");
  }

  for (const capture of validatedManifest.captures) {
    const expectedBytes = captures.get(capture.path);
    if (!expectedBytes) {
      throwRouteError("CR2-CAPTURE", `missing capture bytes for ${capture.path}`);
    }
    if (sha256(expectedBytes) !== capture.sha256) {
      throwRouteError(
        "CR2-CAPTURE",
        `capture bytes do not match manifest SHA-256 for ${capture.path}`,
      );
    }
    capturesByPath.delete(capture.path);
  }

  if (capturesByPath.size !== 0) {
    throwRouteError("CR2-CAPTURE", `undeclared capture ${capturesByPath.keys().next().value}`);
  }

  return {
    generation: validatedManifest.generation,
    captures,
  };
}

function publishFail(kind, message, cause) {
  if (kind === "schema") return throwRouteError("CR2-SCHEMA", message, cause);
  if (kind === "publish") return throwRouteError("CR2-PUBLISH", message, cause);
  if (kind === "read") return throwRouteError("CR2-READ", message, cause);
  return throwRouteError("CR2-CAPTURE", message, cause);
}

export async function publishCurrentRouteGeneration({ root, manifest, captures, inject }) {
  return publishImmutableGeneration({
    root,
    manifest,
    captures,
    inject,
    validate: validateCurrentRoutePublication,
    fail: publishFail,
  });
}

export async function readPublishedCurrentRouteGeneration({
  root,
  requireCurrentSource = true,
}) {
  return readImmutableGeneration({
    root,
    validate(manifest, captures) {
      return validateCurrentRoutePublication(manifest, captures, {
        requireCurrentSource,
      });
    },
    fail(kind, message, cause) {
      if (kind === "schema") return throwRouteError("CR2-SCHEMA", message, cause);
      if (kind === "capture") return throwRouteError("CR2-CAPTURE", message, cause);
      if (kind === "publish") return throwRouteError("CR2-PUBLISH", message, cause);
      return throwRouteError("CR2-READ", message, cause);
    },
  });
}
