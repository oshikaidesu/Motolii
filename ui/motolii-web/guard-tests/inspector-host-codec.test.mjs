import assert from "node:assert/strict";
import test from "node:test";

import { createInspectorHostSender } from "../src/host/inspectorHostCodec.js";

function activeAmount(value = 0.72) {
  return {
    active_effect: {
      layer_id: 5,
      effect_use_id: 17,
      definition_id: 23,
      plugin_id: "core.filter.opacity",
      effect_version: 1,
      params: [{
        id: "amount",
        current: { const: { F64: value } },
        value_type: "F64",
        f64_domain: { min_inclusive: 0, max_inclusive: 1, integer: false },
        control_kind: "F64",
      }],
    },
  };
}

test("emits exact identity with monotonic gesture session and sequence", () => {
  const messages = [];
  const sender = createInspectorHostSender((raw) => messages.push(JSON.parse(raw)));
  sender.project(activeAmount());
  sender.send({ phase: "start", paramId: "amount", value: 0.72 });
  for (let index = 1; index <= 100; index += 1) {
    sender.send({ phase: "update", paramId: "amount", value: index / 100 });
  }
  sender.send({ phase: "commit", paramId: "amount", value: 1 });
  sender.send({ phase: "start", paramId: "amount", value: 1 });
  sender.send({ phase: "cancel", paramId: "amount" });

  assert.equal(messages.length, 104);
  assert.deepEqual(messages[0], {
    kind: "effect-param-gesture",
    phase: "start",
    session: 1,
    sequence: 1,
    layer_id: 5,
    effect_use_id: 17,
    definition_id: 23,
    plugin_id: "core.filter.opacity",
    effect_version: 1,
    param_id: "amount",
    value: 0.72,
  });
  assert.equal(messages[100].sequence, 101);
  assert.equal(messages[101].phase, "commit");
  assert.equal(messages[101].sequence, 102);
  assert.equal(messages[102].session, 2);
  assert.equal(messages[102].sequence, 1);
  assert.equal(messages[103].phase, "cancel");
  assert.equal(Object.hasOwn(messages[103], "value"), false);
});

test("fails closed before posting malformed or ownerless gestures", () => {
  const messages = [];
  const sender = createInspectorHostSender((raw) => messages.push(raw));
  assert.throws(() => sender.send({ phase: "start", paramId: "amount", value: 0.5 }));
  sender.project(activeAmount());
  for (const event of [
    { phase: "update", paramId: "amount", value: 0.5 },
    { phase: "start", paramId: "spread", value: 0.5 },
    { phase: "start", paramId: "amount", value: Number.NaN },
    { phase: "start", paramId: "amount", value: 1.01 },
    { phase: "cancel", paramId: "amount", value: 0.5 },
    { phase: "unknown", paramId: "amount", value: 0.5 },
  ]) {
    assert.throws(() => sender.send(event));
  }
  assert.equal(messages.length, 0);
});

test("projection changes invalidate the local active session", () => {
  const messages = [];
  const sender = createInspectorHostSender((raw) => messages.push(raw));
  sender.project(activeAmount());
  sender.send({ phase: "start", paramId: "amount", value: 0.72 });
  sender.project(null);
  assert.throws(() => sender.send({ phase: "update", paramId: "amount", value: 0.8 }));
  assert.equal(messages.length, 1);
});

test("same-identity projection preserves the active bridge session", () => {
  const messages = [];
  const sender = createInspectorHostSender((raw) => messages.push(JSON.parse(raw)));
  sender.project(activeAmount(0.72));
  sender.send({ phase: "start", paramId: "amount", value: 0.72 });
  sender.project(activeAmount(0.74));
  sender.send({ phase: "update", paramId: "amount", value: 0.8 });
  sender.send({ phase: "cancel", paramId: "amount" });
  assert.deepEqual(messages.map(({ sequence }) => sequence), [1, 2, 3]);
});

test("validation and postMessage failure do not partially advance a gesture", () => {
  const messages = [];
  let failPost = true;
  const sender = createInspectorHostSender((raw) => {
    if (failPost) throw new Error("synthetic post failure");
    messages.push(JSON.parse(raw));
  });
  sender.project(activeAmount());
  assert.throws(() => sender.send({ phase: "start", paramId: "amount", value: Number.NaN }));
  assert.throws(() => sender.send({ phase: "start", paramId: "amount", value: 0.5 }));
  failPost = false;
  sender.send({ phase: "start", paramId: "amount", value: 0.5 });
  sender.send({ phase: "cancel", paramId: "amount" });
  assert.equal(messages[0].session, 1);
  assert.equal(messages[0].sequence, 1);
});
