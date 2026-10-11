part of 'pop_02.dart';

// Echo / Trail = "a moving thing leaves copies behind it": how many, how far apart, how fast they fade. Drag right = more copies, up = spacing / decay.

const _echo = <_Pn>[
  _Pn('Comet', 'cosmic·neon·2D·drag·result', _ec1, a: .5, b: .4),
  _Pn('Penguin steps', 'creature·bold·top·drag·result', _ec2, bg: _cr, a: .55, b: .45),
  _Pn('Ducklings', 'creature·bold·side·drag·result', _ec3, bg: _cy, a: .45, b: .5),
  _Pn('Train', 'machine·bold·iso·drag·result', _ec4, bg: _k2, a: .45, b: .4),
  _Pn('Strobe bounce', 'physics·crisp·2D·drag·result', _ec5, a: .5, b: .35),
  _Pn('Onion skin', 'char·crisp·2D·drag·mech', _ec6, bg: _k2, a: .45, b: .5),
  _Pn('Stamp brush', 'toy·bold·2D·paint·result', _ec7, a: .75, b: .3, seed: _wave),
  _Pn('Sliced loaf', 'food·bold·3D·drag·result', _ec8, bg: _bl, a: .45, b: .4),
  _Pn('Mirror tunnel', 'machine·wild·persp·drag·result', _ec9, a: .5, b: .5),
  _Pn('Canyon yell', 'landscape·bold·side·drag·mech', _ec10, bg: _ye, a: .55, b: .55),
  _Pn('Kite tail', 'weather·bold·2D·fly·result', _ec11, bg: _bl, a: .5, b: .4, p: Offset(108, 30)),
  _Pn('Pendulum exposure', 'machine·crisp·2D·drag·result', _ec12, bg: _cr, a: .5, b: .4),
  _Pn('Firefly', 'nature·neon·2D·drag·result', _ec13, a: .5, b: .4),
  _Pn('Slinky', 'toy·bold·3D·drag·mech', _ec14, bg: _cr, a: .45, b: .5),
  _Pn('Matryoshka', 'char·bold·2D·drag·result', _ec15, bg: _k2, a: .55, b: .5),
  _Pn('Ink runs out', 'material·crisp·2D·drag·result', _ec16, bg: _cr, a: .6, b: .5),
  _Pn('Neon racer', 'machine·wild·side·drag·result', _ec17, a: .5, b: .45),
  _Pn('Ping-pong', 'toy·crisp·top·drag·mech', _ec18, bg: _k2, a: .5, b: .55),
  _Pn('Sunset sea', 'landscape·bold·2D·drag·result', _ec19, bg: _or, a: .5, b: .4),
  _Pn('Kaleido spiral', 'cosmic·wild·2D·drag·result', _ec20, a: .5, b: .4),
];

int _n(double a, int lo, int hi) => lo + (a * (hi - lo)).round();

void _ec1(Canvas c, _St s, double t) {
  for (var i = 0; i < 26; i++) {
    c.drawCircle(Offset(_h(i) * _pw, _h(i + 50) * _ph), .9, _f(_al(_cr, .5)));
  }
  c.drawCircle(_pc, 9, _f(_or));
  c.drawOval(Rect.fromCenter(center: _pc, width: 30, height: 7), _s(_ye, 1.6));
  Offset at(double u) => _pc + Offset(math.cos(u * 2 * math.pi) * 64, math.sin(u * 2 * math.pi) * 36 + math.cos(u * 2 * math.pi) * 10);
  final n = _n(s.a, 1, 14), du = .008 + s.b * .045, u0 = t * .11;
  for (var i = n; i >= 1; i--) {
    final q = at(u0 - i * du), f = i / (n + 1);
    c.drawCircle(q, 6 * (1 - f * .6), _f(_al(_mix(_cy, _vi, f), math.pow(.84, i).toDouble())));
  }
  c.drawCircle(at(u0), 7, _f(_cr));
  c.drawCircle(at(u0), 3.5, _f(_ye));
}

