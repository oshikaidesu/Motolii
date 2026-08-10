import React from 'react';
import renderer, {act} from 'react-test-renderer';

jest.mock('../src/specs/MotoliiStageNativeComponent', () => ({
  __esModule: true,
  default: 'MotoliiStageView',
}));
jest.mock('../src/specs/MotoliiBrowserDragSourceNativeComponent', () => ({
  __esModule: true,
  default: 'MotoliiBrowserDragSourceView',
}));

let mockSnapshotListener: ((snapshotJSON: string) => void) | undefined;
const mockSnapshotSubscriptionRemove = jest.fn();
const mockOnSnapshotChanged = jest.fn(
  (listener: (snapshotJSON: string) => void) => {
    mockSnapshotListener = listener;
    return {remove: mockSnapshotSubscriptionRemove};
  },
);
jest.mock('../src/specs/NativeMotoliiSnapshotChannel', () => ({
  __esModule: true,
  default: {
    onSnapshotChanged: (listener: (snapshotJSON: string) => void) =>
      mockOnSnapshotChanged(listener),
  },
}));

import App from '../App';

const SLOT_TEST_IDS = [
  'motolii-rn-browser-slot',
  'motolii-rn-stage-slot',
  'motolii-rn-timeline-slot',
  'motolii-rn-inspector-slot',
] as const;

function countByTestID(
  root: renderer.ReactTestRenderer,
  testID: string,
): number {
  // RN View は composite + host の両方が同一 testID を持つため host だけ数える
  return root.root.findAll(
    node =>
      typeof node.type === 'string' &&
      typeof node.props === 'object' &&
      node.props !== null &&
      (node.props as {testID?: string}).testID === testID,
  ).length;
}

function countMotoliiStageViews(root: renderer.ReactTestRenderer): number {
  return root.root.findAll(
    node => (node.type as unknown as string) === 'MotoliiStageView',
  ).length;
}

function countBrowserDragSourceViews(root: renderer.ReactTestRenderer): number {
  return root.root.findAll(
    node =>
      (node.type as unknown as string) === 'MotoliiBrowserDragSourceView',
  ).length;
}

function flattenStyle(
  style: unknown,
): Record<string, unknown> {
  if (!style) {
    return {};
  }
  if (Array.isArray(style)) {
    return style.reduce<Record<string, unknown>>(
      (acc, item) => ({...acc, ...flattenStyle(item)}),
      {},
    );
  }
  return style as Record<string, unknown>;
}

function flexOf(root: renderer.ReactTestRenderer, testID: string): number {
  const node = root.root.find(
    candidate =>
      typeof candidate.type === 'string' &&
      (candidate.props as {testID?: string}).testID === testID,
  );
  const flex = flattenStyle(node.props.style).flex;
  expect(typeof flex).toBe('number');
  return flex as number;
}

function expectSingularShell(root: renderer.ReactTestRenderer) {
  expect(countByTestID(root, 'motolii-rn-product-root')).toBe(1);
  for (const testID of SLOT_TEST_IDS) {
    expect(countByTestID(root, testID)).toBe(1);
  }
}

function textWithinTestID(
  root: renderer.ReactTestRenderer,
  testID: string,
): string {
  const node = root.root.find(
    candidate =>
      typeof candidate.type === 'string' &&
      (candidate.props as {testID?: string}).testID === testID,
  );
  const collect = (child: renderer.ReactTestInstance | string): string => {
    if (typeof child === 'string') {
      return child;
    }
    return child.children.map(collect).join('');
  };
  return node.children.map(collect).join('');
}

