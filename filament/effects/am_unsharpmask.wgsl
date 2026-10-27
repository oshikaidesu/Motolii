// Unsharp Mask: AbsentMotion's unsharpmask (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Stylize
// --track@radius:Strength,0.0,2.0,0.02,0.01
// --track@amount:Amount,0.0,2,0.5,0.1
// --track@threshold:Threshold,-0.5,1,0.5,0.01
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
  screen: vec4<f32>,
  time: f32,
  radius: f32,
  amount: f32,
  threshold: f32,
  showMask: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn softGate_hhh(d: f32, t: f32) -> f32 {
  {
    if t <= 0.0 {
      return 1.0;
    }
    return smoothstep(t, t + max(t * 0.5, 0.001), abs(d));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.001 {
      return vec4<f32>(0.0);
    }
    let col: vec3<f32> = c.xyz / c.w;
    let sigma: f32 = ((max(_uniform0.radius, 0.0) * 25.0) * min(_uniform0.screen.z, _uniform0.screen.w)) * 0.0009259259;
    var blurred: vec3<f32> = col;
    if sigma > 0.05 {
      {
        var sum: vec4<f32> = vec4<f32>(c);
        var total: f32 = 1.0;
        {
          var i: i32 = 0;
          for (; i < 64; i = i + i32(1)) {
            {
              let r: f32 = (sqrt((f32(i) + 0.5) * 0.015625) * 2.5) * sigma;
              let t: f32 = f32(i) * 2.39996314;
              let w: f32 = exp(((-0.5 * r) * r) / (sigma * sigma));
              sum = sum + w * vec4<f32>(textureSample(image_Texture, image_Sampler, (coord + r * vec2<f32>(cos(t), sin(t))) / _uniform0.resolution));
              total = total + w;
            }
          }
        }
        blurred = vec3<f32>(sum.xyz / total) / max(c.w, f32(sum.w / total));
      }
    }
    let diff: vec3<f32> = col - blurred;
    let t: f32 = f32(max(_uniform0.threshold - 0.5, 0.0) * 0.25);
    let g: vec3<f32> = vec3<f32>(softGate_hhh(diff.x, t), softGate_hhh(diff.y, t), softGate_hhh(diff.z, t));
    let corrected: vec3<f32> = clamp(col + (f32(_uniform0.amount) * diff) * g, vec3<f32>(0.0), vec3<f32>(1.0));
    let ad: vec3<f32> = abs(diff);
    let m: f32 = clamp(((ad.x + ad.y) + ad.z) * 1.33333325, 0.0, 1.0);
    if _uniform0.showMask > 0.5 {
      return vec4<f32>(vec3<f32>(m) * c.w, c.w);
    }
    return vec4<f32>(corrected * c.w, c.w);
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
