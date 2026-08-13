/**
 * @format
 */

import React from 'react';
import ReactTestRenderer from 'react-test-renderer';
import { Text, TurboModuleRegistry } from 'react-native';
import App from '../App';
import { dispatchHostIntentResult } from '../src/host';

const mockDispatchIntent = jest.fn<string, [string]>(() => '{"accepted":true}');
const mockReadSnapshot = jest.fn(() => '');
const mockIsTimelineInteracting = jest.fn(() => false);
const mockFitStageView = jest.fn(() => true);
const mockStageViewOneToOne = jest.fn(() => true);
const mockPreviewStageTransform = jest.fn(() => '{"accepted":true}');
const mockCommitStageTransformGesture = jest.fn(() => '{"accepted":true}');
const mockCancelStageTransform = jest.fn(() => '{"accepted":true}');
// 0=未束縛。消費成功を返してショートカット製品完了に読ませない。
const mockHostKeyEvent = jest.fn(() => 0);
const actualGet = TurboModuleRegistry.get.bind(TurboModuleRegistry);

function mockHostGet(name: string) {
  if (name === 'NativeMotoliiHost') {
    return {
      dispatchIntent: mockDispatchIntent,
      readSnapshot: mockReadSnapshot,
      isTimelineInteracting: mockIsTimelineInteracting,
      fitStageView: mockFitStageView,
      stageViewOneToOne: mockStageViewOneToOne,
      previewStageTransform: mockPreviewStageTransform,
      commitStageTransformGesture: mockCommitStageTransformGesture,
      cancelStageTransform: mockCancelStageTransform,
      hostKeyEvent: mockHostKeyEvent,
    };
  }
  return actualGet(name);
}

function intentPayloads(): Array<{
  kind: string;
  value?: number;
  output_path?: string;
}> {
  return mockDispatchIntent.mock.calls.map(call =>
    JSON.parse(call[0] as string),
  );
}

function lastIntent(
  kind: string,
): { kind: string; value?: number; output_path?: string } | undefined {
  return intentPayloads()
    .filter(payload => payload.kind === kind)
    .at(-1);
}

function collectText(node: ReactTestRenderer.ReactTestInstance): string[] {
  return node.findAllByType(Text).map(item => {
    const children = item.props.children;
    if (Array.isArray(children)) {
      return children.join('');
    }
    return children == null ? '' : String(children);
  });
}

function mediaCardTestIDs(tree: ReactTestRenderer.ReactTestRenderer): string[] {
  return [
    ...new Set(
      tree.root
        .findAll(
          node =>
            typeof node.props.testID === 'string' &&
            (node.props.testID as string).startsWith('media-item-'),
        )
        .map(node => node.props.testID as string),
    ),
  ].sort();
}

function mediaResultCount(tree: ReactTestRenderer.ReactTestRenderer): number {
  const texts = collectText(
    tree.root.findByProps({ testID: 'thumbnail-grid' }),
  );
  const resultsIndex = texts.indexOf('Results');
  if (resultsIndex < 0) {
    throw new Error(`Results header missing: ${texts.join('|')}`);
  }
  return Number(texts[resultsIndex + 1]);
}

