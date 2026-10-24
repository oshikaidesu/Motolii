// Part of op_01_glow.dart (cut there by size, as it was).
part of 'op_01.dart';

// 14 squint face: what glow does to a viewer. Brighter halo -> eyes close; past the threshold -> shades drop.
void _g14(Canvas c, _P p, double t) {
  const f = Offset(54, 66);
  _ring(c, f, 28, _white, 1.1);
  _ring(c, f + const Offset(-29, 0), 4, _white, 1);
  _ring(c, f + const Offset(29, 0), 4, _white, 1);
  final squint = _cl(p.b * 1.2 + math.sin(t * 3) * .03);
  for (final sx in [-10.0, 10.0]) {
    final eo = f + Offset(sx, -4);
    final open = 5 * (1 - squint);
    c.drawArc(Rect.fromCenter(center: eo, width: 11, height: open * 2 + .1), math.pi, math.pi, false, _s(_white, 1));
    _ln(c, eo - const Offset(5.5, 0), eo + const Offset(5.5, 0), _white, 1);
    if (open > 1.5) _dot(c, eo + Offset(0, -open * .4), 1.3, _white);
    _ln(c, eo + Offset(-5, -6 - squint * 3), eo + Offset(5, -6 + (sx < 0 ? 2 : -2) * squint), _white, 1);
  }
  final shades = p.b > p.c * .9 + .1;
  if (shades) {
    final y = f.dy - 6 + (1 - _ease(_cl((p.b - p.c * .9 - .1) * 6))) * -30;
    c.drawRRect(RRect.fromLTRBR(f.dx - 17, y - 4, f.dx - 3, y + 5, const Radius.circular(3)), _s(_red, 1.1));
    c.drawRRect(RRect.fromLTRBR(f.dx + 3, y - 4, f.dx + 17, y + 5, const Radius.circular(3)), _s(_red, 1.1));
    _ln(c, Offset(f.dx - 3, y - 1), Offset(f.dx + 3, y - 1), _red, 1.1);
  }
  c.drawArc(Rect.fromCenter(center: f + const Offset(0, 12), width: 14, height: 6 + p.b * 6), .2, math.pi - .4, false, _s(_white, 1));
  if (p.b > .6) {
    _pl(c, [f + const Offset(22, -14), f + const Offset(24, -9), f + const Offset(21, -8)], _blue, 1);
  }
  const sun = Offset(128, 24);
  _ring(c, sun, 6, _white, 1.1);
  final rl = 6 + p.a * 30;
  for (var k = 0; k < 10; k++) {
    final a = k * math.pi / 5 + t * .2;
    _ln(c, sun + Offset(math.cos(a), math.sin(a)) * 9, sun + Offset(math.cos(a), math.sin(a)) * (9 + rl * (k.isEven ? 1 : .6)), k.isEven ? _blue : _green, .9);
  }
  _num(c, _nn(p.b), Offset(_pw - 8, 84), 24, _green, 1);
}

// 15 draw the falloff: draw the glow profile with a finger; dotted verticals mark the samples.
void _g15(Canvas c, _P p, double t) {
  const x0 = 14.0, x1 = 146.0, y0 = 100.0, y1 = 20.0;
  _ln(c, const Offset(x0, y0), const Offset(x1, y0), _dim, 1);
  _ln(c, const Offset(x0, y0), const Offset(x0, y1), _dim, 1);
  final thy = y0 - (y0 - y1) * (.15 + p.c * .6);
  _dash(c, Offset(x0, thy), Offset(x1, thy), _red, on: 3, off: 2);
  _lb(c, 'THR', Offset(x1, thy - 9), _red, 6.5, 1);
  List<Offset> curve;
  if (p.pts.length > 3) {
    curve = [...p.pts]..sort((a, b) => a.dx.compareTo(b.dx));
  } else {
    final r = .08 + p.a * .5;
    curve = [for (var i = 0; i <= 40; i++) Offset(x0 + (x1 - x0) * i / 40, y0 - (y0 - y1) * (.3 + p.b * .7) * math.exp(-math.pow((i / 40) / r, 2)))];
  }
  for (var i = 0; i < curve.length - 1; i++) {
    _ln(c, curve[i], curve[i + 1], curve[i].dy < thy ? _green : _blue, 1.2);
  }
  for (var i = 0; i < curve.length; i += math.max(1, curve.length ~/ 6)) {
    _dotted(c, curve[i], Offset(curve[i].dx, y0), _dim);
    _dot(c, curve[i], 1.5, curve[i].dy < thy ? _green : _blue);
  }
  if (p.touch != null) _ring(c, p.touch!, 5, _white, .8);
  _num(c, _nn(p.a), Offset(x1, 6), 22, _blue, 1);
  _lb(c, 'DRAW', const Offset(20, 10), _white);
}

