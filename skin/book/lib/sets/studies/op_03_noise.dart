part of 'op_03.dart';

// Noise: blue = frequency, green = amplitude, red = evolution.

const _noisePanels = <_P>[
  _P('Static TV', 'machine · effect · drag · drawing', _n01, init: Offset(.35, .4)),
  _P('Night drive', 'vehicle · effect · drag · drawing', _n02, init: Offset(.4, .35)),
  _P('Rub the sheep', 'animal · effect · rub · drawing', _n03, init: Offset(.4, .45)),
  _P('Shaky 45', 'type · effect · drag · numeral', _n04, init: Offset(.3, .55)),
  _P('Iso terrain', 'isometric · effect · drag · drawing', _n05, init: Offset(.4, .4)),
  _P('Seismograph', 'machine · effect · flick · drawing', _n06, init: Offset(.4, .5)),
  _P('Signal chain', 'diagram · mech · drag · chart', _n07, init: Offset(.5, .45)),
  _P('Startled cat', 'animal · effect · pinch · drawing', _n08),
  _P('1/f knee', 'diagram · mech · drag · chart', _n09, init: Offset(.4, .3)),
  _P('Draw a coast', 'landscape · effect · draw · drawing', _n10),
  _P('Wobble rings', 'diagram · effect · spin · chart', _n11, init: Offset(.75, .3)),
  _P('Tremor pen', 'character · effect · drag · drawing', _n12, init: Offset(.3, .55)),
  _P('Nebula contours', 'cosmic · effect · drag · drawing', _n13, init: Offset(.4, .5)),
  _P('Jitter type', 'type · effect · drag · type-led', _n14, init: Offset(.4, .55)),
  _P('Dot field device', 'M4L · effect · drag · chart', _n15, init: Offset(.45, .4)),
  _P('Noise drummer', 'character · time · drag · drawing', _n16, init: Offset(.5, .35)),
  _P('Swarm', 'animal · motion · drag · drawing', _n17, init: Offset(.4, .45)),
  _P('Lighthouse swell', 'landscape · effect · drag · drawing', _n18, init: Offset(.45, .5)),
  _P('Wind-up scope', 'machine · effect · spin · drawing', _n19),
  _P('Ridges x20', 'land · effect · drag · EXTREME', _n20, init: Offset(.4, .25)),
];

void _n01(Canvas c, Size s, _St st, double t) {
  final f = 2 + st.a * 26, a = st.b * 7, e = t * .9 + st.rub * 2;
  final scr = RRect.fromLTRBR(34, 38, 92, 92, const Radius.circular(6));
  c.drawRRect(RRect.fromLTRBR(26, 30, 112, 100, const Radius.circular(8)), _s(_kWhite));
  c.drawRRect(scr, _s(_kGrey, .8));
  final wob = _vn(e * .7, 3) * 4 * st.b;
  _ln(c, const Offset(69, 30), Offset(52 + wob, 8), _kWhite);
  _ln(c, const Offset(69, 30), Offset(88 - wob, 10), _kWhite);
  _dot(c, Offset(52 + wob, 8), 1.4, _kWhite);
  _dot(c, Offset(88 - wob, 10), 1.4, _kWhite);
  _ln(c, const Offset(36, 100), const Offset(32, 107), _kWhite);
  _ln(c, const Offset(102, 100), const Offset(106, 107), _kWhite);
  for (var i = 0; i < 5; i++) {
    _ln(c, Offset(98, 46 + i * 4.0), Offset(106, 46 + i * 4.0), _kDim);
  }
  c.save();
  c.clipRRect(scr);
  for (var r = 0; r < 12; r++) {
    final y = 42 + r * 4.2;
    _pl(c, [for (var x = 34.0; x <= 92; x += 1.5) Offset(x, y + a * _fbm(x / 58 * f, r * .7 + e))], _kGreen, w: .9);
  }
  c.restore();
  _kv(c, const Offset(118, 40), 'FREQ', f, _kBlue, h: 12);
  _kv(c, const Offset(118, 70), 'AMP', st.b * 99, _kGreen, h: 12);
}

