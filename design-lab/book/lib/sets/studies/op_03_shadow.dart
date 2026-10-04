part of 'op_03.dart';

// Shadow: blue = direction, green = distance, red = softness (rings that spread), purple = the shadow itself.

const _shadowPanels = <_P>[
  _P('Sundial', 'object · effect · drag · drawing', _s01, init: Offset(.2, .3)),
  _P('Iso blocks', 'isometric · effect · drag · drawing', _s02, init: Offset(.25, .3)),
  _P('Desk lamp', 'machine · effect · drag · drawing', _s03, init: Offset(.35, .3)),
  _P('Drop 45', 'type · effect · drag · numeral', _s04, init: Offset(.65, .65)),
  _P('City at dusk', 'landscape · effect · flick · drawing', _s05),
  _P('Hand puppet', 'character · effect · drag · drawing', _s06, init: Offset(.4, .5)),
  _P('Light chain', 'diagram · mech · drag · chart', _s07, init: Offset(.35, .45)),
  _P('Plane overhead', 'vehicle · effect · drag · drawing', _s08, init: Offset(.3, .4)),
  _P('Eclipse cones', 'cosmic · mech · drag · chart', _s09, init: Offset(.5, .55)),
  _P('Weathervane', 'animal · effect · spin · drawing', _s10),
  _P('Flick the card', 'object · effect · flick · drawing', _s11),
  _P('Long word', 'type · effect · spin · type-led', _s12),
  _P('Spotlight', 'character · effect · drag · drawing', _s13, init: Offset(.3, .2)),
  _P('Halftone device', 'M4L · effect · drag · chart', _s14, init: Offset(.65, .65)),
  _P('Rub the day', 'landscape · effect · rub · drawing', _s15),
  _P('Lion inside', 'animal · effect · drag · PUSHED', _s16, init: Offset(.7, .4)),
  _P('Moon terminator', 'cosmic · effect · drag · drawing', _s17, init: Offset(.35, .5)),
  _P('Contour card', 'diagram · effect · drag · numeral', _s18, init: Offset(.62, .62)),
  _P('Blinds', 'interior · effect · drag · drawing', _s19, init: Offset(.6, .5)),
  _P('Stadium x4', 'character · effect · drag · EXTREME', _s20),
];

List<Offset> _hull(List<Offset> pts) {
  final p = [...pts]..sort((a, b) => a.dx != b.dx ? a.dx.compareTo(b.dx) : a.dy.compareTo(b.dy));
  double cr(Offset o, Offset a, Offset b) => (a.dx - o.dx) * (b.dy - o.dy) - (a.dy - o.dy) * (b.dx - o.dx);
  final lo = <Offset>[], up = <Offset>[];
  for (final q in p) {
    while (lo.length >= 2 && cr(lo[lo.length - 2], lo.last, q) <= 0) {
      lo.removeLast();
    }
    lo.add(q);
  }
  for (final q in p.reversed) {
    while (up.length >= 2 && cr(up[up.length - 2], up.last, q) <= 0) {
      up.removeLast();
    }
    up.add(q);
  }
  return [...lo..removeLast(), ...up..removeLast()];
}

Offset _cen(List<Offset> p) => p.fold(Offset.zero, (a, b) => a + b) / p.length.toDouble();

/// The shadow: purple outline plus red rings spreading out with softness.
void _shade(Canvas c, List<Offset> poly, double soft, {int n = 4, double sy = 1}) {
  final o = _cen(poly);
  for (var k = n; k >= 1; k--) {
    final g = 1 + k * soft * .09;
    _pl(c, [for (final q in poly) o + Offset((q.dx - o.dx) * g, (q.dy - o.dy) * (1 + (g - 1) * sy))], _al(_kRed, .55 - k * .1), w: .7, close: true);
  }
  _pl(c, poly, _kPurple, w: 1.2, close: true);
}

