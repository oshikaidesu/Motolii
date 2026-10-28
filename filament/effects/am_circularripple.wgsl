// Circular Ripple: AbsentMotion's circularripple (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --about:Makes circular ripples spread out from a point, like a drop falling into water.
// --track@R_center_x:Center x,-1,2,0.5,0.01
// --track@R_center_y:Center y,-1,2,0.5,0.01
// --track@frequency:Frequency,0,100,20.0,0.1
// --track@strength:Strength,-1,1,0.03,0.001
// --track@phase:Phase,-1000,1000,0.0,0.001
// --track@radius:Radius,0,0.8,0.3,0.001
// --track@feather:Feather,0,1,0.1,0.001
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
  R_center: vec2<f32>,
  frequency: f32,
  strength: f32,
  phase: f32,
  radius: f32,
  feather: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let m: f32 = min((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).x, (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))).y);
    let d: vec2<f32> = coord - _uniform0.R_center * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let r: f32 = length(d) / m;
    if r <= 0.0 {
      {
        return textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
      }
    }
    let edge: f32 = max(_uniform0.feather * _uniform0.radius, 0.0005);
    let fade: f32 = 1.0 - smoothstep(_uniform0.radius - edge, _uniform0.radius, r);
    let wave: f32 = sin(6.28318548 * (_uniform0.frequency * r - _uniform0.phase));
    return textureSample(image_Texture, image_Sampler, ((coord + (((((d / (r * m)) * wave) * _uniform0.strength) * m) * 0.5) * fade) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
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
