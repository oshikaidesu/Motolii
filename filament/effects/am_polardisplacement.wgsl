// Polar Displacement Map: AbsentMotion's polardisplacement (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Distort
// --picture@backdrop:Behind
// --about:Turns and pulls the layers behind this one around a center point by the layer's colors, then hides the layer.
// --track@angle:Angle,-3600,3600,0.0,0.5
// --track@reach:Inset/Outset,-1000,1000,0.0,0.1
// --track@angleChannel:Angle From,0,1,3.0,0.01
// --track@distanceChannel:Distance From,0,1,3.0,0.01
// --track@edges:Edges,0,1,1.0,0.01
// --track@R_center_x:Center x,-5000,5000,0,1
// --track@R_center_y:Center y,-5000,5000,0,1
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
  R_center: vec2<f32>,
  angle: f32,
  reach: f32,
  angleChannel: f32,
  distanceChannel: f32,
  edges: f32,
  fromCenter: f32,
  invert: f32,
  backdropBox: vec4<f32>,
};
@group(0) @binding(4) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
@group(0) @binding(3) var backdropPic_Sampler: sampler;
@group(0) @binding(2) var backdropPic_Texture: texture_2d<f32>;
fn pick_fh4f(c: vec4<f32>, ch: f32) -> f32 {
  {
    var _skTemp1: vec3<f32>;
    if c.w > 0.0 {
      _skTemp1 = vec3<f32>(c.xyz) / f32(c.w);
    } else {
      _skTemp1 = vec3<f32>(0.0);
    }
    let u: vec3<f32> = _skTemp1;
    if ch < 0.5 {
      return u.x;
    }
    if ch < 1.5 {
      return u.y;
    }
    if ch < 2.5 {
      return u.z;
    }
    if ch < 3.5 {
      return dot(u, vec3<f32>(0.2126, 0.7152, 0.0722));
    }
    return f32(c.w);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return vec4<f32>(0.0);
    }
    var ma: f32 = pick_fh4f(src, _uniform0.angleChannel);
    var mr: f32 = pick_fh4f(src, _uniform0.distanceChannel);
    if _uniform0.invert > 0.5 {
      {
        ma = 1.0 - ma;
        mr = 1.0 - mr;
      }
    }
    if _uniform0.fromCenter > 0.5 {
      {
        ma = ma * 2.0 - 1.0;
        mr = mr * 2.0 - 1.0;
      }
    }
    let c: vec2<f32> = _uniform0.resolution * 0.5 + _uniform0.R_center;
    let d: vec2<f32> = coord - c;
    let r: f32 = length(d) - _uniform0.reach * mr;
    var _skTemp2: f32;
    if dot(d, d) > 0.0 {
      _skTemp2 = atan2(d.y, d.x);
    } else {
      _skTemp2 = 0.0;
    }
    let a: f32 = _skTemp2 - radians(_uniform0.angle) * ma;
    var _0_q: vec2<f32> = ((c + r * vec2<f32>(cos(a), sin(a))) - _uniform0.screen.xy) / _uniform0.screen.zw;
    if _uniform0.edges < 0.5 {
      _0_q = clamp(_0_q, 0.5 / _uniform0.screen.zw, 1.0 - 0.5 / _uniform0.screen.zw);
    } else {
      if _uniform0.edges < 1.5 {
        _0_q = 1.0 - abs(1.0 - (_0_q - 2.0 * floor(_0_q / 2.0)));
      } else {
        if _uniform0.edges > 2.5 {
          _0_q = fract(_0_q);
        }
      }
    }
    return textureSample(backdropPic_Texture, backdropPic_Sampler, _uniform0.backdropBox.xy + ((_uniform0.screen.xy + _0_q * _uniform0.screen.zw) / _uniform0.resolution) * _uniform0.backdropBox.zw) * src.w;
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