void _s01(Canvas c, Size s, _St st, double t) {
  const o = Offset(78, 82), rx = 62.0, ry = 22.0;
  _ov(c, o, rx, ry, _kWhite);
  _ov(c, o, rx - 6, ry - 2.5, _kDim, .8);
  for (var i = 0; i < 13; i++) {
    final a = math.pi + i / 12 * math.pi;
    final d = Offset(math.cos(a), math.sin(a) * ry / rx);
    _ln(c, o + d * (rx - 10), o + d * (rx - 6), _kGrey, .8);
  }
  final sun = st.at(s);
  final dir = math.atan2(o.dy - sun.dy, o.dx - sun.dx);
  final len = 12 + 60 * _sat(sun.dy / 70);
  for (var k = -2; k <= 2; k++) {
    final a = dir + k * .05;
    _ln(c, o, o + Offset(math.cos(a), math.sin(a) * .36) * len, k == 0 ? _kPurple : _al(_kRed, .5), k == 0 ? 2 : .7);
  }
  _pl(c, const [Offset(78, 82), Offset(78, 52), Offset(90, 82)], _kWhite);
  _o(c, sun, 6, _kWhite);
  for (var i = 0; i < 8; i++) {
    _ln(c, sun + _dir(i * math.pi / 4 + t * .3) * 9, sun + _dir(i * math.pi / 4 + t * .3) * 12, _kWhite, .8);
  }
  _kv(c, const Offset(150, 6), 'DIR', ((dir * 180 / math.pi) + 360) % 360, _kBlue, h: 9, align: 1);
  _kv(c, const Offset(116, 6), 'LEN', len, _kGreen, h: 9, align: 1);
}

void _s02(Canvas c, Size s, _St st, double t) {
  const o = Offset(78, 30), k = 7.0;
  final lx = (st.a - .5) * -3.2, ly = (st.p.dy - .5) * -3.2;
  final soft = 1 + _sat(Offset(lx, ly).distance / 2) * 4;
  for (var i = 0; i <= 10; i++) {
    for (var j = 0; j <= 10; j++) {
      c.drawCircle(_iso(o, k, i.toDouble(), j.toDouble(), 0), .5, _f(_kDim));
    }
  }
  const boxes = [(2.0, 2.0, 3.0), (6.0, 3.0, 2.0), (3.0, 6.5, 4.0)];
  for (final (bx, by, h) in boxes) {
    final g = <Offset>[];
    for (final (dx, dy) in const [(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0)]) {
      g.add(Offset(bx + dx, by + dy));
      g.add(Offset(bx + dx + lx * h, by + dy + ly * h));
    }
    _shade(c, [for (final q in _hull(g)) _iso(o, k, q.dx, q.dy, 0)], soft, n: 3);
  }
  for (final (bx, by, h) in boxes) {
    Offset p(double x, double y, double z) => _iso(o, k, bx + x, by + y, z);
    final faces = [
      [p(0, 2, 0), p(2, 2, 0), p(2, 2, h), p(0, 2, h)],
      [p(2, 0, 0), p(2, 2, 0), p(2, 2, h), p(2, 0, h)],
      [p(0, 0, h), p(2, 0, h), p(2, 2, h), p(0, 2, h)],
    ];
    for (final f in faces) {
      c.drawPath(Path()..addPolygon(f, true), _f(_kBg));
      _pl(c, f, _kWhite, close: true, w: .9);
    }
  }
  _kv(c, const Offset(6, 6), 'DIR', (math.atan2(ly, lx) * 180 / math.pi + 360) % 360, _kBlue, h: 9);
  _kv(c, const Offset(150, 6), 'SOFT', soft * 20, _kRed, h: 9, align: 1);
}

void _s03(Canvas c, Size s, _St st, double t) {
  const floor = 100.0, base = Offset(22, 96), l1 = 44.0, l2 = 40.0;
  var head = Offset(st.at(s).dx.clamp(30.0, 120.0), st.at(s).dy.clamp(8.0, 70.0));
  final d = (head - base).distance.clamp(10.0, l1 + l2 - .5);
  head = base + (head - base) / (head - base).distance * d;
  final a = math.atan2(head.dy - base.dy, head.dx - base.dx);
  final cb = ((l1 * l1 + d * d - l2 * l2) / (2 * l1 * d)).clamp(-1.0, 1.0);
  final elbow = base + _dir(a - math.acos(cb)) * l1;
  _ln(c, const Offset(0, floor), const Offset(156, floor), _kWhite);
  _ov(c, base + const Offset(0, 2), 10, 2.4, _kWhite);
  _pl(c, [base, elbow, head], _kWhite, w: 1.3);
  _o(c, elbow, 2, _kWhite, .9);
  const feet = Offset(104, floor);
  final aim = math.atan2(feet.dy - 14 - head.dy, feet.dx - head.dx);
  _pl(c, [head + _dir(aim + 1.9) * 5, head + _dir(aim + .5) * 10, head + _dir(aim - .5) * 10, head + _dir(aim - 1.9) * 5], _kWhite, close: true);
  _dash(c, head + _dir(aim + .5) * 10, head + _dir(aim + .5) * 90, _al(_kWhite, .3), w: .6);
  _dash(c, head + _dir(aim - .5) * 10, head + _dir(aim - .5) * 90, _al(_kWhite, .3), w: .6);
  const ho = 31.0;
  final hl = floor - head.dy, dx = feet.dx - head.dx;
  final len = hl > ho + 1 ? (dx * ho / (hl - ho)).clamp(-200.0, 200.0) : 200.0 * dx.sign;
  final tip = feet + Offset(len, 0);
  final soft = 1 + 4 * _sat(1 - hl / 90);
  _shade(c, [feet + const Offset(0, -1.5), tip + const Offset(0, -2.5), tip + const Offset(0, 2.5), feet + const Offset(0, 1.5)], soft, n: 3, sy: .2);
  _fig(c, feet, 1.75, _kWhite, arm: .2 + math.sin(t * 2) * .05);
  _dash(c, feet + const Offset(0, 6), tip + const Offset(0, 6), _kGreen);
  _kv(c, const Offset(150, 6), 'DIST', len.abs(), _kGreen, h: 9, align: 1);
}

