// Rotation / Spin: angle (blue), speed (green), axis (white), accent (red).
part of 'op_09.dart';

List<_Op> _spinPanels() => const [
      _Op('Turntable', 'instrument · effect · spin angle, out rpm · drawing led', _G.spin, _rTable, a: .1, b: .5, wrap: true),
      _Op('Music box dancer', 'character · effect · drag x angle, up speed', _G.drag, _rDancer, a: .15, b: .3),
      _Op('Tilted planet', 'cosmic · mechanism · drag x spin, up axis tilt', _G.drag, _rPlanet, a: .2, b: .65),
      _Op('Turning numeral', 'typographic · effect · spin angle, out drift · numeral led', _G.spin, _rNumeral,
          a: .12, b: .5, wrap: true),
      _Op('Night drive', 'vehicle · landscape · drag x steer, up speed', _G.drag, _rDrive, a: .62, b: .5),
      _Op('Axis cube', 'isometric object · mechanism · drag x angle, up axis X/Y/Z', _G.drag, _rCube, a: .1, b: .5),
      _Op('Flicked clock', 'machine · effect in time · flick the hour, blur = speed · numeral led', _G.flick, _rClock,
          a: .37, b: .5, wrap: true),
      _Op('Windmill hill', 'landscape · effect · flick blades, up wind', _G.flick, _rMill, a: .1, b: .4, wrap: true),
      _Op('Gyroscope', 'machine · mechanism · spin rotor, out axis tilt', _G.spin, _rGyro, a: .2, b: .4, wrap: true),
      _Op('Spirograph', 'diagram · effect · drag x gear ratio, up pen', _G.drag, _rSpiro, a: .45, b: .6),
      _Op('Tail chaser', 'animal · effect · spin the dog, out faster', _G.spin, _rDog, a: .1, b: .3, wrap: true),
      _Op('Spin chain', 'diagram · mechanism · flick speed, up angle · diagram led', _G.flick, _rChain, a: .34, b: .2),
      _Op('Swept letter', 'sample · effect · draw an arc round the pivot', _G.draw, _rSweep, a: .2),
      _Op('Horizon roll', 'landscape · camera effect · drag x roll, up pitch', _G.drag, _rHorizon, a: .62, b: .5),
      _Op('Galaxy wind', 'cosmic pushed · effect · spin angle, out winding · extreme', _G.spin, _rGalaxy,
          a: .0, b: .45, wrap: true),
      _Op('Spin cycle', 'machine · character · rub to spin up, x angle', _G.rub, _rWasher, a: .3, b: .25, wrap: true),
      _Op('Wobbling top', 'toy · mechanism · drag x spin, up axis tilt', _G.drag, _rTop, a: .3, b: .35),
      _Op('Hamster wheel', 'animal · machine · rub to run, x wheel', _G.rub, _rHamster, a: .2, b: .3, wrap: true),
      _Op('Strobe wheel', 'diagram pushed · real vs seen · drag x angle, up speed · extreme', _G.drag, _rStrobe,
          a: .0, b: .42),
      _Op('Vortex type', 'typographic pushed · effect · spin angle, out twist · extreme', _G.spin, _rVortex,
          a: .0, b: .4, wrap: true),
    ];

String _deg(double turns) => _d3(_wr(turns) * 360 % 360);

void _rTable(Canvas c, Size s, _S st, double t) {
  const o = Offset(56, 62);
  final rpm = 16 + (st.b * 62).round();
  final rot = st.a * _tau + t * rpm / 60 * _tau * .3;
  _ring(c, o, 46, _wh, 1);
  for (var r = 20.0; r < 44; r += 4) {
    _ring(c, o, r, _dg, .6);
  }
  c.drawArc(Rect.fromCircle(center: o, radius: 34), rot + .5, .9, false, _s(_al(_gr, .8), 1));
  c.drawArc(Rect.fromCircle(center: o, radius: 26), rot + .5 + math.pi, .6, false, _s(_al(_gr, .5), 1));
  _ring(c, o, 13, _wh, 1);
  _dot(c, o, 1.6, _wh);
  _ln(c, _pol(o, 4, rot), _pol(o, 12, rot), _bl, 1.4);
  const pv = Offset(124, 50), sty = Offset(84, 92);
  _ring(c, pv, 5, _wh, 1);
  _ln(c, pv, const Offset(96, 86), _wh, 1);
  _ln(c, const Offset(96, 86), sty, _wh, 1);
  _dot(c, sty, 1.5, _rd);
  _lab(c, 'rpm', Offset(s.width - 6, 8), _gr, ax: 1);
  _num(c, _d2(rpm), Offset(s.width - 6, 18), 22, _gr, ax: 1);
  _lab(c, 'deg', Offset(s.width - 6, 86), _bl, ax: 1);
  _num(c, _deg(rot / _tau), Offset(s.width - 6, 96), 15, _bl, ax: 1);
}

