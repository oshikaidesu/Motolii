// Spin Streaks: AbsentMotion's spinstreaks (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --track@R_center_x:Center x,-1,2,0.5,0.01
// --track@R_center_y:Center y,-1,2,0.5,0.01
// --track@alpha:Alpha,0,1,1.0,0.001
// --track@bias:Bias,-1,1,0.0,0.01
// --track@r:R,0,1,1.0,0.01
// --track@g:G,0,1,1.0,0.01
// --track@b:B,0,1,1.0,0.01
// --track@a:A,0,1,1.0,0.01
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
  sweep: f32,
  alpha: f32,
  bias: f32,
  r: f32,
  g: f32,
  b: f32,
  a: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn pick_ffffff(m: f32, lo: f32, hi: f32, avg: f32, src: f32) -> f32 {
  {
    var _skTemp1: f32;
    if m < 0.5 {
      _skTemp1 = lo;
    } else {
      var _skTemp2: f32;
      if m < 1.5 {
        _skTemp2 = hi;
      } else {
        _skTemp2 = select(src, avg, m < 2.5);
      }
      _skTemp1 = _skTemp2;
    }
    return _skTemp1;
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    let c: vec2<f32> = _uniform0.R_center * _uniform0.resolution;
    let span: f32 = radians(_uniform0.sweep);
    if span <= 0.0 {
      return src;
    }
    let d: vec2<f32> = coord - c;
    var hi: vec4<f32> = vec4<f32>(0.0);
    var sum: vec4<f32> = vec4<f32>(0.0);
    var lo: vec4<f32> = vec4<f32>(1.0);
    {
      var i: i32 = 0;
      for (; i < 48; i = i + i32(1)) {
        {
          let t: f32 = (span * 0.5) * mix(_uniform0.bias - 1.0, _uniform0.bias + 1.0, f32(i) * 0.0212765951);
          let cs: f32 = cos(t);
          let sn: f32 = sin(t);
          let v: vec4<f32> = vec4<f32>(textureSample(image_Texture, image_Sampler, (c + vec2<f32>(cs * d.x - sn * d.y, sn * d.x + cs * d.y)) / _uniform0.resolution));
          hi = max(hi, v);
          sum = sum + v;
          lo = min(lo, v);
        }
      }
    }
    let avg: vec4<f32> = sum * 0.020833334;
    let s: vec4<f32> = vec4<f32>(src);
    let acc: vec4<f32> = vec4<f32>(pick_ffffff(_uniform0.r, lo.x, hi.x, avg.x, s.x), pick_ffffff(_uniform0.g, lo.y, hi.y, avg.y, s.y), pick_ffffff(_uniform0.b, lo.z, hi.z, avg.z, s.z), pick_ffffff(_uniform0.a, lo.w, hi.w, avg.w, s.w));
    return vec4<f32>(mix(s, acc, vec4<f32>(clamp(_uniform0.alpha, 0.0, 1.0))));
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
