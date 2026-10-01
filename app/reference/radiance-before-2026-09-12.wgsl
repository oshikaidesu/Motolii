/*{
  "ID": "motolii.radiance",
  "LABEL": "Radiance",
  "DESCRIPTION": "素材の明るい所を光源、素材の形を遮蔽にして、光が周りへ回る(2D の radiance cascades)。距離場は jump flood、段は ISF の PASSES で宣言し、同じ TARGET 名は同じ buffer",
  "OUTPUT_FLOAT": true,
  "PADDING": { "PARAM": "radius", "SCALE": 1.0 },
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "threshold", "LABEL": "Threshold", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0 },
    { "NAME": "intensity", "LABEL": "Intensity", "TYPE": "float", "DEFAULT": 3.0, "MIN": 0.0, "MAX": 16.0 },
    { "NAME": "radius", "LABEL": "Reach", "TYPE": "float", "DEFAULT": 64.0, "MIN": 0.0, "MAX": 512.0 },
    { "NAME": "air", "LABEL": "Air", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 2.0 }
  ],
  "PASSES": [
    { "TARGET": "emitter", "FLOAT": true },
    { "TARGET": "jfa_a", "FLOAT": true },
    { "TARGET": "jfa_b", "FLOAT": true }, { "TARGET": "jfa_a", "FLOAT": true },
    { "TARGET": "jfa_b", "FLOAT": true }, { "TARGET": "jfa_a", "FLOAT": true },
    { "TARGET": "jfa_b", "FLOAT": true }, { "TARGET": "jfa_a", "FLOAT": true },
    { "TARGET": "jfa_b", "FLOAT": true }, { "TARGET": "jfa_a", "FLOAT": true },
    { "TARGET": "jfa_b", "FLOAT": true }, { "TARGET": "jfa_a", "FLOAT": true },
    { "TARGET": "sdf", "FLOAT": true },
    { "TARGET": "casc_a", "FLOAT": true },
    { "TARGET": "casc_b", "FLOAT": true },
    { "TARGET": "casc_a", "FLOAT": true },
    { "TARGET": "casc_b", "FLOAT": true },
    { "TARGET": "casc_a", "FLOAT": true },
    { "TARGET": "casc_b", "FLOAT": true },
    { }
  ]
}*/

// 借り物: vercel-labs/vgpu examples/radiance-cascades(MIT)。jump flood・sphere trace・
// cascade の合流はそのまま、atlas は素材と同じ大きさになるよう探針の間隔を 2px にした。
// 段の並び: 0 emitter / 1 jfa init / 2..11 jfa(jump 512..1) / 12 sdf / 13..18 cascade 5..0 / 19 合成。
// 種は f16 なので座標が正確なのは 2048px まで(それ以上は 1px ずれる)。

@group(0) @binding(0) var source_tex: texture_2d<f32>;
@group(0) @binding(2) var emitter_tex: texture_2d<f32>;
@group(0) @binding(4) var jfa_a_tex: texture_2d<f32>;
@group(0) @binding(6) var jfa_b_tex: texture_2d<f32>;
@group(0) @binding(8) var sdf_tex: texture_2d<f32>;
@group(0) @binding(10) var casc_a_tex: texture_2d<f32>;
@group(0) @binding(12) var casc_b_tex: texture_2d<f32>;
@group(1) @binding(0) var<uniform> threshold: f32;
@group(1) @binding(1) var<uniform> intensity: f32;
@group(1) @binding(2) var<uniform> radius: f32;
@group(1) @binding(3) var<uniform> air: f32;
@group(1) @binding(4) var<uniform> render_size: vec2f;
@group(1) @binding(5) var<uniform> pass_index: f32;

const TAU: f32 = 6.283185307179586;
const JFA_FIRST: i32 = 2;
const JFA_LAST: i32 = 11;
const SDF_PASS: i32 = 12;
const CASCADE_FIRST: i32 = 13;
const CASCADE_LAST: i32 = 18;
const CASCADES: f32 = 6.0;
const SPACING0: f32 = 2.0;
const HIT_EPSILON: f32 = 0.5;
const MIN_STEP: f32 = 0.35;
const MAX_STEPS: i32 = 16;

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

fn clamp_texel(p: vec2f) -> vec2i {
  return vec2i(clamp(floor(p), vec2f(0.0), render_size - 1.0));
}

// --- 0: 光源と遮蔽。rgb = 明るい所の放射輝度、a = 形(遮蔽) ---
fn emitter(texel: vec2i) -> vec4f {
  let s = textureLoad(source_tex, texel, 0);
  let luminance = dot(s.rgb, vec3f(0.2126, 0.7152, 0.0722));
  let contribution = max(luminance - threshold, 0.0) / max(luminance, 0.000001);
  return vec4f(s.rgb * contribution * intensity, s.a);
}

