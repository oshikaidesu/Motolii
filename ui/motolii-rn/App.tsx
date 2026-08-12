import React, {useEffect, useRef, useState} from 'react';
import {
  FlatList,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  TurboModuleRegistry,
  View,
} from 'react-native';

import {panelRegistry} from './src/panels/registry';
import type {Spec as MotoliiHostSpec} from './src/specs/NativeMotoliiHost';
import MotoliiGpuView from './src/specs/MotoliiGpuViewNativeComponent';
import MotoliiTimelineView from './src/specs/MotoliiTimelineViewNativeComponent';

function nativeHost(): MotoliiHostSpec | null {
  return TurboModuleRegistry.get<MotoliiHostSpec>('NativeMotoliiHost');
}

/** dispatch応答snapshotの即時反映先。Appがmount時に登録する。 */
let hostSnapshotApplier: ((state: HostSnapshotState) => void) | null = null;

function setHostSnapshotApplier(applier: ((state: HostSnapshotState) => void) | null) {
  hostSnapshotApplier = applier;
}

function dispatchHostIntent(kind: string, extra: Record<string, unknown> = {}): boolean {
  const host = nativeHost();
  if (!host) {
    return false;
  }
  const response = host.dispatchIntent(
    JSON.stringify({
      version: 1,
      direction: 'rn-to-host',
      kind,
      host_handle: '',
      ...extra,
    }),
  );
  try {
    const parsed = JSON.parse(response) as {accepted?: boolean; snapshot?: unknown};
    if (parsed.accepted !== true) {
      return false;
    }
    // accepted応答に最新snapshotが同梱されていれば即時反映。失敗時は1s pollへ。
    if (parsed.snapshot != null) {
      const state = hostSnapshotStateFromParsed(parsed.snapshot);
      if (state) {
        hostSnapshotApplier?.(state);
      }
    }
    return true;
  } catch {
    return false;
  }
}

type HostRationalTime = {num: number; den: number};
type HostPositionKey = {
  key_id: string;
  time: HostRationalTime;
  value?: [number, number];
};
type HostEffectParam = {param_id: string; value: number};
type HostEffectUse = {
  effect_use_id: string;
  plugin_id: string;
  name: string;
  params: HostEffectParam[];
};
type HostCatalogEffect = {
  plugin_id: string;
  name: string;
  effect_version: number;
};
type HostCatalogSource = {
  plugin_id: string;
  name: string;
  effect_version: number;
};
type HostSourceParam = {param_id: string; value: number};
type HostLayerSeat = {
  displayName: string;
  positionKeyCount: number;
  primaryLayerId: string | null;
  currentTime: HostRationalTime;
  /** playhead exact一致かつ value がある時だけ編集席を出す。 */
  exactKey: {time: HostRationalTime; value: [number, number]} | null;
  effects: HostEffectUse[];
  sourceParams: HostSourceParam[];
};
type HostSnapshotState = {
  statusLabel: string | null;
  layerSeat: HostLayerSeat | null;
  catalogEffects: HostCatalogEffect[] | null;
  catalogSources: HostCatalogSource[] | null;
};

function rationalTimesExactEqual(a: HostRationalTime, b: HostRationalTime): boolean {
  return BigInt(a.num) * BigInt(b.den) === BigInt(b.num) * BigInt(a.den);
}

/** host snapshotが読める時だけInspector Layer席へ渡す。 */
function readHostSnapshotState(): HostSnapshotState {
  const host = nativeHost();
  if (!host) {
    return {statusLabel: null, layerSeat: null, catalogEffects: null, catalogSources: null};
  }
  const snapshot = host.readSnapshot();
  if (!snapshot) {
    return {statusLabel: null, layerSeat: null, catalogEffects: null, catalogSources: null};
  }
  try {
    return hostSnapshotStateFromParsed(JSON.parse(snapshot)) ?? {
      statusLabel: null,
      layerSeat: null,
      catalogEffects: null,
      catalogSources: null,
    };
  } catch {
    return {statusLabel: null, layerSeat: null, catalogEffects: null, catalogSources: null};
  }
}

type ParsedHostSnapshot = {
  primary_layer_id?: string | null;
  current_time?: HostRationalTime;
  revision?: string;
  stage?: {bounds?: unknown[]};
  catalog?: {
    effects?: Array<{plugin_id?: string; name?: string; effect_version?: number}>;
    sources?: Array<{plugin_id?: string; name?: string; effect_version?: number}>;
  };
  timeline?: {
    layers_truncated?: boolean;
    layers?: Array<{
      layer_id: string;
      display_name: string;
      position_keys?: HostPositionKey[];
      keys_truncated?: boolean;
      effects?: Array<{
        effect_use_id?: string;
        plugin_id?: string;
        params?: Array<{param_id?: string; value?: number}>;
      }>;
      effects_truncated?: boolean;
      source_params?: Array<{param_id?: string; value?: number}>;
      source_params_truncated?: boolean;
    }>;
  };
};

