// Part of op_09_scale.dart (cut there by size, as it was).
part of 'op_09.dart';

void _sLock(Canvas c, Size s, _S st, double t) {
  const an = Offset(14, 108), w0 = 36.0, h0 = 26.0;
  final kx = .4 + st.a * 2.9, ky = st.on ? kx : .4 + st.b * 3.4;
  final w = math.min(w0 * kx, 132.0), h = math.min(h0 * ky, 100.0);
  final pts = [an, an + const Offset(w0, 0), an + const Offset(w0, -h0), an + const Offset(0, -h0), an];
  for (var i = 0; i < 4; i++) {
    _dots(c, pts[i], pts[i + 1], _dg);
  }
  c.drawRect(Rect.fromLTWH(an.dx, an.dy - h, w, h), _s(_wh, 1.1));
  if (st.on) _dots(c, an, an + Offset(w, -h), _al(_wh, .5));
  _dot(c, an + Offset(w, -h), 2.6, _rd);
  _hDim(c, an.dx, an.dx + w, an.dy + 6, _bl);
  final gap = st.on ? 0.0 : 3.5;
  c.save();
  c.translate(140, 14);
  c.rotate(-math.pi / 4 + (st.on ? 0 : .25));
  c.drawRRect(RRect.fromLTRBR(-9 - gap, -3.5, 1 - gap, 3.5, const Radius.circular(3.5)), _s(_wh, 1.2));
  c.drawRRect(RRect.fromLTRBR(-1 + gap, -3.5, 9 + gap, 3.5, const Radius.circular(3.5)), _s(_wh, 1.2));
  c.restore();
  _num(c, _pc(kx), const Offset(124, 30), 14, _bl, ax: 1);
  _num(c, _pc(ky), const Offset(124, 48), 14, _gr, ax: 1);
}

void _sScope(Canvas c, Size s, _S st, double t) {
  final ext = st.b, focus = st.a, mag = 1 + ext * 5;
  const pv = Offset(40, 78);
  _ln(c, pv, const Offset(24, 112), _wh, 1);
  _ln(c, pv, const Offset(40, 114), _wh, 1);
  _ln(c, pv, const Offset(58, 112), _wh, 1);
  c.save();
  c.translate(pv.dx, pv.dy);
  c.rotate(-.42);
  final len = [26.0, 22.0, 18.0];
  var x = -18.0;
  for (var i = 0; i < 3; i++) {
    final hw = 7.0 - i * 1.6, l = len[i];
    c.drawRect(Rect.fromLTWH(x, -hw, l, hw * 2), _s(i == 0 ? _wh : _bl, 1));
    x += i == 0 ? l - 4 : l * (.25 + ext * .7);
  }
  _ln(c, Offset(x + 14, -4), Offset(x + 14, 4), _bl, 1);
  c.restore();
  const vo = Offset(120, 40), vr = 30.0;
  _ring(c, vo, vr, _wh, 1);
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: vo, radius: vr - 1)));
  final blur = (focus - ext).abs();
  final mr = 5 * mag;
  for (var k = 0; k < (blur > .08 ? 3 : 1); k++) {
    final off = Offset((k - 1) * blur * 8, 0);
    final col = k == 1 || blur <= .08 ? _wh : _al(_bl, .5);
    _ring(c, vo + off, mr, col, 1);
    for (final (cx, cy, cr) in [(.3, -.2, .18), (-.35, .25, .12), (.05, .45, .09)]) {
      _ring(c, vo + off + Offset(cx * mr, cy * mr), cr * mr, col, .8);
    }
  }
  _ln(c, vo - const Offset(vr, 0), vo + const Offset(vr, 0), _dg, .6);
  _ln(c, vo - const Offset(0, vr), vo + const Offset(0, vr), _dg, .6);
  c.restore();
  _lab(c, 'mag', const Offset(6, 6), _bl);
  _num(c, '${mag.toStringAsFixed(1)}x', const Offset(6, 14), 16, _bl);
  _lab(c, blur <= .08 ? 'focus  ok' : 'focus', const Offset(150, 104), _gr, ax: 1);
}

