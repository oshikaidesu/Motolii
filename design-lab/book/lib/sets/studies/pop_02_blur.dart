part of 'pop_02.dart';

// Blur = "soften it / put it out of focus / smear it in a direction": every panel shows a picture getting soft, and why.

const _blur = <_Pn>[
  _Pn('Fogged window', 'weather·bold·2D·rub·result', _bl1, bg: _bl, seed: _swipe),
  _Pn('Slipping glasses', 'char·bold·2D·drag·result', _bl2, bg: _cy, p: Offset(70, 70)),
  _Pn('Bokeh street', 'machine·neon·2D·drag·aper', _bl3, a: .45, b: .9),
  _Pn('Zoomies', 'creature·bold·2D·throw·dir', _bl4, bg: _cr, p: Offset(122, 46)),
  _Pn('Watercolour bleed', 'material·crisp·2D·drag·result', _bl5, bg: _cr, a: .4, b: .4),
  _Pn('Ice cube', 'food·bold·iso·drag·result', _bl6, bg: _bl, a: .45),
  _Pn('Charcoal smudge', 'material·crisp·2D·rub·dir', _bl7, bg: _cr, seed: _smear),
  _Pn('Focus plane', 'machine·crisp·side·drag·result', _bl8, bg: _cr, b: .6, p: Offset(60, 60)),
  _Pn('Deep fish', 'nature·bold·2D·drag·result', _bl9, bg: _bl, p: Offset(80, 82)),
  _Pn('Mountain haze', 'landscape·bold·2D·drag·result', _bl10, bg: _ye, a: .55),
  _Pn('Sand heart', 'material·crisp·2D·drag·dir', _bl11, a: .4, b: .3),
  _Pn('Hyperspace', 'cosmic·wild·persp·drag·radial', _bl12, a: .5),
  _Pn('Spin the record', 'instr·bold·top·spin·rot', _bl13, bg: _ye),
  _Pn('Iris', 'machine·crisp·2D·drag·aper', _bl14, bg: _k2, a: .5),
  _Pn('Shy ghost', 'creature·neon·2D·drag·result', _bl15, bg: _vi, a: .35),
  _Pn('Noodle steam', 'food·bold·2D·drag·result', _bl16, bg: _cy, a: .55),
  _Pn('Lifted stencil', 'toy·bold·3D·drag·result', _bl17, bg: _cr, a: .45),
  _Pn('Big lamp soft shadow', 'physics·bold·side·drag·aper', _bl18, bg: _bl, a: .45),
  _Pn('Tilt-shift town', 'landscape·bold·iso·drag·result', _bl19, bg: _li, a: .35, p: Offset(78, 62)),
  _Pn('Comb the monster', 'creature·wild·2D·comb·dir', _bl20, bg: _pi, a: .55),
];

const _swipe = [
  Offset(30, 64), Offset(40, 74), Offset(52, 80), Offset(66, 82), Offset(80, 82), Offset(94, 80), Offset(106, 74), Offset(116, 64), //
];
const _smear = [Offset(40, 84), Offset(52, 76), Offset(64, 68), Offset(76, 60), Offset(88, 52), Offset(98, 46)];

void _city(Canvas c, double sg) {
  c.drawCircle(const Offset(118, 28), 14, _bf(_ye, sg));
  c.drawOval(const Rect.fromLTWH(20, 18, 40, 14), _bf(_cr, sg));
  const blds = [(8.0, 58.0, 22.0, _pi), (32.0, 44.0, 20.0, _or), (54.0, 66.0, 18.0, _cy), (74.0, 38.0, 24.0, _vi), (100.0, 56.0, 20.0, _re), (122.0, 48.0, 26.0, _li)];
  for (final (x, y, w, col) in blds) {
    c.drawRect(Rect.fromLTWH(x, y, w, 120 - y), _bf(col, sg));
    for (var j = 0; j < 4; j++) {
      c.drawRect(Rect.fromLTWH(x + 4, y + 6 + j * 12, 5, 5), _bf(_k1, sg));
      c.drawRect(Rect.fromLTWH(x + w - 9, y + 6 + j * 12, 5, 5), _bf(_ye, sg));
    }
  }
}

