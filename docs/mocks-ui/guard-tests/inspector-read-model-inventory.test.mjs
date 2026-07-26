import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { parse } from "@babel/parser";
import traverseModule from "@babel/traverse";

const traverse = traverseModule.default;

const guardDir = dirname(fileURLToPath(import.meta.url));
const mocksUiRoot = join(guardDir, "..");
const repoRoot = join(mocksUiRoot, "..", "..");

const DOC_PATH = join(
  repoRoot,
  "docs/reviews/2026-07-26-cu-0a08is-inspector-read-model-inventory.md",
);
const JSX_PATH = join(
  repoRoot,
  "ui/motolii-web/src/candidates/InspectorCandidate.jsx",
);
const REF_DOC_PATH = join(
  mocksUiRoot,
  "fixtures/reference-document.json",
);

const AUTHORITY_SHA256 = {
  "AGENTS.md": "aecc970b07098d423c52de5642fe6afc8038a309a4d7ea2027c4664c1b9cae4c",
  "docs/reviews/2026-07-26-cu-0a08i-inspector-read-model-split-decision.md":
    "4ec2a5296944d60b9e75275b69b841d3600d9f158d63ffccf345dca05ae12182",
  "docs/reviews/2026-07-22-m3-react-product-asset-promotion-contract.md":
    "5f632d2e7bb2632c47e4b92505add405c4fcfe70d0b104deb08b964797be652c",
  "ui/motolii-web/src/candidates/InspectorCandidate.jsx":
    "1e0bdd3eebd665e517600af4db090f74d50951aef12fdd476e97a828de91a3e4",
  "docs/mocks-ui/fixtures/reference-document.json":
    "a3b9212f13b586ec4a800390a1b524907defd32a85ee0d5bd74f5db6ae63397c",
  "docs/mocks-ui/src/reference/loadReferenceFixtures.js":
    "8476efb890c753a6267c2f7a285f8c9ef37a9abeac51c40d75774fa4d0e97d5d",
  "crates/motolii-doc/src/param.rs":
    "a72aa3ee38879506b0b4b33b5a6ece184fce94638921309e1d9d495d66331263",
  "crates/motolii-doc/src/schema.rs":
    "407dbddba4e903251490159bb53f7348b9585e3bc24e3af8eb1069d922b9e18c",
  "crates/motolii-doc/src/ids.rs":
    "836678e0fe5e558ef5d20f857ed8729fd1bc83965e2dd85392cf182a502625fc",
  "crates/motolii-doc/src/plugin_resolution.rs":
    "7c1cc5c709fc7a23a1ad9b489123ca2bf0c523064a97efa6b2f3d489fd95d5fe",
  "crates/motolii-plugin/src/lib.rs":
    "5129a4983c5edbf0f29cbae4596f71a6e6593996c36f114d4598a59feded2ca7",
  "docs/mocks-ui/guard-tests/inspector-containment.test.mjs":
    "8f5a32efdac280d7ea9c07b3336995828199d44dc743e613b4b6111236444a0b",
  "docs/mocks-ui/package.json":
    "34382a0e9f53dbd3abc61848d68ab5ccec425a3ffbf50700576512c99f111561",
  "scripts/check-docs.sh":
    "7af62c013e65416c61ffc79ede7f125254231a6b4b88c0fe53c52c85868e6cd8",
};

const STALE_PATTERNS = [
  /現在粒はCU-0A08IS/,
  /現在grainはCU-0A08IS/,
  /現在はCU-0A08ISで/,
  /CU-0A08IS.*`DO`/,
  /READY-SPEC.*ledger `DO`/,
];

const WRAPPER_KEYS = new Set([
  "fixtureRevision",
  "document",
  "nodes",
  "target",
]);

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

const R14_WHITELIST = new Set([
  "scale",
  "blend",
  "data",
  "look_at",
  "follow",
  "params",
  "extra",
  "const",
  "keyframes",
  "vec2_axes",
  "target",
  "axis",
  "offset",
  "track",
  "fallback",
  "x",
  "y",
]);

const MODE_HEADINGS = [
  "§3.1 installed-effect-focused",
  "§3.2 installed",
  "§3.3 discover",
  "§3.4 blocked",
  "§3.5 missing",
];

const KEYS_LITERAL_LINES = new Set([466, 472, 491]);

const KNOWN_COMPONENTS = new Set([
  "EffectScrubRow",
  "ScrubControl",
  "ObjectAutoHint",
  "DevInfoInstalled",
  "DevInfoEffectFocused",
  "DevInfoDiscover",
  "DevInfoBlocked",
  "DevInfoMissing",
]);

