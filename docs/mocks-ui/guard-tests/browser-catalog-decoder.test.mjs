import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { decodeBrowserCatalog } from "../../../ui/motolii-web/src/read-model/browserCatalogDecoder.js";

const guardDir = dirname(fileURLToPath(import.meta.url));
const mocksUiRoot = join(guardDir, "..");
const repoRoot = join(mocksUiRoot, "..", "..");

const PARTS_PATH = join(mocksUiRoot, "fixtures/browser-catalog-parts.json");
const DECODER_PATH = join(repoRoot, "ui/motolii-web/src/read-model/browserCatalogDecoder.js");
const INDEX_PATH = join(repoRoot, "ui/motolii-web/src/index.js");

const AUTHORITY_SHA256 = {
  "ui/motolii-web/src/index.js":
    "a2ec126a21dd4637fbe90480460d47e7c2a3258fdc37dd7cf1d19746c6224469",
  "docs/mocks-ui/package.json":
    "d058d3c84d7b7cf688b576d6a5da32820b65405bf78ea06363380091a88b0cf6",
  "ui/motolii-web/src/candidates/DiscoveryBrowserCandidate.jsx":
    "e05b8636d9d07d9cd75612b1025474e34eeec37ef673b377a79fddb5773e42f9",
  "ui/motolii-web/src/candidates/discovery-browser-candidate.css":
    "1dcb6afc3c16907366f6d73ed7cfb1b04c8cea872d169e959ead49b6c6cedccd",
  "ui/motolii-web/src/patterns/DiscoveryBrowser.jsx":
    "1d996ad66dba3ff7fb36cf811ce8d22faec1fee271a2dd5349d953a7cf89a2ea",
  "ui/motolii-web/source-provenance.json":
    "1fb9c32922e37bffededf9e374741f7f6b4aaf5b56aa07204374d803b73a54a6",
};

const FORBIDDEN_KEYS = [
  "fill_color",
  "stroke_color",
  "z_occlusion",
  "occlusion_mode",
  "depth_z",
  "bake_point",
  "composite_bake",
  "driver_route",
  "driver_routes",
  "applied_plugin_history",
  "availability",
  "availability_lifecycle",
  "effect_description",
  "input_socket_label",
  "socket_type_tag",
  "link_label",
  "link_target_label",
  "primary_selection",
  "selected_object",
  "editing_effect",
  "at_key",
];

const MIRRORS = [
  "docs/implementation-ledger.md",
  "docs/README.md",
  "docs/specs/M3-ui-integration.md",
  "docs/decision-index.md",
  "docs/reviews/2026-07-22-m3-comfortable-use-granulation.md",
  "docs/reviews/2026-07-24-m3-vertical-slice-execution-decision.md",
];

const DO_TOKEN = /`(?:CORE \/ )?DO`/;
const DONE_TOKEN = /`(?:CORE \/ )?DONE`/;
const SPLIT_TOKEN = /`(?:CORE \/ )?SPLIT`/;

function selectStaleBrowserDecoderLaneRows(ledgerText) {
  const laneSection = ledgerText.split("## 現在の並列レーン")[1]?.split("##")[0] ?? "";
  return laneSection
    .split("\n")
    .filter((line) => line.includes("|"))
    .filter((line) => {
      const cells = line.split("|").map((cell) => cell.trim());
      return (
        cells[1] === "PRODUCT-ASSET" &&
        cells[2] === "CU-0A08BP" &&
        DO_TOKEN.test(cells[4] ?? "")
      );
    });
}

const STALE_PROSE_LITERAL = "`CU-0A08BP`（`DO`）";

function selectStaleBrowserDecoderProseLines(text) {
  return text
    .split("\n")
    .filter((line) => line.includes(STALE_PROSE_LITERAL));
}

function sha256File(relPath) {
  const abs = join(repoRoot, relPath);
  return createHash("sha256").update(readFileSync(abs)).digest("hex");
}

function loadFixture(overrides = {}) {
  const base = JSON.parse(readFileSync(PARTS_PATH, "utf8"));
  return structuredClone({ ...base, ...overrides });
}

function collectKeys(value, keys = new Set()) {
  if (value === null || typeof value !== "object") {
    return keys;
  }
  if (Array.isArray(value)) {
    for (const entry of value) {
      collectKeys(entry, keys);
    }
    return keys;
  }
  for (const key of Object.keys(value)) {
    keys.add(key);
    collectKeys(value[key], keys);
  }
  return keys;
}

function assertFamily(fn, family) {
  assert.throws(fn, (err) => {
    assert.ok(err instanceof TypeError);
    assert.match(err.message, new RegExp(`^${family}: `));
    return true;
  });
}

function assertFamilyOwner(fn, family, ownerPart) {
  assert.throws(fn, (err) => {
    assert.ok(err instanceof TypeError);
    assert.match(err.message, new RegExp(`^${family}: `));
    assert.match(err.message, new RegExp(ownerPart));
    return true;
  });
}

function assertContainersNotAliased(input, output, path = "root") {
  if (input === null || output === null) {
    return;
  }
  if (typeof input !== "object" || typeof output !== "object") {
    return;
  }
  assert.notEqual(input, output, path);
  if (Array.isArray(input) && Array.isArray(output)) {
    for (let i = 0; i < input.length; i += 1) {
      assertContainersNotAliased(input[i], output[i], `${path}[${i}]`);
    }
    return;
  }
  if (!Array.isArray(input) && !Array.isArray(output)) {
    for (const key of Object.keys(input)) {
      assertContainersNotAliased(input[key], output[key], `${path}.${key}`);
    }
  }
}

function findVocabEntry(fixture, table, id) {
  return fixture.vocabularies[table].find((e) => e.id === id);
}

test("fixture decodes to deepStrictEqual output", () => {
  const input = loadFixture();
  const out = decodeBrowserCatalog(input);
  assert.deepEqual(out, input);
});

test("top-level and nested key order preserved", () => {
  const input = loadFixture();
  const out = decodeBrowserCatalog(input);
  assert.deepEqual(Object.keys(out), ["catalog_revision", "vocabularies", "catalogs"]);
  assert.deepEqual(Object.keys(out.vocabularies), [
    "scopes",
    "taxonomies",
    "providers",
    "packs",
    "install_states",
    "impact_units",
    "tags",
  ]);
  assert.deepEqual(Object.keys(out.vocabularies.scopes[0]), ["id", "label", "scope_ref"]);
  assert.deepEqual(Object.keys(out.catalogs[0].items[0]), [
    "item_id",
    "display_name",
    "taxonomy_refs",
    "provider_ref",
    "pack_ref",
    "install_state_ref",
    "preview_kind",
    "impact",
    "tag_refs",
  ]);
});

