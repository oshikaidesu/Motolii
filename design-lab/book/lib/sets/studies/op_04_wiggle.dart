// OP 04 Wiggle: 20 OP-1 style panels. Values: 0 FREQ (blue), 1 AMP (green), 2 SMOOTH (red, 1 = one soft wave, 0 = jagged).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import 'op_04_kit.dart';

const _w = K.white;
final _dimW = al(K.white, .35);

void _three(Canvas c, OpV v, Offset at, {double h = 5, double gap = 8}) {
  lb(c, 'FREQ ${d2(v[0])}', at, pc(v, 0), h: h);
  lb(c, 'AMP ${d2(v[1])}', at + Offset(0, gap), pc(v, 1), h: h);
  lb(c, 'SMTH ${d2(v[2])}', at + Offset(0, gap * 2), pc(v, 2), h: h);
}

// 1 Hand-held camera: the viewfinder drifts over a still skyline.
void _cam(Canvas c, Size s, OpV v, double t) {
  final sk = <Offset>[const Offset(0, 86)];
  var x = 0.0;
  for (final h in const <double>[10, 22, 14, 34, 18, 26, 12, 40, 20, 16, 28, 8]) {
    sk..add(Offset(x, 86 - h))..add(Offset(x + 13, 86 - h));
    x += 13;
    sk.add(Offset(x, 86));
  }
  pl(c, sk, _dimW, .9);
  ln(c, const Offset(0, 86), const Offset(156, 86), _dimW);
  final o = Offset(wv(t, v, 1) * 18, wv(t, v, 2) * 12);
  final rot = wv(t, v, 3) * .08;
  c.save();
  c.translate(78 + o.dx, 60 + o.dy);
  c.rotate(rot);
  const r = Rect.fromLTRB(-48, -34, 48, 34);
  for (final k in [r.topLeft, r.topRight, r.bottomLeft, r.bottomRight]) {
    final sx = k.dx < 0 ? 1.0 : -1.0, sy = k.dy < 0 ? 1.0 : -1.0;
    pl(c, [k + Offset(0, 10 * sy), k, k + Offset(10 * sx, 0)], _w, 1.3);
  }
  ln(c, const Offset(-5, 0), const Offset(5, 0), _w, .8);
  ln(c, const Offset(0, -5), const Offset(0, 5), _w, .8);
  c.restore();
  if ((t * 1.5) % 1 < .6) dot(c, const Offset(10, 10), 2.4, K.red);
  lb(c, 'REC', const Offset(16, 7), K.red);
  lb(c, 'FREQ ${d2(v[0])}', const Offset(150, 7), pc(v, 0), ax: 1);
  lb(c, 'AMP ${d2(v[1])}', const Offset(6, 104), pc(v, 1), h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(150, 104), pc(v, 2), ax: 1, h: 5);
}

// 2 Chihuahua: rub to make it cold; it shivers.
void _dog(Canvas c, Size s, OpV v, double t) {
  final o = Offset(70 + wv(t, v, 1) * 6, 64 + wv(t, v, 2) * 3);
  c.drawOval(Rect.fromCenter(center: o + const Offset(-6, 14), width: 36, height: 18), st(_w, 1.1));
  final hd = o + const Offset(14, -6);
  ci(c, hd, 11, _w, 1.1);
  pl(c, [hd + const Offset(-9, -5), hd + const Offset(-16, -22), hd + const Offset(-3, -10)], _w, 1);
  pl(c, [hd + const Offset(5, -10), hd + const Offset(16, -22), hd + const Offset(10, -4)], _w, 1);
  ci(c, hd + const Offset(-3, -1), 3, _w, .9);
  ci(c, hd + const Offset(5, -1), 3, _w, .9);
  dot(c, hd + const Offset(-2, -1), 1.3, _w);
  dot(c, hd + const Offset(6, -1), 1.3, _w);
  dot(c, hd + const Offset(2, 5), 1.2, _w);
  for (var k = 0; k < 4; k++) {
    final lx = o.dx - 18 + k * 9.0;
    ln(c, Offset(lx, o.dy + 20), Offset(lx + wv(t + k, v, 5) * 3, 100), _w, .9);
  }
  fn(c, 8, (u) => o + Offset(-24 - u * 8, 10 - u * 10 + math.sin(t * 20) * 2), _w, .9);
  final amp = v[1];
  for (var k = 0; k < 3; k++) {
    c.drawArc(Rect.fromCircle(center: o + const Offset(0, 6), radius: 36.0 + k * 5), -.4, .8, false, st(pc(v, 1, amp * (1 - k * .3)), .8));
    c.drawArc(Rect.fromCircle(center: o + const Offset(0, 6), radius: 36.0 + k * 5), math.pi - .4, .8, false, st(pc(v, 1, amp * (1 - k * .3)), .8));
  }
  ln(c, const Offset(10, 100), const Offset(146, 100), K.dim);
  nm(c, d2(v[1]), const Offset(150, 8), 18, pc(v, 1), ax: 1);
  lb(c, 'SHIVER', const Offset(150, 30), pc(v, 1), ax: 1, h: 5);
  lb(c, 'FREQ ${d2(v[0])}', const Offset(6, 8), pc(v, 0));
  lb(c, 'SMTH ${d2(v[2])}', const Offset(6, 18), pc(v, 2));
}

