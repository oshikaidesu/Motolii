// Part of op_03_shadow.dart (cut there by size, as it was).
part of 'op_03.dart';

void _s17(Canvas c, Size s, _St st, double t) {
  const o = Offset(70, 60), r = 36.0;
  final ph = st.a * math.pi, soft = .5 + st.b * 5;
  for (var i = 0; i < 20; i++) {
    _dot(c, Offset((_hash(i, 31) * .5 + .5) * 156, (_hash(i, 32) * .5 + .5) * 120), .5 + .4 * math.sin(t * 2 + i), _kGrey);
  }
  c.drawCircle(o, r, _f(_kBg));
  _o(c, o, r, _kWhite);
  for (var k = 0; k <= 4; k++) {
    final kx = math.cos(ph) * r - k * soft * 2.5;
    _pl(c, [for (var a = -math.pi / 2; a <= math.pi / 2 + .01; a += .1) o + Offset(math.cos(a) * kx.clamp(-r, r), math.sin(a) * r)],
        k == 0 ? _kPurple : _al(_kRed, .6 - k * .12), w: k == 0 ? 1.4 : .7);
  }
  for (final (q, rr) in [(const Offset(-12, -10), 5.0), (const Offset(8, 12), 3.5), (const Offset(14, -16), 2.5)]) {
    _o(c, o + q, rr, _kDim, .8);
  }
  _kv(c, const Offset(150, 6), 'PHASE', st.a * 99, _kBlue, h: 9, align: 1);
  _kv(c, const Offset(150, 92), 'SOFT', soft * 18, _kRed, h: 9, align: 1);
}

void _s18(Canvas c, Size s, _St st, double t) {
  final off = (st.at(s) - const Offset(78, 60)) * .35;
  final soft = 1 + _sat(off.distance / 18) * 3;
  for (var i = 0; i < 18; i++) {
    for (var j = 0; j < 7; j++) {
      _dot(c, Offset(6 + i * 8.5, 18 + j * 9.0), .45, _kDim);
    }
  }
  final card = RRect.fromLTRBR(52, 22, 104, 54, const Radius.circular(6));
  for (var k = 4; k >= 1; k--) {
    c.drawRRect(card.shift(off).inflate(k * soft * 1.6), _s(_al(_kRed, .55 - k * .11), .7));
  }
  c.drawRRect(card.shift(off), _s(_kPurple, 1.2));
  c.drawRRect(card, _f(_kBg));
  c.drawRRect(card, _s(_kWhite));
  _kv(c, const Offset(8, 84), 'DIR', (math.atan2(off.dy, off.dx) * 180 / math.pi + 360) % 360 / 4, _kBlue, h: 14);
  _kv(c, const Offset(60, 84), 'DIST', off.distance * 3, _kGreen, h: 14);
  _kv(c, const Offset(110, 84), 'SOFT', soft * 20, _kRed, h: 14);
}

void _s19(Canvas c, Size s, _St st, double t) {
  final skew = (st.a - .5) * 90, stretch = .6 + st.b * 1.4;
  _pl(c, const [Offset(0, 66), Offset(156, 66)], _kDim, w: .8);
  c.drawRect(const Rect.fromLTRB(54, 8, 102, 58), _s(_kWhite));
  for (var i = 0; i < 8; i++) {
    final y = 12 + i * 6.0;
    _ln(c, Offset(56, y), Offset(100, y + 1.5), _kGrey, .9);
  }
  _ln(c, const Offset(78, 8), const Offset(78, 58), _kDim, .6);
  final soft = 1 + stretch;
  for (var i = 0; i < 7; i++) {
    final y0 = 68 + i * 7 * stretch, y1 = y0 + 3.2 * stretch;
    if (y0 > 122) break;
    final dx0 = skew * (y0 - 66) / 54, dx1 = skew * (y1 - 66) / 54;
    final q = [Offset(54 + dx0, y0), Offset(102 + dx0, y0), Offset(102 + dx1 + 4, y1), Offset(54 + dx1 - 4, y1)];
    _pl(c, q, _kWhite, close: true, w: .9);
    _pl(c, [for (final p in q) p + Offset(0, soft)], _al(_kRed, .4), close: true, w: .6);
  }
  final cat = Offset(30 + math.sin(t * .4) * 4, 64);
  _ov(c, cat + const Offset(0, -6), 7, 5, _kGrey, .9);
  _o(c, cat + const Offset(-6, -13), 3.5, _kGrey, .9);
  _kv(c, const Offset(150, 6), 'DIR', skew.abs(), _kBlue, h: 9, align: 1);
  _kv(c, const Offset(6, 6), 'DIST', stretch * 40, _kGreen, h: 9);
}

void _s20(Canvas c, Size s, _St st, double t) {
  final feet = Offset(st.at(s).dx.clamp(30.0, 126.0), st.at(s).dy.clamp(40.0, 100.0));
  const towers = [Offset(8, 10), Offset(148, 10), Offset(8, 112), Offset(148, 112)];
  for (final tw in towers) {
    final d = feet - tw, dist = d.distance;
    final len = (5200 / math.max(dist, 10)).clamp(40.0, 260.0);
    final u = d / dist, n = Offset(-u.dy, u.dx);
    final tip = feet + u * len;
    final soft = 1 + len / 60;
    for (var k = 3; k >= 1; k--) {
      final w = 3 + k * soft * 1.4;
      _pl(c, [feet + n * (2 + k * .6), tip + n * w, tip - n * w, feet - n * (2 + k * .6)], _al(_kRed, .45 - k * .1), close: true, w: .6);
    }
    _pl(c, [feet + n * 2, tip + n * 3, tip - n * 3, feet - n * 2], _kPurple, close: true, w: 1);
    _o(c, tw, 4, _kWhite);
    _ln(c, tw + Offset(0, tw.dy < 60 ? 5 : -5), tw + Offset(0, tw.dy < 60 ? 12 : -12), _kWhite);
    _dot(c, tw, 1.6, _kWhite);
  }
  _dash(c, const Offset(78, 4), const Offset(78, 116), _kDim, w: .6);
  _ov(c, const Offset(78, 60), 18, 18, _kDim, .6);
  _o(c, feet + const Offset(0, -2), 3, _kWhite, 1.2);
  _dot(c, feet + const Offset(0, -2), 1.2, _kWhite);
  _t(c, 'X4', const Offset(150, 52), 12, _kGreen, align: 1);
}
