import { TurboModuleRegistry } from 'react-native';

import type { Spec as MotoliiHostSpec } from './specs/NativeMotoliiHost';

export function nativeHost(): MotoliiHostSpec | null {
  return TurboModuleRegistry.get<MotoliiHostSpec>('NativeMotoliiHost');
}

export type MotoliiHostSpecWithTimeline = MotoliiHostSpec & {
  isTimelineInteracting: () => boolean;
};

export function nativeHostWithTimelineSignal(): MotoliiHostSpecWithTimeline | null {
  return TurboModuleRegistry.get<MotoliiHostSpecWithTimeline>(
    'NativeMotoliiHost',
  );
}

export function nativeHostIsTimelineInteracting(): boolean {
  const host = nativeHostWithTimelineSignal();
  if (!host || typeof host.isTimelineInteracting !== 'function') {
    return false;
  }
  try {
    return host.isTimelineInteracting();
  } catch {
    return false;
  }
}

export function nativeHostFitStageView() {
  const host = nativeHost();
  if (!host || typeof host.fitStageView !== 'function') {
    return;
  }
  try {
    host.fitStageView();
  } catch {
    return;
  }
}

export function nativeHostStageOneToOne() {
  const host = nativeHost();
  if (!host || typeof host.stageViewOneToOne !== 'function') {
    return;
  }
  try {
    host.stageViewOneToOne();
  } catch {
    return;
  }
}

export function commitHostStageTransform(
  target: string,
  revision: string | null,
  kind: number,
  a: number,
  b: number,
): boolean {
  const host = nativeHost();
  if (!host || typeof host.commitStageTransform !== 'function' || !revision) {
    return false;
  }
  let ok = false;
  try {
    ok = host.commitStageTransform(target, revision, kind, a, b) === true;
  } catch {
    return false;
  }
  if (!ok) {
    return false;
  }
  const state = readHostSnapshotState();
  hostSnapshotApplier?.(state);
  return true;
}

function stageTransformResult(response: string): HostIntentResult {
  try {
    const parsed = JSON.parse(response) as {
      accepted?: boolean;
      message?: unknown;
    };
    const message =
      typeof parsed.message === 'string' && parsed.message.length > 0
        ? parsed.message
        : null;
    if (parsed.accepted !== true) {
      const reason = message ?? 'Stage transform rejected';
      hostRejectApplier?.(reason);
      return { accepted: false, message: reason, reason: null };
    }
    hostRejectApplier?.(null);
    return { accepted: true, message, reason: null };
  } catch {
    hostRejectApplier?.('Stage transform rejected');
    return {
      accepted: false,
      message: 'Stage transform rejected',
      reason: null,
    };
  }
}

function unavailableStageTransform(message: string): HostIntentResult {
  hostRejectApplier?.(message);
  return { accepted: false, message, reason: null };
}

export function previewHostStageTransform(
  target: string,
  revision: string | null,
  kind: number,
  a: number,
  b: number,
): HostIntentResult {
  const host = nativeHost();
  if (!host || typeof host.previewStageTransform !== 'function') {
    return unavailableStageTransform('Stage preview is unavailable');
  }
  if (!revision) {
    return unavailableStageTransform(
      'The live Document revision is unavailable',
    );
  }
  try {
    return stageTransformResult(
      host.previewStageTransform(target, revision, kind, a, b),
    );
  } catch {
    return unavailableStageTransform('Stage preview failed');
  }
}

export function commitHostStageTransformGesture(
  target: string,
  revision: string | null,
  kind: number,
  a: number,
  b: number,
): HostIntentResult {
  const host = nativeHost();
  if (!host || typeof host.commitStageTransformGesture !== 'function') {
    return unavailableStageTransform('Stage transform commit is unavailable');
  }
  if (!revision) {
    return unavailableStageTransform(
      'The live Document revision is unavailable',
    );
  }
  try {
    const result = stageTransformResult(
      host.commitStageTransformGesture(target, revision, kind, a, b),
    );
    if (result.accepted) {
      hostSnapshotApplier?.(readHostSnapshotState());
    }
    return result;
  } catch {
    return unavailableStageTransform('Stage transform commit failed');
  }
}

