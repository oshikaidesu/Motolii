part of 'pop_10.dart';

// Position / Transform: why you touch it = "put the thing there", "bring it closer / send it back", "spin it about that point".

List<_Pn> _posPanels() => [
      const _Pn('stage spotlight', 'character · flat · drag x/depth · result', _stage, y0: .6),
      const _Pn('iso room', 'toy · isometric · drag on floor · mechanism', _isoRoom, x0: .3, y0: .4),
      const _Pn('hang from pin', 'material · flat · drag anchor · physics', _hang, x0: .3, y0: .2, step: _hangStep, light: true),
      const _Pn('parallax valley', 'landscape · pseudo 3D · up = far · result', _valley, y0: .7),
      const _Pn('treasure route', 'map · top-down · drag path · mechanism', _treasure, x0: .75, y0: .3),
      const _Pn('orbit anchor', 'cosmic · pseudo 3D · spin · anchor', _orbit),
      const _Pn('pool throw', 'physics · top-down · throw · result', _pool, step: _poolStep),
      const _Pn('aquarium depth', 'creature · flat · down = deeper · result', _aquarium, x0: .4, y0: .4),
      const _Pn('marionette', 'character · flat · drag bar · spring', _marionette, step: _marioStep, y0: .2),
      const _Pn('viewfinder thirds', 'camera · flat · drag snaps · result', _viewfinder, x0: .3, y0: .62),
      const _Pn('tilt board', 'physics · pseudo 3D · tilt · mechanism', _tilt, step: _tiltStep),
      const _Pn('claw machine', 'machine · flat · drag + lower · toy', _claw, x0: .4, y0: .4),
      const _Pn('card lift', 'material · pseudo 3D · up = lift z · stack', _cardLift, y0: .3),
      const _Pn('shadow puppet', 'light · flat · down = near lamp · z', _shadow, light: true, x0: .45, y0: .4),
      const _Pn('pinwheel anchor', 'toy · flat · drag anchor · spin', _pinwheel, x0: .7, y0: .6),
      const _Pn('snail route', 'creature · flat · paint path · motion', _snail),
      const _Pn('neon tunnel', 'cosmic · pseudo 3D · down = near · neon', _tunnel, y0: .55),
      const _Pn('board snap', 'game · top-down · drag snaps · grid', _chess, step: _chessStep, light: true, x0: .4, y0: .5),
      const _Pn('radar blip', 'machine · top-down · drag · neon', _radar, x0: .7, y0: .3),
      const _Pn('pixel quest', 'game · pixel 2D · drag steps · map', _pixel, step: _pixelStep, x0: .2, y0: .3),
    ];

void _stage(Canvas c, Size s, _S st) {
  c.drawRect(const Rect.fromLTWH(0, 0, _cw, 60), _f(_k1));
  final floor = _poly(const [Offset(38, 58), Offset(118, 58), Offset(_cw, _ch), Offset(0, _ch)]);
  c.drawPath(floor, _f(_mix(_or, _k0, .62)));
  for (var i = 0; i < 2; i++) {
    final sx = i == 0 ? 0.0 : _cw, d = i == 0 ? 1.0 : -1.0;
    c.drawPath(
        Path()
          ..moveTo(sx, 0)
          ..lineTo(sx + d * 36, 0)
          ..quadraticBezierTo(sx + d * 18, 30, sx + d * 30, 64)
          ..lineTo(sx, 72)
          ..close(),
        _f(_rd));
    for (var k = 1; k < 3; k++) {
      c.drawLine(Offset(sx + d * k * 10, 2), Offset(sx + d * (k * 8 + 2), 66), _s(_mix(_rd, _k0, .35), 1.5));
    }
  }
  c.drawRect(const Rect.fromLTWH(0, 0, _cw, 8), _f(_rd));
  final d = st.y, fy = 60 + d * 48, hw = 40 + d * 36, ax = 78 + (st.x - .5) * 2 * hw * .9, sc = .55 + .7 * d;
  c.drawPath(_poly([const Offset(70, 0), const Offset(86, 0), Offset(ax + 20 * sc, fy), Offset(ax - 20 * sc, fy)]), _f(_al(_ye, .16)));
  c.drawOval(Rect.fromCenter(center: Offset(ax, fy), width: 40 * sc, height: 10 * sc), _f(_al(_ye, .7)));
  final bob = math.sin(st.t * 6).abs() * 2 * sc;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(ax - 8 * sc, fy - 28 * sc - bob, ax + 8 * sc, fy - 3 * sc - bob), Radius.circular(7 * sc)), _f(_pk));
  final hd = Offset(ax, fy - 34 * sc - bob);
  c.drawCircle(hd, 8 * sc, _f(_cr));
  c.drawCircle(hd + Offset(-3, -1) * sc, 1.4 * sc, _f(_k0));
  c.drawCircle(hd + Offset(3, -1) * sc, 1.4 * sc, _f(_k0));
  c.drawArc(Rect.fromCenter(center: hd + Offset(0, 2) * sc, width: 6 * sc, height: 4 * sc), 0, math.pi, false, _s(_k0, 1.2 * sc));
}

