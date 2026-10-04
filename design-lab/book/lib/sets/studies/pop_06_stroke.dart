part of 'pop_06.dart';

// Stroke / Trim paths: v = width (or dash size), w = how much of the path is drawn (trim end / length), tap = dash on/off.
const pop06StrokePanels = <PopPanel>[
  PopPanel('Neon sign', 'sign·neon wild·2D·drag', _s1, v0: .4, w0: .65),
  PopPanel('Toothpaste', 'food·bold flat·2D·squeeze', _s2, v0: .5, w0: .6),
  PopPanel('Snake', 'creature·bold flat·2D·drag', _s3, v0: .5, w0: .7),
  PopPanel('Rail map', 'landscape·light·top-down map', _s4, light: true, v0: .5, w0: .6),
  PopPanel('Fuse & bomb', 'physics·bold flat·2D·drag', _s5, v0: .5, w0: .35),
  PopPanel('Race track', 'toy·bold flat·isometric·spin', _s6, v0: .5, w0: .45),
  PopPanel('Gym ribbon', 'toy·neon·pseudo 3D·spin', _s7, v0: .5, w0: .8),
  PopPanel('Ant trail', 'nature·crisp·2D·drag', _s8, v0: .5, w0: .4),
  PopPanel('Stitches', 'material·light·flat·drag', _s9, light: true, v0: .45, w0: .7),
  PopPanel('Zipper', 'machine·bold flat·2D·pull', _s10, v0: .5, w0: .5),
  PopPanel('Guitar strings', 'instrument·bold flat·2D·fret', _s11, v0: .5, w0: .3),
  PopPanel('Comet tail', 'cosmic·neon·2D orbit', _s12, v0: .5, w0: .4),
  PopPanel('Spaghetti slurp', 'food·character·2D·drag', _s13, v0: .5, w0: .3),
  PopPanel('Lightning', 'weather·neon wild·2D·drag', _s14, v0: .4, w0: .8),
  PopPanel('Signature', 'text-art·light·flat·write', _s15, light: true, v0: .5, w0: .7),
  PopPanel('Garden hose', 'physics·bold flat·2D·aim', _s16, v0: .5, w0: .6),
  PopPanel('Hanging chain', 'machine·bold flat·2D·pull', _s17, v0: .5, w0: .5),
  PopPanel('Rainbow', 'weather·bold flat·landscape', _s18, v0: .5, w0: .75),
  PopPanel('Dominoes', 'toy·physics·pseudo 3D', _s19, v0: .4, w0: .45),
  PopPanel('Laser mirrors', 'machine·neon wild·2D', _s20, v0: .4, w0: .3),
];

void _s1(Canvas c, Size s, PopS st) {
  for (var y = 0.0; y < s.height; y += 10) {
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(const Color(0xFF34343A), 1));
    for (var x = (y ~/ 10).isOdd ? 0.0 : 11.0; x < s.width; x += 22) {
      c.drawLine(Offset(x, y), Offset(x, y + 10), _s(const Color(0xFF34343A), 1));
    }
  }
  final p = Path()
    ..moveTo(16, 84)
    ..cubicTo(36, 14, 74, 20, 62, 64)
    ..cubicTo(52, 100, 98, 102, 100, 60)
    ..cubicTo(102, 22, 142, 28, 142, 84);
  final w = _l(2, 9, st.v), lit = _trim(p, 0, st.w);
  c.drawPath(p, _s(const Color(0xFF45454C), w));
  final flick = st.w > .02 && math.sin(st.t * 23) > .93 ? .5 : 1.0;
  c.drawPath(lit, _s(_a(_pk, .18 * flick), w + 10));
  c.drawPath(lit, _s(_a(_pk, .45 * flick), w + 4));
  if (st.inv) {
    _dash(c, lit, _s(_a(const Color(0xFFFFC2DD), flick), w * .6), 6, 4);
  } else {
    c.drawPath(lit, _s(_a(const Color(0xFFFFC2DD), flick), w * .6));
  }
  for (final f in [.15, .5, .85]) {
    final tg = _at(p, f)!;
    c.drawCircle(tg.position, 2, _f(_a(_cr, .7)));
  }
}

