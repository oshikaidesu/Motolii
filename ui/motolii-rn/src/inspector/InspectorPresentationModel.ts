export type InspectorSelectedTarget = {
  layerId: string;
  displayName: string;
  revision: string;
  projectionGeneration: string;
};

export type InspectorViewModel =
  | {state: 'none'}
  | {state: 'selected'; target: InspectorSelectedTarget}
  | {state: 'invalid'};

const ROOT_KEYS = new Set([
  'version',
  'direction',
  'role',
  'host_handle',
  'revision',
  'projection_generation',
  'primary_layer_id',
  'browser',
  'stage',
  'diagnostics',
]);
const STAGE_KEYS = new Set(['selection', 'bounds']);
const BROWSER_KEYS = new Set(['rectangle_source']);
const BROWSER_SOURCE_KEYS = new Set(['scope_ref', 'item_id']);
const SELECTION_KEYS = new Set(['layer_id']);
const BOUNDS_KEYS = new Set(['layer_id', 'display_name']);
const MAX_BROWSER_SOURCE_ID_LENGTH = 64;
const POSITIVE_DECIMAL = /^[1-9][0-9]*$/;
const DECIMAL = /^(0|[1-9][0-9]*)$/;

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function hasOnlyKeys(
  value: Record<string, unknown>,
  allowed: ReadonlySet<string>,
): boolean {
  return Object.keys(value).every(key => allowed.has(key));
}

function isDecimal(value: unknown, pattern: RegExp): value is string {
  return typeof value === 'string' && pattern.test(value);
}

function isSelectionEntry(value: unknown): value is {layer_id: string} {
  return (
    isRecord(value) &&
    hasOnlyKeys(value, SELECTION_KEYS) &&
    isDecimal(value.layer_id, POSITIVE_DECIMAL)
  );
}

function isBoundsEntry(
  value: unknown,
): value is {layer_id: string; display_name: string} {
  return (
    isRecord(value) &&
    hasOnlyKeys(value, BOUNDS_KEYS) &&
    isDecimal(value.layer_id, POSITIVE_DECIMAL) &&
    typeof value.display_name === 'string' &&
    value.display_name.trim().length > 0
  );
}

function isBrowserProjection(value: unknown, hostHandle: string): boolean {
  if (
    !isRecord(value) ||
    Object.keys(value).length !== BROWSER_KEYS.size ||
    !hasOnlyKeys(value, BROWSER_KEYS) ||
    !isRecord(value.rectangle_source) ||
    Object.keys(value.rectangle_source).length !== BROWSER_SOURCE_KEYS.size ||
    !hasOnlyKeys(value.rectangle_source, BROWSER_SOURCE_KEYS)
  ) {
    return false;
  }
  const source = value.rectangle_source;
  return (
    typeof source.scope_ref === 'string' &&
    source.scope_ref.length > 0 &&
    source.scope_ref.length <= MAX_BROWSER_SOURCE_ID_LENGTH &&
    source.scope_ref === `builtin-${hostHandle}` &&
    source.item_id === 'rectangle'
  );
}

export function decodeInspectorSnapshot(snapshotJSON?: string): InspectorViewModel {
  if (typeof snapshotJSON !== 'string') {
    return {state: 'invalid'};
  }

  let input: unknown;
  try {
    input = JSON.parse(snapshotJSON);
  } catch {
    return {state: 'invalid'};
  }

  if (!isRecord(input) || !hasOnlyKeys(input, ROOT_KEYS)) {
    return {state: 'invalid'};
  }
  if (
    input.version !== 1 ||
    input.direction !== 'host-to-rn' ||
    input.role !== 'product-runtime-seat' ||
    !isDecimal(input.host_handle, POSITIVE_DECIMAL) ||
    !isDecimal(input.revision, DECIMAL) ||
    !isDecimal(input.projection_generation, DECIMAL) ||
    !Array.isArray(input.diagnostics) ||
    !input.diagnostics.every(diagnostic => typeof diagnostic === 'string') ||
    !isRecord(input.stage) ||
    !hasOnlyKeys(input.stage, STAGE_KEYS) ||
    !Array.isArray(input.stage.selection) ||
    !input.stage.selection.every(isSelectionEntry) ||
    !Array.isArray(input.stage.bounds) ||
    !input.stage.bounds.every(isBoundsEntry)
  ) {
    return {state: 'invalid'};
  }
  if (
    Object.hasOwn(input, 'browser') &&
    !isBrowserProjection(input.browser, input.host_handle)
  ) {
    return {state: 'invalid'};
  }

  const boundLayerIds = new Set<string>();
  for (const bound of input.stage.bounds) {
    if (boundLayerIds.has(bound.layer_id)) {
      return {state: 'invalid'};
    }
    boundLayerIds.add(bound.layer_id);
  }

  if (!Object.hasOwn(input, 'primary_layer_id')) {
    return input.stage.selection.length === 0
      ? {state: 'none'}
      : {state: 'invalid'};
  }
  if (!isDecimal(input.primary_layer_id, POSITIVE_DECIMAL)) {
    return {state: 'invalid'};
  }
  if (
    input.stage.selection.length !== 1 ||
    input.stage.selection[0].layer_id !== input.primary_layer_id
  ) {
    return {state: 'invalid'};
  }

  const matchingBounds = input.stage.bounds.filter(
    bound => bound.layer_id === input.primary_layer_id,
  );
  if (matchingBounds.length !== 1) {
    return {state: 'invalid'};
  }

  return {
    state: 'selected',
    target: {
      layerId: input.primary_layer_id,
      displayName: matchingBounds[0].display_name,
      revision: input.revision,
      projectionGeneration: input.projection_generation,
    },
  };
}
