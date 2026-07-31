import assert from "node:assert/strict";
import test from "node:test";
import { Module, register } from "node:module";
import { pathToFileURL } from "node:url";
import {
  createBrowserHostSender,
  decodeBrowserHostSnapshot,
} from "../src/host/browserHostCodec.js";

const TEST_NODE_MODULES = "/tmp/cu205b1g-docs-mocks-node_modules";
process.env.NODE_PATH = TEST_NODE_MODULES;
Module._initPaths();

const jsxLoaderSource = `
  import { readFile } from "node:fs/promises";
  import { createRequire, Module } from "node:module";
  import { pathToFileURL } from "node:url";

  const nodeModules = ${JSON.stringify("/tmp/cu205b1g-docs-mocks-node_modules")};
  process.env.NODE_PATH = nodeModules;
  Module._initPaths();
  const packageRequire = createRequire("file:///tmp/cu205b1i-inline-loader.cjs");
  const { transform } = packageRequire(
    nodeModules + "/esbuild/lib/main.js"
  );

  export async function resolve(specifier, context, nextResolve) {
    if (specifier.endsWith(".css")) {
      return {
        url: "data:text/javascript,export%20default%20%7B%7D%3B#" +
          encodeURIComponent(specifier),
        shortCircuit: true,
      };
    }
    if (specifier === "html-react-parser") {
      return {
        url: pathToFileURL(
          nodeModules + "/html-react-parser/esm/index.mjs"
        ).href,
        shortCircuit: true,
      };
    }
    if (
      !specifier.startsWith(".") &&
      !specifier.startsWith("/") &&
      !specifier.startsWith("file:") &&
      !specifier.startsWith("data:") &&
      !specifier.startsWith("node:")
    ) {
      return {
        url: pathToFileURL(
          packageRequire.resolve(specifier)
        ).href,
        shortCircuit: true,
      };
    }
    return nextResolve(specifier, context);
  }

  export async function load(url, context, nextLoad) {
    if (url.endsWith(".jsx")) {
      const source = await readFile(new URL(url), "utf8");
      const result = await transform(source, {
        format: "esm",
        jsx: "automatic",
        loader: "jsx",
        sourcefile: new URL(url).pathname,
        target: "es2022",
      });
      return { format: "module", source: result.code, shortCircuit: true };
    }
    return nextLoad(url, context);
  }
`;

register(
  `data:text/javascript,${encodeURIComponent(jsxLoaderSource)}`,
  import.meta.url,
);

const ReactModule = await import(
  pathToFileURL(`${TEST_NODE_MODULES}/react/index.js`).href
);
const React = ReactModule.default ?? ReactModule;
const { renderToStaticMarkup } = await import(
  pathToFileURL(`${TEST_NODE_MODULES}/react-dom/server.node.js`).href
);
const { parseDocument } = await import(
  pathToFileURL(`${TEST_NODE_MODULES}/htmlparser2/dist/index.js`).href
);
const { DiscoveryBrowserCandidate } = await import(
  "../src/candidates/DiscoveryBrowserCandidate.jsx"
);

function snapshot() {
  return {
    version: 1,
    direction: "host-to-web",
    role: "browser",
    instance_epoch: "7",
    sequence: "10",
    browser: {
      rectangle_source: {
        scope_ref: "catalog-scope-2",
        item_id: "rectangle",
      },
    },
  };
}

function catalog() {
  return {
    catalog_revision: 1,
    vocabularies: {
      scopes: [{
        id: "first-party-effects",
        label: "First-party effects",
        scope_ref: null,
      }],
      taxonomies: [
        { id: "effect", label: "Effect", scope_ref: "first-party-effects" },
        { id: "color", label: "Color", scope_ref: "first-party-effects" },
      ],
      providers: [{
        id: "built-in",
        label: "Built-in",
        scope_ref: "first-party-effects",
      }],
      packs: [],
      install_states: [{
        id: "installed",
        label: "Installed",
        scope_ref: "first-party-effects",
      }],
      impact_units: [],
      tags: [],
    },
    catalogs: [{
      scope_ref: "first-party-effects",
      items: [{
        item_id: "core.filter.opacity",
        display_name: "Opacity",
        taxonomy_refs: ["effect", "color"],
        provider_ref: "built-in",
        pack_ref: null,
        install_state_ref: "installed",
        preview_kind: "poster",
        impact: null,
        tag_refs: [],
      }],
    }],
  };
}

