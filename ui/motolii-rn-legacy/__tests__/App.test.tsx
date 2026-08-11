import React from 'react';
import {NativeModules} from 'react-native';
import renderer, {act} from 'react-test-renderer';

jest.mock('../src/specs/MotoliiStageNativeComponent', () => ({
  __esModule: true,
  default: 'MotoliiStageView',
}));
jest.mock('../src/specs/MotoliiTimelineNativeComponent', () => ({
  __esModule: true,
  default: 'MotoliiTimelineView',
}));

import App from '../App';

function countOccurrences(haystack: string, needle: string): number {
  return haystack.split(needle).length - 1;
}

describe('Motolii R0 product root', () => {
  afterEach(() => {
    delete NativeModules.MotoliiHostBridge;
  });

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
    expect(countOccurrences(serialized, 'MotoliiTimelineView')).toBe(1);
    expect(countOccurrences(serialized, 'inspector-initial-read-panel')).toBe(
      1,
    );
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
    expect(countOccurrences(serialized, 'MotoliiTimelineView')).toBe(0);
    expect(countOccurrences(serialized, '"host-create-failure"')).toBe(1);
    expect(serialized).toContain('project path missing');
    expect(serialized).toContain('Host unavailable');
  });

  it('dispatches Browser Place and presents the returned snapshot', async () => {
    const dispatchIntent = jest.fn().mockResolvedValue(
      JSON.stringify({
        accepted: true,
        snapshot: {
          version: 1,
          direction: 'host-to-rn',
          role: 'product-runtime-seat',
          host_handle: '7',
          revision: '4',
          projection_generation: '5',
          primary_layer_id: '12',
          stage: {
            selection: [{layer_id: '12'}],
            bounds: [{layer_id: '12', display_name: 'Rectangle 12'}],
          },
          diagnostics: [],
        },
      }),
    );
    NativeModules.MotoliiHostBridge = {dispatchIntent};

    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(
        <App
          hostHandle="7"
          snapshotJSON={JSON.stringify({
            version: 1,
            direction: 'host-to-rn',
            role: 'product-runtime-seat',
            host_handle: '7',
            revision: '3',
            projection_generation: '4',
            primary_layer_id: null,
            stage: {selection: [], bounds: []},
            diagnostics: [],
          })}
        />,
      );
    });

    await act(async () => {
      await root!.root
        .findByProps({testID: 'browser-item-rectangle'})
        .props.onPress();
    });

    expect(dispatchIntent).toHaveBeenCalledTimes(1);
    expect(dispatchIntent.mock.calls[0]![0]).toBe('7');
    expect(JSON.parse(dispatchIntent.mock.calls[0]![1])).toEqual({
      version: 1,
      direction: 'rn-to-host',
      kind: 'place_rectangle',
      host_handle: '7',
      position: [100, 100],
      playhead: {num: 0, den: 1},
    });
    expect(JSON.stringify(root!.toJSON())).toContain('Rectangle 12');
  });
});
