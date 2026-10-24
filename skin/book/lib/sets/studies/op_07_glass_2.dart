// Part of op_07_glass.dart (cut there by size, as it was).
part of 'op_07.dart';

void _diamond(Canvas c, Size s, O7 st, double t) {
  const cx = 78.0, tableY = 34.0, girdle = 50.0, culet = Offset(78, 100);
  final rot = st.spin * .25 + t * .15;
  final ln = _s(_wt, 1.1);
  final gx = <double>[], tx = <double>[];
  for (var j = 0; j < 8; j++) {
    final ph = j * math.pi / 4 + rot;
    if (math.sin(ph) > -.05) {
      gx.add(cx + math.cos(ph) * 40);
      tx.add(cx + math.cos(ph + math.pi / 8) * 22);
    }
  }
  c.drawLine(const Offset(cx - 22, tableY), const Offset(cx + 22, tableY), ln);
  c.drawLine(const Offset(cx - 22, tableY), const Offset(cx - 40, girdle), ln);
  c.drawLine(const Offset(cx + 22, tableY), const Offset(cx + 40, girdle), ln);
  c.drawLine(const Offset(cx - 40, girdle), const Offset(cx + 40, girdle), ln);
  c.drawLine(const Offset(cx - 40, girdle), culet, ln);
  c.drawLine(const Offset(cx + 40, girdle), culet, ln);
  final fl = _s(_o(_wt, .55), 1);
  for (var i = 0; i < gx.length; i++) {
    c.drawLine(Offset(gx[i], girdle), culet, fl);
    c.drawLine(Offset(gx[i], girdle), Offset(tx[i].clamp(cx - 22, cx + 22), tableY), fl);
  }
  final k = st.b;
  const cols = [_bu, _gn, _rd, _wt, _pu];
  final count = (3 + k * 12).round();
  for (var i = 0; i < count; i++) {
    final ph = (t * (1.5 + _h(i) * 2) + _h(i + 7) * 6 + rot * 2) % (math.pi * 2);
    final g = math.max(0.0, math.sin(ph));
    if (g < .2) continue;
    final q = Offset(cx + (_h(i + 11) - .5) * 70 * (1 - _h(i + 13) * .5), 32 + _h(i + 17) * 56);
    final len = (3 + 9 * g) * (.5 + k);
    final p = _s(_o(cols[i % 5], g), 1);
    c.drawLine(q - Offset(len, 0), q + Offset(len, 0), p);
    c.drawLine(q - Offset(0, len), q + Offset(0, len), p);
  }
  _num(c, (1.3 + k * 1.2).toStringAsFixed(2), const Offset(150, 6), 14, _bu, ax: 1);
  _lab(c, 'IOR', const Offset(150, 24), _bu, ax: 1, size: 6.5);
  _lab(c, 'SPIN', const Offset(6, 108), _o(_wt, .6), size: 6.5);
}

void _fluted(Canvas c, Size s, O7 st, double t) {
  final count = 4 + (st.a * 14).round();
  const x0 = 8.0, x1 = 148.0;
  final w = (x1 - x0) / count, sx = 1 - st.b * .75;
  void sample() {
    c.drawCircle(Offset(78 + math.sin(t * .5) * 10, 58), 30, _s(_wt, 1.3));
    c.drawLine(const Offset(10, 104), const Offset(146, 14), _s(_rd, 1.3));
    c.drawRect(const Rect.fromLTWH(96, 70, 22, 22), _s(_gn, 1.3));
  }

  for (var i = 0; i < count; i++) {
    final r = Rect.fromLTRB(x0 + i * w, 6, x0 + (i + 1) * w, 114);
    c.save();
    c.clipRect(r);
    c.translate(r.center.dx, 0);
    c.scale(sx, 1);
    c.translate(-r.center.dx, 0);
    sample();
    c.restore();
    c.drawLine(r.topLeft, r.bottomLeft, _s(_o(_wt, .45), 1));
    c.drawLine(r.topLeft + Offset(w * .22, 0), r.bottomLeft + Offset(w * .22, 0), _s(_o(_bu, .35), 1));
  }
  c.drawRect(const Rect.fromLTRB(x0, 6, x1, 114), _s(_wt, 1.1));
  _lab(c, '$count FLUTES', const Offset(10, 108), _bu, size: 6.5);
  _lab(c, 'DEPTH ${_pct(st.b)}', const Offset(146, 108), _wt, ax: 1, size: 6.5);
}

