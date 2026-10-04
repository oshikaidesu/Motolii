// Opacity / Blend x20: why you touch it = "let what is behind show through" and "how the layer mixes with what is under it".
part of 'pop_07.dart';

final _blendSpecs = <_P7Spec>[
  _P7Spec('Ghost sheet', 'creature · bold flat · drag ↕ · result', _ghost, b: .7),
  _P7Spec('Ink drops', 'material · flat light · drag ↔ · multiply', _inkDrops, a: .45, light: true),
  _P7Spec('Spotlights', 'instrument · neon · drag light · screen', _spotlights, p: const Offset(.62, .5)),
  _P7Spec('Scratch card', 'toy · bold flat · rub · reveal', _scratch, p: const Offset(.72, .62)),
  _P7Spec('Deep water', 'nature · side view · drag ↕ · result', _deepWater, b: .4),
  _P7Spec('Gel stack', 'material · isometric · stack ↑ · result', _gelStack, b: .35),
  _P7Spec('Cocktail layers', 'food · pseudo 3D · drag ↔ · mix', _cocktail, a: .3, light: true),
  _P7Spec('Eclipse', 'cosmic · neon · drag moon · difference', _eclipse, p: const Offset(.68, .36)),
  _P7Spec('Neon heart', 'machine · neon wild · drag ↕ · glow', _neonHeart, b: .8),
  _P7Spec('Blinds', 'machine · pseudo 3D · drag ↕ · result', _blinds, b: .45),
  _P7Spec('Pixel bugs', 'creature · pixel flat · drag ↔ · dissolve', _pixelBugs, a: .3),
  _P7Spec('Halftone', 'print · pop-art light · drag ↕ · result', _halftone, b: .5, light: true),
  _P7Spec('Mode slot', 'toy · machine · flick ↔ · mode', _modeSlot),
  _P7Spec('Palette knife', 'material · flat light · drag ↔ · mix', _knife, a: .4, light: true),
  _P7Spec('Sunrise city', 'weather · landscape · drag ↕ · screen', _sunrise, b: .35),
  _P7Spec('Shadow puppet', 'character · light · drag ↔ · multiply', _puppet, a: .4, light: true),
  _P7Spec('Jelly cubes', 'food · pseudo 3D · drag ↕ · result', _jelly, b: .5),
  _P7Spec('Chameleon', 'nature · bold flat · drag ↔ · blend in', _chameleon, a: .35),
  _P7Spec('Day / night', 'weather · porthole · drag ↔ · crossfade', _dayNight, a: .3),
  _P7Spec('Hologram', 'cosmic · neon wild · drag ↕ · flicker', _hologram, b: .7),
];

void _ghost(Canvas c, Size s, P7State st, double t) {
  final cols = [_or, _ye, _pk, _vi, _cy, _li];
  final w = s.width / 6;
  for (var i = 0; i < 6; i++) {
    c.drawRect(Rect.fromLTWH(i * w, 0, w, s.height), _f(cols[i]));
  }
  c.drawRect(Rect.fromLTWH(0, s.height - 18, s.width, 18), _f(_k1));
  final cx = s.width / 2, cy = s.height / 2 - 6 + math.sin(t * 2) * 3;
  final body = Path()
    ..moveTo(cx - 30, cy + 34)
    ..lineTo(cx - 30, cy - 6)
    ..arcToPoint(Offset(cx + 30, cy - 6), radius: const Radius.circular(30))
    ..lineTo(cx + 30, cy + 34);
  for (var i = 0; i < 4; i++) {
    final x0 = cx + 30 - i * 15.0;
    body.quadraticBezierTo(x0 - 3.75, cy + 26 + math.sin(t * 4 + i) * 2, x0 - 7.5, cy + 32);
    body.quadraticBezierTo(x0 - 11.25, cy + 40, x0 - 15, cy + 34);
  }
  body.close();
  c.drawPath(body, _f(_o(_wh, .08 + .92 * st.b)));
  c.drawPath(body, _s(_o(_ink, .25 + .5 * st.b), 2));
  c.drawOval(Rect.fromCenter(center: Offset(cx - 10, cy - 6), width: 8, height: 12), _f(_ink));
  c.drawOval(Rect.fromCenter(center: Offset(cx + 10, cy - 6), width: 8, height: 12), _f(_ink));
  c.drawOval(Rect.fromCenter(center: Offset(cx, cy + 8), width: 8, height: 6 + 4 * (1 - st.b)), _f(_ink));
}

