import React from 'react';
import renderer, {act} from 'react-test-renderer';

jest.mock('../src/specs/MotoliiStageNativeComponent', () => ({
  __esModule: true,
  default: 'MotoliiStageView',
}));

import App from '../App';

describe('Motolii R0 product root', () => {
  it('renders a host-backed stage and snapshot readout', async () => {
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
    const tree = root!.toJSON();

    expect(JSON.stringify(tree)).toContain('MotoliiStageView');
    expect(JSON.stringify(tree)).toContain('revision');
    expect(JSON.stringify(tree)).toContain('3');
  });

  it('shows a diagnostic instead of fabricating a host', async () => {
    let root: renderer.ReactTestRenderer;
    await act(async () => {
      root = renderer.create(<App diagnostic="project path missing" />);
    });
    const tree = root!.toJSON();

    expect(JSON.stringify(tree)).toContain('project path missing');
    expect(JSON.stringify(tree)).toContain('Host unavailable');
  });
});