void _rDancer(Canvas c, Size s, _S st, double t) {
  final rot = st.a * _tau + t * st.b * 5;
  final co = math.cos(rot), sn = math.sin(rot);
  const cx = 74.0;
  c.drawOval(Rect.fromCenter(center: const Offset(cx, 96), width: 64, height: 12), _s(_wh, 1));
  _ln(c, const Offset(cx - 32, 96), const Offset(cx - 32, 110), _dg, 1);
  _ln(c, const Offset(cx + 32, 96), const Offset(cx + 32, 110), _dg, 1);
  c.drawArc(Rect.fromCenter(center: const Offset(cx, 110), width: 64, height: 12), 0, math.pi, false, _s(_dg, 1));
  for (var k = 0; k < 8; k++) {
    final a = rot + k * math.pi / 4;
    if (math.sin(a) > 0) _dot(c, Offset(cx + 32 * math.cos(a) * .86, 96 + 6 * math.sin(a) * .86), 1, _gr);
  }
  _ring(c, const Offset(cx, 34), 5, _wh, 1);
  _dot(c, Offset(cx - sn * 5, 29), 1.8, _wh);
  _ln(c, const Offset(cx, 39), const Offset(cx, 62), _wh, 1);
  const sh = Offset(cx, 45);
  for (final sd in [-1.0, 1.0]) {
    final el = sh + Offset(sd * co * 11, -6 + sd * sn * 2);
    final hd = sh + Offset(sd * co * 6, -18 + sd * sn * 2);
    _pl(c, [sh, el, hd], _bl, w: 1.2);
  }
  final tut = <Offset>[];
  for (var k = 0; k <= 20; k++) {
    final a = rot + k / 20 * _tau, r = k.isEven ? 22.0 : 17.0;
    tut.add(Offset(cx + r * math.cos(a), 64 + r * .2 * math.sin(a)));
  }
  _pl(c, tut, _wh, w: 1);
  _ln(c, const Offset(cx, 66), const Offset(cx, 90), _wh, 1);
  final kn = Offset(cx + co * 9, 76 + sn * 1.5);
  _pl(c, [const Offset(cx, 68), kn, const Offset(cx, 82)], _bl, w: 1.2);
  if (st.b > .15) {
    for (var k = 0; k < 2; k++) {
      c.drawArc(Rect.fromCenter(center: const Offset(cx, 58), width: 76.0 + k * 14, height: 22.0 + k * 6),
          rot + k * 1.4, -st.b * 1.6, false, _s(_al(_gr, .8 - k * .3), 1));
    }
  }
  _lab(c, 'deg', const Offset(6, 8), _bl);
  _num(c, _deg(rot / _tau), const Offset(5, 18), 18, _bl);
  _lab(c, 'speed', Offset(s.width - 6, 8), _gr, ax: 1);
  _num(c, _d2(st.b * 99), Offset(s.width - 6, 18), 18, _gr, ax: 1);
}

void _rPlanet(Canvas c, Size s, _S st, double t) {
  const o = Offset(66, 62), r = 36.0;
  final tilt = (st.b - .5) * math.pi * .7, spin = st.a * _tau + t * .5;
  final stars = <Offset>[];
  for (var i = 0; i < 30; i++) {
    final p = Offset(_rn(i) * s.width, _rn(i + 40) * s.height);
    if ((p - o).distance > r + 4) stars.add(p);
  }
  c.drawPoints(ui.PointMode.points, stars, _s(_dg, 1.2));
  _ring(c, o, r, _wh, 1);
  c.save();
  c.translate(o.dx, o.dy);
  c.rotate(tilt);
  _ln(c, const Offset(0, -r - 16), const Offset(0, -r), _wh, 1);
  _ln(c, const Offset(0, r), const Offset(0, r + 16), _wh, 1);
  _dots(c, const Offset(0, -r), const Offset(0, r), _al(_wh, .5));
  for (final lat in [-.9, -.45, 0.0, .45, .9]) {
    final hw = r * math.cos(lat);
    c.drawArc(Rect.fromCenter(center: Offset(0, r * math.sin(lat)), width: hw * 2, height: hw * .36), 0, math.pi, false,
        _s(lat == 0 ? _wh : _dg, lat == 0 ? 1 : .8));
  }
  for (var k = 0; k < 12; k++) {
    final ph = spin + k * math.pi / 6;
    if (math.cos(ph) <= 0) continue;
    final pts = <Offset>[];
    for (var j = 0; j <= 16; j++) {
      final lat = -math.pi / 2 + j / 16 * math.pi;
      pts.add(Offset(r * math.sin(ph) * math.cos(lat), r * math.sin(lat)));
    }
    _pl(c, pts, _al(_bl, .35 + .65 * math.cos(ph)), w: 1);
  }
  if (math.cos(spin + .4) > 0) _dot(c, Offset(r * math.sin(spin + .4) * math.cos(-.5), r * math.sin(-.5)), 2, _rd);
  c.restore();
  _lab(c, 'axis', Offset(s.width - 6, 8), _wh, ax: 1);
  _num(c, '${tilt < 0 ? '-' : '+'}${_d2(tilt.abs() * 180 / math.pi)}', Offset(s.width - 6, 18), 18, _wh, ax: 1);
  _lab(c, 'spin', Offset(s.width - 6, 88), _bl, ax: 1);
  _num(c, _deg(spin / _tau), Offset(s.width - 6, 98), 14, _bl, ax: 1);
}

void _rNumeral(Canvas c, Size s, _S st, double t) {
  final drift = (st.b - .5) * 2;
  final ang = st.a * _tau + t * drift;
  _dring(c, _c0, 50, _dg, n: 72);
  for (var k = 0; k < 4; k++) {
    final a = k * math.pi / 2 - math.pi / 2;
    _ln(c, _pol(_c0, 46, a), _pol(_c0, 54, a), _dg, 1);
  }
  final txt = _deg(ang / _tau);
  _dots(c, _c0 - const Offset(40, -16), _c0 + const Offset(40, 16), _dg);
  c.save();
  c.translate(_c0.dx, _c0.dy);
  c.rotate(ang);
  final tp = _tp(txt, 40, _wh, FontWeight.w200, -.8);
  tp.paint(c, Offset(-tp.width / 2, -tp.height / 2));
  _ln(c, Offset(-tp.width / 2, tp.height / 2 + 3), Offset(tp.width / 2, tp.height / 2 + 3), _bl, 1.2);
  c.restore();
  c.drawArc(Rect.fromCircle(center: _c0, radius: 50), -math.pi / 2, _wr(ang / _tau) * _tau, false, _s(_bl, 1.2));
  _dot(c, _pol(_c0, 50, ang - math.pi / 2), 2.2, _rd);
  _lab(c, 'drift', const Offset(6, 8), _gr);
  _num(c, '${drift < 0 ? '-' : '+'}${_d2(drift.abs() * 99)}', const Offset(5, 18), 13, _gr);
}

