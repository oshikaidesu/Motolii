import React from 'react';
import {StyleSheet, Text, View} from 'react-native';

import {decodeInspectorInitialRead} from './decodeInspectorInitialRead';

export type InspectorInitialReadPanelProps = {
  snapshotJSON?: string;
};

export default function InspectorInitialReadPanel({
  snapshotJSON,
}: InspectorInitialReadPanelProps) {
  const result =
    snapshotJSON === undefined
      ? undefined
      : decodeInspectorInitialRead(snapshotJSON);

  return (
    <View style={styles.panel} testID="inspector-initial-read-panel">
      <Text style={styles.heading}>Inspector</Text>
      <Text style={styles.subheading}>Initial snapshot</Text>
      {result === undefined || !result.ok ? (
        <Text style={styles.unavailable}>Inspector unavailable</Text>
      ) : result.value.tag === 'no-selection' ? (
        <View testID="inspector-no-selection">
          <Text style={styles.empty}>No selection</Text>
          <Readout label="Revision" value={result.value.revision} />
          <Readout
            label="Generation"
            value={result.value.projectionGeneration}
          />
        </View>
      ) : (
        <View testID="inspector-selected">
          <Text style={styles.displayName}>{result.value.displayName}</Text>
          <Readout label="Layer ID" value={result.value.layerId} />
          <Readout label="Revision" value={result.value.revision} />
          <Readout
            label="Generation"
            value={result.value.projectionGeneration}
          />
        </View>
      )}
    </View>
  );
}

function Readout({label, value}: {label: string; value: string}) {
  return (
    <View style={styles.readout}>
      <Text style={styles.label}>{label}</Text>
      <Text selectable style={styles.value}>
        {value}
      </Text>
    </View>
  );
}

const styles = StyleSheet.create({
  panel: {
    flex: 1,
    padding: 12,
    borderLeftWidth: 1,
    borderLeftColor: '#34383c',
    backgroundColor: '#1b1d20',
  },
  heading: {
    color: '#e9e9e5',
    fontSize: 12,
    fontWeight: '700',
  },
  subheading: {
    marginTop: 2,
    color: '#9ca1a4',
    fontSize: 9,
    fontWeight: '700',
    letterSpacing: 1,
  },
  unavailable: {
    marginTop: 16,
    color: '#9ca1a4',
    fontSize: 10,
  },
  empty: {
    marginTop: 16,
    marginBottom: 8,
    color: '#d6d8d6',
    fontSize: 11,
  },
  displayName: {
    marginTop: 16,
    marginBottom: 8,
    color: '#e9e9e5',
    fontSize: 13,
    fontWeight: '600',
  },
  readout: {
    marginTop: 8,
  },
  label: {
    color: '#9ca1a4',
    fontSize: 9,
  },
  value: {
    marginTop: 2,
    color: '#d6d8d6',
    fontFamily: 'Menlo',
    fontSize: 10,
  },
});