void _dropWord(Canvas c, Size s, O7 st, double t) {
  final big = _tp('GLASS', _wt, 34, FontWeight.w200);
  const bo = Offset(10, 26);
  final small = _tp('INDEX 1.33  ROUGH 00', _o(_wt, .5), 6.5, FontWeight.w400);
  const so = Offset(12, 72);
  void words(Color? tint) {
    if (tint == null) {
      big.paint(c, bo);
      small.paint(c, so);
    } else {
      _tp('GLASS', tint, 34, FontWeight.w200).paint(c, bo);
      _tp('INDEX 1.33  ROUGH 00', tint, 6.5, FontWeight.w400).paint(c, so);
    }
    for (var x = 10.0; x < 148; x += 4) {
      c.drawLine(Offset(x, 90), Offset(x, x % 20 < 4 ? 82 : 86), _s(_o(tint ?? _wt, .5), 1));
    }
  }

  words(null);
  final dc = Offset(st.p.dx * s.width, st.p.dy * s.height) + Offset(0, math.sin(t * 2) * .8);
  const r = 18.0;
  final m = 1.3 + _pinchK(st) * 1.4;
  final drop = Path()..addOval(Rect.fromCircle(center: dc, radius: r));
  c.save();
  c.clipPath(drop);
  c.drawRect(Offset.zero & s, _f(_bg));
  c.translate(dc.dx, dc.dy);
  c.scale(m, m);
  c.translate(-dc.dx, -dc.dy);
  words(_bu);
  c.restore();
  c.drawPath(drop, _s(_wt, 1.2));
  c.drawArc(Rect.fromCircle(center: dc, radius: r - 4), math.pi * 1.1, .8, false, _s(_wt, 1));
  c.drawArc(Rect.fromCircle(center: dc + const Offset(1.5, 1.5), radius: r), .2, 1.2, false, _s(_o(_wt, .3), 1));
  _num(c, m.toStringAsFixed(1), const Offset(150, 104), 10, _bu, ax: 1, w: 1);
  _lab(c, 'MAG', const Offset(6, 6), _bu, size: 6.5);
}

void _soapFilm(Canvas c, Size s, O7 st, double t) {
  const ctr = Offset(78, 60);
  const r = 46.0;
  final base = 80 + st.b * 620;
  const pal = [_bu, _gn, _wt, _rd, _pu];
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  for (var y = ctr.dy - r; y < ctr.dy + r; y += 2.6) {
    final f = (y - (ctr.dy - r)) / (2 * r);
    final th = base * (.25 + .75 * f) + 50 * math.sin(y * .08 + t * 1.1);
    if (th < 110) continue;
    final col = pal[(th / 85).floor() % 5];
    _pl(c, [for (var x = ctr.dx - r; x <= ctr.dx + r; x += 6) Offset(x, y + math.sin(x * .07 + t * 1.4 + f * 4) * 2.2)],
        _s(_o(col, .85), 1));
  }
  c.restore();
  c.drawCircle(ctr, r, _s(_wt, 1.2));
  c.drawArc(Rect.fromCircle(center: ctr, radius: r - 6), math.pi * 1.15, .6, false, _s(_wt, 1.4));
  c.drawCircle(ctr + const Offset(-20, -24), 2, _s(_wt, 1));
  _num(c, base.round().toString(), const Offset(150, 96), 14, _wt, ax: 1);
  _lab(c, 'NM', const Offset(150, 86), _wt, ax: 1, size: 6.5);
  _lab(c, 'THICK', const Offset(6, 6), _wt);
}

void _peephole(Canvas c, Size s, O7 st, double t) {
  final ln = _s(_wt, 1.1);
  c.drawRect(const Rect.fromLTRB(6, 6, 48, 120), ln);
  c.drawRect(const Rect.fromLTRB(12, 60, 42, 108), _s(_o(_wt, .4), 1));
  c.drawCircle(const Offset(27, 36), 3, _s(_bu, 1.1));
  c.drawCircle(const Offset(40, 70), 2, ln);
  const v = Offset(104, 60);
  const vr = 46.0;
  final k = .1 + st.b * 3.2;
  Offset fe(Offset q) {
    final d = q.distance;
    if (d < 1e-4) return v;
    final r2 = d * (1 + k) / (1 + k * d);
    return v + q / d * r2 * vr;
  }

  List<Offset> loop(Offset ctr, double rx, double ry, [int n = 24]) =>
      [for (var i = 0; i <= n; i++) fe(ctr + Offset(math.cos(i / n * math.pi * 2) * rx, math.sin(i / n * math.pi * 2) * ry))];
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: v, radius: vr)));
  for (final q in const [Offset(-1, -1), Offset(1, -1), Offset(-1, 1), Offset(1, 1)]) {
    _pl(c, _seg(q * 1.2, q * .5, 10, fe), _s(_o(_pu, .6), 1));
  }
  _pl(c, _seg(const Offset(-.5, .5), const Offset(.5, .5), 12, fe), _s(_o(_pu, .6), 1));
  final lean = math.sin(t * .8) * .05;
  final hc = Offset(lean, -.08);
  final fp = _s(_wt, 1.1);
  _pl(c, loop(hc, .26, .32), fp);
  _pl(c, _seg(hc + const Offset(-.4, -.24), hc + const Offset(.4, -.24), 12, fe), _s(_gn, 1.1));
  _pl(c, loop(hc + const Offset(0, -.36), .22, .14, 16), _s(_gn, 1.1));
  _pl(c, loop(hc + const Offset(-.1, -.06), .035, .035, 8), fp);
  _pl(c, loop(hc + const Offset(.1, -.06), .035, .035, 8), fp);
  _pl(c, _seg(hc + const Offset(-.08, .14), hc + const Offset(.08, .14), 6, fe), fp);
  _pl(c, _seg(hc + const Offset(0, -.02), hc + const Offset(.02, .08), 4, fe), fp);
  _pl(c, [..._seg(const Offset(-.8, 1), hc + const Offset(-.3, .36), 8, fe), ..._seg(hc + const Offset(.3, .36), const Offset(.8, 1), 8, fe)], fp);
  c.restore();
  c.drawCircle(v, vr, _s(_wt, 1.2));
  c.drawCircle(v, vr + 3, _s(_o(_wt, .35), 1));
  _num(c, _pct(st.b), const Offset(12, 10), 12, _bu);
  _lab(c, 'BULGE', const Offset(12, 26), _bu, size: 6.5);
}