function inspectorSurfaceText(
  tree: ReactTestRenderer.ReactTestRenderer,
): string {
  return collectText(
    tree.root.findByProps({ testID: 'inspector-surface' }),
  ).join('\n');
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

  expect(tree!.root.findByProps({ testID: 'motolii-rn-shell' })).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'timeline' })).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'browser-surface' })).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'browser-view-EFFECTS' }),
  ).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'rust-wgpu-timeline' })).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'native-timeline-feedback' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'native-timeline-feedback' }).props
      .children,
  ).toBe('00:00.0');
  expect(tree!.root.findByProps({ testID: 'timeline-key-tools' })).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'timeline-add-position-key' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'timeline-key-tools-hint' }).props
      .children,
  ).toBe('Select a layer to edit keys');
  expect(
    tree!.root.findByProps({ testID: 'timeline-add-position-key' }).props
      .accessibilityState,
  ).toEqual({ disabled: true });
  expect(
    tree!.root.findAllByProps({
      accessibilityLabel: 'Select previous native clip',
    }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({
      accessibilityLabel: 'Select next native clip',
    }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'timeline-mode-PACKING' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'timeline-mode-DENSITY' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'timeline-mode-NATIVE' }),
  ).toHaveLength(0);
  expect(tree!.root.findByProps({ testID: 'inspector-surface' })).toBeTruthy();
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-layer-section' }),
  ).toHaveLength(0);
  expect(tree!.root.findByProps({ testID: 'inspector-empty' })).toBeTruthy();
  expect(
    tree!.root.findAllByProps({ testID: 'path-operations-panel' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'stage-transform-projection' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ accessibilityLabel: 'Inspector note' }),
  ).toHaveLength(0);
  expect(inspectorSurfaceText(tree!)).not.toContain('Echo Bloom');
  expect(inspectorSurfaceText(tree!)).toContain('未選択');
  expect(
    tree!.root.findByProps({ testID: 'command-breadcrumb' }).props.children,
  ).toBe('未選択');
  expect(
    tree!.root.findByProps({ testID: 'stage-identity' }).props.children,
  ).toBe('STAGE');
  expect(tree!.root.findByProps({ testID: 'transport-play' })).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'transport-skip-start' }),
  ).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'transport-skip-end' })).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'rust-wgpu-timeline' })
      .props.onTimelineFeedback({
        nativeEvent: { objectIndex: -1, time: 0.27 },
      });
  });
  expect(
    tree!.root.findByProps({ testID: 'native-timeline-feedback' }).props
      .children,
  ).toBe('00:00.0');

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'browser-tab-MEDIA' }).props.onPress();
    tree!.root
      .findByProps({ testID: 'right-panel-EXTENSIONS' })
      .props.onPress();
  });

  expect(tree!.root.findByProps({ testID: 'thumbnail-grid' })).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'browser-view-MEDIA' })).toBeTruthy();
  expect(
    tree!.root.findAllByProps({ testID: 'media-item-asset-0' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ children: 'DIRECTORIES' }).length,
  ).toBeGreaterThan(0);
  expect(
    tree!.root.findAllByProps({ children: 'TAGS' }).length,
  ).toBeGreaterThan(0);
  expect(tree!.root.findAllByProps({ children: '▣  Video' })).toHaveLength(0);
  expect(
    tree!.root.findByProps({ testID: 'extension-panel-asset-tags' }),
  ).toBeTruthy();

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'timeline-add-position-key' })
      .props.onPress();
  });
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'browser-tab-CREATE' }).props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'browser-view-CREATE' }),
  ).toBeTruthy();
  const rectangle = tree!.root.findByProps({ testID: 'create-item-rectangle' });
  expect(rectangle.props.accessibilityState.selected).toBe(false);

  await ReactTestRenderer.act(() => {
    rectangle.props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'create-item-rectangle' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.createdItemId,
  ).toBe('');

  await ReactTestRenderer.act(() => {
    rectangle.props.onPointerDown();
  });
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.draggedItemId,
  ).toBe('rectangle');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.onStageDrop({
      nativeEvent: {
        x: 0.25,
        y: 0.75,
        canonicalX: 0.333333,
        canonicalY: -0.777777,
      },
    });
  });
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.createdItemId,
  ).toBe('');
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.draggedItemId,
  ).toBe('');
  expect(mockDispatchIntent).toHaveBeenCalled();
  const placePayload = JSON.parse(
    mockDispatchIntent.mock.calls[0][0] as string,
  );
  expect(placePayload.kind).toBe('place_rectangle');
  expect(placePayload.position).toEqual([0.333333, -0.777777]);
  expect(placePayload.playhead).toEqual({ num: 0, den: 1 });

  mockDispatchIntent.mockClear();
  // F3: 履歴ありsnapshotを即時適用してからUndoを撃つ。
  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: true,
      snapshot: {
        revision: '1',
        projection_generation: '1',
        current_time: { num: 0, den: 1 },
        history: { can_undo: true, can_redo: false },
        truncated_total: 0,
        stage: { bounds: [] },
        timeline: { layers: [], layers_truncated: false },
      },
    }),
  );
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'create-item-rectangle' })
      .props.onDoubleClick();
  });
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'titlebar-undo' }).props.onPress();
  });
  expect(mockDispatchIntent).toHaveBeenCalled();
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string).kind).toBe(
    'undo',
  );

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'create-item-rectangle' })
      .props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const centerPayload = JSON.parse(
    mockDispatchIntent.mock.calls[0][0] as string,
  );
  expect(centerPayload.kind).toBe('place_rectangle');
  expect(centerPayload.position).toEqual([0, 0]);
  expect(centerPayload.playhead).toEqual({ num: 0, den: 1 });
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.createdItemId,
  ).toBe('');

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'browser-mode-THUMBNAILS' })
      .props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'browser-results-CREATE' }).props
      .numColumns,
  ).toBe(4);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'browser-mode-LIST' }).props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'browser-results-CREATE' }).props
      .numColumns,
  ).toBe(1);

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
      current_time: { num: 2, den: 1 },
      stage: {
        selection: [],
        bounds: [
          { layer_id: '10', display_name: 'seed' },
          { layer_id: '11', display_name: 'other' },
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
    tree!.root.findByProps({ testID: 'browser-tab-CREATE' }).props.onPress();
  });
  const rectangle = tree!.root.findByProps({ testID: 'create-item-rectangle' });
  await ReactTestRenderer.act(() => {
    rectangle.props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const centerPayload = JSON.parse(
    mockDispatchIntent.mock.calls[0][0] as string,
  );
  expect(centerPayload.kind).toBe('place_rectangle');
  expect(centerPayload.position).toEqual([0, 0]);
  expect(centerPayload.playhead).toEqual({ num: 2, den: 1 });

  await ReactTestRenderer.act(() => {
    rectangle.props.onPointerDown();
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.onStageDrop({
      nativeEvent: { x: 0.25, y: 0.75, canonicalX: 0.5, canonicalY: 0.25 },
    });
  });

  expect(mockDispatchIntent).toHaveBeenCalledTimes(2);
  const placePayload = JSON.parse(
    mockDispatchIntent.mock.calls[1][0] as string,
  );
  expect(placePayload.kind).toBe('place_rectangle');
  expect(placePayload.playhead).toEqual({ num: 2, den: 1 });

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('Fit and 100% call the native Spatial camera', async () => {
  mockFitStageView.mockClear();
  mockStageViewOneToOne.mockClear();
  mockReadSnapshot.mockReturnValue('');
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  expect(
    tree!.root.findByProps({ accessibilityLabel: 'Fit Stage' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ accessibilityLabel: 'Stage 100 percent' }),
  ).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'stage-surface' })).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ accessibilityLabel: 'Fit Stage' }).props.onPress();
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ accessibilityLabel: 'Stage 100 percent' })
      .props.onPress();
  });

  expect(mockFitStageView).toHaveBeenCalledTimes(1);
  expect(mockStageViewOneToOne).toHaveBeenCalledTimes(1);
  expect(
    mockDispatchIntent.mock.calls.map(
      call => JSON.parse(call[0] as string).kind,
    ),
  ).not.toContain('fit_stage_view');

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
  expect(tree!.root.findByProps({ testID: 'motolii-rn-shell' })).toBeTruthy();
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-layer-section' }),
  ).toHaveLength(0);
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
      current_time: { num: 2, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [{ key_id: '7', time: { num: 0, den: 1 } }],
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-layer-section' }),
  ).toBeTruthy();
  const layerTexts = tree!.root
    .findByProps({ testID: 'inspector-layer-section' })
    .findAllByType(Text)
    .map(node => {
      const children = node.props.children;
      return Array.isArray(children) ? children.join('') : String(children);
    });
  expect(layerTexts).toContain('seed-layer');
  expect(layerTexts).toContain('position keys: 1');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-add-key-position' })
      .props.onPress();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('add_position_key');
  expect(payload.target).toBe('42');
  expect(payload.time).toEqual({ num: 2, den: 1 });
  expect(payload.property).toBeUndefined();
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-add-position-key' }),
  ).toHaveLength(0);

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-add-key-scale' })
      .props.onPress();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'add_param_key',
    target: '42',
    time: { num: 2, den: 1 },
    property: 'scale',
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'timeline-add-position-key' })
      .props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'add_position_key',
    target: '42',
    time: { num: 2, den: 1 },
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'inspector-delete-clip' }).props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'delete_layer',
    target: '42',
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-duplicate-clip' })
      .props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'duplicate',
    target: '42',
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'inspector-split-clip' }).props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'split',
    target: '42',
    time: { num: 2, den: 1 },
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'inspector-mute-clip' }).props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'mute',
    target: '42',
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'inspector-solo-clip' }).props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'solo',
    target: '42',
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'timeline-split-clip' }).props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'split',
    target: '42',
    time: { num: 2, den: 1 },
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'command-tool-place-rectangle' })
      .props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'place_rectangle',
    position: [0, 0],
    playhead: { num: 2, den: 1 },
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'command-tool-place-ellipse' })
      .props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'place_ellipse',
    position: [0, 0],
    playhead: { num: 2, den: 1 },
  });

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector is empty without bloom fixture when no layer is selected', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 2, den: 1 },
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
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

  expect(tree!.root.findByProps({ testID: 'inspector-empty' })).toBeTruthy();
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-layer-section' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-add-position-key' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-add-key-position' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-add-key-scale' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-add-key-rotation' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-add-key-opacity' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-delete-clip' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-duplicate-clip' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-split-clip' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-mute-clip' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-solo-clip' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-effects-section' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'stage-transform-projection' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'path-operations-panel' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ accessibilityLabel: 'Inspector note' }),
  ).toHaveLength(0);
  const inspectorText = inspectorSurfaceText(tree!);
  expect(inspectorText).toContain('未選択');
  expect(inspectorText).not.toContain('Echo Bloom');
  expect(inspectorText).not.toContain('Pulse rings');
  expect(inspectorText).not.toContain('Lottie');
  expect(inspectorText).not.toContain('日本語IME');
  expect(
    tree!.root.findByProps({ testID: 'command-breadcrumb' }).props.children,
  ).toBe('未選択');
  expect(
    tree!.root.findByProps({ testID: 'stage-identity' }).props.children,
  ).toBe('STAGE');
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
      current_time: { num: 2, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: '7', time: { num: 4, den: 2 }, value: [0.25, -0.5] },
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-y' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }).props.value,
  ).toBe('0.25');
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-y' }).props.value,
  ).toBe('-0.5');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });

  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 2, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: '7', time: { num: 0, den: 1 }, value: [0.25, -0.5] },
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
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-position-key-x' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-position-key-y' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-layer-section' }),
  ).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector exact-on-key editor dispatches remove and interp', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 2, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: '7', time: { num: 2, den: 1 }, value: [1, 2] },
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

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-remove-position-key' })
      .props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'remove_position_key',
    target: '42',
    key_id: '7',
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-key-interp-hold' })
      .props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'set_position_key_interp',
    target: '42',
    time: { num: 2, den: 1 },
    interp: 'Hold',
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-key-interp-linear' })
      .props.onPress();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'set_position_key_interp',
    target: '42',
    time: { num: 2, den: 1 },
    interp: 'Linear',
  });

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector on-key scale rotation opacity commit set_param_key_value', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 1, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            param_keys: [
              {
                property: 'scale',
                key_id: '11',
                time: { num: 1, den: 1 },
                vec: [1.5, 0.5],
              },
              {
                property: 'rotation',
                key_id: '12',
                time: { num: 1, den: 1 },
                value: Math.PI / 2,
              },
              {
                property: 'opacity',
                key_id: '13',
                time: { num: 1, den: 1 },
                value: 0.4,
              },
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-scale-key-x' }).props.value,
  ).toBe('1.5');
  expect(
    tree!.root.findByProps({ testID: 'inspector-scale-key-y' }).props.value,
  ).toBe('0.5');
  expect(
    tree!.root.findByProps({ testID: 'inspector-rotation-key' }).props.value,
  ).toBe('90');
  expect(
    tree!.root.findByProps({ testID: 'inspector-opacity-key' }).props.value,
  ).toBe('0.4');

  const scaleX = tree!.root.findByProps({ testID: 'inspector-scale-key-x' });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    scaleX.props.onChangeText('2');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-scale-key-x' })
      .props.onSubmitEditing();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'set_param_key_value',
    target: '42',
    key_id: '11',
    property: 'scale',
    new: [2, 0.5],
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'inspector-scale-key-x' }).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-rotation-key' })
      .props.onChangeText('180');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-rotation-key' })
      .props.onSubmitEditing();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const rotationPayload = JSON.parse(
    mockDispatchIntent.mock.calls[0][0] as string,
  );
  expect(rotationPayload).toMatchObject({
    kind: 'set_param_key_value',
    target: '42',
    key_id: '12',
    property: 'rotation',
  });
  expect(rotationPayload.value).toBeCloseTo(Math.PI);
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'inspector-rotation-key' }).props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-opacity-key' })
      .props.onChangeText('0.2');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'inspector-opacity-key' }).props.onBlur();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'set_param_key_value',
    target: '42',
    key_id: '13',
    property: 'opacity',
    value: 0.2,
  });

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
      current_time: { num: 1, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: '7', time: { num: 1, den: 1 }, value: [0.1, 0.2] },
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

  const xInput = tree!.root.findByProps({ testID: 'inspector-position-key-x' });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    xInput.props.onChangeText('0.75');
  });
  await ReactTestRenderer.act(() => {
    const committed = tree!.root.findByProps({
      testID: 'inspector-position-key-x',
    });
    committed.props.onSubmitEditing();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('set_position_key_value');
  expect(payload.target).toBe('42');
  expect(payload.time).toEqual({ num: 1, den: 1 });
  expect(payload.new).toEqual([0.75, 0.2]);
  await ReactTestRenderer.act(() => {
    const committed = tree!.root.findByProps({
      testID: 'inspector-position-key-x',
    });
    committed.props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector hides param key editors when playhead misses the key', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 2, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            param_keys: [
              {
                property: 'scale',
                key_id: '11',
                time: { num: 1, den: 1 },
                vec: [1.5, 0.5],
              },
              {
                property: 'rotation',
                key_id: '12',
                time: { num: 1, den: 1 },
                value: 0.5,
              },
              {
                property: 'opacity',
                key_id: '13',
                time: { num: 1, den: 1 },
                value: 0.4,
              },
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
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-scale-key-x' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-rotation-key' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-opacity-key' }),
  ).toHaveLength(0);

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
    current_time: { num: 1, den: 1 },
    primary_layer_id: '42',
    stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
    timeline: {
      fps: { num: 30, den: 1 },
      layers: [
        {
          layer_id: '42',
          display_name: 'seed-layer',
          start: { num: 0, den: 1 },
          duration: { num: 10, den: 1 },
          position_keys: [
            { key_id: 'a', time: { num: 1, den: 1 }, value: [0.1, 0.2] },
          ],
          keys_truncated: false,
        },
      ],
      layers_truncated: false,
    },
  };
  const snapshotB = {
    revision: '3',
    projection_generation: '1',
    current_time: { num: 1, den: 1 },
    primary_layer_id: '43',
    stage: { bounds: [{ layer_id: '43', display_name: 'other-layer' }] },
    timeline: {
      fps: { num: 30, den: 1 },
      layers: [
        {
          layer_id: '43',
          display_name: 'other-layer',
          start: { num: 0, den: 1 },
          duration: { num: 10, den: 1 },
          position_keys: [
            { key_id: 'b', time: { num: 1, den: 1 }, value: [0.5, 0.75] },
          ],
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

  const staleInput = tree!.root.findByProps({
    testID: 'inspector-position-key-x',
  });
  await ReactTestRenderer.act(() => {
    staleInput.props.onChangeText('0.75');
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });

  const activeInput = tree!.root.findByProps({
    testID: 'inspector-position-key-x',
  });
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
      current_time: { num: 1, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: '7', time: { num: 1, den: 1 }, value: [0.1, 0.2] },
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

  const xInput = tree!.root.findByProps({ testID: 'inspector-position-key-x' });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    xInput.props.onChangeText('  ');
  });
  await ReactTestRenderer.act(() => {
    const committed = tree!.root.findByProps({
      testID: 'inspector-position-key-x',
    });
    committed.props.onSubmitEditing();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }).props.value,
  ).toBe('0.1');
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-position-key-x' })
      .props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }).props.value,
  ).toBe('0.1');

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
      current_time: { num: 1, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: '7', time: { num: 1, den: 1 }, value: [0.1, 0.2] },
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

  const xInput = tree!.root.findByProps({ testID: 'inspector-position-key-x' });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    xInput.props.onChangeText('abc');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-position-key-x' })
      .props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-position-key-x' })
      .props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }).props.value,
  ).toBe('0.1');

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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
          { plugin_id: 'core.param.sine', name: 'Sine', effect_version: 2 },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
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

  expect(
    tree!.root.findByProps({ testID: 'effect-item-core.filter.opacity' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'effect-item-core.param.sine' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'effect-item-core.filter.opacity' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(
    tree!.root.findByProps({ testID: 'effect-item-core.param.sine' }).props
      .accessibilityState.selected,
  ).toBe(false);
  expect(
    tree!.root.findAllByProps({ testID: 'effect-item-echo' }),
  ).toHaveLength(0);

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'effect-item-core.filter.opacity' })
      .props.onDoubleClick();
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

test('EFFECTS attach without primary dispatches for host reject; empty catalog shows no bloom fixture', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      stage: { bounds: [] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
      },
      timeline: { fps: { num: 30, den: 1 }, layers: [] },
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
    tree!.root
      .findByProps({ testID: 'effect-item-core.filter.opacity' })
      .props.onDoubleClick();
  });
  expect(
    JSON.parse(mockDispatchIntent.mock.calls[0][0] as string),
  ).toMatchObject({
    kind: 'attach_effect',
    target: null,
    plugin_id: 'core.filter.opacity',
  });

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();

  mockReadSnapshot.mockReturnValue('');
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });
  const names = tree!.root
    .findByProps({ testID: 'browser-view-EFFECTS' })
    .findAllByType(Text)
    .map(node => {
      const children = node.props.children;
      return Array.isArray(children) ? children.join('') : String(children);
    });
  expect(names).not.toContain('Echo Bloom');
  expect(names).not.toContain('Type Pulse');
  expect(
    tree!.root.findAllByProps({ testID: 'effect-item-echo' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'effect-item-core.filter.opacity' }),
  ).toHaveLength(0);

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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{ param_id: 'amount', value: 1 }],
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-effects-section' }),
  ).toBeTruthy();
  const input = tree!.root.findByProps({
    testID: 'inspector-effect-param-input-9-amount',
  });
  expect(input.props.value).toBe('1');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('0.4');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onSubmitEditing();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_effect_param',
    'set_effect_param',
  ]);
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onBlur();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_effect_param',
    'set_effect_param',
  ]);
  const payload = lastIntent('set_effect_param') as {
    kind: string;
    target: string;
    effect_use_id: string;
    param_id: string;
    value: number;
  };
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

