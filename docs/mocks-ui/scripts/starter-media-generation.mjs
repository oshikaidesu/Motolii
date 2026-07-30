import { createHash } from "node:crypto";
import { execFile } from "node:child_process";
import {
  lstat,
  mkdir,
  open,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";
import { PNG } from "pngjs";

const execFileAsync = promisify(execFile);
const SCRIPT_DIR = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(SCRIPT_DIR, "..");
const CAPSULE_DIR = path.join(ROOT, "starter-media");
const MANIFEST_REL = "starter-media/starter-media-provenance.json";
const GENERATOR_REL = "scripts/starter-media-generation.mjs";

const SHA256 = /^[0-9a-f]{64}$/;
const PNG_MAGIC = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

const MANIFEST_FIELDS = new Set([
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
]);
const GENERATOR_FIELDS = new Set(["path", "sha256"]);
const NODE_TOOL_FIELDS = new Set(["version"]);
const PNGJS_TOOL_FIELDS = new Set(["package", "version", "integrity"]);
const FFMPEG_TOOL_FIELDS = new Set(["binary", "version"]);
const TOOLCHAIN_FIELDS = new Set(["node", "pngjs", "ffmpeg"]);
const MEDIA_FIELDS = new Set(["path", "mediaType", "byteLength", "sha256", "origin", "purpose"]);

const CAPSULE_ID = "g0-6h-ag-starter-media";
const SCOPE =
  "project-external fixture-only; not a Project asset, Document, public API, plugin contract, persisted format, or production Registered folder";
const RESPONSIBILITY = "WRAP";
const RETIREMENT = "FROZEN / DELETE-LATER";
const DETERMINISM_CLAIM =
  "byte determinism is claimed only for the recorded toolchain on the recording host; cross-platform, cross-OS, and cross-toolchain byte determinism is not claimed";

const MEDIA_SPECS = [
  {
    rel: "starter-media/media/starter-clip.mp4",
    mediaType: "video/mp4",
    ext: ".mp4",
  },
  {
    rel: "starter-media/media/starter-mark.svg",
    mediaType: "image/svg+xml",
    ext: ".svg",
  },
  {
    rel: "starter-media/media/starter-still.png",
    mediaType: "image/png",
    ext: ".png",
  },
  {
    rel: "starter-media/media/starter-tone.wav",
    mediaType: "audio/wav",
    ext: ".wav",
  },
];

const EXT_TO_TYPE = {
  ".png": "image/png",
  ".mp4": "video/mp4",
  ".wav": "audio/wav",
  ".svg": "image/svg+xml",
};

export class StarterMediaError extends Error {
  constructor(code, message, options) {
    super(`${code}: ${message}`, options);
    this.name = "StarterMediaError";
    this.code = code;
  }
}

function reject(code, message, cause) {
  throw new StarterMediaError(code, message, cause ? { cause } : undefined);
}

function sha256(bytes) {
  return createHash("sha256").update(bytes).digest("hex");
}

function exactFields(value, fields, owner) {
  if (!value || typeof value !== "object" || Array.isArray(value)) {
    reject("SM-SCHEMA", `${owner} must be an object`);
  }
  const actual = Object.keys(value).sort();
  const expected = [...fields].sort();
  if (actual.join("\0") !== expected.join("\0")) {
    reject("SM-SCHEMA", `${owner} fields are outside the closed schema`);
  }
}

function safeMediaPath(value) {
  if (typeof value !== "string" || value.includes("\\")) {
    reject("SM-SCHEMA", "media path must use forward slashes");
  }
  const normalized = path.posix.normalize(value);
  if (
    normalized !== value ||
    !normalized.startsWith("starter-media/media/") ||
    normalized.includes("/../") ||
    path.posix.isAbsolute(normalized)
  ) {
    reject("SM-SCHEMA", `unsafe media path ${JSON.stringify(value)}`);
  }
  return normalized;
}

function assertMediaSignature(mediaPath, bytes) {
  if (mediaPath.endsWith(".png")) {
    if (!bytes.subarray(0, 8).equals(PNG_MAGIC)) {
      reject("SM-SIGNATURE", `${mediaPath} is not a valid PNG`);
    }
    let png;
    try {
      png = PNG.sync.read(bytes);
    } catch (cause) {
      reject("SM-SIGNATURE", `${mediaPath} is not a valid PNG`, cause);
    }
    if (png.width !== 640 || png.height !== 360) {
      reject("SM-SIGNATURE", `${mediaPath} must be 640x360`);
    }
    return;
  }
  if (mediaPath.endsWith(".mp4")) {
    if (bytes.subarray(4, 8).toString("ascii") !== "ftyp") {
      reject("SM-SIGNATURE", `${mediaPath} is not a valid MP4`);
    }
    return;
  }
  if (mediaPath.endsWith(".wav")) {
    if (bytes.subarray(0, 4).toString("ascii") !== "RIFF") {
      reject("SM-SIGNATURE", `${mediaPath} is not a valid WAV`);
    }
    if (bytes.subarray(8, 12).toString("ascii") !== "WAVE") {
      reject("SM-SIGNATURE", `${mediaPath} is not a valid WAV`);
    }
    const chunkSize = bytes.readUInt32LE(4);
    if (chunkSize + 8 !== bytes.length) {
      reject("SM-SIGNATURE", `${mediaPath} WAV chunk size mismatch`);
    }
    return;
  }
  if (mediaPath.endsWith(".svg")) {
    const text = bytes.toString("utf8");
    const stripped = text.replace(/<\?xml[^?]*\?>\s*/i, "").trimStart();
    if (!stripped.startsWith("<svg")) {
      reject("SM-SIGNATURE", `${mediaPath} must start with <svg`);
    }
    const forbidden = [
      "<script",
      "xlink:",
      "href=",
      "<image",
      "<foreignObject",
      "<!ENTITY",
    ];
    for (const token of forbidden) {
      if (text.includes(token)) {
        reject("SM-SIGNATURE", `${mediaPath} contains forbidden SVG token ${token}`);
      }
    }
    const withoutNs = text.replace(/xmlns="http:\/\/www\.w3\.org\/2000\/svg"/g, "");
    if (withoutNs.includes("http")) {
      reject("SM-SIGNATURE", `${mediaPath} contains disallowed http reference`);
    }
  }
}

function assertDeterminismClaim(claim) {
  if (claim !== DETERMINISM_CLAIM) {
    reject("SM-SCHEMA", "determinismClaim must match the fixed capsule wording");
  }
  const lower = String(claim).toLowerCase();
  if (
    lower.includes("reproducible on any") ||
    (lower.includes("cross-platform") && !DETERMINISM_CLAIM.includes(claim))
  ) {
    reject("SM-SCHEMA", "determinismClaim must not assert cross-environment byte identity");
  }
  if (/同一.*byte|any environment|every platform/i.test(claim)) {
    reject("SM-SCHEMA", "determinismClaim overclaims byte determinism");
  }
}

export function validateStarterMediaCapsule(manifest, files, context) {
  if (!context || typeof context !== "object") {
    reject("SM-SCHEMA", "context is required");
  }
  const { lockedPngjs, generatorSha256 } = context;
  if (
    !lockedPngjs ||
    typeof lockedPngjs.version !== "string" ||
    typeof lockedPngjs.integrity !== "string" ||
    typeof generatorSha256 !== "string" ||
    !SHA256.test(generatorSha256)
  ) {
    reject("SM-SCHEMA", "context must include locked pngjs and generator SHA-256");
  }

  exactFields(manifest, MANIFEST_FIELDS, "manifest");
  if (manifest.schemaVersion !== 1) reject("SM-SCHEMA", "schemaVersion must be 1");
  if (manifest.capsuleId !== CAPSULE_ID) reject("SM-SCHEMA", "capsuleId mismatch");
  if (typeof manifest.purpose !== "string" || manifest.purpose.length === 0) {
    reject("SM-SCHEMA", "purpose is required");
  }
  if (manifest.scope !== SCOPE) reject("SM-SCHEMA", "scope mismatch");
  if (manifest.responsibilityDisposition !== RESPONSIBILITY) {
    reject("SM-SCHEMA", "responsibilityDisposition must be WRAP");
  }
  if (manifest.retirement !== RETIREMENT) {
    reject("SM-SCHEMA", "retirement must be FROZEN / DELETE-LATER");
  }
  assertDeterminismClaim(manifest.determinismClaim);

  exactFields(manifest.generator, GENERATOR_FIELDS, "generator");
  if (manifest.generator.path !== GENERATOR_REL) {
    reject("SM-SCHEMA", "generator.path mismatch");
  }
  if (manifest.generator.sha256 !== generatorSha256) {
    reject("SM-SCHEMA", "generator.sha256 mismatch");
  }

  exactFields(manifest.toolchain, TOOLCHAIN_FIELDS, "toolchain");
  exactFields(manifest.toolchain.node, NODE_TOOL_FIELDS, "toolchain.node");
  if (!/^v\d+\.\d+\.\d+$/.test(manifest.toolchain.node.version ?? "")) {
    reject("SM-SCHEMA", "toolchain.node.version invalid");
  }
  exactFields(manifest.toolchain.pngjs, PNGJS_TOOL_FIELDS, "toolchain.pngjs");
  if (manifest.toolchain.pngjs.package !== "pngjs") {
    reject("SM-SCHEMA", "toolchain.pngjs.package must be pngjs");
  }
  if (
    manifest.toolchain.pngjs.version !== lockedPngjs.version ||
    manifest.toolchain.pngjs.integrity !== lockedPngjs.integrity
  ) {
    reject("SM-SCHEMA", "toolchain.pngjs does not match locked package-lock entry");
  }
  exactFields(manifest.toolchain.ffmpeg, FFMPEG_TOOL_FIELDS, "toolchain.ffmpeg");
  if (
    typeof manifest.toolchain.ffmpeg.binary !== "string" ||
    !path.isAbsolute(manifest.toolchain.ffmpeg.binary)
  ) {
    reject("SM-SCHEMA", "toolchain.ffmpeg.binary must be an absolute path");
  }
  if (
    typeof manifest.toolchain.ffmpeg.version !== "string" ||
    manifest.toolchain.ffmpeg.version.length === 0
  ) {
    reject("SM-SCHEMA", "toolchain.ffmpeg.version is required");
  }

  if (!Array.isArray(manifest.media) || manifest.media.length !== 4) {
    reject("SM-SCHEMA", "media must contain exactly four entries");
  }

  const fileMap =
    files instanceof Map
      ? files
      : new Map(Object.entries(files ?? {}));
  const declaredPaths = new Set();
  const sortedPaths = manifest.media.map((entry) => entry.path);
  const canonicalPaths = [...sortedPaths].sort();
  if (sortedPaths.join("\0") !== canonicalPaths.join("\0")) {
    reject("SM-SCHEMA", "media must be sorted by path ascending");
  }

  for (const [index, entry] of manifest.media.entries()) {
    exactFields(entry, MEDIA_FIELDS, `media[${index}]`);
    const mediaPath = safeMediaPath(entry.path);
    if (declaredPaths.has(mediaPath)) {
      reject("SM-SCHEMA", `duplicate media path ${mediaPath}`);
    }
    declaredPaths.add(mediaPath);
    const ext = path.posix.extname(mediaPath);
    if (EXT_TO_TYPE[ext] !== entry.mediaType) {
      reject("SM-SCHEMA", `${mediaPath} mediaType does not match extension`);
    }
    if (!Number.isInteger(entry.byteLength) || entry.byteLength < 0) {
      reject("SM-SCHEMA", `${mediaPath} byteLength invalid`);
    }
    if (!SHA256.test(entry.sha256 ?? "")) {
      reject("SM-SCHEMA", `${mediaPath} sha256 invalid`);
    }
    if (typeof entry.origin !== "string" || entry.origin.length === 0) {
      reject("SM-SCHEMA", `${mediaPath} origin required`);
    }
    if (typeof entry.purpose !== "string" || entry.purpose.length === 0) {
      reject("SM-SCHEMA", `${mediaPath} purpose required`);
    }
    const bytes = fileMap.get(mediaPath);
    if (!bytes || !Buffer.isBuffer(bytes)) {
      reject("SM-CAPSULE", `missing bytes for ${mediaPath}`);
    }
    if (bytes.length !== entry.byteLength) {
      reject("SM-CAPSULE", `${mediaPath} byteLength mismatch`);
    }
    if (sha256(bytes) !== entry.sha256) {
      reject("SM-CAPSULE", `${mediaPath} sha256 mismatch`);
    }
    assertMediaSignature(mediaPath, bytes);
  }

  if (fileMap.size !== declaredPaths.size) {
    reject("SM-CAPSULE", "undeclared media bytes in files map");
  }
  for (const key of fileMap.keys()) {
    safeMediaPath(key);
    if (!declaredPaths.has(key)) {
      reject("SM-CAPSULE", `undeclared file ${key}`);
    }
  }
}

async function listCapsuleFiles(directory, prefix = "") {
  const result = [];
  const entries = await readdir(directory, { withFileTypes: true });
  for (const entry of entries.sort((left, right) => left.name.localeCompare(right.name))) {
    const relative = prefix ? `${prefix}/${entry.name}` : entry.name;
    const filename = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      const nested = await listCapsuleFiles(filename, relative);
      if (nested.length === 0) {
        reject("SM-READ", `empty capsule directory ${relative}`);
      }
      result.push(...nested);
    } else if (entry.isFile()) {
      result.push(relative);
    } else {
      reject("SM-READ", `non-file capsule entry ${relative}`);
    }
  }
  return result;
}

