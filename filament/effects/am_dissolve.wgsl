// Dissolve: AbsentMotion's dissolve (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Noise
// --about:Removes random pixels or blocks from the layer. Animate Strength for a dissolve transition; Seed chooses the pattern and Size sets the size of each speck.
// --track@progress:Strength,0,1,0.75,0.01
// --track@seed:Seed,0,5,0.0,0.01
// --track@grain:Size,1,200,1.0,1
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
  progress: f32,
  grain: f32,
  seed: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _1_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    var _2_p: vec2<f32> = floor(_1_coord / max(_uniform0.grain, 1.0)) + _uniform0.seed * vec2<f32>(17.13, 41.71);
    _2_p = fract(_2_p * vec2<f32>(123.34, 456.21));
    _2_p = _2_p + dot(_2_p, _2_p + 45.32);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_1_coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * f32(step(_uniform0.progress, fract(_2_p.x * _2_p.y)));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
