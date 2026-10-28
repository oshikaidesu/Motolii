// Hue & Saturation: AbsentMotion's huesaturation (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Turns colors around the color wheel and makes them stronger or weaker; vibrance strengthens the duller colors most.
// --track@hue:Hue,-360,360,60.0,1
// --track@saturation:Saturation,0,3,1.4,0.01
// --track@vibrance:Vibrance,-1,1,0.0,0.01
// --track@lightness:Lightness,-1,1,0.0,0.01
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
  hue: f32,
  saturation: f32,
  vibrance: f32,
  lightness: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    let _1_color: vec4<f32> = textureSample(image_Texture, image_Sampler, _stageIn.sk_FragCoord.xy / _uniform0.resolution);
    let _2_c: vec4<f32> = vec4<f32>(_1_color.xyz / max(_1_color.w, 0.0001), _1_color.w);
    const _3_k: vec3<f32> = vec3<f32>(0.57735);
    let _4_a: f32 = radians(_uniform0.hue);
    var _5_v: vec3<f32> = vec3<f32>(_2_c.xyz);
    _5_v = (_5_v * cos(_4_a) + cross(_3_k, _5_v) * sin(_4_a)) + (_3_k * dot(_3_k, _5_v)) * (1.0 - cos(_4_a));
    var _6_rgb: vec3<f32> = vec3<f32>(_5_v);
    _6_rgb = mix(vec3<f32>(dot(_6_rgb, vec3<f32>(0.2126, 0.7152, 0.0722))), _6_rgb, vec3<f32>(f32(_uniform0.saturation)));
    let _7_chromaNow: f32 = max(_6_rgb.x, max(_6_rgb.y, _6_rgb.z)) - min(_6_rgb.x, min(_6_rgb.y, _6_rgb.z));
    _6_rgb = mix(vec3<f32>(dot(_6_rgb, vec3<f32>(0.2126, 0.7152, 0.0722))), _6_rgb, vec3<f32>(f32(1.0 + _uniform0.vibrance * f32(1.0 - saturate(_7_chromaNow))))) + f32(_uniform0.lightness);
    (*_stageOut).sk_FragColor = vec4<f32>(saturate(_6_rgb) * _2_c.w, _2_c.w);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