void _rDrive(Canvas c, Size s, _S st, double t) {
  final steer = (st.a - .5) * 2, sp = st.b;
  const hy = 44.0;
  final sky = <Offset>[const Offset(0, hy)];
  final shift = -steer * 34;
  for (var i = -2; i < 16; i++) {
    final x = i * 11 + shift % 11, h = 4 + _rn(i - (shift / 11).floor() + 9) * 16;
    sky
      ..add(Offset(x, hy - h))
      ..add(Offset(x + 9, hy - h));
  }
  sky.add(Offset(s.width, hy));
  _pl(c, sky, _al(_wh, .55), w: .8);
  _ln(c, const Offset(0, hy), Offset(s.width, hy), _wh, 1);
  final vp = Offset(78 + steer * 42, hy);
  Path edge(double x0, double bend) => Path()
    ..moveTo(x0, 100)
    ..quadraticBezierTo(x0 + (78 - x0) * .4 + steer * bend, 70, vp.dx, vp.dy);
  c.drawPath(edge(8, 18), _s(_bl, 1.2));
  c.drawPath(edge(148, 18), _s(_bl, 1.2));
  Offset mid(double u) {
    const a = Offset(78, 100);
    final k = Offset(78 + steer * 24, 70);
    return a * ((1 - u) * (1 - u)) + k * (2 * u * (1 - u)) + vp * (u * u);
  }

  final ph = _wr(t * (.2 + sp * 2.4));
  for (var i = 0; i < 7; i++) {
    final z = (i + ph) / 7;
    final u0 = 1 - 1 / (1 + z * 4), u1 = 1 - 1 / (1 + (z + .06) * 4);
    _ln(c, mid(u0 * .98), mid(u1 * .98), _gr, 1.4 - z);
  }
  _ln(c, const Offset(0, 102), Offset(s.width, 102), _wh, 1);
  c.save();
  c.translate(78, 124);
  c.rotate(steer * 1.3);
  c.drawArc(Rect.fromCircle(center: Offset.zero, radius: 26), math.pi * 1.1, math.pi * .8, false, _s(_wh, 1.2));
  _dot(c, const Offset(0, -26), 2, _rd);
  _ln(c, const Offset(-10, -8), const Offset(10, -8), _wh, 1);
  c.restore();
  _lab(c, 'km/h', const Offset(6, 108), _gr);
  _num(c, _d3(40 + sp * 180), const Offset(30, 105), 12, _gr);
  _lab(c, steer < 0 ? 'left' : 'right', const Offset(6, 6), _bl);
  _num(c, _d2(steer.abs() * 90), const Offset(5, 15), 16, _bl);
}

void _rCube(Canvas c, Size s, _S st, double t) {
  final ax = (st.b * 2.999).floor(), ang = st.a * _tau + t * .35;
  const o = Offset(62, 62), k = 22.0;
  _V3 tr(_V3 v) {
    final r = ax == 0 ? _rx(v, ang) : (ax == 1 ? _ry(v, ang) : _rz(v, ang));
    return _rx(_ry(r, .62), -.46);
  }

  final vs = [for (var i = 0; i < 8; i++) tr(((i & 1) * 2 - 1.0, (i >> 1 & 1) * 2 - 1.0, (i >> 2 & 1) * 2 - 1.0))];
  for (var i = 0; i < 8; i++) {
    for (final b in [1, 2, 4]) {
      final j = i | b;
      if (j != i) _ln(c, _o3(o, vs[i], k), _o3(o, vs[j], k), _wh, 1);
    }
  }
  final face = [vs[1], vs[3], vs[7], vs[5]].map((v) => _o3(o, v, k)).toList();
  _pl(c, face, _bl, w: 1.3, close: true);
  _dot(c, _o3(o, vs[7], k), 2, _rd);
  final av = ax == 0 ? (1.0, 0.0, 0.0) : (ax == 1 ? (0.0, 1.0, 0.0) : (0.0, 0.0, 1.0));
  final e = _rx(_ry(av, .62), -.46);
  final p0 = _o3(o, (e.$1 * -2.3, e.$2 * -2.3, 0), k), p1 = _o3(o, (e.$1 * 2.3, e.$2 * 2.3, 0), k);
  _dots(c, p0, p1, _wh, gap: 3.5);
  _arrow(c, p1, (p1 - p0).direction, _wh);
  for (var i = 0; i < 3; i++) {
    final y = 30.0 + i * 22, on = i == ax;
    if (on) _ring(c, Offset(s.width - 14, y + 4), 8, _wh, 1);
    _lab(c, 'xyz'[i], Offset(s.width - 14, y), on ? _wh : _dg, ax: .5, size: 9);
  }
  _lab(c, 'axis', Offset(s.width - 6, 8), _wh, ax: 1);
  _num(c, _deg(ang / _tau), const Offset(6, 98), 14, _bl);
}

void _rClock(Canvas c, Size s, _S st, double t) {
  const o = Offset(54, 60), r = 44.0;
  for (var i = 0; i < 60; i++) {
    final a = i / 60 * _tau, big = i % 5 == 0;
    _ln(c, _pol(o, big ? r - 6 : r - 2, a), _pol(o, r, a), big ? _wh : _dg, 1);
  }
  final mA = st.a * 12 * _tau - math.pi / 2, hA = st.a * _tau - math.pi / 2;
  final v = st.va.abs(), n = (v * 18).clamp(0, 14).round(), sg = st.va.sign;
  for (var i = n; i >= 1; i--) {
    _ln(c, o, _pol(o, 36, mA - sg * i * .1), _al(_bl, .5 - i / (n + 1) * .45), 1);
  }
  _ln(c, o, _pol(o, 36, mA), _bl, 1.5);
  _ln(c, o, _pol(o, 22, hA), _wh, 1.6);
  final sA = t * _tau / 6 - math.pi / 2;
  _ln(c, _pol(o, -6, sA), _pol(o, 40, sA), _al(_rd, .8), .7);
  _dot(c, o, 2, _rd);
  final mins = (st.a * 720).floor();
  final hh = mins ~/ 60 == 0 ? 12 : mins ~/ 60;
  _num(c, _d2(hh), Offset(s.width - 6, 12), 28, _wh, ax: 1);
  _num(c, _d2(mins % 60), Offset(s.width - 6, 44), 28, _bl, ax: 1);
  _lab(c, 'spd', Offset(s.width - 6, 96), _gr, ax: 1);
  _num(c, _d2(v * 40), Offset(s.width - 6, 104), 11, _gr, ax: 1);
}

