// Scale / Stretch: x (blue), y (green), link (white), squash (red).
part of 'op_09.dart';

List<_Op> _scalePanels() => const [
      _Op('Bounce squash', 'diagram · effect in time · drag x size, up squash', _G.drag, _sBall, a: .4, b: .6),
      _Op('Cat stretch', 'animal · effect · pinch length, up height', _G.pinch, _sCat, a: .4, b: .5),
      _Op('Squeezebox', 'instrument · effect · rub to pump, x width', _G.rub, _sAccordion, a: .5, b: .4, wrapB: true),
      _Op('Stretched numeral', 'typographic · effect · pinch x, up y · numeral led', _G.pinch, _sNumeral, a: .45, b: .6),
      _Op('Iso crate', 'isometric object · effect · drag x width, up height', _G.drag, _sCrate, a: .4, b: .4),
      _Op('Spring mass', 'machine · mechanism · flick the mass, bounces', _G.flick, _sSpring, a: .5, b: .5),
      _Op('Balloon', 'object pushed · effect · rub to inflate, tap = new · extreme', _G.rub, _sBalloon,
          a: .5, b: .35, tapB: .3),
      _Op('Rubber band', 'material · mechanism · pinch length, up loop', _G.pinch, _sBand, a: .3, b: .4),
      _Op('Funhouse mirror', 'character · effect · drag x width, up height', _G.drag, _sMirror, a: .5, b: .5),
      _Op('Map zoom', 'landscape · effect · pinch zoom, up relief', _G.pinch, _sMap, a: .4, b: .4),
      _Op('Giraffe neck', 'animal pushed · effect · drag x body, up neck · extreme', _G.drag, _sGiraffe, a: .4, b: .35),
      _Op('Scale chain', 'diagram · mechanism · flick x, up y, tap link · diagram led', _G.flick, _sChain,
          a: .5, b: .3, toggle: true),
      _Op('Powers of ten', 'cosmic pushed · effect · spin, out = zoom out · extreme', _G.spin, _sTen, a: 0, b: .3, wrap: true),
      _Op('Scale envelope', 'M4L device · effect in time · draw the curve', _G.draw, _sEnv),
      _Op('Proportion lock', 'diagram · mechanism · drag corner, tap link · numeral led', _G.drag, _sLock,
          a: .4, b: .5, toggle: true),
      _Op('Telescope', 'machine · effect · pinch extend, up focus', _G.pinch, _sScope, a: .4, b: .4),
      _Op('Aspect house', 'sample · effect · spin aspect, out size', _G.spin, _sHouse, a: .1, b: .5, wrap: true),
      _Op('Anvil car', 'vehicle pushed · squash · flick down, up weight · extreme', _G.flick, _sCar, a: .3, b: .5),
      _Op('Tower grow', 'landscape · effect · rub to build, x width', _G.rub, _sTower, a: .35, b: .4),
      _Op('Time stretch', 'instrument · effect · drag x stretch, up gain · numeral led', _G.drag, _sWave, a: .3, b: .6),
    ];

String _pc(double v) => _d3(v * 100);

void _hDim(Canvas c, double x0, double x1, double y, Color col) {
  _ln(c, Offset(x0, y), Offset(x1, y), col, 1);
  _ln(c, Offset(x0, y - 3), Offset(x0, y + 3), col, 1);
  _ln(c, Offset(x1, y - 3), Offset(x1, y + 3), col, 1);
}

void _vDim(Canvas c, double y0, double y1, double x, Color col) {
  _ln(c, Offset(x, y0), Offset(x, y1), col, 1);
  _ln(c, Offset(x - 3, y0), Offset(x + 3, y0), col, 1);
  _ln(c, Offset(x - 3, y1), Offset(x + 3, y1), col, 1);
}

