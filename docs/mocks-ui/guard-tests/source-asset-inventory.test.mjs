import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { parse } from "@babel/parser";
import { htmlToDOM } from "html-react-parser";
import postcss from "postcss";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
import test from "node:test";

const ROOT_DIR = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const REPO_DIR = execFileSync("git", ["rev-parse", "--show-toplevel"], {
  cwd: ROOT_DIR,
  encoding: "utf8",
}).trim();
const MANIFEST_PATH = path.join(ROOT_DIR, "source-asset-inventory.json");

const FIXED_SOURCE_COMMIT = "56c318edcddab7cf95d263cc2f7dd2b4e6791134";
const EXPECTED_BROWSER_SOURCE = "docs/mocks-ui/src/candidates/DiscoveryBrowserCandidate.jsx";
const EXPECTED_CSS_SOURCE = "docs/mocks-ui/src/candidates/discovery-browser-candidate.css";
const EXPECTED_PATTERN_SOURCE = "docs/mocks-ui/src/patterns/DiscoveryBrowser.jsx";
const EXPECTED_PATTERN_IMPORTS = [
  "DiscoveryBrowserLayout",
  "DiscoveryResults",
  "DiscoverySearchBar",
  "DiscoverySection",
  "DiscoverySourceRail",
  "DiscoveryViewToggle",
];
const EXPECTED_RUNTIME_HASHES = {
  [EXPECTED_BROWSER_SOURCE]: "4edb3dfc49726aa700e77a14197571a43de2d80d9838a824c22cb68e0ac3d5b8",
  [EXPECTED_CSS_SOURCE]: "1dcb6afc3c16907366f6d73ed7cfb1b04c8cea872d169e959ead49b6c6cedccd",
  [EXPECTED_PATTERN_SOURCE]: "1d996ad66dba3ff7fb36cf811ce8d22faec1fee271a2dd5349d953a7cf89a2ea",
};
const EXPECTED_EXTERNAL_PACKAGES = ["html-react-parser", "react"];
const EXPECTED_TEST_ROUTE = "plugin-browser-candidate";
const EXPECTED_TEST_PATH = "docs/mocks-ui/tests/browser-candidate.spec.js";
const EXPECTED_EASING_SOURCE = "docs/mocks-ui/src/candidates/EasingGraphCandidate.jsx";
const EXPECTED_EASING_CSS_SOURCE = "docs/mocks-ui/src/candidates/easing-graph-candidate.css";
const EXPECTED_EASING_MODEL_SOURCE = "docs/mocks-ui/src/candidates/easing-graph-model.js";
const EXPECTED_EASING_MODEL_IMPORTS = [
  "ADVANCED_SPECS",
  "PLOT",
  "advancedPathPoints",
  "clamp",
  "makeInitialAdvancedParameters",
  "pointFrom",
  "snap",
  "viewForOvershoot",
  "xOf",
  "yOf",
];
const EXPECTED_EASING_RUNTIME_HASHES = {
  [EXPECTED_EASING_SOURCE]: "1b1a3ab66808504d4356bbb8ffd65bb8ed9aa77726a71b65ffa6b26bd61b4a05",
  [EXPECTED_EASING_CSS_SOURCE]: "644064b649778d6dfe08de4d73751d5e7b65b96133115eba07be799aeb8e0329",
  [EXPECTED_EASING_MODEL_SOURCE]: "22a6745bcd77b6f71f5c62563b0165f6161a234abebd2a10f326d210a0a6fad9",
};
const EXPECTED_EASING_PROMOTION_BOUNDARY = [
  "Graph trigger/icon",
  "current-value summary",
];
const EXPECTED_EASING_NATIVE_ORACLE = [
  "popup frame",
  "presets",
  "user library",
  "form",
  "curve renderer",
  "easing model",
];
const EXPECTED_TIMELINE_SOURCE = "docs/mocks-ui/src/candidates/TimelineCandidate.jsx";
const EXPECTED_TIMELINE_CSS_SOURCE = "docs/mocks-ui/src/candidates/timeline-candidate.css";
const EXPECTED_TIMELINE_RUNTIME_HASHES = {
  [EXPECTED_TIMELINE_SOURCE]: "c777d77a7d9403692090199e7fe7e5caec953e0f8509febaf7bc86f692764eb5",
  [EXPECTED_TIMELINE_CSS_SOURCE]: "ef984d9b365f4efbcb4bf8fc20034a0b54846ab4fb470ea8d6ec8b486aa71397",
};
const EXPECTED_TIMELINE_PROMOTION_BOUNDARY = [
  "candidate-key-tools subtree",
  "candidate-key-tools-open reopen control",
];
const EXPECTED_TIMELINE_NATIVE_ORACLE = [
  "ruler",
  "rails",
  "bars",
  "keys",
  "playhead",
  "graph",
  "packing plane",
  "time surface",
];
const EXPECTED_TIMELINE_MODES = {
  keys: ["align", "stagger", "stretch"],
  layers: ["align", "stagger", "shift"],
};
const EXPECTED_TIMELINE_TEST_PATH = "docs/mocks-ui/tests/timeline-candidate.spec.js";
const EXPECTED_INSPECTOR_CLASSIFICATION = "react-source-absent-legacy-parity-oracle";
const EXPECTED_INSPECTOR_SOURCE_STATUS = "independent-react-source-absent";
const EXPECTED_INSPECTOR_LEGACY_CLOSURE = [
  "docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx",
  "docs/mocks-ui/src/legacy/LegacyRegions.jsx",
  "docs/mocks-ui/src/legacy/legacySource.js",
  "docs/mocks/m3-vism-host-boundary.html",
];
const EXPECTED_INSPECTOR_LEGACY_HASHES = {
  "docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx": "10d780c99d38536bb53aba5c4b6ddbbcb6d706b099f581595d6b3539f76ec416",
  "docs/mocks-ui/src/legacy/LegacyRegions.jsx": "8fb24e75abdb87dd52ae7d1c723782b42e67e486331aa04cbd498111a2733b3a",
  "docs/mocks-ui/src/legacy/legacySource.js": "4fd547d2efa98d6d1fadf403b4ae2379abb0767d50cda172f8f6fa749a230e20",
  "docs/mocks/m3-vism-host-boundary.html": "a20bd36d6b8b241ef4ec21adda6b045e470bf89fbc78e2788207aa6d0875ec68",
};
const EXPECTED_INSPECTOR_LEGACY_EXPORT = "LegacyInspector";
const EXPECTED_INSPECTOR_SKELETON_PATH = "docs/mocks-ui/src/surfaces/InspectorSurface.jsx";
const EXPECTED_INSPECTOR_SKELETON_HASH = "70e3f1094ae6188274779055d20385cccc2efabd7258b994eeba869e3ea82f90";
const EXPECTED_INSPECTOR_NEXT_ACTION = "mock-side-same-shape-react-extraction-and-parity-before-r4";
const EXPECTED_INSPECTOR_PARITY_PATH = "docs/mocks-ui/tests/visual-parity.spec.js";
const EXPECTED_INSPECTOR_REACT_ROUTE = "archive/all-surfaces";
const EXPECTED_INSPECTOR_LEGACY_ROUTE = "all-surfaces";
const EXPECTED_PANEL_LAYOUT_SOURCE = "docs/mocks-ui/src/layout/ResizablePanelLayout.jsx";
const EXPECTED_PANEL_LAYOUT_CSS_SOURCE = "docs/mocks-ui/src/layout/resizable-panel-layout.css";
const EXPECTED_PANEL_LAYOUT_RUNTIME_HASHES = {
  [EXPECTED_PANEL_LAYOUT_SOURCE]: "3af5334dff20551954e9fb7bac1cc1e5fdf894e357706a0d9a33fbe09d211359",
  [EXPECTED_PANEL_LAYOUT_CSS_SOURCE]: "d6ea3b03d48c2f8d5e3102b0a3712f1c6bdc825acdd4c80d6a57d637f89ec249",
};
const EXPECTED_PANEL_LAYOUT_EXPORTS = [
  "ResizableLegacyApp",
  "ResizableLegacyWorkspace",
  "ResizableLegacyTimeline",
  "ResizableTimelineSurface",
];
const EXPECTED_PANEL_LAYOUT_PROMOTION_BOUNDARY = [
  "layout sizing and clamping",
  "PanelLayoutContext",
  "PanelSeparator",
];
const EXPECTED_PANEL_LAYOUT_EXCLUDED_BOUNDARY = [
  "html-react-parser legacy node/options adapters",
  "ResizableLegacy* fixture wrappers",
  "ResizableTimelineSurface fixture wrapper",
  "native viewport bounds and topology",
];
const EXPECTED_PANEL_LAYOUT_TEST_PATH = "docs/mocks-ui/tests/panel-layout.spec.js";
const EXPECTED_PANEL_LAYOUT_TEST_HASH = "419c82363b37c6dbf97ef872f64351750946cd0dc79f664515f6f2bd9326b334";
const EXPECTED_NATIVE_STAGE_TIME_CLOSURE = [
  "docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx",
  "docs/mocks-ui/src/legacy/LegacyRegions.jsx",
  "docs/mocks-ui/src/legacy/legacySource.js",
  "docs/mocks/m3-vism-host-boundary.html",
  EXPECTED_TIMELINE_SOURCE,
  EXPECTED_TIMELINE_CSS_SOURCE,
];
const EXPECTED_NATIVE_STAGE_TIME_HASHES = {
  ...EXPECTED_INSPECTOR_LEGACY_HASHES,
  ...EXPECTED_TIMELINE_RUNTIME_HASHES,
};
const EXPECTED_STAGE_ORACLE = ["stage-shell", "output-frame", "transport"];
const EXPECTED_NATIVE_TIMELINE_ORACLE = EXPECTED_TIMELINE_NATIVE_ORACLE;
const EXPECTED_STAGE_SURFACE_PATH = "docs/mocks-ui/src/surfaces/StageSurface.jsx";
const EXPECTED_STAGE_SURFACE_HASH = "fec7b23593af66194895b736458297d99932a1dc9a6d37b8f38c95c25e31e46f";
const EXPECTED_NATIVE_VISUAL_PARITY_HASH = "c71b2e34d343e21aa6a32387b0692897efd5d1f56931dc8c873fcb617e5687ef";
const EXPECTED_NATIVE_TIMELINE_TEST_HASH = "0c0da4260662dbdcd7de89d7da247c3334db8fdfe29315f1ed77e43b0ac994f4";