function catalogSnapshot(catalogInput) {
  const value = snapshot();
  value.browser.catalog = catalogInput;
  return value;
}

function opacityCatalogProjection() {
  return {
    itemId: "core.filter.opacity",
    name: "Opacity",
    category: { value: "effect", label: "Effect" },
    subtype: { value: "color", label: "Color" },
    mode: "installed",
    folder: "Effect Color",
    labels: "Effect Color",
    search: "Opacity core.filter.opacity Effect Color Built-in Installed",
    thumbnail: "poster",
    kind: "FX",
    state: undefined,
    identity: undefined,
    impact: undefined,
    pack: null,
    motion: false,
    tags: [],
    tagVisible: true,
  };
}

function renderBrowserCandidate(catalogProjection) {
  return renderToStaticMarkup(React.createElement(DiscoveryBrowserCandidate, {
    rectangleIdentity: {
      scope_ref: "catalog-scope-2",
      item_id: "rectangle",
    },
    onPlaceIntent: () => {},
    catalogProjection,
  }));
}

function findElement(root, predicate) {
  if (root.type === "tag" && predicate(root)) {
    return root;
  }
  for (const child of root.children ?? []) {
    const match = findElement(child, predicate);
    if (match) return match;
  }
  return null;
}

function collectElements(root, predicate, matches = []) {
  if (root.type === "tag" && predicate(root)) {
    matches.push(root);
  }
  for (const child of root.children ?? []) {
    collectElements(child, predicate, matches);
  }
  return matches;
}

function textContent(node) {
  if (node.type === "text") return node.data;
  return (node.children ?? []).map(textContent).join("");
}

function renderedPluginBrowser(catalogProjection) {
  const document = parseDocument(renderBrowserCandidate(catalogProjection));
  const browser = findElement(document, (node) => node.attribs?.id === "vism-browser");
  assert.ok(browser, "rendered #vism-browser must exist");
  const cards = collectElements(browser, (node) =>
    node.attribs?.class?.split(" ").includes("candidate-plugin-card"));
  const count = findElement(
    browser,
    (node) => node.attribs?.id === "plugin-result-count",
  );
  assert.ok(count, "rendered Plugin Results count must exist");
  return {
    itemIds: cards.map((card) => card.attribs["data-browser-item"]),
    count: Number(textContent(count)),
  };
}

test("decodes the closed Browser snapshot and emits one sequenced Place message", () => {
  const decoded = decodeBrowserHostSnapshot(snapshot());
  assert.equal(decoded.catalogProjection, undefined);
  const sent = [];
  const send = createBrowserHostSender(decoded, (message) => sent.push(message));
  send(Object.freeze({
    kind: "browser.place",
    source: Object.freeze({
      scope_ref: "catalog-scope-2",
      item_id: "rectangle",
    }),
  }));

  assert.equal(sent.length, 1);
  assert.deepEqual(JSON.parse(sent[0]), {
    version: 1,
    direction: "web-to-host",
    role: "browser",
    instance_epoch: "7",
    sequence: "11",
    kind: "browser.place",
    source: {
      scope_ref: "catalog-scope-2",
      item_id: "rectangle",
    },
  });
});

test("decodes the exact first-party Opacity PluginCard projection", () => {
  const decoded = decodeBrowserHostSnapshot(catalogSnapshot(catalog()));
  assert.deepEqual(decoded.catalogProjection, opacityCatalogProjection());
});