// 3 Polygraph: three pens on scrolling paper, each a different seed.
void _poly(Canvas c, Size s, OpV v, double t) {
  for (var k = 0; k < 9; k++) {
    final x = k * 16.0 - (t * 30) % 16;
    ln(c, Offset(x, 6), Offset(x, 114), K.dim, .5);
  }
  const penX = 112.0;
  for (var k = 0; k < 3; k++) {
    final y0 = 26 + k * 34.0;
    fn(c, 80, (u) {
      final x = u * penX;
      return Offset(x, y0 + wv(t - (penX - x) / 30, v, k * 3.0) * 13);
    }, k == 1 ? K.white : _dimW, 1);
    final py = y0 + wv(t, v, k * 3.0) * 13;
    ln(c, Offset(penX, py), Offset(146, y0), _w, .9);
    ci(c, Offset(146, y0), 2.4, _w, .9);
    dot(c, Offset(penX, py), 1.6, [K.blue, K.green, K.red][k]);
  }
  lb(c, 'FREQ ${d2(v[0])}', const Offset(6, 108), pc(v, 0), h: 5);
  lb(c, 'AMP ${d2(v[1])}', const Offset(56, 108), pc(v, 1), h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(100, 108), pc(v, 2), h: 5);
}

// 4 Firefly: a light that wanders; its trail draws the wiggle in two dimensions.
void _firefly(Canvas c, Size s, OpV v, double t) {
  for (var k = 0; k < 12; k++) {
    final x = 6 + k * 13.0;
    fn(c, 6, (u) => Offset(x + math.sin(u * 2 + t + k) * 3 * u, 118 - u * (12 + k % 3 * 6)), al(K.green, .5), .8);
  }
  const o = Offset(78, 56);
  Offset at(double tt) => o + Offset(wv(tt, v, 1) * 60, wv(tt, v, 2) * 40);
  for (var k = 1; k < 40; k++) {
    dot(c, at(t - k * .04), .9 * (1 - k / 40) + .3, al(K.white, (1 - k / 40) * .8));
  }
  final p = at(t);
  dot(c, p, 2, K.white);
  ci(c, p, 5, pc(v, 2, .6), .7);
  ci(c, p, 9, pc(v, 2, .25), .6);
  ln(c, p + const Offset(-3, -3), p + const Offset(-6, -6), _w, .7);
  ln(c, p + const Offset(3, -3), p + const Offset(6, -6), _w, .7);
  c.drawOval(Rect.fromCenter(center: o, width: 120 * v[1] + 2, height: 80 * v[1] + 2), st(pc(v, 1, .4), .6));
  _three(c, v, const Offset(6, 6));
}

// 5 Heat haze: a mesa shimmering above a desert road; rub harder for a hotter day.
void _haze(Canvas c, Size s, OpV v, double t) {
  ci(c, const Offset(120, 26), 10, _w, 1);
  const hz = 66.0;
  ln(c, const Offset(0, hz), const Offset(156, hz), _dimW);
  pl(c, const [Offset(78, 120), Offset(76, hz)], K.dim);
  pl(c, const [Offset(20, 120), Offset(74, hz)], _w);
  pl(c, const [Offset(136, 120), Offset(82, hz)], _w);
  for (var k = 0; k < 9; k++) {
    final y = hz - 2 - k * 3.0;
    final fall = 1 - k / 9;
    final ox = wv(t * 1.3 + k * .37, v, k.toDouble()) * 10 * fall;
    final mesa = y > hz - 24 ? (y > hz - 6 ? 66.0 : 40.0) : 0.0;
    if (mesa > 0) ln(c, Offset(14 + ox + (hz - y) * .4, y), Offset(14 + mesa + ox - (hz - y) * .3, y), al(K.white, .4 + .5 * fall), .9);
    ln(c, Offset(70 + ox * 1.4, hz - 1 - k * 1.2), Offset(86 + ox * 1.4, hz - 1 - k * 1.2), pc(v, 2, .35 * fall), .7);
  }
  _three(c, v, const Offset(6, 8));
}

