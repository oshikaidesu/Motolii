export function readStageHostSnapshot(bridge) {
  if (
    bridge === null
    || typeof bridge !== "object"
    || Array.isArray(bridge)
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
    || typeof snapshot.easingPressed !== "boolean"
    || !Number.isSafeInteger(snapshot.layoutEpoch)
    || snapshot.layoutEpoch < 0
    || !validActiveInterval(snapshot.activeInterval)
  ) {
    throw new TypeError("Motolii Stage Host snapshot is invalid");
  }
  return Object.freeze({
    mode: snapshot.mode,
    timecode: snapshot.timecode,
    barPosition: snapshot.barPosition,
    tempoStatus: snapshot.tempoStatus,
    qualityStatus: snapshot.qualityStatus,
    activeInterval: snapshot.activeInterval,
    easingPressed: snapshot.easingPressed,
    layoutEpoch: snapshot.layoutEpoch,
  });
}

function validActiveInterval(value) {
  if (value === null) return true;
  return value !== null
    && typeof value === "object"
    && !Array.isArray(value)
    && typeof value.objectName === "string"
    && value.channel === "Position"
    && Number.isSafeInteger(value.layerId)
    && value.layerId >= 0
    && Number.isSafeInteger(value.leftKeyId)
    && value.leftKeyId >= 0
    && Number.isSafeInteger(value.rightKeyId)
    && value.rightKeyId >= 0
    && Number.isSafeInteger(value.projectionGeneration)
    && value.projectionGeneration >= 0
    && validInterp(value.interp);
}

function validInterp(value) {
  if (value === "Hold" || value === "Linear") return true;
  if (
    value === null
    || typeof value !== "object"
    || Array.isArray(value)
    || Object.keys(value).length !== 1
    || !Object.hasOwn(value, "Bezier")
  ) {
    return false;
  }
  const bezier = value.Bezier;
  return bezier !== null
    && typeof bezier === "object"
    && !Array.isArray(bezier)
    && Object.keys(bezier).length === 4
    && ["x1", "y1", "x2", "y2"].every((key) => Number.isFinite(bezier[key]))
    && bezier.x1 >= 0
    && bezier.x1 <= 1
    && bezier.x2 >= 0
    && bezier.x2 <= 1;
}