void _rMill(Canvas c, Size s, _S st, double t) {
  double gy(double x) => 96 + math.sin(x * .045) * 6 + math.sin(x * .13 + 1) * 2;
  _pl(c, [for (var x = 0.0; x <= s.width; x += 4) Offset(x, gy(x))], _wh, w: 1);
  _pl(c, [for (var x = 0.0; x <= s.width; x += 4) Offset(x, 106 + math.sin(x * .07 + 2) * 3)], _dg, w: .8);
  final rot = st.a * _tau + t * st.b * 3;
  for (final (mx, sz) in [(34.0, 1.0), (92.0, .7), (132.0, .48)]) {
    final g = gy(mx), hub = Offset(mx, g - 44 * sz);
    _pl(c, [Offset(mx - 5 * sz, g), Offset(mx - 1.5 * sz, hub.dy), Offset(mx + 1.5 * sz, hub.dy), Offset(mx + 5 * sz, g)],
        _wh, w: 1);
    final rr = rot * (1 / sz) * .7;
    for (var k = 0; k < 4; k++) {
      final a = rr + k * math.pi / 2;
      final tip = _pol(hub, 26 * sz, a);
      _pl(c, [hub, _pol(hub, 9 * sz, a + .22), tip, _pol(hub, 9 * sz, a - .05)], _bl, w: 1, close: true);
    }
    _dot(c, hub, 1.6, _rd);
  }
  for (var i = 0; i < 6; i++) {
    final y = 14.0 + i * 9, sp = 20 + st.b * 140;
    final x = (t * sp + _rn(i) * 180) % 180 - 14;
    _ln(c, Offset(x, y), Offset(x + 6 + st.b * 10, y), _gr, 1);
  }
  _lab(c, 'wind', Offset(s.width - 6, 6), _gr, ax: 1);
  _num(c, _d2(st.b * 99), Offset(s.width - 6, 15), 14, _gr, ax: 1);
  _num(c, _deg(rot / _tau), const Offset(6, 6), 12, _bl);
}

void _rGyro(Canvas c, Size s, _S st, double t) {
  final tilt = (st.b - .5) * 1.6, spin = st.a * _tau + t * 1.4;
  const o = Offset(70, 56), k = 38.0;
  _V3 view(_V3 v) => _rx(_ry(v, .5), -.35);
  _ring3(c, o, k, (v) => view(v), _wh);
  _ring3(c, o, k * .82, (v) => view(_rz(_ry(v, math.pi / 2), tilt)), _pu);
  _ring3(c, o, k * .62, (v) => view(_rz(_rx(v, math.pi / 2), tilt)), _bl, 1.2);
  for (var i = 0; i < 3; i++) {
    final a = spin + i * _tau / 3;
    final p = view(_rz(_rx((math.cos(a) * .62, math.sin(a) * .62, 0), math.pi / 2), tilt));
    _ln(c, o, _o3(o, p, k), _bl, 1);
  }
  final ax0 = _o3(o, view(_rz((0.0, -1.15, 0.0), tilt)), k), ax1 = _o3(o, view(_rz((0.0, 1.15, 0.0), tilt)), k);
  _ln(c, ax0, ax1, _wh, 1);
  _dot(c, ax0, 1.8, _wh);
  _pl(c, [Offset(o.dx - 16, 112), Offset(o.dx, o.dy + k + 4), Offset(o.dx + 16, 112)], _dg, w: 1);
  for (var i = 0; i < 5; i++) {
    final a = spin * 1.3 + i * .3;
    c.drawArc(Rect.fromCircle(center: o, radius: k + 8), a, .12, false, _s(_al(_gr, .8 - i * .15), 1));
  }
  _lab(c, 'axis', Offset(s.width - 6, 8), _wh, ax: 1);
  _num(c, '${tilt < 0 ? '-' : '+'}${_d2(tilt.abs() * 180 / math.pi)}', Offset(s.width - 6, 18), 18, _wh, ax: 1);
  _lab(c, 'rotor', Offset(s.width - 6, 90), _bl, ax: 1);
  _num(c, _deg(spin / _tau), Offset(s.width - 6, 100), 12, _bl, ax: 1);
}

void _rSpiro(Canvas c, Size s, _S st, double t) {
  const o = Offset(60, 60), big = 46.0;
  final r = 8 + (st.a * 26).roundToDouble(), d = (.2 + st.b * 1.1) * r;
  final th = t * 1.4;
  Offset pen(double a) {
    final gc = _pol(o, big - r, a);
    return _pol(gc, d, -a * (big - r) / r);
  }

  final pts = <Offset>[for (var i = 0; i < 260; i++) pen(th - i * .09)];
  _pl(c, pts, _al(_gr, .75), w: .8);
  _ring(c, o, big, _wh, 1);
  for (var i = 0; i < 48; i++) {
    final a = i / 48 * _tau;
    _ln(c, _pol(o, big, a), _pol(o, big + 2.5, a), _dg, 1);
  }
  final gc = _pol(o, big - r, th);
  _ring(c, gc, r, _bl, 1.2);
  final p = pen(th);
  _ln(c, gc, p, _bl, 1);
  _dot(c, p, 2.2, _rd);
  _lab(c, 'ratio', Offset(s.width - 6, 8), _bl, ax: 1);
  _num(c, '46', Offset(s.width - 6, 18), 16, _wh, ax: 1);
  _num(c, _d2(r), Offset(s.width - 6, 38), 16, _bl, ax: 1);
  _lab(c, 'pen', Offset(s.width - 6, 92), _gr, ax: 1);
  _num(c, _d2(st.b * 99), Offset(s.width - 6, 102), 12, _gr, ax: 1);
}

