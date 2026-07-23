export const fixture = Object.freeze({
  id: "g0-9-dense-ui-v1",
  browserItems: 10_000,
  clips: 1_000,
  keyframes: 100_000,
  tracks: 32,
  timelineSeconds: 240,
  browserViewport: [420, 480],
  timelineViewport: [1200, 512],
});

export function browserItem(index) {
  return {
    id: `asset-${String(index).padStart(5, "0")}`,
    label: `Asset ${String(index).padStart(5, "0")}`,
  };
}

export function createTimelineFixture() {
  const clips = Array.from({ length: fixture.clips }, (_, index) => ({
    id: `clip-${String(index).padStart(4, "0")}`,
    track: index % fixture.tracks,
    start: (index * 0.239) % (fixture.timelineSeconds - 8),
    duration: 2 + (index % 13) * 0.37,
  }));

  const keyframes = Array.from({ length: fixture.keyframes }, (_, index) => ({
    id: `key-${String(index).padStart(6, "0")}`,
    track: index % fixture.tracks,
    time: (index * fixture.timelineSeconds) / fixture.keyframes,
  }));

  return { clips, keyframes };
}
