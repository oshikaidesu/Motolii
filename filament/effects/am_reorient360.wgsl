// 360° Reorient Sphere: AbsentMotion's reorient360 (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Turns a 360° panorama layer around inside its sphere, to level the horizon or change where the front is.
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
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let uv: vec2<f32> = coord / _uniform0.resolution;
    let lon: f32 = (uv.x - 0.5) * 6.28318548;
    let lat: f32 = (0.5 - uv.y) * 3.14159274;
    let d: vec3<f32> = vec3<f32>(cos(lat) * sin(lon), sin(lat), cos(lat) * cos(lon));
    if (((coord.x < 0.0) || (coord.y < 0.0)) || (coord.x > _uniform0.resolution.x)) || (coord.y > _uniform0.resolution.y) {
      return vec4<f32>(0.0);
    }
    var _0_d: vec3<f32> = d;
    let _1_y: f32 = radians(_uniform0.yaw);
    let _2_p: f32 = radians(_uniform0.pitch);
    let _3_r: f32 = radians(_uniform0.roll);
    _0_d = vec3<f32>(cos(_3_r) * _0_d.x - sin(_3_r) * _0_d.y, sin(_3_r) * _0_d.x + cos(_3_r) * _0_d.y, _0_d.z);
    _0_d = vec3<f32>(_0_d.x, cos(_2_p) * _0_d.y - sin(_2_p) * _0_d.z, sin(_2_p) * _0_d.y + cos(_2_p) * _0_d.z);
    let _4_d: vec3<f32> = vec3<f32>(cos(_1_y) * _0_d.x + sin(_1_y) * _0_d.z, _0_d.y, -sin(_1_y) * _0_d.x + cos(_1_y) * _0_d.z);
    let _5_lon: f32 = atan2(_4_d.x, _4_d.z);
    let _6_lat: f32 = asin(clamp(_4_d.y, -1.0, 1.0));
    let _7_uv: vec2<f32> = vec2<f32>(_5_lon * 0.159154937 + 0.5, 0.5 - _6_lat * 0.318309873);
    let _skTemp1 = _7_uv.x * _uniform0.resolution.x;
    let _skTemp2 = _uniform0.resolution.x;
    let _8_at: vec2<f32> = vec2<f32>((_skTemp1 - _skTemp2 * floor(_skTemp1 / _skTemp2)), clamp(_7_uv.y * _uniform0.resolution.y, 0.5, _uniform0.resolution.y - 0.5));
    return textureSample(image_Texture, image_Sampler, clamp(_8_at, vec2<f32>(0.5), _uniform0.resolution - 0.5) / _uniform0.resolution);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
