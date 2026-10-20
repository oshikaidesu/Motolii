// Content-Aware Scale: Photoshop's, as a moving picture: each row squeezed where little is going on and kept where much is, then drawn across the whole frame again, so what matters swells and the rest thins (Melt turns it round: what matters goes first, the meme's crumple). Greisen et al., "Algorithm and VLSI Architecture for Real-Time 1080p60 Video Retargeting" (Disney Research, 2012): a column's share min(a x its importance, 1), a found by halving until the shares fill the width kept, a blended with the last frame's; done here per row, on a quarter picture
// --label:Distort
// --pass@e:0,1,s0.25,s0.25
// --pass@band:0,1,s0.25,s0.25
// --pass@scale:1,1,a1,s0.25
// --pass@source:0,1,s0.25,s0.25
// --pass@:0,0,s1,s1
// --track@keep:Keep,0.1,1,0.5,0.01
// --track@melt:Melt,0,1,0,0.01
// --track@hold:Hold together (rows),0,24,3,1
// --track@steady:Steady,0,0.95,0.7,0.01

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var pictureNear: sampler;
@group(0) @binding(2) var e: texture_2d<f32>;
@group(0) @binding(3) var eNear: sampler;
@group(0) @binding(4) var band: texture_2d<f32>;
@group(0) @binding(5) var bandNear: sampler;
@group(0) @binding(6) var scale: texture_2d<f32>;
@group(0) @binding(7) var scaleNear: sampler;
@group(0) @binding(8) var source: texture_2d<f32>;
@group(0) @binding(9) var sourceNear: sampler;
struct Knobs { passindex: f32, frame: f32, keep: f32, melt: f32, hold: f32, steady: f32 }
@group(0) @binding(10) var<uniform> knobs: Knobs;

fn luma(uv: vec2f) -> f32 { return dot(textureSampleLevel(picture, pictureNear, uv, 0.0).rgb, vec3f(0.299, 0.587, 0.114)); }

// (a column's share of the row at that scale: never more than its own width)
fn share(row: i32, x: i32, a: f32) -> f32 { return min(a * textureLoad(band, vec2i(x, row), 0).r, 1.0); }

@fragment fn fs_main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
  let which = i32(knobs.passindex);
  let p = vec2i(frag.xy);
  if (which == 0) {   // (what is going on: the picture's edges, on a quarter of it)
    let d = 1.0 / vec2f(textureDimensions(e));
    let gx = luma(uv + vec2f(d.x, 0.0)) - luma(uv - vec2f(d.x, 0.0));
    let gy = luma(uv + vec2f(0.0, d.y)) - luma(uv - vec2f(0.0, d.y));
    return vec4f(abs(gx) + abs(gy), 0.0, 0.0, 1.0);
  }
  let size = vec2i(textureDimensions(e));
  if (which == 1) {   // (importance: the edges of the rows round it, so neighbouring rows go alike; Melt: the quiet matters, what is busy goes)
    let h = i32(knobs.hold);
    var sum = 0.0;
    for (var k = -h; k <= h; k++) { sum += textureLoad(e, vec2i(p.x, clamp(p.y + k, 0, size.y - 1)), 0).r; }
    let busy = sum / f32(2 * h + 1) + 0.02;
    return vec4f(pow(busy, 1.0 - 2.0 * knobs.melt), 0.0, 0.0, 1.0);
  }
  if (which == 2) {   // (each row's scale a, halved in its logarithm until the shares fill what is kept; the last frame's mixed in)
    let row = p.y;
    let want = knobs.keep * f32(size.x);
    var lo = -16.0;
    var hi = 16.0;
    for (var i = 0; i < 24; i++) {
      let mid = 0.5 * (lo + hi);
      var filled = 0.0;
      for (var x = 0; x < size.x; x++) { filled += share(row, x, exp(mid)); }
      if (filled < want) { lo = mid; } else { hi = mid; }
    }
    var la = 0.5 * (lo + hi);
    let last = textureLoad(scale, vec2i(0, row), 0);
    if (last.g > 0.5 && knobs.frame > 0.0) { la = mix(la, last.r, knobs.steady); }
    var total = 0.0;   // (what the shares fill at that scale: said once a row, not again for every place in it)
    for (var x = 0; x < size.x; x++) { total += share(row, x, exp(la)); }
    return vec4f(la, 1.0, total, 1.0);
  }
  if (which == 3) {   // (where each place of the squeezed row comes from: walked along the shares, across, as a share of the row)
    let row = p.y;
    let said = textureLoad(scale, vec2i(0, row), 0);
    let a = exp(said.r);
    let goal = (f32(p.x) + 0.5) / f32(size.x) * said.b;
    var at = 0.0;
    var src = f32(size.x);
    for (var x = 0; x < size.x; x++) {
      let w = share(row, x, a);
      if (at + w >= goal) { src = f32(x) + (goal - at) / max(w, 1e-6); break; }
      at += w;
    }
    return vec4f(src / f32(size.x), 0.0, 0.0, 1.0);
  }
  // (the picture, each point taken from where its row says, the squeezed row drawn across the whole frame again)
  let q = uv * vec2f(size) - 0.5;
  let i = vec2i(floor(q));
  let f = fract(q);
  let row0 = clamp(i.y, 0, size.y - 1);
  let row1 = clamp(i.y + 1, 0, size.y - 1);
  let c0 = clamp(i.x, 0, size.x - 1);
  let c1 = clamp(i.x + 1, 0, size.x - 1);
  let u0 = mix(textureLoad(source, vec2i(c0, row0), 0).r, textureLoad(source, vec2i(c1, row0), 0).r, f.x);
  let u1 = mix(textureLoad(source, vec2i(c0, row1), 0).r, textureLoad(source, vec2i(c1, row1), 0).r, f.x);
  return textureSampleLevel(picture, pictureNear, vec2f(mix(u0, u1, f.y), uv.y), 0.0);
}