void _sBall(Canvas c, Size s, _S st, double t) {
  const gy = 104.0;
  final r = 7 + st.a * 10, sq = st.b;
  _ln(c, const Offset(4, gy), Offset(s.width - 4, gy), _dg, 1);
  for (var k = 0; k < 2; k++) {
    final x0 = 4.0 + k * 46;
    final arc = <Offset>[for (var i = 0; i <= 20; i++) Offset(x0 + i * 2.3, gy - r - 4 * (i / 20) * (1 - i / 20) * 60 * (1 - k * .3))];
    c.drawPoints(ui.PointMode.points, arc, _s(_dg, 1));
  }
  for (final (x, ph) in [(8.0, .0), (20.0, .2), (31.0, .5), (42.0, .8)]) {
    final h = 4 * ph * (1 - ph) * 60, v = (1 - 2 * ph).abs();
    final sy = ph == 0 ? 1 - sq * .55 : 1 + sq * .45 * v, sx = 1 / sy;
    final rr = 5.0;
    c.drawOval(Rect.fromCenter(center: Offset(x, gy - h - rr * sy), width: rr * 2 * sx, height: rr * 2 * sy), _s(_al(_wh, .35), 1));
  }
  final ph = _wr(t * 1.1);
  final h = 4 * ph * (1 - ph) * 64, v = (1 - 2 * ph).abs();
  final contact = ph < .07 || ph > .93;
  final sy = contact ? 1 - sq * .6 : 1 + sq * .5 * v * v, sx = 1 / sy;
  final cy = gy - h - r * sy;
  c.drawOval(Rect.fromCenter(center: Offset(100, cy), width: r * 2 * sx, height: r * 2 * sy), _s(contact ? _rd : _wh, 1.3));
  if (contact && sq > .1) {
    for (final d in [-1.0, 1.0]) {
      _ln(c, Offset(100 + d * (r * sx + 3), gy - 2), Offset(100 + d * (r * sx + 9), gy - 6), _rd, 1);
    }
  }
  _lab(c, 'squash', Offset(s.width - 6, 8), _rd, ax: 1);
  _num(c, _d2(sq * 99), Offset(s.width - 6, 18), 20, _rd, ax: 1);
  _lab(c, 'size', const Offset(6, 108), _bl);
  _num(c, _d2(r * 2), const Offset(30, 106), 11, _bl);
}

void _sCat(Canvas c, Size s, _S st, double t) {
  const gy = 100.0;
  final len = 36 + st.b * 76, hgt = 16 + st.a * 26, br = math.sin(t * 2) * 1.2;
  final x0 = 78 - len / 2 - 4, top = gy - hgt;
  _ln(c, const Offset(4, gy), Offset(s.width - 4, gy), _dg, 1);
  final back = Path()
    ..moveTo(x0, top + 2)
    ..quadraticBezierTo(x0 + len / 2, top + 6 + br - len * .04, x0 + len, top);
  c.drawPath(back, _s(_wh, 1.1));
  _ln(c, Offset(x0 + 4, top + 12), Offset(x0 + len - 4, top + 11 + br), _wh, 1);
  _pl(c, [Offset(x0, top + 2), Offset(x0 - 1, top + 10), Offset(x0 + 4, top + 12)], _wh, w: 1);
  for (final lx in [x0 + 3, x0 + 8, x0 + len - 6, x0 + len - 1]) {
    _ln(c, Offset(lx, top + 11), Offset(lx, gy), _wh, 1);
  }
  final hd = Offset(x0 + len + 8, top - 4);
  _ring(c, hd, 7, _wh, 1);
  _pl(c, [hd + const Offset(-6, -3), hd + const Offset(-5, -11), hd + const Offset(-1, -6)], _wh, w: 1);
  _pl(c, [hd + const Offset(2, -7), hd + const Offset(6, -12), hd + const Offset(6, -3)], _wh, w: 1);
  _dot(c, hd + const Offset(3, -1), 1, _gr);
  _ln(c, hd + const Offset(6, 2), hd + const Offset(14, 1), _dg, .8);
  _ln(c, hd + const Offset(6, 3), hd + const Offset(14, 5), _dg, .8);
  final sw = math.sin(t * 1.6) * 4;
  final tail = Path()
    ..moveTo(x0, top + 2)
    ..quadraticBezierTo(x0 - 14, top - 2, x0 - 8 + sw, top - 20);
  c.drawPath(tail, _s(_wh, 1));
  _hDim(c, x0, x0 + len, gy + 7, _bl);
  _vDim(c, top, gy, 6, _gr);
  _num(c, _pc(len / 56), Offset(s.width - 6, 8), 18, _bl, ax: 1);
  _num(c, _pc(hgt / 30), Offset(s.width - 6, 30), 12, _gr, ax: 1);
}

