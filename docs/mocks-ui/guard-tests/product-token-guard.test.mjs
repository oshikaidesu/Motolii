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

const ROUTE_LEGACY_ALIAS_BY_ROLE = new Map([
  ["surface-app", "bg"],
  ["surface-panel", "panel"],
  ["surface-raised", "raised"],
  ["surface-hover", "hover"],
  ["border-default", "line"],
  ["border-strong", "line2"],
  ["text-primary", "ink"],
  ["text-secondary", "sub"],
  ["text-muted", "muted"],
  ["action-active", "active"],
  ["data", "data"],
  ["shape", "shape"],
  ["status-warning", "warning"],
  ["status-ok", "ok"],
  ["way-project", "way-project"],
  ["way-files", "way-files"],
  ["way-plugins", "way-plugins"],
  ["way-stage", "way-stage"],
  ["way-inspector", "way-inspector"],
  ["way-timeline", "way-timeline"],
]);

const ROUTE_ROLE_NAMES = [
  "surface-app",
  "surface-panel",
  "surface-raised",
  "surface-hover",
  "border-default",
  "border-strong",
  "text-primary",
  "text-secondary",
  "text-muted",
  "focus",
  "action-active",
  "data",
  "shape",
  "status-warning",
  "status-ok",
  "way-project",
  "way-files",
  "way-plugins",
  "way-stage",
  "way-inspector",
  "way-timeline",
];

const ROUTE_GENERATED_ROLE_NAMES = [...ROLE_SOURCES.keys()];

const ROUTE_LEGACY_COLOR_DECLARATIONS = [
  "bg",
  "panel",
  "raised",
  "hover",
  "line",
  "line2",
  "ink",
  "sub",
  "muted",
  "active",
  "data",
  "shape",
  "warning",
  "ok",
  "way-project",
  "way-files",
  "way-plugins",
  "way-stage",
  "way-inspector",
  "way-timeline",
];

function buildExpectedRouteAdapterSource() {
  const legacyDeclarations = [];
  for (const role of ROUTE_ROLE_NAMES) {
    const alias = ROUTE_LEGACY_ALIAS_BY_ROLE.get(role);
    if (alias) {
      legacyDeclarations.push(`  --${alias}: var(--motolii-color-${role});`);
    }
  }
  const mockDeclarations = ROUTE_ROLE_NAMES.map(
    (role) => `  --mock-role-${role}: var(--motolii-color-${role});`,
  );
  return [
    '@import "../../../../ui/motolii-tokens/generated/tokens.css";',
    "",
    ':root[data-motolii-theme="motolii-dark"]:has(#root > [data-fixture="plugin-browser-candidate"]) {',
    ...legacyDeclarations,
    ...mockDeclarations,
    "}",
    "",
  ].join("\n");
}

const ACCEPTED_ROUTE_LEGACY_REMOVALS = [
  `      --bg:#141414; --panel:#1a1a1a; --raised:#222222; --hover:#2c2c2c;
      --line:#3b3b3b; --line2:#686868; --ink:#f0f0f0; --sub:#c6c6c6; --muted:#929292;
`,
  `      --active:#d8b574; --data:#78b5b0; --shape:#aaa0d0; --warning:#e18a6d;
      --ok:#90b287; `,
  `      --way-project:#6eb3ae; --way-files:#83a8cf; --way-plugins:#9f9fcf;
      --way-stage:#bca072; --way-inspector:#8eb086; --way-timeline:#cc9587;
`,
];

function removeRequiredSpanOnce(source, span) {
  const firstIndex = source.indexOf(span);
  assert.notEqual(firstIndex, -1);
  assert.equal(source.indexOf(span, firstIndex + span.length), -1);
  return source.replace(span, "");
}

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

