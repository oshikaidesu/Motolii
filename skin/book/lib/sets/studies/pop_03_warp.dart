// Warp / Distort sheet: strength (how much the picture bends), radius (how much of it), twist (how much it swirls).
part of 'pop_03.dart';

const _warpPanels = <_P>[
  _P('Gingham Pull', 'material · light · flat · pull', _wFabric, init: Offset(.7, .3), init0: Offset(.5, .55), bg: _kCream, anim: false),
  _P('Whirlpool', 'nature · bold · top · spin', _wWhirl, init: Offset(.8, .5), bg: _kBlue),
  _P('Funhouse Mirror', 'toy · result · flat · drag', _wMirror, init: Offset(.45, .25), bg: _kPanel2, anim: false),
  _P('Black Hole', 'cosmic · neon · flat · drag+spin', _wHole, init: Offset(.55, .4), bg: _kNight),
  _P('Taffy Twist', 'food · bold · 3D · spin', _wTaffy, init: Offset(.5, .3)),
  _P('Door Peephole', 'machine · result · lens · drag', _wPeep, init: Offset(.75, .25), bg: _kPanel2, anim: false),
  _P('Tornado', 'weather · crisp · 3D · spin', _wTornado, init: Offset(.6, .4), bg: _kPanel2),
  _P('Clay Pot', 'material · light · 3D · drag+spin', _wPot, init: Offset(.5, .3), bg: _kCream),
  _P('Melting Poster', 'result · bold · flat · drag', _wMelt, init: Offset(.4, .35), anim: false),
  _P('Squish Face', 'character · bold · flat · pull+let go', _wFace, init: Offset(.62, .4), bg: _kBlue),
  _P('Gravity Well', 'physics · crisp · iso · press', _wWell, init: Offset(.5, .45)),
  _P('Glitch Slices', 'machine · neon! · flat · drag', _wGlitch, init: Offset(.4, .35), bg: _kNight),
  _P('Pebble Pond', 'nature · crisp · top · throw', _wPond, init: Offset(.5, .5), bg: Color(0xFF0E3550)),
  _P('Twisted Tower', 'toy · bold · iso · spin', _wTower, init: Offset(.4, .4)),
  _P('Op-Art Bulge', 'text-art · light · flat · drag+spin', _wOp, init: Offset(.5, .5), bg: _kCream, anim: false),
  _P('Marbled Paint', 'material · bold · flat · paint', _wMarble, init: Offset(.5, .5), bg: _kCream, anim: false),
  _P('Desert Heat Haze', 'landscape · bold · side · drag', _wHaze, init: Offset(.4, .35)),
  _P('Octopus Curl', 'creature · bold · flat · drag', _wOcto, init: Offset(.75, .35), bg: _kBlue),
  _P('Hypno Spiral', 'cosmic · neon! · polar · spin', _wHypno, init: Offset(.4, .4), bg: _kNight),
  _P('Crumpled Poster', 'material · result · 3D · drag', _wCrumple, init: Offset(.5, .35), anim: false),
];

void _wFabric(Canvas c, Size s, _St st, double t) {
  const nx = 12, ny = 9;
  final w = s.width, h = s.height, s0 = st.at0(s), pull = st.at(s) - s0, rad = 52.0, tw = st.spin * .6;
  Offset q(int i, int j) {
    final b = Offset(-6 + i * (w + 12) / nx, -6 + j * (h + 12) / ny);
    final k = _fall((b - s0).distance, rad);
    return s0 + _rot(b - s0, tw * k) + pull * k;
  }

  const a = Color(0xFFFF5DA2), m = Color(0xFFFFB0D0);
  for (var j = 0; j < ny; j++) {
    for (var i = 0; i < nx; i++) {
      final col = i.isEven && j.isEven ? a : (i.isEven || j.isEven ? m : null);
      if (col == null) continue;
      c.drawPath(_poly([q(i, j), q(i + 1, j), q(i + 1, j + 1), q(i, j + 1)]), _f(col));
    }
  }
  c.drawCircle(s0 + pull, 5, _f(_kInk));
  c.drawCircle(s0 + pull, 2, _f(_kYellow));
}

