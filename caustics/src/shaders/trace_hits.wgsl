// DERIVED from visgl/luma.gl spectral-caustics-renderer-shaders.ts (MIT, Copyright (c) vis.gl contributors).
// See third_party/luma.gl-LICENSE. Changed: output (per-texel receiver hits instead of splat records).
// Borrowed from visgl/luma.gl modules/experimental/src/rendering/spectral-caustics-renderer-shaders.ts
// SPDX-License-Identifier: MIT   Copyright (c) vis.gl contributors   (see third_party/luma.gl-LICENSE)
// Only the template constants were substituted; the WGSL is unchanged.
const WAVELENGTH_COUNT: u32 = 6u;
const MARCH_STEP_COUNT: u32 = 16u;

struct Hit {
  newUv: vec2<f32>,
  oldUv: vec2<f32>,
  valid: f32,
  transmission: f32,
  padding: vec2<f32>,
};

struct SpectralCausticsUniforms {
  inverseLightViewProjection: mat4x4<f32>,
  lightViewProjection: mat4x4<f32>,
  receiverOriginWidth: vec4<f32>,
  receiverTangentHeight: vec4<f32>,
  receiverBitangentIntensity: vec4<f32>,
  receiverNormalRefractiveIndex: vec4<f32>,
  absorptionDispersion: vec4<f32>,
  targetSizes: vec4<f32>,
};

struct ExitHit {
  position: vec3<f32>,
  valid: f32,
  normal: vec3<f32>,
  padding: f32,
};

@group(0) @binding(0) var frontNormalTexture: texture_2d<f32>;
@group(0) @binding(1) var frontDepthTexture: texture_depth_2d;
@group(0) @binding(2) var backNormalTexture: texture_2d<f32>;
@group(0) @binding(3) var backDepthTexture: texture_depth_2d;
@group(0) @binding(4) var<storage, read_write> hits: array<Hit>;
@group(0) @binding(5) var<uniform> uniforms: SpectralCausticsUniforms;

const WAVELENGTHS_NANOMETERS = array<f32, 6>(
  664.46, 618.64, 574.60, 528.33, 468.28, 434.72
);

// CIE 1931 2-degree CMFs integrated against D65 in six bands and normalized to Y = 1.
const WAVELENGTH_XYZ = array<vec3<f32>, 6>(
  vec3<f32>(0.03671527, 0.01357357, 0.00000000),
  vec3<f32>(0.30218842, 0.14160332, 0.00008598),
  vec3<f32>(0.36062779, 0.40239216, 0.00124922),
  vec3<f32>(0.07092146, 0.36745709, 0.04500246),
  vec3<f32>(0.08806719, 0.06954351, 0.58510595),
  vec3<f32>(0.09195043, 0.00543034, 0.45738513)
);

const WAVELENGTH_ABSORPTION_WEIGHTS = array<vec3<f32>, 6>(
  vec3<f32>(1.0, 0.0, 0.0),
  vec3<f32>(0.78, 0.22, 0.0),
  vec3<f32>(0.28, 0.72, 0.0),
  vec3<f32>(0.0, 0.82, 0.18),
  vec3<f32>(0.0, 0.28, 0.72),
  vec3<f32>(0.12, 0.0, 0.88)
);

fn reconstructWorldPosition(uv: vec2<f32>, depth: f32) -> vec3<f32> {
  let clipPosition = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
  let worldPosition = uniforms.inverseLightViewProjection * clipPosition;
  return worldPosition.xyz / max(abs(worldPosition.w), 0.000001);
}

fn projectWorldPosition(worldPosition: vec3<f32>) -> vec3<f32> {
  let clipPosition = uniforms.lightViewProjection * vec4<f32>(worldPosition, 1.0);
  let inverseW = 1.0 / max(abs(clipPosition.w), 0.000001);
  let normalized = clipPosition.xyz * inverseW;
  return vec3<f32>(normalized.x * 0.5 + 0.5, 0.5 - normalized.y * 0.5, normalized.z);
}

fn decodeNormal(encodedNormal: vec3<f32>) -> vec3<f32> {
  let normal = encodedNormal * 2.0 - vec3<f32>(1.0);
  return normal * inverseSqrt(max(dot(normal, normal), 0.000001));
}

