const CATALOG_REVISION = 1;

const TOP_LEVEL_KEYS = ["catalog_revision", "vocabularies", "catalogs"];

const VOCAB_TABLE_KEYS = [
  "scopes",
  "taxonomies",
  "providers",
  "packs",
  "install_states",
  "impact_units",
  "tags",
];

const VOCAB_ENTRY_KEYS = ["id", "label", "scope_ref"];

const CATALOG_KEYS = ["scope_ref", "items"];

const ITEM_FIELDS = [
  {
    key: "item_id",
    b13(value, owner) {
      b13RequiredString(value, owner);
    },
    b10(value, owner) {
      b10StringId(value, owner);
    },
    b14(value, owner) {
      b14RequiredString(value, owner);
    },
    b15(value, owner) {
      b15EmptyString(value, owner);
    },
  },
  {
    key: "display_name",
    b13(value, owner) {
      if (value !== null && isContainer(value)) {
        fail("13", owner, "expected string or null");
      }
    },
    b14(value, owner) {
      b14NullableString(value, owner);
    },
  },
  {
    key: "taxonomy_refs",
    b13(value, owner) {
      b13NullableStringArray(value, owner);
      b13ArrayElements(value, owner);
    },
    b10(value, owner) {
      b10NullableStringArray(value, LIMIT_TAXONOMY_REFS, owner);
    },
    b14(value, owner) {
      b14NullableStringArrayElements(value, owner);
    },
    b15(value, owner) {
      b15ArrayElements(value, owner);
    },
  },
  {
    key: "provider_ref",
    b13(value, owner) {
      b13NullableString(value, owner);
    },
    b10(value, owner) {
      b10NullableStringId(value, owner);
    },
    b14(value, owner) {
      b14NullableString(value, owner);
    },
    b15(value, owner) {
      if (value !== null) {
        b15EmptyString(value, owner);
      }
    },
  },
  {
    key: "pack_ref",
    b13(value, owner) {
      b13NullableString(value, owner);
    },
    b10(value, owner) {
      b10NullableStringId(value, owner);
    },
    b14(value, owner) {
      b14NullableString(value, owner);
    },
    b15(value, owner) {
      if (value !== null) {
        b15EmptyString(value, owner);
      }
    },
  },
  {
    key: "install_state_ref",
    b13(value, owner) {
      b13NullableString(value, owner);
    },
    b10(value, owner) {
      b10NullableStringId(value, owner);
    },
    b14(value, owner) {
      b14NullableString(value, owner);
    },
    b15(value, owner) {
      if (value !== null) {
        b15EmptyString(value, owner);
      }
    },
  },
  {
    key: "preview_kind",
    b14(value, owner) {
      validatePreviewKind(value, owner);
    },
  },
  {
    key: "impact",
    b13(value, owner) {
      b13ImpactField(value, owner);
    },
    b10(value, owner) {
      b10ImpactMeasuresCount(value, owner);
    },
  },
  {
    key: "tag_refs",
    b13(value, owner) {
      b13NullableStringArray(value, owner);
      b13ArrayElements(value, owner);
    },
    b10(value, owner) {
      b10NullableStringArray(value, LIMIT_TAG_REFS, owner);
    },
    b14(value, owner) {
      b14NullableStringArrayElements(value, owner);
    },
    b15(value, owner) {
      b15ArrayElements(value, owner);
    },
  },
];

const ITEM_KEYS = ITEM_FIELDS.map((field) => field.key);

const ITEM_FIELD_STAGES = ["b13", "b10", "b14", "b15"];

const IMPACT_KEYS = ["measures"];

const MEASURE_KEYS = ["amount", "unit_ref"];

const PREVIEW_KINDS = new Set(["motion", "poster"]);

const FORBIDDEN_OUTPUT_KEYS = new Set([
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
]);

const REF_SYSTEMS = [
  { field: "taxonomy_refs", table: "taxonomies", kind: "array" },
  { field: "provider_ref", table: "providers", kind: "scalar" },
  { field: "pack_ref", table: "packs", kind: "scalar" },
  { field: "install_state_ref", table: "install_states", kind: "scalar" },
  { field: "unit_ref", table: "impact_units", kind: "measure" },
  { field: "tag_refs", table: "tags", kind: "array" },
];

