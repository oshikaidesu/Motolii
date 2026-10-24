// Part of pop_05_remap.dart (cut there by size, as it was).
part of 'pop_05_remap.dart';

void _cassette(Canvas c, Size s, PopP p) {
  bg(c, s, kP0);
  final v = sp(p);
  final body = RRect.fromRectAndRadius(Rect.fromLTWH(14, 18, s.width - 28, 82), const Radius.circular(8));
  c.drawRRect(body.shift(const Offset(4, 5)), fl(Colors.black));
  c.drawRRect(body, fl(kViolet));
  c.drawRect(Rect.fromLTWH(26, 28, s.width - 52, 14), fl(kCream));
  final win = RRect.fromRectAndRadius(Rect.fromLTWH(30, 48, s.width - 60, 34), const Radius.circular(17));
  c.drawRRect(win, fl(kInk));
  final u = .5 + .45 * math.sin(p.ph * .3);
  final l = Offset(win.left + 18, win.center.dy), r = Offset(win.right - 18, win.center.dy);
  c.drawCircle(l, 7 + 9 * (1 - u), fl(const Color(0xFF6B4A2A)));
  c.drawCircle(r, 7 + 9 * u, fl(const Color(0xFF6B4A2A)));
  for (final (o, sgn) in [(l, 1.0), (r, 1.0)]) {
    c.drawCircle(o, 7, fl(kWhite));
    for (var k = 0; k < 3; k++) {
      final a = p.ph * 3 * sgn + k * math.pi * 2 / 3;
      c.drawLine(pol(o, 2, a), pol(o, 6, a), st(spc(v), 2));
    }
  }
  c.drawPath(poly([Offset(48, 100), Offset(56, 90), Offset(s.width - 56, 90), Offset(s.width - 48, 100)]), fl(const Color(0xFF7A5FE0)));
}

void _cards(Canvas c, Size s, PopP p) {
  bg(c, s, kP1);
  final v = sp(p);
  final gap = v == 0 ? 4.0 : 9 + v.abs() * 2.5;
  const n = 6;
  for (var i = n - 1; i >= 0; i--) {
    final o = Offset(24 + i * gap, 30 + i * gap * .55);
    final r = Rect.fromLTWH(o.dx, o.dy, 64, 46);
    c.drawRRect(RRect.fromRectAndRadius(r.shift(const Offset(2, 3)), const Radius.circular(4)), fl(al(Colors.black, .4)));
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), fl(kCream));
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(4)), st(kInk, 1.4));
    final step = v == 0 ? .5 : frac(p.ph * .15 - i * .12 * v.abs().clamp(.3, 3) * v.sign);
    final bp = Offset(r.left + 10 + step * 44, r.bottom - 8 - math.sin(step * math.pi) * 26);
    c.drawCircle(bp, 6, fl(v == 0 ? kYellow : (v < 0 ? kPink : kOrange)));
    c.drawLine(Offset(r.left + 4, r.bottom - 3), Offset(r.right - 4, r.bottom - 3), st(kInk, 1.4));
  }
  if (v == 0) {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(18, 24, 98, 80), const Radius.circular(8)), st(kYellow, 2.5));
  }
}

void _rain(Canvas c, Size s, PopP p) {
  vgrad(c, Offset.zero & s, const Color(0xFF2E3A5C), kP0);
  final v = sp(p), len = 3 + v.abs() * 7;
  for (var k = 0; k < 22; k++) {
    final x = 8 + (k * 53 % 140).toDouble();
    final u = v == 0 ? (k * .41 % 1) : frac(k * .23 + p.ph * .5);
    final y = v < 0 ? s.height - u * (s.height - 30) : 30 + u * (s.height - 30);
    if (v == 0) {
      c.drawCircle(Offset(x, y), 2.4, fl(kYellow));
    } else {
      final d = v < 0 ? 1 : -1;
      c.drawLine(Offset(x, y), Offset(x - 1.5, y + d * len), st(v < 0 ? kPink : kCyan, 2));
    }
  }
  for (final (x, r) in [(40.0, 18.0), (66.0, 24.0), (96.0, 20.0), (120.0, 15.0)]) {
    c.drawCircle(Offset(x, 16), r, fl(kCream));
  }
}

void _warp(Canvas c, Size s, PopP p) {
  bg(c, s, Colors.black);
  final v = sp(p), ctr = s.center(Offset.zero);
  for (var k = 0; k < 9; k++) {
    final u = frac(k / 9 + p.ph * .12);
    final r = math.pow(u, 2.2).toDouble() * 110 + 2;
    final col = k.isEven ? kPink : kCyan;
    c.drawRect(Rect.fromCenter(center: ctr, width: r * 1.5, height: r), st(al(v == 0 ? kYellow : col, u * 1.4), 1 + u * 3));
  }
  for (var k = 0; k < 12; k++) {
    final a = k * math.pi / 6 + .2;
    c.drawLine(pol(ctr, 14, a), pol(ctr, 90, a), Paint()
      ..shader = ui.Gradient.linear(ctr, pol(ctr, 90, a), [al(kViolet, 0), al(kViolet, .6)])
      ..strokeWidth = 1);
  }
  c.drawCircle(ctr, 4, fl(spc(v)));
}
