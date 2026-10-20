// Content-Aware Scale: Photoshop's, as a moving picture: real seams, each running unbroken from one edge to the other, the cheapest taken out until the picture is as narrow (then as low) as it is kept, in the middle (Stretch draws it across the frame again); Melt turns the cost round: what matters goes first. All seams at once, as Huang et al., "Real-time content-aware image resizing" (Science in China F, 2009): each two neighbouring rows matched (a point goes straight on or changes place with its neighbour), so every point lies on one seam; a seam's cost the importance along it, mixed with the last frame's (Steady); the matching found here in pieces of 16 points, not over the whole row; on a quarter picture
// --label:Distort
// (not in a build given to others before 2027-03-06: seam carving itself is MERL's US7477800 until then; research_notes/content_aware_scale_2026-10-20.md)
// --pass@e:0,1,s0.25,s0.25
// --pass@link:0,1,s0.25,s0.25
// --pass@label:0,1,s0.25,s0.25
// --pass@cost:1,1,s0.25,a1
// --pass@rank:0,1,s0.25,a1
// --pass@flag:0,1,s0.25,s0.25
// --pass@source:0,1,s0.25,s0.25
// --pass@wide:0,1,s0.25,s0.25
// --pass@e:0,1,s0.25,s0.25
// --pass@link:0,1,s0.25,s0.25
// --pass@label:0,1,s0.25,s0.25
// --pass@cost2:1,1,a1,s0.25
// --pass@rank2:0,1,a1,s0.25
// --pass@flag:0,1,s0.25,s0.25
// --pass@source2:0,1,s0.25,s0.25
// --pass@:0,0,s1,s1
// --track@keep:Keep width,0.1,1,0.5,0.01
// --track@keep_y:Keep height,0.1,1,0.6,0.01
// --track@melt:Melt,0,1,0,0.01
// --track@steady:Steady,0,0.95,0.7,0.01
// --track@stretch:Stretch,0,1,0,0.01

@group(0) @binding(0) var picture: texture_2d<f32>;
@group(0) @binding(1) var pictureNear: sampler;
@group(0) @binding(2) var e: texture_2d<f32>;
@group(0) @binding(3) var eNear: sampler;
@group(0) @binding(4) var link: texture_2d<f32>;
@group(0) @binding(5) var linkNear: sampler;
@group(0) @binding(6) var label: texture_2d<f32>;
@group(0) @binding(7) var labelNear: sampler;
@group(0) @binding(8) var cost: texture_2d<f32>;
@group(0) @binding(9) var costNear: sampler;
@group(0) @binding(10) var rank: texture_2d<f32>;
@group(0) @binding(11) var rankNear: sampler;
@group(0) @binding(12) var flag: texture_2d<f32>;
@group(0) @binding(13) var flagNear: sampler;
@group(0) @binding(14) var source: texture_2d<f32>;
@group(0) @binding(15) var sourceNear: sampler;
@group(0) @binding(16) var wide: texture_2d<f32>;
@group(0) @binding(17) var wideNear: sampler;
@group(0) @binding(18) var cost2: texture_2d<f32>;
@group(0) @binding(19) var cost2Near: sampler;
@group(0) @binding(20) var rank2: texture_2d<f32>;
@group(0) @binding(21) var rank2Near: sampler;
@group(0) @binding(22) var source2: texture_2d<f32>;
@group(0) @binding(23) var source2Near: sampler;
struct Knobs { passindex: f32, frame: f32, keep: f32, keep_y: f32, melt: f32, steady: f32, stretch: f32 }
@group(0) @binding(24) var<uniform> knobs: Knobs;

const PIECE = 16;

// (a point of a line: how far along it, and which line; seams across the picture's width run down (axis 0), seams across its height run across (axis 1))
fn pt(axis: i32, along: i32, line: i32) -> vec2i { return select(vec2i(line, along), vec2i(along, line), axis == 0); }
fn val(t: texture_2d<f32>, axis: i32, along: i32, line: i32) -> f32 { return textureLoad(t, pt(axis, along, line), 0).r; }

// (what matters at a point: the picture's edges there; Melt: the quiet matters, what is busy goes first)
fn importance(t: texture_2d<f32>, s: sampler, uv: vec2f, d: vec2f) -> f32 {
  let w = vec3f(0.299, 0.587, 0.114);
  let gx = dot(textureSampleLevel(t, s, uv + vec2f(d.x, 0.0), 0.0).rgb - textureSampleLevel(t, s, uv - vec2f(d.x, 0.0), 0.0).rgb, w);
  let gy = dot(textureSampleLevel(t, s, uv + vec2f(0.0, d.y), 0.0).rgb - textureSampleLevel(t, s, uv - vec2f(0.0, d.y), 0.0).rgb, w);
  return pow(abs(gx) + abs(gy) + 0.02, 1.0 - 2.0 * knobs.melt);
}

