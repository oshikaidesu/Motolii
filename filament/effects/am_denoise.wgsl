// Denoise: AbsentMotion's denoise (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Blur
// --about:Removes image noise with an edge-preserving bilateral filter that averages nearby pixels only when their colors are similar.
// --track@strength:Strength,0.0,1.0,0.5,0.01
// --track@radius:Radius,1.0,6.0,2.0,0.5
// --track@detail:Detail,0.0,1.0,0.5,0.01
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
  strength: f32,
  radius: f32,
  detail: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.001 {
      return vec4<f32>(0.0);
    }
    let col: vec3<f32> = c.xyz / c.w;
    let s: f32 = max(_uniform0.radius, 0.5);
    let rs: f32 = mix(0.35, 0.03, clamp(_uniform0.detail, 0.0, 1.0));
    let inv2s2: f32 = 1.0 / ((2.0 * s) * s);
    let inv2r2: f32 = 1.0 / ((2.0 * rs) * rs);
    var sum: vec3<f32> = vec3<f32>(0.0);
    var wsum: f32 = 0.0;
    {
      var y: i32 = -2;
      for (; y <= 2; y = y + i32(1)) {
        {
          {
            var x: i32 = -2;
            for (; x <= 2; x = x + i32(1)) {
              {
                let off: vec2<f32> = vec2<f32>(f32(x), f32(y)) * s;
                let sc: vec4<f32> = textureSample(image_Texture, image_Sampler, (coord + off) / _uniform0.resolution);
                if sc.w <= 0.001 {
                  continue;
                }
                let sc3: vec3<f32> = sc.xyz / sc.w;
                let d2: f32 = dot(off, off);
                let ws: f32 = exp(-d2 * inv2s2);
                let diff: vec3<f32> = sc3 - col;
                let cd2: f32 = f32(dot(diff, diff));
                let wr: f32 = exp(-cd2 * inv2r2);
                let w: f32 = ws * wr;
                sum = sum + sc3 * f32(w);
                wsum = wsum + w;
              }
            }
          }
        }
      }
    }
    var filtered: vec3<f32> = col;
    if wsum > 1e-05 {
      {
        filtered = sum / f32(wsum);
      }
    }
    let outc: vec3<f32> = clamp(mix(col, filtered, vec3<f32>(f32(clamp(_uniform0.strength, 0.0, 1.0)))), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(outc * c.w, c.w);
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