void _isoRoom(Canvas c, Size s, _S st) {
  const o = Offset(78, 36), ex = Offset(10.5, 6), ey = Offset(-10.5, 6);
  Offset p(double i, double j) => o + ex * i + ey * j;
  const up = Offset(0, -32);
  c.drawPath(_poly([p(0, 0), p(0, 6), p(0, 6) + up, p(0, 0) + up]), _f(_mix(_vi, _k0, .45)));
  c.drawPath(_poly([p(0, 0), p(6, 0), p(6, 0) + up, p(0, 0) + up]), _f(_mix(_vi, _k0, .25)));
  c.drawPath(_poly([p(2.2, 0) + const Offset(0, -24), p(3.8, 0) + const Offset(0, -24), p(3.8, 0) + const Offset(0, -10), p(2.2, 0) + const Offset(0, -10)]), _f(_cy));
  c.drawPath(_poly([p(0, 0), p(6, 0), p(6, 6), p(0, 6)]), _f(_k1));
  for (var i = 0; i <= 6; i++) {
    c.drawLine(p(i * 1.0, 0), p(i * 1.0, 6), _s(_al(_cr, .14), 1));
    c.drawLine(p(0, i * 1.0), p(6, i * 1.0), _s(_al(_cr, .14), 1));
  }
  final u = st.x * 5, v = st.y * 5, ti = (u + .5).floorToDouble().clamp(0.0, 5.0), tj = (v + .5).floorToDouble().clamp(0.0, 5.0);
  c.drawPath(_poly([p(ti, tj), p(ti + 1, tj), p(ti + 1, tj + 1), p(ti, tj + 1)]), _f(_al(_li, .55)));
  final z = 4 + math.sin(st.t * 2.4) * 3;
  c.drawPath(_poly([p(u, v), p(u + 1, v), p(u + 1, v + 1), p(u, v + 1)]), _f(_al(_k0, .5)));
  final lift = Offset(0, -z), top = Offset(0, -z - 15);
  c.drawPath(_poly([p(u + 1, v) + lift, p(u + 1, v + 1) + lift, p(u + 1, v + 1) + top, p(u + 1, v) + top]), _f(_or));
  c.drawPath(_poly([p(u, v + 1) + lift, p(u + 1, v + 1) + lift, p(u + 1, v + 1) + top, p(u, v + 1) + top]), _f(_rd));
  c.drawPath(_poly([p(u, v) + top, p(u + 1, v) + top, p(u + 1, v + 1) + top, p(u, v + 1) + top]), _f(_ye));
}

Offset _hangA(_S st) => Offset((st.x - .5) * 56, (st.y - .5) * 40);

void _hangStep(_S st, double dt) {
  if (st.b.isEmpty) st.b.addAll([0, 0]);
  final a = _hangA(st), m = a.distance;
  final th0 = m < 1 ? st.b[0] : math.pi / 2 - math.atan2(-a.dy, -a.dx);
  var dev = st.b[0] - th0;
  dev = math.atan2(math.sin(dev), math.cos(dev));
  st.b[1] += (-70 * math.min(1, m / 24) * math.sin(dev) - 2.4 * st.b[1]) * dt;
  st.b[0] += st.b[1] * dt;
}

void _hang(Canvas c, Size s, _S st) {
  for (var i = 0; i < 9; i++) {
    c.drawLine(Offset(i * 20.0 - 10, 0), Offset(i * 20.0 + 30, _ch), _s(_al(_or, .08), 6));
  }
  const nail = Offset(78, 34);
  final a = _hangA(st), th = st.b.isEmpty ? 0.0 : st.b[0];
  const r = Rect.fromLTWH(-28, -21, 56, 42);
  c.save();
  c.translate(nail.dx + 3, nail.dy + 5);
  c.rotate(th);
  c.translate(-a.dx, -a.dy);
  c.drawRect(r.inflate(3), _f(_al(_k0, .18)));
  c.restore();
  c.save();
  c.translate(nail.dx, nail.dy);
  c.rotate(th);
  c.translate(-a.dx, -a.dy);
  c.drawRect(r.inflate(3), _f(_wh));
  _vgrad(c, r, _pk, _or);
  c.drawCircle(const Offset(10, -4), 9, _f(_ye));
  c.drawPath(
      Path()
        ..moveTo(-32, 24)
        ..quadraticBezierTo(-10, -2, 14, 14)
        ..quadraticBezierTo(24, 8, 32, 12)
        ..lineTo(32, 24)
        ..close(),
      _f(_vi));
  c.restore();
  c.drawCircle(nail + const Offset(1, 2), 5.5, _f(_al(_k0, .25)));
  c.drawCircle(nail, 5.5, _f(_rd));
  c.drawCircle(nail + const Offset(-1.6, -1.6), 1.8, _f(_al(_wh, .8)));
}

Path _ridge(double base, double amp, double f, double ph, double shift) {
  final p = Path()..moveTo(0, _ch);
  for (var i = 0; i <= 26; i++) {
    final x = i * 6.0, u = (x + shift) * f;
    p.lineTo(x, base - amp * (.55 + .3 * math.sin(u + ph) + .15 * math.sin(u * 2.7 + ph * 2)));
  }
  return p
    ..lineTo(_cw, _ch)
    ..close();
}

void _valley(Canvas c, Size s, _S st) {
  _vgrad(c, Offset.zero & s, _mix(_vi, _bl, .3), _pk);
  c.drawCircle(const Offset(118, 30), 13, _f(_ye));
  final px = (st.x - .5);
  final depth = st.y, sc = .35 + .9 * depth, bx = 78 + px * 110, by = 30 + depth * 42;
  void balloon() {
    final r = 12 * sc;
    c.drawLine(Offset(bx - r * .6, by + r * .7), Offset(bx - r * .3, by + r * 1.55), _s(_k0, .8));
    c.drawLine(Offset(bx + r * .6, by + r * .7), Offset(bx + r * .3, by + r * 1.55), _s(_k0, .8));
    c.drawRect(Rect.fromLTRB(bx - r * .35, by + r * 1.5, bx + r * .35, by + r * 1.95), _f(_or));
    c.drawOval(Rect.fromCenter(center: Offset(bx, by), width: r * 2, height: r * 2.2), _f(_ye));
    c.drawOval(Rect.fromCenter(center: Offset(bx, by), width: r * .9, height: r * 2.2), _f(_rd));
    if (depth < .5) c.drawOval(Rect.fromCenter(center: Offset(bx, by + r * .4), width: r * 2.4, height: r * 3.2), _f(_al(_mix(_vi, _cr, .4), (.5 - depth) * 1.1)));
  }

  c.drawPath(_ridge(66, 26, .045, 1, px * 8), _f(_mix(_vi, _cr, .45)));
  if (depth < .34) balloon();
  c.drawPath(_ridge(84, 26, .06, 3, px * 22), _f(_vi));
  if (depth >= .34 && depth < .67) balloon();
  c.drawPath(_ridge(104, 22, .08, 5, px * 46), _f(_bl));
  if (depth >= .67) balloon();
}