function hashBytes(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function readBlobFromCommit(relativePath, commit) {
  return execFileSync("git", ["show", `${commit}:${relativePath}`], {
    cwd: REPO_DIR,
    encoding: null,
  });
}

function parseModule(source) {
  return parse(source, {
    sourceType: "module",
    plugins: ["jsx", "importAttributes", "topLevelAwait"],
  });
}

function collectNamedExports(ast) {
  const names = new Set();
  for (const statement of ast.program.body) {
    if (statement.type === "ExportDefaultDeclaration") {
      names.add("default");
      continue;
    }
    if (statement.type !== "ExportNamedDeclaration") {
      continue;
    }
    for (const specifier of statement.specifiers ?? []) {
      names.add(specifier.exported.name ?? specifier.exported.value);
    }
    const declaration = statement.declaration;
    if (declaration?.type === "FunctionDeclaration" && declaration.id?.type === "Identifier") {
      names.add(declaration.id.name);
      continue;
    }
    if (declaration?.type === "VariableDeclaration") {
      for (const declarator of declaration.declarations) {
        if (declarator.id.type === "Identifier") {
          names.add(declarator.id.name);
        }
      }
    }
  }
  return names;
}

function ensureExactKeys(value, allowed) {
  assert.deepEqual(Object.keys(value).sort(), [...allowed].sort());
}

function relFromRepo(absolutePath) {
  return path.relative(REPO_DIR, absolutePath).replaceAll(path.sep, "/");
}

function absoluteFromRelative(relativePath) {
  return path.resolve(REPO_DIR, relativePath);
}

function collectCandidateImports(candidateAst, candidatePath) {
  const localImports = new Map();
  const externalPackages = new Set();

  for (const statement of candidateAst.program.body) {
    if (statement.type !== "ImportDeclaration") {
      continue;
    }
    const sourceValue = statement.source.value;
    if (typeof sourceValue !== "string") {
      continue;
    }
    if (sourceValue.startsWith(".")) {
      const resolved = path.resolve(path.dirname(candidatePath), sourceValue);
      const relative = relFromRepo(resolved);
      localImports.set(
        relative,
        statement.specifiers.map((specifier) => {
          if (specifier.type === "ImportDefaultSpecifier") {
            return {
              kind: "default",
              local: specifier.local.name,
            };
          }
          if (specifier.type === "ImportNamespaceSpecifier") {
            return {
              kind: "namespace",
              local: specifier.local.name,
            };
          }
          return {
            kind: "named",
            imported: specifier.imported.name ?? specifier.imported.value,
            local: specifier.local.name,
          };
        }),
      );
      continue;
    }
    externalPackages.add(sourceValue);
  }

  return {
    localImports: Object.fromEntries(localImports),
    externalPackages: [...externalPackages].sort(),
  };
}

function extractRouteFromTest(testAst) {
  const routes = [];
  const stack = [testAst];
  const routePattern = /^https?:\/\/[^"'\s]+#([^"'\s]+)$/;

  const visit = (value) => {
    if (!value || typeof value !== "object") {
      return;
    }
    if (Array.isArray(value)) {
      for (const entry of value) {
        visit(entry);
      }
      return;
    }

    if (value.type === "StringLiteral" && typeof value.value === "string") {
      const match = routePattern.exec(value.value);
      if (match) {
        routes.push(match[1]);
      }
      return;
    }

    if (value.type === "TemplateLiteral") {
      const cooked = value.quasis
        .map((quasi) => quasi.value.cooked ?? quasi.value.raw)
        .join("");
      const match = routePattern.exec(cooked) ?? /#([^"'\s]+)$/.exec(cooked);
      if (match) {
        routes.push(match[1]);
      }
      return;
    }

    for (const child of Object.values(value)) {
      visit(child);
    }
  };

  visit(testAst);
  return [...new Set(routes)];
}

async function manifestFromDisk() {
  return JSON.parse(await readFile(MANIFEST_PATH, "utf8"));
}

function withInventoryEntryAt(manifest, collection, index, patch) {
  return {
    ...manifest,
    [collection]: manifest[collection].map((entry, entryIndex) =>
      entryIndex === index ? patch(entry) : entry,
    ),
  };
}

function countJsxClass(ast, expectedClass) {
  let count = 0;
  const visit = (value) => {
    if (!value || typeof value !== "object") return;
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (
      value.type === "JSXOpeningElement" &&
      value.name.type === "JSXIdentifier"
    ) {
      const className = value.attributes.find(
        (attribute) =>
          attribute.type === "JSXAttribute" &&
          attribute.name.name === "className",
      );
      const classTokens = className?.value?.type === "StringLiteral"
        ? className.value.value.split(/\s+/)
        : className?.value?.type === "JSXExpressionContainer" &&
            className.value.expression.type === "TemplateLiteral"
          ? className.value.expression.quasis.flatMap((quasi) =>
            (quasi.value.cooked ?? quasi.value.raw).split(/\s+/),
          )
          : [];
      if (classTokens.includes(expectedClass)) {
        count += 1;
      }
    }
    Object.values(value).forEach(visit);
  };
  visit(ast);
  return count;
}

function hasJsxIdentifier(ast, identifier) {
  let found = false;
  const visit = (value) => {
    if (!value || typeof value !== "object" || found) return;
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value.type === "JSXIdentifier" && value.name === identifier) {
      found = true;
      return;
    }
    Object.values(value).forEach(visit);
  };
  visit(ast);
  return found;
}

function hasStageParserReplacement(ast) {
  let replacementFound = false;
  const visit = (value) => {
    if (!value || typeof value !== "object") return;
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (
      value.type === "IfStatement" &&
      value.test.type === "CallExpression" &&
      value.test.callee.type === "Identifier" &&
      value.test.callee.name === "matches" &&
      value.test.arguments[1]?.type === "ObjectExpression" &&
      value.test.arguments[1].properties.some(
        (property) =>
          property.type === "ObjectProperty" &&
          property.key.type === "Identifier" &&
          property.key.name === "className" &&
          property.value.type === "StringLiteral" &&
          property.value.value === "stage-shell",
      )
    ) {
      replacementFound = hasJsxIdentifier(value.consequent, "LegacyStageShell");
    }
    Object.values(value).forEach(visit);
  };
  visit(ast);
  return replacementFound;
}

function hasHtmlClass(node, className) {
  return node?.attribs?.class?.split(/\s+/).includes(className) ?? false;
}

function findHtmlClass(node, className) {
  if (hasHtmlClass(node, className)) return node;
  for (const child of node?.children ?? []) {
    const found = findHtmlClass(child, className);
    if (found) return found;
  }
  return null;
}

function hasCssSelectorRoot(cssSource, rootSelector) {
  const stylesheet = postcss.parse(cssSource);
  return stylesheet.nodes.some(
    (node) =>
      node.type === "rule" &&
      node.selectors.some((selector) => {
        const normalized = selector.trim();
        return (
          normalized === rootSelector ||
          normalized.startsWith(`${rootSelector} `) ||
          normalized.startsWith(`${rootSelector}:`) ||
          normalized.startsWith(`${rootSelector}[`) ||
          normalized.startsWith(`${rootSelector} >`)
        );
      }),
  );
}

function hasCssClassToken(cssSource, className) {
  const classPattern = new RegExp(`\\.${className}(?![A-Za-z0-9_-])`);
  const stylesheet = postcss.parse(cssSource);
  return stylesheet.nodes.some(
    (node) =>
      node.type === "rule" &&
      node.selectors.some((selector) => classPattern.test(selector)),
  );
}

function importsNamedExport(ast, source, exported) {
  return ast.program.body.some((statement) =>
    statement.type === "ImportDeclaration" &&
    statement.source.value === source &&
    statement.specifiers.some(
      (specifier) =>
        specifier.type === "ImportSpecifier" &&
        (specifier.imported.name ?? specifier.imported.value) === exported,
    ),
  );
}

function importsSource(ast, source) {
  return ast.program.body.some(
    (statement) => statement.type === "ImportDeclaration" && statement.source.value === source,
  );
}

function hasTopLevelVariable(ast, name) {
  return ast.program.body.some(
    (statement) =>
      statement.type === "VariableDeclaration" &&
      statement.declarations.some(
        (declarator) => declarator.id.type === "Identifier" && declarator.id.name === name,
      ),
  );
}

function hasFunctionDeclaration(ast, name) {
  return ast.program.body.some(
    (statement) =>
      statement.type === "FunctionDeclaration" && statement.id?.name === name,
  );
}

function hasPanelSpec(ast, panel) {
  return ast.program.body.some(
    (statement) =>
      statement.type === "VariableDeclaration" &&
      statement.declarations.some(
        (declarator) =>
          declarator.id.type === "Identifier" &&
          declarator.id.name === "PANEL_SPEC" &&
          declarator.init?.type === "ObjectExpression" &&
          declarator.init.properties.some(
            (property) =>
              property.type === "ObjectProperty" &&
              property.key.type === "Identifier" &&
              property.key.name === panel &&
              property.value.type === "ObjectExpression",
          ),
      ),
  );
}

function hasInspectorParserReplacement(ast) {
  const contains = (value, predicate) => {
    if (!value || typeof value !== "object") return false;
    if (predicate(value)) return true;
    if (Array.isArray(value)) return value.some((entry) => contains(entry, predicate));
    return Object.values(value).some((entry) => contains(entry, predicate));
  };
  const isInspectorMatch = (value) => {
    if (
      value.type !== "CallExpression" ||
      value.callee.type !== "Identifier" ||
      value.callee.name !== "matches" ||
      value.arguments[1]?.type !== "ObjectExpression"
    ) {
      return false;
    }
    return value.arguments[1].properties.some(
      (property) =>
        property.type === "ObjectProperty" &&
        property.key.type === "Identifier" &&
        property.key.name === "id" &&
        property.value.type === "StringLiteral" &&
        property.value.value === "inspector",
    );
  };
  let replacementFound = false;
  const visit = (value) => {
    if (!value || typeof value !== "object") return;
    if (Array.isArray(value)) {
      value.forEach(visit);
      return;
    }
    if (value.type === "IfStatement" && contains(value.test, isInspectorMatch)) {
      replacementFound ||= contains(
        value.consequent,
        (node) =>
          node.type === "JSXOpeningElement" &&
          node.name.type === "JSXIdentifier" &&
          node.name.name === EXPECTED_INSPECTOR_LEGACY_EXPORT,
      );
    }
    Object.values(value).forEach(visit);
  };
  visit(ast);
  return replacementFound;
}

async function validateInventory(manifest, options = {}) {
  const {
    candidateAstSource,
    easingAstSource,
    timelineAstSource,
    timelineCssSource,
    inspectorHostAstSource,
    inspectorRegionsAstSource,
    inspectorLegacySourceAstSource,
    inspectorSkeletonAstSource,
    inspectorParityAstSource,
    panelLayoutAstSource,
    panelLayoutCssSource,
    panelLayoutTestAstSource,
    stageHostAstSource,
    stageRegionsAstSource,
    stageLegacySourceAstSource,
    stageRawHtmlSource,
    nativeTimelineAstSource,
    nativeTimelineCssSource,
    stageSurfaceAstSource,
    nativeVisualParityAstSource,
    nativeTimelineTestAstSource,
    fixedSourceCommit = manifest.fixedSourceCommit,
  } = options;

  assert.equal(Object.getPrototypeOf(manifest), Object.prototype);
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.task, "CU-0A03");
  assert.equal(manifest.scope, "incomplete-multi-surface-r0-slice");
  assert.equal(manifest.completeR0, false);
  assert.equal(manifest.fixedSourceCommit, FIXED_SOURCE_COMMIT);
  assert.equal(fixedSourceCommit, FIXED_SOURCE_COMMIT);

  ensureExactKeys(manifest, [
    "schemaVersion",
    "task",
    "scope",
    "completeR0",
    "fixedSourceCommit",
    "authority",
    "surfaces",
    "modelClosure",
    "behavioralTests",
  ]);

  ensureExactKeys(manifest.authority, [
    "isProductApi",
    "isPersistenceFormat",
    "isProductOwnerTransfer",
  ]);
  assert.equal(manifest.authority.isProductApi, false);
  assert.equal(manifest.authority.isPersistenceFormat, false);
  assert.equal(manifest.authority.isProductOwnerTransfer, false);

  assert.equal(manifest.modelClosure.length, 0);
  assert.equal(Array.isArray(manifest.surfaces), true);
  assert.equal(manifest.surfaces.length, 6);
  assert.equal(Array.isArray(manifest.behavioralTests), true);
  assert.equal(manifest.behavioralTests.length, 2);

  const browser = manifest.surfaces[0];
  ensureExactKeys(browser, [
    "id",
    "classification",
    "componentPath",
    "componentExport",
    "runtimeClosure",
    "localImports",
    "externalPackages",
  ]);

  assert.equal(browser.id, "browser");
  assert.equal(browser.classification, "react-direct-promotion");
  assert.equal(browser.componentPath, EXPECTED_BROWSER_SOURCE);
  assert.equal(browser.componentExport, "DiscoveryBrowserCandidate");
  assert.deepEqual(browser.externalPackages, EXPECTED_EXTERNAL_PACKAGES);

  assert.equal(Array.isArray(browser.runtimeClosure), true);
  assert.equal(browser.runtimeClosure.length, 3);
  assert.equal(Array.isArray(browser.localImports), true);
  assert.equal(browser.localImports.length, 2);

  const expectedRuntimeOrder = [
    EXPECTED_BROWSER_SOURCE,
    EXPECTED_CSS_SOURCE,
    EXPECTED_PATTERN_SOURCE,
  ];

  for (let index = 0; index < expectedRuntimeOrder.length; index += 1) {
    const expectedPath = expectedRuntimeOrder[index];
    const entry = browser.runtimeClosure[index];

    ensureExactKeys(entry, ["path", "role", "sha256"]);
    assert.equal(entry.path, expectedPath);
    assert.equal(entry.role, "runtime");
    assert.equal(entry.sha256, EXPECTED_RUNTIME_HASHES[expectedPath]);
    assert.ok(!entry.path.includes("/legacy/") && !entry.path.includes("/archive/"));

    const commitBytes = readBlobFromCommit(entry.path, fixedSourceCommit);
    assert.equal(hashBytes(commitBytes), entry.sha256);

    const worktreeBytes = await readFile(absoluteFromRelative(entry.path));
    assert.equal(hashBytes(worktreeBytes), entry.sha256);
  }

  ensureExactKeys(browser.localImports[0], ["kind", "source", "specifiers"]);
  ensureExactKeys(browser.localImports[1], ["kind", "source", "specifiers"]);
  assert.equal(browser.localImports[0].kind, "css-side-effect");
  assert.equal(browser.localImports[0].source, EXPECTED_CSS_SOURCE);
  assert.deepEqual(browser.localImports[0].specifiers, []);
  assert.equal(browser.localImports[1].kind, "named");
  assert.equal(browser.localImports[1].source, EXPECTED_PATTERN_SOURCE);
  assert.deepEqual(browser.localImports[1].specifiers, EXPECTED_PATTERN_IMPORTS);

  ensureExactKeys(manifest.behavioralTests[0], ["path", "route"]);
  assert.equal(manifest.behavioralTests[0].path, EXPECTED_TEST_PATH);
  assert.equal(manifest.behavioralTests[0].route, EXPECTED_TEST_ROUTE);
  ensureExactKeys(manifest.behavioralTests[1], ["path", "route"]);
  assert.equal(manifest.behavioralTests[1].path, EXPECTED_TIMELINE_TEST_PATH);
  assert.equal(manifest.behavioralTests[1].route, EXPECTED_TEST_ROUTE);

  const candidateSource = candidateAstSource ?? (await readFile(
    absoluteFromRelative(browser.componentPath),
    "utf8",
  ));
  const candidateAst = parseModule(candidateSource);
  const candidateExports = collectNamedExports(candidateAst);
  assert.ok(candidateExports.has("DiscoveryBrowserCandidate"));

  const { localImports, externalPackages } = collectCandidateImports(
    candidateAst,
    absoluteFromRelative(browser.componentPath),
  );

  assert.deepEqual(externalPackages, EXPECTED_EXTERNAL_PACKAGES);
  assert.deepEqual(
    Object.keys(localImports).sort(),
    [EXPECTED_CSS_SOURCE, EXPECTED_PATTERN_SOURCE].sort(),
  );

  assert.equal(localImports[EXPECTED_CSS_SOURCE].length, 0);
  const patternSpecifiers = localImports[EXPECTED_PATTERN_SOURCE]
    .filter((entry) => entry.kind === "named")
    .map((entry) => entry.imported);
  assert.equal(patternSpecifiers.length, EXPECTED_PATTERN_IMPORTS.length);
  assert.deepEqual(patternSpecifiers, EXPECTED_PATTERN_IMPORTS);

  const patternExports = collectNamedExports(
    parseModule(await readFile(absoluteFromRelative(EXPECTED_PATTERN_SOURCE), "utf8")),
  );
  for (const required of EXPECTED_PATTERN_IMPORTS) {
    assert.ok(patternExports.has(required));
  }

  const easing = manifest.surfaces[1];
  ensureExactKeys(easing, [
    "id",
    "classification",
    "componentPath",
    "componentExport",
    "runtimeClosure",
    "localImports",
    "externalPackages",
    "promotionBoundary",
    "nativeOracle",
    "behavioralEvidence",
  ]);
  assert.equal(easing.id, "easing");
  assert.equal(easing.classification, "react-trigger-native-popup-oracle");
  assert.equal(easing.componentPath, EXPECTED_EASING_SOURCE);
  assert.equal(easing.componentExport, "EasingGraphCandidate");
  assert.deepEqual(easing.externalPackages, ["react"]);
  assert.deepEqual(easing.promotionBoundary, EXPECTED_EASING_PROMOTION_BOUNDARY);
  assert.deepEqual(easing.nativeOracle, EXPECTED_EASING_NATIVE_ORACLE);

  const expectedEasingRuntimeOrder = [
    EXPECTED_EASING_SOURCE,
    EXPECTED_EASING_CSS_SOURCE,
    EXPECTED_EASING_MODEL_SOURCE,
  ];
  assert.equal(easing.runtimeClosure.length, expectedEasingRuntimeOrder.length);
  for (let index = 0; index < expectedEasingRuntimeOrder.length; index += 1) {
    const expectedPath = expectedEasingRuntimeOrder[index];
    const entry = easing.runtimeClosure[index];
    ensureExactKeys(entry, ["path", "role", "sha256"]);
    assert.equal(entry.path, expectedPath);
    assert.equal(entry.role, "runtime");
    assert.equal(entry.sha256, EXPECTED_EASING_RUNTIME_HASHES[expectedPath]);
    assert.ok(!entry.path.includes("/legacy/") && !entry.path.includes("/archive/"));
    assert.equal(hashBytes(readBlobFromCommit(entry.path, fixedSourceCommit)), entry.sha256);
    assert.equal(hashBytes(await readFile(absoluteFromRelative(entry.path))), entry.sha256);
  }

  assert.equal(easing.localImports.length, 2);
  ensureExactKeys(easing.localImports[0], ["kind", "source", "specifiers"]);
  ensureExactKeys(easing.localImports[1], ["kind", "source", "specifiers"]);
  assert.equal(easing.localImports[0].kind, "css-side-effect");
  assert.equal(easing.localImports[0].source, EXPECTED_EASING_CSS_SOURCE);
  assert.deepEqual(easing.localImports[0].specifiers, []);
  assert.equal(easing.localImports[1].kind, "named");
  assert.equal(easing.localImports[1].source, EXPECTED_EASING_MODEL_SOURCE);
  assert.deepEqual(easing.localImports[1].specifiers, EXPECTED_EASING_MODEL_IMPORTS);

  ensureExactKeys(easing.behavioralEvidence, ["path", "route"]);
  assert.deepEqual(easing.behavioralEvidence, manifest.behavioralTests[0]);

  const easingAst = parseModule(easingAstSource ?? await readFile(
    absoluteFromRelative(easing.componentPath),
    "utf8",
  ));
  assert.ok(collectNamedExports(easingAst).has("EasingGraphCandidate"));
  const easingImports = collectCandidateImports(easingAst, absoluteFromRelative(easing.componentPath));
  assert.deepEqual(easingImports.externalPackages, ["react"]);
  assert.deepEqual(
    Object.keys(easingImports.localImports).sort(),
    [EXPECTED_EASING_CSS_SOURCE, EXPECTED_EASING_MODEL_SOURCE].sort(),
  );
  assert.equal(easingImports.localImports[EXPECTED_EASING_CSS_SOURCE].length, 0);
  const modelSpecifiers = easingImports.localImports[EXPECTED_EASING_MODEL_SOURCE]
    .filter((entry) => entry.kind === "named")
    .map((entry) => entry.imported);
  assert.deepEqual(modelSpecifiers, EXPECTED_EASING_MODEL_IMPORTS);
  const modelExports = collectNamedExports(
    parseModule(await readFile(absoluteFromRelative(EXPECTED_EASING_MODEL_SOURCE), "utf8")),
  );
  for (const required of EXPECTED_EASING_MODEL_IMPORTS) {
    assert.ok(modelExports.has(required));
  }

  const testSource = await readFile(absoluteFromRelative(manifest.behavioralTests[0].path), "utf8");
  const testAst = parseModule(testSource);
  const parsedRoutes = extractRouteFromTest(testAst);
  assert.deepEqual(parsedRoutes, [EXPECTED_TEST_ROUTE]);

  const keysLayers = manifest.surfaces[2];
  ensureExactKeys(keysLayers, [
    "id",
    "classification",
    "componentPath",
    "componentExport",
    "runtimeClosure",
    "localImports",
    "externalPackages",
    "promotionBoundary",
    "nativeOracle",
    "modes",
    "behavioralEvidence",
  ]);
  assert.equal(keysLayers.id, "keys-layers");
  assert.equal(keysLayers.classification, "react-subtree-extraction-native-timeline-oracle");
  assert.equal(keysLayers.componentPath, EXPECTED_TIMELINE_SOURCE);
  assert.equal(keysLayers.componentExport, "TimelineCandidate");
  assert.deepEqual(keysLayers.externalPackages, ["react"]);
  assert.deepEqual(keysLayers.promotionBoundary, EXPECTED_TIMELINE_PROMOTION_BOUNDARY);
  assert.deepEqual(keysLayers.nativeOracle, EXPECTED_TIMELINE_NATIVE_ORACLE);
  ensureExactKeys(keysLayers.modes, ["keys", "layers"]);
  assert.deepEqual(keysLayers.modes, EXPECTED_TIMELINE_MODES);
  assert.deepEqual(keysLayers.behavioralEvidence, manifest.behavioralTests[1]);

  const expectedTimelineRuntimeOrder = [EXPECTED_TIMELINE_SOURCE, EXPECTED_TIMELINE_CSS_SOURCE];
  assert.equal(keysLayers.runtimeClosure.length, expectedTimelineRuntimeOrder.length);
  for (let index = 0; index < expectedTimelineRuntimeOrder.length; index += 1) {
    const expectedPath = expectedTimelineRuntimeOrder[index];
    const entry = keysLayers.runtimeClosure[index];
    ensureExactKeys(entry, ["path", "role", "sha256"]);
    assert.equal(entry.path, expectedPath);
    assert.equal(entry.role, "runtime");
    assert.equal(entry.sha256, EXPECTED_TIMELINE_RUNTIME_HASHES[expectedPath]);
    assert.ok(!entry.path.includes("/legacy/") && !entry.path.includes("/archive/"));
    assert.equal(hashBytes(readBlobFromCommit(entry.path, fixedSourceCommit)), entry.sha256);
    assert.equal(hashBytes(await readFile(absoluteFromRelative(entry.path))), entry.sha256);
  }

  assert.equal(keysLayers.localImports.length, 1);
  ensureExactKeys(keysLayers.localImports[0], ["kind", "source", "specifiers"]);
  assert.equal(keysLayers.localImports[0].kind, "css-side-effect");
  assert.equal(keysLayers.localImports[0].source, EXPECTED_TIMELINE_CSS_SOURCE);
  assert.deepEqual(keysLayers.localImports[0].specifiers, []);

  const timelineAst = parseModule(timelineAstSource ?? await readFile(
    absoluteFromRelative(keysLayers.componentPath),
    "utf8",
  ));
  assert.ok(collectNamedExports(timelineAst).has("TimelineCandidate"));
  const timelineImports = collectCandidateImports(
    timelineAst,
    absoluteFromRelative(keysLayers.componentPath),
  );
  assert.deepEqual(timelineImports.externalPackages, ["react"]);
  assert.deepEqual(Object.keys(timelineImports.localImports), [EXPECTED_TIMELINE_CSS_SOURCE]);
  assert.equal(timelineImports.localImports[EXPECTED_TIMELINE_CSS_SOURCE].length, 0);
  assert.equal(countJsxClass(timelineAst, "candidate-key-tools"), 1);
  assert.equal(countJsxClass(timelineAst, "candidate-key-tools-open"), 1);

  const cssSource = timelineCssSource ?? await readFile(
    absoluteFromRelative(EXPECTED_TIMELINE_CSS_SOURCE),
    "utf8",
  );
  assert.ok(hasCssSelectorRoot(cssSource, ".candidate-key-tools"));
  assert.ok(hasCssSelectorRoot(cssSource, ".candidate-key-tools-open"));

  const timelineTestAst = parseModule(await readFile(
    absoluteFromRelative(keysLayers.behavioralEvidence.path),
    "utf8",
  ));
  assert.deepEqual(extractRouteFromTest(timelineTestAst), [EXPECTED_TEST_ROUTE]);

  const inspector = manifest.surfaces[3];
  ensureExactKeys(inspector, [
    "id",
    "classification",
    "sourceStatus",
    "promotionBoundary",
    "legacyOracleClosure",
    "legacyExport",
    "rejectedSkeleton",
    "requiredNextAction",
    "behavioralEvidence",
  ]);
  assert.equal(inspector.id, "inspector");
  assert.equal(inspector.classification, EXPECTED_INSPECTOR_CLASSIFICATION);
  assert.equal(inspector.sourceStatus, EXPECTED_INSPECTOR_SOURCE_STATUS);
  assert.deepEqual(inspector.promotionBoundary, []);
  assert.equal(inspector.legacyExport, EXPECTED_INSPECTOR_LEGACY_EXPORT);
  assert.equal(inspector.requiredNextAction, EXPECTED_INSPECTOR_NEXT_ACTION);
  assert.equal(inspector.legacyOracleClosure.length, EXPECTED_INSPECTOR_LEGACY_CLOSURE.length);
  for (let index = 0; index < EXPECTED_INSPECTOR_LEGACY_CLOSURE.length; index += 1) {
    const expectedPath = EXPECTED_INSPECTOR_LEGACY_CLOSURE[index];
    const entry = inspector.legacyOracleClosure[index];
    ensureExactKeys(entry, ["path", "role", "sha256"]);
    assert.equal(entry.path, expectedPath);
    assert.equal(entry.role, "oracle");
    assert.equal(entry.sha256, EXPECTED_INSPECTOR_LEGACY_HASHES[expectedPath]);
    assert.equal(hashBytes(readBlobFromCommit(entry.path, fixedSourceCommit)), entry.sha256);
  }

  ensureExactKeys(inspector.rejectedSkeleton, ["path", "export", "sha256", "disposition"]);
  assert.equal(inspector.rejectedSkeleton.path, EXPECTED_INSPECTOR_SKELETON_PATH);
  assert.equal(inspector.rejectedSkeleton.export, "InspectorSurface");
  assert.equal(inspector.rejectedSkeleton.sha256, EXPECTED_INSPECTOR_SKELETON_HASH);
  assert.equal(inspector.rejectedSkeleton.disposition, "reduced-skeleton-not-product-source");
  assert.equal(
    hashBytes(readBlobFromCommit(inspector.rejectedSkeleton.path, fixedSourceCommit)),
    inspector.rejectedSkeleton.sha256,
  );

  ensureExactKeys(inspector.behavioralEvidence, ["path", "reactRoute", "legacyOracleRoute"]);
  assert.equal(inspector.behavioralEvidence.path, EXPECTED_INSPECTOR_PARITY_PATH);
  assert.equal(inspector.behavioralEvidence.reactRoute, EXPECTED_INSPECTOR_REACT_ROUTE);
  assert.equal(inspector.behavioralEvidence.legacyOracleRoute, EXPECTED_INSPECTOR_LEGACY_ROUTE);

  const hostAst = parseModule(inspectorHostAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_INSPECTOR_LEGACY_CLOSURE[0]), "utf8",
  ));
  assert.ok(importsNamedExport(hostAst, "./LegacyRegions", EXPECTED_INSPECTOR_LEGACY_EXPORT));
  assert.ok(importsSource(hostAst, "./legacySource"));
  assert.ok(hasInspectorParserReplacement(hostAst));

  const regionsAst = parseModule(inspectorRegionsAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_INSPECTOR_LEGACY_CLOSURE[1]), "utf8",
  ));
  assert.ok(collectNamedExports(regionsAst).has(EXPECTED_INSPECTOR_LEGACY_EXPORT));

  const legacySourceAst = parseModule(inspectorLegacySourceAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_INSPECTOR_LEGACY_CLOSURE[2]), "utf8",
  ));
  assert.ok(importsSource(legacySourceAst, "../../../mocks/m3-vism-host-boundary.html?raw"));

  const skeletonAst = parseModule(inspectorSkeletonAstSource ?? await readFile(
    absoluteFromRelative(inspector.rejectedSkeleton.path), "utf8",
  ));
  assert.ok(collectNamedExports(skeletonAst).has(inspector.rejectedSkeleton.export));

  const parityAst = parseModule(inspectorParityAstSource ?? await readFile(
    absoluteFromRelative(inspector.behavioralEvidence.path), "utf8",
  ));
  const parityRoutes = extractRouteFromTest(parityAst);
  assert.ok(parityRoutes.includes(inspector.behavioralEvidence.reactRoute));
  assert.ok(parityRoutes.includes(inspector.behavioralEvidence.legacyOracleRoute));

  const panelLayout = manifest.surfaces[4];
  ensureExactKeys(panelLayout, [
    "id",
    "classification",
    "containerPath",
    "runtimeClosure",
    "namedExports",
    "localImports",
    "externalPackages",
    "promotionBoundary",
    "excludedBoundary",
    "stateOwner",
    "behavioralEvidence",
  ]);
  assert.equal(panelLayout.id, "resizable-panel-layout");
  assert.equal(panelLayout.classification, "react-layout-subtree-candidate-legacy-adapters-excluded");
  assert.equal(panelLayout.containerPath, EXPECTED_PANEL_LAYOUT_SOURCE);
  assert.deepEqual(panelLayout.namedExports, EXPECTED_PANEL_LAYOUT_EXPORTS);
  assert.deepEqual(panelLayout.externalPackages, EXPECTED_EXTERNAL_PACKAGES);
  assert.deepEqual(panelLayout.promotionBoundary, EXPECTED_PANEL_LAYOUT_PROMOTION_BOUNDARY);
  assert.deepEqual(panelLayout.excludedBoundary, EXPECTED_PANEL_LAYOUT_EXCLUDED_BOUNDARY);
  assert.equal(panelLayout.stateOwner, "local-presentation-oracle");

  const expectedPanelLayoutRuntimeOrder = [EXPECTED_PANEL_LAYOUT_SOURCE, EXPECTED_PANEL_LAYOUT_CSS_SOURCE];
  assert.equal(panelLayout.runtimeClosure.length, expectedPanelLayoutRuntimeOrder.length);
  for (let index = 0; index < expectedPanelLayoutRuntimeOrder.length; index += 1) {
    const expectedPath = expectedPanelLayoutRuntimeOrder[index];
    const entry = panelLayout.runtimeClosure[index];
    ensureExactKeys(entry, ["path", "role", "sha256"]);
    assert.equal(entry.path, expectedPath);
    assert.equal(entry.role, "runtime");
    assert.equal(entry.sha256, EXPECTED_PANEL_LAYOUT_RUNTIME_HASHES[expectedPath]);
    assert.equal(hashBytes(readBlobFromCommit(entry.path, fixedSourceCommit)), entry.sha256);
    assert.equal(hashBytes(await readFile(absoluteFromRelative(entry.path))), entry.sha256);
  }

  assert.equal(panelLayout.localImports.length, 1);
  ensureExactKeys(panelLayout.localImports[0], ["kind", "source", "specifiers"]);
  assert.equal(panelLayout.localImports[0].kind, "css-side-effect");
  assert.equal(panelLayout.localImports[0].source, EXPECTED_PANEL_LAYOUT_CSS_SOURCE);
  assert.deepEqual(panelLayout.localImports[0].specifiers, []);

  ensureExactKeys(panelLayout.behavioralEvidence, ["path", "route", "sha256"]);
  assert.equal(panelLayout.behavioralEvidence.path, EXPECTED_PANEL_LAYOUT_TEST_PATH);
  assert.equal(panelLayout.behavioralEvidence.route, EXPECTED_TEST_ROUTE);
  assert.equal(panelLayout.behavioralEvidence.sha256, EXPECTED_PANEL_LAYOUT_TEST_HASH);
  assert.equal(
    hashBytes(readBlobFromCommit(panelLayout.behavioralEvidence.path, fixedSourceCommit)),
    panelLayout.behavioralEvidence.sha256,
  );

  const panelLayoutAst = parseModule(panelLayoutAstSource ?? await readFile(
    absoluteFromRelative(panelLayout.containerPath), "utf8",
  ));
  assert.deepEqual([...collectNamedExports(panelLayoutAst)].sort(), [...EXPECTED_PANEL_LAYOUT_EXPORTS].sort());
  const panelLayoutImports = collectCandidateImports(
    panelLayoutAst,
    absoluteFromRelative(panelLayout.containerPath),
  );
  assert.deepEqual(panelLayoutImports.externalPackages, EXPECTED_EXTERNAL_PACKAGES);
  assert.deepEqual(Object.keys(panelLayoutImports.localImports), [EXPECTED_PANEL_LAYOUT_CSS_SOURCE]);
  assert.equal(panelLayoutImports.localImports[EXPECTED_PANEL_LAYOUT_CSS_SOURCE].length, 0);
  assert.ok(hasTopLevelVariable(panelLayoutAst, "PanelLayoutContext"));
  assert.ok(hasFunctionDeclaration(panelLayoutAst, "PanelSeparator"));
  for (const panel of ["browser", "inspector", "timeline"]) {
    assert.ok(hasPanelSpec(panelLayoutAst, panel));
  }

  const panelCss = panelLayoutCssSource ?? await readFile(
    absoluteFromRelative(EXPECTED_PANEL_LAYOUT_CSS_SOURCE), "utf8",
  );
  assert.ok(hasCssSelectorRoot(panelCss, '.app[data-resizable-layout="true"]'));
  assert.ok(hasCssSelectorRoot(panelCss, ".react-panel-separator"));

  const panelLayoutTestAst = parseModule(panelLayoutTestAstSource ?? await readFile(
    absoluteFromRelative(panelLayout.behavioralEvidence.path), "utf8",
  ));
  assert.ok(extractRouteFromTest(panelLayoutTestAst).includes(EXPECTED_TEST_ROUTE));

  const nativeStageTime = manifest.surfaces[5];
  ensureExactKeys(nativeStageTime, [
    "id",
    "classification",
    "nativeOwner",
    "reactPromotionBoundary",
    "oracleClosure",
    "stageOracle",
    "timelineOracle",
    "rejectedReactSurfaces",
    "behavioralEvidence",
  ]);
  assert.equal(nativeStageTime.id, "native-stage-time-surface");
  assert.equal(nativeStageTime.classification, "native-wgpu-owned-react-promotion-forbidden");
  assert.equal(nativeStageTime.nativeOwner, "wgpu-stage-and-timeline");
  assert.deepEqual(nativeStageTime.reactPromotionBoundary, []);
  assert.deepEqual(nativeStageTime.stageOracle, EXPECTED_STAGE_ORACLE);
  assert.deepEqual(nativeStageTime.timelineOracle, EXPECTED_NATIVE_TIMELINE_ORACLE);
  assert.equal(nativeStageTime.oracleClosure.length, EXPECTED_NATIVE_STAGE_TIME_CLOSURE.length);
  for (let index = 0; index < EXPECTED_NATIVE_STAGE_TIME_CLOSURE.length; index += 1) {
    const expectedPath = EXPECTED_NATIVE_STAGE_TIME_CLOSURE[index];
    const entry = nativeStageTime.oracleClosure[index];
    ensureExactKeys(entry, ["path", "role", "sha256"]);
    assert.equal(entry.path, expectedPath);
    assert.equal(entry.role, index < 4 ? "legacy-oracle" : "react-oracle");
    assert.equal(entry.sha256, EXPECTED_NATIVE_STAGE_TIME_HASHES[expectedPath]);
    assert.equal(hashBytes(readBlobFromCommit(entry.path, fixedSourceCommit)), entry.sha256);
    assert.equal(hashBytes(await readFile(absoluteFromRelative(entry.path))), entry.sha256);
  }

  assert.equal(nativeStageTime.rejectedReactSurfaces.length, 2);
  const [rejectedStage, rejectedTimeline] = nativeStageTime.rejectedReactSurfaces;
  ensureExactKeys(rejectedStage, ["path", "export", "sha256", "disposition"]);
  assert.deepEqual(rejectedStage, {
    path: EXPECTED_STAGE_SURFACE_PATH,
    export: "StageSurface",
    sha256: EXPECTED_STAGE_SURFACE_HASH,
    disposition: "reference-surface-not-product-stage",
  });
  ensureExactKeys(rejectedTimeline, ["path", "export", "sha256", "disposition"]);
  assert.deepEqual(rejectedTimeline, {
    path: EXPECTED_TIMELINE_SOURCE,
    export: "TimelineCandidate",
    sha256: EXPECTED_TIMELINE_RUNTIME_HASHES[EXPECTED_TIMELINE_SOURCE],
    disposition: "whole-container-not-product-timeline",
  });
  for (const rejected of nativeStageTime.rejectedReactSurfaces) {
    assert.equal(hashBytes(readBlobFromCommit(rejected.path, fixedSourceCommit)), rejected.sha256);
  }

  assert.equal(nativeStageTime.behavioralEvidence.length, 2);
  const [nativeParityEvidence, nativeTimelineEvidence] = nativeStageTime.behavioralEvidence;
  ensureExactKeys(nativeParityEvidence, ["path", "reactRoute", "legacyOracleRoute", "sha256"]);
  assert.deepEqual(nativeParityEvidence, {
    path: EXPECTED_INSPECTOR_PARITY_PATH,
    reactRoute: EXPECTED_INSPECTOR_REACT_ROUTE,
    legacyOracleRoute: EXPECTED_INSPECTOR_LEGACY_ROUTE,
    sha256: EXPECTED_NATIVE_VISUAL_PARITY_HASH,
  });
  ensureExactKeys(nativeTimelineEvidence, ["path", "route", "sha256"]);
  assert.deepEqual(nativeTimelineEvidence, {
    path: EXPECTED_TIMELINE_TEST_PATH,
    route: EXPECTED_TEST_ROUTE,
    sha256: EXPECTED_NATIVE_TIMELINE_TEST_HASH,
  });
  for (const evidence of nativeStageTime.behavioralEvidence) {
    assert.equal(hashBytes(readBlobFromCommit(evidence.path, fixedSourceCommit)), evidence.sha256);
  }

  const stageHostAst = parseModule(stageHostAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[0]), "utf8",
  ));
  assert.ok(importsNamedExport(stageHostAst, "./LegacyRegions", "LegacyStageShell"));
  assert.ok(importsSource(stageHostAst, "./legacySource"));
  assert.ok(hasStageParserReplacement(stageHostAst));
  const stageRegionsAst = parseModule(stageRegionsAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[1]), "utf8",
  ));
  assert.ok(collectNamedExports(stageRegionsAst).has("LegacyStageShell"));
  const stageLegacySourceAst = parseModule(stageLegacySourceAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[2]), "utf8",
  ));
  assert.ok(importsSource(stageLegacySourceAst, "../../../mocks/m3-vism-host-boundary.html?raw"));
  const rawHtml = stageRawHtmlSource ?? await readFile(
    absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[3]), "utf8",
  );
  const stageShell = findHtmlClass({ children: htmlToDOM(rawHtml) }, "stage-shell");
  assert.ok(stageShell);
  assert.ok(findHtmlClass(stageShell, "stage"));
  assert.ok(findHtmlClass(stageShell, "frame"));
  assert.ok(findHtmlClass(stageShell, "transport"));

  const nativeTimelineAst = parseModule(nativeTimelineAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_TIMELINE_SOURCE), "utf8",
  ));
  for (const anchor of ["candidate-beat-ruler", "candidate-band-action-rail", "candidate-time-bar", "candidate-automation-key", "candidate-playhead", "candidate-pack-plane", "candidate-timeline-body"]) {
    assert.ok(countJsxClass(nativeTimelineAst, anchor) > 0);
  }
  assert.ok(hasJsxIdentifier(nativeTimelineAst, "GraphViewComponent"));
  const nativeCss = nativeTimelineCssSource ?? await readFile(
    absoluteFromRelative(EXPECTED_TIMELINE_CSS_SOURCE), "utf8",
  );
  for (const anchor of ["candidate-beat-ruler", "candidate-band-action-rail", "candidate-time-bar", "candidate-automation-key", "candidate-playhead", "candidate-pack-plane", "candidate-timeline-body"]) {
    assert.ok(hasCssClassToken(nativeCss, anchor));
  }
  const stageSurfaceAst = parseModule(stageSurfaceAstSource ?? await readFile(
    absoluteFromRelative(EXPECTED_STAGE_SURFACE_PATH), "utf8",
  ));
  assert.ok(collectNamedExports(stageSurfaceAst).has(rejectedStage.export));
  const nativeParityAst = parseModule(nativeVisualParityAstSource ?? await readFile(
    absoluteFromRelative(nativeParityEvidence.path), "utf8",
  ));
  const nativeParityRoutes = extractRouteFromTest(nativeParityAst);
  assert.ok(nativeParityRoutes.includes(nativeParityEvidence.reactRoute));
  assert.ok(nativeParityRoutes.includes(nativeParityEvidence.legacyOracleRoute));
  const nativeTimelineTestAst = parseModule(nativeTimelineTestAstSource ?? await readFile(
    absoluteFromRelative(nativeTimelineEvidence.path), "utf8",
  ));
  assert.ok(extractRouteFromTest(nativeTimelineTestAst).includes(nativeTimelineEvidence.route));
}

