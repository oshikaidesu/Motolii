const VERSION = 1;
const ROLE = "browser";
const HOST_DIRECTION = "host-to-web";
const WEB_DIRECTION = "web-to-host";
const MAX_ID_BYTES = 128;
const MAX_MESSAGE_BYTES = 1024;
const DECIMAL_U64 = /^(0|[1-9][0-9]*)$/;
const U64_MAX = 18_446_744_073_709_551_615n;
const encoder = new TextEncoder();

function exactKeys(value, keys, owner) {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new TypeError(`${owner} must be an object`);
  }
  const actual = Object.keys(value);
  if (
    actual.length !== keys.length
    || keys.some((key) => !Object.hasOwn(value, key))
    || actual.some((key) => !keys.includes(key))
  ) {
    throw new TypeError(`${owner} has unknown or missing fields`);
  }
  return value;
}

function boundedId(value, owner) {
  if (
    typeof value !== "string"
    || value.length === 0
    || encoder.encode(value).byteLength > MAX_ID_BYTES
  ) {
    throw new TypeError(`${owner} must be a non-empty bounded string`);
  }
  return value;
}

function u64(value, owner) {
  if (typeof value !== "string" || !DECIMAL_U64.test(value)) {
    throw new TypeError(`${owner} must be a canonical decimal u64`);
  }
  const parsed = BigInt(value);
  if (parsed > U64_MAX) {
    throw new TypeError(`${owner} exceeds u64`);
  }
  return parsed;
}

export function decodeBrowserHostSnapshot(value) {
  const root = exactKeys(
    value,
    ["version", "direction", "role", "instance_epoch", "sequence", "browser"],
    "snapshot",
  );
  if (
    root.version !== VERSION
    || root.direction !== HOST_DIRECTION
    || root.role !== ROLE
  ) {
    throw new TypeError("snapshot envelope mismatch");
  }
  const browser = exactKeys(root.browser, ["rectangle_source"], "snapshot.browser");
  const source = exactKeys(
    browser.rectangle_source,
    ["scope_ref", "item_id"],
    "snapshot.browser.rectangle_source",
  );
  return Object.freeze({
    instanceEpoch: u64(root.instance_epoch, "snapshot.instance_epoch"),
    sequence: u64(root.sequence, "snapshot.sequence"),
    rectangleIdentity: Object.freeze({
      scope_ref: boundedId(source.scope_ref, "snapshot.scope_ref"),
      item_id: boundedId(source.item_id, "snapshot.item_id"),
    }),
  });
}

export function createBrowserHostSender(snapshot, postMessage) {
  if (typeof postMessage !== "function") {
    throw new TypeError("Host postMessage must be a function");
  }
  let sequence = snapshot.sequence;
  return (intent) => {
    exactKeys(intent, ["kind", "source"], "intent");
    if (intent.kind !== "browser.place") {
      throw new TypeError("unknown Browser intent");
    }
    const source = exactKeys(intent.source, ["scope_ref", "item_id"], "intent.source");
    if (sequence === U64_MAX) {
      throw new RangeError("Browser Host sequence exhausted");
    }
    sequence += 1n;
    const message = {
      version: VERSION,
      direction: WEB_DIRECTION,
      role: ROLE,
      instance_epoch: snapshot.instanceEpoch.toString(),
      sequence: sequence.toString(),
      kind: "browser.place",
      source: {
        scope_ref: boundedId(source.scope_ref, "intent.scope_ref"),
        item_id: boundedId(source.item_id, "intent.item_id"),
      },
    };
    const encoded = JSON.stringify(message);
    if (encoder.encode(encoded).byteLength > MAX_MESSAGE_BYTES) {
      throw new RangeError("Browser Host message exceeds byte limit");
    }
    postMessage(encoded);
  };
}
