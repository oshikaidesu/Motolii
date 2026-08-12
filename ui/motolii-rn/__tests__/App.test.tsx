/**
 * @format
 */

import React from 'react';
import ReactTestRenderer from 'react-test-renderer';
import {Text, TurboModuleRegistry} from 'react-native';
import App from '../App';

const mockDispatchIntent = jest.fn(() => '{"accepted":true}');
const mockReadSnapshot = jest.fn(() => '');
const mockIsTimelineInteracting = jest.fn(() => false);
const actualGet = TurboModuleRegistry.get.bind(TurboModuleRegistry);

function mockHostGet(name: string) {
  if (name === 'NativeMotoliiHost') {
    return {
      dispatchIntent: mockDispatchIntent,
      readSnapshot: mockReadSnapshot,
      isTimelineInteracting: mockIsTimelineInteracting,
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
  mockReadSnapshot.mockReturnValue('');
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
  expect(placePayload.playhead).toEqual({num: 0, den: 1});

  mockDispatchIntent.mockClear();
  // F3: 履歴ありsnapshotを即時適用してからUndoを撃つ。
  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: true,
      snapshot: {
        revision: '1',
        projection_generation: '1',
        current_time: {num: 0, den: 1},
        history: {can_undo: true, can_redo: false},
        truncated_total: 0,
        stage: {bounds: []},
        timeline: {layers: [], layers_truncated: false},
      },
    }),
  );
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'create-item-rectangle'}).props.onDoubleClick();
  });
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'titlebar-undo'}).props.onPress();
  });
  expect(mockDispatchIntent).toHaveBeenCalled();
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string).kind).toBe('undo');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'create-item-rectangle'}).props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const centerPayload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(centerPayload.kind).toBe('place_rectangle');
  expect(centerPayload.position).toEqual([0, 0]);
  expect(centerPayload.playhead).toEqual({num: 0, den: 1});
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.createdItemId).toBe('rectangle@0.250000,0.750000|trim');

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
  mockReadSnapshot.mockReturnValue('');
});

test('rectangle center and drop placement use snapshot current_time', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 2, den: 1},
      stage: {
        selection: [],
        bounds: [
          {layer_id: '10', display_name: 'seed'},
          {layer_id: '11', display_name: 'other'},
        ],
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

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-tab-CREATE'}).props.onPress();
  });
  const rectangle = tree!.root.findByProps({testID: 'create-item-rectangle'});
  await ReactTestRenderer.act(() => {
    rectangle.props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const centerPayload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(centerPayload.kind).toBe('place_rectangle');
  expect(centerPayload.position).toEqual([0, 0]);
  expect(centerPayload.playhead).toEqual({num: 2, den: 1});

  await ReactTestRenderer.act(() => {
    rectangle.props.onPointerDown();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.onStageDrop({
      nativeEvent: {x: 0.25, y: 0.75, canonicalX: 0.5, canonicalY: 0.25},
    });
  });

  expect(mockDispatchIntent).toHaveBeenCalledTimes(2);
  const placePayload = JSON.parse(mockDispatchIntent.mock.calls[1][0] as string);
  expect(placePayload.kind).toBe('place_rectangle');
  expect(placePayload.playhead).toEqual({num: 2, den: 1});

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
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.value).toBe('0.1');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector X non-finite draft does not dispatch and restores value', async () => {
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
            position_keys: [{key_id: '7', time: {num: 1, den: 1}, value: [0.1, 0.2]}],
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
    xInput.props.onChangeText('abc');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.value).toBe('0.1');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('EFFECTS tab lists host catalog and double-click attaches to primary', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      catalog: {
        effects: [
          {plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1},
          {plugin_id: 'core.param.sine', name: 'Sine', effect_version: 2},
        ],
      },
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
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

  expect(tree!.root.findByProps({testID: 'effect-item-core.filter.opacity'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'effect-item-core.param.sine'})).toBeTruthy();
  expect(
    tree!.root.findByProps({testID: 'effect-item-core.filter.opacity'}).props.accessibilityState.selected,
  ).toBe(true);
  expect(
    tree!.root.findByProps({testID: 'effect-item-core.param.sine'}).props.accessibilityState.selected,
  ).toBe(false);
  expect(tree!.root.findAllByProps({testID: 'effect-item-echo'})).toHaveLength(0);

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'effect-item-core.filter.opacity'}).props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('attach_effect');
  expect(payload.target).toBe('42');
  expect(payload.plugin_id).toBe('core.filter.opacity');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('EFFECTS attach is skipped without primary and fixture remains without catalog', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      stage: {bounds: []},
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
      },
      timeline: {fps: {num: 30, den: 1}, layers: []},
    }),
  );
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'effect-item-core.filter.opacity'}).props.onDoubleClick();
  });
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();

  mockReadSnapshot.mockReturnValue('');
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });
  const names = tree!
    .root.findByProps({testID: 'browser-view-EFFECTS'})
    .findAllByType(Text)
    .map(node => {
      const children = node.props.children;
      return Array.isArray(children) ? children.join('') : String(children);
    });
  expect(names).toContain('Echo Bloom');
  expect(tree!.root.findAllByProps({testID: 'effect-item-core.filter.opacity'})).toHaveLength(0);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
});