void _inkDrops(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2 + 4), d = 8 + st.a * 30;
  final inks = [_cy, _pk, _ye];
  for (var i = 0; i < 3; i++) {
    final ang = -math.pi / 2 + i * 2 * math.pi / 3;
    final p = ctr + Offset(math.cos(ang), math.sin(ang)) * d;
    final pt = _f(inks[i])..blendMode = BlendMode.multiply;
    c.drawCircle(p, 28, pt);
    for (var k = 0; k < 3; k++) {
      final a2 = ang + (k - 1) * .5;
      c.drawCircle(p + Offset(math.cos(a2), math.sin(a2)) * (34 + 4 * _h(i * 7 + k)), 2.5 + 2 * _h(k + i), pt);
    }
  }
}

void _spotlights(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Rect.fromLTWH(0, s.height * .62, s.width, s.height), _f(_k1));
  final xs = [s.width * .25 + math.sin(t * .9) * 8, st.p.dx * s.width, s.width * .75 + math.cos(t * .7) * 8];
  final cols = [_re, _li, _bl];
  for (var i = 0; i < 3; i++) {
    final top = Offset(s.width * (.2 + .3 * i), 0);
    final fx = xs[i], fy = s.height * .8;
    final cone = Path()
      ..moveTo(top.dx - 4, 0)
      ..lineTo(fx - 26, fy)
      ..lineTo(fx + 26, fy)
      ..lineTo(top.dx + 4, 0)
      ..close();
    c.drawPath(cone, _f(_o(cols[i], .22))..blendMode = BlendMode.plus);
    c.drawOval(Rect.fromCenter(center: Offset(fx, fy), width: 56, height: 16), _f(_o(cols[i], .95))..blendMode = BlendMode.plus);
    c.drawRect(Rect.fromCenter(center: top, width: 14, height: 8), _f(_cr));
  }
  final px = s.width / 2;
  c.drawCircle(Offset(px, s.height * .62), 6, _f(_ink));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(px, s.height * .73), width: 12, height: 16), const Radius.circular(4)), _f(_ink));
}

void _scratch(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(_vi));
  final ctr = Offset(s.width / 2, s.height / 2);
  final star = Path();
  for (var i = 0; i < 10; i++) {
    final r = i.isEven ? 40.0 : 17.0, a = -math.pi / 2 + i * math.pi / 5 + t * .3;
    final p = ctr + Offset(math.cos(a), math.sin(a)) * r;
    i == 0 ? star.moveTo(p.dx, p.dy) : star.lineTo(p.dx, p.dy);
  }
  star.close();
  c.drawPath(star, _f(_li));
  c.drawPath(star, _s(_ink, 2));
  final card = Rect.fromLTWH(10, 10, s.width - 20, s.height - 20);
  c.saveLayer(Offset.zero & s, Paint());
  c.drawRRect(RRect.fromRectAndRadius(card, const Radius.circular(8)), _f(_ye));
  for (var x = -s.height; x < s.width; x += 9) {
    c.drawLine(Offset(x, s.height), Offset(x + s.height, 0), _s(_o(_or, .55), 2));
  }
  final clear = Paint()..blendMode = BlendMode.clear;
  for (final m in st.marks) {
    c.drawCircle(Offset(m.dx * s.width, m.dy * s.height), 10, clear);
  }
  if (st.marks.isEmpty) {
    for (var i = 0; i < 9; i++) {
      c.drawCircle(Offset(s.width * (.2 + i * .065), s.height * (.78 - i * .07)), 11, clear);
    }
  }
  c.restore();
  c.drawRRect(RRect.fromRectAndRadius(card, const Radius.circular(8)), _s(_ink, 2));
  final coin = Offset(st.p.dx * s.width, st.p.dy * s.height);
  c.drawCircle(coin + const Offset(10, 10), 9, _f(_or));
  c.drawCircle(coin + const Offset(10, 10), 9, _s(_ink, 2));
}