export async function readStarterMediaCapsule({ root }) {
  root = path.resolve(root ?? ROOT);
  const capsuleRoot = path.join(root, "starter-media");
  try {
    const rootStat = await lstat(capsuleRoot);
    if (!rootStat.isDirectory()) {
      reject("SM-READ", "capsule root is not a directory");
    }
  } catch (cause) {
    if (cause instanceof StarterMediaError) throw cause;
    if (cause && typeof cause === "object" && "code" in cause && cause.code === "ENOENT") {
      reject("SM-ABSENT", "capsule root absent");
    }
    reject("SM-READ", `cannot stat capsule root: ${cause.message}`, cause);
  }
  let relFiles;
  try {
    relFiles = await listCapsuleFiles(capsuleRoot);
  } catch (cause) {
    if (cause instanceof StarterMediaError) throw cause;
    reject("SM-READ", `cannot list capsule: ${cause.message}`, cause);
  }
  const expectedRel = [
    "media/starter-clip.mp4",
    "media/starter-mark.svg",
    "media/starter-still.png",
    "media/starter-tone.wav",
    "starter-media-provenance.json",
  ].sort();
  const actualRel = [...relFiles].sort();
  if (actualRel.join("\0") !== expectedRel.join("\0")) {
    reject("SM-READ", "capsule directory is not a closed five-file set");
  }

  let manifest;
  try {
    manifest = JSON.parse(
      await readFile(path.join(capsuleRoot, "starter-media-provenance.json"), "utf8"),
    );
  } catch (cause) {
    reject("SM-READ", `cannot read manifest: ${cause.message}`, cause);
  }

  const files = new Map();
  for (const rel of relFiles) {
    if (rel === "starter-media-provenance.json") continue;
    const fullRel = `starter-media/${rel}`;
    files.set(fullRel, await readFile(path.join(capsuleRoot, rel)));
  }
  return { manifest, files };
}

