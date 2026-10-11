part of 'pop_06.dart';

// Mask feather / reveal: v = softness of the edge, w = how much is revealed (expand), tap = invert.
const pop06MaskPanels = <PopPanel>[
  PopPanel('Spotlight', 'theatre·bold flat·2D·drag', _m1, v0: .4),
  PopPanel('Cookie cutter', 'food·light·top-down·tap flips', _m2, light: true, v0: .45),
  PopPanel('Fogged window', 'weather·crisp·2D·rub to wipe', _m3, v0: .5),
  PopPanel('Porthole box', 'machine·bold flat·isometric', _m4, v0: .4, w0: .75),
  PopPanel('Synth sunset', 'landscape·neon wild·2D·drag', _m5, v0: .55),
  PopPanel('Jellyfish', 'creature·neon·2D·drag', _m6, v0: .5),
  PopPanel('Halftone print', 'print·light·flat·drag', _m7, light: true, v0: .55),
  PopPanel('Paint roller', 'machine·bold flat·2D·roll', _m8, v0: .62, w0: .5),
  PopPanel('Island & shoals', 'landscape·flat·top-down map', _m9, v0: .6),
  PopPanel('Eclipse corona', 'cosmic·neon wild·2D·drag', _m10, v0: .6),
  PopPanel('Melting ice', 'material·crisp·pseudo 3D', _m11, v0: .5),
  PopPanel('Spray stencil', 'toy·bold flat·2D·shake', _m12, v0: .5),
  PopPanel('Donut hole', 'food·neon·top-down·tap flips', _m13, v0: .5),
  PopPanel('Camera iris', 'machine·crisp·2D·spin', _m14, v0: .45),
  PopPanel('Fuzzy monster', 'character·bold flat·2D', _m15, v0: .55),
  PopPanel('Pond ripple', 'nature·crisp·top-down·throw', _m16, v0: .45),
  PopPanel('Blown dust', 'physics·neon·2D·blow', _m17, v0: .55),
  PopPanel('LED wall', 'machine·neon wild·2D grid', _m18, v0: .5),
  PopPanel('Paper crater', 'toy·flat·stacked 3D·stack', _m19, v0: .5),
  PopPanel('Rain cloud', 'weather·bold flat·2D·drag', _m20, v0: .5),
];

void _m1(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = Offset(s.width * .5, s.height * .64);
  final rad = _l(12, 42, st.w), soft = st.v * 26, lamp = Offset(s.width * .5, 8);
  c.drawRect(Rect.fromLTRB(0, s.height * .8, s.width, s.height), _f(_p0));
  final beam = Path()
    ..moveTo(lamp.dx - 6, lamp.dy)
    ..lineTo(ctr.dx - rad * 1.4 - soft / 2, ctr.dy)
    ..lineTo(ctr.dx + rad * 1.4 + soft / 2, ctr.dy)
    ..lineTo(lamp.dx + 6, lamp.dy)
    ..close();
  c.drawPath(beam, _f(_a(_ye, .14)));
  _masked(c, r, () => _scene(c, r), () => _bands(c, (g) => Path()..addOval(Rect.fromCenter(center: ctr, width: math.max(1, (rad + g) * 2.8), height: math.max(1, (rad + g) * 1.5))), soft),
      inv: st.inv);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: lamp, width: 26, height: 12), const Radius.circular(3)), _f(_or));
  c.drawCircle(lamp + const Offset(0, 6), 4, _f(_ye));
}

void _m2(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = s.center(Offset.zero), ro = _l(20, 50, st.w);
  for (var i = 0; i < 26; i++) {
    c.drawCircle(Offset(_h(i) * s.width, _h(i + 50) * s.height), 1.2, _f(const Color(0xFFE2CDAE)));
  }
  final star = _star(ctr, ro, ro * .52, 5);
  _masked(c, r, () => _scene(c, r), () => c.drawPath(star, _f(_wh)), inv: st.inv);
  final m = star.computeMetrics().first, n = (st.v * 46).round();
  for (var i = 0; i < n; i++) {
    final tg = m.getTangentForOffset(m.length * _h(i + 7))!;
    final out = Offset(tg.vector.dy, -tg.vector.dx);
    final q = tg.position + out * (st.v * 14 * _h(i + 31)) * (st.inv ? -1 : 1);
    c.drawCircle(q, 1.2 + 2 * _h(i + 3), _f(st.inv ? _sceneAt(q, s) : const Color(0xFFD9B98C)));
  }
  c.drawPath(star, _s(_p0, 2));
}

