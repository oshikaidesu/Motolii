// Content-Aware Scale: Photoshop's, as a moving picture: each row squeezed where little is going on and kept where much is, then each column of that the same way, the picture as narrow and as low as it is kept, in the middle (Stretch draws it across the frame again); Crush takes the thinned lines out whole, each its own, the seams' crumple (Melt turns it round: what matters goes first). Greisen et al., "Algorithm and VLSI Architecture for Real-Time 1080p60 Video Retargeting" (Disney Research, 2012): a column's share min(a x its importance, 1), a found by halving until the shares fill what is kept, a blended with the last frame's; done here per row, then per column, on a quarter picture
// --label:Distort
// --pass@e:0,1,s0.25,s0.25
// --pass@band:0,1,s0.25,s0.25
// --pass@scale:1,1,a1,s0.25
// --pass@source:0,1,s0.25,s0.25
// --pass@wide:0,1,s0.25,s0.25
// --pass@e2:0,1,s0.25,s0.25
// --pass@band2:0,1,s0.25,s0.25
// --pass@scale2:1,1,s0.25,a1
// --pass@source2:0,1,s0.25,s0.25
// --pass@:0,0,s1,s1
// --track@keep:Keep width,0.1,1,0.5,0.01
// --track@keep_y:Keep height,0.1,1,0.6,0.01
// --track@melt:Melt,0,1,0,0.01
// --track@hold:Hold together (lines),0,24,3,1
// --track@steady:Steady,0,0.95,0.7,0.01
// --track@crush:Crush,0,1,0.85,0.01
// --track@stretch:Stretch,0,1,0,0.01

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
@group(0) @binding(10) var wide: texture_2d<f32>;
@group(0) @binding(11) var wideNear: sampler;
@group(0) @binding(12) var e2: texture_2d<f32>;
@group(0) @binding(13) var e2Near: sampler;
@group(0) @binding(14) var band2: texture_2d<f32>;
@group(0) @binding(15) var band2Near: sampler;
@group(0) @binding(16) var scale2: texture_2d<f32>;
@group(0) @binding(17) var scale2Near: sampler;
@group(0) @binding(18) var source2: texture_2d<f32>;
@group(0) @binding(19) var source2Near: sampler;
struct Knobs { passindex: f32, frame: f32, keep: f32, keep_y: f32, melt: f32, hold: f32, steady: f32, crush: f32, stretch: f32 }
@group(0) @binding(20) var<uniform> knobs: Knobs;

// (a place on a line: along it, and which line; rows run across (axis 0), columns down (axis 1))
fn at(axis: i32, along: i32, line: i32) -> vec2i { return select(vec2i(line, along), vec2i(along, line), axis == 0); }

// (a share of the line at that scale: never more than its own width; crushed, a thin one goes to nothing, as a seam taken out)
fn share(b: texture_2d<f32>, axis: i32, along: i32, line: i32, a: f32) -> f32 {
  return pow(min(a * textureLoad(b, at(axis, along, line), 0).r, 1.0), 1.0 + 24.0 * knobs.crush);
}

// (what is going on: the picture's edges)
fn edges(t: texture_2d<f32>, s: sampler, uv: vec2f, d: vec2f) -> f32 {
  let w = vec3f(0.299, 0.587, 0.114);
  let gx = dot(textureSampleLevel(t, s, uv + vec2f(d.x, 0.0), 0.0).rgb - textureSampleLevel(t, s, uv - vec2f(d.x, 0.0), 0.0).rgb, w);
  let gy = dot(textureSampleLevel(t, s, uv + vec2f(0.0, d.y), 0.0).rgb - textureSampleLevel(t, s, uv - vec2f(0.0, d.y), 0.0).rgb, w);
  return abs(gx) + abs(gy);
}

// (importance: the edges of the lines round it, so neighbouring lines go alike; Melt: the quiet matters, what is busy goes)
fn importance(t: texture_2d<f32>, axis: i32, p: vec2i, size: vec2i) -> f32 {
  let h = i32(knobs.hold);
  let across = select(p.x, p.y, axis == 0);
  let most = select(size.x, size.y, axis == 0) - 1;
  var sum = 0.0;
  for (var k = -h; k <= h; k++) {
    let q = clamp(across + k, 0, most);
    sum += textureLoad(t, select(vec2i(q, p.y), vec2i(p.x, q), axis == 0), 0).r;
  }
  return pow(sum / f32(2 * h + 1) + 0.02, 1.0 - 2.0 * knobs.melt);
}

