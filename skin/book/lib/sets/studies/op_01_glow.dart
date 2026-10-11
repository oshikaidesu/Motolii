part of 'op_01.dart';

// Glow: radius = blue, intensity = green, threshold = red. White is the lit subject.

const _glowSpecs = <_S>[
  _S('lighthouse', 'landscape|effect|drag|drawing|1', _G.drag, _g01, a: .5, b: .55, c: .5),
  _S('firefly jar', 'insect|effect|rub|drawing|2', _G.rub, _g02, a: .55, b: .5, c: .5),
  _S('neon tubes', 'typographic|effect|drag|drawing|2', _G.drag, _g03, a: .4, b: .6, c: .25),
  _S('three readouts', 'typographic|value|pinch|numeral|1', _G.pinch, _g04, a: .45, b: .34, c: .25),
  _S('screw-in bulb', 'object|mechanism|spin|drawing|2', _G.spin, _g05, a: .6, b: .5, c: .25),
  _S('signal flow', 'diagram|mechanism|drag|diagram|1', _G.drag, _g06, a: .5, b: .34, c: .5),
  _S('luma histogram', 'device|mechanism|drag|diagram|2', _G.drag, _g07, a: .45, b: .6, c: .5),
  _S('moon halo', 'cosmic|effect|flick|drawing|2', _G.flick, _g08, a: .5, b: .6, c: .5),
  _S('firework burst', 'cosmic|effect|flick|drawing|4', _G.flick, _g09, a: .6, b: .6, c: .5),
  _S('night drive', 'vehicle|effect|drag|drawing|2', _G.drag, _g10, a: .55, b: .5, c: .5),
  _S('light cube', 'isometric|mechanism|pinch|drawing|3', _G.pinch, _g11, a: .5, b: .5, c: .5),
  _S('angler fish', 'animal|reason|drag|drawing|3', _G.drag, _g12, a: .5, b: .6, c: .5),
  _S('scope bloom', 'instrument|effect|rub|drawing|3', _G.rub, _g13, a: .5, b: .5, c: .5),
  _S('squint face', 'character|reason|drag|drawing|3', _G.drag, _g14, a: .5, b: .55, c: .75),
  _S('draw falloff', 'diagram|mechanism|draw|diagram|2', _G.draw, _g15, a: .4, b: .7, c: .5),
  _S('pixel bleed', 'grid|effect on sample|drag|diagram|2', _G.drag, _g16, a: .45, b: .6, c: .5),
  _S('cave lantern', 'character|reason|drag|drawing|3', _G.drag, _g17, a: .45, b: .6, c: .5),
  _S('planet air', 'cosmic|mechanism|spin|drawing|2', _G.spin, _g18, a: .45, b: .5, c: .5),
  _S('heat contours', 'map|mechanism|pinch|diagram|3', _G.pinch, _g19, a: .5, b: .6, c: .5),
  _S('supernova', 'cosmic|pushed|rub|drawing|5', _G.rub, _g20, a: .7, b: .8, c: .25),
];

