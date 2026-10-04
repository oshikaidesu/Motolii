part of 'pop_02.dart';

// Falloff = "the effect is strong here and fades away over there": every panel shows WHAT gets less affected with distance.

const _falloff = <_Pn>[
  _Pn('Flashlight', 'machine·bold·2D·drag·result', _fl1, a: .45, b: .4, p: Offset(60, 66)),
  _Pn('Pond stone', 'nature·crisp·top·drag·mech', _fl2, bg: _k2, a: .55, b: .5),
  _Pn('Campfire friends', 'creature·bold·top·pull·result', _fl3, p: Offset(122, 60)),
  _Pn('Sprinkler lawn', 'landscape·bold·iso·drag·result', _fl4, bg: _k2, a: .5, b: .45),
  _Pn('Spray can', 'material·crisp·2D·paint·result', _fl5, bg: _cr, a: .45, b: .55, seed: _wave),
  _Pn('Magnet filings', 'physics·crisp·2D·pull·mech', _fl6, p: Offset(112, 60)),
  _Pn('Gravity well', 'cosmic·neon·3D·drag·mech', _fl7, a: .6, b: .5),
  _Pn('Hot plate eggs', 'food·bold·top·drag·result', _fl8, bg: _k2, a: .5, b: .55),
  _Pn('Skyline', 'landscape·bold·iso·drag·result', _fl9, a: .5, b: .5),
  _Pn('ASCII bloom', 'text-art·crisp·2D·drag·result', _fl10, a: .5, b: .5),
  _Pn('Snow melt', 'weather·bold·top·carry·result', _fl11, bg: _li, a: .5, b: .5, p: Offset(70, 54)),
  _Pn('Jelly pull', 'food·bold·2D·pull·result', _fl12, bg: _k2, a: .5, b: .5),
  _Pn('Cat in the crowd', 'creature·bold·top·drag·result', _fl13, bg: _bl, a: .45, b: .5, p: Offset(70, 58)),
  _Pn('Black hole', 'cosmic·neon·3D·drag·result', _fl14, a: .5, b: .5),
  _Pn('Dominoes', 'toy·crisp·3D·drag·result', _fl15, bg: _cr, a: .5, b: .5, p: Offset(60, 60)),
  _Pn('Wind chimes', 'instr·bold·2D·drag·result', _fl16, bg: _k2, a: .4, b: .5, p: Offset(56, 100)),
  _Pn('Volcano', 'landscape·bold·side·drag·shape', _fl17, bg: _bl, a: .45, b: .5),
  _Pn('Disco floor', 'machine·wild·persp·drag·result', _fl18, a: .5, b: .5, p: Offset(84, 86)),
  _Pn('Push field', 'physics·crisp·2D·drag·mech', _fl19, bg: _k2, a: .5, b: .5),
  _Pn('Pixel blast', 'toy·wild·2D·drag·result', _fl20, a: .55, b: .45, p: Offset(70, 56)),
];

const _wave = [
  Offset(18, 70), Offset(26, 58), Offset(36, 50), Offset(48, 50), Offset(58, 60), Offset(66, 72), Offset(76, 80), Offset(88, 78), //
  Offset(98, 66), Offset(106, 54), Offset(116, 46), Offset(128, 48), Offset(138, 58),
];

