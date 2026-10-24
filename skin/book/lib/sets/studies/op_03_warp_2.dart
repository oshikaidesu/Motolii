// Part of op_03_warp.dart (cut there by size, as it was).
part of 'op_03.dart';

void _w18(Canvas c, Size s, _St st, double t) {
  final hole = st.at(s);
  final re = 12 + 3 * math.sin(t * .6);
  Offset lens(Offset q) {
    final d = q - hole, dd = math.max(d.distance, .5);
    return hole + d / dd * (dd + re * re / dd);
  }

  for (var y = 8.0; y < 120; y += 14) {
    _plw(c, [Offset(0, y), Offset(156, y)], lens, _al(_kPurple, .5), w: .6);
  }
  for (var i = 0; i < 70; i++) {
    final p = Offset((_hash(i, 21) * .5 + .5) * 156, (_hash(i, 22) * .5 + .5) * 120);
    _dot(c, lens(p), .9, _kWhite);
  }
  _o(c, hole, re, _kGreen, .9);
  c.drawCircle(hole, re * .55, _f(_kBg));
  _o(c, hole, re * .55, _kWhite, .8);
  _kv(c, const Offset(6, 6), 'MASS', re * 5, _kBlue, h: 9);
}

void _w19(Canvas c, Size s, _St st, double t) {
  final p = st.at(s), s0 = Offset(st.s0.dx * s.width, st.s0.dy * s.height);
  final k = st.down ? 1.0 : (st.upAt > 0 ? math.exp(-(t - st.upAt) * .8) * math.cos((t - st.upAt) * 6) : 0.0);
  const r = 22.0;
  final pull = (p - s0) * k + Offset(0, math.sin(t * 1.4) * 1.2);
  Offset wf(Offset q) => q + pull * _g((q - s0).distance, r);
  _plw(c, _loop(const Offset(78, 62), (_) => 36, n: 40, sy: 1.12), wf, _kWhite, close: true);
  for (final e in const [Offset(64, 54), Offset(92, 54)]) {
    _plw(c, _loop(e, (_) => 5, n: 16), wf, _kWhite, close: true);
    _dot(c, wf(e + const Offset(1, 1)), 1.6, _kWhite);
  }
  _plw(c, const [Offset(78, 58), Offset(74, 72), Offset(80, 73)], wf, _kWhite);
  _plw(c, [for (var a = .3; a <= math.pi - .3; a += .2) const Offset(78, 78) + Offset(math.cos(a) * 12, math.sin(a) * 6)], wf, _kRed);
  for (var i = 0; i < 7; i++) {
    final x = 56 + i * 7.0;
    _plw(c, [Offset(x, 26 - (i % 2) * 2), Offset(x + 3, 18)], wf, _kGrey, w: .8);
  }
  _plw(c, _loop(const Offset(40, 62), (_) => 5, n: 12, sy: 1.6), wf, _kWhite, close: true);
  _plw(c, _loop(const Offset(116, 62), (_) => 5, n: 12, sy: 1.6), wf, _kWhite, close: true);
  if (st.down) {
    _o(c, s0, r, _al(_kGreen, .6), .7);
    _ln(c, s0, p, _kBlue, .9);
  }
  _kv(c, const Offset(6, 6), 'PULL', (pull.distance).clamp(0, 99), _kBlue, h: 9);
  _kv(c, const Offset(150, 6), 'RADIUS', r, _kGreen, h: 9, align: 1);
}

void _w20(Canvas c, Size s, _St st, double t) {
  final s0 = st.down || st.upAt > 0 ? Offset(st.s0.dx * s.width, st.s0.dy * s.height) : const Offset(78, 60);
  Offset d;
  if (st.down) {
    d = st.at(s) - s0;
  } else if (st.upAt > 0) {
    final dt = t - st.upAt;
    d = (st.at(s) - s0) * (math.exp(-dt * 1.2) * math.cos(dt * 7));
  } else {
    d = Offset(math.sin(t * .9) * 40, math.cos(t * .7) * 18);
  }
  const sv = 2.4, r = 46.0;
  final tw = d.dx / 30;
  Offset wf(Offset q) {
    final g = _g((q - s0).distance, r);
    return s0 + _rot(q - s0, tw * g) + d * sv * g;
  }

  for (var i = 0; i <= 13; i++) {
    _plw(c, [Offset(i * 12.0, -4), Offset(i * 12.0, 124)], wf, i % 3 == 0 ? _kBlue : _al(_kWhite, .8), w: .8);
  }
  for (var j = 0; j <= 10; j++) {
    _plw(c, [Offset(-4, j * 12.0), Offset(160, j * 12.0)], wf, j % 3 == 0 ? _kRed : _al(_kWhite, .8), w: .8);
  }
  _t(c, 'X999', const Offset(150, 104), 9, _kGreen, align: 1);
}
