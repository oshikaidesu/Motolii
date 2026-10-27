// Bump Map: AbsentMotion's bumpmap (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --track@height:Height,0,30,4.0,0.01
// --track@ambient:Ambient,0,1,0.4,0.001
// --track@shine:Shine,0,1,0.3,0.001
// --track@light_x:Light Color R,0,1,1,0.01
// --track@light_y:Light Color G,0,1,1,0.01
// --track@light_z:Light Color B,0,1,1,0.01
// --track@light_w:Light Color A,0,1,1,0.01
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
  angle: f32,
  height: f32,
  ambient: f32,
  shine: f32,
  light: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn lum_ff2(at: vec2<f32>) -> f32 {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, at / _uniform0.resolution);
    var _skTemp1: f32;
    if c.w > 0.0 {
      _skTemp1 = dot(vec3<f32>(c.xyz), vec3<f32>(0.2126, 0.7152, 0.0722));
    } else {
      _skTemp1 = 0.0;
    }
    return _skTemp1;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let dx: f32 = lum_ff2(coord + vec2<f32>(1.0, 0.0)) - lum_ff2(coord - vec2<f32>(1.0, 0.0));
    let dy: f32 = lum_ff2(coord + vec2<f32>(0.0, 1.0)) - lum_ff2(coord - vec2<f32>(0.0, 1.0));
    let n: vec3<f32> = normalize(vec3<f32>(-dx * _uniform0.height, -dy * _uniform0.height, 1.0));
    let a: f32 = radians(_uniform0.angle);
    let l: vec3<f32> = normalize(vec3<f32>(cos(a), -sin(a), 1.0));
    let diffuse: f32 = max(dot(n, l), 0.0);
    let spec: f32 = pow(max(dot(reflect(-l, n), vec3<f32>(0.0, 0.0, 1.0)), 0.0), 24.0) * _uniform0.shine;
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let lit: vec3<f32> = (base * vec3<f32>(_uniform0.light.xyz)) * (_uniform0.ambient + ((1.0 - _uniform0.ambient) * diffuse) / l.z) + vec3<f32>(_uniform0.light.xyz) * spec;
    return vec4<f32>(vec3<f32>(clamp(lit, vec3<f32>(0.0), vec3<f32>(1.0))) * src.w, src.w);
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