void _fl1(Canvas c, _St s, double t) {
  c.drawRect(const Rect.fromLTWH(0, 0, _pw, 86), _f(_bl));
  c.drawRect(const Rect.fromLTWH(0, 86, _pw, 34), _f(_vi));
  c.drawRect(const Rect.fromLTWH(16, 18, 38, 30), _f(_ye));
  c.drawRect(const Rect.fromLTWH(21, 23, 28, 20), _f(_cy));
  c.drawPath(_poly(const [Offset(21, 43), Offset(31, 30), Offset(38, 38), Offset(42, 33), Offset(49, 43)]), _f(_pi));
  c.drawRect(const Rect.fromLTWH(110, 70, 20, 18), _f(_or));
  for (var i = 0; i < 5; i++) {
    c.save();
    c.translate(120, 70);
    c.rotate(-1.2 + i * .6);
    c.drawOval(const Rect.fromLTWH(-4, -24, 8, 22), _f(_li));
    c.restore();
  }
  c.drawOval(const Rect.fromLTWH(64, 64, 26, 24), _f(_pi));
  c.drawCircle(const Offset(80, 60), 9, _f(_pi));
  c.drawPath(_poly(const [Offset(73, 56), Offset(74, 46), Offset(79, 52)]), _f(_pi));
  c.drawPath(_poly(const [Offset(87, 56), Offset(86, 46), Offset(81, 52)]), _f(_pi));
  c.drawCircle(const Offset(40, 98), 7, _f(_re));
  final r = 20 + s.a * 54, k = _k(s.b);
  final stops = [for (var i = 0; i <= 8; i++) i / 8];
  c.drawRect(
      _full,
      Paint()
        ..shader = ui.Gradient.radial(s.p, r, [for (final x in stops) _al(const Color(0xFF0A0A0C), .93 * (1 - _fall(x * r, r, k)))], stops));
  final base = const Offset(150, 116), d = _nrm(s.p - base);
  c.drawLine(base, base + d * 14, _s(_cr, 8));
  c.drawLine(base + d * 13, base + d * 17, _s(_ye, 11)..strokeCap = StrokeCap.butt);
}

void _fl2(Canvas c, _St s, double t) {
  const pond = Rect.fromLTWH(6, 6, 144, 108);
  c.drawOval(pond, _f(_bl));
  final o = s.p, R = 18 + s.a * 84, k = _k(s.b);
  c.save();
  c.clipPath(Path()..addOval(pond));
  for (var i = 0; i < 7; i++) {
    final rr = (t * 20 + i * 12) % 84, amp = _fall(rr, R, k);
    if (amp > .02) c.drawCircle(o, rr, _s(_al(_cr, amp), .8 + 3 * amp));
  }
  c.restore();
  const pads = [Offset(30, 40), Offset(122, 34), Offset(114, 88), Offset(42, 86), Offset(80, 24), Offset(132, 62)];
  for (var i = 0; i < pads.length; i++) {
    final d = (pads[i] - o).distance, amp = _fall(d, R, k);
    final q = pads[i] + Offset(0, math.sin(t * 5 - d * .15) * 3.5 * amp);
    final rr = 8.0 + 2 * amp;
    c.drawCircle(q, rr, _f(_mix(_dk(_li, .25), _li, amp)));
    c.drawArc(Rect.fromCircle(center: q, radius: rr), -.25 + i.toDouble(), .5, true, _f(_bl));
    if (amp > .5) c.drawCircle(q + const Offset(-2, -2), 2.4, _f(_pi));
  }
  c.drawCircle(o, 3.5, _f(_cr));
}

void _fl3(Canvas c, _St s, double t) {
  final r = (s.p - _pc).distance.clamp(14.0, 80.0);
  for (var i = 0; i < 36; i += 2) {
    c.drawArc(Rect.fromCircle(center: _pc, radius: r), i * math.pi / 18, math.pi / 30, false, _s(_al(_or, .7), 2));
  }
  c.drawLine(const Offset(68, 68), const Offset(88, 60), _s(_cr, 3));
  c.drawLine(const Offset(68, 60), const Offset(88, 68), _s(_cr, 3));
  for (var j = 0; j < 3; j++) {
    final h = (14 - j * 4) * (1 + .12 * math.sin(t * 9 + j * 2));
    final w = 8.0 - j * 2;
    c.drawPath(
        Path()
          ..moveTo(78, 64 - h)
          ..quadraticBezierTo(78 + w * 1.4, 60, 78, 66)
          ..quadraticBezierTo(78 - w * 1.4, 60, 78, 64 - h),
        _f([_re, _or, _ye][j]));
  }
  for (var i = 0; i < 13; i++) {
    final d = 22 + i * 4.2, pos = _pc + _pol(i * 2.4, d), w = _fall(d, r, 1.2);
    final q = pos + Offset((1 - w) * math.sin(t * 38 + i) * 1.2, 0);
    c.drawCircle(q, 6, _f(_mix(_cy, _or, w)));
    if (w > .35) {
      c.drawArc(Rect.fromCircle(center: q + const Offset(-2.2, -.5), radius: 1.4), math.pi, math.pi, false, _s(_k1, 1.1));
      c.drawArc(Rect.fromCircle(center: q + const Offset(2.2, -.5), radius: 1.4), math.pi, math.pi, false, _s(_k1, 1.1));
    } else {
      c.drawCircle(q + const Offset(-2.2, -.5), 1, _f(_k1));
      c.drawCircle(q + const Offset(2.2, -.5), 1, _f(_k1));
    }
  }
  c.drawCircle(_pc + _nrm(s.p - _pc) * r, 4, _f(_or));
}

