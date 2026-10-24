// Part of pop_09_rot.dart (cut there by size, as it was).
part of 'pop_09.dart';

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
