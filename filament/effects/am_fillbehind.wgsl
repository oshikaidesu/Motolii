// Fill Behind: AbsentMotion's fillbehind (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --pass@h:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --about:Fills the frame behind the layer with a blurred, zoomed copy of itself, the usual fix for a portrait video in a landscape project.
// --track@blur:Strength,0,600,60.0,0.1
// --track@feather:Feather,0,1,0.05,0.001
// --track@saturation:Saturation,-1,1,0.0,0.001
// --track@brightness:Brightness,-1,1,0.0,0.001
// --track@gamma:Gamma,0.01,9.99,1.0,0.01
// --track@fgAlpha:Foreground Alpha,0,1,1.0,0.001
// --track@bgAlpha:Background Alpha,0,1,1.0,0.001
// --track@tintColor_x:Tint Color R,0,1,0.2392,0.01
// --track@tintColor_y:Tint Color G,0,1,0.4941,0.01
// --track@tintColor_z:Tint Color B,0,1,1,0.01
// --track@tintColor_w:Tint Color A,0,1,1,0.01
// --track@tint:Tint,0,1,0.0,0.001
// --track@edgeColor_x:Edge Color R,0,1,0,0.01
// --track@edgeColor_y:Edge Color G,0,1,0,0.01
// --track@edgeColor_z:Edge Color B,0,1,0,0.01
// --track@edgeColor_w:Edge Color A,0,1,1,0.01
// --track@edgeStrength:Edge Strength,0,1,0.0,0.001
// --track@scanStrength:Scan Line Strength,0,1,0.0,0.001
// --track@scanCount:Scan Line Count,100,2160,720.0,1
// --track@fgOffset_x:FG Offset x,-5000,5000,0,1
// --track@fgOffset_y:FG Offset y,-5000,5000,0,1
// --track@fgScale:FG Scale,0.01,100,1.0,0.01
// --track@bgOffset_x:BG Offset x,-5000,5000,0,1
// --track@bgOffset_y:BG Offset y,-5000,5000,0,1
// --track@bgScale:BG Scale,0.01,100,1.0,0.01
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
  blur: f32,
  feather: f32,
  saturation: f32,
  brightness: f32,
  gamma: f32,
  fgAlpha: f32,
  bgAlpha: f32,
  tintColor: vec4<f32>,
  tint: f32,
  edgeColor: vec4<f32>,
  edgeStrength: f32,
  scanStrength: f32,
  scanCount: f32,
  fgOffset: vec2<f32>,
  fgScale: f32,
  bgOffset: vec2<f32>,
  bgScale: f32,
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
    let c: vec2<f32> = (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) * 0.5;
    let margin: f32 = max(_uniform0.blur, 0.0) * 0.5;
    let inner: vec2<f32> = max((select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0))) - 2.0 * margin, vec2<f32>(1.0));
    let k: f32 = max(_uniform0.screen.z / inner.x, _uniform0.screen.w / inner.y) * max(_uniform0.bgScale, 0.01);
    let b: vec4<f32> = textureSample(blurPicture_Texture, blurPicture_Sampler, ((c + (((coord - _uniform0.screen.xy) - _uniform0.screen.zw * 0.5) - _uniform0.bgOffset) / k) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _skTemp3: vec3<f32>;
    if b.w > 0.0 {
      _skTemp3 = vec3<f32>(b.xyz) / f32(b.w);
    } else {
      _skTemp3 = vec3<f32>(0.0);
    }
    var rgb: vec3<f32> = _skTemp3;
    rgb = clamp(mix(vec3<f32>(dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722))), rgb, vec3<f32>(1.0 + _uniform0.saturation)), vec3<f32>(0.0), vec3<f32>(1.0));
    let l: f32 = dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    if l > 0.0 {
      rgb = rgb * (pow(l, 1.0 / max(_uniform0.gamma, 0.01)) / l);
    }
    var _skTemp4: vec3<f32>;
    if _uniform0.brightness < 0.0 {
      _skTemp4 = rgb * (1.0 + _uniform0.brightness);
    } else {
      _skTemp4 = mix(rgb, vec3<f32>(1.0), vec3<f32>(_uniform0.brightness));
    }
    rgb = _skTemp4;
    let hue: vec3<f32> = (vec3<f32>(_uniform0.tintColor.xyz) * dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722))) / max(dot(vec3<f32>(_uniform0.tintColor.xyz), vec3<f32>(0.2126, 0.7152, 0.0722)), 0.001);
    rgb = mix(rgb, hue, vec3<f32>(clamp(_uniform0.tint, 0.0, 1.0)));
    if (_uniform0.scanStrength > 0.0) && (fract(((coord.y - _uniform0.screen.y) / _uniform0.screen.w) * _uniform0.scanCount) > 0.5) {
      rgb = rgb * (1.0 - clamp(_uniform0.scanStrength, 0.0, 1.0));
    }
    let bg: vec4<f32> = vec4<f32>(vec3<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0))), 1.0) * f32(f32(b.w) * clamp(_uniform0.bgAlpha, 0.0, 1.0));
    let fp: vec2<f32> = c + ((coord - c) - _uniform0.fgOffset) / max(_uniform0.fgScale, 0.01);
    var fg: vec4<f32> = textureSample(image_Texture, image_Sampler, (fp + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let soft: f32 = f32(textureSample(blurPicture_Texture, blurPicture_Sampler, (fp + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution).w);
    let spread: f32 = f32(_uniform0.blur > 0.0);
    fg = fg * f32(mix(1.0, smoothstep(0.5, 1.0, soft), clamp(_uniform0.feather, 0.0, 1.0) * spread) * clamp(_uniform0.fgAlpha, 0.0, 1.0));
    let e: f32 = ((clamp(soft * 2.0, 0.0, 1.0) * clamp(_uniform0.edgeStrength, 0.0, 1.0)) * f32(_uniform0.edgeColor.w)) * spread;
    let under: vec4<f32> = vec4<f32>(_uniform0.edgeColor.xyz * f32(e), f32(e)) + bg * f32(1.0 - e);
    return fg + under * (1.0 - fg.w);
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