describe('Motolii R1-SHELL product root', () => {
  it('places four surface slots around one host-backed stage', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App
          hostHandle="7"
          projectPath="/tmp/project"
          snapshotJSON='{"revision":"3","primary_layer_id":"11"}'
        />,
      );
    });

    expectSingularShell(root!);
    expect(countMotoliiStageViews(root!)).toBe(1);
    expect(countByTestID(root!, 'motolii-stage')).toBe(1);
    expect(countByTestID(root!, 'host-create-failure')).toBe(0);

    const tree = root!.toJSON();
    expect(JSON.stringify(tree)).toContain('revision');
    expect(JSON.stringify(tree)).toContain('3');
    expect(JSON.stringify(tree)).toContain('/tmp/project');
  });

  it('gives workspace a larger flex share than timeline via flex layout', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" projectPath="/tmp/project" />,
      );
    });

    expect(flexOf(root!, 'motolii-rn-workspace')).toBeGreaterThan(
      flexOf(root!, 'motolii-rn-timeline-slot'),
    );
  });

  it('preserves singular root and slot identities across identical-prop rerenders', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App
          hostHandle="7"
          projectPath="/tmp/project"
          snapshotJSON='{"revision":"3"}'
        />,
      );
    });

    expectSingularShell(root!);
    expect(countMotoliiStageViews(root!)).toBe(1);

    await act(async () => {
      root!.update(
        <App
          hostHandle="7"
          projectPath="/tmp/project"
          snapshotJSON='{"revision":"3"}'
        />,
      );
    });
    expectSingularShell(root!);
    expect(countMotoliiStageViews(root!)).toBe(1);

    await act(async () => {
      root!.update(
        <App
          hostHandle="7"
          projectPath="/tmp/project"
          snapshotJSON='{"revision":"3"}'
        />,
      );
    });
    expectSingularShell(root!);
    expect(countMotoliiStageViews(root!)).toBe(1);
    expect(countByTestID(root!, 'motolii-stage')).toBe(1);
  });

  it('shows a diagnostic instead of fabricating a host', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(<App diagnostic="project path missing" />);
    });

    expectSingularShell(root!);
    expect(countMotoliiStageViews(root!)).toBe(0);
    expect(countByTestID(root!, 'motolii-stage')).toBe(0);
    expect(countByTestID(root!, 'host-create-failure')).toBe(1);

    const tree = root!.toJSON();
    expect(JSON.stringify(tree)).toContain('project path missing');
    expect(JSON.stringify(tree)).toContain('Host unavailable');
  });
});

