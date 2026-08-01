import { decodeBrowserCatalog } from "../read-model/browserCatalogDecoder.js";

const VERSION = 1;
const ROLE = "browser";
const HOST_DIRECTION = "host-to-web";
const WEB_DIRECTION = "web-to-host";
const MAX_ID_BYTES = 128;
const MAX_MESSAGE_BYTES = 1024;
const CATALOG_REVISION = 1;
const CATALOG_SCOPE_REF = "first-party-effects";
const CATALOG_SCOPE_LABEL = "First-party effects";
const CATALOG_ITEM_ID = "core.filter.opacity";
const CATALOG_ITEM_NAME = "Opacity";
const CATALOG_CATEGORY_ID = "effect";
const CATALOG_CATEGORY_LABEL = "Effect";
const CATALOG_SUBTYPE_ID = "color";
const CATALOG_SUBTYPE_LABEL = "Color";
const CATALOG_PROVIDER_ID = "built-in";
const CATALOG_PROVIDER_LABEL = "Built-in";
const CATALOG_INSTALL_STATE_ID = "installed";
const CATALOG_INSTALL_STATE_LABEL = "Installed";
const CATALOG_PREVIEW_KIND = "poster";
const DECIMAL_U64 = /^(0|[1-9][0-9]*)$/;
const U64_MAX = 18_446_744_073_709_551_615n;
const encoder = new TextEncoder();