fn getCapturePixel(uv: vec2<f32>, captureSize: i32) -> vec2<i32> {
  return clamp(
    vec2<i32>(uv * f32(captureSize)),
    vec2<i32>(0),
    vec2<i32>(captureSize - 1)
  );
}

fn findExitSurface(
  entryPosition: vec3<f32>,
  insideDirection: vec3<f32>,
  sourceUv: vec2<f32>,
  captureSize: i32
) -> ExitHit {
  let sourcePixel = getCapturePixel(sourceUv, captureSize);
  let sourceBackDepth = textureLoad(backDepthTexture, sourcePixel, 0);
  if (sourceBackDepth <= 0.000001) {
    return ExitHit(vec3<f32>(0.0), 0.0, vec3<f32>(0.0), 0.0);
  }

  let sourceBackPosition = reconstructWorldPosition(sourceUv, sourceBackDepth);
  let maximumDistance = max(distance(entryPosition, sourceBackPosition) * 1.8, 0.01);
  var previousPosition = entryPosition;

  for (var marchStep = 1u; marchStep <= MARCH_STEP_COUNT; marchStep++) {
    let distanceFraction = f32(marchStep) / f32(MARCH_STEP_COUNT);
    let candidatePosition = entryPosition + insideDirection * maximumDistance * distanceFraction;
    let projected = projectWorldPosition(candidatePosition);
    if (
      projected.x < 0.0 || projected.x > 1.0 ||
      projected.y < 0.0 || projected.y > 1.0 ||
      projected.z < 0.0 || projected.z > 1.0
    ) {
      break;
    }

    let backPixel = getCapturePixel(projected.xy, captureSize);
    let backDepth = textureLoad(backDepthTexture, backPixel, 0);
    if (backDepth > 0.000001 && projected.z >= backDepth - 0.0015) {
      let backUv = (vec2<f32>(backPixel) + vec2<f32>(0.5)) / f32(captureSize);
      let backPosition = reconstructWorldPosition(backUv, backDepth);
      let encodedNormal = textureLoad(backNormalTexture, backPixel, 0).xyz;
      return ExitHit(backPosition, 1.0, decodeNormal(encodedNormal), 0.0);
    }
    previousPosition = candidatePosition;
  }

  // Permit the same-pixel fallback only when it remains close to the refracted ray. This preserves
  // nearly axial rays without turning unrelated silhouette backfaces into long caustic streaks.
  let fallbackOffset = sourceBackPosition - entryPosition;
  let fallbackDistance = dot(fallbackOffset, insideDirection);
  let fallbackLateralError = length(fallbackOffset - insideDirection * fallbackDistance);
  let fallbackTolerance = max(length(fallbackOffset) * 0.12, 0.01);
  let fallbackNormal = decodeNormal(textureLoad(backNormalTexture, sourcePixel, 0).xyz);
  let fallbackValid = fallbackDistance > 0.0 &&
    fallbackLateralError <= fallbackTolerance &&
    distance(previousPosition, entryPosition) > 0.0;
  return ExitHit(sourceBackPosition, select(0.0, 1.0, fallbackValid), fallbackNormal, 0.0);
}

fn getRefractiveIndex(wavelengthNanometers: f32) -> f32 {
  let referenceRatio = 550.0 / wavelengthNanometers;
  return max(
    uniforms.receiverNormalRefractiveIndex.w +
      uniforms.absorptionDispersion.w * (referenceRatio * referenceRatio - 1.0),
    1.0001
  );
}

fn writeInvalid(sampleIndex: u32) {
  for (var wavelengthIndex = 0u; wavelengthIndex < WAVELENGTH_COUNT; wavelengthIndex++) {
    hits[sampleIndex * WAVELENGTH_COUNT + wavelengthIndex] = Hit(vec2<f32>(0.0), vec2<f32>(0.0), 0.0, 0.0, vec2<f32>(0.0, 0.0));
  }
}

fn receiverUvOf(p: vec3<f32>) -> vec2<f32> {
  let o = p - uniforms.receiverOriginWidth.xyz;
  return vec2<f32>(
    dot(o, uniforms.receiverTangentHeight.xyz) / uniforms.receiverOriginWidth.w + 0.5,
    dot(o, uniforms.receiverBitangentIntensity.xyz) / uniforms.receiverTangentHeight.w + 0.5
  );
}

