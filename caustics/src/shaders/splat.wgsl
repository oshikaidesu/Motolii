// Borrowed from visgl/luma.gl modules/experimental/src/rendering/spectral-caustics-renderer-shaders.ts
// SPDX-License-Identifier: MIT   Copyright (c) vis.gl contributors   (see third_party/luma.gl-LICENSE)
// Only the template constants were substituted; the WGSL is unchanged.
struct PhotonSplat {
  positionRadius: vec4<f32>,
  xyzEnergy: vec4<f32>,
};

struct SplatVertexOutput {
  @builtin(position) position: vec4<f32>,
  @location(0) localPosition: vec2<f32>,
  @location(1) @interpolate(flat) xyzEnergy: vec3<f32>,
  @location(2) @interpolate(flat) kernelNormalization: f32,
};

@group(0) @binding(0) var<storage, read> photonSplats: array<PhotonSplat>;

@vertex fn vertexMain(
  @builtin(vertex_index) vertexIndex: u32,
  @builtin(instance_index) instanceIndex: u32
) -> SplatVertexOutput {
  let corners = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0)
  );
  let splat = photonSplats[instanceIndex];
  let localPosition = corners[vertexIndex];
  let center = vec2<f32>(
    splat.positionRadius.x * 2.0 - 1.0,
    1.0 - splat.positionRadius.y * 2.0
  );
  let radius = splat.positionRadius.z;
  var output: SplatVertexOutput;
  output.position = vec4<f32>(center + localPosition * radius * 2.0, 0.0, 1.0);
  output.localPosition = localPosition;
  output.xyzEnergy = splat.xyzEnergy.xyz;
  output.kernelNormalization = splat.positionRadius.w;
  return output;
}

@fragment fn fragmentMain(input: SplatVertexOutput) -> @location(0) vec4<f32> {
  let radiusSquared = dot(input.localPosition, input.localPosition);
  if (radiusSquared > 1.0) {
    discard;
  }
  let weight = exp(-radiusSquared * 3.5) * input.kernelNormalization;
  return vec4<f32>(input.xyzEnergy * weight, weight);
}