void _sHouse(Canvas c, Size s, _S st, double t) {
  final th = st.a * _tau, k = .45 + st.b * .9;
  final sx = k * math.pow(2, math.cos(th) * 1.1).toDouble(), sy = k * math.pow(2, math.sin(th) * 1.1).toDouble();
  const base = Offset(70, 106);
  Offset m(Offset p) => base + Offset(p.dx * sx, p.dy * sy);
  _ln(c, const Offset(4, 106), const Offset(152, 106), _dg, 1);
  _shape(c, const [Offset(-20, 0), Offset(-20, -26), Offset(20, -26), Offset(20, 0)], m, _wh, close: false);
  _shape(c, const [Offset(-24, -24), Offset(0, -44), Offset(24, -24)], m, _wh, close: false);
  _shape(c, const [Offset(-6, 0), Offset(-6, -14), Offset(4, -14), Offset(4, 0)], m, _bl, close: false);
  _shape(c, const [Offset(8, -20), Offset(16, -20), Offset(16, -13), Offset(8, -13)], m, _gr);
  _shape(c, const [Offset(10, -34), Offset(10, -42), Offset(15, -42), Offset(15, -30)], m, _wh, close: false);
  final sm = m(const Offset(12.5, -44));
  for (var i = 0; i < 3; i++) {
    _ring(c, sm + Offset(math.sin(t + i) * 2, -i * 5.0 - _wr(t * .5) * 5), 1.5 + i * .6, _al(_wh, .5 - i * .12), .8);
  }
  const ac = Offset(138, 24);
  c.drawRect(Rect.fromCenter(center: ac, width: 10 * sx / k, height: 10 * sy / k), _s(_al(_wh, .7), 1));
  _num(c, _pc(sx), const Offset(6, 6), 12, _bl);
  _num(c, _pc(sy), const Offset(6, 22), 12, _gr);
}

void _sCar(Canvas c, Size s, _S st, double t) {
  const gy = 104.0;
  final q = st.a, sy = 1 - q * .72, sx = 1 + q * .45, wt = .6 + st.b * .9;
  _ln(c, const Offset(4, gy), const Offset(152, gy), _dg, 1);
  Offset m(Offset p) => Offset(78 + p.dx * sx, gy + p.dy * sy);
  _shape(c, const [Offset(-40, -8), Offset(-40, -20), Offset(-22, -22), Offset(-12, -34), Offset(14, -34), Offset(24, -22),
      Offset(40, -20), Offset(40, -8)], m, _wh, w: 1.1, close: false);
  _shape(c, const [Offset(-9, -31), Offset(-2, -31), Offset(-2, -22), Offset(-17, -22)], m, _bl);
  _shape(c, const [Offset(2, -31), Offset(12, -31), Offset(19, -22), Offset(2, -22)], m, _bl);
  for (final wx in [-24.0, 24.0]) {
    final wc = m(Offset(wx, -8));
    c.drawOval(Rect.fromCenter(center: Offset(wc.dx, gy - 8 * sy), width: 16 * sx, height: 16 * sy), _s(_wh, 1));
    _dot(c, Offset(wc.dx, gy - 8 * sy), 1.2, _wh);
  }
  final roof = gy - 34 * sy, aw = 34 * wt, ah = 18 * wt;
  final ax = 78.0, ay = roof - ah;
  _shape(c, [Offset(-aw / 2, 0), Offset(aw / 2, 0), Offset(aw * .3, -ah * .35), Offset(aw * .3, -ah * .6), Offset(aw * .6, -ah * .7),
      Offset(aw * .6, -ah), Offset(-aw * .7, -ah), Offset(-aw * .7, -ah * .75), Offset(-aw * .3, -ah * .6), Offset(-aw * .3, -ah * .35)],
      (p) => Offset(ax, ay + ah) + p, _wh, w: 1.1);
  _num(c, '${(wt * 10).round()}t', Offset(ax, ay + ah * .38), 9, _wh, ax: .5);
  if (q > .45) {
    for (var i = 0; i < 4; i++) {
      final p = Offset(ax - aw + i * aw * .66, roof + 2 + (i.isEven ? -4 : 2));
      for (var k = 0; k < 3; k++) {
        final a = k * math.pi / 3 + t * 3;
        _ln(c, _pol(p, 3, a), _pol(p, 3, a + math.pi), _rd, 1);
      }
    }
  }
  _lab(c, 'squash', const Offset(6, 6), _rd);
  _num(c, _d2(q * 99), const Offset(5, 15), 24, _rd);
}

