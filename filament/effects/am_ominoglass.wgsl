// Omino Glass: AbsentMotion's ominoglass (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --pass@h_:0,1,s1,s1
// --pass@blurPicture:0,1,s1,s1
// --pass@:0,0,s1,s1
// --picture@backdrop:Behind
// --about:Glass made from the layer's brightness: refracts what is behind it with rainbow fringes and lights the surface like a lens.
// --track@blur:Bevel,0,300,10.0,0.1
// --track@refraction:Refraction,-100,100,5.0,0.1
// --track@aberration:Chromatic Aberration,-100,100,5.0,0.1
// --track@feature:Feature Size,5,500,100.0,0.1
// --track@samples:Smoothing Samples,1,20,2.0,1
// --track@color_x:Color R,0,1,1,0.01
// --track@color_y:Color G,0,1,0.3333,0.01
// --track@color_z:Color B,0,1,0.4,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@blend:Blend,0,1,0.0,0.001
// --track@shading:Shading,0,1,1.0,0.01
// --track@intensity:Intensity,0,2,1.0,0.001
// --track@shine:Shine,0,1,0.8,0.001
// --track@specularity:Specularity,1,250,30.0,0.1
// --track@ambient:Ambient,0,1,0.1,0.001
// --track@light_x:Light Color R,0,1,1,0.01
// --track@light_y:Light Color G,0,1,1,0.01
// --track@light_z:Light Color B,0,1,1,0.01
// --track@light_w:Light Color A,0,1,1,0.01
// --track@direction_x:Direction x,-5000,5000,75,1
// --track@direction_y:Direction y,-5000,5000,-75,1
// --track@elevation:Elevation,-100,800,100.0,0.1
// --track@amount:Roughness,-1,1,0.0,0.001
// --track@scale:Roughness Scale,0.01,50,1.0,0.01
// --track@evolution:Evolution,-50,50,0.0,0.01
// --track@seed:Seed,0,5,0.0,0.01
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
  aberration: f32,
  feature: f32,
  samples: f32,
  color: vec4<f32>,
  blend: f32,
  shading: f32,
  intensity: f32,
  shine: f32,
  specularity: f32,
  ambient: f32,
  light: vec4<f32>,
  direction: vec2<f32>,
  elevation: f32,
  useLuma: f32,
  amount: f32,
  scale: f32,
  evolution: f32,
  seed: f32,
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
fn layerSize__f2() -> vec2<f32> {
  {
    return select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
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
fn hash13_ff3(_skParam0: vec3<f32>) -> f32 {
  var p = _skParam0;
  {
    p = fract(p * 0.1031);
    p = p + dot(p, p.zyx + 31.32);
    return fract((p.x + p.y) * p.z);
  }
}
fn vnoise_ff3(p: vec3<f32>) -> f32 {
  {
    let i: vec3<f32> = floor(p);
    var f: vec3<f32> = fract(p);
    f = (f * f) * (3.0 - 2.0 * f);
    let a: f32 = mix(hash13_ff3(i), hash13_ff3(i + vec3<f32>(1.0, 0.0, 0.0)), f.x);
    let b: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 1.0, 0.0)), hash13_ff3(i + vec3<f32>(1.0, 1.0, 0.0)), f.x);
    let c: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 0.0, 1.0)), hash13_ff3(i + vec3<f32>(1.0, 0.0, 1.0)), f.x);
    let d: f32 = mix(hash13_ff3(i + vec3<f32>(0.0, 1.0, 1.0)), hash13_ff3(i + vec3<f32>(1.0)), f.x);
    return mix(mix(a, b, f.y), mix(c, d, f.y), f.z);
  }
}
fn band_f3i(i: i32) -> vec3<f32> {
  {
    if i == 0 {
      return vec3<f32>(0.5, 0.0, 1.0);
    }
    if i == 1 {
      return vec3<f32>(0.0, 0.0, 1.0);
    }
    if i == 2 {
      return vec3<f32>(0.0, 0.6, 1.0);
    }
    if i == 3 {
      return vec3<f32>(0.0, 1.0, 0.0);
    }
    if i == 4 {
      return vec3<f32>(1.0, 1.0, 0.0);
    }
    if i == 5 {
      return vec3<f32>(1.0, 0.5, 0.0);
    }
    return vec3<f32>(1.0, 0.0, 0.0);
  }
}
fn rough_ff2(p: vec2<f32>) -> f32 {
  {
    let q: vec3<f32> = vec3<f32>(p / (24.0 * max(_uniform0.scale, 0.01)) + _uniform0.seed * vec2<f32>(37.1, 17.3), _uniform0.evolution);
    return (vnoise_ff3(q) * 0.6 + vnoise_ff3(q * 2.03 + 5.1) * 0.3) + vnoise_ff3(q * 4.1 + 9.7) * 0.1;
  }
}
fn height_ff2(p: vec2<f32>) -> f32 {
  {
    let b: vec4<f32> = textureSample(blurPicture_Texture, blurPicture_Sampler, (p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _skTemp3: f32;
    if _uniform0.useLuma > 0.5 {
      _skTemp3 = dot(vec3<f32>(b.xyz), vec3<f32>(0.2126, 0.7152, 0.0722));
    } else {
      _skTemp3 = f32(b.w);
    }
    let h: f32 = _skTemp3;
    var _skTemp4: f32;
    if _uniform0.amount != 0.0 {
      _skTemp4 = h + (_uniform0.amount * (rough_ff2(p) - 0.5)) * f32(b.w);
    } else {
      _skTemp4 = h;
    }
    return _skTemp4;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    let steps: f32 = clamp(floor(_uniform0.samples), 1.0, 20.0);
    var g: vec2<f32> = vec2<f32>(0.0);
    {
      var i: i32 = 1;
      for (; i <= 20; i = i + i32(1)) {
        {
          if f32(i) > steps {
            break;
          }
          let d: f32 = f32(i);
          g = g + vec2<f32>(height_ff2(coord + vec2<f32>(d, 0.0)) - height_ff2(coord - vec2<f32>(d, 0.0)), height_ff2(coord + vec2<f32>(0.0, d)) - height_ff2(coord - vec2<f32>(0.0, d))) / (2.0 * d);
        }
      }
    }
    let n: vec3<f32> = normalize(vec3<f32>(((-g / steps) * _uniform0.feature) * 0.1, 1.0));
    var acc: vec3<f32> = vec3<f32>(0.0);
    var alpha: f32 = 0.0;
    {
      var i: i32 = 0;
      for (; i < 7; i = i + i32(1)) {
        {
          let s: vec4<f32> = backdropAt_h4f2(coord - n.xy * (_uniform0.refraction + _uniform0.aberration * (f32(i) * 0.166666672 - 0.5)));
          acc = acc + vec3<f32>(s.xyz) * band_f3i(i);
          alpha = alpha + f32(s.w) * 0.142857149;
        }
      }
    }
    var rgb: vec3<f32> = acc * vec3<f32>(0.2857143, 0.322580665, 0.333333343);
    rgb = mix(rgb, rgb * vec3<f32>(_uniform0.color.xyz), vec3<f32>(clamp(_uniform0.blend, 0.0, 1.0)));
    if _uniform0.shading > 0.5 {
      {
        var _skTemp5: vec3<f32>;
        if _uniform0.shading < 1.5 {
          _skTemp5 = vec3<f32>((layerSize__f2() * 0.5 + _uniform0.direction) - coord, _uniform0.elevation);
        } else {
          _skTemp5 = vec3<f32>(_uniform0.direction, _uniform0.elevation);
        }
        let toLight: vec3<f32> = _skTemp5;
        var _skTemp6: vec3<f32>;
        if length(toLight) > 0.0001 {
          _skTemp6 = normalize(toLight);
        } else {
          _skTemp6 = vec3<f32>(0.0, 0.0, 1.0);
        }
        let l: vec3<f32> = _skTemp6;
        let level: f32 = max(l.z, 0.05);
        let diffuse: f32 = min(max(dot(n, l), 0.0) / level, 2.0);
        let spec: f32 = pow(max(dot(reflect(-l, n), vec3<f32>(0.0, 0.0, 1.0)), 0.0), max(_uniform0.specularity, 1.0)) * _uniform0.shine;
        let lc: vec3<f32> = vec3<f32>(_uniform0.light.xyz) * _uniform0.intensity;
        rgb = rgb * (_uniform0.ambient + ((1.0 - _uniform0.ambient) * diffuse) * lc) + (spec * lc) * max(alpha, 0.5);
        alpha = max(alpha, clamp(spec * _uniform0.intensity, 0.0, 1.0));
      }
    }
    return vec4<f32>(vec3<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(alpha))), f32(alpha)) * src.w;
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    var _skTemp7: vec4<f32>;
    if _uniform0.passindex < 0.5 {
      _skTemp7 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(1.0, 0.0), 0);
    } else {
      var _skTemp8: vec4<f32>;
      if _uniform0.passindex < 1.5 {
        _skTemp8 = gauss__h4f2f2i(_stageIn.sk_FragCoord.xy, vec2<f32>(0.0, 1.0), 1);
      } else {
        _skTemp8 = effect__h4f2(_stageIn.sk_FragCoord.xy - layerAt__f2());
      }
      _skTemp7 = _skTemp8;
    }
    (*_stageOut).sk_FragColor = _skTemp7;
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
