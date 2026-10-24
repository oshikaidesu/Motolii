// Part of pop_07_glass.dart (cut there by size, as it was).
part of 'pop_07.dart';

void _caustics(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF1677C9)));
  const tile = 15.0;
  final grout = _s(_o(_wh, .25), 1);
  for (var x = 0.0; x < s.width; x += tile) {
    c.drawLine(Offset(x, 0), Offset(x, s.height), grout);
  }
  for (var y = 0.0; y < s.height; y += tile) {
    c.drawLine(Offset(0, y), Offset(s.width, y), grout);
  }
  final amp = 2 + st.b * 9, fr = .05 + st.b * .06;
  final line = _s(_o(_cy, .9), 1.6 + (1 - st.b) * 1.2);
  for (var k = 0; k < 7; k++) {
    final y0 = k * 19.0;
    _bent(c, _hline(y0, s.width, 5), (p) => p + Offset(0, math.sin(p.dx * fr + t * 1.6 + k * 2) * amp), line);
  }
  for (var k = 0; k < 9; k++) {
    final x0 = k * 19.0;
    _bent(c, _vline(x0, s.height, 5), (p) => p + Offset(math.sin(p.dy * fr - t * 1.3 + k * 1.7) * amp, 0), line);
  }
  final ring = Offset(s.width * .62 + math.sin(t * .5) * 10, s.height * .45 + math.cos(t * .4) * 8);
  c.drawCircle(ring + const Offset(10, 12), 17, _f(_o(_ink, .25)));
  c.drawCircle(ring, 17, _s(_ye, 9));
  for (var i = 0; i < 4; i++) {
    c.drawArc(Rect.fromCircle(center: ring, radius: 17), i * math.pi / 2, .5, false, _s(_re, 9)..strokeCap = StrokeCap.butt);
  }
}

void _bubble(Canvas c, Size s, P7State st, double t) {
  final wand = Offset(s.width / 2, s.height - 14);
  final r = 14 + st.b * 34;
  final ctr = Offset(s.width / 2 + math.sin(t * 1.3) * 3, wand.dy - 10 - r);
  final thin = 1 - st.b;
  final rot = t * .6 + thin * 4;
  final cols = [_pk, _ye, _cy, _vi, _li, _pk];
  c.drawCircle(ctr, r, _f(_o(_cy, .06)));
  c.drawCircle(
      ctr,
      r - 2,
      _s(_wh, 3 + thin * 6)
        ..shader = SweepGradient(colors: cols, transform: GradientRotation(rot)).createShader(Rect.fromCircle(center: ctr, radius: r)));
  c.drawArc(Rect.fromCircle(center: ctr, radius: r * .7), 3.7, .8, false, _s(_o(_wh, .8), 3));
  c.drawCircle(ctr + Offset(r * .35, r * .4), 2.5, _f(_o(_wh, .6)));
  c.drawLine(wand, Offset(wand.dx, s.height), _s(_or, 4));
  c.drawCircle(wand - const Offset(0, 6), 8, _s(_or, 3));
  for (var i = 0; i < 3; i++) {
    final y = (s.height - (t * 20 + i * 40) % (s.height + 20));
    c.drawCircle(Offset(20 + i * 55.0 + math.sin(t + i) * 6, y), 4 + i.toDouble(), _s(_o(_cy, .7), 1.2));
  }
}