void _sAccordion(Canvas c, Size s, _S st, double t) {
  final breathe = math.sin(t * 2.4) * 3 * st.b;
  final w = 26 + st.a * 92 + breathe, xl = 78 - w / 2, xr = 78 + w / 2;
  const y0 = 34.0, y1 = 86.0;
  c.drawRect(Rect.fromLTRB(xl - 16, y0 - 4, xl, y1 + 4), _s(_wh, 1));
  final btn = <Offset>[for (var i = 0; i < 3; i++) for (var j = 0; j < 6; j++) Offset(xl - 12 + i * 4, y0 + 4 + j * 8.0)];
  c.drawPoints(ui.PointMode.points, btn, _s(_wh, 1.8));
  c.drawRect(Rect.fromLTRB(xr, y0 - 4, xr + 16, y1 + 4), _s(_wh, 1));
  for (var j = 0; j < 9; j++) {
    final y = y0 + j * 6.0;
    _ln(c, Offset(xr + 4, y), Offset(xr + 16, y), _al(_wh, .6), .8);
  }
  const n = 10;
  final top = <Offset>[], bot = <Offset>[];
  for (var i = 0; i <= n * 2; i++) {
    final x = xl + w * i / (n * 2), d = i.isEven ? 0.0 : 6 - w / 40;
    top.add(Offset(x, y0 + d));
    bot.add(Offset(x, y1 - d));
    if (i.isEven) _ln(c, Offset(x, y0), Offset(x, y1), _al(_bl, .7), .8);
  }
  _pl(c, top, _bl, w: 1.1);
  _pl(c, bot, _bl, w: 1.1);
  final amp = 2 + st.b * 8;
  _pl(c, [for (var x = 8.0; x <= 148; x += 2) Offset(x, 104 + math.sin(x * (.25 + st.a * .1) - t * 9) * amp * math.sin((x - 8) / 140 * math.pi))],
      _gr, w: 1);
  _lab(c, 'width', const Offset(6, 6), _bl);
  _num(c, _pc(w / 72), const Offset(6, 14), 13, _bl);
  _lab(c, 'air', Offset(s.width - 6, 6), _gr, ax: 1);
  _num(c, _d2(st.b * 99), Offset(s.width - 6, 14), 13, _gr, ax: 1);
}

void _sNumeral(Canvas c, Size s, _S st, double t) {
  final sx = .3 + st.b * 1.9, sy = .3 + st.a * 1.9;
  final txt = _d3(sx * 100);
  final tp = _tp(txt, 36, _wh, FontWeight.w200, -.6);
  final r0 = Rect.fromCenter(center: _c0, width: tp.width, height: tp.height);
  final pts = <Offset>[r0.topLeft, r0.topRight, r0.bottomRight, r0.bottomLeft, r0.topLeft];
  for (var i = 0; i < 4; i++) {
    _dots(c, pts[i], pts[i + 1], _dg, gap: 3);
  }
  c.save();
  c.translate(_c0.dx, _c0.dy);
  c.scale(sx, sy);
  tp.paint(c, Offset(-tp.width / 2, -tp.height / 2));
  c.restore();
  final r1 = Rect.fromCenter(center: _c0, width: tp.width * sx, height: tp.height * sy);
  for (final p in [r1.topLeft, r1.topRight, r1.bottomLeft, r1.bottomRight]) {
    _dot(c, p, 1.6, _rd);
  }
  _hDim(c, r1.left, r1.right, math.min(r1.bottom + 6, 114), _bl);
  _vDim(c, r1.top, r1.bottom, math.min(r1.right + 6, 150), _gr);
  _lab(c, 'y ${_pc(sy)}', const Offset(6, 6), _gr);
}

