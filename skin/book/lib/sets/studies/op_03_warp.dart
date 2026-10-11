part of 'op_03.dart';

// Warp / Distort: blue = strength, green = radius, red = twist.

const _warpPanels = <_P>[
  _P('Bathtub drain', 'object · effect · spin · drawing', _w01, init: Offset(.75, .45)),
  _P('Jelly floor', 'isometric · effect · press · drawing', _w02, init: Offset(.5, .5)),
  _P('Funhouse mirror', 'character · effect · drag · drawing', _w03, init: Offset(.7, .45)),
  _P('Wring the towel', 'object · effect · flick · drawing', _w04),
  _P('Tutu twirl', 'character · effect · drag · drawing', _w05, init: Offset(.7, .4)),
  _P('Loupe 08', 'type · effect · drag · numeral', _w06, init: Offset(.45, .5)),
  _P('Gravity well', 'cosmic · mech · drag · chart', _w07, init: Offset(.5, .35)),
  _P('Goldfish bowl', 'animal · effect · drag · drawing', _w08, init: Offset(.6, .4)),
  _P('Warp chain', 'diagram · mech · drag · chart', _w09, init: Offset(.55, .4)),
  _P('Heat haze', 'vehicle · effect · drag · drawing', _w10, init: Offset(.4, .45)),
  _P('Melting clock', 'object · effect · drag · PUSHED', _w11, init: Offset(.5, .35)),
  _P('Scanline shove', 'M4L · mech · drag · chart', _w12, init: Offset(.55, .5)),
  _P('Tornado', 'land · effect · drag · PUSHED', _w13, init: Offset(.6, .4)),
  _P('Warped record', 'instrument · effect · drag · drawing', _w14, init: Offset(.4, .45)),
  _P('Spiral type', 'type · effect · spin · type-led', _w15),
  _P('Twirl spaghetti', 'object · effect · spin · drawing', _w16),
  _P('Web twist', 'animal · mech · drag · chart', _w17, init: Offset(.6, .45)),
  _P('Lensing', 'cosmic · effect · drag · drawing', _w18, init: Offset(.55, .5)),
  _P('Rubber face', 'character · effect · pull · drawing', _w19),
  _P('Taffy x999', 'grid · effect · pull · EXTREME', _w20),
];

Offset Function(Offset) _swirl(Offset o, double tw, double r) => (q) {
      final d = q - o;
      return o + _rot(d, tw * _g(d.distance, r));
    };

void _w01(Canvas c, Size s, _St st, double t) {
  const o = Offset(92, 62);
  final tw = 1.5 + st.spin * 1.2, r = 14 + st.b * 40;
  final tub = RRect.fromLTRBR(8, 10, 148, 110, const Radius.circular(24));
  c.drawRRect(tub, _s(_kWhite));
  final inner = RRect.fromLTRBR(14, 16, 142, 104, const Radius.circular(18));
  c.drawRRect(inner, _s(_kDim, .8));
  c.save();
  c.clipRRect(inner);
  final wf = _swirl(o, tw + t * .6, r);
  for (var y = 22.0; y < 104; y += 7) {
    _plw(c, [Offset(10, y), Offset(146, y)], wf, _al(_kWhite, .7), w: .8);
  }
  c.restore();
  _ov(c, o, r, r, _al(_kGreen, .4), .7);
  _o(c, o, 3.5, _kRed);
  _ln(c, o + const Offset(-2, 0), o + const Offset(2, 0), _kRed, .8);
  final dk = o + _dir(t * .9 + tw) * (r * .8 + 8);
  _ov(c, dk, 5, 3, _kWhite);
  _o(c, dk + const Offset(4, -4), 2.4, _kWhite);
  _ln(c, dk + const Offset(6, -4), dk + const Offset(9, -3.5), _kRed);
  _kv(c, const Offset(22, 24), 'TWIST', (tw * 10).abs(), _kRed, h: 10);
  _kv(c, const Offset(22, 78), 'RADIUS', r, _kGreen, h: 10);
}