test("catalog and item counts", () => {
  const out = decodeBrowserCatalog(loadFixture());
  assert.equal(out.catalogs.length, 2);
  assert.equal(out.catalogs[0].items.length, 3);
  assert.equal(out.catalogs[1].items.length, 8);
  let total = 0;
  for (const c of out.catalogs) {
    total += c.items.length;
  }
  assert.equal(total, 11);
});

test("type-pulse identity is per scope_ref", () => {
  const out = decodeBrowserCatalog(loadFixture());
  const ids = out.catalogs.map((c) => ({
    scope: c.scope_ref,
    pulse: c.items.find((i) => i.item_id === "type-pulse"),
  }));
  assert.equal(ids.length, 2);
  assert.ok(ids.every((x) => x.pulse));
});

test("taxonomy-d global scope_ref null accepted from catalog-scope-2", () => {
  const out = decodeBrowserCatalog(loadFixture());
  const glyph = out.catalogs[1].items.find((i) => i.item_id === "glyph-current");
  assert.deepEqual(glyph.taxonomy_refs, ["taxonomy-c", "taxonomy-d"]);
});

test("decode does not mutate input", () => {
  const input = loadFixture();
  const before = structuredClone(input);
  decodeBrowserCatalog(input);
  assert.deepEqual(input, before);
});

test("deep non-aliasing clone", () => {
  const input = loadFixture();
  const out = decodeBrowserCatalog(input);
  assertContainersNotAliased(input, out);
});

test("forbidden output keys absent", () => {
  const out = decodeBrowserCatalog(loadFixture());
  const keys = collectKeys(out);
  for (const forbidden of FORBIDDEN_KEYS) {
    assert.equal(keys.has(forbidden), false, forbidden);
  }
});

test("labels verbatim without semantic branching", () => {
  const input = loadFixture();
  const variants = ["", "   ", "null", "Unknown", "x".repeat(1024)];
  for (const label of variants) {
    const clone = structuredClone(input);
    for (const table of Object.keys(clone.vocabularies)) {
      for (const entry of clone.vocabularies[table]) {
        entry.label = label;
      }
    }
    const out = decodeBrowserCatalog(clone);
    assert.equal(out.vocabularies.scopes[0].label, label);
  }
});

test("B12 display_name empty and null preserved", () => {
  const empty = loadFixture();
  empty.catalogs[0].items[0].display_name = "";
  assert.equal(decodeBrowserCatalog(empty).catalogs[0].items[0].display_name, "");
  const nul = loadFixture();
  nul.catalogs[0].items[0].display_name = null;
  assert.equal(decodeBrowserCatalog(nul).catalogs[0].items[0].display_name, null);
  const src = readFileSync(DECODER_PATH, "utf8");
  assert.doesNotMatch(src, /\bUnknown\b/);
});

test("public surface and authority hashes", () => {
  const src = readFileSync(DECODER_PATH, "utf8");
  assert.doesNotMatch(src, /^import /m);
  assert.doesNotMatch(src, /\brequire\s*\(/);
  assert.doesNotMatch(src, /export\s+default\b/);
  const exports = [...src.matchAll(/export\s*\{\s*([^}]+)\s*\}/g)];
  assert.equal(exports.length, 1);
  assert.match(exports[0][1], /decodeBrowserCatalog/);
  assert.equal(sha256File("ui/motolii-web/src/index.js"), AUTHORITY_SHA256["ui/motolii-web/src/index.js"]);
  assert.doesNotMatch(readFileSync(INDEX_PATH, "utf8"), /browserCatalogDecoder/);
  assert.equal(sha256File("docs/mocks-ui/package.json"), AUTHORITY_SHA256["docs/mocks-ui/package.json"]);
});

test("B8 string absent from decoder source", () => {
  const src = readFileSync(DECODER_PATH, "utf8");
  assert.doesNotMatch(src, /\bB8\b/);
});

test("React source bytes unchanged", () => {
  for (const [rel, expected] of Object.entries(AUTHORITY_SHA256)) {
    if (rel.startsWith("ui/")) {
      assert.equal(sha256File(rel), expected, rel);
    }
  }
  const prov = readFileSync(join(repoRoot, "ui/motolii-web/source-provenance.json"), "utf8");
  assert.match(prov, /56c318edcddab7cf95d263cc2f7dd2b4e6791134/);
});

test("B1 catalog_revision variants", () => {
  for (const value of ["1", 2, 1.5, Number.NaN, Number.POSITIVE_INFINITY, null, true, {}, []]) {
    const input = loadFixture();
    input.catalog_revision = value;
    assertFamily(() => decodeBrowserCatalog(input), "B1");
  }
});

test("B2 top-level key violations", () => {
  const missing = loadFixture();
  delete missing.vocabularies;
  assertFamily(() => decodeBrowserCatalog(missing), "B2");
  const extra = loadFixture();
  extra.items = [];
  assertFamily(() => decodeBrowserCatalog(extra), "B2");
});

test("B3 forbidden and unknown keys", () => {
  for (const key of FORBIDDEN_KEYS) {
    const input = loadFixture();
    input.catalogs[0].items[0][key] = "x";
    assertFamily(() => decodeBrowserCatalog(input), "B3");
  }
  const vocabExtra = loadFixture();
  vocabExtra.vocabularies.scopes[0].extra = 1;
  assertFamily(() => decodeBrowserCatalog(vocabExtra), "B3");
  const catalogExtra = loadFixture();
  catalogExtra.catalogs[0].extra = 1;
  assertFamily(() => decodeBrowserCatalog(catalogExtra), "B3");
});

test("B4 preview_kind violations", () => {
  for (const value of ["static", "", null, 1, {}]) {
    const input = loadFixture();
    input.catalogs[0].items[0].preview_kind = value;
    assertFamily(() => decodeBrowserCatalog(input), "B4");
  }
});

test("B5 duplicate ids", () => {
  const dupTax = loadFixture();
  dupTax.vocabularies.taxonomies.push(structuredClone(dupTax.vocabularies.taxonomies[0]));
  assertFamily(() => decodeBrowserCatalog(dupTax), "B5");
  const dupScope = loadFixture();
  dupScope.catalogs[1].scope_ref = dupScope.catalogs[0].scope_ref;
  assertFamily(() => decodeBrowserCatalog(dupScope), "B5");
  const dupItem = loadFixture();
  dupItem.catalogs[0].items.push(structuredClone(dupItem.catalogs[0].items[0]));
  assertFamily(() => decodeBrowserCatalog(dupItem), "B5");
});

