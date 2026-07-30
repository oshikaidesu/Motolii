import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { decodeInspectorReadModel } from "../../../ui/motolii-web/src/read-model/inspectorReadModelDecoder.js";

const guardDir = dirname(fileURLToPath(import.meta.url));
const mocksUiRoot = join(guardDir, "..");
const repoRoot = join(mocksUiRoot, "..", "..");

const PARTS_PATH = join(mocksUiRoot, "fixtures/inspector-read-model-parts.json");
const REF_DOC_PATH = join(mocksUiRoot, "fixtures/reference-document.json");
const DECODER_PATH = join(
  repoRoot,
  "ui/motolii-web/src/read-model/inspectorReadModelDecoder.js",
);

const AUTHORITY_SHA256 = {
  "ui/motolii-web/src/index.js":
    "b5a245a4f04e53186603ee6212fc1c868683a36f81c0f14b4c4329975255a585",
  "ui/motolii-web/source-provenance.json":
    "675df616f9cf774e5df9cb4b2b3a6d39b253732738a91c8ba6d2d845a61680f9",
  "ui/motolii-web/src/candidates/InspectorCandidate.jsx":
    "71d21793ab1ed19be4c976bb5bb1bf5a97c51f8392a46f034248526c6a215ba0",
  "docs/mocks-ui/fixtures/reference-document.json":
    "a3b9212f13b586ec4a800390a1b524907defd32a85ee0d5bd74f5db6ae63397c",
  "docs/mocks-ui/src/reference/loadReferenceFixtures.js":
    "8476efb890c753a6267c2f7a285f8c9ef37a9abeac51c40d75774fa4d0e97d5d",
  "docs/mocks-ui/package.json":
    "d058d3c84d7b7cf688b576d6a5da32820b65405bf78ea06363380091a88b0cf6",
};

const FORBIDDEN_R14 = [
  "fill_color",
  "stroke_color",
  "z_occlusion",
  "occlusion_mode",
  "depth_z",
  "bake_point",
  "composite_bake",
  "driver_route",
  "driver_routes",
  "applied_plugin_history",
  "availability",
  "availability_lifecycle",
  "effect_description",
  "input_socket_label",
  "socket_type_tag",
  "link_label",
  "link_target_label",
  "primary_selection",
  "selected_object",
  "editing_effect",
  "at_key",
];

function sha256File(relPath) {
  const abs = join(repoRoot, relPath);
  const data = readFileSync(abs);
  return createHash("sha256").update(data).digest("hex");
}

function loadAcceptedInput(overrides = {}) {
  const parts = JSON.parse(readFileSync(PARTS_PATH, "utf8"));
  const document = JSON.parse(readFileSync(REF_DOC_PATH, "utf8"));
  return structuredClone({ ...parts, document, ...overrides });
}

function collectKeys(value, keys = new Set()) {
  if (value === null || typeof value !== "object") {
    return keys;
  }
  if (Array.isArray(value)) {
    for (const entry of value) {
      collectKeys(entry, keys);
    }
    return keys;
  }
  for (const key of Object.keys(value)) {
    keys.add(key);
    collectKeys(value[key], keys);
  }
  return keys;
}

function assertRuleThrows(fn, rulePattern) {
  assert.throws(fn, (err) => {
    assert.ok(err instanceof TypeError);
    assert.match(err.message, rulePattern);
    return true;
  });
}

const P1_EXPECTED = {
  fixture_revision: 1,
  target: {
    layer_id: 5,
    layer_name: "Reference group",
    item_kind: "group",
    child_count: 1,
  },
  effect_definitions: [
    {
      definition_id: 0,
      plugin_id: "core.filter.opacity",
      params: [
        {
          id: "amount",
          value_type: "F64",
          default: { F64: 1.0 },
          f64_domain: {
            min_inclusive: 0.0,
            max_inclusive: 1.0,
            integer: false,
          },
        },
      ],
    },
    {
      definition_id: 2,
      plugin_id: "core.filter.opacity",
      params: [
        {
          id: "amount",
          value_type: "F64",
          default: { F64: 1.0 },
          f64_domain: {
            min_inclusive: 0.0,
            max_inclusive: 1.0,
            integer: false,
          },
        },
      ],
    },
  ],
};