void _treasure(Canvas c, Size s, _S st) {
  _bg(c, s, _cy);
  for (var i = 0; i < 9; i++) {
    final at = Offset(_rn(i) * _cw, _rn(i + 30) * _ch), w = math.sin(st.t * 2 + i) * 2;
    c.drawArc(Rect.fromCenter(center: at + Offset(w, 0), width: 10, height: 6), math.pi * 1.1, math.pi * .8, false, _s(_al(_wh, .7), 1.4));
  }
  final isl = Path();
  for (var i = 0; i <= 40; i++) {
    final a = i / 40 * 2 * math.pi, r = 1 + .14 * math.sin(a * 3 + 1) + .08 * math.sin(a * 5);
    final q = const Offset(78, 60) + Offset(math.cos(a) * 46 * r, math.sin(a) * 32 * r);
    i == 0 ? isl.moveTo(q.dx, q.dy) : isl.lineTo(q.dx, q.dy);
  }
  isl.close();
  c.drawPath(isl, _f(_ye));
  c.drawOval(Rect.fromCenter(center: const Offset(84, 52), width: 46, height: 26), _f(_li));
  c.drawPath(_ngon(const Offset(92, 48), 9, 3, 0), _f(_mix(_li, _k0, .35)));
  const x0 = Offset(44, 80);
  c.drawLine(x0 + const Offset(-6, -6), x0 + const Offset(6, 6), _s(_rd, 3.5));
  c.drawLine(x0 + const Offset(6, -6), x0 + const Offset(-6, 6), _s(_rd, 3.5));
  final ship = Offset(12 + st.x * 132, 12 + st.y * 92);
  final route = st.trail.length > 2 ? st.trail : [x0, Offset((x0.dx + ship.dx) / 2, x0.dy - 26), ship];
  _dash(c, route, _s(_k0, 2), 4, 4);
  c.drawPath(
      Path()
        ..moveTo(ship.dx - 11, ship.dy)
        ..lineTo(ship.dx + 11, ship.dy)
        ..lineTo(ship.dx + 7, ship.dy + 6)
        ..lineTo(ship.dx - 7, ship.dy + 6)
        ..close(),
      _f(_rd));
  c.drawLine(ship, ship + const Offset(0, -17), _s(_k0, 1.5));
  c.drawPath(_poly([ship + const Offset(1, -16), ship + const Offset(1, -2), ship + const Offset(11, -3)]), _f(_wh));
  c.drawPath(_poly([ship + const Offset(-1, -14), ship + const Offset(-1, -2), ship + const Offset(-8, -3)]), _f(_cr));
}

void _orbit(Canvas c, Size s, _S st) {
  for (var i = 0; i < 26; i++) {
    c.drawCircle(Offset(_rn(i) * _cw, _rn(i + 50) * _ch), .6 + _rn(i + 9), _f(_al(_cr, .35 + .3 * math.sin(st.t * 3 + i))));
  }
  const cc = Offset(78, 58);
  final r = 22 + st.x * 40, th = st.a + st.t * .7;
  Offset at(double a) => cc + Offset(math.cos(a) * r, math.sin(a) * r * .45);
  c.drawOval(Rect.fromCenter(center: cc, width: r * 2, height: r * .9), _s(_al(_cr, .22), 1));
  void planet() {
    for (var k = 1; k <= 12; k++) {
      c.drawCircle(at(th - k * .07), 4.5 * (1 - k / 13) + .5, _f(_al(_cy, .5 * (1 - k / 13))));
    }
    final p = at(th), z = math.sin(th), pr = 6.5 + 2.5 * z;
    c.drawCircle(p, pr, _f(_cy));
    c.drawCircle(p + Offset(-pr * .3, -pr * .3), pr * .3, _f(_al(_wh, .5)));
    c.drawOval(Rect.fromCenter(center: p, width: pr * 3.4, height: pr * .9), _s(_pk, 2));
  }

  if (math.sin(th) < 0) planet();
  for (var i = 0; i < 10; i++) {
    final a = i / 10 * 2 * math.pi + st.t * .5;
    c.drawLine(cc + _pol(a, 14), cc + _pol(a, 19 + 2 * math.sin(st.t * 4 + i)), _s(_or, 2.4));
  }
  c.drawCircle(cc, 11, _f(_ye));
  c.drawCircle(cc, 3, _f(_wh));
  if (math.sin(th) >= 0) planet();
}

void _poolStep(_S st, double dt) {
  if (st.b.isEmpty) st.b.addAll([100, 64, 0, 0, -9]);
  final b = st.b;
  if (st.down && st.p != null) {
    b[0] = _cl(st.p!.dx, 16, 140);
    b[1] = _cl(st.p!.dy, 16, 100);
    b[2] = b[3] = 0;
    return;
  }
  if (b[4] != st.rel) {
    b[4] = st.rel;
    b[2] = st.vel.dx * .7;
    b[3] = st.vel.dy * .7;
  }
  b[0] += b[2] * dt;
  b[1] += b[3] * dt;
  final k = math.pow(.3, dt).toDouble();
  b[2] *= k;
  b[3] *= k;
  if (b[0] < 16 || b[0] > 140) {
    b[0] = _cl(b[0], 16, 140);
    b[2] = -b[2] * .85;
  }
  if (b[1] < 16 || b[1] > 100) {
    b[1] = _cl(b[1], 16, 100);
    b[3] = -b[3] * .85;
  }
}