const LIMIT_SCOPES = 16;
const LIMIT_CATALOGS = 16;
const LIMIT_ITEMS_PER_CATALOG = 4096;
const LIMIT_VOCAB_ENTRIES = 4096;
const LIMIT_TAXONOMY_REFS = 2;
const LIMIT_TAG_REFS = 64;
const LIMIT_MEASURES = 16;
const LIMIT_ID_BYTES = 128;
const LIMIT_LABEL_BYTES = 1024;

const encoder = new TextEncoder();

function fail(family, owner, detail) {
  throw new TypeError(`B${family}: ${detail} at ${owner}`);
}

function utf8Bytes(value) {
  return encoder.encode(value).length;
}

function isContainer(value) {
  return value !== null && typeof value === "object";
}

function stageB11(obj, keys, owner) {
  for (const key of keys) {
    if (!Object.hasOwn(obj, key)) {
      fail("11", owner, `missing key ${key}`);
    }
  }
}

function stageB3(obj, keys, owner) {
  for (const key of Object.keys(obj)) {
    if (keys.includes(key)) {
      continue;
    }
    if (FORBIDDEN_OUTPUT_KEYS.has(key)) {
      fail("3", `${owner}.${key}`, `forbidden key ${key}`);
    }
    fail("3", `${owner}.${key}`, `unknown key ${key}`);
  }
}

function requireObject(value, owner) {
  if (!isContainer(value) || Array.isArray(value)) {
    fail("13", owner, "expected object");
  }
  return value;
}

function b13NullableString(value, owner) {
  if (value === null) {
    return;
  }
  if (typeof value === "string") {
    return;
  }
  if (isContainer(value)) {
    fail("13", owner, "expected string or null");
  }
}

function b13RequiredString(value, owner) {
  if (isContainer(value)) {
    fail("13", owner, "expected string");
  }
}

function b13NullableStringArray(value, owner) {
  if (value === null) {
    return;
  }
  if (!Array.isArray(value)) {
    fail("13", owner, "expected array or null");
  }
}

function b13ArrayElements(value, owner) {
  if (value === null || !Array.isArray(value)) {
    return;
  }
  for (let i = 0; i < value.length; i += 1) {
    const el = value[i];
    if (isContainer(el)) {
      fail("13", `${owner}[${i}]`, "expected string");
    }
  }
}

function b10StringId(value, owner) {
  if (typeof value === "string" && utf8Bytes(value) > LIMIT_ID_BYTES) {
    fail("10", owner, "ref exceeds byte limit");
  }
}

function b10NullableStringId(value, owner) {
  if (value !== null && typeof value === "string") {
    b10StringId(value, owner);
  }
}

function b10NullableStringArray(value, limit, ownerBase) {
  if (value === null) {
    return;
  }
  if (value.length > limit) {
    fail("10", ownerBase, "array exceeds limit");
  }
  for (let i = 0; i < value.length; i += 1) {
    const el = value[i];
    if (typeof el === "string" && utf8Bytes(el) > LIMIT_ID_BYTES) {
      fail("10", `${ownerBase}[${i}]`, "ref exceeds byte limit");
    }
  }
}

function b10ImpactMeasuresCount(impact, owner) {
  if (impact === null) {
    return;
  }
  if (!isContainer(impact) || Array.isArray(impact)) {
    return;
  }
  const measures = impact.measures;
  if (!Array.isArray(measures)) {
    return;
  }
  if (measures.length > LIMIT_MEASURES) {
    fail("10", `${owner}.measures`, "measures exceeds limit");
  }
}

function b14NullableString(value, owner) {
  if (value !== null && typeof value !== "string") {
    fail("14", owner, "expected string or null");
  }
}

function b14RequiredString(value, owner) {
  if (typeof value !== "string") {
    fail("14", owner, "expected string");
  }
}

function b14NullableStringArrayElements(value, ownerBase) {
  if (value === null) {
    return;
  }
  for (let i = 0; i < value.length; i += 1) {
    const el = value[i];
    if (typeof el !== "string") {
      fail("14", `${ownerBase}[${i}]`, "expected string");
    }
  }
}

function validatePreviewKind(value, owner) {
  if (typeof value !== "string" || !PREVIEW_KINDS.has(value)) {
    fail("4", owner, `invalid preview_kind ${value}`);
  }
}

