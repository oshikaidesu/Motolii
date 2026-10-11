// Noise sheet: frequency (how fine), amplitude (how strong), evolution (how it changes over time).
// Convention: drag right = finer, drag up = stronger; rubbing or spinning pushes evolution where the picture asks for it.
part of 'pop_03.dart';

const _noisePanels = <_P>[
  _P('Mountain Range', 'landscape · bold · side · drag', _nMount, init: Offset(.35, .35)),
  _P('Topo Map', 'map · crisp · top · drag', _nTopo, init: Offset(.4, .4)),
  _P('Block Terrain', 'toy · bold · iso · drag', _nIso, init: Offset(.4, .35)),
  _P('Little Boat Sea', 'weather · crisp · side · rub', _nOcean, init: Offset(.3, .4)),
  _P('Jelly Critter', 'creature · neon · flat · drag', _nJelly, init: Offset(.35, .4)),
  _P('Retro TV Snow', 'machine · bold · flat · rub', _nTv, init: Offset(.5, .3)),
  _P('Campfire', 'nature · neon! · side · drag', _nFire, init: Offset(.4, .3)),
  _P('Wood Grain', 'material · light · flat · rub', _nWood, init: Offset(.3, .45), bg: _kCream, anim: false),
  _P('Film Grain', 'result · crisp · flat · drag', _nFilm, init: Offset(.4, .45)),
  _P('Boiling Ink Star', 'text-art · light · flat · drag', _nBoil, init: Offset(.4, .35), bg: _kCream),
  _P('Cloud Sky', 'weather · bold · flat · blow', _nCloud, init: Offset(.35, .45), bg: _kBlue),
  _P('Gas Giant', 'cosmic · bold · 3D · spin', _nGiant, init: Offset(.35, .4)),
  _P('Wind in Grass', 'nature · crisp · side · paint', _nGrass, init: Offset(.35, .35), bg: _kPanel2),
  _P('Flow Field', 'physics · neon! · flat · drag', _nFlow, init: Offset(.4, .4), bg: _kNight),
  _P('Lava Lamp', 'toy · neon · 3D · drag', _nLava, init: Offset(.4, .4)),
  _P('Glyph Halftone', 'text-art · crisp · flat · drag', _nGlyph, init: Offset(.4, .35), bg: _kBlue),
  _P('Noodle Bowl', 'food · bold · top · rub', _nNoodle, init: Offset(.35, .4)),
  _P('Jumping Crowd', 'character · light · side · drag', _nCrowd, init: Offset(.2, .3), bg: _kCream),
  _P('Handheld Camera', 'result · crisp · flat · drag', _nCam, init: Offset(.3, .45), bg: _kNight),
  _P('Noise Sun Burst', 'cosmic · neon! · polar · spin', _nBurst, init: Offset(.45, .3)),
];

void _nMount(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, f = 1 + st.a * 10, amp = .15 + st.b * .85;
  c.drawCircle(Offset(w * .74, h * .28), 13, _f(_kYellow));
  const cols = [_kViolet, _kPink, _kOrange];
  for (var i = 0; i < 3; i++) {
    final base = h * (.6 + i * .15);
    final path = Path()..moveTo(0, h);
    for (var x = 0.0; x <= w + 3; x += 3) {
      final n = _fb(x / w * f * (1 + i * .35) + t * .06 * (i + 1) + i * 7, i * 3.3) * .5 + .5;
      path.lineTo(x, base - n * amp * h * (.55 - i * .12));
    }
    path
      ..lineTo(w, h)
      ..close();
    c.drawPath(path, _f(cols[i]));
  }
}

const _topoCols = [_kBlue, _kCyan, _kLime, _kYellow, _kCream];
void _nTopo(Canvas c, Size s, _St st, double t) {
  const cell = 6.0;
  final f = .6 + st.a * 4, amp = .4 + st.b * 1.4;
  final ps = [for (final k in _topoCols) _f(k)];
  for (var y = 0.0; y < s.height; y += cell) {
    for (var x = 0.0; x < s.width; x += cell) {
      final v = _sat(_fb(x / 60 * f, y / 60 * f + t * .05) * amp * .5 + .5);
      c.drawRect(Rect.fromLTWH(x, y, cell + .4, cell + .4), ps[math.min(4, (v * 5).floor())]);
    }
  }
}

