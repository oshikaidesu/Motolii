import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { decodeStarterMediaProjection } from "../../../ui/motolii-web/src/read-model/starterMediaProjectionDecoder.js";

const guardDir = dirname(fileURLToPath(import.meta.url));
const mocksUiRoot = join(guardDir, "..");
const repoRoot = join(mocksUiRoot, "..", "..");

const PROVENANCE_PATH = join(
  mocksUiRoot,
  "starter-media",
  "starter-media-provenance.json",
);
const DECODER_PATH = join(
  repoRoot,
  "ui",
  "motolii-web",
  "src",
  "read-model",
  "starterMediaProjectionDecoder.js",
);
const INDEX_PATH = join(repoRoot, "ui/motolii-web/src/index.js");

const ALLOWED_CODES = ["SMP1", "SMP2", "SMP3", "SMP4", "SMP5", "SMP6", "SMP7"];
const ALLOWED_MEDIA_TYPES = new Set([
  "image/png",
  "image/svg+xml",
  "video/mp4",
  "audio/wav",
]);

function codeOf(err) {
  const message = String(err.message ?? "");
  const colon = message.indexOf(":");
  return colon < 0 ? message : message.slice(0, colon);
}

function assertFailure(fn, expectedCode) {
  try {
    fn();
    assert.fail("expected failure");
  } catch (err) {
    assert.ok(err instanceof TypeError);
    const code = codeOf(err);
    assert.ok(ALLOWED_CODES.includes(code));
    assert.equal(code, expectedCode);
  }
}

function buildBaseEnvelope() {
  const provenance = JSON.parse(readFileSync(PROVENANCE_PATH, "utf8"));
  return {
    media: provenance.media.map(({ path, mediaType }) => ({ path, mediaType })),
  };
}

function expectedProjectionFrom(base) {
  return {
    media: base.media.map(({ path, mediaType }) => ({
      path,
      mediaType,
      name: path.split("/")[path.split("/").length - 1],
    })),
  };
}

test("base succeeds with expected output", () => {
  const input = buildBaseEnvelope();
  const out = decodeStarterMediaProjection(input);
  assert.deepEqual(out, expectedProjectionFrom(input));
});

test("input is not mutated", () => {
  const input = buildBaseEnvelope();
  const before = structuredClone(input);
  decodeStarterMediaProjection(input);
  assert.deepEqual(input, before);
});

test("return media array is independent from input entry (path change)", () => {
  const input = buildBaseEnvelope();
  const out = decodeStarterMediaProjection(input);
  out.media[0].path = "changed/path.mp4";
  assert.notEqual(input.media[0].path, "changed/path.mp4");
});

test("return media array and entry are new objects", () => {
  const input = buildBaseEnvelope();
  const out = decodeStarterMediaProjection(input);
  assert.notEqual(out.media, input.media);
  assert.notEqual(out.media[0], input.media[0]);
});

test("output entry key order is path, mediaType, name", () => {
  const input = buildBaseEnvelope();
  const out = decodeStarterMediaProjection(input);
  for (const entry of out.media) {
    assert.deepEqual(Object.keys(entry), ["path", "mediaType", "name"]);
  }
});

test("base contains all four allowed media types", () => {
  const input = buildBaseEnvelope();
  const actual = new Set(input.media.map((entry) => entry.mediaType));
  assert.equal(actual.size, 4);
  for (const mediaType of ALLOWED_MEDIA_TYPES) {
    assert.ok(actual.has(mediaType));
  }
});

test("base is accepted", () => {
  const input = buildBaseEnvelope();
  decodeStarterMediaProjection(input);
});

test("path without slash is accepted and name derived from path", () => {
  const input = buildBaseEnvelope();
  input.media[2].path = "starter-still.png";
  const out = decodeStarterMediaProjection(input);
  assert.equal(out.media[2].name, "starter-still.png");
});

test("named export is not re-exported from index (starterMediaProjectionDecoder)", () => {
  const indexSrc = readFileSync(INDEX_PATH, "utf8");
  assert.equal(indexSrc.includes("starterMediaProjectionDecoder"), false);
});

test("named export is not re-exported from index (decodeStarterMediaProjection)", () => {
  const indexSrc = readFileSync(INDEX_PATH, "utf8");
  assert.equal(indexSrc.includes("decodeStarterMediaProjection"), false);
});

test("decoder does not import docs/mocks-ui", () => {
  const decoderSrc = readFileSync(DECODER_PATH, "utf8");
  assert.equal(decoderSrc.includes("docs/mocks-ui"), false);
});

test("decoder does not import mocks-ui/", () => {
  const decoderSrc = readFileSync(DECODER_PATH, "utf8");
  assert.equal(decoderSrc.includes("mocks-ui/"), false);
});

