// Part of op_07_blend.dart (cut there by size, as it was).
part of 'op_07.dart';

void _waveMix(Canvas c, Size s, O7 st, double t) {
  final m = ((st.spin / 3).round() % 4 + 4) % 4;
  const names = ['MIX', 'ADD', 'MULT', 'DIFF'];
  double fa(double x) => math.sin(x * .09 + t * 2);
  double fb(double x) {
    final v = math.sin(x * .05 - t * 1.3) * 5;
    return v / (1 + v.abs());
  }

  double out(double x) {
    final a = fa(x), b = fb(x), k = st.a;
    return switch (m) {
      0 => a * (1 - k) + b * k,
      1 => (a + k * b) / (1 + k),
      2 => a * (1 - k + k * b),
      _ => (a - k * b).abs() - .5,
    };
  }

  final ga = <Offset>[], gb = <Offset>[], go = <Offset>[];
  for (var x = 26.0; x <= 148; x += 2) {
    ga.add(Offset(x, 18 + fa(x) * 7));
    gb.add(Offset(x, 42 + fb(x) * 7));
    go.add(Offset(x, 88 + out(x) * 16));
  }
  for (final y in [18.0, 42.0, 88.0]) {
    _dash(c, Offset(26, y), Offset(148, y), _s(_dg, 1), on: 1, off: 3);
  }
  _pl(c, ga, _s(_bu, 1.1));
  _pl(c, gb, _s(_gn, 1.1));
  _pl(c, go, _s(_wt, 1.3));
  _lab(c, 'A', const Offset(8, 14), _bu);
  _lab(c, 'B', const Offset(8, 38), _gn);
  _lab(c, 'OUT', const Offset(8, 84), _wt);
  _lab(c, names[m], const Offset(8, 60), _rd, size: 8);
  _num(c, _pct(st.a), const Offset(150, 56), 12, _rd, ax: 1);
}

void _teaCup(Canvas c, Size s, O7 st, double t) {
  final k = 1 - st.b;
  final ln = _s(_wt, 1.1);
  final cup = Path()
    ..moveTo(36, 60)
    ..lineTo(44, 96)
    ..quadraticBezierTo(46, 104, 56, 104)
    ..lineTo(88, 104)
    ..quadraticBezierTo(98, 104, 100, 96)
    ..lineTo(108, 60);
  c.save();
  c.clipPath(Path()
    ..addPath(cup, Offset.zero)
    ..close());
  final step = 9 - k * 6.5;
  for (var y = 70.0; y < 106; y += step) {
    c.drawLine(Offset(30, y), Offset(114, y + math.sin(t + y) * .8), _s(_o(_rd, .2 + .8 * k), 1));
  }
  final by = 30 + k * 56;
  for (var i = 0; i < 6; i++) {
    final r = ((t * 8 + i * 7) % 26) + 4, an = i * 1.05 + t * .3;
    if (by > 66) _dot(c, Offset(72, by) + Offset(math.cos(an), math.sin(an) * .5) * r, _o(_rd, (1 - r / 30) * k), 1);
  }
  c.restore();
  c.drawPath(cup, ln);
  c.drawOval(const Rect.fromLTRB(36, 56, 108, 66), ln);
  c.drawArc(const Rect.fromLTRB(100, 66, 120, 90), -math.pi / 2, math.pi, false, ln);
  c.drawOval(const Rect.fromLTRB(22, 102, 122, 114), _s(_o(_wt, .6), 1));
  _pl(c, [const Offset(124, 12), Offset(124, 22), Offset(78, by - 8)], _s(_o(_wt, .7), 1));
  c.drawRect(const Rect.fromLTRB(118, 10, 132, 20), _s(_gn, 1));
  final bag = RRect.fromLTRBR(65, by - 8, 79, by + 8, const Radius.circular(2));
  c.drawRRect(bag, _f(_bg));
  c.drawRRect(bag, _s(_gn, 1.1));
  for (var y = by - 4; y < by + 6; y += 3) {
    _dot(c, Offset(72, y), _gn, .7);
  }
  for (var i = 0; i < 3; i++) {
    _pl(c, [for (var j = 0; j <= 10; j++) Offset(56 + i * 14 + math.sin(j * .7 - t * 3 + i) * 2.5, 52 - j * 3.6)], _s(_o(_wt, .25 + .35 * k), 1));
  }
  _num(c, _pct(k), const Offset(8, 8), 18, _rd);
  _lab(c, 'STRENGTH', const Offset(8, 30), _rd);
}