void _pool(Canvas c, Size s, _S st) {
  _bg(c, s, _or);
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(9, 9, 138, 98), const Radius.circular(4)), _f(_bl));
  for (final q in const [Offset(10, 10), Offset(78, 8), Offset(146, 10), Offset(10, 106), Offset(78, 108), Offset(146, 106)]) {
    c.drawCircle(q, 6, _f(_k0));
  }
  for (var i = 0; i < 5; i++) {
    c.drawCircle(Offset(26 + i * 4.0, 40 + (i.isEven ? 0 : 3.0)), 1.2, _f(_al(_wh, .5)));
  }
  final bp = st.b.isEmpty ? const Offset(100, 64) : Offset(st.b[0], st.b[1]);
  for (final (q, col) in [(const Offset(40, 70), _ye), (const Offset(48, 64), _rd), (const Offset(48, 76), _li), (const Offset(56, 70), _wh)]) {
    c.drawCircle(q + const Offset(1.5, 2), 6, _f(_al(_k0, .35)));
    c.drawCircle(q, 6, _f(col));
  }
  if (st.b.length > 3) {
    final v = Offset(st.b[2], st.b[3]);
    if (v.distance > 20) {
      for (var k = 1; k <= 4; k++) {
        c.drawCircle(bp - v * (k * .02), 8 - k * 1.2, _f(_al(_pk, .25 - k * .05)));
      }
    }
  }
  c.drawCircle(bp + const Offset(2, 3), 8.5, _f(_al(_k0, .4)));
  c.drawCircle(bp, 8.5, _f(_pk));
  c.drawCircle(bp, 3.5, _f(_wh));
  c.drawCircle(bp + const Offset(-3, -3), 1.8, _f(_al(_wh, .7)));
}

void _aquarium(Canvas c, Size s, _S st) {
  _vgrad(c, Offset.zero & s, _cy, _mix(_bl, _k0, .35));
  for (var i = 0; i < 3; i++) {
    final x = 20.0 + i * 50 + math.sin(st.t * .5 + i) * 6;
    c.drawPath(_poly([Offset(x, 0), Offset(x + 14, 0), Offset(x + 34, _ch), Offset(x + 12, _ch)]), _f(_al(_wh, .07)));
  }
  c.drawRect(const Rect.fromLTWH(0, 104, _cw, 12), _f(_mix(_ye, _or, .3)));
  for (var i = 0; i < 4; i++) {
    final x = 14.0 + i * 40, sw = math.sin(st.t * 1.6 + i) * 5;
    c.drawPath(
        Path()
          ..moveTo(x, 106)
          ..quadraticBezierTo(x + sw, 92, x - sw * .5, 80 - (i % 2) * 8),
        _s(_li, 4));
  }
  if (st.b.isEmpty) st.b.add(1);
  final tr = st.trail;
  if (st.down && tr.length > 1) {
    final dx = tr.last.dx - tr[tr.length - 2].dx;
    if (dx.abs() > .5) st.b[0] = dx.sign;
  }
  final face = st.b[0], d = st.y;
  final f = Offset(18 + st.x * 120, 16 + d * 78 + math.sin(st.t * 2) * 2);
  final col = _mix(_ye, _mix(_or, _rd, .5), d);
  for (var i = 0; i < 4; i++) {
    final u = _wr(st.t * .6 + i * .25);
    c.drawCircle(f + Offset(face * (14 + math.sin(u * 9 + i) * 2), -8 - u * 40), 1.5 + u * 2, _s(_al(_wh, .8 * (1 - u)), 1.2));
  }
  c.save();
  c.translate(f.dx, f.dy);
  c.scale(face, 1);
  final wag = math.sin(st.t * 8) * 2;
  c.drawPath(_poly([Offset(-10, 0), Offset(-19, -7 + wag), Offset(-19, 7 + wag)]), _f(_mix(col, _pk, .4)));
  c.drawOval(Rect.fromCenter(center: Offset.zero, width: 26, height: 15), _f(col));
  c.drawPath(_poly([const Offset(-4, -6), const Offset(4, -6), const Offset(-2, -11)]), _f(_mix(col, _pk, .4)));
  c.drawCircle(const Offset(6, -2), 2.6, _f(_wh));
  c.drawCircle(const Offset(6.8, -2), 1.3, _f(_k0));
  c.restore();
  c.drawRect(Offset.zero & s, Paint()..color = _al(_k0, d * .25));
}

void _marioStep(_S st, double dt) {
  if (st.b.isEmpty) st.b.addAll([78, 72, 0, 0]);
  final cb = Offset(20 + st.x * 116, 10 + st.y * 30), tx = cb.dx, ty = cb.dy + 54;
  st.b[2] += ((tx - st.b[0]) * 70 - st.b[2] * 4.5) * dt;
  st.b[3] += ((ty - st.b[1]) * 70 - st.b[3] * 4.5) * dt;
  st.b[0] += st.b[2] * dt;
  st.b[1] += st.b[3] * dt;
}