void _wWhirl(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2);
  final rad = (st.at(s) - ctr).distance.clamp(22.0, 76.0), tw = 2.2 + st.spin * .7;
  for (var r = 10.0; r < 90; r += 12) {
    c.drawCircle(ctr, r, _s(_al(_kCyan, .12), 1));
  }
  for (var arm = 0; arm < 6; arm++) {
    final pts = <Offset>[];
    for (var r = 4.0; r < 90; r += 3) {
      final k = _fall(r, rad);
      pts.add(ctr + _dir(arm * math.pi / 3 + tw * k * 1.6 + t * .9 * k) * r);
    }
    c.drawPath(_poly(pts, close: false), _s(arm.isEven ? _kCyan : _kWhite, 2.4));
  }
  for (var i = 0; i < 6; i++) {
    final r = 16 + i * 9.0, k = _fall(r, rad);
    final a = i * 1.7 + t * (.4 + tw * .5) * (k + .05);
    final p = ctr + _dir(a) * r;
    c
      ..save()
      ..translate(p.dx, p.dy)
      ..rotate(a + math.pi / 2);
    c.drawOval(Rect.fromCenter(center: Offset.zero, width: 11, height: 6), _f(_kLime));
    c.restore();
  }
  c.drawCircle(ctr, 6 + 4 * _fall(0, rad) * math.min(1, tw / 4), _f(_kNight));
  c.drawCircle(ctr, rad, _s(_al(_kYellow, .5), 1.2));
}

void _wMirror(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cx = w / 2, mir = Rect.fromLTRB(w * .26, h * .05, w * .74, h * .95);
  c.drawOval(mir, _f(const Color(0xFF16404E)));
  c
    ..save()
    ..clipPath(Path()..addOval(mir));
  final str = st.b * 2.4 - 1.1, rad = 12 + st.a * 50, yc = h * .55;
  for (var y = h * .12; y < h * .9; y += 2) {
    final sc = 1 + str * _fall((y - yc).abs(), rad);
    final segs = <(double, double, Color)>[];
    final hy = h * .27;
    if ((y - hy).abs() < 12) {
      final hw = math.sqrt(144 - (y - hy) * (y - hy));
      segs.add((cx - hw, cx + hw, _kYellow));
    } else if (y >= h * .38 && y < h * .67) {
      segs.add((cx - 13, cx + 13, _kPink));
      if (y < h * .6) segs.add((cx - 20, cx - 15, _kYellow));
      if (y < h * .6) segs.add((cx + 15, cx + 20, _kYellow));
    } else if (y >= h * .67 && y < h * .88) {
      segs.add((cx - 11, cx - 2, _kBlue));
      segs.add((cx + 2, cx + 11, _kBlue));
    }
    for (final (x0, x1, col) in segs) {
      c.drawRect(Rect.fromLTRB(cx + (x0 - cx) * sc, y, cx + (x1 - cx) * sc, y + 2.3), _f(col));
    }
  }
  final hsc = 1 + str * _fall((h * .27 - yc).abs(), rad);
  for (final ex in [-4.0, 4.0]) {
    c.drawCircle(Offset(cx + ex * hsc, h * .25), 1.8, _f(_kInk));
  }
  c.restore();
  c.drawOval(mir, _s(_kYellow, 5));
  for (var k = 0; k < 14; k++) {
    final a = k / 14 * 2 * math.pi;
    c.drawCircle(mir.center + Offset(math.cos(a) * mir.width / 2, math.sin(a) * mir.height / 2), 2.2, _f(_kCream));
  }
}

