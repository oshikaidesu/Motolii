// Part of op_10_macro.dart (cut there by size, as it was).
part of 'op_10.dart';

void _mLanes(Canvas c, _S st, double t) {
  final pts = st.pts.length > 3 ? st.pts : [for (var i = 0; i <= 20; i++) Offset(8 + i * 4.0, 100 - math.pow(i / 20, 1.8) * 84)];
  _rect(c, const Rect.fromLTRB(8, 16, 88, 100), _dg, .8);
  _pl(c, pts, _wh, w: 1.2);
  final m = _wr(t * .15);
  final x = 8 + m * 80;
  var y = 100.0;
  for (var i = 1; i < pts.length; i++) {
    if ((pts[i - 1].dx - x) * (pts[i].dx - x) <= 0) {
      final f = (x - pts[i - 1].dx) / math.max(1e-3, pts[i].dx - pts[i - 1].dx);
      y = _lp(pts[i - 1].dy, pts[i].dy, f.clamp(0.0, 1.0));
      break;
    }
  }
  final v = ((100 - y) / 84).clamp(0.0, 1.0);
  _ln(c, Offset(x, 16), Offset(x, 100), _al(_gr, .6), .8);
  _dot(c, Offset(x, y), 2, _gr);
  const lo = [.1, .4, .0, .6], hi = [.9, .7, .5, 1.0], dir = [1, -1, 1, 1];
  for (var i = 0; i < 4; i++) {
    final ly = 22 + i * 20.0;
    _ln(c, Offset(98, ly), Offset(150, ly), _dg, .8);
    final x0 = 98 + lo[i] * 52, x1 = 98 + hi[i] * 52;
    _ln(c, Offset(x0, ly - 3), Offset(x0, ly + 3), _enc[i], 1);
    _ln(c, Offset(x1, ly - 3), Offset(x1, ly + 3), _enc[i], 1);
    _ln(c, Offset(x0, ly), Offset(x1, ly), _al(_enc[i], .5), 1);
    final vv = dir[i] > 0 ? v : 1 - v;
    _dot(c, Offset(_lp(x0, x1, vv), ly), 2.2, _enc[i]);
    _lab(c, ['size', 'blur', 'hue', 'speed'][i], Offset(98, ly - 11), _al(_enc[i], .7), size: 6.5);
  }
  _lab(c, 'macro', const Offset(8, 6), _gr);
  _lab(c, 'draw curve', const Offset(8, 106), _mg);
}

void _mSteps(Canvas c, _S st, double t) {
  final prob = st.a, seed = (st.b * 50).floor();
  final head = (t * 4).floor() % 16;
  for (var r = 0; r < 4; r++) {
    final y0 = 14 + r * 22.0;
    final pts = <Offset>[];
    for (var s = 0; s < 16; s++) {
      final x = 10 + s * 8.5;
      final on = _rn(seed * 64 + r * 16 + s + 500) < prob;
      final v = _rn(seed * 64 + r * 16 + s);
      final p = Offset(x, y0 + 16 - v * 14);
      if (on) {
        pts.add(p);
        _dot(c, p, s == head ? 2.4 : 1.4, s == head ? _wh : _enc[r]);
      } else {
        _ring(c, Offset(x, y0 + 16), .9, _dg, .7);
      }
    }
    _pl(c, pts, _al(_enc[r], .35), w: .8);
  }
  final hx = 10 + head * 8.5;
  _dots(c, Offset(hx, 10), Offset(hx, 100), _mg, gap: 3);
  _lab(c, 'prob ${_d2(prob * 99)}', const Offset(10, 104), _gr);
  _lab(c, 'rub to reroll · ${_d2(seed)}', const Offset(150, 104), _mg, ax: 1);
}

void _mKaleido(Canvas c, _S st, double t) {
  const ctr = Offset(70, 60);
  final seed = (st.a * 12).floor(), rot = st.a * 2 * math.pi, r = 20 + st.b * 30;
  final motif = [
    for (var i = 0; i < 5; i++) Offset(_rn(seed * 11 + i) * r, (_rn(seed * 13 + i) - .5) * r * .6 + math.sin(t + i) * 1.5),
  ];
  for (var k = 0; k < 6; k++) {
    for (final m in [1.0, -1.0]) {
      final an = rot + k * math.pi / 3;
      final ca = math.cos(an), sa = math.sin(an);
      final pts = [for (final p in motif) ctr + Offset(p.dx * ca - p.dy * m * sa, p.dx * sa + p.dy * m * ca)];
      _pl(c, pts, m > 0 ? _gr : _bl, w: .9);
      _dot(c, pts.last, 1.2, _rd);
    }
  }
  _ring(c, ctr, r + 6, _dg, .8);
  _lab(c, 'turn', const Offset(150, 6), _gr, ax: 1);
  _num(c, _d2(seed + 1), const Offset(150, 15), 26, _gr, ax: 1);
  _lab(c, 'bloom ${_d2(st.b * 99)}', const Offset(150, 108), _bl, ax: 1);
}