test("rejects malformed or mismatched present catalog projections", () => {
  for (const mutate of [
    (value) => { value.catalog_revision = 2; },
    (value) => { value.catalogs[0].scope_ref = "other"; },
    (value) => { value.catalogs[0].items[0].item_id = "other.item"; },
    (value) => { value.catalogs[0].items[0].display_name = "Darkness"; },
    (value) => { value.catalogs[0].items[0].taxonomy_refs.reverse(); },
    (value) => { value.catalogs[0].items[0].preview_kind = "motion"; },
    (value) => { value.catalogs[0].items[0].pack_ref = "pack"; },
    (value) => { value.catalogs[0].items[0].impact = { measures: [] }; },
    (value) => { value.catalogs[0].items[0].tag_refs = ["tag"]; },
    (value) => { value.vocabularies.scopes[0].label = "Effects"; },
    (value) => { value.vocabularies.taxonomies[1].label = "Colour"; },
    (value) => { value.vocabularies.taxonomies[0].scope_ref = null; },
    (value) => {
      value.vocabularies.providers[0].id = "bundled";
      value.catalogs[0].items[0].provider_ref = "bundled";
    },
    (value) => { value.vocabularies.providers[0].label = "Bundled"; },
    (value) => { value.vocabularies.providers[0].scope_ref = null; },
    (value) => {
      value.vocabularies.install_states[0].id = "ready";
      value.catalogs[0].items[0].install_state_ref = "ready";
    },
    (value) => { value.vocabularies.install_states[0].label = "Ready"; },
    (value) => { value.vocabularies.install_states[0].scope_ref = null; },
    (value) => { value.vocabularies.providers.push({
      id: "other",
      label: "Other",
      scope_ref: "first-party-effects",
    }); },
    (value) => { value.catalogs[0].items.push(structuredClone(value.catalogs[0].items[0])); },
  ]) {
    const value = catalog();
    mutate(value);
    assert.throws(() => decodeBrowserHostSnapshot(catalogSnapshot(value)));
  }
});

test("executes the Candidate fork and preserves absent three-card Results 3", () => {
  assert.deepEqual(renderedPluginBrowser(undefined), {
    itemIds: ["echo-bloom", "type-pulse", "fold-field"],
    count: 3,
  });
});

test("executes the Candidate fork and renders one projected Opacity PluginCard", () => {
  assert.deepEqual(renderedPluginBrowser(opacityCatalogProjection()), {
    itemIds: ["core.filter.opacity"],
    count: 1,
  });
});

test("rejects unknown, stale-shaped, oversized, and exhausted inputs", () => {
  for (const mutate of [
    (value) => { value.extra = true; },
    (value) => { value.version = 2; },
    (value) => { value.direction = "web-to-host"; },
    (value) => { value.role = "inspector"; },
    (value) => { value.instance_epoch = "07"; },
    (value) => { value.browser.rectangle_source.item_id = ""; },
    (value) => { value.browser.rectangle_source.scope_ref = "x".repeat(129); },
  ]) {
    const value = snapshot();
    mutate(value);
    assert.throws(() => decodeBrowserHostSnapshot(value));
  }

  const decoded = decodeBrowserHostSnapshot(snapshot());
  const send = createBrowserHostSender(decoded, () => {});
  assert.throws(() => send({
    kind: "browser.unknown",
    source: { scope_ref: "catalog-scope-2", item_id: "rectangle" },
  }));
  assert.throws(() => send({
    kind: "browser.place",
    source: { scope_ref: "catalog-scope-2", item_id: "rectangle", extra: true },
  }));

  const exhausted = decodeBrowserHostSnapshot({
    ...snapshot(),
    sequence: "18446744073709551615",
  });
  assert.throws(() =>
    createBrowserHostSender(exhausted, () => {})({
      kind: "browser.place",
      source: { scope_ref: "catalog-scope-2", item_id: "rectangle" },
    }));
});