void _sandpaper(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(_li));
  for (var i = 0; i < 4; i++) {
    c.drawCircle(Offset(28 + i * 34.0, s.height * .55 + (i.isEven ? -14 : 14)), 15, _f(i.isEven ? _vi : _pk));
  }
  final marks = st.marks.isNotEmpty ? st.marks : [for (var i = 0; i < 14; i++) Offset(.12 + i * .04, .78 - i * .035 + math.sin(i * 1.3) * .05)];
  final frost = st.rub;
  for (final m in marks) {
    final p = Offset(m.dx * s.width, m.dy * s.height);
    c.drawCircle(p, 13, _f(_o(_wh, .28)));
  }
  c.drawRect(Offset.zero & s, _f(_o(_wh, frost * .35)));
  for (var i = 0; i < marks.length; i++) {
    final m = marks[i];
    final p = Offset(m.dx * s.width, m.dy * s.height);
    final a = _h(i) * math.pi;
    c.drawLine(p - Offset(math.cos(a), math.sin(a)) * 9, p + Offset(math.cos(a), math.sin(a)) * 9, _s(_o(_wh, .7), 1));
  }
  final blk = Offset(st.p.dx * s.width, st.p.dy * s.height) + Offset(math.sin(t * 20) * (st.down ? 2 : 0), 0);
  final rect = Rect.fromCenter(center: blk, width: 40, height: 24);
  c.drawRRect(RRect.fromRectAndRadius(rect.shift(const Offset(3, 3)), const Radius.circular(4)), _f(_o(_ink, .3)));
  c.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(4)), _f(_or));
  for (var i = 0; i < 22; i++) {
    c.drawCircle(rect.topLeft + Offset(3 + _h(i) * 34, 3 + _h(i + 40) * 18), .9, _f(_o(_ink, .6)));
  }
  c.drawRRect(RRect.fromRectAndRadius(rect, const Radius.circular(4)), _s(_ink, 1.5));
}

void _lighthouse(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF231A4D)));
  for (var i = 0; i < 16; i++) {
    c.drawCircle(Offset(_h(i) * s.width, _h(i + 9) * s.height * .5), 1, _f(_o(_wh, .7)));
  }
  final lamp = Offset(34, 40);
  final spread = .55 - st.b * .48;
  final dir = math.sin(t * .8) * .25 + .15;
  final beam = Path()
    ..moveTo(lamp.dx, lamp.dy)
    ..lineTo(lamp.dx + math.cos(dir - spread) * 200, lamp.dy + math.sin(dir - spread) * 200)
    ..lineTo(lamp.dx + math.cos(dir + spread) * 200, lamp.dy + math.sin(dir + spread) * 200)
    ..close();
  c.drawPath(beam, _f(_o(_ye, .35 + st.b * .5)));
  final sea = Path()..moveTo(0, s.height - 24);
  for (var x = 0.0; x <= s.width; x += 6) {
    sea.lineTo(x, s.height - 24 + math.sin(x * .15 + t * 2) * 2);
  }
  sea
    ..lineTo(s.width, s.height)
    ..lineTo(0, s.height)
    ..close();
  c.drawPath(sea, _f(_bl));
  c.drawPath(
      Path()..addPolygon([Offset(0, s.height), Offset(0, s.height - 34), Offset(52, s.height - 26), Offset(64, s.height)], true), _f(_k1));
  c.drawPath(Path()..addPolygon([Offset(24, s.height - 30), Offset(44, s.height - 30), Offset(40, lamp.dy + 8), Offset(28, lamp.dy + 8)], true), _f(_cr));
  for (var i = 0; i < 3; i++) {
    final y = s.height - 40 - i * 16.0;
    c.drawRect(Rect.fromLTWH(26 + i * 1.3, y, 16 - i * 2.6, 6), _f(_re));
  }
  c.drawRect(Rect.fromCenter(center: lamp, width: 18, height: 16), _f(_o(_ye, .9)));
  for (var k = 1; k <= 3; k++) {
    c.drawArc(Rect.fromCenter(center: lamp, width: k * 5.0, height: k * 5.0 + 2), -math.pi / 2, math.pi, false, _s(_or, 1.2));
  }
  c.drawPath(Path()..addPolygon([lamp + const Offset(-11, -8), lamp + const Offset(11, -8), lamp + const Offset(0, -18)], true), _f(_re));
  final boat = Offset(s.width * .78 + math.sin(t * .5) * 6, s.height - 26);
  c.drawPath(Path()..addPolygon([boat + const Offset(-10, 0), boat + const Offset(10, 0), boat + const Offset(6, 5), boat + const Offset(-6, 5)], true), _f(_wh));
}

