import React, {useEffect, useState} from 'react';
import {FlatList, Pressable, Text, TextInput, View} from 'react-native';

import {MacPressable, PanelHeader} from './chrome';
import {
  dispatchHostIntent,
  type HostCatalogEffect,
  type HostCatalogSource,
  type HostLibrary,
  type HostLibraryItem,
  type HostRationalTime,
} from './host';
import {styles} from './productStyles';

export type BrowserTab = 'MEDIA' | 'EFFECTS' | 'CREATE';
export type BrowserViewMode = 'THUMBNAILS' | 'GRID' | 'LIST';

export type EffectItem = {
  id: string;
  name: string;
  badge: string;
  tags: string;
  color: string;
  unavailable?: boolean;
};

export type BrowserItem = {
  id: string;
  name: string;
  detail: string;
  color: string;
  badge?: string;
  glyph?: string;
  unavailable?: boolean;
  testID?: string;
};

export const CREATE_ITEMS = [
  {id: 'rectangle', name: 'Rectangle', type: 'Shape', provider: 'Built-in', glyph: '□'},
  {id: 'ellipse', name: 'Ellipse', type: 'Shape', provider: 'Built-in', glyph: '○'},
];

export const MEDIA_COLORS = ['#5d7899', '#746398', '#88704e', '#557f6d', '#8b5962'];

export type MediaScope =
  | {kind: 'all'}
  | {kind: 'directory'; path: string}
  | {kind: 'type'; type: string}
  | {kind: 'tag'; id: string};

export type RailRow = {
  key: string;
  label: string;
  heading?: boolean;
  selected?: boolean;
  testID?: string;
  onPress?: () => void;
};

export function sameMediaScope(left: MediaScope, right: MediaScope): boolean {
  if (left.kind !== right.kind) {
    return false;
  }
  if (left.kind === 'directory' && right.kind === 'directory') {
    return left.path === right.path;
  }
  if (left.kind === 'type' && right.kind === 'type') {
    return left.type === right.type;
  }
  if (left.kind === 'tag' && right.kind === 'tag') {
    return left.id === right.id;
  }
  return left.kind === 'all';
}

export function toggleMediaScope(current: MediaScope, next: MediaScope): MediaScope {
  return sameMediaScope(current, next) ? {kind: 'all'} : next;
}

export function filterLibraryItems(
  items: HostLibraryItem[],
  query: string,
  scope: MediaScope,
): HostLibraryItem[] {
  const needle = query.toLowerCase();
  return items.filter(item => {
    if (needle && !`${item.name} ${item.kind} ${item.directory} ${item.tags.join(' ')}`
      .toLowerCase()
      .includes(needle)) {
      return false;
    }
    if (scope.kind === 'directory') {
      return item.directory === scope.path;
    }
    if (scope.kind === 'type') {
      return item.kind === scope.type;
    }
    if (scope.kind === 'tag') {
      return item.tags.includes(scope.id);
    }
    return true;
  });
}

export function mediaLibraryRail(
  library: HostLibrary | null,
  scope: MediaScope,
  onScope: (scope: MediaScope) => void,
): RailRow[] {
  const kinds = new Set((library?.items ?? []).map(item => item.kind));
  const typeRow = (type: string, label: string): RailRow | null => {
    if (!kinds.has(type)) {
      return null;
    }
    const next: MediaScope = {kind: 'type', type};
    return {
      key: `type-${type}`,
      label,
      selected: sameMediaScope(scope, next),
      testID: `media-rail-type-${type}`,
      onPress: () => onScope(toggleMediaScope(scope, next)),
    };
  };
  const all: MediaScope = {kind: 'all'};
  return [
    {
      key: 'all',
      label: '▦  All media',
      selected: sameMediaScope(scope, all),
      testID: 'media-rail-all',
      onPress: () => onScope(all),
    },
    {key: 'directories', label: 'DIRECTORIES', heading: true},
    ...(library?.directories ?? []).map(directory => {
      const next: MediaScope = {kind: 'directory', path: directory.path};
      return {
        key: `dir-${directory.id}`,
        label: `▣  ${directory.name}`,
        selected: sameMediaScope(scope, next),
        testID: `media-rail-dir-${directory.id}`,
        onPress: () => onScope(toggleMediaScope(scope, next)),
      };
    }),
    {key: 'type', label: 'TYPE', heading: true},
    ...(['video', 'audio', 'image'] as const)
      .map(type => typeRow(
        type,
        type === 'video' ? '▣  Video' : type === 'audio' ? '♪  Audio' : '▧  Image',
      ))
      .filter((row): row is RailRow => row !== null),
    {key: 'tags', label: 'TAGS', heading: true},
    ...(library?.tags ?? []).map(tag => {
      const next: MediaScope = {kind: 'tag', id: tag.id};
      return {
        key: `tag-${tag.id}`,
        label: `◎  ${tag.label}  ${tag.count}`,
        selected: sameMediaScope(scope, next),
        testID: `media-rail-tag-${tag.id}`,
        onPress: () => onScope(toggleMediaScope(scope, next)),
      };
    }),
  ];
}