void _s04(Canvas c, Size s, _St st, double t) {
  final off = (st.at(s) - const Offset(78, 60)) * .45;
  final soft = 1 + _sat(off.distance / 30) * 3;
  const at = Offset(70, 26), h = 58.0;
  for (var k = 4; k >= 1; k--) {
    _t(c, '45', at + off * (1 + k * .1 * soft), h, _al(_kRed, .5 - k * .09), align: .5, w: .8);
  }
  _t(c, '45', at + off, h, _kPurple, align: .5, w: 1.6);
  _t(c, '45', at, h, _kWhite, align: .5, w: 1.3);
  _dash(c, const Offset(78, 60), const Offset(78, 60) + off, _kGreen);
  _kv(c, const Offset(6, 6), 'ANGLE', (math.atan2(off.dy, off.dx) * 180 / math.pi + 360) % 360, _kBlue, h: 8);
  _kv(c, const Offset(150, 6), 'DIST', off.distance, _kGreen, h: 8, align: 1);
}

void _s05(Canvas c, Size s, _St st, double t) {
  final u = ((.3 + st.flow(t).dx * .5 + t * .01) % 1 + 1) % 1;
  const gy = 70.0, o = Offset(78, 70);
  final sa = math.pi + .15 + u * (math.pi - .3);
  final sun = o + _dir(sa) * 60;
  final elev = math.sin(sa).abs();
  c.drawArc(Rect.fromCircle(center: o, radius: 60), math.pi, math.pi, false, _s(_kDim, .7));
  _o(c, sun, 5, _kWhite);
  for (var i = 0; i < 8; i++) {
    _ln(c, sun + _dir(i * math.pi / 4) * 7, sun + _dir(i * math.pi / 4) * 9, _kWhite, .8);
  }
  const bs = [(8.0, 12.0, 22.0), (24.0, 10.0, 34.0), (40.0, 14.0, 16.0), (62.0, 8.0, 40.0), (76.0, 16.0, 24.0), (100.0, 10.0, 30.0), (116.0, 14.0, 18.0), (136.0, 12.0, 28.0)];
  final sx = -math.cos(sa) / math.max(elev, .12) * .7;
  for (final (x, w, h) in bs) {
    final dl = (h * sx).clamp(-160.0, 160.0);
    _shade(c, [Offset(x, gy), Offset(x + w, gy), Offset(x + w + dl, gy + h * .45), Offset(x + dl, gy + h * .45)], 1 + (1 - elev) * 2, n: 2);
  }
  for (final (x, w, h) in bs) {
    c.drawRect(Rect.fromLTWH(x, gy - h, w, h), _f(_kBg));
    c.drawRect(Rect.fromLTWH(x, gy - h, w, h), _s(_kWhite, .9));
  }
  _ln(c, const Offset(0, gy), const Offset(156, gy), _kWhite);
  final hh = (6 + u * 14).floor(), mm = ((6 + u * 14) % 1 * 60).floor();
  _t(c, '${hh.toString().padLeft(2, '0')}:${mm.toString().padLeft(2, '0')}', const Offset(150, 106), 9, _kBlue, align: 1);
  _lb(c, 'FLICK', const Offset(6, 108), _kDim);
}

