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
const FIXED_BROWSER_COMPONENT_SHA256 =
  "4edb3dfc49726aa700e77a14197571a43de2d80d9838a824c22cb68e0ac3d5b8";
const POST_PROMOTION_TASK = "G0-6H-V1ETB";
const POST_PROMOTION_FILE = "ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx";
const POST_PROMOTION_REASON = "development-only Starter Media projection";
const POST_PROMOTION_ENTRY_KEYS = [
  "task",
  "file",
  "reason",
  "fixedSourceSha256",
  "currentSha256",
];

const CURRENT_BROWSER_SOURCE = "ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx";
const CURRENT_BROWSER_CSS = "ui/motolii-web/src/candidates/discovery-browser-candidate.css";
const CURRENT_BROWSER_PATTERN = "ui/motolii-web/src/patterns/DiscoveryBrowser.jsx";
const CURRENT_BROWSER_INDEX = "ui/motolii-web/src/index.js";
const CURRENT_EASING_TRIGGER_SOURCE = "ui/motolii-web/src/candidates/EasingTriggerCandidate.jsx";
const CURRENT_EASING_TRIGGER_CSS = "ui/motolii-web/src/candidates/easing-trigger-candidate.css";
const CURRENT_KEY_TOOLS_SOURCE = "ui/motolii-web/src/candidates/KeyToolsCandidate.jsx";
const CURRENT_KEY_TOOLS_CSS = "ui/motolii-web/src/candidates/key-tools-candidate.css";
const CURRENT_INSPECTOR_SOURCE = "ui/motolii-web/src/candidates/InspectorCandidate.jsx";
const CURRENT_INSPECTOR_CSS = "ui/motolii-web/src/candidates/inspector-candidate.css";
const PRODUCT_RUNTIME_MODULES = [
  CURRENT_BROWSER_INDEX,
  CURRENT_BROWSER_SOURCE,
  CURRENT_BROWSER_PATTERN,
  CURRENT_EASING_TRIGGER_SOURCE,
  CURRENT_KEY_TOOLS_SOURCE,
  CURRENT_INSPECTOR_SOURCE,
];

const FIXED_BROWSER_SOURCE = "docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx";
const FIXED_BROWSER_CSS = "docs/mocks-ui/src/candidates/discovery-browser-candidate.css";
const FIXED_BROWSER_PATTERN = "docs/mocks-ui/src/patterns/DiscoveryBrowser.jsx";
const FIXED_EASING_TRIGGER_SOURCE = "docs/mocks-ui/src/candidates/EasingTriggerCandidate.jsx";
const FIXED_EASING_TRIGGER_CSS = "docs/mocks-ui/src/candidates/easing-trigger-candidate.css";
const FIXED_KEY_TOOLS_SOURCE = "docs/mocks-ui/src/candidates/KeyToolsCandidate.jsx";
const FIXED_KEY_TOOLS_CSS = "docs/mocks-ui/src/candidates/key-tools-candidate.css";
const FIXED_INSPECTOR_SOURCE = "docs/mocks-ui/src/candidates/InspectorCandidate.jsx";
const FIXED_INSPECTOR_CSS = "docs/mocks-ui/src/candidates/inspector-candidate.css";

const EXPECTED_EASING_TRIGGER_SHA256 =
  "6ae4cf7e79586e33cedaed0bb928daa34aa4b8ac9cdc9f6c494637875f502932";
const EXPECTED_EASING_TRIGGER_CSS_SHA256 =
  "1e2d26d51c8ed7b322d6fa56123870842daeca41fd0815e7167055c2d87acfd8";
const EXPECTED_KEY_TOOLS_SHA256 =
  "bf38656a99957a9f2d1465057820510f525fd394a3965cff9f291415223f87ba";
const EXPECTED_KEY_TOOLS_CSS_SHA256 =
  "f84eb7f98f05844fa3bfc72b702cee2709f1fc0bb9be614f2b01039a65b5190d";
const EXPECTED_INSPECTOR_SHA256 =
  "1e0bdd3eebd665e517600af4db090f74d50951aef12fdd476e97a828de91a3e4";
const EXPECTED_INSPECTOR_CSS_SHA256 =
  "730e2861a893b2b07fa66d5acef0038a49bdcf337e8c5a037785b0a58d829cbe";

const ALLOWED_EXTERNAL_PACKAGES = ["react", "html-react-parser"];