void _n02(Canvas c, Size s, _St st, double t) {
  final f = 1.5 + st.a * 10, a = 4 + st.b * 34, e = t * .25;
  const hz = 62.0;
  _pl(c, const [Offset(2, 20), Offset(40, 9), Offset(116, 9), Offset(154, 20)], _kDim);
  final m = [for (var x = 0.0; x <= 156; x += 2) Offset(x, hz - a * _sat(.5 + .55 * _fbm(x / 156 * f + e, 1.3)))];
  _pl(c, m, _kGreen);
  for (var x = (e * 156 / f) % (156 / f); x < 156; x += 156 / f) {
    _dot(c, Offset(156 - x, hz + 2), .9, _kBlue);
  }
  _ln(c, const Offset(0, hz), const Offset(156, hz), _kDim, .8);
  _ln(c, const Offset(74, hz), const Offset(16, 104), _kWhite);
  _ln(c, const Offset(82, hz), const Offset(140, 104), _kWhite);
  for (var i = 0; i < 6; i++) {
    final u = ((i + t * 1.4) % 6) / 6, u2 = math.min(1.0, u + .05);
    _ln(c, Offset(78, hz + (104 - hz) * u * u), Offset(78, hz + (104 - hz) * u2 * u2), _kRed, .6 + u * 1.4);
  }
  _pl(c, const [Offset(0, 106), Offset(40, 100), Offset(116, 100), Offset(156, 106)], _kWhite);
  c.drawArc(Rect.fromCircle(center: const Offset(78, 132), radius: 26), math.pi, math.pi, false, _s(_kWhite));
  _kv(c, const Offset(6, 24), 'FREQ', f, _kBlue, h: 10);
  _kv(c, const Offset(150, 24), 'AMP', a, _kGreen, h: 10, align: 1);
}

void _n03(Canvas c, Size s, _St st, double t) {
  final f = (4 + st.a * 14).roundToDouble(), a = 1 + st.b * 8, e = st.rub * 1.5 + t * .1;
  const o = Offset(64, 62);
  final fl = _loop(o, (th) => 24 + a * (math.sin(th * f / 2 + _vn(th * 1.3, e) * 2).abs()), n: 120, sy: .7);
  _pl(c, fl, _kGreen, close: true);
  _ov(c, const Offset(100, 52), 6.5, 9, _kWhite);
  _ln(c, const Offset(95, 46), const Offset(88, 43), _kWhite);
  _dot(c, const Offset(103, 51), 1.1, _kWhite);
  for (final x in const [48.0, 56.0, 74.0, 82.0]) {
    _ln(c, Offset(x, 80), Offset(x, 98), _kWhite);
  }
  _dash(c, const Offset(8, 98), const Offset(112, 98), _kDim);
  for (var i = 0; i < f; i++) {
    final th = i / f * 2 * math.pi;
    _dot(c, o + Offset(math.cos(th), math.sin(th) * .7) * 20, .8, _kBlue);
  }
  if (st.down) _pl(c, [for (final p in st.trail) Offset(p.dx * s.width, p.dy * s.height)], _al(_kRed, .6), w: .8);
  _kv(c, const Offset(118, 70), 'CURL', f, _kBlue, h: 10);
  _kv(c, const Offset(118, 94), 'FLUFF', st.b * 99, _kGreen, h: 10);
}

void _n04(Canvas c, Size s, _St st, double t) {
  final f = 1 + st.a * 8, a = st.b * 14, e = t * .6;
  final v = (st.b * 99).round().toString().padLeft(2, '0');
  _t(c, v, const Offset(78, 30), 66, _kDim, align: .5);
  _t(c, v, const Offset(78, 30), 66, _kGreen, align: .5, w: 1.2,
      warp: (q) => q + Offset(_vn(q.dx * f / 40, q.dy * f / 40 + e), _vn(q.dx * f / 40 + 9, q.dy * f / 40 - e)) * a);
  _lb(c, 'FREQ', const Offset(6, 8), _kBlue);
  _t(c, f.toStringAsFixed(1), const Offset(6, 15), 7, _kBlue);
  _lb(c, 'AMP', const Offset(150, 8), _kGreen, align: 1);
}