function b15EmptyString(value, owner) {
  if (typeof value === "string" && utf8Bytes(value) === 0) {
    fail("15", owner, "empty ref");
  }
}

function b15ArrayElements(value, ownerBase) {
  if (value === null) {
    return;
  }
  for (let i = 0; i < value.length; i += 1) {
    b15EmptyString(value[i], `${ownerBase}[${i}]`);
  }
}

function b13ImpactField(impact, owner) {
  if (impact === null) {
    return;
  }
  if (!isContainer(impact) || Array.isArray(impact)) {
    fail("13", owner, "expected object or null");
  }
}

function runItemFamilyStages(obj, owner) {
  for (const stage of ITEM_FIELD_STAGES) {
    for (const field of ITEM_FIELDS) {
      const run = field[stage];
      if (run === undefined) {
        continue;
      }
      const fieldOwner = `${owner}.${field.key}`;
      run(obj[field.key], fieldOwner, obj);
    }
  }
}

function validateMeasureFamilyMajor(measure, owner) {
  const obj = requireObject(measure, owner);
  stageB11(obj, MEASURE_KEYS, owner);
  stageB3(obj, MEASURE_KEYS, owner);

  const amount = obj.amount;
  const unitRef = obj.unit_ref;

  const b13 = [
    () => {
      if (isContainer(amount)) {
        fail("13", `${owner}.amount`, "expected number");
      }
    },
    () => {
      if (isContainer(unitRef)) {
        fail("13", `${owner}.unit_ref`, "expected string");
      }
    },
  ];
  const b10 = [
    () => {
      if (typeof unitRef === "string" && utf8Bytes(unitRef) > LIMIT_ID_BYTES) {
        fail("10", `${owner}.unit_ref`, "ref exceeds byte limit");
      }
    },
  ];
  const b14 = [
    () => {
      if (typeof amount !== "number") {
        fail("14", `${owner}.amount`, "expected number");
      }
    },
    () => {
      if (typeof unitRef !== "string") {
        fail("14", `${owner}.unit_ref`, "expected string");
      }
    },
  ];
  const b15 = [
    () => {
      if (typeof unitRef === "string" && utf8Bytes(unitRef) === 0) {
        fail("15", `${owner}.unit_ref`, "empty ref");
      }
    },
  ];
  const b9 = [
    () => {
      if (typeof amount === "number" && !Number.isFinite(amount)) {
        fail("9", `${owner}.amount`, "expected finite number");
      }
    },
  ];

  for (const stages of [b13, b10, b14, b15, b9]) {
    for (const run of stages) {
      run();
    }
  }
}

function validateImpactFamilyMajor(impact, owner) {
  const obj = requireObject(impact, owner);
  stageB11(obj, IMPACT_KEYS, owner);
  stageB3(obj, IMPACT_KEYS, owner);

  const measures = obj.measures;
  if (!Array.isArray(measures)) {
    fail("13", `${owner}.measures`, "expected array");
  }

  for (let i = 0; i < measures.length; i += 1) {
    validateMeasureFamilyMajor(measures[i], `${owner}.measures[${i}]`);
  }
}

function validateItemFamilyMajor(item, owner) {
  const obj = requireObject(item, owner);
  stageB11(obj, ITEM_KEYS, owner);
  stageB3(obj, ITEM_KEYS, owner);
  runItemFamilyStages(obj, owner);
  if (obj.impact !== null) {
    validateImpactFamilyMajor(obj.impact, `${owner}.impact`);
  }
}