Path _blob(Offset o, double r, int seed) {
  final p = Path();
  for (var j = 0; j <= 16; j++) {
    final q = o + _pol(j * math.pi / 8, r * (1 + .14 * math.sin(j * 2.1 + seed)));
    j == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
  }
  return p..close();
}

Path _dabs(Iterable<Offset> pts, double r) {
  final path = Path();
  Offset? prev;
  for (final q in pts) {
    if (prev != null && (q - prev).distance < 40) {
      final n = ((q - prev).distance / 4).ceil();
      for (var i = 1; i < n; i++) {
        path.addOval(Rect.fromCircle(center: Offset.lerp(prev, q, i / n)!, radius: r));
      }
    }
    path.addOval(Rect.fromCircle(center: q, radius: r));
    prev = q;
  }
  return path;
}

void _bl1(Canvas c, _St s, double t) {
  _city(c, 5);
  c.drawRect(_full, _f(_al(_wh, .45)));
  c.save();
  c.clipPath(_dabs(s.pts, 10));
  c.drawRect(_full, _f(_bl));
  _city(c, 0);
  c.restore();
  for (var i = 0; i < 6; i++) {
    final x = _h(i) * 150, y = (_h(i + 9) * 120 + t * 8 * (1 + i % 3)) % 130;
    c.drawLine(Offset(x, y - 6), Offset(x, y), _s(_al(_wh, .7), 1.6));
  }
  c.drawRect(_full, _s(_cr, 8));
  c.drawLine(const Offset(78, 0), const Offset(78, 120), _s(_cr, 4));
  c.drawLine(const Offset(0, 56), const Offset(156, 56), _s(_cr, 4));
}

void _house(Canvas c, double sg) {
  c.drawCircle(const Offset(128, 22), 12, _bf(_ye, sg));
  c.drawRect(const Rect.fromLTWH(0, 92, _pw, 28), _bf(_li, sg));
  c.drawRect(const Rect.fromLTWH(26, 56, 46, 40), _bf(_cr, sg));
  c.drawPath(_poly(const [Offset(20, 58), Offset(49, 30), Offset(78, 58)]), _bf(_re, sg));
  c.drawRect(const Rect.fromLTWH(44, 72, 12, 24), _bf(_bl, sg));
  c.drawRect(const Rect.fromLTWH(31, 64, 9, 9), _bf(_ye, sg));
  c.drawRect(const Rect.fromLTWH(102, 70, 6, 26), _bf(_or, sg));
  c.drawCircle(const Offset(105, 60), 17, _bf(_dk(_li, .25), sg));
  c.drawCircle(const Offset(98, 56), 3, _bf(_pi, sg));
  c.drawCircle(const Offset(112, 64), 3, _bf(_pi, sg));
}

void _bl2(Canvas c, _St s, double t) {
  final sg = 2 + (s.p.dy / 120) * 5;
  _house(c, sg);
  final l = s.p + const Offset(-20, 0), r = s.p + const Offset(20, 0);
  c.save();
  c.clipPath(Path()
    ..addOval(Rect.fromCircle(center: l, radius: 16))
    ..addOval(Rect.fromCircle(center: r, radius: 16)));
  c.drawRect(_full, _f(_cy));
  _house(c, 0);
  c.restore();
  c.drawCircle(l, 16, _s(_k1, 4));
  c.drawCircle(r, 16, _s(_k1, 4));
  c.drawArc(Rect.fromCircle(center: s.p + const Offset(0, 2), radius: 5), math.pi, math.pi, false, _s(_k1, 3));
  c.drawLine(l + const Offset(-16, -2), l + const Offset(-30, -6), _s(_k1, 3));
  c.drawLine(r + const Offset(16, -2), r + const Offset(30, -6), _s(_k1, 3));
  c.drawArc(Rect.fromCircle(center: l + const Offset(-4, -5), radius: 9), -2.4, .9, false, _s(_al(_wh, .8), 2));
}