const _dogUp = [
  Offset(-1.2, .8), Offset(-1.2, -.4), Offset(-.9, -.8), Offset(-.7, -1.6), Offset(-.3, -.85), Offset(.5, -.6), Offset(1.5, -.2), Offset(1.5, .15),
  Offset(.2, .15)
];

List<Offset> _dog(Offset o, double k, double open) => [
      for (final q in _dogUp) o + q * k,
      o + Offset(1.3, .35 + open) * k,
      o + Offset(1.1, .6 + open) * k,
      o + const Offset(-.2, .8) * k,
    ];

void _s06(Canvas c, Size s, _St st, double t) {
  const bulb = Offset(10, 50);
  final hx = 26 + st.a * 50, hy = 34 + st.p.dy * 40;
  final open = math.max(0.0, math.sin(t * 3)) * .25;
  _o(c, bulb, 5, _kWhite);
  _ln(c, bulb + const Offset(-3, 5), bulb + const Offset(-3, 9), _kWhite);
  _ln(c, bulb + const Offset(3, 5), bulb + const Offset(3, 9), _kWhite);
  _pl(c, const [Offset(92, 8), Offset(150, 4), Offset(150, 116), Offset(92, 108)], _kDim, close: true, w: .8);
  final sc = ((110 - bulb.dx) / (hx - bulb.dx)).clamp(1.0, 3.0);
  final sh = Offset(121, 56 + (hy - 50) * sc * .6);
  final soft = 1 + (sc - 1) * 1.2;
  c.save();
  c.clipPath(Path()..addPolygon(const [Offset(92, 8), Offset(150, 4), Offset(150, 116), Offset(92, 108)], true));
  _shade(c, _dog(sh, 8 * sc, open), soft, n: 3);
  c.restore();
  final hand = _dog(Offset(hx, hy), 8, open);
  _pl(c, hand, _kWhite, close: true);
  _dot(c, Offset(hx, hy) + const Offset(.2, -.3) * 8, 1, _kWhite);
  _pl(c, [Offset(hx, hy) + const Offset(-1.2, .8) * 8, Offset(hx - 16, hy + 26)], _kWhite);
  _pl(c, [Offset(hx, hy) + const Offset(-.2, .8) * 8, Offset(hx - 6, hy + 28)], _kWhite);
  _dash(c, bulb, Offset(hx, hy) + const Offset(-.7, -1.6) * 8, _al(_kWhite, .3), w: .6);
  _dash(c, Offset(hx, hy) + const Offset(-.7, -1.6) * 8, sh + const Offset(-.7, -1.6) * 8 * sc, _al(_kWhite, .3), w: .6);
  _kv(c, const Offset(6, 90), 'DIST', (hx - bulb.dx), _kGreen, h: 9);
  _kv(c, const Offset(40, 90), 'SOFT', soft * 15, _kRed, h: 9);
}

void _s07(Canvas c, Size s, _St st, double t) {
  final dir = st.a * 2 * math.pi, dist = st.b * 12;
  _o(c, const Offset(17, 36), 9, _kWhite);
  _ln(c, const Offset(14, 45), const Offset(20, 45), _kWhite);
  for (var i = 0; i < 6; i++) {
    _ln(c, const Offset(17, 36) + _dir(i * 1.05) * 11, const Offset(17, 36) + _dir(i * 1.05) * 13, _kGrey, .8);
  }
  _ln(c, const Offset(30, 36), const Offset(38, 36), _kGrey, .8);
  _pl(c, const [Offset(38, 23), Offset(38, 49), Offset(62, 36)], _kGreen, close: true);
  _t(c, (dist * 8).round().toString().padLeft(2, '0'), const Offset(40.5, 32.5), 7, _kGreen);
  _ln(c, const Offset(62, 36), const Offset(70, 36), _kGrey, .8);
  for (var i = 3; i >= 0; i--) {
    final r = Rect.fromLTWH(70 + i * 2.5, 25 - i * 2.5, 22, 22);
    c.drawRect(r, _f(_kBg));
    c.drawRect(r, _s(_al(_kRed, 1 - i * .22)));
  }
  _t(c, 'SFT', const Offset(73.5, 33.5), 5, _kRed, w: .8);
  _ln(c, const Offset(100, 36), const Offset(108, 36), _kGrey, .8);
  _o(c, const Offset(122, 36), 12, _kWhite);
  for (var i = 0; i < 8; i++) {
    _dot(c, const Offset(122, 36) + _dir(i * math.pi / 4) * 8, .9, _kDim);
  }
  _dot(c, const Offset(122, 36) + _dir(dir) * 8, 2, _kBlue);
  _lb(c, 'LIGHT', const Offset(17, 56), _kWhite, align: .5);
  _lb(c, 'DIST', const Offset(48, 56), _kGreen, align: .5);
  _lb(c, 'SOFT', const Offset(84, 56), _kRed, align: .5);
  _lb(c, 'DIR', const Offset(122, 56), _kBlue, align: .5);
  final off = _dir(dir) * dist;
  const sq = Rect.fromLTWH(66, 76, 24, 24);
  _shade(c, [for (final q in [sq.topLeft, sq.topRight, sq.bottomRight, sq.bottomLeft]) q + off], 2.5, n: 3);
  c.drawRect(sq, _f(_kBg));
  c.drawRect(sq, _s(_kWhite));
}