void _m3(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s;
  _scene(c, r);
  final pts = st.trail.length > 2 ? st.trail : [for (var k = 0; k <= 20; k++) Offset(22 + k * 5.6, s.height * .5 + 20 * math.sin(k * .45))];
  final path = _poly(pts), rad = _l(5, 20, st.w), soft = st.v * 16;
  c.saveLayer(r, Paint());
  c.drawRect(r, _f(_a(const Color(0xFFDDE7EE), .9)));
  for (var i = 0; i < 4; i++) {
    final w = 2 * (rad + soft * (1 - i / 3));
    c.drawPath(path, _s(_a(_wh, i == 3 ? 1 : .3), w)..blendMode = st.inv ? BlendMode.dstIn : BlendMode.dstOut);
  }
  c.restore();
  for (var i = 0; i < pts.length; i += 5) {
    final q = pts[i] + Offset(0, rad + 2);
    c.drawLine(q, q + Offset(0, 6 + 14 * _h(i)), _s(_a(_cy, .9), 2));
  }
}

void _m4(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s;
  const a = 44.0;
  final t = Offset(s.width * .5, 16), rv = Offset(math.cos(math.pi / 6) * a, a / 2), lv = Offset(-rv.dx, rv.dy), dv = const Offset(0, 46);
  final top = _poly([t, t + rv, t + rv + lv, t + lv], close: true);
  final left = _poly([t + lv, t + lv + rv, t + lv + rv + dv, t + lv + dv], close: true);
  final right = _poly([t + rv + lv, t + rv, t + rv + dv, t + rv + lv + dv], close: true);
  c.drawPath(top, _f(_ye));
  c.drawPath(left, _f(_or));
  c.drawPath(right, _f(_rd));
  Offset m(double u, double v) => t + lv + rv * u + dv * v;
  Path hole(double g) {
    final rho = math.max(.02, _l(.12, .4, st.w) + g);
    return _poly([for (var i = 0; i < 40; i++) m(.5 + rho * math.cos(i / 40 * math.pi * 2), .5 + rho * math.sin(i / 40 * math.pi * 2))], close: true);
  }

  _masked(c, r, () => _scene(c, r), () {
    if (st.inv) {
      c.drawPath(left, _f(_wh));
      _bands(c, hole, st.v * .16, mode: BlendMode.dstOut);
    } else {
      _bands(c, hole, st.v * .16);
    }
  });
  c.drawPath(hole(0), _s(_p0, 1.6));
  for (var i = 0; i < 8; i++) {
    final ang = i / 8 * math.pi * 2, rho = _l(.12, .4, st.w) + .06;
    c.drawCircle(m(.5 + rho * math.cos(ang), .5 + rho * math.sin(ang)), 1.6, _f(_p0));
  }
  for (final p in [top, left, right]) {
    c.drawPath(p, _s(_p0, 1.4));
  }
}

void _m5(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, y0 = _l(s.height * .9, s.height * .2, st.w), soft = st.v * 44;
  c.drawRect(r, _f(const Color(0xFF14102A)));
  for (var i = 0; i < 9; i++) {
    final x = s.width / 2 + (i - 4) * 10.0;
    c.drawLine(Offset(x, s.height * .62), Offset(s.width / 2 + (i - 4) * 46.0, s.height), _s(_a(_pk, .6), 1));
  }
  for (var i = 0; i < 5; i++) {
    final y = s.height * .62 + math.pow(i / 4, 1.8) * s.height * .38;
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_a(_pk, .6), 1));
  }
  _masked(c, r, () => _scene(c, r), () {
    c.drawRect(Rect.fromLTRB(0, 0, s.width, y0), _f(_wh));
    for (var i = 0; i < 6; i++) {
      final y = y0 + soft * i / 6, th = soft / 6 * (1 - i / 6);
      if (th > .4) c.drawRect(Rect.fromLTRB(0, y, s.width, y + th), _f(_wh));
    }
  }, inv: st.inv);
  c.drawLine(Offset(0, y0), Offset(s.width, y0), _s(_cy, 1.4));
}

