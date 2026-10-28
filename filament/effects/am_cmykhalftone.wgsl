// CMYK Halftone Dots: AbsentMotion's cmykhalftone (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --about:Redraws the layer like a magazine print: cyan, magenta, yellow and black dots on four rotated screens.
// --track@amount:Amount,0,500,100.0,0.1
// --track@strength:Strength,0,1,0.5,0.001
// --track@black:Black,0,1,0.0,0.001
// --track@angle:Angle,0,3600,45.0,0.5
// --track@phase:Phase,-1000,1000,0.5,0.001
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
  amount: f32,
  strength: f32,
  black: f32,
  angle: f32,
  phase: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn screenDot_ff2fi(coord: vec2<f32>, turn: f32, channel: i32) -> f32 {
  {
    let cell: f32 = _uniform0.resolution.x / max(_uniform0.amount, 0.5);
    let a: f32 = radians(_uniform0.angle + turn);
    let cs: f32 = cos(a);
    let sn: f32 = sin(a);
    let o: vec2<f32> = _uniform0.resolution * 0.5;
    var p: vec2<f32> = coord - o;
    p = vec2<f32>(cs * p.x + sn * p.y, -sn * p.x + cs * p.y) / cell;
    let row: f32 = floor(p.y);
    p.x = p.x - _uniform0.phase * row;
    let m: vec2<f32> = vec2<f32>(floor(p.x) + 0.5, row + 0.5);
    let back: vec2<f32> = vec2<f32>(m.x + _uniform0.phase * row, m.y) * cell;
    let _0_c: vec4<f32> = textureSample(image_Texture, image_Sampler, (o + vec2<f32>(cs * back.x - sn * back.y, sn * back.x + cs * back.y)) / _uniform0.resolution);
    var _skTemp1: vec3<f32>;
    if _0_c.w > 0.0 {
      _skTemp1 = vec3<f32>(_0_c.xyz) / f32(_0_c.w);
    } else {
      _skTemp1 = vec3<f32>(1.0);
    }
    let _1_rgb: vec3<f32> = _skTemp1;
    let _2_cmy: vec3<f32> = 1.0 - _1_rgb;
    let _3_k: f32 = _uniform0.black * min(_2_cmy.x, min(_2_cmy.y, _2_cmy.z));
    let ink: vec4<f32> = vec4<f32>((_2_cmy - _3_k) / max(1.0 - _3_k, 0.0001), _3_k);
    var _skTemp2: f32;
    if channel == 0 {
      _skTemp2 = ink.x;
    } else {
      var _skTemp3: f32;
      if channel == 1 {
        _skTemp3 = ink.y;
      } else {
        _skTemp3 = select(ink.w, ink.z, channel == 2);
      }
      _skTemp2 = _skTemp3;
    }
    let v: f32 = _skTemp2;
    let s: f32 = clamp(_uniform0.strength, 0.001, 0.999);
    let r: f32 = (0.7071 * sqrt(pow(clamp(v, 0.0, 1.0), (1.0 - s) / s))) * cell;
    return clamp((r - length(p - m) * cell) + 0.5, 0.0, 1.0);
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let src: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if src.w <= 0.0 {
      return src;
    }
    let c: f32 = screenDot_ff2fi(coord, -30.0, 0);
    let m: f32 = screenDot_ff2fi(coord, 30.0, 1);
    let y: f32 = screenDot_ff2fi(coord, -45.0, 2);
    var _skTemp4: f32;
    if _uniform0.black > 0.0 {
      _skTemp4 = screenDot_ff2fi(coord, 0.0, 3);
    } else {
      _skTemp4 = 0.0;
    }
    let k: f32 = _skTemp4;
    let rgb: vec3<f32> = (1.0 - vec3<f32>(c, m, y)) * (1.0 - k);
    return vec4<f32>(vec3<f32>(rgb) * src.w, src.w);
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
