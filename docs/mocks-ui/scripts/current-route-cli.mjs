import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { CURRENT_ROUTE_OUTPUT_ROOT } from "./current-route-generation.mjs";
import { CurrentRouteGenerationError } from "./current-route-generation.mjs";
import { captureCurrentRouteBundle } from "./current-route-capture.mjs";
import { decodedPixels } from "./reference-transform.mjs";
import {
  publishCurrentRouteGeneration,
  readPublishedCurrentRouteGeneration,
} from "./current-route-publication.mjs";
import { repositoryFingerprint } from "./repository-fingerprint.mjs";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const OUTPUT_ROOT = path.join(ROOT, "docs/mocks-ui", CURRENT_ROUTE_OUTPUT_ROOT);

function fail(code, message, cause) {
  throw new CurrentRouteGenerationError(code, message, cause ? { cause } : undefined);
}

function existsCurrentRouteOutput() {
  const current = path.join(OUTPUT_ROOT, "CURRENT");
  return existsSync(current) && readFileSync(current, "utf8").trim().length > 0;
}

function bundlesMatch(published, captured) {
  if (!published || !captured) return false;
  if (JSON.stringify(published.manifest) !== JSON.stringify(captured.manifest)) return false;
  if (published.captures.size !== captured.captures.size) return false;

  for (const [capturePath, expectedBytes] of captured.captures) {
    const actualBytes = published.captures.get(capturePath);
    if (!actualBytes) return false;
    if (!decodedPixels(actualBytes).equals(decodedPixels(expectedBytes))) return false;
  }

  return true;
}

async function generate() {
  const fingerprintBefore = await repositoryFingerprint(ROOT);
  const bundle = await captureCurrentRouteBundle({ repoRoot: ROOT });
  if (bundle.repositoryFingerprint !== fingerprintBefore) {
    fail("CR2-SOURCE", "capture bundle does not match repository fingerprint");
  }

  if (existsCurrentRouteOutput()) {
    const current = await readPublishedCurrentRouteGeneration({ root: OUTPUT_ROOT });
    if (bundlesMatch(current, bundle)) {
      console.log(`current-route generation already published: ${current.generation}`);
      return;
    }
    fail("CR2-PUBLISH", "current-route generation differs from capture bundle");
  }

  await publishCurrentRouteGeneration({
    root: OUTPUT_ROOT,
    manifest: bundle.manifest,
    captures: bundle.captures,
  });
  console.log(`current-route generation published: ${bundle.manifest.generation}`);
}

async function check() {
  const before = await repositoryFingerprint(ROOT);
  const bundle = await captureCurrentRouteBundle({ repoRoot: ROOT });
  if (bundle.repositoryFingerprint !== before) {
    fail("CR2-SOURCE", "capture bundle does not match repository fingerprint");
  }

  const current = await readPublishedCurrentRouteGeneration({ root: OUTPUT_ROOT });
  if (!bundlesMatch(current, bundle)) {
    fail("CR2-CAPTURE", "current-route generation does not match captured bundle");
  }

  const after = await repositoryFingerprint(ROOT);
  if (before !== after) {
    fail("CR2-READ", "check-current-route changed repository bytes or status");
  }
  console.log(`current-route generation OK: ${current.generation} (${current.captures.size} PNGs)`);
}

const command = process.argv[2];
if (command === "generate") {
  await generate();
} else if (command === "check") {
  await check();
} else {
  throw new Error(
    `usage: node scripts/current-route-cli.mjs generate|check`,
  );
}