void _w02(Canvas c, Size s, _St st, double t) {
  const o = Offset(78, 28), k = 7.0;
  final pa = (st.at(s).dx - o.dx) / (k * .866), pb = (st.at(s).dy - o.dy) / (k * .5);
  final cx = (pa + pb) / 2, cy = (pb - pa) / 2;
  double sv;
  if (st.down) {
    sv = .35 + .65 * _sat((t - st.downAt) * 2.5);
  } else if (st.upAt > 0) {
    final dt = t - st.upAt;
    sv = .35 + .65 * math.exp(-dt * 2) * math.cos(dt * 9);
  } else {
    sv = .35 + .05 * math.sin(t * 2);
  }
  const r = 2.4;
  double z(double x, double y) => 4.5 * sv * _g(Offset(x - cx, y - cy).distance, r);
  for (var i = 0; i <= 10; i++) {
    _pl(c, [for (var j = 0.0; j <= 10; j += .5) _iso(o, k, i.toDouble(), j, z(i.toDouble(), j))], _kBlue, w: .8);
    _pl(c, [for (var j = 0.0; j <= 10; j += .5) _iso(o, k, j, i.toDouble(), z(j, i.toDouble()))], _al(_kWhite, .5), w: .8);
  }
  _pl(c, [for (var a = 0.0; a <= 6.3; a += .2) _iso(o, k, cx + math.cos(a) * r, cy + math.sin(a) * r, 0)], _kGreen, w: .8);
  _kv(c, const Offset(6, 8), 'STRENGTH', sv * 99, _kBlue, h: 10);
  _kv(c, const Offset(150, 8), 'RADIUS', r * 10, _kGreen, h: 10, align: 1);
  _lb(c, 'PRESS', const Offset(150, 108), _kDim, align: 1);
}

List<List<Offset>> _figLines(Offset feet, double k) {
  final hip = feet + Offset(0, -26 * k), neck = hip + Offset(0, -24 * k), head = neck + Offset(0, -9 * k);
  return [
    _loop(head, (_) => 8 * k, n: 24),
    [neck, hip],
    [feet + Offset(-8 * k, 0), hip, feet + Offset(8 * k, 0)],
    [neck + Offset(-14 * k, 16 * k), neck + Offset(0, 3 * k), neck + Offset(14 * k, 16 * k)],
    [neck + Offset(-6 * k, 2 * k), hip + Offset(-7 * k, 0), hip + Offset(7 * k, 0), neck + Offset(6 * k, 2 * k)],
  ];
}

void _w03(Canvas c, Size s, _St st, double t) {
  final sv = (st.a - .3) * 2.6, fy = st.p.dy * s.height;
  const r = 12.0;
  final frame = RRect.fromLTRBR(66, 6, 136, 104, const Radius.circular(34));
  c.drawRRect(frame, _s(_kWhite));
  c.drawRRect(frame.deflate(4), _s(_kDim, .8));
  _ln(c, const Offset(80, 104), const Offset(74, 114), _kWhite);
  _ln(c, const Offset(122, 104), const Offset(128, 114), _kWhite);
  const cx = 101.0;
  final sway = math.sin(t * 1.5) * 1.5;
  Offset wf(Offset q) => Offset(cx + (q.dx - cx) * (1 + sv * _g(q.dy - fy, r)), q.dy);
  c.save();
  c.clipRRect(frame.deflate(4));
  for (final l in _figLines(Offset(cx + sway, 96), 1.05)) {
    _plw(c, l, wf, _kWhite);
  }
  c.restore();
  for (final l in _figLines(Offset(34 + sway * .6, 104), .62)) {
    _pl(c, l, _kGrey, w: .9);
  }
  _ln(c, Offset(140, fy - r), Offset(140, fy + r), _kGreen);
  _ln(c, Offset(62, fy), Offset(68, fy), _kBlue);
  _ln(c, Offset(134, fy), Offset(142, fy), _kBlue);
  _kv(c, const Offset(6, 8), 'BULGE', (sv * 40).abs(), _kBlue, h: 12);
  _lb(c, sv < 0 ? 'PINCH' : 'PUFF', const Offset(6, 30), _kBlue);
}