void _wHole(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ctr = Offset(w / 2, h / 2);
  final rad = 30 + st.b * 55, str = -(.15 + st.a * .8), tw = 1 + st.spin * .8 + t * .15;
  final grid = _s(_al(_kCyan, .4), 1);
  for (var j = 0; j <= 10; j++) {
    final pts = [for (var i = 0; i <= 32; i++) _tw(Offset(i * w / 32, j * h / 10), ctr, rad, str, tw)];
    c.drawPath(_poly(pts, close: false), grid);
  }
  for (var i = 0; i <= 13; i++) {
    final pts = [for (var j = 0; j <= 24; j++) _tw(Offset(i * w / 13, j * h / 24), ctr, rad, str, tw)];
    c.drawPath(_poly(pts, close: false), grid);
  }
  for (var k = 0; k < 40; k++) {
    final p = Offset((_hash(k, 7) * .5 + .5) * w, (_hash(k, 8) * .5 + .5) * h);
    c.drawCircle(_tw(p, ctr, rad, str, tw), 1.3, _f(_kCream));
  }
  final rr = rad * (.12 + .12 * -str);
  c
    ..save()
    ..translate(ctr.dx, ctr.dy)
    ..rotate(-.25);
  c.drawOval(Rect.fromCenter(center: Offset.zero, width: rr * 4.4, height: rr * 1.3), _s(_kOrange, 3.5));
  c.drawOval(Rect.fromCenter(center: Offset.zero, width: rr * 4.4, height: rr * 1.3), _s(_kYellow, 1.2));
  c.restore();
  c.drawCircle(ctr, rr, _f(const Color(0xFF000000)));
  c.drawCircle(ctr, rr, _s(_kViolet, 2));
}

void _wTaffy(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cy = h / 2, cx = w / 2, x0 = w * .1, x1 = w * .9;
  final str = st.b, rad = 18 + st.a * 60, tw = 1.5 + st.spin * .8;
  const cols = [_kPink, _kCream, _kRed];
  final ps = [for (final k in cols) _f(k)];
  for (var x = x0; x < x1; x += 2) {
    final d = (x - cx), k = _fall(d.abs(), rad);
    final half = 18 * (1 - str * .62 * k);
    final ph = x * .06 + tw * 2.2 * (d.abs() >= rad ? d.sign : d / rad * (2 - (d / rad).abs())) + t * .2;
    double u = 0;
    var m = ph.floor();
    while (u < 1) {
      final nu = math.min(1.0, (m + 1 - ph) / 3);
      final yy0 = cy - half * math.cos(u * math.pi), yy1 = cy - half * math.cos(nu * math.pi);
      c.drawRect(Rect.fromLTRB(x, yy0, x + 2.3, yy1), ps[(m % 3 + 3) % 3]);
      u = nu;
      m++;
    }
    c.drawRect(Rect.fromLTRB(x, cy - half * .72, x + 2.3, cy - half * .55), _f(_al(_kWhite, .35)));
  }
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(x0 - 10, cy - 20, x0 + 2, cy + 20), const Radius.circular(4)), _f(_kCyan));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(x1 - 2, cy - 20, x1 + 10, cy + 20), const Radius.circular(4)), _f(_kCyan));
}

void _wPeep(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2), rad = 34 + st.a * 22, str = st.b * 1.5;
  Offset P(Offset q) {
    final v = q - ctr, d = _sat(v.distance / rad);
    return ctr + v * (1 + str * (1 - d * d) * .7) * (1 - str * .25);
  }

  Path warped(Rect r) {
    final pts = <Offset>[];
    for (var k = 0; k <= 6; k++) {
      pts.add(P(Offset.lerp(r.topLeft, r.topRight, k / 6)!));
    }
    for (var k = 0; k <= 6; k++) {
      pts.add(P(Offset.lerp(r.topRight, r.bottomRight, k / 6)!));
    }
    for (var k = 0; k <= 6; k++) {
      pts.add(P(Offset.lerp(r.bottomRight, r.bottomLeft, k / 6)!));
    }
    for (var k = 0; k <= 6; k++) {
      pts.add(P(Offset.lerp(r.bottomLeft, r.topLeft, k / 6)!));
    }
    return _poly(pts);
  }

  c
    ..save()
    ..clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: rad)));
  c.drawCircle(ctr, rad, _f(_kCyan));
  c.drawPath(warped(Rect.fromLTRB(ctr.dx - 80, ctr.dy + 16, ctr.dx + 80, ctr.dy + 80)), _f(_kPanel));
  const cols = [_kPink, _kOrange, _kViolet, _kYellow, _kRed, _kLime];
  for (var i = 0; i < 6; i++) {
    final x = ctr.dx - 66 + i * 23.0, ht = 30 + (_hash(i, 3) * .5 + .5) * 26;
    final r = Rect.fromLTRB(x, ctr.dy + 16 - ht, x + 20, ctr.dy + 16);
    c.drawPath(warped(r), _f(cols[i]));
    for (var wy = r.top + 5; wy < r.bottom - 6; wy += 9) {
      for (final wx in [r.left + 4, r.left + 12]) {
        c.drawPath(warped(Rect.fromLTWH(wx, wy, 4, 5)), _f(_kCream));
      }
    }
  }
  c.drawPath(warped(Rect.fromLTRB(ctr.dx - 80, ctr.dy + 40, ctr.dx + 80, ctr.dy + 43)), _f(_kYellow));
  c.drawCircle(ctr, rad, Paint()..shader = ui.Gradient.radial(ctr, rad, [_al(_kInk, 0), _al(_kInk, 0), _al(_kInk, .7)], [0, .7, 1]));
  c.restore();
  c.drawCircle(ctr, rad + 3, _s(_kYellow, 6));
  c.drawCircle(ctr, rad + 7, _s(const Color(0xFFB98A1E), 2));
}