function stillPixel(x, y) {
  const r = (x * 3 + y * 5) % 256;
  const g = (x * 7 + y * 11) % 256;
  const b = (x * 13 + y * 17) % 256;
  return { r, g, b, a: 255 };
}

function buildStillPng() {
  const width = 640;
  const height = 360;
  const png = new PNG({ width, height });
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      const idx = (width * y + x) << 2;
      const { r, g, b, a } = stillPixel(x, y);
      png.data[idx] = r;
      png.data[idx + 1] = g;
      png.data[idx + 2] = b;
      png.data[idx + 3] = a;
    }
  }
  return PNG.sync.write(png);
}

function buildMarkSvg() {
  return Buffer.from(
    `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">
<rect x="8" y="8" width="48" height="48" fill="none" stroke="#334" stroke-width="4"/>
<circle cx="32" cy="32" r="12" fill="#556"/>
</svg>
`,
    "utf8",
  );
}

async function resolveMediaTool() {
  let binary;
  try {
    const { stdout } = await execFileAsync("which", ["ffmpeg"], { encoding: "utf8" });
    binary = stdout.trim();
  } catch (cause) {
    reject("SM-TOOLCHAIN", "ffmpeg not found on PATH", cause);
  }
  if (!binary) reject("SM-TOOLCHAIN", "ffmpeg not found on PATH");
  let versionLine;
  try {
    const { stdout } = await execFileAsync(binary, ["-version"], { encoding: "utf8" });
    versionLine = stdout.split("\n")[0] ?? "";
  } catch (cause) {
    reject("SM-TOOLCHAIN", "ffmpeg -version failed", cause);
  }
  return { binary, version: versionLine };
}