void _w04(Canvas c, Size s, _St st, double t) {
  final turns = (1 + st.flow(t).dx * 3).clamp(0.0, 9.0);
  const x0 = 20.0, x1 = 136.0, cy = 56.0, hh = 15.0;
  double ph(double x) => (x - x0) / (x1 - x0) * turns * math.pi;
  final top = <Offset>[], bot = <Offset>[];
  for (var x = x0; x <= x1; x += 1.5) {
    final cs = math.cos(ph(x));
    top.add(Offset(x, cy - hh * cs));
    bot.add(Offset(x, cy + hh * cs));
  }
  _pl(c, top, _kWhite);
  _pl(c, bot, _kWhite);
  for (var i = 1; i < 12; i++) {
    final x = x0 + i * (x1 - x0) / 12, cs = math.cos(ph(x));
    _ln(c, Offset(x, cy - hh * cs), Offset(x, cy + hh * cs), cs > 0 ? _kRed : _al(_kRed, .35), .8);
  }
  c.drawRect(const Rect.fromLTRB(10, 36, 20, 76), _s(_kGrey));
  c.drawRect(const Rect.fromLTRB(136, 36, 146, 76), _s(_kGrey));
  final drip = _sat((turns - 2) / 5);
  for (var i = 0; i < (drip * 9).round(); i++) {
    final x = x0 + 10 + _hash(i, 3).abs() * 96, y = 74 + ((t * 30 + i * 13) % 34);
    _ln(c, Offset(x, y), Offset(x, y + 2.5), _kGreen, 1.2);
  }
  _ln(c, const Offset(8, 110), const Offset(148, 110), _kDim, .8);
  _kv(c, const Offset(6, 6), 'TURNS', turns * 10, _kRed, h: 12);
  _kv(c, const Offset(150, 6), 'DAMP', (1 - drip) * 99, _kGreen, h: 12, align: 1);
  _lb(c, 'FLICK >', const Offset(150, 90), _kDim, align: 1);
}

void _w05(Canvas c, Size s, _St st, double t) {
  final tw = (st.a - .5) * 5 + math.sin(t * 1.2) * .25, r = 14 + st.b * 26;
  const w0 = Offset(78, 56);
  _o(c, const Offset(78, 18), 5, _kWhite);
  _ln(c, const Offset(78, 23), w0, _kWhite);
  _pl(c, const [Offset(78, 30), Offset(66, 20), Offset(76, 8)], _kWhite);
  _pl(c, const [Offset(78, 30), Offset(90, 20), Offset(80, 8)], _kWhite);
  _ln(c, w0, const Offset(74, 102), _kWhite);
  _pl(c, const [w0, Offset(86, 80), Offset(80, 102)], _kWhite);
  _ov(c, w0, 6, 1.6, _kWhite, .8);
  final outer = <Offset>[];
  for (var i = 0; i < 40; i++) {
    final a = i / 40 * 2 * math.pi;
    outer.add(w0 + Offset(math.cos(a + tw) * r, math.sin(a + tw) * r * .26 + 3));
  }
  _pl(c, outer, _kGreen, close: true);
  for (var i = 0; i < 18; i++) {
    final a = i / 18 * 2 * math.pi;
    if (math.sin(a) < -.2) continue;
    final p0 = w0 + Offset(math.cos(a) * 6, math.sin(a) * 1.6);
    final p1 = w0 + Offset(math.cos(a + tw) * r, math.sin(a + tw) * r * .26 + 3);
    _ln(c, p0, p1, _kRed, .8);
  }
  _dash(c, const Offset(36, 104), const Offset(120, 104), _kDim);
  for (var k = 0; k < 3; k++) {
    c.drawArc(Rect.fromCenter(center: w0, width: r * 2 + 16 + k * 8, height: r * .6 + 10 + k * 4), .3 + tw * .2, .9, false,
        _s(_al(_kWhite, .4 - k * .1), .7));
  }
  _kv(c, const Offset(6, 8), 'TWIST', (tw * 20).abs(), _kRed, h: 10);
  _kv(c, const Offset(150, 8), 'RADIUS', r, _kGreen, h: 10, align: 1);
}