void _wTornado(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cx = w / 2, str = st.b, topW = 20 + st.a * 50, tw = 2 + st.spin * .6;
  c.drawOval(Rect.fromCenter(center: Offset(cx, h * .9), width: w * .9, height: 14), _f(const Color(0xFF3E6E2E)));
  for (var i = 0; i < 16; i++) {
    final u = i / 15, y = h * .87 - u * h * .76;
    final rw = 5 + topW * u * u + 3 * u;
    final x = cx + str * 16 * math.sin(u * 3.2 - t * 2.4) * u;
    final rr = Rect.fromCenter(center: Offset(x, y), width: rw * 2, height: rw * .5 + 2);
    c.drawOval(rr, _s(_mix(_kCyan, _kWhite, u), 2));
    final a = t * (2 + tw) + i * .5 * tw;
    c.drawCircle(Offset(x + math.cos(a) * rw, y + math.sin(a) * (rw * .25 + 1)), 2.2, _f(i.isEven ? _kOrange : _kYellow));
  }
}

void _wPot(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cx = w / 2, yt = h * .16, yb = h * .8;
  final str = st.b * 1.7 - .55, rad = 8 + st.a * 36, ym = h * .5, spd = .6 + st.spin.abs() * .6;
  c.drawOval(Rect.fromCenter(center: Offset(cx, yb + 6), width: w * .66, height: 16), _f(_kInk));
  c.drawOval(Rect.fromCenter(center: Offset(cx, yb + 3), width: w * .6, height: 12), _f(_kPanel2));
  double rOf(double y) {
    final u = (y - yt) / (yb - yt);
    return 16 + 8 * math.sin(u * math.pi) + 20 * str * _fall((y - ym).abs(), rad);
  }

  final l = <Offset>[], r = <Offset>[];
  for (var y = yt; y <= yb; y += 2) {
    final rr = math.max(4.0, rOf(y));
    l.add(Offset(cx - rr, y));
    r.add(Offset(cx + rr, y));
  }
  c.drawPath(_poly([...l, ...r.reversed]), _f(_kOrange));
  for (var y = yt + 6; y < yb - 2; y += 7) {
    final rr = math.max(4.0, rOf(y));
    c.drawLine(Offset(cx - rr + 1, y), Offset(cx + rr - 1, y), _s(_al(const Color(0xFFB8461A), .6), 1.2));
    final ph = math.sin(t * spd * 3 + y * .4);
    c.drawCircle(Offset(cx + rr * .85 * ph, y), 1.7, _f(_al(_kCream, .8 * (1 - ph.abs() * .5))));
  }
  final rt = math.max(4.0, rOf(yt));
  c.drawOval(Rect.fromCenter(center: Offset(cx, yt), width: rt * 2, height: 7), _f(const Color(0xFFB8461A)));
}

void _wMelt(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ph = h * .62, str = st.b, rad = 4 + st.a * 26;
  final sun = Offset(w * .5, ph * .55);
  for (var x = 0.0; x < w; x += 2.5) {
    final n = _sat(_vn(x / rad, 3.3) * .7 + .5);
    final drip = str * h * .5 * n * n;
    double off(double y) => drip * math.pow(y / ph, 1.6);
    void seg(double y0, double y1, Color col) => c.drawRect(Rect.fromLTRB(x, y0 + off(y0), x + 2.8, y1 + off(y1)), _f(col));
    seg(0, ph * .35, _kPink);
    seg(ph * .35, ph * .6, _kOrange);
    seg(ph * .6, ph * .78, _kYellow);
    final dx = x - sun.dx;
    if (dx.abs() < 18) {
      final hc = math.sqrt(324 - dx * dx);
      seg(sun.dy - hc, sun.dy + hc, _kCream);
    }
    final hy = ph * .72 + 5 * math.sin(x * .06);
    seg(hy, ph, _kViolet);
    c.drawCircle(Offset(x + 1.3, ph + off(ph) - .5), 1.6, _f(_kViolet));
  }
}

