// Tile: AbsentMotion's tile (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Repeat
// --track@scale:Scale,0.02,1,0.5,0.01
// --track@offset_x:Offset x,-1,2,0,0.01
// --track@offset_y:Offset y,-1,2,0,0.01
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
  scale: f32,
  offset: vec2<f32>,
  mirror: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_t: vec2<f32> = (_stageIn.sk_FragCoord.xy / _uniform0.resolution - _uniform0.offset) / max(_uniform0.scale, 0.02);
    var _1_f: vec2<f32> = fract(_0_t);
    if _uniform0.mirror > 0.5 {
      let _skTemp1 = floor(_0_t);
      _1_f = mix(_1_f, 1.0 - _1_f, (_skTemp1 - 2.0 * floor(_skTemp1 / 2.0)));
    }
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_1_f * _uniform0.resolution) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
