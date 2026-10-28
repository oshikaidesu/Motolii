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
    let _1_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _2_cell: f32 = layerSize__f2().y / (2.475 * max(_uniform0.count, 0.5));
    let _3_a: f32 = radians(_uniform0.angle);
    let _4_cs: f32 = cos(_3_a);
    let _5_sn: f32 = sin(_3_a);
    let _6_c: vec2<f32> = layerSize__f2() * 0.5 + _uniform0.offset;
    var _7_p: vec2<f32> = _1_coord - _6_c;
    _7_p = vec2<f32>(_4_cs * _7_p.x + _5_sn * _7_p.y, -_5_sn * _7_p.x + _4_cs * _7_p.y);
    let _8_dims: vec2<f32> = max(abs(_uniform0.stretch), vec2<f32>(0.01)) * _2_cell;
    let _9_g: vec2<f32> = _7_p / _8_dims;
    let _10_n: vec2<f32> = floor(_9_g);
    var _11_q: vec3<f32> = fract((_10_n + _uniform0.seed * 13.17).xyx * vec3<f32>(0.1031, 0.103, 0.0973));
    _11_q = _11_q + dot(_11_q, _11_q.yzx + 33.33);
    let _12_h: vec2<f32> = fract((_11_q.xx + _11_q.yz) * _11_q.zy);
    let _13_t: f32 = _12_h.x * 6.28318548;
    let _14_d: vec2<f32> = ((vec2<f32>(cos(_13_t), sin(_13_t)) * _12_h.y) * _uniform0.strength) * min(layerSize__f2().x, layerSize__f2().y);
    let _15_inside: vec2<f32> = step(vec2<f32>(0.0), _1_coord) * step(_1_coord, layerSize__f2());
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, ((_1_coord + _14_d) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * f32(_15_inside.x * _15_inside.y);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
