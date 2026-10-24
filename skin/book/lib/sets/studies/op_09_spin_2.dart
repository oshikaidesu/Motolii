// Part of op_09_spin.dart (cut there by size, as it was).
part of 'op_09.dart';

void _rHorizon(Canvas c, Size s, _S st, double t) {
  final roll = (st.a - .5) * math.pi * .5, pitch = (st.b - .5) * 44;
  c.save();
  c.translate(_c0.dx, _c0.dy);
  c.rotate(roll);
  c.translate(0, pitch);
  _ln(c, const Offset(-160, 0), const Offset(160, 0), _wh, 1);
  final m = <Offset>[];
  for (var x = -160.0; x <= 160; x += 8) {
    m.add(Offset(x, -(math.sin(x * .07) * .5 + .5) * 14 * _rn(((x + 160) / 8).round()) - 2));
  }
  _pl(c, m, _bl, w: 1);
  _ring(c, Offset(34 + math.sin(t * .2) * 4, -30), 7, _rd, 1);
  for (final y in [6.0, 14.0, 26.0, 44.0, 70.0]) {
    _ln(c, Offset(-160, y), Offset(160, y), _dg, .7);
  }
  for (var i = -8; i <= 8; i++) {
    _ln(c, Offset(i * 4.0, 0), Offset(i * 40.0, 90), _dg, .7);
  }
  c.restore();
  _pl(c, [_c0 + const Offset(-30, 0), _c0 + const Offset(-10, 0), _c0 + const Offset(0, 7), _c0 + const Offset(10, 0),
      _c0 + const Offset(30, 0)], _gr, w: 1.4);
  _dot(c, _c0, 1.6, _gr);
  for (var k = -3; k <= 3; k++) {
    final a = -math.pi / 2 + k * math.pi / 12;
    _ln(c, _pol(_c0, 50, a), _pol(_c0, k == 0 ? 56 : 53, a), _wh, 1);
  }
  final pa = -math.pi / 2 - roll;
  _pl(c, [_pol(_c0, 45, pa), _pol(_c0, 41, pa - .06), _pol(_c0, 41, pa + .06)], _bl, w: 1.2, close: true);
  for (final (x, y, sx, sy) in [(4.0, 4.0, 1.0, 1.0), (152.0, 4.0, -1.0, 1.0), (4.0, 116.0, 1.0, -1.0), (152.0, 116.0, -1.0, -1.0)]) {
    _pl(c, [Offset(x, y + sy * 8), Offset(x, y), Offset(x + sx * 8, y)], _wh, w: 1);
  }
  final rd = (roll * 180 / math.pi).round();
  _lab(c, 'roll', const Offset(10, 92), _bl);
  _num(c, '${rd < 0 ? '-' : '+'}${_d2(rd.abs())}', const Offset(9, 100), 12, _bl);
  _lab(c, 'pitch', const Offset(146, 92), _gr, ax: 1);
  _num(c, _d2(pitch.abs()), const Offset(146, 100), 12, _gr, ax: 1);
}

void _rGalaxy(Canvas c, Size s, _S st, double t) {
  final rot = st.a * _tau + t * .15, wind = .4 + st.b * 4.5;
  final core = <Offset>[], mid = <Offset>[], out = <Offset>[];
  for (var i = 0; i < 220; i++) {
    final arm = i % 2, u = (i ~/ 2) / 110;
    final r = 3 + u * 56 + (_rn(i) - .5) * 5;
    final ph = arm * math.pi + u * wind * 3 + rot + (_rn(i + 7) - .5) * .35;
    final p = _pol(_c0, r, ph);
    (u < .25 ? core : (u < .65 ? mid : out)).add(p);
  }
  c.drawPoints(ui.PointMode.points, core, _s(_wh, 1.6));
  c.drawPoints(ui.PointMode.points, mid, _s(_bl, 1.3));
  c.drawPoints(ui.PointMode.points, out, _s(_al(_bl, .5), 1));
  final gp = <Offset>[for (var i = 0; i < 18; i++) _pol(_c0, 10 + _rn(i + 50) * 48, _rn(i + 90) * _tau + rot)];
  c.drawPoints(ui.PointMode.points, gp, _s(_gr, 1.4));
  _ring(c, _c0, 3, _wh, 1);
  _dot(c, _pol(_c0, 34, rot + wind * 1.8), 2, _rd);
  _lab(c, 'twist', const Offset(6, 8), _gr);
  _num(c, wind.toStringAsFixed(1), const Offset(5, 18), 16, _gr);
  _num(c, _deg(rot / _tau), Offset(s.width - 6, 100), 12, _bl, ax: 1);
}