void _xrayHand(Canvas c, Size s, O7 st, double t) {
  final skinOp = st.b;
  final bone = _s(_o(_gn, .4 + .6 * (1 - skinOp)), 1.3);
  const xs = [58.0, 70.0, 82.0, 94.0], lens = [34.0, 42.0, 40.0, 30.0];
  for (var i = 0; i < 4; i++) {
    final top = 64 - lens[i];
    for (var j = 0; j < 3; j++) {
      final y0 = top + 4 + j * (lens[i] - 6) / 3, y1 = y0 + (lens[i] - 6) / 3 - 2;
      c.drawLine(Offset(xs[i], y0), Offset(xs[i], y1), bone);
      _dot(c, Offset(xs[i], y1 + 1), _o(_gn, .5 + .5 * (1 - skinOp)), 1);
    }
    c.drawLine(Offset(xs[i], 68), Offset(76 + (xs[i] - 76) * .4, 100), bone);
  }
  c.drawLine(const Offset(50, 90), const Offset(40, 76), bone);
  c.drawLine(const Offset(39, 74), const Offset(32, 64), bone);
  c.drawLine(const Offset(62, 104), const Offset(92, 104), bone);
  final skin = Path();
  for (var i = 0; i < 4; i++) {
    skin.addRRect(RRect.fromLTRBR(xs[i] - 5.5, 64 - lens[i], xs[i] + 5.5, 72, const Radius.circular(5.5)));
  }
  skin.addRRect(RRect.fromLTRBR(52, 64, 100, 116, const Radius.circular(10)));
  const ta = Offset(56, 96), tb = Offset(32, 62);
  final u = (tb - ta) / (tb - ta).distance, n = Offset(-u.dy, u.dx) * 5.5;
  skin.addPath(
      Path()
        ..moveTo((ta + n).dx, (ta + n).dy)
        ..lineTo((tb + n).dx, (tb + n).dy)
        ..arcToPoint(tb - n, radius: const Radius.circular(5.5))
        ..lineTo((ta - n).dx, (ta - n).dy)
        ..close(),
      Offset.zero);
  _occ(c, skin, skinOp * .95);
  c.drawPath(skin, _s(_o(_wt, .2 + .8 * skinOp), 1.1));
  final sy = 14 + (t * 34) % 96;
  c.drawLine(Offset(20, sy), Offset(136, sy), _s(_o(_rd, .8), 1));
  _lab(c, '180 KV', const Offset(150, 6), _rd, ax: 1);
  _num(c, _pct(skinOp), const Offset(8, 8), 16, _wt);
  _lab(c, 'SKIN', const Offset(8, 28), _wt);
}

void _onionMonkey(Canvas c, Size s, O7 st, double t) {
  c.drawLine(const Offset(0, 102), Offset(s.width, 102), _s(_dg));
  void monkey(double x, double ph, Color col) {
    final p = _s(col, 1.1);
    const hy = 56.0;
    final bob = math.sin(ph * 12).abs() * 3;
    final head = Offset(x, hy - bob);
    c.drawCircle(head, 6, p);
    c.drawCircle(head + const Offset(-7, -1), 2.5, p);
    c.drawCircle(head + const Offset(7, -1), 2.5, p);
    c.drawArc(Rect.fromCenter(center: head + const Offset(0, 2), width: 7, height: 5), 0, math.pi, false, p);
    final hip = Offset(x - 2, 82 - bob);
    c.drawLine(head + const Offset(0, 6), hip, p);
    final sw = math.sin(ph * 12);
    final sh = head + const Offset(0, 11);
    c.drawLine(sh, sh + Offset(10 * sw, 8), p);
    c.drawLine(sh, sh + Offset(-10 * sw, 8), p);
    _pl(c, [hip, hip + Offset(6 * sw, 9), Offset(x - 2 + 10 * sw, 101)], p);
    _pl(c, [hip, hip + Offset(-6 * sw, 9), Offset(x - 2 - 10 * sw, 101)], p);
    final tail = Path()
      ..moveTo(hip.dx, hip.dy)
      ..quadraticBezierTo(hip.dx - 16, hip.dy + 2, hip.dx - 12, hip.dy - 12)
      ..quadraticBezierTo(hip.dx - 9, hip.dy - 18, hip.dx - 5, hip.dy - 13);
    c.drawPath(tail, p);
  }

  final x0 = 24 + (t * 26) % 130;
  for (var k = 5; k >= 0; k--) {
    final x = x0 - k * 13;
    if (x < 6) continue;
    final al = k == 0 ? 1.0 : math.pow(st.a, k).toDouble();
    if (al < .03) continue;
    monkey(x, t - k * .09, k == 0 ? _wt : _o(k.isEven ? _pu : _bu, al));
  }
  _num(c, _pct(st.a), const Offset(150, 8), 18, _bu, ax: 1);
  _lab(c, 'ONION', const Offset(150, 30), _bu, ax: 1);
}