void _rDog(Canvas c, Size s, _S st, double t) {
  const o = Offset(68, 62), rd = 30.0;
  final sp = .4 + st.b * 9, th = st.a * _tau + t * sp;
  final b0 = th - math.pi * 1.5, b1 = th - math.pi * .2;
  final ph = t * sp * 3;
  for (final la in [b1 - .25, b1 - .5, b0 + .25, b0 + .5]) {
    for (final d in [-1.0, 1.0]) {
      final sw = math.sin(ph + la * 3 + d) * .18;
      _ln(c, _pol(o, rd + d * 7, la), _pol(o, rd + d * 15, la + sw), _wh, 1);
    }
  }
  for (final rr in [rd - 7, rd + 7]) {
    c.drawArc(Rect.fromCircle(center: o, radius: rr), b0, b1 - b0, false, _s(_wh, 1.1));
  }
  c.drawArc(Rect.fromCircle(center: _pol(o, rd, b0), radius: 7), b0 + math.pi * .5, math.pi, false, _s(_wh, 1.1));
  final ds = <Offset>[for (var a = b0 + .3; a < b1 - .1; a += .32) _pol(o, rd, a)];
  c.drawPoints(ui.PointMode.points, ds, _s(_al(_wh, .45), 1.6));
  final fw = th + math.pi / 2;
  final hd = _pol(o, rd, b1 + .12);
  _ring(c, hd, 9, _bl, 1.2);
  final nose = _pol(hd, 16, fw - .15);
  _pl(c, [_pol(hd, 8, fw - .9), nose, _pol(hd, 8, fw + .7)], _bl, w: 1.1);
  _dot(c, nose, 1.8, _bl);
  for (final ea in [th, th + math.pi]) {
    final eb = _pol(hd, 8, ea);
    _pl(c, [eb, eb + Offset.fromDirection(fw + math.pi, 7), _pol(hd, 8, ea + (ea == th ? -.6 : .6))], _bl, w: 1);
  }
  _dot(c, _pol(hd, 4, fw + .7), 1.2, _wh);
  _dot(c, _pol(hd, 4, fw - .9), 1.2, _wh);
  final tb = _pol(o, rd, b0), wag = math.sin(t * 14) * .45;
  _pl(c, [tb, tb + Offset.fromDirection(b0 - math.pi / 2 + wag * .5, 8), tb + Offset.fromDirection(b0 - math.pi / 2 + wag, 15)],
      _rd, w: 1.4);
  if (st.b > .2) {
    for (var i = 0; i < (st.b * 8).round(); i++) {
      _dot(c, _pol(o, rd + 22 + _rn(i) * 5, th - .6 - i * .3), .9 + _rn(i + 3), _al(_gr, .9 - i * .1));
    }
  }
  _lab(c, 'laps/s', Offset(s.width - 6, 8), _gr, ax: 1);
  _num(c, (sp / _tau).toStringAsFixed(1), Offset(s.width - 6, 18), 18, _gr, ax: 1);
  _num(c, _deg(th / _tau), Offset(s.width - 6, 100), 12, _bl, ax: 1);
}

void _rChain(Canvas c, Size s, _S st, double t) {
  const y = 44.0;
  c.drawRect(const Rect.fromLTWH(4, y - 12, 24, 24), _s(_wh, 1));
  _pl(c, [const Offset(11, y + 7), const Offset(11, y - 7), const Offset(20, y - 7)], _wh, w: 1);
  _ln(c, const Offset(11, y), const Offset(18, y), _wh, 1);
  _ln(c, const Offset(28, y), const Offset(36, y), _dg, 1);
  _pl(c, [const Offset(36, y - 15), const Offset(64, y), const Offset(36, y + 15)], _gr, w: 1.2, close: true);
  _num(c, _d2(st.a * 99), const Offset(38, y - 7), 13, _gr);
  _ln(c, const Offset(64, y), const Offset(74, y), _dg, 1);
  for (var i = 2; i >= 0; i--) {
    final r = Rect.fromLTWH(74.0 + i * 3, y - 12 - i * 3, 24, 24);
    c.drawRect(r, _f(_bk));
    c.drawRect(r, _s(i == 0 ? _wh : _dg, 1));
  }
  c.drawArc(Rect.fromCircle(center: const Offset(86, y), radius: 6), -math.pi * .9, math.pi * 1.5, false, _s(_bl, 1));
  _arrow(c, _pol(const Offset(86, y), 6, math.pi * .6), math.pi * 1.1, _bl, 3);
  _ln(c, const Offset(104, y), const Offset(118, y), _dg, 1);
  const wc = Offset(134, y);
  _ring(c, wc, 15, _wh, 1);
  final ph = st.b * _tau + t * st.a * 8;
  for (var i = 0; i < 8; i++) {
    final a = ph + i * _tau / 8;
    _dot(c, _pol(wc, 10, a), i == 0 ? 2.4 : 1.2, i == 0 ? _rd : _al(_wh, .6));
  }
  for (final (x, l) in [(16.0, 'src'), (48.0, 'speed'), (88.0, 'rotate'), (134.0, 'angle')]) {
    _lab(c, l, Offset(x, y + 22), _al(_wh, .7), ax: .5, size: 7);
  }
  final pts = <Offset>[];
  for (var x = 8.0; x <= 148; x += 2) {
    pts.add(Offset(x, 108 - _wr((x - 8) / 140 * (1 + st.a * 6) + st.b) * 18));
  }
  _pl(c, pts, _bl, w: 1);
  _ln(c, const Offset(8, 108), const Offset(148, 108), _dg, .8);
  final px = 8 + _wr(t * .25) * 140;
  _dots(c, Offset(px, 86), Offset(px, 110), _gr);
}

