// Linear Streaks: AbsentMotion's linearstreaks (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --track@reach:Length,0,4000,100.0,1
// --track@alpha:Alpha,0,1,1.0,0.001
// --track@bias:Bias,-1,1,0.0,0.01
// --track@mode:Blending,0,1,0.0,0.01
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
  angle: f32,
  reach: f32,
  alpha: f32,
  bias: f32,
  mode: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if _uniform0.reach <= 0.0 {
      return src;
    }
    let a: f32 = radians(_uniform0.angle);
    let d: vec2<f32> = (vec2<f32>(cos(a), sin(a)) * _uniform0.reach) * 0.5;
    var hi: vec4<f32> = vec4<f32>(0.0);
    var sum: vec4<f32> = vec4<f32>(0.0);
    var lo: vec3<f32> = vec3<f32>(1.0);
    {
      var i: i32 = 0;
      for (; i < 48; i = i + i32(1)) {
        {
          let v: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, (coord - d * mix(_uniform0.bias - 1.0, _uniform0.bias + 1.0, f32(i) * 0.0212765951)) / _uniform0.resolution));
          hi = max(hi, v);
          sum = sum + v;
          if v.w > 0.002 {
            lo = min(lo, v.xyz / v.w);
          }
        }
      }
    }
    var _skTemp1: vec4<f32>;
    if _uniform0.mode < 0.5 {
      _skTemp1 = hi;
    } else {
      var _skTemp2: vec4<f32>;
      if _uniform0.mode < 1.5 {
        _skTemp2 = sum * 0.020833334;
      } else {
        _skTemp2 = vec4<f32>(lo * hi.w, hi.w);
      }
      _skTemp1 = _skTemp2;
    }
    let acc: vec4<f32> = _skTemp1;
    return vec4<f32>(mix(vec4<f32>(src), acc, vec4<f32>(clamp(_uniform0.alpha, 0.0, 1.0))));
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