test("B6 dangling refs all seven paths", () => {
  const cases = [
    (i) => { i.catalogs[0].items[0].taxonomy_refs[0] = "missing-tax"; },
    (i) => { i.catalogs[1].items[0].provider_ref = "missing-provider"; },
    (i) => { i.catalogs[0].items[0].pack_ref = "missing-pack"; },
    (i) => { i.catalogs[0].items[2].install_state_ref = "missing-install"; },
    (i) => { i.catalogs[0].items[0].tag_refs[0] = "missing-tag"; },
    (i) => { i.catalogs[1].items.find((x) => x.item_id === "type-pulse").impact.measures[0].unit_ref = "missing-unit"; },
    (i) => { i.catalogs[0].scope_ref = "missing-scope"; },
  ];
  for (const mutate of cases) {
    const input = loadFixture();
    mutate(input);
    assertFamily(() => decodeBrowserCatalog(input), "B6");
  }
});

const B7_CASES = [
  {
    name: "taxonomy_refs",
    apply: (i) => { i.catalogs[1].items[0].taxonomy_refs[0] = "taxonomy-a"; },
    b6: (i) => { i.catalogs[1].items[0].taxonomy_refs[0] = "taxonomy-missing"; },
    table: "taxonomies",
    entryId: "taxonomy-a",
    matchScope: "catalog-scope-1",
    catalogScope: "catalog-scope-2",
  },
  {
    name: "provider_ref",
    apply: (i) => { findVocabEntry(i, "providers", "provider-a").scope_ref = "catalog-scope-1"; },
    b6: (i) => { i.catalogs[1].items[0].provider_ref = "provider-missing"; },
    table: "providers",
    entryId: "provider-a",
    matchScope: "catalog-scope-2",
    catalogScope: "catalog-scope-2",
  },
  {
    name: "pack_ref",
    apply: (i) => { findVocabEntry(i, "packs", "motion-kit-alpha").scope_ref = "catalog-scope-2"; },
    b6: (i) => { i.catalogs[0].items[0].pack_ref = "pack-missing"; },
    table: "packs",
    entryId: "motion-kit-alpha",
    matchScope: "catalog-scope-1",
    catalogScope: "catalog-scope-1",
  },
  {
    name: "install_state_ref",
    apply: (i) => { findVocabEntry(i, "install_states", "install-state-a").scope_ref = "catalog-scope-2"; },
    b6: (i) => { i.catalogs[0].items[2].install_state_ref = "install-missing"; },
    table: "install_states",
    entryId: "install-state-a",
    matchScope: "catalog-scope-1",
    catalogScope: "catalog-scope-1",
  },
  {
    name: "tag_refs",
    apply: (i) => { findVocabEntry(i, "tags", "go-to").scope_ref = "catalog-scope-2"; },
    b6: (i) => { i.catalogs[0].items[0].tag_refs[0] = "tag-missing"; },
    table: "tags",
    entryId: "go-to",
    matchScope: "catalog-scope-1",
    catalogScope: "catalog-scope-1",
  },
  {
    name: "unit_ref",
    apply: (i) => { findVocabEntry(i, "impact_units", "impact-unit-a").scope_ref = "catalog-scope-1"; },
    b6: (i) => {
      const item = i.catalogs[1].items.find((x) => x.item_id === "type-pulse");
      item.impact.measures[0].unit_ref = "unit-missing";
    },
    table: "impact_units",
    entryId: "impact-unit-a",
    matchScope: "catalog-scope-2",
    catalogScope: "catalog-scope-2",
  },
];

for (const bc of B7_CASES) {
  test(`B7 cross-scope ${bc.name}`, () => {
    const input = loadFixture();
    bc.apply(input);
    assertFamily(() => decodeBrowserCatalog(input), "B7");
  });
  test(`B6 pair for ${bc.name} not B7`, () => {
    const input = loadFixture();
    bc.b6(input);
    assert.throws(() => decodeBrowserCatalog(input), (err) => {
      assert.match(err.message, /^B6: /);
      assert.doesNotMatch(err.message, /^B7: /);
      return true;
    });
  });
  test(`scope_ref null accept ${bc.name}`, () => {
    const input = loadFixture();
    if (bc.name === "taxonomy_refs") {
      input.catalogs[1].items[0].taxonomy_refs = ["taxonomy-d"];
      assert.equal(findVocabEntry(input, "taxonomies", "taxonomy-d").scope_ref, null);
      decodeBrowserCatalog(input);
      return;
    }
    const entry = findVocabEntry(input, bc.table, bc.entryId);
    assert.equal(entry.scope_ref, null);
    if (bc.name === "provider_ref") {
      input.catalogs[1].items[0].provider_ref = bc.entryId;
    }
    if (bc.name === "pack_ref") {
      input.catalogs[0].items[0].pack_ref = bc.entryId;
    }
    if (bc.name === "install_state_ref") {
      input.catalogs[0].items[2].install_state_ref = bc.entryId;
    }
    if (bc.name === "tag_refs") {
      input.catalogs[0].items[0].tag_refs = [bc.entryId];
    }
    if (bc.name === "unit_ref") {
      const pulse = input.catalogs[1].items.find((x) => x.item_id === "type-pulse");
      pulse.impact.measures[0].unit_ref = bc.entryId;
    }
    decodeBrowserCatalog(input);
  });
  test(`matching scope accept ${bc.name}`, () => {
    const input = loadFixture();
    const entry = findVocabEntry(input, bc.table, bc.entryId);
    const originalScope = entry.scope_ref;
    if (bc.name === "taxonomy_refs") {
      const base = loadFixture();
      const taxA = findVocabEntry(input, "taxonomies", "taxonomy-a");
      const taxOriginal = taxA.scope_ref;
      taxA.scope_ref = "catalog-scope-2";
      for (const item of input.catalogs[0].items) {
        if (item.taxonomy_refs === null) {
          continue;
        }
        item.taxonomy_refs = item.taxonomy_refs.filter((ref) => ref !== "taxonomy-a");
        if (item.taxonomy_refs.length === 0) {
          item.taxonomy_refs = ["taxonomy-b"];
        }
      }
      const rect = input.catalogs[1].items.find((x) => x.item_id === "rectangle");
      rect.taxonomy_refs = ["taxonomy-a"];
      assert.notEqual(taxOriginal, "catalog-scope-2");
      assert.notDeepEqual(input, base);
      decodeBrowserCatalog(input);
      return;
    }
    entry.scope_ref = bc.matchScope;
    if (bc.name === "pack_ref") {
      for (const item of input.catalogs[1].items) {
        if (item.pack_ref === "motion-kit-alpha") {
          item.pack_ref = null;
        }
      }
    }
    if (bc.name === "unit_ref") {
      input.catalogs[0].items[1].impact = null;
    }
    if (bc.name === "provider_ref") {
      input.catalogs[1].items[0].provider_ref = bc.entryId;
    }
    assert.notEqual(originalScope, entry.scope_ref, `anti-no-op ${bc.name}`);
    decodeBrowserCatalog(input);
  });
}