void _rSweep(Canvas c, Size s, _S st, double t) {
  const pv = Offset(62, 64);
  var ang = st.a * _tau;
  if (st.pts.length > 1) {
    ang = 0;
    for (var i = 1; i < st.pts.length; i++) {
      var d = (st.pts[i] - pv).direction - (st.pts[i - 1] - pv).direction;
      if (d > math.pi) d -= _tau;
      if (d < -math.pi) d += _tau;
      ang += d;
    }
  }
  const f = [Offset(0, 0), Offset(0, -34), Offset(20, -34), Offset(0, -34), Offset(0, -18), Offset(14, -18)];
  void drawF(double a, Color col, double w) {
    final ca = math.cos(a), sa = math.sin(a);
    _shape(c, f, (p) => pv + Offset(p.dx * ca - p.dy * sa, p.dx * sa + p.dy * ca), col, w: w, close: false);
  }

  drawF(0, _dg, 1);
  for (var i = 1; i < 6; i++) {
    drawF(ang * i / 6, _al(_bl, .15 + i * .1), 1);
  }
  drawF(ang, _wh, 1.4);
  c.drawArc(Rect.fromCircle(center: pv, radius: 44), -math.pi / 2, ang, false, _s(_gr, 1));
  _arrow(c, _pol(pv, 44, -math.pi / 2 + ang), -math.pi / 2 + ang + (ang >= 0 ? math.pi / 2 : -math.pi / 2), _gr);
  _ln(c, pv - const Offset(5, 0), pv + const Offset(5, 0), _rd, 1);
  _ln(c, pv - const Offset(0, 5), pv + const Offset(0, 5), _rd, 1);
  if (st.pts.length > 1) _pl(c, st.pts, _al(_rd, .45), w: .8);
  final dg = (ang * 180 / math.pi).round();
  _lab(c, 'sweep', Offset(s.width - 6, 8), _bl, ax: 1);
  _num(c, '${dg < 0 ? '-' : ''}${_d3(dg.abs())}', Offset(s.width - 6, 18), 20, _bl, ax: 1);
}

void _rHorizon(Canvas c, Size s, _S st, double t) {
  final roll = (st.a - .5) * math.pi * .5, pitch = (st.b - .5) * 44;
  c.save();
  c.translate(_c0.dx, _c0.dy);
  c.rotate(roll);
  c.translate(0, pitch);
  _ln(c, const Offset(-160, 0), const Offset(160, 0), _wh, 1);
  final m = <Offset>[];
  for (var x = -160.0; x <= 160; x += 8) {
    m.add(Offset(x, -(math.sin(x * .07) * .5 + .5) * 14 * _rn(((x + 160) / 8).round()) - 2));
  }
  _pl(c, m, _bl, w: 1);
  _ring(c, Offset(34 + math.sin(t * .2) * 4, -30), 7, _rd, 1);
  for (final y in [6.0, 14.0, 26.0, 44.0, 70.0]) {
    _ln(c, Offset(-160, y), Offset(160, y), _dg, .7);
  }
  for (var i = -8; i <= 8; i++) {
    _ln(c, Offset(i * 4.0, 0), Offset(i * 40.0, 90), _dg, .7);
  }
  c.restore();
  _pl(c, [_c0 + const Offset(-30, 0), _c0 + const Offset(-10, 0), _c0 + const Offset(0, 7), _c0 + const Offset(10, 0),
      _c0 + const Offset(30, 0)], _gr, w: 1.4);
  _dot(c, _c0, 1.6, _gr);
  for (var k = -3; k <= 3; k++) {
    final a = -math.pi / 2 + k * math.pi / 12;
    _ln(c, _pol(_c0, 50, a), _pol(_c0, k == 0 ? 56 : 53, a), _wh, 1);
  }
  final pa = -math.pi / 2 - roll;
  _pl(c, [_pol(_c0, 45, pa), _pol(_c0, 41, pa - .06), _pol(_c0, 41, pa + .06)], _bl, w: 1.2, close: true);
  for (final (x, y, sx, sy) in [(4.0, 4.0, 1.0, 1.0), (152.0, 4.0, -1.0, 1.0), (4.0, 116.0, 1.0, -1.0), (152.0, 116.0, -1.0, -1.0)]) {
    _pl(c, [Offset(x, y + sy * 8), Offset(x, y), Offset(x + sx * 8, y)], _wh, w: 1);
  }
  final rd = (roll * 180 / math.pi).round();
  _lab(c, 'roll', const Offset(10, 92), _bl);
  _num(c, '${rd < 0 ? '-' : '+'}${_d2(rd.abs())}', const Offset(9, 100), 12, _bl);
  _lab(c, 'pitch', const Offset(146, 92), _gr, ax: 1);
  _num(c, _d2(pitch.abs()), const Offset(146, 100), 12, _gr, ax: 1);
}