// 6 Line boil: the word is redrawn a few times a second, like hand-drawn animation.
void _boil(Canvas c, Size s, OpV v, double t) {
  final fps = 2 + 22 * v[0];
  final fr = (t * fps).floor();
  final f = t * fps - fr;
  for (var copy = 0; copy < 2; copy++) {
    double jit(int i) => (math.sin((fr * 7 + i + copy * 31) * 12.9898) * 43758.5453) % 1;
    double jit2(int i) => (math.sin(((fr + 1) * 7 + i + copy * 31) * 12.9898) * 43758.5453) % 1;
    final m = v[2];
    final dx = lr(jit(1), lr(jit(1), jit2(1), f), m) * 2 - 1, dy = lr(jit(2), lr(jit(2), jit2(2), f), m) * 2 - 1;
    tx(c, 'BOIL', Offset(16 + dx * 5 * v[1], 30 + dy * 5 * v[1]), 34, copy == 0 ? K.white : al(K.white, .35), w: 1.2, track: .45);
  }
  lb(c, 'FPS', const Offset(6, 96), pc(v, 0));
  nm(c, fps.round().toString().padLeft(2, '0'), const Offset(6, 104), 12, pc(v, 0));
  lb(c, 'AMP ${d2(v[1])}', const Offset(60, 104), pc(v, 1), h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(60, 112), pc(v, 2), h: 5);
  lb(c, 'DRAG', const Offset(150, 104), K.grey, ax: 1, h: 5);
}

// 7 Sample and hold lane (Max LFO): RATE, DEPTH, SLEW. Dots are the samples, the line the slew between them.
void _sh(Canvas c, Size s, OpV v, double t) {
  for (var i = 0; i < 26; i++) {
    for (var j = 0; j < 12; j++) {
      dot(c, Offset(6 + i * 5.8, 14 + j * 6.6), .45, K.dim);
    }
  }
  const y0 = 50.0;
  final hz = .3 + 11.7 * v[0] * v[0];
  final n = (hz * 2.4).clamp(2, 40).round();
  final pts = <Offset>[];
  for (var k = 0; k <= n; k++) {
    final x = 6 + 144.0 * k / n;
    final tt = t + k / hz;
    final y = y0 + wg((tt * hz).floorToDouble() / hz, v[0], 0, 7) * 34 * v[1];
    pts.add(Offset(x, y));
  }
  for (var k = 0; k < n; k++) {
    pl(c, [pts[k], Offset(pts[k + 1].dx, pts[k].dy), pts[k + 1]], al(K.white, .25), .7);
    dot(c, pts[k], 1.6, pc(v, 0));
  }
  final slew = v[2];
  fn(c, n * 8, (u) {
    final f = u * n, k = math.min(n - 1, f.floor()), fr = f - k;
    final e = fr < slew ? (1 - math.cos(fr / math.max(.001, slew) * math.pi)) / 2 : 1.0;
    return Offset(lr(pts[k].dx, pts[k + 1].dx, fr), lr(pts[k].dy, pts[k + 1].dy, slew < .02 ? (fr > 0 ? 1 : 0) : e));
  }, _w, 1.2);
  rd(c, v, 0, 'RATE', const Offset(6, 94), h: 10);
  rd(c, v, 1, 'DEPTH', const Offset(56, 94), h: 10);
  rd(c, v, 2, 'SLEW', const Offset(150, 94), h: 10, ax: 1);
}

// 8 Spectrum: the wiggle seen as frequencies; pinch the hump.
void _spec(Canvas c, Size s, OpV v, double t) {
  const x0 = 10.0, x1 = 148.0, y0 = 96.0;
  ln(c, const Offset(x0, y0), const Offset(x1, y0), _dimW);
  for (var k = 0; k < 5; k++) {
    final x = lr(x0, x1, k / 4);
    ln(c, Offset(x, y0), Offset(x, y0 + 3), _dimW, .7);
    lb(c, ['.1', '1', '10', '100', '1K'][k], Offset(x, y0 + 6), K.grey, ax: .5, h: 4);
  }
  final peak = .15 + .6 * v[0], width = .05 + .35 * (1 - v[2]);
  fn(c, 90, (u) {
    final g = math.exp(-((u - peak) * (u - peak)) / (2 * width * width));
    final tail = u > peak ? math.pow(math.max(0.0, 1 - (u - peak) * (1.2 + 3 * v[2])), 2) : 1.0;
    final n = (math.sin(u * 230 + t * 9) * .5 + .5) * (1 - v[2]) * .3;
    final y = (g * .8 + .2 * tail * (u < peak ? u / peak : 1)) * (1 - n) * (.2 + .8 * v[1]);
    return Offset(lr(x0, x1, u), y0 - 70 * y);
  }, _w, 1);
  final px = lr(x0, x1, peak);
  dl(c, Offset(px, y0), Offset(px, 14), pc(v, 0));
  dot(c, Offset(px, 14), 2, pc(v, 0));
  final hz = .3 + 11.7 * v[0] * v[0];
  nm(c, hz.toStringAsFixed(1), Offset(px + 4, 12), 12, pc(v, 0));
  lb(c, 'AMP ${d2(v[1])}', const Offset(10, 8), pc(v, 1), h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(10, 16), pc(v, 2), h: 5);
}