void _glasses(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2 + 6);
  c.drawCircle(ctr, 50, _f(_or));
  c.drawArc(Rect.fromCenter(center: ctr + const Offset(0, 22), width: 30, height: 14), .2, math.pi - .4, false, _s(_ink, 3));
  final thick = st.b;
  final blink = (t % 3.5) < .12;
  for (final sx in [-1.0, 1.0]) {
    final e = ctr + Offset(sx * 20, -8);
    final lensR = 17.0;
    c.save();
    c.clipPath(Path()..addOval(Rect.fromCircle(center: e, radius: lensR)));
    c.drawCircle(e, lensR, _f(_cr));
    final er = 4 + thick * 10;
    if (blink) {
      c.drawLine(e - Offset(er, 0), e + Offset(er, 0), _s(_ink, 3));
    } else {
      c.drawCircle(e, er, _f(_wh));
      c.drawCircle(e, er, _s(_ink, 1.5));
      c.drawCircle(e + Offset(math.sin(t) * er * .3, 0), er * .55, _f(_ink));
      c.drawCircle(e + Offset(-er * .2, -er * .25), er * .15, _f(_wh));
    }
    for (var k = 0; k < (thick * 4).round(); k++) {
      c.drawCircle(e, lensR - 2 - k * 2.5, _s(_o(_cy, .5), 1));
    }
    c.restore();
    c.drawCircle(e, lensR, _s(_ink, 3 + thick * 3));
    c.drawArc(Rect.fromCircle(center: e, radius: lensR - 5), 3.6, .8, false, _s(_o(_wh, .9), 2));
  }
  c.drawLine(ctr + const Offset(-3, -10), ctr + const Offset(3, -10), _s(_ink, 3));
}

void _paperweight(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height * .62);
  const ux = Offset(12, 7), uy = Offset(-12, 7);
  void floor(Offset shift, double alpha) {
    for (var i = -4; i < 4; i++) {
      for (var j = -4; j < 4; j++) {
        if ((i + j).isEven) continue;
        final o = ctr + shift + ux * i.toDouble() + uy * j.toDouble();
        c.drawPath(Path()..addPolygon([o, o + ux, o + ux + uy, o + uy], true), _f(_o(_or, alpha)));
      }
    }
  }

  c.drawRect(Offset.zero & s, _f(_k1));
  floor(Offset.zero, 1);
  final hgt = 6 + st.b * 34;
  final p0 = ctr - ux * 2 - uy * 2 + Offset(0, 0), wx = ux * 4, wy = uy * 4, up = Offset(0, -hgt);
  final top = [p0 + up, p0 + wx + up, p0 + wx + wy + up, p0 + wy + up];
  final left = [p0 + wy, p0 + wy + wx, p0 + wy + wx + up, p0 + wy + up];
  final right = [p0 + wx, p0 + wx + wy, p0 + wx + wy + up, p0 + wx + up];
  c.save();
  c.clipPath(Path()..addPolygon(top, true));
  c.drawRect(Offset.zero & s, _f(_k1));
  floor(Offset(st.b * 8, -hgt * .75), 1);
  c.restore();
  c.drawPath(Path()..addPolygon(top, true), _f(_o(_cy, .22)));
  c.drawPath(Path()..addPolygon(left, true), _f(_o(_cy, .45)));
  c.drawPath(Path()..addPolygon(right, true), _f(_o(_bl, .45)));
  for (final face in [left, right, top]) {
    c.drawPath(Path()..addPolygon(face, true), _s(_wh, 1.5));
  }
  final g = (t * .5) % 1;
  c.drawLine(Offset.lerp(top[0], top[1], g)!, Offset.lerp(top[3], top[2], g)!, _s(_o(_wh, .4 * math.sin(g * math.pi)), 3));
}

