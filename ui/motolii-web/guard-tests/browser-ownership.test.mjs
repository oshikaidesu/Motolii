import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

const TEST_DIR = path.dirname(fileURLToPath(import.meta.url));
const PRODUCT_DIR = path.resolve(TEST_DIR, "..");
const REPO_DIR = execFileSync("git", ["rev-parse", "--show-toplevel"], {
  cwd: PRODUCT_DIR,
  encoding: "utf8",
}).trim();
const DOCS_MOCKS_DIR = path.resolve(REPO_DIR, "docs/mocks-ui");
const DOCS_MOCKS_PACKAGE = path.join(DOCS_MOCKS_DIR, "package.json");
const PRODUCT_PACKAGE = path.join(PRODUCT_DIR, "package.json");

const requireFromMocks = createRequire(DOCS_MOCKS_PACKAGE);
const { parse } = requireFromMocks("@babel/parser");

const FIXED_SOURCE_COMMIT = "56c318edcddab7cf95d263cc2f7dd2b4e6791134";
const PRODUCT_NAME = "@motolii/motolii-web";

const CURRENT_BROWSER_SOURCE = "ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx";
const CURRENT_BROWSER_CSS = "ui/motolii-web/src/candidates/discovery-browser-candidate.css";
const CURRENT_BROWSER_PATTERN = "ui/motolii-web/src/patterns/DiscoveryBrowser.jsx";
const CURRENT_BROWSER_INDEX = "ui/motolii-web/src/index.js";
const CURRENT_EASING_TRIGGER_SOURCE = "ui/motolii-web/src/candidates/EasingTriggerCandidate.jsx";
const CURRENT_EASING_TRIGGER_CSS = "ui/motolii-web/src/candidates/easing-trigger-candidate.css";
const PRODUCT_RUNTIME_MODULES = [
  CURRENT_BROWSER_INDEX,
  CURRENT_BROWSER_SOURCE,
  CURRENT_BROWSER_PATTERN,
  CURRENT_EASING_TRIGGER_SOURCE,
];

const FIXED_BROWSER_SOURCE = "docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx";
const FIXED_BROWSER_CSS = "docs/mocks-ui/src/candidates/discovery-browser-candidate.css";
const FIXED_BROWSER_PATTERN = "docs/mocks-ui/src/patterns/DiscoveryBrowser.jsx";
const FIXED_EASING_TRIGGER_SOURCE = "docs/mocks-ui/src/candidates/EasingTriggerCandidate.jsx";
const FIXED_EASING_TRIGGER_CSS = "docs/mocks-ui/src/candidates/easing-trigger-candidate.css";

const EXPECTED_EASING_TRIGGER_SHA256 =
  "6ae4cf7e79586e33cedaed0bb928daa34aa4b8ac9cdc9f6c494637875f502932";
const EXPECTED_EASING_TRIGGER_CSS_SHA256 =
  "1e2d26d51c8ed7b322d6fa56123870842daeca41fd0815e7167055c2d87acfd8";

const ALLOWED_EXTERNAL_PACKAGES = ["react", "html-react-parser"];

function hashBytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function abs(relativePath) {
  return path.resolve(REPO_DIR, relativePath);
}

function readBlobFromCommit(relativePath, commit) {
  return execFileSync(
    "git",
    ["show", `${commit}:${relativePath}`],
    { cwd: REPO_DIR, encoding: null },
  );
}

function parseModule(source) {
  return parse(source, {
    sourceType: "module",
    plugins: ["jsx", "importAttributes", "topLevelAwait"],
  });
}

function collectImportExportSources(ast) {
  const imports = [];
  const exportNamedFrom = [];

  const walk = (node) => {
    if (!node || typeof node !== "object") {
      return;
    }
    if (Array.isArray(node)) {
      node.forEach(walk);
      return;
    }
    if (node.type === "ImportDeclaration" && node.source?.type === "StringLiteral") {
      imports.push(node.source.value);
    }
    if (node.type === "ExportNamedDeclaration" && node.source?.type === "StringLiteral") {
      exportNamedFrom.push(node.source.value);
    }
    for (const child of Object.values(node)) {
      walk(child);
    }
  };

  walk(ast);
  return { imports, exportNamedFrom };
}

function collectNamedExports(ast) {
  const names = new Set();
  const walk = (node) => {
    if (!node || typeof node !== "object") {
      return;
    }
    if (Array.isArray(node)) {
      node.forEach(walk);
      return;
    }
    if (node.type === "ExportNamedDeclaration") {
      for (const specifier of node.specifiers ?? []) {
        if (specifier.type === "ExportSpecifier") {
          names.add(specifier.exported.name ?? specifier.exported.value);
        }
      }
      if (node.declaration?.type === "VariableDeclaration") {
        for (const declarator of node.declaration.declarations) {
          if (declarator.id?.type === "Identifier") {
            names.add(declarator.id.name);
          }
        }
      }
    }
    if (node.type === "ExportDefaultDeclaration") {
      names.add("default");
    }
    for (const child of Object.values(node)) {
      walk(child);
    }
  };
  walk(ast);
  return names;
}