test('inspector shows selected layer opacity and gizmo transform without bloom fixture', async () => {
  mockDispatchIntent.mockClear();
  mockPreviewStageTransform.mockClear();
  mockCommitStageTransformGesture.mockClear();
  mockCancelStageTransform.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
      },
      selected_doc_params: {
        layer_id: '42',
        opacity: 1.0,
        effects: [
          {
            effect_use_id: '9',
            plugin_id: 'core.filter.opacity',
            params: [{ param_id: 'amount', value: 1 }],
          },
        ],
        source_params: [],
      },
      stage_geometry: {
        layers: [
          {
            layer_id: '42',
            position: [0.25, -0.5],
            rotation: (30 * Math.PI) / 180,
            scale: [1, 1],
          },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{ param_id: 'amount', value: 1 }],
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

  expect(tree!.root.findAllByProps({ testID: 'inspector-empty' })).toHaveLength(
    0,
  );
  expect(
    collectText(tree!.root.findByProps({ testID: 'inspector-layer-opacity' })),
  ).toContain('1');
  expect(
    tree!.root.findByProps({ testID: 'inspector-effects-section' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1');
  expect(
    collectText(
      tree!.root.findByProps({ testID: 'inspector-effects-section' }),
    ),
  ).toContain('Opacity');
  expect(
    tree!.root.findByProps({ testID: 'stage-transform-projection' }),
  ).toBeTruthy();
  const inspectorText = inspectorSurfaceText(tree!);
  expect(inspectorText).toContain('seed-layer');
  expect(inspectorText).not.toContain('Echo Bloom');
  expect(inspectorText).not.toContain('Pulse rings');
  expect(inspectorText).not.toContain('Lottie');
  expect(inspectorText).not.toContain('日本語IME');
  expect(
    tree!.root.findAllByProps({ testID: 'path-operations-panel' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ accessibilityLabel: 'Inspector note' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findByProps({ testID: 'command-breadcrumb' }).props.children,
  ).toBe('seed-layer');
  expect(
    tree!.root.findByProps({ testID: 'stage-identity' }).props.children,
  ).toBe('STAGE');

  expect(tree!.root.findByProps({ value: '0.250' })).toBeTruthy();
  expect(tree!.root.findByProps({ value: '30.0' })).toBeTruthy();

  const mockCommitStageTransform = jest.fn(() => true);
  getSpy.mockImplementation(((name: string) => {
    if (name === 'NativeMotoliiHost') {
      return {
        ...(mockHostGet(name) as object),
        commitStageTransform: mockCommitStageTransform,
      };
    }
    return mockHostGet(name);
  }) as typeof TurboModuleRegistry.get);

  const snapshotAfterRotationCommit = JSON.parse(mockReadSnapshot()) as {
    stage_geometry: { layers: Array<{ rotation: number }> };
  };
  snapshotAfterRotationCommit.stage_geometry.layers[0].rotation =
    (25 * Math.PI) / 180;
  mockReadSnapshot.mockReturnValue(JSON.stringify(snapshotAfterRotationCommit));

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ accessibilityLabel: 'Rotation Z value' })
      .props.onChangeText('25');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ accessibilityLabel: 'Rotation Z value' })
      .props.onSubmitEditing();
  });
  expect(mockCommitStageTransform).toHaveBeenCalledWith(
    '42',
    '3',
    1,
    expect.closeTo(-5 * (Math.PI / 180)),
    0,
  );

  mockDispatchIntent.mockClear();
  const positionX = tree!.root.findByProps({
    accessibilityLabel: 'Position X dial',
  });
  await ReactTestRenderer.act(() => {
    positionX.props.onTouchStart({ nativeEvent: { pageX: 100 } });
    positionX.props.onTouchMove({ nativeEvent: { pageX: 110 } });
  });
  expect(mockPreviewStageTransform).toHaveBeenLastCalledWith(
    '42',
    '3',
    0,
    expect.closeTo(0.1),
    0,
  );
  expect(mockCommitStageTransformGesture).not.toHaveBeenCalled();
  expect(mockDispatchIntent).not.toHaveBeenCalled();
  await ReactTestRenderer.act(() => {
    positionX.props.onTouchEnd();
    positionX.props.onTouchEnd();
  });
  expect(mockCommitStageTransformGesture).toHaveBeenCalledTimes(1);
  expect(mockCommitStageTransformGesture).toHaveBeenLastCalledWith(
    '42',
    '3',
    0,
    expect.closeTo(0.1),
    0,
  );

  const rotation = tree!.root.findByProps({
    accessibilityLabel: 'Rotation Z dial',
  });
  await ReactTestRenderer.act(() => {
    rotation.props.onTouchStart({ nativeEvent: { pageX: 100 } });
    rotation.props.onTouchMove({ nativeEvent: { pageX: 105 } });
  });
  expect(mockPreviewStageTransform).toHaveBeenLastCalledWith(
    '42',
    '3',
    1,
    expect.closeTo(5 * (Math.PI / 180)),
    0,
  );
  await ReactTestRenderer.act(() => {
    rotation.props.onTouchCancel();
  });
  expect(mockCancelStageTransform).toHaveBeenCalledTimes(1);
  expect(mockCommitStageTransformGesture).toHaveBeenCalledTimes(1);

  const scaleX = tree!.root.findByProps({ accessibilityLabel: 'Scale X dial' });
  await ReactTestRenderer.act(() => {
    scaleX.props.onTouchStart({ nativeEvent: { pageX: 100 } });
    scaleX.props.onTouchMove({ nativeEvent: { pageX: 110 } });
    scaleX.props.onTouchEnd();
  });
  expect(mockPreviewStageTransform).toHaveBeenLastCalledWith(
    '42',
    '3',
    2,
    expect.closeTo(1.1),
    1,
  );
  expect(mockCommitStageTransformGesture).toHaveBeenCalledTimes(2);
  expect(mockCommitStageTransformGesture).toHaveBeenLastCalledWith(
    '42',
    '3',
    2,
    expect.closeTo(1.1),
    1,
  );
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  mockCancelStageTransform.mockClear();
  mockCommitStageTransformGesture.mockClear();
  mockDispatchIntent.mockClear();
  const positionY = tree!.root.findByProps({
    accessibilityLabel: 'Position Y dial',
  });
  await ReactTestRenderer.act(() => {
    positionY.props.onTouchStart({ nativeEvent: { pageX: 100 } });
    positionY.props.onTouchMove({ nativeEvent: { pageX: 110 } });
    positionY.props.onTouchMove({ nativeEvent: { pageX: 100 } });
    positionY.props.onTouchEnd();
    positionY.props.onTouchEnd();
  });
  expect(mockCancelStageTransform).toHaveBeenCalledTimes(1);
  expect(mockCommitStageTransformGesture).not.toHaveBeenCalled();
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  const scaleY = tree!.root.findByProps({ accessibilityLabel: 'Scale Y dial' });
  await ReactTestRenderer.act(() => {
    scaleY.props.onTouchStart({ nativeEvent: { pageX: 100 } });
    scaleY.props.onTouchMove({ nativeEvent: { pageX: 110 } });
    scaleY.props.onTouchMove({ nativeEvent: { pageX: 100 } });
    scaleY.props.onResponderTerminate();
    scaleY.props.onResponderTerminate();
  });
  expect(mockCancelStageTransform).toHaveBeenCalledTimes(2);
  expect(mockCommitStageTransformGesture).not.toHaveBeenCalled();
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  mockPreviewStageTransform.mockReturnValueOnce(
    '{"accepted":false,"message":"stale transform revision"}',
  );
  await ReactTestRenderer.act(() => {
    rotation.props.onTouchStart({ nativeEvent: { pageX: 100 } });
    rotation.props.onTouchMove({ nativeEvent: { pageX: 105 } });
    rotation.props.onTouchCancel();
  });
  expect(mockCancelStageTransform).toHaveBeenCalledTimes(2);
  expect(mockCommitStageTransformGesture).not.toHaveBeenCalled();
  expect(mockDispatchIntent).not.toHaveBeenCalled();
  expect(collectText(tree!.root)).toContain('stale transform revision');

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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{ param_id: 'amount', value: 1 }],
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

  const input = tree!.root.findByProps({
    testID: 'inspector-effect-param-input-9-amount',
  });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('   ');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onSubmitEditing();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1');
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1');

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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{ param_id: 'amount', value: 1 }],
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

  const input = tree!.root.findByProps({
    testID: 'inspector-effect-param-input-9-amount',
  });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('abc');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onSubmitEditing();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1');
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(0);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1');

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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{ param_id: 'amount', value: 1 }],
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

  const input = tree!.root.findByProps({
    testID: 'inspector-effect-param-input-9-amount',
  });
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('0.4');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onSubmitEditing();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_effect_param',
    'set_effect_param',
  ]);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1');
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onBlur();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_effect_param',
    'set_effect_param',
  ]);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1');

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
      current_time: { num: 3, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
        sources: [
          {
            plugin_id: 'core.layer_source.radial_repeater',
            name: 'Radial Repeater',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
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
    tree!.root.findByProps({ testID: 'browser-tab-CREATE' }).props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'create-item-rectangle' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({
      testID: 'create-item-core.layer_source.radial_repeater',
    }),
  ).toBeTruthy();

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'create-item-core.layer_source.radial_repeater' })
      .props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const payload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(payload.kind).toBe('place_vism');
  expect(payload.plugin_id).toBe('core.layer_source.radial_repeater');
  expect(payload.position).toEqual([0, 0]);
  expect(payload.playhead).toEqual({ num: 3, den: 1 });

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'create-item-core.layer_source.radial_repeater' })
      .props.onPointerDown();
  });
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.draggedItemId,
  ).toBe('');

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'create-item-rectangle' })
      .props.onPointerDown();
  });
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.draggedItemId,
  ).toBe('rectangle');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('CREATE rectangle double-click applies placed Document identity', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '0',
      projection_generation: '0',
      current_time: { num: 0, den: 1 },
      history: { can_undo: false, can_redo: false },
      truncated_total: 0,
      stage: { bounds: [] },
      catalog: { effects: [], sources: [] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [],
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
    tree!.root.findByProps({ testID: 'browser-tab-CREATE' }).props.onPress();
  });

  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: true,
      snapshot: {
        revision: '1',
        projection_generation: '1',
        current_time: { num: 0, den: 1 },
        primary_layer_id: '99',
        history: { can_undo: true, can_redo: false },
        truncated_total: 0,
        stage: { bounds: [{ layer_id: '99', display_name: 'Rectangle' }] },
        catalog: { effects: [], sources: [] },
        timeline: {
          fps: { num: 30, den: 1 },
          layers: [
            {
              layer_id: '99',
              display_name: 'Rectangle',
              start: { num: 0, den: 1 },
              duration: { num: 10, den: 1 },
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
      },
    }),
  );

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'create-item-rectangle' })
      .props.onDoubleClick();
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string).kind).toBe(
    'place_rectangle',
  );
  expect(
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r1 · 1 layers'),
  ).toBe(true);
  expect(
    tree!.root
      .findByProps({ testID: 'inspector-layer-section' })
      .findAllByType(Text)
      .some(node => node.props.children === 'Rectangle'),
  ).toBe(true);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue('');
});

