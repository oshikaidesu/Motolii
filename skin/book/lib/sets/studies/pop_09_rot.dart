// Rotation / Spin: 20 pictures of why a maker turns something: to set an angle, to make it spin, to pick the axis it spins about.
part of 'pop_09.dart';

List<_Pan9> _rotPanels() => [
      _Pan9('Vinyl', 'instrument · crisp · top-down · flick · mechanism', _G.flick, _vinyl, a: .1, fric: .75),
      _Pan9('Pinwheel', 'toy · bold flat · 2D · rub · result', _G.rub, _pinwheel, b: .35, decay: .12, spd: (s) => s.b * 3),
      _Pan9('Sunflower', 'nature · bold flat · landscape · drag · result', _G.xy, _sunflower, a: .7),
      _Pan9('Axis cube', 'machine · crisp · pseudo 3D · drag · axis', _G.xy, _cube, a: .62, b: .7, spd: (s) => .25),
      _Pan9('Cake stand', 'food · bold flat · isometric · spin · result', _G.spin, _cake, a: .1),
      _Pan9('Spinning top', 'toy · bold flat · pseudo 3D · drag · speed+tilt', _G.xy, _top, a: .35, b: .6, spd: (s) => .3 + s.b * 3),
      _Pan9('Gyro rings', 'machine · neon wild · pseudo 3D · drag · 3 axes', _G.xy, _gyro, a: .1, b: .2, spd: (s) => .3),
      _Pan9('Weathervane', 'weather · bold flat · 2D · wind · angle', _G.wind, _vane, a: .0, b: .4, spd: (s) => 1),
      _Pan9('Ferris wheel', 'landscape · crisp · 2D · flick · result', _G.flick, _ferris, fric: .5),
      _Pan9('Galaxy', 'cosmic · neon wild · top-down · spin · twist', _G.spin, _galaxy, a: .0, b: .45, spd: (s) => .04),
      _Pan9('Owl head', 'creature · bold flat · pseudo 3D · drag · yaw+roll', _G.xy, _owl, a: .62, b: .5),
      _Pan9('Spirograph', 'machine · crisp light · 2D · drag · pattern', _G.xy, _spiro, a: .38, b: .7, spd: (s) => .12),
      _Pan9('Tornado', 'weather · bold flat · pseudo 3D · drag · speed+lean', _G.xy, _tornado, a: .5, b: .4, spd: (s) => .2 + s.a * 2.2),
      _Pan9('Gear train', 'machine · crisp · 2D · spin · direction', _G.spin, _gears, a: .0),
      _Pan9('Photo turn', 'sample image · crisp light · 2D · spin · result', _G.spin, _photo, a: .03),
      _Pan9('Coin flip', 'toy · bold flat · pseudo 3D · flick up · X axis', _G.flickY, _coin, a: .0, fric: .35),
      _Pan9('Globe', 'cosmic · crisp · pseudo 3D · drag · spin+tilt', _G.xy, _globe, a: .3, b: .62),
      _Pan9('Pirouette', 'character · bold flat · 2D · pinch · speed', _G.pinch, _skater, a: .7, spd: (s) => .25 + (1 - s.a) * 2.4),
      _Pan9('Screw in', 'material · bold flat · cutaway · drag · turns', _G.xy, _screw, a: .3),
      _Pan9('Kaleidoscope', 'toy · neon wild · 2D · spin · result', _G.spin, _kaleido, a: .0),
    ];

