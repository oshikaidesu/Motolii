import { createHash } from "node:crypto";
import { lstat, readFile, realpath } from "node:fs/promises";
import path from "node:path";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";
import postcss from "postcss";

const traverse = traverseModule.default ?? traverseModule;
const SHA256 = /^[0-9a-f]{64}$/;
const JS_EXTENSIONS = new Set([".js", ".jsx", ".mjs", ".cjs"]);
const CSS_URL_IMPORT = /url\(\s*['\"]?([^'\"\)\s]+)['\"]?\s*\)/g;

export class CurrentRouteProvenanceError extends Error {
  constructor(code, message) {
    super(`${code}: ${message}`);
    this.name = "CurrentRouteProvenanceError";
    this.code = code;
  }
}

function reject(code, message) {
  throw new CurrentRouteProvenanceError(code, message);
}

export const CURRENT_ROUTE_PROVENANCE_SCHEMA_VERSION = 2;

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function exactFields(value, fields, owner) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    reject("CRP-SCHEMA", `${owner} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...fields].sort();
  if (actual.join("\0") !== expected.join("\0")) {
    reject("CRP-SCHEMA", `${owner} has an invalid closed schema`);
  }
}

function safeRelativePath(value, owner) {
  if (typeof value !== "string" || value.length === 0) {
    reject("CRP-SCHEMA", `${owner} must be a non-empty relative path`);
  }
  if (value.includes("\\") || value.includes("\0")) {
    reject("CRP-PATH", `${owner} must use forward slashes`);
  }
  if (/^[A-Za-z]:\//.test(value)) {
    reject("CRP-PATH", `${owner} must not be absolute path`);
  }
  if (value.startsWith("./") || value === "." || value === "..") {
    reject("CRP-PATH", `${owner} must be canonical`);
  }
  const normalized = path.posix.normalize(value);
  if (
    path.isAbsolute(value) ||
    normalized === "." ||
    normalized === ".." ||
    normalized.startsWith("../") ||
    normalized.includes("/../") ||
    normalized !== value
  ) {
    reject("CRP-SCHEMA", `${owner} must be a canonical repo-relative path`);
  }
  return normalized;
}

function cleanSpecifier(value) {
  const marker = [value.indexOf("?"), value.indexOf("#")]
    .filter((entry) => entry >= 0)
    .sort((left, right) => left - right)[0];
  return marker >= 0 ? value.slice(0, marker) : value;
}

async function assertFile(root, absolutePath, owner) {
  let stat;
  try {
    stat = await lstat(absolutePath);
  } catch (error) {
    if (error.code === "ENOENT" || error.code === "EISDIR" || error.code === "ENOTDIR") {
      reject("CRP-MISSING", `${owner} is missing`);
    }
    throw error;
  }
  if (stat.isSymbolicLink()) {
    reject("CRP-SYMLINK", `${owner} must not be a symlink`);
  }
  if (!stat.isFile()) {
    reject("CRP-PATH", `${owner} must be a file`);
  }
  const [realRoot, realFile] = await Promise.all([realpath(root), realpath(absolutePath)]);
  const expectedRealFile = path.resolve(realRoot, path.relative(root, absolutePath));
  if (realFile !== expectedRealFile) {
    reject("CRP-SYMLINK", `${owner} must not traverse a symlink`);
  }
}

function ensureRepoPath(root, value, owner) {
  const safe = safeRelativePath(value, owner);
  const absolute = path.resolve(root, safe);
  const relative = path.relative(root, absolute);
  if (relative === ".." || relative.startsWith(`..${path.sep}`)) {
    reject("CRP-PATH", `${owner} escapes repository`);
  }
  return { safe, absolute };
}

async function parseModule(filename) {
  let source;
  try {
    source = await readFile(filename, "utf8");
  } catch (error) {
    reject("CRP-MISSING", `${filename}: ${error.message}`);
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
    reject("CRP-PARSE", `${filename}: ${error.message}`);
  }
}

function stripSpecifier(raw) {
  if (typeof raw !== "string" || raw.length === 0) {
    reject("CRP-IMPORT", "import specifier must be a non-empty string");
  }
  return cleanSpecifier(raw);
}

async function workspacePackages(root, importer) {
  let owner = path.dirname(importer);
  for (;;) {
    const relative = path.relative(root, owner);
    if (relative === ".." || relative.startsWith(`..${path.sep}`)) break;
    try {
      const raw = await readFile(path.join(owner, "package.json"), "utf8");
      const manifest = JSON.parse(raw);
      const dependencies = manifest.dependencies ?? {};
      const workspace = new Map();
      for (const [name, dep] of Object.entries(dependencies)) {
        if (typeof dep === "string" && dep.startsWith("file:")) {
          workspace.set(name, path.resolve(owner, dep.slice(5)));
        }
      }
      return workspace;
    } catch (error) {
      if (error.code !== "ENOENT" && error.code !== "ENOTDIR") throw error;
    }
    const parent = path.dirname(owner);
    if (parent === owner) break;
    owner = parent;
  }
  return new Map();
}

function resolvePackageEntry(pkg) {
  const exported = pkg?.exports?.["."];
  if (typeof exported === "string") return exported;
  if (exported && typeof exported === "object") {
    if (typeof exported.import === "string") return exported.import;
    if (typeof exported.require === "string") return exported.require;
    if (typeof exported.default === "string") return exported.default;
  }
  if (typeof pkg?.module === "string") return pkg.module;
  if (typeof pkg?.main === "string") return pkg.main;
  return null;
}

async function resolveImportSpecifier(root, importer, rawSource) {
  const source = stripSpecifier(rawSource);
  if (source.startsWith(".")) {
    return resolveRelativeImport(root, importer, source);
  }
  if (source.startsWith("/") || /^(?:[A-Za-z]:[\\/])/.test(source)) {
    reject("CRP-IMPORT", `${path.relative(root, importer)} uses absolute import ${rawSource}`);
  }
  return resolveWorkspaceImport(root, importer, source);
}

async function resolveRelativeImport(root, importer, source) {
  const base = path.resolve(path.dirname(importer), source);
  const candidates = path.extname(base)
    ? [base]
    : [
        base,
        ...[".js", ".jsx", ".mjs", ".css"].map((ext) => `${base}${ext}`),
        ...[".js", ".jsx", ".mjs", ".css"].map((ext) =>
          path.join(base, `index${ext}`),
        ),
      ];
  for (const candidate of candidates) {
    const relative = path.relative(root, candidate);
    if (relative === ".." || relative.startsWith(`..${path.sep}`)) {
      reject("CRP-PATH", `${path.relative(root, importer)} import escapes repository`);
    }
    let absolute;
    try {
      ({ absolute } = ensureRepoPath(root, relative, "relative import"));
    } catch (error) {
      if (error.code === "CRP-PATH") throw error;
      continue;
    }
    try {
      await assertFile(root, absolute, path.relative(root, candidate));
      return path.relative(root, absolute);
    } catch (error) {
      if (error.code === "CRP-MISSING" || error.code === "CRP-PATH") {
        continue;
      }
      throw error;
    }
  }
  reject("CRP-IMPORT", `${path.relative(root, importer)} has unresolved import ${source}`);
}

async function resolveWorkspaceImport(root, importer, source) {
  const workspace = await workspacePackages(root, importer);
  if (!workspace.has(source)) {
    if ([...workspace.keys()].some((name) => source.startsWith(`${name}/`))) {
      reject("CRP-IMPORT", `workspace package subpath ${source} is unresolved`);
    }
    return null;
  }
  const workspaceRoot = workspace.get(source);
  const packageJson = JSON.parse(await readFile(path.join(workspaceRoot, "package.json"), "utf8"));
  const entry = resolvePackageEntry(packageJson);
  if (!entry) {
    reject("CRP-IMPORT", `workspace package ${source} has no entrypoint`);
  }
  const candidate = path.resolve(workspaceRoot, entry);
  const relative = path.relative(root, candidate);
  const { absolute, safe } = ensureRepoPath(root, relative, `workspace package ${source}`);
  await assertFile(root, absolute, safe);
  return safe;
}

async function collectJsDependencies(root, filename) {
  const { ast } = await parseModule(filename);
  const dependencies = new Set();

  for (const statement of ast.program.body) {
    if (statement.type === "ImportDeclaration") {
      if (typeof statement.source.value !== "string") {
        reject("CRP-IMPORT", `${filename} has non-string import`);
      }
      const resolved = await resolveImportSpecifier(root, filename, statement.source.value);
      if (resolved) dependencies.add(resolved);
    }
    if (statement.type === "ExportNamedDeclaration" && statement.source) {
      if (typeof statement.source.value !== "string") {
        reject("CRP-IMPORT", `${filename} has non-string export source`);
      }
      const resolved = await resolveImportSpecifier(root, filename, statement.source.value);
      if (resolved) dependencies.add(resolved);
    }
    if (statement.type === "ExportAllDeclaration") {
      if (typeof statement.source.value !== "string") {
        reject("CRP-IMPORT", `${filename} has non-string export source`);
      }
      const resolved = await resolveImportSpecifier(root, filename, statement.source.value);
      if (resolved) dependencies.add(resolved);
    }
  }

  traverse(ast, {
    enter(nodePath) {
      const node = nodePath.node;
      if (
        node.type === "ImportExpression" ||
        (node.type === "CallExpression" && node.callee?.type === "Import")
      ) {
        reject("CRP-IMPORT", `${filename} uses dynamic import`);
      }
    },
    CallExpression(callPath) {
      if (
        callPath.node.callee.type === "Identifier" &&
        callPath.node.callee.name === "require"
      ) {
        reject("CRP-IMPORT", `${filename} uses require`);
      }
    },
  });

  return [...dependencies];
}

async function collectCssDependencies(root, filename) {
  const source = await readFile(filename, "utf8");
  let tree;
  try {
    tree = postcss.parse(source, { from: filename });
  } catch (error) {
    reject("CRP-PARSE", `${filename}: ${error.message}`);
  }
  const specifiers = [];
  tree.walkAtRules("import", (rule) => {
    const match = rule.params.match(
      /^(?:url\(\s*)?(?:["']([^"']+)["']|([^\s)]+))/,
    );
    const specifier = match?.[1] ?? match?.[2];
    if (!specifier?.startsWith(".")) return;
    specifiers.push(specifier);
  });
  tree.walkDecls((declaration) => {
    if (typeof declaration.value !== "string") return;
    for (const match of declaration.value.matchAll(CSS_URL_IMPORT)) {
      const raw = match[1];
      if (!raw?.startsWith(".")) continue;
      specifiers.push(raw);
    }
  });

  const dependencies = new Set();
  for (const specifier of specifiers) {
    dependencies.add(await resolveRelativeImport(root, filename, stripSpecifier(specifier)));
  }
  return [...dependencies];
}

export async function computeCurrentRouteSourceClosure({ repoRoot, entries }) {
  if (!Array.isArray(entries) || entries.length === 0) {
    reject("CRP-SCHEMA", "entries must be a non-empty array");
  }
  const root = path.resolve(repoRoot);
  const queue = [...entries];
  const visited = new Set();
  const manifest = [];

  for (let index = 0; index < queue.length; index += 1) {
    const safe = safeRelativePath(queue[index], `entries[${index}]`);
    const absolute = path.resolve(root, safe);
    await assertFile(root, absolute, safe);
    if (visited.has(safe)) continue;
    visited.add(safe);

    const extension = path.extname(safe);
    const bytes = await readFile(absolute);
    manifest.push({ path: safe, sha256: sha256(bytes) });

    let dependencies = [];
    if (JS_EXTENSIONS.has(extension)) {
      dependencies = await collectJsDependencies(root, absolute);
    } else if (extension === ".css") {
      dependencies = await collectCssDependencies(root, absolute);
    }

    for (const dependency of dependencies) {
      const normalized = safeRelativePath(dependency, `dependency from ${safe}`);
      if (!visited.has(normalized) && !queue.includes(normalized)) {
        queue.push(normalized);
      }
    }
  }

  return manifest.sort((left, right) =>
    left.path < right.path ? -1 : left.path > right.path ? 1 : 0,
  );
}

export function validateCurrentRouteProvenance(provenance) {
  exactFields(provenance, new Set(["schemaVersion", "assets"]), "provenance");
  if (provenance.schemaVersion !== CURRENT_ROUTE_PROVENANCE_SCHEMA_VERSION) {
    reject("CRP-SCHEMA", "schemaVersion must be 2");
  }
  if (!Array.isArray(provenance.assets) || provenance.assets.length === 0) {
    reject("CRP-SCHEMA", "assets must be a non-empty array");
  }

  const assets = [];
  let previous = null;
  const seen = new Set();
  for (const [index, raw] of provenance.assets.entries()) {
    exactFields(raw, new Set(["path", "sha256"]), `assets[${index}]`);
    const safe = safeRelativePath(raw.path, `assets[${index}].path`);
    if (seen.has(safe)) {
      reject("CRP-SCHEMA", `duplicate asset ${safe}`);
    }
    seen.add(safe);
    if (!SHA256.test(raw.sha256 ?? "")) {
      reject("CRP-SCHEMA", `${safe} must be a SHA-256`);
    }
    if (previous && previous >= safe) {
      reject("CRP-ORDER", "assets are not in strict path order");
    }
    previous = safe;
    assets.push({ path: safe, sha256: raw.sha256 });
  }
  return { schemaVersion: CURRENT_ROUTE_PROVENANCE_SCHEMA_VERSION, assets };
}

export async function verifyCurrentRouteSourceClosure({ repoRoot, provenance, entries }) {
  const validated = validateCurrentRouteProvenance(provenance);
  const closure = await computeCurrentRouteSourceClosure({
    repoRoot,
    entries,
  });

  const expectedByPath = new Map(validated.assets.map((entry) => [entry.path, entry.sha256]));
  const actualByPath = new Map(closure.map((entry) => [entry.path, entry.sha256]));

  for (const [assetPath] of expectedByPath) {
    if (!actualByPath.has(assetPath)) {
      reject("CRP-EXTRA", `provenance declares non-closure asset ${assetPath}`);
    }
  }
  for (const [assetPath] of actualByPath) {
    if (!expectedByPath.has(assetPath)) {
      reject("CRP-MISSING", `provenance does not declare ${assetPath}`);
    }
  }

  const expectedOrder = validated.assets.map((entry) => entry.path);
  const actualOrder = closure.map((entry) => entry.path);
  if (expectedOrder.join("\0") !== actualOrder.join("\0")) {
    reject("CRP-ORDER", "assets are not in canonical path order");
  }

  for (const [entryPath, expectedHash] of expectedByPath) {
    if (actualByPath.get(entryPath) !== expectedHash) {
      reject("CRP-HASH", `sha256 drift for ${entryPath}`);
    }
  }

  return {
    assets: closure,
  };
}

export function currentRouteSourceManifestSha256(provenanceFileBytes) {
  if (!(provenanceFileBytes instanceof Uint8Array)) {
    reject("CRP-SCHEMA", "provenance file bytes must be a byte array");
  }
  return sha256(provenanceFileBytes);
}