const LIFECYCLE_U_VISIBLE = new Set([
  "Project change",
  "Code execution",
  "Standard panel",
  "Install",
  "Fallback",
  "Required export",
  "Payload",
  "Project open",
  "Unrelated edit",
  "PROJECT",
  "HOST",
  "Undo / Save",
  "Preview / Export",
  "Cache / Resource",
  "SAME EVALUATION",
]);

const LIFECYCLE_S_VISIBLE = new Set([
  "NONE",
  "AVAILABLE AFTER ADD",
  "NOT STARTED",
  "REFUSED",
  "RETAINED",
  "SUCCEEDED",
  "AVAILABLE",
]);

const FIXED_ICON_VISIBLE = new Set(["G", "字", "◇", "◎", "?"]);

/** Explicit neighboring-line collisions: { id, delta, key } */
export const OFF_BY_ONE_COLLISIONS = [];

/** Shared AST source keys with multiple §3 consumer roles. */
export const SHARED_LEAF_SOURCES = [
  {
    key: "text|351|Inspector#1",
    consumers: [
      "installed-effect-focused.chrome-panel-head",
      "installed.chrome-panel-head",
      "discover.chrome-panel-head",
      "blocked.chrome-panel-head",
      "missing.chrome-panel-head",
    ],
  },
  {
    key: "component|130|ScrubControl#1",
    consumers: [
      "installed-effect-focused.scrub-control-intensity",
      "installed-effect-focused.scrub-control-spread",
      "installed.echo-scrub-control-intensity",
      "installed.echo-scrub-control-spread",
    ],
  },
  {
    key: "class|119|automation-mark#1",
    consumers: [
      "installed-effect-focused.intensity-automation",
      "installed-effect-focused.spread-automation",
      "installed.echo-intensity-automation",
      "installed.echo-spread-automation",
    ],
  },
  {
    key: "class|419|automation-mark#1",
    consumers: [
      "installed.position-automation",
      "installed.depth-automation-mark",
      "installed.scale-automation-mark",
      "installed.rotation-automation-mark",
      "installed.opacity-automation-mark",
    ],
  },
  {
    key: "class|98|scrub-dial#1",
    consumers: [
      "installed-effect-focused.intensity-dial",
      "installed-effect-focused.spread-dial",
      "installed.echo-intensity-dial",
      "installed.echo-spread-dial",
    ],
  },
  {
    key: "dynamic|99|${value}%#1",
    consumers: [
      "installed-effect-focused.intensity-output",
      "installed-effect-focused.spread-output",
      "installed.echo-intensity-output",
      "installed.echo-spread-output",
    ],
  },
  {
    key: "dynamic|132|AUTO ON / AUTO OFF#1",
    consumers: [
      "installed-effect-focused.intensity-hint",
      "installed-effect-focused.spread-hint",
      "installed.echo-intensity-hint",
      "installed.echo-spread-hint",
    ],
  },
  {
    key: "dynamic|39|AUTO ON / AUTO OFF#1",
    consumers: ["installed.object-hint-auto"],
  },
  {
    key: "component|431|ObjectAutoHint#1",
    consumers: [
      "installed.position-object-hint",
      "installed.depth-object-hint",
      "installed.scale-object-hint",
      "installed.rotation-object-hint",
      "installed.opacity-object-hint",
    ],
  },
];

/**
 * @param {{ kind: string, line: number, text: string, occ?: number }[]} records
 */
export function assignOccurrences(records) {
  const counters = new Map();
  return records.map((rec) => {
    const base = `${rec.kind}|${rec.line}|${rec.text}`;
    const occ = (counters.get(base) ?? 0) + 1;
    counters.set(base, occ);
    return { ...rec, occ };
  });
}

/**
 * @param {string} source
 * @returns {{ kind: string, line: number, text: string, occ: number }[]}
 */
