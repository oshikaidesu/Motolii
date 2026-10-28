// Block Dissolve: AbsentMotion's blockdissolve (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --about:Removes random grid-aligned blocks from the layer. Animate Strength to take away more and more blocks.
// --track@strength:Strength,0,1,0.75,0.001
// --track@blockSize:Size,1,500,10.0,0.1
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@seed:Seed,0,5,0.0,0.01
// --track@fade:Fade,0,0.5,0.0,0.001
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
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
  strength: f32,
  blockSize: f32,
  stretch: vec2<f32>,
  seed: f32,
  fade: f32,
  offset: vec2<f32>,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _1_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _2_dims: vec2<f32> = max(abs(_uniform0.stretch), vec2<f32>(0.01)) * max(_uniform0.blockSize, 1.0);
    var _3_q: vec3<f32> = fract((floor(((_1_coord - (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5) - _uniform0.offset) / _2_dims) + _uniform0.seed * 11.7).xyx * 0.1031);
    _3_q = _3_q + dot(_3_q, _3_q.yzx + 33.33);
    let _4_h: f32 = fract((_3_q.x + _3_q.y) * _3_q.z);
    let _5_m: f32 = _uniform0.strength * (1.0 + 2.0 * _uniform0.fade) - _uniform0.fade;
    var _skTemp1: f32;
    if _uniform0.fade > 0.0 {
      _skTemp1 = smoothstep(_5_m - _uniform0.fade, _5_m + _uniform0.fade, _4_h);
    } else {
      _skTemp1 = step(_uniform0.strength, _4_h);
    }
    let _6_keep: f32 = _skTemp1;
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_1_coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * f32(_6_keep);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
