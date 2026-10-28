// Pixelate: AbsentMotion's pixelate (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Turns the layer into blocks of one color each, taken from the middle of the block, on a grid centered on the layer (or on the frame with Screen Space). Stretch, Offset and Angle shape the grid and Vignette darkens each block toward its edges.
// --track@blockSize:Size,1,500,20.0,1
// --track@stretch_x:Stretch x,0.01,100,1,0.01
// --track@stretch_y:Stretch y,0.01,100,1,0.01
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@angle:Angle,-3600,3600,0.0,1
// --track@vignette:Vignette,0,1,0.0,0.01
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
  screen: vec4<f32>,
  time: f32,
  blockSize: f32,
  stretch: vec2<f32>,
  offset: vec2<f32>,
  angle: f32,
  vignette: f32,
  screenSpace: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp1: vec2<f32>;
    if _uniform0.screenSpace > 0.5 {
      _skTemp1 = _uniform0.screen.xy + _uniform0.screen.zw * 0.5;
    } else {
      _skTemp1 = _uniform0.resolution * 0.5;
    }
    let _0_origin: vec2<f32> = _skTemp1 + _uniform0.offset;
    let _1_cell: vec2<f32> = max(_uniform0.blockSize * _uniform0.stretch, vec2<f32>(1.0));
    let _2_a: f32 = radians(_uniform0.angle);
    let _3_c: f32 = cos(_2_a);
    let _4_s: f32 = sin(_2_a);
    let _5_d: vec2<f32> = _stageIn.sk_FragCoord.xy - _0_origin;
    let _6_p: vec2<f32> = vec2<f32>(_3_c * _5_d.x + _4_s * _5_d.y, _3_c * _5_d.y - _4_s * _5_d.x) / _1_cell;
    let _7_b: vec2<f32> = floor(_6_p);
    let _8_m: vec2<f32> = (_7_b + 0.5) * _1_cell;
    var _9_col: vec4<f32> = textureSample(image_Texture, image_Sampler, (_0_origin + vec2<f32>(_3_c * _8_m.x - _4_s * _8_m.y, _4_s * _8_m.x + _3_c * _8_m.y)) / _uniform0.resolution);
    if (_uniform0.vignette > 0.0) && (_9_col.w > 0.001) {
      {
        let _10_e: vec2<f32> = abs((_6_p - _7_b) - 0.5) * 2.0;
        let _11_un: vec3<f32> = _9_col.xyz / _9_col.w;
        _9_col = vec4<f32>((max(_11_un - f32(_uniform0.vignette * (0.04 + 0.08 * dot(_10_e, _10_e))), vec3<f32>(0.0)) * _9_col.w), _9_col.w);
      }
    }
    (*_stageOut).sk_FragColor = _9_col;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