test("accepts exact incomplete multi-surface R0 manifest and fixed-commit evidence", async () => {
  const manifest = await manifestFromDisk();
  await validateInventory(manifest);
});

test("rejects unknown top-level manifest keys", async () => {
  const manifest = await manifestFromDisk();
  const mutated = {
    ...manifest,
    extraGate: true,
  };
  await assert.rejects(async () => {
    await validateInventory(mutated);
  });
});

test("rejects wrong fixed source commit", async () => {
  const manifest = await manifestFromDisk();
  const mutated = {
    ...manifest,
    fixedSourceCommit: "0000000000000000000000000000000000000000",
  };
  await assert.rejects(async () => {
    await validateInventory(mutated);
  });
});

test("rejects non-empty model closure", async () => {
  const manifest = await manifestFromDisk();
  const mutated = {
    ...manifest,
    modelClosure: ["docs/mocks-ui/src/patterns/DiscoveryBrowser.jsx"],
  };
  await assert.rejects(async () => {
    await validateInventory(mutated);
  });
});

test("rejects runtime closure reorder, missing, and hash mismatch", async () => {
  const manifest = await manifestFromDisk();

  const reordered = withInventoryEntryAt(manifest, "surfaces", 0, (browser) => ({
    ...browser,
    runtimeClosure: [
      browser.runtimeClosure[0],
      browser.runtimeClosure[2],
      browser.runtimeClosure[1],
    ],
  }));
  await assert.rejects(async () => {
    await validateInventory(reordered);
  });

  const missing = withInventoryEntryAt(manifest, "surfaces", 0, (browser) => ({
    ...browser,
    runtimeClosure: browser.runtimeClosure.slice(0, 2),
  }));
  await assert.rejects(async () => {
    await validateInventory(missing);
  });

  const hashMismatch = withInventoryEntryAt(manifest, "surfaces", 0, (browser) => ({
      ...browser,
      runtimeClosure: browser.runtimeClosure.map((entry, index) =>
        index === 1 ? { ...entry, sha256: "0".repeat(64) } : entry,
      ),
    }));
  await assert.rejects(async () => {
    await validateInventory(hashMismatch);
  });
});