void _deepWater(Canvas c, Size s, P7State st, double t) {
  final surf = 14.0;
  c.drawRect(Rect.fromLTWH(0, 0, s.width, surf), _f(_cr));
  final sand = Path()..moveTo(0, s.height - 16);
  for (var x = 0.0; x <= s.width; x += 8) {
    sand.lineTo(x, s.height - 16 + math.sin(x * .09) * 4);
  }
  sand
    ..lineTo(s.width, s.height)
    ..lineTo(0, s.height)
    ..close();
  c.drawRect(Rect.fromLTWH(0, surf, s.width, s.height), _f(_cy));
  c.drawPath(sand, _f(_ye));
  for (var i = 0; i < 3; i++) {
    final x = 22.0 + i * 50;
    final weed = Path()..moveTo(x, s.height - 14);
    for (var k = 1; k < 5; k++) {
      weed.lineTo(x + math.sin(t * 1.5 + k + i) * 4, s.height - 14 - k * 9.0);
    }
    c.drawPath(weed, _s(_li, 4));
  }
  final fx = s.width / 2 + math.sin(t * .8) * 30, fy = s.height * .62;
  final dir = math.cos(t * .8) >= 0 ? 1.0 : -1.0;
  final fish = Path()
    ..addOval(Rect.fromCenter(center: Offset(fx, fy), width: 34, height: 20))
    ..moveTo(fx - dir * 15, fy)
    ..lineTo(fx - dir * 28, fy - 10)
    ..lineTo(fx - dir * 28, fy + 10)
    ..close();
  c.drawPath(fish, _f(_pk));
  c.drawCircle(Offset(fx + dir * 9, fy - 3), 3, _f(_ink));
  final depth = st.b;
  for (var i = 0; i < 6; i++) {
    final y0 = surf + i * (s.height - surf) / 6;
    c.drawRect(Rect.fromLTWH(0, y0, s.width, s.height), _f(_o(_bl, depth * .2)));
  }
  final wave = Path()..moveTo(0, surf);
  for (var x = 0.0; x <= s.width; x += 6) {
    wave.lineTo(x, surf + math.sin(x * .12 + t * 3) * 2);
  }
  c.drawPath(wave, _s(_wh, 2.5));
  final boatX = s.width * .78;
  c.drawPath(
      Path()
        ..moveTo(boatX - 14, surf - 6)
        ..lineTo(boatX + 14, surf - 6)
        ..lineTo(boatX + 9, surf + 1)
        ..lineTo(boatX - 9, surf + 1)
        ..close(),
      _f(_or));
  final ropeEnd = surf + 4 + depth * (s.height - surf - 24);
  c.drawLine(Offset(boatX, surf), Offset(boatX, ropeEnd), _s(_wh, 1.5));
  c.drawCircle(Offset(boatX, ropeEnd), 3.5, _f(_wh));
}

void _gelStack(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height * .64);
  Path dia(Offset o, double w, double h) => Path()
    ..moveTo(o.dx, o.dy - h)
    ..lineTo(o.dx + w, o.dy)
    ..lineTo(o.dx, o.dy + h)
    ..lineTo(o.dx - w, o.dy)
    ..close();
  for (var i = 0; i < 4; i++) {
    for (var j = 0; j < 4; j++) {
      final o = ctr + Offset((i - j) * 13.0, (i + j - 3) * 7.0);
      c.drawPath(dia(o, 13, 7), _f((i + j).isEven ? _wh : _ink));
    }
  }
  c.drawPath(
      Path()
        ..moveTo(ctr.dx - 18, ctr.dy - 6)
        ..lineTo(ctr.dx - 4, ctr.dy - 30)
        ..lineTo(ctr.dx + 10, ctr.dy - 6),
      _s(_re, 5));
  final n = 1 + (st.b * 7).round();
  final cols = [_cy, _pk, _ye];
  for (var k = 0; k < n; k++) {
    final o = ctr + Offset(0, -10 - k * 6.0 + (k == n - 1 ? math.sin(t * 3) * 1.5 : 0));
    c.drawPath(dia(o, 50, 27), _f(_o(cols[k % 3], .32)));
    c.drawPath(dia(o, 50, 27), _s(_o(_wh, .55), 1));
  }
}

