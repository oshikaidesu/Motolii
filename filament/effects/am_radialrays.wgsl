// Radial Rays: AbsentMotion's radialrays (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Paints solid sunburst rays from a point over the opaque parts of the layer.
// --track@R_center_x:Center x,-1,2,0.5,0.01
// --track@R_center_y:Center y,-1,2,0.5,0.01
// --track@count:Count,1,200,5.0,1
// --track@width:Width,0,1,0.5,0.001
// --track@angle:Angle,-3600,3600,0.0,0.5
// --track@color_x:Color R,0,1,0.1765,0.01
// --track@color_y:Color G,0,1,0.1176,0.01
// --track@color_z:Color B,0,1,0.9647,0.01
// --track@color_w:Color A,0,1,1,0.01
// --track@alpha:Alpha,0,1,1.0,0.001
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
  R_center: vec2<f32>,
  count: f32,
  width: f32,
  angle: f32,
  color: vec4<f32>,
  alpha: f32,
  layerRect: vec4<f32>,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + (select(vec2<f32>(0.0), _uniform0.layerRect.xy, vec2<bool>(_uniform0.layerRect.z > 0.0)))) / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let d: vec2<f32> = coord - _uniform0.R_center * (select(_uniform0.resolution, _uniform0.layerRect.zw, vec2<bool>(_uniform0.layerRect.z > 0.0)));
    let n: f32 = max(floor(_uniform0.count + 0.5), 1.0);
    var _skTemp1: f32;
    if dot(d, d) > 0.0 {
      _skTemp1 = atan2(d.y, d.x);
    } else {
      _skTemp1 = 0.0;
    }
    let direction: f32 = _skTemp1;
    let t: f32 = ((direction - radians(_uniform0.angle)) * 0.159154937) * n;
    let e: f32 = abs(fract(t) - 0.5);
    let px: f32 = (max(length(d), 1.0) * 6.28318548) / n;
    var _skTemp2: f32;
    if _uniform0.width > 0.0 {
      _skTemp2 = clamp((_uniform0.width * 0.5 - e) * px + 0.5, 0.0, 1.0);
    } else {
      _skTemp2 = 0.0;
    }
    let s: f32 = _skTemp2;
    let base: vec3<f32> = vec3<f32>(src.xyz) / f32(src.w);
    let rgb: vec3<f32> = mix(base, vec3<f32>(_uniform0.color.xyz), vec3<f32>((s * f32(_uniform0.color.w)) * clamp(_uniform0.alpha, 0.0, 1.0)));
    return vec4<f32>(vec3<f32>(rgb) * src.w, src.w);
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
