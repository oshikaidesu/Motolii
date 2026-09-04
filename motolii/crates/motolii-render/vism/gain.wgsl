/*{
  "ID": "motolii.gain",
  "DESCRIPTION": "Linear RGB gain, preserving premultiplied alpha",
  "OUTPUT_FLOAT": true,
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "gain", "TYPE": "float", "DEFAULT": 1.0, "MIN": 0.0, "MAX": 4.0 }
  ]
}*/

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> gain: f32;
@group(1) @binding(1) var<uniform> render_size: vec2f;
@group(1) @binding(2) var<uniform> pass_index: f32;

struct VsOut {
  @builtin(position) position: vec4f,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
  let positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  var out: VsOut;
  out.position = vec4f(positions[index], 0.0, 1.0);
  return out;
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  let color = textureLoad(source_tex, vec2i(in.position.xy), 0);
  return vec4f(color.rgb * gain, color.a);
}
