import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";

const traverse = traverseModule.default;

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = join(root, "..", "..");

const ARCHIVED_HTML = join(
  repoRoot,
  "docs/mocks/m3-vism-host-boundary.html",
);
const LEGACY_HOST = join(
  root,
  "src/legacy/LegacyHostBoundaryScreen.jsx",
);
const LEGACY_REGIONS = join(root, "src/legacy/LegacyRegions.jsx");
const INSPECTOR_CANDIDATE = join(
  root,
  "src/candidates/InspectorCandidate.jsx",
);
const INSPECTOR_CSS = join(
  root,
  "src/candidates/inspector-candidate.css",
);

const INSPECTOR_ANCHOR_I1 =
  '$("#inspector").innerHTML=inspector(mode); clearUndo(); bindInspector();';
const INSPECTOR_ANCHOR_I2 =
  'if(chip){chip.style.setProperty("--chip",state.selectedColor);chip.dataset.label=state.selectedColor}';

const INSPECTOR_REPLACEMENT_I1 =
  'document.createElement("aside").innerHTML=inspector(mode); clearUndo();window[Symbol.for("motolii.legacyHostBoundary.inspector")]?.publish({mode,state,setUndo,status,projectAutomation,applyScrubValue,setSurface,syncColorBook,setStageTool,renderPluginHistory,setMode});';
const INSPECTOR_REPLACEMENT_I2 =
  'if(chip)window[Symbol.for("motolii.legacyHostBoundary.inspector")]?.refresh()';

function countNonOverlapping(text, needle) {
  let count = 0;
  let index = text.indexOf(needle);
  while (index !== -1) {
    count += 1;
    index = text.indexOf(needle, index + needle.length);
  }
  return count;
}

function extractLegacyScript(html) {
  const match = html.match(/<script>([\s\S]*?)<\/script>\s*<\/body>/i);
  assert.ok(match, "archived script missing");
  return match[1];
}

function extractLegacyStyle(html) {
  const match = html.match(/<style>([\s\S]*?)<\/style>/i);
  assert.ok(match, "archived style missing");
  return match[1];
}

function applyInspectorContainment(script) {
  return script
    .replace(INSPECTOR_ANCHOR_I1, INSPECTOR_REPLACEMENT_I1)
    .replace(INSPECTOR_ANCHOR_I2, INSPECTOR_REPLACEMENT_I2);
}

function parseModule(path) {
  return parse(readFileSync(path, "utf8"), {
    sourceType: "module",
    plugins: ["jsx"],
  });
}

function collectNamedExports(ast) {
  const names = new Set();
  traverse(ast, {
    ExportNamedDeclaration(nodePath) {
      if (nodePath.node.declaration?.type === "FunctionDeclaration") {
        names.add(nodePath.node.declaration.id.name);
      }
      for (const specifier of nodePath.node.specifiers ?? []) {
        if (specifier.exported.type === "Identifier") {
          names.add(specifier.exported.name);
        }
      }
    },
  });
  return names;
}

function hasInspectorParserReplacement(ast) {
  const contains = (value, predicate) => {
    if (!value || typeof value !== "object") return false;
    if (predicate(value)) return true;
    if (Array.isArray(value)) {
      return value.some((entry) => contains(entry, predicate));
    }
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
          node.name.name === "LegacyInspector",
      );
    }
    Object.values(value).forEach(visit);
  };
  visit(ast);
  return replacementFound;
}

function parseInspectorCandidateCss(cssText) {
  return cssText
    .trim()
    .split("}")
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => `${part}}`);
}