// --- 1: jump flood の種。光源でも遮蔽でも、形のある texel は自分を指す ---
fn jfa_init(texel: vec2i) -> vec4f {
  let mask = textureLoad(emitter_tex, texel, 0).a;
  if (mask > 0.5) {
    return vec4f(vec2f(texel) + 0.5, 0.0, 1.0);
  }
  return vec4f(0.0);
}

fn jfa_pick(current: vec4f, candidate: vec4f, position: vec2f) -> vec4f {
  if (candidate.w < 0.5) { return current; }
  if (current.w < 0.5) { return candidate; }
  return select(current, candidate, distance(candidate.xy, position) < distance(current.xy, position));
}

fn seeds_load(read_a: bool, coord: vec2i) -> vec4f {
  if (read_a) { return textureLoad(jfa_a_tex, coord, 0); }
  return textureLoad(jfa_b_tex, coord, 0);
}

// --- 2..11: 3×3 を jump だけ離れて見て、近い種を残す。枠の外は読まない(縁の偏りを避ける) ---
fn jfa_step(texel: vec2i, stage: i32) -> vec4f {
  let read_a = (stage % 2) == 0;
  let jump = i32(pow(2.0, f32(JFA_LAST - stage)));
  let position = vec2f(texel) + 0.5;
  let limit = vec2i(render_size) - vec2i(1);
  var best = seeds_load(read_a, texel);
  for (var y = -1; y <= 1; y = y + 1) {
    for (var x = -1; x <= 1; x = x + 1) {
      let neighbour = texel + vec2i(x, y) * jump;
      if (neighbour.x < 0 || neighbour.y < 0 || neighbour.x > limit.x || neighbour.y > limit.y) { continue; }
      best = jfa_pick(best, seeds_load(read_a, neighbour), position);
    }
  }
  return best;
}

// --- 12: 種 → 距離(px)。R に入れる ---
fn sdf_finalize(texel: vec2i) -> vec4f {
  let seed = textureLoad(jfa_a_tex, texel, 0);
  let far = length(render_size) * 2.0;
  let d = select(far, distance(seed.xy, vec2f(texel) + 0.5), seed.w >= 0.5);
  return vec4f(d, 0.0, 0.0, 1.0);
}

// --- cascade の住所 ---
fn rc_ray_count(cascade: f32) -> f32 { return pow(4.0, cascade + 1.0); }
fn rc_block_size(cascade: f32) -> f32 { return pow(2.0, cascade + 1.0); }
fn rc_probe_spacing(cascade: f32) -> f32 { return SPACING0 * pow(2.0, cascade); }
fn rc_direction(index: f32, rays: f32) -> vec2f {
  let theta = TAU * (index + 0.5) / rays;
  return vec2f(cos(theta), sin(theta));
}
fn rc_atlas_texel(probe: vec2f, direction_index: f32, block: f32) -> vec2f {
  let slot = vec2f(direction_index % block, floor(direction_index / block));
  return probe * block + slot;
}
fn rc_interval_start(cascade: f32) -> f32 { return 2.0 * (pow(4.0, cascade) - 1.0) / 3.0; }
fn rc_interval_end(cascade: f32) -> f32 { return rc_interval_start(cascade) + 2.0 * pow(4.0, cascade) * 1.02; }

fn sphere_trace(origin: vec2f, direction: vec2f, t_start: f32, t_end: f32) -> vec4f {
  var t = t_start;
  for (var step = 0; step < MAX_STEPS; step = step + 1) {
    let p = origin + direction * t;
    if (p.x < -1.0 || p.y < -1.0 || p.x > render_size.x + 1.0 || p.y > render_size.y + 1.0) { break; }
    let d = textureLoad(sdf_tex, clamp_texel(p), 0).r;
    if (d <= HIT_EPSILON) {
      let e = textureLoad(emitter_tex, clamp_texel(p), 0);
      return vec4f(e.rgb, 0.0);
    }
    t = t + max(d, MIN_STEP);
    if (t > t_end) { break; }
  }
  return vec4f(0.0, 0.0, 0.0, 1.0);
}

fn rc_merge(near: vec4f, far: vec4f) -> vec4f {
  return vec4f(near.rgb + near.a * far.rgb, near.a * far.a);
}

fn upper_load(read_a: bool, coord: vec2i) -> vec4f {
  if (read_a) { return textureLoad(casc_a_tex, coord, 0); }
  return textureLoad(casc_b_tex, coord, 0);
}

