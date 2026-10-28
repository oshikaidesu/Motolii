// Directional Blur: AbsentMotion's directionalblur (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --about:Motion blur by averaging samples along a directional angle.
// --track@blurLength:Length,0,2000,20.0,1
// --track@angle:Angle,0,360,0.0,1
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
  blurLength: f32,
  angle: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _0_rad: f32 = radians(_uniform0.angle);
    let _1_dir: vec2<f32> = vec2<f32>(cos(_0_rad), sin(_0_rad));
    var _2_accum: vec4<f32> = vec4<f32>(0.0);
    {
      var _3_i: i32 = 0;
      for (; _3_i < 32; _3_i = _3_i + i32(1)) {
        {
          let _4_t: f32 = f32(_3_i) * 0.0322580636;
          let _5_offset: vec2<f32> = (_1_dir * _4_t) * _uniform0.blurLength;
          let _6_sampleCoord: vec2<f32> = _stageIn.sk_FragCoord.xy - _5_offset;
          _2_accum = _2_accum + textureSample(image_Texture, image_Sampler, _6_sampleCoord / _uniform0.resolution);
        }
      }
    }
    (*_stageOut).sk_FragColor = _2_accum * 0.03125;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