void _wFace(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2), s0 = st.at0(s);
  final age = t - st.upAt;
  final spring = st.down ? 1.0 : math.exp(-age * 3.5) * math.cos(age * 16);
  final pull = (st.at(s) - s0) * spring;
  Offset W(Offset q) => q + pull * _fall((q - s0).distance, 46);
  final pts = [for (var k = 0; k < 72; k++) W(ctr + _dir(k / 72 * 2 * math.pi) * 40)];
  c.drawPath(_poly(pts).shift(const Offset(3, 4)), _f(_al(_kInk, .35)));
  c.drawPath(_poly(pts), _f(_kYellow));
  c.drawPath(_poly(pts), _s(_kInk, 2.5));
  for (final e in [const Offset(-13, -8), const Offset(13, -8)]) {
    final p = W(ctr + e);
    c.drawOval(Rect.fromCenter(center: p, width: 7, height: st.down ? 4 : 9), _f(_kInk));
  }
  for (final e in [const Offset(-22, 6), const Offset(22, 6)]) {
    c.drawCircle(W(ctr + e), 5, _f(_al(_kPink, .7)));
  }
  final m = [for (var k = 0; k <= 8; k++) W(ctr + Offset(-11 + k * 2.75, 10 + math.sin(k / 8 * math.pi) * (st.down ? -2 : 6)))];
  c.drawPath(_poly(m, close: false), _s(_kInk, 3));
}

void _wWell(Canvas c, Size s, _St st, double t) {
  const n = 10, tw = 7.2, th = 3.6;
  final ox = s.width / 2, oy = s.height * .14;
  final tp = st.at(s), u = (tp.dx - ox) / tw, v = (tp.dy - oy - 12) / th;
  final wi = ((v + u) / 2).clamp(0.0, n.toDouble()), wj = ((v - u) / 2).clamp(0.0, n.toDouble());
  final str = st.down ? math.min(1.0, .25 + (t - st.downAt) * .7) : .25 + .45 * math.exp(-(t - st.upAt) * 2);
  double z(int i, int j) => str * 34 * _fall(math.sqrt((i - wi) * (i - wi) + (j - wj) * (j - wj)), 4);
  Offset P(int i, int j) => Offset(ox + (i - j) * tw, oy + (i + j) * th + z(i, j));
  for (var k = 0; k < 2 * n - 1; k++) {
    for (var i = 0; i < n; i++) {
      final j = k - i;
      if (j < 0 || j >= n) continue;
      final dz = (z(i, j) + z(i + 1, j + 1)) / 2 / 34;
      final col = _mix((i + j).isEven ? _kCyan : _kBlue, _kNight, dz * .9);
      c.drawPath(_poly([P(i, j), P(i + 1, j), P(i + 1, j + 1), P(i, j + 1)]), _f(col));
    }
  }
  final bp = Offset(ox + (wi - wj) * tw, oy + (wi + wj) * th + str * 34);
  c.drawOval(Rect.fromCenter(center: bp, width: 12, height: 5), _f(_al(_kNight, .5)));
  c.drawCircle(bp - const Offset(0, 6), 6, _f(_kOrange));
  c.drawCircle(bp - const Offset(2, 8), 1.8, _f(_kWhite));
}

void _glitchArt(Canvas c, Size s, Color? tint) {
  final w = s.width, h = s.height;
  Paint p(Color k) => _f(tint ?? k);
  c.drawCircle(Offset(w * .42, h * .5), 30, p(_kPink));
  c.drawPath(_poly([Offset(w * .6, h * .2), Offset(w * .86, h * .78), Offset(w * .36, h * .78)]), p(_kYellow));
  c.drawRect(Rect.fromLTWH(w * .1, h * .62, w * .5, 9), p(_kCyan));
  c.drawCircle(Offset(w * .42, h * .5), 9, p(_kNight));
}