void _w06(Canvas c, Size s, _St st, double t) {
  final l = st.at(s);
  final r = 16 + (st.down ? _sat(t - st.downAt) * 12 : 6.0) + math.sin(t * 1.5);
  const sv = .75;
  Offset wf(Offset q) {
    final d = q - l, dd = d.distance;
    if (dd >= r) return q;
    return l + d * (1 + sv * (1 - (dd / r) * (dd / r)));
  }

  _t(c, '08', const Offset(78, 24), 70, _kDim, align: .5, w: .8);
  _t(c, '08', const Offset(78, 24), 70, _kWhite, align: .5, warp: wf, w: 1.2);
  _o(c, l, r, _kGreen);
  _o(c, l, r + 3, _kGreen, .6);
  _ln(c, l + _dir(.8) * (r + 3), l + _dir(.8) * (r + 22), _kWhite, 2);
  _kv(c, const Offset(6, 6), 'MAG', sv * 99, _kBlue, h: 9);
  _kv(c, const Offset(150, 6), 'RADIUS', r, _kGreen, h: 9, align: 1);
}

void _w07(Canvas c, Size s, _St st, double t) {
  final r = 1.2 + st.a * 3, sv = st.b;
  Offset pr(double x, double z, double dip) => Offset(78 + x * (6 + z * .9), 22 + z * 9 + dip);
  double dip(double x, double z) => sv * 40 * _g(Offset(x, (z - 4) * 1.4).distance, r);
  for (var zi = 0; zi <= 8; zi++) {
    _pl(c, [for (var x = -7.0; x <= 7; x += .5) pr(x, zi.toDouble(), dip(x, zi.toDouble()))], _al(_kBlue, .9), w: .8);
  }
  for (var xi = -7; xi <= 7; xi++) {
    _pl(c, [for (var z = 0.0; z <= 8; z += .25) pr(xi.toDouble(), z, dip(xi.toDouble(), z))], _al(_kWhite, .45), w: .7);
  }
  final b = pr(0, 4, dip(0, 4)) + const Offset(0, -6);
  _o(c, b, 6, _kWhite);
  final orb = b + Offset(math.cos(t * 2) * (14 + r * 4), math.sin(t * 2) * (4 + r));
  _dot(c, orb, 1.6, _kRed);
  _kv(c, const Offset(6, 6), 'MASS', sv * 99, _kBlue, h: 9);
  _kv(c, const Offset(150, 6), 'RADIUS', r * 20, _kGreen, h: 9, align: 1);
}

void _w08(Canvas c, Size s, _St st, double t) {
  const o = Offset(78, 64), br = 40.0;
  final sv = st.a * 1.2, r = 18 + st.b * 22;
  Offset lens(Offset q) {
    final d = q - o, dd = d.distance;
    if (dd >= r) return q;
    return o + d * (1 + sv * (1 - (dd / r) * (dd / r)));
  }

  for (var x = 6.0; x < 152; x += 7) {
    _plw(c, [Offset(x, 4), Offset(x, 116)], lens, _al(_kPurple, .7), w: .8);
  }
  c.drawArc(Rect.fromCircle(center: o, radius: br), -1.05, 5.24, false, _s(_kWhite));
  _ov(c, o + const Offset(0, -35), 20, 3.5, _kWhite);
  _o(c, o, r, _al(_kGreen, .5), .7);
  final fp = o + Offset(math.cos(t * .7) * 18, math.sin(t * 1.3) * 8);
  final dirx = math.sin(t * .7) > 0 ? -1.0 : 1.0;
  final body = [for (var a = 0.0; a <= 2 * math.pi; a += .3) fp + Offset(math.cos(a) * 8 * dirx, math.sin(a) * 4.2)];
  _plw(c, body, lens, _kRed, close: true);
  _plw(c, [fp + Offset(-8 * dirx, 0), fp + Offset(-14 * dirx, -4), fp + Offset(-14 * dirx, 4), fp + Offset(-8 * dirx, 0)], lens, _kRed);
  _dot(c, lens(fp + Offset(4.5 * dirx, -1)), 1, _kWhite);
  _ln(c, const Offset(30, 106), const Offset(126, 106), _kDim, .8);
  _kv(c, const Offset(6, 6), 'STRENGTH', sv * 80, _kBlue, h: 9);
  _kv(c, const Offset(150, 6), 'RADIUS', r, _kGreen, h: 9, align: 1);
}