function countNonOverlapping(text, needle) {
  let count = 0;
  let index = text.indexOf(needle);
  while (index !== -1) {
    count += 1;
    index = text.indexOf(needle, index + needle.length);
  }
  return count;
}

function assertInspectorDomTokenCounts(source) {
  assert.equal(countNonOverlapping(source, "document."), 3);
  assert.equal(countNonOverlapping(source, 'document.addEventListener("keydown"'), 1);
  assert.equal(countNonOverlapping(source, 'document.removeEventListener("keydown"'), 1);
  assert.equal(countNonOverlapping(source, 'document.querySelector("#recovery")'), 1);
  assert.equal(countNonOverlapping(source, "window."), 0);
  assert.equal(countNonOverlapping(source, "useState"), 0);
  assert.equal(countNonOverlapping(source, "useMemo"), 0);
  assert.equal(countNonOverlapping(source, "localStorage"), 0);
  assert.equal(countNonOverlapping(source, "fetch("), 0);
}

function validatePostPromotionChanges(provenance, currentComponentSha256) {
  const changes = provenance.postPromotionChanges;
  if (changes === undefined) {
    if (currentComponentSha256 !== FIXED_BROWSER_COMPONENT_SHA256) {
      throw new Error("component bytes differ from fixed commit without postPromotionChanges");
    }
    return;
  }
  if (!Array.isArray(changes)) {
    throw new Error("postPromotionChanges must be an array");
  }
  if (changes.length !== 1) {
    throw new Error("postPromotionChanges must contain exactly one entry");
  }
  const entry = changes[0];
  if (entry === null || typeof entry !== "object" || Array.isArray(entry)) {
    throw new Error("postPromotionChanges entry must be an object");
  }
  const keys = Object.keys(entry);
  if (keys.length !== POST_PROMOTION_ENTRY_KEYS.length) {
    throw new Error("postPromotionChanges entry has wrong key count");
  }
  for (const key of POST_PROMOTION_ENTRY_KEYS) {
    if (!Object.hasOwn(entry, key)) {
      throw new Error(`postPromotionChanges entry missing key ${key}`);
    }
  }
  for (const key of keys) {
    if (!POST_PROMOTION_ENTRY_KEYS.includes(key)) {
      throw new Error(`postPromotionChanges entry has extra key ${key}`);
    }
  }
  if (entry.task !== POST_PROMOTION_TASK) {
    throw new Error("postPromotionChanges task literal mismatch");
  }
  if (entry.file !== POST_PROMOTION_FILE) {
    throw new Error("postPromotionChanges file literal mismatch");
  }
  if (entry.reason !== POST_PROMOTION_REASON) {
    throw new Error("postPromotionChanges reason literal mismatch");
  }
  if (entry.fixedSourceSha256 !== FIXED_BROWSER_COMPONENT_SHA256) {
    throw new Error("postPromotionChanges fixedSourceSha256 mismatch");
  }
  if (entry.currentSha256 !== currentComponentSha256) {
    throw new Error("postPromotionChanges currentSha256 mismatch");
  }
}

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
  assert.equal(exportNames.has("KeyToolsCandidate"), true);
  assert.equal(exportNames.has("InspectorCandidate"), true);
  assert.equal(exportNames.has("InspectorContext"), true);
  assert.equal(exportNames.has("default"), false);
}