// 1. A record you throw: the label art turns, grooves catch the light, the arm rides the edge.
void _vinyl(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k1);
  const o = Offset(66, 58);
  c.drawCircle(o + const Offset(2, 3), 50, _f(const Color(0x55000000)));
  c.drawCircle(o, 50, _f(_k0));
  for (var r = 22.0; r < 49; r += 4) {
    c.drawCircle(o, r, _s(const Color(0x14FFFFFF), 1));
  }
  final ang = st.a * _tau;
  final sheen = Path()
    ..moveTo(o.dx, o.dy)
    ..arcTo(Rect.fromCircle(center: o, radius: 49), ang + .3, .5, false)
    ..close();
  c.drawPath(sheen, _f(const Color(0x22FFFFFF)));
  c.drawPath(
      Path()
        ..moveTo(o.dx, o.dy)
        ..arcTo(Rect.fromCircle(center: o, radius: 49), ang + math.pi + .3, .5, false)
        ..close(),
      _f(const Color(0x18FFFFFF)));
  c.drawCircle(o, 18, _f(_or));
  c.drawPath(
      Path()
        ..moveTo(o.dx, o.dy)
        ..arcTo(Rect.fromCircle(center: o, radius: 18), ang, 1.3, false)
        ..close(),
      _f(_ye));
  c.drawCircle(_pol(o, 11, ang + 3.6), 3, _f(_pk));
  c.drawCircle(o, 2.5, _f(_cr));
  final spd = st.w.abs();
  if (spd > .05) {
    c.drawArc(Rect.fromCircle(center: o, radius: 54), ang - .2, -_cl(spd * .8, 0, 2.5) * st.w.sign, false, _s(_al(_cy, .8), 2));
  }
  const piv = Offset(140, 16);
  c.drawCircle(piv, 7, _f(_cr));
  c.drawLine(piv, const Offset(132, 70), _s(_cr, 3));
  c.drawLine(const Offset(132, 70), const Offset(114, 80), _s(_cr, 3));
  _rr(c, Rect.fromCenter(center: const Offset(112, 81), width: 10, height: 7), 2, _f(_or));
}

// 2. Rub to blow: the harder the wind, the faster the blades.
void _pinwheel(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cr);
  const o = Offset(78, 48);
  c.drawLine(o, const Offset(78, 116), _s(_k0, 4));
  const cols = [_or, _pk, _cy, _ye];
  for (var i = 0; i < 4; i++) {
    final a = st.ph * _tau + i * math.pi / 2;
    c.drawPath(_poly([o, _pol(o, 36, a), _pol(o, 30, a + .75)]), _f(cols[i]));
    c.drawPath(_poly([o, _pol(o, 30, a + .75), _pol(o, 16, a + .9)]), _f(_al(_k0, .18)));
  }
  c.drawCircle(o, 4, _f(_k0));
  if (st.down && st.at != null) {
    for (var i = 0; i < 3; i++) {
      final y = st.at!.dy - 8 + i * 8, x = st.at!.dx + 6 + ((t * 80 + i * 13) % 20);
      c.drawLine(Offset(x, y), Offset(x + 12, y), _s(_al(_bl, .6), 2));
    }
  }
}

// 3. Drag the sun along the sky; the flower head turns to face it.
void _sunflower(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _bl);
  c.drawRect(const Rect.fromLTWH(0, 98, 156, 18), _f(_li));
  final sa = math.pi + st.a * math.pi, sun = _pol(const Offset(78, 102), 66, sa);
  for (var i = 0; i < 8; i++) {
    c.drawLine(_pol(sun, 12, i * math.pi / 4 + t * .4), _pol(sun, 17, i * math.pi / 4 + t * .4), _s(_ye, 2.5));
  }
  c.drawCircle(sun, 9, _f(_ye));
  const head = Offset(78, 64);
  final look = (sun - head).direction, off = Offset(math.cos(look), math.sin(look));
  final stem = Path()
    ..moveTo(78, 116)
    ..quadraticBezierTo(78 - off.dx * 6, 90, head.dx + off.dx * 2, head.dy + 6);
  c.drawPath(stem, _s(const Color(0xFF3E8E3A), 4));
  c.drawPath(stem, _s(_li, 2));
  final face = head + off * 6;
  for (var i = 0; i < 12; i++) {
    final a = i * _tau / 12;
    final tip = face + Offset(math.cos(a) * 22 * (1 - .35 * off.dx.abs()), math.sin(a) * 22 * (1 - .35 * off.dy.abs()));
    c.drawLine(face, tip, _s(_ye, 6));
  }
  c.drawOval(Rect.fromCenter(center: face, width: 22 * (1 - .3 * off.dx.abs()), height: 22 * (1 - .3 * off.dy.abs())),
      _f(const Color(0xFF6B3A1E)));
  c.drawCircle(face + off * 3, 3, _f(_or));
}

class _V3 {
  const _V3(this.x, this.y, this.z);
  final double x, y, z;
  _V3 operator +(_V3 o) => _V3(x + o.x, y + o.y, z + o.z);
  _V3 operator *(double k) => _V3(x * k, y * k, z * k);
  double dot(_V3 o) => x * o.x + y * o.y + z * o.z;
  _V3 cross(_V3 o) => _V3(y * o.z - z * o.y, z * o.x - x * o.z, x * o.y - y * o.x);
  _V3 rot(_V3 k, double a) => this * math.cos(a) + k.cross(this) * math.sin(a) + k * (k.dot(this) * (1 - math.cos(a)));
  _V3 view() {
    const p = .42;
    return _V3(x, y * math.cos(p) - z * math.sin(p), y * math.sin(p) + z * math.cos(p));
  }
}