export function BrowserResults({
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
  rail: RailRow[];
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
        {rail.map(row => row.heading ? (
          <Text key={row.key} numberOfLines={1} style={styles.railHeading}>
            {row.label}
          </Text>
        ) : row.onPress ? (
          <Pressable
            accessibilityRole="button"
            accessibilityState={{selected: !!row.selected}}
            key={row.key}
            onPress={row.onPress}
            style={row.selected ? styles.effectSelected : undefined}
            testID={row.testID}>
            <Text numberOfLines={1} style={styles.railItem}>{row.label}</Text>
          </Pressable>
        ) : (
          <Text key={row.key} numberOfLines={1} style={styles.railItem}>
            {row.label}
          </Text>
        ))}
      </View>
      <FlatList
        contentContainerStyle={styles.resultsContent}
        data={items}
        extraData={testID === 'MEDIA' ? `${selectedId ?? ''}:${items.map(item => item.id).join('|')}` : undefined}
        initialNumToRender={32}
        key={testID === 'MEDIA' ? `${testID}-${view}-${items.map(item => item.id).join('|')}` : `${testID}-${view}`}
        keyExtractor={item => item.id}
        ListHeaderComponent={(
          <View style={styles.resultsHeader}>
            <Text style={styles.resultTitle}>{title}</Text>
            <Text style={styles.panelDetail}>{items.length}</Text>
          </View>
        )}
        maxToRenderPerBatch={32}
        numColumns={columns}
        removeClippedSubviews={testID !== 'MEDIA'}
        renderItem={({item}) => (
          <MacPressable
            accessibilityLabel={`${item.name}${item.detail ? `, ${item.detail}` : ''}`}
            accessibilityRole="button"
            accessibilityState={{selected: selectedId === item.id}}
            onDoubleClick={() => onActivate(item.id)}
            onPointerDown={() => {
              if (testID === 'MEDIA') {
                onSelect(item.id);
              }
              onDragStart(item.id);
            }}
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
                selectedId === item.id && styles.iconButtonActive,
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

export function Browser({
  width,
  onDragStart,
  onDragCancel,
  catalogEffects,
  catalogSources,
  library,
  primaryLayerId,
  currentTime,
  attachedPluginIds,
}: {
  width: number;
  onDragStart: (id: string) => void;
  onDragCancel: () => void;
  catalogEffects: HostCatalogEffect[] | null;
  catalogSources: HostCatalogSource[] | null;
  library: HostLibrary | null;
  primaryLayerId: string | null;
  currentTime: HostRationalTime | null;
  attachedPluginIds: string[];
}) {
  const [tab, setTab] = useState<BrowserTab>('EFFECTS');
  const [query, setQuery] = useState('');
  const [selected, setSelected] = useState<string | null>(null);
  const [selectedCreateItem, setSelectedCreateItem] = useState<string | null>(null);
  const [selectedMediaItem, setSelectedMediaItem] = useState<string | null>(null);
  const [mediaScope, setMediaScope] = useState<MediaScope>({kind: 'all'});
  const [createScope, setCreateScope] = useState<'all' | 'shape' | 'builtin'>('all');
  const [effectScope, setEffectScope] = useState<'all' | 'used'>('all');
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
  const effectSource: EffectItem[] = (catalogEffects ?? []).map((item, index) => ({
    id: item.plugin_id,
    name: item.name,
    badge: 'FX',
    tags: item.plugin_id,
    color: MEDIA_COLORS[index % MEDIA_COLORS.length],
  }));
  const attached = new Set(attachedPluginIds);
  const filteredEffects = effectSource.filter(item => {
    if (effectScope === 'used' && !attached.has(item.id)) {
      return false;
    }
    return `${item.name} ${item.tags}`.toLowerCase().includes(query.toLowerCase());
  });
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
  const filteredCreateItems = createItems.filter(item => {
    if (createScope === 'shape' && item.type !== 'Shape') {
      return false;
    }
    if (createScope === 'builtin' && item.provider !== 'Built-in') {
      return false;
    }
    return `${item.name} ${item.type} ${item.provider}`.toLowerCase().includes(query.toLowerCase());
  });
  const filteredMediaItems = filterLibraryItems(library?.items ?? [], query, mediaScope);
  const browserItems: BrowserItem[] = tab === 'EFFECTS'
    ? filteredEffects.map(item => ({
      id: item.id,
      name: item.name,
      detail: item.tags,
      color: item.color,
      badge: item.badge,
      unavailable: item.unavailable,
      testID: catalogEffects ? `effect-item-${item.id}` : undefined,
    }))
    : tab === 'MEDIA'
      ? filteredMediaItems.map((item, index) => ({
        id: item.id,
        name: item.name,
        detail: [item.kind, item.directory, ...item.tags].filter(Boolean).join(' · '),
        color: MEDIA_COLORS[index % MEDIA_COLORS.length],
        testID: `media-item-${item.name}`,
      }))
      : filteredCreateItems.map(item => ({
        id: item.id,
        name: item.name,
        detail: `${item.type} · ${item.provider}`,
        color: '#2d2b25',
        glyph: item.glyph,
        testID: `create-item-${item.id}`,
      }));
  const rail = tab === 'MEDIA'
    ? mediaLibraryRail(library, mediaScope, setMediaScope)
    : tab === 'EFFECTS'
      ? [
        {
          key: 'fx-all',
          label: '▦  All',
          selected: effectScope === 'all',
          testID: 'effect-rail-all',
          onPress: () => setEffectScope('all'),
        },
        {
          key: 'fx-used',
          label: '◇  Used',
          selected: effectScope === 'used',
          testID: 'effect-rail-used',
          onPress: () => setEffectScope('used'),
        },
      ]
      : [
        {
          key: 'create-all',
          label: '▦  All',
          selected: createScope === 'all',
          testID: 'create-rail-all',
          onPress: () => setCreateScope('all'),
        },
        {key: 'create-type', label: 'TYPE', heading: true},
        {
          key: 'create-shape',
          label: '□  Shapes',
          selected: createScope === 'shape',
          testID: 'create-rail-shapes',
          onPress: () => setCreateScope(createScope === 'shape' ? 'all' : 'shape'),
        },
        {key: 'create-provider', label: 'PROVIDER', heading: true},
        {
          key: 'create-builtin',
          label: 'M  Built-in',
          selected: createScope === 'builtin',
          testID: 'create-rail-builtin',
          onPress: () => setCreateScope(createScope === 'builtin' ? 'all' : 'builtin'),
        },
      ];
  const title = tab === 'CREATE' ? 'Create items' : 'Results';
  const selectedId = tab === 'EFFECTS' ? selected : tab === 'CREATE' ? selectedCreateItem : selectedMediaItem;

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
              return;
            }
            if (id === 'ellipse') {
              dispatchHostIntent('place_ellipse', {
                position: [0, 0],
                playhead: currentTime ?? {num: 0, den: 1},
              });
            }
            return;
          }
          if (tab === 'MEDIA') {
            setSelectedMediaItem(id);
            onDragCancel();
            dispatchHostIntent('place_media', {
              item_id: id,
              position: [0, 0],
              playhead: currentTime ?? {num: 0, den: 1},
            });
            return;
          }
          if (tab === 'EFFECTS' && catalogEffects) {
            dispatchHostIntent('attach_effect', {
              target: primaryLayerId,
              plugin_id: id,
            });
          }
        }}
        onDragStart={id => {
          if (tab === 'MEDIA') {
            setSelectedMediaItem(id);
            onDragStart(id);
            return;
          }
          if (tab === 'CREATE') {
            setSelectedCreateItem(id);
            // Stage drop は Rectangle のみ。CREATE の source カードは drag 開始しない。
            if (catalogSources?.some(item => item.plugin_id === id)) {
              return;
            }
            onDragStart(id);
          }
        }}
        onSelect={id => tab === 'EFFECTS' ? setSelected(id) : tab === 'CREATE' ? setSelectedCreateItem(id) : setSelectedMediaItem(id)}
        rail={rail}
        selectedId={selectedId}
        testID={tab}
        title={title}
        view={view}
      />
    </View>
  );
}