test('CREATE ellipse double-click applies placed Document identity', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '0',
      projection_generation: '0',
      current_time: { num: 0, den: 1 },
      history: { can_undo: false, can_redo: false },
      truncated_total: 0,
      stage: { bounds: [] },
      catalog: { effects: [], sources: [] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [],
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
    tree!.root.findByProps({ testID: 'browser-tab-CREATE' }).props.onPress();
  });

  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: true,
      snapshot: {
        revision: '1',
        projection_generation: '1',
        current_time: { num: 0, den: 1 },
        primary_layer_id: '99',
        history: { can_undo: true, can_redo: false },
        truncated_total: 0,
        stage: { bounds: [{ layer_id: '99', display_name: 'Ellipse' }] },
        catalog: { effects: [], sources: [] },
        timeline: {
          fps: { num: 30, den: 1 },
          layers: [
            {
              layer_id: '99',
              display_name: 'Ellipse',
              start: { num: 0, den: 1 },
              duration: { num: 10, den: 1 },
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
      },
    }),
  );

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'create-item-ellipse' })
      .props.onDoubleClick();
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string).kind).toBe(
    'place_ellipse',
  );
  expect(
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r1 · 1 layers'),
  ).toBe(true);
  expect(
    tree!.root
      .findByProps({ testID: 'inspector-layer-section' })
      .findAllByType(Text)
      .some(node => node.props.children === 'Ellipse'),
  ).toBe(true);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
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
    tree!.root.findByProps({ testID: 'browser-tab-CREATE' }).props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'create-item-rectangle' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'create-item-ellipse' }),
  ).toBeTruthy();
  expect(
    tree!.root.findAllByProps({
      testID: 'create-item-core.layer_source.radial_repeater',
    }),
  ).toHaveLength(0);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('inspector source param commit dispatches set_source_param once', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'Radial Repeater' }] },
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
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'Radial Repeater',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
            source_params: [
              { param_id: 'count', value: 12 },
              { param_id: 'radius', value: 0.3 },
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-source-params-section' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'inspector-source-param-count' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'inspector-source-param-radius' }),
  ).toBeTruthy();
  const input = tree!.root.findByProps({
    testID: 'inspector-source-param-input-count',
  });
  expect(input.props.value).toBe('12');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('8');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-source-param-input-count' })
      .props.onSubmitEditing();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_source_param',
    'set_source_param',
  ]);
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-source-param-input-count' })
      .props.onBlur();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_source_param',
    'set_source_param',
  ]);
  const payload = lastIntent('set_source_param') as {
    kind: string;
    target: string;
    param_id: string;
    value: number;
  };
  expect(payload.kind).toBe('set_source_param');
  expect(payload.target).toBe('42');
  expect(payload.param_id).toBe('count');
  expect(payload.value).toBe(8);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector color source param commit dispatches set_source_param with color array once', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'Radial Repeater' }] },
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
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'Radial Repeater',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
            source_params: [
              { param_id: 'count', value: 12 },
              { param_id: 'color', value: 0, color: [1, 1, 1, 1] },
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-source-params-section' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'inspector-source-param-input-count' }),
  ).toBeTruthy();
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-source-param-input-color' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-source-color-count-r' }),
  ).toHaveLength(0);
  const input = tree!.root.findByProps({
    testID: 'inspector-source-color-input-color-r',
  });
  expect(input.props.value).toBe('1');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('0.2');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-source-color-input-color-r' })
      .props.onSubmitEditing();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_source_param',
    'set_source_param',
  ]);
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-source-color-input-color-r' })
      .props.onBlur();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_source_param',
    'set_source_param',
  ]);
  const payload = lastIntent('set_source_param') as {
    kind: string;
    target: string;
    param_id: string;
    color?: [number, number, number, number];
    value?: number;
  };
  expect(payload.kind).toBe('set_source_param');
  expect(payload.target).toBe('42');
  expect(payload.param_id).toBe('color');
  expect(payload.color).toEqual([0.2, 1, 1, 1]);
  expect(payload.value).toBeUndefined();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('inspector color effect param commit dispatches set_effect_param with color array once', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
        ],
      },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [
                  { param_id: 'amount', value: 1 },
                  { param_id: 'color', value: 0, color: [1, 1, 1, 1] },
                ],
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-effects-section' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' }),
  ).toBeTruthy();
  expect(
    tree!.root.findAllByProps({
      testID: 'inspector-effect-param-input-9-color',
    }),
  ).toHaveLength(0);
  const input = tree!.root.findByProps({
    testID: 'inspector-effect-color-input-9-color-r',
  });
  expect(input.props.value).toBe('1');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    input.props.onChangeText('0.2');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-color-input-9-color-r' })
      .props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-color-input-9-color-r' })
      .props.onBlur();
  });
  expect(intentPayloads().map(payload => payload.kind)).toEqual([
    'preview_effect_param',
    'set_effect_param',
  ]);
  const payload = lastIntent('set_effect_param') as {
    kind: string;
    target: string;
    effect_use_id: string;
    param_id: string;
    color?: [number, number, number, number];
    value?: number;
  };
  expect(payload.kind).toBe('set_effect_param');
  expect(payload.target).toBe('42');
  expect(payload.effect_use_id).toBe('9');
  expect(payload.param_id).toBe('color');
  expect(payload.color).toEqual([0.2, 1, 1, 1]);
  expect(payload.value).toBeUndefined();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('dispatch accepted same-revision higher-generation snapshot applies immediately', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      host_handle: '7',
      revision: '9',
      projection_generation: '0',
      current_time: { num: 0, den: 1 },
      history: { can_undo: true, can_redo: false },
      truncated_total: 0,
      stage: { bounds: [] },
      timeline: { layers: [], layers_truncated: false },
    }),
  );
  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: true,
      snapshot: {
        host_handle: '7',
        revision: '9',
        projection_generation: '2',
        current_time: { num: 3, den: 1 },
        primary_layer_id: '42',
        history: { can_undo: true, can_redo: false },
        truncated_total: 0,
        stage: { bounds: [{ layer_id: '42', display_name: 'live-layer' }] },
        timeline: {
          fps: { num: 30, den: 1 },
          layers: [
            {
              layer_id: '42',
              display_name: 'live-layer',
              start: { num: 0, den: 1 },
              duration: { num: 10, den: 1 },
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
    tree!.root.findByProps({ testID: 'titlebar-undo' }).props.onPress();
  });

  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0])).toEqual({
    version: 1,
    direction: 'rn-to-host',
    kind: 'undo',
    projection_generation: '0',
  });
  expect(
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r9 · 1 layers'),
  ).toBe(true);
  expect(
    tree!.root.findByProps({ testID: 'inspector-layer-section' }),
  ).toBeTruthy();

  await ReactTestRenderer.act(() => {
    dispatchHostIntentResult('redo');
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[1][0])).toEqual({
    version: 1,
    direction: 'rn-to-host',
    kind: 'redo',
    projection_generation: '2',
  });

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('dispatch rejected snapshot applies immediately and preserves typed reason', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      host_handle: '7',
      revision: '1',
      projection_generation: '0',
      current_time: { num: 0, den: 1 },
      history: { can_undo: true, can_redo: false },
      truncated_total: 0,
      stage: { bounds: [] },
      timeline: { layers: [], layers_truncated: false },
    }),
  );
  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: false,
      message: 'journal durable commit failed: disk unavailable',
      diagnostics: [{ reason: 'journal_commit' }],
      snapshot: {
        host_handle: '7',
        revision: '9',
        projection_generation: '2',
        current_time: { num: 3, den: 1 },
        history: { can_undo: true, can_redo: false },
        truncated_total: 0,
        stage: { bounds: [] },
        timeline: { layers: [], layers_truncated: false },
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

  let result: ReturnType<typeof dispatchHostIntentResult> | undefined;
  await ReactTestRenderer.act(() => {
    result = dispatchHostIntentResult('place_ellipse', { x: 0, y: 0 });
  });

  expect(result).toEqual({
    accepted: false,
    message: 'journal durable commit failed: disk unavailable',
    reason: 'journal_commit',
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0])).toEqual({
    version: 1,
    direction: 'rn-to-host',
    kind: 'place_ellipse',
    x: 0,
    y: 0,
    projection_generation: '0',
  });
  expect(
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r9 · 0 layers'),
  ).toBe(true);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
});