test("authority hashes unchanged", () => {
  for (const [rel, expected] of Object.entries(AUTHORITY_SHA256)) {
    assert.equal(sha256File(rel), expected, rel);
  }
});

test("forbidden write/update paths absent from guard", () => {
  const guardSrc = readFileSync(fileURLToPath(import.meta.url), "utf8");
  assert.doesNotMatch(guardSrc, /\bwriteFileSync\s*\(/);
  assert.doesNotMatch(guardSrc, /\bwriteFile\s*\(/);
});

test("P1 reference target 5", () => {
  const out = decodeInspectorReadModel(loadAcceptedInput());
  assert.deepEqual(out, P1_EXPECTED);
});

test("P2 target layer 0 clip without child_count", () => {
  const input = loadAcceptedInput();
  input.target = { layer_id: 0 };
  const out = decodeInspectorReadModel(input);
  assert.equal(out.target.layer_name, "Shared left");
  assert.equal(out.target.item_kind, "clip");
  assert.equal(Object.hasOwn(out.target, "child_count"), false);
});

test("P3 target layer 6 group child clip", () => {
  const input = loadAcceptedInput();
  input.target = { layer_id: 6 };
  const out = decodeInspectorReadModel(input);
  assert.equal(out.target.layer_name, "Group shape child");
  assert.equal(out.target.item_kind, "clip");
  assert.equal(Object.hasOwn(out.target, "child_count"), false);
});

test("P4 decode is pure and non-mutating", () => {
  const input = loadAcceptedInput();
  const before = structuredClone(input);
  const a = decodeInspectorReadModel(input);
  const b = decodeInspectorReadModel(input);
  assert.deepEqual(a, b);
  assert.deepEqual(input, before);
});

test("P5 optional f64_domain omitted in output", () => {
  const input = loadAcceptedInput();
  delete input.nodes[0].params[0].f64_domain;
  const out = decodeInspectorReadModel(input);
  assert.equal(Object.hasOwn(out.effect_definitions[0].params[0], "f64_domain"), false);
});

test("P6 output keys do not intersect forbidden R14 vocabulary", () => {
  const out = decodeInspectorReadModel(loadAcceptedInput());
  const keys = collectKeys(out);
  for (const forbidden of FORBIDDEN_R14) {
    assert.equal(keys.has(forbidden), false, forbidden);
  }
});

test("P7 decoder module has no forbidden imports or default export", () => {
  const src = readFileSync(DECODER_PATH, "utf8");
  assert.doesNotMatch(src, /^import /m);
  assert.doesNotMatch(src, /\brequire\s*\(/);
  assert.doesNotMatch(src, /\bfs\b/);
  assert.doesNotMatch(src, /\bfetch\b/);
  assert.doesNotMatch(src, /\breact\b/);
  assert.doesNotMatch(src, /\bdocument\./);
  assert.doesNotMatch(src, /\bwindow\./);
  assert.doesNotMatch(src, /\bexport default\b/);
});

test("R1 extra top-level keys", () => {
  for (const key of ["screen", "mode", "scenes", "tokens", "extra_key"]) {
    const input = loadAcceptedInput();
    input[key] = "x";
    assertRuleThrows(() => decodeInspectorReadModel(input), /R1:/);
  }
});

test("R2 missing required wrapper keys", () => {
  for (const key of ["fixtureRevision", "document", "nodes", "target"]) {
    const input = loadAcceptedInput();
    delete input[key];
    assertRuleThrows(() => decodeInspectorReadModel(input), /R2:/);
  }
});

test("R3 invalid fixtureRevision", () => {
  for (const value of ["1", 1.5, 2, 0, null]) {
    const input = loadAcceptedInput();
    input.fixtureRevision = value;
    assertRuleThrows(() => decodeInspectorReadModel(input), /R3:/);
  }
});

test("R4 non-object at expected positions", () => {
  const cases = [
    (i) => { i.document = null; },
    (i) => { i.document = []; },
    (i) => { i.document = "x"; },
    (i) => { i.document = 42; },
    (i) => { i.target = []; },
    (i) => { i.nodes[0] = null; },
    (i) => {
      i.document.tracks[0].items[0].envelope = [];
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R4:/);
  }
});

test("R5 non-finite numbers", () => {
  const cases = [
    (i) => { i.target.layer_id = Number.NaN; },
    (i) => { i.target.layer_id = Number.POSITIVE_INFINITY; },
    (i) => { i.target.layer_id = "5"; },
    (i) => {
      i.nodes[0].params[0].f64_domain.min_inclusive = Number.POSITIVE_INFINITY;
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.transform.position.const.Vec2[0] =
        Number.NaN;
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        follow: { target: 0, offset: [0, Number.NaN] },
      };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R5:/);
  }
});

test("R6 unknown typed variants", () => {
  const cases = [
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = { spring: {} };
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        look_at: { target: 0, axis: "plus_z" },
      };
    },
    (i) => { i.document.tracks[0].items[0].kind = "sequence"; },
    (i) => { i.nodes[0].params[0].value_type = "F32"; },
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        const: { F64: 1 },
        keyframes: {},
      };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R6:/);
  }
});