export function cancelHostStageTransform(): HostIntentResult {
  const host = nativeHost();
  if (!host || typeof host.cancelStageTransform !== 'function') {
    return unavailableStageTransform(
      'Stage preview cancellation is unavailable',
    );
  }
  try {
    return stageTransformResult(host.cancelStageTransform());
  } catch {
    return unavailableStageTransform('Stage preview cancellation failed');
  }
}

/** dispatch応答snapshotの即時反映先。Appがmount時に登録する。 */
let hostSnapshotApplier: ((state: HostSnapshotState) => void) | null = null;
let hostSnapshotIdentityReader: (() => HostSnapshotIdentity | null) | null = null;
let hostRejectApplier: ((reason: string | null) => void) | null = null;

export function setHostSnapshotApplier(
  applier: ((state: HostSnapshotState) => void) | null,
  identityReader: (() => HostSnapshotIdentity | null) | null = null,
) {
  hostSnapshotApplier = applier;
  hostSnapshotIdentityReader = identityReader;
}

export function setHostRejectApplier(
  applier: ((reason: string | null) => void) | null,
) {
  hostRejectApplier = applier;
}

/** native rendererの終端通知だけが呼ぶ。snapshot JSONはこの場で最大1回読む。 */
export function applyNativeHostTerminal(
  accepted: boolean,
  message: string,
) {
  const snapshotApplier = hostSnapshotApplier;
  if (!snapshotApplier) {
    return;
  }
  hostRejectApplier?.(
    accepted ? null : message || 'Native edit rejected',
  );
  snapshotApplier(readHostSnapshotState());
}

export type HostIntentResult = {
  accepted: boolean;
  message: string | null;
  reason: string | null;
};

export type HostSnapshotIdentity = {
  hostHandle: string;
  projectionGeneration: string;
};

export function dispatchHostIntentResult(
  kind: string,
  extra: Record<string, unknown> = {},
): HostIntentResult {
  const host = nativeHost();
  if (!host) {
    return { accepted: false, message: 'host unavailable', reason: null };
  }
  try {
    const identity = hostSnapshotIdentityReader?.();
    const response = host.dispatchIntent(
      JSON.stringify({
        version: 1,
        direction: 'rn-to-host',
        kind,
        ...extra,
        ...(identity
          ? {projection_generation: identity.projectionGeneration}
          : {}),
      }),
    );
    const parsed = JSON.parse(response) as {
      accepted?: boolean;
      snapshot?: unknown;
      message?: string;
      diagnostics?: Array<{ reason?: string }>;
    };
    const reason =
      typeof parsed.diagnostics?.[0]?.reason === 'string' &&
      parsed.diagnostics[0].reason.length > 0
        ? parsed.diagnostics[0].reason
        : null;
    const message =
      typeof parsed.message === 'string' && parsed.message.length > 0
        ? parsed.message
        : reason;
    if (parsed.snapshot != null) {
      const state = hostSnapshotStateFromParsed(parsed.snapshot);
      if (state) {
        hostSnapshotApplier?.(state);
      }
    }
    if (parsed.accepted !== true) {
      hostRejectApplier?.(message ?? 'rejected');
      return { accepted: false, message: message ?? 'rejected', reason };
    }
    hostRejectApplier?.(null);
    return { accepted: true, message, reason };
  } catch {
    return { accepted: false, message: 'rejected', reason: null };
  }
}

export function dispatchHostIntent(
  kind: string,
  extra: Record<string, unknown> = {},
): boolean {
  return dispatchHostIntentResult(kind, extra).accepted;
}

export type HostRationalTime = { num: number; den: number };

export type ShellKeyEvent = {
  nativeEvent: {
    key: string;
    metaKey?: boolean;
    shiftKey?: boolean;
    ctrlKey?: boolean;
    altKey?: boolean;
    repeat?: boolean;
  };
  preventDefault?: () => void;
};

/** host_bridge mac_key_token と同じ物理コード。表は増やさない。 */
export function shellMacKeyCode(key: string): number | null {
  if (key === ' ' || key === 'Space') {
    return 49;
  }
  if (key === 'Delete') {
    return 117;
  }
  if (key === 'Backspace') {
    return 51;
  }
  const letter = key.length === 1 ? key.toLowerCase() : '';
  if (letter === 'z') {
    return 6;
  }
  if (letter === 'j') {
    return 38;
  }
  if (letter === 'k') {
    return 40;
  }
  if (letter === 'l') {
    return 37;
  }
  return null;
}