// 1 lighthouse: the beam length is the radius, the lines in the beam the intensity, boats are seen only above the threshold.
void _g01(Canvas c, _P p, double t) {
  const hy = 86.0;
  _ln(c, const Offset(0, hy), const Offset(_pw, hy), _dim, 1);
  for (var k = 0; k < 3; k++) {
    final y = hy + 8 + k * 9.0;
    _pl(c, [for (var x = 0.0; x <= _pw; x += 6) Offset(x, y + math.sin(x * .12 + t * 1.6 + k) * 1.4)], _a(_white, .35 - k * .08), 1);
  }
  const lx = 34.0;
  _pl(c, const [Offset(lx - 9, hy), Offset(lx - 5, 42), Offset(lx + 5, 42), Offset(lx + 9, hy)], _white);
  _ln(c, const Offset(lx - 7.5, 70), const Offset(lx + 7.5, 70), _white, 1);
  _ln(c, const Offset(lx - 6.4, 56), const Offset(lx + 6.4, 56), _white, 1);
  c.drawRect(const Rect.fromLTRB(lx - 5, 33, lx + 5, 42), _s(_white, 1));
  _pl(c, const [Offset(lx - 7, 33), Offset(lx, 26), Offset(lx + 7, 33)], _white, 1);
  const lamp = Offset(lx, 37.5);
  _dot(c, lamp, 1.8, _green);
  final len = 20 + p.a * 120, ang = math.sin(t * .7) * .25 + .12, half = .1 + p.b * .08;
  final e1 = lamp + Offset(math.cos(ang - half), math.sin(ang - half)) * len, e2 = lamp + Offset(math.cos(ang + half), math.sin(ang + half)) * len;
  _ln(c, lamp, e1, _blue);
  _ln(c, lamp, e2, _blue);
  final n = 1 + (p.b * 6).round();
  for (var i = 1; i <= n; i++) {
    final aa = ang - half + 2 * half * i / (n + 1);
    _dash(c, lamp, lamp + Offset(math.cos(aa), math.sin(aa)) * len * .95, _a(_green, .8), on: 3, off: 3, w: .8);
  }
  for (var i = 0; i < 3; i++) {
    final bx = 84.0 + i * 26, seen = (bx - lx) < len && p.b > p.c * .9 + i * .04;
    final col = seen ? _white : _dim;
    _pl(c, [Offset(bx - 6, hy - 2), Offset(bx + 6, hy - 2), Offset(bx + 4, hy + 1), Offset(bx - 4, hy + 1)], col, 1, true);
    _ln(c, Offset(bx, hy - 2), Offset(bx, hy - 10), col, 1);
    _pl(c, [Offset(bx, hy - 10), Offset(bx + 5, hy - 4), Offset(bx, hy - 4)], col, 1);
    if (seen) _dot(c, Offset(bx, hy - 11), 1.2, _green);
  }
  _ln(c, Offset(_pw - 6, 14), Offset(_pw - 6, 14 + 56 * (1 - p.c)), _dim, 1);
  _dot(c, Offset(_pw - 6, 14 + 56 * (1 - p.c)), 1.6, _red);
  _lb(c, 'RADIUS', const Offset(64, 8), _blue);
  _num(c, _nn(p.a), const Offset(64, 17), 24, _blue);
  _lb(c, 'THR', Offset(_pw - 10, 8), _red, 7, 1);
}

// 2 firefly jar: rub to wake them. Only flies brighter than the threshold get halos.
void _g02(Canvas c, _P p, double t) {
  final jar = RRect.fromLTRBR(46, 24, 110, 110, const Radius.circular(10));
  c.drawRRect(jar, _s(_white, 1.1));
  c.drawRect(const Rect.fromLTRB(52, 15, 104, 24), _s(_white, 1));
  for (var x = 56.0; x < 104; x += 6) {
    _ln(c, Offset(x, 15), Offset(x, 24), _dim, .8);
  }
  for (var i = 0; i < 9; i++) {
    final cx = 60 + _h(i) * 36 + math.sin(t * (.6 + _h(i, 2)) + i) * 6;
    final cy = 40 + _h(i, 1) * 58 + math.cos(t * (.5 + _h(i, 3)) + i * 2) * 5;
    final o = Offset(cx, cy);
    final g = p.a * (.45 + .55 * _h(i, 4)) * (.75 + .25 * math.sin(t * 3 + i));
    if (g > p.c * .8) {
      final rr = 3 + p.b * 9;
      final k = 1 + (g * 3).floor();
      for (var j = 1; j <= k; j++) {
        _ring(c, o, rr * j / k, j == k ? _blue : _a(_green, .9 - j * .15), .8);
      }
      _dot(c, o, 1.6, _green);
    } else {
      _dot(c, o, 1.1, _purple);
    }
  }
  final ty = 104 - p.c * 80;
  _dotted(c, const Offset(132, 24), const Offset(132, 104), _dim);
  _ln(c, Offset(127, ty), Offset(137, ty), _red, 1.2);
  _lb(c, 'THR', Offset(132, 10), _red, 7, .5);
  _lb(c, 'GLOW', const Offset(8, 10), _green);
  _num(c, _nn(p.a), const Offset(8, 19), 22, _green);
}

