function finite(value) {
  return typeof value === "number" && Number.isFinite(value);
}

export function encodeStageEasingIntent(anchor, layoutEpoch) {
  if (
    anchor === null || typeof anchor !== "object" || Array.isArray(anchor)
    || Object.keys(anchor).sort().join(",") !== "height,width,x,y"
    || !finite(anchor.x) || !finite(anchor.y) || !finite(anchor.width) || !finite(anchor.height)
    || anchor.width < 0 || anchor.height < 0
    || !Number.isSafeInteger(layoutEpoch) || layoutEpoch <= 0
  ) {
    throw new TypeError("Motolii Easing intent is invalid");
  }
  return JSON.stringify({
    kind: "open-position-easing",
    anchor: { x: anchor.x, y: anchor.y, width: anchor.width, height: anchor.height },
    layoutEpoch,
  });
}
