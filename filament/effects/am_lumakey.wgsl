// Luma Key: AbsentMotion's lumakey (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Key
// --about:Removes the dark parts of the layer (or the light ones).
// --track@threshold:Threshold,0,1,0.3,0.01
// --track@softness:Softness,0,1,0.1,0.01
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
  threshold: f32,
  softness: f32,
  invert: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _1_lum: f32 = dot(vec3<f32>(vec4<f32>(_0_c.xyz / max(_0_c.w, 0.0001), _0_c.w).xyz), vec3<f32>(0.2126, 0.7152, 0.0722));
    var _skTemp1: f32;
    if _uniform0.softness > 0.0 {
      _skTemp1 = smoothstep(_uniform0.threshold, (_uniform0.threshold + _uniform0.softness) + 0.0001, _1_lum);
    } else {
      _skTemp1 = step(_uniform0.threshold, _1_lum);
    }
    let _2_keep: f32 = _skTemp1;
    var _skTemp2: f32;
    if _uniform0.invert > 0.5 {
      _skTemp2 = 1.0 - _2_keep;
    } else {
      _skTemp2 = _2_keep;
    }
    (*_stageOut).sk_FragColor = vec4<f32>(vec4<f32>(_0_c) * _skTemp2);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