const _bayer = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

void _ditherGrid(Canvas c, Size s, O7 st, double t) {
  c.drawCircle(Offset(78 + math.sin(t * .6) * 6, 56), 34, _s(_wt, 1.2));
  c.drawLine(const Offset(14, 100), const Offset(142, 14), _s(_rd, 1.2));
  const ox = 13.0, oy = 10.0, cs = 10.0;
  final painted = <int>{};
  for (final m in st.marks) {
    final gx = ((m.dx * s.width - ox) / cs).floor(), gy = ((m.dy * s.height - oy) / cs).floor();
    painted.add(gx * 100 + gy);
  }
  final dots = <Offset>[];
  var sum = 0.0;
  for (var gx = 0; gx < 13; gx++) {
    for (var gy = 0; gy < 9; gy++) {
      final v = painted.contains(gx * 100 + gy) ? 1.0 : (gx / 12) * st.b;
      sum += v;
      final r = Rect.fromLTWH(ox + gx * cs, oy + gy * cs, cs, cs);
      if (v > 0) c.drawRect(r, _f(_o(_bg, v)));
      for (var i = 0; i < 16; i++) {
        if (v * 16 > _bayer[i] + .5) dots.add(r.topLeft + Offset(1.6 + (i % 4) * 2.3, 1.6 + (i ~/ 4) * 2.3));
      }
    }
  }
  c.drawPoints(ui.PointMode.points, dots, _s(_gn, 1));
  final g = _s(_o(_dg, .8), 1);
  for (var gx = 0; gx <= 13; gx++) {
    c.drawLine(Offset(ox + gx * cs, oy), Offset(ox + gx * cs, oy + 90), g);
  }
  for (var gy = 0; gy <= 9; gy++) {
    c.drawLine(Offset(ox, oy + gy * cs), Offset(ox + 130, oy + gy * cs), g);
  }
  _lab(c, 'DRAW', const Offset(13, 104), _gn);
  _num(c, _pct(sum / 117), const Offset(143, 103), 10, _gn, ax: 1, w: 1);
}

void _doubleExposure(Canvas c, Size s, O7 st, double t) {
  final sky = <Offset>[const Offset(10, 104)];
  var x = 10.0;
  for (var i = 0; x < 146; i++) {
    final w = 8 + _h(i + 3) * 10, top = 104 - 14 - _h(i + 60) * 40;
    sky
      ..add(Offset(x, top))
      ..add(Offset(x + w, top));
    x += w;
  }
  sky.add(const Offset(146, 104));
  _pl(c, sky, _s(_o(_bu, 1 - st.a * .4), 1.1));
  const prof = [
    Offset(40, 10), Offset(52, 14), Offset(58, 30), Offset(56, 38), Offset(64, 48), Offset(58, 52), Offset(60, 58),
    Offset(56, 62), Offset(58, 68), Offset(52, 72), Offset(50, 80), Offset(38, 86), Offset(32, 100),
  ];
  const back = [Offset(40, 10), Offset(20, 16), Offset(10, 36), Offset(12, 60), Offset(20, 78), Offset(22, 100)];
  final drift = math.sin(t * .5) * 2;
  Offset m(Offset p) => Offset(p.dx * 1.0 + 34 + drift, p.dy * 1.0 + 4);
  final pp = _s(_o(_gn, st.a), 1.2);
  _pl(c, prof.map(m).toList(), pp);
  _pl(c, back.map(m).toList(), pp);
  final cm = _s(_wt, 1.2);
  for (final cn in const [Offset(6, 6), Offset(150, 6), Offset(6, 114), Offset(150, 114)]) {
    final sx = cn.dx < 78 ? 1.0 : -1.0, sy = cn.dy < 60 ? 1.0 : -1.0;
    _pl(c, [cn + Offset(0, 8 * sy), cn, cn + Offset(8 * sx, 0)], cm);
  }
  if ((t * 1.5) % 1 < .6) _dot(c, const Offset(16, 16), _rd, 2.2);
  _lab(c, 'EXP 2', const Offset(22, 13), _rd);
  _num(c, _pct(st.a), const Offset(146, 96), 12, _wt, ax: 1);
}