function hostSnapshotStateFromParsed(raw: unknown): HostSnapshotState | null {
  if (raw == null || typeof raw !== 'object') {
    return null;
  }
  try {
    const parsed = raw as ParsedHostSnapshot;
    const primaryLayerId = parsed.primary_layer_id ?? null;
    const layerBounds = parsed.stage?.bounds?.length ?? 0;
    const revision = parsed.revision;
    const timelineLayers = parsed.timeline?.layers ?? [];
    const anyTruncated =
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
        : `DOC r${revision} · ${layerBounds} layers${anyTruncated ? ' (+)' : ''}`;
    const catalogEffects = Array.isArray(parsed.catalog?.effects)
      ? parsed.catalog!.effects!
          .filter(item => typeof item?.plugin_id === 'string' && item.plugin_id.length > 0)
          .map(item => ({
            plugin_id: item.plugin_id!,
            name: typeof item.name === 'string' && item.name.length > 0 ? item.name : item.plugin_id!,
            effect_version: typeof item.effect_version === 'number' ? item.effect_version : 0,
          }))
      : null;
    const catalogSources = Array.isArray(parsed.catalog?.sources)
      ? parsed.catalog!.sources!
          .filter(item => typeof item?.plugin_id === 'string' && item.plugin_id.length > 0)
          .map(item => ({
            plugin_id: item.plugin_id!,
            name: typeof item.name === 'string' && item.name.length > 0 ? item.name : item.plugin_id!,
            effect_version: typeof item.effect_version === 'number' ? item.effect_version : 0,
          }))
      : null;
    if (!parsed.current_time || typeof parsed.current_time.num !== 'number' || typeof parsed.current_time.den !== 'number') {
      return {statusLabel, layerSeat: null, catalogEffects, catalogSources};
    }
    const primary = primaryLayerId
      ? timelineLayers.find(layer => layer.layer_id === primaryLayerId)
      : undefined;
    const positionKeys = primary?.position_keys ?? [];
    const exact = positionKeys.find(key =>
      key
      && typeof key.time?.num === 'number'
      && typeof key.time?.den === 'number'
      && rationalTimesExactEqual(parsed.current_time!, key.time)
      && Array.isArray(key.value)
      && key.value.length === 2
      && Number.isFinite(key.value[0])
      && Number.isFinite(key.value[1]),
    );
    const nameByPlugin = new Map(
      (catalogEffects ?? []).map(item => [item.plugin_id, item.name]),
    );
    const effects: HostEffectUse[] = (primary?.effects ?? [])
      .filter(effect => typeof effect?.effect_use_id === 'string' && typeof effect?.plugin_id === 'string')
      .map(effect => ({
        effect_use_id: effect.effect_use_id!,
        plugin_id: effect.plugin_id!,
        name: nameByPlugin.get(effect.plugin_id!) ?? effect.plugin_id!,
        params: (effect.params ?? [])
          .filter(param => typeof param?.param_id === 'string' && Number.isFinite(param?.value))
          .map(param => ({param_id: param.param_id!, value: param.value as number})),
      }));
    const sourceParams: HostSourceParam[] = (primary?.source_params ?? [])
      .filter(param => typeof param?.param_id === 'string' && Number.isFinite(param?.value))
      .map(param => ({param_id: param.param_id!, value: param.value as number}));
    return {
      statusLabel,
      catalogEffects,
      catalogSources,
      layerSeat: {
        displayName: primary?.display_name ?? 'no selection',
        positionKeyCount: positionKeys.length,
        primaryLayerId,
        currentTime: parsed.current_time,
        exactKey: exact
          ? {time: exact.time, value: [exact.value![0], exact.value![1]]}
          : null,
        effects,
        sourceParams,
      },
    };
  } catch {
    return null;
  }
}

const MacPressable = Pressable as React.ComponentType<
  React.ComponentProps<typeof Pressable> & {onDoubleClick?: () => void}
>;
type BrowserTab = 'MEDIA' | 'EFFECTS' | 'CREATE';
type BrowserViewMode = 'THUMBNAILS' | 'GRID' | 'LIST';
type RightPanel = 'INSPECTOR' | 'EXTENSIONS';


type PathOperationItem = {
  id: string;
  name: string;
  detail: string;
};

type BrowserItem = {
  id: string;
  name: string;
  detail: string;
  color: string;
  badge?: string;
  glyph?: string;
  unavailable?: boolean;
  testID?: string;
};

type StageTransform = {
  x: number;
  y: number;
  z: number;
  rotationX: number;
  rotationY: number;
  rotationZ: number;
};

const INITIAL_STAGE_TRANSFORM: StageTransform = {
  x: 0, y: 0, z: 0, rotationX: 0, rotationY: 0, rotationZ: 0,
};

const EFFECT_COLORS = ['#746d4b', '#5c6477', '#51455f'];

const PATH_OPERATIONS: PathOperationItem[] = [
  {id: 'pucker-bloat', name: 'Pucker / Bloat', detail: 'Contract or expand the path'},
  {id: 'zig-zag', name: 'Zig Zag', detail: 'Add ridges along the path'},
  {id: 'offset', name: 'Offset Paths', detail: 'Inset or outset closed paths'},
  {id: 'round-corners', name: 'Round Corners', detail: 'Round path corners'},
  {id: 'trim', name: 'Trim Paths', detail: 'Reveal a path range'},
  {id: 'twist', name: 'Twist', detail: 'Rotate geometry around a center'},
  {id: 'wiggle', name: 'Wiggle Paths', detail: 'Add deterministic path variation'},
  {id: 'repeater', name: 'Repeater', detail: 'Repeat a path with a transform'},
];

const CREATE_ITEMS = [
  {id: 'rectangle', name: 'Rectangle', type: 'Shape', provider: 'Built-in', glyph: '□'},
];

const clamp = (value: number, min: number, max: number) =>
  Math.max(min, Math.min(max, Math.round(value)));

const createdItemValue = (id: string, x: number, y: number) =>
  `${id}@${x.toFixed(6)},${y.toFixed(6)}`;

const RAIL_HEADINGS = new Set(['COLLECTIONS', 'TAGS', 'PACKS', 'TYPE', 'PROVIDER']);

function Splitter({
  label,
  orientation,
  onStart,
  onDelta,
  onNudge,
}: {
  label: string;
  orientation: 'vertical' | 'horizontal';
  onStart: () => void;
  onDelta: (delta: number) => void;
  onNudge: () => void;
}) {
  const pointerStart = useRef<number | null>(null);
  return (
    <Pressable
      accessibilityLabel={label}
      accessibilityRole="button"
      onPointerCancel={() => { pointerStart.current = null; }}
      onPointerDown={event => {
        pointerStart.current = orientation === 'vertical'
          ? event.nativeEvent.clientX
          : event.nativeEvent.clientY;
        onStart();
      }}
      onPointerMove={event => {
        if (pointerStart.current === null || event.nativeEvent.buttons === 0) {
          return;
        }
        const position = orientation === 'vertical'
          ? event.nativeEvent.clientX
          : event.nativeEvent.clientY;
        onDelta(position - pointerStart.current);
      }}
      onPointerUp={() => { pointerStart.current = null; }}
      onPress={onNudge}
      style={orientation === 'vertical' ? styles.vSplitter : styles.hSplitter}
    />
  );
}