void _w09(Canvas c, Size s, _St st, double t) {
  final tw = (st.a - .5) * 6, sv = st.b;
  c.drawRect(const Rect.fromLTRB(6, 30, 28, 52), _s(_kWhite));
  for (var i = 1; i < 4; i++) {
    _ln(c, Offset(6 + i * 5.5, 30), Offset(6 + i * 5.5, 52), _kGrey, .6);
    _ln(c, Offset(6, 30 + i * 5.5), Offset(28, 30 + i * 5.5), _kGrey, .6);
  }
  _ln(c, const Offset(28, 41), const Offset(36, 41), _kGrey, .8);
  _pl(c, const [Offset(36, 28), Offset(36, 54), Offset(60, 41)], _kBlue, close: true);
  _t(c, (sv * 99).round().toString().padLeft(2, '0'), const Offset(38.5, 37.5), 7, _kBlue);
  _ln(c, const Offset(60, 41), const Offset(68, 41), _kGrey, .8);
  c.drawRect(const Rect.fromLTRB(68, 30, 90, 52), _s(_kRed));
  _pl(c, [for (var a = 0.0; a < 4 * math.pi; a += .2) const Offset(79, 41) + _dir(a + t + tw) * (a / (4 * math.pi) * 9)], _kRed, w: .8);
  _ln(c, const Offset(90, 41), const Offset(108, 41), _kGrey, .8);
  _o(c, const Offset(120, 41), 12, _kWhite);
  _o(c, const Offset(120, 41), 7, _kGreen);
  for (var i = 0; i < 6; i++) {
    _dot(c, const Offset(120, 41) + _dir(i / 6 * 2 * math.pi + t * .3) * 10, 1, _kGrey);
  }
  _lb(c, 'INPUT', const Offset(17, 60), _kWhite, align: .5);
  _lb(c, 'STRENGTH', const Offset(48, 60), _kBlue, align: .5);
  _lb(c, 'TWIST', const Offset(79, 60), _kRed, align: .5);
  _lb(c, 'RADIUS', const Offset(120, 60), _kGreen, align: .5);
  final wf = _swirl(const Offset(78, 92), tw * sv * 2, 18);
  for (var y = 76.0; y <= 110; y += 5) {
    _plw(c, [Offset(10, y), Offset(146, y)], wf, _al(_kWhite, .8), w: .8);
  }
}

