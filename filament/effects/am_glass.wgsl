// Glass: AbsentMotion's glass (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --pass@h_:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --picture@backdrop:Behind
// --about:Turns the layer's shape into beveled glass that bends the layers behind it, with a highlight along the rim.
// --track@blur:Bevel,0,300,24.0,0.1
// --track@refraction:Refraction,-300,300,30.0,0.1
// --track@angle:Light Angle,-3600,3600,135.0,0.5
// --track@highlight:Highlight,0,1,0.6,0.001
// --track@color_x:Color R,0,1,0.749,0.01
// --track@color_y:Color G,0,1,0.902,0.01
// --track@color_z:Color B,0,1,1,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@tint:Tint,0,1,0.15,0.001
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
  refraction: f32,
  angle: f32,
  highlight: f32,
  color: vec4<f32>,
  tint: f32,
  layerRect: vec4<f32>,
  backdropBox: vec4<f32>,
  passindex: f32,
};
@group(0) @binding(8) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var h__Sampler: sampler;
@group(0) @binding(2) var h__Texture: texture_2d<f32>;
@group(0) @binding(5) var blurPicture_Sampler: sampler;
@group(0) @binding(4) var blurPicture_Texture: texture_2d<f32>;
@group(0) @binding(7) var backdropPic_Sampler: sampler;
@group(0) @binding(6) var backdropPic_Texture: texture_2d<f32>;
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
        _skTemp1 = textureSample(h__Texture, h__Sampler, p / _uniform0.resolution);
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
            _skTemp2 = textureSample(h__Texture, h__Sampler, (p + by * x) / _uniform0.resolution);
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
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    let e: f32 = max(_uniform0.blur * 0.15, 1.0);
    let g: vec2<f32> = vec2<f32>(f32(textureSample(blurPicture_Texture, blurPicture_Sampler, ((coord + vec2<f32>(e, 0.0)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w) - f32(textureSample(blurPicture_Texture, blurPicture_Sampler, ((coord - vec2<f32>(e, 0.0)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w), f32(textureSample(blurPicture_Texture, blurPicture_Sampler, ((coord + vec2<f32>(0.0, e)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w) - f32(textureSample(blurPicture_Texture, blurPicture_Sampler, ((coord - vec2<f32>(0.0, e)) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w)) / (2.0 * e);
    let n: vec3<f32> = normalize(vec3<f32>(-g * max(_uniform0.blur, 1.0), 1.0));
    let bg: vec4<f32> = textureSample(backdropPic_Texture, backdropPic_Sampler, _uniform0.backdropBox.xy + (((coord - n.xy * _uniform0.refraction) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution) * _uniform0.backdropBox.zw);
    var _skTemp3: vec3<f32>;
    if bg.w > 0.0 {
      _skTemp3 = vec3<f32>(bg.xyz) / f32(bg.w);
    } else {
      _skTemp3 = vec3<f32>(0.0);
    }
    var rgb: vec3<f32> = _skTemp3;
    rgb = mix(rgb, rgb * vec3<f32>(_uniform0.color.xyz), vec3<f32>(clamp(_uniform0.tint, 0.0, 1.0) * f32(_uniform0.color.w)));
    let a: f32 = radians(_uniform0.angle);
    let l: vec3<f32> = normalize(vec3<f32>(cos(a), -sin(a), 0.6));
    let spec: f32 = pow(max(dot(reflect(-l, n), vec3<f32>(0.0, 0.0, 1.0)), 0.0), 30.0);
    let shine: f32 = clamp((spec + (1.0 - n.z) * 0.5) * _uniform0.highlight, 0.0, 1.0);
    let alpha: f32 = max(f32(bg.w), shine);
    rgb = clamp((rgb * f32(bg.w)) / max(alpha, 0.001) + shine, vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(vec3<f32>(rgb * alpha), f32(alpha)) * src.w;
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
