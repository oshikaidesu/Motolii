// Contour Gradient: AbsentMotion's contourgradient (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --pass@h:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --about:Paints a two-color gradient outwards from the edges of the opaque parts of the layer, like a soft colored halo.
// --track@color1_x:Color 1 R,0,1,1,0.01
// --track@color1_y:Color 1 G,0,1,0.2353,0.01
// --track@color1_z:Color 1 B,0,1,0.4314,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,0.2353,0.01
// --track@color2_y:Color 2 G,0,1,0.7843,0.01
// --track@color2_z:Color 2 B,0,1,1,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
// --track@blur:Width,1,300,25.0,0.1
// --track@alpha:Alpha,0,1,1.0,0.001
// --track@fill:Fill,0,1,1.0,0.001
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
  color1: vec4<f32>,
  color2: vec4<f32>,
  blur: f32,
  alpha: f32,
  fill: f32,
  fade: f32,
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
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let b: f32 = f32(textureSample(blurPicture_Texture, blurPicture_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w);
    var t: f32 = clamp(1.0 - b * 2.0, 0.0, 1.0);
    t = t * t;
    var cover: f32 = smoothstep(0.0, 0.06, b) * clamp(_uniform0.alpha, 0.0, 1.0);
    if _uniform0.fade > 0.5 {
      cover = cover * (1.0 - t);
    }
    let rgb: vec3<f32> = mix(vec3<f32>(_uniform0.color1.xyz), vec3<f32>(_uniform0.color2.xyz), vec3<f32>(t));
    let ca: f32 = cover * mix(f32(_uniform0.color1.w), f32(_uniform0.color2.w), t);
    let halo: vec4<f32> = vec4<f32>(vec3<f32>(rgb) * f32(ca), f32(ca));
    let body: vec4<f32> = src * f32(clamp(_uniform0.fill, 0.0, 1.0));
    return body + halo * (1.0 - body.w);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp3: vec4<f32>;
    if _uniform0.passindex < 0.5 {
      _skTemp3 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(1.0, 0.0), 0);
    } else {
      var _skTemp4: vec4<f32>;
      if _uniform0.passindex < 1.5 {
        _skTemp4 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(0.0, 1.0), 1);
      } else {
        _skTemp4 = effect__h4f2(_stageIn.sk_FragCoord.xy - layerAt__f2());
      }
      _skTemp3 = _skTemp4;
    }
    (*_stageOut).sk_FragColor = _skTemp3;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