void _bl3(Canvas c, _St s, double t) {
  c.drawRect(const Rect.fromLTWH(0, 96, _pw, 24), _f(_k2));
  final r = 1.5 + s.a * 16, sides = s.b > .9 ? 0 : 3 + (s.b * 5.6).floor();
  for (var i = 0; i < 16; i++) {
    final q = Offset(8 + _h(i) * 140, 14 + _h(i + 40) * 74), col = _pop[i % 8];
    final flick = 1 + .06 * math.sin(t * 3 + i);
    final rr = r * (.7 + .5 * _h(i + 7)) * flick;
    final shape = sides == 0 ? (Path()..addOval(Rect.fromCircle(center: q, radius: rr))) : _poly([for (var k = 0; k < sides; k++) q + _pol(-math.pi / 2 + k * 2 * math.pi / sides, rr)]);
    c.drawPath(shape, _f(_al(col, s.a < .1 ? 1 : .45))..blendMode = BlendMode.plus);
    c.drawPath(shape, _s(_al(col, .9), 1));
  }
}

void _bl4(Canvas c, _St s, double t) {
  final v = s.p - _pc, d = _nrm(v), amt = (v.distance / 70).clamp(0.0, 1.2);
  for (var i = 0; i < 4; i++) {
    final off = Offset(-d.dy, d.dx) * (i * 6.0 - 9);
    c.drawLine(_pc + off - d * 18, _pc + off - d * (18 + 40 * amt * (.6 + _h(i) * .6)), _s(_al(_k1, .6), 2));
  }
  for (var i = 8; i >= 1; i--) {
    final q = _pc - d * (i * 4.0 * amt);
    c.drawOval(Rect.fromCenter(center: q, width: 34, height: 26), _f(_al(_pi, .12)));
  }
  c.save();
  c.translate(_pc.dx, _pc.dy);
  c.rotate(d.direction);
  for (var k = 0; k < 4; k++) {
    final a = t * 22 + k * math.pi / 2;
    c.drawLine(Offset(-6 + k * 4.0, 10), Offset(-6 + k * 4.0, 10) + _pol(a, 6), _s(_k1, 2));
  }
  c.drawOval(Rect.fromCenter(center: Offset.zero, width: 34, height: 26), _f(_pi));
  c.drawCircle(const Offset(8, -4), 5, _f(_wh));
  c.drawCircle(const Offset(10, -4), 2.4, _f(_k1));
  c.drawLine(const Offset(14, -11), const Offset(4, -12), _s(_k1, 2));
  c.restore();
  for (var i = 0; i < 3; i++) {
    final f = (t * 1.5 + i / 3) % 1, q = _pc - d * (24 + f * 30) + Offset(0, 10);
    c.drawCircle(q, 3 + f * 5, _f(_al(_or, .5 - f * .5)));
  }
}

void _bl5(Canvas c, _St s, double t) {
  final sg = s.a * 9, sp = 1 + s.b * .5;
  const blobs = [(Offset(56, 50), _cy), (Offset(96, 58), _pi), (Offset(72, 84), _ye)];
  for (var i = 0; i < 3; i++) {
    final (o, col) = blobs[i];
    final p = _blob(o, 20 * sp, i);
    c.drawPath(p, _bf(_al(col, .75), sg)..blendMode = BlendMode.multiply);
    c.drawPath(_blob(o, 20 * sp + sg * .8, i), _s(_al(_dk(col, .2), .55 * (1 - s.a * .6)), 1.2));
  }
  c.drawLine(const Offset(140, 8), const Offset(118, 30), _s(_or, 4));
  c.drawLine(const Offset(118, 30), const Offset(112, 37), _s(_k1, 5));
}