void _nIso(Canvas c, Size s, _St st, double t) {
  const n = 9, tw = 7.6, th = 3.8;
  final ox = s.width / 2, oy = s.height * .3, f = .12 + st.a * .7, amp = 2 + st.b * 40;
  for (var k = 0; k < 2 * n - 1; k++) {
    for (var i = 0; i < n; i++) {
      final j = k - i;
      if (j < 0 || j >= n) continue;
      final hv = _sat(_vn(i * f + t * .15, j * f + 3) * .5 + .5), hh = hv * amp;
      final x = ox + (i - j) * tw, y = oy + (i + j) * th;
      final top = Offset(x, y - hh), rgt = Offset(x + tw, y + th - hh), bot = Offset(x, y + 2 * th - hh), lft = Offset(x - tw, y + th - hh);
      final d = Offset(0, hh + 5);
      final col = _ramp(const [_kBlue, _kCyan, _kLime, _kYellow, _kOrange], hv);
      c.drawPath(_poly([lft, bot, bot + d, lft + d]), _f(_mix(col, _kInk, .45)));
      c.drawPath(_poly([bot, rgt, rgt + d, bot + d]), _f(_mix(col, _kInk, .25)));
      c.drawPath(_poly([top, rgt, bot, lft]), _f(col));
    }
  }
}

void _nOcean(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, f = 1 + st.a * 6, amp = 1.5 + st.b * 11, e = t * .35 + st.rub * 1.4;
  c.drawCircle(Offset(w * .18, h * .2), 9, _f(_kCream));
  double wy(int i, double x) => h * .3 + i * h * .11 + amp * _vn(x / w * f + i * .7 + e * (1 + i * .12), i * 1.7);
  for (var i = 0; i < 7; i++) {
    final path = Path()..moveTo(0, wy(i, 0));
    for (var x = 3.0; x <= w + 3; x += 3) {
      path.lineTo(x, wy(i, x));
    }
    c.drawPath(path, _s(_mix(_kCyan, _kBlue, i / 6), 2.2));
    if (i == 2) {
      final bx = w * .52, by = wy(2, bx), ang = math.atan2(wy(2, bx + 4) - wy(2, bx - 4), 8);
      c
        ..save()
        ..translate(bx, by)
        ..rotate(ang);
      c.drawPath(_poly(const [Offset(-14, -5), Offset(14, -5), Offset(9, 3), Offset(-9, 3)]), _f(_kOrange));
      c.drawPath(_poly(const [Offset(-1, -6), Offset(-1, -26), Offset(12, -8)]), _f(_kWhite));
      c.drawLine(const Offset(-1, -5), const Offset(-1, -27), _s(_kCream, 1.4));
      c.restore();
    }
  }
}

void _nJelly(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ctr = Offset(w / 2, h * .52), f = .6 + st.a * 6, amp = st.b;
  c.drawOval(Rect.fromCenter(center: Offset(w / 2, h * .9), width: 70, height: 8), _f(_al(_kViolet, .35)));
  final pts = <Offset>[];
  for (var k = 0; k < 80; k++) {
    final th = k / 80 * 2 * math.pi;
    final r = 34 * (1 + amp * .38 * _vn(math.cos(th) * f + 5, math.sin(th) * f + t * .9));
    pts.add(ctr + _dir(th) * r);
  }
  final body = _poly(pts);
  c.drawPath(body, _f(_kPink));
  c.drawPath(body, _s(_kYellow, 2.5));
  final look = (st.at(s) - ctr);
  final lk = look.distance < 1 ? Offset.zero : look / look.distance * 2.6;
  for (final ex in [-11.0, 11.0]) {
    final e = ctr + Offset(ex, -6);
    c.drawCircle(e, 7.5, _f(_kWhite));
    c.drawCircle(e + lk, 3.6, _f(_kInk));
  }
  c.drawArc(Rect.fromCenter(center: ctr + const Offset(0, 8), width: 12, height: 8), .2, math.pi - .4, false, _s(_kInk, 2.2));
}