void _cocktail(Canvas c, Size s, P7State st, double t) {
  final cx = s.width / 2, top = 16.0, bot = s.height - 18;
  final glass = Path()
    ..moveTo(cx - 30, top)
    ..lineTo(cx + 30, top)
    ..lineTo(cx + 22, bot)
    ..lineTo(cx - 22, bot)
    ..close();
  final liq = Rect.fromLTRB(cx - 32, top + 12, cx + 32, bot);
  final m = st.a * .18;
  final stops = [0.0, (.33 - m).clamp(0, 1).toDouble(), (.33 + m).clamp(0, 1).toDouble(), (.66 - m).clamp(0, 1).toDouble(), (.66 + m).clamp(0, 1).toDouble(), 1.0];
  final colors = [_ye, _ye, _or, _or, _pk, _pk];
  c.save();
  c.clipPath(glass);
  c.drawRect(liq, Paint()..shader = LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: colors, stops: stops).createShader(liq));
  for (var i = 0; i < 5; i++) {
    final ang = t * 1.2 + i * 1.25;
    c.drawCircle(Offset(cx + math.cos(ang) * 14 * st.a, top + 40 + i * 9 + math.sin(ang) * 4), 2 + st.a * 2, _f(_o(_wh, .5 * st.a)));
  }
  c.restore();
  c.drawPath(glass, _s(_ink, 2.5));
  c.drawLine(Offset(cx + 6, bot - 10), Offset(cx + 34, top - 10), _s(_cy, 5));
  c.drawCircle(Offset(cx - 26, top - 2), 7, _f(_re));
  c.drawLine(Offset(cx - 26, top - 9), Offset(cx - 20, top - 16), _s(_li, 2));
  c.drawLine(Offset(cx - 24, bot), Offset(cx + 24, bot), _s(_ink, 3));
}

void _eclipse(Canvas c, Size s, P7State st, double t) {
  for (var i = 0; i < 36; i++) {
    c.drawCircle(Offset(_h(i) * s.width, _h(i + 99) * s.height), .8 + _h(i + 7) * 1.2, _f(_o(_wh, .4 + .5 * math.sin(t * 2 + i).abs())));
  }
  final ctr = Offset(s.width / 2, s.height / 2);
  for (var i = 0; i < 12; i++) {
    final a = i * math.pi / 6 + t * .4;
    c.drawLine(ctr + Offset(math.cos(a), math.sin(a)) * 34, ctr + Offset(math.cos(a), math.sin(a)) * (42 + 4 * math.sin(t * 3 + i)), _s(_or, 3));
  }
  c.drawCircle(ctr, 30, _f(_ye));
  final moon = Offset(st.p.dx * s.width, st.p.dy * s.height);
  c.drawCircle(moon, 26, _f(_cy)..blendMode = BlendMode.difference);
  c.drawCircle(moon, 26, _s(_wh, 1.5));
}

void _neonHeart(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF3A1F24)));
  for (var r = 0; r < 9; r++) {
    final y = r * 14.0;
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_k0, 2));
    for (var x = (r.isEven ? 0.0 : 14.0); x < s.width; x += 28) {
      c.drawLine(Offset(x, y), Offset(x, y + 14), _s(_k0, 2));
    }
  }
  final cx = s.width / 2, cy = s.height / 2 + 2;
  final heart = Path()
    ..moveTo(cx, cy + 32)
    ..cubicTo(cx - 52, cy - 4, cx - 30, cy - 46, cx, cy - 20)
    ..cubicTo(cx + 30, cy - 46, cx + 52, cy - 4, cx, cy + 32);
  final flick = st.b < .35 && math.sin(t * 23) * math.sin(t * 7) > .6 ? .3 : 1.0;
  final on = st.b * flick;
  for (final (w, a) in [(18.0, .12), (11.0, .22), (6.0, .5)]) {
    c.drawPath(heart, _s(_o(_pk, a * on), w));
  }
  c.drawPath(heart, _s(Color.lerp(const Color(0xFF6A5A60), _wh, on)!, 2.5));
  c.drawLine(Offset(cx, cy + 32), Offset(cx, s.height), _s(_ink, 2));
}

