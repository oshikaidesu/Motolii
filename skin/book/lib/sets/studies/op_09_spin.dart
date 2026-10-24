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
