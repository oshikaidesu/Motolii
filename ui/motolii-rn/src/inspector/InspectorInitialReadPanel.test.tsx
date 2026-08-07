import React from 'react';
import renderer, {act} from 'react-test-renderer';

import InspectorInitialReadPanel from './InspectorInitialReadPanel';

function snapshot(overrides: Record<string, unknown> = {}): string {
  return JSON.stringify({
    version: 1,
    direction: 'host-to-rn',
    role: 'product-runtime-seat',
    host_handle: '7',
    revision: '3',
    projection_generation: '4',
    stage: {selection: [], bounds: []},
    diagnostics: [],
    ...overrides,
  });
}

async function render(snapshotJSON?: string) {
  let root: renderer.ReactTestRenderer;
  await act(async () => {
    root = renderer.create(
      <InspectorInitialReadPanel snapshotJSON={snapshotJSON} />,
    );
  });
  return JSON.stringify(root!.toJSON());
}

describe('InspectorInitialReadPanel', () => {
  it('fails closed for absent and rejected snapshots', async () => {
    expect(await render()).toContain('Inspector unavailable');
    expect(await render('{not-json')).toContain('Inspector unavailable');
  });

  it('presents a valid snapshot with no selection', async () => {
    const serialized = await render(snapshot());

    expect(serialized).toContain('Inspector');
    expect(serialized).toContain('Initial snapshot');
    expect(serialized).toContain('No selection');
    expect(serialized).toContain('Revision');
    expect(serialized).toContain('3');
    expect(serialized).toContain('Generation');
    expect(serialized).toContain('4');
  });

  it('presents the selected layer initial read', async () => {
    const serialized = await render(
      snapshot({
        primary_layer_id: '11',
        stage: {
          selection: [{layer_id: '11'}],
          bounds: [{layer_id: '11', display_name: 'Title card'}],
        },
      }),
    );

    expect(serialized).toContain('Title card');
    expect(serialized).toContain('Layer ID');
    expect(serialized).toContain('11');
    expect(serialized).toContain('Revision');
    expect(serialized).toContain('Generation');
    expect(serialized).not.toContain('Inspector unavailable');
  });
});