// 3 neon tubes: the word itself lights, halos are thin ring tubes, letters below the threshold flicker out.
void _g03(Canvas c, _P p, double t) {
  const word = ['G', 'L', 'O', 'W'];
  var x = 30.0;
  for (var i = 0; i < 4; i++) {
    final fl = .5 + .5 * math.sin(t * (7 + i * 3) + i * 11) * math.sin(t * 1.3 + i);
    final on = fl + p.b * .6 > p.c + .3;
    final col = on ? _white : _dim;
    final sz = _outline(c, word[i], Offset(x, 40), 34, col);
    if (on) {
      final ctr = Offset(x + sz.width / 2, 40 + sz.height * .55);
      final n = 1 + (p.b * 3).round();
      for (var k = 1; k <= n; k++) {
        final g = k * (1.5 + p.a * 4.5);
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: ctr, width: sz.width * .62 + g * 2, height: sz.height * .7 + g * 2), Radius.circular(6 + g)),
            _s(_a(k == n ? _blue : _green, .75 - k * .12), .8));
      }
    }
    x += sz.width + 4;
  }
  _ln(c, const Offset(20, 100), const Offset(136, 100), _dim, 1);
  for (final mx in [36.0, 78.0, 120.0]) {
    _ln(c, Offset(mx, 92), Offset(mx, 100), _dim, 1);
    _dot(c, Offset(mx, 92), 1.2, _white);
  }
  c.drawRect(const Rect.fromLTRB(68, 104, 88, 113), _s(_white, 1));
  _lb(c, '12KV', const Offset(92, 105), _red, 6.5);
  _lb(c, 'NEON', const Offset(8, 8), _white);
  _lb(c, _nn(p.a), Offset(_pw - 8, 8), _blue, 7.5, 1);
}

// 4 three readouts: OP-1 style PUNCH 45 / POWER 45; the numerals themselves bloom.
void _g04(Canvas c, _P p, double t) {
  final vals = [p.a, p.b, p.c], cols = [_blue, _green, _red], names = ['RADIUS', 'INTENS', 'THRESH'];
  for (var i = 0; i < 3; i++) {
    final x = 8.0 + i * 50;
    _lb(c, names[i], Offset(x, 14), cols[i]);
    final s = _nn(vals[i]);
    if (i == 1) {
      final n = (p.b * 4).round();
      for (var k = n; k >= 1; k--) {
        final d = k * (.7 + p.a * 2.2);
        for (final o in [Offset(-d, 0), Offset(d, 0), Offset(0, -d), Offset(0, d)]) {
          _num(c, s, Offset(x, 26) + o, 34, _a(_green, .16));
        }
      }
    }
    _num(c, s, Offset(x, 26), 34, cols[i]);
  }
  const g = Offset(30, 92);
  for (var k = 1; k <= 3; k++) {
    _ring(c, g, k * (2 + p.a * 4), _a(_blue, 1 - k * .2), .9);
  }
  const s2 = Offset(78, 92);
  final rays = 4 + (p.b * 10).round();
  for (var k = 0; k < rays; k++) {
    final a = k * 2 * math.pi / rays + t * .3;
    _ln(c, s2 + Offset(math.cos(a), math.sin(a)) * 4, s2 + Offset(math.cos(a), math.sin(a)) * (8 + p.b * 6), _green, .9);
  }
  final sy = 100 - p.c * 18;
  _pl(c, [const Offset(112, 100), Offset(124, 100), Offset(124, sy), Offset(144, sy)], _red, 1);
  _dotted(c, Offset(112, sy), Offset(124, sy), _dim);
}