Offset _iso(double x, double y, [double sx = 8, double sy = 4.6, Offset o = const Offset(78, 58)]) => o + Offset((x - y) * sx, (x + y) * sy);

void _fl4(Canvas c, _St s, double t) {
  final R = 1.2 + s.a * 4.8, k = _k(s.b);
  for (var y = -4; y <= 4; y++) {
    for (var x = -4; x <= 4; x++) {
      final w = _fall(math.sqrt(x * x + y * y + 0.0), R, k);
      final col = w < .5 ? _mix(_ye, _li, w * 2) : _mix(_li, _cy, (w - .5) * 2);
      final q = _iso(x.toDouble(), y.toDouble());
      c.drawPath(_poly([q + const Offset(0, -4.6), q + const Offset(8, 0), q + const Offset(0, 4.6), q + const Offset(-8, 0)]), _f(col));
      c.drawPath(_poly([q + const Offset(0, -4.6), q + const Offset(8, 0), q + const Offset(0, 4.6), q + const Offset(-8, 0)]), _s(_k2, .8));
    }
  }
  final head = _iso(0, 0) - const Offset(0, 8);
  for (var j = 1; j <= 7; j++) {
    final f = j / 7, a = t * 2.2;
    final g = _iso(math.cos(a) * R * f, math.sin(a) * R * f) - Offset(0, 8 + math.sin(f * math.pi) * 18);
    c.drawCircle(g, 1.8, _f(_wh));
  }
  c.drawLine(_iso(0, 0), head, _s(_cr, 3));
  c.drawCircle(head, 3, _f(_or));
}

void _fl5(Canvas c, _St s, double t) {
  final R = 5 + s.a * 22, k = _k(s.b);
  final pk = <Offset>[], vi = <Offset>[];
  var i = 0;
  Offset? prev;
  for (final q0 in s.trail) {
    if (q0 == null) {
      prev = null;
      continue;
    }
    final n = prev == null ? 1 : math.max(1, ((q0 - prev).distance / 3).ceil());
    for (var m = 1; m <= n; m++) {
      final q = prev == null ? q0 : Offset.lerp(prev, q0, m / n)!;
      for (var j = 0; j < 6; j++) {
        final ang = _h(i * 31 + j) * math.pi * 2, rr = R * math.pow(_h(i * 17 + j * 7 + 3), k);
        (j.isEven ? pk : vi).add(q + _pol(ang, rr.toDouble()));
      }
      i++;
    }
    prev = q0;
  }
  c.drawPoints(ui.PointMode.points, pk, _s(_pi, 2.2));
  c.drawPoints(ui.PointMode.points, vi, _s(_vi, 2.2));
  final last = s.pts.isEmpty ? _pc : s.pts.last;
  final can = last + const Offset(10, -18);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: can, width: 12, height: 22), const Radius.circular(3)), _f(_k1));
  c.drawRect(Rect.fromCenter(center: can + const Offset(0, -2), width: 12, height: 6), _f(_pi));
  c.drawRect(Rect.fromCenter(center: can + const Offset(-2, -14), width: 5, height: 5), _f(_k1));
}