test('poll and dispatch response ignore lower-generation and older-host snapshots', async () => {
  jest.useFakeTimers();
  mockDispatchIntent.mockClear();
  const current = {
    host_handle: '7',
    revision: '9',
    projection_generation: '5',
    current_time: { num: 3, den: 1 },
    primary_layer_id: '42',
    history: { can_undo: true, can_redo: false },
    truncated_total: 0,
    stage: { bounds: [{ layer_id: '42', display_name: 'live-layer' }] },
    timeline: {
      layers: [
        {
          layer_id: '42',
          display_name: 'live-layer',
          position_keys: [],
          keys_truncated: false,
        },
      ],
      layers_truncated: false,
    },
  };
  mockReadSnapshot.mockReturnValue(JSON.stringify(current));
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  const stale = {
    ...current,
    revision: '1',
    projection_generation: '4',
    primary_layer_id: null,
    stage: { bounds: [] },
    timeline: { layers: [], layers_truncated: false },
  };
  mockReadSnapshot.mockReturnValue(JSON.stringify(stale));
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });
  expect(collectText(tree!.root)).toContain('DOC r9 · 1 layers');
  expect(tree!.root.findByProps({ testID: 'inspector-layer-section' })).toBeTruthy();

  mockDispatchIntent.mockImplementationOnce(() =>
    JSON.stringify({
      accepted: true,
      snapshot: {
        ...stale,
        host_handle: '6',
        projection_generation: '99',
      },
    }),
  );
  await ReactTestRenderer.act(() => {
    dispatchHostIntentResult('undo');
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0])).toMatchObject({
    kind: 'undo',
    projection_generation: '5',
  });
  expect(collectText(tree!.root)).toContain('DOC r9 · 1 layers');
  expect(tree!.root.findByProps({ testID: 'inspector-layer-section' })).toBeTruthy();

  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      ...stale,
      host_handle: '6',
      projection_generation: '99',
    }),
  );
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });
  expect(collectText(tree!.root)).toContain('DOC r9 · 1 layers');
  expect(tree!.root.findByProps({ testID: 'inspector-layer-section' })).toBeTruthy();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
  jest.useRealTimers();
});

