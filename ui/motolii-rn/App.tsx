import React, {useEffect, useState} from 'react';
import {StyleSheet, Text, View} from 'react-native';

import BrowserPanel from './src/browser/BrowserPanel';
import {decodeBrowserSnapshot} from './src/browser/BrowserPresentationModel';
import InspectorPanel from './src/inspector/InspectorPanel';
import {decodeInspectorSnapshot} from './src/inspector/InspectorPresentationModel';
import MotoliiStageView from './src/specs/MotoliiStageNativeComponent';
import MotoliiSnapshotChannel from './src/specs/NativeMotoliiSnapshotChannel';

export type MotoliiProductProps = {
  hostHandle?: string;
  projectPath?: string;
  snapshotJSON?: string;
  diagnostic?: string;
};

const MAX_SNAPSHOT_JSON_LENGTH = 16 * 1024;

function acceptsSnapshotEvent(snapshotJSON: string, hostHandle: string): boolean {
  if (
    snapshotJSON.length === 0 ||
    snapshotJSON.length > MAX_SNAPSHOT_JSON_LENGTH ||
    decodeBrowserSnapshot(snapshotJSON).state === 'invalid' ||
    decodeInspectorSnapshot(snapshotJSON).state === 'invalid'
  ) {
    return false;
  }

  try {
    const snapshot = JSON.parse(snapshotJSON) as {host_handle?: unknown};
    return snapshot.host_handle === hostHandle;
  } catch {
    return false;
  }
}

function SnapshotReadout({snapshotJSON}: {snapshotJSON?: string}) {
  return (
    <View style={styles.snapshot} testID="host-snapshot">
      <Text style={styles.label}>HOST SNAPSHOT</Text>
      <Text selectable style={styles.snapshotText}>
        {snapshotJSON ?? 'snapshot unavailable'}
      </Text>
    </View>
  );
}

export default function App({
  hostHandle = '0',
  projectPath,
  snapshotJSON,
  diagnostic,
}: MotoliiProductProps) {
  const hasHost = /^[1-9][0-9]*$/.test(hostHandle);
  const [presentationSnapshotJSON, setPresentationSnapshotJSON] = useState(
    snapshotJSON,
  );

  useEffect(() => {
    if (!hasHost) {
      return;
    }

    let active = true;
    const subscription = MotoliiSnapshotChannel.onSnapshotChanged(nextJSON => {
      if (!active || !acceptsSnapshotEvent(nextJSON, hostHandle)) {
        return;
      }
      setPresentationSnapshotJSON(previousJSON =>
        previousJSON === nextJSON ? previousJSON : nextJSON,
      );
    });
    return () => {
      active = false;
      subscription.remove();
    };
  }, [hasHost, hostHandle]);

  const browserViewModel = decodeBrowserSnapshot(presentationSnapshotJSON);
  const inspectorViewModel = decodeInspectorSnapshot(presentationSnapshotJSON);

  return (
    <View style={styles.root} testID="motolii-rn-product-root">
      <View style={styles.titlebar}>
        <Text style={styles.title}>MOTOLII</Text>
        <Text style={styles.path}>{projectPath ?? 'project path missing'}</Text>
      </View>
      <View style={styles.workspace} testID="motolii-rn-workspace">
        <View style={styles.browserSlot} testID="motolii-rn-browser-slot">
          <BrowserPanel hostHandle={hostHandle} viewModel={browserViewModel} />
        </View>
        <View style={styles.stageSlot} testID="motolii-rn-stage-slot">
          {hasHost ? (
            <MotoliiStageView
              accessibilityLabel="Motolii Fabric Stage placeholder"
              hostHandle={hostHandle}
              style={styles.stage}
              testID="motolii-stage"
            />
          ) : (
            <View style={styles.failure} testID="host-create-failure">
              <Text style={styles.failureTitle}>Host unavailable</Text>
              <Text style={styles.failureText}>
                {diagnostic ?? 'An explicit project path is required.'}
              </Text>
            </View>
          )}
          {/* Stage領域の観測用。InspectorドメインUIではない */}
          <SnapshotReadout snapshotJSON={presentationSnapshotJSON} />
        </View>
        <View style={styles.inspectorSlot} testID="motolii-rn-inspector-slot">
          <InspectorPanel viewModel={inspectorViewModel} />
        </View>
      </View>
      <View style={styles.timelineSlot} testID="motolii-rn-timeline-slot">
        <Text style={styles.slotLabel}>TIMELINE</Text>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  root: {
    flex: 1,
    minWidth: 720,
    minHeight: 480,
    backgroundColor: '#111315',
  },
  titlebar: {
    height: 38,
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 14,
    borderBottomWidth: 1,
    borderBottomColor: '#34383c',
    backgroundColor: '#202326',
  },
  title: {
    color: '#e9e9e5',
    fontSize: 12,
    fontWeight: '700',
    letterSpacing: 1,
  },
  path: {
    flex: 1,
    marginLeft: 16,
    color: '#9ca1a4',
    fontSize: 10,
  },
  workspace: {
    flex: 4,
    flexDirection: 'row',
    minHeight: 0,
  },
  browserSlot: {
    flex: 1,
    minWidth: 0,
    padding: 12,
    borderRightWidth: 1,
    borderRightColor: '#34383c',
    backgroundColor: '#17191b',
  },
  stageSlot: {
    flex: 3,
    minWidth: 0,
    padding: 12,
  },
  inspectorSlot: {
    flex: 1,
    minWidth: 0,
    padding: 12,
    borderLeftWidth: 1,
    borderLeftColor: '#34383c',
    backgroundColor: '#1b1d20',
  },
  timelineSlot: {
    flex: 1,
    minHeight: 0,
    padding: 12,
    borderTopWidth: 1,
    borderTopColor: '#34383c',
    backgroundColor: '#17191b',
  },
  slotLabel: {
    color: '#b9ae73',
    fontSize: 9,
    fontWeight: '700',
    letterSpacing: 1,
  },
  stage: {
    flex: 1,
    minHeight: 160,
  },
  snapshot: {
    marginTop: 8,
    padding: 8,
    borderWidth: 1,
    borderColor: '#34383c',
    backgroundColor: '#1b1d20',
  },
  label: {
    color: '#b9ae73',
    fontSize: 9,
    fontWeight: '700',
    letterSpacing: 1,
  },
  snapshotText: {
    marginTop: 8,
    color: '#d6d8d6',
    fontFamily: 'Menlo',
    fontSize: 9,
  },
  failure: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'center',
    borderWidth: 1,
    borderColor: '#704d4c',
    backgroundColor: '#241b1d',
  },
  failureTitle: {
    color: '#f0d1c9',
    fontSize: 13,
    fontWeight: '700',
  },
  failureText: {
    maxWidth: 360,
    marginTop: 8,
    color: '#d8b8af',
    fontSize: 10,
    textAlign: 'center',
  },
});