void _blinds(Canvas c, Size s, P7State st, double t) {
  final win = Rect.fromLTWH(14, 10, s.width - 28, s.height - 20);
  c.drawRect(win, _f(_cy));
  c.drawCircle(Offset(win.right - 30, win.top + 30), 14, _f(_ye));
  final hills = Path()..moveTo(win.left, win.bottom);
  for (var x = win.left; x <= win.right; x += 4) {
    hills.lineTo(x, win.bottom - 26 - math.sin((x - win.left) * .05) * 10);
  }
  hills
    ..lineTo(win.right, win.bottom)
    ..close();
  c.drawPath(hills, _f(_li));
  c.drawCircle(Offset(win.left + 30, win.bottom - 40), 9, _f(_or));
  c.drawLine(Offset(win.left + 30, win.bottom - 31), Offset(win.left + 30, win.bottom - 22), _s(_ink, 2));
  const n = 9;
  final pitch = win.height / n;
  final open = st.b;
  for (var i = 0; i < n; i++) {
    final y = win.top + i * pitch;
    final hgt = pitch * (1 - open * .9);
    c.drawRect(Rect.fromLTWH(win.left, y, win.width, hgt), _f(_cr));
    c.drawRect(Rect.fromLTWH(win.left, y + hgt - 2, win.width, 2), _f(_o(_ink, .25)));
  }
  c.drawRect(win, _s(_k1, 5));
  final cordY = win.top + 20 + (1 - open) * 50;
  c.drawLine(Offset(win.right - 8, win.top), Offset(win.right - 8, cordY), _s(_ink, 1.5));
  c.drawCircle(Offset(win.right - 8, cordY + 4), 4, _f(_re));
}

void _pixelBugs(Canvas c, Size s, P7State st, double t) {
  const nx = 13, ny = 9;
  final px = (s.width - 20) / nx, py = (s.height - 16) / ny;
  final bands = [_vi, _vi, _pk, _pk, _or, _or, _ye, _ye, _li];
  for (var j = 0; j < ny; j++) {
    for (var i = 0; i < nx; i++) {
      final k = j * nx + i;
      final eaten = _h(k) * .9 + (i / nx) * .1 < st.a;
      if (eaten) continue;
      final sun = (Offset(i - 8.0, j - 5.0)).distance < 2.4;
      c.drawRect(Rect.fromLTWH(10 + i * px, 8 + j * py, px - 1, py - 1), _f(sun ? _wh : bands[j]));
    }
  }
  for (var b = 0; b < 4; b++) {
    final bx = 16 + _h(b + 30) * (s.width - 32) + math.sin(t * 2 + b) * 6, by = 14 + _h(b + 50) * (s.height - 28) + math.cos(t * 2.5 + b) * 4;
    final legs = math.sin(t * 16 + b) * 2;
    for (var l = -1; l <= 1; l++) {
      c.drawLine(Offset(bx + l * 4, by), Offset(bx + l * 5 + legs, by + 7), _s(_ink, 1.5));
      c.drawLine(Offset(bx + l * 4, by), Offset(bx + l * 5 - legs, by - 7), _s(_ink, 1.5));
    }
    c.drawOval(Rect.fromCenter(center: Offset(bx, by), width: 14, height: 9), _f(_ink));
    c.drawRect(Rect.fromCenter(center: Offset(bx + 9, by - 6), width: 6, height: 5), _f(bands[(b * 2) % 9]));
    c.drawCircle(Offset(bx + 6, by - 1), 1.3, _f(_wh));
  }
}

void _halftone(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2);
  final bolt = Path()
    ..moveTo(ctr.dx + 8, ctr.dy - 46)
    ..lineTo(ctr.dx - 24, ctr.dy + 6)
    ..lineTo(ctr.dx - 2, ctr.dy + 6)
    ..lineTo(ctr.dx - 10, ctr.dy + 46)
    ..lineTo(ctr.dx + 26, ctr.dy - 8)
    ..lineTo(ctr.dx + 4, ctr.dy - 8)
    ..close();
  c.drawPath(bolt, _f(_ye));
  c.drawPath(bolt, _s(_ink, 3));
  const pitch = 9.0;
  final r = st.b * pitch * .62;
  final dot = _f(_pk)..blendMode = BlendMode.multiply;
  for (var y = 0.0; y < s.height + pitch; y += pitch) {
    for (var x = ((y / pitch).round().isEven ? 0.0 : pitch / 2); x < s.width + pitch; x += pitch) {
      final fall = 1 - (x / s.width) * .6;
      c.drawCircle(Offset(x, y), r * fall, dot);
    }
  }
  c.drawRect((Offset.zero & s).deflate(5), _s(_ink, 3));
}