// 5 screw-in bulb: spin to screw it in. Filament zigzag = intensity, rays = radius, rays appear only past the threshold.
void _g05(Canvas c, _P p, double t) {
  const o = Offset(70, 50);
  final path = Path()
    ..addArc(Rect.fromCircle(center: o, radius: 22), math.pi * .72, math.pi * 1.56)
    ..moveTo(o.dx - 13, o.dy + 18)
    ..lineTo(o.dx - 9, o.dy + 28)
    ..moveTo(o.dx + 13, o.dy + 18)
    ..lineTo(o.dx + 9, o.dy + 28);
  c.drawPath(path, _s(_white, 1.1));
  final sh = (p.ang * 2) % 4;
  for (var k = 0; k < 4; k++) {
    final y = o.dy + 30 + k * 4 + sh;
    if (y > o.dy + 46) continue;
    _ln(c, Offset(o.dx - 10, y), Offset(o.dx + 10, y - 2), _white, 1);
  }
  c.drawRect(Rect.fromLTRB(o.dx - 10, o.dy + 28, o.dx + 10, o.dy + 46), _s(_dim, .8));
  _ln(c, Offset(o.dx - 4, o.dy + 48), Offset(o.dx + 4, o.dy + 48), _white, 1.2);
  _ln(c, Offset(o.dx - 5, o.dy + 18), Offset(o.dx - 5, o.dy + 2), _dim, .8);
  _ln(c, Offset(o.dx + 5, o.dy + 18), Offset(o.dx + 5, o.dy + 2), _dim, .8);
  final amp = 2 + p.a * 6;
  _pl(c, [for (var i = 0; i <= 8; i++) Offset(o.dx - 5 + i * 10 / 8, o.dy + 2 + (i.isEven ? -amp : amp) * .5)], _green, 1);
  if (p.a > p.c * .7) {
    for (var k = 0; k < 12; k++) {
      final a = k * math.pi / 6 + math.sin(t * 2) * .03;
      if (math.sin(a) > .55) continue;
      _dash(c, o + Offset(math.cos(a), math.sin(a)) * 27, o + Offset(math.cos(a), math.sin(a)) * (30 + p.b * 30), _blue, on: 3, off: 2);
    }
  }
  _lb(c, 'TURNS', const Offset(112, 74), _green);
  _num(c, _nn(p.a), const Offset(112, 82), 24, _green);
  _lb(c, 'THR', const Offset(8, 104), _red);
  _ln(c, const Offset(26, 107), Offset(26 + p.c * 20, 107), _red, 1);
}

// 6 signal flow: source -> threshold gate -> amp 34 -> blur stack -> output wheel.
void _g06(Canvas c, _P p, double t) {
  const y = 52.0;
  c.drawRect(const Rect.fromLTWH(6, y - 11, 22, 22), _s(_white, 1));
  for (var k = 0; k < 4; k++) {
    final a = k * math.pi / 4;
    _ln(c, Offset(17 + math.cos(a) * 6, y + math.sin(a) * 6), Offset(17 - math.cos(a) * 6, y - math.sin(a) * 6), _white, .8);
  }
  c.drawRect(const Rect.fromLTWH(36, y - 11, 22, 22), _s(_red, 1));
  final gy = y + 7 - p.c * 14;
  _pl(c, [Offset(39, y + 7), Offset(47, y + 7), Offset(47, gy), Offset(55, gy)], _red, 1);
  _pl(c, const [Offset(66, y - 12), Offset(88, y), Offset(66, y + 12)], _green, 1.1, true);
  _lb(c, _nn(p.b), const Offset(68, y - 4), _green, 8);
  final n = 1 + (p.a * 4).round();
  for (var k = n - 1; k >= 0; k--) {
    c.drawRect(Rect.fromLTWH(96 + k * 2.5, y - 11 - k * 2.5, 20, 20), _s(k == 0 ? _blue : _a(_blue, .55), 1));
  }
  _pl(c, [for (var i = 0; i <= 12; i++) Offset(99 + i * 1.2, y + 1 - math.exp(-math.pow((i - 6) / (1 + p.a * 3), 2)) * 7)], _blue, .9);
  const w = Offset(138, y);
  _ring(c, w, 11, _white, 1);
  for (var k = 0; k < 6; k++) {
    final a = k * math.pi / 3 + t * .4;
    _dot(c, w + Offset(math.cos(a), math.sin(a)) * 6.5, 1.3, k < 1 + p.b * 5 ? _green : _dim);
  }
  for (final (a, b) in const [(28.0, 36.0), (58.0, 66.0), (88.0, 96.0), (118.0, 127.0)]) {
    _ln(c, Offset(a, y), Offset(b, y), _dim, 1);
  }
  final px = 6 + ((t * 40) % 130);
  _dot(c, Offset(px, y + 18), 1.4, _white);
  _dotted(c, const Offset(6, y + 18), const Offset(146, y + 18), _dim, 4);
  for (final (s, x, col) in const [('SRC', 17.0, _white), ('THR', 47.0, _red), ('AMT', 75.0, _green), ('RADIUS', 106.0, _blue), ('OUT', 138.0, _white)]) {
    _lb(c, s, Offset(x, y + 25), col, 6.5, .5);
  }
}

