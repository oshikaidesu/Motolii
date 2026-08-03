import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { readFile, readdir } from "node:fs/promises";
import path from "node:path";
import assert from "node:assert/strict";
import test from "node:test";

const productRoot = path.resolve(import.meta.dirname, "..");
const repoRoot = path.resolve(productRoot, "../..");
const mockRoot = path.join(repoRoot, "docs/mocks-ui");

const productSource = path.join(productRoot, "src/feedback/Feedback.jsx");
const productCss = path.join(productRoot, "src/feedback/feedback.css");
const mockSource = path.join(mockRoot, "src/feedback/Feedback.jsx");
const mockCss = path.join(mockRoot, "src/feedback/feedback.css");

const sourceSha256 =
  "459fdd6120fd369b78d4a9784d98ac2b29fbb553afb35522f8f680fdfe4e4cd1";
const cssSha256 =
  "7e22e2a183796732c4f77c4bb018eb2342ecb812181e46f348e1aa3aa827ef50";

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

async function sourceFiles(root) {
  const entries = await readdir(root, { recursive: true, withFileTypes: true });
  return entries
    .filter((entry) => entry.isFile())
    .map((entry) => path.join(entry.parentPath, entry.name))
    .filter((file) => /\.(?:js|jsx|mjs|css)$/.test(file));
}

test("CU-203P promotes the fixed feedback bytes and leaves one mock consumer", async () => {
  const [sourceBytes, cssBytes, shim, matrix] = await Promise.all([
    readFile(productSource),
    readFile(productCss),
    readFile(mockSource, "utf8"),
    readFile(
      path.join(mockRoot, "src/diagnostics/FeedbackStateMatrix.jsx"),
      "utf8",
    ),
  ]);

  assert.equal(sha256(sourceBytes), sourceSha256);
  assert.equal(sha256(cssBytes), cssSha256);
  assert.equal(
    shim,
    'export { Feedback } from "@motolii/motolii-web";\n',
  );
  assert.equal(existsSync(mockCss), false);
  assert.match(matrix, /from "\.\.\/feedback\/Feedback\.jsx"/);

  for (const file of await sourceFiles(path.join(mockRoot, "src"))) {
    const source = await readFile(file, "utf8");
    assert.doesNotMatch(
      source,
      /export function (?:Feedback|validateFeedbackModel)\(/,
      `mock source must not retain a second feedback implementation: ${file}`,
    );
  }
});

test("CU-203P exposes only the presentation component at the package root", async () => {
  const [index, source, playwright] = await Promise.all([
    readFile(path.join(productRoot, "src/index.js"), "utf8"),
    readFile(productSource, "utf8"),
    readFile(
      path.join(mockRoot, "tests/feedback-state-matrix.spec.js"),
      "utf8",
    ),
  ]);

  assert.match(
    index,
    /export \{ Feedback \} from "\.\/feedback\/Feedback\.jsx";/,
  );
  assert.doesNotMatch(index, /validateFeedbackModel/);
  assert.match(source, /export function Feedback\(/);
  assert.match(source, /export function validateFeedbackModel\(/);
  assert.match(playwright, /const \{ Feedback \} = await import/);
  assert.match(playwright, /Feedback\(model\);/);
  assert.doesNotMatch(playwright, /validateFeedbackModel/);

  for (const forbidden of [
    "docs/mocks-ui",
    "docs/mocks",
    "legacy",
    "archive",
    "DiagnosticEnvelope",
    "DomainIntent",
    "onRecover",
    "onRetry",
  ]) {
    assert.equal(
      source.includes(forbidden),
      false,
      `product feedback contains forbidden dependency or public semantic: ${forbidden}`,
    );
  }
});

test("CU-203P records product ownership and the exact transfer", async () => {
  const provenance = JSON.parse(
    await readFile(path.join(productRoot, "source-provenance.json"), "utf8"),
  );
  const feedbackExport = provenance.sourceOwnership.exports.find(
    ({ name }) => name === "Feedback",
  );
  assert.deepEqual(feedbackExport, {
    name: "Feedback",
    path: "src/index.js",
  });

  const migration = provenance.migrations.find(
    ({ task }) => task === "CU-203P",
  );
  assert.deepEqual(migration, {
    type: "fixed-source-transfer",
    task: "CU-203P",
    old: {
      component: "docs/mocks-ui/src/feedback/Feedback.jsx",
      componentSha256: sourceSha256,
      css: "docs/mocks-ui/src/feedback/feedback.css",
      cssSha256,
    },
    current: {
      component: "ui/motolii-web/src/feedback/Feedback.jsx",
      componentSha256: sourceSha256,
      css: "ui/motolii-web/src/feedback/feedback.css",
      cssSha256,
    },
  });
});