void _nTv(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height;
  final body = RRect.fromRectAndRadius(Rect.fromLTWH(w * .17, h * .24, w * .66, h * .64), const Radius.circular(12));
  c.drawLine(Offset(w * .5, h * .25), Offset(w * .36, h * .06), _s(_kCream, 2));
  c.drawLine(Offset(w * .5, h * .25), Offset(w * .66, h * .04), _s(_kCream, 2));
  c.drawCircle(Offset(w * .36, h * .06), 3, _f(_kYellow));
  c.drawCircle(Offset(w * .66, h * .04), 3, _f(_kYellow));
  c.drawLine(Offset(w * .28, h * .88), Offset(w * .24, h * .96), _s(_kCream, 3));
  c.drawLine(Offset(w * .72, h * .88), Offset(w * .76, h * .96), _s(_kCream, 3));
  c.drawRRect(body, _f(_kOrange));
  final scr = Rect.fromLTWH(w * .23, h * .32, w * .54, h * .48);
  c
    ..save()
    ..clipRRect(RRect.fromRectAndRadius(scr, const Radius.circular(8)));
  final px = 3 + (1 - st.a) * 7, amp = .15 + st.b * .85, frame = (t * 14).floor() + (st.rub * 6).floor();
  for (var y = scr.top; y < scr.bottom; y += px) {
    for (var x = scr.left; x < scr.right; x += px) {
      final v = _hash((x / px).floor() + frame * 131, (y / px).floor() * 7 + frame);
      c.drawRect(Rect.fromLTWH(x, y, px + .3, px + .3), _f(_mix(_kPanel, _kCream, .5 + .5 * v * amp)));
    }
  }
  c.restore();
}

void _nFire(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, base = h * .8, n = 3 + (st.a * 6).round(), amp = .25 + st.b * .75;
  c.drawCircle(Offset(w * .5, base - 10), 60, Paint()..shader = ui.Gradient.radial(Offset(w * .5, base - 10), 60, [_al(_kOrange, .35), _al(_kOrange, 0)]));
  const layers = [(_kRed, 1.0), (_kOrange, .74), (_kYellow, .46)];
  for (var l = 0; l < 3; l++) {
    final (col, sc) = layers[l];
    for (var i = 0; i < n; i++) {
      final u = n == 1 ? .5 : i / (n - 1), x = w * (.3 + u * .4);
      final bell = 1 - (u - .5).abs() * 1.1;
      final ht = h * (.18 + .55 * amp * (_vn(i * 1.7 + 3, t * 2.4 + l) * .5 + .5)) * sc * bell;
      final wd = (w * .5 / n) * 1.9 * sc, sway = 7 * _vn(i * 2.3, t * 3 + l * 4);
      final p = Path()
        ..moveTo(x - wd / 2, base)
        ..quadraticBezierTo(x - wd / 2, base - ht * .55, x + sway, base - ht)
        ..quadraticBezierTo(x + wd / 2, base - ht * .55, x + wd / 2, base)
        ..close();
      c.drawPath(p, _f(col));
    }
  }
  for (var k = 0; k < 5; k++) {
    final ph = (t * .5 + k * .21) % 1;
    c.drawCircle(Offset(w * (.35 + .3 * (_hash(k, 3) * .5 + .5)) + 6 * math.sin(t * 3 + k), base - ph * h * .75), 1.6 * (1 - ph), _f(_kYellow));
  }
  c.drawLine(Offset(w * .26, base + 8), Offset(w * .74, base - 2), _s(const Color(0xFF8A4A2A), 7));
  c.drawLine(Offset(w * .26, base - 2), Offset(w * .74, base + 8), _s(const Color(0xFF6E3A20), 7));
}

void _nWood(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, n = 5 + (st.a * 12).round(), amp = 1 + st.b * 14, e = st.rub * .3;
  final plank = RRect.fromRectAndRadius(Rect.fromLTWH(8, 10, w - 16, h - 20), const Radius.circular(6));
  c.drawRRect(plank, _f(const Color(0xFFFFB27A)));
  c
    ..save()
    ..clipRRect(plank);
  final knot = Offset(w * .62, h * .5);
  final ink = _s(const Color(0xFF8A3A12), 1.5);
  for (var k = 0; k <= n; k++) {
    final y0 = 10 + k * (h - 20) / n;
    final path = Path();
    for (var x = 4.0; x <= w; x += 3) {
      final dx = x - knot.dx, side = y0 < knot.dy ? -1 : 1;
      final bump = 13 * math.exp(-dx * dx / 260) * math.exp(-(y0 - knot.dy) * (y0 - knot.dy) / 700) * side;
      final y = y0 + amp * _vn(x * .03 + e, k * .37) + bump;
      x == 4.0 ? path.moveTo(x, y) : path.lineTo(x, y);
    }
    c.drawPath(path, ink);
  }
  c.drawOval(Rect.fromCenter(center: knot, width: 14, height: 8), _f(const Color(0xFF8A3A12)));
  c.restore();
  c.drawRRect(plank, _s(_kInk, 2));
}