/** RN が先に食うキーだけ既存 hostKeyEvent → try_dispatch_keymap。kind 表は native 側。 */
export function dispatchHostKeyEventFromShellKey(
  event: ShellKeyEvent,
): boolean {
  const host = nativeHost();
  if (!host || typeof host.hostKeyEvent !== 'function') {
    return false;
  }
  const { key, metaKey, shiftKey, ctrlKey, altKey, repeat } = event.nativeEvent;
  const keyCode = shellMacKeyCode(key);
  if (keyCode == null) {
    return false;
  }
  let bits = 0;
  if (shiftKey === true) {
    bits |= 1;
  }
  if (ctrlKey === true) {
    bits |= 2;
  }
  if (altKey === true) {
    bits |= 4;
  }
  if (metaKey === true) {
    bits |= 8;
  }
  const chars =
    key === ' ' || key === 'Space' ? ' ' : key.length === 1 ? key : '';
  const result = host.hostKeyEvent(
    keyCode,
    bits,
    chars,
    repeat === true,
    false,
  );
  if (result === 0) {
    return false;
  }
  event.preventDefault?.();
  return true;
}

export type HostPositionKey = {
  key_id: string;
  time: HostRationalTime;
  value?: [number, number];
  interp?: unknown;
};
export type HostPositionKeyValue = {
  keyId: string;
  time: HostRationalTime;
  value: [number, number];
  interp: string | null;
};
export type HostParamKey = {
  property: 'scale' | 'rotation' | 'opacity';
  keyId: string;
  time: HostRationalTime;
  value?: number;
  vec?: [number, number];
};
export type HostEffectParam = {
  param_id: string;
  value: number;
  color?: [number, number, number, number];
};
export type HostEffectUse = {
  effect_use_id: string;
  plugin_id: string;
  name: string;
  params: HostEffectParam[];
};
export type HostCatalogEffect = {
  plugin_id: string;
  name: string;
  effect_version: number;
};
export type HostCatalogSource = {
  plugin_id: string;
  name: string;
  effect_version: number;
};
export type HostLibraryDirectory = { id: string; name: string; path: string };
export type HostLibraryTag = { id: string; label: string; count: number };
export type HostLibraryItem = {
  id: string;
  name: string;
  kind: string;
  directory: string;
  tags: string[];
};
export type HostLibrary = {
  rootPath: string | null;
  directories: HostLibraryDirectory[];
  tags: HostLibraryTag[];
  items: HostLibraryItem[];
};
export type HostSourceParam = {
  param_id: string;
  value: number;
  color?: [number, number, number, number];
};
export type StageTransform = {
  x: number;
  y: number;
  z: number;
  rotationX: number;
  rotationY: number;
  rotationZ: number;
  scaleX: number;
  scaleY: number;
};
export const INITIAL_STAGE_TRANSFORM: StageTransform = {
  x: 0,
  y: 0,
  z: 0,
  rotationX: 0,
  rotationY: 0,
  rotationZ: 0,
  scaleX: 1,
  scaleY: 1,
};
export type HostLayerSeat = {
  displayName: string;
  positionKeyCount: number;
  primaryLayerId: string | null;
  currentTime: HostRationalTime;
  /** playhead exact一致かつ value がある時だけ編集席を出す。 */
  exactKey: HostPositionKeyValue | null;
  paramKeys: HostParamKey[];
  /** playhead exact一致の param key。position exactKey とは独立。 */
  exactParamKey: HostParamKey | null;
  effects: HostEffectUse[];
  sourceParams: HostSourceParam[];
  /** selected_doc_params.opacity。欠落時は行を出さない。 */
  opacity: number | null;
  /** stage_geometry の選択layer。欠落時は Transform dial を書かない。 */
  transform: StageTransform | null;
  visible: boolean;
  solo: boolean;
};
export type HostSnapshotState = {
  hostHandle: string | null;
  projectionGeneration: string | null;
  statusLabel: string | null;
  layerSeat: HostLayerSeat | null;
  catalogEffects: HostCatalogEffect[] | null;
  catalogSources: HostCatalogSource[] | null;
  library: HostLibrary | null;
  canUndo: boolean;
  canRedo: boolean;
  revision: string | null;
  currentTime: HostRationalTime | null;
  playing: boolean;
  fps: HostRationalTime | null;
  duration: HostRationalTime | null;
};