void _gem(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2);
  const r = 44.0;
  final rot = st.spin * .25 + t * .15;
  final fire = st.b;
  final cols = [_re, _or, _ye, _li, _cy, _bl, _vi, _pk];
  const n = 8;
  for (var i = 0; i < n; i++) {
    final a0 = rot + i * math.pi * 2 / n, a1 = a0 + math.pi * 2 / n;
    final outer0 = ctr + Offset(math.cos(a0), math.sin(a0)) * r, outer1 = ctr + Offset(math.cos(a1), math.sin(a1)) * r;
    final inner = ctr + Offset(math.cos((a0 + a1) / 2), math.sin((a0 + a1) / 2)) * r * .5;
    final col = Color.lerp(const Color(0xFFDCE6F2), cols[(i + (t * 2).floor()) % 8], fire * (.4 + .6 * math.sin(t * 3 + i).abs()))!;
    c.drawPath(Path()..addPolygon([outer0, outer1, inner], true), _f(col));
    c.drawPath(Path()..addPolygon([ctr, inner, outer1], true), _f(Color.lerp(col, _ink, .25)!));
  }
  final table = Path();
  for (var i = 0; i < n; i++) {
    final a = rot + (i + .5) * math.pi * 2 / n;
    final p = ctr + Offset(math.cos(a), math.sin(a)) * r * .5;
    i == 0 ? table.moveTo(p.dx, p.dy) : table.lineTo(p.dx, p.dy);
  }
  table.close();
  c.drawPath(table, _s(_wh, 1.5));
  for (var i = 0; i < n; i++) {
    final a = rot + i * math.pi * 2 / n;
    c.drawLine(ctr + Offset(math.cos(a), math.sin(a)) * r, ctr, _s(_o(_wh, .6), 1));
  }
  c.drawCircle(ctr, r, _s(_wh, 2));
  for (var k = 0; k < (fire * 5).round(); k++) {
    final ph = (t * .9 + k * .37) % 1;
    final p = ctr + Offset(math.cos(k * 2.3 + rot), math.sin(k * 2.3 + rot)) * (r + 4 + _h(k) * 10);
    final l = 7 * math.sin(ph * math.pi);
    final col = cols[k * 2 % 8];
    c.drawLine(p - Offset(l, 0), p + Offset(l, 0), _s(col, 2));
    c.drawLine(p - Offset(0, l), p + Offset(0, l), _s(col, 2));
  }
}

void _slime(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(_k1));
  final squash = .55 + st.b * .9;
  final wob = math.sin(t * 3) * .05;
  final rx = 42 / math.sqrt(squash) * (1 + wob), ry = 34 * squash * (1 - wob);
  final base = s.height - 12;
  final ctr = Offset(s.width / 2, base - ry);
  final bump = .35 + st.b * .5;
  Offset field(Offset p) {
    final d = Offset((p.dx - ctr.dx) / rx, (p.dy - ctr.dy) / ry);
    final q = d.dx * d.dx + d.dy * d.dy;
    if (q >= 1) return p;
    final k = (1 - q) * bump;
    return Offset(p.dx + (p.dx - ctr.dx) * k, p.dy);
  }

  final cols = [_pk, _ye, _cy, _or];
  for (var i = 0; i < 14; i++) {
    _bent(c, _vline(i * 12.0 + 4, s.height, 3), field, _s(cols[i % 4], 4));
  }
  c.drawRect(Rect.fromLTWH(0, base, s.width, 12), _f(_k0));
  final body = Path()
    ..addOval(Rect.fromCenter(center: ctr, width: rx * 2, height: ry * 2 + 0.0));
  c.save();
  c.clipRect(Rect.fromLTWH(0, 0, s.width, base + 1));
  c.drawPath(body, _f(_o(_li, .28)));
  c.drawPath(body, _s(_li, 3));
  c.restore();
  final eyeY = ctr.dy - ry * .3;
  for (final sx in [-1.0, 1.0]) {
    c.drawCircle(Offset(ctr.dx + sx * 12, eyeY), 6, _f(_wh));
    c.drawCircle(Offset(ctr.dx + sx * 12 + 1.5, eyeY + 1), 3, _f(_ink));
  }
  c.drawArc(Rect.fromCenter(center: Offset(ctr.dx, eyeY + 9), width: 12, height: 8), .2, math.pi - .4, false, _s(_ink, 2));
  c.drawArc(Rect.fromCenter(center: ctr - Offset(rx * .45, ry * .45), width: 14, height: 10), 3.4, 1.2, false, _s(_o(_wh, .9), 2.5));
}
