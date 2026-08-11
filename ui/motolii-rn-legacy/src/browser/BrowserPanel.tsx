import React from 'react';
import {StyleSheet, Text, TouchableOpacity, View} from 'react-native';

type BrowserItem = {
  id: string;
  label: string;
};

export type BrowserPlaceRectangleIntent = {
  kind: 'place_rectangle';
  position: [number, number];
  playhead: number;
};

export type BrowserPanelProps = {
  onPlaceIntent?: (intent: BrowserPlaceRectangleIntent) => void;
};

const browserItems: Array<BrowserItem> = [{id: 'rectangle', label: 'Rectangle'}];

// 既定値は編集作業の開始位置として使いやすい座標(100, 100)を固定。
const DEFAULT_POSITION: [number, number] = [100, 100];
// 既定値は再生時間の先頭(playhead=0)で一貫して配置するため。
const DEFAULT_PLAYHEAD = 0;

export default function BrowserPanel({onPlaceIntent}: BrowserPanelProps) {
  const onPress = () => {
    const intent: BrowserPlaceRectangleIntent = {
      kind: 'place_rectangle',
      position: DEFAULT_POSITION,
      playhead: DEFAULT_PLAYHEAD,
    };
    onPlaceIntent?.(intent);
  };

  return (
    <View style={styles.panel} testID="browser-panel">
      <Text style={styles.heading}>Browser</Text>
      <View style={styles.list}>
        {browserItems.map((item) => (
          <TouchableOpacity
            key={item.id}
            style={styles.item}
            testID={`browser-item-${item.id}`}
            onPress={onPress}>
            <Text>{item.label}</Text>
          </TouchableOpacity>
        ))}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  panel: {
    flex: 1,
    padding: 12,
    backgroundColor: '#1b1d20',
  },
  heading: {
    color: '#e9e9e5',
    fontSize: 12,
    fontWeight: '700',
  },
  list: {
    marginTop: 12,
  },
  item: {
    paddingVertical: 8,
    paddingHorizontal: 10,
    borderWidth: 1,
    borderColor: '#34383c',
    marginBottom: 8,
  },
});
