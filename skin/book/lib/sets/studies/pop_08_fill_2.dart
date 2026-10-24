// Part of pop_08_fill.dart (cut there by size, as it was).
part of 'pop_08.dart';

void _fPhoto(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final fr = Rect.fromLTWH(22, 6, s.width - 44, s.height - 10);
  c.save();
  c.translate(fr.center.dx, fr.center.dy);
  c.rotate(-.04);
  c.translate(-fr.center.dx, -fr.center.dy);
  c.drawRect(fr.shift(const Offset(2, 3)), _f(_al(_k0, .5)));
  c.drawRect(fr, _f(_wh));
  final ph = Rect.fromLTRB(fr.left + 6, fr.top + 6, fr.right - 6, fr.bottom - 18);
  c.save();
  c.clipRect(ph);
  final a = st.a, b = st.b;
  c.drawRect(ph, _f(_h(.55 + a, b * .8, .72)));
  c.drawCircle(ph.topRight + const Offset(-14, 12), 7, _f(_h(.14 + a, b, .62)));
  c.drawOval(Rect.fromLTWH(ph.left - 20, ph.top + 30, 90, 50), _f(_h(.3 + a, b * .7, .45)));
  c.drawRect(Rect.fromLTRB(ph.left, ph.top + 46, ph.right, ph.bottom), _f(_h(.3 + a, b * .8, .38)));
  const hues = [.0, .92, .12, .08];
  for (var r = 0; r < 3; r++) {
    for (var k = 0; k < 7 - r; k++) {
      final x = ph.left + 6 + k * (ph.width - 12) / (6 - r) + r * 5, y = ph.top + 52 + r * 13.0, sz = 3.0 + r * 1.4;
      c.drawLine(Offset(x, y), Offset(x, y + 6 + r * 2), _s(_h(.33 + a, b, .3), 1.2));
      c.drawCircle(Offset(x, y), sz, _f(_h(hues[(k + r) % 4] + a, b, .58)));
    }
  }
  c.restore();
  c.restore();
}

void _fDuotone(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _cr);
  final ink1 = _h(st.a, .9, .58), ink2 = _h(st.b, .85, .5);
  Paint m(Color col) => _f(col)..blendMode = BlendMode.multiply;
  c.drawCircle(Offset(s.width * .4, s.height * .42), 32, m(ink1));
  c.drawPath(_poly([Offset(s.width * .2, s.height - 12), Offset(s.width * .58, s.height * .2), Offset(s.width * .88, s.height - 12)]), m(ink2));
  for (var y = 0; y < 8; y++) {
    for (var x = 0; x < 13; x++) {
      final r = 2.4 * (x / 12);
      if (r > .3) c.drawCircle(Offset(10.0 + x * 11 + (y.isOdd ? 5 : 0), 10.0 + y * 13), r, m(ink2));
    }
  }
  for (final o in [const Offset(8, 8), Offset(s.width - 8, s.height - 8)]) {
    c.drawLine(o - const Offset(5, 0), o + const Offset(5, 0), _s(_k0, 1));
    c.drawLine(o - const Offset(0, 5), o + const Offset(0, 5), _s(_k0, 1));
    c.drawCircle(o, 3, _s(_k0, 1));
  }
}

void _fBubble(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  for (var i = 0; i < 10; i++) {
    c.drawCircle(Offset(_rn(i + 2) * s.width, _rn(i + 21) * s.height), .9, _f(_al(_cr, .5)));
  }
  void bubble(Offset c0, double r, double ph) {
    c.save();
    c.translate(c0.dx, c0.dy);
    c.rotate(ph);
    const cols = [_pk, _ye, _li, _cy, _vi, _pk];
    c.drawCircle(Offset.zero, r, Paint()..shader = ui.Gradient.sweep(Offset.zero, [for (final k in cols) _al(k, .55)], const [0, .2, .4, .6, .8, 1]));
    c.restore();
    final bands = 2 + (st.b * 5).round();
    for (var k = 0; k < bands; k++) {
      final rr = r * (1 - (k + 1) / (bands + 1) * .7);
      c.drawCircle(c0 + Offset(math.cos(ph * 2) * (r - rr) * .4, math.sin(ph * 2) * (r - rr) * .4), rr, _s(_al(_h(st.b * 2 + k * .17 + ph * .05, 1, .65), .5), 1.5));
    }
    c.drawCircle(c0, r, Paint()..shader = ui.Gradient.radial(c0, r, [_al(_wh, 0), _al(_wh, 0), _al(_wh, .55)], [0, .78, 1]));
    c.drawArc(Rect.fromCircle(center: c0, radius: r * .78), math.pi * 1.1, .7, false, _s(_al(_wh, .9), 2.5));
    c.drawCircle(c0 + Offset(-r * .2, -r * .55), r * .06, _f(_wh));
  }

  final ph = st.a * math.pi * 2 + st.b * 8 + t * .3;
  bubble(Offset(s.width * .46, s.height * .52 + math.sin(t * 1.2) * 3), 40, ph);
  bubble(Offset(s.width * .86, s.height * .26 + math.sin(t * 1.6) * 3), 12, -ph);
}

