// RGB Split: AbsentMotion's rgbsplit (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Splits the red, green and blue channels: the Center channel stays put and the other two shift in opposite directions along Angle. Light adds the shifted colors, Dark keeps them solid, Mask keeps only where all three overlap and Luma makes the fringes as opaque as they are bright.
// --track@strength:Strength,-8,8,0.15,0.01
// --track@angle:Angle,-3600,3600,0.0,1
// --track@R_center:Center,0,1,1.0,0.01
// --track@mode:Mode,0,1,0.0,0.01
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
  angle: f32,
  R_center: f32,
  mode: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let a: f32 = radians(_uniform0.angle);
    let o: vec2<f32> = (vec2<f32>(cos(a), sin(a)) * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)))) * (_uniform0.strength * 0.125);
    let k: f32 = floor(_uniform0.R_center + 0.5);
    let sr: vec2<f32> = select(o, vec2<f32>(0.0), vec2<bool>(k < 0.5));
    var _skTemp1: vec2<f32>;
    if k < 0.5 {
      _skTemp1 = o;
    } else {
      _skTemp1 = select(-o, vec2<f32>(0.0), vec2<bool>(k < 1.5));
    }
    let sg: vec2<f32> = _skTemp1;
    let sb: vec2<f32> = select(vec2<f32>(0.0), -o, vec2<bool>(k < 1.5));
    let r: vec4<f32> = textureSample(image_Texture, image_Sampler, ((coord - sr) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let g: vec4<f32> = textureSample(image_Texture, image_Sampler, ((coord - sg) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    let b: vec4<f32> = textureSample(image_Texture, image_Sampler, ((coord - sb) + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    var _skTemp2: f32;
    if k < 0.5 {
      _skTemp2 = r.w;
    } else {
      _skTemp2 = select(b.w, g.w, k < 1.5);
    }
    let ac: f32 = _skTemp2;
    let rgb: vec3<f32> = vec3<f32>(r.x, g.y, b.z);
    let m: f32 = floor(_uniform0.mode + 0.5);
    if m < 0.5 {
      return vec4<f32>(rgb, ac);
    }
    if m < 1.5 {
      return vec4<f32>(rgb, max(r.w, max(g.w, b.w)));
    }
    if m < 2.5 {
      {
        let lo: f32 = min(r.w, min(g.w, b.w));
        var _skTemp3: f32;
        if r.w > 0.001 {
          _skTemp3 = r.x / r.w;
        } else {
          _skTemp3 = 0.0;
        }
        var _skTemp4: f32;
        if g.w > 0.001 {
          _skTemp4 = g.y / g.w;
        } else {
          _skTemp4 = 0.0;
        }
        var _skTemp5: f32;
        if b.w > 0.001 {
          _skTemp5 = b.z / b.w;
        } else {
          _skTemp5 = 0.0;
        }
        let un: vec3<f32> = vec3<f32>(_skTemp3, _skTemp4, _skTemp5);
        return vec4<f32>(un * lo, lo);
      }
    }
    return vec4<f32>(rgb, clamp(max(ac, dot(rgb, vec3<f32>(0.2126, 0.7152, 0.0722))), 0.0, 1.0));
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