void _rWasher(Canvas c, Size s, _S st, double t) {
  final sp = st.b, shake = math.sin(t * 47) * sp * sp * 1.8;
  c.save();
  c.translate(shake, 0);
  c.drawRRect(RRect.fromLTRBR(26, 6, 130, 112, const Radius.circular(4)), _s(_wh, 1));
  _ln(c, const Offset(26, 22), const Offset(130, 22), _wh, 1);
  _ring(c, const Offset(36, 14), 3, _dg, 1);
  _num(c, (sp * 1400).round().toString().padLeft(4, '0'), const Offset(124, 9), 10, _gr, ax: 1);
  const o = Offset(78, 66);
  _ring(c, o, 36, _wh, 1.2);
  _ring(c, o, 30, _dg, 1);
  final dr = st.a * _tau * 2 + t * sp * 14;
  for (var i = 0; i < 12; i++) {
    _dot(c, _pol(o, 27, dr + i * _tau / 12), .9, _gr);
  }
  final pin = sp > .55;
  final tumble = pin ? 0.0 : math.sin(t * 3) * 4;
  final r1 = pin ? 20.0 : 15 + tumble, a1 = pin ? dr : dr * .4 + 1.6;
  final sc = _pol(o, r1, a1);
  _shape(c, const [Offset(-3, -6), Offset(3, -6), Offset(3, 2), Offset(8, 5), Offset(8, 8), Offset(-3, 8)],
      (p) => sc + Offset(p.dx * math.cos(a1) - p.dy * math.sin(a1), p.dx * math.sin(a1) + p.dy * math.cos(a1)), _bl);
  final a2 = pin ? dr + 2.6 : dr * .4 + 3.4, r2 = pin ? 19.0 : 12 - tumble;
  final tc = _pol(o, r2, a2);
  _shape(c, const [Offset(-4, -6), Offset(-9, -3), Offset(-6, 0), Offset(-4, -1), Offset(-4, 7), Offset(4, 7),
      Offset(4, -1), Offset(6, 0), Offset(9, -3), Offset(4, -6)],
      (p) => tc + Offset(p.dx * math.cos(a2) - p.dy * math.sin(a2), p.dx * math.sin(a2) + p.dy * math.cos(a2)), _bl);
  if (!pin) {
    _pl(c, [for (var x = -24.0; x <= 24; x += 3) o + Offset(x, 18 + math.sin(x * .4 + t * 4) * 1.5)], _al(_wh, .4), w: .8);
  }
  c.restore();
  _ln(c, const Offset(30, 114), const Offset(40, 114), _dg, 1);
  _ln(c, const Offset(116, 114), const Offset(126, 114), _dg, 1);
  _lab(c, 'rpm', const Offset(96, 11), _gr, ax: 1, size: 7);
}