void _m6(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, bob = math.sin(st.t * 2) * 3;
  final ctr = Offset(s.width * .5, 46 + bob), rb = _l(18, 38, st.w), soft = st.v * 14;
  c.drawRect(r, _f(const Color(0xFF0E1440)));
  for (var i = 0; i < 7; i++) {
    final y = (s.height - (st.t * 12 + _h(i) * 120) % 130);
    c.drawCircle(Offset(_h(i + 9) * s.width, y), 1.5 + _h(i + 2) * 2, _s(_a(_cy, .6), 1));
  }
  for (var k = 0; k < 6; k++) {
    final x0 = ctr.dx - rb * .8 + k * rb * .32, len = 18 + st.v * 44;
    final p = Path()..moveTo(x0, ctr.dy + rb * .3);
    for (var j = 1; j <= 8; j++) {
      p.lineTo(x0 + math.sin(st.t * 3 + k + j * .8) * 4, ctr.dy + rb * .3 + len * j / 8);
    }
    c.drawPath(p, _s(_a(k.isEven ? _pk : _cy, .85), 2));
  }
  Path bell(double g) {
    final rr = math.max(2.0, rb + g);
    final p = Path()..addArc(Rect.fromCircle(center: ctr, radius: rr), math.pi, math.pi);
    for (var i = 0; i <= 6; i++) {
      p.lineTo(ctr.dx + rr - i * rr / 3, ctr.dy + rr * .3 + (i.isOdd ? rr * .12 : 0));
    }
    return p..close();
  }

  _masked(c, r, () => _scene(c, r), () => _bands(c, bell, soft, n: 4), inv: st.inv);
  c.drawPath(bell(soft / 2), _s(_a(_pk, .9), 1.5));
  c.drawCircle(ctr + Offset(-rb * .3, -rb * .15), 2.4, _f(_wh));
  c.drawCircle(ctr + Offset(rb * .3, -rb * .15), 2.4, _f(_wh));
}

void _m7(Canvas c, Size s, PopS st) {
  final ctr = s.center(Offset.zero), rad = _l(14, 52, st.w), soft = 3 + st.v * 44;
  for (var y = 5.0; y < s.height; y += 7) {
    for (var x = 5.0 + ((y ~/ 7).isOdd ? 3.5 : 0); x < s.width; x += 7) {
      final d = (Offset(x, y) - ctr).distance;
      var m = ((rad + soft / 2 - d) / soft).clamp(0.0, 1.0);
      if (st.inv) m = 1 - m;
      if (m <= .02) continue;
      c.drawCircle(Offset(x + 1.4, y + 1.2), m * 3.1, _f(_a(_pk, .85)));
      c.drawCircle(Offset(x, y), m * 3.3, _f(_p0));
    }
  }
}

void _m8(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, xr = _l(14, s.width - 14, st.v), soft = st.w * 30;
  for (var y = 0.0; y < s.height; y += 12) {
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_a(_cr, .08), 1));
    for (var x = (y ~/ 12).isOdd ? 0.0 : 14.0; x < s.width; x += 28) {
      c.drawLine(Offset(x, y), Offset(x, y + 12), _s(_a(_cr, .08), 1));
    }
  }
  _masked(c, r, () => _scene(c, r), () {
    c.drawRect(Rect.fromLTRB(0, 0, xr, s.height), _f(_wh));
    for (var i = 0; i < 40; i++) {
      final y = i * 3.0;
      c.drawRect(Rect.fromLTRB(xr, y, xr + soft * _h(i + 4), y + 3), _f(_a(_wh, .55)));
      c.drawRect(Rect.fromLTRB(xr, y, xr + soft * _h(i + 4) * .5, y + 3), _f(_a(_wh, .6)));
    }
  }, inv: st.inv);
  final roll = Rect.fromLTRB(xr - 6, 10, xr + 6, s.height - 26);
  c.drawRRect(RRect.fromRectAndRadius(roll, const Radius.circular(6)), _f(_or));
  for (var y = roll.top + 4; y < roll.bottom - 2; y += 5) {
    c.drawLine(Offset(roll.left + 2, y + (st.v * 40) % 5), Offset(roll.right - 2, y + (st.v * 40) % 5), _s(_a(_p0, .35), 1));
  }
  c.drawPath(_poly([Offset(xr, roll.bottom), Offset(xr, roll.bottom + 8), Offset(xr + 14, roll.bottom + 8), Offset(xr + 14, s.height)]), _s(_cr, 2.4));
}

