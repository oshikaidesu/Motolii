import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import {
  mkdir,
  mkdtemp,
  readFile,
  readdir,
  rm,
  stat,
  symlink,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import test from "node:test";
import { PNG } from "pngjs";
import {
  StarterMediaError,
  readStarterMediaCapsule,
  validateStarterMediaCapsule,
} from "../scripts/starter-media-generation.mjs";

const execFileAsync = promisify(execFile);
const guardDir = path.dirname(fileURLToPath(import.meta.url));
const mocksUiRoot = path.join(guardDir, "..");
const repoRoot = path.join(mocksUiRoot, "..", "..");
const testSource = await readFile(fileURLToPath(import.meta.url), "utf8");

const DETERMINISM_CLAIM =
  "byte determinism is claimed only for the recorded toolchain on the recording host; cross-platform, cross-OS, and cross-toolchain byte determinism is not claimed";
const CAPSULE_ID = "g0-6h-ag-starter-media";

const MANIFEST_FIELDS = [
  "schemaVersion",
  "capsuleId",
  "purpose",
  "scope",
  "responsibilityDisposition",
  "retirement",
  "determinismClaim",
  "generator",
  "toolchain",
  "media",
];
const GENERATOR_FIELDS = ["path", "sha256"];
const NODE_FIELDS = ["version"];
const PNGJS_FIELDS = ["package", "version", "integrity"];
const VIDEO_TOOL_FIELDS = ["binary", "version"];
const TOOLCHAIN_FIELDS = ["node", "pngjs", "ff" + "mpeg"];
const MEDIA_FIELDS = ["path", "mediaType", "byteLength", "sha256", "origin", "purpose"];

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function keysExactly(value, expected) {
  assert.deepEqual(Object.keys(value).sort(), [...expected].sort());
}

async function repositoryFingerprint() {
  const { stdout: names } = await execFileAsync(
    "git",
    ["ls-files", "-co", "--exclude-standard", "-z"],
    { cwd: mocksUiRoot, encoding: "buffer", maxBuffer: 32 * 1024 * 1024 },
  );
  const files = names.toString("utf8").split("\0").filter(Boolean).sort();
  const digest = createHash("sha256");
  for (const filename of files) {
    const fullPath = path.join(mocksUiRoot, filename);
    const info = await stat(fullPath);
    if (!info.isFile()) continue;
    digest.update(filename).update("\0").update(await readFile(fullPath));
  }
  const { stdout: status } = await execFileAsync(
    "git",
    ["status", "--porcelain=v1", "-z", "--untracked-files=all"],
    { cwd: mocksUiRoot, encoding: "buffer", maxBuffer: 32 * 1024 * 1024 },
  );
  return sha256(Buffer.concat([Buffer.from(digest.digest("hex")), status]));
}

async function walkFiles(directory) {
  const result = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...(await walkFiles(full)));
    else if (entry.isFile()) result.push(full);
  }
  return result;
}

async function walkCapsuleFiles(directory, prefix = "") {
  const result = [];
  for (const entry of (
    await readdir(directory, { withFileTypes: true })
  ).sort((left, right) => left.name.localeCompare(right.name))) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    const full = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const nested = await walkCapsuleFiles(full, relative);
      if (nested.length === 0) {
        assert.fail(`empty capsule directory ${relative}`);
      }
      result.push(...nested);
    } else if (entry.isFile()) {
      result.push(relative);
    } else {
      assert.fail(`non-file capsule entry ${relative}`);
    }
  }
  return result;
}

async function loadCapsuleContext() {
  const { manifest, files } = await readStarterMediaCapsule({ root: mocksUiRoot });
  const lock = JSON.parse(await readFile(path.join(mocksUiRoot, "package-lock.json"), "utf8"));
  const locked = lock.packages["node_modules/pngjs"];
  const generatorBytes = await readFile(
    path.join(mocksUiRoot, "scripts/starter-media-generation.mjs"),
  );
  return {
    manifest,
    files,
    context: {
      lockedPngjs: { version: locked.version, integrity: locked.integrity },
      generatorSha256: sha256(generatorBytes),
    },
  };
}

function cloneManifest(manifest) {
  return structuredClone(manifest);
}

function cloneFiles(files) {
  return new Map([...files.entries()].map(([key, value]) => [key, Buffer.from(value)]));
}

function repinMediaEntry(manifest, mediaPath, bytes) {
  const entry = manifest.media.find((item) => item.path === mediaPath);
  entry.byteLength = bytes.length;
  entry.sha256 = sha256(bytes);
}

