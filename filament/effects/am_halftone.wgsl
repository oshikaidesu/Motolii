// Halftone: AbsentMotion's halftone (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Printed dots that grow where the picture is dark.
// --track@cell:Size,2,200,10.0,1
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
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_a: f32 = radians(_uniform0.angle);
    let _1_cs: f32 = cos(_0_a);
    let _2_sn: f32 = sin(_0_a);
    let _3_s: f32 = max(_uniform0.cell, 2.0);
    let _4_p: vec2<f32> = vec2<f32>(_1_cs * _stageIn.sk_FragCoord.x + _2_sn * _stageIn.sk_FragCoord.y, -_2_sn * _stageIn.sk_FragCoord.x + _1_cs * _stageIn.sk_FragCoord.y);
    let _5_m: vec2<f32> = (floor(_4_p / _3_s) + 0.5) * _3_s;
    let _6_c: vec4<f32> = textureSample(image_Texture, image_Sampler, vec2<f32>(_1_cs * _5_m.x - _2_sn * _5_m.y, _2_sn * _5_m.x + _1_cs * _5_m.y) / _uniform0.resolution);
    let _7_u: vec4<f32> = vec4<f32>(_6_c.xyz / max(_6_c.w, 0.0001), _6_c.w);
    let _8_radius: f32 = (0.7071 * _3_s) * sqrt(max(1.0 - f32(dot(_7_u.xyz, vec3<f32>(0.2126, 0.7152, 0.0722))), 0.0));
    let _9_ink: f32 = f32(saturate((_8_radius - distance(_4_p, _5_m)) + 0.5));
    let _10_rgb: vec3<f32> = mix(vec3<f32>(1.0), select(vec3<f32>(0.0), _7_u.xyz, vec3<bool>(_uniform0.colored > 0.5)), vec3<f32>(_9_ink));
    (*_stageOut).sk_FragColor = vec4<f32>(_10_rgb * _6_c.w, _6_c.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