void _m9(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = s.center(Offset.zero), rad = _l(12, 40, st.w), soft = st.v * 30;
  c.drawRect(r, _f(_bl));
  for (var i = 3; i >= 1; i--) {
    c.drawPath(_blob(ctr, rad + soft * i / 3), _f(_a(_cy, .22 + .12 * (3 - i))));
  }
  _masked(c, r, () => _scene(c, r), () => c.drawPath(_blob(ctr, rad), _f(_wh)), inv: st.inv);
  c.drawPath(_blob(ctr, rad), _s(_li, 2));
  for (var i = 0; i < 5; i++) {
    final q = _pol(ctr, rad + soft + 8 + 6 * _h(i), i * 1.3 + st.t * .1);
    c.drawLine(q, q + const Offset(6, 0), _s(_a(_wh, .5), 1.2));
  }
}

void _m10(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = s.center(Offset.zero), rad = _l(14, 36, st.w), soft = st.v * 30;
  c.drawRect(r, _f(const Color(0xFF0B0B10)));
  for (var i = 0; i < 18; i++) {
    c.drawCircle(Offset(_h(i) * s.width, _h(i + 40) * s.height), .9, _f(_a(_cr, .6)));
  }
  for (var i = 0; i < 16; i++) {
    final a = i / 16 * math.pi * 2 + st.t * .15, len = rad + 4 + soft * (.6 + .6 * _h(i));
    c.drawLine(_pol(ctr, rad, a), _pol(ctr, len, a), _s(_a(i.isEven ? _ye : _or, .9), i.isEven ? 2.2 : 1.2));
  }
  _masked(c, r, () => _scene(c, r), () {
    for (var i = 0; i < 5; i++) {
      c.drawCircle(ctr, rad + soft * (1 - i / 5) + 2, _f(_a(_wh, .3)));
    }
  }, inv: st.inv);
  c.drawCircle(ctr, rad, _f(const Color(0xFF0B0B10)));
  c.drawCircle(ctr, rad, _s(_a(_cr, .5), 1));
}

void _m11(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = Offset(s.width * .5, s.height * .62), pr = _l(14, 52, st.w), soft = st.v * 16;
  for (var x = 0.0; x < s.width; x += 20) {
    c.drawLine(Offset(x, 0), Offset(x, s.height), _s(_a(_cr, .07), 1));
  }
  for (var y = 0.0; y < s.height; y += 20) {
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_a(_cr, .07), 1));
  }
  Path puddle(double g) => Path()..addOval(Rect.fromCenter(center: ctr, width: math.max(1, (pr + g) * 2.4), height: math.max(1, (pr + g) * 1.1)));
  _masked(c, r, () => _scene(c, r), () => _bands(c, puddle, soft, n: 4), inv: st.inv);
  c.drawPath(puddle(soft / 2), _s(_a(_cy, .9), 1.4));
  final a = _l(24, 5, st.w), t = ctr - Offset(0, a * 1.2), rv = Offset(a * .87, a * .5), lv = Offset(-a * .87, a * .5), dv = Offset(0, a);
  c.drawPath(_poly([t, t + rv, t + rv + lv, t + lv], close: true), _f(_wh));
  c.drawPath(_poly([t + lv, t + lv + rv, t + lv + rv + dv, t + lv + dv], close: true), _f(const Color(0xFFBFEFFF)));
  c.drawPath(_poly([t + rv + lv, t + rv, t + rv + dv, t + rv + lv + dv], close: true), _f(_cy));
  c.drawLine(t + lv * .5 + rv * .3, t + lv * .5 + rv * .6, _s(_wh, 1.4));
}

