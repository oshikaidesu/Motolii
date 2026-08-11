import React, {useMemo, useRef, useState} from 'react';
import {
  FlatList,
  Pressable,
  ScrollView,
  StyleSheet,
  Text,
  TextInput,
  View,
} from 'react-native';

import {panelRegistry} from './src/panels/registry';
import MotoliiGpuView from './src/specs/MotoliiGpuViewNativeComponent';
import MotoliiTimelineView from './src/specs/MotoliiTimelineViewNativeComponent';
import packageJson from './package.json';

const MacPressable = Pressable as React.ComponentType<
  React.ComponentProps<typeof Pressable> & {onDoubleClick?: () => void}
>;

type BrowserTab = 'MEDIA' | 'EFFECTS' | 'CREATE';
type BrowserViewMode = 'THUMBNAILS' | 'GRID' | 'LIST';
type RightPanel = 'INSPECTOR' | 'EXTENSIONS';
type TimelineMode = 'PACKING' | 'DENSITY' | 'NATIVE';


const BUILD_LABEL = `UI ${packageJson.version} · RN 0.81.2 · RERUN 954bf95 · SKIA 0.99.0 · CHROMA`;

type EffectItem = {
  id: string;
  name: string;
  badge: string;
  tags: string;
  color: string;
  unavailable?: boolean;
};

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