void _w10(Canvas c, Size s, _St st, double t) {
  const hz = 56.0;
  final r = 6 + st.a * 30, sv = st.b * 5;
  Offset wf(Offset q) {
    final k = _g(q.dy - hz, r);
    return q + Offset(sv * math.sin(q.dy * 1.1 + t * 7) * k, sv * .3 * math.sin(q.dx * .3 + t * 5) * k);
  }

  _o(c, const Offset(128, 22), 9, _kWhite);
  final sky = <Offset>[const Offset(0, hz)];
  var x = 0.0;
  for (var i = 0; x < 156; i++) {
    final hgt = 6 + (_hash(i, 9).abs()) * 16, w = 6 + _hash(i, 4).abs() * 8;
    sky.addAll([Offset(x, hz - hgt), Offset(x + w, hz - hgt)]);
    x += w;
  }
  sky.add(const Offset(156, hz));
  _plw(c, sky, wf, _al(_kWhite, .8), w: .9);
  _plw(c, const [Offset(74, hz), Offset(14, 116)], wf, _kWhite);
  _plw(c, const [Offset(82, hz), Offset(142, 116)], wf, _kWhite);
  final cz = (t * .15) % 1, cy = hz + 4 + cz * cz * 30, cw = 6 + cz * 22;
  _plw(c, [Offset(78 - cw / 2, cy), Offset(78 - cw / 2, cy - cw * .55), Offset(78 + cw / 2, cy - cw * .55), Offset(78 + cw / 2, cy)], wf, _kRed,
      close: true);
  _dash(c, Offset(4, hz - r), Offset(152, hz - r), _al(_kGreen, .6));
  _dash(c, Offset(4, hz + r), Offset(152, hz + r), _al(_kGreen, .6));
  _kv(c, const Offset(6, 6), 'HAZE', sv * 20, _kBlue, h: 9);
  _lb(c, 'BAND ${r.round()}', const Offset(6, 108), _kGreen);
}

void _w11(Canvas c, Size s, _St st, double t) {
  const o = Offset(76, 44), ex = 84.0, rr = 26.0;
  final sv = .3 + st.b * 2.4, soft = 6 + st.a * 30;
  Offset wf(Offset q) {
    if (q.dx <= ex) return q + Offset(0, sv * 3 * _sat((q.dx - 60) / 40));
    final u = q.dx - ex, th = sv * 1.2 * _sat(u / soft);
    return Offset(ex + u * math.cos(th), q.dy + sv * 3 + u * math.sin(th) * 1.2);
  }

  _plw(c, _loop(o, (_) => rr, n: 48), wf, _kWhite, close: true);
  for (var i = 0; i < 12; i++) {
    final a = i / 12 * 2 * math.pi;
    _plw(c, [o + _dir(a) * (rr - 4), o + _dir(a) * (rr - 1)], wf, _kGrey, w: .8);
  }
  _plw(c, [o, o + _dir(t * .5) * 18], wf, _kRed);
  _plw(c, [o, o + _dir(t * .05 - 1) * 12], wf, _kRed);
  _ln(c, const Offset(0, 74), const Offset(ex, 74), _kWhite);
  _ln(c, const Offset(ex, 74), const Offset(ex, 120), _kWhite);
  _ln(c, const Offset(ex - 6, 80), const Offset(ex - 6, 120), _kDim);
  final tip = wf(o + const Offset(rr, 0));
  final dy = (t * 12) % 30;
  _dot(c, tip + Offset(0, 4 + dy), 1.2 * (1 - dy / 30) + .3, _kBlue);
  _kv(c, const Offset(6, 84), 'MELT', sv * 40, _kBlue, h: 10);
  _kv(c, const Offset(40, 84), 'SOFT', soft, _kGreen, h: 10);
}

void _w12(Canvas c, Size s, _St st, double t) {
  final fy = st.p.dy * s.height, sv = (st.a - .2) * 30;
  const r = 12.0;
  _ln(c, const Offset(4, 15), const Offset(152, 15), _kGrey, .8);
  _lb(c, 'SHOVE~', const Offset(6, 6), _kWhite);
  _lb(c, 'ON', const Offset(150, 6), _kGreen, align: 1);
  _ln(c, const Offset(32, 15), const Offset(32, 116), _kDim, .8);
  _kv(c, const Offset(6, 22), 'STR', sv.abs() * 3, _kBlue, h: 7);
  _kv(c, const Offset(6, 52), 'RAD', r, _kGreen, h: 7);
  for (var i = 0; i < 23; i++) {
    final x = 38 + i * 5.0;
    _plw(c, [Offset(x, 20), Offset(x, 114)], (q) => q + Offset(sv * _g(q.dy - fy, r) * (1 + .15 * math.sin(t * 3 + i)), 0),
        i % 4 == 0 ? _kBlue : _kWhite,
        w: .8);
  }
  _ln(c, Offset(35, fy - r), Offset(35, fy + r), _kGreen, 1.4);
}