void _fl6(Canvas c, _St s, double t) {
  const m = Offset(26, 60);
  final r = (s.p - m).distance.clamp(16.0, 140.0);
  for (var j = 0; j < 8; j++) {
    for (var i = 0; i < 11; i++) {
      final q = Offset(50 + i * 9.5, 10 + j * 14.2), d = q - m, w = _fall(d.distance, r, 1.1);
      final a0 = _h(i * 13 + j * 7) * math.pi, a1 = d.direction + math.sin(d.direction * 2) * .6;
      var da = (a1 - a0) % math.pi;
      if (da > math.pi / 2) da -= math.pi;
      final a = a0 + da * w, len = 3 + 4.5 * w;
      c.drawLine(q - _pol(a, len), q + _pol(a, len), _s(_mix(_al(_cr, .35), _re, w), 1.2 + w));
    }
  }
  c.drawArc(Rect.fromCircle(center: m, radius: 14), math.pi * .5, math.pi, false, _s(_re, 9)..strokeCap = StrokeCap.butt);
  c.drawLine(m + const Offset(0, -14), m + const Offset(10, -14), _s(_re, 9)..strokeCap = StrokeCap.butt);
  c.drawLine(m + const Offset(0, 14), m + const Offset(10, 14), _s(_re, 9)..strokeCap = StrokeCap.butt);
  c.drawLine(m + const Offset(10, -14), m + const Offset(16, -14), _s(_wh, 9)..strokeCap = StrokeCap.butt);
  c.drawLine(m + const Offset(10, 14), m + const Offset(16, 14), _s(_wh, 9)..strokeCap = StrokeCap.butt);
  c.drawArc(Rect.fromCircle(center: m, radius: r), -.9, 1.8, false, _s(_al(_re, .5), 1.2));
  c.drawCircle(m + _nrm(s.p - m) * r, 4, _f(_re));
}

void _fl7(Canvas c, _St s, double t) {
  final depth = .25 + s.a * 1.1, R = 22 + s.b * 60;
  double z(double u, double v) => depth * _fall(math.sqrt(u * u + v * v) * 70, R, 1.7);
  Offset P(double u, double v) {
    final sc = .72 + .28 * (v + 1) / 2;
    return Offset(78 + u * 74 * sc, 52 + v * 30 + z(u, v) * 40);
  }

  for (var j = 0; j <= 8; j++) {
    final v = -1 + j / 4, path = Path();
    for (var i = 0; i <= 28; i++) {
      final q = P(-1 + i / 14, v);
      i == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
    }
    c.drawPath(path, _s(_mix(_vi, _cy, j / 8), 1));
  }
  for (var i = 0; i <= 14; i++) {
    final u = -1 + i / 7, path = Path();
    for (var j = 0; j <= 16; j++) {
      final q = P(u, -1 + j / 8);
      j == 0 ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
    }
    c.drawPath(path, _s(_al(_vi, .8), 1));
  }
  final mu = math.cos(t * 1.3) * .55, mv = math.sin(t * 1.3) * .55;
  final ball = P(0, 0) - const Offset(0, 8);
  final moon = P(mu, mv) - const Offset(0, 4);
  if (mv < 0) c.drawCircle(moon, 3.5, _f(_ye));
  c.drawCircle(ball, 9, _f(_or));
  c.drawOval(Rect.fromCenter(center: ball, width: 30, height: 7), _s(_pi, 2));
  if (mv >= 0) c.drawCircle(moon, 3.5, _f(_ye));
}

void _fl8(Canvas c, _St s, double t) {
  final R = 18 + s.a * 70, k = _k(s.b);
  for (var i = 0; i < 4; i++) {
    c.drawCircle(_pc, 6.0 + i * 6, _s(_al(_re, .9 - i * .18), 2.5));
  }
  for (var i = 0; i < 15; i++) {
    final d = 18 + i * 4.0, q = _pc + _pol(i * 2.39 + .4, d), w = _fall(d, R, k);
    final egg = Path();
    for (var j = 0; j < 12; j++) {
      final rr = 9 + 1.6 * math.sin(j * 1.7 + i);
      final pt = q + _pol(j * math.pi / 6, rr);
      j == 0 ? egg.moveTo(pt.dx, pt.dy) : egg.lineTo(pt.dx, pt.dy);
    }
    egg.close();
    c.drawPath(egg, _f(_al(_wh, .22 + .78 * w)));
    if (w > .45) c.drawPath(egg, _s(_al(_or, (w - .45) * 1.8), 2));
    c.drawCircle(q + const Offset(1, -1), 3.6, _f(_mix(_or, _ye, w)));
  }
}