void _sTower(Canvas c, Size s, _S st, double t) {
  const gy = 112.0;
  final floors = 3 + (st.b * 40).round(), w = 14 + st.a * 34;
  final x0 = 78 - w / 2, ht = floors * 2.3;
  for (final (x, bw, h) in [(4.0, 18.0, 22.0), (24.0, 14.0, 34.0), (112.0, 16.0, 28.0), (130.0, 22.0, 16.0)]) {
    c.drawRect(Rect.fromLTWH(x, gy - h, bw, h), _s(_al(_bl, .7), .9));
  }
  _ln(c, const Offset(0, gy), const Offset(156, gy), _dg, 1);
  c.drawRect(Rect.fromLTWH(x0, gy - ht, w, ht), _s(_wh, 1.1));
  for (var f = 1; f < floors; f++) {
    final y = gy - f * 2.3;
    if (f % 2 == 0) _ln(c, Offset(x0, y), Offset(x0 + w, y), _dg, .6);
  }
  for (var x = x0 + 5; x < x0 + w - 2; x += 5) {
    _ln(c, Offset(x, gy - ht), Offset(x, gy), _al(_dg, .8), .6);
  }
  final top = gy - ht;
  _ln(c, Offset(x0 + w * .3, top), Offset(x0 + w * .3, top - 16), _gr, 1);
  _ln(c, Offset(x0 + w * .3 - 10, top - 16), Offset(x0 + w * .3 + 26, top - 16), _gr, 1);
  final hk = math.sin(t * 1.3) * 3;
  _ln(c, Offset(x0 + w * .3 + 22, top - 16), Offset(x0 + w * .3 + 22 + hk, top - 4), _wh, .8);
  c.drawRect(Rect.fromCenter(center: Offset(x0 + w * .3 + 22 + hk, top - 2), width: 5, height: 4), _s(_rd, 1));
  _vDim(c, top, gy, x0 + w + 6, _gr);
  _lab(c, 'floors', const Offset(6, 6), _gr);
  _num(c, _d2(floors), const Offset(5, 15), 22, _gr);
  _num(c, _pc(w / 30), Offset(s.width - 6, 6), 12, _bl, ax: 1);
}

void _sWave(Canvas c, Size s, _S st, double t) {
  final k = .25 + st.a * 2.75, g = .15 + st.b * 1.1;
  const xs = [10.0, 34.0, 52.0, 76.0];
  double wave(double x, double kk, double amp) {
    var y = 0.0;
    for (final x0 in xs) {
      final p = 10 + (x0 - 10) * kk;
      if (x >= p) y = amp * math.exp(-(x - p) / (9 * kk)) * math.sin((x - p) * .95);
    }
    return y;
  }

  _pl(c, [for (var x = 6.0; x <= 150; x += 1.5) Offset(x, 24 + wave(x, 1, 7))], _dg, w: .8);
  _pl(c, [for (var x = 6.0; x <= 150; x += 1) Offset(x, 76 + wave(x, k, 24 * g))], _gr, w: 1);
  for (final x0 in xs) {
    final p = 10 + (x0 - 10) * k;
    if (p < 152) _dots(c, Offset(p, 40), Offset(p, 108), _rd);
    if (x0 < 152) _ln(c, Offset(x0, 32), Offset(p.clamp(0, 156).toDouble(), 40), _al(_bl, .6), .8);
  }
  final px = 6 + _wr(t * .3) * 144;
  _ln(c, Offset(px, 44), Offset(px, 108), _wh, .8);
  _num(c, k.toStringAsFixed(2), Offset(s.width - 6, 4), 18, _bl, ax: 1);
  _lab(c, 'pitch kept', const Offset(6, 108), _dg);
}