test('titlebar undo and redo stay disabled without history and do not dispatch', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '1',
      projection_generation: '0',
      current_time: { num: 0, den: 1 },
      history: { can_undo: false, can_redo: false },
      truncated_total: 0,
      stage: { bounds: [] },
      timeline: { layers: [], layers_truncated: false },
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
    tree!.root.findByProps({ testID: 'titlebar-undo' }).props.disabled,
  ).toBe(true);
  expect(
    tree!.root.findByProps({ testID: 'titlebar-redo' }).props.disabled,
  ).toBe(true);
  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'titlebar-undo' }).props.onPress?.();
    tree!.root.findByProps({ testID: 'titlebar-redo' }).props.onPress?.();
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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      truncated_total: 1,
      stage: { bounds: [{ layer_id: '42', display_name: 'seed' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
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
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r4 · 1 layers (+1)'),
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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      truncated_total: 2,
      stage: { bounds: [{ layer_id: '42', display_name: 'seed' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
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
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r4 · 1 layers (+2)'),
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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      truncated_total: 1,
      stage: { bounds: [{ layer_id: '42', display_name: 'seed' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '1',
                plugin_id: 'core.filter.opacity',
                params: [],
              },
            ],
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
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r4 · 1 layers (+1)'),
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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      truncated_total: 3,
      stage: { bounds: [{ layer_id: '42', display_name: 'seed' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [],
            effects_truncated: false,
            source_params: [{ param_id: 'p0', value: 0 }],
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
    tree!.root
      .findAllByType(Text)
      .some(node => node.props.children === 'DOC r4 · 1 layers (+3)'),
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
      current_time: { num, den: 30 },
      primary_layer_id: '42',
      history: { can_undo: false, can_redo: false },
      truncated_total: 0,
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: 'a', time: { num: 0, den: 30 }, value: [0.1, 0.2] },
              { key_id: 'b', time: { num: 2, den: 30 }, value: [0.3, 0.4] },
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
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }),
  ).toBeTruthy();

  // gesture実信号がtrueの間は、off-key時刻のsnapshotが来ても行は凍結されたまま。
  mockIsTimelineInteracting.mockReturnValue(true);
  mockReadSnapshot.mockReturnValue(snapshotAt(1));
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }),
  ).toBeTruthy();

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
    tree!.root
      .findByProps({ testID: 'inspector-position-key-x' })
      .props.onChangeText('0.9');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-position-key-x' })
      .props.onSubmitEditing();
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-position-key-x' })
      .props.onBlur();
  });
  expect(mockDispatchIntent).toHaveBeenCalledTimes(1);
  const frozenPayload = JSON.parse(
    mockDispatchIntent.mock.calls[0][0] as string,
  );
  expect(frozenPayload.kind).toBe('set_position_key_value');
  expect(frozenPayload.target).toBe('42');
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue(snapshotAt(1));

  // gesture終了(実信号false)で確定し、off-keyの行は消える。
  mockIsTimelineInteracting.mockReturnValue(false);
  await ReactTestRenderer.act(() => {
    jest.advanceTimersByTime(1000);
  });
  expect(
    tree!.root.findAllByProps({ testID: 'inspector-position-key-x' }),
  ).toHaveLength(0);
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
      current_time: { num: 1, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [
              { key_id: 'a', time: { num: 1, den: 1 }, value: [0.1, 0.2] },
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

  const xInput = tree!.root.findByProps({ testID: 'inspector-position-key-x' });
  const yInput = tree!.root.findByProps({ testID: 'inspector-position-key-y' });
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

  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-x' }).props.value,
  ).toBe('0.55');
  expect(
    tree!.root.findByProps({ testID: 'inspector-position-key-y' }).props.value,
  ).toBe('0.88');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('opacity amount commit dispatches finite value as-is', async () => {
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{ param_id: 'amount', value: 1 }],
              },
            ],
            effects_truncated: false,
          },
        ],
        layers_truncated: false,
      },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
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

  const input = tree!.root.findByProps({
    testID: 'inspector-effect-param-input-9-amount',
  });
  await ReactTestRenderer.act(() => {
    input.props.onFocus();
    input.props.onChangeText('1.5');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onBlur();
  });

  expect(mockDispatchIntent).toHaveBeenCalled();
  const payload = lastIntent('set_effect_param');
  expect(payload?.kind).toBe('set_effect_param');
  expect(payload?.value).toBe(1.5);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1.5');

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
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.contrast',
                params: [{ param_id: 'amount', value: 1 }],
              },
            ],
            effects_truncated: false,
          },
        ],
        layers_truncated: false,
      },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.contrast',
            name: 'Contrast',
            effect_version: 1,
          },
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

  const input = tree!.root.findByProps({
    testID: 'inspector-effect-param-input-9-amount',
  });
  await ReactTestRenderer.act(() => {
    input.props.onFocus();
    input.props.onChangeText('1.5');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onBlur();
  });

  expect(mockDispatchIntent).toHaveBeenCalled();
  const payload = lastIntent('set_effect_param');
  expect(payload?.kind).toBe('set_effect_param');
  expect(payload?.value).toBe(1.5);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('1.5');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('opacity amount below 0 dispatches finite value as-is', async () => {
  mockDispatchIntent.mockClear();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
            keys_truncated: false,
            effects: [
              {
                effect_use_id: '9',
                plugin_id: 'core.filter.opacity',
                params: [{ param_id: 'amount', value: 1 }],
              },
            ],
            effects_truncated: false,
          },
        ],
        layers_truncated: false,
      },
      catalog: {
        effects: [
          {
            plugin_id: 'core.filter.opacity',
            name: 'Opacity',
            effect_version: 1,
          },
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

  const input = tree!.root.findByProps({
    testID: 'inspector-effect-param-input-9-amount',
  });
  await ReactTestRenderer.act(() => {
    input.props.onFocus();
    input.props.onChangeText('-0.25');
  });
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.onBlur();
  });

  expect(mockDispatchIntent).toHaveBeenCalled();
  const payload = lastIntent('set_effect_param');
  expect(payload?.kind).toBe('set_effect_param');
  expect(payload?.value).toBe(-0.25);
  expect(
    tree!.root.findByProps({ testID: 'inspector-effect-param-input-9-amount' })
      .props.value,
  ).toBe('-0.25');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});