async function runFfmpegClip(binary, outputPath) {
  await execFileAsync(
    binary,
    [
      "-hide_banner",
      "-nostdin",
      "-y",
      "-fflags",
      "+bitexact",
      "-flags:v",
      "+bitexact",
      "-f",
      "lavfi",
      "-i",
      "testsrc2=size=320x180:rate=15:duration=2",
      "-c:v",
      "libx264",
      "-preset",
      "veryslow",
      "-pix_fmt",
      "yuv420p",
      "-movflags",
      "+faststart",
      outputPath,
    ],
    { maxBuffer: 16 * 1024 * 1024 },
  );
}

async function runFfmpegTone(binary, outputPath) {
  await execFileAsync(
    binary,
    [
      "-hide_banner",
      "-nostdin",
      "-y",
      "-fflags",
      "+bitexact",
      "-f",
      "lavfi",
      "-i",
      "sine=frequency=440:sample_rate=48000:duration=2",
      "-c:a",
      "pcm_s16le",
      outputPath,
    ],
    { maxBuffer: 16 * 1024 * 1024 },
  );
}

async function loadLockedPngjs() {
  const lock = JSON.parse(await readFile(path.join(ROOT, "package-lock.json"), "utf8"));
  const locked = lock.packages?.["node_modules/pngjs"];
  if (!locked?.version || !locked?.integrity) {
    reject("SM-TOOLCHAIN", "package-lock pngjs entry missing version or integrity");
  }
  return { version: locked.version, integrity: locked.integrity };
}