// 9 Bobber: is it a fish? The float on the water shows the wiggle you want.
void _bobber(Canvas c, Size s, OpV v, double t) {
  const wy = 74.0;
  fn(c, 40, (u) => Offset(u * 156, wy + math.sin(u * 12 + t * 1.5) * 1), al(K.blue, .8), .9);
  for (var k = 0; k < 3; k++) {
    fn(c, 20, (u) => Offset(20 + u * 116, wy + 8 + k * 7 + math.sin(u * 8 + t + k) * 1), al(K.blue, .3 - k * .07), .7);
  }
  final b = Offset(98 + wv(t * .7, v, 2) * 6, wy + wv(t, v, 1) * 10);
  ci(c, b, 6, _w, 1.1);
  ln(c, b + const Offset(-6, 0), b + const Offset(6, 0), _w, .8);
  ln(c, b + const Offset(0, -6), b + const Offset(0, -12), K.red, 1);
  fn(c, 20, (u) => ol(b + const Offset(0, -12), const Offset(30, 16), u) + Offset(0, math.sin(u * math.pi) * 10), _dimW, .7);
  pl(c, const [Offset(0, 116), Offset(30, 16)], _w, 1.3);
  for (var k = 0; k < 2; k++) {
    final r = 8 + ((t * 10 + k * 8) % 16);
    c.drawOval(Rect.fromCenter(center: Offset(b.dx, wy + 2), width: r * 2.6, height: r * .5), st(al(K.white, (1 - (r - 8) / 16) * v[1]), .7));
  }
  _three(c, v, const Offset(100, 8));
}

// 10 Drunk walker: an isometric path, a figure that can't keep to it, footprints of where it went.
void _drunk(Canvas c, Size s, OpV v, double t) {
  const o = Offset(30, 30);
  const sc = 8.0;
  isoGrid(c, o, 12, 3, sc, K.dim);
  dl(c, iso(o, 0, 1.5, 0, sc), iso(o, 12, 1.5, 0, sc), _dimW, 4);
  final ph = (t * .12) % 1;
  for (var k = 1; k < 22; k++) {
    final pk = ph - k * .012;
    if (pk < 0) break;
    final y = 1.5 + wv(t - k * .1, v, 3) * 1.4;
    dot(c, iso(o, pk * 12, y + (k.isEven ? .15 : -.15), 0, sc), .9, al(K.white, 1 - k / 22));
  }
  final y = 1.5 + wv(t, v, 3) * 1.4;
  final f = iso(o, ph * 12, y, 0, sc);
  final lean = wv(t + .3, v, 4) * .5;
  final hd = f + Offset(math.sin(lean) * 18, -math.cos(lean) * 18);
  ln(c, f, hd, _w, 1.1);
  ci(c, hd + Offset(math.sin(lean) * 4, -math.cos(lean) * 4), 3.5, _w);
  final sw = math.sin(t * 8) * 4;
  ln(c, f, f + Offset(-3 + sw, 6), _w, .9);
  ln(c, f, f + Offset(3 - sw, 6), _w, .9);
  ln(c, ol(f, hd, .7), ol(f, hd, .7) + Offset(-6, -2 + sw), _w, .9);
  ln(c, ol(f, hd, .7), ol(f, hd, .7) + Offset(6, 2 - sw), _w, .9);
  _three(c, v, const Offset(100, 92));
}