describe('Motolii R1-BROWSER-READ-PROJECTION', () => {
  const browserSnapshot = (rectangleSource: Record<string, unknown>) =>
    JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      browser: {rectangle_source: rectangleSource},
      stage: {selection: [], bounds: []},
      diagnostics: [],
    });

  it('renders exactly one native Rectangle drag source with Host source identity', async () => {
    const snapshotJSON = browserSnapshot({
      scope_ref: 'builtin-7',
      item_id: 'rectangle',
    });
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshotJSON} />,
      );
    });

    expect(countByTestID(root!, 'browser-rectangle-card')).toBe(1);
    expect(countBrowserDragSourceViews(root!)).toBe(1);
    expect(countByTestID(root!, 'browser-unavailable')).toBe(0);
    expect(countMotoliiStageViews(root!)).toBe(1);
    expect(countByTestID(root!, 'inspector-no-selection')).toBe(1);

    const card = root!.root.find(
      node =>
        typeof node.type === 'string' &&
        (node.props as {testID?: string}).testID ===
          'browser-rectangle-card',
    );
    expect(card.props.accessibilityLabel).toBe(
      'Rectangle · Shape · Built-in',
    );
    expect(card.props.accessibilityValue).toEqual({
      text: 'builtin-7/rectangle',
    });
    expect(card.props.nativeID).toBe('builtin-7/rectangle');
    expect(card.props.hostHandle).toBe('7');
    expect(card.props.scopeRef).toBe('builtin-7');
    expect(card.props.itemId).toBe('rectangle');
    expect(textWithinTestID(root!, 'browser-rectangle-card')).toContain(
      'Rectangle',
    );
    const interactive = card.findAll(node => {
      const props = node.props as Record<string, unknown>;
      return (
        props.accessibilityRole === 'button' ||
        typeof props.onPress === 'function' ||
        typeof props.onLongPress === 'function' ||
        typeof props.onPointerDown === 'function' ||
        typeof props.onPointerMove === 'function' ||
        typeof props.onPointerUp === 'function' ||
        typeof props.onResponderGrant === 'function' ||
        typeof props.onResponderMove === 'function' ||
        typeof props.onResponderRelease === 'function'
      );
    });
    expect(interactive).toHaveLength(0);
  });

  it.each([
    ['empty scope', {scope_ref: '', item_id: 'rectangle'}],
    [
      'oversized scope',
      {scope_ref: `builtin-7${'x'.repeat(65)}`, item_id: 'rectangle'},
    ],
    ['forged scope', {scope_ref: 'builtin-8', item_id: 'rectangle'}],
    ['forged item', {scope_ref: 'builtin-7', item_id: 'ellipse'}],
    [
      'unknown source key',
      {scope_ref: 'builtin-7', item_id: 'rectangle', label: 'Rectangle'},
    ],
  ])('fails closed for %s', async (_label, rectangleSource) => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App
          hostHandle="7"
          snapshotJSON={browserSnapshot(rectangleSource)}
        />,
      );
    });

    expect(countByTestID(root!, 'browser-rectangle-card')).toBe(0);
    expect(countBrowserDragSourceViews(root!)).toBe(0);
    expect(countByTestID(root!, 'browser-unavailable')).toBe(1);
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);
    expect(countMotoliiStageViews(root!)).toBe(1);
  });

  it('shows no Rectangle card when the Browser projection is missing', async () => {
    const snapshotJSON = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      stage: {selection: [], bounds: []},
      diagnostics: [],
    });
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshotJSON} />,
      );
    });

    expect(countByTestID(root!, 'browser-rectangle-card')).toBe(0);
    expect(countBrowserDragSourceViews(root!)).toBe(0);
    expect(countByTestID(root!, 'browser-unavailable')).toBe(1);
    expect(countByTestID(root!, 'inspector-no-selection')).toBe(1);
  });

  it('renders no native drag source when the App host differs from the projection host', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App
          hostHandle="8"
          snapshotJSON={browserSnapshot({
            scope_ref: 'builtin-7',
            item_id: 'rectangle',
          })}
        />,
      );
    });

    expect(countBrowserDragSourceViews(root!)).toBe(0);
    expect(countByTestID(root!, 'browser-rectangle-card')).toBe(0);
    expect(countByTestID(root!, 'browser-unavailable')).toBe(1);
  });

  it('rejects unknown Browser and root fields without changing Inspector behavior', async () => {
    const unknownBrowserKey = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      browser: {
        rectangle_source: {scope_ref: 'builtin-7', item_id: 'rectangle'},
        inferred_catalog: [],
      },
      stage: {selection: [], bounds: []},
      diagnostics: [],
    });
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={unknownBrowserKey} />,
      );
    });
    expect(countByTestID(root!, 'browser-rectangle-card')).toBe(0);
    expect(countByTestID(root!, 'browser-unavailable')).toBe(1);
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);

    const unknownRootKey = JSON.stringify({
      ...JSON.parse(
        browserSnapshot({scope_ref: 'builtin-7', item_id: 'rectangle'}),
      ),
      inferred_catalog: [],
    });
    await act(async () => {
      root!.update(<App hostHandle="7" snapshotJSON={unknownRootKey} />);
    });
    expect(countByTestID(root!, 'browser-rectangle-card')).toBe(0);
    expect(countByTestID(root!, 'browser-unavailable')).toBe(1);
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);
  });
});

