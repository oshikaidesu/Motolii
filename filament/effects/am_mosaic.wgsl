// Mosaic: AbsentMotion's mosaic (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Divides the layer into squares and fills each one with its average color.
// --track@count:Count,1,500,20.0,1
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
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
  count: f32,
  stretch: vec2<f32>,
  offset: vec2<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_dims: vec2<f32> = (max(abs(_uniform0.stretch), vec2<f32>(0.01)) * _uniform0.resolution.y) / max(_uniform0.count, 1.0);
    let _1_o: vec2<f32> = _uniform0.resolution * 0.5 + _uniform0.offset;
    let _2_n: vec2<f32> = floor((_stageIn.sk_FragCoord.xy - _1_o) / _0_dims);
    var _3_sum: vec4<f32> = vec4<f32>(0.0);
    {
      var _4_j: i32 = 0;
      for (; _4_j < 4; _4_j = _4_j + i32(1)) {
        {
          {
            var _5_i: i32 = 0;
            for (; _5_i < 4; _5_i = _5_i + i32(1)) {
              {
                _3_sum = _3_sum + vec4<f32>(textureSample(image_Texture, image_Sampler, (_1_o + (_2_n + (vec2<f32>(f32(_5_i), f32(_4_j)) + 0.5) * 0.25) * _0_dims) / _uniform0.resolution));
              }
            }
          }
        }
      }
    }
    (*_stageOut).sk_FragColor = vec4<f32>(_3_sum * 0.0625);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
