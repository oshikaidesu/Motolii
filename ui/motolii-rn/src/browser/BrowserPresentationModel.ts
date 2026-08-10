export type BrowserRectangleSource = {
  scopeRef: string;
  itemId: 'rectangle';
};

export type BrowserViewModel =
  | {state: 'available'; rectangleSource: BrowserRectangleSource}
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
const BROWSER_KEYS = new Set(['rectangle_source']);
const SOURCE_KEYS = new Set(['scope_ref', 'item_id']);
const STAGE_KEYS = new Set(['selection', 'bounds']);
const MAX_SOURCE_ID_LENGTH = 64;
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

function hasExactKeys(
  value: Record<string, unknown>,
  expected: ReadonlySet<string>,
): boolean {
  return (
    Object.keys(value).length === expected.size && hasOnlyKeys(value, expected)
  );
}

function isBoundedNonempty(value: unknown): value is string {
  return (
    typeof value === 'string' &&
    value.length > 0 &&
    value.length <= MAX_SOURCE_ID_LENGTH
  );
}

export function decodeBrowserSnapshot(snapshotJSON?: string): BrowserViewModel {
  if (typeof snapshotJSON !== 'string') {
    return {state: 'invalid'};
  }
  if (decodeInspectorSnapshot(snapshotJSON).state === 'invalid') {
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
    typeof input.host_handle !== 'string' ||
    !POSITIVE_DECIMAL.test(input.host_handle) ||
    typeof input.revision !== 'string' ||
    !DECIMAL.test(input.revision) ||
    typeof input.projection_generation !== 'string' ||
    !DECIMAL.test(input.projection_generation) ||
    !Array.isArray(input.diagnostics) ||
    !isRecord(input.stage) ||
    !hasExactKeys(input.stage, STAGE_KEYS) ||
    !Array.isArray(input.stage.selection) ||
    !Array.isArray(input.stage.bounds) ||
    !isRecord(input.browser) ||
    !hasExactKeys(input.browser, BROWSER_KEYS) ||
    !isRecord(input.browser.rectangle_source) ||
    !hasExactKeys(input.browser.rectangle_source, SOURCE_KEYS)
  ) {
    return {state: 'invalid'};
  }

  const source = input.browser.rectangle_source;
  if (
    !isBoundedNonempty(source.scope_ref) ||
    source.scope_ref !== `builtin-${input.host_handle}` ||
    source.item_id !== 'rectangle'
  ) {
    return {state: 'invalid'};
  }

  return {
    state: 'available',
    rectangleSource: {
      scopeRef: source.scope_ref,
      itemId: source.item_id,
    },
  };
}
import {decodeInspectorSnapshot} from '../inspector/InspectorPresentationModel';