function PanelHeader({title, detail}: {title: string; detail?: string}) {
  return (
    <View style={styles.panelHeader}>
      <Text style={styles.panelTitle}>{title}</Text>
      {detail ? <Text style={styles.panelDetail}>{detail}</Text> : null}
    </View>
  );
}

function BrowserResults({
  title,
  rail,
  items,
  view,
  selectedId,
  onSelect,
  onActivate,
  onDragStart,
  emptyMessage,
  testID,
}: {
  title: string;
  rail: string[];
  items: BrowserItem[];
  view: BrowserViewMode;
  selectedId: string | null;
  onSelect: (id: string) => void;
  onActivate: (id: string) => void;
  onDragStart: (id: string) => void;
  emptyMessage: string;
  testID: string;
}) {
  const columns = view === 'THUMBNAILS' ? 4 : view === 'GRID' ? 2 : 1;
  const cardWidth: '25%' | '50%' | '100%' = view === 'THUMBNAILS' ? '25%' : view === 'GRID' ? '50%' : '100%';
  return (
    <View style={styles.discoveryBody} testID={`browser-view-${testID}`}>
      <View style={styles.sourceRail}>
        {rail.map(label => (
          <Text
            key={label}
            numberOfLines={1}
            style={RAIL_HEADINGS.has(label) ? styles.railHeading : styles.railItem}>
            {label}
          </Text>
        ))}
      </View>
      <FlatList
        contentContainerStyle={styles.resultsContent}
        data={items}
        initialNumToRender={32}
        key={`${testID}-${view}`}
        keyExtractor={item => item.id}
        ListHeaderComponent={(
          <View style={styles.resultsHeader}>
            <Text style={styles.resultTitle}>{title}</Text>
            <Text style={styles.panelDetail}>{items.length}</Text>
          </View>
        )}
        ListEmptyComponent={(
          <View style={styles.emptyPanel} testID={`browser-empty-${testID}`}>
            <Text style={styles.emptyTitle}>{emptyMessage}</Text>
          </View>
        )}
        maxToRenderPerBatch={32}
        numColumns={columns}
        removeClippedSubviews
        renderItem={({item}) => (
          <MacPressable
            accessibilityLabel={`${item.name}${item.detail ? `, ${item.detail}` : ''}`}
            accessibilityRole="button"
            accessibilityState={{selected: selectedId === item.id}}
            onDoubleClick={() => onActivate(item.id)}
            onPointerDown={() => onDragStart(item.id)}
            onPress={() => onSelect(item.id)}
            testID={item.testID}
            style={[
              styles.browserCard,
              {width: cardWidth},
              view === 'THUMBNAILS' && styles.thumbnailCard,
              view === 'LIST' && styles.browserListCard,
              selectedId === item.id && styles.effectSelected,
            ]}>
            <View
              style={[
                styles.browserThumb,
                view === 'THUMBNAILS' && styles.thumbnailOnlyThumb,
                view === 'LIST' && styles.browserListThumb,
                {backgroundColor: item.color},
              ]}>
              {item.glyph ? <Text style={styles.createGlyph}>{item.glyph}</Text> : null}
              {item.badge ? <Text style={styles.effectBadge}>{item.badge}</Text> : null}
              {item.unavailable ? <Text style={styles.unavailable}>UNAVAILABLE</Text> : null}
              {view !== 'THUMBNAILS' && item.badge ? <Text style={styles.playBadge}>▶</Text> : null}
            </View>
            {view !== 'THUMBNAILS' ? (
              <View style={view === 'LIST' ? styles.browserListCopy : undefined}>
                <Text numberOfLines={1} style={styles.effectName}>{item.name}</Text>
                <Text numberOfLines={1} style={styles.effectTags}>{item.detail}</Text>
              </View>
            ) : null}
          </MacPressable>
        )}
        style={styles.results}
        testID={testID === 'MEDIA' ? 'thumbnail-grid' : `browser-results-${testID}`}
        windowSize={5}
      />
    </View>
  );
}