void _ec2(Canvas c, _St s, double t) {
  const from = Offset(8, 112), to = Offset(128, 30);
  final d = _nrm(to - from), side = Offset(-d.dy, d.dx), stride = 6 + s.b * 12, n = _n(s.a, 1, 18);
  for (var i = n; i >= 1; i--) {
    final q = to - d * (stride * i + 6) + side * (i.isEven ? 4.5 : -4.5);
    final col = _al(_bl, .95 * math.pow(.86, i - 1).toDouble());
    c.save();
    c.translate(q.dx, q.dy);
    c.rotate(d.direction + math.pi / 2);
    for (final a in const [-.5, 0.0, .5]) {
      c.drawLine(Offset.zero, _pol(-math.pi / 2 + a, 5), _s(col, 1.8));
    }
    c.restore();
  }
  final bob = math.sin(t * 8).abs() * 2, wob = math.sin(t * 8) * .15;
  c.save();
  c.translate(to.dx + 6, to.dy - 4 - bob);
  c.rotate(wob);
  c.drawOval(const Rect.fromLTWH(-9, -14, 18, 26), _f(_k1));
  c.drawOval(const Rect.fromLTWH(-5.5, -6, 11, 17), _f(_wh));
  c.drawCircle(const Offset(-3, -8), 1.5, _f(_wh));
  c.drawCircle(const Offset(3, -8), 1.5, _f(_wh));
  c.drawPath(_poly(const [Offset(-2.5, -5), Offset(2.5, -5), Offset(0, -1.5)]), _f(_or));
  c.drawOval(const Rect.fromLTWH(-8, 10, 7, 3.5), _f(_or));
  c.drawOval(const Rect.fromLTWH(1, 10, 7, 3.5), _f(_or));
  c.restore();
}

void _duck(Canvas c, Offset q, double sc, bool mom) {
  c.save();
  c.translate(q.dx, q.dy);
  c.scale(sc);
  c.drawPath(
      Path()
        ..moveTo(-14, 0)
        ..quadraticBezierTo(-12, 10, 2, 10)
        ..quadraticBezierTo(14, 10, 12, -2)
        ..quadraticBezierTo(0, 2, -14, 0),
      _f(mom ? _wh : _ye));
  c.drawCircle(const Offset(8, -8), 6.5, _f(mom ? _wh : _ye));
  c.drawPath(_poly(const [Offset(13, -9), Offset(20, -7), Offset(13, -5)]), _f(_or));
  c.drawCircle(const Offset(9.5, -9.5), 1.4, _f(_k1));
  c.drawPath(Path()..moveTo(-8, 2)..quadraticBezierTo(-2, 0, 2, 5), _s(_al(_k1, .25), 1.4));
  c.restore();
}

void _ec3(Canvas c, _St s, double t) {
  final n = _n(s.a, 0, 9), gap = 14 + s.b * 14;
  double y(double x) => 64 + math.sin(t * 1.6 - x * .045) * 12;
  for (var i = n; i >= 0; i--) {
    final x = 128.0 - (i == 0 ? 0.0 : 6 + gap * i * .9), sc = i == 0 ? 1.0 : .62 * math.pow(.92, i - 1).toDouble();
    final q = Offset(x, y(x));
    c.drawLine(q + Offset(-16 * sc, 10 * sc), q + Offset(-26 * sc, 14 * sc), _s(_al(_wh, .8), 1.4));
    _duck(c, q, sc, i == 0);
  }
}

