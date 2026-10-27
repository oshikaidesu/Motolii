// Sharpen: AbsentMotion's sharpen (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --track@amount:Strength,0.0,20,1.0,0.1
// --track@radius:Radius,0.5,10,1.0,0.5
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
  amount: f32,
  radius: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn tap_h3f2(p: vec2<f32>) -> vec3<f32> {
  {
    let s: vec4<f32> = textureSample(image_Texture, image_Sampler, p / _uniform0.resolution);
    return s.xyz / max(s.w, 0.001);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.001 {
      return vec4<f32>(0.0);
    }
    let col: vec3<f32> = c.xyz / c.w;
    let r: f32 = _uniform0.radius;
    let avg: vec3<f32> = ((((((((tap_h3f2(coord + vec2<f32>(-r)) + 2.0 * tap_h3f2(coord + vec2<f32>(0.0, -r))) + tap_h3f2(coord + vec2<f32>(r, -r))) + 2.0 * tap_h3f2(coord + vec2<f32>(-r, 0.0))) + 4.0 * col) + 2.0 * tap_h3f2(coord + vec2<f32>(r, 0.0))) + tap_h3f2(coord + vec2<f32>(-r, r))) + 2.0 * tap_h3f2(coord + vec2<f32>(0.0, r))) + tap_h3f2(coord + vec2<f32>(r))) * 0.0625;
    let sharpened: vec3<f32> = vec3<f32>(clamp(vec3<f32>(col) + _uniform0.amount * vec3<f32>(col - avg), vec3<f32>(0.0), vec3<f32>(1.0)));
    return vec4<f32>(sharpened * c.w, c.w);
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
