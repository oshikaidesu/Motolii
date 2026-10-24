// Part of op_03_noise.dart (cut there by size, as it was).
part of 'op_03.dart';

void _n19(Canvas c, Size s, _St st, double t) {
  final a = (2 + st.spin * 3).clamp(0.0, 22.0), e = t * .8;
  c.drawRRect(RRect.fromLTRBR(6, 8, 150, 112, const Radius.circular(10)), _s(_kWhite));
  final scr = RRect.fromLTRBR(14, 16, 116, 104, const Radius.circular(6));
  c.drawRRect(scr, _s(_kGrey, .8));
  for (var i = 1; i < 6; i++) {
    _dash(c, Offset(14 + i * 17, 16), Offset(14 + i * 17, 104), _kDim, w: .6, on: 1, off: 3);
  }
  for (var j = 1; j < 4; j++) {
    _dash(c, Offset(14, 16 + j * 22), Offset(116, 16 + j * 22), _kDim, w: .6, on: 1, off: 3);
  }
  c.save();
  c.clipRRect(scr);
  _pl(c, [for (var x = 14.0; x <= 116; x += 1) Offset(x, 60 + 20 * math.sin((x - 14) / 102 * 4 * math.pi + t * 2) + a * _vn(x / 3, e))],
      _kGreen);
  c.restore();
  _kv(c, const Offset(122, 18), 'AMP', a * 4.5, _kGreen, h: 10);
  _kv(c, const Offset(122, 50), 'EVO', (e * 10) % 100, _kRed, h: 10);
  final r = st.spin * 2;
  _o(c, const Offset(133, 92), 9, _kDim, .8);
  _pl(c, [for (var k = 0.0; k <= 1; k += .1) const Offset(133, 92) + _dir(-math.pi / 2 + k * 4) * 6.0], _kGreen, w: .9);
  _dot(c, const Offset(133, 92) + _dir(-math.pi / 2 + r) * 9, 1.6, _kGreen);
}

void _n20(Canvas c, Size s, _St st, double t) {
  final f = 1 + st.a * 5, a = 6 + st.b * 70, e = t * .3;
  for (var i = 0; i < 20; i++) {
    final base = 24 + i * 4.7;
    final pts = [
      for (var x = 4.0; x <= 152; x += 2) Offset(x, base - a * math.exp(-math.pow((x - 78) / 28, 2)) * (.35 + .65 * _fbm(x * f / 30, i * .7 + e).abs()))
    ];
    final path = Path()
      ..addPolygon(pts, false)
      ..lineTo(152, 124)
      ..lineTo(4, 124)
      ..close();
    c.drawPath(path, Paint()..color = _kBg);
    _pl(c, pts, i == 10 ? _kGreen : _kWhite, w: .9);
  }
  _kv(c, const Offset(150, 4), 'AMP', st.b * 99, _kGreen, h: 9, align: 1);
}
