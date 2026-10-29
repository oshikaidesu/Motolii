// Mask Blur: AbsentMotion's maskblur (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --picture@backdrop:Behind
// --about:Blurs the layers behind this one inside its shape; brighter parts of the layer blur more, so a gradient makes a tilt-shift.
// --track@strength:Strength,0,300,30.0,0.1
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
  strength: f32,
  invert: f32,
  layerRect: vec4<f32>,
  backdropBox: vec4<f32>,
};
@group(0) @binding(4) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var backdropPic_Sampler: sampler;
@group(0) @binding(2) var backdropPic_Texture: texture_2d<f32>;
fn backdropAt_h4f2(p: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(backdropPic_Texture, backdropPic_Sampler, _uniform0.backdropBox.xy + ((p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * _uniform0.backdropBox.zw);
    if c.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    let u: vec3<f32> = clamp(c.xyz / c.w, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(mix(1.055 * pow(u, vec3<f32>(0.416666657)) - 0.055, u * 12.92, step(u, vec3<f32>(0.0031308))) * c.w, c.w);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    var lum: f32 = dot(vec3<f32>(src.xyz) / f32(src.w), vec3<f32>(0.2126, 0.7152, 0.0722));
    if _uniform0.invert > 0.5 {
      lum = 1.0 - lum;
    }
    let r: f32 = max(_uniform0.strength, 0.0) * clamp(lum, 0.0, 1.0);
    if r < 0.5 {
      return backdropAt_h4f2(coord) * src.w;
    }
    let spin: f32 = fract(sin(dot(floor(coord), vec2<f32>(12.9898, 78.233))) * 43758.5469) * 6.28318548;
    var sum: vec4<f32> = vec4<f32>(0.0);
    {
      var i: i32 = 0;
      for (; i < 64; i = i + i32(1)) {
        {
          let k: f32 = (f32(i) + 0.5) * 0.015625;
          let a: f32 = f32(i) * 2.39996314 + spin;
          sum = sum + backdropAt_h4f2(coord + (sqrt(k) * r) * vec2<f32>(cos(a), sin(a)));
        }
      }
    }
    return sum * (src.w * 0.015625);
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
