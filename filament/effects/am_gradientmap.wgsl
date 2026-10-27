// Gradient Map: AbsentMotion's gradientmap (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --track@color1_x:Shadows R,0,1,0.1059,0.01
// --track@color1_y:Shadows G,0,1,0.07843,0.01
// --track@color1_z:Shadows B,0,1,0.3922,0.01
// --track@color1_w:Shadows A,0,1,1,0.01
// --track@color2_x:Midtones R,0,1,0.8314,0.01
// --track@color2_y:Midtones G,0,1,0.07843,0.01
// --track@color2_z:Midtones B,0,1,0.3529,0.01
// --track@color2_w:Midtones A,0,1,1,0.01
// --track@color3_x:Highlights R,0,1,0.9843,0.01
// --track@color3_y:Highlights G,0,1,0.6902,0.01
// --track@color3_z:Highlights B,0,1,0.2314,0.01
// --track@color3_w:Highlights A,0,1,1,0.01
// --track@repeat:Repeat,0.1,10,1.0,0.1
// --track@phase:Phase,0,1,0.0,0.01
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
  color1: vec4<f32>,
  color2: vec4<f32>,
  color3: vec4<f32>,
  repeat: f32,
  phase: f32,
  reverse: f32,
  scaleBrightness: f32,
  opacity: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn luma_ff3(c: vec3<f32>) -> f32 {
  {
    return dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    let base: vec3<f32> = vec3<f32>(c.xyz) / f32(c.w);
    let w: f32 = dot(base, vec3<f32>(0.2126, 0.7152, 0.0722)) * _uniform0.repeat + _uniform0.phase;
    var u: f32 = 1.0 - abs(fract(w * 0.5) * 2.0 - 1.0);
    if _uniform0.reverse > 0.5 {
      u = 1.0 - u;
    }
    var _skTemp1: vec4<f32>;
    if u < 0.5 {
      _skTemp1 = mix(vec4<f32>(_uniform0.color1), vec4<f32>(_uniform0.color2), vec4<f32>(u * 2.0));
    } else {
      _skTemp1 = mix(vec4<f32>(_uniform0.color2), vec4<f32>(_uniform0.color3), vec4<f32>(u * 2.0 - 1.0));
    }
    let g: vec4<f32> = _skTemp1;
    var _skTemp2: vec3<f32>;
    if _uniform0.scaleBrightness > 0.5 {
      _skTemp2 = g.xyz * luma_ff3(base);
    } else {
      _skTemp2 = g.xyz;
    }
    let mapped: vec3<f32> = _skTemp2;
    let rgb: vec3<f32> = mix(base, mapped, vec3<f32>(clamp(_uniform0.opacity, 0.0, 1.0) * g.w));
    return vec4<f32>(vec3<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0))) * c.w, c.w);
  }
}
fn _skslMain(_stageIn: FSIn, _stageOut: ptr<function, FSOut>) {
  {
    (*_stageOut).sk_FragColor = effect__h4f2(_stageIn.sk_FragCoord.xy);
  }
}
@fragment fn main(_stageIn: FSIn) -> FSOut {
  var _stageOut: FSOut;
  _skslMain(_stageIn, &_stageOut);
  return _stageOut;
}