// 7 luma histogram (Max device): the part above the threshold is lifted and widened by the kernel.
void _g07(Canvas c, _P p, double t) {
  const x0 = 10.0, x1 = 146.0, base = 94.0;
  _ln(c, const Offset(x0, base), const Offset(x1, base), _dim, 1);
  double hist(double u) => 26 * math.exp(-math.pow((u - .3) / .12, 2)) + 18 * math.exp(-math.pow((u - .72 + math.sin(t * .5) * .02) / .07, 2)) + 4;
  final thx = x0 + (x1 - x0) * (.2 + p.c * .7);
  _pl(c, [for (var i = 0; i <= 60; i++) Offset(x0 + (x1 - x0) * i / 60, base - hist(i / 60))], _white, 1);
  final up = <Offset>[];
  for (var i = 0; i <= 60; i++) {
    final x = x0 + (x1 - x0) * i / 60;
    if (x < thx) continue;
    up.add(Offset(x, base - hist(i / 60) * (1 + p.b * 1.6) - 4));
  }
  _pl(c, up, _green, 1);
  for (var i = 0; i < up.length; i += 6) {
    _dotted(c, up[i], Offset(up[i].dx, base), _a(_green, .6));
  }
  _ln(c, Offset(thx, 18), Offset(thx, base + 4), _red, 1);
  _lb(c, 'THR', Offset(thx + 3, 18), _red, 6.5);
  final kw = 4 + p.a * 22;
  _pl(c, [for (var i = -20; i <= 20; i++) Offset(thx + 22 + i * kw / 10, base + 18 - 10 * math.exp(-math.pow(i / 10, 2) * 2))], _blue, 1);
  _dotted(c, Offset(thx + 22 - kw, base + 4), Offset(thx + 22 - kw, base + 20), _blue);
  _dotted(c, Offset(thx + 22 + kw, base + 4), Offset(thx + 22 + kw, base + 20), _blue);
  _lb(c, 'LUMA', const Offset(x0, 8), _white);
  _num(c, _nn(p.b), Offset(x1, 6), 22, _green, 1);
}