void _s2(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_cy));
  final squeeze = _l(1, .35, st.w), tube = Rect.fromCenter(center: const Offset(26, 50), width: 36, height: 34 * squeeze + 6);
  c.drawPath(_poly([tube.topLeft, tube.topRight + const Offset(0, 4), const Offset(48, 50 - 5), const Offset(48, 50 + 5), tube.bottomRight - const Offset(0, 4), tube.bottomLeft], close: true), _f(_wh));
  c.drawRect(Rect.fromLTWH(tube.left, tube.top, 4, tube.height), _f(_rd));
  c.drawRect(const Rect.fromLTWH(48, 45, 6, 10), _f(_rd));
  final p = Path()
    ..moveTo(55, 50)
    ..cubicTo(80, 46, 70, 84, 96, 84)
    ..cubicTo(120, 84, 112, 56, 146, 60);
  final w = _l(4, 16, st.v), out = _trim(p, 0, st.w);
  c.drawPath(out.shift(const Offset(0, 2)), _s(_a(_p0, .2), w));
  c.drawPath(out, _s(_wh, w));
  if (st.inv) {
    _dash(c, out, _s(_rd, w * .4), 5, 5);
  } else {
    c.drawPath(out, _s(_rd, w * .3));
  }
  c.drawPath(out, _s(_a(_cy, .7), w * .12));
  final tip = _at(p, st.w);
  if (tip != null && st.w > .02) c.drawCircle(tip.position, w * .55, _f(_wh));
}

void _s3(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF1E2A1C)));
  for (var i = 0; i < 10; i++) {
    final q = Offset(_h(i) * s.width, 20 + _h(i + 4) * 96);
    c.drawPath(Path()..moveTo(q.dx - 3, q.dy)..lineTo(q.dx, q.dy - 7)..lineTo(q.dx + 3, q.dy), _s(_a(_li, .35), 1.2));
  }
  final p = _poly([for (var x = 12.0; x <= 146; x += 4) Offset(x, 64 + 20 * math.sin(x * .06 - st.t * 2.4))]);
  final w = _l(4, 18, st.v), body = _trim(p, .02, .02 + _l(.2, .96, st.w));
  c.drawPath(body, _s(const Color(0xFF3E7A2A), w + 3));
  c.drawPath(body, _s(_li, w));
  _dash(c, body, _s(_ye, w * .45, cap: StrokeCap.butt), st.inv ? 3 : 6, st.inv ? 3 : 7);
  final head = _at(p, .02 + _l(.2, .96, st.w))!;
  final ang = -head.angle;
  c.drawCircle(head.position, w * .7 + 2, _f(_li));
  final fwd = Offset(math.cos(ang), math.sin(ang)), side = Offset(-fwd.dy, fwd.dx);
  if (math.sin(st.t * 5) > 0) {
    final tq = head.position + fwd * (w * .7 + 8);
    c.drawPath(_poly([head.position + fwd * (w * .7), tq, tq + fwd * 3 + side * 2, tq, tq + fwd * 3 - side * 2]), _s(_rd, 1.2));
  }
  for (final sd in [-1.0, 1.0]) {
    final e = head.position + fwd * 1.5 + side * sd * (w * .35 + 1);
    c.drawCircle(e, 2.2, _f(_wh));
    c.drawCircle(e + fwd * .8, 1.1, _f(_p0));
  }
}

void _s4(Canvas c, Size s, PopS st) {
  c.drawPath(Path()..moveTo(0, 18)..cubicTo(50, 40, 90, 0, 156, 30), _s(_a(_cy, .7), 9));
  c.drawPath(_blob(const Offset(122, 94), 16, seed: 2), _f(_a(_li, .8)));
  c.drawPath(_blob(const Offset(30, 50), 11, seed: 5), _f(_a(_li, .8)));
  final p = Path()
    ..moveTo(10, 102)
    ..cubicTo(56, 104, 36, 32, 84, 40)
    ..cubicTo(128, 46, 108, 100, 148, 102);
  final g = _l(3, 9, st.v), laid = _trim(p, 0, st.w);
  _dash(c, _trim(p, st.w, 1), _s(_a(_p0, .35), 1.2), 3, 3);
  if (st.inv) {
    c.drawPath(laid, _s(_a(_p0, .5), g + 2));
  } else {
    _dash(c, laid, _s(const Color(0xFF7A4A2A), g + 7, cap: StrokeCap.butt), 2, 3);
  }
  c.drawPath(laid, _s(_p0, g + 1.6));
  c.drawPath(laid, _s(_cr, g - 1.2));
  final tg = _at(p, st.w)!;
  _rotRect(c, tg.position, -tg.angle, Rect.fromCenter(center: Offset.zero, width: 18, height: g + 6), _f(_or));
  _rotRect(c, tg.position, -tg.angle, Rect.fromCenter(center: const Offset(6, 0), width: 6, height: g + 6), _f(_rd));
}

