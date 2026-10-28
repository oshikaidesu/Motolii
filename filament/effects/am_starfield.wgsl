// Starfield: AbsentMotion's starfield (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Fills the opaque parts of the layer with a deep field of stars. Offset moves through the field, with nearby stars moving farther than distant stars.
// --track@offset_x:Offset x,-100000,100000,0,1
// --track@offset_y:Offset y,-100000,100000,0,1
// --track@scale:Scale,0.01,9,0.3,0.001
// --track@brightness:Brightness,0.001,3,0.015,0.001
// --track@darkMatter:Dark Matter,0.001,2,0.3,0.001
diagnostic(off, derivative_uniformity);
diagnostic(off, chromium.unreachable_code);
struct FSIn {
  @builtin(position) sk_FragCoord: vec4<f32>,
};
struct FSOut {
  @location(0) sk_FragColor: vec4<f32>,
};
struct Knobs {
  resolution: vec2<f32>,
  time: f32,
  offset: vec2<f32>,
  scale: f32,
  brightness: f32,
  darkMatter: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn hash13_ff3(_skParam0: vec3<f32>) -> f32 {
  var p = _skParam0;
  {
    p = fract(p * 0.1031);
    p = p + dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let c: vec2<f32> = _uniform0.resolution * 0.5;
    let zoom: f32 = max(_uniform0.scale, 0.01) * 3.33333325;
    var rgb: vec3<f32> = vec3<f32>(0.012, 0.02, 0.04);
    let gain: f32 = _uniform0.brightness * 66.66667;
    {
      var k: i32 = 0;
      for (; k < 7; k = k + i32(1)) {
        {
          let depth: f32 = 1.0 + f32(k) * 0.55;
          let cellSize: f32 = ((46.0 / pow(zoom, 0.25)) * depth) / (1.0 + 0.15 * f32(k));
          let q: vec2<f32> = ((coord - c) + _uniform0.offset / depth) / cellSize;
          let n: vec2<f32> = floor(q);
          var _0_q: vec3<f32> = fract((n + f32(k) * 31.7).xyx * vec3<f32>(0.1031, 0.103, 0.0973));
          _0_q = _0_q + dot(_0_q, _0_q.yzx + 33.33);
          let h: vec2<f32> = fract((_0_q.xx + _0_q.yz) * _0_q.zy);
          let starAt: vec2<f32> = (n + 0.15) + 0.7 * h;
          let d: f32 = length((q - starAt) * cellSize);
          var _1_q: vec3<f32> = fract((n * 1.31 + f32(k) * 7.9).xyx * 0.1031);
          _1_q = _1_q + dot(_1_q, _1_q.yzx + 33.33);
          let b: f32 = fract((_1_q.x + _1_q.y) * _1_q.z);
          let lit: f32 = step(0.55, b) * pow(b, 6.0);
          let core: f32 = exp((-d * d) / (0.9 + (1.6 * b) * b));
          let halo: f32 = exp(-d / (1.6 + 2.0 * b)) * 0.25;
          let tint: vec3<f32> = mix(vec3<f32>(0.75, 0.85, 1.0), mix(vec3<f32>(1.0, 0.7, 0.95), vec3<f32>(0.7, 1.0, 0.95), vec3<f32>(h.y)), vec3<f32>(h.x * 0.6));
          let _2_p: vec3<f32> = vec3<f32>(((coord - c) + _uniform0.offset / depth) / (220.0 / zoom), f32(k) * 3.1);
          let _3_i: vec3<f32> = floor(_2_p);
          var _4_f: vec3<f32> = fract(_2_p);
          _4_f = ((_4_f * _4_f) * _4_f) * (_4_f * (_4_f * 6.0 - 15.0) + 10.0);
          let _5_a: f32 = mix(hash13_ff3(_3_i), hash13_ff3(_3_i + vec3<f32>(1.0, 0.0, 0.0)), _4_f.x);
          let _6_b: f32 = mix(hash13_ff3(_3_i + vec3<f32>(0.0, 1.0, 0.0)), hash13_ff3(_3_i + vec3<f32>(1.0, 1.0, 0.0)), _4_f.x);
          let _7_c: f32 = mix(hash13_ff3(_3_i + vec3<f32>(0.0, 0.0, 1.0)), hash13_ff3(_3_i + vec3<f32>(1.0, 0.0, 1.0)), _4_f.x);
          let _8_d: f32 = mix(hash13_ff3(_3_i + vec3<f32>(0.0, 1.0, 1.0)), hash13_ff3(_3_i + vec3<f32>(1.0)), _4_f.x);
          let dust: f32 = mix(mix(_5_a, _6_b, _4_f.y), mix(_7_c, _8_d, _4_f.y), _4_f.z);
          let veil: f32 = mix(1.0, smoothstep(0.25, 0.75, dust), clamp(_uniform0.darkMatter, 0.0, 1.0));
          rgb = rgb + (((((tint * lit) * (core + halo)) * veil) * gain) * 1.6) / depth;
          rgb = rgb + ((vec3<f32>(0.03, 0.035, 0.07) * dust) * max(_uniform0.darkMatter - 0.5, 0.0)) * 0.4;
        }
      }
    }
    rgb = 1.0 - exp(-rgb * 1.6);
    return vec4<f32>(vec3<f32>(rgb) * src.w, src.w);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
