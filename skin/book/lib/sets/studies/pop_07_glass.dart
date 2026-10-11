// Glass / Refraction x20: why you touch it = "what is behind the glass bends, swells, frosts or splits into colour".
part of 'pop_07.dart';

final _glassSpecs = <_P7Spec>[
  _P7Spec('Straw in water', 'material · side view light · drag ↕ · index', _straw, b: .5, light: true),
  _P7Spec('Magnifier', 'toy · top-down · drag lens · index', _magnifier, p: const Offset(.42, .45)),
  _P7Spec('Steamy door', 'weather · bold flat · drag ↕ · roughness', _steamDoor, b: .5),
  _P7Spec('Prism', 'physics · neon · drag ↕ · dispersion', _prism, b: .5),
  _P7Spec('Fishbowl', 'creature · pseudo 3D · drag ↔ fish · index', _fishbowl, a: .3, b: .6),
  _P7Spec('Ice cube', 'food · isometric · drag ↕ melt · thickness', _iceCube, b: .7),
  _P7Spec('Raindrops', 'weather · night · paint drops · lens', _raindrops),
  _P7Spec('Heat haze', 'landscape · flat · drag ↕ · wobble', _heatHaze, b: .5),
  _P7Spec('Bottle', 'material · pseudo 3D · drag ↕ · thickness', _bottle, b: .5, light: true),
  _P7Spec('Black hole', 'cosmic · neon wild · drag ↕ · bend', _blackHole, b: .5),
  _P7Spec('Ribbed glass', 'material · bold flat light · drag ↔ · ribs', _ribbed, a: .4, light: true),
  _P7Spec('Marble', 'toy · side view · flick ↔ · flip', _marble),
  _P7Spec('Pool caustics', 'nature · top-down map · drag ↕ · roughness', _caustics, b: .5),
  _P7Spec('Soap bubble', 'material · neon · blow ↑ · film thickness', _bubble, b: .5),
  _P7Spec('Sandpaper', 'material · bold flat · rub · roughness', _sandpaper, p: const Offset(.7, .3)),
  _P7Spec('Lighthouse lens', 'machine · landscape · drag ↕ · focus', _lighthouse, b: .6),
  _P7Spec('Thick glasses', 'character · bold flat light · drag ↕ · thickness', _glasses, b: .5, light: true),
  _P7Spec('Paperweight', 'object · isometric · drag ↕ · thickness', _paperweight, b: .5),
  _P7Spec('Gem fire', 'cosmic · neon wild · spin · index', _gem, b: .6),
  _P7Spec('Glass slime', 'creature · bold flat · squish ↕ · bend', _slime, b: .5),
];

/// Polyline through [pts] after moving each point by [f]: the cheap way to show a picture bent by glass.
void _bent(Canvas c, Iterable<Offset> pts, Offset Function(Offset) f, Paint p) {
  final path = Path();
  var first = true;
  for (final q in pts) {
    final r = f(q);
    first ? path.moveTo(r.dx, r.dy) : path.lineTo(r.dx, r.dy);
    first = false;
  }
  c.drawPath(path, p);
}

Iterable<Offset> _vline(double x, double h, [double step = 4]) sync* {
  for (var y = -4.0; y <= h + 4; y += step) {
    yield Offset(x, y);
  }
}

Iterable<Offset> _hline(double y, double w, [double step = 4]) sync* {
  for (var x = -4.0; x <= w + 4; x += step) {
    yield Offset(x, y);
  }
}

