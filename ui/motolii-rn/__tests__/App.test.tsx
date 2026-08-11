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
  expect(tree!.root.findByProps({testID: 'browser-view-EFFECTS'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'packing-timeline'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'inspector-surface'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'path-operations-panel'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'stage-transform-projection'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'ui-version'}).props.children).toContain('UI 0.0.2');

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.onStageTransform({
      nativeEvent: {x: 0.25, y: -0.5, z: 0.75, rotationX: 10, rotationY: 20, rotationZ: 30},
    });
  });
  expect(tree!.root.findByProps({children: '0.250'})).toBeTruthy();
  expect(tree!.root.findByProps({children: '30.0°'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({accessibilityLabel: 'Position X increase'}).props.onPress();
    tree!.root.findByProps({accessibilityLabel: 'Rotation Z decrease'}).props.onPress();
  });
  expect(tree!.root.findByProps({children: '0.275'})).toBeTruthy();
  expect(tree!.root.findByProps({children: '25.0°'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.transformX).toBe(0.275);
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.rotationZ).toBe(25);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'path-operation-trim'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'path-operation-trim'}).props.accessibilityState.selected).toBe(true);
  expect(tree!.root.findByProps({testID: 'path-operation-pucker-bloat'}).props.accessibilityState.selected).toBe(false);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-tab-MEDIA'}).props.onPress();
    tree!.root.findByProps({testID: 'timeline-mode-DENSITY'}).props.onPress();
    tree!.root.findByProps({testID: 'right-panel-EXTENSIONS'}).props.onPress();
  });

  expect(tree!.root.findByProps({testID: 'thumbnail-grid'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'browser-view-MEDIA'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'timeline-density-grid'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'extension-panel-asset-tags'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'timeline-mode-NATIVE'}).props.onPress();
  });

  expect(tree!.root.findByProps({testID: 'rust-wgpu-timeline'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'native-timeline-feedback'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-tab-CREATE'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'browser-view-CREATE'})).toBeTruthy();
  const rectangle = tree!.root.findByProps({testID: 'create-item-rectangle'});
  expect(rectangle.props.accessibilityState.selected).toBe(false);

  await ReactTestRenderer.act(() => {
    rectangle.props.onPress();
  });
  expect(
    tree!.root.findByProps({testID: 'create-item-rectangle'}).props.accessibilityState.selected,
  ).toBe(true);
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.createdItemId).toBe('rectangle@0.500000,0.500000|trim');

  await ReactTestRenderer.act(() => {
    rectangle.props.onPointerDown();
  });
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.draggedItemId).toBe('rectangle');

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.onStageDrop({nativeEvent: {x: 0.25, y: 0.75}});
  });
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.createdItemId).toBe('rectangle@0.250000,0.750000|trim');
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.draggedItemId).toBe('');

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'create-item-rectangle'}).props.onDoubleClick();
  });
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.createdItemId).toBe('rectangle@0.500000,0.500000|trim');

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-mode-THUMBNAILS'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'browser-results-CREATE'}).props.numColumns).toBe(4);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-mode-LIST'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'browser-results-CREATE'}).props.numColumns).toBe(1);
});
