// Vignette: AbsentMotion's vignette (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Darkens the layer toward its edges outside a rounded shape: Size sets the shape, Roundness how round it is, Feather how soft its edge is, Strength how dark it gets and Tint how much it takes on Color. Punch Out makes the edges transparent instead.
// --track@extent:Size,0.001,2,0.95,0.001
// --track@rounding:Roundness,0,9.9,1.5,0.01
// --track@feather:Feather,0.001,1,0.5,0.001
// --track@strength:Strength,0,1,0.8,0.001
// --track@tint:Tint,0,1,0.2,0.01
// --track@color_x:Color R,0,1,0,0.01
// --track@color_y:Color G,0,1,0,0.01
// --track@color_z:Color B,0,1,0,0.01
// --track@color_w:Color A,0,1,1,0.01
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
  extent: f32,
  strength: f32,
  rounding: f32,
  feather: f32,
  tint: f32,
  color: vec4<f32>,
  punch: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.0001 {
      return vec4<f32>(0.0);
    }
    let n: vec2<f32> = abs((coord / max(_uniform0.resolution, vec2<f32>(1.0)) - 0.5) * 2.0);
    let p: f32 = max(_uniform0.rounding, 0.5);
    let d: f32 = pow(pow(n.x, p) + pow(n.y, p), 1.0 / p);
    let s: f32 = max(_uniform0.extent, 0.001);
    let v: f32 = smoothstep(s * (1.0 - _uniform0.feather * 0.25), s * (1.0 + _uniform0.feather * 3.0) + 0.001, d) * clamp(_uniform0.strength, 0.0, 1.0);
    if _uniform0.punch > 0.5 {
      return c * f32(1.0 - v);
    }
    var rgb: vec3<f32> = vec3<f32>(c.xyz / c.w);
    rgb = mix(max(rgb - v * (1.0 - _uniform0.tint), vec3<f32>(0.0)), vec3<f32>(_uniform0.color.xyz), vec3<f32>(_uniform0.tint * v));
    return vec4<f32>(vec3<f32>(rgb) * c.w, c.w);
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