// Derived from luma.gl's trace shader (MIT). Same refraction, exit search and dispersion;
// instead of one splat record per photon it stores, per light-view texel and wavelength, WHERE the
// refracted ray lands on the receiver (newUv) and where it would have landed unrefracted (oldUv).
// The area ratio of neighbouring texels (old/new) is then the caustic brightness, drawn as triangles.
@compute @workgroup_size(64) fn main(
  @builtin(global_invocation_id) globalId: vec3<u32>
) {
  let captureSize = u32(uniforms.targetSizes.x);
  let sampleCount = captureSize * captureSize;
  let sampleIndex = globalId.x;
  if (sampleIndex >= sampleCount) {
    return;
  }
  let pixel = vec2<i32>(i32(sampleIndex % captureSize), i32(sampleIndex / captureSize));
  let frontDepth = textureLoad(frontDepthTexture, pixel, 0);
  if (frontDepth >= 0.999999) {
    writeInvalid(sampleIndex);
    return;
  }
  let uv = (vec2<f32>(pixel) + vec2<f32>(0.5)) / f32(captureSize);
  let entryPosition = reconstructWorldPosition(uv, frontDepth);
  let rayNear = reconstructWorldPosition(uv, 0.0);
  let rayFar = reconstructWorldPosition(uv, 1.0);
  let incomingDirection = normalize(rayFar - rayNear);
  var entryNormal = decodeNormal(textureLoad(frontNormalTexture, pixel, 0).xyz);
  if (dot(incomingDirection, entryNormal) > 0.0) {
    entryNormal = -entryNormal;
  }
  let receiverOrigin = uniforms.receiverOriginWidth.xyz;
  let receiverNormal = uniforms.receiverNormalRefractiveIndex.xyz;

  let oldDenominator = dot(incomingDirection, receiverNormal);
  var oldUv = vec2<f32>(0.0);
  if (abs(oldDenominator) > 0.00001) {
    let oldDistance = dot(receiverOrigin - entryPosition, receiverNormal) / oldDenominator;
    oldUv = receiverUvOf(entryPosition + incomingDirection * oldDistance);
  }

  for (var wavelengthIndex = 0u; wavelengthIndex < WAVELENGTH_COUNT; wavelengthIndex++) {
    let hitIndex = sampleIndex * WAVELENGTH_COUNT + wavelengthIndex;
    hits[hitIndex] = Hit(vec2<f32>(0.0), oldUv, 0.0, 0.0, vec2<f32>(1.0, 0.0));
    let refractiveIndex = getRefractiveIndex(WAVELENGTHS_NANOMETERS[wavelengthIndex]);
    let insideDirection = refract(incomingDirection, entryNormal, 1.0 / refractiveIndex);
    if (dot(insideDirection, insideDirection) < 0.000001) {
      continue;
    }
    let exitHit = findExitSurface(entryPosition, normalize(insideDirection), uv, i32(captureSize));
    if (exitHit.valid < 0.5) {
      continue;
    }
    var exitNormal = exitHit.normal;
    if (dot(insideDirection, exitNormal) < 0.0) {
      exitNormal = -exitNormal;
    }
    let outgoingDirection = refract(normalize(insideDirection), -exitNormal, refractiveIndex);
    let receiverDenominator = dot(outgoingDirection, receiverNormal);
    if (dot(outgoingDirection, outgoingDirection) < 0.000001 || abs(receiverDenominator) < 0.00001) {
      continue;
    }
    let receiverDistance = dot(receiverOrigin - exitHit.position, receiverNormal) / receiverDenominator;
    if (receiverDistance <= 0.0) {
      continue;
    }
    let newUv = receiverUvOf(exitHit.position + outgoingDirection * receiverDistance);
    let opticalThickness = distance(entryPosition, exitHit.position);
    let absorptionCoefficient = dot(uniforms.absorptionDispersion.xyz, WAVELENGTH_ABSORPTION_WEIGHTS[wavelengthIndex]);
    let transmission = exp(-max(absorptionCoefficient, 0.0) * opticalThickness);
    hits[hitIndex] = Hit(newUv, oldUv, 1.0, transmission, vec2<f32>(1.0, 0.0));
  }
}