void _sCrate(Canvas c, Size s, _S st, double t) {
  const o = Offset(70, 52), k = 10.0;
  for (var i = 0; i <= 6; i++) {
    _ln(c, _iso(o, i.toDouble(), 0, 0, k), _iso(o, i.toDouble(), 6, 0, k), _dg, .7);
    _ln(c, _iso(o, 0, i.toDouble(), 0, k), _iso(o, 6, i.toDouble(), 0, k), _dg, .7);
  }
  final w = 1 + st.a * 4, h = .6 + st.b * 4.6 + math.sin(t * 3) * .04;
  _isoBox(c, o, 1, 1, 0, 1.5, 1.5, 1.5, k, _al(_wh, .3), .8);
  _isoBox(c, o, 1, 1, 0, w, 1.5, h, k, _wh, 1.1);
  _ln(c, _iso(o, 1, 1, h, k), _iso(o, 1 + w, 2.5, h, k), _al(_wh, .4), .7);
  _ln(c, _iso(o, 1, 2.5, h, k), _iso(o, 1 + w, 1, h, k), _al(_wh, .4), .7);
  _ln(c, _iso(o, 1, 1, -.4, k), _iso(o, 1 + w, 1, -.4, k), _bl, 1.2);
  _ln(c, _iso(o, 1, 1, 0, k) - const Offset(6, 0), _iso(o, 1, 1, h, k) - const Offset(6, 0), _gr, 1.2);
  final fp = _iso(o, 5, 1.2, 0, k);
  _ring(c, fp + const Offset(0, -15), 2.2, _wh, .9);
  _ln(c, fp + const Offset(0, -13), fp + const Offset(0, -6), _wh, .9);
  _ln(c, fp + const Offset(0, -6), fp + const Offset(-2, 0), _wh, .9);
  _ln(c, fp + const Offset(0, -6), fp + const Offset(2, 0), _wh, .9);
  _num(c, _pc(w / 1.5), const Offset(6, 98), 14, _bl);
  _num(c, _pc(h / 1.5), const Offset(6, 6), 14, _gr);
}

void _sSpring(Canvas c, Size s, _S st, double t) {
  const y = 56.0, rest = .5;
  final mx = 40 + st.a * 86 + math.sin(t * 5) * .8;
  _ln(c, const Offset(10, 26), const Offset(10, 86), _wh, 1);
  for (var i = 0; i < 8; i++) {
    _ln(c, Offset(10, 30.0 + i * 7), Offset(4, 36.0 + i * 7), _dg, 1);
  }
  final squash = st.a < rest - .05;
  _ln(c, const Offset(10, y), const Offset(16, y), _wh, 1);
  _coil(c, const Offset(16, y), Offset(mx - 4, y), 9, 9, squash ? _rd : _bl, 1);
  _ln(c, Offset(mx - 4, y), Offset(mx, y), _wh, 1);
  c.drawRect(Rect.fromLTWH(mx, y - 11, 20, 22), _s(_wh, 1.1));
  _dot(c, Offset(mx + 10, y), 1.5, _rd);
  _ln(c, const Offset(8, 90), const Offset(148, 90), _dg, 1);
  for (var x = 10.0; x <= 148; x += 6) {
    _ln(c, Offset(x, 90), Offset(x, (x - 10) % 30 == 0 ? 95 : 92), _dg, 1);
  }
  final rx = 40 + rest * 86;
  _dots(c, Offset(rx, 22), Offset(rx, 90), _al(_wh, .4));
  _lab(c, 'turns', const Offset(20, 10), _wh);
  _num(c, '09', const Offset(52, 6), 12, _wh);
  final pct = (mx - 40) / (rest * 86) * 100;
  _lab(c, squash ? 'squash' : 'stretch', Offset(s.width - 6, 98), squash ? _rd : _bl, ax: 1);
  _num(c, _d3(pct), Offset(s.width - 6, 106), 11, squash ? _rd : _bl, ax: 1);
  _lab(c, 'damp', const Offset(6, 106), _dg);
}