void _nFilm(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height;
  c.drawRect(Rect.fromLTWH(0, h * .06, w, h * .88), _f(_kInk));
  for (var x = 6.0; x < w; x += 14) {
    for (final y in [h * .08, h * .86]) {
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x, y, 7, 6), const Radius.circular(1.5)), _f(_kCream));
    }
  }
  final img = Rect.fromLTWH(w * .07, h * .2, w * .86, h * .6);
  c
    ..save()
    ..clipRect(img);
  c.drawRect(img, _f(_kPink));
  c.drawRect(Rect.fromLTWH(img.left, img.top + img.height * .35, img.width, img.height), _f(_kOrange));
  c.drawCircle(Offset(img.center.dx + 10, img.top + img.height * .55), 17, _f(_kYellow));
  final hill = Path()..moveTo(img.left, img.bottom);
  for (var x = img.left; x <= img.right + 4; x += 4) {
    hill.lineTo(x, img.top + img.height * .7 + 6 * math.sin(x * .05));
  }
  hill
    ..lineTo(img.right, img.bottom)
    ..close();
  c.drawPath(hill, _f(_kViolet));
  final g = 2.6 + (1 - st.a) * 5, amp = st.b, frame = (t * 12).floor();
  final lite = _f(_al(_kWhite, .55 * amp)), dark = _f(_al(_kInk, .6 * amp));
  for (var y = img.top; y < img.bottom; y += g) {
    for (var x = img.left; x < img.right; x += g) {
      final ix = (x / g).floor(), iy = (y / g).floor(), v = _hash(ix + frame * 97, iy + frame * 31);
      if (v.abs() < .35) continue;
      c.drawCircle(Offset(x + g / 2 + _hash(iy, ix + frame) * g * .3, y + g / 2), g * .42, v > 0 ? lite : dark);
    }
  }
  c.restore();
}

void _nBoil(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height * .52), f = 1 + st.a * 12, amp = .5 + st.b * 8, frame = (t * 8).floor();
  final pts = <Offset>[];
  const m = 140;
  for (var i = 0; i < m; i++) {
    final u = i / m * 10, k = u.floor(), fr = u - k;
    final r0 = k.isEven ? 44.0 : 19.0, r1 = k.isEven ? 19.0 : 44.0;
    final a0 = k / 10 * 2 * math.pi - math.pi / 2, a1 = (k + 1) / 10 * 2 * math.pi - math.pi / 2;
    final p = Offset.lerp(_dir(a0) * r0, _dir(a1) * r1, fr)!;
    final n = amp * _vn(i / m * f * 4, frame * 3.7);
    pts.add(ctr + p + (p / p.distance) * n);
  }
  final star = _poly(pts);
  c.drawPath(star.shift(const Offset(4, 4)), _f(_kPink));
  c.drawPath(star, _f(_kYellow));
  c.drawPath(star, _s(_kInk, 3));
  for (final e in [const Offset(-7, -3), const Offset(7, -3)]) {
    c.drawCircle(ctr + e + Offset(0, _hash(frame, 1) * amp * .2), 2.6, _f(_kInk));
  }
}

void _nCloud(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, f = .5 + st.a * 3, thr = .3 - st.b * .75, drift = t * .1 + st.rub * .5;
  c.drawCircle(Offset(w * .82, h * .2), 12, _f(_kYellow));
  final hill = Path()..moveTo(0, h);
  for (var x = 0.0; x <= w + 4; x += 4) {
    hill.lineTo(x, h * .86 - 6 * math.sin(x * .04 + 1));
  }
  hill
    ..lineTo(w, h)
    ..close();
  c.drawPath(hill, _f(_kLime));
  final white = _f(_kWhite), shade = _f(_mix(_kWhite, _kCyan, .35));
  for (final pass in [0, 1]) {
    for (var gy = 0; gy < 7; gy++) {
      for (var gx = 0; gx < 16; gx++) {
        final x = gx * 10.5, y = 8 + gy * 10.5;
        final v = _fb(x / 50 * f + drift, y / 50 * f);
        if (v <= thr) continue;
        final r = 6 + (v - thr) * 12;
        c.drawCircle(Offset(x, y + (pass == 0 ? 2.5 : 0)), r, pass == 0 ? shade : white);
      }
    }
  }
}