void _m12(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = Offset(s.width * .44, s.height * .55), k = _l(.5, 1.2, st.w), soft = st.v * 24;
  Path heart(double g) {
    final q = k + g / 30;
    return Path()
      ..moveTo(ctr.dx, ctr.dy + 30 * q)
      ..cubicTo(ctr.dx - 44 * q, ctr.dy, ctr.dx - 26 * q, ctr.dy - 34 * q, ctr.dx, ctr.dy - 14 * q)
      ..cubicTo(ctr.dx + 26 * q, ctr.dy - 34 * q, ctr.dx + 44 * q, ctr.dy, ctr.dx, ctr.dy + 30 * q)
      ..close();
  }

  _masked(c, r, () => _scene(c, r), () => c.drawPath(heart(0), _f(_wh)), inv: st.inv);
  for (var i = 0; i < 140; i++) {
    final a = _h(i) * math.pi * 2, d = 26 * k + soft * _h(i + 300) + 4;
    final q = ctr + Offset(math.cos(a) * d * 1.1, math.sin(a) * d * .95);
    if (heart(0).contains(q) != st.inv) continue;
    c.drawCircle(q, .8 + _h(i + 9) * 1.1, _f(_sceneAt(q, s)));
  }
  final can = Offset(s.width - 24, 14);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(can.dx - 7, can.dy, 14, 26), const Radius.circular(3)), _f(_or));
  c.drawRect(Rect.fromLTWH(can.dx - 3, can.dy - 5, 6, 5), _f(_wh));
  final sh = math.sin(st.t * 18) * st.v * 2;
  for (var i = 0; i < 3; i++) {
    c.drawLine(can + Offset(-8, -1 + sh), can + Offset(-18 - i * 3, 2.0 + i * 5 + sh), _s(_a(_cr, .5), 1));
  }
}

void _m13(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = s.center(Offset.zero), ro = 50.0, rh = _l(8, 28, st.w), soft = st.v * 12;
  c.drawRect(r, _f(_vi));
  c.drawCircle(ctr, ro + 4, _f(_a(_p0, .25)));
  c.drawCircle(ctr, ro, _f(const Color(0xFFE0A15E)));
  final icing = Path();
  for (var i = 0; i <= 64; i++) {
    final a = i / 64 * math.pi * 2, rr = ro - 6 + (st.v * 6) * math.sin(a * 9).abs();
    final q = _pol(ctr, rr, a);
    i == 0 ? icing.moveTo(q.dx, q.dy) : icing.lineTo(q.dx, q.dy);
  }
  icing.close();
  c.drawPath(icing, _f(_pk));
  for (var i = 0; i < 22; i++) {
    final q = _pol(ctr, _l(rh + 8, ro - 10, _h(i + 5)), _h(i) * math.pi * 2);
    _rotRect(c, q, _h(i + 70) * math.pi, const Rect.fromLTWH(-3, -1, 6, 2), _f([_ye, _cy, _wh, _li][i % 4]), rad: 1);
  }
  _masked(c, r, () => _scene(c, r), () {
    if (st.inv) {
      c.drawPath(icing, _f(_wh));
      _bands(c, (g) => _circle(ctr, rh + 4 + g), soft, n: 4, mode: BlendMode.dstOut);
    } else {
      _bands(c, (g) => _circle(ctr, rh + g), soft, n: 4);
    }
  });
  if (!st.inv) c.drawCircle(ctr, rh + soft / 2, _s(_a(_p0, .4), 1.2));
}

void _m14(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = s.center(Offset.zero), ap = _l(8, 44, st.w), rot = st.w * 1.6, soft = st.v * 20;
  c.drawCircle(ctr, 56, _f(const Color(0xFF3A3A40)));
  c.drawCircle(ctr, 56, _s(_a(_cr, .4), 1.2));
  c.drawCircle(ctr, 50, _s(_a(_cr, .2), 1));
  Path hex(double g) => _poly([for (var i = 0; i < 6; i++) _pol(ctr, math.max(1, ap + g), rot + i * math.pi / 3)], close: true);
  _masked(c, r, () => _scene(c, r), () => _bands(c, hex, soft, n: 5), inv: st.inv);
  for (var i = 0; i < 6; i++) {
    final p0 = _pol(ctr, ap + soft / 2, rot + i * math.pi / 3);
    final dir = Offset(math.cos(rot + i * math.pi / 3 + 2.1), math.sin(rot + i * math.pi / 3 + 2.1));
    final p1 = p0 + dir * 80;
    c.save();
    c.clipPath(_circle(ctr, 50));
    c.drawLine(p0, p1, _s(_or, 1.6));
    c.restore();
  }
  c.drawPath(hex(soft / 2), _s(_or, 1.6));
}