test("validates fixed Browser bytes and browser export mapping", async () => {
  const productPackage = JSON.parse(await readFile(PRODUCT_PACKAGE, "utf8"));
  const provenance = JSON.parse(await readFile(path.join(PRODUCT_DIR, "source-provenance.json"), "utf8"));

  const fixedSourceMap = {
    [FIXED_BROWSER_CSS]: CURRENT_BROWSER_CSS,
    [FIXED_BROWSER_PATTERN]: CURRENT_BROWSER_PATTERN,
  };

  for (const [fixedPath, currentPath] of Object.entries(fixedSourceMap)) {
    const fixedBytes = readBlobFromCommit(fixedPath, FIXED_SOURCE_COMMIT);
    const worktreeBytes = await readFile(abs(currentPath));
    assert.equal(hashBytes(fixedBytes), hashBytes(worktreeBytes));
  }

  const browserComponentBytes = await readFile(abs(CURRENT_BROWSER_SOURCE));
  const currentBrowserSha256 = hashBytes(browserComponentBytes);
  validatePostPromotionChanges(provenance, currentBrowserSha256);

  const negativeCases = [
    {
      label: "postPromotionChanges entry count 0",
      provenance: { ...provenance, postPromotionChanges: [] },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges entry count 2+",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          provenance.postPromotionChanges[0],
          { ...provenance.postPromotionChanges[0] },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges entry key missing",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          {
            task: POST_PROMOTION_TASK,
            file: POST_PROMOTION_FILE,
            reason: POST_PROMOTION_REASON,
            fixedSourceSha256: FIXED_BROWSER_COMPONENT_SHA256,
          },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges entry key extra",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          {
            ...provenance.postPromotionChanges[0],
            extra: true,
          },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges task literal mismatch",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          {
            ...provenance.postPromotionChanges[0],
            task: "G0-6H-V1ETB-WRONG",
          },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges file literal mismatch",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          {
            ...provenance.postPromotionChanges[0],
            file: "ui/motolii-web/src/candidates/Wrong.jsx",
          },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges reason literal mismatch",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          {
            ...provenance.postPromotionChanges[0],
            reason: "wrong reason",
          },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges fixedSourceSha256 mismatch",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          {
            ...provenance.postPromotionChanges[0],
            fixedSourceSha256: `${FIXED_BROWSER_COMPONENT_SHA256.slice(0, 63)}0`,
          },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "postPromotionChanges currentSha256 mismatch",
      provenance: {
        ...provenance,
        postPromotionChanges: [
          {
            ...provenance.postPromotionChanges[0],
            currentSha256: `${currentBrowserSha256.slice(0, 63)}0`,
          },
        ],
      },
      componentSha256: currentBrowserSha256,
    },
    {
      label: "no postPromotionChanges with mismatched component bytes",
      provenance: (() => {
        const { postPromotionChanges: _removed, ...withoutChanges } = provenance;
        return withoutChanges;
      })(),
      componentSha256: hashBytes(Buffer.from("independent-mismatched-browser-component-bytes-v1etb-guard")),
    },
  ];

  for (const negativeCase of negativeCases) {
    assert.throws(() => {
      validatePostPromotionChanges(negativeCase.provenance, negativeCase.componentSha256);
    });
  }

  assert.equal(productPackage.name, PRODUCT_NAME);
  assert.equal(provenance.task, "CU-0A04 / CU-0A05B / CU-0A06B / CU-0A07C");
  assert.equal(provenance.sourceOwnership.owner, "R1-browser / R2B-easing-trigger / R3B-key-tools / R4C-inspector");
  assert.equal(provenance.sourceOwnership.surface, "Browser / Easing trigger / KEYS-LAYERS key tools / Inspector");
  assert.deepEqual(provenance.sourceOwnership.exports, [
    { name: "DiscoveryBrowserCandidate", path: "src/index.js" },
    { name: "EasingTriggerCandidate", path: "src/index.js" },
    { name: "KeyToolsCandidate", path: "src/index.js" },
    { name: "InspectorCandidate", path: "src/index.js" },
    { name: "InspectorContext", path: "src/index.js" },
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
    {
      type: "fixed-source-transfer",
      old: {
        component: FIXED_KEY_TOOLS_SOURCE,
        css: FIXED_KEY_TOOLS_CSS,
      },
      current: {
        component: CURRENT_KEY_TOOLS_SOURCE,
        css: CURRENT_KEY_TOOLS_CSS,
      },
    },
    {
      type: "fixed-source-transfer",
      old: {
        component: FIXED_INSPECTOR_SOURCE,
        css: FIXED_INSPECTOR_CSS,
      },
      current: {
        component: CURRENT_INSPECTOR_SOURCE,
        css: CURRENT_INSPECTOR_CSS,
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
  assert.throws(() => {
    readBlobFromCommit(FIXED_KEY_TOOLS_SOURCE, FIXED_SOURCE_COMMIT);
  });
  assert.throws(() => {
    readBlobFromCommit(FIXED_KEY_TOOLS_CSS, FIXED_SOURCE_COMMIT);
  });
  assert.throws(() => {
    readBlobFromCommit(FIXED_INSPECTOR_SOURCE, FIXED_SOURCE_COMMIT);
  });
  assert.throws(() => {
    readBlobFromCommit(FIXED_INSPECTOR_CSS, FIXED_SOURCE_COMMIT);
  });

  const keyToolsBytes = await readFile(abs(CURRENT_KEY_TOOLS_SOURCE));
  const keyToolsCssBytes = await readFile(abs(CURRENT_KEY_TOOLS_CSS));
  assert.equal(hashBytes(keyToolsBytes), EXPECTED_KEY_TOOLS_SHA256);
  assert.equal(hashBytes(keyToolsCssBytes), EXPECTED_KEY_TOOLS_CSS_SHA256);
  assert.equal(existsSync(abs(FIXED_KEY_TOOLS_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_KEY_TOOLS_CSS)), false);
  assert.equal(existsSync(abs(FIXED_INSPECTOR_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_INSPECTOR_CSS)), false);

  const inspectorBytes = await readFile(abs(CURRENT_INSPECTOR_SOURCE));
  const inspectorCssBytes = await readFile(abs(CURRENT_INSPECTOR_CSS));
  assert.equal(hashBytes(inspectorBytes), EXPECTED_INSPECTOR_SHA256);
  assert.equal(hashBytes(inspectorCssBytes), EXPECTED_INSPECTOR_CSS_SHA256);

  const indexSource = await readFile(abs("ui/motolii-web/src/index.js"), "utf8");
  const indexAst = parseModule(indexSource);
  assertProductExportFromIndex(indexAst);

  const exportNamedFrom = collectImportExportSources(indexAst).exportNamedFrom;
  assert.deepEqual(exportNamedFrom, [
    "./candidates/DiscoveryBrowserCandidate.jsx",
    "./candidates/EasingTriggerCandidate.jsx",
    "./candidates/InspectorCandidate.jsx",
    "./candidates/KeyToolsCandidate.jsx",
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

  const legacyRegionsSource = await readFile(path.join(DOCS_MOCKS_DIR, "src/legacy/LegacyRegions.jsx"), "utf8");
  const legacyHostSource = await readFile(path.join(DOCS_MOCKS_DIR, "src/legacy/LegacyHostBoundaryScreen.jsx"), "utf8");
  const regionsImports = collectConsumerBrowserImports(legacyRegionsSource);
  const hostImports = collectConsumerBrowserImports(legacyHostSource);
  assert.equal(regionsImports.filter((importSource) => importSource === PRODUCT_NAME).length, 1);
  assert.equal(hostImports.filter((importSource) => importSource === PRODUCT_NAME).length, 1);
  assert.equal(regionsImports.includes("../candidates/InspectorCandidate.jsx"), false);
  assert.equal(hostImports.includes("../candidates/InspectorCandidate.jsx"), false);
});

test("rejects old source-path ownership and legacy/archive/raw-import closure from product runtime", async () => {
  const source = await readFile(abs(CURRENT_BROWSER_SOURCE), "utf8");
  const easingTriggerSource = await readFile(abs(CURRENT_EASING_TRIGGER_SOURCE), "utf8");
  const keyToolsSource = await readFile(abs(CURRENT_KEY_TOOLS_SOURCE), "utf8");
  const inspectorSource = await readFile(abs(CURRENT_INSPECTOR_SOURCE), "utf8");
  assert.equal(existsSync(abs(FIXED_BROWSER_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_EASING_TRIGGER_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_EASING_TRIGGER_CSS)), false);
  assert.equal(existsSync(abs(FIXED_KEY_TOOLS_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_KEY_TOOLS_CSS)), false);
  assert.equal(existsSync(abs(FIXED_INSPECTOR_SOURCE)), false);
  assert.equal(existsSync(abs(FIXED_INSPECTOR_CSS)), false);
  assert.equal(existsSync(abs("docs/mocks-ui/src/patterns/DiscoveryBrowser.jsx")), false);
  assert.equal(existsSync(abs("docs/mocks-ui/src/candidates/discovery-browser-candidate.css")), false);

  for (const runtimeModule of PRODUCT_RUNTIME_MODULES) {
    validateProductRuntimeSource(
      abs(runtimeModule),
      await readFile(abs(runtimeModule), "utf8"),
    );
  }

  // 逆例は重複束縛のparse失敗ではなくvalidatorのimport境界拒否で落とす
  const forbiddenKeyToolsMockImport =
    `import { KeyToolsCandidate as ForbiddenMockKeyToolsCandidate } from "../../../docs/mocks-ui/src/candidates/KeyToolsCandidate.jsx";\n${keyToolsSource}`;

  const forbiddenInspectorMockImport =
    `import { InspectorCandidate as ForbiddenMockInspectorCandidate } from "../../../docs/mocks-ui/src/candidates/InspectorCandidate.jsx";\n${inspectorSource}`;

  const badCandidates = [
    [CURRENT_BROWSER_SOURCE, `import { LegacyHostBoundaryScreen } from "../legacy/LegacyHostBoundaryScreen.jsx";\n${source}`],
    [CURRENT_BROWSER_SOURCE, `import { foo } from "docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx";\n${source}`],
    [CURRENT_BROWSER_SOURCE, `import boundary from "docs/mocks/m3-vism-host-boundary.html?raw";\n${source}`],
    [CURRENT_BROWSER_SOURCE, `import { bad } from "../archive/legacy-script.js";\n${source}`],
    [CURRENT_BROWSER_SOURCE, `import { bad } from "/src/legacy/legacySource.js";\n${source}`],
    [CURRENT_EASING_TRIGGER_SOURCE, `import { EasingTriggerCandidate } from "../../../docs/mocks-ui/src/candidates/EasingTriggerCandidate.jsx";\n${easingTriggerSource}`],
    [CURRENT_EASING_TRIGGER_SOURCE, `import boundary from "docs/mocks/m3-vism-host-boundary.html?raw";\n${easingTriggerSource}`],
    [CURRENT_EASING_TRIGGER_SOURCE, `import { bad } from "../legacy/legacySource.js";\n${easingTriggerSource}`],
    [CURRENT_EASING_TRIGGER_SOURCE, `import { createRoot } from "react-dom";\n${easingTriggerSource}`],
    [CURRENT_KEY_TOOLS_SOURCE, forbiddenKeyToolsMockImport],
    [CURRENT_KEY_TOOLS_SOURCE, `import boundary from "docs/mocks/m3-vism-host-boundary.html?raw";\n${keyToolsSource}`],
    [CURRENT_KEY_TOOLS_SOURCE, `import { bad } from "../legacy/legacySource.js";\n${keyToolsSource}`],
    [CURRENT_KEY_TOOLS_SOURCE, `import { TimelineCandidate } from "../../../docs/mocks-ui/src/candidates/TimelineCandidate.jsx";\n${keyToolsSource}`],
    [CURRENT_INSPECTOR_SOURCE, forbiddenInspectorMockImport],
    [CURRENT_INSPECTOR_SOURCE, `import boundary from "docs/mocks/m3-vism-host-boundary.html?raw";\n${inspectorSource}`],
    [CURRENT_INSPECTOR_SOURCE, `import { bad } from "../legacy/legacySource.js";\n${inspectorSource}`],
    [CURRENT_INSPECTOR_SOURCE, `import { createRoot } from "react-dom";\n${inspectorSource}`],
  ];

  assert.doesNotThrow(() => parseModule(forbiddenInspectorMockImport));
  assert.doesNotThrow(() => parseModule(forbiddenKeyToolsMockImport));

  for (const [modulePath, candidate] of badCandidates) {
    assert.throws(() => {
      validateProductRuntimeSource(abs(modulePath), candidate);
    });
  }

  for (const forbidden of [
    "document.",
    "window.",
    "useState",
    "useEffect",
    "useRef",
    "useMemo",
    "useReducer",
    "keyToolsOpen",
    "selectedKeys",
    "selectedObjects",
    "applyKeyOperation",
    "applyLayerOperation",
  ]) {
    assert.ok(!keyToolsSource.includes(forbidden), `forbidden token ${forbidden}`);
  }

  assertInspectorDomTokenCounts(inspectorSource);
  assert.throws(() => {
    assertInspectorDomTokenCounts(`${inspectorSource}\ndocument.querySelector("x");`);
  });
  assert.throws(() => {
    assertInspectorDomTokenCounts(
      inspectorSource.replace(
        'document.addEventListener("keydown"',
        'document.querySelector("keydown"',
      ),
    );
  });

  const badIndexReexport = `export { EasingTriggerCandidate } from "../../../docs/mocks-ui/src/candidates/EasingTriggerCandidate.jsx";\n`;
  const badInspectorIndexReexport =
    `export { InspectorCandidate } from "../../../docs/mocks-ui/src/candidates/InspectorCandidate.jsx";\n`;
  assert.throws(() => {
    validateProductRuntimeSource(abs(CURRENT_BROWSER_INDEX), badIndexReexport);
  });
  assert.throws(() => {
    validateProductRuntimeSource(abs(CURRENT_BROWSER_INDEX), badInspectorIndexReexport);
  });
});