// 4. Drag to aim the pink axis; the cube spins about it.
void _cube(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k0);
  final yaw = (st.a - .5) * math.pi, pit = (st.b - .5) * math.pi;
  final k = _V3(math.sin(yaw) * math.cos(pit), -math.cos(yaw) * math.cos(pit), math.sin(pit));
  final th = st.ph * _tau;
  Offset pr(_V3 v) {
    final w = v.view(), z = 1 / (1 - w.z * .12);
    return _ctr + Offset(w.x, w.y) * 27 * z;
  }

  final ax0 = pr(k * 2.2), ax1 = pr(k * -2.2);
  c.drawLine(ax1, ax0, _s(_al(_pk, .45), 3));
  const faces = [
    [_V3(0, 0, 1), _or],
    [_V3(0, 0, -1), _or],
    [_V3(1, 0, 0), _cy],
    [_V3(-1, 0, 0), _cy],
    [_V3(0, 1, 0), _ye],
    [_V3(0, -1, 0), _ye],
  ];
  for (final f in faces) {
    final n = f[0] as _V3, nr = n.rot(k, th).view();
    if (nr.z <= 0) continue;
    final u = n.x != 0 ? const _V3(0, 1, 0) : const _V3(1, 0, 0), v = n.cross(u);
    final pts = [for (final q in [(1, 1), (1, -1), (-1, -1), (-1, 1)]) pr((n + u * q.$1.toDouble() + v * q.$2.toDouble()).rot(k, th))];
    c.drawPath(_poly(pts), _f(Color.lerp(f[1] as Color, _k0, (1 - nr.z) * .45)!));
    c.drawPath(_poly(pts), _s(_k0, 1.5));
  }
  c.drawLine(pr(k * 1.75), ax0, _s(_pk, 3));
  c.drawCircle(ax0, 4, _f(_pk));
}

// 5. A cake on a turntable, iso: spin it and the strawberries come round to the front.
void _cake(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _pk);
  const o = Offset(78, 66), rx = 50.0, ry = 20.0;
  c.drawOval(Rect.fromCenter(center: o + const Offset(0, 26), width: 120, height: 40), _f(_cr));
  c.drawOval(Rect.fromCenter(center: o + const Offset(0, 22), width: 120, height: 40), _f(_wh));
  final body = Rect.fromLTRB(o.dx - rx, o.dy - 6, o.dx + rx, o.dy + 18);
  c.drawRect(body, _f(_k1));
  c.drawOval(Rect.fromCenter(center: o + const Offset(0, 18), width: rx * 2, height: ry * 2), _f(_k1));
  c.drawRect(Rect.fromLTRB(o.dx - rx, o.dy + 4, o.dx + rx, o.dy + 8), _f(_cr));
  c.drawOval(Rect.fromCenter(center: o - const Offset(0, 6), width: rx * 2, height: ry * 2), _f(_cr));
  final berries = <(double, int)>[];
  for (var i = 0; i < 7; i++) {
    berries.add((st.a * _tau + i * _tau / 7, i));
  }
  berries.sort((p, q) => math.sin(p.$1).compareTo(math.sin(q.$1)));
  for (final (a, i) in berries) {
    final p = o + Offset(math.cos(a) * rx * .75, math.sin(a) * ry * .75 - 9), front = (math.sin(a) + 1) / 2;
    final col = i == 0 ? _ye : _rd;
    c.drawCircle(p, 5 + front * 2, _f(col));
    c.drawCircle(p + const Offset(-1.5, -2), 1.4, _f(_al(_wh, .8)));
  }
  final ca = st.a * _tau + 1.1, cp = o + Offset(math.cos(ca) * rx * .2, math.sin(ca) * ry * .2 - 7);
  c.drawLine(cp, cp - const Offset(0, 18), _s(_cy, 4));
  c.drawPath(Path()..addOval(Rect.fromCenter(center: cp - Offset(0, 23 + math.sin(t * 9) * .8), width: 6, height: 9)), _f(_or));
}