test("R7 dangling references", () => {
  const cases = [
    (i) => { i.target.layer_id = 99; },
    (i) => {
      i.document.tracks[0].items[0].envelope.effects[0].definition_id = 7;
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        look_at: { target: 99, axis: "plus_y" },
      };
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        follow: { target: 99, offset: [0, 0] },
      };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R7:/);
  }
});

test("R7-b ambiguous track item for target layer", () => {
  const noItem = loadAcceptedInput();
  noItem.document.layers.entries.push({ id: 7, name: "Orphan" });
  noItem.target = { layer_id: 7 };
  assertRuleThrows(() => decodeInspectorReadModel(noItem), /R7-b:/);

  const dup = loadAcceptedInput();
  const layer5 = dup.document.tracks[0].items.find(
    (item) => item.envelope.layer_id === 5,
  );
  dup.document.tracks[0].items.push(structuredClone(layer5));
  assertRuleThrows(() => decodeInspectorReadModel(dup), /R7-b:/);
});

test("R8 duplicate ids", () => {
  const dupNode = loadAcceptedInput();
  dupNode.nodes.push(structuredClone(dupNode.nodes[0]));
  assertRuleThrows(() => decodeInspectorReadModel(dupNode), /R8:/);

  const dupParam = loadAcceptedInput();
  dupParam.nodes[0].params.push(structuredClone(dupParam.nodes[0].params[0]));
  assertRuleThrows(() => decodeInspectorReadModel(dupParam), /R8:/);
});

test("R9 unresolved plugin id", () => {
  const other = loadAcceptedInput();
  other.nodes[0].id = "core.filter.other";
  assertRuleThrows(() => decodeInspectorReadModel(other), /R9:/);

  const empty = loadAcceptedInput();
  empty.nodes = [];
  assertRuleThrows(() => decodeInspectorReadModel(empty), /R9:/);
});

test("R10 blend domain and opacity bounds", () => {
  const cases = [
    (i) => { i.document.tracks[0].items[0].envelope.blend = "screen"; },
    (i) => {
      i.document.tracks[0].items[5].children[0].envelope.blend = "screen";
    },
    (i) => {
      i.document.effect_definitions[0].params.opacity.const.F64 = 1.2;
    },
    (i) => {
      i.document.effect_definitions[0].params.opacity.const.F64 = -0.1;
    },
    (i) => {
      i.nodes[0].params[0].value_type = "Vec2";
      i.nodes[0].params[0].default = { Vec2: [0, 0] };
      i.nodes[0].params[0].f64_domain = {
        min_inclusive: 0,
        max_inclusive: 1,
        integer: false,
      };
    },
    (i) => {
      i.nodes[0].params[0].f64_domain = {
        min_inclusive: 1,
        max_inclusive: 0,
        integer: false,
      };
    },
    (i) => {
      i.nodes[0].params[0].f64_domain.max_inclusive = Number.NaN;
    },
    (i) => {
      i.nodes[0].params[0].default = { F64: 2.0 };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R10:/);
  }
});

test("R11 missing required fields without invention", () => {
  const cases = [
    (i) => { delete i.nodes[0].params[0].default; },
    (i) => { delete i.nodes[0].params[0].value_type; },
    (i) => { delete i.target.layer_id; },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R11:/);
  }
});