void _nGiant(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ctr = Offset(w * .5, h * .52);
  const r = 40.0;
  for (var k = 0; k < 18; k++) {
    c.drawCircle(Offset((_hash(k, 1) * .5 + .5) * w, (_hash(k, 2) * .5 + .5) * h), 1, _f(_al(_kCream, .7)));
  }
  final ring = Rect.fromCenter(center: ctr, width: r * 3.1, height: r * .7);
  c.drawArc(ring, math.pi, math.pi, false, _s(_kViolet, 4));
  final f = .6 + st.a * 4, amp = .2 + st.b * 1.6, e = t * .12 + st.spin * .5;
  const cols = [_kOrange, _kCream, _kPink, _kYellow];
  final ps = [for (final k in cols) _f(k)];
  c
    ..save()
    ..clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  for (var y = ctr.dy - r; y < ctr.dy + r; y += 3) {
    for (var x = ctr.dx - r; x < ctr.dx + r; x += 4) {
      final u = (y - ctr.dy) / r;
      final band = u * f * 1.6 + amp * _vn(x * .06 + e, y * .09);
      final idx = (((band * 2).floor()) % 4 + 4) % 4;
      c.drawRect(Rect.fromLTWH(x, y, 4.3, 3.3), ps[idx]);
    }
  }
  c.drawCircle(ctr, r, Paint()..shader = ui.Gradient.radial(ctr - const Offset(14, 14), r * 1.5, [_al(_kInk, 0), _al(_kInk, .1), _al(_kInk, .7)], [0, .5, 1]));
  c.restore();
  c.drawArc(ring, 0, math.pi, false, _s(_kViolet, 4));
}

void _nGrass(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, base = h * .9, f = .5 + st.a * 5, amp = 2 + st.b * 22;
  c.drawRect(Rect.fromLTWH(0, base, w, h - base), _f(const Color(0xFF3E7A2E)));
  final tp = st.at(s);
  for (var i = 0; i < 52; i++) {
    final x = i * w / 51 + _hash(i, 4) * 2, len = 26 + (_hash(i, 9) * .5 + .5) * 22;
    var lean = amp * _vn(x / w * f - t * .9, 0) + 4;
    if (st.down) {
      final d = x - tp.dx;
      lean += 26 * math.exp(-d * d / 300) * (d >= 0 ? 1 : -1);
    }
    final tip = Offset(x + lean, base - len + lean.abs() * .3);
    final p = Path()
      ..moveTo(x, base)
      ..quadraticBezierTo(x, base - len * .6, tip.dx, tip.dy);
    c.drawPath(p, _s(i % 3 == 0 ? _kYellow : _kLime, 2.2));
    if (i % 9 == 4) c.drawCircle(tip, 3.4, _f(_kPink));
  }
}

void _nFlow(Canvas c, Size s, _St st, double t) {
  final f = .4 + st.a * 3, amp = .3 + st.b * 2.4, e = t * .25;
  final add = Paint()
    ..blendMode = BlendMode.plus
    ..strokeWidth = 2.2
    ..strokeCap = StrokeCap.round
    ..style = PaintingStyle.stroke;
  for (var y = 6.0; y < s.height; y += 11) {
    for (var x = 6.0; x < s.width; x += 11) {
      final ang = _fb(x / 60 * f, y / 60 * f + e) * math.pi * amp;
      final d = _dir(ang) * 5;
      add.color = _mix(_kCyan, _kPink, math.sin(ang) * .5 + .5);
      c.drawLine(Offset(x, y) - d, Offset(x, y) + d, add);
    }
  }
}