function Browser({
  width,
  onDragStart,
  onDragCancel,
  catalogEffects,
  catalogSources,
  primaryLayerId,
  currentTime,
}: {
  width: number;
  onDragStart: (id: string) => void;
  onDragCancel: () => void;
  catalogEffects: HostCatalogEffect[] | null;
  catalogSources: HostCatalogSource[] | null;
  primaryLayerId: string | null;
  currentTime: HostRationalTime | null;
}) {
  const [tab, setTab] = useState<BrowserTab>('EFFECTS');
  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const [selectedCreateItem, setSelectedCreateItem] = useState<string | null>(null);
  const [view, setView] = useState<BrowserViewMode>('GRID');
  useEffect(() => {
    if (!catalogEffects || catalogEffects.length === 0) {
      setSelected(null);
      return;
    }
    setSelected(current =>
      current !== null && catalogEffects.some(item => item.plugin_id === current)
        ? current
        : catalogEffects[0].plugin_id,
    );
  }, [catalogEffects]);
  const effectSource = (catalogEffects ?? [])
    .map((item, index) => ({
      id: item.plugin_id,
      name: item.name,
      badge: 'FX',
      tags: item.plugin_id,
      color: EFFECT_COLORS[index % EFFECT_COLORS.length],
    }));
  const filteredEffects = effectSource.filter(item =>
    `${item.name} ${item.tags}`.toLowerCase().includes(query.toLowerCase()),
  );
  const createItems = catalogSources
    ? [
      ...CREATE_ITEMS,
      ...catalogSources.map(item => ({
        id: item.plugin_id,
        name: item.name,
        type: 'Vism',
        provider: 'Plugin',
        glyph: '□',
      })),
    ]
    : CREATE_ITEMS;
  const filteredCreateItems = createItems.filter(item =>
    `${item.name} ${item.type} ${item.provider}`.toLowerCase().includes(query.toLowerCase()),
  );
  const browserItems: BrowserItem[] = tab === 'EFFECTS'
    ? filteredEffects.map(item => ({
      id: item.id,
      name: item.name,
      detail: item.tags,
      color: item.color,
      badge: item.badge,
      testID: `effect-item-${item.id}`,
    }))
    : tab === 'MEDIA'
      ? []
      : filteredCreateItems.map(item => ({
        id: item.id,
        name: item.name,
        detail: `${item.type} · ${item.provider}`,
        color: '#2d2b25',
        glyph: item.glyph,
        testID: `create-item-${item.id}`,
      }));
  const rail = tab === 'EFFECTS'
    ? ['▦  All effects']
    : tab === 'MEDIA'
      ? ['▦  All media']
      : ['▦  All items', 'TYPE', '□  Shapes', 'PROVIDER', 'M  Built-in', '□  Vism sources'];
  const title = tab === 'CREATE' ? 'Create items' : 'Results';
  const emptyMessage = query.trim()
    ? 'No results'
    : tab === 'MEDIA'
      ? 'No media imported'
      : tab === 'EFFECTS'
        ? 'No effects available'
        : 'No create items available';
  const selectedId = tab === 'EFFECTS' ? selected : tab === 'CREATE' ? selectedCreateItem : null;

  return (
    <View style={[styles.browser, {width}]} testID="browser-surface">
      <PanelHeader title="Browser" detail="MEDIA / CREATE / EFFECTS" />
      <View style={styles.tabRow}>
        {(['MEDIA', 'EFFECTS', 'CREATE'] as BrowserTab[]).map(value => (
          <Pressable
            accessibilityRole="tab"
            accessibilityState={{selected: tab === value}}
            key={value}
            onPress={() => setTab(value)}
            testID={`browser-tab-${value}`}
            style={[styles.tab, tab === value && styles.tabActive]}>
            <Text style={styles.tabText}>{value[0] + value.slice(1).toLowerCase()}</Text>
          </Pressable>
        ))}
      </View>
      <View style={styles.searchRow}>
        <TextInput
          accessibilityLabel={`Search ${tab.toLowerCase()}`}
          onChangeText={setQuery}
          placeholder={`Search ${tab.toLowerCase()}`}
          placeholderTextColor="#747984"
          style={styles.search}
          value={query}
        />
        <Pressable
          accessibilityLabel="Thumbnail only view"
          onPress={() => setView('THUMBNAILS')}
          style={[styles.iconButton, view === 'THUMBNAILS' && styles.iconButtonActive]}
          testID="browser-mode-THUMBNAILS">
          <Text style={styles.iconText}>▦</Text>
        </Pressable>
        <Pressable
          accessibilityLabel="Card view"
          onPress={() => setView('GRID')}
          style={[styles.iconButton, view === 'GRID' && styles.iconButtonActive]}
          testID="browser-mode-GRID">
          <Text style={styles.iconText}>▤</Text>
        </Pressable>
        <Pressable
          accessibilityLabel="List view"
          onPress={() => setView('LIST')}
          style={[styles.iconButton, view === 'LIST' && styles.iconButtonActive]}
          testID="browser-mode-LIST">
          <Text style={styles.iconText}>☷</Text>
        </Pressable>
      </View>
      <BrowserResults
        items={browserItems}
        emptyMessage={emptyMessage}
        onActivate={id => {
          if (tab === 'CREATE') {
            setSelectedCreateItem(id);
            onDragCancel();
            if (catalogSources?.some(item => item.plugin_id === id)) {
              dispatchHostIntent('place_vism', {
                plugin_id: id,
                position: [0, 0],
                playhead: currentTime ?? {num: 0, den: 1},
              });
              return;
            }
            if (id === 'rectangle') {
              dispatchHostIntent('place_rectangle', {
                position: [0, 0],
                playhead: currentTime ?? {num: 0, den: 1},
              });
            }
            return;
          }
          if (tab === 'EFFECTS' && catalogEffects) {
            if (!primaryLayerId) {
              return;
            }
            dispatchHostIntent('attach_effect', {
              target: primaryLayerId,
              plugin_id: id,
            });
          }
        }}
        onDragStart={id => {
          if (tab === 'CREATE') {
            setSelectedCreateItem(id);
            // Stage drop は Rectangle のみ。CREATE の source カードは drag 開始しない。
            if (catalogSources?.some(item => item.plugin_id === id)) {
              return;
            }
            onDragStart(id);
          }
        }}
        onSelect={id => tab === 'EFFECTS' ? setSelected(id) : tab === 'CREATE' ? setSelectedCreateItem(id) : undefined}
        rail={rail}
        selectedId={selectedId}
        testID={tab}
        title={title}
        view={view}
      />
    </View>
  );
}

function Stage({createdItemId, draggedItemId, pathOperationId, onDrop, onTransform, transform}: {createdItemId: string; draggedItemId: string; pathOperationId: string; onDrop: (x: number, y: number, canonicalX: number, canonicalY: number) => void; onTransform: (transform: StageTransform) => void; transform: StageTransform}) {
  return (
    <View style={styles.stage} testID="stage-surface">
      <View style={styles.stageViewport} testID="stage-viewport">
        <MotoliiGpuView
          accessible
          accessibilityLabel="Rerun Spatial Viewer Stage"
          createdItemId={`${createdItemId || 'rectangle@0.500000,0.500000'}|${pathOperationId}`}
          draggedItemId={draggedItemId}
          transformX={transform.x}
          transformY={transform.y}
          transformZ={transform.z}
          rotationX={transform.rotationX}
          rotationY={transform.rotationY}
          rotationZ={transform.rotationZ}
          onStageDrop={event =>
            onDrop(
              event.nativeEvent.x,
              event.nativeEvent.y,
              event.nativeEvent.canonicalX,
              event.nativeEvent.canonicalY,
            )
          }
          onStageTransform={event => onTransform(event.nativeEvent)}
          style={styles.gpuStage}
          testID="rust-wgpu-stage"
        />
      </View>
    </View>
  );
}