test("rejects extra runtime closure entry", async () => {
  const manifest = await manifestFromDisk();
  const extra = withInventoryEntryAt(manifest, "surfaces", 0, (browser) => ({
      ...browser,
      runtimeClosure: [
        ...browser.runtimeClosure,
        {
          path: EXPECTED_PATTERN_SOURCE,
          role: "runtime",
          sha256: EXPECTED_RUNTIME_HASHES[EXPECTED_PATTERN_SOURCE],
        },
      ],
    }));
  await assert.rejects(async () => {
    await validateInventory(extra);
  });
});

test("rejects missing or wrong component export and non-browser surface", async () => {
  const manifest = await manifestFromDisk();

  const wrongExport = withInventoryEntryAt(manifest, "surfaces", 0, (browser) => ({
      ...browser,
      componentExport: "BrowserCandidate",
    }));
  await assert.rejects(async () => {
    await validateInventory(wrongExport);
  });

  const nonBrowserSurface = withInventoryEntryAt(manifest, "surfaces", 0, (browser) => ({
      ...browser,
      id: "inspector",
    }));
  await assert.rejects(async () => {
    await validateInventory(nonBrowserSurface);
  });
});

test("rejects unexpected local imports outside declared runtime closure", async () => {
  const manifest = await manifestFromDisk();
  const source = await readFile(
    absoluteFromRelative(manifest.surfaces[0].componentPath),
    "utf8",
  );
  const injected = `${source}\nimport { load } from "../legacy/legacySource.js";\n`;
  await assert.rejects(async () => {
    await validateInventory(manifest, { candidateAstSource: injected });
  });
});

