import React from 'react';
import renderer, {act} from 'react-test-renderer';

import BrowserPanel, {BrowserPlaceRectangleIntent} from './BrowserPanel';

async function render(onPlaceIntent?: (intent: BrowserPlaceRectangleIntent) => void) {
  let root: renderer.ReactTestRenderer;
  await act(async () => {
    root = renderer.create(<BrowserPanel onPlaceIntent={onPlaceIntent} />);
  });
  return root!;
}

describe('BrowserPanel', () => {
  it('renders item list', async () => {
    const root = await render();

    const serialized = JSON.stringify(root.toJSON());

    expect(serialized).toContain('Browser');
    expect(serialized).toContain('Rectangle');
  });

  it('calls onPlaceIntent once per selection', async () => {
    const onPlaceIntent = jest.fn();
    const root = await render(onPlaceIntent);
    const button = root.root.findByProps({testID: 'browser-item-rectangle'});

    await act(async () => {
      button.props.onPress();
    });
    await act(async () => {
      button.props.onPress();
    });

    expect(onPlaceIntent).toHaveBeenCalledTimes(2);

    const payload = onPlaceIntent.mock.calls[0]![0] as BrowserPlaceRectangleIntent;

    expect(payload.kind).toBe('place_rectangle');
    expect(Array.isArray(payload.position)).toBe(true);
    expect(payload.position).toHaveLength(2);
    expect(typeof payload.position[0]).toBe('number');
    expect(typeof payload.position[1]).toBe('number');
    expect(typeof payload.playhead).toBe('number');
  });

  it('does not crash when onPlaceIntent is omitted', async () => {
    const root = await render();
    const button = root.root.findByProps({testID: 'browser-item-rectangle'});

    await act(async () => {
      button.props.onPress();
    });

    expect(root.toJSON()).toBeTruthy();
  });
});