function Inspector({width, layerSeat}: {width: number; layerSeat: HostLayerSeat | null}) {
  const [panel, setPanel] = useState<RightPanel>('INSPECTOR');
  const [extensionId, setExtensionId] = useState<string>(panelRegistry[0].id);
  const extension = panelRegistry.find(item => item.id === extensionId)!;
  const selectedLayerSeat = layerSeat?.primaryLayerId ? layerSeat : null;

  return (
    <View style={[styles.inspector, {width}]} testID="inspector-surface">
      <PanelHeader title={panel === 'INSPECTOR' ? 'Inspector' : 'Extensions'} detail="ON OBJECT" />
      <View style={styles.tabRow}>
        {(['INSPECTOR', 'EXTENSIONS'] as RightPanel[]).map(value => (
          <Pressable key={value} onPress={() => setPanel(value)} style={[styles.tab, panel === value && styles.tabActive]} testID={`right-panel-${value}`}>
            <Text style={styles.tabText}>{value === 'INSPECTOR' ? 'Effect' : 'Custom'}</Text>
          </Pressable>
        ))}
      </View>
      {panel === 'INSPECTOR' ? (
        <ScrollView disableScrollViewPanResponder>
          {selectedLayerSeat ? (
            <View style={styles.pathOperationSection} testID="inspector-layer-section">
              <Text style={styles.pathOperationTitle}>Layer</Text>
              <Text style={styles.pathOperationDescription}>{selectedLayerSeat.displayName}</Text>
              <Text style={styles.pathOperationDescription}>position keys: {selectedLayerSeat.positionKeyCount}</Text>
              {selectedLayerSeat.exactKey ? (
                <ExactOnKeyValueEditor
                  key={`${selectedLayerSeat.primaryLayerId}:${selectedLayerSeat.exactKey.time.num}/${selectedLayerSeat.exactKey.time.den}`}
                  primaryLayerId={selectedLayerSeat.primaryLayerId!}
                  keyTime={selectedLayerSeat.exactKey.time}
                  value={selectedLayerSeat.exactKey.value}
                />
              ) : null}
              <View style={styles.pathOperationGrid}>
                <Pressable
                  accessibilityRole="button"
                  accessibilityLabel="Add Position Key"
                  onPress={() => {
                    // add_position_keyはtarget+time必須(rn_product_host 718-747)。Wake時刻は使わない。
                    dispatchHostIntent('add_position_key', {
                      target: selectedLayerSeat.primaryLayerId,
                      time: selectedLayerSeat.currentTime,
                    });
                  }}
                  style={styles.pathOperationButton}
                  testID="inspector-add-position-key">
                  <Text style={styles.pathOperationButtonText}>◆ Add Position Key</Text>
                </Pressable>
              </View>
            </View>
          ) : (
            <View style={styles.inspectorEmpty} testID="inspector-empty-state">
              <Text style={styles.inspectorEmptyTitle}>No layer selected</Text>
              <Text style={styles.muted}>Select a layer on the Stage or Timeline to edit it.</Text>
            </View>
          )}
          {selectedLayerSeat && selectedLayerSeat.sourceParams.length > 0 ? (
            <View style={styles.pathOperationSection} testID="inspector-source-params-section">
              <Text style={styles.pathOperationTitle}>Source</Text>
              {selectedLayerSeat.sourceParams.map(param => (
                <ParameterRow
                  key={`source:${param.param_id}`}
                  label={param.param_id}
                  value={String(param.value)}
                  testID={`inspector-source-param-${param.param_id}`}
                />
              ))}
            </View>
          ) : null}
          {selectedLayerSeat && selectedLayerSeat.effects.length > 0 ? (
            <View style={styles.pathOperationSection} testID="inspector-effects-section">
              <Text style={styles.pathOperationTitle}>Effects</Text>
              {selectedLayerSeat.effects.map(effect => (
                <View key={effect.effect_use_id} testID={`inspector-effect-${effect.effect_use_id}`}>
                  <Text style={styles.pathOperationDescription}>{effect.name}</Text>
                  {effect.params.map(param => (
                    <EffectParamEditor
                      key={`${selectedLayerSeat.primaryLayerId}:${effect.effect_use_id}:${param.param_id}`}
                      primaryLayerId={selectedLayerSeat.primaryLayerId!}
                      effectUseId={effect.effect_use_id}
                      pluginId={effect.plugin_id}
                      paramId={param.param_id}
                      value={param.value}
                    />
                  ))}
                </View>
              ))}
            </View>
          ) : null}
        </ScrollView>
      ) : (
        <View style={styles.extensionBody}>
          <View style={styles.extensionTabs}>
            {panelRegistry.map(item => (
              <Pressable key={item.id} onPress={() => setExtensionId(item.id)} style={[styles.extensionTab, extensionId === item.id && styles.iconButtonActive]}>
                <Text style={styles.tabText}>{item.title}</Text>
              </Pressable>
            ))}
          </View>
          <extension.Component />
        </View>
      )}
    </View>
  );
}

function ParameterRow({label, value, onDecrease, onIncrease, testID}: {label: string; value: string; onDecrease?: () => void; onIncrease?: () => void; testID?: string}) {
  return (
    <View style={styles.parameterRow} testID={testID}>
      <Text style={styles.parameterLabel}>{label}</Text>
      {onDecrease ? <Pressable accessibilityLabel={`${label} decrease`} onPress={onDecrease} style={styles.stepButton}><Text style={styles.stepText}>−</Text></Pressable> : null}
      <Text numberOfLines={1} style={styles.parameterValue}>{value}</Text>
      {onIncrease ? <Pressable accessibilityLabel={`${label} increase`} onPress={onIncrease} style={styles.stepButton}><Text style={styles.stepText}>＋</Text></Pressable> : null}
    </View>
  );
}