// 6. A top: drag right to spin it faster, up to make it lean; the lean precesses.
void _top(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _vi);
  const base = Offset(78, 98);
  c.drawOval(Rect.fromCenter(center: base + const Offset(0, 4), width: 70, height: 14), _f(_al(_k0, .35)));
  final lean = (1 - st.b) * .5, prec = t * (1.2 + st.a * 2);
  final tilt = lean * math.cos(prec);
  c.save();
  c.translate(base.dx, base.dy);
  c.rotate(tilt);
  final squash = .35 + .1 * math.sin(prec);
  final body = Path()
    ..moveTo(0, 0)
    ..lineTo(-34, -42)
    ..quadraticBezierTo(0, -42 - 34 * squash, 34, -42)
    ..close();
  c.drawPath(body, _f(_ye));
  c.save();
  c.clipPath(body);
  for (var i = 0; i < 6; i++) {
    final x = ((st.ph * 6 + i) % 6) / 6 * 2 - 1;
    final xx = math.sin(x * math.pi / 2) * 34;
    c.drawLine(Offset(xx * .1, 0), Offset(xx, -44), _s(i.isEven ? _rd : _or, 5));
  }
  c.restore();
  c.drawOval(Rect.fromCenter(center: const Offset(0, -42), width: 68, height: 68 * squash * .6), _f(_cr));
  c.drawLine(const Offset(0, -42), const Offset(0, -62), _s(_k0, 5));
  c.restore();
  final spd = .3 + st.a * 3;
  if (spd > 1.2) {
    for (var i = 0; i < 3; i++) {
      final a = -t * 3 - i * 2.1;
      c.drawArc(Rect.fromCenter(center: base + Offset(0, -30), width: 96, height: 30), a, .7, false, _s(_al(_cr, .7), 2));
    }
  }
}

// 7. Three rings, three axes: drag across turns the vertical ring, drag up turns the horizontal one, the flat ring runs.
void _gyro(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF0E0E12));
  const o = Offset(78, 58), r = 42.0;
  void ring(Rect rc, Color col) {
    c.drawOval(rc, _s(_al(col, .25), 7));
    c.drawOval(rc, _s(col, 2.4));
  }

  final cy = math.cos(st.a * _tau), cx = math.cos(st.b * _tau);
  ring(Rect.fromCenter(center: o, width: r * 2, height: r * 2), _cy);
  ring(Rect.fromCenter(center: o, width: (r - 7) * 2 * cy.abs() + 2, height: (r - 7) * 2), _pk);
  ring(Rect.fromCenter(center: o, width: (r - 14) * 2, height: (r - 14) * 2 * cx.abs() + 2), _li);
  c.drawCircle(_pol(o, r, st.ph * _tau), 5, _f(_cy));
  c.drawCircle(o + Offset(0, -(r - 7)), 4, _f(_pk));
  c.drawCircle(o + Offset((r - 14) * (cx >= 0 ? 1 : -1), 0), 4, _f(_li));
  c.drawCircle(o, 7, _f(_ye));
}

// 8. Drag the wind: streaks blow along it and the rooster swings to point into it.
void _vane(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cy);
  final wa = st.a * _tau, dir = Offset(math.cos(wa), math.sin(wa));
  for (var i = 0; i < 9; i++) {
    final along = ((t * (40 + st.b * 120) + _rn(i) * 300) % 220) - 30;
    final across = (_rn(i + 20) - .5) * 150;
    final p = _ctr + dir * (along - 110) + Offset(-dir.dy, dir.dx) * across;
    c.drawLine(p, p + dir * 16, _s(_al(_wh, .9), 2.5));
  }
  c.drawCircle(const Offset(30, 24), 10, _f(_wh));
  c.drawCircle(const Offset(42, 22), 13, _f(_wh));
  c.drawCircle(const Offset(54, 26), 9, _f(_wh));
  c.drawLine(const Offset(78, 116), const Offset(78, 58), _s(_k0, 4));
  c.save();
  c.translate(78, 58);
  c.rotate(wa + math.pi);
  c.drawLine(const Offset(-30, 0), const Offset(34, 0), _s(_k0, 3));
  c.drawPath(_poly(const [Offset(36, 0), Offset(26, -7), Offset(26, 7)]), _f(_k0));
  c.drawPath(_poly(const [Offset(-30, 0), Offset(-40, -10), Offset(-36, 0), Offset(-40, 10)]), _f(_k0));
  c.restore();
  c.save();
  c.translate(78, 52);
  if (math.cos(wa + math.pi) < 0) c.scale(-1, 1);
  c.drawOval(const Rect.fromLTWH(-12, -18, 26, 14), _f(_rd));
  c.drawCircle(const Offset(14, -18), 6, _f(_rd));
  c.drawPath(_poly(const [Offset(19, -19), Offset(26, -17), Offset(19, -15)]), _f(_ye));
  c.drawPath(_poly(const [Offset(-12, -12), Offset(-22, -26), Offset(-6, -16)]), _f(_or));
  c.drawLine(const Offset(0, -4), const Offset(0, 6), _s(_k0, 2));
  c.restore();
  c.drawCircle(const Offset(78, 58), 4, _f(_ye));
}