void _ec4(Canvas c, _St s, double t) {
  Offset P(double x, double y) => _iso(x, y, 10, 5.8, const Offset(78, 56));
  c.drawLine(P(-8, -.45), P(8, -.45), _s(_al(_cr, .5), 1.2));
  c.drawLine(P(-8, .45), P(8, .45), _s(_al(_cr, .5), 1.2));
  for (var x = -8.0; x < 8; x += .7) {
    c.drawLine(P(x, -.6), P(x, .6), _s(_al(_cr, .25), 1.6));
  }
  final n = _n(s.a, 0, 8), gap = .15 + s.b * .9, head = 4.2;
  void box(double x, double len, Color col, double h, double fade) {
    final a0 = P(x - len / 2, -.4), a1 = P(x + len / 2, -.4), b1 = P(x + len / 2, .4), b0 = P(x - len / 2, .4);
    final up = Offset(0, -h);
    final cc = _mix(_k2, col, fade);
    c.drawPath(_poly([b0, b1, b1 + up, b0 + up]), _f(_dk(cc, .3)));
    c.drawPath(_poly([b1, a1, a1 + up, b1 + up]), _f(_dk(cc, .15)));
    c.drawPath(_poly([a0 + up, a1 + up, b1 + up, b0 + up]), _f(cc));
  }

  for (var i = n; i >= 1; i--) {
    box(head - 1.1 - i * (1.6 + gap) + .8, 1.6, _pop[(i + 1) % 8], 10, math.pow(.8, i - 1).toDouble());
  }
  box(head, 2, _re, 13, 1);
  final ch = P(head + .6, 0) - const Offset(0, 20);
  c.drawRect(Rect.fromCenter(center: ch + const Offset(0, 3), width: 4, height: 7), _f(_k1));
  for (var j = 0; j < 3; j++) {
    final f = (t * .7 + j / 3) % 1;
    c.drawCircle(ch + Offset(-f * 18, -f * 16), 2.5 + f * 5, _f(_al(_cr, .9 - f * .8)));
  }
  final tun = P(-7.5, 0);
  c.drawPath(
      Path()
        ..moveTo(tun.dx - 14, tun.dy + 8)
        ..lineTo(tun.dx - 14, tun.dy - 6)
        ..arcToPoint(Offset(tun.dx + 14, tun.dy - 6), radius: const Radius.circular(14))
        ..lineTo(tun.dx + 14, tun.dy + 8)
        ..close(),
      _f(_k1));
}

void _ec5(Canvas c, _St s, double t) {
  c.drawLine(const Offset(0, 108), const Offset(156, 108), _s(_cr, 2));
  Offset at(double tau) {
    final x = 10 + (tau * 44) % 140, ph = (tau * 2.9) % math.pi;
    return Offset(x, 100 - math.sin(ph) * 76);
  }

  final n = _n(s.a, 1, 16), dt = .03 + s.b * .12, head = at(t);
  for (var i = n; i >= 1; i--) {
    final q = at(t - i * dt);
    if (q.dx > head.dx) continue;
    c.drawCircle(q, 8, _s(_al(_pop[i % 8], math.pow(.86, i).toDouble()), 2.2));
  }
  final squash = head.dy > 96 ? .75 : 1.0;
  c.drawOval(Rect.fromCenter(center: head, width: 16 / squash, height: 16 * squash), _f(_or));
  c.drawCircle(head + const Offset(-3, -3), 2.5, _f(_al(_wh, .8)));
}

void _bean(Canvas c, double tau, Paint p) {
  final y = 74 - (math.sin(tau * 2.6)).abs() * 34, sq = 1 + .2 * math.cos(tau * 5.2);
  final q = Offset(78 + math.sin(tau * 1.3) * 26, y);
  c.drawOval(Rect.fromCenter(center: q, width: 20 * sq, height: 28 / sq), p);
  final arm = math.sin(tau * 5.2) * .9, ap = _s(p.color, 2.6);
  c.drawLine(q + const Offset(-9, -2), q + const Offset(-9, -2) + _pol(math.pi + arm, 11), ap);
  c.drawLine(q + const Offset(9, -2), q + const Offset(9, -2) + _pol(-arm, 11), ap);
}