export function rationalTimesExactEqual(
  a: HostRationalTime,
  b: HostRationalTime,
): boolean {
  return BigInt(a.num) * BigInt(b.den) === BigInt(b.num) * BigInt(a.den);
}

export function exactParamKeyFor(
  keys: HostParamKey[],
  time: HostRationalTime,
  property: HostParamKey['property'],
): HostParamKey | null {
  return (
    keys.find(
      key =>
        key.property === property && rationalTimesExactEqual(key.time, time),
    ) ?? null
  );
}

export function parseRationalTime(value: unknown): HostRationalTime | null {
  if (value == null || typeof value !== 'object') {
    return null;
  }
  const rec = value as { num?: unknown; den?: unknown };
  if (
    typeof rec.num !== 'number' ||
    typeof rec.den !== 'number' ||
    rec.den === 0
  ) {
    return null;
  }
  return { num: rec.num, den: rec.den };
}

/** composition duration 以下の最終 frame。set_time の暗黙 clamp をしない。 */
export function lastInclusiveFrame(
  fps: HostRationalTime,
  duration: HostRationalTime,
): number | null {
  const num = BigInt(duration.num) * BigInt(fps.num);
  const den = BigInt(duration.den) * BigInt(fps.den);
  if (den === 0n) {
    return null;
  }
  const frame = num / den;
  if (frame < 0n || frame > BigInt(Number.MAX_SAFE_INTEGER)) {
    return null;
  }
  return Number(frame);
}

export const EMPTY_HOST_SNAPSHOT: HostSnapshotState = {
  hostHandle: null,
  projectionGeneration: null,
  statusLabel: null,
  layerSeat: null,
  catalogEffects: null,
  catalogSources: null,
  library: null,
  canUndo: false,
  canRedo: false,
  revision: null,
  currentTime: null,
  playing: false,
  fps: null,
  duration: null,
};

export const toSaturatingNonNegativeTotal = (value: number): number => {
  if (!Number.isFinite(value)) {
    return 0;
  }
  if (value <= 0) {
    return 0;
  }
  if (value >= Number.MAX_SAFE_INTEGER) {
    return Number.MAX_SAFE_INTEGER;
  }
  return Math.floor(value);
};

/** host snapshotが読める時だけInspector Layer席へ渡す。 */
export function readHostSnapshotState(): HostSnapshotState {
  const host = nativeHost();
  if (!host) {
    return EMPTY_HOST_SNAPSHOT;
  }
  const snapshot = host.readSnapshot();
  if (!snapshot) {
    return EMPTY_HOST_SNAPSHOT;
  }
  try {
    return (
      hostSnapshotStateFromParsed(JSON.parse(snapshot)) ?? EMPTY_HOST_SNAPSHOT
    );
  } catch {
    return EMPTY_HOST_SNAPSHOT;
  }
}

export type ParsedHostSnapshot = {
  host_handle?: string;
  projection_generation?: string;
  primary_layer_id?: string | null;
  current_time?: HostRationalTime;
  revision?: string;
  truncated_total?: number;
  history?: { can_undo?: boolean; can_redo?: boolean };
  playback_state?: string;
  stage?: { bounds?: unknown[] };
  stage_geometry?: {
    layers?: Array<{
      layer_id?: string;
      position?: unknown;
      rotation?: unknown;
      scale?: unknown;
    }>;
  };
  catalog?: {
    effects?: Array<{
      plugin_id?: string;
      name?: string;
      effect_version?: number;
    }>;
    sources?: Array<{
      plugin_id?: string;
      name?: string;
      effect_version?: number;
    }>;
  };
  library?: {
    root?: { id?: string; name?: string; path?: string };
    directories?: Array<{ id?: string; name?: string; path?: string }>;
    tags?: Array<{ id?: string; label?: string; count?: number }>;
    items?: Array<{
      id?: string;
      name?: string;
      kind?: string;
      directory?: string;
      tags?: string[];
    }>;
  };
  timeline?: {
    fps?: HostRationalTime;
    duration?: HostRationalTime;
    layers_truncated?: boolean;
    layers?: Array<{
      layer_id: string;
      display_name: string;
      position_keys?: HostPositionKey[];
      param_keys?: Array<{
        property?: string;
        key_id?: string;
        time?: { num?: number; den?: number };
        value?: number;
        vec?: [number, number];
      }>;
      keys_truncated?: boolean;
      effects?: Array<{
        effect_use_id?: string;
        plugin_id?: string;
        params?: Array<{
          param_id?: string;
          value?: number;
          color?: [number, number, number, number];
        }>;
      }>;
      effects_truncated?: boolean;
      source_params?: Array<{
        param_id?: string;
        value?: number;
        color?: [number, number, number, number];
      }>;
      source_params_truncated?: boolean;
      visible?: boolean;
      solo?: boolean;
    }>;
  };
  /** 選択layerの Document params。未選択時は欠落。transformは載せない。 */
  selected_doc_params?: {
    layer_id?: string;
    opacity?: number;
    effects?: Array<{
      effect_use_id?: string;
      plugin_id?: string;
      params?: Array<{
        param_id?: string;
        value?: number;
        color?: [number, number, number, number];
      }>;
    }>;
    source_params?: Array<{
      param_id?: string;
      value?: number;
      color?: [number, number, number, number];
    }>;
  };
};