void _straw(Canvas c, Size s, P7State st, double t) {
  final cx = s.width / 2, top = 12.0, bot = s.height - 10, water = s.height * .45;
  final glass = Path()
    ..moveTo(cx - 34, top)
    ..lineTo(cx + 34, top)
    ..lineTo(cx + 27, bot)
    ..lineTo(cx - 27, bot)
    ..close();
  c.save();
  c.clipPath(glass);
  c.drawRect(Rect.fromLTRB(0, water, s.width, s.height), _f(_o(_cy, .55)));
  c.restore();
  final a0 = Offset(cx + 44, top - 24), hit = Offset(cx + 10, water);
  final bend = st.b;
  final ang0 = math.atan2(hit.dy - a0.dy, hit.dx - a0.dx);
  final ang = ang0 + (math.pi / 2 - ang0) * bend * 1.1;
  final kink = hit + Offset(bend * 24, 0);
  final end = kink + Offset(math.cos(ang), math.sin(ang)) * 70;
  final straw = _s(_pk, 7)..strokeCap = StrokeCap.butt;
  c.drawLine(a0, hit, straw);
  c.save();
  c.clipPath(glass);
  c.clipRect(Rect.fromLTRB(0, water, s.width, s.height));
  c.drawLine(kink, end, straw);
  c.drawLine(kink, end, _s(_o(_cy, .2), 7)..strokeCap = StrokeCap.butt);
  c.restore();
  final wave = Path()..moveTo(cx - 32, water);
  for (var x = cx - 32; x <= cx + 32; x += 4) {
    wave.lineTo(x, water + math.sin(x * .3 + t * 3) * 1.2);
  }
  c.drawPath(wave, _s(_bl, 2));
  c.drawPath(glass, _s(_ink, 2.5));
  c.drawCircle(Offset(cx - 14, water + 20 + math.sin(t * 2) * 10), 2.5, _s(_bl, 1.2));
}

void _magnifier(Canvas c, Size s, P7State st, double t) {
  final lens = Offset(st.p.dx * s.width, st.p.dy * s.height);
  const r = 30.0;
  final cols = [_or, _ye, _cy, _li, _pk, _vi];
  for (var j = 0; j < 9; j++) {
    for (var i = 0; i < 12; i++) {
      final p = Offset(8 + i * 13.0 + (j.isEven ? 0 : 6), 8 + j * 13.0);
      final d = p - lens;
      var q = p;
      var rad = 2.2;
      if (d.distance < r) {
        final k = 1 - (d.distance / r);
        q = lens + d * (.45 + .55 * d.distance / r);
        rad = 2.2 * (1 + 2.2 * k);
      }
      c.drawCircle(q, rad, _f(cols[(i + j) % cols.length]));
    }
  }
  c.drawCircle(lens, r, _f(_o(_wh, .08)));
  c.drawLine(lens + const Offset(r * .72, r * .72), lens + const Offset(r * 1.5, r * 1.5), _s(_or, 7));
  c.drawCircle(lens, r, _s(_wh, 4));
  c.drawArc(Rect.fromCircle(center: lens, radius: r - 7), 3.6, .9, false, _s(_o(_wh, .7), 2.5));
}

void _steamDoor(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(_bl));
  for (var y = 0.0; y < s.height; y += 16) {
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_o(_wh, .25), 1));
  }
  final rough = st.b;
  final duck = Offset(s.width / 2, s.height * .58);
  void drawDuck(Offset o, double a) {
    c.drawOval(Rect.fromCenter(center: o, width: 46, height: 28), _f(_o(_ye, a)));
    c.drawCircle(o + const Offset(14, -18), 13, _f(_o(_ye, a)));
    c.drawPath(
        Path()
          ..moveTo(o.dx + 25, o.dy - 20)
          ..lineTo(o.dx + 36, o.dy - 16)
          ..lineTo(o.dx + 25, o.dy - 13)
          ..close(),
        _f(_o(_or, a)));
    c.drawCircle(o + const Offset(18, -21), 2.2, _f(_o(_ink, a)));
  }

  final bob = Offset(0, math.sin(t * 2) * 2);
  final n = rough < .05 ? 1 : 7;
  for (var i = 0; i < n; i++) {
    final j = n == 1 ? Offset.zero : Offset(math.cos(i * 2.4), math.sin(i * 2.4)) * rough * 9;
    drawDuck(duck + bob + j, n == 1 ? 1 : .32);
  }
  c.drawRect(Offset.zero & s, _f(_o(_cr, rough * .45)));
  for (var i = 0; i < 70; i++) {
    if (_h(i) > rough) continue;
    c.drawCircle(Offset(_h(i + 3) * s.width, _h(i + 9) * s.height), .8 + _h(i + 1) * 1.6, _f(_o(_wh, .7)));
  }
  for (var i = 0; i < 3; i++) {
    final x = 30 + i * 46.0, y0 = (t * 8 + i * 30) % (s.height + 20) - 10;
    if (rough > .3) c.drawLine(Offset(x, y0), Offset(x, y0 + 12), _s(_o(_wh, .55), 2));
  }
  c.drawRect((Offset.zero & s).deflate(3), _s(_k1, 6));
}