void _n05(Canvas c, Size s, _St st, double t) {
  final f = .15 + st.a * .6, a = st.b * 3.4, e = t * .35;
  const o = Offset(78, 40), k = 8.4;
  double z(int i, int j) => a * (.5 + .5 * _fbm(i * f, j * f + e));
  for (var i = 0; i <= 8; i++) {
    _pl(c, [for (var j = 0; j <= 8; j++) _iso(o, k, i - 4.0, j - 4.0, z(i, j))], _al(_kWhite, .55), w: .8);
    _pl(c, [for (var j = 0; j <= 8; j++) _iso(o, k, j - 4.0, i - 4.0, z(j, i))], _kBlue, w: .8);
  }
  var bi = 0, bj = 0;
  for (var i = 0; i <= 8; i++) {
    for (var j = 0; j <= 8; j++) {
      if (z(i, j) > z(bi, bj)) {
        bi = i;
        bj = j;
      }
    }
  }
  _dash(c, _iso(o, k, bi - 4.0, bj - 4.0, 0), _iso(o, k, bi - 4.0, bj - 4.0, z(bi, bj)), _kGreen);
  _dot(c, _iso(o, k, bi - 4.0, bj - 4.0, z(bi, bj)), 1.8, _kGreen);
  _lb(c, 'X', const Offset(10, 64), _kGrey);
  _lb(c, 'Y', const Offset(144, 64), _kGrey);
  _kv(c, const Offset(6, 8), 'FREQ', f * 100, _kBlue, h: 10);
  _kv(c, const Offset(150, 8), 'AMP', st.b * 99, _kGreen, h: 10, align: 1);
}

void _n06(Canvas c, Size s, _St st, double t) {
  final fl = st.flow(t), pos = t * .4 + fl.dx * 4 - fl.dy * 2;
  final a = 5 + st.b * 16, f = 1 + st.a * 4;
  _ov(c, const Offset(132, 32), 10, 3, _kWhite);
  _ov(c, const Offset(132, 92), 10, 3, _kWhite);
  _ln(c, const Offset(122, 32), const Offset(122, 92), _kWhite);
  _ln(c, const Offset(142, 32), const Offset(142, 92), _kWhite);
  _ln(c, const Offset(122, 38), const Offset(6, 38), _kGrey, .8);
  _ln(c, const Offset(122, 88), const Offset(6, 88), _kGrey, .8);
  for (var x = 6.0 + (pos * 30) % 12; x < 122; x += 12) {
    _ln(c, Offset(122 - x + 6, 40), Offset(122 - x + 6, 86), _kDim, .6);
  }
  double y(double x) => 63 + a * _fbm((122 - x) / 40 * f - pos * 2, 2);
  _pl(c, [for (var x = 6.0; x <= 120; x += 1.5) Offset(x, y(x))], _kGreen);
  final pen = Offset(120, y(120));
  _o(c, const Offset(108, 12), 3, _kWhite);
  _ln(c, const Offset(108, 12), pen, _kWhite);
  _dot(c, pen, 1.6, _kRed);
  _kv(c, const Offset(6, 8), 'EVO', (pos * 10) % 100, _kRed, h: 10);
  _lb(c, 'FLICK', const Offset(6, 100), _kDim);
}