void _ec6(Canvas c, _St s, double t) {
  c.drawLine(const Offset(20, 104), const Offset(136, 104), _s(_al(_cr, .4), 1.5));
  final n = _n(s.a, 1, 6), dec = .45 + s.b * .45, sp = .09;
  for (var i = n; i >= 1; i--) {
    final o = math.pow(dec, i - 1).toDouble();
    _bean(c, t - i * sp, _s(_al(_pi, o), 2));
    _bean(c, t + i * sp, _s(_al(_cy, o), 2));
  }
  _bean(c, t, _f(_cr));
  final y = 74 - (math.sin(t * 2.6)).abs() * 34, x = 78 + math.sin(t * 1.3) * 26;
  c.drawCircle(Offset(x - 3.5, y - 5), 1.6, _f(_k1));
  c.drawCircle(Offset(x + 3.5, y - 5), 1.6, _f(_k1));
}

void _ec7(Canvas c, _St s, double t) {
  final pts = s.trail.reversed.toList();
  final sp = 6 + s.b * 22, keep = .55 + s.a * .43;
  var acc = 0.0, k = 0;
  Offset? prev;
  for (final q in pts) {
    if (q == null) {
      prev = null;
      continue;
    }
    if (prev != null) acc += (q - prev).distance;
    prev = q;
    if (k == 0 || acc >= sp) {
      final o = math.pow(keep, k).toDouble();
      if (o < .04) break;
      c.drawPath(_star(q, 7, 5, .45, k * .5), _f(_al(_pop[k % 5], o)));
      acc = 0;
      k++;
    }
  }
}

void _ec8(Canvas c, _St s, double t) {
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(8, 92, 140, 14), const Radius.circular(4)), _f(_or));
  final n = _n(s.a, 1, 11), gap = 3 + s.b * 10;
  Path slice(Offset o) => Path()
    ..moveTo(o.dx - 15, o.dy)
    ..lineTo(o.dx - 15, o.dy - 22)
    ..cubicTo(o.dx - 24, o.dy - 30, o.dx - 14, o.dy - 46, o.dx, o.dy - 42)
    ..cubicTo(o.dx + 14, o.dy - 46, o.dx + 24, o.dy - 30, o.dx + 15, o.dy - 22)
    ..lineTo(o.dx + 15, o.dy)
    ..close();
  for (var i = n - 1; i >= 0; i--) {
    final o = Offset(40 + i * gap, 96 - i * gap * .35), f = n == 1 ? 0.0 : i / (n - 1);
    final sl = slice(o);
    c.drawPath(sl, _f(_mix(_cr, _ye, f)));
    c.drawPath(sl, _s(_dk(_or, .15 * f), 3));
  }
}

void _ec9(Canvas c, _St s, double t) {
  final n = _n(s.a, 2, 16), ratio = .7 + s.b * .22, ph = (t * .7) % 1;
  const cols = [_pi, _vi, _cy, _bl];
  for (var i = n; i >= 0; i--) {
    final e = i - ph;
    if (e < 0) continue;
    final sc = math.pow(ratio, e).toDouble(), ctr = Offset.lerp(s.p, _pc, sc)!;
    final r = RRect.fromRectAndRadius(Rect.fromCenter(center: ctr, width: 150 * sc, height: 112 * sc), Radius.circular(10 * sc));
    final col = cols[i % 4], o = math.pow(.88, e).toDouble();
    c.drawRRect(r, _s(_al(col, o * .3), 6 * sc + 2));
    c.drawRRect(r, _s(_al(col, o), 1.6 * sc + .6));
  }
}