void _prism(Canvas c, Size s, P7State st, double t) {
  final a = Offset(s.width * .46, 16), b = Offset(s.width * .28, s.height - 18), d = Offset(s.width * .64, s.height - 18);
  final hitIn = Offset.lerp(a, b, .5)!, hitOut = Offset.lerp(a, d, .55)!;
  c.drawLine(Offset(0, hitIn.dy + 14), hitIn, _s(_wh, 4));
  final cols = [_re, _or, _ye, _li, _cy, _vi];
  final spread = .05 + st.b * .5;
  final mid = Offset(hitIn.dx + 14, hitIn.dy + 4);
  for (var i = 0; i < 6; i++) {
    final ang = -.05 + (i - 2.5) * spread / 5 + .25;
    final e = hitOut + Offset(math.cos(ang), math.sin(ang)) * 120;
    final band = Path()
      ..moveTo(mid.dx, mid.dy)
      ..lineTo(hitOut.dx, hitOut.dy)
      ..lineTo(e.dx, e.dy);
    c.drawPath(band, _s(_o(cols[i], .95), 3.5));
  }
  final tri = Path()
    ..moveTo(a.dx, a.dy)
    ..lineTo(b.dx, b.dy)
    ..lineTo(d.dx, d.dy)
    ..close();
  c.drawPath(tri, _f(_o(_wh, .12)));
  c.drawPath(tri, _s(_wh, 2.5));
  c.drawLine(Offset.lerp(a, b, .2)!, Offset.lerp(a, b, .45)!, _s(_o(_wh, .6), 4));
  final sparkle = .5 + .5 * math.sin(t * 5);
  c.drawCircle(hitIn, 3 + sparkle * 2, _f(_wh));
}

void _fishbowl(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height * .54);
  const r = 46.0;
  c.drawRect(Rect.fromLTWH(0, ctr.dy + r - 4, s.width, s.height), _f(_k1));
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  c.drawRect(Rect.fromLTRB(0, ctr.dy - r * .55, s.width, s.height), _f(_o(_cy, .35)));
  for (var i = 0; i < 9; i++) {
    c.drawCircle(Offset(ctr.dx - 34 + i * 8.5, ctr.dy + r - 6 + _h(i) * 3), 5, _f([_pk, _ye, _li][i % 3]));
  }
  final fx = ctr.dx + (st.a - .5) * 2 * (r - 12);
  final u = (fx - ctr.dx) / r;
  final mag = 1 + st.b * 1.1 * (1 - u * u);
  final fy = ctr.dy + math.sin(t * 1.4) * 6;
  c.save();
  c.translate(fx, fy);
  c.scale(mag, mag * .9 + .1);
  final flap = math.sin(t * 8) * 3;
  c.drawPath(
      Path()
        ..addOval(Rect.fromCenter(center: Offset.zero, width: 24, height: 15))
        ..moveTo(-10, 0)
        ..lineTo(-20, -7 + flap)
        ..lineTo(-20, 7 + flap)
        ..close(),
      _f(_or));
  c.drawCircle(const Offset(6, -2), 2.4, _f(_wh));
  c.drawCircle(const Offset(6.6, -2), 1.2, _f(_ink));
  c.restore();
  for (var i = 0; i < 3; i++) {
    final by = ctr.dy + r - ((t * 14 + i * 22) % (r * 1.4));
    c.drawCircle(Offset(fx + 10 + i * 3, by), 2 + i * .5, _s(_o(_wh, .7), 1));
  }
  c.restore();
  final water = Path()..moveTo(ctr.dx - r * .84, ctr.dy - r * .55);
  for (var x = -r * .84; x <= r * .84; x += 4) {
    water.lineTo(ctr.dx + x, ctr.dy - r * .55 + math.sin(x * .3 + t * 3) * 1.2);
  }
  c.drawPath(water, _s(_wh, 1.8));
  c.drawArc(Rect.fromCircle(center: ctr, radius: r), -math.pi * .30, math.pi * 1.6, false, _s(_wh, 3));
  c.drawArc(Rect.fromCircle(center: ctr, radius: r - 8), math.pi * 1.05, .7, false, _s(_o(_wh, .55), 4));
}

