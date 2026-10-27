// Tunnel: AbsentMotion's tunnel (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --track@phase:Phase,-1000,1000,0.0,0.01
// --track@speed:Speed,-20,20,1.0,0.01
// --track@twist:Twist,-360,360,0.0,0.1
// --track@repeat:Repeat,1,16,3.0,1
// --track@fog:Fog,0,1,0.4,0.01
// --track@fogColor_x:Fog Color R,0,1,0,0.01
// --track@fogColor_y:Fog Color G,0,1,0,0.01
// --track@fogColor_z:Fog Color B,0,1,0,0.01
// --track@fogColor_w:Fog Color A,0,1,1,0.01
// --track@R_position_x:Position x,-5000,5000,0,1
// --track@R_position_y:Position y,-5000,5000,0,1
// --track@scale:Scale,0.05,10,1.0,0.01
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
  rotX: f32,
  rotY: f32,
  rotZ: f32,
  phase: f32,
  speed: f32,
  twist: f32,
  repeat: f32,
  fog: f32,
  fogColor: vec4<f32>,
  R_position: vec2<f32>,
  scale: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let unit: f32 = max((min(_uniform0.resolution.x, _uniform0.resolution.y) * 0.5) * _uniform0.scale, 0.5);
    let _0_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _1_cx: f32 = cos(_0_a.x);
    let _2_sx: f32 = sin(_0_a.x);
    let _3_cy: f32 = cos(_0_a.y);
    let _4_sy: f32 = sin(_0_a.y);
    let _5_cz: f32 = cos(_0_a.z);
    let _6_sz: f32 = sin(_0_a.z);
    let rd: vec3<f32> = ((mat3x3<f32>(_5_cz, -_6_sz, 0.0, _6_sz, _5_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_3_cy, 0.0, _4_sy, 0.0, 1.0, 0.0, -_4_sy, 0.0, _3_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _1_cx, -_2_sx, 0.0, _2_sx, _1_cx)) * normalize(vec3<f32>(((coord - _uniform0.resolution * 0.5) - _uniform0.R_position) / unit, 1.4));
    let r: f32 = length(rd.xy);
    let haze: vec4<f32> = vec4<f32>(vec3<f32>(_uniform0.fogColor.xyz * _uniform0.fogColor.w), f32(_uniform0.fogColor.w));
    if r < 0.0001 {
      return vec4<f32>(haze);
    }
    let p: vec3<f32> = rd / r;
    let tile: f32 = 6.28318548 / max(floor(_uniform0.repeat + 0.5), 1.0);
    let along: f32 = (tile * _uniform0.resolution.y) / max(_uniform0.resolution.x, 0.0001);
    let z: f32 = p.z + _uniform0.time * _uniform0.speed;
    let u: f32 = fract((atan2(p.y, p.x) + radians(_uniform0.twist) * p.z) / tile);
    let v: f32 = fract(z / along + _uniform0.phase);
    let c: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, clamp(vec2<f32>(u, v) * _uniform0.resolution, vec2<f32>(0.5), max(_uniform0.resolution - 0.5, vec2<f32>(0.5))) / _uniform0.resolution));
    let k: f32 = 1.0 - exp((-abs(p.z) * _uniform0.fog) * 0.6);
    return vec4<f32>(mix(c, haze, vec4<f32>(k)));
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