void _rGalaxy(Canvas c, Size s, _S st, double t) {
  final rot = st.a * _tau + t * .15, wind = .4 + st.b * 4.5;
  final core = <Offset>[], mid = <Offset>[], out = <Offset>[];
  for (var i = 0; i < 220; i++) {
    final arm = i % 2, u = (i ~/ 2) / 110;
    final r = 3 + u * 56 + (_rn(i) - .5) * 5;
    final ph = arm * math.pi + u * wind * 3 + rot + (_rn(i + 7) - .5) * .35;
    final p = _pol(_c0, r, ph);
    (u < .25 ? core : (u < .65 ? mid : out)).add(p);
  }
  c.drawPoints(ui.PointMode.points, core, _s(_wh, 1.6));
  c.drawPoints(ui.PointMode.points, mid, _s(_bl, 1.3));
  c.drawPoints(ui.PointMode.points, out, _s(_al(_bl, .5), 1));
  final gp = <Offset>[for (var i = 0; i < 18; i++) _pol(_c0, 10 + _rn(i + 50) * 48, _rn(i + 90) * _tau + rot)];
  c.drawPoints(ui.PointMode.points, gp, _s(_gr, 1.4));
  _ring(c, _c0, 3, _wh, 1);
  _dot(c, _pol(_c0, 34, rot + wind * 1.8), 2, _rd);
  _lab(c, 'twist', const Offset(6, 8), _gr);
  _num(c, wind.toStringAsFixed(1), const Offset(5, 18), 16, _gr);
  _num(c, _deg(rot / _tau), Offset(s.width - 6, 100), 12, _bl, ax: 1);
}

void _rWasher(Canvas c, Size s, _S st, double t) {
  final sp = st.b, shake = math.sin(t * 47) * sp * sp * 1.8;
  c.save();
  c.translate(shake, 0);
  c.drawRRect(RRect.fromLTRBR(26, 6, 130, 112, const Radius.circular(4)), _s(_wh, 1));
  _ln(c, const Offset(26, 22), const Offset(130, 22), _wh, 1);
  _ring(c, const Offset(36, 14), 3, _dg, 1);
  _num(c, (sp * 1400).round().toString().padLeft(4, '0'), const Offset(124, 9), 10, _gr, ax: 1);
  const o = Offset(78, 66);
  _ring(c, o, 36, _wh, 1.2);
  _ring(c, o, 30, _dg, 1);
  final dr = st.a * _tau * 2 + t * sp * 14;
  for (var i = 0; i < 12; i++) {
    _dot(c, _pol(o, 27, dr + i * _tau / 12), .9, _gr);
  }
  final pin = sp > .55;
  final tumble = pin ? 0.0 : math.sin(t * 3) * 4;
  final r1 = pin ? 20.0 : 15 + tumble, a1 = pin ? dr : dr * .4 + 1.6;
  final sc = _pol(o, r1, a1);
  _shape(c, const [Offset(-3, -6), Offset(3, -6), Offset(3, 2), Offset(8, 5), Offset(8, 8), Offset(-3, 8)],
      (p) => sc + Offset(p.dx * math.cos(a1) - p.dy * math.sin(a1), p.dx * math.sin(a1) + p.dy * math.cos(a1)), _bl);
  final a2 = pin ? dr + 2.6 : dr * .4 + 3.4, r2 = pin ? 19.0 : 12 - tumble;
  final tc = _pol(o, r2, a2);
  _shape(c, const [Offset(-4, -6), Offset(-9, -3), Offset(-6, 0), Offset(-4, -1), Offset(-4, 7), Offset(4, 7),
      Offset(4, -1), Offset(6, 0), Offset(9, -3), Offset(4, -6)],
      (p) => tc + Offset(p.dx * math.cos(a2) - p.dy * math.sin(a2), p.dx * math.sin(a2) + p.dy * math.cos(a2)), _bl);
  if (!pin) {
    _pl(c, [for (var x = -24.0; x <= 24; x += 3) o + Offset(x, 18 + math.sin(x * .4 + t * 4) * 1.5)], _al(_wh, .4), w: .8);
  }
  c.restore();
  _ln(c, const Offset(30, 114), const Offset(40, 114), _dg, 1);
  _ln(c, const Offset(116, 114), const Offset(126, 114), _dg, 1);
  _lab(c, 'rpm', const Offset(96, 11), _gr, ax: 1, size: 7);
}

void _rTop(Canvas c, Size s, _S st, double t) {
  final tilt = st.b * .7, psi = t * (1.4 - st.b * .6), spin = st.a * _tau + t * 9;
  const tip = Offset(70, 100);
  _ln(c, const Offset(8, 101), const Offset(148, 101), _dg, 1);
  final ax = Offset(math.sin(tilt) * math.cos(psi), -math.cos(tilt) + math.sin(tilt) * math.sin(psi) * .25);
  final top = tip + ax * 62;
  _dots(c, Offset(tip.dx - 62 * math.sin(tilt), top.dy), Offset(tip.dx + 62 * math.sin(tilt), top.dy), _dg);
  c.drawOval(Rect.fromCenter(center: Offset(tip.dx, tip.dy - 62 * math.cos(tilt)), width: 124 * math.sin(tilt) + .1,
      height: 31 * math.sin(tilt) + .1), _s(_dg, .8));
  c.save();
  c.translate(tip.dx, tip.dy);
  c.rotate(ax.direction + math.pi / 2);
  _ln(c, Offset.zero, const Offset(-20, -24), _wh, 1);
  _ln(c, Offset.zero, const Offset(20, -24), _wh, 1);
  c.drawOval(Rect.fromCenter(center: const Offset(0, -24), width: 40, height: 9), _s(_wh, 1));
  _pl(c, [const Offset(-20, -24), const Offset(-6, -34), const Offset(6, -34), const Offset(20, -24)], _wh, w: 1);
  c.drawRect(const Rect.fromLTRB(-2.5, -46, 2.5, -34), _s(_wh, 1));
  for (var k = 0; k < 6; k++) {
    final a = spin + k * math.pi / 3;
    if (math.sin(a) > 0) _ln(c, Offset(20 * math.cos(a), -24 + 4.5 * math.sin(a)), Offset.zero, _gr, 1);
  }
  _dots(c, const Offset(0, -46), const Offset(0, -64), _wh);
  c.restore();
  _dot(c, tip, 1.5, _rd);
  _lab(c, 'axis', Offset(s.width - 6, 8), _wh, ax: 1);
  _num(c, _d2(tilt * 180 / math.pi), Offset(s.width - 6, 18), 20, _wh, ax: 1);
  _lab(c, 'spin', const Offset(6, 8), _gr);
  _num(c, _deg(spin / _tau), const Offset(5, 18), 12, _gr);
}