function validateCatalog(catalog, owner) {
  const obj = requireObject(catalog, owner);
  stageB11(obj, CATALOG_KEYS, owner);
  stageB3(obj, CATALOG_KEYS, owner);

  const scopeRef = obj.scope_ref;
  const items = obj.items;

  const stagesB13 = [
    () => {
      if (isContainer(scopeRef)) {
        fail("13", `${owner}.scope_ref`, "expected string");
      }
    },
    () => {
      if (!Array.isArray(items)) {
        fail("13", `${owner}.items`, "expected array");
      }
    },
  ];
  const stagesB10 = [
    () => {
      if (typeof scopeRef === "string" && utf8Bytes(scopeRef) > LIMIT_ID_BYTES) {
        fail("10", `${owner}.scope_ref`, "scope_ref exceeds byte limit");
      }
    },
    () => {
      if (Array.isArray(items) && items.length > LIMIT_ITEMS_PER_CATALOG) {
        fail("10", `${owner}.items`, "items exceeds limit");
      }
    },
  ];
  const stagesB14 = [
    () => {
      if (typeof scopeRef !== "string") {
        fail("14", `${owner}.scope_ref`, "expected string");
      }
    },
  ];
  const stagesB15 = [
    () => {
      if (typeof scopeRef === "string" && utf8Bytes(scopeRef) === 0) {
        fail("15", `${owner}.scope_ref`, "empty scope_ref");
      }
    },
  ];

  for (const stages of [stagesB13, stagesB10, stagesB14, stagesB15]) {
    for (const run of stages) {
      run();
    }
  }

  for (let i = 0; i < items.length; i += 1) {
    validateItemFamilyMajor(items[i], `${owner}.items[${i}]`);
  }
}

function validateVocabEntryFamilyMajor(entry, owner) {
  const obj = requireObject(entry, owner);
  stageB11(obj, VOCAB_ENTRY_KEYS, owner);
  stageB3(obj, VOCAB_ENTRY_KEYS, owner);

  const idVal = obj.id;
  const labelVal = obj.label;
  const scopeRef = obj.scope_ref;

  const b13 = [
    () => {
      if (isContainer(idVal)) {
        fail("13", `${owner}.id`, "expected string");
      }
    },
    () => {
      if (isContainer(labelVal)) {
        fail("13", `${owner}.label`, "expected string");
      }
    },
    () => {
      if (scopeRef !== null && isContainer(scopeRef)) {
        fail("13", `${owner}.scope_ref`, "expected string or null");
      }
    },
  ];
  const b10 = [
    () => {
      if (typeof idVal === "string" && utf8Bytes(idVal) > LIMIT_ID_BYTES) {
        fail("10", `${owner}.id`, "id exceeds byte limit");
      }
    },
    () => {
      if (typeof labelVal === "string" && utf8Bytes(labelVal) > LIMIT_LABEL_BYTES) {
        fail("10", `${owner}.label`, "label exceeds byte limit");
      }
    },
    () => {
      if (typeof scopeRef === "string" && utf8Bytes(scopeRef) > LIMIT_ID_BYTES) {
        fail("10", `${owner}.scope_ref`, "scope_ref exceeds byte limit");
      }
    },
  ];
  const b14 = [
    () => {
      if (typeof idVal !== "string") {
        fail("14", `${owner}.id`, "expected string");
      }
    },
    () => {
      if (typeof labelVal !== "string") {
        fail("14", `${owner}.label`, "expected string");
      }
    },
    () => {
      if (scopeRef !== null && typeof scopeRef !== "string") {
        fail("14", `${owner}.scope_ref`, "expected string or null");
      }
    },
  ];
  const b15 = [
    () => {
      if (typeof idVal === "string" && utf8Bytes(idVal) === 0) {
        fail("15", `${owner}.id`, "empty id");
      }
    },
    () => {
      if (typeof scopeRef === "string" && utf8Bytes(scopeRef) === 0) {
        fail("15", `${owner}.scope_ref`, "empty scope_ref");
      }
    },
  ];

  for (const stages of [b13, b10, b14, b15]) {
    for (const run of stages) {
      run();
    }
  }
}

function validateVocabTable(entries, owner, maxEntries) {
  if (!Array.isArray(entries)) {
    fail("13", owner, "expected array");
  }
  if (entries.length > maxEntries) {
    fail("10", owner, "vocabulary table exceeds limit");
  }
  for (let i = 0; i < entries.length; i += 1) {
    validateVocabEntryFamilyMajor(entries[i], `${owner}[${i}]`);
  }
}

function validateVocabularies(vocabularies) {
  const voc = requireObject(vocabularies, "vocabularies");
  stageB11(voc, VOCAB_TABLE_KEYS, "vocabularies");
  stageB3(voc, VOCAB_TABLE_KEYS, "vocabularies");
  for (const tableKey of VOCAB_TABLE_KEYS) {
    const limit = tableKey === "scopes" ? LIMIT_SCOPES : LIMIT_VOCAB_ENTRIES;
    validateVocabTable(voc[tableKey], `vocabularies.${tableKey}`, limit);
  }
  return voc;
}