/** effect f64 param: commit 時1回 set_effect_param。二重送信防止と巻き戻しは exact-on-key と同作法。 */
function EffectParamEditor({
  primaryLayerId,
  effectUseId,
  pluginId,
  paramId,
  value,
}: {
  primaryLayerId: string;
  effectUseId: string;
  pluginId: string;
  paramId: string;
  value: number;
}) {
  const [draft, setDraft] = useState(String(value));
  const live = useRef(value);
  const editing = useRef(false);

  useEffect(() => {
    live.current = value;
    if (!editing.current) {
      setDraft(String(value));
    }
  }, [value]);

  const commit = () => {
    if (!editing.current) {
      return;
    }
    editing.current = false;
    if (draft.trim().length === 0) {
      setDraft(String(live.current));
      return;
    }
    const parsed = Number(draft);
    if (!Number.isFinite(parsed)) {
      setDraft(String(live.current));
      return;
    }
    const isOpacityAmount = pluginId === 'core.filter.opacity' && paramId === 'amount';
    // opacity amount のみ 0..1 にクランプして、他paramはそのまま送信。
    const committed = isOpacityAmount ? Math.max(0, Math.min(1, parsed)) : parsed;
    const accepted = dispatchHostIntent('set_effect_param', {
      target: primaryLayerId,
      effect_use_id: effectUseId,
      param_id: paramId,
      value: committed,
    });
    if (!accepted) {
      setDraft(String(live.current));
      return;
    }
    live.current = committed;
    setDraft(String(committed));
  };

  return (
    <View style={styles.parameterRow} testID={`inspector-effect-param-${effectUseId}-${paramId}`}>
      <Text style={styles.parameterLabel}>{paramId}</Text>
      <TextInput
        accessibilityLabel={`Effect ${paramId}`}
        keyboardType="decimal-pad"
        onBlur={commit}
        onChangeText={text => {
          editing.current = true;
          setDraft(text);
        }}
        onFocus={() => {
          editing.current = true;
        }}
        onSubmitEditing={() => {}}
        selectTextOnFocus
        style={styles.parameterValue}
        testID={`inspector-effect-param-input-${effectUseId}-${paramId}`}
        value={draft}
      />
    </View>
  );
}

/** U4b-0V: exact-on-key の時だけ X/Y を commit 時1回で送る。Dialは使わない。 */
function ExactOnKeyValueEditor({
  primaryLayerId,
  keyTime,
  value,
}: {
  primaryLayerId: string;
  keyTime: HostRationalTime;
  value: [number, number];
}) {
  const [draftX, setDraftX] = useState(String(value[0]));
  const [draftY, setDraftY] = useState(String(value[1]));
  const live = useRef(value);
  const editingX = useRef(false);
  const editingY = useRef(false);
  const valueX = value[0];
  const valueY = value[1];

  useEffect(() => {
    live.current = [valueX, valueY];
    if (!editingX.current) {
      setDraftX(String(valueX));
    }
    if (!editingY.current) {
      setDraftY(String(valueY));
    }
  }, [valueX, valueY]);

  const commitAxis = (axis: 0 | 1, draft: string) => {
    const isEditing = axis === 0 ? editingX : editingY;
    if (!isEditing.current) {
      return;
    }
    isEditing.current = false;

    if (draft.trim().length === 0) {
      if (axis === 0) {
        setDraftX(String(live.current[0]));
      } else {
        setDraftY(String(live.current[1]));
      }
      return;
    }

    const parsed = Number(draft);
    if (!Number.isFinite(parsed)) {
      if (axis === 0) {
        setDraftX(String(live.current[0]));
      } else {
        setDraftY(String(live.current[1]));
      }
      return;
    }
    const next: [number, number] = [live.current[0], live.current[1]];
    next[axis] = parsed;
    // set_position_key_value: target/time/new (rn_product_host 743-849)
    const accepted = dispatchHostIntent('set_position_key_value', {
      target: primaryLayerId,
      time: keyTime,
      new: next,
    });
    if (!accepted) {
      if (axis === 0) {
        setDraftX(String(live.current[0]));
      } else {
        setDraftY(String(live.current[1]));
      }
      return;
    }
    live.current = next;
    // 他軸が編集中ならその draft を上書きしない。
    if (axis === 0) {
      setDraftX(String(next[0]));
      if (!editingY.current) {
        setDraftY(String(next[1]));
      }
    } else {
      setDraftY(String(next[1]));
      if (!editingX.current) {
        setDraftX(String(next[0]));
      }
    }
  };

  return (
    <>
      <View style={styles.parameterRow} testID="inspector-position-key-x-row">
        <Text style={styles.parameterLabel}>X</Text>
        <TextInput
          accessibilityLabel="Position key X"
          keyboardType="decimal-pad"
          onBlur={() => commitAxis(0, draftX)}
        onChangeText={text => {
          editingX.current = true;
          setDraftX(text);
        }}
        onFocus={() => {
          editingX.current = true;
        }}
        onSubmitEditing={() => {}}
        selectTextOnFocus
        style={styles.parameterValue}
        testID="inspector-position-key-x"
        value={draftX}
        />
      </View>
      <View style={styles.parameterRow} testID="inspector-position-key-y-row">
        <Text style={styles.parameterLabel}>Y</Text>
        <TextInput
          accessibilityLabel="Position key Y"
          keyboardType="decimal-pad"
          onBlur={() => commitAxis(1, draftY)}
        onChangeText={text => {
          editingY.current = true;
          setDraftY(text);
        }}
        onFocus={() => {
          editingY.current = true;
        }}
        onSubmitEditing={() => {}}
        selectTextOnFocus
        style={styles.parameterValue}
        testID="inspector-position-key-y"
          value={draftY}
        />
      </View>
    </>
  );
}

function NativeTimeline() {
  const [selectedObjectIndex, setSelectedObjectIndex] = useState(-1);
  const [playhead, setPlayhead] = useState(0);

  return (
    <View style={styles.nativeTimelineBody}>
      <MotoliiTimelineView
        accessible
        accessibilityLabel="Timeline editing surface"
        onTimelineFeedback={event => {
          setSelectedObjectIndex(event.nativeEvent.objectIndex);
          setPlayhead(event.nativeEvent.time);
        }}
        playhead={playhead}
        selectedObjectIndex={selectedObjectIndex}
        style={styles.nativeTimelineSurface}
        testID="rust-wgpu-timeline"
      />
    </View>
  );
}

function Timeline({height}: {height: number}) {
  return (
    <View style={[styles.timeline, {height}]} testID="timeline">
      <View style={styles.timelineHeader}>
        <Text style={styles.panelTitle}>Timeline</Text>
      </View>
      <NativeTimeline />
    </View>
  );
}

