export const GRAPH_VIEW_RANGE = Object.freeze({
  timeStart: 52,
  timeEnd: 56,
  valueMin: 0,
  valueMax: 100,
});

export function createGraphLayout(width = 920, height = 520) {
  const safeWidth = Math.max(320, Number.isFinite(width) ? width : 920);
  const safeHeight = Math.max(140, Number.isFinite(height) ? height : 520);
  return {
    width: safeWidth,
    height: safeHeight,
    left: 62,
    right: 20,
    top: 26,
    bottom: safeHeight < 260 ? 24 : 42,
  };
}

export const DEFAULT_GRAPH_LAYOUT = Object.freeze(createGraphLayout());

export function cloneKeys(keys) {
  return keys.map((key) => ({
    ...key,
    in: key.in ? { ...key.in } : null,
    out: key.out ? { ...key.out } : null,
  }));
}

export function xOf(time, layout, view = GRAPH_VIEW_RANGE) {
  const width = layout.width - layout.left - layout.right;
  return (
    layout.left +
    ((time - view.timeStart) / (view.timeEnd - view.timeStart)) * width
  );
}

export function yOf(value, layout, view = GRAPH_VIEW_RANGE) {
  const height = layout.height - layout.top - layout.bottom;
  return (
    layout.top +
    ((view.valueMax - value) / (view.valueMax - view.valueMin)) * height
  );
}

export function pointFromClient(
  clientX,
  clientY,
  rect,
  layout,
  view = GRAPH_VIEW_RANGE,
) {
  const graphX = ((clientX - rect.left) / rect.width) * layout.width;
  const graphY = ((clientY - rect.top) / rect.height) * layout.height;
  const width = layout.width - layout.left - layout.right;
  const height = layout.height - layout.top - layout.bottom;
  return {
    time:
      view.timeStart +
      ((graphX - layout.left) / width) * (view.timeEnd - view.timeStart),
    value:
      view.valueMax -
      ((graphY - layout.top) / height) * (view.valueMax - view.valueMin),
  };
}

export function pathFor(keys, layout, view = GRAPH_VIEW_RANGE) {
  if (keys.length === 0) return "";
  return keys.slice(1).reduce((path, key, index) => {
    const previous = keys[index];
    const out = previous.out ?? previous;
    const incoming = key.in ?? key;
    return `${path} C ${xOf(out.time, layout, view).toFixed(2)} ${yOf(
      out.value,
      layout,
      view,
    ).toFixed(2)} ${xOf(incoming.time, layout, view).toFixed(2)} ${yOf(
      incoming.value,
      layout,
      view,
    ).toFixed(2)} ${xOf(key.time, layout, view).toFixed(2)} ${yOf(
      key.value,
      layout,
      view,
    ).toFixed(2)}`;
  }, `M ${xOf(keys[0].time, layout, view).toFixed(2)} ${yOf(keys[0].value, layout, view).toFixed(2)}`);
}

function lerpPoint(left, right, amount) {
  return {
    time: left.time + (right.time - left.time) * amount,
    value: left.value + (right.value - left.value) * amount,
  };
}

export function splitSegment(keys, segmentIndex, amount, keyId) {
  const next = cloneKeys(keys);
  const left = next[segmentIndex];
  const right = next[segmentIndex + 1];
  const p0 = { time: left.time, value: left.value };
  const p1 = left.out ?? p0;
  const p3 = { time: right.time, value: right.value };
  const p2 = right.in ?? p3;
  const q0 = lerpPoint(p0, p1, amount);
  const q1 = lerpPoint(p1, p2, amount);
  const q2 = lerpPoint(p2, p3, amount);
  const r0 = lerpPoint(q0, q1, amount);
  const r1 = lerpPoint(q1, q2, amount);
  const point = lerpPoint(r0, r1, amount);

  left.out = q0;
  right.in = q2;
  next.splice(segmentIndex + 1, 0, {
    id: keyId,
    time: point.time,
    value: point.value,
    in: r0,
    out: r1,
  });
  return next;
}

export function constrainHandle(point, origin, original, modifiers) {
  const dx = point.time - origin.time;
  const dy = point.value - origin.value;
  const originalDx = original.time - origin.time;
  const originalDy = original.value - origin.value;
  const originalLength = Math.hypot(originalDx, originalDy);
  const nextLength = Math.hypot(dx, dy);
  const originalAngle = Math.atan2(originalDy, originalDx);
  const nextAngle = Math.atan2(dy, dx);

  if (modifiers.ctrlKey || modifiers.metaKey) {
    return {
      time: origin.time + Math.cos(originalAngle) * nextLength,
      value: origin.value + Math.sin(originalAngle) * nextLength,
    };
  }
  if (modifiers.altKey) {
    return {
      time: origin.time + Math.cos(nextAngle) * originalLength,
      value: origin.value + Math.sin(nextAngle) * originalLength,
    };
  }
  return point;
}
