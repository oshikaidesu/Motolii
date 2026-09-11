/*{
  "ID": "motolii.material_encoding", "EXPOSE": false, "OUTPUT_FLOAT": true,
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "to_linear", "TYPE": "float", "DEFAULT": 0.0 }
  ]
}*/
@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> to_linear: f32;
@group(1) @binding(1) var<uniform> render_size: vec2f;
@group(1) @binding(2) var<uniform> pass_index: f32;

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> @builtin(position) vec4f {
  let p = array<vec2f,3>(vec2f(-1.0,-1.0),vec2f(3.0,-1.0),vec2f(-1.0,3.0));
  return vec4f(p[i],0.0,1.0);
}

// IEC sRGB transfer, as used by rerun's utils/srgb.wgsl.
fn decode_srgb(c: vec3f) -> vec3f {
  return select(pow(max((c + 0.055) / 1.055, vec3f(0.0)),vec3f(2.4)), c / 12.92, c <= vec3f(0.04045));
}
fn encode_srgb(c: vec3f) -> vec3f {
  return select(1.055 * pow(max(c,vec3f(0.0)),vec3f(1.0/2.4)) - 0.055, c * 12.92, c <= vec3f(0.0031308));
}
@fragment
fn fs_main(@builtin(position) p: vec4f) -> @location(0) vec4f {
  let color = textureLoad(source_tex, vec2i(p.xy), 0);
  if to_linear > 0.5 { return vec4f(decode_srgb(color.rgb) * color.a, color.a); }
  if color.a <= 0.0 { return vec4f(0.0); }
  return vec4f(encode_srgb(color.rgb / color.a), color.a);
}