void _modeSlot(Canvas c, Size s, P7State st, double t) {
  final body = Rect.fromLTWH(14, 8, s.width - 40, s.height - 16);
  c.drawRRect(RRect.fromRectAndRadius(body, const Radius.circular(12)), _f(_re));
  final win = body.deflate(12);
  c.drawRRect(RRect.fromRectAndRadius(win, const Radius.circular(6)), _f(_k0));
  const modes = [BlendMode.srcOver, BlendMode.multiply, BlendMode.screen, BlendMode.difference, BlendMode.overlay, BlendMode.plus];
  final pos = st.spin;
  final base = pos.floorToDouble();
  final itemH = win.height * .8;
  c.save();
  c.clipRRect(RRect.fromRectAndRadius(win, const Radius.circular(6)));
  for (var k = -1; k <= 2; k++) {
    final idx = (base + k).toInt();
    final m = modes[idx % modes.length];
    final cy = win.center.dy + (k - (pos - base)) * itemH;
    final r = Rect.fromCenter(center: Offset(win.center.dx, cy), width: win.width, height: itemH);
    c.saveLayer(r, Paint());
    c.drawRect(r.deflate(4), _f(_k1));
    c.drawCircle(Offset(r.center.dx - 10, r.center.dy), 20, _f(_or));
    c.drawCircle(Offset(r.center.dx + 10, r.center.dy), 20, _f(_cy)..blendMode = m);
    c.restore();
  }
  c.drawRect(Rect.fromLTWH(win.left, win.top, win.width, 10), Paint()..shader = LinearGradient(begin: Alignment.topCenter, end: Alignment.bottomCenter, colors: [_k0, _o(_k0, 0)]).createShader(Rect.fromLTWH(win.left, win.top, win.width, 10)));
  c.restore();
  c.drawLine(Offset(win.left - 4, win.center.dy), Offset(win.left + 4, win.center.dy), _s(_ye, 3));
  c.drawLine(Offset(win.right - 4, win.center.dy), Offset(win.right + 4, win.center.dy), _s(_ye, 3));
  final pull = (st.spinV.abs() / 30).clamp(0.0, 1.0);
  final lx = body.right + 12;
  c.drawLine(Offset(lx - 10, body.center.dy), Offset(lx, body.center.dy), _s(_k1, 5));
  final knobY = body.top + 10 + pull * 50;
  c.drawLine(Offset(lx, body.center.dy), Offset(lx, knobY), _s(_cr, 3));
  c.drawCircle(Offset(lx, knobY), 7, _f(_ye));
}

void _knife(Canvas c, Size s, P7State st, double t) {
  final board = Path()..addOval(Rect.fromLTWH(-20, 6, s.width + 30, s.height - 4));
  c.drawPath(board, _f(const Color(0xFFE2C79A)));
  c.drawCircle(Offset(s.width - 24, s.height - 26), 9, _f(_cr));
  final y = s.height * .48;
  final l = Offset(26, y), r = Offset(s.width - 34, y);
  final mixL = Offset.lerp(l, r, .5 - st.a * .45)!, mixR = Offset.lerp(l, r, .5 + st.a * .45)!;
  final smear = Rect.fromLTRB(l.dx, y - 12, r.dx, y + 12);
  final mid = Color.lerp(_bl, _ye, .55)!;
  c.drawRRect(
      RRect.fromRectAndRadius(smear, const Radius.circular(12)),
      Paint()
        ..shader = LinearGradient(colors: [_bl, mid, mid, _ye], stops: [0, (mixL.dx - l.dx) / (r.dx - l.dx), (mixR.dx - l.dx) / (r.dx - l.dx), 1])
            .createShader(smear));
  for (final (p, col) in [(l, _bl), (r, _ye)]) {
    c.drawCircle(p, 17, _f(col));
    c.drawCircle(p + const Offset(-5, -5), 4, _f(_o(_wh, .7)));
  }
  final kx = l.dx + (r.dx - l.dx) * st.a;
  final blade = Path()
    ..moveTo(kx - 10, y - 8)
    ..lineTo(kx + 10, y - 8)
    ..lineTo(kx + 6, y + 12)
    ..lineTo(kx - 6, y + 12)
    ..close();
  c.drawPath(blade, _f(const Color(0xFFC9CCD2)));
  c.drawPath(blade, _s(_ink, 1.5));
  c.drawLine(Offset(kx, y - 8), Offset(kx + 4, y - 40), _s(_ink, 2));
  c.drawLine(Offset(kx + 4, y - 40), Offset(kx + 7, y - 56), _s(_or, 7));
}