export function hostSnapshotStateFromParsed(
  raw: unknown,
): HostSnapshotState | null {
  if (raw == null || typeof raw !== 'object') {
    return null;
  }
  try {
    const parsed = raw as ParsedHostSnapshot;
    const hostHandle = parseWireCounter(parsed.host_handle);
    const projectionGeneration = parseWireCounter(
      parsed.projection_generation,
    );
    const primaryLayerId = parsed.primary_layer_id ?? null;
    const layerBounds = parsed.stage?.bounds?.length ?? 0;
    const revision = parsed.revision;
    const timelineLayers = parsed.timeline?.layers ?? [];
    const truncatedTotal =
      typeof parsed.truncated_total === 'number'
        ? toSaturatingNonNegativeTotal(parsed.truncated_total)
        : 0;
    const anyTruncated =
      truncatedTotal > 0 ||
      parsed.timeline?.layers_truncated === true ||
      timelineLayers.some(
        layer =>
          layer.keys_truncated === true ||
          layer.effects_truncated === true ||
          layer.source_params_truncated === true,
      );
    const statusLabel =
      revision == null
        ? null
        : `DOC r${revision} · ${layerBounds} layers${
            anyTruncated
              ? ` (+${truncatedTotal > 0 ? truncatedTotal : ''})`
              : ''
          }`;
    const canUndo = parsed.history?.can_undo === true;
    const canRedo = parsed.history?.can_redo === true;
    const catalogEffects = Array.isArray(parsed.catalog?.effects)
      ? parsed
          .catalog!.effects!.filter(
            item =>
              typeof item?.plugin_id === 'string' && item.plugin_id.length > 0,
          )
          .map(item => ({
            plugin_id: item.plugin_id!,
            name:
              typeof item.name === 'string' && item.name.length > 0
                ? item.name
                : item.plugin_id!,
            effect_version:
              typeof item.effect_version === 'number' ? item.effect_version : 0,
          }))
      : null;
    const catalogSources = Array.isArray(parsed.catalog?.sources)
      ? parsed
          .catalog!.sources!.filter(
            item =>
              typeof item?.plugin_id === 'string' && item.plugin_id.length > 0,
          )
          .map(item => ({
            plugin_id: item.plugin_id!,
            name:
              typeof item.name === 'string' && item.name.length > 0
                ? item.name
                : item.plugin_id!,
            effect_version:
              typeof item.effect_version === 'number' ? item.effect_version : 0,
          }))
      : null;
    const library = parseHostLibrary(parsed.library);
    const fps = parseRationalTime(parsed.timeline?.fps);
    const duration = parseRationalTime(parsed.timeline?.duration);
    if (
      !parsed.current_time ||
      typeof parsed.current_time.num !== 'number' ||
      typeof parsed.current_time.den !== 'number'
    ) {
      return {
        hostHandle,
        projectionGeneration,
        statusLabel,
        layerSeat: null,
        catalogEffects,
        catalogSources,
        library,
        canUndo,
        canRedo,
        revision: revision ?? null,
        currentTime: null,
        playing: parsed.playback_state === 'playing',
        fps,
        duration,
      };
    }
    const primary = primaryLayerId
      ? timelineLayers.find(layer => layer.layer_id === primaryLayerId)
      : undefined;
    const positionKeys = primary?.position_keys ?? [];
    const exact = positionKeys.find(
      key =>
        key &&
        typeof key.time?.num === 'number' &&
        typeof key.time?.den === 'number' &&
        typeof key.key_id === 'string' &&
        rationalTimesExactEqual(parsed.current_time!, key.time) &&
        Array.isArray(key.value) &&
        key.value.length === 2 &&
        Number.isFinite(key.value[0]) &&
        Number.isFinite(key.value[1]),
    );
    const paramKeys = (
      Array.isArray(primary?.param_keys) ? primary.param_keys : []
    )
      .map(parseHostParamKey)
      .filter((key): key is HostParamKey => key != null);
    const exactParamKey =
      paramKeys.find(key =>
        rationalTimesExactEqual(parsed.current_time!, key.time),
      ) ?? null;
    const nameByPlugin = new Map(
      (catalogEffects ?? []).map(item => [item.plugin_id, item.name]),
    );
    const docParams = parsed.selected_doc_params;
    const effectRows = docParams ? docParams.effects : primary?.effects;
    const sourceRows = docParams
      ? docParams.source_params
      : primary?.source_params;
    const effects: HostEffectUse[] = (effectRows ?? [])
      .filter(
        effect =>
          typeof effect?.effect_use_id === 'string' &&
          typeof effect?.plugin_id === 'string',
      )
      .map(effect => ({
        effect_use_id: effect.effect_use_id!,
        plugin_id: effect.plugin_id!,
        name: nameByPlugin.get(effect.plugin_id!) ?? effect.plugin_id!,
        params: (effect.params ?? [])
          .filter(
            param =>
              typeof param?.param_id === 'string' &&
              Number.isFinite(param?.value),
          )
          .map(param => {
            const color = parseRgba(param.color);
            return color
              ? {
                  param_id: param.param_id!,
                  value: param.value as number,
                  color,
                }
              : { param_id: param.param_id!, value: param.value as number };
          }),
      }));
    const sourceParams: HostSourceParam[] = (sourceRows ?? [])
      .filter(
        param =>
          typeof param?.param_id === 'string' && Number.isFinite(param?.value),
      )
      .map(param => {
        const color = parseRgba(param.color);
        return color
          ? { param_id: param.param_id!, value: param.value as number, color }
          : { param_id: param.param_id!, value: param.value as number };
      });
    return {
      hostHandle,
      projectionGeneration,
      statusLabel,
      catalogEffects,
      catalogSources,
      library,
      canUndo,
      canRedo,
      revision: revision ?? null,
      currentTime: parsed.current_time,
      playing: parsed.playback_state === 'playing',
      fps,
      duration,
      layerSeat: {
        displayName: primary?.display_name ?? 'no selection',
        positionKeyCount: positionKeys.length,
        primaryLayerId,
        currentTime: parsed.current_time,
        exactKey: exact
          ? {
              keyId: exact.key_id,
              time: exact.time,
              value: [exact.value![0], exact.value![1]],
              interp: parseInterpLabel(exact.interp),
            }
          : null,
        paramKeys,
        exactParamKey,
        effects,
        sourceParams,
        opacity: parseFiniteNumber(docParams?.opacity),
        transform: parseSelectedStageTransform(
          parsed.stage_geometry,
          primaryLayerId,
        ),
        visible: primary?.visible !== false,
        solo: primary?.solo === true,
      },
    };
  } catch {
    return null;
  }
}