void _m15(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = Offset(s.width * .5, s.height * .52), br = _l(18, 38, st.w), fur = st.v * 18;
  c.drawRect(r, _f(_ye));
  c.drawOval(Rect.fromCenter(center: Offset(ctr.dx, s.height - 8), width: br * 2.2, height: 8), _f(_a(_p0, .2)));
  for (var i = 0; i < 72; i++) {
    final a = i / 72 * math.pi * 2 + .02 * math.sin(st.t * 4 + i);
    final q0 = _pol(ctr, br - 2, a), q1 = _pol(ctr, br + 2 + fur * (.55 + .45 * _h(i)), a);
    c.drawLine(q0, q1, _s(st.inv ? _p0 : _sceneAt(q1, s), 2.6));
  }
  if (st.inv) {
    c.drawCircle(ctr, br, _f(_p0));
  } else {
    _masked(c, r, () => _scene(c, r), () => c.drawCircle(ctr, br, _f(_wh)));
  }
  final look = st.p == null ? Offset.zero : Offset.fromDirection((st.p! - ctr).direction, 1.6);
  for (final dx in [-.32, .32]) {
    final e = ctr + Offset(dx * br, -br * .25);
    c.drawCircle(e, 5.5, _f(_wh));
    c.drawCircle(e + look, 2.6, _f(_p0));
  }
  c.drawArc(Rect.fromCenter(center: ctr + Offset(0, br * .2), width: br * .6, height: br * .4), .2, math.pi - .4, false, _s(st.inv ? _wh : _p0, 2));
}

void _m16(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = Offset(s.width * .46, s.height * .52), soft = st.v * 22;
  final ph = (st.t * .35) % 1, rr = 6 + ph * _l(20, 70, st.w);
  c.drawRect(r, _f(const Color(0xFF16328F)));
  for (var i = 1; i <= 3; i++) {
    c.drawCircle(ctr, rr + soft / 2 + i * 7, _s(_a(_cy, .55 - i * .15), 1.4));
  }
  _masked(c, r, () => _scene(c, r), () => _bands(c, (g) => _circle(ctr, rr + g), soft, n: 4), inv: st.inv);
  c.drawCircle(ctr, rr + soft / 2, _s(_cy, 1.6));
  final pad = Offset(s.width * .84, s.height * .22);
  c.drawPath(Path()..addArc(Rect.fromCircle(center: pad, radius: 13), .4, math.pi * 2 - .8)..lineTo(pad.dx, pad.dy)..close(), _f(_li));
  c.drawCircle(ctr, 3 * (1 - ph), _f(_cr));
}

void _m17(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, ctr = Offset(s.width * .56, s.height * .5), cr = _l(8, 46, st.w), soft = st.v * 30;
  c.drawRect(r, _f(const Color(0xFF15151A)));
  _masked(c, r, () => _scene(c, r), () => _bands(c, (g) => _circle(ctr, cr + g), soft * .6, n: 4), inv: st.inv);
  for (var i = 0; i < 200; i++) {
    final q0 = Offset(_h(i) * s.width, _h(i + 500) * s.height), v = q0 - ctr, d = v.distance;
    if (d < 1) continue;
    final dir = v / d, edge = cr + soft * _h(i + 900);
    final inside = st.inv ? d > edge + 12 : d < edge;
    final q = inside ? (st.inv ? ctr + dir * (edge + 12) : ctr + dir * edge) : q0;
    final push = (1 - ((d - cr) / 40).clamp(0.0, 1.0)) * (st.inv ? 0 : 1);
    c.drawLine(q, q + dir * (1.5 + push * 7), _s(_a(i % 3 == 0 ? _ye : _cr, .9), 1.6));
  }
  for (var i = 0; i < 3; i++) {
    final y = ctr.dy - 10 + i * 10.0, x = 8 + (st.t * 30 + i * 9) % 18;
    c.drawPath(Path()..moveTo(x, y)..quadraticBezierTo(x + 10, y - 5, x + 18, y), _s(_cy, 1.8));
  }
}