function App() {
  const [browserWidth, setBrowserWidth] = useState(284);
  const [inspectorWidth, setInspectorWidth] = useState(326);
  const [timelineHeight, setTimelineHeight] = useState(270);
  const [createdItemId, setCreatedItemId] = useState('');
  const [draggedItemId, setDraggedItemId] = useState('');
  const pathOperationId = PATH_OPERATIONS[0].id;
  const [stageTransform, setStageTransform] = useState<StageTransform>(INITIAL_STAGE_TRANSFORM);
  const [hostStatusLabel, setHostStatusLabel] = useState<string | null>(null);
  const [hostLayerSeat, setHostLayerSeat] = useState<HostLayerSeat | null>(null);
  const [hostCatalogEffects, setHostCatalogEffects] = useState<HostCatalogEffect[] | null>(null);
  const [hostCatalogSources, setHostCatalogSources] = useState<HostCatalogSource[] | null>(null);
  const browserStart = useRef(browserWidth);
  const inspectorStart = useRef(inspectorWidth);
  const timelineStart = useRef(timelineHeight);
  useEffect(() => {
    const apply = (snapshotState: HostSnapshotState) => {
      setHostStatusLabel(snapshotState.statusLabel);
      setHostLayerSeat(snapshotState.layerSeat);
      setHostCatalogEffects(snapshotState.catalogEffects);
      setHostCatalogSources(snapshotState.catalogSources);
    };
    setHostSnapshotApplier(apply);
    const tick = () => {
      apply(readHostSnapshotState());
    };
    tick();
    // 1s pollは補助。dispatch応答snapshotの即時反映が主経路。
    const id = setInterval(tick, 1000);
    return () => {
      clearInterval(id);
      setHostSnapshotApplier(null);
    };
  }, []);
  const completeStageDrop = (x: number, y: number, canonicalX: number, canonicalY: number) => {
    const itemId = draggedItemId;
    setDraggedItemId('');
    if (itemId && x >= 0 && y >= 0) {
      setCreatedItemId(createdItemValue(itemId, x, y));
      if (itemId === 'rectangle') {
        dispatchHostIntent('place_rectangle', {
          position: [canonicalX, canonicalY],
          playhead: hostLayerSeat?.currentTime ?? {num: 0, den: 1},
        });
      }
    }
  };

  return (
    <View style={styles.shell} testID="motolii-rn-shell">
      <View style={styles.titlebar}>
        <Text style={styles.brand}>MOTOLII</Text>
        {hostStatusLabel ? <Text style={styles.statusLabel}>{hostStatusLabel}</Text> : null}
        <View style={styles.grow} />
        {([['undo', '↶ Undo'], ['redo', '↷ Redo']] as const).map(([kind, label]) => (
          <Pressable
            accessibilityLabel={label}
            key={kind}
            onPress={() => dispatchHostIntent(kind)}
            style={styles.titleAction}
            testID={`titlebar-${kind}`}>
            <Text style={styles.titleActionText}>{label}</Text>
          </Pressable>
        ))}
      </View>
      <View style={styles.workspace}>
        <Browser
          catalogEffects={hostCatalogEffects}
          catalogSources={hostCatalogSources}
          onDragCancel={() => setDraggedItemId('')}
          onDragStart={setDraggedItemId}
          primaryLayerId={hostLayerSeat?.primaryLayerId ?? null}
          currentTime={hostLayerSeat?.currentTime ?? null}
          width={browserWidth}
        />
        <Splitter
          label="Browserのサイズを変更"
          orientation="vertical"
          onStart={() => { browserStart.current = browserWidth; }}
          onDelta={delta => setBrowserWidth(clamp(browserStart.current + delta, 210, 430))}
          onNudge={() => setBrowserWidth(value => value >= 348 ? 284 : value + 64)}
        />
        <View style={styles.centerColumn}>
          <Stage createdItemId={createdItemId} draggedItemId={draggedItemId} pathOperationId={pathOperationId} onDrop={completeStageDrop} onTransform={setStageTransform} transform={stageTransform} />
        </View>
        <Splitter
          label="Inspectorのサイズを変更"
          orientation="vertical"
          onStart={() => { inspectorStart.current = inspectorWidth; }}
          onDelta={delta => setInspectorWidth(clamp(inspectorStart.current - delta, 240, 440))}
          onNudge={() => setInspectorWidth(value => value >= 390 ? 326 : value + 64)}
        />
        <Inspector width={inspectorWidth} layerSeat={hostLayerSeat} />
      </View>
      <Splitter
        label="Timelineのサイズを変更"
        orientation="horizontal"
        onStart={() => { timelineStart.current = timelineHeight; }}
        onDelta={delta => setTimelineHeight(clamp(timelineStart.current - delta, 190, 460))}
        onNudge={() => setTimelineHeight(value => value >= 334 ? 270 : value + 64)}
      />
      <Timeline height={timelineHeight} />
    </View>
  );
}