void _s5(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_vi));
  const bomb = Offset(118, 80);
  final p = Path()
    ..moveTo(12, 30)
    ..cubicTo(40, 0, 60, 70, 84, 40)
    ..cubicTo(100, 20, 110, 40, 112, 58);
  final w = _l(2, 7, st.v), rest = _trim(p, st.w, 1);
  _dash(c, _trim(p, 0, st.w), _s(_a(_p0, .35), 1.4), 1.5, 4);
  c.drawPath(rest, _s(const Color(0xFFC79A5B), w));
  if (st.inv) _dash(c, rest, _s(const Color(0xFF7A5630), w, cap: StrokeCap.butt), 2, 3);
  c.drawCircle(bomb + const Offset(3, 3), 24, _f(_a(_p0, .3)));
  c.drawCircle(bomb, 24, _f(const Color(0xFF111114)));
  c.drawArc(Rect.fromCircle(center: bomb, radius: 17), 3.6, .9, false, _s(_a(_cr, .7), 3));
  c.drawRect(Rect.fromCenter(center: bomb - const Offset(5, 23), width: 10, height: 7), _f(const Color(0xFF111114)));
  if (st.w < .99) {
    final q = _at(p, st.w)!.position;
    for (var i = 0; i < 9; i++) {
      final a = i / 9 * math.pi * 2 + st.t * 7, len = 5 + 6 * _h(i + (st.t * 12).floor());
      c.drawLine(q, _pol(q, len, a), _s(i.isEven ? _ye : _wh, 1.6));
    }
    c.drawCircle(q, 3, _f(_or));
  } else {
    c.drawCircle(bomb, 30 + 4 * math.sin(st.t * 20), _s(_ye, 3));
  }
}

void _s6(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_li));
  final oval = Rect.fromCenter(center: s.center(Offset.zero), width: 128, height: 74);
  c.drawOval(oval.inflate(10), _f(_a(_p0, .12)));
  _dash(c, Path()..addOval(oval.inflate(11)), _s(_rd, 3, cap: StrokeCap.butt), 5, 5);
  c.drawOval(oval, _s(_p1, 18));
  _dash(c, Path()..addOval(oval), _s(_a(_wh, .7), 1.4, cap: StrokeCap.butt), 4, 5);
  final loop = Path()
    ..addOval(oval)
    ..addOval(oval);
  final m = loop.computeMetrics().toList(), per = m.first.length;
  final head = per * (1 + (st.t * .12) % 1), len = per * _l(.04, .95, st.w), w = _l(2, 12, st.v);
  Path seg(double a, double b) {
    final out = Path();
    var off = 0.0;
    for (final mm in m) {
      final sa = (a - off).clamp(0.0, mm.length), sb = (b - off).clamp(0.0, mm.length);
      if (sb > sa) out.addPath(mm.extractPath(sa, sb), Offset.zero);
      off += mm.length;
    }
    return out;
  }

  final trail = seg(head - len, head);
  if (st.inv) {
    _dash(c, trail, _s(_pk, w, cap: StrokeCap.butt), 3, 4);
  } else {
    c.drawPath(trail, _s(_pk, w));
  }
  final mm = m[head >= per ? 1 : 0];
  final tg = mm.getTangentForOffset(head - (head >= per ? per : 0))!;
  _rotRect(c, tg.position + const Offset(1.5, 2), -tg.angle, const Rect.fromLTWH(-7, -4.5, 14, 9), _f(_a(_p0, .35)));
  _rotRect(c, tg.position, -tg.angle, const Rect.fromLTWH(-7, -4.5, 14, 9), _f(_ye));
  _rotRect(c, tg.position, -tg.angle, const Rect.fromLTWH(0, -3, 4, 6), _f(_bl), rad: 1);
}

