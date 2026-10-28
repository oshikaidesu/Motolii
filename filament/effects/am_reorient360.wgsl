// 360° Reorient Sphere: AbsentMotion's reorient360 (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Turns a 360° panorama layer around inside its sphere, to level the horizon or change where the front is.
// --track@yaw:Yaw,-3600,3600,0.0,0.5
// --track@pitch:Pitch,-3600,3600,0.0,0.5
// --track@roll:Roll,-3600,3600,0.0,0.5
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
  yaw: f32,
  pitch: f32,
  roll: f32,
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
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let uv: vec2<f32> = coord / layerSize__f2();
    let lon: f32 = (uv.x - 0.5) * 6.28318548;
    let lat: f32 = (0.5 - uv.y) * 3.14159274;
    let d: vec3<f32> = vec3<f32>(cos(lat) * sin(lon), sin(lat), cos(lat) * cos(lon));
    var _skTemp1: bool;
    var _skTemp2: bool;
    if (coord.x < 0.0) || (coord.y < 0.0) {
      _skTemp2 = true;
    } else {
      _skTemp2 = coord.x > layerSize__f2().x;
    }
    if _skTemp2 {
      _skTemp1 = true;
    } else {
      _skTemp1 = coord.y > layerSize__f2().y;
    }
    if _skTemp1 {
      return vec4<f32>(0.0);
    }
    var _1_d: vec3<f32> = d;
    let _2_y: f32 = radians(_uniform0.yaw);
    let _3_p: f32 = radians(_uniform0.pitch);
    let _4_r: f32 = radians(_uniform0.roll);
    _1_d = vec3<f32>(cos(_4_r) * _1_d.x - sin(_4_r) * _1_d.y, sin(_4_r) * _1_d.x + cos(_4_r) * _1_d.y, _1_d.z);
    _1_d = vec3<f32>(_1_d.x, cos(_3_p) * _1_d.y - sin(_3_p) * _1_d.z, sin(_3_p) * _1_d.y + cos(_3_p) * _1_d.z);
    let _5_d: vec3<f32> = vec3<f32>(cos(_2_y) * _1_d.x + sin(_2_y) * _1_d.z, _1_d.y, -sin(_2_y) * _1_d.x + cos(_2_y) * _1_d.z);
    let _6_lon: f32 = atan2(_5_d.x, _5_d.z);
    let _7_lat: f32 = asin(clamp(_5_d.y, -1.0, 1.0));
    let _8_uv: vec2<f32> = vec2<f32>(_6_lon * 0.159154937 + 0.5, 0.5 - _7_lat * 0.318309873);
    let _skTemp3 = _8_uv.x * layerSize__f2().x;
    let _skTemp4 = layerSize__f2().x;
    let _9_at: vec2<f32> = vec2<f32>((_skTemp3 - _skTemp4 * floor(_skTemp3 / _skTemp4)), clamp(_8_uv.y * layerSize__f2().y, 0.5, layerSize__f2().y - 0.5));
    let _10_p: vec2<f32> = clamp(_9_at, vec2<f32>(0.5), layerSize__f2() - 0.5);
    return textureSample(image_Texture, image_Sampler, (_10_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0))));
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
