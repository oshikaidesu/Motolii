// Fractal Ridges: AbsentMotion's fractalridges (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --about:Fills the opaque parts of the layer with sharp, turbulent ridges of fractal noise.
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@evolution:Evolution,-50,50,0.0,0.01
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@intensity:Intensity,0.1,3,1.0,0.01
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
  offset: vec2<f32>,
  evolution: f32,
  scale: f32,
  stretch: vec2<f32>,
  intensity: f32,
  seed: f32,
  color1: vec4<f32>,
  color2: vec4<f32>,
  alpha: f32,
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
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let unit: f32 = (0.4 * _uniform0.resolution.y) * max(_uniform0.scale, 0.01);
    var p: vec3<f32> = vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.offset) / (unit * max(abs(_uniform0.stretch), vec2<f32>(0.01))) + _uniform0.seed * vec2<f32>(29.3, 13.1), _uniform0.evolution);
    var sum: f32 = 0.0;
    var amp: f32 = 0.5;
    var norm: f32 = 0.0;
    var weight: f32 = 1.0;
    {
      var i: i32 = 0;
      for (; i < 9; i = i + i32(1)) {
        {
          if f32(i) >= _uniform0.octaves {
            break;
          }
          let _0_i: vec3<f32> = floor(p);
          var _1_f: vec3<f32> = fract(p);
          _1_f = ((_1_f * _1_f) * _1_f) * (_1_f * (_1_f * 6.0 - 15.0) + 10.0);
          let _2_a: f32 = mix(hash13_ff3(_0_i), hash13_ff3(_0_i + vec3<f32>(1.0, 0.0, 0.0)), _1_f.x);
          let _3_b: f32 = mix(hash13_ff3(_0_i + vec3<f32>(0.0, 1.0, 0.0)), hash13_ff3(_0_i + vec3<f32>(1.0, 1.0, 0.0)), _1_f.x);
          let _4_c: f32 = mix(hash13_ff3(_0_i + vec3<f32>(0.0, 0.0, 1.0)), hash13_ff3(_0_i + vec3<f32>(1.0, 0.0, 1.0)), _1_f.x);
          let _5_d: f32 = mix(hash13_ff3(_0_i + vec3<f32>(0.0, 1.0, 1.0)), hash13_ff3(_0_i + vec3<f32>(1.0)), _1_f.x);
          var r: f32 = 1.0 - abs(mix(mix(_2_a, _3_b, _1_f.y), mix(_4_c, _5_d, _1_f.y), _1_f.z) * 2.0 - 1.0);
          r = (r * r) * weight;
          weight = clamp(r * 1.6, 0.0, 1.0);
          sum = sum + amp * r;
          norm = norm + amp;
          amp = amp * 0.5;
          p = vec3<f32>(p.xy * 2.07 + vec2<f32>(5.3, 11.7), p.z * 1.6 + 2.3);
        }
      }
    }
    let v: f32 = clamp(pow((sum / max(norm, 0.0001)) * 1.4, 1.0 / max(_uniform0.intensity, 0.1)), 0.0, 1.0);
    let fill: vec3<f32> = mix(vec3<f32>(_uniform0.color1.xyz), vec3<f32>(_uniform0.color2.xyz), vec3<f32>(v));
    let amount: f32 = clamp(_uniform0.alpha, 0.0, 1.0) * mix(f32(_uniform0.color1.w), f32(_uniform0.color2.w), v);
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    return vec4<f32>(vec3<f32>(mix(base, fill, vec3<f32>(amount))) * src.w, src.w);
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