void _s7(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF241A4A)));
  final w = _l(4, 26, st.v), n = (st.w * 46).round();
  final pts = [for (var i = 0; i <= 46; i++) Offset(22 + i * 2.8, 60 + 30 * math.sin(i * .17 + st.t * 1.6) * math.min(1, i / 10))];
  for (var i = 0; i < math.min(n, pts.length - 1); i++) {
    final a = pts[i], b = pts[i + 1], d = b - a, nn = Offset(-d.dy, d.dx) / d.distance;
    final tw = math.cos(i * .22 - st.t * 2.2), h0 = w / 2 * tw;
    final tw1 = math.cos((i + 1) * .22 - st.t * 2.2), h1 = w / 2 * tw1;
    final quad = _poly([a + nn * h0, b + nn * h1, b - nn * h1, a - nn * h0], close: true);
    final front = tw > 0;
    if (st.inv && i.isOdd) continue;
    c.drawPath(quad, _f(front ? _pk : _vi));
    c.drawPath(quad, _s(front ? _pk : _vi, .6));
  }
  c.drawLine(const Offset(6, 98), pts.first, _s(_cr, 3));
  c.drawLine(const Offset(6, 98), const Offset(12, 86), _s(_or, 5));
}

void _s8(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF2E2620)));
  final p = Path()
    ..moveTo(18, 100)
    ..cubicTo(60, 120, 40, 50, 80, 56)
    ..cubicTo(120, 62, 110, 20, 138, 22);
  _dash(c, p, _s(_a(_cr, .15), 1.2), 2, 4);
  c.drawPath(_star(const Offset(18, 100), 10, 4, 3, .4), _f(_li));
  c.drawPath(_blob(const Offset(138, 22), 8, seed: 3), _f(_ye));
  final m = p.computeMetrics().first, gap = _l(8, 26, 1 - st.w), sz = _l(1.2, 3, st.v);
  for (var d = (st.t * 14) % gap; d < m.length; d += gap) {
    final tg = m.getTangentForOffset(d)!, f = Offset(math.cos(-tg.angle), math.sin(-tg.angle)), side = Offset(-f.dy, f.dx);
    final q = tg.position, wig = math.sin(st.t * 14 + d) * sz * .6;
    for (final k in [-1.0, 0.0, 1.0]) {
      c.drawLine(q + f * k * sz * 1.6 + side * (sz * 1.4 + wig), q + f * k * sz * 1.6 - side * (sz * 1.4 - wig), _s(_a(_or, .6), .8));
    }
    c.drawCircle(q - f * sz * 1.8, sz * 1.05, _f(_or));
    c.drawCircle(q, sz * .8, _f(_or));
    c.drawCircle(q + f * sz * 1.6, sz * .85, _f(_rd));
    if (st.inv) c.drawCircle(q + f * sz * 2.8, sz * .9, _f(_li));
  }
}

void _s9(Canvas c, Size s, PopS st) {
  for (var x = 0.0; x < s.width; x += 6) {
    c.drawLine(Offset(x, 0), Offset(x, s.height), _s(_a(_p0, .05), 1));
  }
  for (var y = 0.0; y < s.height; y += 6) {
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_a(_p0, .05), 1));
  }
  final ctr = s.center(const Offset(0, 4));
  final p = Path()
    ..moveTo(ctr.dx, ctr.dy - 22)
    ..cubicTo(ctr.dx + 26, ctr.dy - 56, ctr.dx + 66, ctr.dy - 10, ctr.dx, ctr.dy + 40)
    ..cubicTo(ctr.dx - 66, ctr.dy - 10, ctr.dx - 26, ctr.dy - 56, ctr.dx, ctr.dy - 22);
  _dash(c, _trim(p, st.w, 1), _s(_a(_vi, .4), 1.2), 3, 3);
  final on = _l(2.5, 13, st.v);
  final sewn = _trim(p, 0, st.w);
  if (st.inv) {
    final m = sewn.computeMetrics();
    for (final mm in m) {
      for (var d = 0.0; d < mm.length; d += on + 3) {
        final tg = mm.getTangentForOffset(d)!, f = Offset(math.cos(-tg.angle), math.sin(-tg.angle)), sd = Offset(-f.dy, f.dx) * 3;
        c.drawLine(tg.position - sd, tg.position + f * on + sd, _s(_pk, 2.2));
      }
    }
  } else {
    _dash(c, sewn, _s(_pk, 2.6), on, on * .7);
  }
  final tg = _at(p, st.w)!;
  final f = Offset(math.cos(-tg.angle), math.sin(-tg.angle));
  c.drawLine(tg.position, tg.position + f * 22, _s(const Color(0xFF8A8A94), 2.4));
  c.drawOval(Rect.fromCenter(center: tg.position + f * 18, width: 3, height: 3), _f(_cr));
  c.drawPath(Path()..moveTo(tg.position.dx + f.dx * 18, tg.position.dy + f.dy * 18)..quadraticBezierTo(tg.position.dx + 30, tg.position.dy - 20, tg.position.dx + 10, tg.position.dy - 30), _s(_pk, 1.2));
}