test("key order independence when entries are {mediaType, path}", () => {
  const input = buildBaseEnvelope();
  input.media = input.media.map((entry) => ({
    mediaType: entry.mediaType,
    path: entry.path,
  }));
  const out = decodeStarterMediaProjection(input);
  assert.deepEqual(out, expectedProjectionFrom(buildBaseEnvelope()));
});

for (let i = 0; i < 4; i += 1) {
  test(`key order independence with index ${i} inverted`, () => {
    const input = buildBaseEnvelope();
    const entry = input.media[i];
    input.media[i] = {
      mediaType: entry.mediaType,
      path: entry.path,
    };
    const out = decodeStarterMediaProjection(input);
    assert.deepEqual(out, expectedProjectionFrom(buildBaseEnvelope()));
  });
}

test("deterministic pure decoding from same input", () => {
  const inA = buildBaseEnvelope();
  const outA = decodeStarterMediaProjection(inA);
  const inB = buildBaseEnvelope();
  const outB = decodeStarterMediaProjection(inB);
  assert.notEqual(outA, outB);
  assert.deepEqual(outA, outB);
});

for (const [index, path] of [
  [1, "a/b/c.png"],
  [2, "c.png"],
  [2, "a-b_c.1/x.wav"],
]) {
  test(`accepted path form index ${index}: ${path}`, () => {
    const input = buildBaseEnvelope();
    input.media[index].path = path;
    const out = decodeStarterMediaProjection(input);
    assert.equal(out.media[index].path, path);
    assert.equal(out.media[index].name, path.split("/")[path.split("/").length - 1]);
  });
}

for (const value of [null, undefined, [], "{}", 1, false]) {
  test(`A root is SMP1 for ${String(value)}`, () => {
    assertFailure(() => decodeStarterMediaProjection(value), "SMP1");
  });
}