test("rejects legacy/archive paths promoted in runtime closure", async () => {
  const manifest = await manifestFromDisk();
  const withLegacy = withInventoryEntryAt(manifest, "surfaces", 0, (browser) => ({
      ...browser,
      runtimeClosure: browser.runtimeClosure.map((entry) =>
        entry.path === EXPECTED_PATTERN_SOURCE
          ? {
            ...entry,
            path: "docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx",
            sha256: "0".repeat(64),
          }
          : entry,
      ),
    }));
  await assert.rejects(async () => {
    await validateInventory(withLegacy);
  });
});

test("rejects missing or wrong test evidence route", async () => {
  const manifest = await manifestFromDisk();

  const missingPath = {
    ...manifest,
    ...withInventoryEntryAt(manifest, "behavioralTests", 0, (evidence) => ({
      ...evidence,
      path: "docs/mocks-ui/tests/browser-candidate.spec.missing.js",
    })),
  };
  await assert.rejects(async () => {
    await validateInventory(missingPath);
  });

  const wrongRoute = {
    ...manifest,
    ...withInventoryEntryAt(manifest, "behavioralTests", 0, (evidence) => ({
      ...evidence,
      route: "plugin-browser",
    })),
  };
  await assert.rejects(async () => {
    await validateInventory(wrongRoute);
  });
});

