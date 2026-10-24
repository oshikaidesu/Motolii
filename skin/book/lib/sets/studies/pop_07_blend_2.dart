// Part of pop_07_blend.dart (cut there by size, as it was).
part of 'pop_07.dart';

void _jelly(Canvas c, Size s, P7State st, double t) {
  const sq = 13.0;
  for (var y = 0.0; y < s.height; y += sq) {
    for (var x = 0.0; x < s.width; x += sq) {
      if (((x + y) / sq).round().isEven) c.drawRect(Rect.fromLTWH(x, y, sq, sq), _f(_pk));
    }
  }
  c.drawOval(Rect.fromLTWH(10, s.height * .55, s.width - 20, s.height * .42), _f(_cr));
  final cols = [_li, _re, _cy];
  final op = .18 + .78 * st.b;
  for (var i = 0; i < 3; i++) {
    final base = Offset(30 + i * 46.0, s.height * .78 - (i == 1 ? 6 : 0));
    final wob = math.sin(t * 4 + i * 2) * 3;
    const w = 30.0, hgt = 30.0, dx = 10.0, dy = 7.0;
    final front = Path()
      ..moveTo(base.dx - w / 2, base.dy)
      ..lineTo(base.dx + w / 2, base.dy)
      ..lineTo(base.dx + w / 2 + wob, base.dy - hgt)
      ..lineTo(base.dx - w / 2 + wob, base.dy - hgt)
      ..close();
    final topF = Path()
      ..moveTo(base.dx - w / 2 + wob, base.dy - hgt)
      ..lineTo(base.dx + w / 2 + wob, base.dy - hgt)
      ..lineTo(base.dx + w / 2 + wob + dx, base.dy - hgt - dy)
      ..lineTo(base.dx - w / 2 + wob + dx, base.dy - hgt - dy)
      ..close();
    final side = Path()
      ..moveTo(base.dx + w / 2, base.dy)
      ..lineTo(base.dx + w / 2 + dx, base.dy - dy)
      ..lineTo(base.dx + w / 2 + wob + dx, base.dy - hgt - dy)
      ..lineTo(base.dx + w / 2 + wob, base.dy - hgt)
      ..close();
    c.drawPath(front, _f(_o(cols[i], op)));
    c.drawPath(side, _f(_o(Color.lerp(cols[i], _ink, .3)!, op)));
    c.drawPath(topF, _f(_o(Color.lerp(cols[i], _wh, .4)!, op)));
    for (final p in [front, side, topF]) {
      c.drawPath(p, _s(_o(_wh, .7), 1.2));
    }
    c.drawLine(Offset(base.dx - w / 2 + 5 + wob * .6, base.dy - hgt + 6), Offset(base.dx - w / 2 + 5 + wob * .3, base.dy - 8), _s(_o(_wh, .8), 2.5));
  }
}

void _chameleon(Canvas c, Size s, P7State st, double t) {
  const stripe = 14.0;
  void bg() {
    for (var x = -s.height; x < s.width; x += stripe * 2) {
      c.drawPath(
          Path()
            ..moveTo(x, s.height)
            ..lineTo(x + stripe, s.height)
            ..lineTo(x + stripe + s.height, 0)
            ..lineTo(x + s.height, 0)
            ..close(),
          _f(_or));
    }
  }

  c.drawRect(Offset.zero & s, _f(_ye));
  bg();
  c.drawLine(Offset(0, s.height * .78), Offset(s.width, s.height * .7), _s(const Color(0xFF6B4A2E), 7));
  final cx = s.width * .48, cy = s.height * .5;
  final body = Path()
    ..addOval(Rect.fromCenter(center: Offset(cx, cy), width: 70, height: 38))
    ..addOval(Rect.fromCenter(center: Offset(cx + 34, cy - 6), width: 30, height: 26));
  final tail = Path()..addArc(Rect.fromCenter(center: Offset(cx - 42, cy + 8), width: 22, height: 22), -math.pi / 2, math.pi * 1.6);
  final blend = st.a;
  c.drawPath(body, _f(_o(_li, 1 - blend)));
  c.drawPath(tail, _s(_o(_li, 1 - blend), 7));
  c.drawPath(body, _s(_o(_ink, .9 - blend * .7), 2));
  for (var k = 0; k < 3; k++) {
    c.drawLine(Offset(cx - 18 + k * 14.0, cy + 14), Offset(cx - 22 + k * 14.0, s.height * .74), _s(_o(_ink, 1 - blend * .6), 3));
  }
  final eye = Offset(cx + 38, cy - 10);
  c.drawCircle(eye, 8, _f(_wh));
  c.drawCircle(eye, 8, _s(_ink, 2));
  final look = Offset(math.cos(t * 1.3), math.sin(t * 1.7)) * 3;
  c.drawCircle(eye + look, 3.2, _f(_ink));
  final tongue = (math.sin(t * 1.1) > .93) ? 30.0 : 0.0;
  if (tongue > 0) c.drawLine(Offset(cx + 48, cy - 2), Offset(cx + 48 + tongue, cy - 12), _s(_pk, 3));
}

