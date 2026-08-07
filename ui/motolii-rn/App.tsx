import React from 'react';
import {StyleSheet, Text, View} from 'react-native';

import InspectorInitialReadPanel from './src/inspector/InspectorInitialReadPanel';
import MotoliiStageView from './src/specs/MotoliiStageNativeComponent';

export type MotoliiProductProps = {
  hostHandle?: string;
  projectPath?: string;
  snapshotJSON?: string;
  diagnostic?: string;
};

export default function App({
  hostHandle = '0',
  projectPath,
  snapshotJSON,
  diagnostic,
}: MotoliiProductProps) {
  const hasHost = /^[1-9][0-9]*$/.test(hostHandle);

  return (
    <View style={styles.root} testID="motolii-rn-product-root">
      <View style={styles.titlebar}>
        <Text style={styles.title}>MOTOLII · R0</Text>
        <Text style={styles.path}>{projectPath ?? 'project path missing'}</Text>
      </View>
      <View style={styles.workspace}>
        <View style={styles.browserSlot} testID="browser-slot">
          <Text style={styles.slotLabel}>Browser</Text>
        </View>
        <View style={styles.stageSlot} testID="stage-slot">
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
        </View>
        <View style={styles.inspectorSlot} testID="inspector-slot">
          <InspectorInitialReadPanel snapshotJSON={snapshotJSON} />
        </View>
      </View>
      <View style={styles.timelineSlot} testID="timeline-slot">
        <Text style={styles.slotLabel}>Timeline</Text>
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
    flex: 1,
    flexDirection: 'row',
  },
  browserSlot: {
    width: 200,
    padding: 12,
    borderRightWidth: 1,
    borderRightColor: '#34383c',
    backgroundColor: '#1b1d20',
  },
  stageSlot: {
    flex: 1,
    padding: 12,
  },
  inspectorSlot: {
    width: 240,
  },
  timelineSlot: {
    height: 120,
    padding: 12,
    borderTopWidth: 1,
    borderTopColor: '#34383c',
    backgroundColor: '#1b1d20',
  },
  slotLabel: {
    color: '#9ca1a4',
    fontSize: 10,
    fontWeight: '700',
    letterSpacing: 1,
  },
  stage: {
    flex: 1,
    minHeight: 320,
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
