// Noise: AbsentMotion's noise (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --track@amount:Strength,0,1,0.25,0.001
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@overshoot:Overshoot,0.5,2,2.0,0.001
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
  amount: f32,
  mono: f32,
  freeze: f32,
  extremes: f32,
  offset: vec2<f32>,
  overshoot: f32,
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
    let n: vec2<f32> = floor(coord - _uniform0.offset);
    var _skTemp1: f32;
    if _uniform0.freeze > 0.5 {
      _skTemp1 = 0.0;
    } else {
      let _skTemp2 = floor(_uniform0.time * 60.0 + 0.001);
      _skTemp1 = (_skTemp2 - 4096.0 * floor(_skTemp2 / 4096.0));
    }
    let frame: f32 = _skTemp1;
    let k: vec3<f32> = vec3<f32>(n, frame);
    var v: vec3<f32> = vec3<f32>(hash13_ff3(k), hash13_ff3(k + 19.7), hash13_ff3(k + 41.3));
    if _uniform0.mono > 0.5 {
      v = v.xxx;
    }
    if _uniform0.extremes > 0.5 {
      v = step(vec3<f32>(0.5), v);
    }
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let rgb: vec3<f32> = clamp(base * mix(vec3<f32>(1.0), v * _uniform0.overshoot, vec3<f32>(clamp(_uniform0.amount, 0.0, 1.0))), vec3<f32>(0.0), vec3<f32>(1.0));
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