void _sBalloon(Canvas c, Size s, _S st, double t) {
  const o = Offset(72, 48);
  final popped = st.b > .97;
  final size = .25 + st.b * .78, sx = (.6 + st.a * .8), sy = 1 / math.sqrt(sx);
  final rx = 30 * size * sx, ry = 34 * size * sy;
  if (popped) {
    for (var i = 0; i < 14; i++) {
      final a = i / 14 * _tau + _rn(i) * .3;
      _ln(c, _pol(o, 10 + _rn(i + 9) * 8, a), _pol(o, 28 + _rn(i + 3) * 18, a), i.isEven ? _rd : _wh, 1);
    }
    _num(c, 'POP', o + const Offset(0, -10), 22, _rd, ax: .5);
    _lab(c, 'tap = new', const Offset(6, 108), _dg);
    return;
  }
  final sway = math.sin(t * 1.3) * 2;
  final path = Path()
    ..moveTo(o.dx + sway, o.dy - ry)
    ..cubicTo(o.dx + sway + rx * 1.1, o.dy - ry, o.dx + rx * 1.05, o.dy + ry * .45, o.dx, o.dy + ry)
    ..cubicTo(o.dx - rx * 1.05, o.dy + ry * .45, o.dx + sway - rx * 1.1, o.dy - ry, o.dx + sway, o.dy - ry);
  c.drawPath(path, _s(st.b > .85 ? _rd : _wh, 1.2));
  c.drawArc(Rect.fromCenter(center: o + Offset(-rx * .35 + sway, -ry * .35), width: rx * .7, height: ry * .7), math.pi * 1.05,
      math.pi * .35, false, _s(_al(_wh, .6), 1));
  final kn = o + Offset(0, ry);
  _pl(c, [kn, kn + const Offset(-3, 4), kn + const Offset(3, 4)], _wh, w: 1, close: true);
  _pl(c, [for (var i = 0; i <= 12; i++) kn + Offset(math.sin(i * .9 + t * 2) * 2.5, 4 + i * (112 - kn.dy - 4) / 12)], _dg, w: .9);
  _lab(c, 'vol', Offset(s.width - 6, 8), _wh, ax: 1);
  _num(c, _d2(st.b * 99), Offset(s.width - 6, 18), 22, _wh, ax: 1);
  _lab(c, 'x', const Offset(6, 98), _bl);
  _num(c, _pc(sx), const Offset(14, 96), 11, _bl);
}

void _sBand(Canvas c, Size s, _S st, double t) {
  final w = 20 + st.b * 116, h = 6 + st.a * 30;
  final xl = 78 - w / 2, xr = 78 + w / 2, cy = 54.0;
  final thick = (3.2 * 36 / (w + h)).clamp(.35, 3.2);
  final amp = st.b * 2.4 * math.sin(t * 34);
  Offset sv(double x, double y0) => Offset(x, y0 + math.sin((x - xl) / w * math.pi) * amp);
  final col = thick < .6 ? _rd : _bl;
  _pl(c, [for (var x = xl; x <= xr; x += 3) sv(x, cy - h / 2)], col, w: thick);
  _pl(c, [for (var x = xl; x <= xr; x += 3) sv(x, cy + h / 2)], col, w: thick);
  c.drawArc(Rect.fromCenter(center: Offset(xl, cy), width: h, height: h), math.pi / 2, math.pi, false, _s(col, thick));
  c.drawArc(Rect.fromCenter(center: Offset(xr, cy), width: h, height: h), -math.pi / 2, math.pi, false, _s(col, thick));
  for (final x in [xl, xr]) {
    _ring(c, Offset(x, cy), 3, _wh, 1);
    _ln(c, Offset(x - 1.5, cy - 1.5), Offset(x + 1.5, cy + 1.5), _wh, .8);
  }
  if (thick < .6) {
    for (var k = 0; k < 3; k++) {
      _ln(c, Offset(78.0 - 6 + k * 6, cy - h / 2 - 6), Offset(78.0 - 8 + k * 8, cy - h / 2 - 12), _rd, 1);
    }
  }
  _hDim(c, xl, xr, 104, _bl);
  _lab(c, 'thick', const Offset(6, 6), col);
  _num(c, thick.toStringAsFixed(1), const Offset(6, 14), 16, col);
  _lab(c, 'loop', Offset(s.width - 6, 6), _gr, ax: 1);
  _num(c, _d2(h), Offset(s.width - 6, 14), 12, _gr, ax: 1);
}

const _fig = [
  [Offset(0, -10), Offset(0, 8)],
  [Offset(-9, -4), Offset(0, -7), Offset(9, -4)],
  [Offset(-6, 22), Offset(0, 8), Offset(6, 22)],
];