void _m18(Canvas c, Size s, PopS st) {
  final ctr = s.center(Offset.zero), rad = _l(14, 70, st.w), soft = 4 + st.v * 50;
  c.drawRect(Offset.zero & s, _f(const Color(0xFF09090C)));
  const cell = 10.4;
  for (var j = 0; j < 11; j++) {
    for (var i = 0; i < 15; i++) {
      final q = Offset(4 + i * cell + cell / 2, 3 + j * cell + cell / 2);
      final d = (q.dx - ctr.dx).abs() + (q.dy - ctr.dy).abs();
      var m = ((rad + soft / 2 - d) / soft).clamp(0.0, 1.0);
      if (st.inv) m = 1 - m;
      m = (m * 4).round() / 4;
      final col = m == 0 ? const Color(0xFF222228) : Color.lerp(const Color(0xFF222228), _sceneAt(q, s), m)!;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: q, width: 8, height: 8), const Radius.circular(2)), _f(col));
    }
  }
}

void _m19(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, n = 2 + (st.v * 6).round(), base = _l(8, 30, st.w), ctr0 = Offset(s.width * .5, s.height * .62);
  final bottom = Rect.fromCenter(center: ctr0, width: 120, height: 70);
  c.drawRect(r, _f(_cy));
  c.save();
  c.clipRRect(RRect.fromRectAndRadius(bottom, const Radius.circular(4)));
  _scene(c, bottom);
  c.restore();
  const cols = [_or, _ye, _pk, _li, _vi, _wh, _rd, _bl];
  for (var i = 0; i < n; i++) {
    final off = Offset(0, -(i + 1) * 4.0), card = bottom.shift(off);
    final hr = base + (i + 1) * _l(2, 4.5, st.v);
    final hole = Path()..addOval(Rect.fromCenter(center: ctr0 + off, width: hr * 2.4, height: hr * 1.4));
    final sheet = st.inv
        ? hole
        : Path.combine(PathOperation.difference, Path()..addRRect(RRect.fromRectAndRadius(card, const Radius.circular(4))), hole);
    c.drawPath(sheet.shift(const Offset(1.5, 2)), _f(_a(_p0, .3)));
    c.drawPath(sheet, _f(cols[i % cols.length]));
    c.drawPath(sheet, _s(_a(_p0, .5), 1));
  }
}

void _m20(Canvas c, Size s, PopS st) {
  final r = Offset.zero & s, k = _l(.55, 1.15, st.w), soft = st.v * 14, dx = math.sin(st.t * .5) * 6;
  final ctr = Offset(s.width * .5 + dx, s.height * .4);
  c.drawRect(r, _f(const Color(0xFF1D2350)));
  for (var i = 0; i < 12; i++) {
    c.drawCircle(Offset(_h(i + 3) * s.width, _h(i + 33) * s.height * .5), .9, _f(_a(_cr, .6)));
  }
  final puffs = [(const Offset(-26, 6), 16.0), (const Offset(-6, -8), 22.0), (const Offset(18, -2), 18.0), (const Offset(32, 8), 12.0), (const Offset(0, 10), 16.0)];
  Path cloud(double g) {
    var p = Path();
    for (final (o, rr) in puffs) {
      p = Path.combine(PathOperation.union, p, _circle(ctr + o * k, math.max(1, rr * k + g)));
    }
    return p;
  }

  _masked(c, r, () => _scene(c, r), () => _bands(c, cloud, soft, n: 4), inv: st.inv);
  final nd = 4 + (st.v * 14).round();
  for (var i = 0; i < nd; i++) {
    final x = ctr.dx + (_h(i) - .5) * 70 * k, y = ctr.dy + 18 * k + (st.t * 50 + _h(i + 8) * 60) % 44;
    c.drawLine(Offset(x, y), Offset(x - 2, y + 6), _s(_cy, 1.6));
  }
}