void _ec10(Canvas c, _St s, double t) {
  c.drawPath(_poly(const [Offset(0, 0), Offset(30, 0), Offset(24, 30), Offset(36, 56), Offset(22, 84), Offset(30, 120), Offset(0, 120)]), _f(_re));
  c.drawPath(_poly(const [Offset(156, 0), Offset(124, 0), Offset(132, 26), Offset(120, 52), Offset(134, 80), Offset(122, 120), Offset(156, 120)]), _f(_or));
  c.drawRect(const Rect.fromLTWH(0, 108, _pw, 12), _f(_vi));
  const mouth = Offset(42, 98);
  c.drawCircle(mouth + const Offset(-6, 0), 7, _f(_cr));
  c.drawCircle(mouth + const Offset(-4, -1), 2.4, _f(_k1));
  final n = _n(s.a, 1, 8), dec = .5 + s.b * .45;
  final hits = [for (var i = 0; i < 8; i++) Offset(i.isEven ? 122 : 34, 86 - i * 11.0)];
  var from = mouth;
  for (var i = 0; i < n; i++) {
    final to = hits[i], o = math.pow(dec, i).toDouble(), sz = 11 * math.pow(.9, i).toDouble();
    for (var k = 0; k < 6; k++) {
      final f0 = k / 6, f1 = (k + .5) / 6;
      c.drawLine(Offset.lerp(from, to, f0)!, Offset.lerp(from, to, f1)!, _s(_al(_k1, o * .5), 1.2));
    }
    final face = i.isEven ? math.pi : 0.0, pulse = 1 + .12 * math.sin(t * 6 - i);
    for (var k = 0; k < 3; k++) {
      c.drawArc(Rect.fromCircle(center: to, radius: (sz - k * 3.5) * pulse), face - .9, 1.8, false, _s(_al(k == 0 ? _bl : _k1, o), 2.6));
    }
    from = to;
  }
}

void _ec11(Canvas c, _St s, double t) {
  for (var i = 0; i < 3; i++) {
    final x = (_h(i) * 180 + t * 6 * (i + 1)) % 190 - 20, y = 20 + i * 34.0;
    c.drawOval(Rect.fromCenter(center: Offset(x, y), width: 30, height: 9), _f(_al(_cr, .25)));
  }
  final k = s.p, top = k + const Offset(0, -12), bot = k + const Offset(0, 16);
  c.drawPath(Path()..moveTo(14, 118)..quadraticBezierTo(k.dx * .4, k.dy + 40, k.dx, k.dy), _s(_al(_cr, .6), 1));
  final n = _n(s.a, 1, 12), sp = 6 + s.b * 9;
  final tail = Path()..moveTo(bot.dx, bot.dy);
  final bows = <Offset>[];
  for (var i = 1; i <= n; i++) {
    final q = bot + Offset(-i * sp * .55 + math.sin(t * 3.2 - i * .8) * 3.0 * math.min(i / 3, 1.5), i * sp * .8);
    tail.lineTo(q.dx, q.dy);
    bows.add(q);
  }
  c.drawPath(tail, _s(_cr, 1));
  for (var i = 0; i < bows.length; i++) {
    final q = bows[i], sz = 4.5 * math.pow(.94, i).toDouble(), col = [_ye, _li, _cy][i % 3];
    c.drawPath(_poly([q, q + Offset(-sz, -sz * .8), q + Offset(-sz, sz * .8)]), _f(col));
    c.drawPath(_poly([q, q + Offset(sz, -sz * .8), q + Offset(sz, sz * .8)]), _f(col));
  }
  final kite = [top, k + const Offset(11, -2), bot, k + const Offset(-11, -2)];
  c.drawPath(_poly(kite), _f(_pi));
  c.drawPath(_poly([top, k + const Offset(11, -2), k + const Offset(0, -2)]), _f(_ye));
  c.drawLine(top, bot, _s(_cr, 1));
  c.drawLine(kite[1], kite[3], _s(_cr, 1));
}