void _modeReel(Canvas c, Size s, O7 st, double t) {
  const bm = [BlendMode.srcOver, BlendMode.multiply, BlendMode.screen, BlendMode.plus, BlendMode.difference, BlendMode.overlay];
  final off = -st.spin * .35;
  final base = off.floor(), frac = off - base;
  for (var k = -3; k <= 3; k++) {
    final y = 54 + (k - frac) * 17;
    if (y < 8 || y > 104) continue;
    final idx = ((base + k) % 6 + 6) % 6;
    final near = (y - 54).abs() < 8.5;
    _lab(c, _modes[idx], Offset(10, y), near ? _wt : (y - 54).abs() < 26 ? const Color(0xFF6A6A72) : _dg,
        size: near ? 11 : 9, w: FontWeight.w300);
  }
  final sel = ((off.round()) % 6 + 6) % 6;
  final br = _s(_gn, 1.1);
  _pl(c, const [Offset(6, 50), Offset(4, 50), Offset(4, 68), Offset(6, 68)], br);
  _pl(c, const [Offset(92, 50), Offset(94, 50), Offset(94, 68), Offset(92, 68)], br);
  c.saveLayer(const Rect.fromLTRB(100, 20, 156, 100), Paint());
  for (var y = 37.0; y < 62; y += 2) {
    c.drawLine(Offset(104, y), Offset(130, y), _s(_bu, 1));
  }
  final pb = _s(_rd, 1.4)..blendMode = bm[sel];
  for (var x = 119.0; x < 144; x += 2) {
    c.drawLine(Offset(x, 50), Offset(x, 76), pb);
  }
  c.restore();
  c.drawRect(const Rect.fromLTWH(104, 36, 26, 26), _s(_o(_wt, .5), 1));
  c.drawRect(const Rect.fromLTWH(118, 50, 26, 26), _s(_o(_wt, .5), 1));
  _lab(c, 'A', const Offset(104, 26), _bu);
  _lab(c, 'B', const Offset(140, 80), _rd);
  _num(c, '0${sel + 1}', const Offset(150, 104), 10, _gn, ax: 1, w: 1);
}