test("B1 root missing media is SMP2", () => {
  const input = buildBaseEnvelope();
  delete input.media;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("B2 root extra key is SMP2", () => {
  const input = buildBaseEnvelope();
  input.extra = true;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("B3 root rename media to Media keeps SMP2", () => {
  const input = buildBaseEnvelope();
  delete input.media;
  input.Media = buildBaseEnvelope().media;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

for (const media of [{}, "x", null, 1]) {
  test(`C media is SMP1 for ${typeof media}`, () => {
    const input = buildBaseEnvelope();
    input.media = media;
    assertFailure(() => decodeStarterMediaProjection(input), "SMP1");
  });
}

test("D zero length media fails SMP4", () => {
  const input = buildBaseEnvelope();
  input.media = [];
  assertFailure(() => decodeStarterMediaProjection(input), "SMP4");
});

test("D short media fails SMP4", () => {
  const input = buildBaseEnvelope();
  input.media = input.media.slice(0, 3);
  assertFailure(() => decodeStarterMediaProjection(input), "SMP4");
});

test("D long media with null fifth entry fails SMP4", () => {
  const input = buildBaseEnvelope();
  input.media[4] = null;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP4");
});

for (const [index, shape] of [
  [0, null],
  [0, []],
  [0, "entry"],
  [0, 1],
  [3, null],
  [3, []],
  [3, "entry"],
  [3, 1],
]) {
  test(`E entry shape ${typeof shape} at index ${index} is SMP1`, () => {
    const input = buildBaseEnvelope();
    input.media[index] = shape;
    assertFailure(() => decodeStarterMediaProjection(input), "SMP1");
  });
}

test("F missing path is SMP2", () => {
  const input = buildBaseEnvelope();
  delete input.media[0].path;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("F missing mediaType is SMP2", () => {
  const input = buildBaseEnvelope();
  delete input.media[0].mediaType;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("F unknown byteLength added is SMP2", () => {
  const input = buildBaseEnvelope();
  input.media[0].byteLength = 0;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("F unknown sha256 added is SMP2", () => {
  const input = buildBaseEnvelope();
  input.media[0].sha256 = "x";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("F unknown origin added is SMP2", () => {
  const input = buildBaseEnvelope();
  input.media[0].origin = "x";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("F rename path to Path is SMP2", () => {
  const input = buildBaseEnvelope();
  const nextPath = input.media[0].path;
  delete input.media[0].path;
  input.media[0].Path = nextPath;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

for (const [index, invalidPath] of [
  [2, 1],
  [2, null],
  [2, []],
  [2, {}],
  [2, false],
]) {
  test(`G invalid path type ${String(invalidPath)} at index ${index} is SMP3`, () => {
    const input = buildBaseEnvelope();
    input.media[index].path = invalidPath;
    assertFailure(() => decodeStarterMediaProjection(input), "SMP3");
  });
}

for (const [index, invalidType] of [
  [1, 1],
  [1, null],
  [1, []],
  [1, false],
]) {
  test(`G invalid mediaType at index ${index} is SMP3`, () => {
    const input = buildBaseEnvelope();
    input.media[index].mediaType = invalidType;
    assertFailure(() => decodeStarterMediaProjection(input), "SMP3");
  });
}

for (const invalidPath of [
  "",
  "/starter-media/media/starter-still.png",
  "starter-media\\media\\starter-still.png",
  "../starter-media/media/starter-still.png",
  "starter-media/../media/starter-still.png",
  "starter-media/media/..",
  "starter-media/media/",
  "starter-media//media/starter-still.png",
  "./starter-media/media/starter-still.png",
  "starter-media/./media/starter-still.png",
  "/",
]) {
  test(`H path format ${invalidPath} is SMP5`, () => {
    const input = buildBaseEnvelope();
    input.media[2].path = invalidPath;
    assertFailure(() => decodeStarterMediaProjection(input), "SMP5");
  });
}

test("I duplicate path at index 1 is SMP6", () => {
  const input = buildBaseEnvelope();
  input.media[1].path = input.media[0].path;
  const fixture = input;
  assertFailure(() => decodeStarterMediaProjection(fixture), "SMP6");
});

test("I duplicate mediaType at index 1 is SMP6", () => {
  const input = buildBaseEnvelope();
  input.media[1].mediaType = input.media[0].mediaType;
  const fixture = input;
  assertFailure(() => decodeStarterMediaProjection(fixture), "SMP6");
});

for (const mediaType of [
  "application/json",
  "image/jpeg",
  "IMAGE/PNG",
  "image/png ",
  " image/png",
  "",
  "video/mp4;codecs=avc1",
]) {
  test(`J unsupported mediaType ${mediaType} is SMP7`, () => {
    const input = buildBaseEnvelope();
    input.media[0].mediaType = mediaType;
    assertFailure(() => decodeStarterMediaProjection(input), "SMP7");
  });
}

test("K1 duplicate path before path-invalid entry yields SMP5", () => {
  const input = buildBaseEnvelope();
  input.media[1].path = input.media[0].path;
  input.media[2].path = "starter-media/media/";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP5");
});

test("K2 duplicate mediaType before unsupported mediaType yields SMP7", () => {
  const input = buildBaseEnvelope();
  input.media[1].mediaType = input.media[0].mediaType;
  input.media[3].mediaType = "image/jpeg";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP7");
});

test("K3 invalid path type and unsupported mediaType in same entry yields SMP3", () => {
  const input = buildBaseEnvelope();
  input.media[0].path = 1;
  input.media[0].mediaType = "image/jpeg";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP3");
});

test("K4 invalid path and unsupported mediaType in same entry yields SMP5", () => {
  const input = buildBaseEnvelope();
  input.media[0].path = "starter-media/media/";
  input.media[0].mediaType = "image/jpeg";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP5");
});

test("K5 earlier unknown key and later invalid path yields SMP2", () => {
  const input = buildBaseEnvelope();
  input.media[0].unknown = "x";
  input.media[3].path = "starter-media/media/";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP2");
});

test("K5 earlier invalid path and later unknown key yields SMP5", () => {
  const input = buildBaseEnvelope();
  input.media[0].path = "starter-media/media/";
  input.media[3].unknown = "x";
  assertFailure(() => decodeStarterMediaProjection(input), "SMP5");
});

test("K6 len 5 with trailing null entry is SMP4", () => {
  const input = buildBaseEnvelope();
  input.media[4] = null;
  assertFailure(() => decodeStarterMediaProjection(input), "SMP4");
});

test("K7 exact { media: {}, extra: true } is SMP2", () => {
  assertFailure(() => decodeStarterMediaProjection({ media: {}, extra: true }), "SMP2");
});

test("assertFailure rejects SMP8 as not allowed code", () => {
  assert.throws(
    () => assertFailure(() => {
      throw new TypeError("SMP8: x at input");
    }, "SMP8"),
    (err) => err instanceof assert.AssertionError,
  );
});

test("assertFailure rejects SMP9 as not allowed code", () => {
  assert.throws(
    () => assertFailure(() => {
      throw new TypeError("SMP9: x at input");
    }, "SMP9"),
    (err) => err instanceof assert.AssertionError,
  );
});

test("assertFailure rejects non-TypeError throwers", () => {
  assert.throws(
    () => assertFailure(() => {
      throw new Error("SMP1: x at input");
    }, "SMP1"),
    (err) => err instanceof assert.AssertionError,
  );
});

test("allowed codes include only SMP1..SMP7", () => {
  assert.equal(ALLOWED_CODES.includes("SMP8"), false);
  assert.equal(ALLOWED_CODES.includes("SMP9"), false);
});

test("decoder source contains SMP1..SMP7", () => {
  const src = readFileSync(DECODER_PATH, "utf8");
  for (const code of ALLOWED_CODES) {
    assert.ok(src.includes(code));
  }
});

test("decoder source does not contain SMP8 or SMP9", () => {
  const src = readFileSync(DECODER_PATH, "utf8");
  assert.equal(src.includes("SMP8"), false);
  assert.equal(src.includes("SMP9"), false);
});