void _ec12(Canvas c, _St s, double t) {
  const piv = Offset(78, 10);
  double th(double tau) => .95 * math.sin(tau * 2.3);
  c.drawArc(Rect.fromCircle(center: piv, radius: 84), math.pi / 2 - .95, 1.9, false, _s(_al(_k1, .15), 1));
  final n = _n(s.a, 1, 14), dt = .02 + s.b * .07;
  for (var i = n; i >= 1; i--) {
    final b = piv + _pol(math.pi / 2 + th(t - i * dt), 84), o = .6 * math.pow(.82, i).toDouble();
    c.drawLine(piv, b, _s(_al(_bl, o * .6), 1.2));
    c.drawCircle(b, 9, _f(_al(_bl, o)));
  }
  final b = piv + _pol(math.pi / 2 + th(t), 84);
  c.drawLine(piv, b, _s(_k1, 2));
  c.drawCircle(b, 9.5, _f(_re));
  c.drawCircle(piv, 3.5, _f(_k1));
}

void _ec13(Canvas c, _St s, double t) {
  c.drawCircle(const Offset(130, 18), 9, _f(_cr));
  for (var i = 0; i < 6; i++) {
    final x = i * 30.0 + 4, h = 34 + (i * 13 % 3) * 12.0;
    c.drawPath(_poly([Offset(x - 16, 120), Offset(x, 120 - h), Offset(x + 16, 120)]), _f(_k2));
  }
  Offset at(double tau) => Offset(78 + 60 * math.sin(tau * 1.05), 52 + 32 * math.sin(tau * 1.7 + 1));
  final n = _n(s.a, 2, 26), dt = .025 + s.b * .09;
  for (var i = n; i >= 1; i--) {
    final q = at(t - i * dt), o = math.pow(.9, i).toDouble();
    c.drawCircle(q, 3.6, _bf(_al(_li, o * .7), 2.2));
    c.drawCircle(q, 1.6, _f(_al(_ye, o)));
  }
  final q = at(t);
  c.drawCircle(q, 6, _bf(_al(_ye, .7), 3));
  c.drawCircle(q, 3, _f(_ye));
  c.drawOval(Rect.fromCenter(center: q + const Offset(-3, -3), width: 4, height: 2.5), _f(_al(_wh, .7)));
}

void _ec14(Canvas c, _St s, double t) {
  for (var i = 0; i < 5; i++) {
    c.drawRect(Rect.fromLTWH(i * 31.0, 48 + i * 16.0, 156 - i * 31.0, 120), _f(i.isEven ? _bl : _dk(_bl, .15)));
  }
  final n = _n(s.a, 4, 26), hgt = 10 + s.b * 40 + math.sin(t * 3) * 3;
  const a = Offset(22, 48), b = Offset(76, 80);
  for (var i = 0; i < n; i++) {
    final f = n == 1 ? 0.0 : i / (n - 1);
    final q = Offset.lerp(a, b, f)! + Offset(0, -math.sin(f * math.pi) * hgt);
    final tan = Offset(b.dx - a.dx, (b.dy - a.dy) - math.cos(f * math.pi) * hgt * math.pi);
    c.save();
    c.translate(q.dx, q.dy - 7);
    c.rotate(tan.direction);
    c.drawOval(Rect.fromCenter(center: Offset.zero, width: 4.5, height: 18), _s(_pop[i % 8], 2));
    c.restore();
  }
}

