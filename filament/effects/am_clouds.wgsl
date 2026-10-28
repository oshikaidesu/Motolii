// Clouds: AbsentMotion's clouds (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --about:Covers the opaque parts of the layer with a soft cloud pattern, also handy as a noise source.
// --track@parallax_x:Parallax x,-5000,5000,0,1
// --track@parallax_y:Parallax y,-5000,5000,0,1
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@intensity:Intensity,0.25,0.75,0.5,0.001
// --track@seed:Seed,0,5,0.0,0.01
// --track@color1_x:Color 1 R,0,1,0,0.01
// --track@color1_y:Color 1 G,0,1,0,0.01
// --track@color1_z:Color 1 B,0,1,0,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,1,0.01
// --track@color2_y:Color 2 G,0,1,1,0.01
// --track@color2_z:Color 2 B,0,1,1,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
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
  parallax: vec2<f32>,
  offset: vec2<f32>,
  scale: f32,
  stretch: vec2<f32>,
  intensity: f32,
  seed: f32,
  color1: vec4<f32>,
  color2: vec4<f32>,
  octaves: f32,
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
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let unit: f32 = (0.4 * _uniform0.resolution.y) * max(_uniform0.scale, 0.01);
    let p: vec2<f32> = ((coord - _uniform0.resolution * 0.5) - _uniform0.offset) / (unit * max(abs(_uniform0.stretch), vec2<f32>(0.01)));
    var _0_p: vec3<f32> = vec3<f32>(p + _uniform0.seed * vec2<f32>(23.7, 41.9), 0.0);
    var _1_sum: f32 = 0.0;
    var _2_amp: f32 = 0.5;
    var _3_norm: f32 = 0.0;
    {
      var _4_i: i32 = 0;
      for (; _4_i < 9; _4_i = _4_i + i32(1)) {
        {
          if f32(_4_i) >= _uniform0.octaves {
            break;
          }
          _1_sum = _1_sum + _2_amp * vnoise_ff3(_0_p);
          _3_norm = _3_norm + _2_amp;
          _2_amp = _2_amp * 0.55;
          _0_p = vec3<f32>((_0_p.xy * 2.03 + _uniform0.parallax / unit) + vec2<f32>(17.13, 9.71), _0_p.z * 1.7 + 3.1);
        }
      }
    }
    var v: f32 = _1_sum / max(_3_norm, 0.0001);
    v = smoothstep(0.0, 1.0, clamp(((v - 0.5) * 2.6 + (_uniform0.intensity - 0.5) * 2.0) + 0.5, 0.0, 1.0));
    let rgb: vec3<f32> = mix(vec3<f32>(_uniform0.color1.xyz), vec3<f32>(_uniform0.color2.xyz), vec3<f32>(v));
    let a: f32 = mix(f32(_uniform0.color1.w), f32(_uniform0.color2.w), v) * f32(src.w);
    return vec4<f32>(vec3<f32>(rgb) * f32(a), f32(a));
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