void _mRobot(Canvas c, _S st, double t) {
  final chaos = st.b, polite = st.a;
  const hc = Offset(64, 54);
  final j = Offset(math.sin(t * 23), math.cos(t * 19)) * chaos * chaos * 2;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hc + j, width: 64, height: 50), const Radius.circular(6)), _s(_wh, 1.2));
  final ant = hc + j - const Offset(0, 25);
  _ln(c, ant, ant - const Offset(0, 14), _wh, 1);
  _ring(c, ant - const Offset(0, 17), 3, _rd, 1);
  if (chaos > .5) {
    for (var i = 0; i < 4; i++) {
      final an = i * math.pi / 2 + t * 5;
      _ln(c, ant - const Offset(0, 17) + Offset(math.cos(an), math.sin(an)) * 5, ant - const Offset(0, 17) + Offset(math.cos(an), math.sin(an)) * 9,
          _al(_rd, chaos), .8);
    }
  }
  for (final sd in [-1.0, 1.0]) {
    final e = hc + j + Offset(sd * 14, -6);
    if (chaos > .6) {
      final p = Path();
      for (var i = 0; i < 24; i++) {
        final an = i * .5 + t * 6 * sd, rr = i * .3;
        final q = e + Offset(math.cos(an), math.sin(an)) * rr;
        i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
      }
      c.drawPath(p, _s(_bl, .9));
    } else {
      _ring(c, e, 6, _bl, 1.1);
      _ln(c, e - Offset(4, 0), e + Offset(4, 0) + Offset(0, (polite - .5) * 6), _bl, 1);
    }
  }
  for (var i = 0; i < 7; i++) {
    final x = hc.dx + j.dx - 15 + i * 5.0;
    final hgt = 2 + (polite * 4) * math.sin(i / 6 * math.pi) + chaos * _rn(i + (t * 8).floor()) * 6;
    _ln(c, Offset(x, hc.dy + j.dy + 14 - hgt / 2), Offset(x, hc.dy + j.dy + 14 + hgt / 2), _gr, 1);
  }
  _ln(c, const Offset(52, 79), const Offset(52, 92), _mg, 1);
  _ln(c, const Offset(76, 79), const Offset(76, 92), _mg, 1);
  _ln(c, const Offset(30, 92), const Offset(98, 92), _mg, 1);
  _lab(c, 'manners', const Offset(150, 6), _gr, ax: 1);
  _num(c, _d2(polite * 99), const Offset(150, 15), 18, _gr, ax: 1);
  _lab(c, 'chaos', const Offset(150, 84), _rd, ax: 1);
  _num(c, _d2(chaos * 99), const Offset(150, 93), 18, _rd, ax: 1);
}

void _mVolcano(Canvas c, _S st, double t) {
  const peakL = Offset(62, 44), peakR = Offset(82, 44);
  _pl(c, const [Offset(4, 96), Offset(40, 70), peakL, Offset(66, 48), Offset(76, 48), peakR, Offset(108, 72), Offset(150, 96)], _wh, w: 1.2);
  _ln(c, const Offset(0, 96), const Offset(_cw, 96), _dg, 1);
  final pr = st.b;
  final lvl = 92 - pr * 44;
  for (var i = 0; i < 4; i++) {
    final y = lvl + i * 5;
    if (y > 94) break;
    final half = _lp(42, 8, (94 - y) / 50);
    final p = Path()..moveTo(72 - half, y);
    for (var x = -half; x <= half; x += 3) {
      p.lineTo(72 + x, y + math.sin(x * .5 + t * 4 + i) * 1.2);
    }
    c.drawPath(p, _s(_al(_rd, .8 - i * .15), .9));
  }
  final since = t - st.rel;
  final seed = st.n;
  final erupting = !st.down && st.n > 0 && since < 2.4;
  final land = <Offset>[];
  for (var i = 0; i < 4; i++) {
    final vx = (_rn(seed * 4 + i) - .5) * 90, vy = -60 - _rn(seed * 4 + i + 2) * 40;
    final tt = math.min(since, 2.4) * .9;
    var p = Offset(72 + vx * tt, 44 + vy * tt + 70 * tt * tt);
    if (p.dy > 96 || !erupting) {
      final tl = (-vy + math.sqrt(vy * vy + 4 * 70 * 52)) / (2 * 70);
      p = Offset((72 + vx * tl).clamp(6, 150), 96);
    }
    land.add(p);
    if (erupting && p.dy < 96) {
      _ring(c, p, 2.4, _enc[i], 1.2);
    } else if (st.n > 0) {
      _dot(c, p, 2.2, _enc[i]);
      _ln(c, p, p - const Offset(0, 6), _al(_enc[i], .5), .8);
    }
  }
  if (erupting && since < .6) {
    for (var i = 0; i < 10; i++) {
      final an = -math.pi / 2 + (_rn(i + seed) - .5) * 1.4;
      _ln(c, const Offset(72, 44) + Offset(math.cos(an), math.sin(an)) * since * 30,
          const Offset(72, 44) + Offset(math.cos(an), math.sin(an)) * (since * 30 + 5), _rd, 1);
    }
  }
  _lab(c, 'pressure', const Offset(6, 6), _rd);
  _num(c, _d2(pr * 99), const Offset(6, 14), 20, _rd);
  _lab(c, 'drag up · let go', const Offset(150, 6), _mg, ax: 1);
  _four(c, [for (final p in land) st.n > 0 ? p.dx / _cw : 0], ['a', 'b', 'c', 'd'], y: 106);
}
