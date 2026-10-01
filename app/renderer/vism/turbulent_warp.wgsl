/*{
  "ID": "motolii.turbulent_warp",
  "LABEL": "Turbulent Warp (2D, retired)",
  "STAGE": "warp",
  "EXPOSE": false,
  "OUTPUT_FLOAT": true,
  "DESCRIPTION": "Retired from the shelf 2026-09-13: Turbulent Displace (Direction XY) does this with the silhouette. Kept as the Warp stage's contract (material.rs tests)",
  "PADDING": { "PARAM": "amount", "SCALE": 1.0 },
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "amount", "LABEL": "Amount", "TYPE": "float", "DEFAULT": 50.0, "MIN": -1000.0, "MAX": 1000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "size", "LABEL": "Size", "TYPE": "float", "DEFAULT": 100.0, "MIN": 1.0, "MAX": 10000.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "complexity", "LABEL": "Complexity", "TYPE": "float", "DEFAULT": 3.0, "MIN": 1.0, "MAX": 8.0 },
    { "NAME": "evolution", "LABEL": "Evolution", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TIME" },
    { "NAME": "offset_x", "LABEL": "Offset X", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TRANSLATION" },
    { "NAME": "offset_y", "LABEL": "Offset Y", "TYPE": "float", "DEFAULT": 0.0, "SUBTYPE": "TRANSLATION" },
    { "NAME": "seed", "LABEL": "Seed", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 10000.0 },
    { "NAME": "direction", "LABEL": "Direction", "TYPE": "long", "DEFAULT": 0, "LABELS": ["Both", "Horizontal", "Vertical"] }
  ]
}*/

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> amount: f32;
@group(1) @binding(1) var<uniform> size: f32;
@group(1) @binding(2) var<uniform> complexity: f32;
@group(1) @binding(3) var<uniform> evolution: f32;
@group(1) @binding(4) var<uniform> offset_x: f32;
@group(1) @binding(5) var<uniform> offset_y: f32;
@group(1) @binding(6) var<uniform> seed: f32;
@group(1) @binding(7) var<uniform> direction: f32;
@group(1) @binding(8) var<uniform> render_size: vec2f;
@group(1) @binding(9) var<uniform> pass_index: f32;
@group(1) @binding(10) var<uniform> material_origin: vec2f;

struct VsOut {
  @builtin(position) position: vec4f,
  @location(0) uv: vec2f,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
  let positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  let uvs = array<vec2f, 3>(vec2f(0.0, 1.0), vec2f(2.0, 1.0), vec2f(0.0, -1.0));
  return VsOut(vec4f(positions[index], 0.0, 1.0), uvs[index]);
}

fn read_source(p: vec2i) -> vec4f {
  let dims = vec2i(textureDimensions(source_tex));
  if any(p < vec2i(0)) || any(p >= dims) { return vec4f(0.0); }
  return textureLoad(source_tex, p, 0);
}

fn sample_source(uv: vec2f) -> vec4f {
  let p = uv * vec2f(textureDimensions(source_tex)) - 0.5;
  let i = vec2i(floor(p));
  let f = fract(p);
  return mix(mix(read_source(i), read_source(i + vec2i(1,0)), f.x),
             mix(read_source(i + vec2i(0,1)), read_source(i + vec2i(1,1)), f.x), f.y);
}

fn turbulence(p: vec3f) -> f32 {
  let n = clamp(complexity, 1.0, 8.0);
  let low = u32(floor(n));
  return mix(fbm3(p, low), fbm3(p, min(low + 1u, 8u)), fract(n));
}

@fragment
fn fs_main(@location(0) uv: vec2f) -> @location(0) vec4f {
  if amount == 0.0 { return read_source(vec2i(uv * vec2f(textureDimensions(source_tex)))); }
  let local = uv * render_size + material_origin;
  let q = vec3f((local + vec2f(offset_x, offset_y)) / max(size, 1.0), 0.0)
        + evolution * vec3f(0.53, 0.71, 0.89) + seed * vec3f(0.137, 0.173, 0.193);
  var delta = amount * vec2f(turbulence(q), turbulence(q + vec3f(31.7, 47.3, 11.9)));
  if direction > 0.5 && direction < 1.5 { delta.y = 0.0; }
  if direction >= 1.5 { delta.x = 0.0; }
  return sample_source((local + delta - material_origin) / render_size);
}