void _fl9(Canvas c, _St s, double t) {
  c.drawCircle(const Offset(136, 16), 8, _f(_ye));
  c.drawCircle(const Offset(132, 13), 7, _f(_k1));
  final R = 1.6 + s.a * 3.6, k = _k(s.b);
  const bands = [_bl, _vi, _pi, _or, _ye];
  for (var sum = -6; sum <= 6; sum++) {
    for (var x = -3; x <= 3; x++) {
      final y = sum - x;
      if (y < -3 || y > 3) continue;
      final w = _fall(math.sqrt(x * x + y * y + 0.0), R, k), h = 3 + 46 * w;
      final P = _iso(x.toDouble(), y.toDouble(), 9, 5.2, const Offset(78, 76)), T = P - Offset(0, h);
      final col = bands[(w * 4.99).floor()];
      c.drawPath(_poly([T + const Offset(-7.5, 0), T + const Offset(0, 4.3), P + const Offset(0, 4.3), P + const Offset(-7.5, 0)]), _f(_dk(col, .35)));
      c.drawPath(_poly([T + const Offset(0, 4.3), T + const Offset(7.5, 0), P + const Offset(7.5, 0), P + const Offset(0, 4.3)]), _f(_dk(col, .18)));
      c.drawPath(_poly([T + const Offset(0, -4.3), T + const Offset(7.5, 0), T + const Offset(0, 4.3), T + const Offset(-7.5, 0)]), _f(col));
    }
  }
}

List<TextPainter>? _glyphs;

void _fl10(Canvas c, _St s, double t) {
  const ramp = ' .:-=+*#%@';
  _glyphs ??= [
    for (var i = 0; i < 10; i++)
      TextPainter(
        text: TextSpan(
            text: ramp[i],
            style: TextStyle(fontFamily: 'Courier', fontSize: 13, fontWeight: FontWeight.w700, color: i < 5 ? _mix(_vi, _pi, i / 4) : _mix(_pi, _ye, (i - 5) / 4))),
        textDirection: TextDirection.ltr,
      )..layout(),
  ];
  final R = 22 + s.a * 60, k = _k(s.b);
  for (var j = 0; j < 9; j++) {
    for (var i = 0; i < 16; i++) {
      final q = Offset(4 + i * 9.6, 1 + j * 13.2), w = _fall((q + const Offset(4, 7) - s.p).distance, R, k);
      final flick = .04 * math.sin(t * 6 + i * 1.3 + j * 2.1);
      final g = _glyphs![((w + flick) * 9.4).round().clamp(0, 9)];
      g.paint(c, q);
    }
  }
}

void _fl11(Canvas c, _St s, double t) {
  final R = 16 + s.a * 56, k = _k(s.b);
  for (var j = 0; j < 10; j++) {
    for (var i = 0; i < 13; i++) {
      final q = Offset(6 + i * 12.0 + (j.isOdd ? 6 : 0), 6 + j * 12.0), w = _fall((q - s.p).distance, R, k);
      if (w > .72) {
        for (var n = 0; n < 5; n++) {
          c.drawCircle(q + _pol(n * 1.2566, 2.6), 2, _f(_pi));
        }
        c.drawCircle(q, 1.6, _f(_ye));
      } else {
        c.drawCircle(q, 7.2 * (1 - w), _f(_wh));
        c.drawCircle(q + const Offset(1.4, 1.6), 7.2 * (1 - w) * .55, _f(_al(_cy, .25)));
      }
    }
  }
  c.save();
  c.translate(s.p.dx, s.p.dy);
  c.rotate(t * .8);
  for (var n = 0; n < 8; n++) {
    c.drawLine(_pol(n * math.pi / 4, 10), _pol(n * math.pi / 4, 15), _s(_or, 2.5));
  }
  c.restore();
  c.drawCircle(s.p, 8, _f(_ye));
}

