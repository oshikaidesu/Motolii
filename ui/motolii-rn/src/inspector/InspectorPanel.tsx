import React from 'react';
import {StyleSheet, Text, View} from 'react-native';

import type {InspectorViewModel} from './InspectorPresentationModel';

export type InspectorPanelProps = {
  viewModel: InspectorViewModel;
};

export default function InspectorPanel({viewModel}: InspectorPanelProps) {
  return (
    <View style={styles.surface} testID="motolii-inspector">
      <View style={styles.header}>
        <View style={styles.marker} />
        <Text style={styles.headerText}>INSPECTOR</Text>
      </View>
      {viewModel.state === 'none' ? (
        <View style={styles.message} testID="inspector-no-selection">
          <Text style={styles.messageTitle}>No selection</Text>
          <Text style={styles.messageBody}>
            Select an item to inspect its properties.
          </Text>
        </View>
      ) : viewModel.state === 'invalid' ? (
        <View style={styles.invalid} testID="inspector-invalid-data">
          <Text style={styles.invalidTitle}>Inspector data unavailable</Text>
          <Text style={styles.invalidBody}>
            The current selection could not be displayed safely.
          </Text>
        </View>
      ) : (
        <View style={styles.identity} testID="inspector-selected-target">
          <View style={styles.identityIcon}>
            <Text style={styles.identityIconText}>L</Text>
          </View>
          <View style={styles.identityText}>
            <Text numberOfLines={1} style={styles.targetName}>
              {viewModel.target.displayName}
            </Text>
            <Text style={styles.targetIdentity}>
              Layer {viewModel.target.layerId}
            </Text>
            <Text style={styles.targetRevision}>
              Revision {viewModel.target.revision} · Generation{' '}
              {viewModel.target.projectionGeneration}
            </Text>
          </View>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  surface: {
    flex: 1,
    minWidth: 0,
  },
  header: {
    height: 28,
    flexDirection: 'row',
    alignItems: 'center',
    borderBottomWidth: 1,
    borderBottomColor: '#34383c',
  },
  marker: {
    width: 7,
    height: 7,
    marginRight: 7,
    backgroundColor: '#b9ae73',
  },
  headerText: {
    color: '#d6d8d6',
    fontSize: 10,
    fontWeight: '700',
    letterSpacing: 0.8,
  },
  message: {
    paddingVertical: 20,
    paddingHorizontal: 4,
  },
  messageTitle: {
    color: '#d6d8d6',
    fontSize: 11,
    fontWeight: '600',
  },
  messageBody: {
    marginTop: 6,
    color: '#858b8f',
    fontSize: 10,
    lineHeight: 15,
  },
  invalid: {
    marginTop: 12,
    padding: 10,
    borderWidth: 1,
    borderColor: '#704d4c',
    backgroundColor: '#241b1d',
  },
  invalidTitle: {
    color: '#f0d1c9',
    fontSize: 11,
    fontWeight: '700',
  },
  invalidBody: {
    marginTop: 6,
    color: '#d8b8af',
    fontSize: 10,
    lineHeight: 15,
  },
  identity: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 14,
    paddingHorizontal: 4,
    borderBottomWidth: 1,
    borderBottomColor: '#34383c',
  },
  identityIcon: {
    width: 28,
    height: 28,
    alignItems: 'center',
    justifyContent: 'center',
    borderWidth: 1,
    borderColor: '#545a5e',
    backgroundColor: '#262a2d',
  },
  identityIconText: {
    color: '#b9ae73',
    fontSize: 11,
    fontWeight: '700',
  },
  identityText: {
    flex: 1,
    minWidth: 0,
    marginLeft: 9,
  },
  targetName: {
    color: '#e2e4e2',
    fontSize: 11,
    fontWeight: '700',
  },
  targetIdentity: {
    marginTop: 3,
    color: '#aeb3b6',
    fontFamily: 'Menlo',
    fontSize: 9,
  },
  targetRevision: {
    marginTop: 4,
    color: '#777e82',
    fontFamily: 'Menlo',
    fontSize: 8,
  },
});