void _bl6(Canvas c, _St s, double t) {
  final L = 22 + s.a * 22, sg = s.a * 4;
  const o = Offset(78, 70);
  c.drawOval(Rect.fromCenter(center: o + Offset(0, L * .9), width: L * 2.6, height: L * .7), _f(_al(_k1, .35)));
  c.save();
  c.translate(o.dx, o.dy - L * .1);
  c.scale(1.5);
  c.drawPath(Path()..moveTo(0, -12)..cubicTo(-16, -22, -18, 6, 0, 14)..cubicTo(18, 6, 16, -22, 0, -12), _bf(_re, sg));
  for (var i = 0; i < 6; i++) {
    c.drawCircle(Offset(-6 + (i % 3) * 6.0, -4 + (i ~/ 3) * 8.0), 1.2, _bf(_ye, sg));
  }
  c.drawPath(_poly(const [Offset(-8, -16), Offset(0, -11), Offset(8, -16), Offset(0, -20)]), _bf(_li, sg));
  c.restore();
  final up = Offset(0, -L), ex = Offset(L * .9, L * .52), ey = Offset(-L * .9, L * .52);
  final top = o + up * .7 - (ex + ey) / 2;
  final A = top, B = top + ex, C = top + ex + ey, D = top + ey;
  final hgt = Offset(0, L * 1.1);
  c.drawPath(_poly([D, C, C + hgt, D + hgt]), _f(_al(_cy, .33)));
  c.drawPath(_poly([C, B, B + hgt, C + hgt]), _f(_al(_cy, .22)));
  c.drawPath(_poly([A, B, C, D]), _f(_al(_wh, .55)));
  for (final e in [
    [A, B], [B, C], [C, D], [D, A], [D, D + hgt], [C, C + hgt], [B, B + hgt], [D + hgt, C + hgt], [C + hgt, B + hgt], //
  ]) {
    c.drawLine(e[0], e[1], _s(_wh, 1.4));
  }
  for (var i = 0; i < 10; i++) {
    c.drawCircle(Offset.lerp(Offset.lerp(D, C, _h(i))!, Offset.lerp(D + hgt, C + hgt, _h(i))!, _h(i + 5))!, 1, _f(_al(_wh, .8)));
  }
}

void _sketch(Canvas c, Paint p) {
  c.drawPath(_star(const Offset(52, 56), 24, 5, .45), p);
  c.drawCircle(const Offset(110, 62), 20, p);
  c.drawPath(_poly(const [Offset(20, 100), Offset(40, 92), Offset(60, 104), Offset(80, 92), Offset(100, 104), Offset(120, 92), Offset(140, 100)])..fillType = PathFillType.nonZero, p);
}

void _bl7(Canvas c, _St s, double t) {
  _sketch(c, _s(_k1, 3.2));
  c.save();
  c.clipPath(_dabs(s.pts, 9));
  c.drawRect(_full, _f(_cr));
  for (var k = 1; k <= 6; k++) {
    _sketch(c, _s(_al(_k1, .26), 3.2)..maskFilter = const MaskFilter.blur(BlurStyle.normal, 2.4));
    c.translate(s.dir.dx * 2.4, s.dir.dy * 2.4);
  }
  c.restore();
  c.drawPath(_dabs(s.pts, 9), _f(_al(_k1, .06)));
}

void _bl8(Canvas c, _St s, double t) {
  c.drawRect(const Rect.fromLTWH(0, 100, _pw, 20), _f(_li));
  const trees = [(36.0, _or), (62.0, _pi), (90.0, _vi), (118.0, _bl), (144.0, _re)];
  final fx = s.p.dx.clamp(26.0, 152.0), ap = .02 + s.b * .16;
  c.drawLine(const Offset(14, 76), Offset(fx, 46), _s(_al(_k1, .25), 1));
  c.drawLine(const Offset(14, 76), Offset(fx, 106), _s(_al(_k1, .25), 1));
  for (final (x, col) in trees) {
    final sg = (x - fx).abs() * ap;
    c.drawRect(Rect.fromLTWH(x - 2, 88, 4, 14), _bf(_k1, sg));
    c.drawPath(_poly([Offset(x - 12, 92), Offset(x, 52), Offset(x + 12, 92)]), _bf(col, sg));
  }
  for (var y = 30.0; y < 112; y += 7) {
    c.drawLine(Offset(fx, y), Offset(fx, y + 3.5), _s(_or, 2));
  }
  c.drawPath(_poly([Offset(fx - 6, 22), Offset(fx + 6, 22), Offset(fx, 29)]), _f(_or));
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(2, 68, 16, 14), const Radius.circular(3)), _f(_k1));
  c.drawCircle(const Offset(16, 75), 4, _f(_cy));
}