function isForbiddenRelativeImport(importerPath, sourceValue) {
  if (!sourceValue.startsWith(".")) {
    return false;
  }
  const resolved = path.resolve(path.dirname(importerPath), sourceValue);
  const normalized = resolved.replaceAll(path.sep, "/");
  return normalized.includes("/docs/mocks-ui/src/")
    || normalized.includes("/docs/mocks/")
    || normalized.includes("/src/legacy/")
    || normalized.includes("/src/archive/");
}

function isForbiddenBareImport(sourceValue) {
  return sourceValue.endsWith(".html")
    || sourceValue.includes("docs/mocks-ui")
    || sourceValue.includes("/legacy/")
    || sourceValue.includes("/archive/");
}

function validateProductRuntimeSource(sourcePath, sourceText) {
  const ast = parseModule(sourceText);
  const { imports, exportNamedFrom } = collectImportExportSources(ast);

  for (const source of imports) {
    if (isForbiddenRelativeImport(sourcePath, source)) {
      throw new Error(`forbidden runtime import: ${source}`);
    }
    if (isForbiddenBareImport(source)) {
      throw new Error(`forbidden bare import: ${source}`);
    }
    if (!source.startsWith(".") && !ALLOWED_EXTERNAL_PACKAGES.includes(source)) {
      throw new Error(`forbidden bare import: ${source}`);
    }
  }

  for (const source of exportNamedFrom) {
    if (source.startsWith(".") && isForbiddenRelativeImport(sourcePath, source)) {
      throw new Error(`forbidden runtime re-export: ${source}`);
    }
    if (isForbiddenBareImport(source)) {
      throw new Error(`forbidden bare runtime re-export: ${source}`);
    }
    if (!source.startsWith(".") && !ALLOWED_EXTERNAL_PACKAGES.includes(source)) {
      throw new Error(`forbidden bare runtime re-export: ${source}`);
    }
  }
}

function collectConsumerBrowserImports(sourceText) {
  const ast = parseModule(sourceText);
  const { imports } = collectImportExportSources(ast);
  return imports;
}

function assertProductExportFromIndex(ast) {
  const exportNames = collectNamedExports(ast);
  assert.equal(exportNames.has("DiscoveryBrowserCandidate"), true);
  assert.equal(exportNames.has("EasingTriggerCandidate"), true);
  assert.equal(exportNames.has("default"), false);
}

test("validates fixed Browser bytes and browser export mapping", async () => {
  const productPackage = JSON.parse(await readFile(PRODUCT_PACKAGE, "utf8"));
  const provenance = JSON.parse(await readFile(path.join(PRODUCT_DIR, "source-provenance.json"), "utf8"));

  const fixedSourceMap = {
    [FIXED_BROWSER_SOURCE]: CURRENT_BROWSER_SOURCE,
    [FIXED_BROWSER_CSS]: CURRENT_BROWSER_CSS,
    [FIXED_BROWSER_PATTERN]: CURRENT_BROWSER_PATTERN,
  };

  for (const [fixedPath, currentPath] of Object.entries(fixedSourceMap)) {
    const fixedBytes = readBlobFromCommit(fixedPath, FIXED_SOURCE_COMMIT);
    const worktreeBytes = await readFile(abs(currentPath));
    assert.equal(hashBytes(fixedBytes), hashBytes(worktreeBytes));
  }

  assert.equal(productPackage.name, PRODUCT_NAME);
  assert.equal(provenance.task, "CU-0A04 / CU-0A05B");
  assert.equal(provenance.sourceOwnership.owner, "R1-browser / R2B-easing-trigger");
  assert.equal(provenance.sourceOwnership.surface, "Browser / Easing trigger");
  assert.deepEqual(provenance.sourceOwnership.exports, [
    { name: "DiscoveryBrowserCandidate", path: "src/index.js" },
    { name: "EasingTriggerCandidate", path: "src/index.js" },
  ]);
  assert.deepEqual(provenance.migrations, [
    {
      type: "fixed-source-transfer",
      old: {
        component: "docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx",
        css: "docs/mocks-ui/src/candidates/discovery-browser-candidate.css",
        pattern: "docs/mocks-ui/src/patterns/DiscoveryBrowser.jsx",
      },
      current: {
        component: "ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx",
        css: "ui/motolii-web/src/candidates/discovery-browser-candidate.css",
        pattern: "ui/motolii-web/src/patterns/DiscoveryBrowser.jsx",
      },
    },
    {
      type: "fixed-source-transfer",
      old: {
        component: FIXED_EASING_TRIGGER_SOURCE,
        css: FIXED_EASING_TRIGGER_CSS,
      },
      current: {
        component: CURRENT_EASING_TRIGGER_SOURCE,
        css: CURRENT_EASING_TRIGGER_CSS,
      },
    },
  ]);

  const easingTriggerBytes = await readFile(abs(CURRENT_EASING_TRIGGER_SOURCE));
  const easingTriggerCssBytes = await readFile(abs(CURRENT_EASING_TRIGGER_CSS));
  assert.equal(hashBytes(easingTriggerBytes), EXPECTED_EASING_TRIGGER_SHA256);
  assert.equal(hashBytes(easingTriggerCssBytes), EXPECTED_EASING_TRIGGER_CSS_SHA256);
  assert.throws(() => {
    readBlobFromCommit(FIXED_EASING_TRIGGER_SOURCE, FIXED_SOURCE_COMMIT);
  });

  const indexSource = await readFile(abs("ui/motolii-web/src/index.js"), "utf8");
  const indexAst = parseModule(indexSource);
  assertProductExportFromIndex(indexAst);

  const exportNamedFrom = collectImportExportSources(indexAst).exportNamedFrom;
  assert.deepEqual(exportNamedFrom, [
    "./candidates/DiscoveryBrowserCandidate.jsx",
    "./candidates/EasingTriggerCandidate.jsx",
  ]);
});

