// Fill / Colour / Gradient: twenty pictures of "what colour it is, how strong, where one colour turns into the next" (hue, saturation, stops).
part of 'pop_08.dart';

List<_Pan8> _fillPanels() => const [
      _Pan8('Palette mix', 'material · flat · rub → mix, x → yellow…blue', _G.rub, _fPalette, a: .5, b: .25),
      _Pan8('Sunset sky', 'landscape · bold flat · drag the sun', _G.xy, _fSunset, a: .55, b: .3),
      _Pan8('Prism', 'physics · crisp · spin → which colour lands', _G.spin, _fPrism, a: .5),
      _Pan8('Chameleon', 'creature · flat · x → hue, up → vivid', _G.xy, _fChameleon, a: .3, b: .8),
      _Pan8('Lava lamp', 'neon wild · x → top hue, up → bottom hue', _G.xy, _fLava, a: .85, b: .1),
      _Pan8('Blush', 'character · flat · up → blush, x → hair', _G.xy, _fBlush, a: .75, b: .6),
      _Pan8('Sunrise glass', 'food · pseudo 3D · x → stop, up → soft', _G.xy, _fGlass, a: .45, b: .4),
      _Pan8('Spray wall', 'graffiti · neon · paint, hue runs on', _G.spray, _fSpray),
      _Pan8('Clownfish', 'creature · bold flat · x → stripes, up → hue', _G.xy, _fFish, a: .3, b: .05),
      _Pan8('Rain wash', 'weather · flat · down → washed grey', _G.xy, _fRain, b: .65),
      _Pan8('Aurora', 'cosmic · glow · x → hue, up → strength', _G.xy, _fAurora, a: .3, b: .8),
      _Pan8('Thermal cat', 'machine · pixel · x → palette, up → contrast', _G.xy, _fThermal, a: .5, b: .5),
      _Pan8('Stained glass', 'material · flat · paint cells to light', _G.spray, _fStained),
      _Pan8('Neon sign', 'neon wild · x → hue, up → gas on', _G.xy, _fNeon, a: .92, b: .9),
      _Pan8('Barber pole', 'toy · pseudo 3D · spin → turn, out → width', _G.spin, _fPole, a: .1, b: .4),
      _Pan8('Tulip photo', 'result · light · x → hue shift, up → sat', _G.xy, _fPhoto, a: 0, b: .8),
      _Pan8('Duotone print', 'poster · light · x → ink 1, up → ink 2', _G.xy, _fDuotone, a: .97, b: .6),
      _Pan8('Soap bubble', 'material · pseudo 3D · rub → swirl', _G.rub, _fBubble, a: .3, b: .4),
      _Pan8('Topo map', 'landscape · top-down · x → sea, up → snow', _G.xy, _fTopo, a: .4, b: .35),
      _Pan8('Indicator flask', 'science · flat · tap / drag → drops', _G.xy, _fFlask, a: .55, tap: true),
    ];

void _fPalette(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final pal = Rect.fromCenter(center: s.center(Offset.zero), width: s.width * .86, height: s.height * .8);
  c.drawOval(pal.shift(const Offset(2, 3)), _f(_al(_k0, .5)));
  c.drawOval(pal, _f(_cr));
  c.drawCircle(pal.centerLeft + const Offset(18, 12), 7, _f(_k1));
  final yb = pal.topLeft + const Offset(34, 18), bb = pal.topRight + const Offset(-26, 22), mix = pal.center + const Offset(8, 6);
  c.drawCircle(yb, 11, _f(_ye));
  c.drawCircle(bb, 11, _f(_bl));
  final hue = (50 + st.a * 175) / 360;
  c.drawCircle(mix, 22, _f(_h(hue, .85, .5)));
  final raw = 1 - st.b;
  if (raw > .02) {
    c.save();
    c.clipPath(Path()..addOval(Rect.fromCircle(center: mix, radius: 22)));
    for (var k = 0; k < 4; k++) {
      final r = 5.0 + k * 5, ph = st.b * 9 + k * 1.7;
      c.drawArc(Rect.fromCircle(center: mix, radius: r), ph, 2.2, false, _s(_al(k.isEven ? _ye : _bl, raw), 3.5));
    }
    c.restore();
  }
  final tip = st.down ? (st.at ?? mix) : mix + Offset(math.cos(t * 1.5) * 6, math.sin(t * 1.5) * 4);
  c.drawLine(tip, tip + const Offset(26, -32), _s(_or, 4));
  c.drawLine(tip, tip + const Offset(7, -9), _s(_k0, 5));
  c.drawCircle(tip, 3, _f(_h(hue, .85, .5)));
}

