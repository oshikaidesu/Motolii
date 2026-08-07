import React from 'react';
import renderer, {act} from 'react-test-renderer';

jest.mock('../src/specs/MotoliiStageNativeComponent', () => ({
  __esModule: true,
  default: 'MotoliiStageView',
}));

import App from '../App';

function countOccurrences(haystack: string, needle: string): number {
  return haystack.split(needle).length - 1;
}

describe('Motolii R0 product root', () => {
  it('renders a host-backed stage and snapshot readout', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App
          hostHandle="7"
          projectPath="/tmp/project"
          snapshotJSON={JSON.stringify({
            version: 1,
            direction: 'host-to-rn',
            role: 'product-runtime-seat',
            host_handle: '7',
            revision: '3',
            projection_generation: '4',
            primary_layer_id: '11',
            stage: {
              selection: [{layer_id: '11'}],
              bounds: [{layer_id: '11', display_name: 'Title card'}],
            },
            diagnostics: [],
          })}
        />,
      );
    });
    const tree = root!.toJSON();
    const serialized = JSON.stringify(tree);

    expect(countOccurrences(serialized, '"browser-slot"')).toBe(1);
    expect(countOccurrences(serialized, '"stage-slot"')).toBe(1);
    expect(countOccurrences(serialized, '"inspector-slot"')).toBe(1);
    expect(countOccurrences(serialized, '"timeline-slot"')).toBe(1);
    expect(countOccurrences(serialized, 'MotoliiStageView')).toBe(1);
    expect(countOccurrences(serialized, 'inspector-initial-read-panel')).toBe(1);
    expect(serialized).toContain('Inspector');
    expect(serialized).toContain('Initial snapshot');
    expect(serialized).toContain('Title card');
    expect(serialized).toContain('Revision');
    expect(serialized).toContain('3');
    expect(serialized).not.toContain('primary_layer_id');
  });

  it('shows a diagnostic instead of fabricating a host', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(<App diagnostic="project path missing" />);
    });
    const tree = root!.toJSON();
    const serialized = JSON.stringify(tree);

    expect(countOccurrences(serialized, '"browser-slot"')).toBe(1);
    expect(countOccurrences(serialized, '"stage-slot"')).toBe(1);
    expect(countOccurrences(serialized, '"inspector-slot"')).toBe(1);
    expect(countOccurrences(serialized, '"timeline-slot"')).toBe(1);
    expect(countOccurrences(serialized, 'MotoliiStageView')).toBe(0);
    expect(countOccurrences(serialized, '"host-create-failure"')).toBe(1);
    expect(serialized).toContain('project path missing');
    expect(serialized).toContain('Host unavailable');
  });
});