void _wGlitch(Canvas c, Size s, _St st, double t) {
  final sh = 3 + st.a * 16, str = st.b, frame = (t * 7).floor();
  var sl = 0;
  for (var y = 0.0; y < s.height; y += sh, sl++) {
    final v = _hash(sl, frame);
    final dx = (v.abs() > .55 ? v * 40 : v * 6) * str;
    c
      ..save()
      ..clipRect(Rect.fromLTWH(0, y, s.width, sh))
      ..translate(dx, 0);
    c.translate(-5 * str - 1.5, 0);
    _glitchArt(c, s, _kRed);
    c.translate(10 * str + 3, 0);
    _glitchArt(c, s, _kCyan);
    c.translate(-5 * str - 1.5, 0);
    _glitchArt(c, s, null);
    c.restore();
  }
}

const _pebCols = [_kOrange, _kPink, _kYellow, _kLime, _kCream, _kViolet];
void _wPond(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height;
  final touched = st.downAt > -50;
  final ctr = touched ? st.at0(s) : Offset(w * .5, h * .5);
  final age = touched ? t - st.downAt : t % 3.5;
  final str = touched ? .5 + math.min(1.0, (st.p - st.s0).distance * 4) : .9;
  final rr = age * 48;
  for (var k = 0; k < 26; k++) {
    final p = Offset((_hash(k, 1) * .5 + .5) * w, (_hash(k, 2) * .5 + .5) * h);
    final v = p - ctr, d = v.distance, dd = d - rr;
    final disp = str * 9 * math.sin(dd * .28) * math.exp(-dd * dd / 300) * math.exp(-age * .5);
    final q = d < .1 ? p : p + v / d * disp;
    final r = 4 + (_hash(k, 3) * .5 + .5) * 5;
    c.drawOval(Rect.fromCenter(center: q, width: r * 2.2 * (1 + disp * .03), height: r * 1.6), _f(_pebCols[k % 6]));
  }
  final lily = Path()
    ..addArc(Rect.fromCircle(center: Offset(w * .2, h * .25), radius: 13), .5, 2 * math.pi - .8)
    ..lineTo(w * .2, h * .25)
    ..close();
  c.drawPath(lily, _f(const Color(0xFF4FB548)));
  for (var i = 0; i < 3; i++) {
    final r = rr - i * 12;
    if (r <= 0) continue;
    c.drawCircle(ctr, r, _s(_al(_kWhite, (1 - age / 3) * .8), 2 - i * .5));
  }
}

const _towerCols = [_kYellow, _kOrange, _kPink, _kViolet, _kBlue, _kCyan];
void _wTower(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cx = w / 2, base = h * .84;
  final tw = 1.2 + st.spin * .6, str = st.b, rad = 3 + st.a * 9;
  for (var i = 0; i < 10; i++) {
    final u = i / 9, k = _fall((9 - i).toDouble(), rad + .01);
    final r = 19 * (1 - str * .5 * u) * (1 + .15 * math.sin(t * 1.4 + i * .5) * str);
    final th = tw * u * (.4 + .6 * k) + t * .15;
    final y = base - i * 7.5;
    final pts = <Offset>[];
    for (final q in const [Offset(-1, -1), Offset(1, -1), Offset(1, 1), Offset(-1, 1)]) {
      final v = _rot(q * r, th);
      pts.add(Offset(cx + (v.dx - v.dy) * .95, y + (v.dx + v.dy) * .48));
    }
    final col = _towerCols[i % 6];
    c.drawPath(_poly(pts).shift(const Offset(0, 6)), _f(_mix(col, _kInk, .45)));
    c.drawPath(_poly(pts), _f(col));
  }
}

void _wOp(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ctr = st.at(s), tw = st.spin * .9;
  const bands = 16;
  Offset W(Offset q) {
    final v = q - ctr, k = _fall(v.distance, 58);
    return ctr + _rot(v, tw * k) * (1 + .9 * k);
  }

  final ink = _f(_kInk);
  for (var b = 0; b < bands; b += 2) {
    final y0 = -8 + b * (h + 16) / bands, y1 = -8 + (b + 1) * (h + 16) / bands;
    final top = [for (var x = -10.0; x <= w + 10; x += 4) W(Offset(x, y0))];
    final bot = [for (var x = w + 10; x >= -10; x -= 4) W(Offset(x, y1))];
    c.drawPath(_poly([...top, ...bot]), ink);
  }
}