void _iceCube(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height * .66);
  const ux = Offset(16, 9), uy = Offset(-16, 9);
  for (var i = -3; i < 3; i++) {
    for (var j = -3; j < 3; j++) {
      final o = ctr + ux * i.toDouble() + uy * j.toDouble();
      final quad = Path()..addPolygon([o, o + ux, o + ux + uy, o + uy], true);
      c.drawPath(quad, _f((i + j).isEven ? _re : _cr));
    }
  }
  final size = .35 + .65 * st.b;
  final puddle = 1 - st.b;
  c.drawOval(Rect.fromCenter(center: ctr + const Offset(0, 9), width: 40 + puddle * 80, height: 20 + puddle * 40), _f(_o(_cy, .45)));
  final e = 34 * size;
  final ex = Offset(e * .87, e * .5), ey = Offset(-e * .87, e * .5), ez = Offset(0, -e);
  final b0 = ctr + const Offset(0, -2) - (ex + ey) / 2;
  final topF = [b0 + ez, b0 + ez + ex, b0 + ez + ex + ey, b0 + ez + ey];
  final left = [b0 + ey, b0 + ex + ey, b0 + ex + ey + ez, b0 + ey + ez];
  final right = [b0 + ex, b0 + ex + ey, b0 + ex + ey + ez, b0 + ex + ez];
  c.save();
  c.clipPath(Path()..addPolygon(left, true)..addPolygon(right, true));
  final shift = Offset(0, -e * .35);
  for (var i = -3; i < 3; i++) {
    for (var j = -3; j < 3; j++) {
      final o = ctr + shift + ux * i.toDouble() + uy * j.toDouble();
      if ((i + j).isEven) c.drawPath(Path()..addPolygon([o, o + ux, o + ux + uy, o + uy], true), _f(_o(_re, .55)));
    }
  }
  c.restore();
  c.drawPath(Path()..addPolygon(left, true), _f(_o(_cy, .28)));
  c.drawPath(Path()..addPolygon(right, true), _f(_o(_bl, .25)));
  c.drawPath(Path()..addPolygon(topF, true), _f(_o(_wh, .55)));
  for (final face in [left, right, topF]) {
    c.drawPath(Path()..addPolygon(face, true), _s(_wh, 1.8));
  }
  c.drawLine(b0 + ey + ez * .3 + ex * .2, b0 + ey + ez * .7 + ex * .5, _s(_wh, 1.2));
  final drip = (t * .7) % 1;
  c.drawCircle(b0 + ex + ey + Offset(0, 4 + drip * 10), 2.2, _f(_o(_cy, 1 - drip)));
}

void _raindrops(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF14142A)));
  final lights = <(Offset, Color)>[];
  for (var i = 0; i < 14; i++) {
    final p = Offset(_h(i) * s.width, s.height * (.35 + _h(i + 20) * .6));
    final col = [_or, _pk, _ye, _cy, _vi][i % 5];
    lights.add((p, col));
    c.drawCircle(p, 7 + _h(i + 5) * 8, _f(_o(col, .35)));
  }
  final drops = <Offset>[
    for (var i = 0; i < 9; i++) Offset(_h(i + 60) * s.width, ((_h(i + 70) * s.height) + t * (3 + _h(i) * 6)) % (s.height + 20) - 10),
    for (final m in st.marks) Offset(m.dx * s.width, m.dy * s.height),
  ];
  for (var k = 0; k < drops.length; k++) {
    final d = drops[k];
    final r = k < 9 ? 5 + _h(k + 80) * 6 : 6.0;
    c.drawCircle(d, r, _f(const Color(0xFF26264A)));
    c.save();
    c.clipPath(Path()..addOval(Rect.fromCircle(center: d, radius: r)));
    for (final (p, col) in lights) {
      final q = d - (p - Offset(s.width / 2, s.height / 2)) * (r / 60);
      c.drawCircle(q, r * .35, _f(col));
    }
    c.restore();
    c.drawCircle(d, r, _s(_o(_wh, .7), 1.2));
    c.drawCircle(d + Offset(-r * .4, -r * .4), r * .22, _f(_wh));
  }
}