test('transport play dispatches toggle_playback', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue('');
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'transport-play' }).props.onPress();
  });
  expect(lastIntent('toggle_playback')?.kind).toBe('toggle_playback');

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('transport skip start and end dispatch set_time', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '1',
      projection_generation: '1',
      current_time: { num: 1, den: 1 },
      timeline: {
        fps: { num: 30, den: 1 },
        duration: { num: 2, den: 1 },
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

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'transport-skip-start' }).props.onPress();
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string)).toEqual(
    expect.objectContaining({ kind: 'set_time', frame: 0 }),
  );

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'transport-skip-end' }).props.onPress();
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string)).toEqual(
    expect.objectContaining({ kind: 'set_time', frame: 60 }),
  );

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('transport I O dispatch trim at playhead', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '3',
      projection_generation: '1',
      current_time: { num: 2, den: 1 },
      primary_layer_id: '42',
      stage: { bounds: [{ layer_id: '42', display_name: 'seed-layer' }] },
      timeline: {
        fps: { num: 30, den: 1 },
        duration: { num: 10, den: 1 },
        layers: [
          {
            layer_id: '42',
            display_name: 'seed-layer',
            start: { num: 0, den: 1 },
            duration: { num: 10, den: 1 },
            position_keys: [],
          },
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
    tree!.root.findByProps({ testID: 'transport-mark-in' }).props.onPress();
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string)).toEqual(
    expect.objectContaining({
      kind: 'trim_clip_in',
      target: '42',
      time: { num: 2, den: 1 },
    }),
  );

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'transport-mark-out' }).props.onPress();
  });
  expect(JSON.parse(mockDispatchIntent.mock.calls[0][0] as string)).toEqual(
    expect.objectContaining({
      kind: 'trim_clip_out',
      target: '42',
      time: { num: 2, den: 1 },
    }),
  );

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('transport I O without primary does not dispatch trim', async () => {
  mockDispatchIntent.mockClear();
  mockReadSnapshot.mockReturnValue('');
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'transport-mark-in' }).props.onPress();
    tree!.root.findByProps({ testID: 'transport-mark-out' }).props.onPress();
  });
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('titlebar export opens export screen and dispatches export_document', async () => {
  mockDispatchIntent.mockImplementation(() =>
    JSON.stringify({
      accepted: false,
      message: 'document has no video source clip',
    }),
  );
  mockReadSnapshot.mockReturnValue('');
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  expect(tree!.root.findAllByProps({ testID: 'chrome-modal' })).toHaveLength(0);
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'titlebar-export' }).props.onPress();
  });
  expect(tree!.root.findByProps({ testID: 'chrome-modal' })).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'export-screen' })).toBeTruthy();
  expect(tree!.root.findAllByProps({ testID: 'settings-screen' })).toHaveLength(
    0,
  );
  expect(
    tree!.root.findByProps({ testID: 'export-status' }).props.children,
  ).toBe('Idle');

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'export-run' }).props.onPress?.();
  });
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'export-output-path' })
      .props.onChangeText('/tmp/motolii-out.mp4');
  });
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'export-run' }).props.onPress();
  });
  expect(lastIntent('export_document')?.kind).toBe('export_document');
  expect(lastIntent('export_document')?.output_path).toBe(
    '/tmp/motolii-out.mp4',
  );
  expect(
    tree!.root.findByProps({ testID: 'export-status' }).props.children,
  ).toBe('document has no video source clip');

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'chrome-modal-close' }).props.onPress();
  });
  expect(tree!.root.findAllByProps({ testID: 'chrome-modal' })).toHaveLength(0);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockDispatchIntent.mockImplementation(() => '{"accepted":true}');
});

