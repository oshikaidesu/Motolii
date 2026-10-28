// Static: AbsentMotion's static (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --about:Flickering gray noise, like a television with no signal.
// --track@amount:Amount,0,1,0.4,0.01
// --track@grain:Size,0.5,50,2.0,0.1
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
  grain: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _2_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _3_color: vec4<f32> = textureSample(image_Texture, image_Sampler, (_2_coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let _4_c: vec4<f32> = vec4<f32>(_3_color.xyz / max(_3_color.w, 0.0001), _3_color.w);
    let _skTemp1 = floor((_uniform0.time + 0.000999) * 30.0);
    let _5_frame: f32 = (_skTemp1 - 4096.0 * floor(_skTemp1 / 4096.0));
    var _6_p: vec3<f32> = vec3<f32>(floor(_2_coord / max(_uniform0.grain, 0.5)), _5_frame);
    _6_p = fract(_6_p * 0.1031);
    _6_p = _6_p + dot(_6_p, _6_p.zyx + 31.32);
    let _7_n: f32 = f32(fract((_6_p.x + _6_p.y) * _6_p.z));
    (*_stageOut).sk_FragColor = vec4<f32>(mix(_4_c.xyz, vec3<f32>(_7_n), vec3<f32>(f32(_uniform0.amount))) * _4_c.w, _4_c.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