// 16 pixel bleed: a tiny sample in dots; pixels above threshold become sources and bleed into neighbours.
void _g16(Canvas c, _P p, double t) {
  const cols = 15, rows = 11, cs = 9.0, ox = 15.0, oy = 12.0;
  final src = <(int, int, double)>[(7, 5, 1), (6, 5, .8), (8, 5, .8), (7, 4, .8), (7, 6, .8), (3, 2, .55), (11, 8, .4), (12, 3, .7), (2, 8, .3)];
  for (var i = 0; i < cols; i++) {
    for (var j = 0; j < rows; j++) {
      final o = Offset(ox + i * cs, oy + j * cs);
      var g = 0.0;
      for (final (si, sj, b) in src) {
        if (b < p.c) continue;
        final d = math.sqrt(math.pow(i - si, 2) + math.pow(j - sj, 2));
        g = math.max(g, b * p.b * math.max(0, 1 - d / (.5 + p.a * 5)));
      }
      final isSrc = src.any((e) => e.$1 == i && e.$2 == j);
      if (isSrc) {
        final b = src.firstWhere((e) => e.$1 == i && e.$2 == j).$3;
        _dot(c, o, 2.2, b >= p.c ? _white : _red);
      } else if (g > .05) {
        _dot(c, o, .8 + g * 2.2, _a(g > .4 ? _green : _blue, .5 + g));
      } else {
        _dot(c, o, .6, _dim);
      }
    }
  }
  _lb(c, 'R ${_nn(p.a)}', const Offset(8, 110), _blue, 6.5);
  _lb(c, 'I ${_nn(p.b)}', const Offset(60, 110), _green, 6.5);
  _lb(c, 'T ${_nn(p.c)}', const Offset(112, 110), _red, 6.5);
}

// 17 cave lantern: the radius is how much of the cave you can see. Crystals answer only above the threshold.
void _g17(Canvas c, _P p, double t) {
  final walk = math.sin(t * 2.4);
  final f = Offset(56 + math.sin(t * .5) * 6, 98);
  final lamp = f + const Offset(12, -26);
  final rr = 14 + p.a * 70;
  List<Offset> wall(double y0, int s, double dir) => [for (var x = 0.0; x <= _pw; x += 8) Offset(x, y0 + dir * (_h(x.toInt(), s) * 14))];
  final top = wall(14, 1, 1), bot = wall(104, 2, -1);
  for (final w in [top, bot]) {
    for (var i = 0; i < w.length - 1; i++) {
      final m = Offset.lerp(w[i], w[i + 1], .5)!;
      final inside = (m - lamp).distance < rr;
      if (inside) {
        _ln(c, w[i], w[i + 1], _white, 1.1);
      } else {
        _dotted(c, w[i], w[i + 1], _dim, 4);
      }
    }
  }
  for (var i = 0; i < 6; i++) {
    final q = Offset(12 + _h(i, 5) * 132, i.isEven ? 26 + _h(i, 6) * 6 : 90 - _h(i, 6) * 6);
    if ((q - lamp).distance < rr && p.b > p.c * .8) {
      _pl(c, [q + const Offset(0, -4), q + const Offset(3, 0), q + const Offset(0, 4), q + const Offset(-3, 0)], _green, 1, true);
      _ln(c, q + const Offset(5, -5), q + const Offset(7, -7), _green, .8);
    }
  }
  _ring(c, lamp, rr, _a(_blue, .8), .8);
  final n = (p.b * 3).round();
  for (var k = 1; k <= n; k++) {
    _ring(c, lamp, rr * k / (n + 1), _a(_green, .35), .7);
  }
  _stick(c, f, 30, _white, arm: .45, step: walk);
  c.drawRect(Rect.fromCenter(center: lamp, width: 5, height: 6), _s(_white, 1));
  _dot(c, lamp, 1.2, _green);
  _lb(c, 'SEE ${_nn(p.a)}', const Offset(112, 108), _blue, 6.5);
}

