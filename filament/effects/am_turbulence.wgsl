// Turbulence: AbsentMotion's turbulence (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --track@parallax_x:Parallax x,-5000,5000,0,1
// --track@parallax_y:Parallax y,-5000,5000,0,1
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@evolution:Evolution,-50,50,0.0,0.01
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@intensity:Intensity,0.1,1.5,1.0,0.01
// --track@seed:Seed,0,5,0.0,0.01
// --track@color1_x:Color 1 R,0,1,0,0.01
// --track@color1_y:Color 1 G,0,1,0,0.01
// --track@color1_z:Color 1 B,0,1,0,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,1,0.01
// --track@color2_y:Color 2 G,0,1,1,0.01
// --track@color2_z:Color 2 B,0,1,1,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
// --track@alpha:Alpha,0,1,1.0,0.001
// --track@blend:Blending Mode,0,1,0.0,0.01
// --track@bias:Bias,-1,1,0.0,0.01
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
  evolution: f32,
  scale: f32,
  stretch: vec2<f32>,
  intensity: f32,
  seed: f32,
  color1: vec4<f32>,
  color2: vec4<f32>,
  alpha: f32,
  blend: f32,
  bias: f32,
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
fn blendWith_f3ff3f3(m: f32, b: vec3<f32>, s: vec3<f32>) -> vec3<f32> {
  {
    if m < 0.5 {
      return s;
    }
    if m < 1.5 {
      return b * s;
    }
    if m < 2.5 {
      return (b + s) - b * s;
    }
    if m < 3.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), b));
    }
    if m < 4.5 {
      {
        let d: vec3<f32> = mix(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, step(b, vec3<f32>(0.25)));
        return mix(b - ((1.0 - 2.0 * s) * b) * (1.0 - b), b + (2.0 * s - 1.0) * (d - b), step(vec3<f32>(0.5), s));
      }
    }
    if m < 5.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), s));
    }
    if m < 6.5 {
      return min(b + s, vec3<f32>(1.0));
    }
    if m < 7.5 {
      return min(b, s);
    }
    if m < 8.5 {
      return max(b, s);
    }
    if m < 9.5 {
      return abs(b - s);
    }
    if m < 10.5 {
      return min(b / max(1.0 - s, vec3<f32>(0.0001)), vec3<f32>(1.0));
    }
    return 1.0 - min((1.0 - b) / max(s, vec3<f32>(0.0001)), vec3<f32>(1.0));
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
    var _0_p: vec3<f32> = vec3<f32>(p + _uniform0.seed * vec2<f32>(37.1, 17.3), _uniform0.evolution);
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
          _2_amp = _2_amp * 0.5;
          _0_p = vec3<f32>((_0_p.xy * 2.03 + _uniform0.parallax / unit) + vec2<f32>(17.13, 9.71), _0_p.z * 1.7 + 3.1);
        }
      }
    }
    var v: f32 = _1_sum / max(_3_norm, 0.0001);
    v = clamp((0.465 + ((v - 0.5) * 1.12) * _uniform0.intensity) + _uniform0.bias * 0.5, 0.0, 1.0);
    let fill: vec3<f32> = mix(vec3<f32>(_uniform0.color1.xyz), vec3<f32>(_uniform0.color2.xyz), vec3<f32>(v));
    let amount: f32 = clamp(_uniform0.alpha, 0.0, 1.0) * mix(f32(_uniform0.color1.w), f32(_uniform0.color2.w), v);
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let rgb: vec3<f32> = mix(base, clamp(blendWith_f3ff3f3(_uniform0.blend, base, fill), vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(amount));
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
