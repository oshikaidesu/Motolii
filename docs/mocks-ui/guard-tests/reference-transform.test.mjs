import assert from "node:assert/strict";
import test from "node:test";
import { PNG } from "pngjs";
import {
  deriveReferencePng,
  encodeSrgb,
  linearizeSrgb,
  transformRgba,
} from "../scripts/reference-transform.mjs";

test("fixes sRGB thresholds and transparent alpha", () => {
  assert.equal(linearizeSrgb(0.04045), 0.04045 / 12.92);
  assert.equal(encodeSrgb(0.0031308), 12.92 * 0.0031308);
  const input = Uint8Array.from([255, 0, 0, 0, 0, 255, 0, 127, 0, 0, 255, 255]);
  assert.deepEqual([...transformRgba(input, "lightness")], [136, 136, 136, 0, 224, 224, 224, 127, 82, 82, 82, 255]);
  assert.deepEqual([...transformRgba(input, "grayscale")], [127, 127, 127, 0, 220, 220, 220, 127, 76, 76, 76, 255]);
});

test("fixes Machado severity 1.0 vectors", () => {
  const input = Uint8Array.from([255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255]);
  assert.deepEqual([...transformRgba(input, "protanopia")], [109, 95, 0, 255, 255, 229, 0, 255, 0, 89, 255, 255]);
  assert.deepEqual([...transformRgba(input, "deuteranopia")], [163, 144, 0, 255, 239, 214, 58, 255, 0, 61, 251, 255]);
  assert.deepEqual([...transformRgba(input, "tritanopia")], [255, 0, 15, 255, 0, 247, 217, 255, 0, 107, 150, 255]);
});

test("rejects unknown variants, malformed RGBA, and broken PNG", () => {
  assert.throws(() => transformRgba(Uint8Array.from([0, 0, 0, 255]), "normal"), /unknown/);
  assert.throws(() => transformRgba(Uint8Array.from([0, 0, 0]), "grayscale"), /RGBA8/);
  assert.throws(() => deriveReferencePng(Buffer.from("broken"), "grayscale"), /not a PNG/);
  const image = new PNG({ width: 1, height: 1 });
  image.data.set([30, 40, 50, 60]);
  const derived = PNG.sync.read(deriveReferencePng(PNG.sync.write(image), "grayscale"));
  assert.equal(derived.width, 1);
  assert.equal(derived.height, 1);
  assert.equal(derived.data[3], 60);
});