void _rTop(Canvas c, Size s, _S st, double t) {
  final tilt = st.b * .7, psi = t * (1.4 - st.b * .6), spin = st.a * _tau + t * 9;
  const tip = Offset(70, 100);
  _ln(c, const Offset(8, 101), const Offset(148, 101), _dg, 1);
  final ax = Offset(math.sin(tilt) * math.cos(psi), -math.cos(tilt) + math.sin(tilt) * math.sin(psi) * .25);
  final top = tip + ax * 62;
  _dots(c, Offset(tip.dx - 62 * math.sin(tilt), top.dy), Offset(tip.dx + 62 * math.sin(tilt), top.dy), _dg);
  c.drawOval(Rect.fromCenter(center: Offset(tip.dx, tip.dy - 62 * math.cos(tilt)), width: 124 * math.sin(tilt) + .1,
      height: 31 * math.sin(tilt) + .1), _s(_dg, .8));
  c.save();
  c.translate(tip.dx, tip.dy);
  c.rotate(ax.direction + math.pi / 2);
  _ln(c, Offset.zero, const Offset(-20, -24), _wh, 1);
  _ln(c, Offset.zero, const Offset(20, -24), _wh, 1);
  c.drawOval(Rect.fromCenter(center: const Offset(0, -24), width: 40, height: 9), _s(_wh, 1));
  _pl(c, [const Offset(-20, -24), const Offset(-6, -34), const Offset(6, -34), const Offset(20, -24)], _wh, w: 1);
  c.drawRect(const Rect.fromLTRB(-2.5, -46, 2.5, -34), _s(_wh, 1));
  for (var k = 0; k < 6; k++) {
    final a = spin + k * math.pi / 3;
    if (math.sin(a) > 0) _ln(c, Offset(20 * math.cos(a), -24 + 4.5 * math.sin(a)), Offset.zero, _gr, 1);
  }
  _dots(c, const Offset(0, -46), const Offset(0, -64), _wh);
  c.restore();
  _dot(c, tip, 1.5, _rd);
  _lab(c, 'axis', Offset(s.width - 6, 8), _wh, ax: 1);
  _num(c, _d2(tilt * 180 / math.pi), Offset(s.width - 6, 18), 20, _wh, ax: 1);
  _lab(c, 'spin', const Offset(6, 8), _gr);
  _num(c, _deg(spin / _tau), const Offset(5, 18), 12, _gr);
}

void _rHamster(Canvas c, Size s, _S st, double t) {
  const o = Offset(68, 54), r = 42.0;
  final sp = st.b, ang = st.a * _tau * 2 + t * sp * 5;
  _ln(c, o, const Offset(46, 114), _dg, 1);
  _ln(c, o, const Offset(90, 114), _dg, 1);
  _ln(c, const Offset(38, 114), const Offset(98, 114), _dg, 1);
  _ring(c, o, r, _wh, 1);
  _ring(c, o, r - 4, _wh, .7);
  for (var i = 0; i < 24; i++) {
    final a = ang + i * _tau / 24;
    _ln(c, _pol(o, r - 4, a), _pol(o, r, a), _bl, 1);
  }
  for (var i = 0; i < 6; i++) {
    _ln(c, o, _pol(o, r - 4, ang + i * _tau / 6), _al(_bl, .5), .8);
  }
  _dot(c, o, 2, _wh);
  const fy = 54.0 + r - 4;
  final run = sp > .08, ph = t * (6 + sp * 24), bob = run ? math.sin(ph * 2) * 1.2 : 0.0;
  final bc = Offset(66, fy - 9 + bob);
  c.drawOval(Rect.fromCenter(center: bc, width: 26, height: 14), _s(_wh, 1));
  final hd = bc + const Offset(13, -3);
  _ring(c, hd, 6, _wh, 1);
  _ring(c, hd + const Offset(-2, -6), 2, _wh, 1);
  _dot(c, hd + const Offset(2, -1), 1, _wh);
  _dot(c, hd + const Offset(6, 1), 1, _rd);
  for (var k = 0; k < 4; k++) {
    final lx = bc.dx - 8 + k * 6, sw = run ? math.sin(ph + k * 1.6) * 4 : 0.0;
    _ln(c, Offset(lx, bc.dy + 5), Offset(lx + sw, fy), _wh, 1);
  }
  if (run) {
    for (var i = 0; i < 3; i++) {
      final y = bc.dy - 4 + i * 4.0;
      _ln(c, Offset(bc.dx - 18 - i * 2, y), Offset(bc.dx - 18 - i * 2 - sp * 16, y), _gr, 1);
    }
  }
  _lab(c, 'speed', Offset(s.width - 6, 8), _gr, ax: 1);
  _num(c, _d2(sp * 99), Offset(s.width - 6, 18), 20, _gr, ax: 1);
  _lab(c, 'turns', Offset(s.width - 6, 88), _bl, ax: 1);
  _num(c, _d3((ang / _tau).abs() % 1000), Offset(s.width - 6, 98), 14, _bl, ax: 1);
}

