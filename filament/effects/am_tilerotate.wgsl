// Tile Rotate: AbsentMotion's tilerotate (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Divides the layer into a grid of tiles and turns the picture inside every tile on its own.
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@count:Count,0.5,200,10.0,0.1
// --track@stretch_x:Stretch x,0.01,10,1,0.01
// --track@stretch_y:Stretch y,0.01,10,1,0.01
// --track@random:Randomize,0,1,0.0,0.01
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
  rotation: f32,
  offset: vec2<f32>,
  count: f32,
  stretch: vec2<f32>,
  angle: f32,
  random: f32,
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
    var _10_l: vec2<f32> = ((_8_g - _9_n) - 0.5) * _7_dims;
    var _11_q: vec3<f32> = fract((_9_n + _uniform0.seed * 7.31).xyx * 0.1031);
    _11_q = _11_q + dot(_11_q, _11_q.yzx + 33.33);
    let _12_r: f32 = radians(_uniform0.rotation + (_uniform0.random * 360.0) * (fract((_11_q.x + _11_q.y) * _11_q.z) - 0.5));
    let _13_rc: f32 = cos(_12_r);
    let _14_rs: f32 = sin(_12_r);
    _10_l = vec2<f32>(_13_rc * _10_l.x + _14_rs * _10_l.y, -_14_rs * _10_l.x + _13_rc * _10_l.y);
    var _15_q: vec2<f32> = (_9_n + 0.5) * _7_dims + _10_l;
    _15_q = vec2<f32>(_3_cs * _15_q.x - _4_sn * _15_q.y, _4_sn * _15_q.x + _3_cs * _15_q.y) + _5_c;
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, _15_q / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