test("rejects complete R0, wrong Easing classification, and changed Easing packages", async () => {
  const manifest = await manifestFromDisk();

  const complete = { ...manifest, completeR0: true };
  await assert.rejects(async () => {
    await validateInventory(complete);
  });

  const wrongClassification = withInventoryEntryAt(manifest, "surfaces", 1, (easing) => ({
    ...easing,
    classification: "react-direct-promotion",
  }));
  await assert.rejects(async () => {
    await validateInventory(wrongClassification);
  });

  const changedPackages = withInventoryEntryAt(manifest, "surfaces", 1, (easing) => ({
    ...easing,
    externalPackages: ["react", "react-dom"],
  }));
  await assert.rejects(async () => {
    await validateInventory(changedPackages);
  });
});

test("rejects Easing closure, ownership split, evidence, and extra local dependency", async () => {
  const manifest = await manifestFromDisk();

  const wrongExport = withInventoryEntryAt(manifest, "surfaces", 1, (easing) => ({
    ...easing,
    componentExport: "EasingCandidate",
  }));
  await assert.rejects(async () => {
    await validateInventory(wrongExport);
  });

  const wrongHash = withInventoryEntryAt(manifest, "surfaces", 1, (easing) => ({
        ...easing,
        runtimeClosure: easing.runtimeClosure.map((entry, index) =>
          index === 2 ? { ...entry, sha256: "0".repeat(64) } : entry,
        ),
      }));
  await assert.rejects(async () => {
    await validateInventory(wrongHash);
  });

  const missingModel = withInventoryEntryAt(manifest, "surfaces", 1, (easing) => ({
    ...easing,
    runtimeClosure: easing.runtimeClosure.slice(0, 2),
  }));
  await assert.rejects(async () => {
    await validateInventory(missingModel);
  });

  const promotedPopup = withInventoryEntryAt(manifest, "surfaces", 1, (easing) => ({
    ...easing,
    promotionBoundary: [...easing.promotionBoundary, "popup frame"],
  }));
  await assert.rejects(async () => {
    await validateInventory(promotedPopup);
  });

  const missingEvidence = withInventoryEntryAt(manifest, "surfaces", 1, (easing) => ({
    ...easing,
    behavioralEvidence: { ...easing.behavioralEvidence, route: "archive/easing" },
  }));
  await assert.rejects(async () => {
    await validateInventory(missingEvidence);
  });

  const source = await readFile(absoluteFromRelative(EXPECTED_EASING_SOURCE), "utf8");
  const injected = `${source}\nimport { load } from "../legacy/legacySource.js";\n`;
  await assert.rejects(async () => {
    await validateInventory(manifest, { easingAstSource: injected });
  });
});