// 9. Throw the wheel round; the cabins stay upright and the lights trail the speed.
void _ferris(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _or);
  c.drawCircle(const Offset(128, 26), 12, _f(_ye));
  c.drawRect(const Rect.fromLTWH(0, 100, 156, 16), _f(_k0));
  const o = Offset(70, 56), r = 40.0;
  c.drawLine(o, const Offset(48, 104), _s(_k0, 4));
  c.drawLine(o, const Offset(92, 104), _s(_k0, 4));
  c.drawCircle(o, r, _s(_cr, 2.5));
  const cols = [_cy, _pk, _ye, _li, _vi, _rd, _cy, _pk];
  for (var i = 0; i < 8; i++) {
    final a = st.a * _tau + i * _tau / 8, p = _pol(o, r, a);
    c.drawLine(o, p, _s(_cr, 1.5));
    _rr(c, Rect.fromCenter(center: p + const Offset(0, 8), width: 12, height: 10), 3, _f(cols[i]));
    c.drawLine(p, p + const Offset(0, 3), _s(_k0, 1.5));
  }
  if (st.w.abs() > .08) {
    c.drawArc(Rect.fromCircle(center: o, radius: r + 4), st.a * _tau, -_cl(st.w * .9, -2.4, 2.4), false, _s(_al(_ye, .9), 3));
  }
  c.drawCircle(o, 5, _f(_k0));
}

// 10. Turn the galaxy; pull outward to wind the arms tighter.
void _galaxy(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF09090D));
  for (var i = 0; i < 24; i++) {
    c.drawCircle(Offset(_rn(i) * 156, _rn(i + 50) * 116), .8, _f(_al(_wh, .5)));
  }
  final twist = 1 + st.b * 7, rot = (st.a + st.ph) * _tau;
  const cols = [_pk, _vi, _cy];
  for (var arm = 0; arm < 2; arm++) {
    for (var i = 0; i < 70; i++) {
      final u = i / 70, rr = 4 + u * 62;
      final a = rot + arm * math.pi + u * twist + (_rn(i + arm * 99) - .5) * .5;
      final p = _ctr + Offset(math.cos(a) * rr, math.sin(a) * rr * .72);
      c.drawCircle(p, 1.4 + (1 - u) * 2.4, _f(_al(cols[(i + arm) % 3], .95 - u * .45)));
    }
  }
  c.drawCircle(_ctr, 10, _f(_al(_ye, .35)));
  c.drawCircle(_ctr, 5, _f(_wh));
}

// 11. The owl turns its head further than you would: across = yaw (past 90 you see the back), up = roll.
void _owl(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _bl);
  c.drawCircle(const Offset(132, 22), 10, _f(_cr));
  c.drawCircle(const Offset(127, 19), 9, _f(_bl));
  c.drawLine(const Offset(10, 108), const Offset(146, 108), _s(const Color(0xFF6B3A1E), 6));
  c.drawOval(const Rect.fromLTWH(54, 58, 48, 52), _f(_vi));
  c.drawOval(const Rect.fromLTWH(64, 70, 28, 34), _f(_al(_cr, .5)));
  final yaw = (st.a - .5) * 1.5 * math.pi, roll = (st.b - .5) * 1.1;
  c.save();
  c.translate(78, 50);
  c.rotate(roll);
  c.drawOval(const Rect.fromLTWH(-30, -26, 60, 46), _f(_vi));
  c.drawPath(_poly(const [Offset(-26, -18), Offset(-22, -34), Offset(-12, -22)]), _f(_vi));
  c.drawPath(_poly(const [Offset(26, -18), Offset(22, -34), Offset(12, -22)]), _f(_vi));
  final sx = math.sin(yaw), front = math.cos(yaw);
  if (front > 0) {
    final w = 44 * (.35 + .65 * front);
    c.drawOval(Rect.fromCenter(center: Offset(sx * 14, -2), width: w, height: 36), _f(_cr));
    for (final e in [-1.0, 1.0]) {
      final ex = sx * 18 + e * 10 * front;
      c.drawCircle(Offset(ex, -4), 8 * (.5 + .5 * front), _f(_ye));
      c.drawCircle(Offset(ex + sx * 2, -4), 4 * (.5 + .5 * front), _f(_k0));
    }
    c.drawPath(_poly([Offset(sx * 22, 2), Offset(sx * 22 - 4, 6), Offset(sx * 24, 12), Offset(sx * 22 + 4, 6)]), _f(_or));
  } else {
    for (var i = 0; i < 3; i++) {
      c.drawArc(Rect.fromCenter(center: Offset(-sx * 6, -6 + i * 8), width: 30, height: 10), .3, 2.5, false,
          _s(_al(_k0, .35), 2));
    }
  }
  c.restore();
}