test("owner v73 vocab id null is B14 not B13", () => {
  const input = loadFixture();
  input.vocabularies.scopes[0].id = null;
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B14", /vocabularies\.scopes\[0\]\.id/);
});

test("owner v73 vocab label null is B14 not B13", () => {
  const input = loadFixture();
  input.vocabularies.scopes[0].label = null;
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B14", /vocabularies\.scopes\[0\]\.label/);
});

test("owner v73 vocab id null loses to label 1025 B10", () => {
  const input = loadFixture();
  input.vocabularies.scopes[0].id = null;
  input.vocabularies.scopes[0].label = "x".repeat(1025);
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B10", /vocabularies\.scopes\[0\]\.label/);
});

test("owner v73 impact empty object is B11 missing measures", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].impact = {};
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B11", /catalogs\[0\]\.items\[0\]\.impact/);
});

test("owner v73 impact unknown key only is B11 missing measures", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].impact = { x: 1 };
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B11", /catalogs\[0\]\.items\[0\]\.impact/);
});

test("owner v73 impact unknown key beats bad measures B3", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].impact = { x: 1, measures: "bad" };
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B3", /catalogs\[0\]\.items\[0\]\.impact\.x/);
});

test("owner v73 impact extra key plus 17 measures item-owned B10", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].impact = {
    x: 1,
    measures: Array.from({ length: 17 }, () => ({ amount: 1, unit_ref: "impact-unit-a" })),
  };
  assertFamilyOwner(
    () => decodeBrowserCatalog(input),
    "B10",
    /catalogs\[0\]\.items\[0\]\.impact\.measures/,
  );
});

function assertInputKeyOrderMirrored(input, output, path = "root") {
  if (input === null || output === null || typeof input !== "object") {
    return;
  }
  if (Array.isArray(input)) {
    for (let i = 0; i < input.length; i += 1) {
      assertInputKeyOrderMirrored(input[i], output[i], `${path}[${i}]`);
    }
    return;
  }
  assert.deepEqual(Object.keys(output), Object.keys(input), path);
  for (const key of Object.keys(input)) {
    assertInputKeyOrderMirrored(input[key], output[key], `${path}.${key}`);
  }
}

test("v73 reordered keys preserved top-level and nested", () => {
  const input = loadFixture();
  const reorderedRoot = {};
  reorderedRoot.catalogs = input.catalogs;
  reorderedRoot.vocabularies = input.vocabularies;
  reorderedRoot.catalog_revision = input.catalog_revision;
  const voc = reorderedRoot.vocabularies;
  const scopes0 = voc.scopes[0];
  voc.scopes[0] = { scope_ref: scopes0.scope_ref, label: scopes0.label, id: scopes0.id };
  const item0 = reorderedRoot.catalogs[0].items[0];
  reorderedRoot.catalogs[0].items[0] = {
    tag_refs: item0.tag_refs,
    impact: item0.impact,
    preview_kind: item0.preview_kind,
    install_state_ref: item0.install_state_ref,
    pack_ref: item0.pack_ref,
    provider_ref: item0.provider_ref,
    taxonomy_refs: item0.taxonomy_refs,
    display_name: item0.display_name,
    item_id: item0.item_id,
  };
  const pulse = reorderedRoot.catalogs[1].items.find((x) => x.item_id === "type-pulse");
  const impact = pulse.impact;
  pulse.impact = {
    measures: impact.measures.map((m) => ({ unit_ref: m.unit_ref, amount: m.amount })),
  };
  const measure0 = pulse.impact.measures[0];
  pulse.impact.measures[0] = { unit_ref: measure0.unit_ref, amount: measure0.amount };
  const out = decodeBrowserCatalog(reorderedRoot);
  assertInputKeyOrderMirrored(reorderedRoot, out);
  assertContainersNotAliased(reorderedRoot, out);
});

test("owner v72 taxonomy number beats preview_kind B4", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].taxonomy_refs[0] = 1;
  input.catalogs[0].items[0].preview_kind = "static";
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B14", /taxonomy_refs\[0\]/);
});

test("owner v72 taxonomy object is B13", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].taxonomy_refs[0] = {};
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B13", /taxonomy_refs\[0\]/);
});

test("owner v72 tag_refs array element is B13", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].tag_refs[0] = [];
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B13", /tag_refs\[0\]/);
});

test("owner v72 item_id number beats display_name number", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].item_id = 1;
  input.catalogs[0].items[0].display_name = 1;
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B14", /\.item_id/);
});

test("owner v72 display_name object and taxonomy_refs string", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].display_name = {};
  input.catalogs[0].items[0].taxonomy_refs = "not-array";
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B13", /display_name/);
});

test("owner v72 provider pack install 129 byte refs", () => {
  const longRef = "€".repeat(43);
  for (const field of ["provider_ref", "pack_ref", "install_state_ref"]) {
    const input = loadFixture();
    input.catalogs[1].items[0][field] = longRef;
    assertFamilyOwner(() => decodeBrowserCatalog(input), "B10", new RegExp(`${field}`));
  }
});

test("owner v72 provider_ref 128 bytes accepted", () => {
  const input = loadFixture();
  const id128 = "a".repeat(128);
  input.vocabularies.providers.push({ id: id128, label: id128, scope_ref: null });
  input.catalogs[1].items[0].provider_ref = id128;
  decodeBrowserCatalog(input);
});

test("owner v72 taxonomy empty beats provider empty", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].taxonomy_refs[0] = "";
  input.catalogs[0].items[0].provider_ref = "";
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B15", /taxonomy_refs\[0\]/);
});

test("owner v72 taxonomy 129 bytes beats tag_refs length 65", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].taxonomy_refs = ["€".repeat(43)];
  input.catalogs[0].items[0].tag_refs = Array.from({ length: 65 }, (_, i) => `v72-tag-${i}`);
  for (let i = 0; i < 65; i += 1) {
    input.vocabularies.tags.push({
      id: `v72-tag-${i}`,
      label: `v72-tag-${i}`,
      scope_ref: null,
    });
  }
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B10", /taxonomy_refs\[0\]/);
});

