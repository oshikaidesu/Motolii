// Part of pop_01_glow.dart (cut there by size, as it was).
part of 'pop_01.dart';

void _gPaint(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, k = .2 + v.b * .8, r = _rad(v, 3, 20);
  final path = Path();
  if (v.trail.isEmpty) {
    path.moveTo(16, h * .6);
    for (var x = 16.0; x <= w - 16; x += 4) {
      path.lineTo(x, h * .5 + math.sin(x * .07) * 22 + math.sin(x * .19) * 6);
    }
  } else {
    var pen = false;
    for (final q in v.trail) {
      if (q == null) {
        pen = false;
      } else if (!pen) {
        path.moveTo(q.dx, q.dy);
        pen = true;
      } else {
        path.lineTo(q.dx, q.dy);
      }
    }
  }
  final col = _mix(_K.pink, _K.violet, .5 + .5 * math.sin(v.t));
  c.drawPath(path, _s(col, 3 + r * 1.4, k * .1));
  c.drawPath(path, _s(col, 3 + r * .7, k * .2));
  c.drawPath(path, _s(col, 4, .5 + .5 * k));
  c.drawPath(path, _s(_K.white, 1.4, .4 + .6 * k));
  final end = v.trail.isEmpty ? Offset(w - 16, h * .5 + math.sin((w - 16) * .07) * 22 + math.sin((w - 16) * .19) * 6) : v.trail.last ?? s.center(Offset.zero);
  c.drawPath(_star(end, 6 + 2 * math.sin(v.t * 6), 1.5, 4, v.t), _f(_K.white, k));
}

void _gThermal(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, thr = _thr(v), o = Offset(w / 2, h * .34);
  const layers = [(14.0, Color(0xFF2E5BFF), .1), (10.0, Color(0xFF9B7BFF), .3), (6.5, Color(0xFFFF5DA2), .5), (3.5, Color(0xFFFF7A3D), .7), (1.0, Color(0xFFFFD23F), .9)];
  for (final (g, col, lvl) in layers) {
    final hot = lvl > thr;
    final paint = _f(hot ? _mix(col, _K.white, .35) : col);
    c.drawCircle(o, 11 + g, paint);
    c.drawRRect(RRect.fromLTRBR(o.dx - 22 - g, o.dy + 16 - g * .4, o.dx + 22 + g, h + 20, Radius.circular(14 + g)), paint);
  }
  final k = _cl((1 - thr) * 1.4 - .2);
  for (final p in [o, o + const Offset(0, 30)]) {
    _glow(c, p, 8 + v.a * 30, _K.white, k * .8);
  }
  final br = _s(_K.cream, 2);
  for (final (x, y, sx, sy) in [(8.0, 8.0, 1.0, 1.0), (w - 8, 8.0, -1.0, 1.0), (8.0, h - 8, 1.0, -1.0), (w - 8, h - 8, -1.0, -1.0)]) {
    c.drawLine(Offset(x, y), Offset(x + 10 * sx, y), br);
    c.drawLine(Offset(x, y), Offset(x, y + 10 * sy), br);
  }
  c.drawCircle(Offset(w - 16, 16), 3, _f(_K.red, .5 + .5 * math.sin(v.t * 5)));
}

void _gMoon(Canvas c, Size s, PV v) {
  final w = s.width, h = s.height, o = Offset(w * .5, h * .5), k = .2 + v.b * .8, ring = 17 + v.a * 30;
  for (var i = 0; i < 16; i++) {
    c.drawCircle(Offset(_h(i * 2 + 9) * w, _h(i * 2 + 10) * h), .9, _f(_K.white, .5));
  }
  _glow(c, o, ring + 8, _K.cream, k * .35);
  c.drawCircle(o, ring + 2, _s(_K.cyan, 2.5, k * .6));
  c.drawCircle(o, ring, _s(_K.cream, 2, k * .8));
  c.drawCircle(o, ring - 2.5, _s(_K.red, 2, k * .5));
  _glow(c, o, 18, _K.white, .3 + k * .4);
  c.drawCircle(o, 10, _f(_K.cream));
  c.drawCircle(o + const Offset(-3, -2), 2.2, _f(const Color(0xFFD9CFC0)));
  c.drawCircle(o + const Offset(3, 3), 1.6, _f(const Color(0xFFD9CFC0)));
  for (var i = 0; i < 3; i++) {
    final x = (v.t * (6 + i * 3) + i * 60) % (w + 80) - 40, y = h * (.28 + i * .25);
    for (var j = 0; j < 3; j++) {
      c.drawOval(Rect.fromCenter(center: Offset(x + j * 11, y + (j == 1 ? -3 : 0)), width: 22, height: 9), _f(_K.cream, .14));
    }
  }
}
