// Soft Glow: AbsentMotion's softglow (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --pass@h:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --about:Screens a soft, blurred copy of the picture over it for a dreamy glow, mostly from the highlights.
// --track@blur:Size,0,300,20.0,0.1
// --track@brightness:Brightness,0,3,1.0,0.01
// --track@contrast:Contrast,-1,1,0.2,0.01
// --track@highlights:Highlights,0,1,0.5,0.01
// --track@color_x:Color R,0,1,1,0.01
// --track@color_y:Color G,0,1,0.3333,0.01
// --track@color_z:Color B,0,1,0.4,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@blend:Blend,0,1,0.25,0.01
// --track@opacity:Opacity,0,1,1.0,0.01
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
  blur: f32,
  brightness: f32,
  contrast: f32,
  highlights: f32,
  color: vec4<f32>,
  blend: f32,
  opacity: f32,
  layerRect: vec4<f32>,
  passindex: f32,
};
@group(0) @binding(6) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var h_Sampler: sampler;
@group(0) @binding(2) var h_Texture: texture_2d<f32>;
@group(0) @binding(5) var blurPicture_Sampler: sampler;
@group(0) @binding(4) var blurPicture_Texture: texture_2d<f32>;
fn layerAt__f2() -> vec2<f32> {
  {
    return select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn gauss__h4f2f2i(p: vec2<f32>, by: vec2<f32>, R_from: i32) -> vec4<f32> {
  {
    let sigma: f32 = max(_uniform0.blur, 0.0) * 0.333333343;
    if sigma < 0.5 {
      var _skTemp1: vec4<f32>;
      if R_from == 0 {
        _skTemp1 = textureSample(image_Texture, image_Sampler, p / _uniform0.resolution);
      } else {
        _skTemp1 = textureSample(h_Texture, h_Sampler, p / _uniform0.resolution);
      }
      return _skTemp1;
    }
    var _0_all: vec4<f32> = vec4<f32>(0.0);
    var weight: f32 = 0.0;
    {
      var i: i32 = -96;
      for (; i <= 96; i = i + i32(1)) {
        {
          let x: f32 = f32(i);
          if abs(x) > (3.0 * sigma) {
            continue;
          }
          let w: f32 = exp((-x * x) / ((2.0 * sigma) * sigma));
          var _skTemp2: vec4<f32>;
          if R_from == 0 {
            _skTemp2 = textureSample(image_Texture, image_Sampler, (p + by * x) / _uniform0.resolution);
          } else {
            _skTemp2 = textureSample(h_Texture, h_Sampler, (p + by * x) / _uniform0.resolution);
          }
          let c: vec4<f32> = _skTemp2;
          _0_all = _0_all + vec4<f32>(vec4<f32>(c) * w);
          weight = weight + w;
        }
      }
    }
    return vec4<f32>(vec4<f32>(_0_all) / weight);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let g: vec4<f32> = textureSample(blurPicture_Texture, blurPicture_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _skTemp3: vec3<f32>;
    if g.w > 0.0 {
      _skTemp3 = vec3<f32>(g.xyz) / f32(g.w);
    } else {
      _skTemp3 = vec3<f32>(0.0);
    }
    let soft: vec3<f32> = _skTemp3;
    let l: f32 = clamp(((dot(soft, vec3<f32>(0.2126, 0.7152, 0.0722)) - 0.5) * (1.0 + _uniform0.contrast) + 0.5) * _uniform0.brightness, 0.0, 1.0);
    let w: f32 = mix(1.0, smoothstep(0.25, 1.0, l), clamp((_uniform0.highlights - 0.5) * 2.0, 0.0, 1.0));
    let tint: vec3<f32> = mix(vec3<f32>(1.0), vec3<f32>(_uniform0.color.xyz), vec3<f32>(clamp(_uniform0.blend, 0.0, 1.0)));
    let s: vec3<f32> = (((vec3<f32>(l) * w) * tint) * f32(_uniform0.color.w)) * clamp(_uniform0.opacity, 0.0, 1.0);
    let spill: f32 = f32(g.w) * (1.0 - f32(c.w));
    let a: f32 = f32(c.w) + (((spill * l) * w) * f32(_uniform0.color.w)) * clamp(_uniform0.opacity, 0.0, 1.0);
    let rgb: vec3<f32> = vec3<f32>(c.xyz) * (1.0 - s) + s * (f32(c.w) + spill);
    return vec4<f32>(vec3<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(a))), f32(a));
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp4: vec4<f32>;
    if _uniform0.passindex < 0.5 {
      _skTemp4 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(1.0, 0.0), 0);
    } else {
      var _skTemp5: vec4<f32>;
      if _uniform0.passindex < 1.5 {
        _skTemp5 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(0.0, 1.0), 1);
      } else {
        _skTemp5 = effect__h4f2(_stageIn.sk_FragCoord.xy - layerAt__f2());
      }
      _skTemp4 = _skTemp5;
    }
    (*_stageOut).sk_FragColor = _skTemp4;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