void _sMirror(Canvas c, Size s, _S st, double t) {
  final sx = .45 + st.a * 1.7, sy = .5 + st.b * 1.1;
  void fig(Offset at, double kx, double ky, double wav, Color col) {
    Offset m(Offset p) => at + Offset(p.dx * kx * (1 + wav * math.sin(p.dy * .18 + t * 1.6)), p.dy * ky);
    for (final l in _fig) {
      _shape(c, l, m, col, close: false);
    }
    _shape(c, [for (var i = 0; i < 16; i++) Offset(math.cos(i / 16 * _tau) * 5, -15 + math.sin(i / 16 * _tau) * 5)], m, col);
  }

  _ln(c, const Offset(4, 100), const Offset(152, 100), _dg, 1);
  fig(const Offset(20, 78), 1, 1, 0, _wh);
  final fr = RRect.fromLTRBAndCorners(56, 8, 146, 100, topLeft: const Radius.circular(45), topRight: const Radius.circular(45));
  c.drawRRect(fr, _s(_pu, 1.2));
  c.save();
  c.clipRRect(fr);
  for (var x = 62.0; x < 146; x += 9) {
    _pl(c, [for (var y = 10.0; y <= 100; y += 6) Offset(x + math.sin(y * .1 + t) * 2, y)], _al(_dg, .7), w: .7);
  }
  fig(Offset(101, 100 - 22 * sy), sx, sy, .25, _wh);
  c.restore();
  _dots(c, const Offset(32, 70), const Offset(52, 70), _dg);
  _num(c, _pc(sx), const Offset(56, 104), 11, _bl);
  _num(c, _pc(sy), const Offset(146, 104), 11, _gr, ax: 1);
}

void _sMap(Canvas c, Size s, _S st, double t) {
  final z = math.pow(2, st.b * 4 - 1).toDouble(), relief = .2 + st.a * 1.6;
  const mr = Rect.fromLTRB(6, 6, 150, 70);
  c.drawRect(mr, _s(_dg, 1));
  c.save();
  c.clipRect(mr);
  for (final (pk, sd) in [(const Offset(62, 40), 0.0), (const Offset(112, 30), 2.0)]) {
    for (var k = 1; k <= 7; k++) {
      final r = k * 6.0 * z * (sd > 0 ? .7 : 1);
      _shape(
          c,
          [for (var i = 0; i < 40; i++) Offset.fromDirection(i / 40 * _tau, r * (1 + .18 * math.sin(i / 40 * _tau * 3 + k + sd)))],
          (p) => const Offset(78, 38) + (pk - const Offset(78, 38)) * z + p,
          k == 4 ? _wh : _al(_bl, .7),
          w: k == 4 ? 1 : .8);
    }
  }
  c.restore();
  final pts = <Offset>[];
  for (var x = 6.0; x <= 150; x += 2) {
    final mx = 78 + (x - 78) / z;
    final h = math.exp(-math.pow((mx - 62) / 22, 2)) * 1 + math.exp(-math.pow((mx - 112) / 16, 2)) * .7;
    pts.add(Offset(x, 108 - h * 22 * relief * math.min(1.5, z * .6 + .4)));
  }
  _pl(c, pts, _gr, w: 1);
  _ln(c, const Offset(6, 108), const Offset(150, 108), _dg, .8);
  _hDim(c, 10, 40, 80, _bl);
  _lab(c, '1:${(25000 / z).round()}', const Offset(44, 77), _bl);
}