void _marionette(Canvas c, Size s, _S st) {
  c.drawRect(const Rect.fromLTWH(0, 104, _cw, 12), _f(_k1));
  final cb = Offset(20 + st.x * 116, 10 + st.y * 30);
  final px = st.b.isEmpty ? 78.0 : st.b[0], py = st.b.isEmpty ? 72.0 : st.b[1], vx = st.b.isEmpty ? 0.0 : st.b[2];
  final tilt = _cl((px - cb.dx) / 60, -.5, .5);
  final l = cb + _pol(math.pi + tilt, 18), r = cb + _pol(tilt, 18);
  final head = Offset(px, py - 24), hl = Offset(px - 14, py - 6 - tilt * 10), hr = Offset(px + 14, py - 6 + tilt * 10);
  c.drawOval(Rect.fromCenter(center: Offset(px, 106), width: 30 - (106 - py) * .15, height: 4), _f(_al(_k0, .6)));
  for (final (a, b) in [(l, hl), (r, hr), (cb, head - const Offset(0, 7))]) {
    c.drawLine(a, b, _s(_al(_cr, .55), .8));
  }
  final sw = _cl(-vx * .04, -8, 8);
  c.drawLine(Offset(px - 4, py + 4), Offset(px - 6 + sw, py + 22), _s(_vi, 4));
  c.drawLine(Offset(px + 4, py + 4), Offset(px + 6 + sw, py + 22), _s(_vi, 4));
  c.drawLine(Offset(px - 6, py - 12), hl, _s(_ye, 3.5));
  c.drawLine(Offset(px + 6, py - 12), hr, _s(_ye, 3.5));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(px - 8, py - 16, px + 8, py + 6), const Radius.circular(5)), _f(_vi));
  c.drawCircle(head, 7.5, _f(_cr));
  c.drawCircle(head + const Offset(-2.5, -1), 1.3, _f(_k0));
  c.drawCircle(head + const Offset(2.5, -1), 1.3, _f(_k0));
  c.drawCircle(head + const Offset(-4.5, 2.5), 1.6, _f(_al(_pk, .8)));
  c.drawCircle(head + const Offset(4.5, 2.5), 1.6, _f(_al(_pk, .8)));
  c.drawLine(l, r, _s(_or, 5));
  c.drawLine(cb + const Offset(0, -5), cb + const Offset(0, 5), _s(_or, 4));
}

void _viewfinder(Canvas c, Size s, _S st) {
  _vgrad(c, const Rect.fromLTWH(0, 0, _cw, 84), _or, _ye);
  c.drawCircle(const Offset(112, 60), 16, _f(_rd));
  c.drawPath(
      Path()
        ..moveTo(0, 80)
        ..quadraticBezierTo(50, 58, 96, 84)
        ..quadraticBezierTo(130, 70, _cw, 82)
        ..lineTo(_cw, _ch)
        ..lineTo(0, _ch)
        ..close(),
      _f(_vi));
  c.drawRect(const Rect.fromLTWH(0, 96, _cw, 20), _f(_bl));
  final g = _s(_al(_wh, .45), 1);
  for (var i = 1; i < 3; i++) {
    c.drawLine(Offset(_cw * i / 3, 0), Offset(_cw * i / 3, _ch), g);
    c.drawLine(Offset(0, _ch * i / 3), Offset(_cw, _ch * i / 3), g);
  }
  var at = Offset(10 + st.x * 136, 10 + st.y * 96);
  for (var i = 1; i < 3; i++) {
    for (var j = 1; j < 3; j++) {
      final q = Offset(_cw * i / 3, _ch * j / 3);
      if ((q - at).distance < 11) {
        at = q;
        c.drawCircle(q, 15, _s(_li, 2.5));
      }
    }
  }
  c.drawPath(_star(at + const Offset(2, 2), 12, math.sin(st.t) * .2), _f(_al(_k0, .3)));
  c.drawPath(_star(at, 12, math.sin(st.t) * .2), _f(_pk));
  final br = _s(_wh, 2.5);
  for (final (q, dx, dy) in const [(Offset(6, 6), 1.0, 1.0), (Offset(150, 6), -1.0, 1.0), (Offset(6, 110), 1.0, -1.0), (Offset(150, 110), -1.0, -1.0)]) {
    c.drawLine(q, q + Offset(dx * 12, 0), br);
    c.drawLine(q, q + Offset(0, dy * 12), br);
  }
  if (_wr(st.t) < .55) c.drawCircle(const Offset(20, 18), 3.5, _f(_rd));
}

void _tiltStep(_S st, double dt) {
  if (st.b.isEmpty) st.b.addAll([0, 0, 0, 0]);
  final b = st.b;
  b[2] += ((st.x - .5) * 7 - b[2] * .6) * dt;
  b[3] += ((st.y - .5) * 7 - b[3] * .6) * dt;
  b[0] += b[2] * dt;
  b[1] += b[3] * dt;
  for (final k in [0, 1]) {
    if (b[k].abs() > .86) {
      b[k] = b[k].sign * .86;
      b[k + 2] = -b[k + 2] * .5;
    }
  }
}

void _tilt(Canvas c, Size s, _S st) {
  _vgrad(c, Offset.zero & s, _k1, _k0);
  final tx = st.x - .5, ty = st.y - .5;
  const cc = Offset(78, 56);
  Offset p(double u, double v) {
    final z = -(u * tx + v * ty);
    return cc + Offset(u * (46 + v * 8), v * 26 - z * 22);
  }

  c.drawPath(_poly([p(-1, 1), p(1, 1), p(1, 1) + const Offset(0, 8), p(-1, 1) + const Offset(0, 8)]), _f(_mix(_li, _k0, .45)));
  c.drawPath(_poly([p(1, -1), p(1, 1), p(1, 1) + const Offset(0, 8), p(1, -1) + const Offset(0, 8)]), _f(_mix(_li, _k0, .6)));
  c.drawPath(_poly([p(-1, -1), p(1, -1), p(1, 1), p(-1, 1)]), _f(_li));
  for (var i = -1; i <= 1; i++) {
    c.drawLine(p(i * .5, -1), p(i * .5, 1), _s(_al(_k0, .12), 1));
    c.drawLine(p(-1, i * .5), p(1, i * .5), _s(_al(_k0, .12), 1));
  }
  final g = p(0, 0);
  c.drawOval(Rect.fromCenter(center: g, width: 18, height: 9), _s(_bl, 2));
  final u = st.b.isEmpty ? 0.0 : st.b[0], v = st.b.isEmpty ? 0.0 : st.b[1], m = p(u, v);
  c.drawOval(Rect.fromCenter(center: m + const Offset(3, 1), width: 14, height: 6), _f(_al(_k0, .3)));
  c.drawCircle(m + const Offset(0, -6), 7, _f(_ye));
  c.drawCircle(m + const Offset(-2.5, -8.5), 2.2, _f(_al(_wh, .85)));
}