void _fl12(Canvas c, _St s, double t) {
  final R = 26 + s.a * 50, k = _k(s.b);
  final idle = !s.down && s.tUp < -50, age = s.now - s.tUp;
  final amt = s.down || idle ? 1.0 : math.cos(age * 14) * math.exp(-age * 4.5);
  final p0 = idle ? const Offset(70, 52) : s.p0, pull = idle ? Offset(math.sin(t * 1.4) * 18, math.cos(t * 1.1) * 12) : s.pull;
  Offset at(int i, int j) {
    final q = Offset(28 + i * 12.5, 22 + j * 12.6);
    return q + pull * _fall((q - p0).distance, R, k) * .85 * amt;
  }

  c.drawOval(const Rect.fromLTWH(10, 70, 136, 44), _f(_cr));
  c.drawOval(const Rect.fromLTWH(24, 80, 108, 28), _f(_dk(_cr, .08)));
  for (var j = 0; j < 6; j++) {
    for (var i = 0; i < 8; i++) {
      final quad = [at(i, j), at(i + 1, j), at(i + 1, j + 1), at(i, j + 1)];
      c.drawPath(_poly(quad), _f(i.isEven ? _pi : _mix(_pi, _re, .45)));
    }
  }
  final rim = Path()..moveTo(at(0, 0).dx, at(0, 0).dy);
  for (var i = 1; i <= 8; i++) {
    rim.lineTo(at(i, 0).dx, at(i, 0).dy);
  }
  for (var j = 1; j <= 6; j++) {
    rim.lineTo(at(8, j).dx, at(8, j).dy);
  }
  for (var i = 7; i >= 0; i--) {
    rim.lineTo(at(i, 6).dx, at(i, 6).dy);
  }
  for (var j = 5; j >= 1; j--) {
    rim.lineTo(at(0, j).dx, at(0, j).dy);
  }
  rim.close();
  c.drawPath(rim, _s(_wh, 2));
  for (var j = 1; j < 6; j++) {
    final path = Path()..moveTo(at(0, j).dx, at(0, j).dy);
    for (var i = 1; i <= 8; i++) {
      path.lineTo(at(i, j).dx, at(i, j).dy);
    }
    c.drawPath(path, _s(_al(_wh, .35), 1));
  }
  final hl = Offset.lerp(at(1, 1), at(2, 2), .5)!;
  c.drawOval(Rect.fromCenter(center: hl, width: 14, height: 6), _f(_al(_wh, .8)));
  c.drawCircle(p0 + pull * amt, 4.5, _f(_ye));
}

void _fl13(Canvas c, _St s, double t) {
  final R = 22 + s.a * 56, k = _k(s.b);
  for (var j = 0; j < 4; j++) {
    for (var i = 0; i < 6; i++) {
      final q0 = Offset(16 + i * 25.0 + (j.isOdd ? 10 : 0) + _h(i + j * 9) * 4, 18 + j * 27.0);
      final away = q0 - s.p, w = _fall(away.distance, R, k), q = q0 + _nrm(away) * 6 * w;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: q, width: 15, height: 14), const Radius.circular(6)), _f(_mix(_cy, _pi, w)));
      final e = 1.2 + 2 * w, look = _nrm(s.p - q) * (w * 1.2);
      c.drawCircle(q + const Offset(-3, -2), e, _f(_wh));
      c.drawCircle(q + const Offset(3, -2), e, _f(_wh));
      c.drawCircle(q + const Offset(-3, -2) + look, 1, _f(_k1));
      c.drawCircle(q + const Offset(3, -2) + look, 1, _f(_k1));
      if (w > .5) {
        c.drawCircle(q + const Offset(0, 3.5), 1.8, _f(_k1));
      } else {
        c.drawLine(q + const Offset(-2, 3.5), q + const Offset(2, 3.5), _s(_k1, 1));
      }
    }
  }
  final p = s.p;
  c.drawPath(_poly([p + const Offset(-11, -4), p + const Offset(-8, -16), p + const Offset(-2, -8)]), _f(_or));
  c.drawPath(_poly([p + const Offset(11, -4), p + const Offset(8, -16), p + const Offset(2, -8)]), _f(_or));
  c.drawCircle(p, 11, _f(_or));
  c.drawOval(Rect.fromCenter(center: p + const Offset(-4, -1), width: 3, height: 5), _f(_k1));
  c.drawOval(Rect.fromCenter(center: p + const Offset(4, -1), width: 3, height: 5), _f(_k1));
  c.drawCircle(p + const Offset(0, 4), 1.6, _f(_pi));
}