// 11 Quake city: rub the ground; the towers sway, each its own way.
void _quake(Canvas c, Size s, OpV v, double t) {
  const g = 100.0;
  final boost = 1 + v.energy * 2;
  final hs = [40.0, 64, 30, 76, 50, 36];
  for (var k = 0; k < hs.length; k++) {
    final x0 = 8 + k * 24.0, w = 18.0, h = hs[k];
    final sway = wv(t, v, k * 2.0) * 10 * boost * (h / 76);
    pl(c, [Offset(x0, g), Offset(x0 + sway, g - h), Offset(x0 + w + sway, g - h), Offset(x0 + w, g)], _w, 1);
    for (var j = 1; j * 8 < h - 4; j++) {
      final f = j * 8 / h;
      for (var i = 0; i < 3; i++) {
        dot(c, Offset(x0 + 4 + i * 5 + sway * f, g - j * 8), .6, al(K.white, .45));
      }
    }
  }
  final shake = wv(t, v, 9) * 2 * boost;
  pl(c, [Offset(0, g + shake), Offset(52, g - shake), Offset(60, g + 4), Offset(66, g - 2), Offset(156, g + shake)], _dimW, .9);
  lb(c, 'FREQ ${d2(v[0])}', const Offset(6, 106), pc(v, 0), h: 5);
  lb(c, 'AMP ${d2(v[1])}', const Offset(56, 106), pc(v, 1), h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(104, 106), pc(v, 2), h: 5);
}

// 12 Tumbling satellite: attitude jitter over an orbit; spin it for frequency.
void _sat(Canvas c, Size s, OpV v, double t) {
  for (var k = 0; k < 16; k++) {
    dot(c, Offset((k * 47.0) % 156, (k * 31.0) % 116 + 2), .6, _dimW);
  }
  c.drawArc(const Rect.fromLTWH(-80, 80, 200, 140), -math.pi * .9, math.pi * .6, false, st(_w, 1));
  c.drawArc(const Rect.fromLTWH(-110, 50, 260, 200), -math.pi * .95, math.pi * .7, false, st(K.dim, .7));
  final o = Offset(96 + wv(t * .5, v, 7) * 6, 50 + wv(t * .5, v, 8) * 4);
  final a = wv(t, v, 1) * 1.4 + .3;
  c.save();
  c.translate(o.dx, o.dy);
  c.rotate(a);
  rc(c, const Rect.fromLTWH(-6, -6, 12, 12), _w, 1.1);
  for (final sx in [-1.0, 1.0]) {
    final r = Rect.fromLTWH(sx < 0 ? -32 : 8, -5, 24, 10);
    rc(c, r, pc(v, 0), .9);
    for (var k = 1; k < 4; k++) {
      ln(c, Offset(r.left + k * 6, r.top), Offset(r.left + k * 6, r.bottom), pc(v, 0, .5), .6);
    }
  }
  ln(c, const Offset(0, -6), const Offset(0, -14), _w, .9);
  ci(c, const Offset(0, -16), 2, _w, .9);
  c.restore();
  dl(c, o, o + Offset(math.cos(a - math.pi / 2), math.sin(a - math.pi / 2)) * 34, al(K.white, .3), 4, .5);
  rd(c, v, 0, 'SPIN', const Offset(6, 6), h: 14);
  lb(c, 'AMP ${d2(v[1])}', const Offset(150, 104), pc(v, 1), ax: 1, h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(150, 112), pc(v, 2), ax: 1, h: 5);
}

// 13 Hummingbird: hovering at the flower, never still.
void _bird(Canvas c, Size s, OpV v, double t) {
  const fl = Offset(128, 54);
  fn(c, 12, (u) => Offset(128 + math.sin(u * 3) * 4, 54 + u * 66), K.green, 1);
  for (var k = 0; k < 5; k++) {
    final a = k * 2 * math.pi / 5 + .3;
    c.drawOval(Rect.fromCenter(center: pol(fl, 7, a), width: 10, height: 10), st(K.red, .9));
  }
  dot(c, fl, 1.8, _w);
  final b = Offset(84 + wv(t, v, 1) * 14, 52 + wv(t, v, 2) * 10);
  final body = Path()
    ..moveTo(b.dx - 16, b.dy + 6)
    ..quadraticBezierTo(b.dx - 2, b.dy + 10, b.dx + 8, b.dy)
    ..quadraticBezierTo(b.dx - 2, b.dy - 6, b.dx - 16, b.dy + 6);
  c.drawPath(body, st(_w, 1.1));
  ci(c, b + const Offset(8, -2), 4, _w);
  dot(c, b + const Offset(9, -3), .9, _w);
  ln(c, b + const Offset(12, -2), b + const Offset(28, 0), _w, .9);
  final flap = (t * 30) % 1;
  for (var k = 0; k < 4; k++) {
    final a = -math.pi / 2 - .9 + k * .55 + flap * .2;
    ln(c, b + const Offset(-2, 0), pol(b + const Offset(-2, 0), 20, a), al(K.white, k == (flap * 4).floor() ? .9 : .2), .8);
  }
  pl(c, [b + const Offset(-16, 6), b + const Offset(-24, 10), b + const Offset(-22, 4)], _w, .9);
  _three(c, v, const Offset(6, 90));
}