void _claw(Canvas c, Size s, _S st) {
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(6, 6, 144, 104), const Radius.circular(8)), _f(_k1));
  for (var i = 0; i < 9; i++) {
    c.drawCircle(Offset(14 + i * 16.0, 8), 2.4, _f(_al(_ye, _wr(st.t * 2 + i * .3) < .5 ? 1 : .35)));
  }
  for (var i = 0; i < 12; i++) {
    final q = Offset(18 + (i % 6) * 18.0 + (i ~/ 6) * 9, 100 - (i ~/ 6) * 11.0);
    c.drawCircle(q, 8, _f(_pop[i % 8]));
    c.drawCircle(q + const Offset(-2.5, -2.5), 2, _f(_al(_wh, .45)));
  }
  c.drawRect(const Rect.fromLTWH(124, 78, 22, 30), _f(_k0));
  c.drawLine(const Offset(10, 18), const Offset(146, 18), _s(_cr, 3));
  final cx = 20 + st.x * 100, len = 10 + st.y * 52, sway = math.sin(st.t * 2.2) * 2;
  final tip = Offset(cx + sway, 18 + len);
  c.drawRect(Rect.fromCenter(center: Offset(cx, 18), width: 14, height: 6), _f(_or));
  c.drawLine(Offset(cx, 18), tip, _s(_cr, 1.4));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: tip + const Offset(0, 3), width: 14, height: 7), const Radius.circular(2)), _f(_ye));
  final toy = tip + const Offset(0, 17);
  c.drawCircle(toy, 9, _f(_wh));
  c.drawArc(Rect.fromCircle(center: toy, radius: 9), math.pi, math.pi, true, _f(_pk));
  for (final d in [-1.0, 1.0]) {
    c.drawPath(
        Path()
          ..moveTo(tip.dx + d * 5, tip.dy + 5)
          ..quadraticBezierTo(tip.dx + d * 15, tip.dy + 14, tip.dx + d * 9, tip.dy + 26),
        _s(_ye, 3));
  }
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(6, 6, 144, 104), const Radius.circular(8)), _s(_pk, 4));
}

Path _card(Offset c0, double sc) =>
    _poly([c0 + Offset(-28, -16) * sc, c0 + Offset(36, -16) * sc, c0 + Offset(28, 16) * sc, c0 + Offset(-36, 16) * sc]);

void _cardLift(Canvas c, Size s, _S st) {
  c.drawOval(Rect.fromCenter(center: const Offset(78, 82), width: 150, height: 44), _f(_k1));
  const base = Offset(78, 80);
  for (var i = 2; i >= 0; i--) {
    c.drawPath(_card(base + Offset(i * -3.0, i * 4.0), 1), _f([_cy, _vi, _li][i]));
  }
  final z = 1 - st.y, dx = (st.x - .5) * 70, top = base + Offset(dx, -z * 42 - 4), sc = 1 + z * .3;
  for (var k = 0; k < 3; k++) {
    c.drawPath(_card(base + Offset(dx + z * 6, 2 + z * 4), 1 + z * (.1 + k * .08)), _f(_al(_k0, .18 - k * .04)));
  }
  final card = _card(top, sc);
  c.drawPath(card, _f(_or));
  c.save();
  c.clipPath(card);
  c.drawCircle(top + Offset(10, -2) * sc, 8 * sc, _f(_ye));
  c.drawPath(_ngon(top + Offset(-10, 6) * sc, 14 * sc, 3, 0), _f(_rd));
  c.restore();
  c.drawPath(card, _s(_wh, 2));
}

Path _bird(Offset o, double k) => Path()
  ..addOval(Rect.fromCenter(center: o, width: 26 * k, height: 13 * k))
  ..addPolygon([o + Offset(-4, -2) * k, o + Offset(4, -2) * k, o + Offset(-6, -16) * k], true)
  ..addPolygon([o + Offset(12, -3) * k, o + Offset(19, 0) * k, o + Offset(12, 2) * k], true)
  ..addPolygon([o + Offset(-11, -1) * k, o + Offset(-20, -7) * k, o + Offset(-18, 3) * k], true)
  ..addOval(Rect.fromCenter(center: o + Offset(9, -6) * k, width: 9 * k, height: 9 * k));

void _shadow(Canvas c, Size s, _S st) {
  c.drawCircle(const Offset(78, 128), 70, Paint()..shader = ui.Gradient.radial(const Offset(78, 128), 70, [_al(_ye, .9), _al(_ye, 0)]));
  const lamp = Offset(78, 150);
  final op = Offset(26 + st.x * 104, 84), k = 1.15 + st.y * 1.05;
  final sh = lamp + (op - lamp) * k;
  c.drawPath(_bird(sh, k), _f(_al(_k0, .82)));
  c.drawLine(op + const Offset(0, 4), Offset(op.dx, _ch), _s(_k1, 2));
  c.drawPath(_bird(op, 1), _f(_or));
  c.drawCircle(op + const Offset(10, -7), 1.4, _f(_k0));
  c.drawCircle(const Offset(78, 118), 9, _f(_wh));
}

