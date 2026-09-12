/*{
  "ID": "motolii.glow",
  "LABEL": "Glow",
  "DESCRIPTION": "明部を抜いて、半分ずつ 6 段に落とし(13 tap、初段は Karis 平均で火花を殺す)、3×3 tent で戻しながら足す(Call of Duty: Advanced Warfare / Bevy の bloom の作法)。Radius が届く段まで、Spread で遠い光を持ち上げ、Anamorphic で横に伸ばし、Chromatic で外周ほど色相を回す。光は SPILL で層の Blend と独立に下へ乗る",
  "OUTPUT_FLOAT": true,
  "FILTER": "linear",
  "SPILL": "screen",
  "PADDING": { "PARAM": "radius", "SCALE": 1.5 },
  "INPUTS": [
    { "NAME": "source", "TYPE": "image" },
    { "NAME": "threshold", "LABEL": "Threshold", "TYPE": "float", "DEFAULT": 0.6, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "LEVEL" },
    { "NAME": "softness", "LABEL": "Softness", "TYPE": "float", "DEFAULT": 0.3, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "FACTOR" },
    { "NAME": "intensity", "LABEL": "Intensity", "TYPE": "float", "DEFAULT": 1.5, "MIN": 0.0, "MAX": 8.0, "SUBTYPE": "FACTOR" },
    { "NAME": "radius", "LABEL": "Radius", "TYPE": "float", "DEFAULT": 64.0, "MIN": 1.0, "MAX": 1024.0, "SUBTYPE": "DISTANCE" },
    { "NAME": "spread", "LABEL": "Spread", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "FACTOR" },
    { "NAME": "anamorphic", "LABEL": "Anamorphic", "TYPE": "float", "DEFAULT": 0.0, "MIN": -1.0, "MAX": 1.0, "SUBTYPE": "FACTOR", "ADVANCED": true },
    { "NAME": "chromatic", "LABEL": "Chromatic", "TYPE": "float", "DEFAULT": 0.0, "MIN": 0.0, "MAX": 1.0, "SUBTYPE": "FACTOR", "ADVANCED": true }
  ],
  "PASSES": [
    { "TARGET": "down1", "FLOAT": true, "WIDTH": "$WIDTH/2",  "HEIGHT": "$HEIGHT/2" },
    { "TARGET": "down2", "FLOAT": true, "WIDTH": "$WIDTH/4",  "HEIGHT": "$HEIGHT/4" },
    { "TARGET": "down3", "FLOAT": true, "WIDTH": "$WIDTH/8",  "HEIGHT": "$HEIGHT/8" },
    { "TARGET": "down4", "FLOAT": true, "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16" },
    { "TARGET": "down5", "FLOAT": true, "WIDTH": "$WIDTH/32", "HEIGHT": "$HEIGHT/32" },
    { "TARGET": "down6", "FLOAT": true, "WIDTH": "$WIDTH/64", "HEIGHT": "$HEIGHT/64" },
    { "TARGET": "up5",   "FLOAT": true, "WIDTH": "$WIDTH/32", "HEIGHT": "$HEIGHT/32" },
    { "TARGET": "up4",   "FLOAT": true, "WIDTH": "$WIDTH/16", "HEIGHT": "$HEIGHT/16" },
    { "TARGET": "up3",   "FLOAT": true, "WIDTH": "$WIDTH/8",  "HEIGHT": "$HEIGHT/8" },
    { "TARGET": "up2",   "FLOAT": true, "WIDTH": "$WIDTH/4",  "HEIGHT": "$HEIGHT/4" },
    { "TARGET": "up1",   "FLOAT": true, "WIDTH": "$WIDTH/2",  "HEIGHT": "$HEIGHT/2" },
    { }
  ]
}*/

// image の並び = INPUTS の image → PASSES の TARGET(宣言順)。texture は 2n、sampler は 2n+1。
@group(0) @binding(0)  var source_tex: texture_2d<f32>;
@group(0) @binding(1)  var source_smp: sampler;
@group(0) @binding(2)  var down1_tex: texture_2d<f32>;
@group(0) @binding(3)  var down1_smp: sampler;
@group(0) @binding(4)  var down2_tex: texture_2d<f32>;
@group(0) @binding(5)  var down2_smp: sampler;
@group(0) @binding(6)  var down3_tex: texture_2d<f32>;
@group(0) @binding(7)  var down3_smp: sampler;
@group(0) @binding(8)  var down4_tex: texture_2d<f32>;
@group(0) @binding(9)  var down4_smp: sampler;
@group(0) @binding(10) var down5_tex: texture_2d<f32>;
@group(0) @binding(11) var down5_smp: sampler;
@group(0) @binding(12) var down6_tex: texture_2d<f32>;
@group(0) @binding(13) var down6_smp: sampler;
@group(0) @binding(14) var up5_tex: texture_2d<f32>;
@group(0) @binding(15) var up5_smp: sampler;
@group(0) @binding(16) var up4_tex: texture_2d<f32>;
@group(0) @binding(17) var up4_smp: sampler;
@group(0) @binding(18) var up3_tex: texture_2d<f32>;
@group(0) @binding(19) var up3_smp: sampler;
@group(0) @binding(20) var up2_tex: texture_2d<f32>;
@group(0) @binding(21) var up2_smp: sampler;
@group(0) @binding(22) var up1_tex: texture_2d<f32>;
@group(0) @binding(23) var up1_smp: sampler;
@group(1) @binding(0) var<uniform> threshold: f32;
@group(1) @binding(1) var<uniform> softness: f32;
@group(1) @binding(2) var<uniform> intensity: f32;
@group(1) @binding(3) var<uniform> radius: f32;
@group(1) @binding(4) var<uniform> spread: f32;
@group(1) @binding(5) var<uniform> anamorphic: f32;
@group(1) @binding(6) var<uniform> chromatic: f32;
@group(1) @binding(7) var<uniform> render_size: vec2f;
@group(1) @binding(8) var<uniform> pass_index: f32;

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