const _marbleCols = [_kBlue, _kCream, _kPink, _kYellow, _kCyan, _kCream, _kOrange, _kViolet, _kCream, _kLime];
void _wMarble(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height;
  final trail = st.trail.isNotEmpty
      ? st.trail
      : [
          for (var i = 0; i < 26; i++)
            (Offset(.15 + i * .028, .5 + .3 * math.sin(i * .35)), Offset(.028, .3 * .35 * math.cos(i * .35))),
        ];
  Offset W(Offset q) {
    var p = q;
    for (final (pos, d) in trail) {
      final pp = Offset(pos.dx * w, pos.dy * h), dd = Offset(d.dx * w, d.dy * h);
      final r2 = (p - pp).distanceSquared;
      p += dd * 1.6 * math.exp(-r2 / 260);
    }
    return p;
  }

  const nb = 10;
  List<Offset> line(int k) => [for (var y = -10.0; y <= h + 10; y += 5) W(Offset(-10 + k * (w + 20) / nb, y))];
  var prev = line(0);
  for (var k = 0; k < nb; k++) {
    final next = line(k + 1);
    c.drawPath(_poly([...prev, ...next.reversed]), _f(_marbleCols[k]));
    prev = next;
  }
}

void _hazeScene(Canvas c, Size s) {
  final w = s.width, h = s.height, hz = h * .5;
  c.drawRect(Rect.fromLTWH(0, 0, w, hz * .55), _f(_kOrange));
  c.drawRect(Rect.fromLTWH(0, hz * .55, w, hz * .45), _f(_kYellow));
  c.drawCircle(Offset(w * .7, hz * .6), 15, _f(_kCream));
  c.drawRect(Rect.fromLTWH(0, hz, w, h - hz), _f(_kPink));
  c.drawRect(Rect.fromLTWH(0, hz + 14, w, h), _f(const Color(0xFFE0447F)));
  c.drawPath(_poly([Offset(w * .47, hz), Offset(w * .53, hz), Offset(w * .8, h), Offset(w * .2, h)]), _f(_kPanel2));
  for (var k = 0; k < 4; k++) {
    final y0 = hz + 4 + k * k * 4.0, y1 = y0 + 2 + k * 2;
    c.drawRect(Rect.fromLTRB(w * .5 - .6 - k * .6, y0, w * .5 + .6 + k * .6, y1), _f(_kYellow));
  }
  for (final (x, sc) in [(w * .16, 1.0), (w * .85, .7)]) {
    final g = _s(const Color(0xFF3F9E3A), 5 * sc);
    final b = hz + 22 * sc;
    c.drawLine(Offset(x, b), Offset(x, b - 26 * sc), g);
    c.drawPath(_poly([Offset(x, b - 12 * sc), Offset(x - 8 * sc, b - 12 * sc), Offset(x - 8 * sc, b - 20 * sc)], close: false), g);
    c.drawPath(_poly([Offset(x, b - 16 * sc), Offset(x + 8 * sc, b - 16 * sc), Offset(x + 8 * sc, b - 24 * sc)], close: false), g);
  }
}

void _wHaze(Canvas c, Size s, _St st, double t) {
  final hz = s.height * .5, rad = 6 + st.a * 34, str = st.b;
  _hazeScene(c, s);
  for (var y = hz - rad; y < hz + rad; y += 2) {
    final dx = str * 7 * math.sin(y * .7 + t * 7) * _fall((y - hz).abs(), rad);
    c
      ..save()
      ..clipRect(Rect.fromLTWH(0, y, s.width, 2))
      ..translate(dx, 0);
    _hazeScene(c, s);
    c.restore();
  }
}