void _s10(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_bl));
  const y = 60.0;
  final sx = _l(18, 140, st.w), tooth = _l(3.5, 9, st.v), spread = .42;
  Offset top(double x) => x <= sx ? Offset(x, y) : Offset(x, y - (x - sx) * spread);
  Offset bot(double x) => x <= sx ? Offset(x, y) : Offset(x, y + (x - sx) * spread);
  c.drawPath(_poly([for (var x = 0.0; x <= s.width; x += 4) top(x) - const Offset(0, 6)]), _s(_cr, 9, cap: StrokeCap.butt));
  c.drawPath(_poly([for (var x = 0.0; x <= s.width; x += 4) bot(x) + const Offset(0, 6)]), _s(_cr, 9, cap: StrokeCap.butt));
  var k = 0;
  for (var x = 2.0; x < s.width; x += tooth, k++) {
    final up = k.isEven, q = up ? top(x) : bot(x);
    final col = st.inv ? (up ? _ye : _pk) : _ye;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: q + Offset(0, up ? -2 : 2), width: tooth * .8, height: 7), const Radius.circular(1.5)), _f(col));
  }
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(sx, y), width: 16, height: 22), const Radius.circular(4)), _f(_or));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(sx - 4, y + 8, 8, 22), const Radius.circular(4)), _f(_or));
  c.drawCircle(Offset(sx, y + 25), 2, _f(_bl));
}

void _s11(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_bl));
  const nut = Offset(14, 16), bridge = Offset(124, 92), body = Offset(112, 82);
  final dir = (bridge - nut) / (bridge - nut).distance, nrm = Offset(-dir.dy, dir.dx);
  c.drawCircle(body + const Offset(3, 4), 32, _f(_a(_p0, .3)));
  c.drawCircle(body, 32, _f(_or));
  c.drawCircle(body - dir * 26, 22, _f(_or));
  c.drawCircle(body - dir * 14, 9, _f(_p0));
  c.drawLine(nut - dir * 6, body - dir * 30, _s(const Color(0xFF4A2E1E), 20, cap: StrokeCap.butt));
  for (var i = 1; i < 8; i++) {
    final q = nut + dir * (i * 12.0);
    c.drawLine(q + nrm * 10, q - nrm * 10, _s(_a(_cr, .55), 1.2));
  }
  c.drawLine(nut + nrm * 10, nut - nrm * 10, _s(_cr, 3));
  _rotRect(c, bridge, dir.direction, const Rect.fromLTWH(-3, -12, 6, 24), _f(_p0));
  final g = _l(.6, 2.6, st.v), fl = (bridge - nut).distance, fu = _l(.08, .55, st.w), fret = nut + dir * (fl * fu);
  final amp = 7 * (.4 + .6 * math.sin(st.t * 1.3).abs()), ph = math.sin(st.t * 31);
  for (var i = 0; i < 4; i++) {
    final off = nrm * ((i - 1.5) * 4.0), w = g * (.7 + i * .35), col = i == 2 ? _ye : _cr;
    if (st.inv) {
      _dash(c, Path()..moveTo((nut + off).dx, (nut + off).dy)..lineTo((fret + off).dx, (fret + off).dy), _s(col, w, cap: StrokeCap.butt), 1.2, 1.2);
    } else {
      c.drawLine(nut + off, fret + off, _s(col, w));
    }
    final a = i == 2 ? amp : amp * .15;
    c.drawPath(_poly([for (var k = 0; k <= 24; k++) fret + off + (bridge - fret) * (k / 24) + nrm * (a * ph * math.sin(math.pi * k / 24))]), _s(col, w));
    if (i == 2) {
      final env = _poly([
        for (var k = 0; k <= 24; k++) fret + off + (bridge - fret) * (k / 24) + nrm * (a * math.sin(math.pi * k / 24)),
        for (var k = 24; k >= 0; k--) fret + off + (bridge - fret) * (k / 24) - nrm * (a * math.sin(math.pi * k / 24)),
      ], close: true);
      c.drawPath(env, _f(_a(_ye, .18)));
    }
  }
  _rotRect(c, fret, dir.direction, const Rect.fromLTWH(-5, -13, 10, 18), _f(_pk), rad: 5);
}

