import assert from "node:assert/strict";
import { mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { scanRawColors } from "../scripts/raw-color-scanner.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const PRODUCT_ROOT = path.resolve(HERE, "../../../ui/motolii-tokens");
const ALLOWED_FILES = [
  "generated/manifest.json",
  "generated/tokens.css",
  "generated/tokens.rs",
  "sources/motolii-dark.json",
];

const ROLE_SOURCES = new Map([
  ["color-action-active", ["active", "accent"]],
  ["color-border-default", ["line", "neutral-700"]],
  ["color-border-strong", ["line2", "neutral-500"]],
  ["color-data", ["data", "data"]],
  ["color-focus", ["ink", "neutral-050"]],
  ["color-shape", ["shape", "shape"]],
  ["color-status-ok", ["ok", "ok"]],
  ["color-status-warning", ["warning", "warning"]],
  ["color-surface-app", ["bg", "neutral-950"]],
  ["color-surface-hover", ["hover", "neutral-800"]],
  ["color-surface-panel", ["panel", "neutral-925"]],
  ["color-surface-raised", ["raised", "neutral-875"]],
  ["color-text-muted", ["muted", "neutral-300"]],
  ["color-text-primary", ["ink", "neutral-050"]],
  ["color-text-secondary", ["sub", "neutral-150"]],
  ["color-way-files", ["way-files", "way-files"]],
  ["color-way-inspector", ["way-inspector", "way-inspector"]],
  ["color-way-plugins", ["way-plugins", "way-plugins"]],
  ["color-way-project", ["way-project", "way-project"]],
  ["color-way-stage", ["way-stage", "way-stage"]],
  ["color-way-timeline", ["way-timeline", "way-timeline"]],
]);

async function walkFiles(root) {
  const files = [];
  async function visit(directory) {
    const entries = await readdir(directory, { withFileTypes: true });
    entries.sort((left, right) => left.name.localeCompare(right.name));
    for (const entry of entries) {
      const filename = path.join(directory, entry.name);
      if (entry.isDirectory()) await visit(filename);
      else if (entry.isFile()) files.push(path.relative(root, filename).split(path.sep).join("/"));
      else assert.fail(`unexpected non-file product token entry: ${filename}`);
    }
  }
  await visit(root);
  return files;
}

function cssVariables(source, prefix = "") {
  return new Map(
    [...source.matchAll(/--([a-z0-9-]+)\s*:\s*(#[0-9a-f]{6,8})\s*;/gi)]
      .filter((match) => match[1].startsWith(prefix))
      .map((match) => [match[1].slice(prefix.length), match[2].toLowerCase()]),
  );
}

test("product token root is the closed source and generated bundle", async () => {
  assert.deepEqual(await walkFiles(PRODUCT_ROOT), ALLOWED_FILES);
  const manifest = JSON.parse(
    await readFile(path.join(PRODUCT_ROOT, "generated/manifest.json"), "utf8"),
  );
  assert.equal(manifest.generator.id, "motolii-ui-token-gen");
  assert.equal(manifest.generator.version, 2);
  assert.deepEqual(manifest.themes, ["motolii-dark"]);
  assert.deepEqual(manifest.outputs, ["tokens.rs", "tokens.css", "manifest.json"]);
  assert.equal(manifest.tokens.length, ROLE_SOURCES.size);
  assert.equal(manifest.tokens.some(({ path: tokenPath }) => tokenPath.includes("object")), false);
});

test("generated Dark roles match both accepted suppliers", async () => {
  const generated = cssVariables(
    await readFile(path.join(PRODUCT_ROOT, "generated/tokens.css"), "utf8"),
    "motolii-",
  );
  const legacy = cssVariables(
    await readFile(path.resolve(PRODUCT_ROOT, "../../docs/mocks/m3-vism-host-boundary.html"), "utf8"),
  );
  const mock = cssVariables(
    await readFile(
      path.resolve(PRODUCT_ROOT, "../../docs/mocks-ui/src/tokens/mock-candidates.css"),
      "utf8",
    ),
    "mock-candidate-color-",
  );

  assert.equal(generated.size, ROLE_SOURCES.size);
  for (const [role, [legacyName, mockName]] of ROLE_SOURCES) {
    const generatedValue = generated.get(role)?.replace(/ff$/, "");
    assert.equal(generatedValue, legacy.get(legacyName), `${role} differs from legacy supplier`);
    assert.equal(generatedValue, mock.get(mockName), `${role} differs from mock supplier`);
  }
});

test("shared raw-color scanner rejects handwritten supplier CSS and JSX", async () => {
  const root = await mkdtemp(path.join(tmpdir(), "motolii-token-guard-"));
  try {
    for (const [name, source] of [
      ["supplier.css", ".supplier { color: #fff; }\n"],
      ["supplier.jsx", 'export const Supplier = () => <div style={{ color: "#fff" }} />;\n'],
    ]) {
      const filename = path.join(root, name);
      await writeFile(filename, source);
      await assert.rejects(
        scanRawColors(filename, (code, message) => {
          const error = new Error(message);
          error.code = code;
          throw error;
        }),
        (error) => error.code === "RG-RAW-COLOR",
      );
    }
  } finally {
    await rm(root, { recursive: true, force: true });
  }
});
