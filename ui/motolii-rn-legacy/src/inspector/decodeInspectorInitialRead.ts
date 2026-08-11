/**
 * Host→RN Inspector initial-read wire decoder（純関数 read model）。
 * host_handle / diagnostics は検証のみで Inspector 意味には露出しない。
 */

export type InspectorInitialRead =
  | {
      tag: "no-selection";
      revision: string;
      projectionGeneration: string;
    }
  | {
      tag: "selected";
      layerId: string;
      displayName: string;
      revision: string;
      projectionGeneration: string;
    };

export type DecodeInspectorInitialReadError =
  | { tag: "invalid-json" }
  | { tag: "non-object" }
  | { tag: "unknown-field"; path: string; field: string }
  | { tag: "missing-field"; path: string; field: string }
  | { tag: "invalid-constant"; path: string; field: string }
  | { tag: "invalid-decimal"; path: string; field: string }
  | { tag: "invalid-type"; path: string; field: string }
  | { tag: "duplicate-selection-id" }
  | { tag: "duplicate-bounds-id" }
  | { tag: "dangling-primary" }
  | { tag: "invalid-selection-count" }
  | { tag: "mismatch" };

export type DecodeInspectorInitialReadResult =
  | { ok: true; value: InspectorInitialRead }
  | { ok: false; error: DecodeInspectorInitialReadError };

const ROOT_KEYS = new Set([
  "version",
  "direction",
  "role",
  "host_handle",
  "revision",
  "projection_generation",
  "primary_layer_id",
  "stage",
  "diagnostics",
]);

const REQUIRED_ROOT_KEYS = [
  "version",
  "direction",
  "role",
  "host_handle",
  "revision",
  "projection_generation",
  "stage",
  "diagnostics",
] as const;

const STAGE_KEYS = new Set(["selection", "bounds"]);
const SELECTION_ENTRY_KEYS = new Set(["layer_id"]);
const BOUNDS_ENTRY_KEYS = new Set(["layer_id", "display_name"]);

const UNSIGNED_DECIMAL = /^(0|[1-9][0-9]*)$/;
const POSITIVE_DECIMAL = /^[1-9][0-9]*$/;

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function rejectUnknownFields(
  obj: Record<string, unknown>,
  allowed: Set<string>,
  path: string,
): DecodeInspectorInitialReadError | null {
  for (const key of Object.keys(obj)) {
    if (!allowed.has(key)) {
      return { tag: "unknown-field", path, field: key };
    }
  }
  return null;
}

function requireStringField(
  obj: Record<string, unknown>,
  field: string,
  path: string,
): string | DecodeInspectorInitialReadError {
  if (!Object.prototype.hasOwnProperty.call(obj, field)) {
    return { tag: "missing-field", path, field };
  }
  const value = obj[field];
  if (typeof value !== "string") {
    return { tag: "invalid-type", path, field };
  }
  return value;
}

function parseUnsignedDecimal(
  value: string,
  path: string,
  field: string,
): string | DecodeInspectorInitialReadError {
  if (!UNSIGNED_DECIMAL.test(value)) {
    return { tag: "invalid-decimal", path, field };
  }
  return value;
}

function parsePositiveDecimal(
  value: string,
  path: string,
  field: string,
): string | DecodeInspectorInitialReadError {
  if (!POSITIVE_DECIMAL.test(value)) {
    return { tag: "invalid-decimal", path, field };
  }
  return value;
}

function decodeSelectionEntry(
  value: unknown,
  path: string,
): { layerId: string } | DecodeInspectorInitialReadError {
  if (!isPlainObject(value)) {
    return { tag: "invalid-type", path, field: "entry" };
  }
  const unknown = rejectUnknownFields(value, SELECTION_ENTRY_KEYS, path);
  if (unknown) return unknown;
  const layerIdRaw = requireStringField(value, "layer_id", path);
  if (typeof layerIdRaw !== "string") return layerIdRaw;
  const layerId = parsePositiveDecimal(layerIdRaw, path, "layer_id");
  if (typeof layerId !== "string") return layerId;
  return { layerId };
}

