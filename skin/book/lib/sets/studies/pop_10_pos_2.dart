// Part of pop_10_pos.dart (cut there by size, as it was).
part of 'pop_10.dart';

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