// (where a point of line r goes on line r + 1: straight on, or changed with a neighbour, so what is alike goes on alike;
// the best matching of its piece of 16, the pieces set a little further on each line so seams pass between them)
fn linked(axis: i32, x: i32, r: i32, n: i32, lines: i32) -> i32 {
  if (r >= lines - 1) { return x; }
  let o = (r * 7) % PIECE;
  let s = max(((x - o + PIECE) / PIECE) * PIECE - PIECE + o, 0);
  let t = min(s + PIECE, n);
  var best: array<f32, 17>;
  var swapped: array<bool, 17>;
  best[0] = 0.0;
  for (var k = 1; k <= t - s; k++) {
    let i = s + k - 1;
    best[k] = best[k - 1] + abs(val(e, axis, i, r) - val(e, axis, i, r + 1));
    swapped[k] = false;
    if (k >= 2) {
      let c = best[k - 2] + abs(val(e, axis, i - 1, r) - val(e, axis, i, r + 1)) + abs(val(e, axis, i, r) - val(e, axis, i - 1, r + 1)) + 0.01;
      if (c < best[k]) { best[k] = c; swapped[k] = true; }
    }
  }
  var k = t - s;
  loop {
    if (k <= 0) { break; }
    let i = s + k - 1;
    if (swapped[k]) {
      if (x == i) { return i - 1; }
      if (x == i - 1) { return i; }
      k -= 2;
    } else {
      if (x == i) { return i; }
      k -= 1;
    }
  }
  return x;
}

// (the seam a point lies on: named by where it begins, found by going back up the lines; a change of places undoes itself)
fn seam(axis: i32, x: i32, line: i32) -> i32 {
  var at = x;
  for (var r = line - 1; r >= 0; r--) { at = i32(val(link, axis, at, r)); }
  return at;
}

// (a seam's cost: what matters along it, from one edge to the other)
fn along(axis: i32, s: i32, lines: i32) -> f32 {
  var at = s;
  var sum = val(e, axis, at, 0);
  for (var r = 0; r < lines - 1; r++) {
    at = i32(val(link, axis, at, r));
    sum += val(e, axis, at, r + 1);
  }
  return sum;
}

// (how many seams cost less than this one: the cheapest goes first)
fn ranked(c: texture_2d<f32>, axis: i32, s: i32, n: i32) -> f32 {
  let mine = val(c, axis, s, 0);
  var below = 0;
  for (var t = 0; t < n; t++) {
    let theirs = val(c, axis, t, 0);
    if (theirs < mine || (theirs == mine && t < s)) { below++; }
  }
  return f32(below);
}

// (where a place of the line left comes from: between the points kept, as a share of the line)
fn taken(axis: i32, place: i32, line: i32, n: i32, kept: i32) -> f32 {
  let goal = clamp((f32(place) + 0.5) / f32(n) * f32(kept) - 0.5, 0.0, f32(kept - 1));
  let k0 = i32(floor(goal));
  var count = 0;
  var p0 = 0.0;
  for (var x = 0; x < n; x++) {
    if (val(flag, axis, x, line) < 0.5) { continue; }
    if (count == k0) { p0 = f32(x); }
    if (count == k0 + 1) { return (mix(p0, f32(x), fract(goal)) + 0.5) / f32(n); }
    count++;
  }
  return (p0 + 0.5) / f32(n);
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

fn one(v: f32) -> vec4f { return vec4f(v, 0.0, 0.0, 1.0); }

@fragment fn fs_main(@builtin(position) frag: vec4f, @location(0) uv: vec2f) -> @location(0) vec4f {
  let which = i32(knobs.passindex);
  let p = vec2i(frag.xy);
  let size = vec2i(textureDimensions(e));
  let d = 1.0 / vec2f(size);
  let axis = select(0, 1, which >= 8);
  let n = select(size.x, size.y, axis == 1);       // (points along a line)
  let lines = select(size.y, size.x, axis == 1);
  let x = select(p.x, p.y, axis == 1);             // (this point: along its line, and its line)
  let r = select(p.y, p.x, axis == 1);
  let kept = max(1, i32(round(select(knobs.keep, knobs.keep_y, axis == 1) * f32(n))));
  switch which {
    case 0: { return one(importance(picture, pictureNear, uv, d)); }
    case 8: { return one(importance(wide, wideNear, uv, d)); }
    case 1, 9: { return one(f32(linked(axis, x, r, n, lines))); }
    case 2, 10: { return one(f32(seam(axis, x, r))); }
    case 3: {
      let last = textureLoad(cost, p, 0);
      var c = along(0, x, lines);
      if (last.g > 0.5 && knobs.frame > 0.0) { c = mix(c, last.r, knobs.steady); }
      return vec4f(c, 1.0, 0.0, 1.0);
    }
    case 11: {
      let last = textureLoad(cost2, p, 0);
      var c = along(1, x, lines);
      if (last.g > 0.5 && knobs.frame > 0.0) { c = mix(c, last.r, knobs.steady); }
      return vec4f(c, 1.0, 0.0, 1.0);
    }
    case 4: { return one(ranked(cost, 0, x, n)); }
    case 12: { return one(ranked(cost2, 1, x, n)); }
    case 5: { return one(select(0.0, 1.0, val(rank, 0, i32(val(label, 0, x, r)), 0) >= f32(n - kept))); }
    case 13: { return one(select(0.0, 1.0, val(rank2, 1, i32(val(label, 1, x, r)), 0) >= f32(n - kept))); }
    case 6, 14: { return one(taken(axis, x, r, n, kept)); }
    case 7: { return textureSampleLevel(picture, pictureNear, vec2f(between(source, uv), uv.y), 0.0); }   // (the seams across its width taken out, for the others to be found in)
    default: {}
  }
  // (the picture: each point's height from its column, then its place across from that row; in the middle, as wide and as high as kept, or stretched)
  let span = mix(vec2f(knobs.keep, knobs.keep_y), vec2f(1.0), knobs.stretch);
  let w = (uv - 0.5) / span + 0.5;
  if (any(w < vec2f(0.0)) || any(w > vec2f(1.0))) { return vec4f(0.0); }
  let v = between(source2, w);
  return textureSampleLevel(picture, pictureNear, vec2f(between(source, vec2f(w.x, v)), v), 0.0);
}
