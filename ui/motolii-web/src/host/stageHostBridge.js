export function readStageHostSnapshot(bridge) {
  if (
    bridge === null
    || typeof bridge !== "object"
    || Array.isArray(bridge)
    || Object.keys(bridge).length !== 1
    || !Object.hasOwn(bridge, "snapshot")
  ) {
    throw new TypeError("Motolii Stage Host bridge is unavailable");
  }
  const snapshot = bridge.snapshot;
  if (
    snapshot === null
    || typeof snapshot !== "object"
    || Array.isArray(snapshot)
    || typeof snapshot.mode !== "string"
    || typeof snapshot.timecode !== "string"
    || typeof snapshot.barPosition !== "string"
    || typeof snapshot.tempoStatus !== "string"
    || typeof snapshot.qualityStatus !== "string"
  ) {
    throw new TypeError("Motolii Stage Host snapshot is invalid");
  }
  return Object.freeze({
    mode: snapshot.mode,
    timecode: snapshot.timecode,
    barPosition: snapshot.barPosition,
    tempoStatus: snapshot.tempoStatus,
    qualityStatus: snapshot.qualityStatus,
  });
}