test('inspector effect param commit dispatches set_effect_param once', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
      },
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{param_id: 'amount', value: 1}],
              },
            ],
            effects_truncated: false,
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

  expect(tree!.root.findByProps({testID: 'inspector-effects-section'})).toBeTruthy();
  const input = tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'});
  expect(input.props.value).toBe('1');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('0.4');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('set_effect_param');
  expect(payload.target).toBe('42');
  expect(payload.effect_use_id).toBe('9');
  expect(payload.param_id).toBe('amount');
  expect(payload.value).toBe(0.4);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector effect param empty draft does not dispatch and restores value', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
      },
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{param_id: 'amount', value: 1}],
              },
            ],
            effects_truncated: false,
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

  const input = tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'});
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('   ');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.value).toBe('1');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector effect param non-finite draft does not dispatch and restores value', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
      },
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{param_id: 'amount', value: 1}],
              },
            ],
            effects_truncated: false,
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

  const input = tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'});
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('abc');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.value).toBe('1');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector effect param rejected dispatch restores draft and keeps one dispatch', async () => {
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":false}');
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
      },
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{param_id: 'amount', value: 1}],
              },
            ],
            effects_truncated: false,
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

  const input = tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'});
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('0.4');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  expect(tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.value).toBe('1');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('CREATE tab lists host sources and double-click places vism', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 3, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'seed-layer'}]},
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
        sources: [
          {
            plugin_id: 'core.layer_source.radial_repeater',
            name: 'Radial Repeater',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
            source_params: [],
            source_params_truncated: false,
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

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-tab-CREATE'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'create-item-rectangle'})).toBeTruthy();
  expect(
    tree!.root.findByProps({testID: 'create-item-core.layer_source.radial_repeater'}),
  ).toBeTruthy();

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({testID: 'create-item-core.layer_source.radial_repeater'})
      .props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('place_vism');
  expect(payload.plugin_id).toBe('core.layer_source.radial_repeater');
  expect(payload.position).toEqual([0, 0]);
  expect(payload.playhead).toEqual({num: 3, den: 1});

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({testID: 'create-item-core.layer_source.radial_repeater'})
      .props.onPointerDown();
  });
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.draggedItemId).toBe('');

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'create-item-rectangle'}).props.onPointerDown();
  });
  expect(tree!.root.findByProps({testID: 'rust-wgpu-stage'}).props.draggedItemId).toBe(
    'rectangle',
  );

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('CREATE fixture remains without host catalog sources', async () => {
  mockReadSnapshot.mockReturnValue('');
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockNullHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'browser-tab-CREATE'}).props.onPress();
  });
  expect(tree!.root.findByProps({testID: 'create-item-rectangle'})).toBeTruthy();
  expect(
    tree!.root.findAllByProps({testID: 'create-item-core.layer_source.radial_repeater'}),
  ).toHaveLength(0);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('inspector shows source params as display-only rows', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      stage: {bounds: [{layer_id: '42', display_name: 'Radial Repeater'}]},
      catalog: {
        effects: [],
        sources: [
          {
            plugin_id: 'core.layer_source.radial_repeater',
            name: 'Radial Repeater',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'Radial Repeater',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
            source_params: [
              {param_id: 'count', value: 12},
              {param_id: 'radius', value: 0.3},
            ],
            source_params_truncated: false,
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

  expect(tree!.root.findByProps({testID: 'inspector-source-params-section'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'inspector-source-param-count'})).toBeTruthy();
  expect(tree!.root.findByProps({testID: 'inspector-source-param-radius'})).toBeTruthy();
  mockDispatchIntent.mockClear();
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('dispatch accepted snapshot applies immediately without waiting for poll', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '1',
      projection_generation: '0',
      current_time: {num: 0, den: 1},
      history: {can_undo: true, can_redo: false},
      truncated_total: 0,
      stage: {bounds: []},
      timeline: {layers: [], layers_truncated: false},
    }),
  );
  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: true,
      snapshot: {
        revision: '9',
        projection_generation: '2',
        current_time: {num: 3, den: 1},
        primary_layer_id: '42',
        history: {can_undo: true, can_redo: false},
        truncated_total: 0,
        stage: {bounds: [{layer_id: '42', display_name: 'live-layer'}]},
        timeline: {
          fps: {num: 30, den: 1},
          layers: [
            {
              layer_id: '42',
              display_name: 'live-layer',
              start: {num: 0, den: 1},
              duration: {num: 10, den: 1},
              position_keys: [],
              keys_truncated: false,
            },
          ],
          layers_truncated: false,
        },
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

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'titlebar-undo'}).props.onPress();
  });

  expect(
    tree!.root.findAllByType(Text).some(node => node.props.children === 'DOC r9 · 1 layers'),
  ).toBe(true);
  expect(tree!.root.findByProps({testID: 'inspector-layer-section'})).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('titlebar undo and redo stay disabled without history and do not dispatch', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '1',
      projection_generation: '0',
      current_time: {num: 0, den: 1},
      history: {can_undo: false, can_redo: false},
      truncated_total: 0,
      stage: {bounds: []},
      timeline: {layers: [], layers_truncated: false},
    }),
  );
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  expect(tree!.root.findByProps({testID: 'titlebar-undo'}).props.disabled).toBe(true);
  expect(tree!.root.findByProps({testID: 'titlebar-redo'}).props.disabled).toBe(true);
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'titlebar-undo'}).props.onPress?.();
    tree!.root.findByProps({testID: 'titlebar-redo'}).props.onPress?.();
  });
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('DOC status label appends (+) when any position key projection is truncated', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '4',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      truncated_total: 1,
      stage: {bounds: [{layer_id: '42', display_name: 'seed'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: true,
            effects_truncated: false,
            source_params_truncated: false,
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

  expect(
    tree!.root.findAllByType(Text).some(node => node.props.children === 'DOC r4 · 1 layers (+1)'),
  ).toBe(true);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('DOC status label appends (+) when layers are truncated', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '4',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      truncated_total: 2,
      stage: {bounds: [{layer_id: '42', display_name: 'seed'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
            source_params_truncated: false,
          },
        ],
        layers_truncated: true,
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

  expect(
    tree!.root.findAllByType(Text).some(node => node.props.children === 'DOC r4 · 1 layers (+2)'),
  ).toBe(true);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('DOC status label appends (+) when effects are truncated', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '4',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      truncated_total: 1,
      stage: {bounds: [{layer_id: '42', display_name: 'seed'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [{effect_use_id: '1', plugin_id: 'core.filter.opacity', params: []}],
            effects_truncated: true,
            source_params_truncated: false,
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

  expect(
    tree!.root.findAllByType(Text).some(node => node.props.children === 'DOC r4 · 1 layers (+1)'),
  ).toBe(true);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('DOC status label appends (+) when source params are truncated', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '4',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
      primary_layer_id: '42',
      truncated_total: 3,
      stage: {bounds: [{layer_id: '42', display_name: 'seed'}]},
      timeline: {
        fps: {num: 30, den: 1},
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: {num: 0, den: 1},
            duration: {num: 10, den: 1},
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
            source_params: [{param_id: 'p0', value: 0}],
            source_params_truncated: true,
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

  expect(
    tree!.root.findAllByType(Text).some(node => node.props.children === 'DOC r4 · 1 layers (+3)'),
  ).toBe(true);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector freezes exact-on-key row while playhead time-motion continues', async () => {
  jest.useFakeTimers();
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  const snapshotAt = (num: number) =>
    JSON.stringify({
      revision: '3',
      projection_generation: String(num + 1),
      current_time: {num, den: 30},
      primary_layer_id: '42',
      history: {can_undo: false, can_redo: false},
      truncated_total: 0,
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
              {key_id: 'a', time: {num: 0, den: 30}, value: [0.1, 0.2]},
              {key_id: 'b', time: {num: 2, den: 30}, value: [0.3, 0.4]},
            ],
            keys_truncated: false,
          },
        ],
        layers_truncated: false,
      },
    });
  mockReadSnapshot.mockReturnValue(snapshotAt(0));
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });
  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'})).toBeTruthy();

  // gesture実信号がtrueの間は、off-key時刻のsnapshotが来ても行は凍結されたまま。
  mockIsTimelineInteracting.mockReturnValue(true);
  mockReadSnapshot.mockReturnValue(snapshotAt(1));
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });
  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'})).toBeTruthy();

  // 凍結中にprimaryが差し替わっても、commitはcaptureしたidentity(layer 42)へ飛ぶ。
  const swapped = JSON.parse(snapshotAt(2));
  swapped.primary_layer_id = '77';
  swapped.timeline.layers.push({
    ...swapped.timeline.layers[0],
    layer_id: '77',
    display_name: 'other-layer',
  });
  mockReadSnapshot.mockReturnValue(JSON.stringify(swapped));
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.onChangeText('0.9');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const frozenPayload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(frozenPayload.kind).toBe('set_position_key_value');
  expect(frozenPayload.target).toBe('42');
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue(snapshotAt(1));

  // gesture終了(実信号false)で確定し、off-keyの行は消える。
  mockIsTimelineInteracting.mockReturnValue(false);
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });
  expect(tree!.root.findAllByProps({testID: 'inspector-position-key-x'})).toHaveLength(0);
  mockIsTimelineInteracting.mockReturnValue(false);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  jest.useRealTimers();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector X commit preserves Y draft while Y is being edited', async () => {
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
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
            position_keys: [{key_id: 'a', time: {num: 1, den: 1}, value: [0.1, 0.2]}],
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
  const yInput = tree!.root.findByProps({testID: 'inspector-position-key-y'});
  await ReactTestRenderer.act(() => {
    yInput.props.onFocus();
    yInput.props.onChangeText('0.88');
  });
  await ReactTestRenderer.act(() => {
    xInput.props.onFocus();
    xInput.props.onChangeText('0.55');
  });
  await ReactTestRenderer.act(() => {
    xInput.props.onBlur();
  });

  expect(tree!.root.findByProps({testID: 'inspector-position-key-x'}).props.value).toBe('0.55');
  expect(tree!.root.findByProps({testID: 'inspector-position-key-y'}).props.value).toBe('0.88');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('opacity amount commit clamps below 0 and above 1 before dispatch', async () => {
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
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
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{param_id: 'amount', value: 1}],
              },
            ],
            effects_truncated: false,
          },
        ],
        layers_truncated: false,
      },
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
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

  const input = tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'});
  await ReactTestRenderer.act(() => {
    input.props.onFocus();
    input.props.onChangeText('1.5');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onBlur();
  });

  expect(mockDispatchIntent).toHaveBeenCalled();
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('set_effect_param');
  expect(payload.value).toBe(1);
  expect(tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.value).toBe(
    '1',
  );

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('non-opacity amount is not clamped', async () => {
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
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
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.contrast',
                params: [{param_id: 'amount', value: 1}],
              },
            ],
            effects_truncated: false,
          },
        ],
        layers_truncated: false,
      },
      catalog: {
        effects: [{plugin_id: 'core.filter.contrast', name: 'Contrast', effect_version: 1}],
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

  const input = tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'});
  await ReactTestRenderer.act(() => {
    input.props.onFocus();
    input.props.onChangeText('1.5');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onBlur();
  });

  expect(mockDispatchIntent).toHaveBeenCalled();
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('set_effect_param');
  expect(payload.value).toBe(1.5);
  expect(tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.value).toBe(
    '1.5',
  );

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('opacity amount clamps below 0 before dispatch', async () => {
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: {num: 0, den: 1},
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
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{param_id: 'amount', value: 1}],
              },
            ],
            effects_truncated: false,
          },
        ],
        layers_truncated: false,
      },
      catalog: {
        effects: [{plugin_id: 'core.filter.opacity', name: 'Opacity', effect_version: 1}],
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

  const input = tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'});
  await ReactTestRenderer.act(() => {
    input.props.onFocus();
    input.props.onChangeText('-0.25');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.onBlur();
  });

  expect(mockDispatchIntent).toHaveBeenCalled();
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('set_effect_param');
  expect(payload.value).toBe(0);
  expect(tree!.root.findByProps({testID: 'inspector-effect-param-input-9-amount'}).props.value).toBe(
    '0',
  );

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});
