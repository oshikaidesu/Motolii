// Hexagon Tiling: AbsentMotion's hexagontiling (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Repeats the middle of the layer in hexagonal tiles that fill the screen.
// --track@scale:Scale,0.05,10,1.0,0.01
// --track@angle:Angle,-3600,3600,0.0,0.5
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
  angle: f32,
  mirror: f32,
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
    let _1_c: vec2<f32> = layerSize__f2() * 0.5;
    let _2_unit: f32 = (0.5 * min(layerSize__f2().x, layerSize__f2().y)) * max(_uniform0.scale, 0.05);
    let _3_a: f32 = radians(_uniform0.angle);
    let _4_cs: f32 = cos(_3_a);
    let _5_sn: f32 = sin(_3_a);
    var _6_p: vec2<f32> = (_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) - _1_c;
    _6_p = vec2<f32>(_4_cs * _6_p.x + _5_sn * _6_p.y, -_5_sn * _6_p.x + _4_cs * _6_p.y) / _2_unit;
    const _7_r: vec2<f32> = vec2<f32>(1.0, 1.73205078);
    const _8_h: vec2<f32> = vec2<f32>(0.5, 0.8660254);
    let _9_ga: vec2<f32> = (_6_p - _7_r * floor(_6_p / _7_r)) - _8_h;
    let _skTemp1 = _6_p - _8_h;
    let _10_gb: vec2<f32> = (_skTemp1 - _7_r * floor(_skTemp1 / _7_r)) - _8_h;
    var _11_g: vec2<f32> = select(_10_gb, _9_ga, vec2<bool>(dot(_9_ga, _9_ga) < dot(_10_gb, _10_gb)));
    if _uniform0.mirror > 0.5 {
      {
        let _12_id: vec2<f32> = _6_p - _11_g;
        let _skTemp2 = floor(_12_id.x + 0.5) + floor((_12_id.y * 0.577350259) * 2.0 + 0.5);
        let _13_k: f32 = (_skTemp2 - 2.0 * floor(_skTemp2 / 2.0));
        _11_g.x = mix(_11_g.x, -_11_g.x, _13_k);
      }
    }
    _11_g = _11_g * _2_unit;
    _11_g = vec2<f32>(_4_cs * _11_g.x - _5_sn * _11_g.y, _5_sn * _11_g.x + _4_cs * _11_g.y);
    let _14_p: vec2<f32> = clamp(_1_c + _11_g, vec2<f32>(0.5), layerSize__f2() - 0.5);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_14_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
