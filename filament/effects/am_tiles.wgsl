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
    let _2_a: f32 = radians(_uniform0.angle);
    let _3_cs: f32 = cos(_2_a);
    let _4_sn: f32 = sin(_2_a);
    var _5_p: vec2<f32> = (_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) - _1_c;
    _5_p = vec2<f32>(_3_cs * _5_p.x + _4_sn * _5_p.y, -_4_sn * _5_p.x + _3_cs * _5_p.y);
    if _uniform0.sizing > 0.5 {
      _5_p = _5_p / max(_uniform0.zoom, 0.005);
    } else {
      _5_p = _5_p * max(_uniform0.zoom, 0.005);
    }
    var _6_g: vec2<f32> = _5_p / max(layerSize__f2(), vec2<f32>(1.0)) + 0.5;
    if _uniform0.vertical > 0.5 {
      _6_g.y = _6_g.y - _uniform0.offset * floor(_6_g.x);
    } else {
      _6_g.x = _6_g.x - _uniform0.offset * floor(_6_g.y);
    }
    let _7_n: vec2<f32> = floor(_6_g);
    var _8_t: vec2<f32> = _6_g - _7_n;
    if _uniform0.mirror > 0.5 {
      _8_t = mix(_8_t, 1.0 - _8_t, (_7_n - 2.0 * floor(_7_n / 2.0)));
    }
    let _9_p: vec2<f32> = clamp(_8_t * layerSize__f2(), vec2<f32>(0.5), layerSize__f2() - 0.5);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_9_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