void _fSunset(Canvas c, Size s, _P8 st, double t) {
  final w = s.width, h = s.height, hz = h * .66, low = 1 - st.b;
  final top = Color.lerp(_bl, const Color(0xFF3B1E6E), low)!, mid = Color.lerp(_cy, _pk, low)!, hor = Color.lerp(const Color(0xFFBFF3FF), _or, low)!;
  c.drawRect(Rect.fromLTWH(0, 0, w, hz), Paint()..shader = ui.Gradient.linear(Offset.zero, Offset(0, hz), [top, mid, hor], [0, .55, 1]));
  final sun = Offset(16 + st.a * (w - 32), hz + 8 - st.b * (hz - 6));
  c.drawCircle(sun, 13, _f(Color.lerp(_ye, const Color(0xFFFFE9A6), st.b)!));
  c.drawPath(
      Path()
        ..moveTo(0, hz)
        ..lineTo(18, hz - 22)
        ..lineTo(34, hz - 10)
        ..lineTo(50, hz - 30)
        ..lineTo(74, hz)
        ..close(),
      _f(Color.lerp(const Color(0xFF2B5C8A), _k0, low)!));
  c.drawRect(Rect.fromLTWH(0, hz, w, h - hz), _f(Color.lerp(const Color(0xFF1F7FD1), const Color(0xFF2A1450), low)!));
  for (var k = 0; k < 5; k++) {
    final y = hz + 5 + k * 7.0, hw = (14 - k * 2) * (.8 + .2 * math.sin(t * 2 + k));
    c.drawLine(Offset(sun.dx - hw, y), Offset(sun.dx + hw, y), _s(_al(Color.lerp(_ye, _or, low)!, .9 - k * .14), 2));
  }
}

void _fPrism(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  final h = s.height, w = s.width, pc = Offset(w * .36, h * .5), rot = (st.a - .5) * 1.2;
  final tri = [for (var k = 0; k < 3; k++) pc + Offset(math.cos(rot - math.pi / 2 + k * 2 * math.pi / 3), math.sin(rot - math.pi / 2 + k * 2 * math.pi / 3)) * 24];
  final enter = Offset.lerp(tri[0], tri[2], .55)!, exit = Offset.lerp(tri[0], tri[1], .55)!;
  c.drawLine(Offset(0, h * .62), enter, _s(_wh, 3));
  c.drawLine(enter, exit, _s(_al(_wh, .5), 2));
  final target = Offset(w - 15, h * .5);
  const k7 = 7;
  final spread = 11.0, shift = (st.a - .5) * 120;
  var best = 0;
  var bd = 1e9;
  final ys = [for (var k = 0; k < k7; k++) h * .5 + (k - 3) * spread + shift];
  for (var k = 0; k < k7; k++) {
    if ((ys[k] - target.dy).abs() < bd) {
      bd = (ys[k] - target.dy).abs();
      best = k;
    }
  }
  for (var k = 0; k < k7; k++) {
    final col = _h(k / k7 * .8, 1, .58), sel = k == best;
    c.drawPath(_poly([exit, Offset(w - 26, ys[k] - (sel ? 5 : 3.5)), Offset(w - 26, ys[k] + (sel ? 5 : 3.5))]), _f(_al(col, sel ? 1 : .55)));
  }
  c.drawPath(_poly(tri), _f(_al(_wh, .1)));
  c.drawPath(_poly(tri), _s(_wh, 1.6));
  c.drawCircle(target, 10, _f(_h(best / k7 * .8, 1, .58)));
  c.drawCircle(target, 10, _s(_wh, 1.5));
}

