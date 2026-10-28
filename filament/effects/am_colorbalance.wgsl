// Color Balance: AbsentMotion's colorbalance (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Pushes the shadows, midtones and highlights towards their own colors (gray leaves a range unchanged).
// --track@shadows_x:Shadows R,0,1,0.1882,0.01
// --track@shadows_y:Shadows G,0,1,0.3765,0.01
// --track@shadows_z:Shadows B,0,1,0.6275,0.01
// --track@shadows_w:Shadows A,0,1,1,0.01
// --track@midtones_x:Midtones R,0,1,0.502,0.01
// --track@midtones_y:Midtones G,0,1,0.502,0.01
// --track@midtones_z:Midtones B,0,1,0.502,0.01
// --track@midtones_w:Midtones A,0,1,1,0.01
// --track@highlights_x:Highlights R,0,1,1,0.01
// --track@highlights_y:Highlights G,0,1,0.6902,0.01
// --track@highlights_z:Highlights B,0,1,0.4392,0.01
// --track@highlights_w:Highlights A,0,1,1,0.01
// --track@amount:Amount,0.0,1.0,0.5,0.01
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
  shadows: vec4<f32>,
  midtones: vec4<f32>,
  highlights: vec4<f32>,
  amount: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    let base: vec3<f32> = vec3<f32>(c.xyz) / f32(c.w);
    let l: f32 = dot(base, vec3<f32>(0.2126, 0.7152, 0.0722));
    let lows: f32 = 1.0 - smoothstep(0.0, 0.5, l);
    let highs: f32 = smoothstep(0.5, 1.0, l);
    let shift: vec3<f32> = (lows * (vec3<f32>(_uniform0.shadows.xyz) - 0.5) + ((1.0 - lows) - highs) * (vec3<f32>(_uniform0.midtones.xyz) - 0.5)) + highs * (vec3<f32>(_uniform0.highlights.xyz) - 0.5);
    let rgb: vec3<f32> = base + shift * clamp(_uniform0.amount, 0.0, 1.0);
    return vec4<f32>(vec3<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0))) * c.w, c.w);
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