test("accepted route token adapter is exact", async () => {
  const adapterSource = await readFile(
    path.resolve(PRODUCT_ROOT, "../../docs/mocks-ui/src/tokens/accepted-route-product-tokens.css"),
    "utf8",
  );
  const mainSource = await readFile(
    path.resolve(PRODUCT_ROOT, "../../docs/mocks-ui/src/main.jsx"),
    "utf8",
  );
  const legacyHostBoundarySource = await readFile(
    path.resolve(PRODUCT_ROOT, "../../docs/mocks-ui/src/legacy/LegacyHostBoundaryScreen.jsx"),
    "utf8",
  );
  const legacySourceCode = await readFile(
    path.resolve(PRODUCT_ROOT, "../../docs/mocks-ui/src/legacy/legacySource.js"),
    "utf8",
  );
  const legacySourceHtml = await readFile(
    path.resolve(PRODUCT_ROOT, "../../docs/mocks/m3-vism-host-boundary.html"),
    "utf8",
  );
  const mockSource = await readFile(
    path.resolve(PRODUCT_ROOT, "../../docs/mocks-ui/src/tokens/mock-candidates.css"),
    "utf8",
  );
  const generatedSource = await readFile(
    path.join(PRODUCT_ROOT, "generated/tokens.css"),
    "utf8",
  );

  const adapterImport = 'import "./tokens/accepted-route-product-tokens.css";';
  const discoveryImport =
    'import { DiscoveryBrowserCandidate, EasingTriggerCandidate } from "@motolii/motolii-web";';
  const adapterImportIndex = mainSource.indexOf(adapterImport);
  const discoveryImportIndex = mainSource.indexOf(discoveryImport);
  assert.equal(adapterImportIndex >= 0, true);
  assert.equal(discoveryImportIndex >= 0, true);
  assert.equal(adapterImportIndex < discoveryImportIndex, true);

  assert.equal((mainSource.match(/productTokenConsumer:\s*true/g) || []).length, 1);
  const mainSourceLines = mainSource.split("\n");
  const pluginRouteLineIndex = mainSourceLines.findIndex((line) =>
    line.includes('"plugin-browser-candidate"'),
  );
  assert.equal(pluginRouteLineIndex >= 0, true);
  const pluginRouteEndLineIndex = mainSourceLines.findIndex(
    (line, index) => index > pluginRouteLineIndex && /^\s*".+?"\s*:\s*{/.test(line),
  );
  const pluginRouteSource =
    pluginRouteEndLineIndex === -1
      ? mainSourceLines.slice(pluginRouteLineIndex).join("\n")
      : mainSourceLines.slice(pluginRouteLineIndex, pluginRouteEndLineIndex).join("\n");
  assert.equal((pluginRouteSource.match(/archive:\s*true/g) || []).length, 0);
  assert.equal((mainSource.match(/productTokenConsumer:\s*false/g) || []).length, 0);
  assert.equal((mainSource.match(/data-motolii-theme/g) || []).length, 1);
  assert.equal(
    (
      mainSource.match(
        /document\.documentElement\.setAttribute\(\s*"data-motolii-theme",\s*"motolii-dark",?\s*\)/g,
      ) || []
    ).length,
    1,
  );
  assert.equal(/rootElement\.setAttribute|#root[^]*data-motolii-theme/.test(mainSource), false);

  const expectedAdapterSource = buildExpectedRouteAdapterSource();
  assert.equal(adapterSource, expectedAdapterSource);

  const declarationLines = adapterSource
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.startsWith("--"));
  assert.equal(declarationLines.length, 41);
  const routeAdapterDeclarations = new Map(
    [
      ...adapterSource.matchAll(
        /^\s*--([a-z0-9-]+)\s*:\s*var\(--motolii-color-([a-z0-9-]+)\)\s*;/gm,
      ),
    ].map((match) => [match[1], match[2]]),
  );
  assert.equal(routeAdapterDeclarations.size, 41);

  const expectedLegacyDeclarations = [];
  const expectedMockDeclarations = [];
  for (const role of ROUTE_ROLE_NAMES) {
    const legacyAlias = ROUTE_LEGACY_ALIAS_BY_ROLE.get(role);
    if (legacyAlias) {
      expectedLegacyDeclarations.push(legacyAlias);
      assert.equal(routeAdapterDeclarations.get(legacyAlias), role);
    }
    expectedMockDeclarations.push(`mock-role-${role}`);
    assert.equal(routeAdapterDeclarations.get(`mock-role-${role}`), role);
  }
  assert.deepEqual(
    [...routeAdapterDeclarations.keys()]
      .filter((alias) => !alias.startsWith("mock-role-"))
      .sort(),
    expectedLegacyDeclarations.sort(),
  );
  assert.deepEqual(
    [...routeAdapterDeclarations.keys()].filter((alias) => alias.startsWith("mock-role-")).sort(),
    expectedMockDeclarations.sort(),
  );

  const generated = cssVariables(generatedSource, "motolii-");
  assert.equal(generated.size, ROUTE_GENERATED_ROLE_NAMES.length);
  assert.deepEqual([...generated.keys()].sort(), [...ROUTE_GENERATED_ROLE_NAMES].sort());

  const rawLegacy = cssVariables(legacySourceHtml);
  const mock = cssVariables(mockSource, "mock-candidate-color-");
  for (const [role, [legacyAlias, mockAlias]] of ROLE_SOURCES) {
    const generatedValue = generated.get(role)?.replace(/ff$/, "");
    if (role !== "color-focus") {
      assert.equal(generatedValue, rawLegacy.get(legacyAlias), `${role} differs from legacy source`);
    }
    assert.equal(generatedValue, mock.get(mockAlias), `${role} differs from mock supplier`);
  }

  const legacyStyleMatch = legacySourceHtml.match(/<style>([\s\S]*?)<\/style>/i);
  assert.ok(legacyStyleMatch);
  const importLegacySource = async (sourceHtml) => {
    const executableSource = legacySourceCode.replace(
      /import sourceHtml from ["'][^"']+\?raw["'];/,
      `const sourceHtml = ${JSON.stringify(sourceHtml)};`,
    );
    return import(`data:text/javascript,${encodeURIComponent(executableSource)}`);
  };
  const legacySourceModule = await importLegacySource(legacySourceHtml);
  assert.equal(legacySourceModule.legacyStyle, legacyStyleMatch[1]);

  const expectedSanitizedLegacyStyle = ACCEPTED_ROUTE_LEGACY_REMOVALS.reduce(
    removeRequiredSpanOnce,
    legacySourceModule.legacyStyle,
  );
  assert.equal(expectedSanitizedLegacyStyle.includes("--mono"), true);
  assert.equal(expectedSanitizedLegacyStyle.includes("color-scheme"), true);
  for (const declaration of ROUTE_LEGACY_COLOR_DECLARATIONS) {
    assert.equal(expectedSanitizedLegacyStyle.includes(`--${declaration}:`), false);
  }
  assert.equal(expectedSanitizedLegacyStyle.includes("--object-1"), true);
  assert.equal(legacySourceModule.acceptedRouteLegacyStyle, expectedSanitizedLegacyStyle);
  for (const driftedSource of [
    legacySourceHtml.replace("--bg:#141414", "--bg: #141414"),
    legacySourceHtml.replace(
      "--bg:#141414; --panel:#1a1a1a;",
      "--panel:#1a1a1a; --bg:#141414;",
    ),
    legacySourceHtml.replace(
      "--line:#3b3b3b;",
      "--unexpected:#000000; --line:#3b3b3b;",
    ),
  ]) {
    await assert.rejects(
      importLegacySource(driftedSource),
      /Legacy host boundary palette span drifted/,
    );
  }

  assert.equal((legacyHostBoundarySource.match(/productTokenConsumer = false/g) || []).length, 1);
  const fixtureMatch = legacyHostBoundarySource.match(/function LegacyFixture\(([\s\S]*?)\)\s*\{/);
  assert.ok(fixtureMatch);
  assert.equal(/productTokenConsumer\s*=/.test(fixtureMatch[1]), false);
  assert.equal((legacyHostBoundarySource.match(/productTokenConsumer={productTokenConsumer}/g) || []).length, 1);
  const exportedFalseCount = legacyHostBoundarySource.split("productTokenConsumer =").length - 1;
  assert.equal(exportedFalseCount, 1);
});
