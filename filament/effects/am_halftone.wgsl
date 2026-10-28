// Halftone: AbsentMotion's halftone (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Printed dots that grow where the picture is dark.
// --track@cell:Size,2,200,10.0,1
// --track@angle:Angle,-360,360,45.0,1
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
  cell: f32,
  angle: f32,
  colored: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_coord: vec2<f32> = _stageIn.sk_FragCoord.xy - (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let _1_a: f32 = radians(_uniform0.angle);
    let _2_cs: f32 = cos(_1_a);
    let _3_sn: f32 = sin(_1_a);
    let _4_s: f32 = max(_uniform0.cell, 2.0);
    let _5_p: vec2<f32> = vec2<f32>(_2_cs * _0_coord.x + _3_sn * _0_coord.y, -_3_sn * _0_coord.x + _2_cs * _0_coord.y);
    let _6_m: vec2<f32> = (floor(_5_p / _4_s) + 0.5) * _4_s;
    let _7_c: vec4<f32> = textureSample(image_Texture, image_Sampler, (vec2<f32>(_2_cs * _6_m.x - _3_sn * _6_m.y, _3_sn * _6_m.x + _2_cs * _6_m.y) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let _8_u: vec4<f32> = vec4<f32>(_7_c.xyz / max(_7_c.w, 0.0001), _7_c.w);
    let _9_radius: f32 = (0.7071 * _4_s) * sqrt(max(1.0 - f32(dot(_8_u.xyz, vec3<f32>(0.2126, 0.7152, 0.0722))), 0.0));
    let _10_ink: f32 = f32(saturate((_9_radius - distance(_5_p, _6_m)) + 0.5));
    let _11_rgb: vec3<f32> = mix(vec3<f32>(1.0), select(vec3<f32>(0.0), _8_u.xyz, vec3<bool>(_uniform0.colored > 0.5)), vec3<f32>(_10_ink));
    (*_stageOut).sk_FragColor = vec4<f32>(_11_rgb * _7_c.w, _7_c.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
