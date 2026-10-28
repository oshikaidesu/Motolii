// Roughen Edges: AbsentMotion's roughenedges (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --pass@h:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --about:Eats into the edges of the opaque parts with noise, for a torn, worn or burnt outline.
// --track@blur:Border,0,300,12.0,0.1
// --track@roughness:Roughness,0,1,0.6,0.001
// --track@scale:Scale,0.05,20,1.0,0.01
// --track@evolution:Evolution,-50,50,0.0,0.01
// --track@seed:Seed,0,5,0.0,0.01
// --track@octaves:Octaves,1,9,6.0,1
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
  blur: f32,
  roughness: f32,
  scale: f32,
  evolution: f32,
  seed: f32,
  octaves: f32,
  layerRect: vec4<f32>,
  passindex: f32,
};
@group(0) @binding(6) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var h_Sampler: sampler;
@group(0) @binding(2) var h_Texture: texture_2d<f32>;
@group(0) @binding(5) var blurPicture_Sampler: sampler;
@group(0) @binding(4) var blurPicture_Texture: texture_2d<f32>;
fn layerAt__f2() -> vec2<f32> {
  {
    return select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn gauss__h4f2f2i(p: vec2<f32>, by: vec2<f32>, R_from: i32) -> vec4<f32> {
  {
    let sigma: f32 = max(_uniform0.blur, 0.0) * 0.333333343;
    if sigma < 0.5 {
      var _skTemp1: vec4<f32>;
      if R_from == 0 {
        _skTemp1 = textureSample(image_Texture, image_Sampler, p / _uniform0.resolution);
      } else {
        _skTemp1 = textureSample(h_Texture, h_Sampler, p / _uniform0.resolution);
      }
      return _skTemp1;
    }
    var _0_all: vec4<f32> = vec4<f32>(0.0);
    var weight: f32 = 0.0;
    {
      var i: i32 = -96;
      for (; i <= 96; i = i + i32(1)) {
        {
          let x: f32 = f32(i);
          if abs(x) > (3.0 * sigma) {
            continue;
          }
          let w: f32 = exp((-x * x) / ((2.0 * sigma) * sigma));
          var _skTemp2: vec4<f32>;
          if R_from == 0 {
            _skTemp2 = textureSample(image_Texture, image_Sampler, (p + by * x) / _uniform0.resolution);
          } else {
            _skTemp2 = textureSample(h_Texture, h_Sampler, (p + by * x) / _uniform0.resolution);
          }
          let c: vec4<f32> = _skTemp2;
          _0_all = _0_all + vec4<f32>(vec4<f32>(c) * w);
          weight = weight + w;
        }
      }
    }
    return vec4<f32>(vec4<f32>(_0_all) / weight);
  }
}
fn hash13_ff3(_skParam0: vec3<f32>) -> f32 {
  var p = _skParam0;
  {
    p = fract(p * 0.1031);
    p = p + dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
  }
}
fn vnoise_ff3(p: vec3<f32>) -> f32 {
  {
    let i: vec3<f32> = floor(p);
    var f: vec3<f32> = fract(p);
    f = ((f * f) * f) * (f * (f * 6.0 - 15.0) + 10.0);
    let a: f32 = mix(hash13_ff3(i), hash13_ff3(i + vec3<f32>(1.0, 0.0, 0.0)), f.x);
    let b: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 1.0, 0.0)), hash13_ff3(i + vec3<f32>(1.0, 1.0, 0.0)), f.x);
    let c: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 0.0, 1.0)), hash13_ff3(i + vec3<f32>(1.0, 0.0, 1.0)), f.x);
    let d: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 1.0, 1.0)), hash13_ff3(i + vec3<f32>(1.0)), f.x);
    return mix(mix(a, b, f.y), mix(c, d, f.y), f.z);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if (src.w <= 0.0) || (_uniform0.blur <= 0.0) {
      return src;
    }
    let b: f32 = f32(textureSample(blurPicture_Texture, blurPicture_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w);
    var _0_p: vec3<f32> = vec3<f32>(coord / ((max(_uniform0.scale, 0.05) * max(_uniform0.blur, 1.0)) * 1.5) + _uniform0.seed * vec2<f32>(9.1, 3.7), _uniform0.evolution);
    var _1_sum: f32 = 0.0;
    var _2_amp: f32 = 0.5;
    var _3_norm: f32 = 0.0;
    {
      var _4_i: i32 = 0;
      for (; _4_i < 9; _4_i = _4_i + i32(1)) {
        {
          if f32(_4_i) >= clamp(floor(_uniform0.octaves + 0.5), 1.0, 9.0) {
            break;
          }
          _1_sum = _1_sum + _2_amp * vnoise_ff3(_0_p);
          _3_norm = _3_norm + _2_amp;
          _2_amp = _2_amp * 0.55;
          _0_p = vec3<f32>(_0_p.xy * 2.03 + vec2<f32>(17.13, 9.71), _0_p.z * 1.7 + 3.1);
        }
      }
    }
    let n: f32 = _1_sum / max(_3_norm, 0.0001);
    let keep: f32 = smoothstep(0.45, 0.55, (b + ((n - 0.5) * 2.0) * _uniform0.roughness) - 0.25 * _uniform0.roughness);
    return src * f32(keep);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp3: vec4<f32>;
    if _uniform0.passindex < 0.5 {
      _skTemp3 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(1.0, 0.0), 0);
    } else {
      var _skTemp4: vec4<f32>;
      if _uniform0.passindex < 1.5 {
        _skTemp4 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(0.0, 1.0), 1);
      } else {
        _skTemp4 = effect__h4f2(_stageIn.sk_FragCoord.xy - layerAt__f2());
      }
      _skTemp3 = _skTemp4;
    }
    (*_stageOut).sk_FragColor = _skTemp3;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