// 14 Coffee cup from above: rub the table, the ripples tremble.
void _cup(Canvas c, Size s, OpV v, double t) {
  const o = Offset(70, 60);
  ci(c, o, 44, _w, 1.2);
  ci(c, o, 40, _dimW, .8);
  c.drawArc(Rect.fromCircle(center: o + const Offset(48, 0), radius: 10), -1.2, 2.4, false, st(_w, 1.2));
  final kick = 1 + v.energy * 2;
  for (var k = 0; k < 6; k++) {
    final base = 6.0 + k * 6;
    fn(c, 48, (u) {
      final a = u * 2 * math.pi;
      final r = base + wv(t - k * .08 + math.sin(a * 3) * .05, v, k.toDouble()) * 4 * kick;
      return pol(o, r, a);
    }, al(K.white, .9 - k * .12), .8);
  }
  rd(c, v, 1, 'AMP', const Offset(150, 6), h: 14, ax: 1);
  lb(c, 'FREQ ${d2(v[0])}', const Offset(150, 104), pc(v, 0), ax: 1, h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(150, 112), pc(v, 2), ax: 1, h: 5);
}

// 15 Scope: the signal itself on a graticule, frequency as the big number.
void _scope(Canvas c, Size s, OpV v, double t) {
  for (var i = 0; i <= 10; i++) {
    for (var j = 0; j <= 6; j++) {
      dot(c, Offset(8 + i * 14.0, 40 + j * 12.0), .5, K.dim);
    }
  }
  dl(c, const Offset(8, 76), const Offset(148, 76), K.dim, 2.8, .5);
  fn(c, 120, (u) => Offset(8 + u * 140, 76 + wv(t + (u - 1) * 2, v, 2) * 32), K.green, 1.1);
  final hz = .3 + 11.7 * v[0] * v[0];
  nm(c, hz.toStringAsFixed(1), const Offset(6, 6), 26, pc(v, 0));
  lb(c, 'HZ', const Offset(6, 34), K.grey, h: 4);
  lb(c, 'AMP ${d2(v[1])}', const Offset(150, 8), pc(v, 1), ax: 1, h: 5);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(150, 16), pc(v, 2), ax: 1, h: 5);
}

// 16 Scribble: draw any line; it comes back alive with your wiggle.
void _fitScribble(OpV v) {
  final tr = v.trail;
  var turns = 0;
  var minY = 1.0, maxY = 0.0;
  for (var i = 2; i < tr.length; i++) {
    final a = tr[i - 1].dy - tr[i - 2].dy, b = tr[i].dy - tr[i - 1].dy;
    if (a * b < 0) turns++;
    minY = math.min(minY, tr[i].dy);
    maxY = math.max(maxY, tr[i].dy);
  }
  v.set(0, math.sqrt(turns / 40));
  v.set(1, (maxY - minY) * 1.6);
  v.hot = -1;
}

void _scribble(Canvas c, Size s, OpV v, double t) {
  final pts = v.trail.length > 4
      ? [for (final q in v.trail) Offset(q.dx * s.width, q.dy * s.height)]
      : [for (var i = 0; i <= 60; i++) Offset(14 + i * 2.1 + math.sin(i * .35) * 10, 60 - math.cos(i * .35) * 18 + math.sin(i * .1) * 6)];
  pl(c, pts, al(K.white, .15), .8);
  final out = <Offset>[];
  for (var i = 0; i < pts.length; i++) {
    final a = pts[math.max(0, i - 1)], b = pts[math.min(pts.length - 1, i + 1)];
    final d = b - a;
    final n = d.distance == 0 ? Offset.zero : Offset(-d.dy, d.dx) / d.distance;
    out.add(pts[i] + n * wv(t + i * .03, v, 1) * 8);
  }
  pl(c, out, _w, 1.2);
  dot(c, out.last, 2, pc(v, 2));
  lb(c, 'DRAW A LINE', const Offset(150, 8), K.grey, ax: 1, h: 5);
  _three(c, v, const Offset(6, 94));
}

