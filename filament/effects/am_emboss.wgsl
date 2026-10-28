// Emboss: AbsentMotion's emboss (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Applies an emboss effect using luminance gradients along a specified light angle.
// --track@angle:Light Angle,0.0,360.0,45.0,1.0
// --track@strength:Strength,0.0,4.0,1.0,0.1
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
  angle: f32,
  strength: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let current: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let aCurrent: f32 = f32(current.w);
    if aCurrent < 1e-05 {
      {
        return current;
      }
    }
    let rgbCur: vec3<f32> = vec3<f32>(vec3<f32>(current.xyz) / aCurrent);
    let rad: f32 = radians(_uniform0.angle);
    let dir: vec2<f32> = vec2<f32>(cos(rad), sin(rad));
    let sampleCoord: vec2<f32> = coord + dir;
    let neighbor: vec4<f32> = textureSample(image_Texture, image_Sampler, (sampleCoord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var rgbNeighbor: vec3<f32>;
    let aNeighbor: f32 = f32(neighbor.w);
    if aNeighbor < 1e-05 {
      {
        rgbNeighbor = vec3<f32>(0.0);
      }
    } else {
      {
        rgbNeighbor = vec3<f32>(vec3<f32>(neighbor.xyz) / aNeighbor);
      }
    }
    let lumCur: f32 = dot(rgbCur, vec3<f32>(0.299, 0.587, 0.114));
    let lumN: f32 = dot(rgbNeighbor, vec3<f32>(0.299, 0.587, 0.114));
    let delta: f32 = lumN - lumCur;
    var val: f32 = f32(0.5 + f32(delta) * _uniform0.strength);
    val = clamp(val, 0.0, 1.0);
    let outRgb: vec3<f32> = vec3<f32>(val);
    return vec4<f32>(vec3<f32>(vec3<f32>(outRgb) * aCurrent), f32(aCurrent));
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