void _bl9(Canvas c, _St s, double t) {
  for (var i = 0; i < 4; i++) {
    final x = 10 + i * 40.0 + math.sin(t + i) * 4;
    c.drawPath(_poly([Offset(x, 18), Offset(x + 16, 18), Offset(x + 34, 120), Offset(x + 14, 120)]), _f(_al(_cy, .12)));
  }
  final wave = Path()..moveTo(0, 0);
  for (var x = 0; x <= 156; x += 4) {
    wave.lineTo(x.toDouble(), 16 + math.sin(x * .12 + t * 3) * 2.5);
  }
  wave
    ..lineTo(156, 0)
    ..close();
  c.drawPath(wave, _f(_cy));
  final depth = _cl((s.p.dy - 22) / 90), sg = depth * 6, q = Offset(s.p.dx, s.p.dy.clamp(24.0, 116.0));
  final body = _mix(_or, _bl, depth * .65), fin = _mix(_ye, _bl, depth * .65);
  c.drawPath(_poly([q + const Offset(-14, 0), q + const Offset(-26, -9), q + const Offset(-26, 9)]), _bf(fin, sg));
  c.drawOval(Rect.fromCenter(center: q, width: 34, height: 20), _bf(body, sg));
  c.drawCircle(q + const Offset(9, -3), 3, _bf(_wh, sg));
  c.drawCircle(q + const Offset(10, -3), 1.5, _bf(_k1, sg));
  for (var i = 0; i < 4; i++) {
    final f = (t * .5 + i / 4) % 1, b = Offset.lerp(q + const Offset(18, -6), Offset(q.dx + 22, 18), f)!;
    c.drawCircle(b + Offset(math.sin(t * 5 + i) * 2, 0), 2 + f, _s(_al(_wh, .8), 1.2)..maskFilter = MaskFilter.blur(BlurStyle.normal, (1 - f) * sg * .6 + .01));
  }
}

void _bl10(Canvas c, _St s, double t) {
  c.drawCircle(const Offset(110, 30), 13, _f(_wh));
  const cols = [_pi, _or, _re, _vi, _k1];
  for (var k = 0; k < 5; k++) {
    final base = 46 + k * 14.0, p = Path()..moveTo(0, 120);
    for (var x = 0; x <= 156; x += 4) {
      p.lineTo(x.toDouble(), base - 14 * math.sin(x * (.03 + k * .008) + k * 1.7).abs() - 6 * math.sin(x * .11 + k));
    }
    p
      ..lineTo(156, 120)
      ..close();
    c.drawPath(p, _bf(cols[k], (4 - k) * s.a * 1.8));
    if (k < 4) c.drawRect(_full, _f(_al(_cr, s.a * .16)));
  }
}

List<Offset>? _heartPts;

void _bl11(Canvas c, _St s, double t) {
  _heartPts ??= () {
    final out = <Offset>[];
    for (var i = 0; out.length < 420; i++) {
      final x = _h(i * 2) * 2.6 - 1.3, y = _h(i * 2 + 1) * 2.6 - 1.25;
      final f = math.pow(x * x + y * y - 1, 3) - x * x * y * y * y;
      if (f <= 0) out.add(Offset(78 + x * 34, 64 - y * 34));
    }
    return out;
  }();
  final R = s.a * 16, wind = s.dir * s.b * 22;
  final ye = <Offset>[], or = <Offset>[];
  for (var i = 0; i < _heartPts!.length; i++) {
    final r1 = _h(i + 900), r2 = _h(i + 1900);
    final q = _heartPts![i] + _pol(r1 * math.pi * 2, R * math.sqrt(r2)) + wind * _h(i + 2900) + Offset(0, math.sin(t * 2 + i) * .3);
    (i.isEven ? ye : or).add(q);
  }
  c.drawPoints(ui.PointMode.points, ye, _s(_ye, 2.2));
  c.drawPoints(ui.PointMode.points, or, _s(_or, 2.2));
}

void _bl12(Canvas c, _St s, double t) {
  final sp = .15 + s.a * 1.4;
  for (var i = 0; i < 90; i++) {
    final dir = _pol(_h(i) * math.pi * 2, 1), f = (_h(i + 100) + t * .18 * sp) % 1;
    final r = 4 + f * f * 110, len = 1 + s.a * r * .7;
    final col = [_cr, _cy, _vi, _pi][i % 4];
    c.drawLine(s.p + dir * r, s.p + dir * (r + len), _s(_al(col, .3 + .7 * f), 1 + f * 1.4));
  }
  c.drawCircle(s.p, 2.5, _f(_wh));
}