void _ec15(Canvas c, _St s, double t) {
  final n = _n(s.a, 1, 7), dec = .55 + s.b * .35;
  var x = 8.0;
  for (var i = 0; i < n; i++) {
    final sc = math.pow(dec, i).toDouble(), w = 34 * sc, col = [_re, _ye, _bl, _li, _pi, _vi, _or][i % 7];
    final base = Offset(x + w / 2, 104);
    c.save();
    c.translate(base.dx, base.dy);
    c.rotate(math.sin(t * 3 + i * 1.3) * .07);
    c.scale(sc);
    c.drawOval(const Rect.fromLTWH(-17, -40, 34, 40), _f(col));
    c.drawCircle(const Offset(0, -48), 14, _f(col));
    c.drawCircle(const Offset(0, -47), 9, _f(_cr));
    c.drawCircle(const Offset(-3.5, -48), 1.4, _f(_k1));
    c.drawCircle(const Offset(3.5, -48), 1.4, _f(_k1));
    c.drawCircle(const Offset(-5, -44), 2, _f(_al(_pi, .8)));
    c.drawCircle(const Offset(5, -44), 2, _f(_al(_pi, .8)));
    for (var k = 0; k < 5; k++) {
      c.drawCircle(const Offset(0, -20) + _pol(k * 1.2566, 5), 3, _f(_cr));
    }
    c.drawCircle(const Offset(0, -20), 2.6, _f(_ye));
    c.restore();
    x += w + 3;
  }
}

void _ec16(Canvas c, _St s, double t) {
  final n = _n(s.a, 1, 8), keep = .5 + s.b * .45;
  Path heart(Offset o, double r) => Path()
    ..moveTo(o.dx, o.dy + r)
    ..cubicTo(o.dx - r * 1.6, o.dy - r * .2, o.dx - r * .8, o.dy - r * 1.4, o.dx, o.dy - r * .5)
    ..cubicTo(o.dx + r * .8, o.dy - r * 1.4, o.dx + r * 1.6, o.dy - r * .2, o.dx, o.dy + r)
    ..close();
  Offset at(int i) => Offset(132 - i * 16.0, 92 - i * 10.0 + (i.isOdd ? 6 : 0));
  for (var i = n - 1; i >= 0; i--) {
    final q = at(i), ink = math.pow(keep, i).toDouble();
    c.drawPath(heart(q, 10), _f(_al(_re, .35 + .65 * ink)));
    c.save();
    c.clipPath(heart(q, 10));
    for (var k = 0; k < 40; k++) {
      if (_h(k * 7 + i * 131) > ink) c.drawCircle(q + Offset(_h(k + i * 53) * 24 - 12, _h(k * 3 + i * 17) * 22 - 12), 1.2 + _h(k) * 2.4, _f(_cr));
    }
    c.restore();
  }
  final top = at(0) + Offset(0, -14 - math.sin(t * 4).abs() * 6);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: top, width: 26, height: 7), const Radius.circular(2)), _f(_k1));
  c.drawRect(Rect.fromCenter(center: top + const Offset(0, -8), width: 6, height: 10), _f(_or));
  c.drawCircle(top + const Offset(0, -16), 6, _f(_or));
}

void _ec17(Canvas c, _St s, double t) {
  for (var i = 0; i < 6; i++) {
    final x = 160 - ((t * 140 + i * 30) % 190);
    c.drawLine(Offset(x, 104), Offset(x + 14, 104), _s(_al(_cr, .5), 2));
  }
  Path car(double x) => Path()
    ..moveTo(x - 30, 88)
    ..lineTo(x - 30, 78)
    ..lineTo(x - 14, 74)
    ..lineTo(x - 6, 64)
    ..lineTo(x + 12, 64)
    ..lineTo(x + 22, 74)
    ..lineTo(x + 30, 77)
    ..lineTo(x + 30, 88)
    ..close();
  final n = _n(s.a, 1, 10), gap = 6 + s.b * 14;
  const cols = [_cy, _pi, _vi];
  for (var i = n; i >= 1; i--) {
    c.drawPath(car(110 - i * gap), _s(_al(cols[i % 3], math.pow(.84, i).toDouble()), 1.8));
  }
  c.drawPath(car(110), _f(_ye));
  c.drawPath(_poly(const [Offset(106, 66), Offset(119, 66), Offset(126, 74), Offset(104, 74)]), _f(_k1));
  for (final x in const [92.0, 126.0]) {
    c.drawCircle(Offset(x, 89), 7, _f(_k2));
    c.drawCircle(Offset(x, 89), 3, _f(_cr));
  }
}

