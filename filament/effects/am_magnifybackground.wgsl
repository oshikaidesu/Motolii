// Magnify Background: AbsentMotion's magnifybackground (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --picture@backdrop:Behind
// --about:Makes the layer a magnifying glass: its shape shows the layers behind it enlarged around its center.
// --track@scale:Scale,0.01,50,1.0,0.01
// --track@sampling:Sampling,0,1,1.0,0.01
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
  screen: vec4<f32>,
  scale: f32,
  magnify: f32,
  sampling: f32,
  layerRect: vec4<f32>,
  backdropBox: vec4<f32>,
};
@group(0) @binding(4) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var backdropPic_Sampler: sampler;
@group(0) @binding(2) var backdropPic_Texture: texture_2d<f32>;
fn srgbOf__h4h4(c: vec4<f32>) -> vec4<f32> {
  {
    if c.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    let u: vec3<f32> = clamp(c.xyz / c.w, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(mix(1.055 * pow(u, vec3<f32>(0.416666657)) - 0.055, u * 12.92, step(u, vec3<f32>(0.0031308))) * c.w, c.w);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec2<f32> = (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5;
    var p: vec2<f32> = c + (coord - c) / max(_uniform0.scale, 0.01);
    let m: f32 = textureSample(image_Texture, image_Sampler, ((select(coord, p, vec2<bool>(_uniform0.magnify > 0.5))) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w;
    if m <= 0.0 {
      return vec4<f32>(0.0);
    }
    if _uniform0.sampling < 0.5 {
      p = (_uniform0.screen.xy + floor(p - _uniform0.screen.xy)) + 0.5;
    }
    return srgbOf__h4h4(textureSample(backdropPic_Texture, backdropPic_Sampler, _uniform0.backdropBox.xy + ((p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * _uniform0.backdropBox.zw)) * m;
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