// 17 Jitter cubes: an isometric field of boxes, each shaking on its own seed.
void _cubes(Canvas c, Size s, OpV v, double t) {
  const o = Offset(78, 34);
  const sc = 9.0;
  for (var i = 0; i < 3; i++) {
    for (var j = 0; j < 3; j++) {
      final x = i * 2.6, y = j * 2.6;
      pl(c, [iso(o, x, y, 0, sc), iso(o, x + 1.6, y, 0, sc), iso(o, x + 1.6, y + 1.6, 0, sc), iso(o, x, y + 1.6, 0, sc)], K.dim, .7, true);
      final sd = (i * 3 + j).toDouble();
      final dx = wv(t, v, sd) * .8, dy = wv(t, v, sd + 20) * .8, dz = (wv(t, v, sd + 40) + v[1]) * 1.2;
      isoBox(c, o, x + dx, y + dy, math.max(0.0, dz), 1.6, 1.6, 1.6, sc, i == 1 && j == 1 ? K.white : al(K.white, .55), 1);
    }
  }
  lb(c, 'FREQ ${d2(v[0])}', const Offset(6, 8), pc(v, 0));
  lb(c, 'AMP', const Offset(150, 8), pc(v, 1), ax: 1);
  nm(c, d2(v[1]), const Offset(150, 18), 18, pc(v, 1), ax: 1);
  lb(c, 'SMTH ${d2(v[2])}', const Offset(6, 18), pc(v, 2));
}

// 18 Leaves in the wind: rub for a gust.
void _leaf(Canvas c, Size s, OpV v, double t) {
  fn(c, 20, (u) => Offset(u * 150, 20 + u * 30 + math.sin(u * 3) * 6), _w, 1.4);
  for (var k = 0; k < 6; k++) {
    final u = .15 + k * .14;
    final stem = Offset(u * 150, 20 + u * 30 + math.sin(u * 3) * 6);
    final a = math.pi / 2 + (k.isEven ? -.4 : .4) + wv(t, v, k.toDouble()) * 1.2;
    final tip = pol(stem, 26, a);
    final mid = ol(stem, tip, .5), n = Offset(-math.sin(a), math.cos(a)) * 7;
    final p = Path()
      ..moveTo(stem.dx, stem.dy)
      ..quadraticBezierTo(mid.dx + n.dx, mid.dy + n.dy, tip.dx, tip.dy)
      ..quadraticBezierTo(mid.dx - n.dx, mid.dy - n.dy, stem.dx, stem.dy);
    c.drawPath(p, st(K.green, 1));
    ln(c, stem, ol(stem, tip, .85), al(K.green, .5), .6);
  }
  for (var k = 0; k < 3; k++) {
    final x = (t * 40 * (.3 + v[0]) + k * 60) % 220 - 40;
    final y = 90 + k * 8.0;
    fn(c, 14, (u) => Offset(x + u * 34, y + math.sin(u * 6) * 2), al(K.white, .4), .8);
    c.drawArc(Rect.fromCircle(center: Offset(x + 38, y - 3), radius: 3), math.pi / 2, math.pi * 1.5, false, st(al(K.white, .4), .8));
  }
  _three(c, v, const Offset(100, 92));
}

// 19 Maraca monkey: rub faster, it shakes faster.
void _maraca(Canvas c, Size s, OpV v, double t) {
  const hd = Offset(78, 52);
  ci(c, hd, 14, _w, 1.1);
  ci(c, hd + const Offset(-16, -2), 5, _w);
  ci(c, hd + const Offset(16, -2), 5, _w);
  c.drawOval(Rect.fromCenter(center: hd + const Offset(0, 6), width: 16, height: 10), st(_w, .9));
  dot(c, hd + const Offset(-5, -4), 1.3, _w);
  dot(c, hd + const Offset(5, -4), 1.3, _w);
  c.drawArc(Rect.fromCircle(center: hd + const Offset(0, 6), radius: 4), .3, math.pi - .6, false, st(_w, .8));
  pl(c, [hd + const Offset(-8, 12), const Offset(66, 96), const Offset(90, 96), hd + const Offset(8, 12)], _w);
  for (final side in [-1.0, 1.0]) {
    final sh = hd + Offset(side * 10, 20);
    final ang = -math.pi / 2 + side * .9 + wv(t, v, side + 2) * .9;
    final hand = pol(sh, 22, ang + side * .3);
    pl(c, [sh, hand], _w, 1);
    final top = pol(hand, 14, ang);
    ln(c, hand, ol(hand, top, .4), pc(v, 0), 1);
    final mc = ol(hand, top, .8);
    c.save();
    c.translate(mc.dx, mc.dy);
    c.rotate(ang);
    c.drawOval(const Rect.fromLTWH(-6, -5, 14, 10), st(pc(v, 0), 1));
    for (var k = 0; k < 3; k++) {
      dot(c, Offset(-1.0 + k * 3, (k - 1) * 2.0 + wv(t + k, v, 9) * 2), .9, pc(v, 2));
    }
    c.restore();
    if (v[1] > .15) lb(c, 'CHK', pol(top, 12, ang) - const Offset(6, 3), al(K.white, v[1]), h: 4);
  }
  _three(c, v, const Offset(6, 96));
}