async function syncFile(filename) {
  const handle = await open(filename, "r");
  try {
    await handle.sync();
  } finally {
    await handle.close();
  }
}

function buildManifest({ mediaEntries, generatorSha256, lockedPngjs, toolchain }) {
  return {
    schemaVersion: 1,
    capsuleId: CAPSULE_ID,
    purpose:
      "Fixture-only evidence that project-external starter media bytes exist for the G0-6H empty-project scenario without registering them as Project assets.",
    scope: SCOPE,
    responsibilityDisposition: RESPONSIBILITY,
    retirement: RETIREMENT,
    determinismClaim: DETERMINISM_CLAIM,
    generator: {
      path: GENERATOR_REL,
      sha256: generatorSha256,
    },
    toolchain: {
      node: { version: process.version },
      pngjs: {
        package: "pngjs",
        version: lockedPngjs.version,
        integrity: lockedPngjs.integrity,
      },
      ffmpeg: {
        binary: toolchain.binary,
        version: toolchain.version,
      },
    },
    media: mediaEntries,
  };
}

function mediaEntry(rel, mediaType, bytes, origin, purpose) {
  return {
    path: rel,
    mediaType,
    byteLength: bytes.length,
    sha256: sha256(bytes),
    origin,
    purpose,
  };
}

async function computeBundle() {
  const lockedPngjs = await loadLockedPngjs();
  const toolchain = await resolveMediaTool();
  const generatorBytes = await readFile(path.join(ROOT, GENERATOR_REL));
  const generatorSha256 = sha256(generatorBytes);

  const still = buildStillPng();
  const mark = buildMarkSvg();
  const tmpDir = path.join(os.tmpdir(), `starter-media-${process.pid}-${Date.now()}`);
  await mkdir(tmpDir, { recursive: true });
  const clipPath = path.join(tmpDir, "starter-clip.mp4");
  const tonePath = path.join(tmpDir, "starter-tone.wav");
  try {
    await runFfmpegClip(toolchain.binary, clipPath);
    await runFfmpegTone(toolchain.binary, tonePath);
    const clip = await readFile(clipPath);
    const tone = await readFile(tonePath);

    const mediaEntries = [
      mediaEntry(
        "starter-media/media/starter-clip.mp4",
        "video/mp4",
        clip,
        "ffmpeg lavfi testsrc2 320x180 15fps 2.0s libx264 yuv420p +faststart bitexact",
        "Short H.264 clip proving an external video byte is available outside Project scope.",
      ),
      mediaEntry(
        "starter-media/media/starter-mark.svg",
        "image/svg+xml",
        mark,
        "literal SVG string in starter-media-generation.mjs viewBox 0 0 64 64 ASCII-only",
        "Minimal vector mark showing declarative non-raster media exists as fixture bytes.",
      ),
      mediaEntry(
        "starter-media/media/starter-still.png",
        "image/png",
        still,
        "pngjs PNG.sync.write 640x360 RGBA from pure (x,y) function bitexact",
        "Still image bytes demonstrating pngjs-generated raster media outside Project registration.",
      ),
      mediaEntry(
        "starter-media/media/starter-tone.wav",
        "audio/wav",
        tone,
        "ffmpeg lavfi sine 440Hz 48000Hz 2.0s pcm_s16le bitexact",
        "PCM WAV tone proving audio fixture bytes exist without AUDIO LIBRARY registration.",
      ),
    ];

    const manifest = buildManifest({
      mediaEntries,
      generatorSha256,
      lockedPngjs,
      toolchain,
    });
    const files = new Map(
      mediaEntries.map((entry) => [entry.path, fileBytesForEntry(entry, { still, mark, clip, tone })]),
    );
    validateStarterMediaCapsule(manifest, files, { lockedPngjs, generatorSha256 });
    const manifestBytes = Buffer.from(`${JSON.stringify(manifest, null, 2)}\n`);
    return { manifest, manifestBytes, files };
  } finally {
    await rm(tmpDir, { recursive: true, force: true }).catch(() => {});
  }
}