void _cheshire(Canvas c, Size s, O7 st, double t) {
  final a = st.a;
  final body = _ss(.6, .9, a), head = _ss(.35, .65, a), eyes = _ss(.12, .4, a);
  final branch = Path()
    ..moveTo(0, 100)
    ..quadraticBezierTo(78, 88, 156, 96);
  c.drawPath(branch, _s(_o(_wt, .7), 1.1));
  for (final x in [24.0, 120.0]) {
    c.drawLine(Offset(x, 96), Offset(x + 8, 88), _s(_o(_gn, .7), 1));
  }
  final bob = math.sin(t * 1.4) * 2 * (1 - a);
  if (body > 0) {
    final bp = _s(_o(_pu, body), 1.1);
    c.drawOval(const Rect.fromLTRB(64, 64, 96, 94), bp);
    final tail = <Offset>[for (var i = 0; i <= 14; i++) Offset(96 + i * 2.2, 90 + i * 1.4 + math.sin(i * .6 + t * 2) * 3)];
    _pl(c, tail, bp);
    for (var i = 2; i < 14; i += 3) {
      c.drawLine(tail[i] + const Offset(0, -3), tail[i] + const Offset(0, 3), bp);
    }
  }
  const hc = Offset(80, 50);
  if (head > 0) {
    final hp = _s(_o(_pu, head), 1.1);
    c.drawOval(Rect.fromCenter(center: hc, width: 46, height: 36), hp);
    _pl(c, [hc + const Offset(-20, -10), hc + const Offset(-18, -28), hc + const Offset(-6, -17)], hp);
    _pl(c, [hc + const Offset(20, -10), hc + const Offset(18, -28), hc + const Offset(6, -17)], hp);
    for (final sx in [-1.0, 1.0]) {
      for (var i = 0; i < 3; i++) {
        c.drawLine(hc + Offset(sx * 16, 4.0 + i * 3), hc + Offset(sx * 34, 0.0 + i * 5), _s(_o(_wt, head * .7), 1));
      }
    }
  }
  if (eyes > 0) {
    final ep = _s(_o(_gn, eyes), 1.1);
    for (final sx in [-1.0, 1.0]) {
      final e = hc + Offset(sx * 9, -5);
      c.drawOval(Rect.fromCenter(center: e, width: 9, height: 7), ep);
      c.drawLine(e + const Offset(0, -3), e + const Offset(0, 3), ep);
    }
  }
  final g = hc + Offset(0, 6 + bob);
  final grin = Path()
    ..moveTo(g.dx - 17, g.dy - 2)
    ..quadraticBezierTo(g.dx, g.dy + 16, g.dx + 17, g.dy - 2)
    ..quadraticBezierTo(g.dx, g.dy + 5, g.dx - 17, g.dy - 2);
  c.drawPath(grin, _s(_rd, 1.2));
  for (var i = -3; i <= 3; i++) {
    final x = g.dx + i * 4.0;
    final y0 = g.dy + 1.5 + (1 - (i / 4) * (i / 4)) * 3, y1 = y0 + 2.5 * (1 - (i / 4).abs());
    c.drawLine(Offset(x, y0), Offset(x, y1 + 1), _s(_rd, 1));
  }
  _num(c, _pct(a), const Offset(8, 8), 18, _bu);
  _lab(c, 'OPACITY', const Offset(8, 30), _bu);
}

void _whale(Canvas c, Size s, O7 st, double t) {
  final d = 1 - st.b;
  const surf = 34.0;
  final wave = [for (var x = 0.0; x <= s.width; x += 4) Offset(x, surf + math.sin(x * .12 + t * 2) * 1.5)];
  for (var i = 0; i < 3; i++) {
    final bx = 30 + i * 22 + math.sin(t * .8 + i) * 4, by = 12 + i * 3.0;
    _pl(c, [Offset(bx - 4, by - 2), Offset(bx, by), Offset(bx + 4, by - 2)], _s(_o(_wt, .5), 1));
  }
  final y = 48 + d * 50, x0 = 34 + math.sin(t * .4) * 6;
  final body = Path()
    ..moveTo(x0, y)
    ..quadraticBezierTo(x0 + 34, y - 20, x0 + 72, y - 8)
    ..quadraticBezierTo(x0 + 86, y, x0 + 72, y + 9)
    ..quadraticBezierTo(x0 + 34, y + 14, x0, y)
    ..moveTo(x0, y)
    ..lineTo(x0 - 12, y - 9)
    ..moveTo(x0, y)
    ..lineTo(x0 - 11, y + 7);
  final wp = _s(_wt, 1.2);
  c.drawPath(body, wp);
  for (var i = 0; i < 4; i++) {
    c.drawLine(Offset(x0 + 50 + i * 4.0, y + 5), Offset(x0 + 56 + i * 4.0, y + 8), _s(_wt, .9));
  }
  _dot(c, Offset(x0 + 66, y - 2), _wt, 1.2);
  for (var wy = surf + 6; wy < y + 14; wy += 7) {
    if (wy > y - 18) {
      c.drawRect(Rect.fromLTWH(0, wy, s.width, 7), _f(_o(_bg, .16)));
    }
    _dash(c, Offset(0, wy), Offset(s.width, wy), _s(_o(_bu, .45), 1), on: 3, off: 4);
  }
  _pl(c, wave, _s(_bu, 1.2));
  if (d < .15) {
    for (var i = 0; i < 9; i++) {
      final u = (t * 1.6 + i / 9) % 1, an = -math.pi / 2 + (i - 4) * .16;
      _dot(c, Offset(x0 + 62, surf) + Offset(math.cos(an), math.sin(an)) * u * 26, _o(_gn, 1 - u), 1.1);
    }
  }
  _num(c, _pct(st.b), const Offset(150, 6), 16, _gn, ax: 1);
  _lab(c, 'SURFACE', const Offset(108, 24), _gn);
}