void _heatHaze(Canvas c, Size s, P7State st, double t) {
  c.drawRect(Offset.zero & s, _f(_ye));
  final hz = s.height * .48;
  c.drawCircle(Offset(s.width * .78, hz - 30), 13, _f(_wh));
  final amp = st.b * 7;
  void scenery(double dy, double flip) {
    final m = Path()..moveTo(0, hz);
    for (final (x, y) in [(0.0, 0.0), (22.0, -22.0), (40.0, -10.0), (62.0, -34.0), (86.0, -8.0), (106.0, -20.0), (130.0, 0.0), (156.0, -12.0)]) {
      m.lineTo(x, hz + dy + y * flip);
    }
    m
      ..lineTo(s.width, hz + dy)
      ..lineTo(s.width, hz)
      ..close();
    c.drawPath(m, _f(_pk));
    c.drawRect(Rect.fromLTWH(118, hz + dy + (flip > 0 ? -26 : 0), 5, 26), _f(_li));
  }

  scenery(0, 1);
  c.drawRect(Rect.fromLTWH(0, hz, s.width, s.height), _f(_or));
  final road = Path()
    ..moveTo(s.width / 2 - 4, hz)
    ..lineTo(s.width / 2 + 4, hz)
    ..lineTo(s.width * .95, s.height)
    ..lineTo(s.width * .05, s.height)
    ..close();
  c.drawPath(road, _f(_k1));
  for (var i = 0; i < 4; i++) {
    final y0 = hz + 8 + i * i * 6.0, y1 = y0 + 3 + i * 2.0;
    c.drawLine(Offset(s.width / 2, y0), Offset(s.width / 2, y1), _s(_ye, 1 + i * .8));
  }
  for (var k = 0; k < 7; k++) {
    final y0 = hz - 18 + k * 5.0;
    c.save();
    c.clipRect(Rect.fromLTWH(0, y0, s.width, 5));
    c.translate(math.sin(k * 1.3 + t * 5) * amp, 0);
    if (k >= 4) {
      c.drawRect(Rect.fromLTWH(-10, y0, s.width + 20, 5), _f(_o(_cy, st.b * .7)));
    } else {
      c.drawRect(Rect.fromLTWH(-10, y0, s.width + 20, 5), _f(_ye));
      scenery(0, 1);
    }
    c.restore();
  }
}

void _bottle(Canvas c, Size s, P7State st, double t) {
  final cols = [_or, _ye, _pk, _bl, _li];
  void stripes() {
    for (var i = 0; i < 14; i++) {
      c.drawRect(Rect.fromLTWH(i * 12.0 - 6 + (t * 6) % 12, 0, 6, s.height), _f(cols[i % cols.length]));
    }
  }

  stripes();
  final cx = s.width / 2;
  final bottle = Path()
    ..moveTo(cx - 7, 6)
    ..lineTo(cx + 7, 6)
    ..lineTo(cx + 7, 30)
    ..quadraticBezierTo(cx + 26, 40, cx + 26, 56)
    ..lineTo(cx + 26, s.height - 8)
    ..lineTo(cx - 26, s.height - 8)
    ..lineTo(cx - 26, 56)
    ..quadraticBezierTo(cx - 26, 40, cx - 7, 30)
    ..close();
  final k = 1 - st.b * .7;
  c.save();
  c.clipPath(bottle);
  c.drawRect(Offset.zero & s, _f(_cr));
  c.translate(cx, 0);
  c.scale(-k, 1);
  c.translate(-cx, 0);
  stripes();
  c.restore();
  c.drawPath(bottle, _f(_o(_li, .18 + st.b * .25)));
  c.drawPath(bottle, _s(_ink, 2.5 + st.b * 3));
  c.drawLine(Offset(cx - 18, 60), Offset(cx - 18, s.height - 18), _s(_o(_wh, .75), 3));
}

void _blackHole(Canvas c, Size s, P7State st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2);
  final m = 8 + st.b * 16;
  for (var i = 0; i < 90; i++) {
    final p = Offset(_h(i) * s.width, _h(i + 300) * s.height);
    final d = p - ctr;
    final r = d.distance;
    if (r < 1) continue;
    final r2 = r + m * m / r;
    final q = ctr + d / r * r2;
    c.drawCircle(q, .9 + _h(i + 5) * 1.1, _f(_o(_wh, .5 + .5 * _h(i + 9))));
    final img = ctr - d / r * (m * m / r2);
    if ((img - ctr).distance > m * .9) c.drawCircle(img, .9, _f(_o(_cy, .7)));
  }
  final disk = Rect.fromCenter(center: Offset.zero, width: m * 5.2, height: m * 1.2);
  c.save();
  c.translate(ctr.dx, ctr.dy);
  c.rotate(-.18);
  c.drawOval(disk, _s(_or, m * .34));
  c.drawOval(disk.deflate(m * .25), _s(_ye, m * .12));
  c.restore();
  c.drawCircle(ctr, m * 1.42, _s(_or, m * .32));
  c.drawCircle(ctr, m * 1.42, _s(_ye, m * .1));
  c.drawCircle(ctr, m * 1.12, _s(_pk, 1.5));
  c.drawCircle(ctr, m * 1.05, _f(const Color(0xFF000000)));
  c.save();
  c.translate(ctr.dx, ctr.dy);
  c.rotate(-.18);
  c.clipRect(Rect.fromLTRB(-200, 0, 200, 200));
  c.drawOval(disk, _s(_or, m * .34));
  c.drawOval(disk.deflate(m * .25), _s(_ye, m * .12));
  c.restore();
  final spin = t * 2;
  c.drawCircle(ctr + Offset(math.cos(spin) * m * 2.1, math.sin(spin) * m * .5).scale(1, 1), 2, _f(_wh));
}