void _fChameleon(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  for (var i = 0; i < 5; i++) {
    c.save();
    c.translate(_rn(i) * s.width, 10 + _rn(i + 9) * 30);
    c.rotate(_rn(i + 4) * 3);
    c.drawOval(const Rect.fromLTWH(-4, -9, 8, 18), _f(_al(_li, .5)));
    c.restore();
  }
  final y0 = s.height * .78;
  c.drawLine(Offset(0, y0 + 6), Offset(s.width, y0 + 2), _s(const Color(0xFF8A5A33), 6));
  final col = _h(st.a, st.b, .55), dark = _h(st.a + .05, st.b, .38);
  final tail = Path()..moveTo(48, y0 - 16);
  for (var k = 0; k <= 30; k++) {
    final a = k / 30 * math.pi * 3.2, r = 16 * (1 - k / 34);
    tail.lineTo(36 - math.sin(a) * r * .9, y0 - 16 + (1 - math.cos(a)) * r * .45);
  }
  c.drawPath(tail, _s(col, 6));
  final body = Path()
    ..moveTo(46, y0 - 8)
    ..quadraticBezierTo(70, y0 - 50, 112, y0 - 26)
    ..lineTo(128, y0 - 22)
    ..quadraticBezierTo(124, y0 - 8, 104, y0 - 8)
    ..close();
  c.drawPath(body, _f(col));
  for (var k = 0; k < 4; k++) {
    c.drawCircle(Offset(62.0 + k * 11, y0 - 22 - (k == 1 || k == 2 ? 6 : 0)), 3, _f(dark));
  }
  for (final x in [60.0, 98.0]) {
    c.drawLine(Offset(x, y0 - 10), Offset(x + 4, y0 + 2), _s(col, 4));
  }
  final eye = const Offset(112, -32) + Offset(0, y0);
  c.drawCircle(eye, 7, _f(dark));
  c.drawCircle(eye, 5, _f(_cr));
  c.drawCircle(eye + Offset(math.cos(t * .9) * 2, math.sin(t * 1.3) * 2), 2.2, _f(_k0));
}

void _fLava(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  final cx = s.width / 2, top = 14.0, bot = s.height - 26;
  final glass = Path()
    ..moveTo(cx - 10, top)
    ..lineTo(cx + 10, top)
    ..lineTo(cx + 22, bot)
    ..lineTo(cx - 22, bot)
    ..close();
  final ct = _h(st.a, .95, .55), cb = _h(st.b, .95, .55);
  c.drawPath(glass, Paint()..shader = ui.Gradient.linear(Offset(0, top), Offset(0, bot), [ct, cb]));
  c.save();
  c.clipPath(glass);
  for (var k = 0; k < 4; k++) {
    final q = .5 + .45 * math.sin(t * (.4 + k * .13) + k * 2);
    final y = top + q * (bot - top), col = Color.lerp(ct, cb, q)!;
    c.drawOval(Rect.fromCenter(center: Offset(cx + (k - 1.5) * 6, y), width: 12 + k * 2, height: 14 + k * 1.5), _f(Color.lerp(col, _wh, .35)!));
  }
  c.restore();
  c.drawPath(glass, _s(_al(_wh, .35), 1));
  c.drawPath(_poly([Offset(cx - 9, top), Offset(cx + 9, top), Offset(cx + 6, top - 8), Offset(cx - 6, top - 8)]), _f(const Color(0xFF55555C)));
  c.drawPath(_poly([Offset(cx - 22, bot), Offset(cx + 22, bot), Offset(cx + 14, s.height - 6), Offset(cx - 14, s.height - 6)]), _f(const Color(0xFF55555C)));
  c.drawCircle(Offset(cx, bot + 4), 30, _f(_al(cb, .08)));
}