test("owner v72 earlier provider dangling beats later catalog scope", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].provider_ref = "missing-prov-v72";
  input.catalogs[1].scope_ref = "missing-scope-v72";
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B6", /catalogs\[0\].*provider_ref/);
});

test("owner v72 same item unit dangling beats tag dangling", () => {
  const input = loadFixture();
  const pulse = input.catalogs[1].items.find((x) => x.item_id === "type-pulse");
  pulse.impact.measures[0].unit_ref = "missing-unit-v72";
  pulse.tag_refs[0] = "missing-tag-v72";
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B6", /impact\.measures\[0\]\.unit_ref/);
});

test("owner v72 same item unit cross-scope beats tag cross-scope", () => {
  const input = loadFixture();
  findVocabEntry(input, "impact_units", "impact-unit-a").scope_ref = "catalog-scope-2";
  findVocabEntry(input, "tags", "kinetic").scope_ref = "catalog-scope-2";
  const pulse = input.catalogs[0].items.find((x) => x.item_id === "type-pulse");
  assert.ok(pulse.impact);
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B7", /catalogs\[0\].*impact\.measures\[0\]\.unit_ref/);
});

test("owner v72 preview_kind omitted is B11", () => {
  const input = loadFixture();
  delete input.catalogs[0].items[0].preview_kind;
  assertFamily(() => decodeBrowserCatalog(input), "B11");
});

test("owner v72 measures 17 with item_id number", () => {
  const input = loadFixture();
  const item = input.catalogs[0].items[0];
  item.item_id = 1;
  item.impact = {
    measures: Array.from({ length: 17 }, () => ({ amount: 1, unit_ref: "impact-unit-a" })),
  };
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B10", /catalogs\[0\]\.items\[0\]\.impact\.measures/);
});

test("owner v72 measures 17 with item_id empty", () => {
  const input = loadFixture();
  const item = input.catalogs[0].items[0];
  item.item_id = "";
  item.impact = {
    measures: Array.from({ length: 17 }, () => ({ amount: 1, unit_ref: "impact-unit-a" })),
  };
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B10", /catalogs\[0\]\.items\[0\]\.impact\.measures/);
});

test("owner v72 measures 17 with tag_refs number", () => {
  const input = loadFixture();
  const item = input.catalogs[0].items[0];
  item.tag_refs[0] = 1;
  item.impact = {
    measures: Array.from({ length: 17 }, () => ({ amount: 1, unit_ref: "impact-unit-a" })),
  };
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B10", /catalogs\[0\]\.items\[0\]\.impact\.measures/);
});

test("owner v72 impact extra key and measures 17", () => {
  const input = loadFixture();
  const pulse = input.catalogs[1].items.find((x) => x.item_id === "type-pulse");
  pulse.impact.extra = 1;
  pulse.impact.measures = Array.from({ length: 17 }, () => ({
    amount: 1,
    unit_ref: "impact-unit-a",
  }));
  assertFamilyOwner(() => decodeBrowserCatalog(input), "B10", /impact\.measures/);
});

test("B7 scopes table entry non-null scope_ref", () => {
  const input = loadFixture();
  findVocabEntry(input, "scopes", "catalog-scope-1").scope_ref = "catalog-scope-2";
  assertFamily(() => decodeBrowserCatalog(input), "B7");
});

test("B9 non-finite amount", () => {
  for (const value of [Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY]) {
    const input = loadFixture();
    input.catalogs[1].items.find((x) => x.item_id === "type-pulse").impact.measures[0].amount = value;
    assertFamily(() => decodeBrowserCatalog(input), "B9");
  }
});

test("B10 limits and accepting boundaries", () => {
  const tax3 = loadFixture();
  tax3.catalogs[0].items[0].taxonomy_refs = ["taxonomy-a", "taxonomy-b", "taxonomy-c"];
  assertFamily(() => decodeBrowserCatalog(tax3), "B10");
  const tags65 = loadFixture();
  tags65.catalogs[0].items[0].tag_refs = Array.from({ length: 65 }, (_, i) => `tag-${i}`);
  for (let i = 0; i < 65; i += 1) {
    tags65.vocabularies.tags.push({ id: `tag-${i}`, label: `tag-${i}`, scope_ref: null });
  }
  assertFamily(() => decodeBrowserCatalog(tags65), "B10");
  const measures17 = loadFixture();
  const pulse = measures17.catalogs[1].items.find((x) => x.item_id === "type-pulse");
  pulse.impact.measures = Array.from({ length: 17 }, () => ({ amount: 1, unit_ref: "impact-unit-a" }));
  assertFamily(() => decodeBrowserCatalog(measures17), "B10");
  const itemId128 = loadFixture();
  itemId128.catalogs[0].items[0].item_id = "a".repeat(128);
  decodeBrowserCatalog(itemId128);
  const itemId129 = loadFixture();
  itemId129.catalogs[0].items[0].item_id = "a".repeat(129);
  assertFamily(() => decodeBrowserCatalog(itemId129), "B10");
  const itemId129Mb = loadFixture();
  itemId129Mb.catalogs[0].items[0].item_id = "€".repeat(43);
  assertFamily(() => decodeBrowserCatalog(itemId129Mb), "B10");
  const label1024 = loadFixture();
  label1024.vocabularies.scopes[0].label = "x".repeat(1024);
  decodeBrowserCatalog(label1024);
  const label1025 = loadFixture();
  label1025.vocabularies.scopes[0].label = "x".repeat(1025);
  assertFamily(() => decodeBrowserCatalog(label1025), "B10");
});

test("B11 missing required keys", () => {
  const cases = [
    (i) => { delete i.catalogs[0].items[0].provider_ref; },
    (i) => { delete i.vocabularies.scopes[0].scope_ref; },
    (i) => { delete i.catalogs[0].items; },
    (i) => { delete i.vocabularies.tags; },
    (i) => {
      const pulse = i.catalogs[1].items.find((x) => x.item_id === "type-pulse");
      delete pulse.impact.measures[0].unit_ref;
    },
  ];
  for (const mutate of cases) {
    const input = loadFixture();
    mutate(input);
    assertFamily(() => decodeBrowserCatalog(input), "B11");
  }
});

