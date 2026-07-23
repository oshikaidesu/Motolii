import { createHash, randomInt } from "node:crypto";
import { readFile, readdir, mkdtemp, writeFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";
import postcss from "postcss";

const traverse = traverseModule.default ?? traverseModule;
const JS_EXTENSIONS = new Set([".js", ".jsx", ".mjs", ".cjs"]);
const REFERENCE_KINDS = new Set([
  "candidate",
  "reference",
  "diagnostic",
  "archive",
]);
const FIXTURE_LAYERS = ["document", "scenes", "tokens"];
const RAW_COLOR = /(?:^|[\s(:,])#[0-9a-f]{3,8}\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch|color)\s*\(/i;
const FORBIDDEN_RUNTIME_SEGMENTS = new Set(["legacy", "archive"]);

export class ReferenceGuardError extends Error {
  constructor(code, message) {
    super(`${code}: ${message}`);
    this.name = "ReferenceGuardError";
    this.code = code;
  }
}

function reject(code, message) {
  throw new ReferenceGuardError(code, message);
}

function posix(relativePath) {
  return relativePath.split(path.sep).join("/");
}

function safeRelativePath(value, label) {
  if (typeof value !== "string" || value.length === 0) {
    reject("RG-SCHEMA", `${label} must be a non-empty relative path`);
  }
  const normalized = posix(path.normalize(value));
  if (
    path.isAbsolute(value) ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    normalized.includes("\0")
  ) {
    reject("RG-SCHEMA", `${label} escapes the reference root: ${value}`);
  }
  return normalized.replace(/^\.\//, "");
}

function absoluteFrom(root, relativePath, label = "path") {
  const safe = safeRelativePath(relativePath, label);
  const absolute = path.resolve(root, safe);
  const relative = path.relative(root, absolute);
  if (relative === ".." || relative.startsWith(`..${path.sep}`)) {
    reject("RG-SCHEMA", `${label} escapes the reference root: ${relativePath}`);
  }
  return absolute;
}

async function sha256File(filename) {
  const bytes = await readFile(filename);
  return createHash("sha256").update(bytes).digest("hex");
}

function sha256Bytes(value) {
  return createHash("sha256").update(value).digest("hex");
}

async function parseModule(filename) {
  let source;
  try {
    source = await readFile(filename, "utf8");
  } catch (error) {
    reject("RG-MISSING", `${filename}: ${error.message}`);
  }
  try {
    return {
      source,
      ast: parse(source, {
        sourceType: "module",
        plugins: ["jsx", "importAttributes", "topLevelAwait"],
      }),
    };
  } catch (error) {
    reject("RG-PARSE", `${filename}: ${error.message}`);
  }
}

function staticKey(node) {
  if (!node || node.computed) return null;
  if (node.key?.type === "Identifier") return node.key.name;
  if (node.key?.type === "StringLiteral") return node.key.value;
  return null;
}

function staticStringProperty(objectExpression, propertyName) {
  const matches = objectExpression.properties.filter(
    (property) => staticKey(property) === propertyName,
  );
  if (matches.length !== 1 || matches[0].value?.type !== "StringLiteral") {
    reject(
      "RG-REGISTRY",
      `registry entry requires one static ${propertyName} string`,
    );
  }
  return matches[0].value.value;
}

function exportedRegistryObject(ast) {
  for (const statement of ast.program.body) {
    if (statement.type !== "ExportNamedDeclaration") continue;
    const declaration = statement.declaration;
    if (declaration?.type !== "VariableDeclaration") continue;
    for (const variable of declaration.declarations) {
      if (
        variable.id.type === "Identifier" &&
        variable.id.name === "screenRegistry" &&
        variable.init?.type === "ObjectExpression"
      ) {
        return variable.init;
      }
    }
  }
  reject(
    "RG-REGISTRY",
    "registry module must export const screenRegistry as a static object",
  );
}

export async function inspectRegistry(registryFilename) {
  const { ast } = await parseModule(registryFilename);
  const registry = exportedRegistryObject(ast);
  const routes = new Map();

  for (const property of registry.properties) {
    if (property.type !== "ObjectProperty" || property.computed) {
      reject("RG-REGISTRY", "registry cannot contain spreads or computed routes");
    }
    const route = staticKey(property);
    if (!route || property.value.type !== "ObjectExpression") {
      reject("RG-REGISTRY", "registry routes and entries must be static objects");
    }
    if (routes.has(route)) {
      reject("RG-REGISTRY", `duplicate route: ${route}`);
    }
    const kind = staticStringProperty(property.value, "catalogKind");
    if (!REFERENCE_KINDS.has(kind)) {
      reject("RG-REGISTRY", `${route} has unknown catalogKind ${kind}`);
    }
    if (kind === "archive" && !route.startsWith("archive/")) {
      reject("RG-ROUTE", `archive route must use archive/: ${route}`);
    }
    if (kind === "reference" && !route.startsWith("reference/")) {
      reject("RG-ROUTE", `reference route must use reference/: ${route}`);
    }
    if (
      kind === "candidate" &&
      /^(?:archive|reference|diagnostic)\//.test(route)
    ) {
      reject("RG-ROUTE", `candidate route uses a reserved prefix: ${route}`);
    }
    routes.set(route, kind);
  }

  for (const requiredKind of ["candidate", "diagnostic", "archive"]) {
    if (![...routes.values()].includes(requiredKind)) {
      reject("RG-REGISTRY", `registry has no ${requiredKind} route`);
    }
  }
  return routes;
}

async function inspectRegistryComponents(root, registryRelativePath) {
  const registryFilename = absoluteFrom(root, registryRelativePath);
  const { ast } = await parseModule(registryFilename);
  const imports = importRecords(ast);
  const registry = exportedRegistryObject(ast);
  const components = new Map();
  for (const property of registry.properties) {
    const route = staticKey(property);
    if (!route || property.value?.type !== "ObjectExpression") continue;
    const componentProperties = property.value.properties.filter(
      (entry) => staticKey(entry) === "Component",
    );
    if (
      componentProperties.length !== 1 ||
      componentProperties[0].value.type !== "Identifier"
    ) {
      reject("RG-REGISTRY", `${route} requires one static Component binding`);
    }
    const local = componentProperties[0].value.name;
    const imported = imports.filter((entry) => entry.local === local);
    if (imported.length === 0) {
      components.set(route, null);
      continue;
    }
    if (imported.length !== 1) {
      reject("RG-REGISTRY", `${route} Component import is ambiguous`);
    }
    components.set(route, {
      source: await resolveRelativeImport(
        root,
        registryFilename,
        imported[0].source,
      ),
      imported: imported[0].imported,
    });
  }
  return components;
}

function importRecords(ast) {
  const records = [];
  for (const statement of ast.program.body) {
    if (statement.type !== "ImportDeclaration") continue;
    if (statement.specifiers.length === 0) {
      records.push({ source: statement.source.value, imported: null, local: null });
      continue;
    }
    for (const specifier of statement.specifiers) {
      let imported;
      if (specifier.type === "ImportDefaultSpecifier") imported = "default";
      else if (specifier.type === "ImportNamespaceSpecifier") imported = "*";
      else imported = specifier.imported.name ?? specifier.imported.value;
      records.push({
        source: statement.source.value,
        imported,
        local: specifier.local.name,
      });
    }
  }
  return records;
}

function moduleExports(ast) {
  const exports = new Set();
  for (const statement of ast.program.body) {
    if (statement.type === "ExportDefaultDeclaration") {
      exports.add("default");
      continue;
    }
    if (statement.type !== "ExportNamedDeclaration") continue;
    for (const specifier of statement.specifiers) {
      exports.add(specifier.exported.name ?? specifier.exported.value);
    }
    const declaration = statement.declaration;
    if (
      declaration?.type === "FunctionDeclaration" ||
      declaration?.type === "ClassDeclaration"
    ) {
      if (declaration.id) exports.add(declaration.id.name);
    } else if (declaration?.type === "VariableDeclaration") {
      for (const variable of declaration.declarations) {
        if (variable.id.type === "Identifier") exports.add(variable.id.name);
      }
    }
  }
  return exports;
}

function functionBindingName(functionPath) {
  if (functionPath.node.id?.type === "Identifier") return functionPath.node.id.name;
  if (
    functionPath.parentPath?.isVariableDeclarator() &&
    functionPath.parentPath.node.id.type === "Identifier"
  ) {
    return functionPath.parentPath.node.id.name;
  }
  return null;
}

function hasControlFlowBetween(descendantPath, ancestorPath) {
  let current = descendantPath.parentPath;
  while (current && current !== ancestorPath) {
    if (
      current.isIfStatement() ||
      current.isSwitchStatement() ||
      current.isTryStatement() ||
      current.isLoop?.() ||
      current.isConditionalExpression() ||
      current.isLogicalExpression() ||
      current.isFunction()
    ) {
      return true;
    }
    current = current.parentPath;
  }
  return current !== ancestorPath;
}

function requiredComponentRenderNodes(ast, screenExport, localName) {
  const rendered = new Set();
  traverse(ast, {
    ReturnStatement(returnPath) {
      const functionPath = returnPath.getFunctionParent();
      if (
        !functionPath ||
        functionBindingName(functionPath) !== screenExport ||
        hasControlFlowBetween(returnPath, functionPath)
      ) {
        return;
      }
      returnPath.traverse({
        JSXOpeningElement(openingPath) {
          if (
            openingPath.node.name.type === "JSXIdentifier" &&
            openingPath.node.name.name === localName &&
            !hasControlFlowBetween(openingPath, returnPath)
          ) {
            rendered.add(openingPath.node);
          }
        },
      });
    },
  });
  return rendered;
}

function projectionRoot(node) {
  let current = node;
  while (
    current?.type === "AwaitExpression" ||
    current?.type === "ParenthesizedExpression" ||
    current?.type === "TSAsExpression" ||
    current?.type === "TSTypeAssertion" ||
    current?.type === "TypeCastExpression"
  ) {
    current = current.argument ?? current.expression;
  }
  while (
    current?.type === "MemberExpression" ||
    current?.type === "OptionalMemberExpression"
  ) {
    current = projectionRoot(current.object);
  }
  return current;
}

function requiredPropTarget(valuePath, requiredComponents) {
  const attributePath = valuePath.findParent((candidate) =>
    candidate.isJSXAttribute(),
  );
  if (
    !attributePath ||
    attributePath.node.name.type !== "JSXIdentifier" ||
    attributePath.node.value?.type !== "JSXExpressionContainer" ||
    projectionRoot(attributePath.node.value.expression) !==
      projectionRoot(valuePath.node)
  ) {
    return null;
  }
  const openingPath = attributePath.findParent((candidate) =>
    candidate.isJSXOpeningElement(),
  );
  if (
    !openingPath ||
    openingPath.node.name.type !== "JSXIdentifier"
  ) {
    return null;
  }
  const localName = openingPath.node.name.name;
  const required = requiredComponents.get(localName);
  return required?.fixtureProp === attributePath.node.name.name &&
    required.renderNodes.has(openingPath.node)
    ? openingPath.node
    : null;
}

function fixtureLoaderCallsFeedRequiredProps(
  ast,
  requiredComponents,
  loaderLocals,
) {
  let callCount = 0;
  let allFlow = true;
  const fedRenderNodes = new Set();
  traverse(ast, {
    CallExpression(callPath) {
      if (
        callPath.node.callee.type !== "Identifier" ||
        !loaderLocals.has(callPath.node.callee.name)
      ) {
        return;
      }
      callCount += 1;
      let valuePath = callPath;
      while (valuePath.parentPath?.isAwaitExpression()) {
        valuePath = valuePath.parentPath;
      }
      const directTarget = requiredPropTarget(valuePath, requiredComponents);
      if (directTarget) {
        fedRenderNodes.add(directTarget);
        return;
      }
      if (
        valuePath.parentPath?.isVariableDeclarator() &&
        valuePath.parentPath.node.id.type === "Identifier"
      ) {
        const binding = valuePath.scope.getBinding(
          valuePath.parentPath.node.id.name,
        );
        if (
          !binding?.constant ||
          !valuePath.parentPath.parentPath?.isVariableDeclaration({
            kind: "const",
          })
        ) {
          allFlow = false;
          return;
        }
        const targets = new Set(
          (binding?.referencePaths ?? [])
            .map((referencePath) =>
              requiredPropTarget(referencePath, requiredComponents),
            )
            .filter(Boolean),
        );
        if (targets.size > 0) {
          for (const target of targets) fedRenderNodes.add(target);
          return;
        }
      }
      allFlow = false;
    },
  });
  return (
    callCount > 0 &&
    allFlow &&
    [...requiredComponents.values()].every((required) =>
      [...required.renderNodes].every((renderNode) =>
        fedRenderNodes.has(renderNode),
      ),
    )
  );
}

async function resolveRelativeImport(root, importer, source) {
  if (!source.startsWith(".")) return null;
  const base = path.resolve(path.dirname(importer), source);
  const candidates = path.extname(base)
    ? [base]
    : [base, ...[".js", ".jsx", ".mjs", ".css"].map((ext) => `${base}${ext}`)];
  for (const candidate of candidates) {
    try {
      await readFile(candidate);
      return safeRelativePath(path.relative(root, candidate), "resolved import");
    } catch (error) {
      if (error.code !== "ENOENT" && error.code !== "EISDIR") throw error;
    }
  }
  reject(
    "RG-IMPORT",
    `${posix(path.relative(root, importer))} has unresolved import ${source}`,
  );
}

async function cssRelativeImports(root, filename) {
  const source = await readFile(filename, "utf8");
  let tree;
  try {
    tree = postcss.parse(source, { from: filename });
  } catch (error) {
    reject("RG-PARSE", `${filename}: ${error.message}`);
  }
  const imports = [];
  tree.walkAtRules("import", (rule) => {
    const match = rule.params.match(/^(?:url\()?['"]([^'"]+)['"]/);
    if (match?.[1]?.startsWith(".")) imports.push(match[1]);
  });
  const resolved = [];
  for (const imported of imports) {
    resolved.push(await resolveRelativeImport(root, filename, imported));
  }
  return resolved;
}

function callResultIsDiscarded(callPath) {
  let current = callPath;
  while (
    current.parentPath?.isAwaitExpression() ||
    current.parentPath?.isTSAsExpression?.() ||
    current.parentPath?.isParenthesizedExpression?.() ||
    current.parentPath?.isChainExpression?.()
  ) {
    current = current.parentPath;
  }
  return (
    current.parentPath?.isExpressionStatement() ||
    (current.parentPath?.isUnaryExpression() &&
      current.parentPath.node.operator === "void")
  );
}

function isFixtureLoaderCall(
  node,
  loaderLocals = new Set(["loadReferenceFixtures"]),
) {
  if (node.callee.type === "Identifier") {
    return loaderLocals.has(node.callee.name);
  }
  return (
    node.callee.type === "MemberExpression" &&
    !node.callee.computed &&
    node.callee.property.type === "Identifier" &&
    node.callee.property.name === "loadReferenceFixtures"
  );
}

function rejectDynamicImports(ast, filename) {
  traverse(ast, {
    enter(nodePath) {
      if (
        nodePath.node.type === "ImportExpression" ||
        (nodePath.node.type === "CallExpression" &&
          nodePath.node.callee.type === "Import")
      ) {
        reject(
          "RG-IMPORT",
          `${filename}:${nodePath.node.loc.start.line} uses a dynamic import`,
        );
      }
    },
  });
}

async function countFixtureLoaderCalls(filename) {
  if (!JS_EXTENSIONS.has(path.extname(filename))) return 0;
  const { ast } = await parseModule(filename);
  const loaderLocals = new Set(["loadReferenceFixtures"]);
  for (const imported of importRecords(ast)) {
    if (imported.imported === "loadReferenceFixtures" && imported.local) {
      loaderLocals.add(imported.local);
    }
  }
  let count = 0;
  traverse(ast, {
    CallExpression(callPath) {
      if (isFixtureLoaderCall(callPath.node, loaderLocals)) count += 1;
    },
  });
  return count;
}

async function scanReferenceSource(filename) {
  const extension = path.extname(filename);
  const source = await readFile(filename, "utf8");
  if (extension === ".css") {
    let tree;
    try {
      tree = postcss.parse(source, { from: filename });
    } catch (error) {
      reject("RG-PARSE", `${filename}: ${error.message}`);
    }
    tree.walkDecls((declaration) => {
      if (
        RAW_COLOR.test(declaration.value) ||
        /(?:color|background|border|fill|stroke)/i.test(declaration.prop)
      ) {
        reject(
          "RG-RAW-COLOR",
          `${filename}:${declaration.source.start.line} contains a raw color`,
        );
      }
    });
    return;
  }
  if (!JS_EXTENSIONS.has(extension)) return;
  const { ast } = await parseModule(filename);
  rejectDynamicImports(ast, filename);
  traverse(ast, {
    CallExpression(callPath) {
      if (isFixtureLoaderCall(callPath.node) && callResultIsDiscarded(callPath)) {
        reject(
          "RG-FIXTURE-LOAD",
          `${filename}:${callPath.node.loc.start.line} discards loadReferenceFixtures result`,
        );
      }
    },
    StringLiteral(stringPath) {
      if (RAW_COLOR.test(stringPath.node.value)) {
        reject(
          "RG-RAW-COLOR",
          `${filename}:${stringPath.node.loc.start.line} contains a raw color`,
        );
      }
    },
    TemplateElement(templatePath) {
      if (RAW_COLOR.test(templatePath.node.value.raw)) {
        reject(
          "RG-RAW-COLOR",
          `${filename}:${templatePath.node.loc.start.line} contains a raw color`,
        );
      }
    },
    JSXAttribute(attributePath) {
      if (
        attributePath.node.name.type !== "JSXIdentifier" ||
        attributePath.node.name.name !== "style" ||
        attributePath.node.value?.type !== "JSXExpressionContainer" ||
        attributePath.node.value.expression.type !== "ObjectExpression"
      ) {
        return;
      }
      for (const property of attributePath.node.value.expression.properties) {
        const key = staticKey(property);
        if (
          !key ||
          /(?:color|background|border|fill|stroke)/i.test(key)
        ) {
          reject(
            "RG-RAW-COLOR",
            `${filename}:${attributePath.node.loc.start.line} contains an inline color style`,
          );
        }
      }
    },
  });
}

async function walkFiles(root) {
  const files = [];
  async function visit(directory) {
    let entries;
    try {
      entries = await readdir(directory, { withFileTypes: true });
    } catch (error) {
      reject("RG-MISSING", `${directory}: ${error.message}`);
    }
    entries.sort((left, right) => left.name.localeCompare(right.name));
    for (const entry of entries) {
      const filename = path.join(directory, entry.name);
      if (entry.isDirectory()) await visit(filename);
      else if (entry.isFile() || entry.isSymbolicLink()) files.push(filename);
    }
  }
  await visit(root);
  return files;
}

function validateManifestShape(manifest) {
  if (!manifest || manifest.schemaVersion !== 1) {
    reject("RG-SCHEMA", "manifest schemaVersion must be 1");
  }
  for (const field of [
    "registryModule",
    "referenceRoots",
    "referenceFiles",
    "assets",
    "screens",
  ]) {
    if (!(field in manifest)) reject("RG-SCHEMA", `manifest is missing ${field}`);
  }
  if (
    !Array.isArray(manifest.referenceRoots) ||
    manifest.referenceRoots.length === 0 ||
    !Array.isArray(manifest.referenceFiles) ||
    manifest.referenceFiles.length === 0 ||
    !Array.isArray(manifest.assets) ||
    !Array.isArray(manifest.screens) ||
    manifest.screens.length === 0
  ) {
    reject("RG-SCHEMA", "manifest collections have invalid cardinality");
  }
}

function forbidLegacyRuntime(relativePath) {
  const segments = relativePath.split("/");
  if (segments.some((segment) => FORBIDDEN_RUNTIME_SEGMENTS.has(segment))) {
    reject("RG-LEGACY", `runtime closure reaches ${relativePath}`);
  }
}

export async function verifyReferenceManifest(root, manifest) {
  root = path.resolve(root);
  validateManifestShape(manifest);
  const registryModule = safeRelativePath(
    manifest.registryModule,
    "registryModule",
  );
  const routes = await inspectRegistry(absoluteFrom(root, registryModule));
  const registryComponents = await inspectRegistryComponents(
    root,
    registryModule,
  );
  const referenceRoots = manifest.referenceRoots.map((value, index) =>
    safeRelativePath(value, `referenceRoots[${index}]`),
  );
  const referenceFiles = new Set(
    manifest.referenceFiles.map((value, index) =>
      safeRelativePath(value, `referenceFiles[${index}]`),
    ),
  );
  if (referenceFiles.size !== manifest.referenceFiles.length) {
    reject("RG-SCHEMA", "referenceFiles contains duplicates");
  }

  const actualReferenceFiles = [];
  for (const referenceRoot of referenceRoots) {
    for (const filename of await walkFiles(absoluteFrom(root, referenceRoot))) {
      actualReferenceFiles.push(posix(path.relative(root, filename)));
    }
  }
  const actualSet = new Set(actualReferenceFiles);
  for (const filename of referenceFiles) {
    if (!actualSet.has(filename)) {
      reject("RG-CLOSURE", `declared reference file is missing: ${filename}`);
    }
  }
  for (const filename of actualSet) {
    if (!referenceFiles.has(filename)) {
      reject("RG-CLOSURE", `undeclared reference file: ${filename}`);
    }
    await scanReferenceSource(absoluteFrom(root, filename));
  }

  const assets = new Map();
  const testAssets = [];
  for (const [index, rawAsset] of manifest.assets.entries()) {
    const assetPath = safeRelativePath(rawAsset.path, `assets[${index}].path`);
    if (assets.has(assetPath)) reject("RG-SCHEMA", `duplicate asset ${assetPath}`);
    if (!new Set(["runtime", "test"]).has(rawAsset.role)) {
      reject("RG-SCHEMA", `${assetPath} has invalid role ${rawAsset.role}`);
    }
    if (!/^[0-9a-f]{64}$/.test(rawAsset.sha256 ?? "")) {
      reject("RG-SCHEMA", `${assetPath} has invalid SHA-256`);
    }
    const actualHash = await sha256File(absoluteFrom(root, assetPath));
    if (actualHash !== rawAsset.sha256) {
      reject("RG-PROVENANCE", `${assetPath} SHA-256 does not match manifest`);
    }
    if (rawAsset.role === "runtime") forbidLegacyRuntime(assetPath);
    else testAssets.push(assetPath);
    assets.set(assetPath, {
      ...rawAsset,
      path: assetPath,
      exports: new Set(rawAsset.exports ?? []),
    });
  }
  if (testAssets.length === 0) {
    reject("RG-PROVENANCE", "manifest must include source-asset test evidence");
  }

  for (const asset of assets.values()) {
    if (!JS_EXTENSIONS.has(path.extname(asset.path))) continue;
    const { ast } = await parseModule(absoluteFrom(root, asset.path));
    const actualExports = moduleExports(ast);
    for (const expectedExport of asset.exports) {
      if (!actualExports.has(expectedExport)) {
        reject(
          "RG-PROVENANCE",
          `${asset.path} does not export ${expectedExport}`,
        );
      }
    }
  }

  for (const testAsset of testAssets) {
    const seenTestClosure = new Set();
    let reachesRuntime = false;
    async function visitTestEvidence(assetPath) {
      if (seenTestClosure.has(assetPath)) return;
      seenTestClosure.add(assetPath);
      const filename = absoluteFrom(root, assetPath);
      let dependencies = [];
      if (JS_EXTENSIONS.has(path.extname(assetPath))) {
        const { ast } = await parseModule(filename);
        for (const entry of importRecords(ast)) {
          if (entry.source.startsWith(".")) {
            dependencies.push(
              await resolveRelativeImport(root, filename, entry.source),
            );
          }
        }
      } else if (path.extname(assetPath) === ".css") {
        dependencies = await cssRelativeImports(root, filename);
      }
      for (const dependency of dependencies) {
        const declared = assets.get(dependency);
        if (!declared) {
          reject(
            "RG-CLOSURE",
            `test evidence dependency is outside manifest: ${dependency}`,
          );
        }
        if (declared.role === "runtime") reachesRuntime = true;
        else await visitTestEvidence(dependency);
      }
    }
    await visitTestEvidence(testAsset);
    if (!reachesRuntime) {
      reject(
        "RG-PROVENANCE",
        `test evidence does not reach a runtime source asset: ${testAsset}`,
      );
    }
  }

  const screenIds = new Set();
  const screenModules = new Set();
  const requiredByModule = new Map();
  for (const [index, screen] of manifest.screens.entries()) {
    if (typeof screen.id !== "string" || screenIds.has(screen.id)) {
      reject("RG-SCHEMA", `invalid or duplicate screen id at index ${index}`);
    }
    screenIds.add(screen.id);
    const screenModule = safeRelativePath(screen.module, `screens[${index}].module`);
    if (typeof screen.export !== "string" || screen.export.length === 0) {
      reject("RG-SCHEMA", `${screen.id} has no exported screen binding`);
    }
    if (!referenceFiles.has(screenModule)) {
      reject("RG-CLOSURE", `screen module is outside referenceFiles: ${screenModule}`);
    }
    if (screenModules.has(screenModule)) {
      reject("RG-COPY", `multiple screens reuse one reference leaf: ${screenModule}`);
    }
    screenModules.add(screenModule);
    if (routes.get(screen.route) !== "reference") {
      reject("RG-ROUTE", `${screen.route} is not a reference-only registry route`);
    }
    const registryComponent = registryComponents.get(screen.route);
    if (
      !registryComponent ||
      registryComponent.source !== screenModule ||
      registryComponent.imported !== screen.export
    ) {
      reject(
        "RG-ROUTE",
        `${screen.route} does not bind ${screen.export} from ${screenModule}`,
      );
    }
    if (!Array.isArray(screen.requiredImports) || screen.requiredImports.length === 0) {
      reject("RG-PROVENANCE", `${screen.id} has no required source import`);
    }
    requiredByModule.set(screenModule, {
      screenExport: screen.export,
      imports: screen.requiredImports.map((entry, importIndex) => ({
        source: safeRelativePath(
          entry.source,
          `screens[${index}].requiredImports[${importIndex}].source`,
        ),
        imported: entry.imported,
        fixtureProp: entry.fixtureProp,
      })),
    });
  }

  for (const referenceFile of referenceFiles) {
    if (
      !screenModules.has(referenceFile) &&
      (await countFixtureLoaderCalls(absoluteFrom(root, referenceFile))) > 0
    ) {
      reject(
        "RG-FIXTURE-LOAD",
        `fixture loading is only allowed in declared screen modules: ${referenceFile}`,
      );
    }
  }
  for (const asset of assets.values()) {
    if (
      asset.role === "runtime" &&
      (await countFixtureLoaderCalls(absoluteFrom(root, asset.path))) > 0
    ) {
      reject(
        "RG-FIXTURE-LOAD",
        `source asset runtime cannot own fixture loading: ${asset.path}`,
      );
    }
  }

  const visited = new Set();
  const reachableRuntimeAssets = new Set();
  async function visit(relativePath) {
    if (visited.has(relativePath)) return;
    visited.add(relativePath);
    const filename = absoluteFrom(root, relativePath);
    let dependencies = [];
    let directImports = [];
    let moduleAst = null;
    if (JS_EXTENSIONS.has(path.extname(relativePath))) {
      ({ ast: moduleAst } = await parseModule(filename));
      rejectDynamicImports(moduleAst, filename);
      directImports = importRecords(moduleAst);
      for (const imported of directImports) {
        const resolved = await resolveRelativeImport(root, filename, imported.source);
        if (resolved) dependencies.push(resolved);
      }
    } else if (path.extname(relativePath) === ".css") {
      dependencies = await cssRelativeImports(root, filename);
    }

    if (requiredByModule.has(relativePath)) {
      const requiredConfig = requiredByModule.get(relativePath);
      const requiredComponents = new Map();
      const loaderImports = directImports.filter(
        (entry) => entry.imported === "loadReferenceFixtures" && entry.local,
      );
      if (loaderImports.length !== 1) {
        reject(
          "RG-FIXTURE-LOAD",
          `${relativePath} requires one named loadReferenceFixtures import`,
        );
      }
      const loaderSource = await resolveRelativeImport(
        root,
        filename,
        loaderImports[0].source,
      );
      if (!loaderSource || !referenceFiles.has(loaderSource)) {
        reject(
          "RG-FIXTURE-LOAD",
          `${relativePath} fixture loader must come from the declared reference closure`,
        );
      }
      const loaderLocals = new Set(loaderImports.map((entry) => entry.local));
      if (!moduleExports(moduleAst).has(requiredConfig.screenExport)) {
        reject(
          "RG-PROVENANCE",
          `${relativePath} does not export ${requiredConfig.screenExport}`,
        );
      }
      for (const required of requiredConfig.imports) {
        const matching = [];
        for (const imported of directImports) {
          const resolved = await resolveRelativeImport(root, filename, imported.source);
          if (resolved === required.source && imported.imported === required.imported) {
            matching.push(imported);
          }
        }
        if (matching.length !== 1) {
          reject(
            "RG-PROVENANCE",
            `${relativePath} must directly import ${required.imported} from ${required.source}`,
          );
        }
        if (required.imported === null || !matching[0].local) {
          reject(
            "RG-PROVENANCE",
            `${relativePath} requires a named source component import from ${required.source}`,
          );
        }
        if (
          typeof required.fixtureProp !== "string" ||
          required.fixtureProp.length === 0
        ) {
          reject(
            "RG-SCHEMA",
            `${relativePath}/${required.imported} has no fixtureProp`,
          );
        }
        const renderNodes = requiredComponentRenderNodes(
          moduleAst,
          requiredConfig.screenExport,
          matching[0].local,
        );
        requiredComponents.set(matching[0].local, {
          fixtureProp: required.fixtureProp,
          renderNodes,
        });
        if (renderNodes.size === 0) {
          reject(
            "RG-PROVENANCE",
            `${relativePath} does not render ${required.imported} from ${required.source} in its unconditional screen return`,
          );
        }
      }
      if (
        !fixtureLoaderCallsFeedRequiredProps(
          moduleAst,
          requiredComponents,
          loaderLocals,
        )
      ) {
        reject(
          "RG-FIXTURE-LOAD",
          `${relativePath} must feed loadReferenceFixtures result to a required source component prop`,
        );
      }
    }

    for (const dependency of dependencies) {
      forbidLegacyRuntime(dependency);
      if (dependency === registryModule && referenceFiles.has(relativePath)) {
        reject("RG-SELF-REGISTER", `${relativePath} imports the central registry`);
      }
      if (referenceFiles.has(dependency)) {
        await visit(dependency);
        continue;
      }
      const asset = assets.get(dependency);
      if (!asset || asset.role !== "runtime") {
        reject("RG-CLOSURE", `runtime dependency is outside manifest: ${dependency}`);
      }
      reachableRuntimeAssets.add(dependency);
      await visit(dependency);
    }
  }

  for (const screenModule of screenModules) await visit(screenModule);
  const declaredRuntimeAssets = [...assets.values()]
    .filter((asset) => asset.role === "runtime")
    .map((asset) => asset.path);
  for (const assetPath of declaredRuntimeAssets) {
    if (!reachableRuntimeAssets.has(assetPath)) {
      reject("RG-PROVENANCE", `runtime asset is not reachable: ${assetPath}`);
    }
  }
  for (const testAsset of testAssets) {
    if (visited.has(testAsset)) {
      reject("RG-CLOSURE", `test evidence entered runtime closure: ${testAsset}`);
    }
  }

  const runtimeHashes = new Map();
  for (const assetPath of declaredRuntimeAssets) {
    runtimeHashes.set(await sha256File(absoluteFrom(root, assetPath)), assetPath);
  }
  for (const referenceFile of referenceFiles) {
    const hash = await sha256File(absoluteFrom(root, referenceFile));
    if (runtimeHashes.has(hash)) {
      reject(
        "RG-COPY",
        `${referenceFile} duplicates source asset ${runtimeHashes.get(hash)}`,
      );
    }
  }

  return {
    routes,
    screenIds,
    runtimeAssets: reachableRuntimeAssets,
  };
}

function cloneJson(value) {
  return JSON.parse(JSON.stringify(value));
}

function decodePointerToken(token) {
  return token.replaceAll("~1", "/").replaceAll("~0", "~");
}

function setJsonPointer(document, pointer, value) {
  if (typeof pointer !== "string" || !pointer.startsWith("/") || pointer === "/") {
    reject("RG-CAUSAL", `invalid JSON pointer: ${pointer}`);
  }
  const tokens = pointer.slice(1).split("/").map(decodePointerToken);
  let owner = document;
  for (const token of tokens.slice(0, -1)) {
    if (owner === null || typeof owner !== "object" || !(token in owner)) {
      reject("RG-CAUSAL", `probe pointer does not exist: ${pointer}`);
    }
    owner = owner[token];
  }
  const finalToken = tokens.at(-1);
  if (owner === null || typeof owner !== "object" || !(finalToken in owner)) {
    reject("RG-CAUSAL", `probe pointer does not exist: ${pointer}`);
  }
  const before = JSON.stringify(owner[finalToken]);
  const after = JSON.stringify(value);
  if (before === after) reject("RG-CAUSAL", `probe does not change ${pointer}`);
  owner[finalToken] = cloneJson(value);
}

function normalizeCaptures(raw, screenIds) {
  const captures = raw instanceof Map ? raw : new Map(Object.entries(raw ?? {}));
  for (const screenId of screenIds) {
    if (!captures.has(screenId)) {
      reject("RG-CAUSAL", `renderer omitted normal capture ${screenId}`);
    }
  }
  if (captures.size !== screenIds.size) {
    reject("RG-CAUSAL", "renderer returned an unknown or duplicate screen capture");
  }
  return new Map(
    [...captures].map(([screenId, bytes]) => {
      if (
        !Buffer.isBuffer(bytes) &&
        !(bytes instanceof Uint8Array) &&
        typeof bytes !== "string"
      ) {
        reject("RG-CAUSAL", `${screenId} capture is not byte-like`);
      }
      return [screenId, sha256Bytes(bytes)];
    }),
  );
}

function shuffled(values) {
  const result = [...values];
  for (let index = result.length - 1; index > 0; index -= 1) {
    const swap = randomInt(index + 1);
    [result[index], result[swap]] = [result[swap], result[index]];
  }
  return result;
}

async function loadCausalRenderer(root, manifest) {
  const config = manifest.causalRenderer;
  if (
    !config ||
    typeof config.path !== "string" ||
    typeof config.export !== "string"
  ) {
    reject("RG-CAUSAL", "manifest causalRenderer path/export is required");
  }
  const rendererPath = safeRelativePath(config.path, "causalRenderer.path");
  const asset = manifest.assets.find((entry) => entry.path === rendererPath);
  if (!asset || asset.role !== "test") {
    reject(
      "RG-CAUSAL",
      `causal renderer must be declared as test evidence: ${rendererPath}`,
    );
  }
  const filename = absoluteFrom(root, rendererPath);
  const actualHash = await sha256File(filename);
  if (actualHash !== asset.sha256) {
    reject("RG-PROVENANCE", `${rendererPath} SHA-256 does not match manifest`);
  }
  const module = await import(
    `${pathToFileURL(filename).href}?sha256=${actualHash}`
  );
  if (typeof module[config.export] !== "function") {
    reject(
      "RG-CAUSAL",
      `${rendererPath} does not export renderer ${config.export}`,
    );
  }
  return module[config.export];
}

export async function verifyFixtureCausality({ root, manifest }) {
  root = path.resolve(root);
  const layers = manifest.fixtureLayers;
  if (!Array.isArray(layers) || layers.map((layer) => layer.id).join(",") !== FIXTURE_LAYERS.join(",")) {
    reject(
      "RG-CAUSAL",
      `fixtureLayers must be exactly ${FIXTURE_LAYERS.join(", ")} in order`,
    );
  }
  const screenIds = new Set(manifest.screens.map((screen) => screen.id));
  const sourcePaths = {};
  const sourceBytes = new Map();
  const sourceHashes = new Map();
  for (const layer of layers) {
    sourcePaths[layer.id] = absoluteFrom(root, layer.path, `${layer.id} fixture`);
    sourceBytes.set(layer.id, await readFile(sourcePaths[layer.id]));
    sourceHashes.set(layer.id, await sha256File(sourcePaths[layer.id]));
    if (
      !layer.probe ||
      !Array.isArray(layer.changedScreens) ||
      layer.changedScreens.length === 0
    ) {
      reject("RG-CAUSAL", `${layer.id} lacks a probe or changedScreens`);
    }
    for (const screenId of [
      ...layer.changedScreens,
      ...(layer.unchangedScreens ?? []),
    ]) {
      if (!screenIds.has(screenId)) {
        reject("RG-CAUSAL", `${layer.id} names unknown screen ${screenId}`);
      }
    }
  }

  const assertSourcesUnchanged = async () => {
    for (const layer of layers) {
      if ((await sha256File(sourcePaths[layer.id])) !== sourceHashes.get(layer.id)) {
        reject("RG-CAUSAL", `renderer mutated source fixture ${layer.id}`);
      }
    }
  };

  const renderNormal = await loadCausalRenderer(root, manifest);
  await assertSourcesUnchanged();

  const temporaryRoot = await mkdtemp(path.join(tmpdir(), "motolii-reference-causal-"));
  try {
    const fixturePaths = Object.fromEntries(
      layers.map((layer) => [
        layer.id,
        path.join(temporaryRoot, `${layer.id}.json`),
      ]),
    );
    const states = new Map();
    states.set("baseline", new Map(sourceBytes));
    for (const layer of layers) {
      const parsed = JSON.parse(sourceBytes.get(layer.id).toString("utf8"));
      setJsonPointer(parsed, layer.probe.pointer, layer.probe.value);
      const stateBytes = new Map(sourceBytes);
      stateBytes.set(
        layer.id,
        Buffer.from(`${JSON.stringify(parsed, null, 2)}\n`),
      );
      states.set(layer.id, stateBytes);
    }

    const renderState = async (stateId) => {
      const stateBytes = states.get(stateId);
      for (const layer of layers) {
        await writeFile(fixturePaths[layer.id], stateBytes.get(layer.id));
      }
      const captures = normalizeCaptures(
        await renderNormal({ fixturePaths: { ...fixturePaths } }),
        screenIds,
      );
      for (const layer of layers) {
        const expected = sha256Bytes(stateBytes.get(layer.id));
        if ((await sha256File(fixturePaths[layer.id])) !== expected) {
          reject(
            "RG-CAUSAL",
            `renderer mutated causal fixture copy ${layer.id}`,
          );
        }
      }
      await assertSourcesUnchanged();
      return captures;
    };

    const stateIds = [...states.keys()];
    const firstOrder = shuffled(stateIds);
    let secondOrder = shuffled(stateIds);
    if (secondOrder.join("\0") === firstOrder.join("\0")) {
      secondOrder = [...secondOrder.slice(1), secondOrder[0]];
    }
    const firstCaptures = new Map();
    const secondCaptures = new Map();
    for (const stateId of firstOrder) {
      firstCaptures.set(stateId, await renderState(stateId));
    }
    for (const stateId of secondOrder) {
      secondCaptures.set(stateId, await renderState(stateId));
    }
    for (const stateId of stateIds) {
      for (const screenId of screenIds) {
        if (
          firstCaptures.get(stateId).get(screenId) !==
          secondCaptures.get(stateId).get(screenId)
        ) {
          reject(
            "RG-CAUSAL",
            `renderer is not deterministic for ${stateId}/${screenId}`,
          );
        }
      }
    }

    const baseline = firstCaptures.get("baseline");
    for (const layer of layers) {
      const captures = firstCaptures.get(layer.id);
      for (const screenId of layer.changedScreens) {
        if (captures.get(screenId) === baseline.get(screenId)) {
          reject(
            "RG-CAUSAL",
            `${layer.id} probe did not change ${screenId} normal capture`,
          );
        }
      }
      for (const screenId of layer.unchangedScreens ?? []) {
        if (captures.get(screenId) !== baseline.get(screenId)) {
          reject(
            "RG-CAUSAL",
            `${layer.id} probe unexpectedly changed ${screenId} normal capture`,
          );
        }
      }
    }
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true });
  }
}

async function main(argv) {
  const [command, input] = argv;
  if (command === "check-registry" && input) {
    const routes = await inspectRegistry(path.resolve(input));
    console.log(`reference-guard: registry OK (${routes.size} routes)`);
    return;
  }
  if (command === "check-manifest" && input) {
    const manifestPath = path.resolve(input);
    const manifest = JSON.parse(await readFile(manifestPath, "utf8"));
    await verifyReferenceManifest(path.dirname(manifestPath), manifest);
    console.log("reference-guard: manifest OK");
    return;
  }
  console.error(
    "Usage: node scripts/reference-guard.mjs check-registry <registry.jsx>\n" +
      "       node scripts/reference-guard.mjs check-manifest <manifest.json>",
  );
  process.exitCode = 2;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  main(process.argv.slice(2)).catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