void _fBlush(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _cy);
  final c0 = Offset(s.width / 2, s.height * .56), r = 36.0, b = st.b;
  c.drawCircle(c0, r, _f(Color.lerp(_cr, const Color(0xFFFFC2B0), b * .6)!));
  final hair = _h(st.a, .85, .48);
  c.drawPath(
      Path()
        ..moveTo(c0.dx - r - 2, c0.dy + 4)
        ..arcTo(Rect.fromCircle(center: c0, radius: r + 3), math.pi * .97, math.pi * 1.06, false)
        ..lineTo(c0.dx + r * .4, c0.dy - r * .45)
        ..lineTo(c0.dx + r * .1, c0.dy - r * .25)
        ..lineTo(c0.dx - r * .3, c0.dy - r * .48)
        ..lineTo(c0.dx - r * .7, c0.dy - r * .2)
        ..close(),
      _f(hair));
  final ey = c0.dy + 2;
  for (final sx in [-1.0, 1.0]) {
    final e = Offset(c0.dx + sx * 13, ey);
    if (b > .55) {
      c.drawArc(Rect.fromCircle(center: e + const Offset(0, 2), radius: 4), math.pi * 1.1, math.pi * .8, false, _s(_k0, 2));
    } else {
      c.drawCircle(e, 2.6, _f(_k0));
    }
    c.drawOval(Rect.fromCenter(center: Offset(c0.dx + sx * 20, ey + 10), width: 13, height: 8), _f(_al(_pk, b)));
  }
  c.drawArc(Rect.fromCenter(center: Offset(c0.dx, ey + 11), width: 8, height: 6), .2, math.pi - .4, false, _s(_k0, 1.8));
  if (b > .8) {
    for (var k = 0; k < 2; k++) {
      final q = _wr(t * .8 + k * .5);
      c.drawCircle(Offset(c0.dx + (k == 0 ? -24 : 26), c0.dy - r - 4 - q * 10), 3 + q * 4, _f(_al(_wh, 1 - q)));
    }
  }
}

void _fGlass(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _vi);
  final cx = s.width / 2, top = 12.0, bot = s.height - 10, tw = 28.0, bw = 22.0;
  final glass = _poly([Offset(cx - tw, top), Offset(cx + tw, top), Offset(cx + bw, bot), Offset(cx - bw, bot)]);
  final lt = top + 14 + math.sin(t * 2) * .6;
  final m = .3 + st.a * .55, soft = .02 + st.b * .3;
  c.save();
  c.clipPath(glass);
  c.drawRect(
      Rect.fromLTRB(cx - tw, lt, cx + tw, bot),
      Paint()
        ..shader = ui.Gradient.linear(Offset(0, lt), Offset(0, bot), [_ye, _ye, _or, _rd, _rd],
            [0, (m - soft).clamp(0.0, 1.0), m, (m + soft).clamp(0.0, 1.0), 1]));
  c.drawOval(Rect.fromLTRB(cx - tw + 1, lt - 3, cx + tw - 1, lt + 3), _f(const Color(0xFFFFE680)));
  for (var k = 0; k < 3; k++) {
    final o = Offset(cx - 14 + k * 13.0, lt + 6 + (k.isOdd ? 6 : 0));
    c.save();
    c.translate(o.dx, o.dy);
    c.rotate(.3 * k - .2);
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-6, -6, 12, 12), const Radius.circular(3)), _f(_al(_wh, .45)));
    c.restore();
  }
  c.drawRect(Rect.fromLTRB(cx + tw - 8, top, cx + tw - 4, bot), _f(_al(_wh, .25)));
  c.restore();
  c.drawPath(glass, _s(_wh, 1.8));
  c.drawLine(Offset(cx + 6, bot - 12), Offset(cx + 26, top - 8), _s(_pk, 4));
  final sl = Offset(cx - tw + 2, top);
  c.drawArc(Rect.fromCircle(center: sl, radius: 11), math.pi, math.pi, true, _f(_or));
  c.drawArc(Rect.fromCircle(center: sl, radius: 8.5), math.pi, math.pi, true, _f(const Color(0xFFFFB066)));
}

