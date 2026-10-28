// Tiles: AbsentMotion's tiles (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Repeats the layer to fill the screen. Raise Crop to make smaller tiles, with optional mirroring, row offset and rotation. Legacy Scale preserves older projects.
// --track@zoom:Crop,0.005,20,1.0,0.005
// --track@offset:Offset,-100,100,0.0,0.01
// --track@angle:Angle,-1800,1800,0.0,0.5
// --track@sizing:Sizing,0,1,0.0,0.01
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
  zoom: f32,
  sizing: f32,
  offset: f32,
  mirror: f32,
  vertical: f32,
  angle: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_c: vec2<f32> = _uniform0.resolution * 0.5;
    let _1_a: f32 = radians(_uniform0.angle);
    let _2_cs: f32 = cos(_1_a);
    let _3_sn: f32 = sin(_1_a);
    var _4_p: vec2<f32> = _stageIn.sk_FragCoord.xy - _0_c;
    _4_p = vec2<f32>(_2_cs * _4_p.x + _3_sn * _4_p.y, -_3_sn * _4_p.x + _2_cs * _4_p.y);
    if _uniform0.sizing > 0.5 {
      _4_p = _4_p / max(_uniform0.zoom, 0.005);
    } else {
      _4_p = _4_p * max(_uniform0.zoom, 0.005);
    }
    var _5_g: vec2<f32> = _4_p / max(_uniform0.resolution, vec2<f32>(1.0)) + 0.5;
    if _uniform0.vertical > 0.5 {
      _5_g.y = _5_g.y - _uniform0.offset * floor(_5_g.x);
    } else {
      _5_g.x = _5_g.x - _uniform0.offset * floor(_5_g.y);
    }
    let _6_n: vec2<f32> = floor(_5_g);
    var _7_t: vec2<f32> = _5_g - _6_n;
    if _uniform0.mirror > 0.5 {
      _7_t = mix(_7_t, 1.0 - _7_t, (_6_n - 2.0 * floor(_6_n / 2.0)));
    }
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, clamp(_7_t * _uniform0.resolution, vec2<f32>(0.5), _uniform0.resolution - 0.5) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