void _n07(Canvas c, Size s, _St st, double t) {
  final oct = 1 + (st.a * 4.99).floor(), a = st.b, e = t * .5;
  c.drawRect(const Rect.fromLTRB(6, 36, 28, 58), _s(_kWhite));
  _pl(c, [for (var x = 8.0; x <= 26; x += 1) Offset(x, 47 + 5 * _fbm(x / 5, e))], _kWhite, w: .8);
  _ln(c, const Offset(28, 47), const Offset(38, 47), _kGrey, .8);
  _pl(c, const [Offset(38, 34), Offset(38, 60), Offset(62, 47)], _kGreen, close: true);
  _t(c, (a * 99).round().toString().padLeft(2, '0'), const Offset(40.5, 43.5), 7, _kGreen);
  _ln(c, const Offset(62, 47), const Offset(70, 47), _kGrey, .8);
  for (var i = oct - 1; i >= 0; i--) {
    c.drawRect(Rect.fromLTWH(70 + i * 2.5, 36 - i * 2.5, 22, 22), Paint()..color = _kBg);
    c.drawRect(Rect.fromLTWH(70 + i * 2.5, 36 - i * 2.5, 22, 22), _s(_kBlue));
  }
  _t(c, '$oct', const Offset(78, 43), 8, _kBlue);
  _ln(c, Offset(92 + (oct - 1) * 2.5, 47), const Offset(110, 47), _kGrey, .8);
  _o(c, const Offset(122, 47), 12, _kWhite);
  for (var i = 0; i < 7; i++) {
    _dot(c, const Offset(122, 47) + _dir(i / 7 * 2 * math.pi) * 7.5, 1.2, _kDim);
  }
  _dot(c, const Offset(122, 47) + _dir(e * 1.5) * 7.5, 1.8, _kRed);
  _lb(c, 'SOURCE', const Offset(17, 66), _kWhite, align: .5);
  _lb(c, 'AMOUNT', const Offset(48, 66), _kGreen, align: .5);
  _lb(c, 'DETAIL', const Offset(84, 66), _kBlue, align: .5);
  _lb(c, 'EVOLVE', const Offset(122, 66), _kRed, align: .5);
  _pl(c, [for (var x = 6.0; x <= 150; x += 1.5) Offset(x, 96 + a * 14 * _fbm(x / 30 - e, 4, oct))], _kGreen);
}

Offset _qb(Offset a, Offset b, Offset c, double u) => a * ((1 - u) * (1 - u)) + b * (2 * u * (1 - u)) + c * (u * u);

void _n08(Canvas c, Size s, _St st, double t) {
  final a = _sat(st.pinch(s) / 55), f = 6 + st.a * 18;
  final breath = math.sin(t * 2) * .8;
  const p0 = Offset(40, 82), p2 = Offset(100, 72);
  final p1 = Offset(70, 54 - a * 18 + breath);
  _pl(c, [for (var u = 0.0; u <= 1.001; u += .05) _qb(p0, p1, p2, u)], _kWhite);
  final n = f.round();
  for (var i = 1; i < n; i++) {
    final u = i / n, q = _qb(p0, p1, p2, u), q2 = _qb(p0, p1, p2, u + .01);
    final nn = Offset((q2 - q).dy, -(q2 - q).dx) / (q2 - q).distance;
    final len = 1.5 + a * 13 * (.55 + .45 * _vn(u * f, t * 3));
    _ln(c, q, q - nn * len, _kGreen, .9);
  }
  _pl(c, [for (var u = 0.0; u <= 1.001; u += .1) _qb(p0, const Offset(70, 94), const Offset(100, 82), u)], _kWhite);
  for (final l in const [
    [46.0, 86.0, 44.0, 100.0],
    [54.0, 89.0, 54.0, 100.0],
    [90.0, 86.0, 90.0, 100.0],
    [97.0, 82.0, 99.0, 100.0]
  ]) {
    _ln(c, Offset(l[0], l[1] + a * 2), Offset(l[2], l[3]), _kWhite);
  }
  _o(c, const Offset(108, 66), 9, _kWhite);
  _pl(c, const [Offset(101, 60), Offset(102, 50), Offset(108, 57)], _kWhite);
  _pl(c, const [Offset(110, 57), Offset(116, 50), Offset(115, 61)], _kWhite);
  _o(c, const Offset(111, 65), .9 + a * 1.8, _kWhite, .9);
  for (final d in const [-3.0, 1.0, 5.0]) {
    _ln(c, const Offset(116, 70), Offset(130, 68 + d * (1 + a)), _kGrey, .7);
  }
  _pl(c, [for (var u = 0.0; u <= 1.001; u += .1) _qb(p0, Offset(18 - a * 6, 66), Offset(28, 46 - a * 8), u)], _kWhite);
  _dash(c, const Offset(6, 100), const Offset(150, 100), _kDim);
  _kv(c, const Offset(6, 8), 'BRISTLE', a * 99, _kGreen, h: 10);
  _lb(c, 'PINCH', const Offset(150, 8), _kDim, align: 1);
}