void _rHamster(Canvas c, Size s, _S st, double t) {
  const o = Offset(68, 54), r = 42.0;
  final sp = st.b, ang = st.a * _tau * 2 + t * sp * 5;
  _ln(c, o, const Offset(46, 114), _dg, 1);
  _ln(c, o, const Offset(90, 114), _dg, 1);
  _ln(c, const Offset(38, 114), const Offset(98, 114), _dg, 1);
  _ring(c, o, r, _wh, 1);
  _ring(c, o, r - 4, _wh, .7);
  for (var i = 0; i < 24; i++) {
    final a = ang + i * _tau / 24;
    _ln(c, _pol(o, r - 4, a), _pol(o, r, a), _bl, 1);
  }
  for (var i = 0; i < 6; i++) {
    _ln(c, o, _pol(o, r - 4, ang + i * _tau / 6), _al(_bl, .5), .8);
  }
  _dot(c, o, 2, _wh);
  const fy = 54.0 + r - 4;
  final run = sp > .08, ph = t * (6 + sp * 24), bob = run ? math.sin(ph * 2) * 1.2 : 0.0;
  final bc = Offset(66, fy - 9 + bob);
  c.drawOval(Rect.fromCenter(center: bc, width: 26, height: 14), _s(_wh, 1));
  final hd = bc + const Offset(13, -3);
  _ring(c, hd, 6, _wh, 1);
  _ring(c, hd + const Offset(-2, -6), 2, _wh, 1);
  _dot(c, hd + const Offset(2, -1), 1, _wh);
  _dot(c, hd + const Offset(6, 1), 1, _rd);
  for (var k = 0; k < 4; k++) {
    final lx = bc.dx - 8 + k * 6, sw = run ? math.sin(ph + k * 1.6) * 4 : 0.0;
    _ln(c, Offset(lx, bc.dy + 5), Offset(lx + sw, fy), _wh, 1);
  }
  if (run) {
    for (var i = 0; i < 3; i++) {
      final y = bc.dy - 4 + i * 4.0;
      _ln(c, Offset(bc.dx - 18 - i * 2, y), Offset(bc.dx - 18 - i * 2 - sp * 16, y), _gr, 1);
    }
  }
  _lab(c, 'speed', Offset(s.width - 6, 8), _gr, ax: 1);
  _num(c, _d2(sp * 99), Offset(s.width - 6, 18), 20, _gr, ax: 1);
  _lab(c, 'turns', Offset(s.width - 6, 88), _bl, ax: 1);
  _num(c, _d3((ang / _tau).abs() % 1000), Offset(s.width - 6, 98), 14, _bl, ax: 1);
}

void _rStrobe(Canvas c, Size s, _S st, double t) {
  const fps = 12.0, n = 8;
  final w = st.b * 40;
  final ts = (t * fps).floor() / fps;
  double real(double tt) => st.a * _tau + w * tt;
  const lo = Offset(42, 54), ro = Offset(114, 54), r = 30.0;
  _ring(c, lo, r, _wh, 1);
  _ring(c, ro, r, _wh, 1);
  for (var j = 0; j < 7; j++) {
    final a0 = real(t - j * .006);
    for (var i = 0; i < n; i++) {
      _ln(c, lo, _pol(lo, r, a0 + i * _tau / n), _al(_bl, .7 - j * .09), 1);
    }
  }
  final sa = real(ts);
  for (var i = 0; i < n; i++) {
    _ln(c, ro, _pol(ro, r, sa + i * _tau / n), _wh, 1.2);
  }
  _dot(c, _pol(ro, r, sa), 2, _rd);
  final step = _wr((w / fps) / (_tau / n));
  final back = step > .5, still = step < .03 || (step - .5).abs() < .02 || step > .97;
  final ac = still ? _dg : (back ? _rd : _gr);
  final y = 98.0;
  c.drawArc(Rect.fromCircle(center: Offset(ro.dx, y - 6), radius: 10), math.pi * .2, math.pi * .6, false, _s(ac, 1));
  _arrow(c, _pol(Offset(ro.dx, y - 6), 10, back ? math.pi * .8 : math.pi * .2), back ? math.pi * 1.3 : math.pi * -.3, ac);
  _lab(c, 'real', Offset(lo.dx, 92), _bl, ax: .5);
  _num(c, _d2(w / _tau * 10), Offset(lo.dx, 101), 12, _gr, ax: .5);
  _lab(c, back ? 'seen  back' : (still ? 'seen  still' : 'seen  fwd'), Offset(ro.dx, 106), ac, ax: .5, size: 7);
  if (_wr(t * fps) < .3) _dot(c, Offset(s.width - 8, 8), 2.5, _rd);
  _lab(c, '12 fps', const Offset(6, 6), _dg);
}

void _rVortex(Canvas c, Size s, _S st, double t) {
  final rot = st.a * _tau + t * .5, twist = 1 + st.b * 4;
  const w = 'SPIN·';
  for (var i = 0; i < 46; i++) {
    final u = i / 46;
    final r = 60 * (1 - u) + 4, ph = rot + u * _tau * twist;
    final p = _pol(_c0, r, ph);
    final sz = (5 + (1 - u) * 11).roundToDouble();
    final col = u < .3 ? _bl : (u < .7 ? _wh : _al(_gr, .9));
    final tp = _tp(w[i % w.length], sz, col, FontWeight.w300, 0);
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(ph + math.pi / 2);
    tp.paint(c, Offset(-tp.width / 2, -tp.height / 2));
    c.restore();
  }
  _dot(c, _c0, 2, _rd);
  _lab(c, 'twist', const Offset(6, 6), _gr);
  _num(c, twist.toStringAsFixed(1), const Offset(5, 15), 12, _gr);
}