void _wOcto(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cx = w / 2, hy = h * .3;
  final reach = .5 + st.b, curl = (st.a * 2 - 1) * 3.2 + st.spin * .5;
  for (var k = 0; k < 4; k++) {
    final ph = (t * .25 + k * .27) % 1;
    c.drawCircle(Offset(w * (.15 + k * .22) + 3 * math.sin(t * 2 + k), h - ph * h), 2 + k % 2, _s(_al(_kCyan, .7), 1.2));
  }
  for (var j = 0; j < 6; j++) {
    final sgn = j < 3 ? -1.0 : 1.0;
    var p = Offset(cx - 16 + j * 6.4, hy + 14);
    var a = math.pi / 2 + (j - 2.5) * .32;
    for (var i = 0; i < 14; i++) {
      final u = i / 14;
      a += sgn * curl * .05 * u * 2 + .06 * math.sin(t * 2.2 + j + i * .5);
      final q = p + _dir(a) * (2.6 * reach + 1);
      c.drawLine(p, q, _s(_kPink, 7 * (1 - u) + 1.5));
      p = q;
    }
  }
  c.drawOval(Rect.fromCenter(center: Offset(cx, hy), width: 50, height: 42), _f(_kPink));
  for (final ex in [-9.0, 9.0]) {
    c.drawCircle(Offset(cx + ex, hy + 2), 6, _f(_kWhite));
    c.drawCircle(Offset(cx + ex + 1, hy + 3), 3, _f(_kInk));
  }
  c.drawCircle(Offset(cx - 14, hy - 10), 3, _f(_al(_kWhite, .5)));
}

void _wHypno(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2), tw = 1 + st.spin * .8 + st.a * 5;
  final rad = 50 + 35 * st.b * (.5 + .5 * math.sin(t * 3));
  const arms = 10;
  for (var k = 0; k < arms; k++) {
    final a0 = k * 2 * math.pi / arms + t * .9, a1 = (k + 1) * 2 * math.pi / arms + t * .9;
    final pts = <Offset>[];
    for (var i = 0; i <= 20; i++) {
      final r = i / 20 * rad;
      pts.add(ctr + _dir(a0 + tw * r / rad) * r);
    }
    for (var i = 20; i >= 0; i--) {
      final r = i / 20 * rad;
      pts.add(ctr + _dir(a1 + tw * r / rad) * r);
    }
    c.drawPath(_poly(pts), _f(k.isEven ? _kPink : _kYellow));
  }
  c.drawCircle(ctr, 5, _f(_kCyan));
}

Color _posterAt(Offset q) {
  if ((q - const Offset(.62, .38)).distance < .15) return _kYellow;
  if (q.dy > .62 - .12 * math.sin(q.dx * 6)) return q.dy > .8 ? _kOrange : _kViolet;
  return q.dy < .3 ? _kPink : const Color(0xFFFF8FC0);
}

void _wCrumple(Canvas c, Size s, _St st, double t) {
  const nx = 9, ny = 7;
  final w = s.width, h = s.height, rect = Rect.fromLTWH(w * .1, h * .1, w * .8, h * .8);
  final str = st.b, tw = st.spin * .3 + (st.a - .5) * 1.2;
  Offset V(int i, int j) {
    final b = Offset(rect.left + i * rect.width / nx, rect.top + j * rect.height / ny);
    final edge = (i == 0 || j == 0 || i == nx || j == ny) ? .5 : 1.0;
    final q = b + Offset(_hash(i, j * 3), _hash(j + 9, i)) * 8 * str * edge;
    return _tw(q, rect.center, 70, -.15 * str, tw);
  }

  c.drawRect(rect.shift(const Offset(3, 4)), _f(_al(_kNight, .6)));
  for (var j = 0; j < ny; j++) {
    for (var i = 0; i < nx; i++) {
      for (final up in [true, false]) {
        final tri = up ? [V(i, j), V(i + 1, j), V(i, j + 1)] : [V(i + 1, j), V(i + 1, j + 1), V(i, j + 1)];
        final cen = Offset((i + (up ? .33 : .66)) / nx, (j + (up ? .33 : .66)) / ny);
        final shade = _hash(i * 2 + (up ? 0 : 1), j) * str;
        final col = shade > 0 ? _mix(_posterAt(cen), _kWhite, shade * .35) : _mix(_posterAt(cen), _kInk, -shade * .45);
        c.drawPath(_poly(tri), _f(col)..isAntiAlias = false);
      }
    }
  }
}
