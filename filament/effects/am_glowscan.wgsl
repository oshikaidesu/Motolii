// Glow Scan: AbsentMotion's glowscan (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --pass@h:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --about:Blurs and lights up only the parts of the layer whose brightness falls in a band; animate Position to sweep the glow from shadows to highlights.
// --track@blur:Blur,0,300,20.0,0.1
// --track@R_position:Position,0,1,0.5,0.001
// --track@width:Width,0,1,0.2,0.001
// --track@softness:Softness,0,1,0.1,0.001
// --track@intensity:Intensity,0,5,1.0,0.001
// --track@color_x:Glow Color R,0,1,0.498,0.01
// --track@color_y:Glow Color G,0,1,0.8314,0.01
// --track@color_z:Glow Color B,0,1,1,0.01
// --track@color_w:Glow Color A,0,1,1,0.01
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
  R_position: f32,
  width: f32,
  softness: f32,
  intensity: f32,
  color: vec4<f32>,
  show: f32,
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
    let soft: vec4<f32> = textureSample(blurPicture_Texture, blurPicture_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _skTemp3: f32;
    if src.w > 0.0 {
      _skTemp3 = dot(vec3<f32>(src.xyz) / f32(src.w), vec3<f32>(0.2126, 0.7152, 0.0722));
    } else {
      _skTemp3 = 0.0;
    }
    let lum: f32 = _skTemp3;
    let half_band: f32 = max(_uniform0.width, 0.0) * 0.5;
    var m: f32 = 1.0 - smoothstep(half_band, half_band + max(_uniform0.softness, 0.001), abs(lum - _uniform0.R_position));
    m = m * f32(src.w);
    let glowRgb: vec3<f32> = ((vec3<f32>(soft.xyz) * vec3<f32>(_uniform0.color.xyz)) * max(_uniform0.intensity, 0.0)) * m;
    var _skTemp4: vec4<f32>;
    if _uniform0.show > 0.5 {
      _skTemp4 = src * f32(m);
    } else {
      _skTemp4 = src;
    }
    let base: vec4<f32> = _skTemp4;
    let mixed: vec4<f32> = vec4<f32>(mix(base.xyz, soft.xyz, vec3<f32>(f32(m))), base.w);
    return vec4<f32>(min(mixed.xyz + vec3<f32>(glowRgb), vec3<f32>(mixed.w)), mixed.w);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp5: vec4<f32>;
    if _uniform0.passindex < 0.5 {
      _skTemp5 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(1.0, 0.0), 0);
    } else {
      var _skTemp6: vec4<f32>;
      if _uniform0.passindex < 1.5 {
        _skTemp6 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(0.0, 1.0), 1);
      } else {
        _skTemp6 = effect__h4f2(_stageIn.sk_FragCoord.xy - layerAt__f2());
      }
      _skTemp5 = _skTemp6;
    }
    (*_stageOut).sk_FragColor = _skTemp5;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