// (each line's scale a, halved in its logarithm until the shares fill what is kept; the last frame's mixed in; and what they fill then)
fn scaled(b: texture_2d<f32>, last: vec4f, axis: i32, line: i32, n: i32, keep: f32) -> vec4f {
  let want = keep * f32(n);
  var lo = -16.0;
  var hi = 16.0;
  for (var i = 0; i < 16; i++) {
    let mid = 0.5 * (lo + hi);
    var filled = 0.0;
    for (var x = 0; x < n; x++) { filled += share(b, axis, x, line, exp(mid)); }
    if (filled < want) { lo = mid; } else { hi = mid; }
  }
  var la = 0.5 * (lo + hi);
  if (last.g > 0.5 && knobs.frame > 0.0) { la = mix(la, last.r, knobs.steady); }
  var total = 0.0;
  for (var x = 0; x < n; x++) { total += share(b, axis, x, line, exp(la)); }
  return vec4f(la, 1.0, total, 1.0);
}

// (where a place of the squeezed line comes from: walked along the shares, as a share of the line)
fn taken(b: texture_2d<f32>, said: vec4f, axis: i32, place: i32, line: i32, n: i32) -> f32 {
  let a = exp(said.r);
  let goal = (f32(place) + 0.5) / f32(n) * said.b;
  var gone = 0.0;
  for (var x = 0; x < n; x++) {
    let w = share(b, axis, x, line, a);
    if (gone + w >= goal) { return (f32(x) + (goal - gone) / max(w, 1e-6)) / f32(n); }
    gone += w;
  }
  return 1.0;
}

// (a map read between its places, as the picture is drawn finer than it)
fn between(t: texture_2d<f32>, uv: vec2f) -> f32 {
  let size = vec2i(textureDimensions(t));
  let q = uv * vec2f(size) - 0.5;
  let i = vec2i(floor(q));
  let f = fract(q);
  let a = clamp(i, vec2i(0), size - 1);
  let b = clamp(i + 1, vec2i(0), size - 1);
  let top = mix(textureLoad(t, a, 0).r, textureLoad(t, vec2i(b.x, a.y), 0).r, f.x);
  let low = mix(textureLoad(t, vec2i(a.x, b.y), 0).r, textureLoad(t, b, 0).r, f.x);
  return mix(top, low, f.y);
}

@fragment fn fs_main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
  let which = i32(knobs.passindex);
  let p = vec2i(frag.xy);
  let size = vec2i(textureDimensions(e));
  let d = 1.0 / vec2f(size);
  switch which {
    case 0: { return vec4f(edges(picture, pictureNear, uv, d), 0.0, 0.0, 1.0); }
    case 1: { return vec4f(importance(e, 0, p, size), 0.0, 0.0, 1.0); }
    case 2: { return scaled(band, textureLoad(scale, vec2i(0, p.y), 0), 0, p.y, size.x, knobs.keep); }
    case 3: { return vec4f(taken(band, textureLoad(scale, vec2i(0, p.y), 0), 0, p.x, p.y, size.x), 0.0, 0.0, 1.0); }
    case 4: { return textureSampleLevel(picture, pictureNear, vec2f(between(source, uv), uv.y), 0.0); }   // (the rows squeezed, for the columns to see)
    case 5: { return vec4f(edges(wide, wideNear, uv, d), 0.0, 0.0, 1.0); }
    case 6: { return vec4f(importance(e2, 1, p, size), 0.0, 0.0, 1.0); }
    case 7: { return scaled(band2, textureLoad(scale2, vec2i(p.x, 0), 0), 1, p.x, size.y, knobs.keep_y); }
    case 8: { return vec4f(taken(band2, textureLoad(scale2, vec2i(p.x, 0), 0), 1, p.y, p.x, size.y), 0.0, 0.0, 1.0); }
    default: {}
  }
  // (the picture: each point's height from its column, then its place across from that row; in the middle, as wide and as high as kept, or stretched)
  let span = mix(vec2f(knobs.keep, knobs.keep_y), vec2f(1.0), knobs.stretch);
  let w = (uv - 0.5) / span + 0.5;
  if (any(w < vec2f(0.0)) || any(w > vec2f(1.0))) { return vec4f(0.0); }
  let v = between(source2, w);
  return textureSampleLevel(picture, pictureNear, vec2f(between(source, vec2f(w.x, v)), v), 0.0);
}