void _n09(Canvas c, Size s, _St st, double t) {
  final kx = 24 + st.a * 100, amp = 8 + st.b * 66;
  const x0 = 14.0, y0 = 96.0;
  _ln(c, const Offset(x0, 12), const Offset(x0, y0), _kGrey, .8);
  _ln(c, const Offset(x0, y0), const Offset(150, y0), _kGrey, .8);
  _lb(c, 'MAX', const Offset(18, 12), _kDim);
  double h(double x) => amp / (1 + math.max(0, (x - kx) / 14));
  final pts = [for (var x = x0; x <= 150; x += 2) Offset(x, y0 - h(x) - .6 * _vn(x / 4, t * 2))];
  _pl(c, pts, _kWhite);
  for (var k = 0; k < 7; k++) {
    final x = x0 + 8 + k * 20.0;
    _dash(c, Offset(x, y0), Offset(x, y0 - h(x)), _kDim, w: .8);
    _dot(c, Offset(x, y0 - h(x)), 1.1, _kWhite);
  }
  _dot(c, Offset(kx, y0 - h(kx)), 2.2, _kBlue);
  _ln(c, Offset(x0, y0 - amp), Offset(kx, y0 - amp), _al(_kGreen, .5), .8);
  _dot(c, Offset(x0, y0 - amp), 2, _kGreen);
  final sw = x0 + ((t * 30) % 136);
  _ln(c, Offset(sw, y0 + 2), Offset(sw, y0 + 6), _kRed);
  _lb(c, 'FREQ', Offset(kx, y0 + 9), _kBlue, align: .5);
  _kv(c, const Offset(150, 10), 'AMP', st.b * 99, _kGreen, h: 10, align: 1);
}

const _coast0 = [
  Offset(0, .62), Offset(.12, .58), Offset(.25, .66), Offset(.38, .54), Offset(.5, .5), Offset(.62, .58), Offset(.75, .44),
  Offset(.88, .5), Offset(1, .4)
];

void _n10(Canvas c, Size s, _St st, double t) {
  final src = st.trail.length > 3 ? st.trail : _coast0;
  final pts = <Offset>[];
  var acc = 0.0;
  for (var i = 1; i < src.length; i++) {
    final a = Offset(src[i - 1].dx * s.width, src[i - 1].dy * s.height), b = Offset(src[i].dx * s.width, src[i].dy * s.height);
    final d = b - a, len = d.distance;
    if (len < .01) continue;
    final nrm = Offset(-d.dy, d.dx) / len;
    for (var k = 0.0; k < len; k += 1.5) {
      pts.add(a + d * (k / len) + nrm * (6 * _fbm((acc + k) / 9, t * .15)));
    }
    acc += len;
  }
  _pl(c, pts, _kGreen);
  for (var i = 0; i < pts.length; i += 7) {
    _dot(c, pts[i] + const Offset(0, -7), .6, _kDim);
  }
  for (final w in const [Offset(20, 96), Offset(64, 102), Offset(108, 92), Offset(130, 108)]) {
    final dx = math.sin(t + w.dx) * 2;
    _pl(c, [for (var x = 0.0; x <= 10; x += 1) w + Offset(x + dx, math.sin(x * .9) * 1.4)], _kBlue, w: .8);
  }
  final b = Offset(118 + math.sin(t * .5) * 6, 88);
  _pl(c, [b + const Offset(-7, 0), b + const Offset(7, 0), b + const Offset(4, 3), b + const Offset(-4, 3)], _kWhite, close: true);
  _pl(c, [b, b + const Offset(0, -11), b + const Offset(6, -2)], _kWhite);
  _ln(c, const Offset(14, 22), const Offset(14, 10), _kRed);
  _pl(c, const [Offset(11, 13), Offset(14, 10), Offset(17, 13)], _kRed);
  _lb(c, 'N', const Offset(12.4, 24), _kRed);
  _lb(c, 'DRAW A COAST', const Offset(150, 8), _kDim, align: 1);
}