void _sGiraffe(Canvas c, Size s, _S st, double t) {
  const gy = 112.0;
  final bw = 22 + st.a * 36, neck = 12 + st.b * 150, bx = 40 - st.a * 14;
  _ln(c, const Offset(4, gy), const Offset(152, gy), _dg, 1);
  for (var i = 0; i < 3; i++) {
    final cx = (i * 60 + t * 4) % 200 - 20, cy = 18.0 + i * 12;
    c.drawArc(Rect.fromCenter(center: Offset(cx, cy), width: 22, height: 10), math.pi, math.pi, false, _s(_dg, 1));
    c.drawArc(Rect.fromCenter(center: Offset(cx + 10, cy - 2), width: 16, height: 10), math.pi, math.pi, false, _s(_dg, 1));
  }
  const by = 80.0;
  c.drawRRect(RRect.fromLTRBR(bx, by, bx + bw, by + 14, const Radius.circular(6)), _s(_wh, 1));
  for (final lx in [bx + 3, bx + 7, bx + bw - 7, bx + bw - 3]) {
    _ln(c, Offset(lx, by + 14), Offset(lx, gy), _wh, 1);
  }
  for (var i = 0; i < (bw / 10).floor(); i++) {
    _ring(c, Offset(bx + 6 + i * 9, by + 6 + (i.isEven ? 0 : 3)), 2, _al(_wh, .6), .8);
  }
  final sway = math.sin(t * .9) * neck * .03;
  final n0 = Offset(bx + bw - 4, by + 2), n1 = Offset(bx + bw + neck * .22 + sway, by - neck);
  _ln(c, n0, n1, _gr, 1.1);
  _ln(c, n0 + const Offset(8, 2), n1 + const Offset(7, 4), _gr, 1.1);
  final hd = n1 + const Offset(6, 0);
  c.drawOval(Rect.fromCenter(center: hd + const Offset(4, 2), width: 16, height: 7), _s(_wh, 1));
  _ln(c, hd + const Offset(-1, -2), hd + const Offset(-2, -8), _wh, 1);
  _dot(c, hd + const Offset(-2, -8), 1.3, _wh);
  _ln(c, hd + const Offset(3, -2), hd + const Offset(3, -8), _wh, 1);
  _dot(c, hd + const Offset(3, -8), 1.3, _wh);
  _dot(c, hd + const Offset(5, 1), 1, _wh);
  for (var y = gy; y > 4; y -= 10) {
    _ln(c, Offset(146, y), Offset(152, y), y > n1.dy ? _gr : _dg, 1);
  }
  _lab(c, 'neck', const Offset(142, 6), _gr, ax: 1);
  _num(c, '${(1.5 + neck * .04).toStringAsFixed(1)}m', const Offset(142, 14), 12, _gr, ax: 1);
  _num(c, _pc(bw / 40), const Offset(6, 98), 11, _bl);
  if (n1.dy < 0) _arrow(c, Offset(math.min(n1.dx, 136), 3), -math.pi / 2, _rd, 4);
}

void _sChain(Canvas c, Size s, _S st, double t) {
  final sx = .3 + st.a * 1.7, sy = st.on ? sx : .3 + st.b * 1.7;
  const y = 40.0;
  c.drawRect(const Rect.fromLTWH(4, y - 10, 20, 20), _s(_wh, 1));
  c.drawRect(const Rect.fromLTWH(10, y - 4, 8, 8), _s(_wh, 1));
  _ln(c, const Offset(24, y), const Offset(28, y), _dg, 1);
  _pl(c, [const Offset(28, y - 13), const Offset(52, y), const Offset(28, y + 13)], _bl, w: 1.2, close: true);
  _num(c, _d3(sx * 100), const Offset(29, y - 5), 9, _bl);
  _ln(c, const Offset(52, y), const Offset(56, y), _dg, 1);
  _pl(c, [const Offset(56, y - 13), const Offset(80, y), const Offset(56, y + 13)], _gr, w: 1.2, close: true);
  _num(c, _d3(sy * 100), const Offset(57, y - 5), 9, _gr);
  _ln(c, const Offset(80, y), const Offset(84, y), _dg, 1);
  final gap = st.on ? 0.0 : 3.0;
  c.save();
  c.translate(92, y);
  c.rotate(st.on ? 0 : .3);
  c.drawRRect(RRect.fromLTRBR(-8 - gap, -3, 1 - gap, 3, const Radius.circular(3)), _s(_wh, 1.1));
  c.drawRRect(RRect.fromLTRBR(-1 + gap, -3, 8 + gap, 3, const Radius.circular(3)), _s(_wh, 1.1));
  c.restore();
  _ln(c, const Offset(100, y), const Offset(106, y), _dg, 1);
  const ob = Rect.fromLTRB(106, 4, 152, 76);
  c.drawRect(ob, _s(_dg, 1));
  final oc = ob.center;
  c.drawRect(Rect.fromCenter(center: oc, width: 20, height: 20), _s(_dg, .8));
  final pulse = 1 + math.sin(t * 3) * .03;
  c.drawRect(Rect.fromCenter(center: oc, width: (20 * sx * pulse).clamp(2, 44), height: (20 * sy * pulse).clamp(2, 70)),
      _s(_wh, 1.2));
  for (final (x, l) in [(14.0, 'src'), (40.0, 'x'), (68.0, 'y'), (92.0, st.on ? 'link' : 'free'), (129.0, 'out')]) {
    _lab(c, l, Offset(x, y + 40 > 80 ? 82 : y + 40), l == 'x' ? _bl : (l == 'y' ? _gr : _al(_wh, .7)), ax: .5, size: 7);
  }
  _lab(c, 'tap = link', const Offset(6, 108), _dg);
}