describe('Motolii R1-INSPECTOR-PRESENTATION', () => {
  it('renders one read-only Inspector surface for no selection', async () => {
    const snapshotJSON = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      stage: {selection: [], bounds: []},
      diagnostics: [],
    });
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshotJSON} />,
      );
    });

    expect(countByTestID(root!, 'motolii-inspector')).toBe(1);
    expect(countByTestID(root!, 'inspector-no-selection')).toBe(1);
    expect(countByTestID(root!, 'inspector-selected-target')).toBe(0);
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(0);
    expect(JSON.stringify(root!.toJSON())).toContain('No selection');
  });

  it('shows only typed identity for a valid selected target', async () => {
    const snapshotJSON = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      primary_layer_id: '11',
      stage: {
        selection: [{layer_id: '11'}],
        bounds: [{layer_id: '11', display_name: 'Rectangle'}],
      },
      diagnostics: [],
    });
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshotJSON} />,
      );
    });

    expect(countByTestID(root!, 'motolii-inspector')).toBe(1);
    expect(countByTestID(root!, 'inspector-no-selection')).toBe(0);
    expect(countByTestID(root!, 'inspector-selected-target')).toBe(1);
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(0);
    const inspectorText = textWithinTestID(root!, 'motolii-inspector');
    expect(inspectorText).toContain('Rectangle');
    expect(inspectorText).toContain('Layer 11');
    expect(inspectorText).toContain('Revision 3 · Generation 4');
  });

  it('fails closed for malformed or unknown target data', async () => {
    const snapshotJSON = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      primary_layer_id: '11',
      stage: {
        selection: [{layer_id: '11'}],
        bounds: [{layer_id: '12', display_name: 'Rectangle'}],
      },
      diagnostics: [],
    });
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshotJSON} />,
      );
    });

    expect(countByTestID(root!, 'motolii-inspector')).toBe(1);
    expect(countByTestID(root!, 'inspector-no-selection')).toBe(0);
    expect(countByTestID(root!, 'inspector-selected-target')).toBe(0);
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);
    const inspectorText = textWithinTestID(root!, 'motolii-inspector');
    expect(inspectorText).toContain('Inspector data unavailable');
    expect(inspectorText).not.toContain('Rectangle');

    const unknownFieldSnapshot = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      stage: {selection: [], bounds: [], inferred_kind: 'clip'},
      diagnostics: [],
    });
    await act(async () => {
      root!.update(
        <App hostHandle="7" snapshotJSON={unknownFieldSnapshot} />,
      );
    });
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);

    const selectionMismatchSnapshot = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      primary_layer_id: '11',
      stage: {
        selection: [{layer_id: '12'}],
        bounds: [{layer_id: '11', display_name: 'Rectangle'}],
      },
      diagnostics: [],
    });
    await act(async () => {
      root!.update(
        <App hostHandle="7" snapshotJSON={selectionMismatchSnapshot} />,
      );
    });
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);

    const selectionWithoutPrimarySnapshot = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      stage: {
        selection: [{layer_id: '11'}],
        bounds: [{layer_id: '11', display_name: 'Rectangle'}],
      },
      diagnostics: [],
    });
    await act(async () => {
      root!.update(
        <App hostHandle="7" snapshotJSON={selectionWithoutPrimarySnapshot} />,
      );
    });
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);

    const duplicateBoundsSnapshot = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      primary_layer_id: '11',
      stage: {
        selection: [{layer_id: '11'}],
        bounds: [
          {layer_id: '11', display_name: 'Rectangle'},
          {layer_id: '12', display_name: 'Other'},
          {layer_id: '12', display_name: 'Duplicate other'},
        ],
      },
      diagnostics: [],
    });
    await act(async () => {
      root!.update(
        <App hostHandle="7" snapshotJSON={duplicateBoundsSnapshot} />,
      );
    });
    expect(countByTestID(root!, 'inspector-invalid-data')).toBe(1);
  });

  it('contains no interactive editing controls and preserves one Stage', async () => {
    const snapshotJSON = JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision: '3',
      projection_generation: '4',
      primary_layer_id: '12',
      stage: {
        selection: [{layer_id: '12'}],
        bounds: [{layer_id: '12', display_name: 'Rectangle'}],
      },
      diagnostics: [],
    });
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshotJSON} />,
      );
    });

    const inspector = root!.root.find(
      node =>
        typeof node.type === 'string' &&
        (node.props as {testID?: string}).testID === 'motolii-inspector',
    );
    const editingControls = inspector.findAll(node => {
      const props = node.props as Record<string, unknown>;
      return (
        props.accessibilityRole === 'button' ||
        typeof props.onPress === 'function' ||
        typeof props.onChangeText === 'function' ||
        props.editable === true
      );
    });

    expect(editingControls).toHaveLength(0);
    expect(countMotoliiStageViews(root!)).toBe(1);
    expect(countByTestID(root!, 'motolii-stage')).toBe(1);
  });
});

