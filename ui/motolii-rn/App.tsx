import React, {useEffect, useRef, useState} from 'react';
import {Pressable, Text, View} from 'react-native';

import {Browser} from './src/Browser';
import {
  ChromeModal,
  ExportScreen,
  SettingsScreen,
  ShellView,
  Splitter,
  type ChromeModalKind,
  type ExportPhase,
} from './src/chrome';
import {
  clamp,
  dispatchHostIntent,
  dispatchHostIntentResult,
  dispatchHostKeyEventFromShellKey,
  INITIAL_STAGE_TRANSFORM,
  lastInclusiveFrame,
  nativeHostIsTimelineInteracting,
  readHostSnapshotState,
  setHostRejectApplier,
  setHostSnapshotApplier,
  type HostCatalogEffect,
  type HostCatalogSource,
  type HostLayerSeat,
  type HostLibrary,
  type HostRationalTime,
  type HostSnapshotState,
} from './src/host';
import {Inspector} from './src/Inspector';
import {styles} from './src/productStyles';
import {Stage} from './src/Stage';
import {Timeline} from './src/Timeline';

const BUILD_LABEL = 'B002 · RN 0.81.2 · RERUN 954bf95 · SKIA 0.99.0 · CHROMA';

function App() {
  const [browserWidth, setBrowserWidth] = useState(284);
  const [inspectorWidth, setInspectorWidth] = useState(326);
  const [timelineHeight, setTimelineHeight] = useState(270);
  const [showGpuStage, setShowGpuStage] = useState(true);
  const [draggedItemId, setDraggedItemId] = useState('');
  const [hostStatusLabel, setHostStatusLabel] = useState<string | null>(null);
  const [hostLayerSeat, setHostLayerSeat] = useState<HostLayerSeat | null>(null);
  const [hostCatalogEffects, setHostCatalogEffects] = useState<HostCatalogEffect[] | null>(null);
  const [hostCatalogSources, setHostCatalogSources] = useState<HostCatalogSource[] | null>(null);
  const [hostLibrary, setHostLibrary] = useState<HostLibrary | null>(null);
  const [canUndo, setCanUndo] = useState(false);
  const [canRedo, setCanRedo] = useState(false);
  const [scrubFreeze, setScrubFreeze] = useState(false);
  const [playing, setPlaying] = useState(false);
  const [rejectLabel, setRejectLabel] = useState<string | null>(null);
  const [chromeModal, setChromeModal] = useState<ChromeModalKind | null>(null);
  const [exportOutputPath, setExportOutputPath] = useState('');
  const [exportPhase, setExportPhase] = useState<ExportPhase>('idle');
  const [exportStatus, setExportStatus] = useState('Idle');
  const stageTransform = hostLayerSeat?.transform ?? INITIAL_STAGE_TRANSFORM;
  const lastSnapshotMeta = useRef<{revision: string | null; currentTime: HostRationalTime | null}>({
    revision: null,
    currentTime: null,
  });
  const transportClock = useRef<{fps: HostRationalTime | null; duration: HostRationalTime | null}>({
    fps: null,
    duration: null,
  });
  const browserStart = useRef(browserWidth);
  const inspectorStart = useRef(inspectorWidth);
  const timelineStart = useRef(timelineHeight);
  useEffect(() => {
    const apply = (snapshotState: HostSnapshotState) => {
      // F5: 凍結はnativeのgesture実信号で判定する(snapshot差分heuristicは廃止)。
      // 新しいデータがUIを変え得るのはapplyの瞬間だけなので、その瞬間の実信号が正。
      setScrubFreeze(nativeHostIsTimelineInteracting());
      setHostLayerSeat(snapshotState.layerSeat);
      setHostStatusLabel(snapshotState.statusLabel);
      setHostCatalogEffects(snapshotState.catalogEffects);
      setHostCatalogSources(snapshotState.catalogSources);
      setHostLibrary(snapshotState.library);
      setCanUndo(snapshotState.canUndo);
      setCanRedo(snapshotState.canRedo);
      setPlaying(snapshotState.playing);
      transportClock.current = {fps: snapshotState.fps, duration: snapshotState.duration};
      lastSnapshotMeta.current = {
        revision: snapshotState.revision,
        currentTime: snapshotState.currentTime,
      };
    };
    setHostSnapshotApplier(apply);
    setHostRejectApplier(setRejectLabel);
    const tick = () => {
      apply(readHostSnapshotState());
    };
    tick();
    // 1s pollは補助。dispatch応答snapshotの即時反映が主経路。
    const id = setInterval(tick, 1000);
    return () => {
      clearInterval(id);
      setHostSnapshotApplier(null);
      setHostRejectApplier(null);
    };
  }, []);
  const completeStageDrop = (x: number, y: number, canonicalX: number, canonicalY: number) => {
    const itemId = draggedItemId;
    setDraggedItemId('');
    if (!itemId || x < 0 || y < 0) {
      return;
    }
    const playhead = hostLayerSeat?.currentTime ?? {num: 0, den: 1};
    if (itemId === 'rectangle') {
      dispatchHostIntent('place_rectangle', {
        position: [canonicalX, canonicalY],
        playhead,
      });
      return;
    }
    if (itemId === 'ellipse') {
      dispatchHostIntent('place_ellipse', {
        position: [canonicalX, canonicalY],
        playhead,
      });
      return;
    }
    if (hostLibrary?.items.some(item => item.id === itemId)) {
      dispatchHostIntent('place_media', {
        item_id: itemId,
        position: [canonicalX, canonicalY],
        playhead,
      });
    }
  };

  return (
    <ShellView
      onKeyDown={event => {
        dispatchHostKeyEventFromShellKey(event);
      }}
      style={styles.shell}
      testID="motolii-rn-shell">
      <View style={styles.titlebar}>
        <Text style={styles.brand}>MOTOLII</Text>
        <Text style={styles.project}>composition</Text>
        <Text style={styles.buildLabel}>{BUILD_LABEL}</Text>
        {hostStatusLabel ? <Text style={styles.buildLabel}>{hostStatusLabel}</Text> : null}
        {rejectLabel ? <Text style={styles.buildLabel}>{rejectLabel}</Text> : null}
        <View style={styles.grow} />
        {['Settings', '↶ Undo', '↷ Redo', 'Export'].map(label => {
          if (label === 'Settings' || label === 'Export') {
            const kind: ChromeModalKind = label === 'Settings' ? 'settings' : 'export';
            return (
              <Pressable
                key={label}
                accessibilityLabel={label}
                onPress={() => {
                  setChromeModal(kind);
                  if (kind === 'export') {
                    setExportPhase('idle');
                    setExportStatus('Idle');
                  }
                }}
                style={styles.titleAction}
                testID={kind === 'settings' ? 'titlebar-settings' : 'titlebar-export'}>
                <Text style={styles.titleActionText}>{label}</Text>
              </Pressable>
            );
          }
          if (label === '↶ Undo' || label === '↷ Redo') {
            const kind = label === '↶ Undo' ? 'undo' : 'redo';
            const enabled = kind === 'undo' ? canUndo : canRedo;
            return (
              <Pressable
                key={label}
                accessibilityLabel={label}
                disabled={!enabled}
                onPress={() => {
                  if (!enabled) {
                    return;
                  }
                  dispatchHostIntent(kind);
                }}
                style={[styles.titleAction, !enabled && styles.titleActionDisabled]}
                testID={kind === 'undo' ? 'titlebar-undo' : 'titlebar-redo'}>
                <Text style={styles.titleActionText}>{label}</Text>
              </Pressable>
            );
          }
          return (
            <Text key={label} style={styles.titleAction}>
              {label}
            </Text>
          );
        })}
      </View>
      <View style={styles.commandbar}>
        <Pressable
          accessibilityLabel="Select"
          accessibilityRole="button"
          style={[styles.commandToolButton, styles.commandToolActive]}
          testID="command-tool-select">
          <Text style={styles.commandTool}>↖</Text>
        </Pressable>
        <Pressable
          accessibilityLabel="Place rectangle"
          accessibilityRole="button"
          onPress={() => {
            dispatchHostIntent('place_rectangle', {
              position: [0, 0],
              playhead: hostLayerSeat?.currentTime ?? {num: 0, den: 1},
            });
          }}
          style={styles.commandToolButton}
          testID="command-tool-place-rectangle">
          <Text style={styles.commandTool}>◇</Text>
        </Pressable>
        <Pressable
          accessibilityLabel="Place ellipse"
          accessibilityRole="button"
          onPress={() => {
            dispatchHostIntent('place_ellipse', {
              position: [0, 0],
              playhead: hostLayerSeat?.currentTime ?? {num: 0, den: 1},
            });
          }}
          style={styles.commandToolButton}
          testID="command-tool-place-ellipse">
          <Text style={styles.commandTool}>○</Text>
        </Pressable>
        <Text
          style={[
            styles.breadcrumb,
            hostLayerSeat?.primaryLayerId ? styles.breadcrumbStrong : null,
          ]}
          testID="command-breadcrumb">
          {hostLayerSeat?.primaryLayerId ? hostLayerSeat.displayName : '未選択'}
        </Text>
      </View>
      <View style={styles.workspace}>
        <Browser
          catalogEffects={hostCatalogEffects}
          catalogSources={hostCatalogSources}
          library={hostLibrary}
          onDragCancel={() => setDraggedItemId('')}
          onDragStart={setDraggedItemId}
          primaryLayerId={hostLayerSeat?.primaryLayerId ?? null}
          currentTime={hostLayerSeat?.currentTime ?? null}
          attachedPluginIds={(hostLayerSeat?.effects ?? []).map(effect => effect.plugin_id)}
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
          <Stage
            draggedItemId={draggedItemId}
            showGpu={showGpuStage}
            onDrop={completeStageDrop}
            onToggleGpu={() => setShowGpuStage(value => !value)}
            transform={stageTransform}
            currentTime={hostLayerSeat?.currentTime ?? lastSnapshotMeta.current.currentTime}
            playing={playing}
            primaryLayerId={hostLayerSeat?.primaryLayerId ?? null}
            onTogglePlayback={() => { dispatchHostIntent('toggle_playback'); }}
            onSkipStart={() => { dispatchHostIntent('set_time', {frame: 0}); }}
            onSkipEnd={() => {
              const clock = transportClock.current;
              if (!clock.fps || !clock.duration) {
                return;
              }
              const frame = lastInclusiveFrame(clock.fps, clock.duration);
              if (frame == null) {
                return;
              }
              dispatchHostIntent('set_time', {frame});
            }}
            onMarkIn={() => {
              const target = hostLayerSeat?.primaryLayerId;
              const time = hostLayerSeat?.currentTime ?? lastSnapshotMeta.current.currentTime;
              if (!target || !time) {
                return;
              }
              dispatchHostIntent('trim_clip_in', {target, time});
            }}
            onMarkOut={() => {
              const target = hostLayerSeat?.primaryLayerId;
              const time = hostLayerSeat?.currentTime ?? lastSnapshotMeta.current.currentTime;
              if (!target || !time) {
                return;
              }
              dispatchHostIntent('trim_clip_out', {target, time});
            }}
          />
        </View>
        <Splitter
          label="Inspectorのサイズを変更"
          orientation="vertical"
          onStart={() => { inspectorStart.current = inspectorWidth; }}
          onDelta={delta => setInspectorWidth(clamp(inspectorStart.current - delta, 240, 440))}
          onNudge={() => setInspectorWidth(value => value >= 390 ? 326 : value + 64)}
        />
        <Inspector
          width={inspectorWidth}
          transform={stageTransform}
          layerSeat={hostLayerSeat}
          scrubFreeze={scrubFreeze}
          revision={lastSnapshotMeta.current.revision}
        />
      </View>
      <Splitter
        label="Timelineのサイズを変更"
        orientation="horizontal"
        onStart={() => { timelineStart.current = timelineHeight; }}
        onDelta={delta => setTimelineHeight(clamp(timelineStart.current - delta, 190, 460))}
        onNudge={() => setTimelineHeight(value => value >= 334 ? 270 : value + 64)}
      />
      <Timeline
        currentTime={hostLayerSeat?.currentTime ?? lastSnapshotMeta.current.currentTime}
        duration={transportClock.current.duration}
        height={timelineHeight}
        primaryLayerId={hostLayerSeat?.primaryLayerId ?? null}
      />
      {chromeModal ? (
        <ChromeModal
          onClose={() => setChromeModal(null)}
          testID="chrome-modal"
          title={chromeModal === 'export' ? 'Export' : 'Settings'}>
          {chromeModal === 'export' ? (
            <ExportScreen
              onExport={() => {
                const path = exportOutputPath.trim();
                if (!path) {
                  setExportPhase('failed');
                  setExportStatus('output path is required');
                  return;
                }
                setExportPhase('exporting');
                setExportStatus('Exporting…');
                const result = dispatchHostIntentResult('export_document', {output_path: path});
                if (result.accepted) {
                  setExportPhase('complete');
                  setExportStatus(result.message ?? 'Export complete');
                } else {
                  setExportPhase('failed');
                  setExportStatus(result.message ?? 'Export refused');
                }
              }}
              onOutputPathChange={setExportOutputPath}
              outputPath={exportOutputPath}
              phase={exportPhase}
              statusText={exportStatus}
            />
          ) : (
            <SettingsScreen />
          )}
        </ChromeModal>
      ) : null}
    </ShellView>
  );
}

export default App;
