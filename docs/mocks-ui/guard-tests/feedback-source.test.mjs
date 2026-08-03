import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const root = path.resolve(import.meta.dirname, "..");
const read = (relativePath) =>
  readFile(path.join(root, relativePath), "utf8");
const repoRoot = path.resolve(root, "../..");
const readProduct = (relativePath) =>
  readFile(path.join(repoRoot, "ui/motolii-web", relativePath), "utf8");

test("CU-203P product source keeps the closed presentation vocabulary", async () => {
  const source = await readProduct("src/feedback/Feedback.jsx");

  for (const value of ["inline", "target", "badge", "cursor"]) {
    assert.match(source, new RegExp(`"${value}"`));
  }
  for (const value of [
    "neutral",
    "valid",
    "warning",
    "error",
    "loading",
    "disabled",
  ]) {
    assert.match(source, new RegExp(`"${value}"`));
  }
  for (const value of [
    "retry-with-changed-input",
    "requires-another-action",
    "unrecoverable",
  ]) {
    assert.match(source, new RegExp(`"${value}"`));
  }

  assert.match(source, /export function Feedback\(/);
  assert.match(source, /export function validateFeedbackModel\(/);
  assert.match(source, /contextRequiredTones/);
  assert.match(source, /reason and recovery must be supplied together/);
  assert.match(source, /requires typed reason and recovery/);
  assert.match(source, /aria-describedby=/);
  assert.match(source, /aria-busy=/);
  assert.match(source, /data-feedback-reason=/);
  assert.match(source, /data-feedback-recovery=/);

  for (const forbidden of [
    "useState",
    "useEffect",
    "useReducer",
    "localStorage",
    "sessionStorage",
    "document.",
    "window.",
    "onRecover",
    "onRetry",
    "DomainIntent",
    "DiagnosticEnvelope",
    "<svg",
    "glyph=",
  ]) {
    assert.equal(
      source.includes(forbidden),
      false,
      `feedback source contains forbidden owner or invented surface: ${forbidden}`,
    );
  }
});

test("CU-203P product CSS uses product colors and component-local geometry only", async () => {
  const css = await readProduct("src/feedback/feedback.css");

  assert.doesNotMatch(css, /#[0-9a-f]{3,8}\b/i);
  assert.doesNotMatch(css, /\b(?:rgb|hsl)a?\(/i);
  assert.doesNotMatch(css, /url\(/i);
  assert.doesNotMatch(css, /content\s*:/i);
  assert.match(css, /--motolii-color-status-ok/);
  assert.match(css, /--motolii-color-status-warning/);
  assert.match(css, /border-style:\s*dashed/);
  assert.match(css, /border-radius:\s*50%/);
  assert.match(css, /prefers-reduced-motion:\s*no-preference/);
});

test("CU-203P matrix remains exact and diagnostic-only", async () => {
  const [matrix, main, playwright] = await Promise.all([
    read("src/diagnostics/FeedbackStateMatrix.jsx"),
    read("src/main.jsx"),
    read("tests/feedback-state-matrix.spec.js"),
  ]);

  const caseIds = [
    "inline-neutral",
    "target-valid",
    "target-invalid",
    "disabled-action",
    "warning",
    "error-unrecoverable",
    "loading",
    "semantic-badge",
    "cursor-context",
  ];
  for (const id of caseIds) {
    assert.equal(
      matrix.match(new RegExp(`id: "${id}"`, "g"))?.length,
      1,
      `${id} must occur exactly once as a fixture id`,
    );
    assert.match(playwright, new RegExp(`"${id}"`));
  }
  assert.equal(matrix.match(/\bid:\s*"/g)?.length, caseIds.length);
  assert.match(
    main,
    /"diagnostics\/feedback-states":\s*\{\s*title:\s*"Diagnostics \/ common feedback states",\s*Component:\s*FeedbackStateMatrix,\s*catalogKind:\s*"diagnostic"/s,
  );
  assert.doesNotMatch(main, /catalogKind:\s*"candidate"[^}]*FeedbackStateMatrix/s);
  assert.doesNotMatch(main, /catalogKind:\s*"archive"[^}]*FeedbackStateMatrix/s);
});
