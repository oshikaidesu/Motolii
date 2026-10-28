// Hexagon Tiling: AbsentMotion's hexagontiling (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Repeats the middle of the layer in hexagonal tiles that fill the screen.
// --track@scale:Scale,0.05,10,1.0,0.01
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
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec2<f32> = _uniform0.resolution * 0.5;
    let _1_unit: f32 = (0.5 * min(_uniform0.resolution.x, _uniform0.resolution.y)) * max(_uniform0.scale, 0.05);
    let _2_a: f32 = radians(_uniform0.angle);
    let _3_cs: f32 = cos(_2_a);
    let _4_sn: f32 = sin(_2_a);
    var _5_p: vec2<f32> = _stageIn.sk_FragCoord.xy - _0_c;
    _5_p = vec2<f32>(_3_cs * _5_p.x + _4_sn * _5_p.y, -_4_sn * _5_p.x + _3_cs * _5_p.y) / _1_unit;
    const _6_r: vec2<f32> = vec2<f32>(1.0, 1.73205078);
    const _7_h: vec2<f32> = vec2<f32>(0.5, 0.8660254);
    let _8_ga: vec2<f32> = (_5_p - _6_r * floor(_5_p / _6_r)) - _7_h;
    let _skTemp1 = _5_p - _7_h;
    let _9_gb: vec2<f32> = (_skTemp1 - _6_r * floor(_skTemp1 / _6_r)) - _7_h;
    var _10_g: vec2<f32> = select(_9_gb, _8_ga, vec2<bool>(dot(_8_ga, _8_ga) < dot(_9_gb, _9_gb)));
    if _uniform0.mirror > 0.5 {
      {
        let _11_id: vec2<f32> = _5_p - _10_g;
        let _skTemp2 = floor(_11_id.x + 0.5) + floor((_11_id.y * 0.577350259) * 2.0 + 0.5);
        let _12_k: f32 = (_skTemp2 - 2.0 * floor(_skTemp2 / 2.0));
        _10_g.x = mix(_10_g.x, -_10_g.x, _12_k);
      }
    }
    _10_g = _10_g * _1_unit;
    _10_g = vec2<f32>(_3_cs * _10_g.x - _4_sn * _10_g.y, _4_sn * _10_g.x + _3_cs * _10_g.y);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, clamp(_0_c + _10_g, vec2<f32>(0.5), _uniform0.resolution - 0.5) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
