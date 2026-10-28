// Tile Shift: AbsentMotion's tileshift (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Divides the layer into a grid of tiles and slides the picture inside every tile in a random direction.
// --track@strength:Strength,0,1,0.1,0.001
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@count:Count,0.5,200,10.0,0.1
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@angle:Angle,-3600,3600,0.0,0.5
// --track@seed:Seed,0,5,0.0,0.01
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
  offset: vec2<f32>,
  count: f32,
  stretch: vec2<f32>,
  angle: f32,
  seed: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _1_cell: f32 = _uniform0.resolution.y / (2.475 * max(_uniform0.count, 0.5));
    let _2_a: f32 = radians(_uniform0.angle);
    let _3_cs: f32 = cos(_2_a);
    let _4_sn: f32 = sin(_2_a);
    let _5_c: vec2<f32> = _uniform0.resolution * 0.5 + _uniform0.offset;
    var _6_p: vec2<f32> = _stageIn.sk_FragCoord.xy - _5_c;
    _6_p = vec2<f32>(_3_cs * _6_p.x + _4_sn * _6_p.y, -_4_sn * _6_p.x + _3_cs * _6_p.y);
    let _7_dims: vec2<f32> = max(abs(_uniform0.stretch), vec2<f32>(0.01)) * _1_cell;
    let _8_g: vec2<f32> = _6_p / _7_dims;
    let _9_n: vec2<f32> = floor(_8_g);
    var _10_q: vec3<f32> = fract((_9_n + _uniform0.seed * 13.17).xyx * vec3<f32>(0.1031, 0.103, 0.0973));
    _10_q = _10_q + dot(_10_q, _10_q.yzx + 33.33);
    let _11_h: vec2<f32> = fract((_10_q.xx + _10_q.yz) * _10_q.zy);
    let _12_t: f32 = _11_h.x * 6.28318548;
    let _13_d: vec2<f32> = ((vec2<f32>(cos(_12_t), sin(_12_t)) * _11_h.y) * _uniform0.strength) * min(_uniform0.resolution.x, _uniform0.resolution.y);
    let _14_inside: vec2<f32> = step(vec2<f32>(0.0), _stageIn.sk_FragCoord.xy) * step(_stageIn.sk_FragCoord.xy, _uniform0.resolution);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_stageIn.sk_FragCoord.xy + _13_d) / _uniform0.resolution) * f32(_14_inside.x * _14_inside.y);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