void _bl13(Canvas c, _St s, double t) {
  const o = Offset(70, 60);
  c.drawCircle(o, 52, _f(_k1));
  for (var r = 24.0; r < 50; r += 3.5) {
    c.drawCircle(o, r, _s(_k2, 1));
  }
  final rot = s.ang + t * .3, spread = (s.w.abs() * .06).clamp(0.0, 2.6);
  const n = 7;
  for (var k = 0; k < n; k++) {
    final a = rot - spread * k / (n - 1);
    final o2 = spread < .02 ? 1.0 : 1.8 / n;
    for (var j = 0; j < 6; j++) {
      c.drawArc(Rect.fromCircle(center: o, radius: 20), a + j * math.pi / 3, math.pi / 3, true, _f(_al(_pop[j], o2)));
    }
    c.drawArc(Rect.fromCircle(center: o, radius: 40), a - .25, .5, false, _s(_al(_cr, o2 * .8), 2));
    if (spread < .02) break;
  }
  c.drawCircle(o, 3, _f(_cr));
  c.drawLine(const Offset(148, 12), const Offset(136, 76), _s(_cr, 3));
  c.drawLine(const Offset(136, 76), const Offset(118, 84), _s(_cr, 3));
  c.drawCircle(const Offset(148, 12), 5, _f(_k2));
}

void _bl14(Canvas c, _St s, double t) {
  final R = 12 + s.a * 40, b = 1.5 + s.a * 9;
  for (var i = 0; i < 12; i++) {
    final q = Offset(30 + _h(i) * 96, 20 + _h(i + 3) * 80);
    c.drawPath(_poly([for (var k = 0; k < 6; k++) q + _pol(k * math.pi / 3, b * (.7 + _h(i + 6) * .6))]), _f(_al(_pop[i % 8], .7))..blendMode = BlendMode.plus);
  }
  final rot = s.a * 1.4;
  final hole = [for (var k = 0; k < 6; k++) _pc + _pol(rot + k * math.pi / 3, R)];
  for (var k = 0; k < 6; k++) {
    final a = hole[k], b2 = hole[(k + 1) % 6], b3 = hole[(k + 2) % 6];
    final d1 = _nrm(b2 - a), d2 = _nrm(b3 - b2);
    c.drawPath(_poly([a - d1 * 220, a, b2, b2 - d2 * 220]), _f(k.isEven ? _vi : _bl));
    c.drawLine(a, a - d1 * 220, _s(_k1, 1.5));
  }
  c.drawPath(_poly(hole), _s(_cr, 1.5));
  c.drawCircle(_pc, 70, _s(_k1, 22));
}

void _bl15(Canvas c, _St s, double t) {
  for (var i = 0; i < 8; i++) {
    c.drawCircle(Offset(_h(i) * 156, _h(i + 20) * 50), 1.2, _f(_al(_wh, .7)));
  }
  final sg = s.a * 9, o = s.a * .65, q = _pc + Offset(0, math.sin(t * 2) * 5);
  final ghost = Path()
    ..moveTo(q.dx - 24, q.dy + 30)
    ..lineTo(q.dx - 24, q.dy - 6)
    ..arcToPoint(Offset(q.dx + 24, q.dy - 6), radius: const Radius.circular(24))
    ..lineTo(q.dx + 24, q.dy + 30);
  for (var k = 0; k < 4; k++) {
    final x = q.dx + 24 - k * 12.0, wv = math.sin(t * 6 + k) * 3;
    ghost.quadraticBezierTo(x - 6, q.dy + 24 + wv, x - 12, q.dy + 30);
  }
  ghost.close();
  c.drawPath(ghost, _bf(_al(_wh, 1 - o), sg));
  for (final dx in const [-9.0, 9.0]) {
    c.drawOval(Rect.fromCenter(center: q + Offset(dx, -6), width: 7, height: 10), _bf(_al(_k1, 1 - o), sg * .7));
  }
  c.drawOval(Rect.fromCenter(center: q + const Offset(0, 8), width: 8, height: 6), _bf(_al(_pi, 1 - o), sg * .7));
  c.drawOval(Rect.fromCenter(center: Offset(q.dx, 112), width: 40 - s.a * 10, height: 6), _f(_al(_k1, .25)));
}