void _ec18(Canvas c, _St s, double t) {
  const tbl = Rect.fromLTWH(22, 24, 112, 72);
  c.drawRect(tbl, _f(_bl));
  c.drawRect(tbl, _s(_wh, 1.6));
  c.drawLine(const Offset(78, 20), const Offset(78, 100), _s(_cr, 2.2));
  c.drawLine(const Offset(22, 60), const Offset(134, 60), _s(_al(_wh, .4), .8));
  final n = _n(s.a, 1, 10), dec = .5 + s.b * .45;
  final hits = [for (var i = 0; i < 10; i++) Offset(i.isEven ? 110 - i * 2.0 : 46 + i * 2.0, 60 + math.sin(i * 1.9) * 22)];
  for (var i = n - 1; i >= 0; i--) {
    final o = math.pow(dec, i).toDouble(), q = hits[i], r = 4.6 * (.4 + .6 * o);
    if (i < n - 1) c.drawLine(hits[i], hits[i + 1], _s(_al(_cr, o * .35), 1));
    c.drawOval(Rect.fromCenter(center: q + const Offset(3, 4), width: r * 2, height: r), _f(_al(_k1, o * .4)));
    c.drawCircle(q, r, _f(_al(i.isEven ? _or : _ye, .25 + .75 * o)));
  }
  final ph = (t * 2) % 2, hand = ph < 1 ? -1.0 : 1.0;
  c.drawLine(const Offset(10, 60), const Offset(4, 70), _s(_k1, 4));
  c.drawCircle(Offset(12, 58 + hand * 3), 9, _f(_re));
  c.drawLine(const Offset(146, 60), const Offset(152, 70), _s(_k1, 4));
  c.drawCircle(Offset(144, 58 - hand * 3), 9, _f(_k1));
  c.drawCircle(Offset(144, 58 - hand * 3), 9, _s(_re, 2));
}

void _ec19(Canvas c, _St s, double t) {
  c.drawRect(const Rect.fromLTWH(0, 0, _pw, 26), _f(_pi));
  c.drawCircle(const Offset(78, 62), 22, _f(_ye));
  c.drawRect(const Rect.fromLTWH(0, 62, _pw, 58), _f(_bl));
  final n = _n(s.a, 2, 16), sp = 2.6 + s.b * 6;
  for (var i = 0; i < n; i++) {
    final y = 66 + i * sp, w = 44 * math.pow(.88, i).toDouble(), wob = math.sin(t * 2.4 + i * 1.3) * (2 + i * .4);
    final o = math.pow(.86, i).toDouble();
    if (y > 120) break;
    c.drawLine(Offset(78 - w / 2 + wob, y), Offset(78 + w / 2 + wob, y), _s(_al(_ye, o), 2.4));
  }
  for (var i = 0; i < 2; i++) {
    final q = Offset(30 + i * 16 + math.sin(t + i) * 4, 34.0 + i * 6);
    c.drawPath(Path()..moveTo(q.dx - 5, q.dy)..quadraticBezierTo(q.dx - 2, q.dy - 4, q.dx, q.dy)..quadraticBezierTo(q.dx + 2, q.dy - 4, q.dx + 5, q.dy), _s(_k1, 1.4));
  }
}

void _ec20(Canvas c, _St s, double t) {
  final n = _n(s.a, 3, 36), twist = .25 + s.b * 1.1;
  for (var i = n; i >= 0; i--) {
    final ang = i * twist + t * .6, r = 3 + i * 2.2, q = _pc + _pol(ang, r), sz = 3 + i * .32;
    final o = 1 - i / (n + 6);
    c.drawPath(_star(q, sz, 5, .42, ang * 2), _f(_al(_pop[i % 8], o)));
  }
  c.drawCircle(_pc, 4, _f(_wh));
}