test("validates product export/consumer import topology via parsed AST", async () => {
  const mainSource = await readFile(path.join(DOCS_MOCKS_DIR, "src/main.jsx"), "utf8");
  const storySource = await readFile(path.join(DOCS_MOCKS_DIR, "src/stories/LegacyHostBoundaryScreen.stories.jsx"), "utf8");

  const mainImports = collectConsumerBrowserImports(mainSource);
  const storyImports = collectConsumerBrowserImports(storySource);

  assert.equal(mainImports.includes(PRODUCT_NAME), true);
  assert.equal(storyImports.includes(PRODUCT_NAME), true);
  assert.equal(mainImports.includes("./candidates/DiscoveryBrowserCandidate.jsx"), false);
  assert.equal(mainImports.includes("./candidates/EasingTriggerCandidate.jsx"), false);
  assert.equal(storyImports.includes("../candidates/DiscoveryBrowserCandidate.jsx"), false);
  assert.equal(mainImports.filter((importSource) => importSource === PRODUCT_NAME).length, 1);
});

test("rejects old source-path ownership and legacy/archive/raw-import closure from product runtime", async () => {
  const source = await readFile(abs(CURRENT_BROWSER_SOURCE), "utf8");
  const easingTriggerSource = await readFile(abs(CURRENT_EASING_TRIGGER_SOURCE), "utf8");
  assert.equal(existsSync(abs(FIXED_BROWSER_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_EASING_TRIGGER_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_EASING_TRIGGER_CSS)), false);
  assert.equal(existsSync(abs("docs/mocks-ui/src/patterns/DiscoveryBrowser.jsx")), false);
  assert.equal(existsSync(abs("docs/mocks-ui/src/candidates/discovery-browser-candidate.css")), false);

  for (const runtimeModule of PRODUCT_RUNTIME_MODULES) {
    validateProductRuntimeSource(
      abs(runtimeModule),
      await readFile(abs(runtimeModule), "utf8"),
    );
  }

  const badCandidates = [
    `import { LegacyHostBoundaryScreen } from "../legacy/LegacyHostBoundaryScreen.jsx";\n${source}`,
    `import { foo } from "docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx";\n${source}`,
    `import boundary from "docs/mocks/m3-vism-host-boundary.html?raw";\n${source}`,
    `import { bad } from "../archive/legacy-script.js";\n${source}`,
    `import { bad } from "/src/legacy/legacySource.js";\n${source}`,
    `import { EasingTriggerCandidate } from "../../../docs/mocks-ui/src/candidates/EasingTriggerCandidate.jsx";\n${easingTriggerSource}`,
    `import boundary from "docs/mocks/m3-vism-host-boundary.html?raw";\n${easingTriggerSource}`,
    `import { bad } from "../legacy/legacySource.js";\n${easingTriggerSource}`,
    `import { createRoot } from "react-dom";\n${easingTriggerSource}`,
  ];

  for (const candidate of badCandidates) {
    assert.throws(() => {
      validateProductRuntimeSource(
        abs(candidate.includes("EasingTrigger") ? CURRENT_EASING_TRIGGER_SOURCE : CURRENT_BROWSER_SOURCE),
        candidate,
      );
    });
  }

  const badIndexReexport = `export { EasingTriggerCandidate } from "../../../docs/mocks-ui/src/candidates/EasingTriggerCandidate.jsx";\n`;
  assert.throws(() => {
    validateProductRuntimeSource(abs(CURRENT_BROWSER_INDEX), badIndexReexport);
  });
});
