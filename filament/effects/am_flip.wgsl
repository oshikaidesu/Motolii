// Flip Layer: AbsentMotion's flip (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:3d
// --about:Turns the layer like a card in 3D, with its back see-through, a picture or a color.
// --track@rotX:Rotation X,-3600,3600,0.0,0.1
// --track@rotY:Rotation Y,-3600,3600,35.0,0.1
// --track@rotZ:Rotation Z,-3600,3600,0.0,0.1
// --track@back:Back,0,1,0.0,0.01
// --track@backColor_x:Back Color R,0,1,1,0.01
// --track@backColor_y:Back Color G,0,1,1,0.01
// --track@backColor_z:Back Color B,0,1,1,0.01
// --track@backColor_w:Back Color A,0,1,1,0.01
// --track@R_position_x:Position x,-5000,5000,0,1
// --track@R_position_y:Position y,-5000,5000,0,1
// --track@scale:Scale,0.05,10,1.0,0.01
// --track@R_perspective:Perspective,0,1,0.5,0.01
// --track@shading:Shading,0,1,0.6,0.01
// --track@light:Light Angle,-3600,3600,135.0,0.1
// --track@shine:Shine,0,1,0.3,0.01
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
  back: f32,
  backColor: vec4<f32>,
  R_position: vec2<f32>,
  scale: f32,
  R_perspective: f32,
  shading: f32,
  light: f32,
  shine: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn layerSize__f2() -> vec2<f32> {
  {
    return select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let side: f32 = min(layerSize__f2().x, layerSize__f2().y);
    let unit: f32 = max((side * 0.5) * _uniform0.scale, 0.5);
    let h: vec2<f32> = layerSize__f2() / side;
    let dist: f32 = max(2.0 + 40.0 * pow(1.0 - clamp(_uniform0.R_perspective, 0.0, 1.0), 3.0), length(h) + 0.5);
    let rw: vec3<f32> = normalize(vec3<f32>(((coord - layerSize__f2() * 0.5) - _uniform0.R_position) / unit, dist));
    let _1_a: vec3<f32> = radians(vec3<f32>(_uniform0.rotX, _uniform0.rotY, _uniform0.rotZ));
    let _2_cx: f32 = cos(_1_a.x);
    let _3_sx: f32 = sin(_1_a.x);
    let _4_cy: f32 = cos(_1_a.y);
    let _5_sy: f32 = sin(_1_a.y);
    let _6_cz: f32 = cos(_1_a.z);
    let _7_sz: f32 = sin(_1_a.z);
    let m: mat3x3<f32> = (mat3x3<f32>(_6_cz, -_7_sz, 0.0, _7_sz, _6_cz, 0.0, 0.0, 0.0, 1.0) * mat3x3<f32>(_4_cy, 0.0, _5_sy, 0.0, 1.0, 0.0, -_5_sy, 0.0, _4_cy)) * mat3x3<f32>(1.0, 0.0, 0.0, 0.0, _2_cx, -_3_sx, 0.0, _3_sx, _2_cx);
    let ro: vec3<f32> = vec3<f32>(0.0, 0.0, -dist) * m;
    let rd: vec3<f32> = rw * m;
    if abs(rd.z) < 1e-06 {
      return vec4<f32>(0.0);
    }
    let t: f32 = -ro.z / rd.z;
    if t <= 0.0 {
      return vec4<f32>(0.0);
    }
    let p: vec2<f32> = (ro + rd * t).xy;
    let px: f32 = (unit * dist) / (t * rw.z);
    let edge: f32 = clamp(min(h.x - abs(p.x), h.y - abs(p.y)) * px + 0.5, 0.0, 1.0);
    if edge <= 0.0 {
      return vec4<f32>(0.0);
    }
    let seen: bool = rd.z > 0.0;
    let uv: vec2<f32> = (p / h) * 0.5 + 0.5;
    let _8_p: vec2<f32> = clamp(uv * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
    var c: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, (_8_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
    if (!seen) && (_uniform0.back > 1.5) {
      c = vec4<f32>(vec3<f32>(_uniform0.backColor.xyz * _uniform0.backColor.w), f32(_uniform0.backColor.w));
    } else {
      if (!seen) && (_uniform0.back > 0.5) {
        {
          let _9_p: vec2<f32> = clamp(vec2<f32>(1.0 - uv.x, uv.y) * layerSize__f2(), vec2<f32>(0.5), max(layerSize__f2() - 0.5, vec2<f32>(0.5)));
          c = vec4<f32>(textureSample(image_Texture, image_Sampler, (_9_p + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution));
        }
      }
    }
    let _10_n: vec3<f32> = normalize(m * vec3<f32>(0.0, 0.0, f32(select(1.0, -1.0, seen))));
    let _11_a: f32 = radians(_uniform0.light);
    let _12_l: vec3<f32> = normalize(vec3<f32>(cos(_11_a), -sin(_11_a), -1.2));
    let _13_k: f32 = (1.0 - _uniform0.shading) + _uniform0.shading * (0.3 + 0.85 * max(dot(_10_n, _12_l), 0.0));
    let _14_s: f32 = _uniform0.shine * pow(max(dot(_10_n, normalize(_12_l + -rw)), 0.0), 40.0);
    return vec4<f32>(vec4<f32>(min(c.xyz * _13_k + _14_s * c.w, vec3<f32>(c.w)), c.w) * edge);
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