void _nLava(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cx = w / 2, top = h * .16, bot = h * .78;
  final glass = _poly([Offset(cx - 13, top), Offset(cx + 13, top), Offset(cx + 26, bot), Offset(cx - 26, bot)]);
  c.drawPath(_poly([Offset(cx - 9, h * .04), Offset(cx + 9, h * .04), Offset(cx + 13, top), Offset(cx - 13, top)]), _f(_kCream));
  c.drawPath(glass, _f(_kViolet));
  c
    ..save()
    ..clipPath(glass);
  final n = 2 + (st.a * 6).round(), amp = st.b;
  c.drawOval(Rect.fromCenter(center: Offset(cx, bot), width: 56, height: 16), _f(_kOrange));
  for (var i = 0; i < n; i++) {
    final ph = .5 - .5 * math.cos((t * .09 * (1 + i * .17) + i * .31) * 2 * math.pi);
    final y = bot - 6 - ph * (bot - top - 8), x = cx + 9 * _vn(i * 3.1, t * .3);
    final r = 4 + amp * 11 * (_vn(i * 2.1, t * .4) * .5 + .5);
    c.drawOval(Rect.fromCenter(center: Offset(x, y), width: r * 2, height: r * 2.3), _f(i.isEven ? _kOrange : _kYellow));
  }
  c.restore();
  c.drawLine(Offset(cx - 8, top + 6), Offset(cx - 15, bot - 8), _s(_al(_kWhite, .35), 2.5));
  c.drawPath(_poly([Offset(cx - 26, bot), Offset(cx + 26, bot), Offset(cx + 20, h * .95), Offset(cx - 20, h * .95)]), _f(_kCream));
}

void _nGlyph(Canvas c, Size s, _St st, double t) {
  final f = .5 + st.a * 4, amp = .4 + st.b * 1.5;
  final y1 = _f(_kYellow), cr = _f(_kCream), ln = _s(_kYellow, 2);
  for (var y = 8.0; y < s.height; y += 12) {
    for (var x = 8.0; x < s.width; x += 12) {
      final v = _sat(_fb(x / 55 * f, y / 55 * f + t * .07) * amp * .5 + .5);
      final lv = (v * 5).floor(), o = Offset(x, y);
      switch (lv) {
        case 0:
          c.drawCircle(o, .9, y1);
        case 1:
          c.drawCircle(o, 2.2, y1);
        case 2:
          c.drawLine(o - const Offset(3.5, 0), o + const Offset(3.5, 0), ln);
        case 3:
          c
            ..drawLine(o - const Offset(4, 0), o + const Offset(4, 0), ln)
            ..drawLine(o - const Offset(0, 4), o + const Offset(0, 4), ln);
        default:
          c.drawRect(Rect.fromCenter(center: o, width: 9, height: 9), cr);
      }
    }
  }
}

void _nNoodle(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2), f = .3 + st.a * 3, amp = 1.5 + st.b * 12, e = t * .12 + st.rub * .8;
  c.drawCircle(ctr + const Offset(3, 4), 54, _f(_al(_kInk, .5)));
  c.drawCircle(ctr, 54, _f(_kRed));
  c.drawCircle(ctr, 47, _f(const Color(0xFFFFE3A8)));
  c
    ..save()
    ..clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: 47)));
  final under = _s(const Color(0xFFD99A1E), 5.5), top = _s(_kYellow, 3.6);
  for (var k = 0; k < 8; k++) {
    final y0 = ctr.dy - 38 + k * 11, path = Path();
    for (var x = ctr.dx - 50; x <= ctr.dx + 50; x += 3) {
      final y = y0 + amp * _vn(x * .04 * f + k * 5, k * 2.3 + e);
      x == ctr.dx - 50 ? path.moveTo(x, y) : path.lineTo(x, y);
    }
    c
      ..drawPath(path, under)
      ..drawPath(path, top);
  }
  c.restore();
  final nar = ctr + const Offset(17, -15);
  c.drawCircle(nar, 10, _f(_kCream));
  final sp = Path()..moveTo(nar.dx, nar.dy);
  for (var a = 0.0; a < 4 * math.pi; a += .3) {
    sp.lineTo(nar.dx + math.cos(a) * a * .6, nar.dy + math.sin(a) * a * .6);
  }
  c.drawPath(sp, _s(_kPink, 1.6));
  for (var k = 0; k < 6; k++) {
    c.drawCircle(ctr + Offset(-22 + _hash(k, 2) * 14, 10 + _hash(k, 5) * 18), 3, _s(_kLime, 2));
  }
}