function parseWireCounter(value: unknown): string | null {
  return typeof value === 'string' && /^(0|[1-9]\d*)$/.test(value)
    ? value
    : null;
}

export function parseFiniteNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}

export function parseInterpLabel(value: unknown): string | null {
  if (value === 'Hold' || value === 'Linear') {
    return value;
  }
  if (value && typeof value === 'object') {
    return 'Bezier';
  }
  return null;
}

export function parseHostParamKey(raw: unknown): HostParamKey | null {
  if (raw == null || typeof raw !== 'object') {
    return null;
  }
  const rec = raw as {
    property?: unknown;
    key_id?: unknown;
    time?: unknown;
    value?: unknown;
    vec?: unknown;
  };
  if (
    rec.property !== 'scale' &&
    rec.property !== 'rotation' &&
    rec.property !== 'opacity'
  ) {
    return null;
  }
  if (typeof rec.key_id !== 'string') {
    return null;
  }
  const time = parseRationalTime(rec.time);
  if (!time) {
    return null;
  }
  const value = parseFiniteNumber(rec.value);
  const vec = parseVec2(rec.vec);
  return {
    property: rec.property,
    keyId: rec.key_id,
    time,
    ...(value != null ? { value } : {}),
    ...(vec ? { vec } : {}),
  };
}