function validateInspectorCandidateCss(cssText, archivedStyle) {
  const rules = parseInspectorCandidateCss(cssText);
  for (const rule of rules) {
    if (!archivedStyle.includes(rule)) {
      throw new Error(`missing verbatim rule ${rule}`);
    }
    const selector = rule.slice(0, rule.indexOf("{")).trim();
    if (!/^(#inspector|\.inspector)/.test(selector)) {
      throw new Error(`selector must be inspector-rooted: ${selector}`);
    }
  }
}

function collectImportClosure(ast) {
  const external = new Set();
  const local = new Set();
  traverse(ast, {
    ImportDeclaration(nodePath) {
      const source = nodePath.node.source.value;
      if (source.startsWith(".") || source.startsWith("/")) {
        local.add(source);
      } else {
        external.add(source);
      }
    },
  });
  return { external, local };
}

test("inspector containment anchors occur exactly once in archived source", () => {
  const html = readFileSync(ARCHIVED_HTML, "utf8");
  const script = extractLegacyScript(html);
  assert.equal(countNonOverlapping(script, INSPECTOR_ANCHOR_I1), 1);
  assert.equal(countNonOverlapping(script, INSPECTOR_ANCHOR_I2), 1);
  assert.equal(
    countNonOverlapping(
      script.replace(INSPECTOR_ANCHOR_I1, ""),
      INSPECTOR_ANCHOR_I1,
    ),
    0,
  );
});

test("contained script removes inspector innerHTML and bindInspector calls", () => {
  const html = readFileSync(ARCHIVED_HTML, "utf8");
  const script = applyInspectorContainment(extractLegacyScript(html));
  assert.equal(countNonOverlapping(script, '$("#inspector").innerHTML'), 0);
  const withoutDefinition = script.replace("function bindInspector(){", "");
  assert.equal(countNonOverlapping(withoutDefinition, "bindInspector()"), 0);
  assert.equal(countNonOverlapping(script, "function bindInspector"), 1);
});

test("legacy export and parser replacement remain", () => {
  const regionsAst = parseModule(LEGACY_REGIONS);
  assert.ok(collectNamedExports(regionsAst).has("LegacyInspector"));
  const hostAst = parseModule(LEGACY_HOST);
  assert.ok(hasInspectorParserReplacement(hostAst));
  const hostSource = readFileSync(LEGACY_HOST, "utf8");
  assert.ok(hostSource.includes(INSPECTOR_REPLACEMENT_I1));
  assert.ok(hostSource.includes("class ContainmentInitializationError extends Error"));
});

test("inspector candidate css is verbatim inspector-rooted subset", () => {
  const html = readFileSync(ARCHIVED_HTML, "utf8");
  const style = extractLegacyStyle(html);
  const css = readFileSync(INSPECTOR_CSS, "utf8");
  assert.doesNotThrow(() => validateInspectorCandidateCss(css, style));
  assert.throws(() =>
    validateInspectorCandidateCss(`${css}\n.row{color:red}`, style),
  );
  assert.throws(() =>
    validateInspectorCandidateCss(
      css.replace(
        ".inspector{border-left:1px solid var(--line)}",
        ".inspector{border-left:2px solid var(--line)}",
      ),
      style,
    ),
  );
});

test("inspector candidate import closure", () => {
  const ast = parseModule(INSPECTOR_CANDIDATE);
  const { external, local } = collectImportClosure(ast);
  assert.deepEqual([...external].sort(), ["react"]);
  assert.deepEqual([...local].sort(), ["./inspector-candidate.css"]);
  const source = readFileSync(INSPECTOR_CANDIDATE, "utf8");
  assert.ok(!source.includes("data-react-surface"));
  assert.ok(!source.includes("docs/mocks/"));
  assert.ok(!source.includes("../legacy/"));
  assert.ok(!source.includes("#plugin-history"));
  assert.ok(!source.includes("plugin-history-item"));
});

test("inspector candidate avoids inline plugin history reconstruction", () => {
  const source = readFileSync(INSPECTOR_CANDIDATE, "utf8");
  assert.ok(!source.includes('querySelector("#plugin-history")'));
  assert.ok(!source.includes("plugin-history-item"));
  assert.ok(!/createElement\(\s*"button"\s*\)[\s\S]*plugin-history/.test(source));
});