void _pinwheel(Canvas c, Size s, _S st) {
  _bg(c, s, _k1);
  const cc = Offset(78, 58);
  final a = cc + Offset((st.x - .5) * 70, (st.y - .5) * 54), rad = (cc - a).distance;
  if (rad > 2) {
    final circ = [for (var i = 0; i <= 48; i++) a + _pol(i / 48 * 2 * math.pi, rad)];
    _dash(c, circ, _s(_al(_cr, .3), 1.2), 3, 4);
  }
  final th = st.t * 2.2 + st.a;
  c.save();
  c.translate(a.dx, a.dy);
  c.rotate(th);
  c.translate(-a.dx, -a.dy);
  for (var i = 0; i < 4; i++) {
    final f = i * math.pi / 2;
    c.drawPath(_poly([cc, cc + _pol(f, 30), cc + _pol(f + 1.0, 17)]), _f([_pk, _ye, _cy, _li][i]));
    c.drawPath(_poly([cc, cc + _pol(f + 1.0, 17), cc + _pol(f + math.pi / 2, 8)]), _f(_mix([_pk, _ye, _cy, _li][i], _k0, .3)));
  }
  c.drawCircle(cc, 3, _f(_wh));
  c.restore();
  c.drawCircle(a, 6, _s(_rd, 2.5));
  c.drawCircle(a, 2.5, _f(_wh));
}

List<Offset> _defRoute() => [for (var i = 0; i <= 20; i++) Offset(14 + i * 6.4, 70 - 30 * math.sin(i / 20 * math.pi * 1.5))];

void _snail(Canvas c, Size s, _S st) {
  final leaf = Path()
    ..moveTo(4, 110)
    ..quadraticBezierTo(10, 10, 152, 6)
    ..quadraticBezierTo(150, 100, 4, 110)
    ..close();
  c.drawPath(leaf, _f(_mix(_li, _k0, .5)));
  c.drawLine(const Offset(4, 110), const Offset(150, 8), _s(_al(_li, .6), 2));
  for (var i = 1; i < 6; i++) {
    final q = Offset(4 + i * 24.0, 110 - i * 16.9);
    c.drawLine(q, q + Offset(-6, -18 + i * 1.0), _s(_al(_li, .35), 1.2));
    c.drawLine(q, q + const Offset(16, 6), _s(_al(_li, .35), 1.2));
  }
  final route = st.trail.length > 3 ? st.trail : _defRoute();
  final len = _plen(route), d = st.down ? len : math.min(len, (st.t - st.rel).clamp(0.0, 999.0) * 30 % (len + 40));
  _dash(c, route, _s(_al(_cr, .45), 1.4), 2, 4);
  final done = _sub(route, d);
  _line(c, done, _s(_al(_cy, .45), 7));
  _line(c, done, _s(_al(_wh, .8), 1.5));
  final (p, ang) = _at(route, math.min(d, len));
  c.save();
  c.translate(p.dx, p.dy);
  c.rotate(ang);
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-12, -2, 24, 7), const Radius.circular(4)), _f(_ye));
  c.drawLine(const Offset(9, -1), const Offset(13, -8), _s(_ye, 1.5));
  c.drawLine(const Offset(7, -1), const Offset(9, -9), _s(_ye, 1.5));
  c.drawCircle(const Offset(13, -8), 1.6, _f(_k0));
  c.drawCircle(const Offset(-2, -6), 8, _f(_pk));
  final sp = Path();
  for (var i = 0; i <= 24; i++) {
    final q = const Offset(-2, -6) + _pol(i * .5, 6.5 * (1 - i / 26));
    i == 0 ? sp.moveTo(q.dx, q.dy) : sp.lineTo(q.dx, q.dy);
  }
  c.drawPath(sp, _s(_wh, 1.4));
  c.restore();
}

void _tunnel(Canvas c, Size s, _S st) {
  const vp = Offset(78, 50);
  final od = st.y * st.y;
  var drawn = false;
  void obj() {
    final at = vp + Offset((st.x - .5) * 120, 22) * od, r = 3 + od * 24;
    c.drawCircle(at, r * 1.5, _f(_al(_ye, .25)));
    c.drawCircle(at, r, _f(_ye));
    c.drawCircle(at, r * .45, _s(_wh, 1 + r * .08));
    drawn = true;
  }

  final rings = [for (var i = 0; i < 9; i++) _wr(i / 9 + st.t * .12)]..sort();
  for (final z in rings) {
    final zz = z * z;
    if (!drawn && zz > od) obj();
    final h = 3 + zz * 100, col = [_pk, _cy, _vi][(z * 9 + st.t * 1.08).floor() % 3];
    final r = Rect.fromCenter(center: vp + Offset(0, zz * 10), width: h * 1.5, height: h);
    c.drawRect(r, _s(_al(col, .22 * z + .05), 6));
    c.drawRect(r, _s(_al(col, z + .1), 1.4));
  }
  if (!drawn) obj();
}

void _chessStep(_S st, double dt) {
  if (st.b.isEmpty) st.b.addAll([0, 0]);
  final tx = 18 + (st.x * 5.999).floor() * 20.0 + 10, ty = 8 + (st.y * 4.999).floor() * 20.0 + 10;
  final k = 1 - math.pow(.0005, dt).toDouble();
  st.b[0] = st.b[0] == 0 ? tx : st.b[0] + (tx - st.b[0]) * k;
  st.b[1] = st.b[1] == 0 ? ty : st.b[1] + (ty - st.b[1]) * k;
}

