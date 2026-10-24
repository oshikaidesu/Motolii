// Part of op_06_stroke.dart (cut there by size, as it was).
part of 'op_06.dart';

// 18. A width profile lane (draw it); the stroke above swells and thins along its length to match.
void _s18(Canvas c, Size z, OpS s) {
  final prof = List<double>.filled(9, .5);
  for (var i = 0; i < 9; i++) {
    prof[i] = .35 + .3 * math.sin(i * .9);
  }
  for (final q in s.trail) {
    final i = ((q.dx - 8) / 140 * 8).round().clamp(0, 8);
    if (q.dy > 66) prof[i] = (1 - (q.dy - 70) / 44).clamp(0.0, 1.0);
  }
  _profile = prof;
  c.drawRect(const Rect.fromLTWH(6, 70, 144, 44), _st(_dg, .8));
  for (var i = 0; i <= 8; i++) {
    _vdots(c, 8 + i * 17.5, 72, 112, _al(_dg, .8));
  }
  final pp = [for (var i = 0; i <= 8; i++) Offset(8 + i * 17.5, 112 - prof[i] * 40)];
  _path(c, _poly(pp), _bl, 1.1);
  for (final q in pp) {
    _dot(c, q, _bl, 1.8);
  }
  final line = Path()
    ..moveTo(10, 36)
    ..cubicTo(50, 6, 100, 66, 146, 30);
  _ribbon(c, line, 22, _bl, taper: _taper, lw: .9);
  _path(c, line, _al(_wh, .5), .6);
  _dot(c, const Offset(10, 36), _gr, 2);
  _dot(c, const Offset(146, 30), _wh, 2);
}

var _profile = List<double>.filled(9, .5);
double _taper(double u) {
  final f = u * 8, i = f.floor().clamp(0, 7);
  return math.max(.05, _lp(_profile[i], _profile[i + 1], f - i));
}

// 19. A cat bats the yarn ball (flick); the thread unrolls behind it.
void _s19(Canvas c, Size z, OpS s) {
  const paw = Offset(30, 74);
  final bx = 44 + s.a * 100;
  final ball = Offset(bx, 96);
  final thread = Path()
    ..moveTo(paw.dx + 6, paw.dy + 6)
    ..cubicTo(paw.dx + 20, 120, bx - 30, 80, bx - 9, 98);
  _dash(c, thread, _wh, 2 + s.b * 6, 2, w: .9);
  _ring(c, ball, 9, _wh, 1.1);
  final rot = s.a * 14;
  for (var i = 0; i < 3; i++) {
    final a = rot + i * 1.05;
    c.drawArc(Rect.fromCircle(center: ball, radius: 9), a, 1.8, false, _st(_rd, .8));
    _path(c, Path()..addOval(Rect.fromCenter(center: ball, width: 18 * math.cos(a).abs() + .5, height: 18)), _al(_rd, .6), .7);
  }
  final cat = Path()
    ..moveTo(10, 108)
    ..quadraticBezierTo(4, 80, 14, 60)
    ..lineTo(12, 46)
    ..lineTo(20, 54)
    ..lineTo(28, 54)
    ..lineTo(34, 44)
    ..lineTo(34, 60)
    ..quadraticBezierTo(40, 80, 34, 108);
  _path(c, cat, _wh, 1.1);
  final swat = math.sin(s.t * 4) * 3 + s.e * 6;
  _path(c, Path()
    ..moveTo(28, 70)
    ..quadraticBezierTo(34 + swat, 66, paw.dx + 4 + swat, paw.dy), _wh, 1.1);
  _dot(c, const Offset(19, 60), _wh, 1);
  _dot(c, const Offset(28, 60), _wh, 1);
  final tail = math.sin(s.t * 1.6) * 6;
  _path(c, Path()
    ..moveTo(10, 106)
    ..quadraticBezierTo(-4, 90 + tail, 6, 76 + tail), _wh, 1);
  _ln(c, 4, 108, 152, 108, _dg, 1);
  _t(c, 'UNROLL', 150, 6, _wh, right: true);
  _num(c, _n(s.a), 150, 14, _wh, 22, true);
}

// 20. Fireworks: every trail is a trimmed path — start chases end outward; width and dash go wild.
void _s20(Canvas c, Size z, OpS s) {
  const o = Offset(78, 56);
  final ph = ((s.t * .35) + s.a) % 1;
  final st = math.max(0.0, ph * 1.3 - .35), en = math.min(1.0, ph * 1.3);
  final cols = [_bl, _gr, _wh, _rd, _pu];
  final n = 10 + (s.b * 14).round();
  for (var i = 0; i < n; i++) {
    final a = i / n * math.pi * 2 + .2;
    final len = 30 + _h(i) * 30;
    final end = o + Offset(math.cos(a) * len, math.sin(a) * len * .85 + len * .3);
    final ctl = o + Offset(math.cos(a) * len * .6, math.sin(a) * len * .6 - 10);
    final p = Path()
      ..moveTo(o.dx, o.dy)
      ..quadraticBezierTo(ctl.dx, ctl.dy, end.dx, end.dy);
    final col = cols[i % cols.length];
    if (s.b > .6 && i.isEven) {
      _ribbon(c, p, 1 + s.b * 4, col, a: st, b: en, lw: .6);
    } else {
      _dash(c, _trim(p, st, en), col, 1 + (1 - s.b) * 5, 1 + s.b * 4, w: 1);
    }
    final tip = _at(p, en)!.position;
    _dot(c, tip, col, 1.2);
  }
  _ln(c, 78, 118, 78, 56 + (1 - math.min(1.0, ph * 4)) * 60, _al(_wh, .4), .8);
  _num(c, _n(s.b), 150, 14, _rd, 22, true);
  _t(c, 'BURST', 150, 6, _rd, right: true, size: 6.5);
}