function decodeBoundsEntry(
  value: unknown,
  path: string,
):
  | { layerId: string; displayName: string }
  | DecodeInspectorInitialReadError {
  if (!isPlainObject(value)) {
    return { tag: "invalid-type", path, field: "entry" };
  }
  const unknown = rejectUnknownFields(value, BOUNDS_ENTRY_KEYS, path);
  if (unknown) return unknown;
  const layerIdRaw = requireStringField(value, "layer_id", path);
  if (typeof layerIdRaw !== "string") return layerIdRaw;
  const layerId = parsePositiveDecimal(layerIdRaw, path, "layer_id");
  if (typeof layerId !== "string") return layerId;
  const displayName = requireStringField(value, "display_name", path);
  if (typeof displayName !== "string") return displayName;
  return { layerId, displayName };
}

function decodeFromValue(value: unknown): DecodeInspectorInitialReadResult {
  if (Array.isArray(value)) {
    return { ok: false, error: { tag: "non-object" } };
  }
  if (!isPlainObject(value)) {
    return { ok: false, error: { tag: "non-object" } };
  }

  const unknownRoot = rejectUnknownFields(value, ROOT_KEYS, "");
  if (unknownRoot) return { ok: false, error: unknownRoot };

  for (const field of REQUIRED_ROOT_KEYS) {
    if (!Object.prototype.hasOwnProperty.call(value, field)) {
      return {
        ok: false,
        error: { tag: "missing-field", path: "", field },
      };
    }
  }

  if (value.version !== 1) {
    return {
      ok: false,
      error: { tag: "invalid-constant", path: "", field: "version" },
    };
  }
  if (value.direction !== "host-to-rn") {
    return {
      ok: false,
      error: { tag: "invalid-constant", path: "", field: "direction" },
    };
  }
  if (value.role !== "product-runtime-seat") {
    return {
      ok: false,
      error: { tag: "invalid-constant", path: "", field: "role" },
    };
  }

  if (typeof value.host_handle !== "string") {
    return {
      ok: false,
      error: { tag: "invalid-type", path: "", field: "host_handle" },
    };
  }
  const hostHandle = parsePositiveDecimal(
    value.host_handle,
    "",
    "host_handle",
  );
  if (typeof hostHandle !== "string") {
    return { ok: false, error: hostHandle };
  }

  if (typeof value.revision !== "string") {
    return {
      ok: false,
      error: { tag: "invalid-type", path: "", field: "revision" },
    };
  }
  const revision = parseUnsignedDecimal(value.revision, "", "revision");
  if (typeof revision !== "string") {
    return { ok: false, error: revision };
  }

  if (typeof value.projection_generation !== "string") {
    return {
      ok: false,
      error: {
        tag: "invalid-type",
        path: "",
        field: "projection_generation",
      },
    };
  }
  const projectionGeneration = parseUnsignedDecimal(
    value.projection_generation,
    "",
    "projection_generation",
  );
  if (typeof projectionGeneration !== "string") {
    return { ok: false, error: projectionGeneration };
  }

  let primaryLayerId: string | undefined;
  if (Object.prototype.hasOwnProperty.call(value, "primary_layer_id")) {
    if (typeof value.primary_layer_id !== "string") {
      return {
        ok: false,
        error: { tag: "invalid-type", path: "", field: "primary_layer_id" },
      };
    }
    const parsedPrimary = parsePositiveDecimal(
      value.primary_layer_id,
      "",
      "primary_layer_id",
    );
    if (typeof parsedPrimary !== "string") {
      return { ok: false, error: parsedPrimary };
    }
    primaryLayerId = parsedPrimary;
  }

  if (!isPlainObject(value.stage)) {
    return {
      ok: false,
      error: { tag: "invalid-type", path: "", field: "stage" },
    };
  }
  const stageUnknown = rejectUnknownFields(value.stage, STAGE_KEYS, "stage");
  if (stageUnknown) return { ok: false, error: stageUnknown };
  if (!Object.prototype.hasOwnProperty.call(value.stage, "selection")) {
    return {
      ok: false,
      error: { tag: "missing-field", path: "stage", field: "selection" },
    };
  }
  if (!Object.prototype.hasOwnProperty.call(value.stage, "bounds")) {
    return {
      ok: false,
      error: { tag: "missing-field", path: "stage", field: "bounds" },
    };
  }
  if (!Array.isArray(value.stage.selection)) {
    return {
      ok: false,
      error: { tag: "invalid-type", path: "stage", field: "selection" },
    };
  }
  if (!Array.isArray(value.stage.bounds)) {
    return {
      ok: false,
      error: { tag: "invalid-type", path: "stage", field: "bounds" },
    };
  }

  if (!Array.isArray(value.diagnostics)) {
    return {
      ok: false,
      error: { tag: "invalid-type", path: "", field: "diagnostics" },
    };
  }

  const selectionIds: string[] = [];
  for (let i = 0; i < value.stage.selection.length; i += 1) {
    const entry = decodeSelectionEntry(
      value.stage.selection[i],
      `stage.selection[${i}]`,
    );
    if ("tag" in entry) {
      return { ok: false, error: entry };
    }
    selectionIds.push(entry.layerId);
  }

  const selectionIdSet = new Set<string>();
  for (const id of selectionIds) {
    if (selectionIdSet.has(id)) {
      return { ok: false, error: { tag: "duplicate-selection-id" } };
    }
    selectionIdSet.add(id);
  }

  if (selectionIds.length !== 0 && selectionIds.length !== 1) {
    return { ok: false, error: { tag: "invalid-selection-count" } };
  }

  const boundsById = new Map<string, string>();
  for (let i = 0; i < value.stage.bounds.length; i += 1) {
    const entry = decodeBoundsEntry(
      value.stage.bounds[i],
      `stage.bounds[${i}]`,
    );
    if ("tag" in entry) {
      return { ok: false, error: entry };
    }
    if (boundsById.has(entry.layerId)) {
      return { ok: false, error: { tag: "duplicate-bounds-id" } };
    }
    boundsById.set(entry.layerId, entry.displayName);
  }

  // 失敗判定は host_handle / diagnostics を読んだ後でも意味へは載せない
  void hostHandle;
  void value.diagnostics;

  if (primaryLayerId === undefined && selectionIds.length === 0) {
    return {
      ok: true,
      value: {
        tag: "no-selection",
        revision,
        projectionGeneration,
      },
    };
  }

  if (primaryLayerId !== undefined && selectionIds.length === 0) {
    return { ok: false, error: { tag: "dangling-primary" } };
  }

  if (primaryLayerId === undefined && selectionIds.length === 1) {
    return { ok: false, error: { tag: "mismatch" } };
  }

  // primary あり・selection ちょうど1
  const soleSelectionId = selectionIds[0]!;
  if (primaryLayerId !== soleSelectionId) {
    return { ok: false, error: { tag: "mismatch" } };
  }

  const displayName = boundsById.get(soleSelectionId);
  if (displayName === undefined) {
    return { ok: false, error: { tag: "mismatch" } };
  }

  return {
    ok: true,
    value: {
      tag: "selected",
      layerId: soleSelectionId,
      displayName,
      revision,
      projectionGeneration,
    },
  };
}

/** JSON 文字列を parse し、Inspector initial-read の read model へ写す。 */
export function decodeInspectorInitialRead(
  jsonText: string,
): DecodeInspectorInitialReadResult {
  let parsed: unknown;
  try {
    parsed = JSON.parse(jsonText);
  } catch {
    return { ok: false, error: { tag: "invalid-json" } };
  }
  return decodeFromValue(parsed);
}

/** 既に parse 済みの値向け（テスト／呼び出し側の二重 parse 回避）。 */
export function decodeInspectorInitialReadValue(
  value: unknown,
): DecodeInspectorInitialReadResult {
  return decodeFromValue(value);
}