void _fSpray(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, const Color(0xFF3A2E35));
  for (var r = 0; r < 9; r++) {
    final y = r * 13.0;
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_k0, 1.5));
    for (var x = (r.isEven ? 0.0 : 14.0); x < s.width; x += 28) {
      c.drawLine(Offset(x, y), Offset(x, y + 13), _s(_k0, 1.5));
    }
  }
  final pts = st.pts.length > 1 ? st.pts : [for (var i = 0; i < 62; i++) Offset(12 + i * 1.75, s.height * .5 + math.sin(i * .16) * 26)];
  for (var i = 0; i < pts.length; i++) {
    final col = _h(i * .011 + .9, 1, .6), p = pts[i];
    c.drawCircle(p, 5, _f(_al(col, .9)));
    for (var k = 0; k < 2; k++) {
      c.drawCircle(p + Offset(_rn(i * 3 + k) - .5, _rn(i * 5 + k) - .5) * 18, .9, _f(col));
    }
    if (i % 11 == 5) c.drawLine(p, p + Offset(0, 8 + _rn(i) * 8), _s(col, 1.4));
  }
  final can = pts.last + const Offset(10, 6);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(can.dx, can.dy, 10, 19), const Radius.circular(2)), _f(_wh));
  c.drawRect(Rect.fromLTWH(can.dx, can.dy + 6, 10, 6), _f(_h(pts.length * .011 + .9, 1, .6)));
  c.drawRect(Rect.fromLTWH(can.dx + 3, can.dy - 3, 4, 3), _f(_k0));
}

void _fFish(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _bl);
  for (var k = 0; k < 4; k++) {
    final q = _wr(t * .3 + k * .25);
    c.drawCircle(Offset(s.width * .82 + math.sin(q * 9 + k) * 3, s.height * (1 - q)), 2 + k * .6, _s(_al(_wh, .7), 1));
  }
  final sw = math.sin(t * 3) * .06;
  final c0 = Offset(s.width * .46, s.height * .5);
  c.save();
  c.translate(c0.dx, c0.dy);
  c.rotate(sw);
  final bodyR = const Rect.fromLTWH(-38, -22, 76, 44);
  final fish = Path()
    ..addOval(bodyR)
    ..addPath(_poly(const [Offset(-32, 0), Offset(-52, -18), Offset(-48, 0), Offset(-52, 18)]), Offset.zero)
    ..addPath(_poly(const [Offset(-12, -18), Offset(4, -32), Offset(14, -18)]), Offset.zero);
  final base = _h(.06 + st.b, 1, .56);
  c.drawPath(fish, _f(base));
  c.save();
  c.clipPath(fish);
  for (var k = 0; k < 3; k++) {
    final x = -36 + (k * .36 + st.a * .3) * 76, wdt = 7.0 + (k == 1 ? 3 : 0);
    c.drawRect(Rect.fromLTWH(x - 1.6, -30, wdt + 3.2, 60), _f(_k0));
    c.drawRect(Rect.fromLTWH(x, -30, wdt, 60), _f(_wh));
  }
  c.restore();
  c.drawCircle(const Offset(26, -5), 5, _f(_wh));
  c.drawCircle(const Offset(27, -5), 2.6, _f(_k0));
  c.restore();
}

void _fRain(Canvas c, Size s, _P8 st, double t) {
  final b = st.b, w = s.width, gy = s.height - 22;
  c.drawRect(Offset.zero & s, _f(Color.lerp(const Color(0xFF8C9096), const Color(0xFF7FE0FF), b)!));
  if (b > .55) c.drawCircle(Offset(w - 22, 20), 8 + (b - .55) * 18, _f(_ye));
  final cl = Color.lerp(const Color(0xFF5B5E66), _wh, b)!;
  for (final o in const [Offset(20, 14), Offset(36, 8), Offset(54, 14), Offset(30, 18), Offset(46, 18)]) {
    c.drawCircle(o, 11, _f(cl));
  }
  final drops = ((1 - b) * 46).round();
  for (var i = 0; i < drops; i++) {
    final x = _rn(i) * w, y = _wr(t * (1.1 + _rn(i + 50) * .5) + _rn(i + 9)) * (gy + 8);
    c.drawLine(Offset(x, y), Offset(x - 2, y + 7), _s(_al(_wh, .8), 1.4));
  }
  c.drawRect(Rect.fromLTWH(0, gy, w, 22), _f(_h(.28, b, .5)));
  const hues = [.0, .12, .6, .85, .33];
  for (var k = 0; k < 5; k++) {
    final x = 8.0 + k * 29, hh = 22.0 + (k % 2) * 8;
    c.drawRect(Rect.fromLTWH(x, gy - hh, 22, hh), _f(_h(hues[k], b, .58)));
    c.drawPath(_poly([Offset(x - 3, gy - hh), Offset(x + 25, gy - hh), Offset(x + 11, gy - hh - 12)]), _f(_h(hues[(k + 2) % 5], b, .45)));
    c.drawRect(Rect.fromLTWH(x + 7, gy - hh + 6, 8, 7), _f(_h(.14, b, .7)));
  }
}