test("B13 container type mismatches", () => {
  const cases = [
    (i) => { i.catalogs = {}; },
    (i) => { i.catalogs[0].items = "x"; },
    (i) => {
      const pulse = i.catalogs[1].items.find((x) => x.item_id === "type-pulse");
      pulse.impact.measures = {};
    },
    (i) => { i.catalogs[0].items[0].item_id = {}; },
    (i) => { i.vocabularies = []; },
  ];
  for (const mutate of cases) {
    const input = loadFixture();
    mutate(input);
    assertFamily(() => decodeBrowserCatalog(input), "B13");
  }
  assertFamily(() => decodeBrowserCatalog("root"), "B13");
  assertFamily(() => decodeBrowserCatalog(null), "B13");
  assertFamily(() => decodeBrowserCatalog([]), "B13");
});

test("B14 scalar type mismatches", () => {
  const cases = [
    (i) => { i.catalogs[0].items[0].item_id = 1; },
    (i) => { i.catalogs[0].items[0].display_name = 1; },
    (i) => { i.vocabularies.scopes[0].label = true; },
    (i) => { i.vocabularies.taxonomies[0].scope_ref = 1; },
    (i) => { i.catalogs[1].items[0].provider_ref = 1; },
    (i) => { i.catalogs[0].items[0].tag_refs[0] = 1; },
    (i) => {
      const pulse = i.catalogs[1].items.find((x) => x.item_id === "type-pulse");
      pulse.impact.measures[0].amount = "12";
    },
    (i) => { i.catalogs[0].items[0].item_id = null; },
  ];
  for (const mutate of cases) {
    const input = loadFixture();
    mutate(input);
    assertFamily(() => decodeBrowserCatalog(input), "B14");
  }
});

test("B15 empty id and ref strings", () => {
  const cases = [
    (i) => { i.catalogs[0].items[0].item_id = ""; },
    (i) => { i.catalogs[0].scope_ref = ""; },
    (i) => { i.vocabularies.scopes[0].id = ""; },
    (i) => { i.catalogs[1].items[0].provider_ref = ""; },
    (i) => { i.catalogs[0].items[0].pack_ref = ""; },
    (i) => { i.catalogs[0].items[2].install_state_ref = ""; },
    (i) => { i.catalogs[0].items[0].taxonomy_refs[0] = ""; },
    (i) => { i.catalogs[0].items[0].tag_refs[0] = ""; },
    (i) => {
      const pulse = i.catalogs[1].items.find((x) => x.item_id === "type-pulse");
      pulse.impact.measures[0].unit_ref = "";
    },
    (i) => { i.vocabularies.taxonomies[0].scope_ref = ""; },
  ];
  for (const mutate of cases) {
    const input = loadFixture();
    mutate(input);
    assertFamily(() => decodeBrowserCatalog(input), "B15");
  }
});

test("overlap B2 beats B1", () => {
  const input = loadFixture();
  input.extra = 1;
  input.catalog_revision = 2;
  assertFamily(() => decodeBrowserCatalog(input), "B2");
});

test("overlap B1 beats B3 on revision string", () => {
  const input = loadFixture();
  input.catalog_revision = "1";
  input.catalogs[0].items[0].availability = "x";
  assertFamily(() => decodeBrowserCatalog(input), "B1");
});

test("overlap B3 beats B14", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].availability = "x";
  input.catalogs[0].items[0].item_id = 1;
  assertFamily(() => decodeBrowserCatalog(input), "B3");
});

test("overlap B11 beats B3 on missing and extra", () => {
  const input = loadFixture();
  delete input.catalogs[0].items[0].provider_ref;
  input.catalogs[0].items[0].extra = 1;
  assertFamily(() => decodeBrowserCatalog(input), "B11");
});

test("overlap B10 beats B14 on id number and long label", () => {
  const input = loadFixture();
  input.vocabularies.scopes[0].id = 1;
  input.vocabularies.scopes[0].label = "x".repeat(1025);
  assertFamily(() => decodeBrowserCatalog(input), "B10");
});

test("overlap B10 beats B14 on taxonomy length and number element", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].taxonomy_refs = ["taxonomy-a", "taxonomy-b", 1];
  assertFamily(() => decodeBrowserCatalog(input), "B10");
});

test("overlap B4 beats B15", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].preview_kind = "static";
  input.catalogs[0].items[0].item_id = "";
  assertFamily(() => decodeBrowserCatalog(input), "B4");
});

test("overlap B3 beats B4 on extra key", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].preview_kind = "static";
  input.catalogs[0].items[0].extra = 1;
  assertFamily(() => decodeBrowserCatalog(input), "B3");
});

test("overlap B15 beats B5 on item duplicate", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].item_id = "";
  input.catalogs[0].items.push(structuredClone(input.catalogs[0].items[1]));
  input.catalogs[0].items[2].item_id = "";
  assertFamily(() => decodeBrowserCatalog(input), "B15");
});

test("overlap B5 beats B6 on dangling and duplicate", () => {
  const input = loadFixture();
  input.catalogs[0].items.push(structuredClone(input.catalogs[0].items[0]));
  input.catalogs[0].items[0].provider_ref = "missing";
  assertFamily(() => decodeBrowserCatalog(input), "B5");
});

test("overlap B6 beats B7 on dangling vs cross-scope", () => {
  const input = loadFixture();
  findVocabEntry(input, "providers", "provider-a").scope_ref = "catalog-scope-1";
  input.catalogs[1].items[0].provider_ref = "missing-provider";
  assertFamily(() => decodeBrowserCatalog(input), "B6");
});

test("overlap B6 beats B7 on cross-scope provider and dangling tag", () => {
  const input = loadFixture();
  findVocabEntry(input, "providers", "provider-b").scope_ref = "catalog-scope-1";
  input.catalogs[1].items[5].tag_refs[0] = "missing-tag";
  assertFamily(() => decodeBrowserCatalog(input), "B6");
});

test("overlap B14 beats B9 on amount string vs NaN", () => {
  const input = loadFixture();
  const pulse = input.catalogs[1].items.find((x) => x.item_id === "type-pulse");
  pulse.impact.measures.push({ amount: Number.NaN, unit_ref: "impact-unit-a" });
  pulse.impact.measures[0].amount = "12";
  assertFamily(() => decodeBrowserCatalog(input), "B14");
});

test("traversal vocab scopes before providers", () => {
  const input = loadFixture();
  input.vocabularies.scopes[0].id = "";
  input.vocabularies.providers[0].id = "";
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /^B15: /);
    assert.match(err.message, /vocabularies\.scopes/);
    return true;
  });
});

test("traversal taxonomies before tags", () => {
  const input = loadFixture();
  input.vocabularies.taxonomies[0].id = "";
  input.vocabularies.tags[0].id = "";
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /vocabularies\.taxonomies/);
    return true;
  });
});