const _plane = [
  Offset(18, 0), Offset(14, -2), Offset(4, -2), Offset(-2, -16), Offset(-6, -16), Offset(-4, -2), Offset(-12, -2), Offset(-16, -7), Offset(-18, -7),
  Offset(-16, 0), Offset(-18, 7), Offset(-16, 7), Offset(-12, 2), Offset(-4, 2), Offset(-6, 16), Offset(-2, 16), Offset(4, 2), Offset(14, 2)
];

void _s08(Canvas c, Size s, _St st, double t) {
  final dir = math.pi * .25 + (st.a - .5) * 3, alt = st.b;
  final scroll = (t * 14) % 40;
  for (var i = -1; i < 6; i++) {
    final x = i * 40.0 - scroll;
    _ln(c, Offset(x, 0), Offset(x + 12, 120), _kDim, .7);
  }
  for (var j = 0; j < 4; j++) {
    _ln(c, Offset(0, 14 + j * 32.0), Offset(156, 8 + j * 32.0), _kDim, .7);
  }
  _pl(c, [for (var x = 0.0; x <= 156; x += 3) Offset(x, 96 + math.sin((x + t * 14) / 14) * 4)], _al(_kBlue, .5), w: .8);
  const o = Offset(66, 50);
  final off = _dir(dir) * (6 + alt * 40);
  _shade(c, [for (final q in _plane) o + off + q * (1.0 - alt * .25)], 1 + alt * 4, n: 3);
  c.drawPath(Path()..addPolygon([for (final q in _plane) o + q * 1.1], true), _f(_kBg));
  _pl(c, [for (final q in _plane) o + q * 1.1], _kWhite, close: true);
  _dash(c, o, o + off, _kGreen);
  _kv(c, const Offset(150, 6), 'ALT', alt * 99, _kGreen, h: 9, align: 1);
  _kv(c, const Offset(150, 92), 'SUN', (dir * 180 / math.pi + 360) % 360, _kBlue, h: 9, align: 1);
}

void _s09(Canvas c, Size s, _St st, double t) {
  const sun = Offset(16, 60), scrX = 150.0;
  final sr = 6 + (1 - st.b) * 10, mx = 46 + st.a * 70, my = 60 + math.sin(t * .4) * 3;
  const mr = 5.0;
  _o(c, sun, sr, _kRed);
  for (var i = 0; i < 10; i++) {
    _ln(c, sun + _dir(i * .628) * (sr + 2), sun + _dir(i * .628) * (sr + 4), _al(_kRed, .6), .8);
  }
  _o(c, Offset(mx, my), mr, _kWhite);
  _ln(c, const Offset(scrX, 10), const Offset(scrX, 110), _kWhite);
  double yAt(Offset a, Offset b) => a.dy + (b.dy - a.dy) * (scrX - a.dx) / (b.dx - a.dx);
  final ut = yAt(sun + Offset(0, -sr), Offset(mx, my - mr)), ub = yAt(sun + Offset(0, sr), Offset(mx, my + mr));
  final pt = yAt(sun + Offset(0, sr), Offset(mx, my - mr)), pb = yAt(sun + Offset(0, -sr), Offset(mx, my + mr));
  _dash(c, sun + Offset(0, -sr), Offset(scrX, ut), _al(_kWhite, .4), w: .6);
  _dash(c, sun + Offset(0, sr), Offset(scrX, ub), _al(_kWhite, .4), w: .6);
  _dash(c, sun + Offset(0, sr), Offset(scrX, pt), _al(_kRed, .5), w: .6);
  _dash(c, sun + Offset(0, -sr), Offset(scrX, pb), _al(_kRed, .5), w: .6);
  if (ut < ub) _ln(c, Offset(scrX - 3, ut), Offset(scrX - 3, ub), _kPurple, 2.4);
  _ln(c, Offset(scrX + 3, pt.clamp(0, 120)), Offset(scrX + 3, pb.clamp(0, 120)), _kRed, 1);
  _dash(c, Offset(mx, 104), const Offset(scrX, 104), _kGreen);
  _kv(c, const Offset(6, 6), 'SOFT', sr * 6, _kRed, h: 9);
  _kv(c, const Offset(40, 6), 'DIST', scrX - mx, _kGreen, h: 9);
}