void _fAurora(Canvas c, Size s, _P8 st, double t) {
  final w = s.width, h = s.height;
  c.drawRect(Offset.zero & s, Paint()..shader = ui.Gradient.linear(Offset.zero, Offset(0, h), [_k0, const Color(0xFF0D1B4D)]));
  for (var i = 0; i < 20; i++) {
    c.drawCircle(Offset(_rn(i) * w, _rn(i + 30) * h * .6), .9, _f(_al(_wh, .7)));
  }
  final b = st.b;
  for (var x = 0.0; x < w; x += 3) {
    final top = 12 + math.sin(x * .045 + t * .7) * 9, len = 46 + math.sin(x * .09 + t * 1.1) * 14;
    final col = _h(st.a + x / w * .18, .95, .6);
    c.drawRect(
        Rect.fromLTWH(x, top, 3.2, len),
        Paint()
          ..shader = ui.Gradient.linear(Offset(0, top), Offset(0, top + len), [_al(col, 0), _al(col, .25 * b), _al(col, .95 * b)], [0, .5, 1]));
  }
  c.drawPath(
      Path()
        ..moveTo(0, h)
        ..lineTo(0, h - 22)
        ..lineTo(30, h - 38)
        ..lineTo(52, h - 26)
        ..lineTo(84, h - 44)
        ..lineTo(118, h - 24)
        ..lineTo(w, h - 34)
        ..lineTo(w, h)
        ..close(),
      _f(const Color(0xFF07070A)));
}

void _fThermal(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  const cell = 6.0;
  const pal = [Color(0xFF14083F), Color(0xFF4B1FA8), _pk, _rd, _or, _ye, _wh];
  final hc = Offset(s.width * .5, s.height * .4), bc = Offset(s.width * .52, s.height * .86);
  final ears = Path()
    ..addPolygon([hc + const Offset(-22, -6), hc + const Offset(-20, -34), hc + const Offset(-4, -18)], true)
    ..addPolygon([hc + const Offset(22, -6), hc + const Offset(20, -34), hc + const Offset(4, -18)], true);
  for (var y = 0.0; y < s.height; y += cell) {
    for (var x = 0.0; x < s.width; x += cell) {
      final p = Offset(x + cell / 2, y + cell / 2);
      final dh = (p - hc).distance / 22, db = Offset((p.dx - bc.dx) / 34, (p.dy - bc.dy) / 22).distance;
      final ear = ears.contains(p);
      var tt = .12 + .05 * p.dy / s.height;
      if (dh < 1) tt = math.max(tt, .62 + .38 * (1 - dh));
      if (db < 1) tt = math.max(tt, .55 + .3 * (1 - db));
      if (ear) tt = math.max(tt, .7);
      tt += (_rn((x * 7 + y * 131).toInt()) - .5) * .06 + math.sin(t * 2 + x * .1) * .015;
      final v = ((tt - .5) * (.6 + st.b * 1.8) + .5 + (st.a - .5) * .7).clamp(0.0, .999) * (pal.length - 1);
      c.drawRect(Rect.fromLTWH(x, y, cell + .3, cell + .3), _f(Color.lerp(pal[v.floor()], pal[v.floor() + 1], v - v.floor())!));
    }
  }
  for (final o in [const Offset(8, 8), Offset(s.width - 8, 8), Offset(8, s.height - 8), Offset(s.width - 8, s.height - 8)]) {
    final sx = o.dx < 20 ? 1.0 : -1.0, sy = o.dy < 20 ? 1.0 : -1.0;
    c.drawLine(o, o + Offset(8 * sx, 0), _s(_wh, 1.5));
    c.drawLine(o, o + Offset(0, 8 * sy), _s(_wh, 1.5));
  }
  c.drawCircle(hc, 4, _s(_wh, 1.2));
}

