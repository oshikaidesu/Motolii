import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { readFile } from "node:fs/promises";
import { parse } from "@babel/parser";
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

function extractBrowserRouteFromTest(testAst) {
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

    if (value.type === "TemplateLiteral" && value.expressions.length === 0) {
      const cooked = value.quasis[0].value.cooked ?? value.quasis[0].value.raw;
      const match = routePattern.exec(cooked);
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

async function validateInventory(manifest, options = {}) {
  const { candidateAstSource, fixedSourceCommit = manifest.fixedSourceCommit } = options;

  assert.equal(Object.getPrototypeOf(manifest), Object.prototype);
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.task, "CU-0A03");
  assert.equal(manifest.scope, "browser-only-r0-slice");
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
  assert.equal(manifest.surfaces.length, 1);
  assert.equal(Array.isArray(manifest.behavioralTests), true);
  assert.equal(manifest.behavioralTests.length, 1);

  const surface = manifest.surfaces[0];
  ensureExactKeys(surface, [
    "id",
    "classification",
    "componentPath",
    "componentExport",
    "runtimeClosure",
    "localImports",
    "externalPackages",
  ]);

  assert.equal(surface.id, "browser");
  assert.equal(surface.classification, "react-direct-promotion");
  assert.equal(surface.componentPath, EXPECTED_BROWSER_SOURCE);
  assert.equal(surface.componentExport, "DiscoveryBrowserCandidate");
  assert.deepEqual(surface.externalPackages, EXPECTED_EXTERNAL_PACKAGES);

  assert.equal(Array.isArray(surface.runtimeClosure), true);
  assert.equal(surface.runtimeClosure.length, 3);
  assert.equal(Array.isArray(surface.localImports), true);
  assert.equal(surface.localImports.length, 2);

  const expectedRuntimeOrder = [
    EXPECTED_BROWSER_SOURCE,
    EXPECTED_CSS_SOURCE,
    EXPECTED_PATTERN_SOURCE,
  ];

  for (let index = 0; index < expectedRuntimeOrder.length; index += 1) {
    const expectedPath = expectedRuntimeOrder[index];
    const entry = surface.runtimeClosure[index];

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

  ensureExactKeys(surface.localImports[0], ["kind", "source", "specifiers"]);
  ensureExactKeys(surface.localImports[1], ["kind", "source", "specifiers"]);
  assert.equal(surface.localImports[0].kind, "css-side-effect");
  assert.equal(surface.localImports[0].source, EXPECTED_CSS_SOURCE);
  assert.deepEqual(surface.localImports[0].specifiers, []);
  assert.equal(surface.localImports[1].kind, "named");
  assert.equal(surface.localImports[1].source, EXPECTED_PATTERN_SOURCE);
  assert.deepEqual(surface.localImports[1].specifiers, EXPECTED_PATTERN_IMPORTS);

  ensureExactKeys(manifest.behavioralTests[0], ["path", "route"]);
  assert.equal(manifest.behavioralTests[0].path, EXPECTED_TEST_PATH);
  assert.equal(manifest.behavioralTests[0].route, EXPECTED_TEST_ROUTE);

  const candidateSource = candidateAstSource ?? (await readFile(
    absoluteFromRelative(surface.componentPath),
    "utf8",
  ));
  const candidateAst = parseModule(candidateSource);
  const candidateExports = collectNamedExports(candidateAst);
  assert.ok(candidateExports.has("DiscoveryBrowserCandidate"));

  const { localImports, externalPackages } = collectCandidateImports(
    candidateAst,
    absoluteFromRelative(surface.componentPath),
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

  const testSource = await readFile(absoluteFromRelative(manifest.behavioralTests[0].path), "utf8");
  const testAst = parseModule(testSource);
  const parsedRoutes = extractBrowserRouteFromTest(testAst);
  assert.deepEqual(parsedRoutes, [EXPECTED_TEST_ROUTE]);
}

test("accepts exact Browser-only R0 manifest and fixed-commit evidence", async () => {
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

  const reordered = {
    ...manifest,
    surfaces: [
      {
        ...manifest.surfaces[0],
        runtimeClosure: [
          manifest.surfaces[0].runtimeClosure[0],
          manifest.surfaces[0].runtimeClosure[2],
          manifest.surfaces[0].runtimeClosure[1],
        ],
      },
    ],
  };
  await assert.rejects(async () => {
    await validateInventory(reordered);
  });

  const missing = {
    ...manifest,
    surfaces: [{
      ...manifest.surfaces[0],
      runtimeClosure: manifest.surfaces[0].runtimeClosure.slice(0, 2),
    }],
  };
  await assert.rejects(async () => {
    await validateInventory(missing);
  });

  const hashMismatch = {
    ...manifest,
    surfaces: [{
      ...manifest.surfaces[0],
      runtimeClosure: manifest.surfaces[0].runtimeClosure.map((entry, index) =>
        index === 1 ? { ...entry, sha256: "0".repeat(64) } : entry,
      ),
    }],
  };
  await assert.rejects(async () => {
    await validateInventory(hashMismatch);
  });
});

test("rejects extra runtime closure entry", async () => {
  const manifest = await manifestFromDisk();
  const extra = {
    ...manifest,
    surfaces: [{
      ...manifest.surfaces[0],
      runtimeClosure: [
        ...manifest.surfaces[0].runtimeClosure,
        {
          path: EXPECTED_PATTERN_SOURCE,
          role: "runtime",
          sha256: EXPECTED_RUNTIME_HASHES[EXPECTED_PATTERN_SOURCE],
        },
      ],
    }],
  };
  await assert.rejects(async () => {
    await validateInventory(extra);
  });
});

test("rejects missing or wrong component export and non-browser surface", async () => {
  const manifest = await manifestFromDisk();

  const wrongExport = {
    ...manifest,
    surfaces: [{
      ...manifest.surfaces[0],
      componentExport: "BrowserCandidate",
    }],
  };
  await assert.rejects(async () => {
    await validateInventory(wrongExport);
  });

  const nonBrowserSurface = {
    ...manifest,
    surfaces: [{
      ...manifest.surfaces[0],
      id: "inspector",
    }],
  };
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
  const withLegacy = {
    ...manifest,
    surfaces: [{
      ...manifest.surfaces[0],
      runtimeClosure: manifest.surfaces[0].runtimeClosure.map((entry) =>
        entry.path === EXPECTED_PATTERN_SOURCE
          ? {
            ...entry,
            path: "docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx",
            sha256: "0".repeat(64),
          }
          : entry,
      ),
    }],
  };
  await assert.rejects(async () => {
    await validateInventory(withLegacy);
  });
});

test("rejects missing or wrong test evidence route", async () => {
  const manifest = await manifestFromDisk();

  const missingPath = {
    ...manifest,
    behavioralTests: [{
      ...manifest.behavioralTests[0],
      path: "docs/mocks-ui/tests/browser-candidate.spec.missing.js",
    }],
  };
  await assert.rejects(async () => {
    await validateInventory(missingPath);
  });

  const wrongRoute = {
    ...manifest,
    behavioralTests: [{
      ...manifest.behavioralTests[0],
      route: "plugin-browser",
    }],
  };
  await assert.rejects(async () => {
    await validateInventory(wrongRoute);
  });
});