test("traversal vocabularies before catalogs", () => {
  const input = loadFixture();
  input.vocabularies.scopes[0].id = "";
  input.catalogs[0].items[0].item_id = "";
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /vocabularies\.scopes/);
    return true;
  });
});

test("traversal catalogs0 before catalogs1", () => {
  const input = loadFixture();
  input.catalogs[0].items[2].item_id = "";
  input.catalogs[1].items[0].item_id = "";
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /catalogs\[0\]/);
    return true;
  });
});

test("traversal items0 before items1 in catalog", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].item_id = "";
  input.catalogs[0].items[1].item_id = "";
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /catalogs\[0\]\.items\[0\]/);
    return true;
  });
});

test("traversal B7 catalogs0 before catalogs1", () => {
  const input = loadFixture();
  findVocabEntry(input, "tags", "go-to").scope_ref = "catalog-scope-2";
  findVocabEntry(input, "providers", "provider-a").scope_ref = "catalog-scope-1";
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /^B7: /);
    assert.match(err.message, /catalogs\[0\]/);
    return true;
  });
});

test("remediation item_id number loses to taxonomy_refs B10", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].item_id = 1;
  input.catalogs[0].items[0].taxonomy_refs = ["taxonomy-a", "taxonomy-b", "taxonomy-c"];
  assertFamily(() => decodeBrowserCatalog(input), "B10");
});

test("remediation provider_ref number loses to tag_refs B10", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].provider_ref = 1;
  input.catalogs[0].items[0].tag_refs = Array.from({ length: 65 }, (_, i) => `extra-tag-${i}`);
  for (let i = 0; i < 65; i += 1) {
    input.vocabularies.tags.push({
      id: `extra-tag-${i}`,
      label: `extra-tag-${i}`,
      scope_ref: null,
    });
  }
  assertFamily(() => decodeBrowserCatalog(input), "B10");
});

test("remediation amount string loses to unit_ref B10", () => {
  const input = loadFixture();
  const pulse = input.catalogs[1].items.find((x) => x.item_id === "type-pulse");
  pulse.impact.measures[0].amount = "12";
  pulse.impact.measures[0].unit_ref = "€".repeat(43);
  assertFamily(() => decodeBrowserCatalog(input), "B10");
});

test("remediation scopes 17 beats catalogs B14", () => {
  const input = loadFixture();
  while (input.vocabularies.scopes.length < 17) {
    const n = input.vocabularies.scopes.length + 1;
    input.vocabularies.scopes.push({
      id: `scope-pad-${n}`,
      label: `scope-pad-${n}`,
      scope_ref: null,
    });
  }
  input.catalogs[0].items[0].item_id = 1;
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /^B10: /);
    assert.match(err.message, /vocabularies\.scopes/);
    return true;
  });
});

test("remediation scopes 17 beats catalogs length 17", () => {
  const input = loadFixture();
  while (input.vocabularies.scopes.length < 17) {
    const n = input.vocabularies.scopes.length + 1;
    input.vocabularies.scopes.push({
      id: `scope-pad-${n}`,
      label: `scope-pad-${n}`,
      scope_ref: null,
    });
  }
  while (input.catalogs.length < 17) {
    input.catalogs.push({
      scope_ref: `scope-pad-${input.catalogs.length + 1}`,
      items: [],
    });
  }
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /^B10: /);
    assert.match(err.message, /vocabularies\.scopes/);
    return true;
  });
});

test("remediation vocab table 4097 beats earlier empty id", () => {
  const input = loadFixture();
  const table = input.vocabularies.providers;
  while (table.length < 4097) {
    const n = table.length + 1;
    table.push({ id: `provider-pad-${n}`, label: `provider-pad-${n}`, scope_ref: null });
  }
  table[0].id = "";
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /^B10: /);
    assert.match(err.message, /vocabularies\.providers/);
    return true;
  });
});

test("remediation catalog scope empty loses to items 4097 B10", () => {
  const input = loadFixture();
  input.catalogs[0].scope_ref = "";
  input.catalogs[0].items = Array.from({ length: 4097 }, (_, i) => ({
    item_id: `pad-item-${i}`,
    display_name: null,
    taxonomy_refs: null,
    provider_ref: null,
    pack_ref: null,
    install_state_ref: null,
    preview_kind: "poster",
    impact: null,
    tag_refs: null,
  }));
  assert.throws(() => decodeBrowserCatalog(input), (err) => {
    assert.match(err.message, /^B10: /);
    assert.match(err.message, /\.items/);
    return true;
  });
});

test("remediation item_id empty loses to taxonomy ref 129 bytes B10", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].item_id = "";
  input.catalogs[0].items[0].taxonomy_refs = ["€".repeat(43)];
  assertFamily(() => decodeBrowserCatalog(input), "B10");
});

test("remediation taxonomy empty loses to tag_refs non-string B14", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].taxonomy_refs[0] = "";
  input.catalogs[0].items[0].tag_refs[0] = 1;
  assertFamily(() => decodeBrowserCatalog(input), "B14");
});

test("remediation earlier B14 beats invalid preview_kind", () => {
  const input = loadFixture();
  input.catalogs[0].items[0].display_name = 1;
  input.catalogs[0].items[0].preview_kind = "static";
  assertFamily(() => decodeBrowserCatalog(input), "B14");
});

test("remediation B10 reject all vocabulary table ceilings", () => {
  const tables = [
    "taxonomies",
    "providers",
    "packs",
    "install_states",
    "impact_units",
    "tags",
  ];
  for (const table of tables) {
    const input = loadFixture();
    const entries = input.vocabularies[table];
    while (entries.length < 4097) {
      const n = entries.length + 1;
      entries.push({ id: `${table}-pad-${n}`, label: `${table}-pad-${n}`, scope_ref: null });
    }
    assert.throws(() => decodeBrowserCatalog(input), (err) => {
      assert.match(err.message, /^B10: /);
      assert.match(err.message, new RegExp(`vocabularies\\.${table}`));
      return true;
    });
  }
});

test("remediation B10 accept boundary counts", () => {
  const acceptScopes = loadFixture();
  while (acceptScopes.vocabularies.scopes.length < 16) {
    const n = acceptScopes.vocabularies.scopes.length + 1;
    acceptScopes.vocabularies.scopes.push({
      id: `scope-bound-${n}`,
      label: `scope-bound-${n}`,
      scope_ref: null,
    });
  }
  decodeBrowserCatalog(acceptScopes);

  const acceptCatalogs = loadFixture();
  while (acceptCatalogs.catalogs.length < 16) {
    const n = acceptCatalogs.catalogs.length + 1;
    acceptCatalogs.vocabularies.scopes.push({
      id: `scope-cat-${n}`,
      label: `scope-cat-${n}`,
      scope_ref: null,
    });
    acceptCatalogs.catalogs.push({
      scope_ref: `scope-cat-${n}`,
      items: [],
    });
  }
  decodeBrowserCatalog(acceptCatalogs);

  const acceptItems = loadFixture();
  const template = structuredClone(acceptItems.catalogs[0].items[0]);
  acceptItems.catalogs[0].items = Array.from({ length: 4096 }, (_, i) => ({
    ...structuredClone(template),
    item_id: `bulk-item-${i}`,
  }));
  decodeBrowserCatalog(acceptItems);
});