export function extractInspectorAstRecords(source) {
  const ast = parse(source, { sourceType: "module", plugins: ["jsx"] });
  /** @type {{ kind: string, line: number, text: string }[]} */
  const records = [];

  function jsxTextLine(node) {
    const raw = node.value;
    const leading = raw.match(/^\s*/)[0];
    const newlines = (leading.match(/\n/g) ?? []).length;
    return node.loc.start.line + newlines;
  }

  function pushClassTokens(path, text, line) {
    const name =
      path.node.name.type === "JSXIdentifier" ? path.node.name.name : "";
    const tokens = [];
    if (text.includes("inspector") && name === "aside") {
      tokens.push({ kind: "class", text: "aside.inspector#inspector" });
    }
    if (text.includes("scrub-dial")) tokens.push({ kind: "class", text: "scrub-dial" });
    if (text.includes("automation-mark")) {
      tokens.push({ kind: "class", text: "automation-mark" });
    }
    if (text.includes("color-chip")) tokens.push({ kind: "class", text: "color-chip" });
    if (text.includes("driver-mini")) tokens.push({ kind: "class", text: "driver-mini" });
    if (text.includes("at-key")) tokens.push({ kind: "class", text: "at-key" });
    for (const t of tokens) records.push({ kind: t.kind, line, text: t.text });
  }

  function classNameRecord(path) {
    for (const a of path.node.attributes) {
      if (a.type !== "JSXAttribute" || a.name.name !== "className") continue;
      const v = a.value;
      if (!v) continue;
      let text = "";
      const line = v.loc?.start?.line ?? path.node.loc.start.line;
      if (v.type === "StringLiteral") text = v.value;
      else if (v.type === "JSXExpressionContainer") {
        const ex = v.expression;
        if (ex.type === "TemplateLiteral") {
          text = ex.quasis.map((q) => q.value.cooked ?? "").join("");
        } else if (ex.type === "StringLiteral") text = ex.value;
      }
      if (text) pushClassTokens(path, text, line);
    }
  }

  traverse(ast, {
    JSXText(path) {
      const text = path.node.value.replace(/\s+/g, " ").trim();
      if (!text) return;
      records.push({ kind: "text", line: jsxTextLine(path.node), text });
    },
    JSXOpeningElement(path) {
      if (path.node.name.type === "JSXIdentifier") {
        const name = path.node.name.name;
        if (/^[A-Z]/.test(name)) {
          records.push({ kind: "component", line: path.node.loc.start.line, text: name });
        }
        if (name === "span" && path.node.selfClosing && path.node.attributes.length === 0) {
          records.push({
            kind: "empty-span",
            line: path.node.loc.start.line,
            text: "(empty span)",
          });
        }
        if (KNOWN_COMPONENTS.has(name)) {
          for (const a of path.node.attributes) {
            if (a.type !== "JSXAttribute" || a.name.name !== "label") continue;
            if (a.value?.type === "StringLiteral") {
              records.push({
                kind: "prop-label",
                line: a.value.loc.start.line,
                text: a.value.value,
              });
            }
          }
        }
      }
      classNameRecord(path);
    },
    JSXExpressionContainer(path) {
      const parent = path.parent;
      if (parent.type !== "JSXElement" && parent.type !== "JSXFragment") return;
      const expr = path.node.expression;
      if (expr.type === "TemplateLiteral" && expr.expressions.length) {
        const slice = source.slice(expr.start, expr.end);
        if (slice.includes("value") && slice.includes("%")) {
          records.push({ kind: "dynamic", line: path.node.loc.start.line, text: "${value}%" });
        }
      }
      if (expr.type === "ConditionalExpression") {
        const src = source.slice(expr.start, expr.end);
        if (src.includes("AUTO ON") || src.includes("AUTO OFF")) {
          records.push({
            kind: "dynamic",
            line: path.node.loc.start.line,
            text: "AUTO ON / AUTO OFF",
          });
        }
      }
    },
  });

  traverse(ast, {
    CallExpression(path) {
      if (path.node.callee.type !== "Identifier" || path.node.callee.name !== "objectRow") {
        return;
      }
      const label = path.node.arguments[1];
      if (label?.type === "StringLiteral") {
        records.push({ kind: "text", line: label.loc.start.line, text: label.value });
      }
      const keys = path.node.arguments[3];
      if (keys?.type === "StringLiteral") {
        records.push({
          kind: "keys-arg",
          line: keys.loc.start.line,
          text: keys.value === "" ? "(no KEYS literal)" : keys.value,
        });
      }
      const extra = path.node.arguments[4];
      if (extra?.type === "StringLiteral" && extra.value.includes("at-key")) {
        records.push({ kind: "class", line: extra.loc.start.line, text: "at-key" });
      }
    },
  });

  return assignOccurrences(records);
}

export function recordKey(rec) {
  return `${rec.kind}|${rec.line}|${rec.text}#${rec.occ}`;
}

export function visibleMatchesRecord(visible, rec) {
  if (rec.kind === "empty-span") return visible === "(empty span)";
  if (rec.kind === "keys-arg") return visible === rec.text;
  return visible === rec.text;
}

/**
 * @param {{ kind: string, line: number, text: string, occ: number }[]} extracted
 * @param {number} line
 * @param {number} occ
 * @param {string} visible
 */
export function findExtractionRecord(extracted, line, occ, visible) {
  const matches = extracted.filter(
    (rec) => rec.line === line && rec.occ === occ && visibleMatchesRecord(visible, rec),
  );
  assert.equal(
    matches.length,
    1,
    `expected one AST record for ${visible}@${line}#${occ}, got ${matches.length}`,
  );
  return matches[0];
}

/**
 * @param {Map<string, unknown>} index
 * @param {string} canonicalKey
 */
export function validateCanonicalKey(index, canonicalKey) {
  return index.has(canonicalKey);
}

function sha256File(relPath) {
  const abs = join(repoRoot, relPath);
  const data = readFileSync(abs);
  return createHash("sha256").update(data).digest("hex");
}