test('titlebar settings opens settings screen with color theme row and does not dispatch', async () => {
  mockReadSnapshot.mockReturnValue('');
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'titlebar-settings' }).props.onPress();
  });
  expect(tree!.root.findByProps({ testID: 'chrome-modal' })).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'settings-screen' })).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'settings-color-theme' }),
  ).toBeTruthy();
  expect(tree!.root.findByProps({ testID: 'settings-keyboard' })).toBeTruthy();
  expect(tree!.root.findAllByProps({ testID: 'export-screen' })).toHaveLength(
    0,
  );
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('media tab lists host library files directories and tags not a fake grid', async () => {
  mockReadSnapshot.mockReturnValue(
    JSON.stringify({
      revision: '1',
      projection_generation: '1',
      current_time: { num: 0, den: 1 },
      history: { can_undo: false, can_redo: false },
      truncated_total: 0,
      stage: { bounds: [] },
      timeline: { layers: [], layers_truncated: false },
      catalog: { effects: [], sources: [] },
      library: {
        root: {
          id: 'root-0',
          name: 'media',
          path: '/private/tmp/motolii-timeline-ux-repair-20260813/docs/mocks-ui/starter-media/media',
        },
        directories: [
          { id: 'root-0:', name: 'media', path: '' },
          { id: 'root-0:b-roll', name: 'b-roll', path: 'b-roll' },
        ],
        tags: [
          { id: 'audio', label: 'Audio', count: 1 },
          { id: 'image', label: 'Image', count: 3 },
          { id: 'mp4', label: 'mp4', count: 1 },
          { id: 'png', label: 'png', count: 2 },
          { id: 'svg', label: 'svg', count: 1 },
          { id: 'video', label: 'Video', count: 1 },
          { id: 'wav', label: 'wav', count: 1 },
          { id: 'absent', label: 'Absent', count: 0 },
        ],
        items: [
          {
            id: 'root-0:starter-clip.mp4',
            name: 'starter-clip.mp4',
            kind: 'video',
            directory: '',
            tags: ['video', 'mp4'],
          },
          {
            id: 'root-0:starter-mark.svg',
            name: 'starter-mark.svg',
            kind: 'image',
            directory: '',
            tags: ['image', 'svg'],
          },
          {
            id: 'root-0:starter-still.png',
            name: 'starter-still.png',
            kind: 'image',
            directory: '',
            tags: ['image', 'png'],
          },
          {
            id: 'root-0:starter-tone.wav',
            name: 'starter-tone.wav',
            kind: 'audio',
            directory: '',
            tags: ['audio', 'wav'],
          },
          {
            id: 'root-0:b-roll/nested-still.png',
            name: 'nested-still.png',
            kind: 'image',
            directory: 'b-roll',
            tags: ['image', 'png', 'b-roll'],
          },
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
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'browser-tab-MEDIA' }).props.onPress();
  });

  expect(
    tree!.root.findByProps({ testID: 'media-item-starter-clip.mp4' }),
  ).toBeTruthy();
  expect(
    tree!.root.findByProps({ testID: 'media-item-starter-tone.wav' }),
  ).toBeTruthy();
  expect(
    tree!.root.findAllByProps({ children: '▣  media' }).length,
  ).toBeGreaterThan(0);
  expect(
    tree!.root.findAllByProps({ children: '▣  b-roll' }).length,
  ).toBeGreaterThan(0);
  expect(
    tree!.root.findAllByProps({ children: '◎  Video  1' }).length,
  ).toBeGreaterThan(0);
  expect(
    tree!.root.findAllByProps({ children: '◎  Absent  0' }).length,
  ).toBeGreaterThan(0);
  expect(
    tree!.root.findAllByProps({ children: '◎  Interview  1' }),
  ).toHaveLength(0);
  expect(
    tree!.root.findAllByProps({ children: '▣  Video' }).length,
  ).toBeGreaterThan(0);
  expect(
    tree!.root.findAllByProps({ testID: 'media-item-asset-0' }),
  ).toHaveLength(0);
  expect(mediaResultCount(tree!)).toBe(5);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'media-rail-tag-video' }).props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'media-rail-tag-video' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(mediaCardTestIDs(tree!)).toEqual(['media-item-starter-clip.mp4']);
  expect(mediaResultCount(tree!)).toBe(1);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'media-rail-tag-absent' }).props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'media-rail-tag-absent' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(mediaCardTestIDs(tree!)).toEqual([]);
  expect(mediaResultCount(tree!)).toBe(0);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'media-rail-type-image' }).props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'media-rail-type-image' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(mediaCardTestIDs(tree!)).toEqual([
    'media-item-nested-still.png',
    'media-item-starter-mark.svg',
    'media-item-starter-still.png',
  ]);
  expect(mediaResultCount(tree!)).toBe(3);

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'media-rail-dir-root-0:b-roll' })
      .props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'media-rail-dir-root-0:b-roll' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(mediaCardTestIDs(tree!)).toEqual(['media-item-nested-still.png']);
  expect(mediaResultCount(tree!)).toBe(1);

  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'media-rail-all' }).props.onPress();
  });
  expect(mediaResultCount(tree!)).toBe(5);

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'media-item-starter-clip.mp4' })
      .props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'media-item-starter-clip.mp4' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(intentPayloads().some(payload => payload.kind === 'place_media')).toBe(
    false,
  );

  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'media-item-starter-clip.mp4' })
      .props.onPress();
  });
  expect(
    tree!.root.findByProps({ testID: 'media-item-starter-clip.mp4' }).props
      .accessibilityState.selected,
  ).toBe(true);
  expect(intentPayloads().some(payload => payload.kind === 'place_media')).toBe(
    false,
  );

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'media-item-starter-clip.mp4' })
      .props.onDoubleClick();
  });
  expect(mockDispatchIntent).toHaveBeenCalled();
  const placePayload = JSON.parse(
    mockDispatchIntent.mock.calls[0][0] as string,
  );
  expect(placePayload.kind).toBe('place_media');
  expect(placePayload.item_id).toBe('root-0:starter-clip.mp4');
  expect(placePayload.position).toEqual([0, 0]);
  expect(placePayload.playhead).toEqual({ num: 0, den: 1 });

  mockDispatchIntent.mockClear();
  await ReactTestRenderer.act(() => {
    tree!.root
      .findByProps({ testID: 'media-item-starter-clip.mp4' })
      .props.onPointerDown();
  });
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.draggedItemId,
  ).toBe('root-0:starter-clip.mp4');
  await ReactTestRenderer.act(() => {
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.onStageDrop({
      nativeEvent: {
        x: 0.25,
        y: 0.75,
        canonicalX: 0.333333,
        canonicalY: -0.777777,
      },
    });
  });
  expect(
    tree!.root.findByProps({ testID: 'rust-wgpu-stage' }).props.draggedItemId,
  ).toBe('');
  expect(mockDispatchIntent).toHaveBeenCalled();
  const dropPayload = JSON.parse(mockDispatchIntent.mock.calls[0][0] as string);
  expect(dropPayload.kind).toBe('place_media');
  expect(dropPayload.item_id).toBe('root-0:starter-clip.mp4');
  expect(dropPayload.position).toEqual([0.333333, -0.777777]);

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
});

test('rn_unit_shell_keydown_forwards_mapped_keys_to_hostKeyEvent', async () => {
  mockReadSnapshot.mockReturnValue('');
  const getSpy = jest
    .spyOn(TurboModuleRegistry, 'get')
    .mockImplementation(mockHostGet as typeof TurboModuleRegistry.get);

  let tree: ReactTestRenderer.ReactTestRenderer;
  await ReactTestRenderer.act(() => {
    tree = ReactTestRenderer.create(<App />);
  });

  const fireKey = async (nativeEvent: {
    key: string;
    metaKey?: boolean;
    shiftKey?: boolean;
  }) => {
    await ReactTestRenderer.act(() => {
      tree!.root.findByProps({ testID: 'motolii-rn-shell' }).props.onKeyDown({
        nativeEvent,
        preventDefault: () => {},
      });
    });
  };

  mockDispatchIntent.mockClear();
  mockHostKeyEvent.mockClear();
  await fireKey({ key: ' ' });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(49, 0, ' ', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'k', metaKey: true });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(40, 8, 'k', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'j' });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(38, 0, 'j', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'k' });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(40, 0, 'k', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'l' });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(37, 0, 'l', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'z', metaKey: true });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(6, 8, 'z', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'z', metaKey: true, shiftKey: true });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(6, 9, 'z', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'Delete' });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(117, 0, '', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'Backspace' });
  expect(mockHostKeyEvent).toHaveBeenCalledWith(51, 0, '', false, false);

  mockHostKeyEvent.mockClear();
  await fireKey({ key: 'ArrowLeft' });
  expect(mockHostKeyEvent).not.toHaveBeenCalled();
  expect(mockDispatchIntent).not.toHaveBeenCalled();

  await ReactTestRenderer.act(() => {
    tree!.unmount();
  });
  getSpy.mockRestore();
  mockReadSnapshot.mockReturnValue('');
});
