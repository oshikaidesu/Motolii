// Split Tone: AbsentMotion's splittone (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Tints the shadows towards one hue and the highlights towards another, keeping black, white and brightness.
// --track@shadowAmount:Shadow Amount,0,1,0.6,0.01
// --track@highlightAmount:Highlight Amount,0,1,0.6,0.01
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
  shadowHue: f32,
  shadowAmount: f32,
  highlightHue: f32,
  highlightAmount: f32,
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
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    let base: vec3<f32> = vec3<f32>(c.xyz) / f32(c.w);
    let l: f32 = dot(base, vec3<f32>(0.2126, 0.7152, 0.0722));
    let reach: f32 = 2.0 * min(l, 1.0 - l);
    let dark: vec3<f32> = clamp(l + chroma_f3f(_uniform0.shadowHue) * reach, vec3<f32>(0.0), vec3<f32>(1.0));
    let light: vec3<f32> = clamp(l + chroma_f3f(_uniform0.highlightHue) * reach, vec3<f32>(0.0), vec3<f32>(1.0));
    var rgb: vec3<f32> = mix(base, dark, vec3<f32>(clamp(_uniform0.shadowAmount, 0.0, 1.0) * (1.0 - l)));
    rgb = mix(rgb, light, vec3<f32>(clamp(_uniform0.highlightAmount, 0.0, 1.0) * l));
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