// 18 planet air: spin the planet; atmosphere shells are the radius, stars brighter than threshold sparkle.
void _g18(Canvas c, _P p, double t) {
  const o = Offset(78, 60);
  for (var i = 0; i < 20; i++) {
    final s = Offset(_h(i, 7) * _pw, _h(i, 8) * _ph);
    if ((s - o).distance < 50) continue;
    final b = _h(i, 9);
    if (b > p.c) {
      _ln(c, s - const Offset(2.5, 0), s + const Offset(2.5, 0), _white, .7);
      _ln(c, s - const Offset(0, 2.5), s + const Offset(0, 2.5), _white, .7);
    } else {
      _dot(c, s, .7, _dim);
    }
  }
  _ring(c, o, 22, _white, 1.1);
  final rot = p.ang + t * .3;
  for (var k = 0; k < 5; k++) {
    final ph = (rot + k * math.pi / 5) % math.pi;
    final w = 44 * math.cos(ph).abs();
    if (w < 2) continue;
    c.drawArc(Rect.fromCenter(center: o, width: w, height: 44), -math.pi / 2, math.cos(ph) > 0 ? math.pi : -math.pi, false, _s(_a(_white, .45), .8));
  }
  for (final y in [-11.0, 0.0, 11.0]) {
    final hw = math.sqrt(484 - y * y);
    _ln(c, o + Offset(-hw, y), o + Offset(hw, y), _a(_white, .35), .7);
  }
  final n = 1 + (p.b * 3).round();
  for (var k = 1; k <= n; k++) {
    _ring(c, o, 22 + (4 + p.a * 30) * k / n, k == n ? _blue : _a(_green, .75), .8);
  }
  final ma = t * .8;
  final mo = o + Offset(math.cos(ma) * 58, math.sin(ma) * 20);
  _ring(c, mo, 3.5, _white, 1);
  _lb(c, 'AIR', const Offset(8, 8), _blue);
  _num(c, _nn(p.a), const Offset(8, 16), 22, _blue);
}

// 19 heat contours: isolines around hot spots; contour spacing = radius, count = intensity, red cut = threshold.
void _g19(Canvas c, _P p, double t) {
  for (final (a, b) in const [(Offset(8, 8), Offset(1, 1)), (Offset(148, 8), Offset(-1, 1)), (Offset(8, 112), Offset(1, -1)), (Offset(148, 112), Offset(-1, -1))]) {
    _ln(c, a, a + Offset(8 * b.dx, 0), _white, 1);
    _ln(c, a, a + Offset(0, 8 * b.dy), _white, 1);
  }
  final spots = [Offset(60 + math.sin(t * .4) * 4, 56), const Offset(108, 72)];
  final n = 2 + (p.b * 5).round();
  for (var si = 0; si < 2; si++) {
    final o = spots[si];
    for (var k = n; k >= 1; k--) {
      final lvl = 1 - k / (n + 1);
      final r = k * (3 + p.a * 6) * (si == 0 ? 1 : .7);
      final above = lvl > p.c;
      final path = Path();
      for (var i = 0; i <= 36; i++) {
        final a = i * math.pi / 18;
        final wob = 1 + .12 * math.sin(a * 3 + k + t * .6) + .06 * math.sin(a * 5 - k);
        final q = o + Offset(math.cos(a) * r * 1.3 * wob, math.sin(a) * r * wob);
        i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
      }
      c.drawPath(path, _s(above ? (k == 1 ? _white : _green) : _a(_blue, .55), above ? 1 : .7));
    }
    _ln(c, o - const Offset(3, 3), o + const Offset(3, 3), _red, 1);
    _ln(c, o - const Offset(3, -3), o + const Offset(3, -3), _red, 1);
  }
  _lb(c, 'TEMP', const Offset(20, 12), _white);
  _lb(c, 'CUT ${_nn(p.c)}', const Offset(20, 100), _red, 7);
  _num(c, _nn(p.b), const Offset(136, 14), 22, _green, 1);
}

// 20 supernova (pushed): rub until the drawing overloads; rays leave the frame, the frame itself strobes.
void _g20(Canvas c, _P p, double t) {
  const o = Offset(78, 60);
  final n = 20 + (p.b * 90).round(), len = 30 + p.a * 260;
  for (var i = 0; i < n; i++) {
    final a = i * 2 * math.pi / n + math.sin(t * 7 + i) * .01 * p.a;
    final l = len * (.4 + .6 * _h(i, 3)) * (.9 + .1 * math.sin(t * 9 + i));
    final col = [_blue, _green, _white, _purple][i % 4];
    _ln(c, o + Offset(math.cos(a), math.sin(a)) * 6, o + Offset(math.cos(a), math.sin(a)) * l, _a(col, .85), .7);
  }
  for (var k = 1; k <= 6; k++) {
    final r = ((t * 40 + k * 30) % 200);
    _ring(c, o, r, _a(_white, (1 - r / 200) * .6), .8);
  }
  _ring(c, o, 6, _white, 1.4);
  if (p.a > p.c) {
    _num(c, p.a > .95 ? '99' : _nn(p.a), Offset(o.dx, 6), 92, _a(_white, .9), .5);
  }
  if (p.a > .85 && (t * 8).floor().isEven) {
    c.drawRect(const Rect.fromLTWH(2, 2, _pw - 4, _ph - 4), _s(_red, 1.5));
    _lb(c, 'OVERLOAD', const Offset(8, 8), _red);
  }
}