void _dayNight(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(_k1));
  final ctr = Offset(s.width / 2, s.height / 2), r = 48.0;
  final x = st.a;
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  c.drawRect(Offset.zero & s, _f(Color.lerp(_cy, const Color(0xFF151A4A), x)!));
  for (var i = 0; i < 14; i++) {
    c.drawCircle(Offset(_h(i) * s.width, _h(i + 40) * s.height * .6), 1.2, _f(_o(_wh, x)));
  }
  final arc = math.pi * (1 - x);
  final sun = ctr + Offset(math.cos(arc) * r * .7, -math.sin(arc) * r * .7 + 12);
  c.drawCircle(sun, 10, _f(Color.lerp(_ye, _cr, x)!));
  if (x > .5) c.drawCircle(sun + const Offset(5, -3), 9, _f(_o(const Color(0xFF151A4A), (x - .5) * 2)));
  final sea = Path()..moveTo(ctr.dx - r, ctr.dy + 18);
  for (var px = ctr.dx - r; px <= ctr.dx + r; px += 4) {
    sea.lineTo(px, ctr.dy + 18 + math.sin(px * .2 + t * 2) * 2);
  }
  sea
    ..lineTo(ctr.dx + r, s.height)
    ..lineTo(ctr.dx - r, s.height)
    ..close();
  c.drawPath(sea, _f(Color.lerp(_bl, const Color(0xFF0D1030), x)!));
  c.drawLine(Offset(sun.dx - 8, ctr.dy + 24), Offset(sun.dx + 8, ctr.dy + 24), _s(_o(_ye, 1 - x * .5), 2));
  c.restore();
  c.drawCircle(ctr, r, _s(_or, 9));
  for (var i = 0; i < 8; i++) {
    final a = i * math.pi / 4;
    c.drawCircle(ctr + Offset(math.cos(a), math.sin(a)) * r, 2.2, _f(_ink));
  }
}

void _hologram(Canvas c, Size s, P7State st, double t) {
  final base = Offset(s.width / 2, s.height - 14);
  final cone = Path()
    ..moveTo(base.dx - 10, base.dy)
    ..lineTo(base.dx - 46, 8)
    ..lineTo(base.dx + 46, 8)
    ..lineTo(base.dx + 10, base.dy)
    ..close();
  c.drawPath(cone, _f(_o(_cy, .08 + .1 * st.b)));
  final on = st.b;
  final cx = s.width / 2, cy = s.height * .42;
  final head = Path()
    ..addOval(Rect.fromCircle(center: Offset(cx, cy), radius: 28))
    ..moveTo(cx - 26, cy - 10)
    ..lineTo(cx - 22, cy - 38)
    ..lineTo(cx - 6, cy - 26)
    ..moveTo(cx + 26, cy - 10)
    ..lineTo(cx + 22, cy - 38)
    ..lineTo(cx + 6, cy - 26);
  c.save();
  c.clipPath(Path()
    ..addOval(Rect.fromCircle(center: Offset(cx, cy), radius: 28))
    ..addPolygon([Offset(cx - 27, cy - 8), Offset(cx - 22, cy - 38), Offset(cx - 4, cy - 26)], true)
    ..addPolygon([Offset(cx + 27, cy - 8), Offset(cx + 22, cy - 38), Offset(cx + 4, cy - 26)], true));
  for (var y = cy - 40; y < cy + 30; y += 3) {
    final glitch = (math.sin(y * 1.7 + t * 9) > .9 - (1 - on) * .8) ? (math.sin(t * 31 + y) * 8) : 0.0;
    final a = on * (.55 + .45 * math.sin(y * .3 - t * 6).abs());
    c.drawLine(Offset(cx - 40 + glitch, y), Offset(cx + 40 + glitch, y), _s(_o(_cy, a), 1.6));
  }
  c.restore();
  c.drawPath(head, _s(_o(_wh, on * .8), 1.5));
  c.drawCircle(Offset(cx - 10, cy - 2), 4, _f(_o(_pk, on)));
  c.drawCircle(Offset(cx + 10, cy - 2), 4, _f(_o(_pk, on)));
  c.drawOval(Rect.fromCenter(center: base, width: 60, height: 14), _f(_k1));
  c.drawOval(Rect.fromCenter(center: base, width: 60, height: 14), _s(_vi, 2));
  c.drawOval(Rect.fromCenter(center: base - const Offset(0, 2), width: 22, height: 6), _f(_o(_cy, .4 + .6 * on)));
}