function exactKeys(value, keys, owner) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${owner} must be an object`);
  }
  const actual = Object.keys(value);
  if (
    actual.length !== keys.length
    || keys.some((key) => !Object.hasOwn(value, key))
    || actual.some((key) => !keys.includes(key))
  ) {
    throw new TypeError(`${owner} has unknown or missing fields`);
  }
  return value;
}

function boundedId(value, owner) {
  if (
    typeof value !== "string"
    || value.length === 0
    || encoder.encode(value).byteLength > MAX_ID_BYTES
  ) {
    throw new TypeError(`${owner} must be a non-empty bounded string`);
  }
  return value;
}

function exactLength(value, length, owner) {
  if (!Array.isArray(value) || value.length !== length) {
    throw new TypeError(`${owner} must contain exactly ${length} entries`);
  }
  return value;
}

function exactVocabularyEntry(entry, expected, owner) {
  if (
    entry.id !== expected.id
    || entry.label !== expected.label
    || entry.scope_ref !== expected.scope_ref
  ) {
    throw new TypeError(`${owner} mismatch`);
  }
  return entry;
}

function decodeOpacityCatalogProjection(input) {
  const decoded = decodeBrowserCatalog(input);
  if (decoded.catalog_revision !== CATALOG_REVISION) {
    throw new TypeError("snapshot.browser.catalog catalog_revision mismatch");
  }

  const [scope] = exactLength(
    decoded.vocabularies.scopes,
    1,
    "snapshot.browser.catalog.vocabularies.scopes",
  );
  exactVocabularyEntry(scope, {
    id: CATALOG_SCOPE_REF,
    label: CATALOG_SCOPE_LABEL,
    scope_ref: null,
  }, "snapshot.browser.catalog.vocabularies.scopes entry");

  const [category, subtype] = exactLength(
    decoded.vocabularies.taxonomies,
    2,
    "snapshot.browser.catalog.vocabularies.taxonomies",
  );
  exactVocabularyEntry(category, {
    id: CATALOG_CATEGORY_ID,
    label: CATALOG_CATEGORY_LABEL,
    scope_ref: CATALOG_SCOPE_REF,
  }, "snapshot.browser.catalog.vocabularies.taxonomies category");
  exactVocabularyEntry(subtype, {
    id: CATALOG_SUBTYPE_ID,
    label: CATALOG_SUBTYPE_LABEL,
    scope_ref: CATALOG_SCOPE_REF,
  }, "snapshot.browser.catalog.vocabularies.taxonomies subtype");

  const [provider] = exactLength(
    decoded.vocabularies.providers,
    1,
    "snapshot.browser.catalog.vocabularies.providers",
  );
  exactVocabularyEntry(provider, {
    id: CATALOG_PROVIDER_ID,
    label: CATALOG_PROVIDER_LABEL,
    scope_ref: CATALOG_SCOPE_REF,
  }, "snapshot.browser.catalog.vocabularies.providers entry");

  const [installState] = exactLength(
    decoded.vocabularies.install_states,
    1,
    "snapshot.browser.catalog.vocabularies.install_states",
  );
  exactVocabularyEntry(installState, {
    id: CATALOG_INSTALL_STATE_ID,
    label: CATALOG_INSTALL_STATE_LABEL,
    scope_ref: CATALOG_SCOPE_REF,
  }, "snapshot.browser.catalog.vocabularies.install_states entry");

  for (const table of ["packs", "impact_units", "tags"]) {
    exactLength(
      decoded.vocabularies[table],
      0,
      `snapshot.browser.catalog.vocabularies.${table}`,
    );
  }

  const [catalog] = exactLength(
    decoded.catalogs,
    1,
    "snapshot.browser.catalog.catalogs",
  );
  if (catalog.scope_ref !== CATALOG_SCOPE_REF) {
    throw new TypeError("snapshot.browser.catalog scope mismatch");
  }
  const [item] = exactLength(
    catalog.items,
    1,
    "snapshot.browser.catalog first-party items",
  );
  if (
    item.item_id !== CATALOG_ITEM_ID
    || item.display_name !== CATALOG_ITEM_NAME
    || !Array.isArray(item.taxonomy_refs)
    || item.taxonomy_refs.length !== 2
    || item.taxonomy_refs[0] !== category.id
    || item.taxonomy_refs[1] !== subtype.id
    || item.provider_ref !== provider.id
    || item.pack_ref !== null
    || item.install_state_ref !== installState.id
    || item.preview_kind !== CATALOG_PREVIEW_KIND
    || item.impact !== null
    || !Array.isArray(item.tag_refs)
    || item.tag_refs.length !== 0
  ) {
    throw new TypeError("snapshot.browser.catalog Opacity item mismatch");
  }

  const taxonomyLabels = `${category.label} ${subtype.label}`;
  return Object.freeze({
    scopeRef: catalog.scope_ref,
    itemId: item.item_id,
    name: item.display_name,
    category: Object.freeze({ value: category.id, label: category.label }),
    subtype: Object.freeze({ value: subtype.id, label: subtype.label }),
    mode: installState.id,
    folder: taxonomyLabels,
    labels: taxonomyLabels,
    search: [
      item.display_name,
      item.item_id,
      category.label,
      subtype.label,
      provider.label,
      installState.label,
    ].join(" "),
    thumbnail: item.preview_kind,
    kind: "FX",
    state: undefined,
    identity: undefined,
    impact: undefined,
    pack: item.pack_ref,
    motion: false,
    tags: Object.freeze([]),
    tagVisible: true,
  });
}

function u64(value, owner) {
  if (typeof value !== "string" || !DECIMAL_U64.test(value)) {
    throw new TypeError(`${owner} must be a canonical decimal u64`);
  }
  const parsed = BigInt(value);
  if (parsed > U64_MAX) {
    throw new TypeError(`${owner} exceeds u64`);
  }
  return parsed;
}

export function decodeBrowserHostSnapshot(value) {
  const root = exactKeys(
    value,
    ["version", "direction", "role", "instance_epoch", "sequence", "browser"],
    "snapshot",
  );
  if (
    root.version !== VERSION
    || root.direction !== HOST_DIRECTION
    || root.role !== ROLE
  ) {
    throw new TypeError("snapshot envelope mismatch");
  }
  const hasCatalog = Object.hasOwn(root.browser, "catalog");
  const browser = exactKeys(
    root.browser,
    hasCatalog ? ["rectangle_source", "catalog"] : ["rectangle_source"],
    "snapshot.browser",
  );
  const source = exactKeys(
    browser.rectangle_source,
    ["scope_ref", "item_id"],
    "snapshot.browser.rectangle_source",
  );
  const catalogProjection = hasCatalog
    ? decodeOpacityCatalogProjection(browser.catalog)
    : undefined;
  return Object.freeze({
    instanceEpoch: u64(root.instance_epoch, "snapshot.instance_epoch"),
    sequence: u64(root.sequence, "snapshot.sequence"),
    rectangleIdentity: Object.freeze({
      scope_ref: boundedId(source.scope_ref, "snapshot.scope_ref"),
      item_id: boundedId(source.item_id, "snapshot.item_id"),
    }),
    catalogProjection,
  });
}

export function createBrowserHostSender(snapshot, postMessage) {
  if (typeof postMessage !== "function") {
    throw new TypeError("Host postMessage must be a function");
  }
  let sequence = snapshot.sequence;
  return (intent) => {
    exactKeys(intent, ["kind", "source"], "intent");
    if (
      intent.kind !== "browser.place"
      && intent.kind !== "browser.attach-effect"
    ) {
      throw new TypeError("unknown Browser intent");
    }
    const source = exactKeys(intent.source, ["scope_ref", "item_id"], "intent.source");
    const scopeRef = boundedId(source.scope_ref, "intent.scope_ref");
    const itemId = boundedId(source.item_id, "intent.item_id");
    if (
      intent.kind === "browser.attach-effect"
      && (
        snapshot.catalogProjection === undefined
        || scopeRef !== snapshot.catalogProjection.scopeRef
        || itemId !== snapshot.catalogProjection.itemId
      )
    ) {
      throw new TypeError("Browser attach source does not match the current projection");
    }
    if (sequence === U64_MAX) {
      throw new RangeError("Browser Host sequence exhausted");
    }
    sequence += 1n;
    const message = {
      version: VERSION,
      direction: WEB_DIRECTION,
      role: ROLE,
      instance_epoch: snapshot.instanceEpoch.toString(),
      sequence: sequence.toString(),
      kind: intent.kind,
      source: {
        scope_ref: scopeRef,
        item_id: itemId,
      },
    };
    const encoded = JSON.stringify(message);
    if (encoder.encode(encoded).byteLength > MAX_MESSAGE_BYTES) {
      throw new RangeError("Browser Host message exceeds byte limit");
    }
    postMessage(encoded);
  };
}