void _chess(Canvas c, Size s, _S st) {
  for (var i = 0; i < 6; i++) {
    for (var j = 0; j < 5; j++) {
      c.drawRect(Rect.fromLTWH(18 + i * 20.0, 8 + j * 20.0, 20, 20), _f((i + j).isEven ? _bl : _mix(_cr, _wh, .5)));
    }
  }
  final ci = (st.x * 5.999).floor(), cj = (st.y * 4.999).floor();
  c.drawRect(Rect.fromLTWH(18 + ci * 20.0, 8 + cj * 20.0, 20, 20), _f(_ye));
  c.drawRect(const Rect.fromLTWH(18, 8, 120, 100), _s(_k0, 2));
  if (st.down) c.drawCircle(Offset(18 + st.x * 120, 8 + st.y * 100), 9, _s(_al(_k0, .6), 1.5));
  final p = st.b.isEmpty ? Offset(28 + ci * 20.0, 18 + cj * 20.0) : Offset(st.b[0], st.b[1]);
  final hop = st.down ? 4.0 : 0.0;
  c.drawOval(Rect.fromCenter(center: p + const Offset(0, 7), width: 15, height: 5), _f(_al(_k0, .35)));
  c.drawPath(_poly([p + Offset(-7, 7 - hop), p + Offset(7, 7 - hop), p + Offset(3, -2 - hop), p + Offset(-3, -2 - hop)]), _f(_pk));
  c.drawCircle(p + Offset(0, -6 - hop), 5, _f(_pk));
  c.drawCircle(p + Offset(-1.5, -7.5 - hop), 1.5, _f(_al(_wh, .7)));
}

void _radar(Canvas c, Size s, _S st) {
  const cc = Offset(78, 58);
  for (final r in [17.0, 34.0, 51.0]) {
    c.drawCircle(cc, r, _s(_al(_li, .35), 1));
  }
  c.drawLine(const Offset(27, 58), const Offset(129, 58), _s(_al(_li, .25), 1));
  c.drawLine(const Offset(78, 7), const Offset(78, 109), _s(_al(_li, .25), 1));
  final sw = st.t * 2.2;
  for (var k = 0; k < 14; k++) {
    c.drawArc(Rect.fromCircle(center: cc, radius: 51), sw - (k + 1) * .07, .07, true, _f(_al(_li, .32 * (1 - k / 14))));
  }
  c.drawLine(cc, cc + _pol(sw, 51), _s(_li, 2));
  var b = Offset((st.x - .5) * 100, (st.y - .5) * 100);
  if (b.distance > 46) b = b / b.distance * 46;
  for (var i = 0; i < 3; i++) {
    final e = Offset((_rn(i + 3) - .5) * 70, (_rn(i + 7) - .5) * 70);
    c.drawCircle(cc + e, 1.6, _f(_al(_li, .35)));
  }
  final since = _wr((sw - b.direction) / (2 * math.pi)) * 2 * math.pi, glow = math.exp(-since * .9);
  c.drawCircle(cc + b, 10, _f(_al(_pk, .35 * glow + .05)));
  c.drawCircle(cc + b, 4, _f(_mix(_mix(_pk, _k1, .6), _pk, glow)));
  c.drawCircle(cc + b, 7 + (1 - glow) * 8, _s(_al(_pk, glow * .8), 1.2));
}

int _tile(int i, int j) {
  final river = (j - (4 + 2.2 * math.sin(i * .6 + .5))).abs() < .8;
  if (river) return 1;
  if (i == 6) return 2;
  return _rn(i * 13 + j * 7) > .82 ? 3 : 0;
}

void _pixelStep(_S st, double dt) {
  if (st.b.isEmpty) st.b.addAll([-1, -1]);
  final hx = (st.x * 11).roundToDouble(), hy = (st.y * 8).roundToDouble();
  if (st.b[0] != hx || st.b[1] != hy) {
    if (st.b[0] >= 0) st.b.addAll([st.b[0], st.b[1]]);
    st.b[0] = hx;
    st.b[1] = hy;
    if (st.b.length > 26) st.b.removeRange(2, 4);
  }
}

void _pixel(Canvas c, Size s, _S st) {
  const t = 13.0;
  for (var i = 0; i < 12; i++) {
    for (var j = 0; j < 9; j++) {
      final k = _tile(i, j), r = Rect.fromLTWH(i * t, j * t, t, t);
      switch (k) {
        case 1:
          c.drawRect(r, _f(_bl));
          if ((i + j + (st.t * 2).floor()).isEven) c.drawRect(Rect.fromLTWH(r.left + 3, r.top + 5, 5, 2), _f(_cy));
        case 2:
          c.drawRect(r, _f(_mix(_ye, _or, .2)));
        case 3:
          c.drawRect(r, _f(_li));
          c.drawRect(r.deflate(2), _f(_mix(_li, _k0, .45)));
          c.drawRect(Rect.fromLTWH(r.left + 5, r.top + 9, 3, 4), _f(_or));
        default:
          c.drawRect(r, _f((i + j).isEven ? _li : _mix(_li, _ye, .25)));
      }
    }
  }
  for (var k = 2; k + 1 < st.b.length; k += 2) {
    final q = Offset(st.b[k] * t, st.b[k + 1] * t);
    c.drawRect(Rect.fromLTWH(q.dx + 4, q.dy + 8, 2, 2), _f(_al(_wh, .8)));
    c.drawRect(Rect.fromLTWH(q.dx + 7, q.dy + 5, 2, 2), _f(_al(_wh, .8)));
  }
  final h = Offset((st.x * 11).roundToDouble() * t, (st.y * 8).roundToDouble() * t - (_wr(st.t * 2) < .5 ? 1 : 0));
  void px(int x, int y, Color col, [int w = 1, int hh = 1]) =>
      c.drawRect(Rect.fromLTWH(h.dx + 1.5 + x * 2.0, h.dy + y * 2.0, w * 2.0, hh * 2.0), _f(col));
  px(1, 0, _rd, 3);
  px(1, 1, _cr, 3, 2);
  px(2, 1, _k0);
  px(0, 3, _pk, 5, 2);
  px(1, 5, _k0);
  px(3, 5, _k0);
}