describe('Motolii R1-SNAPSHOT-CHANNEL-MAC', () => {
  const snapshot = (
    revision: string,
    selection: Array<{layer_id: string}> = [],
    bounds: Array<{layer_id: string; display_name: string}> = [],
  ) =>
    JSON.stringify({
      version: 1,
      direction: 'host-to-rn',
      role: 'product-runtime-seat',
      host_handle: '7',
      revision,
      projection_generation: revision,
      ...(selection.length === 1
        ? {primary_layer_id: selection[0].layer_id}
        : {}),
      browser: {
        rectangle_source: {scope_ref: 'builtin-7', item_id: 'rectangle'},
      },
      stage: {selection, bounds},
      diagnostics: [],
    });

  beforeEach(() => {
    mockSnapshotListener = undefined;
    mockOnSnapshotChanged.mockClear();
    mockSnapshotSubscriptionRemove.mockClear();
  });

  it('renders initialProps once, then replaces the one App-root snapshot for Browser and Inspector', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshot('3')} />,
      );
    });

    expect(mockOnSnapshotChanged).toHaveBeenCalledTimes(1);
    expect(textWithinTestID(root!, 'host-snapshot')).toContain(
      '"revision":"3"',
    );
    expect(countByTestID(root!, 'inspector-no-selection')).toBe(1);

    const nextSnapshot = snapshot(
      '4',
      [{layer_id: '11'}],
      [{layer_id: '11', display_name: 'Rectangle 11'}],
    );
    await act(async () => {
      mockSnapshotListener!(nextSnapshot);
    });

    expect(textWithinTestID(root!, 'host-snapshot')).toContain(
      '"revision":"4"',
    );
    expect(textWithinTestID(root!, 'motolii-inspector')).toContain(
      'Revision 4 · Generation 4',
    );
    const card = root!.root.find(
      node =>
        typeof node.type === 'string' &&
        (node.props as {testID?: string}).testID === 'browser-rectangle-card',
    );
    expect(card.props.scopeRef).toBe('builtin-7');

    await act(async () => {
      mockSnapshotListener!(nextSnapshot);
    });
    expect(mockOnSnapshotChanged).toHaveBeenCalledTimes(1);
    expect(textWithinTestID(root!, 'host-snapshot')).toContain(
      '"revision":"4"',
    );
  });

  it.each([
    ['malformed', '{'],
    ['oversized', 'x'.repeat(16 * 1024 + 1)],
    [
      'unknown field',
      JSON.stringify({
        ...JSON.parse(snapshot('9')),
        inferred_state: 'forbidden',
      }),
    ],
    [
      'wrong host',
      JSON.stringify({
        ...JSON.parse(snapshot('9')),
        host_handle: '8',
        browser: {
          rectangle_source: {scope_ref: 'builtin-8', item_id: 'rectangle'},
        },
      }),
    ],
  ])('retains the prior valid snapshot after a %s event', async (_label, event) => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshot('3')} />,
      );
    });

    await act(async () => {
      mockSnapshotListener!(event);
    });

    expect(textWithinTestID(root!, 'host-snapshot')).toContain(
      '"revision":"3"',
    );
    expect(countByTestID(root!, 'browser-rectangle-card')).toBe(1);
    expect(countByTestID(root!, 'inspector-no-selection')).toBe(1);
  });

  it('removes the listener and ignores a late event after unmount', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App hostHandle="7" snapshotJSON={snapshot('3')} />,
      );
    });
    const lateListener = mockSnapshotListener!;

    await act(async () => {
      root!.unmount();
    });
    expect(mockSnapshotSubscriptionRemove).toHaveBeenCalledTimes(1);

    await act(async () => {
      lateListener(snapshot('4'));
    });
    expect(mockOnSnapshotChanged).toHaveBeenCalledTimes(1);
  });

  it('does not subscribe without a valid Host', async () => {
    await act(async () => {
      renderer.create(<App hostHandle="0" snapshotJSON={snapshot('3')} />);
    });

    expect(mockOnSnapshotChanged).not.toHaveBeenCalled();
  });
});