const _rooster = [
  Offset(0, 0), Offset(3, -4), Offset(6, -4), Offset(8, -8), Offset(10, -5), Offset(9, -2), Offset(12, 0), Offset(8, 2), Offset(2, 2)
];

void _s10(Canvas c, Size s, _St st, double t) {
  final a = st.spin * .8 + .4 + math.sin(t * .9) * .06;
  const top = Offset(78, 34);
  _pl(c, const [Offset(4, 112), Offset(78, 70), Offset(152, 112)], _kWhite);
  _ln(c, const Offset(78, 70), top, _kWhite);
  for (final (k, l) in const [(0.0, 'E'), (math.pi / 2, 'S'), (math.pi, 'W'), (-math.pi / 2, 'N')]) {
    final d = Offset(math.cos(k) * 14, math.sin(k) * 5);
    _ln(c, top + const Offset(0, 8), top + const Offset(0, 8) + d, _kGrey, .8);
    _lb(c, l, top + const Offset(0, 8) + d * 1.3 + const Offset(-1.2, -2), _kGrey);
  }
  Offset pr(Offset base, double x, double y) => base + Offset(math.cos(a) * x, math.sin(a) * x * .35 + y);
  final vane = [
    pr(top, 18, 0), pr(top, 13, -3), pr(top, 13, 3), pr(top, 18, 0), pr(top, -12, 0),
    for (final q in _rooster) pr(top, -12 - q.dx, q.dy),
  ];
  const shOff = Offset(14, 40);
  final shv = [for (final q in vane) Offset(q.dx + shOff.dx + (q.dx - 78) * .2, q.dy + shOff.dy + (q.dx - 78) * .55)];
  _pl(c, shv, _kPurple, w: 1.1);
  _pl(c, [for (final q in shv) q + const Offset(1.5, 1.5)], _al(_kRed, .4), w: .7);
  _pl(c, vane, _kWhite);
  _dot(c, top, 1.6, _kWhite);
  _kv(c, const Offset(6, 6), 'DIR', ((a * 180 / math.pi) % 360 + 360) % 360, _kBlue, h: 10);
  _lb(c, 'SPIN', const Offset(150, 6), _kDim, align: 1);
}

void _s11(Canvas c, Size s, _St st, double t) {
  double hgt;
  if (st.down) {
    hgt = ((st.s0.dy - st.p.dy) * 120).clamp(0.0, 44.0);
  } else if (st.upAt > 0) {
    final dt = t - st.upAt, a0 = (-st.vel.dy * 14).clamp(0.0, 44.0);
    hgt = a0 * math.exp(-dt * 1.4) * math.cos(dt * 5).abs();
  } else {
    hgt = 0;
  }
  hgt += 6 + math.sin(t * 1.6) * 1.5;
  _pl(c, const [Offset(0, 100), Offset(30, 64), Offset(126, 64), Offset(156, 100)], _kDim, w: .8);
  const card = [Offset(44, 74), Offset(100, 74), Offset(110, 92), Offset(36, 92)];
  final off = Offset(hgt * .5, hgt * .12);
  _shade(c, [for (final q in card) q + off], 1 + hgt / 10, n: 4, sy: .5);
  final lifted = [for (final q in card) q - Offset(0, hgt)];
  c.drawPath(Path()..addPolygon(lifted, true), _f(_kBg));
  _pl(c, lifted, _kWhite, close: true);
  _ln(c, lifted[0] + const Offset(8, 4), lifted[1] + const Offset(-4, 4), _kGrey, .7);
  _ln(c, lifted[0] + const Offset(8, 8), lifted[1] + const Offset(-10, 8), _kGrey, .7);
  _dash(c, lifted[2], card[2] + off, _kGreen);
  _kv(c, const Offset(6, 6), 'LIFT', hgt * 2, _kGreen, h: 10);
  _lb(c, 'FLICK UP', const Offset(150, 6), _kDim, align: 1);
}

