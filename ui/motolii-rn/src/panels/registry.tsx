import React from 'react';
import {StyleSheet, Text, View} from 'react-native';

import {AssetTaggingPanel} from './AssetTaggingPanel';

function ExportNotesPanel() {
  return (
    <View style={styles.root} testID="extension-panel-export-notes">
      <Text style={styles.title}>Export Notes Extension</Text>
      <Text style={styles.body}>A second panel proves registry-driven selection.</Text>
    </View>
  );
}

export const panelRegistry = [
  {id: 'asset-tags', title: 'Tags', Component: AssetTaggingPanel},
  {id: 'export-notes', title: 'Notes', Component: ExportNotesPanel},
] as const;

const styles = StyleSheet.create({
  root: {padding: 12},
  title: {fontSize: 14, fontWeight: '600', color: '#f0f2f6'},
  body: {fontSize: 11, marginTop: 8, color: '#9da3af'},
});
