// Duotone: AbsentMotion's duotone (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Applies a two-color gradient map based on luminance with contrast and amount controls.
// --track@contrast:Contrast,0.0,2.0,1.0,0.05
// --track@amount:Amount,0.0,1.0,1.0,0.05
// --track@color1_x:Shadow Color R,0,1,0,0.01
// --track@color1_y:Shadow Color G,0,1,0,0.01
// --track@color1_z:Shadow Color B,0,1,0,0.01
// --track@color1_w:Shadow Color A,0,1,1,0.01
// --track@color2_x:Highlight Color R,0,1,1,0.01
// --track@color2_y:Highlight Color G,0,1,1,0.01
// --track@color2_z:Highlight Color B,0,1,1,0.01
// --track@color2_w:Highlight Color A,0,1,1,0.01
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
  contrast: f32,
  amount: f32,
  color1: vec4<f32>,
  color2: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    let a: f32 = c.w;
    if a < 0.001 {
      {
        return vec4<f32>(0.0);
      }
    }
    var rgb: vec3<f32> = c.xyz / a;
    var luminance: f32 = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    luminance = f32(clamp(f32(luminance - 0.5) * _uniform0.contrast + 0.5, 0.0, 1.0));
    let duotone: vec3<f32> = mix(_uniform0.color1.xyz, _uniform0.color2.xyz, vec3<f32>(luminance));
    rgb = vec3<f32>(mix(vec3<f32>(rgb), vec3<f32>(duotone), vec3<f32>(_uniform0.amount)));
    return vec4<f32>(rgb * a, a);
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
