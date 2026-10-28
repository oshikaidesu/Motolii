// Lift Gamma Gain: AbsentMotion's colorwheels (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Color wheels for the shadows (lift), midtones (gamma) and highlights (gain): a hue, how far to push it, and a level.
// --track@liftHue:Lift Hue,0,360,210.0,1
// --track@liftAmount:Lift Amount,0,1,0.3,0.01
// --track@liftLevel:Lift Level,-1,1,0.0,0.01
// --track@gammaHue:Gamma Hue,0,360,0.0,1
// --track@gammaAmount:Gamma Amount,0,1,0.0,0.01
// --track@gammaLevel:Gamma Level,-1,1,0.0,0.01
// --track@gainHue:Gain Hue,0,360,35.0,1
// --track@gainAmount:Gain Amount,0,1,0.3,0.01
// --track@gainLevel:Gain Level,-1,1,0.0,0.01
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
  liftHue: f32,
  liftAmount: f32,
  liftLevel: f32,
  gammaHue: f32,
  gammaAmount: f32,
  gammaLevel: f32,
  gainHue: f32,
  gainAmount: f32,
  gainLevel: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn chroma_f3f(hue: f32) -> vec3<f32> {
  {
    let k: vec3<f32> = clamp(abs(fract(hue * 0.00277777785 + vec3<f32>(1.0, 0.6666667, 0.333333343)) * 6.0 - 3.0) - 1.0, vec3<f32>(0.0), vec3<f32>(1.0));
    return k - dot(k, vec3<f32>(0.2126, 0.7152, 0.0722));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    let base: vec3<f32> = vec3<f32>(c.xyz) / f32(c.w);
    let lift: vec3<f32> = (0.5 * chroma_f3f(_uniform0.liftHue)) * _uniform0.liftAmount + _uniform0.liftLevel;
    let gain: vec3<f32> = (1.0 + chroma_f3f(_uniform0.gainHue) * _uniform0.gainAmount) + _uniform0.gainLevel;
    let gamma: vec3<f32> = max((1.0 + chroma_f3f(_uniform0.gammaHue) * _uniform0.gammaAmount) + _uniform0.gammaLevel, vec3<f32>(0.01));
    let rgb: vec3<f32> = pow(max(gain * (base + lift * (1.0 - base)), vec3<f32>(0.0)), 1.0 / gamma);
    return vec4<f32>(vec3<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0))) * c.w, c.w);
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