// 横に伸ばす: 標本の間隔を x に広げ、y を詰める(Bevy の scale と同じ、面積は保つ)。
fn tap_scale() -> vec2f {
  let a = anamorphic * 3.0;
  if (a >= 0.0) { return vec2f(1.0 + a, 1.0 / (1.0 + a)); }
  return vec2f(1.0 / (1.0 - a), 1.0 - a);
}

// 輝度は線形の Rec.709(合成が線形光で走っているのに合わせる)
fn luminance(c: vec3f) -> f32 { return dot(c, vec3f(0.2126, 0.7152, 0.0722)); }

// Catlike Coding の soft knee: 閾値の周りを softness の幅で滑らかに立ち上げる。
fn soft_threshold(c: vec4f) -> vec4f {
  let brightness = max(c.r, max(c.g, c.b));
  let knee = threshold * softness;
  var soft = brightness - threshold + knee;
  soft = clamp(soft, 0.0, 2.0 * knee);
  soft = soft * soft / max(4.0 * knee, 1e-5);
  let contribution = max(brightness - threshold, soft) / max(brightness, 1e-5);
  return c * contribution;
}

// Karis 2013: 1 画素だけ極端に明るい火花が段を降りる時に爆発しないよう、輝度で重みを下げる。
fn karis(c: vec4f) -> f32 { return 1.0 / (1.0 + luminance(c.rgb) * 0.25); }

// [COD] slide 153 の 13 tap。first の段だけ 5 つの箱を Karis 平均で混ぜる。
fn downsample13(t: texture_2d<f32>, s: sampler, uv: vec2f, first: bool) -> vec4f {
  let ps = tap_scale() / vec2f(textureDimensions(t));
  let a = textureSampleLevel(t, s, uv + vec2f(-2.0,  2.0) * ps, 0.0);
  let b = textureSampleLevel(t, s, uv + vec2f( 0.0,  2.0) * ps, 0.0);
  let c = textureSampleLevel(t, s, uv + vec2f( 2.0,  2.0) * ps, 0.0);
  let d = textureSampleLevel(t, s, uv + vec2f(-2.0,  0.0) * ps, 0.0);
  let e = textureSampleLevel(t, s, uv, 0.0);
  let f = textureSampleLevel(t, s, uv + vec2f( 2.0,  0.0) * ps, 0.0);
  let g = textureSampleLevel(t, s, uv + vec2f(-2.0, -2.0) * ps, 0.0);
  let h = textureSampleLevel(t, s, uv + vec2f( 0.0, -2.0) * ps, 0.0);
  let i = textureSampleLevel(t, s, uv + vec2f( 2.0, -2.0) * ps, 0.0);
  let j = textureSampleLevel(t, s, uv + vec2f(-1.0,  1.0) * ps, 0.0);
  let k = textureSampleLevel(t, s, uv + vec2f( 1.0,  1.0) * ps, 0.0);
  let l = textureSampleLevel(t, s, uv + vec2f(-1.0, -1.0) * ps, 0.0);
  let m = textureSampleLevel(t, s, uv + vec2f( 1.0, -1.0) * ps, 0.0);
  if (first) {
    var g0 = soft_threshold((a + b + d + e) * 0.25);
    var g1 = soft_threshold((b + c + e + f) * 0.25);
    var g2 = soft_threshold((d + e + g + h) * 0.25);
    var g3 = soft_threshold((e + f + h + i) * 0.25);
    var g4 = soft_threshold((j + k + l + m) * 0.25);
    g0 *= karis(g0); g1 *= karis(g1); g2 *= karis(g2); g3 *= karis(g3); g4 *= karis(g4);
    return (g0 + g1 + g2 + g3) * 0.125 + g4 * 0.5;
  }
  return (a + c + g + i) * 0.03125 + (b + d + f + h) * 0.0625 + (e + j + k + l + m) * 0.125;
}