test("rejects KEYS/LAYERS promotion beyond the fixed Timeline subtree evidence", async () => {
  const manifest = await manifestFromDisk();

  const wrongBoundary = withInventoryEntryAt(manifest, "surfaces", 2, (keysLayers) => ({
    ...keysLayers,
    promotionBoundary: [...keysLayers.promotionBoundary, "ruler"],
  }));
  await assert.rejects(async () => {
    await validateInventory(wrongBoundary);
  });

  const wrongNativeOracle = withInventoryEntryAt(manifest, "surfaces", 2, (keysLayers) => ({
    ...keysLayers,
    nativeOracle: keysLayers.nativeOracle.filter((entry) => entry !== "playhead"),
  }));
  await assert.rejects(async () => {
    await validateInventory(wrongNativeOracle);
  });

  const wrongModes = withInventoryEntryAt(manifest, "surfaces", 2, (keysLayers) => ({
    ...keysLayers,
    modes: { ...keysLayers.modes, keys: ["align", "stagger"] },
  }));
  await assert.rejects(async () => {
    await validateInventory(wrongModes);
  });

  const wrongEvidence = withInventoryEntryAt(manifest, "surfaces", 2, (keysLayers) => ({
    ...keysLayers,
    behavioralEvidence: { ...keysLayers.behavioralEvidence, path: EXPECTED_TEST_PATH },
  }));
  await assert.rejects(async () => {
    await validateInventory(wrongEvidence);
  });

  const source = await readFile(absoluteFromRelative(EXPECTED_TIMELINE_SOURCE), "utf8");
  const withoutReopenControl = source.replace('className="candidate-key-tools-open"', 'className="candidate-key-tools-closed"');
  await assert.rejects(async () => {
    await validateInventory(manifest, { timelineAstSource: withoutReopenControl });
  });

  const css = await readFile(absoluteFromRelative(EXPECTED_TIMELINE_CSS_SOURCE), "utf8");
  const withoutToolPanelSelector = css.replaceAll(/\.candidate-key-tools(?!-open)/g, ".candidate-key-panel");
  await assert.rejects(async () => {
    await validateInventory(manifest, { timelineCssSource: withoutToolPanelSelector });
  });
});

test("rejects Inspector promotion, legacy oracle, skeleton, action, and parity route drift", async () => {
  const manifest = await manifestFromDisk();
  const mutateInspector = (patch) =>
    withInventoryEntryAt(manifest, "surfaces", 3, patch);

  for (const mutated of [
    mutateInspector((inspector) => ({ ...inspector, promotionBoundary: ["InspectorSurface"] })),
    mutateInspector((inspector) => ({ ...inspector, legacyOracleClosure: inspector.legacyOracleClosure.slice(0, 3) })),
    mutateInspector((inspector) => ({
      ...inspector,
      legacyOracleClosure: [
        inspector.legacyOracleClosure[1],
        inspector.legacyOracleClosure[0],
        ...inspector.legacyOracleClosure.slice(2),
      ],
    })),
    mutateInspector((inspector) => ({
      ...inspector,
      legacyOracleClosure: inspector.legacyOracleClosure.map((entry, index) =>
        index === 1 ? { ...entry, sha256: "0".repeat(64) } : entry,
      ),
    })),
    mutateInspector((inspector) => ({ ...inspector, legacyExport: "LegacyTimeline" })),
    mutateInspector((inspector) => ({
      ...inspector,
      rejectedSkeleton: { ...inspector.rejectedSkeleton, export: "LegacyInspector" },
    })),
    mutateInspector((inspector) => ({
      ...inspector,
      rejectedSkeleton: {
        ...inspector.rejectedSkeleton,
        path: "docs/mocks-ui/src/legacy/LegacyRegions.jsx",
      },
    })),
    mutateInspector((inspector) => ({
      ...inspector,
      rejectedSkeleton: {
        ...inspector.rejectedSkeleton,
        disposition: "promotion-candidate",
      },
    })),
    mutateInspector((inspector) => ({
      ...inspector,
      requiredNextAction: "promote-inspector-skeleton",
    })),
    mutateInspector((inspector) => ({
      ...inspector,
      behavioralEvidence: { ...inspector.behavioralEvidence, reactRoute: "inspector" },
    })),
    mutateInspector((inspector) => ({
      ...inspector,
      behavioralEvidence: { ...inspector.behavioralEvidence, legacyOracleRoute: "archive/all-surfaces" },
    })),
  ]) {
    await assert.rejects(async () => {
      await validateInventory(mutated);
    });
  }
});