void _sunrise(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF1B1840)));
  final sunY = s.height * (.95 - st.b * .75);
  final sun = Offset(s.width * .55, sunY);
  final rng = [0.0, 18.0, 30.0, 52.0, 66.0, 88.0, 104.0, 124.0, 140.0, 160.0];
  for (var i = 0; i < rng.length - 1; i++) {
    final bh = 30 + _h(i + 3) * 50;
    final rect = Rect.fromLTRB(rng[i], s.height - bh, rng[i + 1] - 2, s.height);
    c.drawRect(rect, _f(_k0));
    for (var wy = rect.top + 6; wy < s.height - 6; wy += 9) {
      for (var wx = rect.left + 3; wx < rect.right - 4; wx += 6) {
        if (_h((wx * 7 + wy).toInt()) > .6 - st.b * .1) {
          c.drawRect(Rect.fromLTWH(wx, wy, 3, 4), _f(_o(_ye, .8 * (1 - st.b))));
        }
      }
    }
  }
  c.drawCircle(sun, 16, _f(_ye)..blendMode = BlendMode.screen);
  c.drawRect(
      Offset.zero & s,
      Paint()
        ..blendMode = BlendMode.screen
        ..shader = ui.Gradient.radial(sun, 70 + st.b * 60, [_o(_or, .95 * st.b + .2), _o(_pk, .5 * st.b), _o(_vi, 0)], [0, .45, 1]));
}

void _puppet(Canvas c, Size s, P7State st, double t) {
  for (var i = 0; i < 8; i++) {
    c.drawRect(Rect.fromLTWH(i * 20.0, 0, 10, s.height), _f(_o(_cy, .45)));
  }
  final d = st.a;
  final scale = .6 + d * .7;
  final ctr = Offset(s.width * .55, s.height * .48);
  c.save();
  c.translate(ctr.dx, ctr.dy);
  c.scale(scale);
  final ear = math.sin(t * 3) * .12;
  final rabbit = Path()
    ..addOval(Rect.fromCenter(center: Offset.zero, width: 52, height: 40))
    ..addOval(Rect.fromCenter(center: const Offset(26, 10), width: 26, height: 16));
  rabbit.addPath(
      Path()..addOval(Rect.fromCenter(center: const Offset(-6, -40), width: 14, height: 46)), Offset.zero,
      matrix4: (Matrix4.identity()..rotateZ(-.3 + ear)).storage);
  rabbit.addPath(
      Path()..addOval(Rect.fromCenter(center: const Offset(10, -40), width: 14, height: 46)), Offset.zero,
      matrix4: (Matrix4.identity()..rotateZ(.25 - ear)).storage);
  rabbit.addRect(Rect.fromLTWH(-30, 14, 26, 60));
  final shade = _f(_o(const Color(0xFF3B2A6E), .95 - d * .6))..blendMode = BlendMode.multiply;
  c.drawPath(rabbit, shade);
  c.drawCircle(const Offset(8, -4), 3.5, _f(_cr));
  c.restore();
  final lamp = Offset(14, s.height - 14);
  c.drawCircle(lamp, 10 + d * 3, _f(_o(_ye, .5)));
  c.drawCircle(lamp, 6, _f(_ye));
  c.drawCircle(lamp, 6, _s(_ink, 1.5));
}

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