void _s12(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF0B0B12)));
  for (var i = 0; i < 20; i++) {
    c.drawCircle(Offset(_h(i + 2) * s.width, _h(i + 61) * s.height), .9, _f(_a(_cr, .5)));
  }
  final oval = Rect.fromCenter(center: s.center(Offset.zero), width: 132, height: 76);
  _dash(c, Path()..addOval(oval), _s(_a(_cr, .2), 1), 2, 5);
  c.drawCircle(oval.center + const Offset(-30, 0), 9, _f(_ye));
  final m = (Path()..addOval(oval)).computeMetrics().first, per = m.length;
  final head = (st.t * .12) % 1 * per, len = per * _l(.05, .6, st.w), w = _l(2, 12, st.v);
  const segs = 16;
  for (var i = 0; i < segs; i++) {
    final a = head - len * (1 - i / segs), b = head - len * (1 - (i + 1) / segs);
    if (st.inv && i.isOdd) continue;
    final f = (i + 1) / segs;
    final seg = Path();
    for (final (x, y) in [(a, b)]) {
      if (x < 0 && y <= 0) {
        seg.addPath(m.extractPath(x + per, y + per), Offset.zero);
      } else if (x < 0) {
        seg
          ..addPath(m.extractPath(x + per, per), Offset.zero)
          ..addPath(m.extractPath(0, y), Offset.zero);
      } else {
        seg.addPath(m.extractPath(x, y), Offset.zero);
      }
    }
    c.drawPath(seg, _s(Color.lerp(_vi, _cy, f)!.withValues(alpha: f), math.max(.6, w * f), cap: StrokeCap.butt));
  }
  final q = m.getTangentForOffset(head)!.position;
  c.drawCircle(q, w * .55 + 2, _f(_wh));
}

void _s13(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_ye));
  const plate = Offset(38, 84);
  c.drawOval(Rect.fromCenter(center: plate, width: 64, height: 30), _f(_wh));
  c.drawOval(Rect.fromCenter(center: plate, width: 44, height: 18), _s(_a(_p0, .1), 1.4));
  c.drawOval(Rect.fromCenter(center: plate, width: 26, height: 10), _f(_rd));
  const face = Offset(124, 48);
  c.drawCircle(face, 26, _f(_pk));
  c.drawCircle(face + const Offset(-8, -8), 3, _f(_p0));
  c.drawCircle(face + const Offset(8, -8), 3, _f(_p0));
  c.drawCircle(face + const Offset(-14, 2), 4, _f(_a(_rd, .5)));
  c.drawCircle(face + const Offset(14, 2), 4, _f(_a(_rd, .5)));
  final mouth = face + const Offset(-6, 10);
  c.drawOval(Rect.fromCenter(center: mouth, width: 9, height: 8), _f(_p0));
  final p = Path()
    ..moveTo(30, 84)
    ..cubicTo(60, 40, 40, 110, 70, 92)
    ..cubicTo(100, 74, 80, 40, mouth.dx, mouth.dy);
  final wig = math.sin(st.t * 10) * 2;
  final w = _l(2, 8, st.v), nood = _trim(p, _l(0, .9, st.w), 1).shift(Offset(0, wig * st.w));
  c.drawPath(nood, _s(const Color(0xFFC8962E), w + 2));
  if (st.inv) {
    _dash(c, nood, _s(const Color(0xFFFFF0B8), w, cap: StrokeCap.butt), 4, 2);
  } else {
    c.drawPath(nood, _s(const Color(0xFFFFF0B8), w));
  }
  for (var i = 0; i < 6; i++) {
    final a = math.pi * (1 - i / 5) * .7 - .3;
    c.drawLine(_pol(face, 30, a + 1.2), _pol(face, 34 + 3 * st.w, a + 1.2), _s(_a(_p0, .6 * st.w), 1.4));
  }
}