test("R12 decoder source has no semantic write patterns", () => {
  const src = readFileSync(DECODER_PATH, "utf8");
  assert.doesNotMatch(src, /\bdocument\s*=/);
  assert.doesNotMatch(src, /\bwindow\s*=/);
  assert.doesNotMatch(src, /\.push\s*\(/);
});

test("R13 authority byte hashes", () => {
  for (const [rel, expected] of Object.entries(AUTHORITY_SHA256)) {
    assert.equal(sha256File(rel), expected);
  }
});

test("R14 forbidden output keys on synthetic objects", () => {
  const validate = decodeInspectorReadModel.validateOutput;
  assert.throws(
    () => validate({ at_key: 1 }, "synthetic"),
    (err) => err.message.includes("R14:"),
  );
  assert.throws(
    () => validate({ availability: true }, "synthetic"),
    (err) => err.message.includes("R14:"),
  );
  assert.throws(
    () => validate({ depth_z: 0 }, "synthetic"),
    (err) => err.message.includes("R14:"),
  );
});

test("R14 serde keys on input remain accepted", () => {
  const input = loadAcceptedInput();
  input.document.tracks[0].items[0].envelope.transform.scale = {
    const: { Vec2: [1, 1] },
  };
  assert.doesNotThrow(() => decodeInspectorReadModel(input));
});

test("R5 rejects invalid F64 payload on ParamDef default", () => {
  const cases = [
    (i) => {
      delete i.nodes[0].params[0].f64_domain;
      i.nodes[0].params[0].default = { F64: Number.NaN };
    },
    (i) => {
      delete i.nodes[0].params[0].f64_domain;
      i.nodes[0].params[0].default = { F64: Number.POSITIVE_INFINITY };
    },
    (i) => {
      delete i.nodes[0].params[0].f64_domain;
      i.nodes[0].params[0].default = { F64: "1" };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R5:/);
  }
});

test("R5 rejects invalid F64 payload on document DocValue", () => {
  const cases = [
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        const: { F64: Number.NaN },
      };
    },
    // reference-document has no data-track opacity; const -Infinity covers non-finite F64 in DocParam
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        const: { F64: Number.NEGATIVE_INFINITY },
      };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R5:/);
  }
});

test("R5 rejects invalid AssetRef payload", () => {
  const cases = [
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        const: { AssetRef: Number.NaN },
      };
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.opacity = {
        const: { AssetRef: 1.5 },
      };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R5:/);
  }
});

test("R4 rejects wrong Vec2 Vec3 Color arity", () => {
  const cases = [
    (i) => {
      i.document.tracks[0].items[0].envelope.transform.position.const.Vec2 =
        [0.1];
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.transform.position.const.Vec2 =
        [0.1, 0.2, 0.3];
    },
    (i) => {
      i.document.tracks[0].items[0].envelope.transform.position.const = {};
    },
    (i) => {
      i.document.effect_definitions[0].params.opacity = {
        const: { Color: [0, 0, 0] },
      };
    },
    (i) => {
      i.document.effect_definitions[0].params.opacity = {
        const: { Vec3: [0, 0] },
      };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R4:/);
  }
});

test("R10 rejects value_type and default tag mismatch", () => {
  const cases = [
    (i) => {
      delete i.nodes[0].params[0].f64_domain;
      i.nodes[0].params[0].value_type = "F64";
      i.nodes[0].params[0].default = { Vec2: [0, 0] };
    },
    (i) => {
      delete i.nodes[0].params[0].f64_domain;
      i.nodes[0].params[0].value_type = "Vec2";
      i.nodes[0].params[0].default = { F64: 1 };
    },
    (i) => {
      delete i.nodes[0].params[0].f64_domain;
      i.nodes[0].params[0].value_type = "Color";
      i.nodes[0].params[0].default = { Vec3: [0, 0, 0] };
    },
  ];
  for (const mutate of cases) {
    const input = loadAcceptedInput();
    mutate(input);
    assertRuleThrows(() => decodeInspectorReadModel(input), /R10:/);
  }
});

test("R10 rejects non-integer F64 default when f64_domain.integer is true", () => {
  const input = loadAcceptedInput();
  input.nodes[0].params[0].f64_domain = {
    min_inclusive: 0,
    max_inclusive: 2,
    integer: true,
  };
  input.nodes[0].params[0].default = { F64: 0.5 };
  assertRuleThrows(() => decodeInspectorReadModel(input), /R10:/);
});