void _n11(Canvas c, Size s, _St st, double t) {
  const o = Offset(62, 60);
  final e = st.spin * .5 + t * .08, f = 1.4;
  final a = 1 + 9 * _sat((st.at(s) - const Offset(78, 60)).distance / 60);
  c.drawRect(const Rect.fromLTRB(16, 14, 108, 106), _s(_kDim, .8));
  for (var i = 0; i < 6; i++) {
    final r = 7.0 + i * 7;
    _pl(c, _loop(o, (th) => r + a * _vn(math.cos(th) * f + i * .45 + e, math.sin(th) * f - e), n: 72), i.isEven ? _kWhite : _kPurple,
        close: true, w: .9);
  }
  _ln(c, const Offset(16, 60), const Offset(62, 60), _kRed);
  _dot(c, o, 1.6, _kRed);
  _lb(c, 'INPUT', const Offset(4, 52), _kRed);
  _kv(c, const Offset(114, 18), 'AMP', a * 10, _kGreen, h: 12);
  _kv(c, const Offset(114, 52), 'EVO', (e * 30) % 100, _kRed, h: 12);
  _lb(c, 'SPIN', const Offset(114, 92), _kDim);
}

void _n12(Canvas c, Size s, _St st, double t) {
  final a = st.b * 18, f = 1 + st.a * 6;
  final u = (t * .22) % 1, px = 8 + u * 104;
  double y(double x) => 78 + a * _fbm(x * f / 40, 3.3);
  _dash(c, const Offset(6, 92), const Offset(150, 92), _kDim);
  _pl(c, [for (var x = 8.0; x <= px; x += 1) Offset(x, y(x))], _kGreen);
  final tip = Offset(px, y(px));
  final hand = tip + const Offset(17, -24);
  _ln(c, tip, tip + const Offset(14, -20), _kWhite);
  c.save();
  c.translate(hand.dx, hand.dy);
  c.rotate(-.6);
  c.drawOval(Rect.fromCenter(center: Offset.zero, width: 20, height: 12), _s(_kWhite));
  c.restore();
  _ln(c, hand + const Offset(4, -6), hand + const Offset(26, -46), _kWhite);
  _ln(c, hand + const Offset(10, 0), hand + const Offset(34, -40), _kWhite);
  if (a > 5) {
    final j = math.sin(t * 40) * 1.5;
    c.drawArc(Rect.fromCircle(center: hand + Offset(j, 0), radius: 14), -2.4, .7, false, _s(_kGreen, .8));
    c.drawArc(Rect.fromCircle(center: hand + Offset(-j, 0), radius: 18), -2.5, .6, false, _s(_kGreen, .8));
  }
  _kv(c, const Offset(6, 8), 'TREMOR', st.b * 99, _kGreen, h: 10);
  _kv(c, const Offset(6, 34), 'FREQ', f * 10, _kBlue, h: 8);
}

void _n13(Canvas c, Size s, _St st, double t) {
  final f = .5 + st.a * 2.5, a = st.b, e = t * .12;
  for (var i = 0; i < 26; i++) {
    final p = Offset((_hash(i, 1) * .5 + .5) * 156, (_hash(i, 2) * .5 + .5) * 120);
    _dot(c, p, .5 + .5 * (math.sin(t * 2 + i) * .5 + .5), _kGrey);
  }
  for (final (o, n, col) in [(const Offset(58, 62), 6, _kPurple), (const Offset(104, 46), 4, _kBlue)]) {
    for (var k = 1; k <= n; k++) {
      final r = k * 6.5;
      _pl(c, _loop(o, (th) => r * (1 + a * .4 * _vn(math.cos(th) * f + k * .25 + e, math.sin(th) * f + e))), k == 1 ? _kGreen : col,
          close: true, w: .8);
    }
  }
  _o(c, const Offset(128, 98), 5, _kWhite);
  c.save();
  c.translate(128, 98);
  c.rotate(-.3);
  c.drawOval(Rect.fromCenter(center: Offset.zero, width: 24, height: 6), _s(_kWhite, .9));
  c.restore();
  _kv(c, const Offset(6, 8), 'FREQ', f * 30, _kBlue, h: 9);
  _kv(c, const Offset(6, 94), 'AMP', a * 99, _kGreen, h: 9);
}