void _w13(Canvas c, Size s, _St st, double t) {
  final tw = .5 + st.a * 2.5, sv = st.b;
  _ln(c, const Offset(0, 106), const Offset(156, 106), _kWhite);
  for (var i = 0; i < 6; i++) {
    final x = 60 + _hash(i, 5) * 40 + math.sin(t * 3 + i) * 6;
    _dash(c, Offset(x, 104), Offset(x + 10, 101), _kGrey, w: .7);
  }
  Offset cen(int i) => Offset(78 + math.sin(i * tw * .5 - t * 3) * sv * 14 * (1 - i / 15), 12 + i * 6.4);
  for (var i = 0; i < 15; i++) {
    final rx = 3 + math.pow((14 - i) / 14, 1.6) * 40 * (.5 + sv * .5);
    _ov(c, cen(i), rx.toDouble(), rx * .2 + 1, i.isEven ? _kWhite : _al(_kWhite, .45), .8);
  }
  final hp = cen(6) + Offset(math.cos(t * 2.4) * 34 * (.5 + sv * .5), math.sin(t * 2.4) * 6);
  c.save();
  c.translate(hp.dx, hp.dy);
  c.rotate(t * 3);
  c.drawRect(const Rect.fromLTRB(-4, -3, 4, 4), _s(_kRed, .9));
  _pl(c, const [Offset(-5, -3), Offset(0, -7), Offset(5, -3)], _kRed, w: .9);
  c.restore();
  _kv(c, const Offset(6, 6), 'TWIST', tw * 30, _kRed, h: 9);
  _kv(c, const Offset(150, 6), 'STRENGTH', sv * 99, _kBlue, h: 9, align: 1);
}

void _w14(Canvas c, Size s, _St st, double t) {
  const o = Offset(66, 66);
  final sv = st.b * 8, r = .4 + st.a * 1.6;
  c.drawRRect(RRect.fromLTRBR(6, 30, 150, 110, const Radius.circular(4)), _s(_kGrey, .8));
  Offset g(double rad, double a) {
    final y = sv * math.sin(a * 2 + t * 3.5) * math.pow(rad / 50, r);
    return o + Offset(math.cos(a) * rad, math.sin(a) * rad * .42 + y);
  }

  for (var k = 0; k < 7; k++) {
    final rad = 14 + k * 6.0;
    _pl(c, [for (var a = 0.0; a <= 2 * math.pi + .01; a += .12) g(rad, a)], k == 6 ? _kWhite : _al(_kWhite, .5), w: .8);
  }
  _pl(c, [for (var a = 0.0; a <= 2 * math.pi + .01; a += .3) g(9, a)], _kRed, w: .9);
  _dot(c, o, 1, _kWhite);
  const piv = Offset(132, 40);
  _o(c, piv, 4, _kWhite);
  final sty = g(44, .6);
  _pl(c, [piv, piv + const Offset(-6, 20), sty], _kWhite);
  _dot(c, sty, 1.4, _kGreen);
  _kv(c, const Offset(6, 6), 'WARP', sv * 12, _kBlue, h: 9);
  _kv(c, const Offset(150, 6), 'SPREAD', r * 40, _kGreen, h: 9, align: 1);
}

