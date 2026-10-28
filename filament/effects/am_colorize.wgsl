// Colorize: AbsentMotion's colorize (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Tints the whole layer with one hue while keeping its light and shade.
// --track@hue:Hue,0,360,0.0,0.5
// --track@amount:Amount,0,1,0.5,0.001
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
  amount: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let rgb: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let l: f32 = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let _skTemp1 = (_uniform0.hue * 0.00277777785) * 6.0 + vec3<f32>(0.0, 4.0, 2.0);
    let tint: vec3<f32> = clamp(abs((_skTemp1 - 6.0 * floor(_skTemp1 / 6.0)) - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    var _skTemp2: vec3<f32>;
    if l < 0.5 {
      _skTemp2 = (tint * l) * 2.0;
    } else {
      _skTemp2 = mix(tint, vec3<f32>(1.0), vec3<f32>(l * 2.0 - 1.0));
    }
    let colored: vec3<f32> = _skTemp2;
    return vec4<f32>(vec3<f32>(mix(rgb, colored, vec3<f32>(clamp(_uniform0.amount, 0.0, 1.0)))) * src.w, src.w);
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
