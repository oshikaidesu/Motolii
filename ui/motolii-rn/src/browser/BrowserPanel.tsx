import React from 'react';
import {StyleSheet, Text, View} from 'react-native';

import type {BrowserViewModel} from './BrowserPresentationModel';
import MotoliiBrowserDragSourceView from '../specs/MotoliiBrowserDragSourceNativeComponent';

export type BrowserPanelProps = {
  hostHandle: string;
  viewModel: BrowserViewModel;
};

export default function BrowserPanel({hostHandle, viewModel}: BrowserPanelProps) {
  const sourceIsHostIssued =
    viewModel.state === 'available' &&
    /^[1-9][0-9]*$/.test(hostHandle) &&
    viewModel.rectangleSource.scopeRef === `builtin-${hostHandle}`;
  return (
    <View style={styles.surface} testID="motolii-browser">
      <View style={styles.header}>
        <View style={styles.marker} />
        <Text style={styles.headerText}>BROWSER</Text>
      </View>
      <Text style={styles.context}>CREATE</Text>
      {viewModel.state === 'available' && sourceIsHostIssued ? (
        <MotoliiBrowserDragSourceView
          accessibilityLabel="Rectangle · Shape · Built-in"
          accessibilityValue={{
            text: `${viewModel.rectangleSource.scopeRef}/${viewModel.rectangleSource.itemId}`,
          }}
          hostHandle={hostHandle}
          itemId={viewModel.rectangleSource.itemId}
          nativeID={`${viewModel.rectangleSource.scopeRef}/${viewModel.rectangleSource.itemId}`}
          scopeRef={viewModel.rectangleSource.scopeRef}
          style={styles.card}
          testID="browser-rectangle-card">
          <View style={styles.preview}>
            <Text style={styles.glyph}>□</Text>
            <Text style={styles.provider}>Built-in</Text>
          </View>
          <View style={styles.cardCopy}>
            <Text style={styles.name}>Rectangle</Text>
            <Text style={styles.kind}>Shape</Text>
          </View>
        </MotoliiBrowserDragSourceView>
      ) : (
        <View style={styles.unavailable} testID="browser-unavailable">
          <Text style={styles.unavailableTitle}>Browser unavailable</Text>
          <Text style={styles.unavailableBody}>
            Create items could not be displayed safely.
          </Text>
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
  context: {
    marginTop: 12,
    color: '#858b8f',
    fontSize: 9,
    fontWeight: '700',
    letterSpacing: 0.8,
  },
  card: {
    marginTop: 8,
    padding: 8,
    borderWidth: 1,
    borderColor: '#454a4e',
    backgroundColor: '#202326',
  },
  preview: {
    height: 54,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#292d30',
  },
  glyph: {
    color: '#d6d8d6',
    fontSize: 22,
  },
  provider: {
    marginTop: 3,
    color: '#858b8f',
    fontSize: 8,
  },
  cardCopy: {
    marginTop: 7,
  },
  name: {
    color: '#e2e4e2',
    fontSize: 11,
    fontWeight: '700',
  },
  kind: {
    marginTop: 2,
    color: '#858b8f',
    fontSize: 9,
  },
  unavailable: {
    marginTop: 12,
    padding: 10,
    borderWidth: 1,
    borderColor: '#704d4c',
    backgroundColor: '#241b1d',
  },
  unavailableTitle: {
    color: '#f0d1c9',
    fontSize: 11,
    fontWeight: '700',
  },
  unavailableBody: {
    marginTop: 6,
    color: '#d8b8af',
    fontSize: 10,
    lineHeight: 15,
  },
});