void _fStained(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  final win = Rect.fromLTWH(s.width / 2 - 40, 6, 80, s.height - 12);
  final arch = Path()
    ..moveTo(win.left, win.bottom)
    ..lineTo(win.left, win.top + 40)
    ..arcToPoint(Offset(win.right, win.top + 40), radius: const Radius.circular(40))
    ..lineTo(win.right, win.bottom)
    ..close();
  const nx = 5, ny = 6;
  Offset v(int i, int j) {
    final e = i == 0 || j == 0 || i == nx || j == ny;
    return Offset(win.left + win.width * i / nx + (e ? 0 : (_rn(i * 17 + j) - .5) * 11), win.top + win.height * j / ny + (e ? 0 : (_rn(i * 5 + j * 29) - .5) * 11));
  }

  final pts = st.pts.isNotEmpty ? st.pts : const <Offset>[];
  c.save();
  c.clipPath(arch);
  var n = 0;
  for (var j = 0; j < ny; j++) {
    for (var i = 0; i < nx; i++) {
      for (final tri in [
        [v(i, j), v(i + 1, j), v(i + 1, j + 1)],
        [v(i, j), v(i + 1, j + 1), v(i, j + 1)],
      ]) {
        n++;
        final path = _poly(tri);
        final lit = st.pts.isEmpty ? _rn(n * 3) > .45 : pts.any(path.contains);
        c.drawPath(path, _f(lit ? _h(_rn(n) * .9 + .55, .95, .58) : const Color(0xFF34343A)));
        c.drawPath(path, _s(_k0, 2.4));
      }
    }
  }
  c.restore();
  c.drawPath(arch, _s(_cr, 2.5));
}

void _fNeon(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, const Color(0xFF17151C));
  for (var r = 0; r < 9; r++) {
    c.drawLine(Offset(0, r * 13.0 + 6), Offset(s.width, r * 13.0 + 6), _s(const Color(0xFF221F29), 1));
  }
  final c0 = s.center(const Offset(0, 4));
  final p = _heart(c0, 34);
  final on = st.b * (.94 + .06 * math.sin(t * 37) * math.sin(t * 5));
  final col = _h(st.a, on, .6);
  for (final (wd, a) in [(18.0, .14), (11.0, .26), (6.0, .55)]) {
    c.drawPath(p, _s(_al(col, a * on), wd));
  }
  c.drawPath(p, _s(Color.lerp(const Color(0xFF6E6A75), Color.lerp(col, _wh, .55)!, on)!, 2.6));
  for (final o in [c0 + const Offset(-26, -30), c0 + const Offset(26, -30), c0 + const Offset(0, 30)]) {
    c.drawCircle(o, 1.8, _f(const Color(0xFF8A8690)));
  }
}

void _fPole(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _cr);
  final cx = s.width / 2, body = Rect.fromLTWH(cx - 15, 18, 30, s.height - 36);
  c.save();
  c.clipRRect(RRect.fromRectAndRadius(body, const Radius.circular(4)));
  c.drawRect(body, _f(_wh));
  const per = 22.0;
  final ph = _wr(st.a + t * .25) * per * 2, rw = per * (.2 + st.b * .6);
  for (var k = -6; k < 8; k++) {
    final x0 = body.left + k * per * 2 + ph - 40;
    c.drawPath(_poly([Offset(x0, body.bottom), Offset(x0 + rw, body.bottom), Offset(x0 + rw + body.height * .55, body.top), Offset(x0 + body.height * .55, body.top)]), _f(_rd));
    final x1 = x0 + per;
    c.drawPath(_poly([Offset(x1, body.bottom), Offset(x1 + rw * .45, body.bottom), Offset(x1 + rw * .45 + body.height * .55, body.top), Offset(x1 + body.height * .55, body.top)]), _f(_bl));
  }
  c.drawRect(
      body,
      Paint()
        ..shader = ui.Gradient.linear(body.centerLeft, body.centerRight, [_al(_k0, .35), _al(_k0, 0), _al(_wh, .35), _al(_k0, 0), _al(_k0, .4)], [0, .3, .42, .6, 1]));
  c.restore();
  for (final y in [body.top - 8, body.bottom]) {
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(cx - 19, y, 38, 8), const Radius.circular(3)), _f(_k1));
  }
  c.drawCircle(Offset(cx, body.top - 13), 7, _f(_k1));
}

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