const styles = StyleSheet.create({
  shell: {flex: 1, minWidth: 980, minHeight: 650, backgroundColor: '#111315'},
  titlebar: {height: 34, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 12, borderBottomWidth: 1, borderBottomColor: '#36393d', backgroundColor: '#202224'},
  brand: {fontSize: 12, fontWeight: '800', letterSpacing: 1.4, color: '#f2f2f0'},
  statusLabel: {marginLeft: 14, fontSize: 8, color: '#858a8d'},
  grow: {flex: 1},
  titleAction: {fontSize: 10, color: '#d8d8d5', paddingVertical: 6, paddingHorizontal: 10, marginLeft: 6, borderWidth: 1, borderColor: '#414448'},
  titleActionText: {fontSize: 10, color: '#d8d8d5'},
  workspace: {flex: 1, flexDirection: 'row', minHeight: 260},
  centerColumn: {flex: 1, minWidth: 420},
  browser: {backgroundColor: '#202326'},
  inspector: {backgroundColor: '#1b1d20'},
  inspectorEmpty: {paddingHorizontal: 14, paddingVertical: 18, gap: 5},
  inspectorEmptyTitle: {fontSize: 11, fontWeight: '600', color: '#d5d6d3'},
  panelHeader: {height: 31, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 9, borderBottomWidth: 1, borderBottomColor: '#3a3d41'},
  panelTitle: {fontSize: 11, fontWeight: '700', color: '#dedfdd'},
  panelDetail: {marginLeft: 'auto', fontSize: 8, color: '#85898c'},
  tabRow: {height: 31, flexDirection: 'row', borderBottomWidth: 1, borderBottomColor: '#3a3d41'},
  tab: {flex: 1, alignItems: 'center', justifyContent: 'center'},
  tabActive: {borderBottomWidth: 2, borderBottomColor: '#b4a66a', backgroundColor: '#191b1e'},
  tabText: {fontSize: 9, color: '#c6c7c5'},
  searchRow: {height: 36, flexDirection: 'row', alignItems: 'center', gap: 4, paddingHorizontal: 5, borderBottomWidth: 1, borderBottomColor: '#33363a'},
  search: {flex: 1, height: 27, paddingHorizontal: 7, fontSize: 10, color: '#efefec', borderWidth: 1, borderColor: '#44484d', backgroundColor: '#17191b'},
  iconButton: {width: 27, height: 27, alignItems: 'center', justifyContent: 'center', borderWidth: 1, borderColor: '#45494d'},
  iconButtonActive: {borderColor: '#c6b975', backgroundColor: '#38372f'},
  iconText: {fontSize: 10, color: '#d4d5d3'},
  discoveryBody: {flex: 1, flexDirection: 'row'},
  sourceRail: {width: 104, paddingTop: 4, borderRightWidth: 1, borderRightColor: '#3a3d41', backgroundColor: '#181a1d'},
  railItem: {height: 23, paddingHorizontal: 8, paddingTop: 5, fontSize: 9, color: '#b9bcbd'},
  railHeading: {height: 20, paddingHorizontal: 8, paddingTop: 7, fontSize: 7, letterSpacing: 0.8, color: '#74797c'},
  results: {flex: 1},
  resultsContent: {paddingBottom: 4},
  resultsHeader: {height: 28, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 7},
  resultTitle: {fontSize: 10, fontWeight: '700', color: '#d5d6d3'},
  browserCard: {padding: 4, opacity: 0.92},
  thumbnailCard: {padding: 3},
  browserListCard: {flexDirection: 'row', alignItems: 'center'},
  browserListCopy: {flex: 1, paddingLeft: 6},
  effectSelected: {backgroundColor: '#393b3b'},
  browserThumb: {height: 52, padding: 4, justifyContent: 'space-between', borderWidth: 1, borderColor: '#3c4044'},
  thumbnailOnlyThumb: {height: 42, padding: 3},
  browserListThumb: {width: 52, height: 36},
  createGlyph: {fontSize: 25, color: '#e8dfb3'},
  effectBadge: {fontSize: 10, color: '#ffffff'},
  unavailable: {fontSize: 7, color: '#f0cfbc'},
  playBadge: {alignSelf: 'flex-end', fontSize: 9, color: '#ffffff'},
  effectName: {marginTop: 3, fontSize: 9, fontWeight: '700', color: '#ededeb'},
  effectTags: {fontSize: 7, color: '#9ca0a2'},
  emptyPanel: {padding: 14},
  emptyTitle: {fontSize: 13, color: '#e2e3e1'},
  muted: {fontSize: 9, color: '#858a8d'},
  vSplitter: {width: 8, backgroundColor: '#272a2d', borderLeftWidth: 1, borderRightWidth: 1, borderColor: '#3e4246'},
  hSplitter: {height: 8, backgroundColor: '#272a2d', borderTopWidth: 1, borderBottomWidth: 1, borderColor: '#3e4246'},
  stage: {flex: 1, backgroundColor: '#0d0f11'},
  stageViewport: {flex: 1, overflow: 'hidden'},
  gpuStage: {position: 'absolute', inset: 0},
  effectIdentity: {flexDirection: 'row', gap: 10, padding: 10, alignItems: 'center'},
  effectIcon: {width: 38, height: 38, borderWidth: 1, borderColor: '#b6aa70', alignItems: 'center', justifyContent: 'center'},
  effectIconText: {color: '#cabe7d'},
  inspectorTitle: {fontSize: 12, fontWeight: '700', color: '#f0f0ed'},
  inspectorDescription: {padding: 10, fontSize: 9, lineHeight: 14, color: '#c0c2c2', borderTopWidth: 1, borderBottomWidth: 1, borderColor: '#383c40'},
  parameterRow: {height: 34, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 8, borderBottomWidth: 1, borderBottomColor: '#33363a'},
  parameterLabel: {width: 74, fontSize: 9, color: '#b9bcbd'},
  parameterValue: {flex: 1, paddingHorizontal: 7, paddingVertical: 4, fontSize: 9, textAlign: 'right', color: '#e7e7e4', borderWidth: 1, borderColor: '#45494d'},
  stepButton: {width: 25, height: 24, alignItems: 'center', justifyContent: 'center'},
  stepText: {fontSize: 11, color: '#c8bc7b'},
  inspectorInput: {height: 30, margin: 9, paddingHorizontal: 7, fontSize: 9, color: '#eeeeeb', borderWidth: 1, borderColor: '#575c61'},
  pathOperationSection: {borderTopWidth: 1, borderBottomWidth: 1, borderColor: '#383c40', paddingVertical: 8},
  transformSection: {borderTopWidth: 1, borderBottomWidth: 1, borderColor: '#33454c', paddingVertical: 8},
  pathOperationTitle: {paddingHorizontal: 10, fontSize: 10, fontWeight: '700', color: '#e7e7e4'},
  pathOperationDescription: {paddingHorizontal: 10, paddingTop: 4, fontSize: 8, lineHeight: 12, color: '#aeb2b3'},
  pathOperationGrid: {flexDirection: 'row', flexWrap: 'wrap', gap: 5, padding: 10},
  pathOperationButton: {borderWidth: 1, borderColor: '#45494d', paddingHorizontal: 6, paddingVertical: 5},
  pathOperationButtonActive: {borderColor: '#b4a66a', backgroundColor: '#29291f'},
  pathOperationButtonText: {fontSize: 8, color: '#d7d8d4'},
  extensionBody: {flex: 1},
  extensionTabs: {flexDirection: 'row', padding: 7, gap: 5},
  extensionTab: {paddingHorizontal: 9, paddingVertical: 5, borderWidth: 1, borderColor: '#44484d'},
  timeline: {backgroundColor: '#17191b', borderTopWidth: 1, borderTopColor: '#3a3d41'},
  timelineHeader: {height: 31, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 9, borderBottomWidth: 1, borderBottomColor: '#3a3d41'},
  nativeTimelineBody: {flex: 1},
  nativeTimelineSurface: {flex: 1},
});

export default App;