void _s12(Canvas c, Size s, _St st, double t) {
  final dir = st.spin * .7 + .6;
  const word = 'SHADOW', h = 16.0, at = Offset(78, 40);
  final step = _dir(dir) * 1.5;
  for (var k = 28; k >= 1; k--) {
    _t(c, word, at + step * k.toDouble(), h, _al(k > 22 ? _kRed : _kPurple, (1 - k / 30) * .8), align: .5, w: .8);
  }
  _t(c, word, at, h, _kWhite, align: .5, w: 1.3);
  const g = Offset(136, 100);
  _o(c, g, 8, _kDim, .8);
  _ln(c, g, g + _dir(dir) * 8, _kBlue, 1.2);
  _kv(c, const Offset(6, 90), 'DIR', ((dir * 180 / math.pi) % 360 + 360) % 360, _kBlue, h: 10);
}

void _s13(Canvas c, Size s, _St st, double t) {
  final lx = 30 + st.a * 96;
  final actor = Offset(78 + math.sin(t * .6) * 6, 98);
  c.drawRect(const Rect.fromLTRB(28, 14, 128, 82), _s(_kDim, .8));
  _pl(c, const [Offset(28, 82), Offset(6, 112)], _kDim, w: .8);
  _pl(c, const [Offset(128, 82), Offset(150, 112)], _kDim, w: .8);
  for (var i = 0; i < 4; i++) {
    final x = 4 + i * 5.0;
    _pl(c, [for (var y = 4.0; y <= 116; y += 4) Offset(x + math.sin(y / 8 + i) * 1.5, y)], _kRed, w: .7);
    _pl(c, [for (var y = 4.0; y <= 116; y += 4) Offset(152 - i * 5 + math.sin(y / 8 + i) * 1.5, y)], _kRed, w: .7);
  }
  final light = Offset(lx, 4);
  _dash(c, light, actor + const Offset(-14, 2), _al(_kWhite, .45), w: .6);
  _dash(c, light, actor + const Offset(14, 2), _al(_kWhite, .45), w: .6);
  _ov(c, actor + const Offset(0, 1), 16, 3, _al(_kWhite, .4), .7);
  c.drawRect(Rect.fromCenter(center: light, width: 10, height: 6), _s(_kWhite));
  final sc = 1.9, shx = actor.dx + (actor.dx - lx) * .5;
  final shp = Offset(shx, 80);
  _fig(c, shp, 1.6 * sc, _kPurple, w: 1.2);
  _fig(c, shp + const Offset(1.5, 1.5), 1.6 * sc, _al(_kRed, .4), w: .7);
  _fig(c, actor, 1.6, _kWhite, arm: .9);
  _kv(c, const Offset(36, 18), 'DIR', (lx - 78).abs(), _kBlue, h: 8);
}

void _s14(Canvas c, Size s, _St st, double t) {
  final off = (st.at(s) - const Offset(94, 64)) * .9;
  final soft = 11 + 4 * math.sin(t * .5);
  _ln(c, const Offset(4, 15), const Offset(152, 15), _kGrey, .8);
  _lb(c, 'SHADOW~', const Offset(6, 6), _kWhite);
  _lb(c, 'ON', const Offset(150, 6), _kGreen, align: 1);
  _ln(c, const Offset(32, 15), const Offset(32, 116), _kDim, .8);
  _kv(c, const Offset(6, 22), 'DIR', (math.atan2(off.dy, off.dx) * 180 / math.pi + 360) % 360 / 4, _kBlue, h: 7);
  _kv(c, const Offset(6, 52), 'DST', off.distance, _kGreen, h: 7);
  _kv(c, const Offset(6, 82), 'SFT', soft * 6, _kRed, h: 7);
  const obj = Rect.fromLTRB(80, 50, 108, 78);
  final sh = obj.shift(off);
  for (var i = 0; i < 15; i++) {
    for (var j = 0; j < 12; j++) {
      final p = Offset(40 + i * 7.6, 22 + j * 7.8);
      if (obj.inflate(2).contains(p)) continue;
      final dx = math.max(0.0, math.max(sh.left - p.dx, p.dx - sh.right)), dy = math.max(0.0, math.max(sh.top - p.dy, p.dy - sh.bottom));
      final v = _sat(1 - math.sqrt(dx * dx + dy * dy) / soft);
      _dot(c, p, .35 + 2.3 * v, v > .95 ? _kPurple : _kWhite);
    }
  }
  c.drawRect(obj, _s(_kWhite));
}

