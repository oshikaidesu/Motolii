// Sepia: AbsentMotion's sepia (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Warm brown tones, like an old photograph.
// --track@amount:Amount,0,1,1.0,0.01
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
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _1_color: vec4<f32> = textureSample(image_Texture, image_Sampler, ((_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let _2_c: vec4<f32> = vec4<f32>(_1_color.xyz / max(_1_color.w, 0.0001), _1_color.w);
    let _3_brown: vec3<f32> = dot(_2_c.xyz, vec3<f32>(0.2126, 0.7152, 0.0722)) * vec3<f32>(1.12, 0.92, 0.68) + vec3<f32>(0.04, 0.02, 0.0);
    (*_stageOut).sk_FragColor = vec4<f32>(mix(_2_c.xyz, saturate(_3_brown), vec3<f32>(f32(_uniform0.amount))) * _2_c.w, _2_c.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