export function parseVec2(value: unknown): [number, number] | null {
  if (!Array.isArray(value) || value.length !== 2) {
    return null;
  }
  const x = parseFiniteNumber(value[0]);
  const y = parseFiniteNumber(value[1]);
  if (x == null || y == null) {
    return null;
  }
  return [x, y];
}

/** Color Const の 4 数配列だけ採用。欠落・長さ違い・非有限は Color ではない。 */
export function parseRgba(
  value: unknown,
): [number, number, number, number] | null {
  if (!Array.isArray(value) || value.length !== 4) {
    return null;
  }
  const r = parseFiniteNumber(value[0]);
  const g = parseFiniteNumber(value[1]);
  const b = parseFiniteNumber(value[2]);
  const a = parseFiniteNumber(value[3]);
  if (r == null || g == null || b == null || a == null) {
    return null;
  }
  return [r, g, b, a];
}

export function parseSelectedStageTransform(
  stageGeometry: ParsedHostSnapshot['stage_geometry'],
  layerId: string | null,
): StageTransform | null {
  if (!layerId || !Array.isArray(stageGeometry?.layers)) {
    return null;
  }
  const layer = stageGeometry.layers.find(item => item?.layer_id === layerId);
  if (!layer) {
    return null;
  }
  const position = parseVec2(layer.position);
  const rotation = parseFiniteNumber(layer.rotation);
  const scale = parseVec2(layer.scale) ?? [1, 1];
  if (!position || rotation == null) {
    return null;
  }
  return {
    x: position[0],
    y: position[1],
    z: 0,
    rotationX: 0,
    rotationY: 0,
    rotationZ: rotation * (180 / Math.PI),
    scaleX: scale[0],
    scaleY: scale[1],
  };
}

export function parseHostLibrary(
  raw: ParsedHostSnapshot['library'],
): HostLibrary | null {
  if (raw == null || typeof raw !== 'object') {
    return null;
  }
  const directories = Array.isArray(raw.directories)
    ? raw.directories
        .filter(
          item =>
            typeof item?.id === 'string' && typeof item?.name === 'string',
        )
        .map(item => ({
          id: item.id!,
          name: item.name!,
          path: typeof item.path === 'string' ? item.path : '',
        }))
    : [];
  const tags = Array.isArray(raw.tags)
    ? raw.tags
        .filter(
          item =>
            typeof item?.id === 'string' && typeof item?.label === 'string',
        )
        .map(item => ({
          id: item.id!,
          label: item.label!,
          count: typeof item.count === 'number' ? item.count : 0,
        }))
    : [];
  const items = Array.isArray(raw.items)
    ? raw.items
        .filter(
          item =>
            typeof item?.id === 'string' && typeof item?.name === 'string',
        )
        .map(item => ({
          id: item.id!,
          name: item.name!,
          kind: typeof item.kind === 'string' ? item.kind : 'file',
          directory: typeof item.directory === 'string' ? item.directory : '',
          tags: Array.isArray(item.tags)
            ? item.tags.filter((tag): tag is string => typeof tag === 'string')
            : [],
        }))
    : [];
  return {
    rootPath: typeof raw.root?.path === 'string' ? raw.root.path : null,
    directories,
    tags,
    items,
  };
}

export const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, Math.round(value)));

export const formatPlayhead = (time: HostRationalTime | null): string => {
  if (!time || time.den === 0) {
    return '00:00.0';
  }
  const seconds = Number(time.num) / Number(time.den);
  if (!Number.isFinite(seconds) || seconds < 0) {
    return '00:00.0';
  }
  const minutes = Math.floor(seconds / 60);
  const rest = seconds - minutes * 60;
  return `${String(minutes).padStart(2, '0')}:${rest
    .toFixed(1)
    .padStart(4, '0')}`;
};

export function playheadFraction(
  time: HostRationalTime | null,
  duration: HostRationalTime | null,
): number {
  if (!time || !duration || duration.den === 0 || time.den === 0) {
    return 0;
  }
  const t = Number(time.num) / Number(time.den);
  const d = Number(duration.num) / Number(duration.den);
  if (!Number.isFinite(t) || !Number.isFinite(d) || d <= 0) {
    return 0;
  }
  return Math.max(0, Math.min(1, t / d));
}