void _fl14(Canvas c, _St s, double t) {
  final pull = 10 + s.a * 46, R = 28 + s.b * 70;
  final back = Rect.fromCenter(center: _pc, width: 58, height: 16);
  c.drawArc(back, math.pi, math.pi, false, _s(_or, 3.5));
  for (var i = 0; i < 80; i++) {
    final q = Offset(_h(i) * _pw, _h(i + 300) * _ph), d = q - _pc, w = _fall(d.distance, R, 1.8);
    final dd = math.min(pull * w, d.distance - 12), n = _nrm(d), qq = q - n * math.max(dd, 0);
    final tan = Offset(-n.dy, n.dx) * (1 + 7 * w);
    c.drawLine(qq - tan, qq + tan, _s(i % 5 == 0 ? _cy : _cr, 1.3));
  }
  c.drawCircle(_pc, 11, _f(const Color(0xFF000000)));
  c.drawCircle(_pc, 11.5, _s(_vi, 1.6));
  c.drawArc(back, 0, math.pi, false, _s(_ye, 3.5));
}

void _fl15(Canvas c, _St s, double t) {
  final R = 22 + s.a * 64, k = _k(s.b);
  for (var j = 0; j < 5; j++) {
    for (var i = 0; i < 9; i++) {
      final base = Offset(12 + i * 16.5 + (j.isOdd ? 8 : 0), 30 + j * 21.0);
      final side = base.dx < s.p.dx ? -1.0 : 1.0, w = _fall((base - s.p).distance, R, k);
      final lean = w * 1.35 * side, hgt = 17.0;
      final top = base + Offset(math.sin(lean) * hgt, -math.cos(lean) * hgt * .72);
      final n = Offset(3.5, 0);
      final col = w > .15 ? _pop[(i + j * 3) % 8] : _k1;
      c.drawPath(_poly([base - n, base + n, top + n, top - n]), _f(col));
      final mid = Offset.lerp(base, top, .5)!;
      c.drawLine(mid - n * .8, mid + n * .8, _s(_cr, 1));
      c.drawCircle(Offset.lerp(base, top, .75)!, 1, _f(_cr));
      c.drawCircle(Offset.lerp(base, top, .25)!, 1, _f(_cr));
    }
  }
  c.drawOval(Rect.fromCenter(center: s.p + const Offset(0, 6), width: 14, height: 4), _f(_al(_k1, .25)));
  c.drawCircle(s.p, 6, _f(_re));
}

void _fl16(Canvas c, _St s, double t) {
  final R = 14 + s.a * 70, k = _k(s.b);
  c.drawLine(const Offset(20, 12), const Offset(136, 12), _s(_cr, 3));
  for (var i = 0; i < 9; i++) {
    final x = 26 + i * 13.0, w = _fall((x - s.p.dx).abs(), R, k);
    final len = 26 + (i * 7 % 5) * 7.0, a = w * .55 * math.sin(t * 5 + i * .9);
    c.save();
    c.translate(x, 12);
    c.rotate(a);
    c.drawLine(Offset.zero, const Offset(0, 8), _s(_al(_cr, .6), 1));
    c.drawLine(const Offset(0, 10), Offset(0, 10 + len), _s(_pop[i % 8], 5.5));
    c.drawLine(const Offset(-1.5, 12), Offset(-1.5, 8 + len), _s(_al(_wh, .45), 1.2));
    c.restore();
  }
  final cl = Offset(s.p.dx, 104);
  for (final o in const [Offset(-9, 2), Offset(0, -3), Offset(9, 2), Offset(-3, 4), Offset(5, 4)]) {
    c.drawCircle(cl + o, 7, _f(_cy));
  }
  for (var j = 0; j < 3; j++) {
    final y = 90 - ((t * 30 + j * 12) % 30);
    c.drawLine(Offset(cl.dx - 8 + j * 8, y), Offset(cl.dx - 8 + j * 8, y - 8), _s(_al(_cy, .7), 1.6));
  }
}

void _fl17(Canvas c, _St s, double t) {
  c.drawCircle(const Offset(128, 24), 10, _f(_ye));
  final W = 18 + s.a * 64, k = _k(s.b);
  Path mtn(double sc) {
    final p = Path()..moveTo(0, 112);
    for (var x = 0; x <= 156; x += 3) {
      p.lineTo(x.toDouble(), 112 - 76 * sc * _fall((x - 78).abs().toDouble(), W, k));
    }
    return p
      ..lineTo(156, 112)
      ..close();
  }

  c.drawPath(mtn(1), _f(_pi));
  c.drawPath(mtn(.72), _f(_or));
  c.drawPath(mtn(.44), _f(_re));
  c.drawPath(mtn(.2), _f(_vi));
  c.drawRect(const Rect.fromLTWH(0, 110, _pw, 10), _f(_k1));
  final top = Offset(78, 112 - 76.0 * _fall(0, W, k));
  c.drawCircle(top + const Offset(0, 1), 4, _f(_ye));
  for (var j = 0; j < 3; j++) {
    final f = (t * .4 + j / 3) % 1;
    c.drawCircle(top + Offset(f * 14, -6 - f * 26), 3 + f * 6, _f(_al(_cr, .9 - f * .8)));
  }
}