// 20 Static: the extreme, a whole TV of jitter and a channel number that can't hold still.
void _static(Canvas c, Size s, OpV v, double t) {
  final r = RRect.fromRectAndRadius(const Rect.fromLTWH(10, 18, 136, 88), const Radius.circular(10));
  c.drawRRect(r, st(_w, 1.2));
  ln(c, const Offset(66, 20), const Offset(52, 4), _w, .9);
  ln(c, const Offset(90, 20), const Offset(106, 4), _w, .9);
  c.save();
  c.clipRRect(r.deflate(4));
  final amp = .3 + v[1] * 2.7;
  for (var row = 0; row < 25; row++) {
    final y = 24 + row * 3.4;
    final tear = wv(t + row * .02, v, 3) * 26 * amp;
    final fr = (t * (4 + 40 * v[0])).floor();
    for (var k = 0; k < 9; k++) {
      final h = math.sin((fr * 13 + row * 7 + k) * 12.9898) * 43758.5453;
      final f = h - h.floorToDouble();
      final x = 14 + ((k * 17 + f * 30 + tear) % 130);
      ln(c, Offset(x, y), Offset(x + 3 + f * 10, y), al(K.white, .15 + .45 * f), .8);
    }
  }
  final j = Offset(wv(t, v, 7) * 10 * amp, wv(t, v, 8) * 6 * amp);
  nm(c, d2(v[1]), const Offset(78, 44) + j, 40, pc(v, 1), ax: .5);
  nm(c, d2(v[1]), const Offset(80, 44) - j * .5, 40, al(K.red, .7), ax: .5);
  c.restore();
  lb(c, 'CH', const Offset(14, 10), K.grey, h: 5);
  lb(c, 'FREQ ${d2(v[0])}  SMTH ${d2(v[2])}', const Offset(150, 114), _w, ax: 1, h: 4);
}

final wigglePanels = <OpPanel>[
  OpPanel('hand-held viewfinder', 'machine · effect · sensible', _cam),
  OpPanel('shivering chihuahua', 'animal · effect · playful', _dog, g: const OpRub(1, 0)),
  OpPanel('polygraph pens', 'machine · history · diagram', _poly),
  OpPanel('firefly trail', 'cosmic+insect · effect 2D', _firefly, g: const OpPinch(1, 0)),
  OpPanel('heat haze mesa', 'landscape · effect · drawing', _haze, g: const OpRub(1, 2)),
  OpPanel('BOIL line boil', 'typographic · effect · type', _boil, g: const OpDrag(0, 1)),
  OpPanel('S&H lane: rate depth slew', 'Max device · mechanism', _sh),
  OpPanel('spectrum hump', 'diagram · mechanism · numeral', _spec, g: const OpPinch(0, 2)),
  OpPanel('fishing bobber', 'landscape · effect · drawing', _bobber, g: const OpFlick(1, 0)),
  OpPanel('drunk walker', 'isometric+figure · effect', _drunk),
  OpPanel('quake city', 'landscape · effect · rub', _quake, g: const OpRub(1, 0)),
  OpPanel('tumbling satellite', 'cosmic · effect · drawing', _sat, g: const OpSpin(0, Offset(.62, .42), 1)),
  OpPanel('hummingbird hover', 'animal · effect · drawing', _bird),
  OpPanel('coffee cup from above', 'object · effect · rub table', _cup, g: const OpRub(1, 0)),
  OpPanel('scope trace', 'instrument · signal · numeral', _scope, g: const OpDrag(0, 1)),
  OpPanel('scribble comes alive', 'your line · effect · draw', _scribble, g: const OpDraw(_fitScribble)),
  OpPanel('jitter cubes', 'isometric · effect · field', _cubes, g: const OpPinch(1, 0)),
  OpPanel('leaves in the wind', 'landscape · effect · rub', _leaf, g: const OpRub(0, 1)),
  OpPanel('maraca monkey', 'character · effect · playful', _maraca, g: const OpRub(0, 1)),
  OpPanel('static TV', 'type+machine · 振り切れた', _static, g: const OpRub(1, 0), init: const [.6, .55, .1]),
];
