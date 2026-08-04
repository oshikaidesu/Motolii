import assert from "node:assert/strict";
import test from "node:test";
import { encodeStageEasingIntent } from "../src/host/stage-easing-intent-codec.js";

test("Stage Easing intent contains only anchor and layout epoch", () => {
  assert.deepEqual(JSON.parse(encodeStageEasingIntent({ x: 1, y: 2, width: 3, height: 4 }, 7)), {
    kind: "open-position-easing",
    anchor: { x: 1, y: 2, width: 3, height: 4 },
    layoutEpoch: 7,
  });
});

test("Stage Easing intent rejects identity, time and malformed geometry", () => {
  assert.throws(() => encodeStageEasingIntent({ x: 0, y: 0, width: -1, height: 1 }, 1));
  assert.throws(() => encodeStageEasingIntent({ x: 0, y: 0, width: 1, height: 1, layer: 9 }, 1));
  assert.throws(() => encodeStageEasingIntent({ x: 0, y: 0, width: 1, height: 1 }, 0));
});