const _crowdCols = [_kOrange, _kPink, _kBlue, _kViolet, _kRed, _kCyan];
void _nCrowd(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, floor = h * .86, f = .04 + st.a * 1.6, amp = 4 + st.b * 50;
  c.drawLine(Offset(4, floor), Offset(w - 4, floor), _s(_kInk, 2));
  for (var i = 0; i < 9; i++) {
    final x = 13 + i * (w - 26) / 8;
    final jump = amp * _sat(_vn(i * f, t * 1.5) * .9 + .45);
    final sq = jump < 3 ? .82 : 1.06;
    c.drawOval(Rect.fromCenter(center: Offset(x, floor), width: 14 * (1 - jump / 90), height: 3.5), _f(_al(_kInk, .25)));
    final ctr = Offset(x, floor - 8 * sq - jump);
    c.drawOval(Rect.fromCenter(center: ctr, width: 16 / sq, height: 16 * sq), _f(_crowdCols[i % 6]));
    for (final ex in [-3.0, 3.0]) {
      c.drawCircle(ctr + Offset(ex, -2), 2.3, _f(_kWhite));
      c.drawCircle(ctr + Offset(ex, -1.5), 1.1, _f(_kInk));
    }
  }
}

void _nCam(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, fr = Rect.fromLTWH(14, 12, w - 28, h - 24);
  final f = .4 + st.a * 7, amp = st.b;
  final dx = amp * 13 * _vn(t * f, 1.3), dy = amp * 9 * _vn(4.1, t * f), rr = amp * .08 * _vn(t * f * .7, 9);
  c
    ..save()
    ..clipRect(fr)
    ..translate(w / 2 + dx, h / 2 + dy)
    ..rotate(rr)
    ..translate(-w / 2, -h / 2);
  c.drawRect(Rect.fromLTWH(-20, -20, w + 40, h + 40), _f(_kBlue));
  c.drawCircle(Offset(w * .7, h * .35), 13, _f(_kYellow));
  final hill = Path()..moveTo(-20, h + 20);
  for (var x = -20.0; x <= w + 20; x += 4) {
    hill.lineTo(x, h * .7 - 10 * math.sin(x * .035));
  }
  hill
    ..lineTo(w + 20, h + 20)
    ..close();
  c.drawPath(hill, _f(_kLime));
  c.drawRect(Rect.fromLTWH(w * .3, h * .45, 4, 20), _f(_kInk));
  c.drawCircle(Offset(w * .3 + 2, h * .43), 11, _f(_kPink));
  c.restore();
  final br = _s(_kWhite, 2.2);
  for (final (o, sx, sy) in [(fr.topLeft, 1.0, 1.0), (fr.topRight, -1.0, 1.0), (fr.bottomLeft, 1.0, -1.0), (fr.bottomRight, -1.0, -1.0)]) {
    c.drawPath(_poly([o + Offset(0, 10 * sy), o, o + Offset(10 * sx, 0)], close: false), br);
  }
  if ((t * 1.5).floor().isEven) c.drawCircle(Offset(fr.left + 10, fr.top + 10), 3.5, _f(_kRed));
}

void _nBurst(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2), f = .5 + st.a * 4.5, amp = st.b;
  const rings = [(_kPink, 1.0), (_kYellow, .7), (_kCyan, .42)];
  final add = Paint()
    ..blendMode = BlendMode.plus
    ..style = PaintingStyle.stroke
    ..strokeWidth = 3
    ..strokeJoin = StrokeJoin.round;
  for (var k = 0; k < 3; k++) {
    final (col, sc) = rings[k];
    final pts = <Offset>[];
    for (var i = 0; i < 96; i++) {
      final th = i / 96 * 2 * math.pi;
      final n = _vn(math.cos(th + st.spin) * f + k * 3, math.sin(th + st.spin) * f + t * .6) * .5 + .5;
      pts.add(ctr + _dir(th) * (22 + amp * 36 * n) * sc);
    }
    if (k == 0) c.drawPath(_poly(pts), _f(_al(_kViolet, .35)));
    add.color = col;
    c.drawPath(_poly(pts), add);
  }
  c.drawCircle(ctr, 3.5, _f(_kWhite));
}