// --- 13..18: 段 5 から 0 へ。各 texel は (探針, 方向) の 1 組。上の段を 4 方向 × 4 探針で合流 ---
fn cascade_pass(texel: vec2i, stage: i32) -> vec4f {
  let cascade = f32(CASCADE_LAST - stage);
  let write_a = (stage % 2) == 1;
  let block = rc_block_size(cascade);
  let rays = rc_ray_count(cascade);
  let t = vec2f(texel);
  let probe = floor(t / block);
  let slot = t - probe * block;
  let direction_index = slot.y * block + slot.x;
  let spacing = rc_probe_spacing(cascade);
  let origin = (probe + 0.5) * spacing;
  let direction = rc_direction(direction_index, rays);
  var radiance = sphere_trace(origin, direction, rc_interval_start(cascade), rc_interval_end(cascade));
  if (cascade < CASCADES - 1.0) {
    let upper_block = block * 2.0;
    let upper_spacing = spacing * 2.0;
    let upper_grid = ceil(render_size / upper_block);
    let position = origin / upper_spacing - 0.5;
    let base = floor(position);
    let f = clamp(position - base, vec2f(0.0), vec2f(1.0));
    var weights = array<f32, 4>((1.0 - f.x) * (1.0 - f.y), f.x * (1.0 - f.y), (1.0 - f.x) * f.y, f.x * f.y);
    var far = vec4f(0.0);
    for (var branch = 0; branch < 4; branch = branch + 1) {
      let upper_direction = direction_index * 4.0 + f32(branch);
      var interpolated = vec4f(0.0);
      for (var corner = 0; corner < 4; corner = corner + 1) {
        let offset = vec2f(f32(corner % 2), f32(corner / 2));
        let neighbour = clamp(base + offset, vec2f(0.0), upper_grid - 1.0);
        let coord = rc_atlas_texel(neighbour, upper_direction, upper_block);
        interpolated += weights[corner] * upper_load(!write_a, vec2i(coord));
      }
      far += interpolated * 0.25;
    }
    radiance = rc_merge(radiance, far);
  }
  return radiance;
}

// 段 0 の探針(間隔 2px)を双一次に混ぜて、その画素に届く光にする
fn irradiance(pixel: vec2f) -> vec3f {
  let block = rc_block_size(0.0);
  let grid = ceil(render_size / block);
  let position = pixel / rc_probe_spacing(0.0) - 0.5;
  let base = floor(position);
  let f = clamp(position - base, vec2f(0.0), vec2f(1.0));
  var weights = array<f32, 4>((1.0 - f.x) * (1.0 - f.y), f.x * (1.0 - f.y), (1.0 - f.x) * f.y, f.x * f.y);
  var total = vec3f(0.0);
  for (var corner = 0; corner < 4; corner = corner + 1) {
    let offset = vec2f(f32(corner % 2), f32(corner / 2));
    let probe = clamp(base + offset, vec2f(0.0), grid - 1.0);
    var rays = vec3f(0.0);
    for (var i = 0.0; i < 4.0; i = i + 1.0) {
      rays += textureLoad(casc_b_tex, vec2i(rc_atlas_texel(probe, i, block)), 0).rgb;
    }
    total += weights[corner] * rays * 0.25;
  }
  return total;
}

// --- 19: 素材 + 素材に当たる光 + 空気中の光。入力も出力も乗算済み(2026-09-13 に列の規約が乗算済み線形へ
// 変わった時、この参照写しも同じ適応をした。試験の主題(energy の pre-averaging)は触っていない) ---
fn composite(texel: vec2i, pixel: vec2f) -> vec4f {
  let s = textureLoad(source_tex, texel, 0);
  let light = irradiance(pixel);
  let in_air = light * air * (1.0 - s.a);
  let air_alpha = clamp(max(in_air.r, max(in_air.g, in_air.b)), 0.0, 1.0);
  return clamp(vec4f(s.rgb + light * s.rgb + in_air, s.a + air_alpha * (1.0 - s.a)), vec4f(0.0), vec4f(1.0));
}

@fragment
fn fs_main(@builtin(position) position: vec4f) -> @location(0) vec4f {
  let pixel = position.xy;
  let texel = vec2i(pixel);
  let stage = i32(pass_index + 0.5);
  if (stage == 0) { return emitter(texel); }
  if (stage == 1) { return jfa_init(texel); }
  if (stage >= JFA_FIRST && stage <= JFA_LAST) { return jfa_step(texel, stage); }
  if (stage == SDF_PASS) { return sdf_finalize(texel); }
  if (stage >= CASCADE_FIRST && stage <= CASCADE_LAST) { return cascade_pass(texel, stage); }
  return composite(texel, pixel);
}
