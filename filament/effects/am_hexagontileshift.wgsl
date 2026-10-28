// Hexagon Tile Shift: AbsentMotion's hexagontileshift (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Divides the layer into hexagonal tiles and slides the picture inside every tile in a random direction.
// --track@strength:Strength,0,1,0.1,0.001
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@count:Count,0.5,200,10.0,0.1
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
  angle: f32,
  seed: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _5_unit: f32 = _uniform0.resolution.y / (2.475 * max(_uniform0.count, 0.5));
    let _6_a: f32 = radians(_uniform0.angle);
    let _7_cs: f32 = cos(_6_a);
    let _8_sn: f32 = sin(_6_a);
    let _9_c: vec2<f32> = _uniform0.resolution * 0.5 + _uniform0.offset;
    var _10_p: vec2<f32> = _stageIn.sk_FragCoord.xy - _9_c;
    _10_p = vec2<f32>(_7_cs * _10_p.x + _8_sn * _10_p.y, -_8_sn * _10_p.x + _7_cs * _10_p.y) / _5_unit;
    const _11_r: vec2<f32> = vec2<f32>(1.0, 1.73205078);
    const _12_h: vec2<f32> = vec2<f32>(0.5, 0.8660254);
    let _13_a: vec2<f32> = (_10_p - _11_r * floor(_10_p / _11_r)) - _12_h;
    let _skTemp1 = _10_p - _12_h;
    let _14_b: vec2<f32> = (_skTemp1 - _11_r * floor(_skTemp1 / _11_r)) - _12_h;
    let _15_g: vec2<f32> = select(_14_b, _13_a, vec2<bool>(dot(_13_a, _13_a) < dot(_14_b, _14_b)));
    let _16_id: vec2<f32> = floor(((_10_p - _15_g) * vec2<f32>(1.0, 0.5773503)) * 2.0 + 0.5);
    var _17_q: vec3<f32> = fract((_16_id + _uniform0.seed * 13.17).xyx * vec3<f32>(0.1031, 0.103, 0.0973));
    _17_q = _17_q + dot(_17_q, _17_q.yzx + 33.33);
    let _18_h: vec2<f32> = fract((_17_q.xx + _17_q.yz) * _17_q.zy);
    let _19_t: f32 = _18_h.x * 6.28318548;
    let _20_d: vec2<f32> = ((vec2<f32>(cos(_19_t), sin(_19_t)) * _18_h.y) * _uniform0.strength) * min(_uniform0.resolution.x, _uniform0.resolution.y);
    let _21_inside: vec2<f32> = step(vec2<f32>(0.0), _stageIn.sk_FragCoord.xy) * step(_stageIn.sk_FragCoord.xy, _uniform0.resolution);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_stageIn.sk_FragCoord.xy + _20_d) / _uniform0.resolution) * f32(_21_inside.x * _21_inside.y);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