// 8 moon halo: flick the halo, it swells and settles; stars near the moon wash out.
void _g08(Canvas c, _P p, double t) {
  const m = Offset(64, 50);
  final path = Path.combine(PathOperation.difference, Path()..addOval(Rect.fromCircle(center: m, radius: 14)),
      Path()..addOval(Rect.fromCircle(center: m + const Offset(7, -4), radius: 12)));
  c.drawPath(path, _s(_white, 1.1));
  final hr = 18 + p.a * 44, n = 1 + (p.b * 4).round();
  for (var k = 1; k <= n; k++) {
    _ring(c, m, 16 + (hr - 16) * k / n, _a(k == n ? _blue : _green, 1 - k * .15), .8);
  }
  for (var i = 0; i < 22; i++) {
    final s = Offset(_h(i) * _pw, _h(i, 1) * 88);
    final d = (s - m).distance;
    if (d < hr * (.6 + p.c * .8)) continue;
    final tw = .5 + .5 * math.sin(t * 2 + i);
    _ln(c, s - Offset(1.5 + tw, 0), s + Offset(1.5 + tw, 0), _white, .8);
    _ln(c, s - Offset(0, 1.5 + tw), s + Offset(0, 1.5 + tw), _white, .8);
  }
  for (var k = 0; k < 2; k++) {
    final cx = ((t * (8 + k * 5) + k * 80) % 220) - 40, cy = 78.0 + k * 14;
    final pth = Path()..moveTo(cx, cy);
    for (var j = 0; j < 4; j++) {
      pth.arcToPoint(Offset(cx + (j + 1) * 9, cy), radius: Radius.circular(5 + (j % 2) * 2));
    }
    pth.lineTo(cx, cy);
    c.drawPath(pth, _s(_a(_white, .7), 1));
  }
  _ln(c, const Offset(0, 106), const Offset(_pw, 106), _dim, 1);
  _lb(c, 'HALO', const Offset(118, 10), _blue);
  _num(c, _nn(p.a), const Offset(118, 19), 22, _blue);
}

// 9 firework: flick to launch; spray length = radius, spark count = intensity, dim sparks fall red.
void _g09(Canvas c, _P p, double t) {
  final cyc = (t % 2.6) / 2.6;
  const o = Offset(78, 46);
  if (cyc < .25) {
    final y = 112 - (112 - o.dy) * _ease(cyc / .25);
    _dash(c, Offset(o.dx, 112), Offset(o.dx, y), _a(_white, .6), on: 2, off: 4);
    _dot(c, Offset(o.dx, y), 1.6, _white);
  } else {
    final k = _ease((cyc - .25) / .75), n = 8 + (p.b * 28).round(), len = (10 + p.a * 60) * k;
    for (var i = 0; i < n; i++) {
      final a = i * 2 * math.pi / n + _h(i) * .2;
      final br = _h(i, 3);
      final u = Offset(math.cos(a), math.sin(a));
      if (br < p.c * .8) {
        final q = o + u * len * .6 + Offset(0, k * k * 26);
        _ln(c, q - const Offset(1.5, 1.5), q + const Offset(1.5, 1.5), _red, .9);
        _ln(c, q - const Offset(1.5, -1.5), q + const Offset(1.5, -1.5), _red, .9);
      } else {
        final g = Offset(0, k * k * 10);
        _ln(c, o + u * len * .35 + g * .3, o + u * len + g, _a(i.isEven ? _green : _blue, 1 - k * .5), 1);
        _dot(c, o + u * len + g, 1.2, _white);
      }
    }
  }
  _ln(c, const Offset(0, 112), const Offset(_pw, 112), _dim, 1);
  _num(c, _nn(p.a), const Offset(8, 8), 30, _a(_blue, .8));
  _lb(c, 'SPRAY', const Offset(9, 40), _blue);
}