// [COD] slide 162 の 3×3 tent。
fn tent(t: texture_2d<f32>, s: sampler, uv: vec2f) -> vec4f {
  let ps = tap_scale() / vec2f(textureDimensions(t));
  let x = ps.x; let y = ps.y;
  return textureSampleLevel(t, s, uv + vec2f(-x,  y), 0.0) * 0.0625 + textureSampleLevel(t, s, uv + vec2f(0.0,  y), 0.0) * 0.125 + textureSampleLevel(t, s, uv + vec2f(x,  y), 0.0) * 0.0625
       + textureSampleLevel(t, s, uv + vec2f(-x, 0.0), 0.0) * 0.125  + textureSampleLevel(t, s, uv, 0.0) * 0.25                  + textureSampleLevel(t, s, uv + vec2f(x, 0.0), 0.0) * 0.125
       + textureSampleLevel(t, s, uv + vec2f(-x, -y), 0.0) * 0.0625 + textureSampleLevel(t, s, uv + vec2f(0.0, -y), 0.0) * 0.125 + textureSampleLevel(t, s, uv + vec2f(x, -y), 0.0) * 0.0625;
}

// 段 k(1 = 1/2 … 6 = 1/64)の重み。Radius(画素)が届く段まで 1、その先は消える。Spread は遠い段を持ち上げる。
fn level_weight(k: f32) -> f32 {
  let reach = log2(max(radius, 1.0));           // 64 px → 6 段
  let gate = clamp(reach - (k - 1.0), 0.0, 1.0);
  return gate * (1.0 + spread * (k - 1.0) * 0.6);
}

// 色相を回す(YIQ)。外周(深い段)ほど回る。
fn hue_rotate(c: vec3f, angle: f32) -> vec3f {
  let yiq = mat3x3f(vec3f(0.299, 0.596, 0.211), vec3f(0.587, -0.274, -0.523), vec3f(0.114, -0.322, 0.312)) * c;
  let cs = cos(angle); let sn = sin(angle);
  let rotated = vec3f(yiq.x, yiq.y * cs - yiq.z * sn, yiq.y * sn + yiq.z * cs);
  return mat3x3f(vec3f(1.0, 1.0, 1.0), vec3f(0.956, -0.272, -1.106), vec3f(0.621, -0.647, 1.703)) * rotated;
}

fn colored(c: vec4f, k: f32) -> vec4f {
  if (chromatic <= 0.0) { return c; }
  return vec4f(max(hue_rotate(c.rgb, chromatic * (k - 1.0) * 0.6), vec3f(0.0)), c.a);
}

// 戻し: 上の段の tent に、この段の絵を重みで足す。
fn up(from_tex: texture_2d<f32>, from_smp: sampler, own_tex: texture_2d<f32>, own_smp: sampler, uv: vec2f, k: f32) -> vec4f {
  return tent(from_tex, from_smp, uv) + colored(textureSampleLevel(own_tex, own_smp, uv, 0.0), k) * level_weight(k);
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4f {
  let uv = in.position.xy / render_size;
  let stage = u32(pass_index);
  if (stage == 0u) { return downsample13(source_tex, source_smp, uv, true); }
  if (stage == 1u) { return downsample13(down1_tex, down1_smp, uv, false); }
  if (stage == 2u) { return downsample13(down2_tex, down2_smp, uv, false); }
  if (stage == 3u) { return downsample13(down3_tex, down3_smp, uv, false); }
  if (stage == 4u) { return downsample13(down4_tex, down4_smp, uv, false); }
  if (stage == 5u) { return downsample13(down5_tex, down5_smp, uv, false); }
  if (stage == 6u) { return colored(textureSampleLevel(down6_tex, down6_smp, uv, 0.0), 6.0) * level_weight(6.0) + colored(textureSampleLevel(down5_tex, down5_smp, uv, 0.0), 5.0) * level_weight(5.0); }
  if (stage == 7u) { return up(up5_tex, up5_smp, down4_tex, down4_smp, uv, 4.0); }
  if (stage == 8u) { return up(up4_tex, up4_smp, down3_tex, down3_smp, uv, 3.0); }
  if (stage == 9u) { return up(up3_tex, up3_smp, down2_tex, down2_smp, uv, 2.0); }
  if (stage == 10u) { return up(up2_tex, up2_smp, down1_tex, down1_smp, uv, 1.0); }
  // 出力: 元の絵 + 光。段の和は正規化しない(近くは全段が重なって強く、遠くは深い段だけで淡い — bloom の形)。
  let light = tent(up1_tex, up1_smp, uv) * intensity;
  let src = textureLoad(source_tex, vec2i(in.position.xy), 0);
  return vec4f(src.rgb + light.rgb, clamp(src.a + light.a * (1.0 - src.a), 0.0, 1.0));
}