void _laserSlab(Canvas c, Size s, O7 st, double t) {
  final w = (24 * st.pinch).clamp(10.0, 70.0);
  final n = 1 + st.b * .9;
  final x0 = 80 - w / 2, x1 = 80 + w / 2;
  c.drawRect(const Rect.fromLTRB(4, 28, 26, 40), _s(_rd, 1.1));
  _lab(c, 'LASER', const Offset(4, 18), _rd, size: 6.5);
  const src = Offset(26, 34);
  final y0 = 52.0;
  final th1 = math.atan2(y0 - src.dy, x0 - src.dx);
  final th2 = math.asin(math.sin(th1) / n);
  final ye = y0 + w * math.tan(th2);
  final beam = _s(_rd, 1.3);
  c.drawLine(src, Offset(x0, y0), beam);
  c.drawLine(Offset(x0, y0), Offset(x1, ye), _s(_o(_rd, .8), 1.1));
  _dash(c, Offset(x1, ye), Offset(x1, ye) + Offset(math.cos(th1), math.sin(th1)) * 66, _s(_o(_wt, .25), 1), on: 1, off: 3);
  final spread = st.a * .7;
  for (var i = -3; i <= 3; i++) {
    final an = th1 + i / 3 * spread;
    final al = i == 0 ? 1.0 : (1 - st.a * .3) * (1 - i.abs() / 4) * math.min(1, st.a * 4);
    if (al < .03) continue;
    c.drawLine(Offset(x1, ye), Offset(x1, ye) + Offset(math.cos(an), math.sin(an)) * 70, _s(_o(i == 0 ? _rd : _gn, al), 1));
  }
  final slab = Rect.fromLTRB(x0, 14, x1, 108);
  c.drawRect(slab, _s(_wt, 1.2));
  for (var y = 18.0; y < 108; y += 6) {
    c.drawLine(Offset(x0 + 2, y), Offset(x0 + 5, y - 3), _s(_o(_wt, .3), 1));
  }
  final speck = [for (var i = 0; i < (st.a * 26).round(); i++) Offset(x1 - 1 - _h(i) * 3, 16 + _h(i + 40) * 90)];
  c.drawPoints(ui.PointMode.points, speck, _s(_gn, 1.2));
  final u = (t * .8) % 1;
  _dot(c, Offset.lerp(src, Offset(x0, y0), u)!, _wt, 1.3);
  _num(c, n.toStringAsFixed(2), const Offset(150, 6), 12, _bu, ax: 1);
  _lab(c, 'INDEX', const Offset(150, 22), _bu, ax: 1, size: 6.5);
  _lab(c, 'THICK ${w.round()}', const Offset(80, 110), _wt, ax: .5, size: 6.5);
  _lab(c, 'ROUGH', const Offset(4, 108), _gn, size: 6.5);
}

