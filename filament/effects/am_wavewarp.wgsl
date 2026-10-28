// Wave Warp: AbsentMotion's wavewarp (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Bends the layer with a sine wave: Spacing sets how many waves cross it, Magnitude how far they push, Angle the direction the waves run and Warp Angle the direction they push.
// --track@phase:Phase,0,500,0.0,0.01
// --track@angle:Angle,-3600,3600,0.0,1
// --track@spacing:Spacing,0,500,20.0,0.1
// --track@magnitude:Magnitude,0,30,4.0,0.1
// --track@warpAngle:Warp Angle,-3600,3600,90.0,1
// --track@speed:Speed,0,10,0.0,0.1
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
  screen: vec4<f32>,
  time: f32,
  phase: f32,
  angle: f32,
  spacing: f32,
  magnitude: f32,
  warpAngle: f32,
  speed: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_a: f32 = radians(_uniform0.angle);
    let _1_b: f32 = radians(_uniform0.warpAngle);
    let _2_u: f32 = dot(_stageIn.sk_FragCoord.xy / max(_uniform0.resolution, vec2<f32>(1.0)), vec2<f32>(cos(_0_a), sin(_0_a)));
    let _3_d: f32 = ((_uniform0.magnitude * min(_uniform0.screen.z, _uniform0.screen.w)) * 0.01) * sin((_uniform0.spacing * _2_u + 6.28318548 * _uniform0.phase) + _uniform0.time * _uniform0.speed);
    (*_stageOut).sk_FragColor = textureSample(image_Texture, image_Sampler, (_stageIn.sk_FragCoord.xy + _3_d * vec2<f32>(cos(_1_b), sin(_1_b))) / _uniform0.resolution);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
