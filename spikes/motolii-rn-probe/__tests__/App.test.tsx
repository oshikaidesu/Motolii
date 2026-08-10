/**
 * @format
 */

import React from 'react';
import ReactTestRenderer from 'react-test-renderer';
import App from '../App';

test('renders correctly', async () => {
  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  expect(tree!.root.findByProps({testID: 'motolii-rn-shell'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'timeline'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'browser-surface'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'packing-timeline'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'inspector-surface'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-tab-MEDIA'}).props.onPress();
    tree!.root.findByProps({testID: 'timeline-mode-DENSITY'}).props.onPress();
    tree!.root.findByProps({testID: 'right-panel-EXTENSIONS'}).props.onPress();
  });

  expect(tree!.root.findByProps({testID: 'thumbnail-grid'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'timeline-density-grid'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'extension-panel-asset-tags'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'timeline-mode-NATIVE'}).props.onPress();
  });

  expect(tree!.root.findByProps({testID: 'rust-wgpu-timeline'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'native-timeline-feedback'})).toBeTruthy();
});