void _rStrobe(Canvas c, Size s, _S st, double t) {
  const fps = 12.0, n = 8;
  final w = st.b * 40;
  final ts = (t * fps).floor() / fps;
  double real(double tt) => st.a * _tau + w * tt;
  const lo = Offset(42, 54), ro = Offset(114, 54), r = 30.0;
  _ring(c, lo, r, _wh, 1);
  _ring(c, ro, r, _wh, 1);
  for (var j = 0; j < 7; j++) {
    final a0 = real(t - j * .006);
    for (var i = 0; i < n; i++) {
      _ln(c, lo, _pol(lo, r, a0 + i * _tau / n), _al(_bl, .7 - j * .09), 1);
    }
  }
  final sa = real(ts);
  for (var i = 0; i < n; i++) {
    _ln(c, ro, _pol(ro, r, sa + i * _tau / n), _wh, 1.2);
  }
  _dot(c, _pol(ro, r, sa), 2, _rd);
  final step = _wr((w / fps) / (_tau / n));
  final back = step > .5, still = step < .03 || (step - .5).abs() < .02 || step > .97;
  final ac = still ? _dg : (back ? _rd : _gr);
  final y = 98.0;
  c.drawArc(Rect.fromCircle(center: Offset(ro.dx, y - 6), radius: 10), math.pi * .2, math.pi * .6, false, _s(ac, 1));
  _arrow(c, _pol(Offset(ro.dx, y - 6), 10, back ? math.pi * .8 : math.pi * .2), back ? math.pi * 1.3 : math.pi * -.3, ac);
  _lab(c, 'real', Offset(lo.dx, 92), _bl, ax: .5);
  _num(c, _d2(w / _tau * 10), Offset(lo.dx, 101), 12, _gr, ax: .5);
  _lab(c, back ? 'seen  back' : (still ? 'seen  still' : 'seen  fwd'), Offset(ro.dx, 106), ac, ax: .5, size: 7);
  if (_wr(t * fps) < .3) _dot(c, Offset(s.width - 8, 8), 2.5, _rd);
  _lab(c, '12 fps', const Offset(6, 6), _dg);
}

void _rVortex(Canvas c, Size s, _S st, double t) {
  final rot = st.a * _tau + t * .5, twist = 1 + st.b * 4;
  const w = 'SPIN·';
  for (var i = 0; i < 46; i++) {
    final u = i / 46;
    final r = 60 * (1 - u) + 4, ph = rot + u * _tau * twist;
    final p = _pol(_c0, r, ph);
    final sz = (5 + (1 - u) * 11).roundToDouble();
    final col = u < .3 ? _bl : (u < .7 ? _wh : _al(_gr, .9));
    final tp = _tp(w[i % w.length], sz, col, FontWeight.w300, 0);
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(ph + math.pi / 2);
    tp.paint(c, Offset(-tp.width / 2, -tp.height / 2));
    c.restore();
  }
  _dot(c, _c0, 2, _rd);
  _lab(c, 'twist', const Offset(6, 6), _gr);
  _num(c, twist.toStringAsFixed(1), const Offset(5, 15), 12, _gr);
}
