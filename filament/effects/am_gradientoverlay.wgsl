// Gradient Overlay: AbsentMotion's gradientoverlay (Apache-2.0, Copyright 2026 The AbsentMotion authors, github.com/Comett1225/AbsentMotion), its SkSL made WGSL by Skia's skslc (filament/effects/from_absent.py)
// --label:Color
// --track@color1_x:Color 1 R,0,1,0.2275,0.01
// --track@color1_y:Color 1 G,0,1,0.1098,0.01
// --track@color1_z:Color 1 B,0,1,0.4431,0.01
// --track@color1_w:Color 1 A,0,1,1,0.01
// --track@color2_x:Color 2 R,0,1,1,0.01
// --track@color2_y:Color 2 G,0,1,0.6863,0.01
// --track@color2_z:Color 2 B,0,1,0.4824,0.01
// --track@color2_w:Color 2 A,0,1,1,0.01
// --track@shape:Shape,0,1,0.0,0.01
// --track@scale:Scale,0.05,10,1.0,0.01
// --track@opacity:Opacity,0,1,0.6,0.01
// --track@blend:Blend,0,1,0.0,0.01
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
  color1: vec4<f32>,
  color2: vec4<f32>,
  shape: f32,
  angle: f32,
  scale: f32,
  opacity: f32,
  blend: f32,
};
@group(0) @binding(2) var<uniform> _uniform0 : Knobs;
@group(0) @binding(1) var image_Sampler: sampler;
@group(0) @binding(0) var image_Texture: texture_2d<f32>;
fn blendWith_f3ff3f3(m: f32, b: vec3<f32>, s: vec3<f32>) -> vec3<f32> {
  {
    if m < 0.5 {
      return s;
    }
    if m < 1.5 {
      return b * s;
    }
    if m < 2.5 {
      return (b + s) - b * s;
    }
    if m < 3.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), b));
    }
    if m < 4.5 {
      {
        let d: vec3<f32> = mix(sqrt(b), ((16.0 * b - 12.0) * b + 4.0) * b, step(b, vec3<f32>(0.25)));
        return mix(b - ((1.0 - 2.0 * s) * b) * (1.0 - b), b + (2.0 * s - 1.0) * (d - b), step(vec3<f32>(0.5), s));
      }
    }
    if m < 5.5 {
      return mix((2.0 * b) * s, 1.0 - (2.0 * (1.0 - b)) * (1.0 - s), step(vec3<f32>(0.5), s));
    }
    if m < 6.5 {
      return min(b + s, vec3<f32>(1.0));
    }
    if m < 7.5 {
      return min(b, s);
    }
    if m < 8.5 {
      return max(b, s);
    }
    if m < 9.5 {
      return abs(b - s);
    }
    if m < 10.5 {
      return min(b / max(1.0 - s, vec3<f32>(0.0001)), vec3<f32>(1.0));
    }
    return 1.0 - min((1.0 - b) / max(s, vec3<f32>(0.0001)), vec3<f32>(1.0));
  }
}
fn effect__h4f2(coord: vec2<f32>) -> vec4<f32> {
  {
    let c: vec4<f32> = textureSample(image_Texture, image_Sampler, coord / _uniform0.resolution);
    if c.w <= 0.0 {
      return c;
    }
    let base: vec3<f32> = vec3<f32>(c.xyz) / f32(c.w);
    let p: vec2<f32> = coord - 0.5 * _uniform0.resolution;
    let r: f32 = radians(_uniform0.angle);
    let dir: vec2<f32> = vec2<f32>(cos(r), sin(r));
    let k: f32 = max(_uniform0.scale, 0.001);
    let reach: f32 = max(abs(_uniform0.resolution.x * dir.x) + abs(_uniform0.resolution.y * dir.y), 1.0);
    var u: f32;
    if _uniform0.shape < 0.5 {
      u = dot(p, dir) / (k * reach) + 0.5;
    } else {
      if _uniform0.shape < 1.5 {
        u = length(p / max(0.5 * _uniform0.resolution, vec2<f32>(1.0))) / k;
      } else {
        if _uniform0.shape < 2.5 {
          u = (2.0 * abs(dot(p, dir))) / (k * reach);
        } else {
          u = fract((atan2(p.y, p.x) - r) * 0.159154937);
        }
      }
    }
    let g: vec4<f32> = mix(vec4<f32>(_uniform0.color1), vec4<f32>(_uniform0.color2), vec4<f32>(clamp(u, 0.0, 1.0)));
    let rgb: vec3<f32> = mix(base, clamp(blendWith_f3ff3f3(_uniform0.blend, base, g.xyz), vec3<f32>(0.0), vec3<f32>(1.0)), vec3<f32>(clamp(_uniform0.opacity, 0.0, 1.0) * g.w));
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