test("rejects Inspector parser/export/raw-source/skeleton/parity evidence drift", async () => {
  const manifest = await manifestFromDisk();
  const host = await readFile(absoluteFromRelative(EXPECTED_INSPECTOR_LEGACY_CLOSURE[0]), "utf8");
  const regions = await readFile(absoluteFromRelative(EXPECTED_INSPECTOR_LEGACY_CLOSURE[1]), "utf8");
  const legacySource = await readFile(absoluteFromRelative(EXPECTED_INSPECTOR_LEGACY_CLOSURE[2]), "utf8");
  const skeleton = await readFile(absoluteFromRelative(EXPECTED_INSPECTOR_SKELETON_PATH), "utf8");
  const parity = await readFile(absoluteFromRelative(EXPECTED_INSPECTOR_PARITY_PATH), "utf8");

  for (const options of [
    { inspectorHostAstSource: host.replace("<LegacyInspector {...props} />", "<LegacyTimeline {...props} />") },
    { inspectorRegionsAstSource: regions.replace("LegacyInspector", "ArchivedInspector") },
    { inspectorLegacySourceAstSource: legacySource.replace("m3-vism-host-boundary.html?raw", "missing.html?raw") },
    { inspectorSkeletonAstSource: skeleton.replace("InspectorSurface", "ArchivedInspectorSurface") },
    { inspectorParityAstSource: parity.replace("#archive/all-surfaces", "#archive/inspector") },
  ]) {
    await assert.rejects(async () => {
      await validateInventory(manifest, options);
    });
  }
});

test("rejects resizable panel layout classification, closure, boundary, state, and evidence drift", async () => {
  const manifest = await manifestFromDisk();
  const mutatePanelLayout = (patch) =>
    withInventoryEntryAt(manifest, "surfaces", 4, patch);

  for (const mutated of [
    mutatePanelLayout((panelLayout) => ({ ...panelLayout, classification: "react-direct-promotion" })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      runtimeClosure: [panelLayout.runtimeClosure[1], panelLayout.runtimeClosure[0]],
    })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      runtimeClosure: panelLayout.runtimeClosure.map((entry, index) =>
        index === 1 ? { ...entry, sha256: "0".repeat(64) } : entry,
      ),
    })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      namedExports: panelLayout.namedExports.slice(0, 3),
    })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      externalPackages: ["react"],
    })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      promotionBoundary: ["PanelSeparator"],
    })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      excludedBoundary: panelLayout.excludedBoundary.slice(0, 3),
    })),
    mutatePanelLayout((panelLayout) => ({ ...panelLayout, stateOwner: "workspace-profile" })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      behavioralEvidence: { ...panelLayout.behavioralEvidence, sha256: "0".repeat(64) },
    })),
    mutatePanelLayout((panelLayout) => ({
      ...panelLayout,
      behavioralEvidence: { ...panelLayout.behavioralEvidence, route: "archive/all-surfaces" },
    })),
  ]) {
    await assert.rejects(async () => {
      await validateInventory(mutated);
    });
  }
});

test("rejects resizable panel layout AST, CSS selector, and route drift", async () => {
  const manifest = await manifestFromDisk();
  const source = await readFile(absoluteFromRelative(EXPECTED_PANEL_LAYOUT_SOURCE), "utf8");
  const css = await readFile(absoluteFromRelative(EXPECTED_PANEL_LAYOUT_CSS_SOURCE), "utf8");
  const panelTest = await readFile(absoluteFromRelative(EXPECTED_PANEL_LAYOUT_TEST_PATH), "utf8");

  for (const options of [
    { panelLayoutAstSource: source.replace("PanelLayoutContext", "PanelContext") },
    { panelLayoutAstSource: source.replace("function PanelSeparator", "function PanelResizeHandle") },
    { panelLayoutAstSource: source.replace("const PANEL_SPEC", "const PANEL_SIZES") },
    { panelLayoutAstSource: source.replace('import "./resizable-panel-layout.css";', "") },
    { panelLayoutCssSource: css.replaceAll('.app[data-resizable-layout="true"]', ".app") },
    { panelLayoutCssSource: css.replaceAll(".react-panel-separator", ".react-panel-handle") },
    { panelLayoutTestAstSource: panelTest.replaceAll("#plugin-browser-candidate", "#archive/all-surfaces") },
  ]) {
    await assert.rejects(async () => {
      await validateInventory(manifest, options);
    });
  }
});

test("rejects native Stage/time ownership, closure, oracle, rejected-surface, and evidence drift", async () => {
  const manifest = await manifestFromDisk();
  const mutateNativeStageTime = (patch) =>
    withInventoryEntryAt(manifest, "surfaces", 5, patch);

  for (const mutated of [
    mutateNativeStageTime((surface) => ({ ...surface, classification: "react-direct-promotion" })),
    mutateNativeStageTime((surface) => ({ ...surface, nativeOwner: "react-stage-and-timeline" })),
    mutateNativeStageTime((surface) => ({ ...surface, reactPromotionBoundary: ["timeline"] })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      oracleClosure: [surface.oracleClosure[1], surface.oracleClosure[0], ...surface.oracleClosure.slice(2)],
    })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      oracleClosure: surface.oracleClosure.map((entry, index) =>
        index === 4 ? { ...entry, role: "legacy-oracle" } : entry,
      ),
    })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      oracleClosure: surface.oracleClosure.map((entry, index) =>
        index === 5 ? { ...entry, sha256: "0".repeat(64) } : entry,
      ),
    })),
    mutateNativeStageTime((surface) => ({ ...surface, stageOracle: surface.stageOracle.slice(0, 2) })),
    mutateNativeStageTime((surface) => ({ ...surface, timelineOracle: surface.timelineOracle.slice(0, 7) })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      rejectedReactSurfaces: surface.rejectedReactSurfaces.map((entry, index) =>
        index === 0 ? { ...entry, export: "LegacyStageShell" } : entry,
      ),
    })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      rejectedReactSurfaces: surface.rejectedReactSurfaces.map((entry, index) =>
        index === 1 ? { ...entry, sha256: "0".repeat(64) } : entry,
      ),
    })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      rejectedReactSurfaces: surface.rejectedReactSurfaces.map((entry, index) =>
        index === 1 ? { ...entry, disposition: "react-timeline-owner" } : entry,
      ),
    })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      behavioralEvidence: surface.behavioralEvidence.map((entry, index) =>
        index === 0 ? { ...entry, sha256: "0".repeat(64) } : entry,
      ),
    })),
    mutateNativeStageTime((surface) => ({
      ...surface,
      behavioralEvidence: surface.behavioralEvidence.map((entry, index) =>
        index === 1 ? { ...entry, route: "archive/all-surfaces" } : entry,
      ),
    })),
  ]) {
    await assert.rejects(async () => {
      await validateInventory(mutated);
    });
  }
});

test("rejects native Stage/time structured source anchors and key-glyph drift", async () => {
  const manifest = await manifestFromDisk();
  const host = await readFile(absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[0]), "utf8");
  const regions = await readFile(absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[1]), "utf8");
  const legacySource = await readFile(absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[2]), "utf8");
  const rawHtml = await readFile(absoluteFromRelative(EXPECTED_NATIVE_STAGE_TIME_CLOSURE[3]), "utf8");
  const timeline = await readFile(absoluteFromRelative(EXPECTED_TIMELINE_SOURCE), "utf8");
  const css = await readFile(absoluteFromRelative(EXPECTED_TIMELINE_CSS_SOURCE), "utf8");
  const parity = await readFile(absoluteFromRelative(EXPECTED_INSPECTOR_PARITY_PATH), "utf8");
  const timelineTest = await readFile(absoluteFromRelative(EXPECTED_TIMELINE_TEST_PATH), "utf8");

  for (const options of [
    { stageHostAstSource: host.replace("<LegacyStageShell {...props} />", "<LegacyTimeline {...props} />") },
    { stageRegionsAstSource: regions.replace("LegacyStageShell", "ArchivedStageShell") },
    { stageLegacySourceAstSource: legacySource.replace("m3-vism-host-boundary.html?raw", "missing.html?raw") },
    { stageRawHtmlSource: rawHtml.replace('class="stage-shell"', 'class="stage-panel"') },
    { nativeTimelineAstSource: timeline.replace('className="candidate-automation-key"', 'className="candidate-automation-point"') },
    { nativeTimelineCssSource: css.replaceAll(".candidate-automation-key", ".candidate-automation-point") },
    { nativeVisualParityAstSource: parity.replace("#archive/all-surfaces", "#archive/stage") },
    { nativeTimelineTestAstSource: timelineTest.replaceAll("#plugin-browser-candidate", "#archive/timeline") },
  ]) {
    await assert.rejects(async () => {
      await validateInventory(manifest, options);
    });
  }
});

test("native key-glyph validation is independent from the KEYS/LAYERS tool-panel validator", async () => {
  const manifest = await manifestFromDisk();
  const timeline = await readFile(absoluteFromRelative(EXPECTED_TIMELINE_SOURCE), "utf8");
  const css = await readFile(absoluteFromRelative(EXPECTED_TIMELINE_CSS_SOURCE), "utf8");
  await validateInventory(manifest, {
    nativeTimelineAstSource: timeline.replaceAll("candidate-key-tools", "candidate-tool-panel"),
    nativeTimelineCssSource: css.replaceAll("candidate-key-tools", "candidate-tool-panel"),
  });
});
