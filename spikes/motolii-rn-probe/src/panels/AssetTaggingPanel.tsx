import React, {useState} from 'react';
import {Pressable, StyleSheet, Text, TextInput, View} from 'react-native';

export function AssetTaggingPanel() {
  const [tags, setTags] = useState(['Interview', 'B-roll']);
  const [draft, setDraft] = useState('');

  return (
    <View style={styles.root} testID="extension-panel-asset-tags">
      <Text style={styles.heading}>Asset Tagging Extension</Text>
      <Text style={styles.caption}>
        Registered from a separate TypeScript module
      </Text>
      <TextInput
        accessibilityLabel="New asset tag"
        onChangeText={setDraft}
        onSubmitEditing={() => {
          if (draft.trim()) {
            setTags(current => [...current, draft.trim()]);
            setDraft('');
          }
        }}
        placeholder="Add tag and press Return"
        placeholderTextColor="#69707d"
        style={styles.input}
        value={draft}
      />
      <View style={styles.tags}>
        {tags.map(tag => (
          <Pressable key={tag} style={styles.tag}>
            <Text style={styles.tagText}>{tag}</Text>
          </Pressable>
        ))}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  root: {padding: 12},
  heading: {fontSize: 14, fontWeight: '600', color: '#f0f2f6'},
  caption: {fontSize: 10, marginTop: 3, color: '#8e95a2'},
  input: {
    height: 32,
    marginTop: 14,
    paddingHorizontal: 8,
    color: '#f4f5f7',
    backgroundColor: '#15171c',
    borderWidth: 1,
    borderColor: '#484e59',
    borderRadius: 4,
  },
  tags: {flexDirection: 'row', flexWrap: 'wrap', gap: 6, marginTop: 10},
  tag: {paddingVertical: 4, paddingHorizontal: 8, backgroundColor: '#343b49', borderRadius: 10},
  tagText: {fontSize: 10, color: '#dce0e7'},
});
