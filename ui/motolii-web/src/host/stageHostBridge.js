function validateStageHostBridge(bridge) {
  if (
    bridge === null
    || typeof bridge !== "object"
    || Array.isArray(bridge)
    || !Object.hasOwn(bridge, "snapshot")
  ) {
    throw new TypeError("Motolii Stage Host bridge is unavailable");
  }
  const keys = Object.keys(bridge).sort();
  const isHeaderBridge = keys.length === 1 && keys[0] === "snapshot";
  const isTransportBridge = keys.length === 3
    && keys[0] === "publish"
    && keys[1] === "snapshot"
    && keys[2] === "subscribe"
    && typeof bridge.subscribe === "function"
    && typeof bridge.publish === "function";
  if (!isHeaderBridge && !isTransportBridge) {
    throw new TypeError("Motolii Stage Host bridge is unavailable");
  }
  return Object.freeze({ bridge, isTransportBridge });
}

function decodeStageHostSnapshot(snapshot, requireActiveInterval) {
  const snapshotKeys = Object.keys(snapshot ?? {}).sort();
  const expectedSnapshotKeys = requireActiveInterval
    ? ["activeInterval", "barPosition", "mode", "playbackState", "qualityStatus", "tempoStatus", "timecode"]
    : ["barPosition", "mode", "qualityStatus", "tempoStatus", "timecode"];
  if (
    snapshot === null
    || typeof snapshot !== "object"
    || Array.isArray(snapshot)
    || typeof snapshot.mode !== "string"
    || typeof snapshot.timecode !== "string"
    || typeof snapshot.barPosition !== "string"
    || typeof snapshot.tempoStatus !== "string"
    || typeof snapshot.qualityStatus !== "string"
    || (requireActiveInterval && !["idle", "preparing", "playing"].includes(snapshot.playbackState))
    || snapshotKeys.length !== expectedSnapshotKeys.length
    || snapshotKeys.some((key, index) => key !== expectedSnapshotKeys[index])
    || (requireActiveInterval && (!Object.hasOwn(snapshot, "activeInterval")
      || (snapshot.activeInterval !== null
      && (typeof snapshot.activeInterval !== "object"
        || Array.isArray(snapshot.activeInterval)
        || Object.keys(snapshot.activeInterval).sort().join(",") !== "channel,objectName"
        || typeof snapshot.activeInterval.objectName !== "string"
        || snapshot.activeInterval.objectName.length === 0
        || snapshot.activeInterval.channel !== "Position"))))
  ) {
    throw new TypeError("Motolii Stage Host snapshot is invalid");
  }
  const decoded = {
    mode: snapshot.mode,
    timecode: snapshot.timecode,
    barPosition: snapshot.barPosition,
    tempoStatus: snapshot.tempoStatus,
    qualityStatus: snapshot.qualityStatus,
    activeInterval: snapshot.activeInterval === null || !requireActiveInterval
      ? null
      : Object.freeze({
        objectName: snapshot.activeInterval.objectName,
        channel: "Position",
      }),
  };
  if (requireActiveInterval) decoded.playbackState = snapshot.playbackState;
  return Object.freeze(decoded);
}

export function readStageHostSnapshot(bridge) {
  const validated = validateStageHostBridge(bridge);
  return decodeStageHostSnapshot(
    validated.bridge.snapshot,
    validated.isTransportBridge,
  );
}

export function subscribeStageTransportSnapshot(bridge, next) {
  const validated = validateStageHostBridge(bridge);
  if (typeof validated.bridge.subscribe !== "function" || typeof next !== "function") {
    throw new TypeError("Motolii Stage transport subscription is unavailable");
  }
  validated.bridge.subscribe((snapshot) => next(decodeStageHostSnapshot(snapshot, true)));
}
