/**
 * @format
 */

import React from 'react';
import ReactTestRenderer from 'react-test-renderer';
import {Text, TurboModuleRegistry} from 'react-native';
import App from '../App';

const mockDispatchIntent = jest.fn(() => '{"accepted":true}');
const mockReadSnapshot = jest.fn(() => '');
const actualGet = TurboModuleRegistry.get.bind(TurboModuleRegistry);

function mockHostGet(name: string) {
  if (name === 'NativeMotoliiHost') {
    return {
      dispatchIntent: mockDispatchIntent,
      readSnapshot: mockReadSnapshot,
    };
  }
  return actualGet(name);
}

function mockNullHostGet(name: string) {
  if (name === 'NativeMotoliiHost') {
    return null;
  }
  return actualGet(name);
}

test('renders correctly', async () => {
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  expect(tree!.root.findByProps({testID: 'motolii-rn-shell'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'timeline'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'browser-surface'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'browser-view-EFFECTS'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'rust-wgpu-timeline'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'native-timeline-feedback'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'native-timeline-feedback'}).props.children).toBe(
    'clip 1 · 27.0%',
  );
  expect(tree!.root.findByProps({testID: 'timeline-key-tools'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'timeline-key-mode-KEYS'}).props.accessibilityState.selected).toBe(true);
  expect(tree!.root.findAllByProps({accessibilityLabel: 'Select previous native clip'})).toHaveLength(0);
  expect(tree!.root.findAllByProps({accessibilityLabel: 'Select next native clip'})).toHaveLength(0);
  expect(tree!.root.findAllByProps({testID: 'timeline-mode-PACKING'})).toHaveLength(0);
  expect(tree!.root.findAllByProps({testID: 'timeline-mode-DENSITY'})).toHaveLength(0);
  expect(tree!.root.findAllByProps({testID: 'timeline-mode-NATIVE'})).toHaveLength(0);
  expect(tree!.root.findByProps({testID: 'inspector-surface'})).toBeTruthy();
  expect(tree!.root.findAllByProps({testID: 'inspector-layer-section'})).toHaveLength(0);
  expect(tree!.root.findByProps({testID: 'path-operations-panel'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'stage-transform-projection'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'rust-wgpu-timeline'}).props.onTimelineFeedback({
      nativeEvent: {objectIndex: -1, time: 0.27},
    });
  });
  expect(tree!.root.findByProps({testID: 'native-timeline-feedback'}).props.children).toBe(
    'no clip · 27.0%',
  );
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.onStageTransform({
      nativeEvent: {x: 0.25, y: -0.5, z: 0.75, rotationX: 10, rotationY: 20, rotationZ: 30},
    });
  });
  expect(tree!.root.findByProps({value: '0.250'})).toBeTruthy();
  expect(tree!.root.findByProps({value: '30.0'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    const positionDial = tree!.root.findByProps({accessibilityLabel: 'Position X dial'});
    const startTouch = {
      touchActive: true,
      currentTimeStamp: 1,
      currentPageX: 100,
      currentPageY: 100,
      previousPageX: 100,
      previousPageY: 100,
    };
    const moveTouch = {...startTouch, currentTimeStamp: 2, currentPageX: 125};
    const startEvent = {touchHistory: {numberActiveTouches: 1, indexOfSingleActiveTouch: 0, touchBank: [startTouch], mostRecentTimeStamp: 1}};
    const moveEvent = {touchHistory: {numberActiveTouches: 1, indexOfSingleActiveTouch: 0, touchBank: [moveTouch], mostRecentTimeStamp: 2}};
    positionDial.props.onResponderGrant(startEvent);
    positionDial.props.onResponderMove(moveEvent);
    positionDial.props.onResponderRelease(moveEvent);
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({accessibilityLabel: 'Rotation Z value'}).props.onChangeText('25');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({accessibilityLabel: 'Rotation Z value'}).props.onSubmitEditing();
  });
  expect(tree!.root.findByProps({value: '0.500'})).toBeTruthy();
  expect(tree!.root.findByProps({value: '25.0'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.transformX).toBe(0.5);
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.rotationZ).toBe(25);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'path-operation-trim'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'path-operation-trim'}).props.accessibilityState.selected).toBe(true);
  expect(tree!.root.findByProps({testID: 'path-operation-pucker-bloat'}).props.accessibilityState.selected).toBe(false);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-tab-MEDIA'}).props.onPress();
    tree!.root.findByProps({testID: 'right-panel-EXTENSIONS'}).props.onPress();
  });

  expect(tree!.root.findByProps({testID: 'thumbnail-grid'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'browser-view-MEDIA'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'extension-panel-asset-tags'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'timeline-key-mode-LAYERS'}).props.onPress();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({accessibilityLabel: 'stagger layer section'}).props.onPress();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({accessibilityLabel: 'Layerを等間隔に分布'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'timeline-key-mode-LAYERS'}).props.accessibilityState.selected).toBe(true);
  expect(tree!.root.findByProps({testID: 'timeline-key-tools-hint'}).props.children).toBe('Layerを等間隔に分布 requires a Timeline selection');

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

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.onStageDrop({
      nativeEvent: {x: 0.25, y: 0.75, canonicalX: 0.333333, canonicalY: -0.777777},
    });
  });
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.createdItemId).toBe('rectangle@0.250000,0.750000|trim');
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.draggedItemId).toBe('');
  expect(mockDispatchIntent).toHaveBeenCalled();
  const placePayload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(placePayload.kind).toBe('place_rectangle');
  expect(placePayload.position).toEqual([0.333333, -0.777777]);

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'titlebar-undo'}).props.onPress();
  });
  expect(mockDispatchIntent).toHaveBeenCalled();
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string).kind).toBe('undo');

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

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('renders without NativeMotoliiHost module', async () => {
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockNullHostGet as typeof TurboModuleRegistry.get);
  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });
  expect(tree!.root.findByProps({testID: 'motolii-rn-shell'})).toBeTruthy();
  expect(tree!.root.findAllByProps({testID: 'inspector-layer-section'})).toHaveLength(0);
  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('inspector layer section shows snapshot seat and dispatches add_position_key', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 2, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [{key_id: '7', time: {num: 0, den: 1}}],
            keys_truncated: false,
          },
        ],
        layers_truncated: false,
      },
    }),
  );
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  expect(tree!.root.findByProps({testID: 'inspector-layer-section'})).toBeTruthy();
  const layerTexts = tree!.root
    .findByProps({testID: 'inspector-layer-section'})
    .findAllByType(Text)
    .map(node => {
      const children = node.props.children;
      return Array.isArray(children) ? children.join('') : String(children);
    });
  expect(layerTexts).toContain('seed-layer');
  expect(layerTexts).toContain('position keys: 1');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-add-position-key'}).props.onPress();
  });
  expect(mockDispatchIntent).toHaveBeenCalled();
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('add_position_key');
  expect(payload.target).toBe('42');
  expect(payload.time).toEqual({num: 2, den: 1});

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector add position key is disabled when no layer is selected', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 2, den: 1},
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [],
      },
    }),
  );
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  const addButton = tree!.root.findByProps({testID: 'inspector-add-position-key'});
  expect(addButton.props.disabled).toBe(true);
  await ReactTestRenderer.act(() => {
    addButton.props.onPress();
  });
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector shows exact-on-key X/Y rows only when playhead matches key time', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 2, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [
              {key_id: '7', time: {num: 4, den: 2}, value: [0.25, -0.5]},
            ],
            keys_truncated: false,
          },
        ],
        layers_truncated: false,
      },
    }),
  );
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'inspector-position-key-y'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.value).toBe('0.25');
  expect(tree!.root.findByProps({testID: 'inspector-position-key-y'}).props.value).toBe('-0.5');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });

  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 2, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [
              {key_id: '7', time: {num: 0, den: 1}, value: [0.25, -0.5]},
            ],
            keys_truncated: false,
          },
        ],
        layers_truncated: false,
      },
    }),
  );
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });
  expect(tree!.root.findAllByProps({testID: 'inspector-position-key-x'})).toHaveLength(0);
  expect(tree!.root.findAllByProps({testID: 'inspector-position-key-y'})).toHaveLength(0);
  expect(tree!.root.findByProps({testID: 'inspector-layer-section'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector X commit dispatches set_position_key_value once with exact time', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 1, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [
              {key_id: '7', time: {num: 1, den: 1}, value: [0.1, 0.2]},
            ],
            keys_truncated: false,
          },
        ],
        layers_truncated: false,
      },
    }),
  );
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  const xInput = tree!.root.findByProps({testID: 'inspector-position-key-x'});
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    xInput.props.onChangeText('0.75');
  });
  await ReactTestRenderer.act(() => {
    const committed = tree!.root.findByProps({testID: 'inspector-position-key-x'});
    committed.props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    const committed = tree!.root.findByProps({testID: 'inspector-position-key-x'});
    committed.props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('set_position_key_value');
  expect(payload.target).toBe('42');
  expect(payload.time).toEqual({num: 1, den: 1});
  expect(payload.new).toEqual([0.75, 0.2]);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector key editor remounts on primary/key change and stale draft is not committed', async () => {
  mockDispatchIntent.mockClear();
  jest.useFakeTimers();
  const snapshotA = {
    revision: '3',
    projection_generation: '1',
    current_time: {num: 1, den: 1},
    primary_layer_id: '42',
    stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
    timeline: {
      fps: {num: 30, den: 1},
      layers: [
        {
          layer_id: '42',
          display_name: 'seed-layer',
          start: {num: 0, den: 1},
          duration: {num: 10, den: 1},
          position_keys: [{key_id: 'a', time: {num: 1, den: 1}, value: [0.1, 0.2]}],
          keys_truncated: false,
        },
      ],
      layers_truncated: false,
    },
  };
  const snapshotB = {
    revision: '3',
    projection_generation: '1',
    current_time: {num: 1, den: 1},
    primary_layer_id: '43',
    stage: {bounds: [{layer_id: '43', display_name: 'other-layer'}]},
    timeline: {
      fps: {num: 30, den: 1},
      layers: [
        {
          layer_id: '43',
          display_name: 'other-layer',
          start: {num: 0, den: 1},
          duration: {num: 10, den: 1},
          position_keys: [{key_id: 'b', time: {num: 1, den: 1}, value: [0.5, 0.75]}],
          keys_truncated: false,
        },
      ],
      layers_truncated: false,
    },
  };
  let readCount = 0;
  mockReadSnapshot.mockImplementation(() => {
    const snapshot = readCount === 0 ? snapshotA : snapshotB;
    readCount += 1;
    return JSON.stringify(snapshot);
  });

  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  const staleInput = tree!.root.findByProps({testID: 'inspector-position-key-x'});
  await ReactTestRenderer.act(() => {
    staleInput.props.onChangeText('0.75');
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });

  const activeInput = tree!.root.findByProps({testID: 'inspector-position-key-x'});
  expect(activeInput.props.value).toBe('0.5');
  await ReactTestRenderer.act(() => {
    activeInput.props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    activeInput.props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
  jest.useRealTimers();
});

test('inspector X empty draft does not dispatch and restores value', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 1, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [
              {key_id: '7', time: {num: 1, den: 1}, value: [0.1, 0.2]},
            ],
            keys_truncated: false,
          },
        ],
        layers_truncated: false,
      },
    }),
  );
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  const xInput = tree!.root.findByProps({testID: 'inspector-position-key-x'});
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    xInput.props.onChangeText('  ');
  });
  await ReactTestRenderer.act(() => {
    const committed = tree!.root.findByProps({testID: 'inspector-position-key-x'});
    committed.props.onSubmitEditing();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.value).toBe('0.1');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});
