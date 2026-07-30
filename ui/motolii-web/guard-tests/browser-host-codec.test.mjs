import assert from "node:assert/strict";
import test from "node:test";
import {
  createBrowserHostSender,
  decodeBrowserHostSnapshot,
} from "../src/host/browserHostCodec.js";

function snapshot() {
  return {
    version: 1,
    direction: "host-to-web",
    role: "browser",
    instance_epoch: "7",
    sequence: "10",
    browser: {
      rectangle_source: {
        scope_ref: "catalog-scope-2",
        item_id: "rectangle",
      },
    },
  };
}

test("decodes the closed Browser snapshot and emits one sequenced Place message", () => {
  const decoded = decodeBrowserHostSnapshot(snapshot());
  const sent = [];
  const send = createBrowserHostSender(decoded, (message) => sent.push(message));
  send(Object.freeze({
    kind: "browser.place",
    source: Object.freeze({
      scope_ref: "catalog-scope-2",
      item_id: "rectangle",
    }),
  }));

  assert.equal(sent.length, 1);
  assert.deepEqual(JSON.parse(sent[0]), {
    version: 1,
    direction: "web-to-host",
    role: "browser",
    instance_epoch: "7",
    sequence: "11",
    kind: "browser.place",
    source: {
      scope_ref: "catalog-scope-2",
      item_id: "rectangle",
    },
  });
});

test("rejects unknown, stale-shaped, oversized, and exhausted inputs", () => {
  for (const mutate of [
    (value) => { value.extra = true; },
    (value) => { value.version = 2; },
    (value) => { value.direction = "web-to-host"; },
    (value) => { value.role = "inspector"; },
    (value) => { value.instance_epoch = "07"; },
    (value) => { value.browser.rectangle_source.item_id = ""; },
    (value) => { value.browser.rectangle_source.scope_ref = "x".repeat(129); },
  ]) {
    const value = snapshot();
    mutate(value);
    assert.throws(() => decodeBrowserHostSnapshot(value));
  }

  const decoded = decodeBrowserHostSnapshot(snapshot());
  const send = createBrowserHostSender(decoded, () => {});
  assert.throws(() => send({
    kind: "browser.unknown",
    source: { scope_ref: "catalog-scope-2", item_id: "rectangle" },
  }));
  assert.throws(() => send({
    kind: "browser.place",
    source: { scope_ref: "catalog-scope-2", item_id: "rectangle", extra: true },
  }));

  const exhausted = decodeBrowserHostSnapshot({
    ...snapshot(),
    sequence: "18446744073709551615",
  });
  assert.throws(() =>
    createBrowserHostSender(exhausted, () => {})({
      kind: "browser.place",
      source: { scope_ref: "catalog-scope-2", item_id: "rectangle" },
    }));
});
