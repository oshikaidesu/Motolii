import assert from "node:assert/strict";
import test from "node:test";
import { encodeStagePlaybackToggle } from "../src/host/stage-playback-intent-codec.js";

test("Stage playback intent is the exact bounded toggle message", () => {
  assert.deepEqual(JSON.parse(encodeStagePlaybackToggle()), { kind: "toggle-playback" });
  assert.equal(Object.keys(JSON.parse(encodeStagePlaybackToggle())).length, 1);
});
