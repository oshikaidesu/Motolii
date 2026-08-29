/*{
  "INPUTS": [
    { "NAME": "glow", "TYPE": "float", "DEFAULT": 0.5, "MIN": 0.0, "MAX": 1.0,
      "MAPS": [
        { "CONST": "FALLOFF_K", "EXPR": "mix(120.0, 20.0, glow)" },
        { "CONST": "AMBIENT", "EXPR": "mix(0.0, 0.08, glow)" }
      ]
    }
  ]
}*/
// vgpu(vercel-labs)`triangle-led-front` の1パス版。元は2パス(led-emitters が
// LED を texture へ描き、direct-triangle-raycast が textureLoad で読む)+ uniform
// Config——ここでは Config の値を const へ焼き、texture read を辺上の LED 位置から
// 直接色を合成する式へ置き換えて、`gradient.wgsl` と同じ「バインディング0本」に畳んだ。
// import されていた hash21(hash.wgsl)はそのままインライン化。

struct VsOut {
  @builtin(position) position: vec4f,
  @location(0) uv: vec2f,
};

@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> VsOut {
  let positions = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
  let uvs = array<vec2f, 3>(vec2f(0.0, 1.0), vec2f(2.0, 1.0), vec2f(0.0, -1.0));
  var out: VsOut;
  out.position = vec4f(positions[index], 0.0, 1.0);
  out.uv = uvs[index];
  return out;
}

// vgpu settings.ts: 正三角形、circumradius = height*2/3, inradius = height/3。
// uv 空間(0..1)で height=0.7 を選んだ(自分で選んだ値 — 見た目が画面いっぱいに
// 収まる程度)。
const TRI_TOP: vec2f = vec2f(0.5, 0.083);
const TRI_LEFT: vec2f = vec2f(0.096, 0.783);
const TRI_RIGHT: vec2f = vec2f(0.904, 0.783);

// vgpu settings.ts の LEDS_PER_EDGE(実測値、そのまま焼いた)。
const LEDS_PER_EDGE: f32 = 24.0;
// 自分で選んだ値(vgpu の距離減衰式そのままは texture 前提なので置き換え):
// LED からの距離に対する光の減衰の強さ。
const FALLOFF_K: f32 = 55.0;
const AMBIENT: f32 = 0.02;

fn hash21(p: vec2f) -> f32 {
  return fract(sin(dot(p, vec2f(127.1, 311.7))) * 43758.5453123);
}

// 点 p から線分 a-b への最短距離と、その最近点の線分上の位置 t(0..1)。
fn segment_nearest(p: vec2f, a: vec2f, b: vec2f) -> vec2f {
  let e = b - a;
  let t = clamp(dot(p - a, e) / max(dot(e, e), 1e-6), 0.0, 1.0);
  return vec2f(distance(p, a + e * t), t);
}

// 辺 edge_id 上の t 位置に最も近い LED の色。texture 越しの `load_light_source`
// の代わりに、LED index を hash して色相を作る(vgpu の LED ごとの個別色/輝度と
// 同じ役割を hash が代行する)。
fn edge_led_color(edge_id: f32, t: f32) -> vec3f {
  let idx = floor(t * LEDS_PER_EDGE);
  let hue = hash21(vec2f(idx, edge_id * 97.0));
  let color = 0.5 + 0.5 * cos(6.28318 * (hue + vec3f(0.0, 0.33, 0.67)));
  let brightness = 0.6 + 0.4 * hash21(vec2f(idx * 3.1, edge_id * 13.0 + 1.0));
  return color * brightness;
}

@fragment
fn fs_main(@location(0) uv: vec2f) -> @location(0) vec4f {
  let ab = segment_nearest(uv, TRI_TOP, TRI_LEFT);
  let bc = segment_nearest(uv, TRI_LEFT, TRI_RIGHT);
  let ca = segment_nearest(uv, TRI_RIGHT, TRI_TOP);

  var best_dist = ab.x;
  var best_t = ab.y;
  var best_edge = 0.0;
  if (bc.x < best_dist) {
    best_dist = bc.x;
    best_t = bc.y;
    best_edge = 1.0;
  }
  if (ca.x < best_dist) {
    best_dist = ca.x;
    best_t = ca.y;
    best_edge = 2.0;
  }

  let led = edge_led_color(best_edge, best_t);
  let falloff = exp(-best_dist * FALLOFF_K);
  let rgb = led * falloff + vec3f(AMBIENT);
  return vec4f(rgb, 1.0);
}