void _bl16(Canvas c, _St s, double t) {
  final sg = s.a * 7;
  c.drawCircle(const Offset(78, 44), 26, _bf(_pi, sg));
  c.drawPath(_poly(const [Offset(56, 30), Offset(58, 8), Offset(72, 22)]), _bf(_pi, sg));
  c.drawPath(_poly(const [Offset(100, 30), Offset(98, 8), Offset(84, 22)]), _bf(_pi, sg));
  c.drawCircle(const Offset(68, 40), 3, _bf(_k1, sg));
  c.drawCircle(const Offset(88, 40), 3, _bf(_k1, sg));
  c.drawCircle(const Offset(78, 50), 2.5, _bf(_k1, sg));
  for (var k = 0; k < 4; k++) {
    final x = 52 + k * 17.0, p = Path()..moveTo(x, 84);
    for (var y = 84.0; y > 0; y -= 4) {
      p.lineTo(x + math.sin(y * .14 + t * 3 + k) * 5, y);
    }
    c.drawPath(p, _s(_al(_wh, .2 + .7 * s.a), 3 + s.a * 4));
  }
  c.drawLine(const Offset(104, 60), const Offset(140, 18), _s(_or, 3));
  c.drawLine(const Offset(110, 62), const Offset(148, 24), _s(_or, 3));
  final bowl = Path()
    ..moveTo(30, 82)
    ..arcToPoint(const Offset(126, 82), radius: const Radius.circular(48), clockwise: false)
    ..close();
  for (var k = 0; k < 5; k++) {
    final y = 78.0 - k * 2;
    c.drawPath(Path()..moveTo(38, y)..quadraticBezierTo(60, y - 10, 78, y)..quadraticBezierTo(98, y + 8, 118, y - 2), _s(_ye, 2.5));
  }
  c.drawCircle(const Offset(100, 76), 8, _f(_wh));
  c.drawCircle(const Offset(100, 76), 4, _f(_or));
  c.drawPath(bowl, _f(_re));
  c.drawLine(const Offset(28, 82), const Offset(128, 82), _s(_cr, 4));
  c.drawLine(const Offset(44, 96), const Offset(112, 96), _s(_al(_cr, .8), 2.5));
}

void _bl17(Canvas c, _St s, double t) {
  final h = s.a;
  c.drawPath(_star(const Offset(80, 66), 26, 5), _bf(_pi, .3 + h * 8));
  final off = Offset(-6 - h * 16, -6 - h * 22);
  final plate = Rect.fromCenter(center: const Offset(80, 66) + off, width: 74, height: 74);
  c.drawRect(plate.shift(Offset(4 + h * 8, 6 + h * 12)), _f(_al(_k1, .16)));
  c.drawPath(Path()..fillType = PathFillType.evenOdd..addRect(plate)..addPath(_star(const Offset(80, 66) + off, 26, 5), Offset.zero), _f(_vi));
  c.drawLine(plate.bottomRight, plate.bottomRight - off, _s(_al(_k1, .4), 1));
  c.drawLine(plate.topRight, plate.topRight - off, _s(_al(_k1, .2), 1));
  const can = Offset(140, 18);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: can, width: 12, height: 24), const Radius.circular(3)), _f(_k1));
  for (var i = 0; i < 9; i++) {
    final f = (t * 1.2 + _h(i)) % 1;
    c.drawCircle(Offset.lerp(can + const Offset(-6, 6), const Offset(80, 66) + off, f)! + Offset(0, (_h(i + 4) - .5) * 20 * f), 1.4, _f(_al(_pi, 1 - f)));
  }
}

