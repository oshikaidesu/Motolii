// Light-space capture of one closed refractor: world normal (encoded) + depth.
// Front pass: cull back faces, depth Less.  Back pass: cull front faces, depth Greater.
struct Light {
  viewProjection: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> light: Light;

struct VertexIn {
  @location(0) position: vec3<f32>,
  @location(1) normal: vec3<f32>,
};
struct VertexOut {
  @builtin(position) clip: vec4<f32>,
  @location(0) normal: vec3<f32>,
};

@vertex fn vs(input: VertexIn) -> VertexOut {
  var out: VertexOut;
  out.clip = light.viewProjection * vec4<f32>(input.position, 1.0);
  out.normal = input.normal;
  return out;
}

@fragment fn fs(input: VertexOut) -> @location(0) vec4<f32> {
  return vec4<f32>(normalize(input.normal) * 0.5 + vec3<f32>(0.5), 1.0);
}