void _w15(Canvas c, Size s, _St st, double t) {
  const o = Offset(78, 62), word = 'WARP DRAIN WARP DRAIN WARP DRAIN WARP DRAIN ';
  final tw = 2.2 + st.spin * .8;
  final k = (2.2 / tw.abs().clamp(.6, 9)).clamp(.15, 3.0);
  var a = st.spin * .6 + t * .2, r = 56.0;
  for (var i = 0; i < word.length && r > 6; i++) {
    final ch = word[i], h = (r / 5).clamp(3.0, 11.0);
    final p = o + _dir(a) * r;
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(a + math.pi / 2);
    _t(c, ch, Offset(-h / 3, -h / 2), h, i % 5 == 0 ? _kRed : _kWhite, w: .9);
    c.restore();
    final adv = h / 6 * _adv(ch) + .8;
    a += adv / r;
    r -= k * adv / 4;
  }
  _dot(c, o, 1.6, _kRed);
  _kv(c, const Offset(6, 6), 'TWIST', tw.abs() * 10, _kRed, h: 9);
}

void _w16(Canvas c, Size s, _St st, double t) {
  final turns = (1.2 + st.spin * .5).clamp(0.0, 7.0);
  _ov(c, const Offset(78, 96), 60, 16, _kWhite);
  _ov(c, const Offset(78, 96), 44, 11, _kDim, .8);
  _ln(c, const Offset(78, 4), const Offset(78, 44), _kWhite, 1.4);
  _pl(c, const [Offset(78, 44), Offset(72, 50), Offset(84, 50), Offset(78, 44)], _kWhite);
  for (var i = 0; i < 4; i++) {
    _ln(c, Offset(72 + i * 4, 50), Offset(72 + i * 4, 70), _kWhite, .9);
  }
  for (var k = 0; k < 6; k++) {
    final ph = k * 1.05 + st.spin;
    final helix = <Offset>[];
    final n = (turns * 20).round();
    for (var i = 0; i <= n; i++) {
      final u = i / 20 * 2 * math.pi + ph;
      helix.add(Offset(78 + math.sin(u) * 11, 70 - i * .45 - k * .8));
    }
    final end = helix.first;
    final rim = const Offset(78, 96) + Offset(math.cos(k * 1.1) * 40, math.sin(k * 1.1) * 9);
    _pl(c, [rim, Offset.lerp(rim, end, .5)! + Offset(math.sin(t + k) * 4, 4), end], _kWhite, w: .9);
    for (var i = 1; i < helix.length; i++) {
      final front = math.cos(i / 20 * 2 * math.pi + ph) > 0;
      _ln(c, helix[i - 1], helix[i], front ? _kRed : _al(_kRed, .3), .9);
    }
  }
  _kv(c, const Offset(6, 6), 'TWIRL', turns * 10, _kRed, h: 12);
  _lb(c, 'SPIN', const Offset(150, 6), _kDim, align: 1);
}

void _w17(Canvas c, Size s, _St st, double t) {
  const o = Offset(70, 62);
  final tw = (st.a - .3) * 4, r = 10 + st.b * 40;
  Offset web(double rad, double a) => o + _dir(a + tw * _g(rad, r) + math.sin(t + rad * .1) * .02) * rad;
  for (var i = 0; i < 12; i++) {
    final a = i / 12 * 2 * math.pi;
    _pl(c, [for (var rad = 2.0; rad <= 62; rad += 3) web(rad, a)], _kWhite, w: .8);
  }
  final hi = (r / 7.5).round().clamp(1, 7);
  for (var k = 1; k <= 7; k++) {
    final rad = k * 7.5;
    _pl(c, [for (var i = 0; i <= 12; i++) web(rad, i / 12 * 2 * math.pi)], k == hi ? _kGreen : _al(_kWhite, .45), w: .7);
  }
  final sp = web(4, t * .3);
  _dot(c, sp, 2.6, _kRed);
  for (var l = 0; l < 8; l++) {
    final a = l / 8 * 2 * math.pi + .2;
    _pl(c, [sp, sp + _dir(a) * 4, sp + _dir(a + .5) * 7], _kRed, w: .8);
  }
  _kv(c, const Offset(150, 6), 'TWIST', tw.abs() * 25, _kRed, h: 9, align: 1);
  _kv(c, const Offset(150, 92), 'RADIUS', r, _kGreen, h: 9, align: 1);
}

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
