// Part of op_10_pos.dart (cut there by size, as it was).
part of 'op_10.dart';

void _pLoupe(Canvas c, _S st, double t) {
  const ctr = Offset(62, 60), rad = 48.0, px = 12.0;
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: rad)));
  final off = (st.a - .5) * 4;
  final ex = ctr.dx + off * px;
  for (var i = -5; i <= 5; i++) {
    _ln(c, Offset(ctr.dx + i * px, 0), Offset(ctr.dx + i * px, _ch), _dg, .6);
    _ln(c, Offset(0, ctr.dy + i * px), Offset(_cw, ctr.dy + i * px), _dg, .6);
  }
  for (var i = -5; i < 5; i++) {
    final x0 = ctr.dx + i * px;
    final cov = ((ex - x0) / px).clamp(0.0, 1.0);
    if (cov <= 0) continue;
    for (var j = -5; j < 5; j++) {
      _dot(c, Offset(x0 + px / 2, ctr.dy + j * px + px / 2), .5 + cov * 1.3, _al(_wh, .25 + cov * .7));
    }
  }
  _ln(c, Offset(ex, 0), Offset(ex, _ch), _rd, 1.2);
  c.restore();
  _ring(c, ctr, rad, _wh, 1.2);
  _ring(c, ctr, rad + 3, _dg, .8);
  _ln(c, ctr + Offset(rad * .7, rad * .7), ctr + Offset(rad * .7 + 14, rad * .7 + 14), _wh, 2);
  _lab(c, 'sub px', const Offset(150, 6), _rd, ax: 1);
  final v = off - off.floorToDouble();
  _num(c, '.${_d2(v * 100)}', const Offset(150, 15), 26, _rd, ax: 1);
  _lab(c, 'rub', const Offset(150, 46), _mg, ax: 1);
  _num(c, '${off >= 0 ? '+' : '-'}${off.abs().floor()}', const Offset(150, 56), 12, _wh, ax: 1);
}

void _pShip(Canvas c, _S st, double t) {
  final sea = Path()..moveTo(0, 34);
  for (var x = 0.0; x <= _cw; x += 4) {
    sea.lineTo(x, 34 + math.sin(x * .12 + t * 2) * 1.6);
  }
  c.drawPath(sea, _s(_bl, 1));
  final bed = Path()..moveTo(0, 108);
  for (var x = 0.0; x <= _cw; x += 8) {
    bed.lineTo(x, 108 - _rn(x.toInt()) * 4);
  }
  c.drawPath(bed, _s(_mg, .8));
  final ax = 20 + st.a * 116, ay = 108 - 20 - st.b * 30;
  final drift = math.sin(t * .7) * 26;
  final bp = Offset(ax + drift, 34 + math.sin((ax + drift) * .12 + t * 2) * 1.6);
  final chain = <Offset>[];
  for (var i = 0; i <= 12; i++) {
    final f = i / 12;
    chain.add(Offset(_lp(bp.dx, ax, f), _lp(bp.dy + 3, ay, f) + math.sin(f * math.pi) * 6));
  }
  for (var i = 0; i < chain.length - 1; i++) {
    c.drawOval(Rect.fromCenter(center: _lp2(chain[i], chain[i + 1], .5), width: 3, height: 2), _s(_wh, .7));
  }
  _boat(c, bp, .9, _wh, tilt: math.cos(t * .7) * .12);
  _anchor(c, Offset(ax, ay), 1, _rd);
  for (var i = 0; i < 2; i++) {
    _fish(c, Offset(_wr(t * .05 + i * .5) * 180 - 12, 60 + i * 18), 9, _al(_gr, .6), wag: math.sin(t * 5 + i), w: .8);
  }
  _lab(c, 'anchor', const Offset(6, 6), _rd);
  _num(c, '${_d2(st.a * 99)}·${_d2(st.b * 99)}', const Offset(150, 4), 16, _rd, ax: 1);
}

void _pGlobe(Canvas c, _S st, double t) {
  const ctr = Offset(62, 62);
  final r = 36 + st.b * 14;
  _ring(c, ctr, r, _wh, 1.2);
  final lon = st.a * 2 * math.pi;
  for (var k = 0; k < 6; k++) {
    final ang = lon + k * math.pi / 6;
    final w = math.cos(ang).abs() * r;
    c.drawArc(Rect.fromCenter(center: ctr, width: w * 2, height: r * 2), -math.pi / 2, math.sin(ang) > 0 ? math.pi : -math.pi, false,
        _s(_al(_bl, .7), .8));
  }
  for (var j = -2; j <= 2; j++) {
    final y = ctr.dy + j * r / 3, hw = math.sqrt(math.max(0, r * r - (j * r / 3) * (j * r / 3)));
    _ln(c, Offset(ctr.dx - hw, y), Offset(ctr.dx + hw, y), _dg, .8);
  }
  const plat = .5, plon = -.3;
  final al = plon + lon;
  final pin = ctr + Offset(math.sin(al) * math.cos(plat) * r, -math.sin(plat) * r);
  if (math.cos(al) > 0) {
    _ln(c, pin, pin - const Offset(0, 12), _rd, 1.2);
    _ring(c, pin - const Offset(0, 15), 3, _rd, 1.2);
    _dot(c, pin, 1.6, _rd);
  } else {
    _ring(c, pin, 1.6, _al(_rd, .4), .8);
  }
  final sa = t * .9;
  final sat = ctr + Offset(math.cos(sa) * (r + 12), math.sin(sa) * (r + 12) * .35);
  c.drawOval(Rect.fromCenter(center: ctr, width: (r + 12) * 2, height: (r + 12) * .7), _s(_al(_pu, .5), .7));
  _rect(c, Rect.fromCenter(center: sat, width: 4, height: 4), _pu, 1);
  _lab(c, 'lon', const Offset(150, 6), _bl, ax: 1);
  _num(c, _d3(st.a * 360), const Offset(150, 15), 18, _bl, ax: 1);
  _lab(c, 'zoom', const Offset(150, 84), _gr, ax: 1);
  _num(c, _d2(st.b * 99), const Offset(150, 93), 18, _gr, ax: 1);
}
