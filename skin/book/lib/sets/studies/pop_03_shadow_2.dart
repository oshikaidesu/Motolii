// Part of pop_03_shadow.dart (cut there by size, as it was).
part of 'pop_03.dart';

void _sCandle(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ty = h * .8, cx = w * .2;
  final fl = _vn(t * 7, 1.3), flame = Offset(cx + fl * 1.5, ty - 30 - fl.abs() * 2);
  final ox = st.at(s).dx.clamp(w * .32, w * .88), base = Offset(ox, ty);
  final m = 1 + 26 / ((ox - cx) * .55 + 4);
  final vase = _vasePath(base);
  c.drawCircle(flame, 90, Paint()..shader = ui.Gradient.radial(flame, 90, [_al(_kOrange, .45), _al(_kOrange, 0)]));
  c
    ..save()
    ..clipRect(Rect.fromLTWH(0, 0, w, ty))
    ..translate(flame.dx, flame.dy)
    ..scale(m)
    ..translate(-flame.dx, -flame.dy);
  c.drawPath(vase.shift(Offset((base.dx - flame.dx) * .25, 0)), _soft(_al(const Color(0xFF000000), .8), (1 + m * .7) / m));
  c.restore();
  c.drawRect(Rect.fromLTWH(0, ty, w, h - ty), _f(const Color(0xFF3A2418)));
  c.drawRect(Rect.fromLTWH(cx - 5, ty - 22, 10, 22), _f(_kCream));
  final fp = Path()
    ..moveTo(cx - 4, ty - 23)
    ..quadraticBezierTo(cx - 5, ty - 30, flame.dx, flame.dy - 6)
    ..quadraticBezierTo(cx + 5, ty - 30, cx + 4, ty - 23)
    ..close();
  c.drawPath(fp, _f(_kOrange));
  c.drawCircle(Offset(cx, ty - 26), 2.4, _f(_kYellow));
  c.drawPath(vase, _f(_kPink));
  c.drawPath(Path()..addOval(Rect.fromCircle(center: Offset(base.dx - 5, base.dy - 33), radius: 5)), _f(_kYellow));
  c.drawPath(Path()..addOval(Rect.fromCircle(center: Offset(base.dx + 6, base.dy - 31), radius: 4)), _f(_kViolet));
}

void _sIsland(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, sea = h * .62;
  c.drawRect(Rect.fromLTWH(0, sea, w, h - sea), _f(const Color(0xFF1E8FC4)));
  for (var k = 0; k < 4; k++) {
    final y = sea + 6 + k * 10.0;
    c.drawLine(Offset(w * (.1 + .2 * (k % 2)), y), Offset(w * (.3 + .2 * (k % 2)), y), _s(_al(_kCyan, .7), 1.5));
    c.drawLine(Offset(w * (.6 - .1 * (k % 2)), y + 4), Offset(w * (.8 - .1 * (k % 2)), y + 4), _s(_al(_kCyan, .7), 1.5));
  }
  c.drawCircle(Offset(w * .86, h * .14), 10, _f(_kYellow));
  final alt = st.b, x = st.at(s).dx.clamp(w * .25, w * .75), y = h * .5 - alt * h * .36 + math.sin(t * 1.4) * 2;
  final hgt = (sea + 12) - y;
  c.drawOval(Rect.fromCenter(center: Offset(x - hgt * .35, sea + 14), width: 54 - alt * 10, height: 12), _soft(_al(const Color(0xFF07304A), .55), 1 + alt * 6));
  c.drawPath(_poly([Offset(x - 26, y), Offset(x + 26, y), Offset(x + 6, y + 24), Offset(x - 2, y + 30)]), _f(const Color(0xFFB0572B)));
  c.drawLine(Offset(x + 20, y + 2), Offset(x + 20, y + 22 + alt * 10), _s(_al(_kCyan, .9), 2));
  c.drawOval(Rect.fromCenter(center: Offset(x, y), width: 56, height: 14), _f(_kLime));
  c.drawLine(Offset(x - 8, y - 2), Offset(x - 8, y - 14), _s(const Color(0xFF7A4A22), 2.5));
  c.drawCircle(Offset(x - 8, y - 17), 7, _f(_kPink));
  c.drawCircle(Offset(x + 8, y - 6), 5, _f(const Color(0xFF4FB548)));
}

Path _monster() => Path()
  ..moveTo(-1, 0)
  ..lineTo(-.95, -.55)
  ..lineTo(-1.35, -.95)
  ..lineTo(-.85, -.8)
  ..lineTo(-.7, -1.1)
  ..lineTo(-.95, -1.5)
  ..lineTo(-.5, -1.25)
  ..lineTo(-.3, -1.35)
  ..lineTo(-.4, -1.85)
  ..lineTo(-.05, -1.4)
  ..lineTo(.05, -1.4)
  ..lineTo(.4, -1.85)
  ..lineTo(.3, -1.35)
  ..lineTo(.5, -1.25)
  ..lineTo(.95, -1.5)
  ..lineTo(.7, -1.1)
  ..lineTo(.85, -.8)
  ..lineTo(1.35, -.95)
  ..lineTo(.95, -.55)
  ..lineTo(1, 0)
  ..close();

void _sMonster(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, fy = h * .82;
  c.drawRect(Rect.fromLTWH(0, fy, w, h - fy), _f(const Color(0xFF221B33)));
  final cr = Offset(w * .5, fy), torch = st.at(s);
  final dist = (torch - cr).distance, m = 26 + 2600 / (dist + 10);
  final dir = (cr - torch) / math.max(dist, 1);
  final sc = Offset(cr.dx + dir.dx * m * .5, fy - 2);
  c.drawPath(_poly([torch, torch + _rot(dir, .45) * 200, torch + _rot(dir, -.45) * 200]), _f(_al(_kYellow, .22)));
  c
    ..save()
    ..clipRect(Rect.fromLTWH(0, 0, w, fy))
    ..translate(sc.dx, sc.dy)
    ..scale(m * .55, m * .6);
  c.drawPath(_monster(), _soft(const Color(0xFF05030A), .01 + .02 * (dist / 60)));
  c.restore();
  final eye = Offset(sc.dx, sc.dy - m * .6 * .9);
  for (final ex in [-.3, .3]) {
    c.drawPath(_poly([eye + Offset(ex * m * .55 - m * .08, 0), eye + Offset(ex * m * .55 + m * .08, -m * .04), eye + Offset(ex * m * .55, m * .05)]), _f(_kRed));
  }
  c.drawCircle(cr - const Offset(0, 9), 10, _f(_kLime));
  for (final ex in [-3.5, 3.5]) {
    c.drawCircle(cr + Offset(ex, -11), 2.8, _f(_kWhite));
    c.drawCircle(cr + Offset(ex, -11), 1.3, _f(_kInk));
  }
  c
    ..save()
    ..translate(torch.dx, torch.dy)
    ..rotate(math.atan2(dir.dy, dir.dx));
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-14, -3.5, 14, 7), const Radius.circular(2)), _f(_kCream));
  c.drawCircle(Offset.zero, 3.5, _f(_kYellow));
  c.restore();
}
