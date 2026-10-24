// Part of op_08_fill.dart (cut there by size, as it was).
part of 'op_08.dart';

void _fPiano(Canvas c, Size s, _S st, double t) {
  const k = 14, x0 = 8.0, top = 64.0, kw = 10.0;
  final sel = (st.a * (k - 1)).round(), sat = st.b;
  for (var i = 0; i < k; i++) {
    final r = Rect.fromLTWH(x0 + i * kw, top + (i == sel ? 2 : 0), kw, 46);
    c.drawRect(r, _s(i == sel ? _hc(i / k, sat) : _al(_wh, .7), i == sel ? 1.3 : 1));
    _dot(c, Offset(r.center.dx, r.bottom - 6), i == sel ? 2 : 1.2, _hc(i / k, sat));
  }
  for (var i = 0; i < k - 1; i++) {
    if (i % 7 == 2 || i % 7 == 6) continue;
    final r = Rect.fromLTWH(x0 + (i + 1) * kw - 3, top, 6, 26);
    c.drawRect(r, _f(_bk));
    c.drawRect(r, _s(_al(_wh, .7), 1));
  }
  final col = _hc(sel / k, sat);
  final wave = [for (var i = 0; i <= 60; i++) Offset(8 + i * 2.33, 34 + math.sin(i * (.2 + sel * .03) - t * 5) * 10)];
  _pl(c, wave, col, w: 1.2);
  _lab(c, 'hue', const Offset(6, 6), _bl);
  _num(c, _d3(sel / k * 360), Offset(s.width - 6, 4), 14, _bl, ax: 1);
  _lab(c, 'sat ${_d2(sat * 99)}', const Offset(40, 6), _gr);
}

void _fNova(Canvas c, Size s, _S st, double t) {
  final cen = s.center(Offset.zero);
  final n = 2 + (st.b * 38).round();
  const rays = 220;
  for (var i = 0; i < rays; i++) {
    final a = i / rays * 2 * math.pi + t * .05;
    final l = 14 + _rn(i) * 60 + math.sin(t * 2 + i) * 3;
    final hue = st.a + (i % n) / n * .8;
    final d = Offset(math.cos(a), math.sin(a));
    _ln(c, cen + d * (6 + _rn(i + 99) * 6), cen + d * l, _al(_hc(hue, .9), .75), .8);
  }
  _ring(c, cen, 5, _wh, 1);
  _dot(c, cen, 1.5, _wh);
  _lab(c, 'stops', const Offset(6, 6), _wh);
  _num(c, _d2(n), Offset(s.width - 6, 4), 14, _wh, ax: 1);
}