void _s14(Canvas c, Size s, PopS st) {
  final flash = st.w > .97 ? .35 + .2 * math.sin(st.t * 30) : 0.0;
  c.drawRect(Offset.zero & s, _f(Color.lerp(const Color(0xFF15102C), _vi, flash)!));
  for (final (o, r) in [(const Offset(50, 12), 18.0), (const Offset(78, 6), 22.0), (const Offset(106, 14), 18.0), (const Offset(80, 22), 16.0)]) {
    c.drawCircle(o, r, _f(const Color(0xFF3B3560)));
  }
  final pts = <Offset>[const Offset(78, 26)];
  for (var i = 1; i <= 7; i++) {
    pts.add(Offset(78 + (i.isOdd ? 1 : -1) * (8 + 8 * _h(i)) + i * 1.5, 26 + i * 12.5));
  }
  final main = _poly(pts), br = _poly([pts[3], pts[3] + const Offset(-22, 12), pts[3] + const Offset(-30, 30)]);
  final w = _l(1.5, 7, st.v);
  for (final (p, f) in [(main, st.w), (br, ((st.w - .4) / .6).clamp(0.0, 1.0))]) {
    final t = _trim(p, 0, f);
    c.drawPath(t, _s(_a(_vi, .45), w + 8));
    c.drawPath(t, _s(_ye, w + 2));
    if (st.inv) {
      _dash(c, t, _s(_wh, w), 4, 4);
    } else {
      c.drawPath(t, _s(_wh, w));
    }
  }
  c.drawLine(Offset(0, s.height - 6), Offset(s.width, s.height - 6), _s(_a(_cr, .3), 1));
}

void _s15(Canvas c, Size s, PopS st) {
  final pts = <Offset>[];
  for (var i = 0; i <= 140; i++) {
    final u = i / 140, th = u * math.pi * 2 * 5.5;
    pts.add(Offset(16 + 118 * u - 9 * math.sin(th), 62 - 16 * math.cos(th) * (1 - .4 * u) + 6 * math.sin(u * 6)));
  }
  final nb = _l(1.2, 6, st.v), n = (st.w * (pts.length - 1)).round();
  for (var i = 0; i < n; i++) {
    final d = pts[i + 1] - pts[i];
    if (st.inv && (i ~/ 4).isOdd) continue;
    final wv = nb * (.3 + .7 * math.sin(d.direction - math.pi / 4).abs());
    c.drawLine(pts[i], pts[i + 1], _s(const Color(0xFF1E2A66), wv));
  }
  if (st.w > .95) c.drawPath(Path()..moveTo(14, 98)..quadraticBezierTo(80, 88, 142, 100), _s(_or, 2.4));
  final q = pts[n];
  _rotRect(c, q + const Offset(9, -14), -.9, const Rect.fromLTWH(-4, -16, 8, 26), _f(_bl), rad: 3);
  c.drawPath(_poly([q, q + const Offset(2, -7), q + const Offset(7, -4)], close: true), _f(_ye));
}

void _s16(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF233042)));
  c.drawRect(Rect.fromLTWH(0, s.height - 16, s.width, 16), _f(_li));
  c.drawPath(Path()..moveTo(-4, 112)..cubicTo(20, 112, 6, 70, 30, 72), _s(const Color(0xFF2E9E4A), 6));
  const noz = Offset(36, 68);
  final pw = _l(.25, 1, st.w), w = _l(1.5, 7, st.v), range = 30 + pw * 104;
  final arc = _poly([for (var i = 0; i <= 30; i++) Offset(noz.dx + range * i / 30, noz.dy - pw * 44 * math.sin(math.pi * i / 30 * .9) + 40 * (i / 30) * (i / 30) * .9)]);
  if (st.inv) {
    c.drawPath(arc, _s(_a(_cy, .9), w));
  } else {
    _dash(c, arc, _s(_cy, w), w * 1.4 + 2, w + 4, st.t * 60);
  }
  _rotRect(c, noz, -.35, const Rect.fromLTWH(-8, -3, 12, 6), _f(_or));
  final land = Offset(noz.dx + range, s.height - 16);
  c.drawOval(Rect.fromCenter(center: land, width: 12 + w * 3, height: 5), _f(_a(_cy, .8)));
  for (var i = 0; i < 3; i++) {
    c.drawLine(land, land + Offset((i - 1) * 6.0, -5 - 3 * math.sin(st.t * 12 + i).abs()), _s(_cy, 1.2));
  }
}

void _s17(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_or));
  const a = Offset(16, 18), b = Offset(140, 18);
  final sag = _l(8, 86, st.w), link = _l(6, 16, st.v);
  final p = Path()
    ..moveTo(a.dx, a.dy)
    ..quadraticBezierTo((a.dx + b.dx) / 2, a.dy + sag * 2, b.dx, b.dy);
  final m = p.computeMetrics().first;
  var k = 0;
  for (var d = 0.0; d <= m.length; d += link * .8, k++) {
    final tg = m.getTangentForOffset(d)!;
    final flat = k.isEven;
    final col = st.inv && k % 4 == 0 ? _ye : _p0;
    _rotRect(c, tg.position, -tg.angle, Rect.fromCenter(center: Offset.zero, width: link, height: flat ? link * .6 : 2.6), flat ? _s(col, 2.2) : _f(col), rad: link * .3);
  }
  for (final h in [a, b]) {
    c.drawCircle(h, 5, _f(_cr));
    c.drawCircle(h, 2, _f(_or));
  }
}