void _n14(Canvas c, Size s, _St st, double t) {
  final f = .1 + st.a * 1.6, a = st.b * 16, e = t * .7;
  const word = 'NOISE', h = 26.0, y = 44.0;
  var x = 78 - _tw(word, h) / 2;
  _dash(c, Offset(8, y + h + 4), Offset(148, y + h + 4), _kRed);
  for (var i = 0; i < word.length; i++) {
    final o = Offset(x, y);
    x += h / 6 * _adv(word[i]);
    _t(c, word[i], o, h, _kDim, w: .8);
    final d = Offset(_vn(i * f, e), _vn(i * f + 5, e)) * a, r = _vn(i * f + 11, e) * a * .05;
    c.save();
    c.translate(o.dx + h / 3 + d.dx, o.dy + h / 2 + d.dy);
    c.rotate(r);
    _t(c, word[i], Offset(-h / 3, -h / 2), h, _kWhite, w: 1.2);
    c.restore();
  }
  _kv(c, const Offset(6, 8), 'FREQ', f * 50, _kBlue, h: 9);
  _kv(c, const Offset(150, 8), 'AMP', st.b * 99, _kGreen, h: 9, align: 1);
}

void _n15(Canvas c, Size s, _St st, double t) {
  final f = .1 + st.a * .5, a = st.b, e = t * .3;
  _ln(c, const Offset(4, 15), const Offset(152, 15), _kGrey, .8);
  _lb(c, 'NOISE~', const Offset(6, 6), _kWhite);
  _lb(c, 'ON', const Offset(150, 6), _kGreen, align: 1);
  _ln(c, const Offset(32, 15), const Offset(32, 116), _kDim, .8);
  _kv(c, const Offset(6, 22), 'FRQ', f * 100, _kBlue, h: 7);
  _kv(c, const Offset(6, 52), 'AMP', a * 99, _kGreen, h: 7);
  _kv(c, const Offset(6, 82), 'EVO', (e * 20) % 100, _kRed, h: 7);
  for (var i = 0; i < 15; i++) {
    for (var j = 0; j < 12; j++) {
      final v = .5 + .5 * _fbm(i * f, j * f + e);
      final r = .3 + 2.6 * a * v;
      _dot(c, Offset(40 + i * 7.6, 22 + j * 7.8), r, v > .72 ? _kGreen : _kWhite);
    }
  }
}

void _n16(Canvas c, Size s, _St st, double t) {
  final f = .8 + st.a * 6, a = st.b;
  _ln(c, const Offset(46, 84), const Offset(62, 84), _kWhite);
  _ln(c, const Offset(54, 84), const Offset(54, 104), _kWhite);
  _fig(c, const Offset(54, 84), 1.4, _kWhite, arm: .6);
  const snare = Offset(92, 76), cym = Offset(116, 44);
  _ov(c, snare, 13, 3.5, _kWhite);
  _ln(c, snare + const Offset(-13, 0), snare + const Offset(-13, 10), _kWhite);
  _ln(c, snare + const Offset(13, 0), snare + const Offset(13, 10), _kWhite);
  _ov(c, snare + const Offset(0, 10), 13, 3.5, _kGrey, .8);
  _ln(c, snare + const Offset(0, 14), snare + const Offset(0, 28), _kGrey, .8);
  _ov(c, cym, 14, 2.2, _kWhite);
  _ln(c, cym, cym + const Offset(0, 60), _kGrey, .8);
  _o(c, const Offset(22, 88), 14, _kWhite);
  _o(c, const Offset(22, 88), 4, _kDim);
  const sh = Offset(54, 60.6);
  for (final (tgt, seed) in [(snare, 1.0), (cym, 7.0)]) {
    final lift = a * 22 * _sat(.5 + .9 * _vn(t * f, seed));
    final tip = tgt + Offset(-4, -lift);
    final hand = Offset.lerp(sh, tip, .45)! + const Offset(0, 6);
    _pl(c, [sh, hand], _kWhite);
    _ln(c, hand, tip, _kGreen);
    _dot(c, tip, 1.4, _kGreen);
    if (lift < 2.5) {
      for (var k = -1; k <= 1; k++) {
        _ln(c, tgt + _dir(-math.pi / 2 + k * .6) * 6, tgt + _dir(-math.pi / 2 + k * .6) * 11, _kRed, .9);
      }
    }
  }
  _kv(c, const Offset(6, 8), 'BUSY', f * 14, _kBlue, h: 9);
  _kv(c, const Offset(40, 8), 'SWING', a * 99, _kGreen, h: 9);
}

