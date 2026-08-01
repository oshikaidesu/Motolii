import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { decodeInspectorReadModel } from "../src/read-model/inspectorReadModelDecoder.js";

function inputWithPositionControl(overrides = {}) {
  const parts = JSON.parse(readFileSync(
    new URL("../../../docs/mocks-ui/fixtures/inspector-read-model-parts.json", import.meta.url),
    "utf8",
  ));
  const document = JSON.parse(readFileSync(
    new URL("../../../docs/mocks-ui/fixtures/reference-document.json", import.meta.url),
    "utf8",
  ));
  return {
    ...parts,
    document,
    position_control: {
      layer_id: parts.target.layer_id,
      projection_generation: 9,
      key_count: 2,
      ...overrides,
    },
  };
}

test("projects only the exact Position key action identity and count", () => {
  const output = decodeInspectorReadModel(inputWithPositionControl());
  assert.deepEqual(output.position_control, {
    layer_id: 5,
    projection_generation: 9,
    key_count: 2,
  });
});

test("rejects stale-shape, cross-layer, and invalid Position key projections", () => {
  for (const input of [
    inputWithPositionControl({ layer_id: 6 }),
    inputWithPositionControl({ projection_generation: -1 }),
    inputWithPositionControl({ key_count: -1 }),
    inputWithPositionControl({ playhead: { num: 5, den: 1 } }),
    inputWithPositionControl({ at_key: true }),
  ]) {
    assert.throws(() => decodeInspectorReadModel(input));
  }
});
