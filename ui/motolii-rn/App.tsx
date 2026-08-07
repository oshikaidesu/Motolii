import React from 'react';
import {StyleSheet, Text, View} from 'react-native';

import MotoliiStageView from './src/specs/MotoliiStageNativeComponent';

export type MotoliiProductProps = {
  hostHandle?: string;
  projectPath?: string;
  snapshotJSON?: string;
  diagnostic?: string;
};

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

  return (
    <View style={styles.root} testID="motolii-rn-product-root">
      <View style={styles.titlebar}>
        <Text style={styles.title}>MOTOLII · R0</Text>
        <Text style={styles.path}>{projectPath ?? 'project path missing'}</Text>
      </View>
      <View style={styles.content}>
        <View style={styles.stageColumn}>
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
        <SnapshotReadout snapshotJSON={snapshotJSON} />
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
  content: {
    flex: 1,
    flexDirection: 'row',
  },
  stageColumn: {
    flex: 1,
    padding: 12,
  },
  stage: {
    flex: 1,
    minHeight: 320,
  },
  snapshot: {
    width: 240,
    padding: 12,
    borderLeftWidth: 1,
    borderLeftColor: '#34383c',
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