void _n17(Canvas c, Size s, _St st, double t) {
  final f = .3 + st.a * 2.5, a = st.b;
  _ln(c, const Offset(34, 62), const Offset(34, 72), _kWhite);
  _ln(c, const Offset(4, 62), const Offset(60, 60), _kGrey, .8);
  final skep = Path()
    ..moveTo(16, 104)
    ..cubicTo(14, 70, 54, 70, 52, 104)
    ..close();
  c.drawPath(skep, _s(_kWhite));
  c.save();
  c.clipPath(skep);
  for (var y = 78.0; y < 104; y += 5) {
    _ln(c, Offset(10, y), Offset(60, y), _kGrey, .7);
  }
  c.restore();
  c.drawArc(Rect.fromCenter(center: const Offset(34, 104), width: 8, height: 8), math.pi, math.pi, false, _s(_kWhite));
  const o = Offset(98, 54);
  final r = 6 + a * 44;
  _o(c, o, r, _al(_kGreen, .25), .7);
  for (var i = 0; i < 16; i++) {
    Offset at(double tt) => o + Offset(_vn(tt * f + i * 3.1, i * 1.0), _vn(tt * f + i * 3.1 + 40, i * 1.0)) * r;
    final p = at(t), q = at(t - .12);
    _ln(c, q, p, _al(_kGreen, .6), .7);
    _dot(c, p, 1.3, _kWhite);
    _ln(c, p + const Offset(-1.5, -2.2), p + const Offset(1.5, -2.2), _kBlue, .7);
  }
  _kv(c, const Offset(6, 8), 'BUZZ', f * 30, _kBlue, h: 9);
  _kv(c, const Offset(150, 96), 'SPREAD', a * 99, _kGreen, h: 9, align: 1);
}

void _n18(Canvas c, Size s, _St st, double t) {
  final f = .5 + st.a * 3, a = 1 + st.b * 9, e = t * .5;
  _ln(c, const Offset(0, 54), const Offset(156, 54), _kDim, .8);
  _pl(c, const [Offset(22, 54), Offset(38, 54), Offset(35, 24), Offset(25, 24)], _kWhite, close: true);
  _ln(c, const Offset(23.5, 42), const Offset(36.6, 42), _kRed, .9);
  _ln(c, const Offset(24.5, 32), const Offset(35.6, 32), _kRed, .9);
  c.drawRect(const Rect.fromLTRB(25, 17, 35, 24), _s(_kWhite));
  _pl(c, const [Offset(24, 17), Offset(30, 11), Offset(36, 17)], _kWhite);
  final bang = math.sin(t * .8) * .25;
  _dash(c, const Offset(35, 20), const Offset(35, 20) + _dir(bang - .08) * 120, _kPurple);
  _dash(c, const Offset(35, 21), const Offset(35, 21) + _dir(bang + .08) * 120, _kPurple);
  _pl(c, const [Offset(14, 58), Offset(20, 54), Offset(42, 54), Offset(48, 59)], _kGrey, w: .8);
  double wy(int i, double x) => 62 + i * 12 + a * (.35 + i * .3) * _fbm(x * f / 40 + e * (1 + i * .2), i * 2.0);
  for (var i = 0; i < 5; i++) {
    _pl(c, [for (var x = 0.0; x <= 156; x += 2) Offset(x, wy(i, x))], i == 0 ? _kGreen : _al(_kGreen, .8 - i * .12), w: .9);
  }
  final bx = 112.0, by = wy(1, bx) - 1;
  final tilt = (wy(1, bx + 4) - wy(1, bx - 4)) / 8;
  c.save();
  c.translate(bx, by);
  c.rotate(math.atan(tilt));
  _pl(c, const [Offset(-8, -3), Offset(8, -3), Offset(5, 1), Offset(-5, 1)], _kWhite, close: true);
  _pl(c, const [Offset(0, -3), Offset(0, -15), Offset(7, -4)], _kWhite);
  c.restore();
  _kv(c, const Offset(150, 8), 'CHOP', f * 25, _kBlue, h: 9, align: 1);
  _kv(c, const Offset(116, 8), 'SWELL', st.b * 99, _kGreen, h: 9, align: 1);
}

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
