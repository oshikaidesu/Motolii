// Color Temperature: AbsentMotion's temperature (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --track@kelvin:Temperature,1000,40000,3200.0,100
// --track@strength:Strength,0,1,1.0,0.01
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
  kelvin: f32,
  strength: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn light_f3f(kelvin: f32) -> vec3<f32> {
  {
    let t: f32 = clamp(kelvin, 1000.0, 40000.0) * 0.01;
    var _skTemp1: f32;
    if t <= 66.0 {
      _skTemp1 = 255.0;
    } else {
      _skTemp1 = 329.69873 * pow(t - 60.0, -0.133204758);
    }
    let r: f32 = _skTemp1;
    var _skTemp2: f32;
    if t <= 66.0 {
      _skTemp2 = 99.4708 * log(t) - 161.119568;
    } else {
      _skTemp2 = 288.122162 * pow(t - 60.0, -0.0755148456);
    }
    let g: f32 = _skTemp2;
    var _skTemp3: f32;
    if t >= 66.0 {
      _skTemp3 = 255.0;
    } else {
      var _skTemp4: f32;
      if t <= 19.0 {
        _skTemp4 = 0.0;
      } else {
        _skTemp4 = 138.517731 * log(t - 10.0) - 305.0448;
      }
      _skTemp3 = _skTemp4;
    }
    let b: f32 = _skTemp3;
    return clamp(vec3<f32>(r, g, b) * 0.003921569, vec3<f32>(0.0), vec3<f32>(1.0));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    let base: vec3<f32> = vec3<f32>(c.xyz) / f32(c.w);
    var rgb: vec3<f32> = (base * light_f3f(_uniform0.kelvin)) / light_f3f(6500.0);
    rgb = mix(base, rgb, vec3<f32>(clamp(_uniform0.strength, 0.0, 1.0)));
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