// 12. Wheel within a wheel: across = the small gear's size, up = where the pen sits; the rosette is what turning draws.
void _spiro(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cr);
  const o = Offset(78, 58), bigR = 48.0;
  c.drawCircle(o, bigR, _s(_al(_k0, .2), 3));
  final r = bigR * (.18 + .5 * st.a), d = r * (.25 + .9 * st.b);
  final path = Path();
  const n = 900;
  for (var i = 0; i <= n; i++) {
    final th = i / n * _tau * 12;
    final p = o + Offset((bigR - r) * math.cos(th) + d * math.cos((bigR - r) / r * th),
        (bigR - r) * math.sin(th) - d * math.sin((bigR - r) / r * th));
    i == 0 ? path.moveTo(p.dx, p.dy) : path.lineTo(p.dx, p.dy);
  }
  c.drawPath(path, _s(_pk, 1.1));
  final th = st.ph * _tau * 3, gc = _pol(o, bigR - r, th), ga = -(bigR - r) / r * th;
  c.drawCircle(gc, r, _f(_al(_cy, .45)));
  c.drawCircle(gc, r, _s(_bl, 1.5));
  final pen = gc + Offset(math.cos(ga), math.sin(ga)) * d;
  c.drawLine(gc, pen, _s(_bl, 1.5));
  c.drawCircle(pen, 3.5, _f(_rd));
}

// 13. A twister: across = how fast it whirls, up = how far it leans; debris shows the speed.
void _tornado(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _vi);
  c.drawRect(const Rect.fromLTWH(0, 102, 156, 14), _f(_li));
  final lean = (st.b - .5) * 60;
  for (var i = 0; i < 9; i++) {
    final u = i / 8, y = 100 - u * 84, w = 10 + u * u * 80, x = 78 + lean * u * u + math.sin(t * 2 + u * 4) * 3;
    c.drawOval(Rect.fromCenter(center: Offset(x, y), width: w, height: 8 + u * 6), _s(_cr, 3));
  }
  for (var i = 0; i < 10; i++) {
    final u = .1 + _rn(i) * .8, y = 100 - u * 84, w = 10 + u * u * 80, x = 78 + lean * u * u;
    final a = st.ph * _tau * (1 + _rn(i + 7)) + i;
    final p = Offset(x + math.cos(a) * w * .55, y + math.sin(a) * (4 + u * 3));
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(a * 2);
    c.drawRect(const Rect.fromLTWH(-3, -2, 6, 4), _f(math.sin(a) > 0 ? _k0 : _al(_k0, .4)));
    c.restore();
  }
}

// 14. Turn the orange gear: its neighbours answer in the opposite direction, at their own speed.
void _gears(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k1);
  final a = st.a * _tau;
  const o1 = Offset(54, 62), o2 = Offset(54 + 30 + 19, 46), o3 = Offset(103 + 19 + 11, 82);
  void g(Offset o, double r, int n, double rot, Color col) {
    c.drawPath(_gear(o, r, n, rot), _f(col));
    c.drawCircle(o, r * .35, _f(_k1));
    c.drawLine(_pol(o, r * .35, rot), _pol(o, r * .7, rot), _s(_k1, 4));
  }

  g(o1, 34, 16, a, _or);
  g(o2, 22, 10, -a * 34 / 22 + .3, _cy);
  g(o3, 14, 7, a * 34 / 14 + .1, _ye);
  final dirA = st.w.sign >= 0 ? 1.0 : -1.0;
  c.drawArc(Rect.fromCircle(center: o1, radius: 40), -2.4, .9 * dirA, false, _s(_cr, 2));
}

