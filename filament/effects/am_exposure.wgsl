// Exposure: AbsentMotion's exposure (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --about:Simulates photographic exposure with stops, offset, and gamma correction.
// --track@stops:Stops,-4.0,4.0,0.7,0.1
// --track@offset:Offset,-0.5,0.5,0.0,0.01
// --track@gamma:Gamma,0.1,10.0,1.0,0.1
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
  stops: f32,
  offset: f32,
  gamma: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let a: f32 = f32(c.w);
    if a <= 0.0 {
      return vec4<f32>(0.0);
    }
    var rgb: vec3<f32> = vec3<f32>(vec3<f32>(c.xyz) / a);
    rgb = vec3<f32>(pow(max(vec3<f32>(rgb) + _uniform0.offset, vec3<f32>(0.0)), vec3<f32>(vec3<f32>(f32(1.0 / _uniform0.gamma)))) * pow(2.0, _uniform0.stops));
    rgb = clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(vec3<f32>(rgb) * a), f32(a));
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