function validateCatalogRevision(value) {
  if (typeof value !== "number" || !Number.isInteger(value) || value !== CATALOG_REVISION) {
    fail("1", "catalog_revision", "revision mismatch");
  }
}

function validateTopLevel(input) {
  if (input === null || typeof input !== "object" || Array.isArray(input)) {
    fail("13", "root", "expected object");
  }
  const root = input;
  const keys = Object.keys(root);
  const expected = new Set(TOP_LEVEL_KEYS);
  if (keys.length !== TOP_LEVEL_KEYS.length) {
    fail("2", "root", "top-level key count mismatch");
  }
  for (const key of keys) {
    if (!expected.has(key)) {
      fail("2", `root.${key}`, `unexpected top-level key ${key}`);
    }
  }
  for (const key of TOP_LEVEL_KEYS) {
    if (!Object.hasOwn(root, key)) {
      fail("2", "root", `missing top-level key ${key}`);
    }
  }
  validateCatalogRevision(root.catalog_revision);
  const voc = validateVocabularies(root.vocabularies);
  const catalogs = root.catalogs;
  if (!Array.isArray(catalogs)) {
    fail("13", "catalogs", "expected array");
  }
  if (catalogs.length > LIMIT_CATALOGS) {
    fail("10", "catalogs", "catalogs exceeds limit");
  }
  for (let c = 0; c < catalogs.length; c += 1) {
    validateCatalog(catalogs[c], `catalogs[${c}]`);
  }
  return { root, voc, catalogs };
}

function buildVocabMaps(voc) {
  const maps = {};
  for (const tableKey of VOCAB_TABLE_KEYS) {
    const map = new Map();
    maps[tableKey] = map;
    const entries = voc[tableKey];
    for (let i = 0; i < entries.length; i += 1) {
      const entry = entries[i];
      map.set(entry.id, entry);
    }
  }
  return maps;
}

function checkDuplicateIds(entries, owner) {
  const seen = new Set();
  for (let i = 0; i < entries.length; i += 1) {
    const id = entries[i].id;
    if (seen.has(id)) {
      fail("5", `${owner}[${i}].id`, `duplicate id ${id}`);
    }
    seen.add(id);
  }
}

function resolveEntryScope(maps, table, ref) {
  return maps[table].get(ref).scope_ref;
}

function checkB6ForItemRefs(item, itemOwner, maps) {
  for (const sys of REF_SYSTEMS) {
    if (sys.kind === "array") {
      const refs = item[sys.field];
      if (refs !== null) {
        for (let t = 0; t < refs.length; t += 1) {
          const ref = refs[t];
          if (!maps[sys.table].has(ref)) {
            fail("6", `${itemOwner}.${sys.field}[${t}]`, `dangling ref ${ref}`);
          }
        }
      }
    } else if (sys.kind === "scalar") {
      const ref = item[sys.field];
      if (ref !== null && !maps[sys.table].has(ref)) {
        fail("6", `${itemOwner}.${sys.field}`, `dangling ref ${ref}`);
      }
    } else if (sys.kind === "measure") {
      const impact = item.impact;
      if (impact !== null) {
        const measures = impact.measures;
        for (let m = 0; m < measures.length; m += 1) {
          const unitRef = measures[m].unit_ref;
          if (!maps[sys.table].has(unitRef)) {
            fail("6", `${itemOwner}.impact.measures[${m}].unit_ref`, `dangling ref ${unitRef}`);
          }
        }
      }
    }
  }
}