// 15. The photo itself turns about its pivot; recent angles stay as ghosts while it moves.
void _photo(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cr);
  const rc = Rect.fromLTWH(-34, -24, 68, 48);
  final ang = st.a * _tau;
  for (var i = 5; i >= 1; i--) {
    final ga = ang - st.w * i * .06;
    if ((ga - ang).abs() < .01) continue;
    c.save();
    c.translate(_ctr.dx, _ctr.dy);
    c.rotate(ga);
    _rr(c, rc, 3, _s(_al([_pk, _vi, _cy, _li, _ye][i - 1], .9 - i * .12), 2));
    c.restore();
  }
  c.save();
  c.translate(_ctr.dx, _ctr.dy);
  c.rotate(ang);
  _rr(c, rc.inflate(3), 4, _f(_wh));
  _rr(c, rc.inflate(3), 4, _s(_al(_k0, .25), 1));
  _pic(c, rc);
  c.restore();
  c.drawCircle(_ctr, 4, _f(_pk));
  c.drawCircle(_ctr, 4, _s(_wh, 1.5));
}

// 16. Flick it up: the coin flips about its horizontal axis, sun side and moon side.
void _coin(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _pk);
  final a = st.a * _tau, h = math.cos(a), lift = (st.w.abs() * 10).clamp(0, 26).toDouble();
  final o = Offset(78, 56 - lift);
  c.drawOval(Rect.fromCenter(center: const Offset(78, 102), width: 60 - lift, height: 10), _f(_al(_k0, .25)));
  final rx = 34.0, ry = 34 * h.abs();
  c.drawOval(Rect.fromCenter(center: o + const Offset(0, 4), width: rx * 2, height: ry * 2 + 2), _f(const Color(0xFFB8541F)));
  final face = Rect.fromCenter(center: o, width: rx * 2, height: ry * 2 + .5);
  c.drawOval(face, _f(h >= 0 ? _ye : _or));
  c.drawOval(face.deflate(4), _s(_al(_wh, .6), 1.5));
  c.save();
  c.translate(o.dx, o.dy);
  c.scale(1, h.abs() + .01);
  if (h >= 0) {
    c.drawPath(_star(Offset.zero, 16, 0), _f(_or));
  } else {
    c.drawCircle(Offset.zero, 14, _f(_ye));
    c.drawCircle(const Offset(6, -4), 12, _f(_or));
  }
  c.restore();
}

// 17. A globe: across rolls it round, up tilts its axis.
void _globe(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k0);
  for (var i = 0; i < 18; i++) {
    c.drawCircle(Offset(_rn(i + 3) * 156, _rn(i + 33) * 116), .9, _f(_al(_cr, .6)));
  }
  const o = Offset(78, 58), r = 40.0;
  final tilt = (st.b - .5) * 1.4;
  c.drawCircle(o, r, _f(_bl));
  final lon0 = st.a * _tau * 2;
  for (var i = 0; i < 90; i++) {
    final cl = i % 5;
    final lon = cl * 1.3 + (_rn(i) - .5) * .9 + lon0, lat = (_rn(cl + 40) - .5) * 1.8 + (_rn(i + 9) - .5) * .7;
    final x = math.cos(lat) * math.sin(lon), y = -math.sin(lat), z = math.cos(lat) * math.cos(lon);
    if (z < 0) continue;
    final rx = x * math.cos(tilt) - y * math.sin(tilt), ry = x * math.sin(tilt) + y * math.cos(tilt);
    c.drawCircle(o + Offset(rx, ry) * r, 2.5 + z * 3, _f(_li));
  }
  c.drawCircle(o, r, _s(_cy, 2));
  final pole = Offset(math.sin(tilt), -math.cos(tilt));
  c.drawLine(o - pole * (r + 10), o + pole * (r + 10), _s(_or, 2.5));
  c.drawCircle(o + pole * (r + 10), 3.5, _f(_or));
}