void _ribbed(Canvas c, Size s, P7State st, double t) {
  void scene() {
    c.drawRect(Offset.zero & s, _f(_pk));
    final face = Offset(s.width / 2 + math.sin(t * .8) * 18, s.height / 2);
    c.drawCircle(face, 34, _f(_ye));
    c.drawCircle(face + const Offset(-12, -6), 4.5, _f(_ink));
    c.drawCircle(face + const Offset(12, -6), 4.5, _f(_ink));
    c.drawArc(Rect.fromCenter(center: face + const Offset(0, 6), width: 34, height: 24), .3, math.pi - .6, false, _s(_ink, 3.5));
  }

  final w = 6 + st.a * 22;
  for (var x = 0.0; x < s.width; x += w) {
    final cx = x + w / 2;
    c.save();
    c.clipRect(Rect.fromLTWH(x, 0, w, s.height));
    c.translate(cx, 0);
    c.scale(-1.4, 1);
    c.translate(-cx, 0);
    scene();
    c.restore();
    c.drawLine(Offset(x, 0), Offset(x, s.height), _s(_o(_wh, .55), 1.2));
    c.drawLine(Offset(x + w * .25, 0), Offset(x + w * .25, s.height), _s(_o(_wh, .25), 1.5));
  }
}

void _marble(Canvas c, Size s, P7State st, double t) {
  final ground = s.height * .7;
  c.drawRect(Rect.fromLTWH(0, 0, s.width, ground), _f(_cy));
  c.drawRect(Rect.fromLTWH(0, ground, s.width, s.height), _f(_li));
  c.drawCircle(Offset(s.width * .82, 22), 10, _f(_ye));
  void flower(Offset o, double k) {
    for (var i = 0; i < 5; i++) {
      final a = i * math.pi * 2 / 5;
      c.drawCircle(o + Offset(math.cos(a), math.sin(a)) * 5 * k, 4 * k, _f(_pk));
    }
    c.drawCircle(o, 3 * k, _f(_ye));
  }

  c.drawLine(Offset(28, ground), Offset(28, ground - 22), _s(const Color(0xFF3B7A2E), 2.5));
  flower(Offset(28, ground - 24), 1);
  const r = 24.0;
  final span = s.width - 2 * r - 8;
  final u = (st.spin / 6) % 2;
  final x = r + 4 + (u < 1 ? u : 2 - u) * span;
  final ctr = Offset(x, ground - r);
  c.drawOval(Rect.fromCenter(center: Offset(x, ground + 2), width: r * 1.8, height: 6), _f(_o(_ink, .25)));
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  c.drawRect(Rect.fromLTRB(ctr.dx - r, ctr.dy - r, ctr.dx + r, ctr.dy - 4), _f(_li));
  c.drawRect(Rect.fromLTRB(ctr.dx - r, ctr.dy - 4, ctr.dx + r, ctr.dy + r), _f(_cy));
  c.drawCircle(ctr + const Offset(-10, 13), 4, _f(_ye));
  flower(ctr + const Offset(8, -10), .55);
  c.translate(ctr.dx, ctr.dy);
  c.rotate(st.spin * .6);
  c.drawArc(Rect.fromCircle(center: Offset.zero, radius: r * .7), 0, 2.2, false, _s(_o(_or, .9), 4));
  c.restore();
  c.drawCircle(ctr, r, _s(_wh, 2.5));
  c.drawCircle(ctr + const Offset(-9, -10), 5, _f(_o(_wh, .85)));
}

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