function checkB7ForItemRefs(item, itemOwner, catalogScope, maps) {
  for (const sys of REF_SYSTEMS) {
    if (sys.kind === "array") {
      const refs = item[sys.field];
      if (refs !== null) {
        for (let t = 0; t < refs.length; t += 1) {
          const ref = refs[t];
          const entryScope = resolveEntryScope(maps, sys.table, ref);
          if (entryScope !== null && entryScope !== catalogScope) {
            fail("7", `${itemOwner}.${sys.field}[${t}]`, "cross-scope ref");
          }
        }
      }
    } else if (sys.kind === "scalar") {
      const ref = item[sys.field];
      if (ref !== null) {
        const entryScope = resolveEntryScope(maps, sys.table, ref);
        if (entryScope !== null && entryScope !== catalogScope) {
          fail("7", `${itemOwner}.${sys.field}`, "cross-scope ref");
        }
      }
    } else if (sys.kind === "measure") {
      const impact = item.impact;
      if (impact !== null) {
        const measures = impact.measures;
        for (let m = 0; m < measures.length; m += 1) {
          const unitRef = measures[m].unit_ref;
          const entryScope = resolveEntryScope(maps, sys.table, unitRef);
          if (entryScope !== null && entryScope !== catalogScope) {
            fail("7", `${itemOwner}.impact.measures[${m}].unit_ref`, "cross-scope ref");
          }
        }
      }
    }
  }
}

function checkB6Dangling(voc, catalogs, maps) {
  const scopesMap = maps.scopes;
  for (let c = 0; c < catalogs.length; c += 1) {
    const catalogScope = catalogs[c].scope_ref;
    if (!scopesMap.has(catalogScope)) {
      fail("6", `catalogs[${c}].scope_ref`, `dangling scope_ref ${catalogScope}`);
    }
    const items = catalogs[c].items;
    for (let i = 0; i < items.length; i += 1) {
      checkB6ForItemRefs(items[i], `catalogs[${c}].items[${i}]`, maps);
    }
  }
}

function checkB7CrossScope(voc, catalogs, maps) {
  for (const entry of voc.scopes) {
    if (entry.scope_ref !== null) {
      fail("7", `vocabularies.scopes entry ${entry.id}`, "scopes entry scope_ref must be null");
    }
  }

  for (let c = 0; c < catalogs.length; c += 1) {
    const catalogScope = catalogs[c].scope_ref;
    const items = catalogs[c].items;
    for (let i = 0; i < items.length; i += 1) {
      checkB7ForItemRefs(items[i], `catalogs[${c}].items[${i}]`, catalogScope, maps);
    }
  }
}

function validateSnapshotRelations(voc, catalogs, maps) {
  for (const tableKey of VOCAB_TABLE_KEYS) {
    checkDuplicateIds(voc[tableKey], `vocabularies.${tableKey}`);
  }

  const scopeSeen = new Set();
  for (let c = 0; c < catalogs.length; c += 1) {
    const scopeRef = catalogs[c].scope_ref;
    if (scopeSeen.has(scopeRef)) {
      fail("5", `catalogs[${c}].scope_ref`, `duplicate scope_ref ${scopeRef}`);
    }
    scopeSeen.add(scopeRef);
  }

  for (let c = 0; c < catalogs.length; c += 1) {
    const catalog = catalogs[c];
    const itemSeen = new Set();
    const items = catalog.items;
    for (let i = 0; i < items.length; i += 1) {
      const itemId = items[i].item_id;
      if (itemSeen.has(itemId)) {
        fail("5", `catalogs[${c}].items[${i}].item_id`, `duplicate item_id ${itemId}`);
      }
      itemSeen.add(itemId);
    }
  }

  checkB6Dangling(voc, catalogs, maps);
  checkB7CrossScope(voc, catalogs, maps);
}

function assertNoForbiddenOutputKeys(value, owner) {
  if (value === null || typeof value !== "object") {
    return;
  }
  if (Array.isArray(value)) {
    for (let i = 0; i < value.length; i += 1) {
      assertNoForbiddenOutputKeys(value[i], `${owner}[${i}]`);
    }
    return;
  }
  for (const key of Object.keys(value)) {
    if (FORBIDDEN_OUTPUT_KEYS.has(key)) {
      fail("3", `${owner}.${key}`, `forbidden output key ${key}`);
    }
    assertNoForbiddenOutputKeys(value[key], `${owner}.${key}`);
  }
}

function decodeBrowserCatalog(input) {
  const { root, voc, catalogs } = validateTopLevel(input);
  const maps = buildVocabMaps(voc);
  validateSnapshotRelations(voc, catalogs, maps);
  const output = structuredClone(root);
  assertNoForbiddenOutputKeys(output, "output");
  return output;
}

export { decodeBrowserCatalog };