void _einsteinRing(Canvas c, Size s, O7 st, double t) {
  final field = [for (var i = 0; i < 46; i++) Offset(_h(i + 300) * s.width, _h(i + 400) * s.height)];
  c.drawPoints(ui.PointMode.points, field, _s(_dg, 1.2));
  const lc = Offset(78, 60);
  const te = 26.0;
  final src = Offset(st.p.dx * s.width, st.p.dy * s.height);
  final beta = src - lc;
  final b = beta.distance;
  final dir = b < .01 ? 0.0 : math.atan2(beta.dy, beta.dx);
  final rp = (b + math.sqrt(b * b + 4 * te * te)) / 2, rm = (math.sqrt(b * b + 4 * te * te) - b) / 2;
  final span = math.min(math.pi * 2, 1.2 * te / (b + 1));
  final pulse = .8 + .2 * math.sin(t * 2);
  for (var i = 0; i < 3; i++) {
    final dr = (i - 1) * 1.6;
    final col = [_bu, _wt, _pu][i];
    c.drawArc(Rect.fromCircle(center: lc, radius: rp + dr), dir - span / 2, span, false, _s(_o(col, pulse), 1.1));
    c.drawArc(Rect.fromCircle(center: lc, radius: rm + dr), dir + math.pi - span * rm / rp / 2, span * rm / rp, false,
        _s(_o(col, pulse * .8), 1));
  }
  c.drawOval(Rect.fromCenter(center: lc, width: 12, height: 6), _s(_rd, 1.1));
  _dot(c, lc, _rd, 1.6);
  final sp = _s(_o(_wt, .35), 1);
  for (var i = 0; i < 3; i++) {
    final an = t * .6 + i * math.pi * 2 / 3;
    c.drawArc(Rect.fromCircle(center: src, radius: 5), an, 1.4, false, sp);
  }
  _dash(c, src, lc, _s(_o(_wt, .15), 1), on: 1, off: 3);
  _num(c, _pct(1 - _cl(b / 60)), const Offset(150, 6), 14, _bu, ax: 1);
  _lab(c, 'ALIGN', const Offset(150, 24), _bu, ax: 1, size: 6.5);
  _lab(c, 'MASS', const Offset(6, 108), _rd, size: 6.5);
}

void _crystalBall(Canvas c, Size s, O7 st, double t) {
  const bc = Offset(78, 50);
  const br = 32.0;
  final sx = (st.a - .5) * 70;
  final m = .5 + st.b * .5;
  void scene(double a, Color c1, Color c2) {
    final o = Offset(78 + sx, 0);
    final p1 = _s(_o(c1, a), 1.1), p2 = _s(_o(c2, a), 1.1);
    c.drawRect(Rect.fromLTRB(o.dx - 24, 62, o.dx - 4, 80), p1);
    _pl(c, [Offset(o.dx - 27, 63), Offset(o.dx - 14, 50), Offset(o.dx - 1, 63)], p1);
    c.drawRect(Rect.fromLTRB(o.dx - 17, 70, o.dx - 11, 80), p1);
    c.drawLine(Offset(o.dx + 14, 80), Offset(o.dx + 14, 64), p2);
    c.drawCircle(Offset(o.dx + 14, 58), 8, p2);
    c.drawCircle(Offset(o.dx + 30, 26), 5, _s(_o(_wt, a), 1));
    c.drawLine(Offset(o.dx - 50, 80), Offset(o.dx + 50, 80), _s(_o(_wt, a * .6), 1));
  }

  scene(.35, _pu, _gn);
  final ball = Path()..addOval(Rect.fromCircle(center: bc, radius: br));
  _occ(c, ball, .92);
  c.save();
  c.clipPath(ball);
  c.translate(bc.dx, bc.dy);
  c.scale(-m, -m);
  c.translate(-bc.dx, -bc.dy - 18);
  scene(1, _pu, _gn);
  c.restore();
  for (var i = 0; i < 10; i++) {
    final an = t * .4 + i * .63, rr = 10 + _h(i) * 18;
    _dot(c, bc + Offset(math.cos(an), math.sin(an) * .6) * rr, _o(_wt, .35), .8);
  }
  c.drawPath(ball, _s(_wt, 1.2));
  c.drawArc(Rect.fromCircle(center: bc, radius: br - 5), math.pi * 1.1, .7, false, _s(_wt, 1));
  final ln = _s(_wt, 1.1);
  _pl(c, const [Offset(62, 80), Offset(56, 96), Offset(100, 96), Offset(94, 80)], ln);
  c.drawLine(const Offset(48, 100), const Offset(108, 100), ln);
  for (final sd in [-1.0, 1.0]) {
    final hx = 78 + sd * 40;
    final hand = Path()
      ..moveTo(hx + sd * 16, 118)
      ..quadraticBezierTo(hx + sd * 6, 92, hx, 72)
      ..quadraticBezierTo(hx - sd * 2, 56, hx + sd * 2, 44);
    c.drawPath(hand, _s(_rd, 1.1));
    for (var f = 0; f < 4; f++) {
      final fy = 46.0 + f * 7;
      c.drawLine(Offset(hx + sd * 2, fy), Offset(hx - sd * 5, fy - 3 + math.sin(t * 2 + f) * .8), _s(_rd, 1.1));
    }
  }
  _num(c, _pct(st.b), const Offset(6, 6), 12, _bu);
  _lab(c, 'INDEX', const Offset(6, 22), _bu, size: 6.5);
}