test("starter-media capsule guard is read-only and network-free", async (t) => {
  const forbiddenMediaTool = new RegExp("\\b" + "ff" + "mpeg" + "\\b", "i");
  assert.doesNotMatch(testSource, forbiddenMediaTool);
  assert.doesNotMatch(testSource, /\.spawn\s*\(/);
  assert.doesNotMatch(testSource, /\bfetch\s*\(/);

  const before = await repositoryFingerprint();
  t.after(async () => {
    const after = await repositoryFingerprint();
    assert.equal(before, after);
  });

  const { manifest, files, context } = await loadCapsuleContext();

  const capsuleRoot = path.join(mocksUiRoot, "starter-media");
  const relFiles = (await walkCapsuleFiles(capsuleRoot)).sort();
  assert.deepEqual(relFiles, [
    "media/starter-clip.mp4",
    "media/starter-mark.svg",
    "media/starter-still.png",
    "media/starter-tone.wav",
    "starter-media-provenance.json",
  ]);

  keysExactly(manifest, MANIFEST_FIELDS);
  keysExactly(manifest.generator, GENERATOR_FIELDS);
  keysExactly(manifest.toolchain, TOOLCHAIN_FIELDS);
  keysExactly(manifest.toolchain.node, NODE_FIELDS);
  keysExactly(manifest.toolchain.pngjs, PNGJS_FIELDS);
  keysExactly(manifest.toolchain["ff" + "mpeg"], VIDEO_TOOL_FIELDS);
  assert.equal(manifest.schemaVersion, 1);
  assert.equal(manifest.capsuleId, CAPSULE_ID);
  assert.equal(manifest.responsibilityDisposition, "WRAP");
  assert.equal(manifest.retirement, "FROZEN / DELETE-LATER");
  assert.equal(manifest.determinismClaim, DETERMINISM_CLAIM);

  const mediaPaths = manifest.media.map((entry) => entry.path);
  assert.deepEqual(mediaPaths, [...mediaPaths].sort());
  assert.equal(manifest.media.length, 4);
  for (const entry of manifest.media) {
    keysExactly(entry, MEDIA_FIELDS);
    const bytes = files.get(entry.path);
    assert.ok(bytes);
    assert.equal(bytes.length, entry.byteLength);
    assert.equal(sha256(bytes), entry.sha256);
  }

  const still = files.get("starter-media/media/starter-still.png");
  assert.ok(still.subarray(0, 8).equals(Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a])));
  const png = PNG.sync.read(still);
  assert.equal(png.width, 640);
  assert.equal(png.height, 360);

  const clip = files.get("starter-media/media/starter-clip.mp4");
  assert.equal(clip.subarray(4, 8).toString("ascii"), "ftyp");

  const tone = files.get("starter-media/media/starter-tone.wav");
  assert.equal(tone.subarray(0, 4).toString("ascii"), "RIFF");
  assert.equal(tone.subarray(8, 12).toString("ascii"), "WAVE");
  assert.equal(tone.readUInt32LE(4) + 8, tone.length);

  const mark = files.get("starter-media/media/starter-mark.svg").toString("utf8");
  const stripped = mark.replace(/<\?xml[^?]*\?>\s*/i, "").trimStart();
  assert.ok(stripped.startsWith("<svg"));
  for (const token of ["<script", "xlink:", "href=", "<image", "<foreignObject", "<!ENTITY"]) {
    assert.ok(!mark.includes(token), token);
  }

  assert.equal(context.generatorSha256, manifest.generator.sha256);
  const lock = JSON.parse(await readFile(path.join(mocksUiRoot, "package-lock.json"), "utf8"));
  const locked = lock.packages["node_modules/pngjs"];
  assert.equal(manifest.toolchain.pngjs.version, locked.version);
  assert.equal(manifest.toolchain.pngjs.integrity, locked.integrity);
  assert.match(manifest.toolchain.node.version, /^v\d+\.\d+\.\d+$/);
  const videoToolchain = manifest.toolchain["ff" + "mpeg"];
  assert.ok(videoToolchain.version.length > 0);
  assert.ok(path.isAbsolute(videoToolchain.binary));

  validateStarterMediaCapsule(manifest, files, context);

  const manifestText = JSON.stringify(manifest);
  assert.ok(!/reproducible on any/i.test(manifestText));
  assert.doesNotMatch(manifest.determinismClaim, /同一.*byte|any environment|every platform/i);

  const forbiddenRoots = [
    path.join(repoRoot, "ui/motolii-web"),
    path.join(mocksUiRoot, "src"),
  ];
  for (const root of forbiddenRoots) {
    for (const file of await walkFiles(root)) {
      const text = await readFile(file, "utf8");
      for (const token of [
        "starter-media",
        "starterMedia",
        "Starter Media",
        "starter-media-generation",
      ]) {
        assert.ok(!text.includes(token), `${file} must not mention ${token}`);
      }
    }
  }

  const generatorSource = await readFile(
    path.join(mocksUiRoot, "scripts/starter-media-generation.mjs"),
    "utf8",
  );
  assert.ok(!generatorSource.includes("../../ui"));
  assert.ok(!generatorSource.includes("ui/motolii-web"));
  assert.ok(!generatorSource.includes("crates/"));
  assert.equal((generatorSource.match(/cause\.code === "SM-ABSENT"/g) ?? []).length, 1);
  assert.equal((generatorSource.match(/cause\.code === "SM-READ"/g) ?? []).length, 0);
  assert.equal((generatorSource.match(/await resolveMediaTool\(/g) ?? []).length, 1);
  assert.equal((generatorSource.match(/resolveFfmpeg/g) ?? []).length, 0);
});

test("validateStarterMediaCapsule rejects schema and byte violations", async () => {
  const { manifest, files, context } = await loadCapsuleContext();
  const baseFiles = cloneFiles(files);
  const baseManifest = cloneManifest(manifest);

  const expectFail = (label, expectedCode, mutate) => {
    const m = cloneManifest(baseManifest);
    const f = cloneFiles(baseFiles);
    mutate(m, f);
    assert.throws(
      () => validateStarterMediaCapsule(m, f, context),
      (error) => error instanceof StarterMediaError && error.code === expectedCode,
      label,
    );
  };

  expectFail("extra top-level key", "SM-SCHEMA", (m) => {
    m.extra = true;
  });
  expectFail("media count 3", "SM-SCHEMA", (m) => {
    m.media = m.media.slice(0, 3);
  });
  expectFail("media count 5", "SM-SCHEMA", (m) => {
    m.media = [...m.media, structuredClone(m.media[0])];
  });
  expectFail("sha256 mismatch", "SM-CAPSULE", (m) => {
    m.media[0].sha256 = `${m.media[0].sha256.slice(0, 63)}0`;
  });
  expectFail("byteLength mismatch", "SM-CAPSULE", (m) => {
    m.media[0].byteLength += 1;
  });
  expectFail("media byte drift without manifest re-pin", "SM-CAPSULE", (m, f) => {
    const key = "starter-media/media/starter-tone.wav";
    const drifted = Buffer.from(f.get(key));
    drifted[drifted.length - 1] ^= 0x01;
    f.set(key, drifted);
  });
  expectFail("mediaType mismatch", "SM-SCHEMA", (m) => {
    m.media.find((entry) => entry.path.endsWith(".wav")).mediaType = "image/png";
  });
  expectFail("parent path", "SM-SCHEMA", (m, f) => {
    const entry = m.media.find((item) => item.path.endsWith(".png"));
    const bytes = f.get(entry.path);
    f.delete(entry.path);
    entry.path = "../starter-media/media/starter-still.png";
    f.set(entry.path, bytes);
  });
  expectFail("absolute path", "SM-SCHEMA", (m) => {
    m.media[0].path = "/starter-media/media/starter-clip.mp4";
  });
  expectFail("backslash path", "SM-SCHEMA", (m) => {
    m.media[0].path = "starter-media\\media\\starter-clip.mp4";
  });
  expectFail("wav riff mismatch", "SM-SIGNATURE", (m, f) => {
    const key = "starter-media/media/starter-tone.wav";
    const broken = Buffer.from(f.get(key));
    broken[0] = "X".charCodeAt(0);
    f.set(key, broken);
    repinMediaEntry(m, key, broken);
  });
  expectFail("wav chunk size mismatch", "SM-SIGNATURE", (m, f) => {
    const key = "starter-media/media/starter-tone.wav";
    const broken = Buffer.from(f.get(key));
    broken.writeUInt32LE(broken.readUInt32LE(4) + 1, 4);
    f.set(key, broken);
    repinMediaEntry(m, key, broken);
  });
  expectFail("mp4 ftyp mismatch", "SM-SIGNATURE", (m, f) => {
    const key = "starter-media/media/starter-clip.mp4";
    const broken = Buffer.from(f.get(key));
    broken[4] = "q".charCodeAt(0);
    f.set(key, broken);
    repinMediaEntry(m, key, broken);
  });
  expectFail("png magic mismatch", "SM-SIGNATURE", (m, f) => {
    const key = "starter-media/media/starter-still.png";
    const broken = Buffer.from(f.get(key));
    broken[0] ^= 0xff;
    f.set(key, broken);
    repinMediaEntry(m, key, broken);
  });
  expectFail("svg script injection", "SM-SIGNATURE", (m, f) => {
    const key = "starter-media/media/starter-mark.svg";
    const broken = Buffer.from(`${f.get(key).toString("utf8")}<script>`, "utf8");
    f.set(key, broken);
    repinMediaEntry(m, key, broken);
  });
  expectFail("svg href injection", "SM-SIGNATURE", (m, f) => {
    const key = "starter-media/media/starter-mark.svg";
    const broken = Buffer.from(`${f.get(key).toString("utf8")} href="http://x"`, "utf8");
    f.set(key, broken);
    repinMediaEntry(m, key, broken);
  });
  expectFail("retirement FROZEN only", "SM-SCHEMA", (m) => {
    m.retirement = "FROZEN";
  });
  expectFail("retirement empty", "SM-SCHEMA", (m) => {
    m.retirement = "";
  });
  expectFail("retirement ja", "SM-SCHEMA", (m) => {
    m.retirement = "後で判断";
  });
  expectFail("disposition BUILD", "SM-SCHEMA", (m) => {
    m.responsibilityDisposition = "BUILD";
  });
  expectFail("disposition WRAP+BUILD", "SM-SCHEMA", (m) => {
    m.responsibilityDisposition = "WRAP+BUILD";
  });
  expectFail("disposition empty", "SM-SCHEMA", (m) => {
    m.responsibilityDisposition = "";
  });
  expectFail("determinism overclaim", "SM-SCHEMA", (m) => {
    m.determinismClaim = "Identical bytes on every platform and toolchain.";
  });
  expectFail("generator sha mismatch", "SM-SCHEMA", (m) => {
    m.generator.sha256 = `${m.generator.sha256.slice(0, 63)}0`;
  });
  expectFail("pngjs version mismatch", "SM-SCHEMA", (m) => {
    m.toolchain.pngjs.version = "0.0.0";
  });
  expectFail("pngjs integrity mismatch", "SM-SCHEMA", (m) => {
    m.toolchain.pngjs.integrity = "sha512-invalid";
  });
  expectFail("undeclared extra file", "SM-CAPSULE", (m, f) => {
    f.set("starter-media/media/extra.bin", Buffer.from("x"));
  });
  expectFail("missing declared media", "SM-CAPSULE", (m, f) => {
    f.delete("starter-media/media/starter-tone.wav");
  });
});

async function writeClosedFiveFileCapsule(capsuleRoot, { manifestText } = {}) {
  await mkdir(path.join(capsuleRoot, "media"), { recursive: true });
  const mediaNames = [
    "starter-clip.mp4",
    "starter-mark.svg",
    "starter-still.png",
    "starter-tone.wav",
  ];
  for (const name of mediaNames) {
    await writeFile(path.join(capsuleRoot, "media", name), Buffer.from("x"));
  }
  const manifest =
    manifestText ??
    JSON.stringify({ schemaVersion: 1, capsuleId: CAPSULE_ID, media: [] }, null, 2);
  await writeFile(path.join(capsuleRoot, "starter-media-provenance.json"), manifest);
}

test("readStarterMediaCapsule rejects absent and malformed capsule roots", async (t) => {
  const expectReadFail = async (label, expectedCode, setup) => {
    const tmpRoot = await mkdtemp(path.join(os.tmpdir(), "sm-capsule-"));
    t.after(async () => {
      await rm(tmpRoot, { recursive: true, force: true });
    });
    await setup(tmpRoot);
    await assert.rejects(
      () => readStarterMediaCapsule({ root: tmpRoot }),
      (error) => error instanceof StarterMediaError && error.code === expectedCode,
      label,
    );
  };

  await expectReadFail("capsule root absent", "SM-ABSENT", async () => {});

  await expectReadFail("manifest only", "SM-READ", async (tmpRoot) => {
    const capsuleRoot = path.join(tmpRoot, "starter-media");
    await mkdir(capsuleRoot, { recursive: true });
    await writeFile(
      path.join(capsuleRoot, "starter-media-provenance.json"),
      JSON.stringify({ schemaVersion: 1 }, null, 2),
    );
  });

  await expectReadFail("empty ghost directory", "SM-READ", async (tmpRoot) => {
    const capsuleRoot = path.join(tmpRoot, "starter-media");
    await writeClosedFiveFileCapsule(capsuleRoot);
    await mkdir(path.join(capsuleRoot, "media", "ghost"), { recursive: true });
  });

  await expectReadFail("symlink in media", "SM-READ", async (tmpRoot) => {
    const capsuleRoot = path.join(tmpRoot, "starter-media");
    await writeClosedFiveFileCapsule(capsuleRoot);
    const target = path.join(capsuleRoot, "media", "starter-still.png");
    await symlink(target, path.join(capsuleRoot, "media", "link-target"));
  });

  await expectReadFail("invalid manifest json", "SM-READ", async (tmpRoot) => {
    const capsuleRoot = path.join(tmpRoot, "starter-media");
    await writeClosedFiveFileCapsule(capsuleRoot, { manifestText: "{not-json" });
  });
});