/**
 * @param {string} jsxPos
 */
export function parseJsxPosition(jsxPos) {
  const m = jsxPos.match(/^InspectorCandidate\.jsx:(\d+)(?:#(\d+))?$/);
  if (!m) {
    throw new Error(`malformed JSX position: ${jsxPos}`);
  }
  const line = Number(m[1]);
  const occ = m[2] ? Number(m[2]) : 1;
  if (!Number.isInteger(line) || line < 1 || !Number.isInteger(occ) || occ < 1) {
    throw new Error(`malformed JSX position: ${jsxPos}`);
  }
  return { line, occ };
}

/**
 * @param {string} lineCol
 */
export function parseManifestLineColumn(lineCol) {
  const m = lineCol.match(/^(\d+)(?:#(\d+))?$/);
  if (!m) {
    throw new Error(`malformed manifest line: ${lineCol}`);
  }
  const line = Number(m[1]);
  const occ = m[2] ? Number(m[2]) : 1;
  if (!Number.isInteger(line) || line < 1 || !Number.isInteger(occ) || occ < 1) {
    throw new Error(`malformed manifest line: ${lineCol}`);
  }
  return { line, occ };
}

function parseSection3Tables(doc, extracted) {
  const rows = [];
  const tableRe = /^\| ([^|]+) \| ([^|]+) \| ([^|]+) \| ([^|]+) \|$/gm;
  let m;
  while ((m = tableRe.exec(doc)) !== null) {
    const [, id, jsxPos, visible, classification] = m.map((s) => s.trim());
    if (id === "要素ID" || id.startsWith("---")) continue;
    if (!/^[a-z0-9.-]+\.[a-z0-9-]+$/i.test(id)) continue;
    const { line, occ } = parseJsxPosition(jsxPos);
    const rec = findExtractionRecord(extracted, line, occ, visible);
    rows.push({
      id,
      line,
      occ,
      visible,
      classification,
      kind: rec.kind,
      canonicalKey: recordKey(rec),
    });
  }
  return rows;
}

function parseManifest(doc, extracted) {
  const start = doc.indexOf("## §9 coverage manifest");
  assert.ok(start >= 0, "§9 missing");
  const section = doc.slice(start);
  const tableStart = section.indexOf("| 可視literal / component |");
  assert.ok(tableStart >= 0, "§9 table start marker missing");
  const tableEnd = section.indexOf("\n\n## §10");
  assert.ok(tableEnd >= 0, "§9 table end marker missing");
  const table = section.slice(tableStart, tableEnd);
  const rows = [];
  const seenKeys = new Set();
  for (const line of table.split("\n")) {
    const m = line.match(/^\| (.+?) \| (\d+(?:#\d+)?) \| ([^|]+) \|$/);
    if (!m) continue;
    const [, text, lineCol, elementId] = m;
    if (text.includes("可視literal")) continue;
    if (text.startsWith("---")) continue;
    const trimmed = text.trim();
    const { line: jsxLine, occ } = parseManifestLineColumn(lineCol.trim());
    const rec = findExtractionRecord(extracted, jsxLine, occ, trimmed);
    const canonicalKey = recordKey(rec);
    assert.ok(!seenKeys.has(canonicalKey), `duplicate §9 canonical key ${canonicalKey}`);
    seenKeys.add(canonicalKey);
    rows.push({
      text: trimmed,
      line: jsxLine,
      occ,
      elementId: elementId.trim(),
      kind: rec.kind,
      canonicalKey,
    });
  }
  return rows;
}

function parseStats(doc) {
  const m = doc.match(
    /<!-- STATS: section3=(\d+) manifest=(\d+) rules=(\d+) forbidden=(\d+) -->/,
  );
  assert.ok(m, "STATS comment missing");
  return {
    section3: Number(m[1]),
    manifest: Number(m[2]),
    rules: Number(m[3]),
    forbidden: Number(m[4]),
  };
}

function parseRules(doc) {
  const section = doc.match(/## §6 拒否規則 R1〜R14[\s\S]*?(?=### §6a|## §7)/);
  assert.ok(section, "§6 missing");
  const ids = [];
  for (const line of section[0].split("\n")) {
    const m = line.match(/^\| (R\d+) \|/);
    if (m) ids.push(m[1]);
  }
  return ids;
}

function parseCapabilities(doc) {
  const section = doc.match(/## §4 source capability[\s\S]*?(?=## §5)/);
  assert.ok(section, "§4 missing");
  const rows = [];
  for (const line of section[0].split("\n")) {
    const m = line.match(/^\| (CAP-[^|]+) \|[^|]+\|[^|]+\| ([^|]+) \| (採用|未採用) \|$/);
    if (m) {
      rows.push({ id: m[1].trim(), targets: m[2].trim(), adopted: m[3].trim() === "採用" });
    }
  }
  return rows;
}

function collectJsonKeys(value, out = new Set()) {
  if (Array.isArray(value)) {
    for (const entry of value) collectJsonKeys(entry, out);
    return out;
  }
  if (value && typeof value === "object") {
    for (const [k, v] of Object.entries(value)) {
      out.add(k);
      collectJsonKeys(v, out);
    }
  }
  return out;
}

const CU_TASK_ID_RE = /CU-[0-9A-Z]+(?:-[0-9A-Z]+)*/g;

function staleEvaluationSegments(text) {
  const segments = [];
  for (const line of text.split("\n")) {
    const sentences = [];
    let sentStart = 0;
    for (let i = 0; i < line.length; i++) {
      if (line[i] === "。") {
        sentences.push(line.slice(sentStart, i + 1));
        sentStart = i + 1;
      }
    }
    if (sentStart < line.length) {
      sentences.push(line.slice(sentStart));
    }
    for (const sentence of sentences) {
      const idMatches = [...sentence.matchAll(CU_TASK_ID_RE)];
      if (idMatches.length <= 1) {
        if (sentence.length > 0) {
          segments.push(sentence);
        }
        continue;
      }
      let segStart = 0;
      for (let j = 1; j < idMatches.length; j++) {
        const cutAt = idMatches[j].index;
        const piece = sentence.slice(segStart, cutAt);
        if (piece.length > 0) {
          segments.push(piece);
        }
        segStart = cutAt;
      }
      const tail = sentence.slice(segStart);
      if (tail.length > 0) {
        segments.push(tail);
      }
    }
  }
  return segments.filter((s) => s.length > 0);
}

function assertNoStale(text) {
  for (const segment of staleEvaluationSegments(text)) {
    for (const pattern of STALE_PATTERNS) {
      if (pattern.test(segment)) {
        throw new Error(`stale pattern matched: ${pattern}`);
      }
    }
  }
}

function multisetFromRecords(records) {
  const bag = new Map();
  for (const r of records) {
    const k = recordKey(r);
    bag.set(k, (bag.get(k) ?? 0) + 1);
  }
  return bag;
}

function buildExtractionIndex(extracted) {
  const index = new Map();
  for (const rec of extracted) {
    const k = recordKey(rec);
    assert.ok(!index.has(k), `duplicate canonical extraction key ${k}`);
    index.set(k, rec);
  }
  return index;
}

function sharedLeafForConsumer(consumerId) {
  return SHARED_LEAF_SOURCES.find((leaf) => leaf.consumers.includes(consumerId));
}

function canonicalKeyForSection3Row(row) {
  if (sharedConsumerIds.has(row.id)) {
    const leaf = sharedLeafForConsumer(row.id);
    assert.ok(leaf, row.id);
    return leaf.key;
  }
  return row.canonicalKey;
}

function sourceRecordForSection3Row(row) {
  if (sharedConsumerIds.has(row.id)) {
    const leaf = sharedLeafForConsumer(row.id);
    assert.ok(leaf, row.id);
    const rec = extractionIndex.get(leaf.key);
    assert.ok(rec, leaf.key);
    return rec;
  }
  return findExtractionRecord(extracted, row.line, row.occ, row.visible);
}

const doc = readFileSync(DOC_PATH, "utf8");
const jsxSource = readFileSync(JSX_PATH, "utf8");
const extracted = extractInspectorAstRecords(jsxSource);
const extractionIndex = buildExtractionIndex(extracted);
const section3 = parseSection3Tables(doc, extracted);
const manifest = parseManifest(doc, extracted);
const stats = parseStats(doc);
const rules = parseRules(doc);
const capabilities = parseCapabilities(doc);

const sharedConsumerIds = new Set(
  SHARED_LEAF_SOURCES.flatMap((entry) => entry.consumers),
);

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

test("T1-pos section3 classifications", () => {
  assert.equal(section3.length, stats.section3);
  const ids = new Set();
  for (const row of section3) {
    assert.match(row.classification, /^[ADUS]$/, row.id);
    assert.ok(!ids.has(row.id), `duplicate §3 id ${row.id}`);
    ids.add(row.id);
  }
});

test("T1-neg classification validator", () => {
  const bad = ["A/D", "A（候補）", "S — 未決", "", "X"];
  for (const cell of bad) {
    assert.ok(!/^[ADUS]$/.test(cell));
  }
});

test("T2-extract nonempty and sentinels", () => {
  assert.ok(extracted.length > 0);
  const texts = new Set(extracted.map((r) => r.text));
  assert.ok(texts.has("Inspector"));
  assert.ok(texts.has("PROJECT INSTANCE"));
  assert.ok(texts.has("Vism (.vism)"));
  assert.ok(extracted.some((r) => r.kind === "component" && r.text === "ScrubControl"));
  assert.ok(extracted.some((r) => r.kind === "prop-label" && r.text === "Intensity"));
  const none193 = extracted.filter((r) => r.text === "NONE" && r.line === 193);
  assert.equal(none193.length, 2);
  assert.deepEqual(none193.map((r) => r.occ), [1, 2]);
});

test("T1-lifecycle and icon classification mechanical", () => {
  for (const row of section3) {
    if (LIFECYCLE_S_VISIBLE.has(row.visible)) {
      assert.equal(row.classification, "S", `${row.id} lifecycle S`);
    }
    if (LIFECYCLE_U_VISIBLE.has(row.visible)) {
      assert.equal(row.classification, "U", `${row.id} lifecycle U`);
    }
    if (FIXED_ICON_VISIBLE.has(row.visible)) {
      assert.equal(row.classification, "U", `${row.id} icon`);
    }
    if (row.kind === "prop-label") {
      assert.equal(row.classification, "U", `${row.id} prop-label`);
    }
  }
});

test("T-parse-neg production parsers fail closed", () => {
  assert.throws(() => parseJsxPosition("InspectorCandidate.jsx:12x"), /malformed/);
  assert.throws(() => parseJsxPosition("InspectorCandidate.jsx:12#0"), /malformed/);
  assert.throws(() => parseManifestLineColumn("12#0"), /malformed/);
  assert.throws(() => parseManifestLineColumn("12#x"), /malformed/);
  const badDoc = doc.replace(
    "| discover.lifecycle-code | InspectorCandidate.jsx:193 | Code execution | U |",
    "| discover.lifecycle-code | InspectorCandidate.jsx:193#9 | Code execution | U |",
  );
  assert.throws(() => parseSection3Tables(badDoc, extracted), /expected one AST record/);
});

test("T2-pos manifest bidirectional multiset closure", () => {
  assert.equal(manifest.length, stats.manifest);
  const section3Ids = new Set(section3.map((r) => r.id));
  for (const row of manifest) {
    assert.ok(section3Ids.has(row.elementId), `manifest id ${row.elementId}`);
  }
  const manifestBag = new Map();
  for (const row of manifest) {
    manifestBag.set(row.canonicalKey, (manifestBag.get(row.canonicalKey) ?? 0) + 1);
  }
  const extractedBag = multisetFromRecords(extracted);
  assert.deepEqual(
    [...manifestBag.entries()].sort(),
    [...extractedBag.entries()].sort(),
    "§9 multiset must equal AST extraction multiset",
  );
});

test("T2-section3 resolves to AST or shared leaf", () => {
  const byId = new Map(section3.map((r) => [r.id, r]));
  for (const leaf of SHARED_LEAF_SOURCES) {
    assert.ok(extractionIndex.has(leaf.key), `missing shared source ${leaf.key}`);
    for (const consumerId of leaf.consumers) {
      assert.ok(byId.has(consumerId), `missing consumer ${consumerId}`);
    }
    assert.equal(leaf.consumers.length, new Set(leaf.consumers).size, leaf.key);
  }
  const sharedSourcesUsed = new Set();
  for (const row of section3) {
    if (sharedConsumerIds.has(row.id)) {
      const leaf = sharedLeafForConsumer(row.id);
      sharedSourcesUsed.add(leaf.key);
      assert.equal(canonicalKeyForSection3Row(row), leaf.key, row.id);
      continue;
    }
    assert.ok(
      extractionIndex.has(row.canonicalKey),
      `§3 ${row.id} missing canonical key ${row.canonicalKey}`,
    );
  }
  for (const leaf of SHARED_LEAF_SOURCES) {
    assert.ok(sharedSourcesUsed.has(leaf.key), `unused shared source ${leaf.key}`);
  }
  for (const rec of extracted) {
    const key = recordKey(rec);
    const covered = section3.some(
      (row) => !sharedConsumerIds.has(row.id) && row.canonicalKey === key,
    );
    const shared = SHARED_LEAF_SOURCES.some((leaf) => leaf.key === key);
    assert.ok(covered || shared, `uncovered AST record ${key}`);
  }
});

test("T2-line-pos canonical index membership for §3 and §9", () => {
  for (const row of section3) {
    const key = canonicalKeyForSection3Row(row);
    assert.ok(
      validateCanonicalKey(extractionIndex, key),
      `§3 ${row.id} key ${key}`,
    );
  }
  for (const row of manifest) {
    assert.ok(
      validateCanonicalKey(extractionIndex, row.canonicalKey),
      `§9 ${row.text}@${row.line}#${row.occ}`,
    );
  }
});

test("T2-line-neg off-by-one through same validator", () => {
  const collisionById = new Map(
    OFF_BY_ONE_COLLISIONS.map((entry) => [`${entry.id}|${entry.delta}`, entry.key]),
  );
  const anchors = [
    ...section3.map((r) => ({ id: r.id, rec: sourceRecordForSection3Row(r) })),
    ...manifest.map((r) => ({
      id: r.elementId,
      rec: findExtractionRecord(extracted, r.line, r.occ, r.text),
    })),
  ];
  for (const { id, rec } of anchors) {
    for (const delta of [-1, 1]) {
      const mutatedKey = `${rec.kind}|${rec.line + delta}|${rec.text}#${rec.occ}`;
      const pass = validateCanonicalKey(extractionIndex, mutatedKey);
      assert.equal(
        pass,
        extractionIndex.has(mutatedKey),
        `validator drift ${mutatedKey}`,
      );
      const expectedCollision = collisionById.get(`${id}|${delta}`);
      if (expectedCollision) {
        assert.equal(mutatedKey, expectedCollision, id);
      }
    }
  }
});

test("T3 wrapper keys", () => {
  assert.deepEqual([...WRAPPER_KEYS].sort(), [
    "document",
    "fixtureRevision",
    "nodes",
    "target",
  ]);
  assert.ok(!WRAPPER_KEYS.has("screen"));
});

test("T4 rules R1-R14", () => {
  assert.equal(rules.length, 14);
  assert.equal(stats.rules, 14);
  const uniq = new Set(rules);
  assert.equal(uniq.size, 14);
  for (let i = 1; i <= 14; i++) {
    assert.ok(rules.includes(`R${i}`));
  }
});

test("T4-cap adopted capabilities", () => {
  const byId = new Map(section3.map((r) => [r.id, r]));
  for (const cap of capabilities) {
    if (!cap.adopted) continue;
    for (const part of cap.targets.split(/,\s*/)) {
      const id = part.trim();
      if (!id.includes(".")) continue;
      const row = byId.get(id);
      assert.ok(row, `missing §3 row for ${id}`);
      assert.ok(
        row.classification === "A" || row.classification === "D",
        `adopted cap ${cap.id} -> ${id}`,
      );
    }
  }
  const group = capabilities.find((c) => c.id === "CAP-GROUP-CHILD");
  assert.ok(group);
  assert.match(doc, /schema\.rs:193-196/);
  assert.match(doc, /schema\.rs:782-785/);
});

test("T4-cap-neg adopted target classified S is rejected", () => {
  const byId = new Map(section3.map((r) => [r.id, r]));
  for (const cap of capabilities) {
    if (!cap.adopted) continue;
    for (const part of cap.targets.split(/,\s*/)) {
      const id = part.trim();
      if (!id.includes(".")) continue;
      const row = byId.get(id);
      assert.notEqual(row.classification, "S", cap.id);
    }
  }
});

test("T5 R14 forbidden vocabulary", () => {
  assert.equal(stats.forbidden, FORBIDDEN_R14.length);
  for (const key of FORBIDDEN_R14) {
    assert.ok(!R14_WHITELIST.has(key), key);
  }
  const ref = JSON.parse(readFileSync(REF_DOC_PATH, "utf8"));
  const refKeys = collectJsonKeys(ref);
  for (const key of FORBIDDEN_R14) {
    assert.ok(!R14_WHITELIST.has(key));
    assert.ok(!refKeys.has(key), `forbidden ${key} in reference doc`);
  }
});

test("T6 five mode sections", () => {
  for (const heading of MODE_HEADINGS) {
    assert.ok(doc.includes(heading), heading);
  }
});

test("T7 progress docs stale-free", () => {
  const paths = [
    "docs/implementation-ledger.md",
    "docs/decision-index.md",
    "docs/specs/M3-ui-integration.md",
    "docs/reviews/2026-07-25-parallel-lane-readiness-map.md",
  ];
  assert.equal(STALE_PATTERNS.length, 5);
  assert.equal(paths.length, 4);

  const staleControls = [
    ["S1", "CU-0A08IPは`DONE`。現在粒はCU-0A08ISで進行中である。"],
    ["S2", "現在grainはCU-0A08ISである。"],
    ["S3", "現在はCU-0A08ISで作業中である。"],
    ["S4", "CU-0A08ISの現在状態は`DO`である。"],
    ["S5", "| ORACLE-GUARD | CU-0A08IS | M3 | `DO` | — | |"],
    [
      "S6",
      "READY-SPECの現行粒は完了しており、implementation ledger `DO` が残っている。",
    ],
  ];
  for (const [label, sample] of staleControls) {
    assert.throws(() => assertNoStale(sample), /stale pattern matched/, label);
  }

  const m3Spec = readFileSync(
    join(repoRoot, "docs/specs/M3-ui-integration.md"),
    "utf8",
  );
  const selectRegressionLines = (text) =>
    text.split("\n").filter(
      (line) =>
        line.includes("CU-0A08IS") && line.includes("CU-0A08BP"),
    );

  const m3RegressionLines = selectRegressionLines(m3Spec);
  assert.equal(
    m3RegressionLines.length,
    1,
    "A1: unique M3-ui-integration regression line",
  );
  const m3RegressionLine = m3RegressionLines[0];

  const syntheticBpDo =
    "CU-0A08ISはinventoryを固定した。次のPRODUCT-ASSET粒は `CU-0A08BP`（`DO`）。";
  const syntheticBpDone =
    "CU-0A08ISはinventoryを固定した。次のPRODUCT-ASSET粒は `CU-0A08BP`（`DONE`）。";
  assert.equal(selectRegressionLines(syntheticBpDo).length, 1, "A6 selector");
  assert.equal(selectRegressionLines(syntheticBpDone).length, 1, "A7 selector");
  assert.equal(
    selectRegressionLines("CU-0A08ISだけの行\nCU-0A08BPだけの行").length,
    0,
    "A8 selector requires co-occurrence",
  );

  const acceptControls = [
    ["A1", m3RegressionLine],
    [
      "A2",
      "CU-0A08ISはinventoryを固定した。次のPRODUCT-ASSET粒は `CU-0A08BP`（`DO`）。",
    ],
    [
      "A3",
      "CU-0A08ISはinventoryを固定し、CU-0A08BPは`DO`である。",
    ],
    ["A4", "| CU-0A08IS | `DONE` |"],
    [
      "A5",
      "READY-SPECはCU-0A08IS完了を記録し、次の実装粒はCU-0A08BPが`DO`である。",
    ],
    ["A6", syntheticBpDo],
    ["A7", syntheticBpDone],
  ];
  for (const [label, sample] of acceptControls) {
    assert.doesNotThrow(() => assertNoStale(sample), label);
  }

  for (const rel of paths) {
    const text = readFileSync(join(repoRoot, rel), "utf8");
    try {
      assertNoStale(text);
    } catch (err) {
      throw new Error(`${rel}: ${err.message}`);
    }
  }
  const ledger = readFileSync(join(repoRoot, "docs/implementation-ledger.md"), "utf8");
  const doneRows = ledger.match(/\| CU-0A08IS \| `DONE` \|/g) ?? [];
  assert.equal(doneRows.length, 1);
  assert.match(ledger, /\| PRODUCT-ASSET \| CU-0A08IP \|/);
  assert.match(ledger, /`READY-RECHECK`/);
});

test("T8 non-JSON example", () => {
  assert.ok(doc.includes("docs/mocks-ui/fixtures/reference-document.json"));
  assert.ok(doc.includes("非JSON"));
  const exampleBlock = doc.match(/```text[\s\S]*?```/);
  assert.ok(exampleBlock);
  assert.doesNotMatch(exampleBlock[0], /\{\}.*accepted/i);
});

test("T9 STATS match parse", () => {
  assert.equal(section3.length, stats.section3);
  assert.equal(manifest.length, stats.manifest);
  assert.equal(FORBIDDEN_R14.length, stats.forbidden);
});

test("KEYS literal closure", () => {
  const keysRows = section3.filter((r) => r.visible === "2 KEYS" || r.visible === "1 KEY");
  const lines = new Set(keysRows.map((r) => r.line));
  for (const line of lines) {
    assert.ok(KEYS_LITERAL_LINES.has(line), `KEYS at unexpected line ${line}`);
  }
  for (const line of KEYS_LITERAL_LINES) {
    assert.ok(
      section3.some((r) => r.line === line && /KEY/.test(r.visible)),
      `missing KEYS row at ${line}`,
    );
  }
});

test("scrub dial L98 closure", () => {
  const dialRows = section3.filter((r) => r.visible === "scrub-dial");
  assert.ok(dialRows.length >= 4);
  for (const row of dialRows) {
    assert.equal(row.line, 98);
  }
});

test("empty third-cell span closure", () => {
  const spans = section3.filter((r) => r.kind === "empty-span");
  assert.ok(spans.length >= 9);
});

test("R6/R7/R10 vocabulary present unabridged", () => {
  assert.match(doc, /const\/keyframes\/data\/vec2_axes\/look_at\/follow/);
  assert.match(doc, /plus_y\/plus_x/);
  assert.match(doc, /clip\/group/);
  assert.match(doc, /F64\/Vec2\/Vec3\/Color\/AssetRef/);
  assert.match(doc, /target\.layer_id/);
  assert.match(doc, /LookAt\.target/);
  assert.match(doc, /Follow\.target/);
  assert.match(doc, /normal\/add\/multiply/);
  assert.match(doc, /\[0,1\]/);
  assert.match(doc, /f64_domain/);
  assert.match(doc, /TrackItem\.kind/);
  assert.match(doc, /min_inclusive > max_inclusive/);
});
