import { PNG } from "pngjs";

export const REFERENCE_VARIANTS = [
  "normal",
  "lightness",
  "grayscale",
  "protanopia",
  "deuteranopia",
  "tritanopia",
];

export const REFERENCE_TRANSFORM_VERSION = "u0e2-srgb-machado-2009-v1";

const MATRICES = Object.freeze({
  protanopia: [
    0.152286, 1.052583, -0.204868,
    0.114503, 0.786281, 0.099216,
    -0.003882, -0.048116, 1.051998,
  ],
  deuteranopia: [
    0.367322, 0.860646, -0.227968,
    0.280085, 0.672501, 0.047413,
    -0.011820, 0.042940, 0.968881,
  ],
  tritanopia: [
    1.255528, -0.076749, -0.178779,
    -0.078411, 0.930809, 0.147602,
    0.004733, 0.691367, 0.303900,
  ],
});

function clamp(value, min = 0, max = 1) {
  if (!Number.isFinite(value)) throw new TypeError("reference transform produced a non-finite channel");
  return Math.max(min, Math.min(max, value));
}

export function linearizeSrgb(channel) {
  const value = clamp(channel);
  return value <= 0.04045
    ? value / 12.92
    : ((value + 0.055) / 1.055) ** 2.4;
}

export function encodeSrgb(channel) {
  const value = clamp(channel);
  return value <= 0.0031308
    ? 12.92 * value
    : 1.055 * value ** (1 / 2.4) - 0.055;
}

function toByte(channel) {
  return Math.round(clamp(channel) * 255);
}

function luminance(red, green, blue) {
  return 0.2126 * red + 0.7152 * green + 0.0722 * blue;
}

function lightness(linearY) {
  const delta = 6 / 29;
  const f = linearY > delta ** 3
    ? Math.cbrt(linearY)
    : linearY / (3 * delta ** 2) + 4 / 29;
  return clamp((116 * f - 16) / 100);
}

export function transformRgba(input, variant) {
  if (!(input instanceof Uint8Array) || input.length === 0 || input.length % 4 !== 0) {
    throw new TypeError("reference transform requires non-empty RGBA8 bytes");
  }
  if (!REFERENCE_VARIANTS.includes(variant) || variant === "normal") {
    throw new TypeError(`unknown derived reference variant ${variant}`);
  }
  const output = Buffer.alloc(input.length);
  const matrix = MATRICES[variant];
  for (let offset = 0; offset < input.length; offset += 4) {
    const red = linearizeSrgb(input[offset] / 255);
    const green = linearizeSrgb(input[offset + 1] / 255);
    const blue = linearizeSrgb(input[offset + 2] / 255);
    if (variant === "lightness" || variant === "grayscale") {
      const y = luminance(red, green, blue);
      const encoded = variant === "lightness" ? lightness(y) : encodeSrgb(y);
      const byte = toByte(encoded);
      output[offset] = byte;
      output[offset + 1] = byte;
      output[offset + 2] = byte;
    } else {
      const transformed = [
        matrix[0] * red + matrix[1] * green + matrix[2] * blue,
        matrix[3] * red + matrix[4] * green + matrix[5] * blue,
        matrix[6] * red + matrix[7] * green + matrix[8] * blue,
      ];
      output[offset] = toByte(encodeSrgb(transformed[0]));
      output[offset + 1] = toByte(encodeSrgb(transformed[1]));
      output[offset + 2] = toByte(encodeSrgb(transformed[2]));
    }
    output[offset + 3] = input[offset + 3];
  }
  return output;
}

export function deriveReferencePng(normalBytes, variant) {
  let normal;
  try {
    normal = PNG.sync.read(normalBytes);
  } catch (error) {
    throw new TypeError(`normal reference capture is not a PNG: ${error.message}`);
  }
  if (!Number.isInteger(normal.width) || normal.width <= 0 || !Number.isInteger(normal.height) || normal.height <= 0) {
    throw new TypeError("normal reference capture has invalid dimensions");
  }
  const derived = new PNG({ width: normal.width, height: normal.height });
  derived.data.set(transformRgba(normal.data, variant));
  return PNG.sync.write(derived);
}

export function decodedPixels(pngBytes) {
  const image = PNG.sync.read(pngBytes);
  return Buffer.concat([
    Buffer.from(`${image.width}x${image.height}\0`),
    Buffer.from(image.data),
  ]);
}