// 18. A skater: pull the arms in (drag toward her) and she spins faster.
void _skater(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF14243A));
  c.drawOval(const Rect.fromLTWH(18, 92, 120, 20), _f(_al(_cy, .5)));
  const hip = Offset(78, 70);
  final arm = 8 + st.a * 30, ph = st.ph * _tau, sx = math.cos(ph), depth = math.sin(ph);
  void arms(bool back) {
    for (final sign in [-1.0, 1.0]) {
      final isBack = depth * sign < 0;
      if (isBack != back) continue;
      final sh = hip + const Offset(0, -26);
      c.drawLine(sh, sh + Offset(sx * arm * sign, -4 - (1 - st.a) * 10), _s(back ? const Color(0xFFB04E7E) : _pk, 4));
    }
  }

  arms(true);
  c.drawPath(_poly([hip + const Offset(0, -30), hip + Offset(-18 - sx * 3, 4), hip + Offset(18 - sx * 3, 4)]), _f(_pk));
  c.drawLine(hip + const Offset(0, 4), const Offset(78, 98), _s(_cr, 3));
  c.drawLine(hip + const Offset(0, 4), Offset(78 + sx * 10, 86), _s(_cr, 3));
  c.drawCircle(hip + const Offset(0, -38), 7, _f(_cr));
  c.drawCircle(hip + Offset(sx * 3, -43), 4, _f(_or));
  arms(false);
  final spd = .25 + (1 - st.a) * 2.4;
  for (var i = 0; i < (spd * 2).round(); i++) {
    c.drawArc(Rect.fromCenter(center: hip + Offset(0, -20 + i * 9), width: 70, height: 14), -ph * 2 + i, 1.2, false,
        _s(_al(_ye, .8), 2));
  }
}

// 19. Turns are depth: drag across to drive the screw into the wood; the slot shows each turn.
void _screw(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k1);
  const wood = Rect.fromLTWH(0, 58, 156, 58);
  c.drawRect(wood, _f(_or));
  for (var i = 0; i < 6; i++) {
    final y = 66 + i * 9.0;
    c.drawPath(
        Path()
          ..moveTo(0, y)
          ..cubicTo(50, y - 4, 100, y + 5, 156, y - 2),
        _s(_al(const Color(0xFF8A3A12), .5), 1.5));
  }
  final turns = st.a * 8, depth = st.a * 44;
  final top = 24 + depth;
  c.save();
  c.clipRect(const Rect.fromLTWH(0, 0, 156, 116));
  c.drawRect(Rect.fromLTWH(70, top + 6, 16, 44), _f(_cr));
  for (var i = 0; i < 9; i++) {
    final y = top + 8 + ((i + _wr(turns)) * 5.5);
    c.drawLine(Offset(70, y), Offset(86, y - 3), _s(_k1, 1.5));
  }
  c.drawPath(_poly([Offset(70, top + 50), Offset(86, top + 50), Offset(78, top + 60)]), _f(_cr));
  c.restore();
  c.drawRect(const Rect.fromLTWH(60, 58, 36, 3), _f(_k0));
  final hc = Offset(78, top);
  c.drawOval(Rect.fromCenter(center: hc, width: 40, height: 14), _f(_wh));
  final sa = turns * _tau, sl = math.cos(sa) * 16;
  c.drawLine(hc + Offset(-sl, math.sin(sa) * 4), hc + Offset(sl, -math.sin(sa) * 4), _s(_k0, 3));
  c.drawArc(Rect.fromCenter(center: hc + const Offset(0, -2), width: 58, height: 22), math.pi + .4, 2.3, false, _s(_ye, 2.5));
  c.drawPath(_poly([hc + const Offset(26, -9), hc + const Offset(30, -2), hc + const Offset(21, -3)]), _f(_ye));
}

// 20. Turn the bead chamber; six mirrors repeat it into a pattern that blooms as it turns.
void _kaleido(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF0B0B10));
  const r = 54.0;
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: _ctr, radius: r)));
  c.drawCircle(_ctr, r, _f(_k0));
  final rot = st.a * _tau;
  const cols = [_pk, _ye, _cy, _li, _or, _vi];
  for (var m = 0; m < 6; m++) {
    for (final mir in [1.0, -1.0]) {
      c.save();
      c.translate(_ctr.dx, _ctr.dy);
      c.rotate(m * math.pi / 3);
      c.scale(1, mir);
      for (var i = 0; i < 5; i++) {
        final a = rot + i * 1.7, rr = 10 + i * 9.0;
        final w = math.pi / 6 + math.pi / 6 * math.sin(a);
        final p = Offset(math.cos(w) * rr, math.sin(w) * rr);
        c.drawPath(_star(p, 4 + i * .8, a, k: 3 + i % 3), _f(_al(cols[(i + m) % 6], .9)));
      }
      c.restore();
    }
  }
  c.restore();
  c.drawCircle(_ctr, r, _s(_cr, 3));
}