void _sTen(Canvas c, Size s, _S st, double t) {
  final e = st.b * 6 + math.sin(t * .4) * .03;
  const names = ['cell', 'hand', 'room', 'town', 'land', 'earth', 'moon', 'sun'];
  c.save();
  c.translate(_c0.dx, _c0.dy);
  c.rotate(st.a * _tau * .25);
  for (var k = 0; k < 8; k++) {
    final side = 22 * math.pow(10, k - e + .3).toDouble();
    if (side < 1.5 || side > 600) continue;
    final cur = side >= 22 && side < 220;
    final col = cur ? _wh : (side < 22 ? _bl : _dg);
    c.drawRect(Rect.fromCenter(center: Offset.zero, width: side, height: side), _s(col, cur ? 1.2 : .8));
    if (side > 14 && side < 230) _lab(c, '1e${k - 4}', Offset(-side / 2 + 2, -side / 2 + 2), col, size: 7);
  }
  c.restore();
  _dot(c, _c0, 1.6, _rd);
  c.drawRect(Rect.fromLTWH(s.width - 52, 0, 52, 34), _f(_bk));
  c.drawRect(Rect.fromLTWH(0, 98, s.width, 22), _f(_bk));
  final cur = (e - .3).floor().clamp(0, 7);
  _lab(c, names[cur.toInt()], const Offset(6, 106), _gr, size: 9);
  final ex = (e - 4).toStringAsFixed(1), ten = _tp('10', 18, _gr, FontWeight.w200, -.36);
  final xw = _tp(ex, 9, _gr, FontWeight.w200, -.18).width, x0 = s.width - 6 - xw - ten.width - 1;
  _lab(c, 'zoom', Offset(s.width - 6, 4), _gr, ax: 1);
  _num(c, '10', Offset(x0, 14), 18, _gr);
  _num(c, ex, Offset(x0 + ten.width + 1, 12), 9, _gr);
  for (var k = 0; k <= 6; k++) {
    final x = 70.0 + k * 12;
    _ln(c, Offset(x, 110), Offset(x, k == 0 || k == 6 ? 104 : 107), _dg, 1);
  }
  _ln(c, const Offset(70, 110), const Offset(142, 110), _dg, 1);
  _pl(c, [Offset(70 + st.b * 72, 103), Offset(67 + st.b * 72, 98), Offset(73 + st.b * 72, 98)], _gr, w: 1, close: true);
}

void _sEnv(Canvas c, Size s, _S st, double t) {
  const l = 10.0, r = 146.0, top = 10.0, bot = 70.0, n = 8;
  double val(int i) {
    final x = l + (r - l) * (i + .5) / n;
    if (st.pts.length < 2) return .2 + .7 * math.pow(math.sin((i + .5) / n * math.pi), 2).toDouble();
    var best = st.pts.first;
    for (final p in st.pts) {
      if ((p.dx - x).abs() < (best.dx - x).abs()) best = p;
    }
    return ((bot - best.dy) / (bot - top)).clamp(0.0, 1.0);
  }

  final dotsBg = <Offset>[for (var x = l; x <= r; x += 6) for (var y = top; y <= bot; y += 6) Offset(x, y)];
  c.drawPoints(ui.PointMode.points, dotsBg, _s(_dg, 1));
  final vs = [for (var i = 0; i < n; i++) val(i)];
  final pts = [for (var i = 0; i < n; i++) Offset(l + (r - l) * (i + .5) / n, bot - vs[i] * (bot - top))];
  _pl(c, pts, _bl, w: 1.2);
  final play = (t * 2).floor() % n;
  for (var i = 0; i < n; i++) {
    _dots(c, pts[i], Offset(pts[i].dx, bot), i == play ? _gr : _al(_bl, .5));
    _dot(c, pts[i], i == play ? 2.4 : 1.5, i == play ? _gr : _bl);
    final sz = 3 + vs[i] * 13;
    c.drawRect(Rect.fromCenter(center: Offset(pts[i].dx, 96), width: sz, height: sz), _s(i == play ? _gr : _wh, 1));
  }
  _ln(c, const Offset(l, bot), const Offset(r, bot), _dg, 1);
  if (st.pts.length > 1) _pl(c, st.pts, _al(_rd, .4), w: .8);
  _num(c, _pc(.3 + vs[play] * 1.7), const Offset(150, 106), 10, _gr, ax: 1);
}

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