void _fl18(Canvas c, _St s, double t) {
  final R = 24 + s.a * 56, k = _k(s.b);
  double y(int j) => 40 + j * j * 1.2 + j * 3.0;
  double hw(int j) => 6 + j * 1.8;
  const cols = [_pi, _vi, _cy, _ye];
  for (var j = 0; j < 8; j++) {
    for (var u = -7; u < 7; u++) {
      final q = [
        Offset(78 + u * hw(j), y(j)),
        Offset(78 + (u + 1) * hw(j), y(j)),
        Offset(78 + (u + 1) * hw(j + 1), y(j + 1)),
        Offset(78 + u * hw(j + 1), y(j + 1)),
      ];
      final m = (q[0] + q[2]) / 2, d = Offset(m.dx - s.p.dx, (m.dy - s.p.dy) * 1.5).distance, w = _fall(d, R, k);
      final band = cols[((d / 12 - t * 3) % 4).floor()];
      c.drawPath(_poly(q), _f(_mix(_k2, band, .08 + .92 * w)));
      c.drawPath(_poly(q), _s(_k1, .8));
    }
  }
  const ball = Offset(78, 16);
  c.drawLine(const Offset(78, 0), ball, _s(_cr, 1));
  c.drawLine(ball, s.p, _s(_al(_pi, .5), 2));
  c.drawLine(ball, s.p + const Offset(8, 0), _s(_al(_cy, .4), 1.5));
  c.drawCircle(ball, 8, _f(_cr));
  for (var i = -1; i <= 1; i++) {
    c.drawLine(ball + Offset(-8, i * 4.0), ball + Offset(8, i * 4.0), _s(_k2, .8));
    c.drawLine(ball + Offset(i * 4.0, -8), ball + Offset(i * 4.0, 8), _s(_k2, .8));
  }
}

void _fl19(Canvas c, _St s, double t) {
  final R = 24 + s.a * 64, k = _k(s.b);
  for (var j = 0; j < 7; j++) {
    for (var i = 0; i < 10; i++) {
      final q = Offset(12 + i * 14.7, 12 + j * 16.0), d = q - s.p, w = _fall(d.distance, R, k);
      final n = _nrm(d), len = 2 + 9 * w, tip = q + n * len, col = _mix(_al(_cr, .3), _cy, w);
      c.drawLine(q - n * 2, tip, _s(col, 1.4));
      if (w > .05) {
        final side = Offset(-n.dy, n.dx) * 2.6;
        c.drawPath(_poly([tip + n * 2.5, tip + side, tip - side]), _f(col));
      }
    }
  }
  c.drawCircle(s.p, 5, _s(_li, 2));
  c.drawCircle(s.p, 1.8, _f(_li));
}

void _fl20(Canvas c, _St s, double t) {
  final power = 6 + s.a * 46, R = 30 + s.b * 70;
  for (var j = 0; j < 9; j++) {
    for (var i = 0; i < 12; i++) {
      final q = Offset(8 + i * 12.6, 7 + j * 13.2), d = q - s.p, w = _fall(d.distance, R, 1.4);
      final rnd = _h(i * 11 + j * 5);
      final p = q + _nrm(d) * power * w * (.7 + .6 * rnd);
      c.save();
      c.translate(p.dx, p.dy);
      c.rotate((rnd - .5) * 5 * w);
      final sz = 9.5 * (1 - .45 * w);
      c.drawRect(Rect.fromCenter(center: Offset.zero, width: sz, height: sz), _f(_pop[(rnd * 8).floor()]));
      c.restore();
    }
  }
  final burst = 8 + s.a * 10 + math.sin(t * 10) * 1.5;
  c.drawPath(_star(s.p, burst, 8, .45, t), _f(_ye));
  c.drawPath(_star(s.p, burst * .55, 8, .45, -t), _f(_wh));
}