test("remediation B10 accept each non-scopes vocabulary table at 4096", () => {
  const tables = [
    "taxonomies",
    "providers",
    "packs",
    "install_states",
    "impact_units",
    "tags",
  ];
  for (const table of tables) {
    const input = loadFixture();
    const entries = input.vocabularies[table];
    while (entries.length < 4096) {
      const n = entries.length + 1;
      entries.push({
        id: `${table}-lim-${n}`,
        label: `${table}-lim-${n}`,
        scope_ref: null,
      });
    }
    assert.equal(entries.length, 4096, table);
    decodeBrowserCatalog(input);
  }
});

test("remediation B10 reject scopes catalogs items ceilings", () => {
  const scopes17 = loadFixture();
  while (scopes17.vocabularies.scopes.length < 17) {
    const n = scopes17.vocabularies.scopes.length + 1;
    scopes17.vocabularies.scopes.push({
      id: `scope-max-${n}`,
      label: `scope-max-${n}`,
      scope_ref: null,
    });
  }
  assertFamily(() => decodeBrowserCatalog(scopes17), "B10");

  const catalogs17 = loadFixture();
  while (catalogs17.catalogs.length < 17) {
    const n = catalogs17.catalogs.length + 1;
    catalogs17.vocabularies.scopes.push({
      id: `scope-extra-${n}`,
      label: `scope-extra-${n}`,
      scope_ref: null,
    });
    catalogs17.catalogs.push({ scope_ref: `scope-extra-${n}`, items: [] });
  }
  assertFamily(() => decodeBrowserCatalog(catalogs17), "B10");

  const items4097 = loadFixture();
  const baseItem = structuredClone(items4097.catalogs[0].items[0]);
  items4097.catalogs[0].items = Array.from({ length: 4097 }, (_, i) => ({
    ...structuredClone(baseItem),
    item_id: `over-item-${i}`,
  }));
  assertFamily(() => decodeBrowserCatalog(items4097), "B10");
});

test("docs stale mirror count", () => {
  assert.equal(MIRRORS.length, 6);
});

test("docs stale literal CU-0A08BP (DO) zero", () => {
  for (const rel of MIRRORS) {
    const text = readFileSync(join(repoRoot, rel), "utf8");
    assert.equal(selectStaleBrowserDecoderProseLines(text).length, 0, rel);
  }
});

test("docs stale CU-0A08BP prose synthetic negative", () => {
  const syntheticProse = "- `CU-0A08BP`（`DO`）は現在粒。\n";
  assert.equal(selectStaleBrowserDecoderProseLines(syntheticProse).length, 1);
});

test("docs stale CU-0A08BP prose synthetic positive", () => {
  const syntheticProse = "- `CU-0A08BP`は`DONE`。次粒`CU-0A08BS`は`DO`。\n";
  assert.equal(selectStaleBrowserDecoderProseLines(syntheticProse).length, 0);
});

test("docs DONE line per mirror", () => {
  for (const rel of MIRRORS) {
    const lines = readFileSync(join(repoRoot, rel), "utf8").split("\n");
    const hits = lines.filter((line) => line.includes("CU-0A08BP") && DONE_TOKEN.test(line));
    assert.ok(hits.length >= 1, rel);
  }
});

test("docs parent CU-0A08BT SPLIT per mirror", () => {
  for (const rel of MIRRORS) {
    const lines = readFileSync(join(repoRoot, rel), "utf8").split("\n");
    const hits = lines.filter(
      (line) =>
        /(?:`|\|\s*)CU-0A08BT(?:`|\s*\|)/.test(line) &&
        SPLIT_TOKEN.test(line),
    );
    assert.ok(hits.length >= 1, rel);
  }
});

test("docs no stale CU-0A08BP PRODUCT-ASSET DO lane", () => {
  const ledger = readFileSync(join(repoRoot, "docs/implementation-ledger.md"), "utf8");
  assert.equal(selectStaleBrowserDecoderLaneRows(ledger).length, 0);
});

test("docs no stale CU-0A08BP PRODUCT-ASSET DO lane synthetic negative", () => {
  const syntheticLedger = `## 現在の並列レーン
| lane | 現在粒 | Phase / slice / checklist | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| PRODUCT-ASSET | CU-0A08BP | M3 / VS-1 / SPEC / synthetic stale hit row | \`DO\` | — | synthetic hit for CU-0A08BP | state must match exact \`DO\` |
`;
  assert.equal(selectStaleBrowserDecoderLaneRows(syntheticLedger).length, 1);
});

test("docs no stale CU-0A08BP PRODUCT-ASSET DO lane synthetic positive", () => {
  const syntheticLedger = `## 現在の並列レーン
| lane | 現在粒 | Phase / slice / checklist | 状態 | Issue | 依存確認 | 完了後 |
|---|---|---|---|---|---|---|
| PRODUCT-ASSET | SYN-LANE-1 | M3 / VS-1 / SPEC / synthetic unrelated lane | \`DO\` | — | unrelated synthetic id should pass | only status match is for CU-0A08BP |
| ORACLE-GUARD | CU-0A08BP | M3 prerequisite / synthetic lane mention | \`DO\` | — | mentions CU-0A08BP and DO in descriptive text | lane is the only mismatch |
| PRODUCT-ASSET | CU-0A08BQ | M3 prerequisite / synthetic id mismatch | \`DO\` | — | mentions CU-0A08BP and DO in descriptive text | ID is the only mismatch |
`;
  assert.equal(selectStaleBrowserDecoderLaneRows(syntheticLedger).length, 0);
});

test("docs M3 spec A1 unique IS BP line", () => {
  const lines = readFileSync(join(repoRoot, "docs/specs/M3-ui-integration.md"), "utf8").split("\n");
  const both = lines.filter((line) => line.includes("CU-0A08IS") && line.includes("CU-0A08BP"));
  assert.equal(both.length, 1);
  assert.match(both[0], /`CU-0A08BP`は`DONE`/);
});