// 10 night drive: headlight reach is the radius, the signs reflect only past the threshold.
void _g10(Canvas c, _P p, double t) {
  const road = 92.0;
  _ln(c, const Offset(0, road), const Offset(_pw, road), _white, 1);
  final sh = (t * 30) % 20;
  for (var x = -sh; x < _pw; x += 20) {
    _ln(c, Offset(x, road + 8), Offset(x + 8, road + 8), _dim, 1);
  }
  final car = [const Offset(6, 88), const Offset(6, 80), const Offset(14, 78), const Offset(22, 70), const Offset(40, 70), const Offset(50, 78), const Offset(60, 80), const Offset(60, 88)];
  _pl(c, car, _white, 1.1, true);
  _ring(c, const Offset(18, 89), 4, _white, 1);
  _ring(c, const Offset(48, 89), 4, _white, 1);
  _pl(c, const [Offset(24, 77), Offset(27, 72), Offset(37, 72), Offset(40, 77)], _dim, .8, true);
  const hl = Offset(60, 82);
  final len = 16 + p.a * 96, sp = .06 + p.b * .12;
  _ln(c, hl, hl + Offset(math.cos(-sp), math.sin(-sp)) * len, _blue);
  _ln(c, hl, hl + Offset(math.cos(sp * .6), math.sin(sp * .6)) * len, _blue);
  for (var i = 0; i < 1 + (p.b * 4).round(); i++) {
    final a = -sp + (sp * 1.6) * (i + 1) / (2 + p.b * 4);
    _dash(c, hl, hl + Offset(math.cos(a), math.sin(a)) * len * .9, _green, on: 4, off: 3, w: .7);
  }
  for (var i = 0; i < 3; i++) {
    final sx = 88.0 + i * 24 + (i == 2 ? 0 : 0);
    final lit = sx - hl.dx < len && p.b + .2 > p.c + i * .1;
    final col = lit ? _white : _dim;
    _ln(c, Offset(sx, road), Offset(sx, road - 18), col, 1);
    _pl(c, [Offset(sx, road - 30), Offset(sx + 6, road - 24), Offset(sx, road - 18), Offset(sx - 6, road - 24)], col, 1, true);
    if (lit) {
      for (final a in [-2.2, -1.6, -1.0]) {
        _ln(c, Offset(sx, road - 24) + Offset(math.cos(a), math.sin(a)) * 9, Offset(sx, road - 24) + Offset(math.cos(a), math.sin(a)) * 13, _green, .9);
      }
    }
  }
  _lb(c, 'REACH', const Offset(8, 10), _blue);
  _num(c, _nn(p.a), const Offset(8, 19), 24, _blue);
  _lb(c, 'REFL ${_nn(p.c)}', Offset(_pw - 8, 10), _red, 7, 1);
}

// 11 light cube: nested wire cubes are the halo, floor tiles light within the radius above the threshold.
void _g11(Canvas c, _P p, double t) {
  const o = Offset(78, 38);
  _isoGrid(c, o, 6, 10, _dim);
  for (var i = 0; i < 7; i++) {
    for (var j = 0; j < 7; j++) {
      final d = math.sqrt(math.pow(i - 3, 2) + math.pow(j - 3, 2));
      final lit = d < 1 + p.a * 3.5 && (1 - d / 5) * p.b * 1.4 > p.c * .6;
      if (lit) _dot(c, _iso(o, i * 10.0, j * 10.0, 0), 1.3, d < 1.5 ? _white : _green);
    }
  }
  final n = 1 + (p.b * 3).round();
  final br = math.sin(t * 1.5) * .6;
  for (var k = n; k >= 0; k--) {
    final s = 8 + k * (3 + p.a * 5) + (k > 0 ? br : 0);
    _isoBox(c, o, 30 - s / 2, 30 - s / 2, 6, s, s, s, k == 0 ? _white : (k == n ? _blue : _a(_green, .7)), k == 0 ? 1.1 : .8);
  }
  _lb(c, 'X', const Offset(14, 54), _white, 7);
  _lb(c, 'Y', const Offset(34, 30), _white, 7);
  _lb(c, 'Z', const Offset(136, 30), _red, 7);
  _num(c, _nn(p.a), Offset(_pw - 8, 86), 22, _blue, 1);
}