void _bl18(Canvas c, _St s, double t) {
  c.drawRect(const Rect.fromLTWH(0, 90, _pw, 30), _f(_ye));
  final L = 2 + s.a * 18;
  const lamp = Offset(44, 18), ball = Offset(84, 58);
  final dir = _nrm(ball - lamp), hit = ball + dir * ((100 - ball.dy) / dir.dy);
  c.drawOval(Rect.fromCenter(center: hit, width: 34 + L * 2.4, height: 10 + L * .6), _bf(_al(_k1, .85 - s.a * .3), .5 + L * .9));
  c.drawLine(lamp + Offset(-L, 0), ball + const Offset(-12, 0), _s(_al(_ye, .3), 1));
  c.drawLine(lamp + Offset(L, 0), ball + const Offset(12, 0), _s(_al(_ye, .3), 1));
  c.drawCircle(ball, 13, _f(_cy));
  c.drawCircle(ball + const Offset(-4, -5), 4, _f(_al(_wh, .7)));
  c.drawLine(Offset(lamp.dx, 0), lamp - Offset(0, L), _s(_cr, 1.2));
  c.drawCircle(lamp, L + 3, _f(_al(_ye, .3)));
  c.drawCircle(lamp, L, _f(_wh));
}

void _town(Canvas c, double sg) {
  for (var sum = -6; sum <= 6; sum++) {
    for (var x = -3; x <= 3; x++) {
      final y = sum - x;
      if (y < -3 || y > 3) continue;
      final P = _iso(x.toDouble(), y.toDouble(), 11, 7, const Offset(78, 52)), r = _h(x * 7 + y * 13 + 50);
      if (r < .3) {
        c.drawCircle(P - const Offset(0, 6), 6, _bf(_dk(_li, .35), sg));
        continue;
      }
      final col = _pop[(r * 80).floor() % 8], h = 8 + r * 8, T = P - Offset(0, h);
      c.drawPath(_poly([T + const Offset(-8, 0), T + const Offset(0, 4.6), P + const Offset(0, 4.6), P + const Offset(-8, 0)]), _bf(_dk(col, .3), sg));
      c.drawPath(_poly([T + const Offset(0, 4.6), T + const Offset(8, 0), P + const Offset(8, 0), P + const Offset(0, 4.6)]), _bf(_dk(col, .12), sg));
      c.drawPath(_poly([T + const Offset(-8, 0), T + const Offset(0, -8), T + const Offset(8, 0), T + const Offset(0, 4.6)]), _bf(col, sg));
    }
  }
}

void _bl19(Canvas c, _St s, double t) {
  final w = 7 + s.a * 26;
  _town(c, 3.2);
  c.save();
  c.clipRect(Rect.fromLTRB(0, s.p.dy - w, _pw, s.p.dy + w));
  c.drawRect(_full, _f(_li));
  _town(c, 0);
  c.restore();
}

void _bl20(Canvas c, _St s, double t) {
  final len = 3 + s.a * 18, d = s.dir;
  for (var i = 0; i < 150; i++) {
    final ang = _h(i) * math.pi * 2, rr = 26 * math.sqrt(.35 + .65 * _h(i + 500));
    final root = _pc + _pol(ang, rr), rad = _pol(ang, 1);
    final dir = _nrm(rad * .45 + d * (1 + _h(i + 70) * .3)) * len * (.7 + .5 * _h(i + 30));
    final wob = Offset(-dir.dy, dir.dx) * .15 * math.sin(t * 4 + i);
    c.drawLine(root, root + dir + wob, _s(i % 3 == 0 ? _ye : _vi, 1.6));
  }
  c.drawCircle(_pc, 22, _f(_vi));
  for (final dx in const [-8.0, 8.0]) {
    c.drawCircle(_pc + Offset(dx, -6), 6.5, _f(_wh));
    c.drawCircle(_pc + Offset(dx, -6) + d * 3, 2.8, _f(_k1));
  }
  c.drawArc(Rect.fromCenter(center: _pc + const Offset(0, 6), width: 18, height: 12), .2, math.pi - .4, true, _f(_k1));
  c.drawPath(_poly([_pc + const Offset(-5, 7), _pc + const Offset(-2, 7), _pc + const Offset(-3.5, 10.5)]), _f(_wh));
  c.drawPath(_poly([_pc + const Offset(2, 7), _pc + const Offset(5, 7), _pc + const Offset(3.5, 10.5)]), _f(_wh));
}