void _fTopo(Canvas c, Size s, _P8 st, double t) {
  const cell = 4.0;
  final sea = .12 + st.a * .45, snow = .95 - st.b * .45;
  const pk = [(.3, .4, .26, 1.0), (.68, .55, .2, .85), (.55, .2, .14, .6)];
  for (var y = 0.0; y < s.height; y += cell) {
    for (var x = 0.0; x < s.width; x += cell) {
      final u = x / s.width, v = y / s.height;
      var hgt = .08 * math.sin(u * 17 + v * 9) + .05 * math.sin(v * 23 - u * 5);
      for (final (px, py, r, a) in pk) {
        final d2 = ((u - px) * (u - px) * 1.7 + (v - py) * (v - py)) / (r * r);
        hgt += a * math.exp(-d2);
      }
      final Color col;
      if (hgt < sea) {
        col = Color.lerp(const Color(0xFF1B3FD6), _cy, (hgt / sea).clamp(0.0, 1.0))!;
      } else if (hgt < sea + .04) {
        col = const Color(0xFFFFE08A);
      } else if (hgt < snow) {
        col = Color.lerp(_li, const Color(0xFF3E8F3A), ((hgt - sea) / (snow - sea)).clamp(0.0, 1.0))!;
      } else {
        col = _wh;
      }
      c.drawRect(Rect.fromLTWH(x, y, cell + .3, cell + .3), _f(col));
    }
  }
  final wv = Offset(s.width * .12 + math.sin(t) * 3, s.height * .85);
  c.drawLine(wv, wv + const Offset(8, 0), _s(_al(_wh, .8), 1.2));
}

void _fFlask(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final cx = s.width / 2, bot = s.height - 8, nt = 40.0;
  final flask = Path()
    ..moveTo(cx - 7, nt)
    ..lineTo(cx + 7, nt)
    ..lineTo(cx + 7, nt + 16)
    ..lineTo(cx + 34, bot - 6)
    ..quadraticBezierTo(cx + 36, bot, cx + 28, bot)
    ..lineTo(cx - 28, bot)
    ..quadraticBezierTo(cx - 36, bot, cx - 34, bot - 6)
    ..lineTo(cx - 7, nt + 16)
    ..close();
  final liq = _h(st.a * .8, .95, .55), ly = nt + 34;
  c.save();
  c.clipPath(flask);
  c.drawRect(Rect.fromLTRB(0, ly, s.width, bot), _f(liq));
  for (var k = 0; k < 4; k++) {
    final q = _wr(t * .5 + k * .27);
    c.drawCircle(Offset(cx - 14 + k * 9.0, bot - 4 - q * (bot - ly - 6)), 1.8, _s(_al(_wh, .8 - q * .6), 1));
  }
  c.restore();
  c.drawPath(flask, _s(_wh, 2));
  final q = _wr(t * 1.1);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(cx - 3, 4, 6, 18), const Radius.circular(2)), _f(_al(_cr, .9)));
  c.drawOval(Rect.fromCenter(center: Offset(cx, 6), width: 12, height: 10), _f(_rd));
  final dropY = 24 + q * (ly - 26);
  c.drawPath(
      Path()
        ..moveTo(cx, dropY - 4)
        ..quadraticBezierTo(cx + 3.5, dropY + 1, cx, dropY + 3)
        ..quadraticBezierTo(cx - 3.5, dropY + 1, cx, dropY - 4),
      _f(_vi));
  if (q > .85) c.drawOval(Rect.fromCenter(center: Offset(cx, ly), width: (q - .85) * 120, height: (q - .85) * 20), _s(_al(_wh, .7), 1));
}