void _s15(Canvas c, Size s, _St st, double t) {
  final a = math.pi + .3 + ((.4 + st.rub * .5 + t * .02) % 1) * (math.pi - .6);
  const gy = 92.0, trunk = Offset(50, gy);
  final sun = const Offset(78, 92) + _dir(a) * 70;
  final elev = math.sin(a).abs();
  _o(c, sun, 5, _kWhite);
  _ln(c, const Offset(0, gy), const Offset(156, gy), _kWhite);
  final len = (-math.cos(a) / math.max(elev, .15)) * 40;
  final soft = 1 + (1 - elev) * 4;
  final cc = trunk + Offset(len, 12);
  _shade(c, _loop(cc, (th) => 22 + 3 * _vn(th * 3, 2), n: 36, sy: .22), soft, n: 3, sy: .22);
  _ln(c, trunk, cc + Offset(-len.sign * 10, -2), _kPurple, 1.2);
  _ln(c, const Offset(47, gy), const Offset(47, 60), _kWhite);
  _ln(c, const Offset(53, gy), const Offset(53, 60), _kWhite);
  final can = _loop(const Offset(50, 42), (th) => 22 + 3 * _vn(th * 3, 2) + math.sin(t * 1.3 + th * 2) * .6, n: 48);
  c.drawPath(Path()..addPolygon(can, true), _f(_kBg));
  _pl(c, can, _kGreen, close: true);
  _ln(c, const Offset(50, 64), const Offset(42, 52), _kWhite, .9);
  _ln(c, const Offset(50, 60), const Offset(60, 48), _kWhite, .9);
  _kv(c, const Offset(150, 100), 'LEN', len.abs(), _kGreen, h: 8, align: 1);
  _lb(c, 'RUB', const Offset(6, 108), _kDim);
}

void _s16(Canvas c, Size s, _St st, double t) {
  final dx = (st.a - .5) * 70, sc = 1 + st.b * 1.8;
  const floor = 104.0;
  _ln(c, const Offset(0, floor), const Offset(156, floor), _kWhite);
  final base = Offset(54 + dx, floor);
  Offset sh(Offset q) => base + Offset((q.dx - 54) * sc, (q.dy - floor) * sc);
  final mane = _loop(const Offset(54, 64), (th) => 13 + 2.4 * math.sin(th * 9 + t * 2), n: 60);
  _pl(c, [for (final q in mane) sh(q)], _kPurple, close: true, w: 1.1);
  _pl(c, [for (final q in _loop(const Offset(54, 88), (_) => 14, n: 30, sy: 1.15)) sh(q)], _kPurple, close: true, w: 1.1);
  _pl(c, [sh(const Offset(66, 100)), sh(const Offset(84, 98)), sh(const Offset(88, 84)), sh(const Offset(92, 80))], _kPurple, w: 1.1);
  _o(c, sh(const Offset(92, 80)), 2.5 * sc, _kPurple, 1.1);
  _pl(c, [for (final q in _loop(const Offset(54, 88), (_) => 14, n: 30, sy: 1.15)) sh(q) + const Offset(2, 1)], _al(_kRed, .4), close: true, w: .7);
  final body = _loop(const Offset(102, 92), (_) => 9, n: 30, sy: 1.3);
  c.drawPath(Path()..addPolygon(body, true), _f(_kBg));
  _pl(c, body, _kWhite, close: true);
  c.drawCircle(const Offset(102, 74), 6.5, _f(_kBg));
  _o(c, const Offset(102, 74), 6.5, _kWhite);
  _pl(c, const [Offset(97, 70), Offset(97, 64), Offset(101, 68)], _kWhite);
  _pl(c, const [Offset(103, 68), Offset(107, 64), Offset(107, 70)], _kWhite);
  _dot(c, const Offset(100, 74), .8, _kWhite);
  _dot(c, const Offset(105, 74), .8, _kWhite);
  _pl(c, [const Offset(110, 100), Offset(120, 98 + math.sin(t * 2) * 2), const Offset(122, 88)], _kWhite);
  _kv(c, const Offset(6, 6), 'DIST', st.b * 99, _kGreen, h: 9);
  _kv(c, const Offset(150, 6), 'DIR', (st.a * 360), _kBlue, h: 9, align: 1);
}

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