// 12 angler fish: the lure's glow is what pulls the small fish in. Too dim (below threshold) and they ignore it.
void _g12(Canvas c, _P p, double t) {
  const e = Offset(46, 70);
  final body = Path()
    ..moveTo(e.dx + 30, e.dy - 4)
    ..quadraticBezierTo(e.dx + 10, e.dy - 34, e.dx - 30, e.dy - 14)
    ..quadraticBezierTo(e.dx - 40, e.dy, e.dx - 30, e.dy + 16)
    ..quadraticBezierTo(e.dx + 6, e.dy + 30, e.dx + 30, e.dy + 6);
  c.drawPath(body, _s(_white, 1.1));
  _pl(c, [for (var i = 0; i <= 8; i++) Offset(e.dx + 30 - i * 3, e.dy - 4 + (i.isEven ? 0 : 3.5))], _white, .9);
  _pl(c, [for (var i = 0; i <= 8; i++) Offset(e.dx + 30 - i * 3, e.dy + 6 - (i.isEven ? 0 : 3.5))], _white, .9);
  _pl(c, [Offset(e.dx - 34, e.dy - 2), Offset(e.dx - 46, e.dy - 12), Offset(e.dx - 44, e.dy + 2), Offset(e.dx - 46, e.dy + 14), Offset(e.dx - 34, e.dy + 6)], _white, 1);
  _ring(c, e + const Offset(8, -12), 4, _white, 1);
  _dot(c, e + const Offset(9, -12), 1.4, _white);
  final lure = e + Offset(40 + math.sin(t) * 2, -40 + math.cos(t * 1.3) * 2);
  final stalk = Path()
    ..moveTo(e.dx - 4, e.dy - 24)
    ..quadraticBezierTo(e.dx + 12, e.dy - 62, lure.dx, lure.dy);
  c.drawPath(stalk, _s(_white, 1));
  _dot(c, lure, 2, _green);
  final rr = 6 + p.a * 30, n = 1 + (p.b * 3).round();
  for (var k = 1; k <= n; k++) {
    _ring(c, lure, rr * k / n, k == n ? _blue : _a(_green, .7), .8);
  }
  final pull = p.b > p.c * .9;
  for (var i = 0; i < 4; i++) {
    final home = Offset(122 + _h(i) * 28, 20 + _h(i, 1) * 80);
    final ph = (t * .4 + _h(i, 2)) % 1;
    final pos = pull ? Offset.lerp(home, lure + Offset(rr * .7 + 4, (i - 1.5) * 6), _ease(ph))! : home + Offset(math.sin(t + i) * 6, math.cos(t * .7 + i) * 4);
    final dir = pull ? -1.0 : (math.cos(t + i) > 0 ? 1.0 : -1.0);
    _pl(c, [pos + Offset(-5 * dir, -3), pos, pos + Offset(-5 * dir, 3)], pull ? _white : _dim, 1);
    _ln(c, pos + Offset(-5 * dir, 0), pos + Offset(-9 * dir, 0), pull ? _white : _dim, 1);
  }
  _lb(c, 'LURE', const Offset(8, 8), _green);
  _num(c, _nn(p.b), const Offset(8, 16), 22, _green);
}

// 13 scope bloom: rub the CRT. Peaks above the red line bloom into parallel traces.
void _g13(Canvas c, _P p, double t) {
  final scr = RRect.fromLTRBR(8, 8, 148, 112, const Radius.circular(8));
  c.drawRRect(scr, _s(_white, 1));
  for (var x = 22.0; x < 148; x += 14) {
    _dotted(c, Offset(x, 14), Offset(x, 106), _a(_dim, .9), 5);
  }
  for (var y = 22.0; y < 110; y += 14) {
    _dotted(c, Offset(14, y), Offset(142, y), _a(_dim, .9), 5);
  }
  double f(double x) => 60 - 26 * math.sin(x * .09 + t * 2) * (.6 + .4 * math.sin(x * .023 - t * .7));
  final ty = 60 - 26 * (1 - p.c) * .9;
  _ln(c, Offset(14, ty), Offset(142, ty), _red, .9);
  final n = (p.a * 5).round();
  for (var k = n; k >= 1; k--) {
    final segs = <Offset>[];
    for (var x = 14.0; x <= 142; x += 2) {
      final y = f(x);
      if (y < ty) {
        segs.add(Offset(x, y - k * (1 + p.b * 2.5)));
      } else if (segs.isNotEmpty) {
        _pl(c, segs, _a(k == n ? _blue : _green, .85 - k * .12), .8);
        segs.clear();
      }
    }
    _pl(c, segs, _a(k == n ? _blue : _green, .85 - k * .12), .8);
  }
  _pl(c, [for (var x = 14.0; x <= 142; x += 2) Offset(x, f(x))], _white, 1.1);
  _lb(c, 'BLOOM ${_nn(p.a)}', const Offset(16, 98), _green, 7);
}

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