type PackingObject = {
  id: string;
  kind: string;
  name: string;
  left: number;
  width: number;
  color: string;
  flow?: string;
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

const EFFECTS: EffectItem[] = [
  {id: 'echo', name: 'Echo Bloom', badge: 'FX', tags: '#go-to #atmosphere', color: '#746d4b'},
  {id: 'type', name: 'Type Pulse', badge: 'Aa', tags: '#kinetic', color: '#5c6477'},
  {id: 'fold', name: 'Fold Field', badge: 'FX', tags: '#review', color: '#51455f', unavailable: true},
];

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

const PACKING_OBJECTS: PackingObject[] = [
  {id: 'song', kind: '♪', name: 'night_drive.wav  ╱╲╱▁╲╱╲▁╱╲', left: 1, width: 98, color: '#75a7a5'},
  {id: 'pulse', kind: 'G', name: 'Pulse rings', left: 4, width: 88, color: '#7770a9', flow: 'IN → Echo Bloom → OUT'},
  {id: 'grid', kind: 'S', name: '↳ Pulse rings   City grid', left: 13, width: 75, color: '#62756d'},
  {id: 'title', kind: 'T', name: 'NIGHT DRIVE', left: 27, width: 38, color: '#a08b64'},
  {id: 'city', kind: 'V', name: 'city_loop.mp4', left: 16, width: 43, color: '#9b6d69'},
  {id: 'traffic', kind: 'V', name: 'traffic_pass.mp4', left: 64, width: 34, color: '#68819c'},
];

const DENSITY_CLIPS = Array.from({length: 500}, (_, index) => ({
  id: `clip-${index}`,
  width: 46 + (index % 6) * 10,
  color: ['#576fbd', '#8b5eaa', '#548d78', '#a36e43'][index % 4],
}));

const MEDIA_ITEMS = Array.from({length: 5000}, (_, index) => ({
  id: `asset-${index}`,
  color: ['#5d7899', '#746398', '#88704e', '#557f6d', '#8b5962'][index % 5],
}));

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

function Browser({width, onCreateItem, onDragStart, onDragCancel}: {width: number; onCreateItem: (id: string) => void; onDragStart: (id: string) => void; onDragCancel: () => void}) {
  const [tab, setTab] = useState<BrowserTab>('EFFECTS');
  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState('echo');
  const [selectedCreateItem, setSelectedCreateItem] = useState<string | null>(null);
  const [view, setView] = useState<BrowserViewMode>('GRID');
  const filteredEffects = EFFECTS.filter(item =>
    `${item.name} ${item.tags}`.toLowerCase().includes(query.toLowerCase()),
  );
  const filteredCreateItems = CREATE_ITEMS.filter(item =>
    `${item.name} ${item.type} ${item.provider}`.toLowerCase().includes(query.toLowerCase()),
  );
  const filteredMediaItems = MEDIA_ITEMS.filter(item =>
    item.id.toLowerCase().includes(query.toLowerCase()),
  );
  const browserItems: BrowserItem[] = tab === 'EFFECTS'
    ? filteredEffects.map(item => ({
      id: item.id,
      name: item.name,
      detail: item.tags,
      color: item.color,
      badge: item.badge,
      unavailable: item.unavailable,
    }))
    : tab === 'MEDIA'
      ? filteredMediaItems.map(item => ({
        id: item.id,
        name: item.id,
        detail: 'Media',
        color: item.color,
      }))
      : filteredCreateItems.map(item => ({
        id: item.id,
        name: item.name,
        detail: `${item.type} · ${item.provider}`,
        color: '#2d2b25',
        glyph: item.glyph,
        testID: `create-item-${item.id}`,
      }));
  const rail = tab === 'EFFECTS'
    ? ['▦  All', '◇  Used', '↺  Recent', 'COLLECTIONS', '◎  Favorites', 'Aa  Type', 'TAGS', '◎  Go-to  1', '◌  Atmosphere  1', '⌁  Kinetic  1', '✓  Review  1', 'PACKS', '▤  Motion Kit α']
    : tab === 'MEDIA'
      ? ['▦  All media', '◇  Used', '↺  Recent', 'TYPE', '▣  Video', '♪  Audio', '▧  Image']
      : ['▦  All', 'TYPE', '□  Shapes', 'PROVIDER', 'M  Built-in'];
  const title = tab === 'CREATE' ? 'Create items' : 'Results';
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
        onActivate={id => {
          if (tab === 'CREATE') {
            setSelectedCreateItem(id);
            onDragCancel();
            onCreateItem(createdItemValue(id, 0.5, 0.5));
          }
        }}
        onDragStart={id => {
          if (tab === 'CREATE') {
            setSelectedCreateItem(id);
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

function Stage({createdItemId, draggedItemId, pathOperationId, showGpu, onDrop, onToggleGpu, onTransform, transform}: {createdItemId: string; draggedItemId: string; pathOperationId: string; showGpu: boolean; onDrop: (x: number, y: number) => void; onToggleGpu: () => void; onTransform: (transform: StageTransform) => void; transform: StageTransform}) {
  return (
    <View style={styles.stage} testID="stage-surface">
      <View style={styles.stageTools}>
        <Text style={styles.stageToolText}>Fit</Text>
        <Text style={styles.stageToolText}>100%</Text>
        <Pressable onPress={onToggleGpu} style={styles.stageGpuButton}>
          <Text style={styles.stageToolText}>{showGpu ? 'GPU ON' : 'GPU OFF'}</Text>
        </Pressable>
        <Text style={styles.stageIdentity}>STAGE / ECHO BLOOM</Text>
      </View>
      <View style={styles.stageViewport} testID="stage-viewport">
        {showGpu ? (
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
            onStageDrop={event => onDrop(event.nativeEvent.x, event.nativeEvent.y)}
            onStageTransform={event => onTransform(event.nativeEvent)}
            style={styles.gpuStage}
            testID="rust-wgpu-stage"
          />
        ) : (
          <View style={styles.gpuStageOff}><Text style={styles.muted}>Native Stage unmounted</Text></View>
        )}
        <View pointerEvents="none" style={styles.frameGrid}>
          <Text style={styles.outputLabel}>OUTPUT FRAME</Text>
          <View style={styles.titleBounds}>
            <Text style={styles.stageTitle}>NIGHT{`\n`}DRIVE</Text>
            <Text style={styles.stageSubtitle}>54.2 / CITY SIGNAL</Text>
          </View>
          <View style={styles.rings}>
            <View style={[styles.ring, styles.ringLarge]} />
            <View style={[styles.ring, styles.ringMedium]} />
            <View style={[styles.ring, styles.ringSmall]} />
            <View style={styles.ringCore} />
          </View>
        </View>
      </View>
      <View style={styles.transport}>
        <Text style={styles.transportButton}>|‹</Text>
        <Text style={styles.transportButton}>▶</Text>
        <Text style={styles.transportButton}>›|</Text>
        <Text style={styles.timecode}>00:54.2  BAR 54.2.00  120 BPM · SNAP BEAT</Text>
        <Text style={styles.quality}>DRAFT · FP16 · 1/2</Text>
      </View>
    </View>
  );
}

function Inspector({width, pathOperationId, onPathOperationChange, transform, onTransformChange}: {width: number; pathOperationId: string; onPathOperationChange: (id: string) => void; transform: StageTransform; onTransformChange: (update: Partial<StageTransform>) => void}) {
  const [panel, setPanel] = useState<RightPanel>('INSPECTOR');
  const [intensity, setIntensity] = useState(64);
  const [spread, setSpread] = useState(42);
  const [extensionId, setExtensionId] = useState<string>(panelRegistry[0].id);
  const extension = panelRegistry.find(item => item.id === extensionId)!;
  const selectedPathOperation = PATH_OPERATIONS.find(item => item.id === pathOperationId)!;

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
        <ScrollView>
          <View style={styles.effectIdentity}>
            <View style={styles.effectIcon}><Text style={styles.effectIconText}>◎</Text></View>
            <View><Text style={styles.inspectorTitle}>Echo Bloom</Text><Text style={styles.muted}>Pulse rings · Effect</Text></View>
          </View>
          <Text style={styles.inspectorDescription}>Layered light pulses that follow the selected object. Adjust while watching the native Stage.</Text>
          <ParameterRow label="Input" value="Pulse rings composite" />
          <ParameterRow label="Intensity" value={`${intensity}%`} onDecrease={() => setIntensity(v => clamp(v - 2, 0, 100))} onIncrease={() => setIntensity(v => clamp(v + 2, 0, 100))} />
          <ParameterRow label="Spread" value={`${spread}%`} onDecrease={() => setSpread(v => clamp(v - 2, 0, 100))} onIncrease={() => setSpread(v => clamp(v + 2, 0, 100))} />
          <ParameterRow label="Blend" value="Screen" />
          <View style={styles.transformSection} testID="stage-transform-projection">
            <Text style={styles.pathOperationTitle}>Transform</Text>
            <Text style={styles.pathOperationDescription}>Shared live value with the Stage gizmo. This fixture is not saved to Document yet.</Text>
            <ParameterRow label="Position X" value={transform.x.toFixed(3)} onDecrease={() => onTransformChange({x: transform.x - 0.025})} onIncrease={() => onTransformChange({x: transform.x + 0.025})} />
            <ParameterRow label="Position Y" value={transform.y.toFixed(3)} onDecrease={() => onTransformChange({y: transform.y - 0.025})} onIncrease={() => onTransformChange({y: transform.y + 0.025})} />
            <ParameterRow label="Position Z" value={transform.z.toFixed(3)} onDecrease={() => onTransformChange({z: transform.z - 0.025})} onIncrease={() => onTransformChange({z: transform.z + 0.025})} />
            <ParameterRow label="Rotation X" value={`${transform.rotationX.toFixed(1)}°`} onDecrease={() => onTransformChange({rotationX: transform.rotationX - 5})} onIncrease={() => onTransformChange({rotationX: transform.rotationX + 5})} />
            <ParameterRow label="Rotation Y" value={`${transform.rotationY.toFixed(1)}°`} onDecrease={() => onTransformChange({rotationY: transform.rotationY - 5})} onIncrease={() => onTransformChange({rotationY: transform.rotationY + 5})} />
            <ParameterRow label="Rotation Z" value={`${transform.rotationZ.toFixed(1)}°`} onDecrease={() => onTransformChange({rotationZ: transform.rotationZ - 5})} onIncrease={() => onTransformChange({rotationZ: transform.rotationZ + 5})} />
          </View>
          <View style={styles.pathOperationSection} testID="path-operations-panel">
            <Text style={styles.pathOperationTitle}>Lottie Path Operations</Text>
            <Text style={styles.pathOperationDescription}>Choose an operation to preview its evaluated vector path on the Stage. This fixture is not saved to Document yet.</Text>
            <View style={styles.pathOperationGrid}>
              {PATH_OPERATIONS.map(item => (
                <Pressable
                  key={item.id}
                  accessibilityRole="button"
                  accessibilityState={{selected: item.id === pathOperationId}}
                  onPress={() => onPathOperationChange(item.id)}
                  style={[styles.pathOperationButton, item.id === pathOperationId && styles.pathOperationButtonActive]}
                  testID={`path-operation-${item.id}`}>
                  <Text style={styles.pathOperationButtonText}>{item.name}</Text>
                </Pressable>
              ))}
            </View>
            <ParameterRow label="Selected" value={selectedPathOperation.name} />
            <Text style={styles.pathOperationDescription}>{selectedPathOperation.detail}</Text>
          </View>
          <TextInput
            accessibilityLabel="Inspector note"
            defaultValue="日本語IME / focus probe"
            style={styles.inspectorInput}
          />
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

function ParameterRow({label, value, onDecrease, onIncrease}: {label: string; value: string; onDecrease?: () => void; onIncrease?: () => void}) {
  return (
    <View style={styles.parameterRow}>
      <Text style={styles.parameterLabel}>{label}</Text>
      {onDecrease ? <Pressable accessibilityLabel={`${label} decrease`} onPress={onDecrease} style={styles.stepButton}><Text style={styles.stepText}>−</Text></Pressable> : null}
      <Text numberOfLines={1} style={styles.parameterValue}>{value}</Text>
      {onIncrease ? <Pressable accessibilityLabel={`${label} increase`} onPress={onIncrease} style={styles.stepButton}><Text style={styles.stepText}>＋</Text></Pressable> : null}
    </View>
  );
}

function PackingTimeline() {
  const [selected, setSelected] = useState('pulse');
  const [muted, setMuted] = useState<Set<string>>(() => new Set());
  return (
    <View style={styles.packingBody} testID="packing-timeline">
      <View style={styles.keyTools}>
        <View style={styles.keyTabs}><Text style={styles.keyTabActive}>KEYS</Text><Text style={styles.keyTab}>LAYERS</Text></View>
        <Text style={styles.keyToolIcon}>◆ 0</Text>
        <View style={styles.keyToolRow}><Text style={styles.keyToolButton}>│◆</Text><Text style={styles.keyToolButton}>◆┆◆</Text><Text style={styles.keyToolButton}>◆│</Text></View>
      </View>
      <View style={styles.timelineCanvas}>
        <View style={styles.ruler}><Text style={styles.rulerText}>TIME / BEAT       52              52.2              53              53.2              54</Text></View>
        {PACKING_OBJECTS.map((object, index) => {
          const isMuted = muted.has(object.id);
          return (
            <View key={object.id} style={styles.packingRow}>
              <View style={styles.rowControls}>
                <Text style={styles.kind}>{object.kind}</Text>
                <Text style={styles.sm}>S</Text>
                <Pressable
                  accessibilityLabel={`${object.name}をMute`}
                  onPress={() => setMuted(current => {
                    const next = new Set(current);
                    next.has(object.id) ? next.delete(object.id) : next.add(object.id);
                    return next;
                  })}
                  style={[styles.smButton, isMuted && styles.smActive]}>
                  <Text style={styles.sm}>M</Text>
                </Pressable>
              </View>
              <Pressable
                accessibilityRole="button"
                accessibilityState={{selected: selected === object.id}}
                onPress={() => setSelected(object.id)}
                style={[
                  styles.objectBar,
                  {left: `${object.left}%`, width: `${object.width}%`, backgroundColor: object.color},
                  selected === object.id && styles.objectSelected,
                  isMuted && styles.objectMuted,
                  index === 2 && styles.childObject,
                ]}>
                <Text numberOfLines={1} style={styles.objectText}>{object.name}</Text>
                {object.flow ? <Text numberOfLines={1} style={styles.objectFlow}>{object.flow}</Text> : null}
              </Pressable>
            </View>
          );
        })}
      </View>
    </View>
  );
}

function DensityTimeline() {
  const tracks = useMemo(() => Array.from({length: 20}, (_, track) => ({id: `track-${track}`, clips: DENSITY_CLIPS.slice(track * 25, track * 25 + 25)})), []);
  return (
    <FlatList
      data={tracks}
      initialNumToRender={8}
      testID="timeline-density-grid"
      windowSize={5}
      keyExtractor={item => item.id}
      renderItem={({item, index}) => (
        <View style={styles.densityTrack}>
          <Text style={styles.densityTrackLabel}>V{index + 1}</Text>
          <FlatList
            horizontal
            data={item.clips}
            initialNumToRender={10}
            windowSize={3}
            keyExtractor={clip => clip.id}
            renderItem={({item: clip}) => (
              <View style={[styles.densityClip, {width: clip.width, backgroundColor: clip.color}]}>
                <Text style={styles.densityClipText}>{clip.id.replace('clip-', 'C')}</Text>
              </View>
            )}
          />
        </View>
      )}
    />
  );
}

function NativeTimeline() {
  const [selectedObjectIndex, setSelectedObjectIndex] = useState(1);
  const [playhead, setPlayhead] = useState(0.54);

  return (
    <View style={styles.nativeTimelineBody}>
      <View style={styles.nativeTimelineControls}>
        <Text style={styles.nativeTimelineTitle}>Rust / wgpu · 500 clips · 20 tracks</Text>
        <Pressable
          accessibilityLabel="Select previous native clip"
          onPress={() => setSelectedObjectIndex(value => Math.max(0, value - 1))}
          style={styles.nativeTimelineStep}>
          <Text style={styles.modeText}>CLIP −</Text>
        </Pressable>
        <Pressable
          accessibilityLabel="Select next native clip"
          onPress={() => setSelectedObjectIndex(value => Math.min(499, value + 1))}
          style={styles.nativeTimelineStep}>
          <Text style={styles.modeText}>CLIP ＋</Text>
        </Pressable>
        <Text style={styles.nativeTimelineFeedback} testID="native-timeline-feedback">
          clip {selectedObjectIndex} · {(playhead * 100).toFixed(1)}%
        </Text>
      </View>
      <MotoliiTimelineView
        accessible
        accessibilityLabel="Rust wgpu Timeline time surface"
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
  const [mode, setMode] = useState<TimelineMode>('PACKING');
  return (
    <View style={[styles.timeline, {height}]} testID="timeline">
      <View style={styles.timelineHeader}>
        <Text style={styles.panelTitle}>譜面 / Timeline</Text>
        <View style={styles.timelineModes}>
          {(['PACKING', 'DENSITY', 'NATIVE'] as TimelineMode[]).map(value => (
            <Pressable key={value} onPress={() => setMode(value)} style={[styles.modeButton, mode === value && styles.iconButtonActive]} testID={`timeline-mode-${value}`}>
              <Text style={styles.modeText}>{value === 'PACKING' ? 'PACKING' : value === 'DENSITY' ? 'RN 500' : 'NATIVE 500'}</Text>
            </Pressable>
          ))}
        </View>
        <Text style={styles.panelDetail}>{mode === 'PACKING' ? 'single time surface' : mode === 'DENSITY' ? '20 RN virtualized tracks' : 'one native time surface'}</Text>
      </View>
      {mode === 'PACKING' ? <PackingTimeline /> : mode === 'DENSITY' ? <DensityTimeline /> : <NativeTimeline />}
    </View>
  );
}

function App() {
  const [browserWidth, setBrowserWidth] = useState(284);
  const [inspectorWidth, setInspectorWidth] = useState(326);
  const [timelineHeight, setTimelineHeight] = useState(270);
  const [showGpuStage, setShowGpuStage] = useState(true);
  const [createdItemId, setCreatedItemId] = useState('');
  const [draggedItemId, setDraggedItemId] = useState('');
  const [pathOperationId, setPathOperationId] = useState(PATH_OPERATIONS[0].id);
  const [stageTransform, setStageTransform] = useState<StageTransform>(INITIAL_STAGE_TRANSFORM);
  const browserStart = useRef(browserWidth);
  const inspectorStart = useRef(inspectorWidth);
  const timelineStart = useRef(timelineHeight);
  const completeStageDrop = (x: number, y: number) => {
    const itemId = draggedItemId;
    setDraggedItemId('');
    if (itemId && x >= 0 && y >= 0) {
      setCreatedItemId(createdItemValue(itemId, x, y));
    }
  };

  return (
    <View style={styles.shell} testID="motolii-rn-shell">
      <View style={styles.titlebar}>
        <Text style={styles.brand}>MOTOLII</Text>
        <Text style={styles.project}>night_drive.mtl / Main composition</Text>
        <Text style={styles.buildLabel} testID="ui-version">{BUILD_LABEL}</Text>
        <View style={styles.grow} />
        {['Settings', '↶ Undo', '↷ Redo', 'Export'].map(label => <Text key={label} style={styles.titleAction}>{label}</Text>)}
      </View>
      <View style={styles.commandbar}>
        {['↖', '✥', '◇', 'T', '⌁', 'Δ', '▣'].map((tool, index) => <Text key={`${tool}-${index}`} style={[styles.commandTool, index === 0 && styles.commandToolActive]}>{tool}</Text>)}
        <Text style={styles.colorBook}>COLOR BOOK</Text>
        <Text style={styles.breadcrumb}>Pulse rings / <Text style={styles.breadcrumbStrong}>Echo Bloom</Text></Text>
      </View>
      <View style={styles.workspace}>
        <Browser
          onCreateItem={setCreatedItemId}
          onDragCancel={() => setDraggedItemId('')}
          onDragStart={setDraggedItemId}
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
          <Stage createdItemId={createdItemId} draggedItemId={draggedItemId} pathOperationId={pathOperationId} showGpu={showGpuStage} onDrop={completeStageDrop} onToggleGpu={() => setShowGpuStage(value => !value)} onTransform={setStageTransform} transform={stageTransform} />
        </View>
        <Splitter
          label="Inspectorのサイズを変更"
          orientation="vertical"
          onStart={() => { inspectorStart.current = inspectorWidth; }}
          onDelta={delta => setInspectorWidth(clamp(inspectorStart.current - delta, 240, 440))}
          onNudge={() => setInspectorWidth(value => value >= 390 ? 326 : value + 64)}
        />
        <Inspector width={inspectorWidth} pathOperationId={pathOperationId} onPathOperationChange={setPathOperationId} transform={stageTransform} onTransformChange={update => setStageTransform(current => ({...current, ...update}))} />
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
  project: {marginLeft: 14, fontSize: 11, color: '#d5d6d3'},
  buildLabel: {marginLeft: 14, fontSize: 8, color: '#858a8d'},
  grow: {flex: 1},
  titleAction: {fontSize: 10, color: '#d8d8d5', paddingVertical: 6, paddingHorizontal: 10, marginLeft: 6, borderWidth: 1, borderColor: '#414448'},
  commandbar: {height: 32, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 8, borderBottomWidth: 1, borderBottomColor: '#36393d', backgroundColor: '#151719'},
  commandTool: {width: 32, textAlign: 'center', fontSize: 12, color: '#b7b9bb'},
  commandToolActive: {borderWidth: 1, borderColor: '#d7d8d5', paddingVertical: 4, color: '#ffffff'},
  colorBook: {marginLeft: 8, paddingLeft: 14, borderLeftWidth: 1, borderLeftColor: '#414448', fontSize: 11, color: '#d1d2d0'},
  breadcrumb: {marginLeft: 14, fontSize: 10, color: '#999d9f'},
  breadcrumbStrong: {fontWeight: '700', color: '#e7e7e4'},
  workspace: {flex: 1, flexDirection: 'row', minHeight: 260},
  centerColumn: {flex: 1, minWidth: 420},
  browser: {backgroundColor: '#202326'},
  inspector: {backgroundColor: '#1b1d20'},
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
  stageTools: {height: 31, flexDirection: 'row', alignItems: 'center', gap: 16, paddingHorizontal: 9, borderBottomWidth: 1, borderBottomColor: '#393c40'},
  stageToolText: {fontSize: 9, color: '#d4d5d3'},
  stageGpuButton: {paddingHorizontal: 7, paddingVertical: 4, borderWidth: 1, borderColor: '#716a4c'},
  stageIdentity: {marginLeft: 'auto', fontSize: 7, color: '#b7ac73'},
  stageViewport: {flex: 1, overflow: 'hidden'},
  gpuStage: {position: 'absolute', inset: 0},
  gpuStageOff: {position: 'absolute', inset: 0, alignItems: 'center', justifyContent: 'center'},
  frameGrid: {position: 'absolute', inset: 0, borderWidth: 1, borderColor: 'rgba(255,255,255,0.04)'},
  outputLabel: {position: 'absolute', top: 8, left: 10, fontSize: 7, letterSpacing: 1, color: '#bfc3c5'},
  titleBounds: {position: 'absolute', left: 10, top: 22, width: 126, height: 80, padding: 6, borderWidth: 1, borderColor: '#c3b88b'},
  stageTitle: {fontSize: 16, lineHeight: 15, fontWeight: '800', letterSpacing: 3, color: '#f1f1ee'},
  stageSubtitle: {marginTop: 6, fontSize: 5.5, letterSpacing: 0.8, color: '#c6c8c7'},
  rings: {position: 'absolute', left: 144, top: 22, width: 80, height: 80, alignItems: 'center', justifyContent: 'center'},
  ring: {position: 'absolute', borderWidth: 1, borderColor: 'rgba(235,238,232,0.42)', borderRadius: 200},
  ringLarge: {width: 76, height: 76},
  ringMedium: {width: 52, height: 52},
  ringSmall: {width: 30, height: 30},
  ringCore: {width: 14, height: 14, borderRadius: 7, backgroundColor: '#ebebe7'},
  transport: {height: 31, flexDirection: 'row', alignItems: 'center', paddingHorizontal: 10, borderTopWidth: 1, borderTopColor: '#3a3d41', backgroundColor: '#1b1d20'},
  transportButton: {width: 34, fontSize: 10, color: '#dbdcda'},
  timecode: {fontSize: 9, fontWeight: '600', color: '#e4e4e1'},
  quality: {marginLeft: 'auto', fontSize: 7, color: '#84898c'},
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
  timelineModes: {flexDirection: 'row', marginLeft: 18, gap: 4},
  modeButton: {paddingHorizontal: 8, paddingVertical: 4, borderWidth: 1, borderColor: '#414549'},
  modeText: {fontSize: 7, color: '#c6c8c7'},
  packingBody: {flex: 1, flexDirection: 'row'},
  keyTools: {width: 200, padding: 5, borderRightWidth: 1, borderRightColor: '#55595d'},
  keyTabs: {height: 25, flexDirection: 'row'},
  keyTabActive: {flex: 1, paddingTop: 7, textAlign: 'center', fontSize: 7, color: '#d5c77d', borderWidth: 1, borderColor: '#a69b63'},
  keyTab: {flex: 1, paddingTop: 7, textAlign: 'center', fontSize: 7, color: '#a3a7a8', borderWidth: 1, borderColor: '#45494d'},
  keyToolIcon: {marginTop: 9, padding: 8, fontSize: 9, color: '#e2d58a', borderWidth: 1, borderColor: '#7a724f'},
  keyToolRow: {flexDirection: 'row', gap: 3, marginTop: 8},
  keyToolButton: {flex: 1, padding: 7, textAlign: 'center', fontSize: 9, color: '#c8c9c7', borderWidth: 1, borderColor: '#45494d'},
  timelineCanvas: {flex: 1, overflow: 'hidden'},
  ruler: {height: 25, justifyContent: 'center', paddingHorizontal: 8, borderBottomWidth: 1, borderBottomColor: '#45494d'},
  rulerText: {fontSize: 7, color: '#a9a26f'},
  packingRow: {height: 34, borderBottomWidth: 1, borderBottomColor: '#3b3e41'},
  rowControls: {position: 'absolute', left: 0, width: 56, height: 34, flexDirection: 'row', alignItems: 'center', gap: 3, paddingLeft: 4, backgroundColor: '#202326', zIndex: 2},
  kind: {width: 15, fontSize: 8, color: '#d6d7d4'},
  smButton: {padding: 3},
  smActive: {backgroundColor: '#85524c'},
  sm: {fontSize: 7, color: '#e4e5e2'},
  objectBar: {position: 'absolute', top: 4, height: 26, minWidth: 70, flexDirection: 'row', alignItems: 'center', gap: 10, paddingHorizontal: 8, borderWidth: 1, borderColor: 'rgba(255,255,255,0.34)'},
  childObject: {marginLeft: 18},
  objectSelected: {borderWidth: 2, borderColor: '#eee7bf'},
  objectMuted: {opacity: 0.38},
  objectText: {fontSize: 8, fontWeight: '700', color: '#17191b'},
  objectFlow: {fontSize: 7, color: '#1f2020'},
  densityTrack: {height: 36, flexDirection: 'row', alignItems: 'center', borderBottomWidth: 1, borderBottomColor: '#303337'},
  densityTrackLabel: {width: 38, paddingLeft: 8, fontSize: 8, color: '#8e9396'},
  densityClip: {height: 27, justifyContent: 'center', marginRight: 2, paddingHorizontal: 5},
  densityClipText: {fontSize: 7, color: '#ffffff'},
  nativeTimelineBody: {flex: 1},
  nativeTimelineControls: {height: 28, flexDirection: 'row', alignItems: 'center', gap: 5, paddingHorizontal: 8, borderBottomWidth: 1, borderBottomColor: '#3b3e41'},
  nativeTimelineTitle: {fontSize: 8, color: '#b9bcbd'},
  nativeTimelineStep: {paddingHorizontal: 7, paddingVertical: 4, borderWidth: 1, borderColor: '#555a5e'},
  nativeTimelineFeedback: {marginLeft: 'auto', fontSize: 8, color: '#d8c97f'},
  nativeTimelineSurface: {flex: 1},
});

export default App;