function fileBytesForEntry(entry, blobs) {
  switch (entry.path) {
    case "starter-media/media/starter-still.png":
      return blobs.still;
    case "starter-media/media/starter-mark.svg":
      return blobs.mark;
    case "starter-media/media/starter-clip.mp4":
      return blobs.clip;
    case "starter-media/media/starter-tone.wav":
      return blobs.tone;
    default:
      reject("SM-INTERNAL", `unknown media path ${entry.path}`);
  }
}

async function bundlesEqual(existing, computed) {
  const existingManifestBytes = await readFile(path.join(CAPSULE_DIR, "starter-media-provenance.json"));
  if (!existingManifestBytes.equals(computed.manifestBytes)) {
    return false;
  }
  for (const [mediaPath, bytes] of computed.files) {
    const rel = mediaPath.replace(/^starter-media\//, "");
    const onDisk = await readFile(path.join(CAPSULE_DIR, rel));
    if (!onDisk.equals(bytes)) return false;
  }
  return true;
}

async function publishCapsule({ manifestBytes, files }) {
  const nonce = `${process.pid}-${Date.now()}`;
  const stage = path.join(ROOT, `.starter-media-stage-${nonce}`);
  let published = false;
  try {
    await mkdir(path.join(stage, "media"), { recursive: true });
    for (const spec of MEDIA_SPECS) {
      const relInCapsule = spec.rel.replace(/^starter-media\//, "");
      const filename = path.join(stage, relInCapsule);
      await writeFile(filename, files.get(spec.rel), { flag: "wx" });
      await syncFile(filename);
    }
    await syncFile(path.join(stage, "media"));
    const manifestPath = path.join(stage, "starter-media-provenance.json");
    await writeFile(manifestPath, manifestBytes, { flag: "wx" });
    await syncFile(manifestPath);
    await syncFile(stage);
    try {
      await lstat(CAPSULE_DIR);
      await rm(stage, { recursive: true, force: true }).catch(() => {});
      reject("SM-CONFLICT", "capsule directory already exists");
    } catch (cause) {
      if (cause instanceof StarterMediaError) throw cause;
      if (!(cause && typeof cause === "object" && "code" in cause && cause.code === "ENOENT")) {
        throw cause;
      }
    }
    await rename(stage, CAPSULE_DIR);
    published = true;
    await syncFile(path.join(ROOT, "starter-media"));
    await syncFile(ROOT);
  } catch (cause) {
    if (!published) await rm(stage, { recursive: true, force: true }).catch(() => {});
    if (cause instanceof StarterMediaError) throw cause;
    reject("SM-PUBLISH", `publication failed: ${cause.message}`, cause);
  }
}

async function generateCommand() {
  const computed = await computeBundle();
  try {
    const existing = await readStarterMediaCapsule({ root: ROOT });
    validateStarterMediaCapsule(existing.manifest, existing.files, {
      lockedPngjs: {
        version: computed.manifest.toolchain.pngjs.version,
        integrity: computed.manifest.toolchain.pngjs.integrity,
      },
      generatorSha256: computed.manifest.generator.sha256,
    });
    if (await bundlesEqual(existing, computed)) {
      console.log("starter-media capsule already current");
      return;
    }
    const diffs = [];
    const existingManifestBytes = await readFile(
      path.join(CAPSULE_DIR, "starter-media-provenance.json"),
    );
    if (!existingManifestBytes.equals(computed.manifestBytes)) {
      diffs.push(MANIFEST_REL);
    }
    for (const [mediaPath, expected] of computed.files) {
      const rel = mediaPath.replace(/^starter-media\//, "");
      const onDisk = await readFile(path.join(CAPSULE_DIR, rel));
      if (!onDisk.equals(expected)) diffs.push(mediaPath);
    }
    reject("SM-CONFLICT", `existing capsule differs; refusing overwrite: ${diffs.join(", ")}`);
  } catch (cause) {
    if (cause instanceof StarterMediaError && cause.code === "SM-ABSENT") {
      await publishCapsule(computed);
      console.log("starter-media capsule published");
      return;
    }
    if (cause instanceof StarterMediaError && cause.code === "SM-CONFLICT") {
      throw cause;
    }
    if (cause instanceof StarterMediaError) {
      reject("SM-CONFLICT", "existing capsule is incomplete or invalid", cause);
    }
    throw cause;
  }
}

async function main() {
  const command = process.argv[2];
  if (command !== "generate") {
    console.error("usage: node scripts/starter-media-generation.mjs generate");
    process.exit(1);
  }
  try {
    await generateCommand();
  } catch (cause) {
    console.error(cause instanceof StarterMediaError ? cause.message : cause);
    process.exit(1);
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  main();
}