test("accepts integer f64_domain with integer default", () => {
  const input = loadAcceptedInput();
  input.nodes[0].params[0].f64_domain = {
    min_inclusive: 0,
    max_inclusive: 2,
    integer: true,
  };
  input.nodes[0].params[0].default = { F64: 1 };
  const out = decodeInspectorReadModel(input);
  assert.deepEqual(out.effect_definitions[0].params[0].f64_domain, {
    min_inclusive: 0,
    max_inclusive: 2,
    integer: true,
  });
});

test("accepts f64_domain without integer with fractional default", () => {
  const input = loadAcceptedInput();
  input.nodes[0].params[0].f64_domain = {
    min_inclusive: 0,
    max_inclusive: 2,
  };
  input.nodes[0].params[0].default = { F64: 0.5 };
  assert.doesNotThrow(() => decodeInspectorReadModel(input));
});

test("N1 output default is not the same object as input", () => {
  const input = loadAcceptedInput();
  const out = decodeInspectorReadModel(input);
  assert.notEqual(
    out.effect_definitions[0].params[0].default,
    input.nodes[0].params[0].default,
  );
  out.effect_definitions[0].params[0].default.F64 = 99;
  assert.equal(input.nodes[0].params[0].default.F64, 1);
});

test("N2 duplicate plugin_id definitions do not share default objects", () => {
  const input = loadAcceptedInput();
  const out = decodeInspectorReadModel(input);
  const a = out.effect_definitions[0].params[0].default;
  const b = out.effect_definitions[1].params[0].default;
  assert.notEqual(a, b);
  a.F64 = 99;
  assert.equal(b.F64, 1);
});

test("N3 cloneDocValue covers all value_type tags", () => {
  const cases = [
    { value_type: "F64", default: { F64: 0.25 } },
    { value_type: "Vec2", default: { Vec2: [0.1, 0.2] } },
    { value_type: "Vec3", default: { Vec3: [0.1, 0.2, 0.3] } },
    { value_type: "Color", default: { Color: [0, 0, 0, 1] } },
    { value_type: "AssetRef", default: { AssetRef: 42 } },
  ];
  for (const { value_type, default: def } of cases) {
    const input = loadAcceptedInput();
    delete input.nodes[0].params[0].f64_domain;
    input.nodes[0].params[0].value_type = value_type;
    input.nodes[0].params[0].default = def;
    const out = decodeInspectorReadModel(input);
    const outDef = out.effect_definitions[0].params[0].default;
    const inDef = input.nodes[0].params[0].default;
    assert.deepEqual(outDef, inDef);
    assert.notEqual(outDef, inDef);
    const tag = Object.keys(def)[0];
    if (tag === "Vec2" || tag === "Vec3" || tag === "Color") {
      assert.notEqual(outDef[tag], inDef[tag]);
      outDef[tag][0] = 999;
      assert.notEqual(inDef[tag][0], 999);
    }
  }
});

test("N4 double decode keeps independent default objects", () => {
  const input = loadAcceptedInput();
  const outA = decodeInspectorReadModel(input);
  const outB = decodeInspectorReadModel(input);
  outA.effect_definitions[0].params[0].default.F64 = 99;
  assert.deepEqual(outB, P1_EXPECTED);
});

test("N5 R4 rejects non-array Vec2 payload in position.const", () => {
  const input = loadAcceptedInput();
  input.document.tracks[0].items[0].envelope.transform.position.const.Vec2 = {};
  assertRuleThrows(() => decodeInspectorReadModel(input), /R4:/);
});

test("N6 output shape lock D1 through D6", () => {
  const out = decodeInspectorReadModel(loadAcceptedInput());
  assert.deepEqual(Object.keys(out), ["fixture_revision", "target", "effect_definitions"]);
  assert.deepEqual(Object.keys(out.target), [
    "layer_id",
    "layer_name",
    "item_kind",
    "child_count",
  ]);
  assert.deepEqual(Object.keys(out.effect_definitions[0].params[0]), [
    "id",
    "value_type",
    "default",
    "f64_domain",
  ]);
});
