// Hexagon Tile Shift: AbsentMotion's hexagontileshift (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Divides the layer into hexagonal tiles and slides the picture inside every tile in a random direction.
// --track@strength:Strength,0,1,0.1,0.001
// --track@offset_x:Offset x,-5000,5000,0,1
// --track@offset_y:Offset y,-5000,5000,0,1
// --track@count:Count,0.5,200,10.0,0.1
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
  angle: f32,
  seed: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn layerSize__f2() -> vec2<f32> {
  {
    return select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _5_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _6_unit: f32 = layerSize__f2().y / (2.475 * max(_uniform0.count, 0.5));
    let _7_a: f32 = radians(_uniform0.angle);
    let _8_cs: f32 = cos(_7_a);
    let _9_sn: f32 = sin(_7_a);
    let _10_c: vec2<f32> = layerSize__f2() * 0.5 + _uniform0.offset;
    var _11_p: vec2<f32> = _5_coord - _10_c;
    _11_p = vec2<f32>(_8_cs * _11_p.x + _9_sn * _11_p.y, -_9_sn * _11_p.x + _8_cs * _11_p.y) / _6_unit;
    const _12_r: vec2<f32> = vec2<f32>(1.0, 1.73205078);
    const _13_h: vec2<f32> = vec2<f32>(0.5, 0.8660254);
    let _14_a: vec2<f32> = (_11_p - _12_r * floor(_11_p / _12_r)) - _13_h;
    let _skTemp1 = _11_p - _13_h;
    let _15_b: vec2<f32> = (_skTemp1 - _12_r * floor(_skTemp1 / _12_r)) - _13_h;
    let _16_g: vec2<f32> = select(_15_b, _14_a, vec2<bool>(dot(_14_a, _14_a) < dot(_15_b, _15_b)));
    let _17_id: vec2<f32> = floor(((_11_p - _16_g) * vec2<f32>(1.0, 0.5773503)) * 2.0 + 0.5);
    var _18_q: vec3<f32> = fract((_17_id + _uniform0.seed * 13.17).xyx * vec3<f32>(0.1031, 0.103, 0.0973));
    _18_q = _18_q + dot(_18_q, _18_q.yzx + 33.33);
    let _19_h: vec2<f32> = fract((_18_q.xx + _18_q.yz) * _18_q.zy);
    let _20_t: f32 = _19_h.x * 6.28318548;
    let _21_d: vec2<f32> = ((vec2<f32>(cos(_20_t), sin(_20_t)) * _19_h.y) * _uniform0.strength) * min(layerSize__f2().x, layerSize__f2().y);
    let _22_inside: vec2<f32> = step(vec2<f32>(0.0), _5_coord) * step(_5_coord, layerSize__f2());
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, ((_5_coord + _21_d) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * f32(_22_inside.x * _22_inside.y);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