void _s18(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_cy));
  const ctr = Offset(78, 116), cols = [_rd, _or, _ye, _li, _bl, _vi];
  final bw = _l(2, 7, st.v);
  for (var i = 0; i < cols.length; i++) {
    final rr = 74 - i * bw;
    final arc = Path()..addArc(Rect.fromCircle(center: ctr, radius: rr), math.pi, math.pi * st.w);
    if (st.inv) {
      _dash(c, arc, _s(cols[i], bw, cap: StrokeCap.butt), 6, 4);
    } else {
      c.drawPath(arc, _s(cols[i], bw + .4, cap: StrokeCap.butt));
    }
  }
  for (final o in [ctr + const Offset(-70, -4), ctr + Offset.fromDirection(math.pi + math.pi * st.w, 70)]) {
    for (final (d, r) in [(const Offset(-8, 0), 8.0), (const Offset(2, -4), 10.0), (const Offset(11, 1), 7.0)]) {
      c.drawCircle(o + d, r, _f(_wh));
    }
  }
}

void _s19(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_pk));
  final ctr = s.center(const Offset(0, 4));
  final p = _poly([for (var i = 0; i <= 80; i++) ctr + Offset(math.cos(i * .11 + 2.6) * (12 + i * .62) * 1.25, math.sin(i * .11 + 2.6) * (12 + i * .62) * .62)]);
  final m = p.computeMetrics().first, gap = _l(6, 15, st.v), front = m.length * (1 - st.w);
  final items = <(Offset, double, bool)>[];
  for (var d = m.length; d > 0; d -= gap) {
    final tg = m.getTangentForOffset(d)!;
    items.add((tg.position, -tg.angle, d > front));
  }
  items.sort((x, y) => x.$1.dy.compareTo(y.$1.dy));
  for (final (q, ang, down) in items) {
    if (down) {
      _rotRect(c, q, ang, const Rect.fromLTWH(-1, -3, 13, 6), _f(_a(_p0, .3)));
      _rotRect(c, q, ang, const Rect.fromLTWH(-2, -3.5, 12, 6), _f(_wh));
      _rotRect(c, q, ang, const Rect.fromLTWH(1.5, -.8, 1.6, 1.6), _f(_p0), rad: 1);
    } else {
      c.drawRect(Rect.fromLTWH(q.dx - 2.5, q.dy - 14, 5, 14), _f(_wh));
      c.drawRect(Rect.fromLTWH(q.dx + 2.5, q.dy - 14, 1.6, 14), _f(_a(_p0, .35)));
      c.drawCircle(Offset(q.dx, q.dy - 10), .9, _f(_p0));
      c.drawCircle(Offset(q.dx, q.dy - 4), .9, _f(_p0));
      if (st.inv) c.drawRect(Rect.fromLTWH(q.dx - 2.5, q.dy - 14, 5, 2), _f(_ye));
    }
  }
}

void _s20(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF0C0C10)));
  const pts = [Offset(12, 24), Offset(54, 102), Offset(96, 22), Offset(144, 96), Offset(118, 112)];
  for (var i = 1; i < pts.length; i++) {
    final dir = i.isOdd ? -.5 : .5;
    c.drawLine(pts[i] + Offset.fromDirection(dir, 9), pts[i] - Offset.fromDirection(dir, 9), _s(_cy, 3));
  }
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: pts.first, width: 14, height: 10), const Radius.circular(2)), _f(_p1));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: pts.first, width: 14, height: 10), const Radius.circular(2)), _s(_rd, 1.2));
  final path = _poly(pts);
  _dash(c, path, _s(_a(_rd, .2), 1), 2, 4);
  final len = _l(.05, .6, st.w), head = (st.t * .3) % (1 + len), w = _l(1, 6, st.v);
  final seg = _trim(path, head - len, head);
  c.drawPath(seg, _s(_a(_rd, .3), w + 8));
  if (st.inv) {
    _dash(c, seg, _s(_wh, w), 5, 5, st.t * 80);
  } else {
    c.drawPath(seg, _s(_rd, w + 2));
    c.drawPath(seg, _s(_wh, w * .6));
  }
}
