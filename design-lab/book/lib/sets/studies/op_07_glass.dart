// Glass / Refraction x20. Why you touch it: "make what is behind bend, magnify, smear or split like real glass".
// Encoders: INDEX blue (how much it bends), ROUGH green (frost), THICK white (how far it shifts), DISP red (colour split).
part of 'op_07.dart';

final _glassSpecs = <_O7Spec>[
  _O7Spec('Loupe on grid', 'diagram · effect · drag lens + pinch · drawing', _loupeGrid, p: const Offset(.42, .48)),
  _O7Spec('Broken straw', 'object · effect · drag ↕ · drawing', _straw, b: .45),
  _O7Spec('Prism fan', 'diagram · mechanism · drag ↕↔ · diagram', _prism, a: .5, b: .5),
  _O7Spec('Fishbowl', 'animal · effect · drag ↕ · drawing', _fishbowl, b: .6),
  _O7Spec('Shower door', 'character · effect · rub · drawing', _showerDoor),
  _O7Spec('Iso glass block', 'isometric · mechanism · drag ↕↔ · object', _isoBlock, a: .55, b: .5),
  _O7Spec('Pool tiles', 'landscape · effect · drag ↕ · drawing', _pool, b: .45),
  _O7Spec('Spectacles', 'character · effect · drag ↕ · numeral', _spectacles, b: .7),
  _O7Spec('Telescope moon', 'instrument · effect · drag ↔ focus ↕ zoom', _telescope, a: .4, b: .4),
  _O7Spec('Rain on glass', 'landscape · effect · drag ↕ · drawing', _rainWindow, b: .4),
  _O7Spec('Snell protractor', 'diagram · mechanism · drag ray + pinch · numeral', _snell, p: const Offset(.22, .14)),
  _O7Spec('Heat mirage', 'vehicle · effect · drag ↕ · drawing', _heatHaze, b: .55),
  _O7Spec('Diamond fire', 'object · effect · spin · pushed', _diamond, b: .7),
  _O7Spec('Fluted pane', 'M4L strips · mechanism · drag ↔ count ↕ depth', _fluted, a: .35, b: .5),
  _O7Spec('Drop on a word', 'typographic · effect · drag drop + pinch · word', _dropWord, p: const Offset(.4, .42)),
  _O7Spec('Soap film', 'film · effect · drag ↕ · colour', _soapFilm, b: .45),
  _O7Spec('Door peephole', 'character · effect · drag ↕ · drawing', _peephole, b: .55),
  _O7Spec('Laser slab', 'diagram · mechanism · pinch + drag ↕↔ · diagram', _laserSlab, a: .3, b: .5),
  _O7Spec('Einstein ring', 'cosmic · effect · drag source · extreme', _einsteinRing, p: const Offset(.58, .44)),
  _O7Spec('Crystal ball', 'character · effect · drag ↔ · pushed', _crystalBall, a: .5, b: .5),
];

double _pinchK(O7 st) => _cl((st.pinch - .4) / 1.2);
String _ior(double k) => (1 + k * .9).toStringAsFixed(2);

void _loupeGrid(Canvas c, Size s, O7 st, double t) {
  final lc = Offset(st.p.dx * s.width, st.p.dy * s.height);
  const r = 30.0;
  final k = .2 + _pinchK(st) * 1.0;
  Offset f(Offset q) => _lens(q, lc, r, k);
  void grid(Paint p) {
    for (var x = 6.0; x < s.width; x += 12) {
      _pl(c, _seg(Offset(x, 0), Offset(x, s.height), 24, f), p);
    }
    for (var y = 6.0; y < s.height; y += 12) {
      _pl(c, _seg(Offset(0, y), Offset(s.width, y), 30, f), p);
    }
  }

  grid(_s(_dg, 1));
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: lc, radius: r)));
  c.drawRect(Offset.zero & s, _f(_bg));
  grid(_s(_bu, 1));
  c.restore();
  c.drawCircle(lc, r, _s(_wt, 1.2));
  c.drawArc(Rect.fromCircle(center: lc, radius: r - 4), math.pi * 1.1, .7, false, _s(_o(_wt, .6), 1));
  final hd = lc + Offset(math.cos(.8), math.sin(.8)) * r;
  c.drawLine(hd, hd + const Offset(14, 14), _s(_wt, 1.2));
  _num(c, _ior(_pinchK(st)), const Offset(150, 100), 12, _bu, ax: 1);
  _lab(c, 'INDEX', const Offset(150, 90), _bu, ax: 1, size: 6.5);
}

void _straw(Canvas c, Size s, O7 st, double t) {
  final k = st.b;
  const water = 54.0;
  final ln = _s(_wt, 1.1);
  final glass = Path()
    ..moveTo(36, 18)
    ..lineTo(46, 110)
    ..lineTo(98, 110)
    ..lineTo(108, 18);
  for (var y = water + 8; y < 108; y += 7) {
    final dx = (y - 18) / 92 * 10;
    _dash(c, Offset(36 + dx + 3, y), Offset(108 - dx - 3, y), _s(_o(_bu, .35), 1), on: 2, off: 4);
  }
  const top = Offset(128, 6), hit = Offset(82, water);
  final u = (hit - top) / (hit - top).distance;
  final n = Offset(-u.dy, u.dx) * 2.5;
  final ang0 = math.atan2(u.dy, u.dx);
  final ang = ang0 + (math.pi / 2 - ang0) * k * .55;
  final kink = hit + Offset(-k * 16, 0);
  final v = Offset(math.cos(ang), math.sin(ang));
  final end = kink + v * ((104 - water) / v.dy);
  final n2 = Offset(-v.dy, v.dx) * 2.5;
  final sp = _s(_rd, 1.1);
  c.drawLine(top + n, hit + n, sp);
  c.drawLine(top - n, hit - n, sp);
  c.drawLine(kink + n2, end + n2, sp);
  c.drawLine(kink - n2, end - n2, sp);
  c.drawLine(end + n2, end - n2, sp);
  _dash(c, hit, hit + u * 56, _s(_o(_rd, .3), 1), on: 1, off: 3);
  final wave = [for (var x = 40.0; x <= 104; x += 4) Offset(x, water + math.sin(x * .3 + t * 3) * 1)];
  _pl(c, wave, _s(_bu, 1.2));
  c.drawPath(glass, ln);
  for (var i = 0; i < 4; i++) {
    final y = 106 - ((t * 14 + i * 13) % 50);
    c.drawCircle(Offset(58 + i * 9 + math.sin(t * 3 + i) * 2, y), 1.6, _s(_o(_wt, .7), 1));
  }
  _num(c, _ior(k), const Offset(8, 8), 14, _bu);
  _lab(c, 'INDEX', const Offset(8, 26), _bu);
}

void _prism(Canvas c, Size s, O7 st, double t) {
  const a = Offset(70, 14), bl = Offset(32, 92), br = Offset(108, 92);
  final tri = Path()
    ..moveTo(a.dx, a.dy)
    ..lineTo(bl.dx, bl.dy)
    ..lineTo(br.dx, br.dy)
    ..close();
  final hit = Offset.lerp(a, bl, .55)!;
  const src = Offset(0, 74);
  final angIn = math.atan2(hit.dy - src.dy, hit.dx - src.dx);
  final angMid = angIn - .25 + st.b * .3;
  var e = hit;
  final dv = Offset(math.cos(angMid), math.sin(angMid));
  for (var i = 0; i < 200; i++) {
    e += dv * .5;
    if (e.dx > a.dx + (e.dy - a.dy) * (br.dx - a.dx) / (br.dy - a.dy)) break;
  }
  c.drawPath(tri, _s(_wt, 1.2));
  c.drawLine(src, hit, _s(_wt, 1.3));
  c.drawLine(hit, e, _s(_o(_wt, .8), 1.2));
  final disp = .03 + st.a * .14;
  const cols = [_rd, _gn, _bu];
  for (var i = 0; i < 3; i++) {
    final an = angMid + .3 + st.b * .35 + i * disp;
    final end = e + Offset(math.cos(an), math.sin(an)) * 90;
    c.drawLine(e, end, _s(cols[i], 1.2));
    final u = (t * .6 + i * .2) % 1;
    _dot(c, Offset.lerp(e, end, u)!, cols[i], 1.5);
  }
  final u = (t * .6) % 1;
  _dot(c, Offset.lerp(src, hit, u)!, _wt, 1.5);
  _num(c, _ior(st.b), const Offset(8, 8), 12, _bu);
  _lab(c, 'INDEX', const Offset(8, 24), _bu, size: 6.5);
  _num(c, _pct(st.a), const Offset(150, 100), 12, _rd, ax: 1);
  _lab(c, 'DISP', const Offset(116, 90), _rd, size: 6.5);
}

void _fishbowl(Canvas c, Size s, O7 st, double t) {
  const ctr = Offset(78, 60);
  const r = 44.0;
  final k = st.b;
  c.drawArc(Rect.fromCircle(center: ctr, radius: r), -math.pi * .3, math.pi * 1.6, false, _s(_wt, 1.2));
  c.drawLine(ctr + Offset(math.cos(-math.pi * .3), math.sin(-math.pi * .3)) * r,
      ctr + Offset(math.cos(-math.pi * .3), math.sin(-math.pi * .3)) * r + const Offset(4, -4), _s(_wt, 1.2));
  c.drawLine(ctr + Offset(math.cos(-math.pi * .7), math.sin(-math.pi * .7)) * r,
      ctr + Offset(math.cos(-math.pi * .7), math.sin(-math.pi * .7)) * r + const Offset(-4, -4), _s(_wt, 1.2));
  final wl = [for (var x = 42.0; x <= 114; x += 4) Offset(x, 32 + math.sin(x * .2 + t * 2) * 1.2)];
  _pl(c, wl, _s(_bu, 1.1));
  final gravel = [for (var i = 0; i < 26; i++) Offset(50 + _h(i) * 56, 96 + _h(i + 30) * 6)];
  c.drawPoints(ui.PointMode.points, gravel, _s(_o(_wt, .5), 1.6));
  final fx = 78 + math.sin(t * .5) * 28, fy = 64 + math.sin(t * .9) * 10;
  final dir = math.cos(t * .5) >= 0 ? 1.0 : -1.0;
  final dist = ((Offset(fx, fy) - ctr).distance / r).clamp(0.0, 1.0);
  final mag = 1 + k * 1.1 * (1 - dist * dist);
  void fish(Offset o, double m, Paint p) {
    c.drawOval(Rect.fromCenter(center: o, width: 20 * m, height: 11 * m), p);
    _pl(c, [o + Offset(-dir * 10 * m, 0), o + Offset(-dir * 17 * m, -6 * m), o + Offset(-dir * 17 * m, 6 * m), o + Offset(-dir * 10 * m, 0)], p);
    c.drawArc(Rect.fromCenter(center: o + Offset(0, -5 * m), width: 8 * m, height: 6 * m), math.pi, math.pi, false, p);
  }

  fish(Offset(fx, fy), 1, _s(_o(_dg, 1), 1));
  fish(Offset(fx, fy), mag, _s(_gn, 1.1));
  _dot(c, Offset(fx + dir * 5 * mag, fy - 1.5 * mag), _gn, 1.2 * mag);
  for (var i = 0; i < 3; i++) {
    final u = (t * .5 + i / 3) % 1;
    c.drawCircle(Offset(fx + dir * 12 * mag, fy - 4 - u * 24), 1 + u, _s(_o(_wt, 1 - u), 1));
  }
  _num(c, _pct(k), const Offset(150, 6), 14, _wt, ax: 1);
  _lab(c, 'THICK', const Offset(6, 6), _wt);
}

void _showerDoor(Canvas c, Size s, O7 st, double t) {
  final rough = 1 - st.rub;
  final fig = <List<Offset>>[
    [for (var i = 0; i <= 20; i++) Offset(78 + math.cos(i / 20 * math.pi * 2) * 10, 40 + math.sin(i / 20 * math.pi * 2) * 11)],
    [const Offset(52, 118), const Offset(56, 70), const Offset(66, 58), const Offset(90, 58), const Offset(100, 70), const Offset(104, 118)],
    [const Offset(56, 70), const Offset(60, 40), const Offset(64, 26)],
  ];
  final breathe = math.sin(t * 1.3) * 1.5;
  for (var copy = 0; copy < 3; copy++) {
    final p = _s(_o(_wt, copy == 0 ? 1 - rough * .5 : rough * .35), 1.1);
    for (var j = 0; j < fig.length; j++) {
      final pts = <Offset>[];
      for (var i = 0; i < fig[j].length; i++) {
        final sd = (j * 50 + i * 7 + copy * 131);
        final shift = Offset(_h(copy + 70) - .5, _h(copy + 80) - .5) * 10 * rough;
        pts.add(fig[j][i] + shift + Offset((_h(sd) - .5) * 5 * rough, (_h(sd + 3) - .5) * 5 * rough + breathe));
      }
      _pl(c, pts, p);
    }
    if (rough < .05) break;
  }
  final drops = [for (var i = 0; i < 8; i++) Offset(96 + i * 3.0, 20 + ((t * 40 + i * 13) % 30))];
  c.drawPoints(ui.PointMode.points, drops, _s(_o(_bu, .8), 1.2));
  c.drawLine(const Offset(90, 14), const Offset(112, 14), _s(_bu, 1.1));
  final speck = <Offset>[];
  for (var i = 0; i < (rough * 260).round(); i++) {
    speck.add(Offset(30 + _h(i + 500) * 96, 6 + _h(i + 900) * 112));
  }
  c.drawPoints(ui.PointMode.points, speck, _s(_o(_gn, .55), 1));
  final ln = _s(_wt, 1.2);
  c.drawRect(const Rect.fromLTRB(28, 4, 128, 120), ln);
  c.drawLine(const Offset(120, 54), const Offset(120, 70), _s(_wt, 2));
  _num(c, _pct(rough), const Offset(6, 8), 12, _gn);
  _lab(c, 'ROUGH', const Offset(130, 8), _gn, size: 6.5);
}

void _isoBlock(Canvas c, Size s, O7 st, double t) {
  const o = Offset(78, 30), k = .95;
  final hgt = 10 + st.b * 40, n = 1 + st.a * .9;
  final g = _s(_dg, 1);
  for (var u = 0.0; u <= 80; u += 10) {
    c.drawLine(_iso(o, u, 0, 0, k), _iso(o, u, 80, 0, k), g);
    c.drawLine(_iso(o, 0, u, 0, k), _iso(o, 80, u, 0, k), g);
  }
  Offset p(double x, double y, double z) => _iso(o, x, y, z, k);
  final sil = Path()..addPolygon([p(20, 20, hgt), p(60, 20, hgt), p(60, 20, 0), p(60, 60, 0), p(20, 60, 0), p(20, 60, hgt)], true);
  _occ(c, sil, .9);
  final lift = hgt * (1 - 1 / n);
  c.save();
  c.clipPath(sil);
  final rg = _s(_bu, 1);
  for (var u = 0.0; u <= 80; u += 10) {
    c.drawLine(p(u, 0, lift), p(u, 80, lift), rg);
    c.drawLine(p(0, u, lift), p(80, u, lift), rg);
  }
  c.restore();
  final e = _s(_wt, 1.2);
  c.drawPath(Path()..addPolygon([p(20, 20, hgt), p(60, 20, hgt), p(60, 60, hgt), p(20, 60, hgt)], true), e);
  c.drawLine(p(60, 20, hgt), p(60, 20, 0), e);
  c.drawLine(p(60, 60, hgt), p(60, 60, 0), e);
  c.drawLine(p(20, 60, hgt), p(20, 60, 0), e);
  _pl(c, [p(60, 20, 0), p(60, 60, 0), p(20, 60, 0)], e);
  final hd = _s(_o(_wt, .35), 1);
  _dash(c, p(20, 20, 0), p(60, 20, 0), hd);
  _dash(c, p(20, 20, 0), p(20, 60, 0), hd);
  _dash(c, p(20, 20, 0), p(20, 20, hgt), hd);
  _num(c, n.toStringAsFixed(2), const Offset(6, 6), 12, _bu);
  _lab(c, 'INDEX', const Offset(6, 22), _bu, size: 6.5);
  _num(c, _pct(st.b), const Offset(150, 6), 12, _wt, ax: 1);
  _lab(c, 'THICK', const Offset(150, 22), _wt, ax: 1, size: 6.5);
}

void _pool(Canvas c, Size s, O7 st, double t) {
  final amp = .5 + st.b * 6;
  Offset w(Offset q) => q + Offset(math.sin(q.dy * .09 + t * 1.5) * amp, math.cos(q.dx * .08 + t * 1.2) * amp * .7);
  final g = _s(_o(_bu, .8), 1);
  for (var x = 10.0; x < 150; x += 13) {
    _pl(c, _seg(Offset(x, 14), Offset(x, 116), 20, w), g);
  }
  for (var y = 14.0; y < 118; y += 13) {
    _pl(c, _seg(Offset(4, y), Offset(152, y), 26, w), g);
  }
  for (var i = 0; i < 4; i++) {
    final y0 = 26 + i * 24.0;
    _pl(c, [for (var x = 4.0; x <= 152; x += 4) Offset(x, y0 + math.sin(x * .11 + t * 2 + i * 1.7) * amp * 1.6)],
        _s(_o(_gn, .25 + .1 * st.b * 4), 1));
  }
  c.drawRect(const Rect.fromLTRB(4, 8, 152, 116), _s(_wt, 1.2));
  final lad = _s(_wt, 1.1);
  c.drawLine(const Offset(126, 2), const Offset(126, 34), lad);
  c.drawLine(const Offset(138, 2), const Offset(138, 34), lad);
  for (var y = 12.0; y < 34; y += 7) {
    c.drawLine(Offset(126, y), Offset(138, y), lad);
  }
  _num(c, _pct(st.b), const Offset(10, 92), 18, _gn);
  _lab(c, 'ROUGH', const Offset(42, 104), _gn);
}

void _spectacles(Canvas c, Size s, O7 st, double t) {
  final d = (st.b - .5) * 8;
  final m = math.pow(1.35, d / 2).toDouble();
  final ln = _s(_wt, 1.1);
  final face = Path()
    ..moveTo(40, 120)
    ..quadraticBezierTo(34, 20, 78, 14)
    ..quadraticBezierTo(122, 20, 116, 120);
  c.drawPath(face, ln);
  c.drawLine(const Offset(78, 56), const Offset(74, 76), ln);
  c.drawLine(const Offset(74, 76), const Offset(80, 77), ln);
  c.drawArc(const Rect.fromLTRB(66, 84, 92, 96), .2, math.pi - .4, false, ln);
  final blink = (t % 4) < .12 ? .15 : 1.0;
  for (final ex in [62.0, 94.0]) {
    final e = Offset(ex, 54);
    c.drawLine(e + const Offset(-8, -16), e + const Offset(8, -18), ln);
    c.drawOval(Rect.fromCenter(center: e, width: 13 * m, height: 7 * m * blink), _s(_wt, 1));
    if (blink > .5) _dot(c, e, _wt, 1.9 * m);
    final rim = (st.b - .5).abs() * 7;
    c.drawCircle(e, 13, _s(_bu, 1.3));
    if (rim > .8) c.drawCircle(e, 13 - rim, _s(_o(_bu, .4), 1));
  }
  c.drawArc(const Rect.fromLTRB(72, 48, 84, 56), math.pi, math.pi, false, _s(_bu, 1.3));
  c.drawLine(const Offset(49, 52), const Offset(38, 48), _s(_bu, 1.3));
  c.drawLine(const Offset(107, 52), const Offset(118, 48), _s(_bu, 1.3));
  final txt = '${d >= 0 ? '+' : '-'}${d.abs().toStringAsFixed(1)}';
  _num(c, txt, const Offset(150, 96), 16, _rd, ax: 1);
  _lab(c, 'DIOPTER', const Offset(6, 6), _rd);
}

void _telescope(Canvas c, Size s, O7 st, double t) {
  final ln = _s(_wt, 1.1);
  const a = Offset(12, 98), b = Offset(62, 62);
  final u = (b - a) / (b - a).distance, n = Offset(-u.dy, u.dx);
  c.drawLine(a + n * 4, b + n * 6, ln);
  c.drawLine(a - n * 4, b - n * 6, ln);
  c.drawLine(b + n * 6, b - n * 6, ln);
  c.drawLine(a + n * 4, a - n * 4, ln);
  for (final f in [.3, .62]) {
    final q = Offset.lerp(a, b, f)!;
    c.drawLine(q + n * 5, q - n * 5, _s(_gn, 1.1));
  }
  final mid = Offset.lerp(a, b, .45)!;
  c.drawLine(mid, const Offset(20, 120), ln);
  c.drawLine(mid, const Offset(48, 120), ln);
  c.drawLine(mid, const Offset(36, 120), _s(_o(_wt, .5), 1));
  const v = Offset(110, 46);
  const vr = 34.0;
  final blur = ((st.a - .62).abs() * 2.6).clamp(0.0, 1.0);
  final mr = 10 + st.b * 40;
  final mc = v + Offset(math.sin(t * .2) * 6, math.cos(t * .17) * 4);
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: v, radius: vr)));
  for (var k = 0; k < (blur > .05 ? 3 : 1); k++) {
    final o = Offset((_h(k) - .5), (_h(k + 5) - .5)) * blur * 8;
    final p = _s(_o(_wt, k == 0 ? 1 - blur * .5 : .4), 1);
    c.drawCircle(mc + o, mr, p);
    for (var i = 0; i < 6; i++) {
      final cc = mc + o + Offset(_h(i + 20) - .5, _h(i + 40) - .5) * mr * 1.3;
      c.drawCircle(cc, mr * (.06 + _h(i + 60) * .12), _s(_o(_gn, k == 0 ? 1 - blur * .6 : .35), 1));
    }
  }
  c.restore();
  c.drawCircle(v, vr, _s(_wt, 1.2));
  _dash(c, v - const Offset(vr, 0), v + const Offset(vr, 0), _s(_o(_rd, .5), 1), on: 2, off: 3);
  _dash(c, v - const Offset(0, vr), v + const Offset(0, vr), _s(_o(_rd, .5), 1), on: 2, off: 3);
  _num(c, (1 + st.b * 9).toStringAsFixed(1), const Offset(150, 92), 14, _wt, ax: 1);
  _lab(c, 'ZOOM', const Offset(150, 108), _wt, ax: 1, size: 6.5);
  _lab(c, 'FOCUS', const Offset(6, 6), _gn);
  _num(c, _pct(1 - blur), const Offset(6, 18), 10, _gn, w: 1);
}

void _rainWindow(Canvas c, Size s, O7 st, double t) {
  final sky = <Offset>[const Offset(0, 120)];
  var x = 0.0;
  for (var i = 0; x < s.width; i++) {
    final w = 8 + _h(i + 70) * 12, top = 120 - 14 - _h(i + 80) * 44;
    sky
      ..add(Offset(x, top))
      ..add(Offset(x + w, top));
    x += w;
  }
  _pl(c, sky, _s(_o(_pu, .45), 1));
  final n = 6 + (st.b * 34).round();
  for (var i = 0; i < n; i++) {
    final r = 2.2 + _h(i + 3) * 3.4, sp = 3 + _h(i + 9) * 12;
    final dx = 6 + _h(i) * 144, dy = (_h(i + 50) * 130 + t * sp) % 134 - 8;
    final d = Offset(dx, dy);
    _dash(c, d - Offset(0, 6 + sp), d - Offset(0, r + 1), _s(_o(_wt, .2), 1), on: 1, off: 2);
    c.drawCircle(d, r, _f(_bg));
    c.drawCircle(d, r, _s(_o(_wt, .85), 1));
    c.drawArc(Rect.fromCircle(center: d, radius: r * .6), .3, math.pi - .6, false, _s(_bu, 1));
    _dot(c, d + Offset(-r * .4, -r * .4), _wt, .6);
  }
  _num(c, n.toString().padLeft(2, '0'), const Offset(150, 6), 14, _wt, ax: 1);
  _lab(c, 'DROPS', const Offset(6, 6), _wt);
}

void _snell(Canvas c, Size s, O7 st, double t) {
  const o = Offset(78, 62);
  final n = 1 + _pinchK(st) * 1.4;
  final src = Offset(st.p.dx * s.width, math.min(st.p.dy * s.height, 48));
  final dv = o - src;
  final th1 = math.atan2(dv.dx.abs(), dv.dy).clamp(0.0, 1.45);
  final th2 = math.asin(math.sin(th1) / n);
  final sg = dv.dx >= 0 ? 1.0 : -1.0;
  final step = 10 - (n - 1) * 5;
  c.save();
  c.clipRect(Rect.fromLTRB(0, o.dy, s.width, s.height));
  for (var x = -60.0; x < s.width; x += step) {
    c.drawLine(Offset(x, o.dy), Offset(x + 60, s.height), _s(_o(_pu, .35), 1));
  }
  c.restore();
  c.drawLine(Offset(0, o.dy), Offset(s.width, o.dy), _s(_wt, 1.1));
  _dash(c, Offset(o.dx, 4), Offset(o.dx, 116), _s(_o(_wt, .5), 1), on: 2, off: 3);
  c.drawLine(src, o, _s(_rd, 1.3));
  final out = o + Offset(sg * math.sin(th2), math.cos(th2)) * 60;
  c.drawLine(o, out, _s(_bu, 1.3));
  _dash(c, o, o + Offset(sg * math.sin(th1), -math.cos(th1)) * 50, _s(_o(_rd, .35), 1), on: 2, off: 3);
  final u = (t * .7) % 1;
  _dot(c, u < .5 ? Offset.lerp(src, o, u * 2)! : Offset.lerp(o, out, u * 2 - 1)!, _wt, 1.6);
  c.drawCircle(src, 3, _s(_rd, 1.1));
  for (var i = 0; i < 8; i++) {
    final an = i * math.pi / 4 + t * .5;
    c.drawLine(src + Offset(math.cos(an), math.sin(an)) * 5, src + Offset(math.cos(an), math.sin(an)) * 8, _s(_rd, 1));
  }
  c.drawArc(Rect.fromCircle(center: o, radius: 16), -math.pi / 2, -sg * th1, false, _s(_rd, 1));
  c.drawArc(Rect.fromCircle(center: o, radius: 22), math.pi / 2, -sg * th2, false, _s(_bu, 1));
  final deg1 = (th1 * 180 / math.pi).round(), deg2 = (th2 * 180 / math.pi).round();
  _num(c, deg1.toString().padLeft(2, '0'), Offset(sg > 0 ? 150 : 6, 6), 16, _rd, ax: sg > 0 ? 1 : 0);
  _num(c, deg2.toString().padLeft(2, '0'), Offset(sg > 0 ? 6 : 150, 96), 16, _bu, ax: sg > 0 ? 0 : 1);
  _lab(c, 'AIR', Offset(sg > 0 ? 6 : 150, 50), _o(_wt, .7), ax: sg > 0 ? 0 : 1, size: 6.5);
  _lab(c, 'N ${n.toStringAsFixed(2)}', Offset(sg > 0 ? 150 : 6, 66), _pu, ax: sg > 0 ? 1 : 0, size: 6.5);
}

void _heatHaze(Canvas c, Size s, O7 st, double t) {
  final heat = st.b;
  const hz = 50.0;
  final hills = [for (var x = 0.0; x <= s.width; x += 6) Offset(x, hz - 4 - (math.sin(x * .05) + 1) * 5 - _h(x.toInt()) * 2)];
  _pl(c, hills, _s(_o(_pu, .6), 1));
  final ln = _s(_wt, 1.1);
  c.drawLine(const Offset(70, hz), const Offset(8, 120), ln);
  c.drawLine(const Offset(86, hz), const Offset(148, 120), ln);
  for (var i = 0; i < 6; i++) {
    final u = ((t * .35 + i / 6) % 1);
    final z = u * u;
    final y0 = hz + z * 70, y1 = hz + math.min(1, (u + .06) * (u + .06)) * 70;
    c.drawLine(Offset(78, y0), Offset(78, y1), _s(_o(_wt, .3 + .7 * u), 1 + u));
  }
  double wob(double y) => math.sin(y * 1.3 + t * 9) * heat * 1.8;
  List<Offset> car(double sy, double base) => [
        Offset(64, base), Offset(64, base - 6 * sy), Offset(68, base - 6 * sy), Offset(71, base - 12 * sy),
        Offset(85, base - 12 * sy), Offset(88, base - 6 * sy), Offset(92, base - 6 * sy), Offset(92, base),
        Offset(64, base),
      ].map((q) => q + Offset(wob(q.dy), 0)).toList();
  _pl(c, car(1, hz - 1), _s(_bu, 1.1));
  _dot(c, Offset(69 + wob(hz), hz - 1), _bu, 1.8);
  _dot(c, Offset(87 + wob(hz), hz - 1), _bu, 1.8);
  if (heat > .1) {
    _pl(c, car(-1, hz + 1).map((q) => q + Offset(0, (_h(q.dx.toInt()) - .5) * heat * 2)).toList(), _s(_o(_bu, heat * .8), 1));
    for (var i = 0; i < 5; i++) {
      final y = hz + 3 + i * 3.0;
      _pl(c, [for (var x = 40.0; x <= 116; x += 4) Offset(x, y + math.sin(x * .3 + t * 6 + i) * heat * 1.6)],
          _s(_o(_rd, heat * (.5 - i * .08)), 1));
    }
  }
  c.drawCircle(const Offset(130, 18), 7, _s(_rd, 1.1));
  _num(c, (20 + heat * 40).round().toString(), const Offset(6, 6), 16, _rd);
  _lab(c, 'HEAT', const Offset(6, 26), _rd);
}

void _diamond(Canvas c, Size s, O7 st, double t) {
  const cx = 78.0, tableY = 34.0, girdle = 50.0, culet = Offset(78, 100);
  final rot = st.spin * .25 + t * .15;
  final ln = _s(_wt, 1.1);
  final gx = <double>[], tx = <double>[];
  for (var j = 0; j < 8; j++) {
    final ph = j * math.pi / 4 + rot;
    if (math.sin(ph) > -.05) {
      gx.add(cx + math.cos(ph) * 40);
      tx.add(cx + math.cos(ph + math.pi / 8) * 22);
    }
  }
  c.drawLine(const Offset(cx - 22, tableY), const Offset(cx + 22, tableY), ln);
  c.drawLine(const Offset(cx - 22, tableY), const Offset(cx - 40, girdle), ln);
  c.drawLine(const Offset(cx + 22, tableY), const Offset(cx + 40, girdle), ln);
  c.drawLine(const Offset(cx - 40, girdle), const Offset(cx + 40, girdle), ln);
  c.drawLine(const Offset(cx - 40, girdle), culet, ln);
  c.drawLine(const Offset(cx + 40, girdle), culet, ln);
  final fl = _s(_o(_wt, .55), 1);
  for (var i = 0; i < gx.length; i++) {
    c.drawLine(Offset(gx[i], girdle), culet, fl);
    c.drawLine(Offset(gx[i], girdle), Offset(tx[i].clamp(cx - 22, cx + 22), tableY), fl);
  }
  final k = st.b;
  const cols = [_bu, _gn, _rd, _wt, _pu];
  final count = (3 + k * 12).round();
  for (var i = 0; i < count; i++) {
    final ph = (t * (1.5 + _h(i) * 2) + _h(i + 7) * 6 + rot * 2) % (math.pi * 2);
    final g = math.max(0.0, math.sin(ph));
    if (g < .2) continue;
    final q = Offset(cx + (_h(i + 11) - .5) * 70 * (1 - _h(i + 13) * .5), 32 + _h(i + 17) * 56);
    final len = (3 + 9 * g) * (.5 + k);
    final p = _s(_o(cols[i % 5], g), 1);
    c.drawLine(q - Offset(len, 0), q + Offset(len, 0), p);
    c.drawLine(q - Offset(0, len), q + Offset(0, len), p);
  }
  _num(c, (1.3 + k * 1.2).toStringAsFixed(2), const Offset(150, 6), 14, _bu, ax: 1);
  _lab(c, 'IOR', const Offset(150, 24), _bu, ax: 1, size: 6.5);
  _lab(c, 'SPIN', const Offset(6, 108), _o(_wt, .6), size: 6.5);
}

void _fluted(Canvas c, Size s, O7 st, double t) {
  final count = 4 + (st.a * 14).round();
  const x0 = 8.0, x1 = 148.0;
  final w = (x1 - x0) / count, sx = 1 - st.b * .75;
  void sample() {
    c.drawCircle(Offset(78 + math.sin(t * .5) * 10, 58), 30, _s(_wt, 1.3));
    c.drawLine(const Offset(10, 104), const Offset(146, 14), _s(_rd, 1.3));
    c.drawRect(const Rect.fromLTWH(96, 70, 22, 22), _s(_gn, 1.3));
  }

  for (var i = 0; i < count; i++) {
    final r = Rect.fromLTRB(x0 + i * w, 6, x0 + (i + 1) * w, 114);
    c.save();
    c.clipRect(r);
    c.translate(r.center.dx, 0);
    c.scale(sx, 1);
    c.translate(-r.center.dx, 0);
    sample();
    c.restore();
    c.drawLine(r.topLeft, r.bottomLeft, _s(_o(_wt, .45), 1));
    c.drawLine(r.topLeft + Offset(w * .22, 0), r.bottomLeft + Offset(w * .22, 0), _s(_o(_bu, .35), 1));
  }
  c.drawRect(const Rect.fromLTRB(x0, 6, x1, 114), _s(_wt, 1.1));
  _lab(c, '$count FLUTES', const Offset(10, 108), _bu, size: 6.5);
  _lab(c, 'DEPTH ${_pct(st.b)}', const Offset(146, 108), _wt, ax: 1, size: 6.5);
}

void _dropWord(Canvas c, Size s, O7 st, double t) {
  final big = _tp('GLASS', _wt, 34, FontWeight.w200);
  const bo = Offset(10, 26);
  final small = _tp('INDEX 1.33  ROUGH 00', _o(_wt, .5), 6.5, FontWeight.w400);
  const so = Offset(12, 72);
  void words(Color? tint) {
    if (tint == null) {
      big.paint(c, bo);
      small.paint(c, so);
    } else {
      _tp('GLASS', tint, 34, FontWeight.w200).paint(c, bo);
      _tp('INDEX 1.33  ROUGH 00', tint, 6.5, FontWeight.w400).paint(c, so);
    }
    for (var x = 10.0; x < 148; x += 4) {
      c.drawLine(Offset(x, 90), Offset(x, x % 20 < 4 ? 82 : 86), _s(_o(tint ?? _wt, .5), 1));
    }
  }

  words(null);
  final dc = Offset(st.p.dx * s.width, st.p.dy * s.height) + Offset(0, math.sin(t * 2) * .8);
  const r = 18.0;
  final m = 1.3 + _pinchK(st) * 1.4;
  final drop = Path()..addOval(Rect.fromCircle(center: dc, radius: r));
  c.save();
  c.clipPath(drop);
  c.drawRect(Offset.zero & s, _f(_bg));
  c.translate(dc.dx, dc.dy);
  c.scale(m, m);
  c.translate(-dc.dx, -dc.dy);
  words(_bu);
  c.restore();
  c.drawPath(drop, _s(_wt, 1.2));
  c.drawArc(Rect.fromCircle(center: dc, radius: r - 4), math.pi * 1.1, .8, false, _s(_wt, 1));
  c.drawArc(Rect.fromCircle(center: dc + const Offset(1.5, 1.5), radius: r), .2, 1.2, false, _s(_o(_wt, .3), 1));
  _num(c, m.toStringAsFixed(1), const Offset(150, 104), 10, _bu, ax: 1, w: 1);
  _lab(c, 'MAG', const Offset(6, 6), _bu, size: 6.5);
}

void _soapFilm(Canvas c, Size s, O7 st, double t) {
  const ctr = Offset(78, 60);
  const r = 46.0;
  final base = 80 + st.b * 620;
  const pal = [_bu, _gn, _wt, _rd, _pu];
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  for (var y = ctr.dy - r; y < ctr.dy + r; y += 2.6) {
    final f = (y - (ctr.dy - r)) / (2 * r);
    final th = base * (.25 + .75 * f) + 50 * math.sin(y * .08 + t * 1.1);
    if (th < 110) continue;
    final col = pal[(th / 85).floor() % 5];
    _pl(c, [for (var x = ctr.dx - r; x <= ctr.dx + r; x += 6) Offset(x, y + math.sin(x * .07 + t * 1.4 + f * 4) * 2.2)],
        _s(_o(col, .85), 1));
  }
  c.restore();
  c.drawCircle(ctr, r, _s(_wt, 1.2));
  c.drawArc(Rect.fromCircle(center: ctr, radius: r - 6), math.pi * 1.15, .6, false, _s(_wt, 1.4));
  c.drawCircle(ctr + const Offset(-20, -24), 2, _s(_wt, 1));
  _num(c, base.round().toString(), const Offset(150, 96), 14, _wt, ax: 1);
  _lab(c, 'NM', const Offset(150, 86), _wt, ax: 1, size: 6.5);
  _lab(c, 'THICK', const Offset(6, 6), _wt);
}

void _peephole(Canvas c, Size s, O7 st, double t) {
  final ln = _s(_wt, 1.1);
  c.drawRect(const Rect.fromLTRB(6, 6, 48, 120), ln);
  c.drawRect(const Rect.fromLTRB(12, 60, 42, 108), _s(_o(_wt, .4), 1));
  c.drawCircle(const Offset(27, 36), 3, _s(_bu, 1.1));
  c.drawCircle(const Offset(40, 70), 2, ln);
  const v = Offset(104, 60);
  const vr = 46.0;
  final k = .1 + st.b * 3.2;
  Offset fe(Offset q) {
    final d = q.distance;
    if (d < 1e-4) return v;
    final r2 = d * (1 + k) / (1 + k * d);
    return v + q / d * r2 * vr;
  }

  List<Offset> loop(Offset ctr, double rx, double ry, [int n = 24]) =>
      [for (var i = 0; i <= n; i++) fe(ctr + Offset(math.cos(i / n * math.pi * 2) * rx, math.sin(i / n * math.pi * 2) * ry))];
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: v, radius: vr)));
  for (final q in const [Offset(-1, -1), Offset(1, -1), Offset(-1, 1), Offset(1, 1)]) {
    _pl(c, _seg(q * 1.2, q * .5, 10, fe), _s(_o(_pu, .6), 1));
  }
  _pl(c, _seg(const Offset(-.5, .5), const Offset(.5, .5), 12, fe), _s(_o(_pu, .6), 1));
  final lean = math.sin(t * .8) * .05;
  final hc = Offset(lean, -.08);
  final fp = _s(_wt, 1.1);
  _pl(c, loop(hc, .26, .32), fp);
  _pl(c, _seg(hc + const Offset(-.4, -.24), hc + const Offset(.4, -.24), 12, fe), _s(_gn, 1.1));
  _pl(c, loop(hc + const Offset(0, -.36), .22, .14, 16), _s(_gn, 1.1));
  _pl(c, loop(hc + const Offset(-.1, -.06), .035, .035, 8), fp);
  _pl(c, loop(hc + const Offset(.1, -.06), .035, .035, 8), fp);
  _pl(c, _seg(hc + const Offset(-.08, .14), hc + const Offset(.08, .14), 6, fe), fp);
  _pl(c, _seg(hc + const Offset(0, -.02), hc + const Offset(.02, .08), 4, fe), fp);
  _pl(c, [..._seg(const Offset(-.8, 1), hc + const Offset(-.3, .36), 8, fe), ..._seg(hc + const Offset(.3, .36), const Offset(.8, 1), 8, fe)], fp);
  c.restore();
  c.drawCircle(v, vr, _s(_wt, 1.2));
  c.drawCircle(v, vr + 3, _s(_o(_wt, .35), 1));
  _num(c, _pct(st.b), const Offset(12, 10), 12, _bu);
  _lab(c, 'BULGE', const Offset(12, 26), _bu, size: 6.5);
}

void _laserSlab(Canvas c, Size s, O7 st, double t) {
  final w = (24 * st.pinch).clamp(10.0, 70.0);
  final n = 1 + st.b * .9;
  final x0 = 80 - w / 2, x1 = 80 + w / 2;
  c.drawRect(const Rect.fromLTRB(4, 28, 26, 40), _s(_rd, 1.1));
  _lab(c, 'LASER', const Offset(4, 18), _rd, size: 6.5);
  const src = Offset(26, 34);
  final y0 = 52.0;
  final th1 = math.atan2(y0 - src.dy, x0 - src.dx);
  final th2 = math.asin(math.sin(th1) / n);
  final ye = y0 + w * math.tan(th2);
  final beam = _s(_rd, 1.3);
  c.drawLine(src, Offset(x0, y0), beam);
  c.drawLine(Offset(x0, y0), Offset(x1, ye), _s(_o(_rd, .8), 1.1));
  _dash(c, Offset(x1, ye), Offset(x1, ye) + Offset(math.cos(th1), math.sin(th1)) * 66, _s(_o(_wt, .25), 1), on: 1, off: 3);
  final spread = st.a * .7;
  for (var i = -3; i <= 3; i++) {
    final an = th1 + i / 3 * spread;
    final al = i == 0 ? 1.0 : (1 - st.a * .3) * (1 - i.abs() / 4) * math.min(1, st.a * 4);
    if (al < .03) continue;
    c.drawLine(Offset(x1, ye), Offset(x1, ye) + Offset(math.cos(an), math.sin(an)) * 70, _s(_o(i == 0 ? _rd : _gn, al), 1));
  }
  final slab = Rect.fromLTRB(x0, 14, x1, 108);
  c.drawRect(slab, _s(_wt, 1.2));
  for (var y = 18.0; y < 108; y += 6) {
    c.drawLine(Offset(x0 + 2, y), Offset(x0 + 5, y - 3), _s(_o(_wt, .3), 1));
  }
  final speck = [for (var i = 0; i < (st.a * 26).round(); i++) Offset(x1 - 1 - _h(i) * 3, 16 + _h(i + 40) * 90)];
  c.drawPoints(ui.PointMode.points, speck, _s(_gn, 1.2));
  final u = (t * .8) % 1;
  _dot(c, Offset.lerp(src, Offset(x0, y0), u)!, _wt, 1.3);
  _num(c, n.toStringAsFixed(2), const Offset(150, 6), 12, _bu, ax: 1);
  _lab(c, 'INDEX', const Offset(150, 22), _bu, ax: 1, size: 6.5);
  _lab(c, 'THICK ${w.round()}', const Offset(80, 110), _wt, ax: .5, size: 6.5);
  _lab(c, 'ROUGH', const Offset(4, 108), _gn, size: 6.5);
}

void _einsteinRing(Canvas c, Size s, O7 st, double t) {
  final field = [for (var i = 0; i < 46; i++) Offset(_h(i + 300) * s.width, _h(i + 400) * s.height)];
  c.drawPoints(ui.PointMode.points, field, _s(_dg, 1.2));
  const lc = Offset(78, 60);
  const te = 26.0;
  final src = Offset(st.p.dx * s.width, st.p.dy * s.height);
  final beta = src - lc;
  final b = beta.distance;
  final dir = b < .01 ? 0.0 : math.atan2(beta.dy, beta.dx);
  final rp = (b + math.sqrt(b * b + 4 * te * te)) / 2, rm = (math.sqrt(b * b + 4 * te * te) - b) / 2;
  final span = math.min(math.pi * 2, 1.2 * te / (b + 1));
  final pulse = .8 + .2 * math.sin(t * 2);
  for (var i = 0; i < 3; i++) {
    final dr = (i - 1) * 1.6;
    final col = [_bu, _wt, _pu][i];
    c.drawArc(Rect.fromCircle(center: lc, radius: rp + dr), dir - span / 2, span, false, _s(_o(col, pulse), 1.1));
    c.drawArc(Rect.fromCircle(center: lc, radius: rm + dr), dir + math.pi - span * rm / rp / 2, span * rm / rp, false,
        _s(_o(col, pulse * .8), 1));
  }
  c.drawOval(Rect.fromCenter(center: lc, width: 12, height: 6), _s(_rd, 1.1));
  _dot(c, lc, _rd, 1.6);
  final sp = _s(_o(_wt, .35), 1);
  for (var i = 0; i < 3; i++) {
    final an = t * .6 + i * math.pi * 2 / 3;
    c.drawArc(Rect.fromCircle(center: src, radius: 5), an, 1.4, false, sp);
  }
  _dash(c, src, lc, _s(_o(_wt, .15), 1), on: 1, off: 3);
  _num(c, _pct(1 - _cl(b / 60)), const Offset(150, 6), 14, _bu, ax: 1);
  _lab(c, 'ALIGN', const Offset(150, 24), _bu, ax: 1, size: 6.5);
  _lab(c, 'MASS', const Offset(6, 108), _rd, size: 6.5);
}

void _crystalBall(Canvas c, Size s, O7 st, double t) {
  const bc = Offset(78, 50);
  const br = 32.0;
  final sx = (st.a - .5) * 70;
  final m = .5 + st.b * .5;
  void scene(double a, Color c1, Color c2) {
    final o = Offset(78 + sx, 0);
    final p1 = _s(_o(c1, a), 1.1), p2 = _s(_o(c2, a), 1.1);
    c.drawRect(Rect.fromLTRB(o.dx - 24, 62, o.dx - 4, 80), p1);
    _pl(c, [Offset(o.dx - 27, 63), Offset(o.dx - 14, 50), Offset(o.dx - 1, 63)], p1);
    c.drawRect(Rect.fromLTRB(o.dx - 17, 70, o.dx - 11, 80), p1);
    c.drawLine(Offset(o.dx + 14, 80), Offset(o.dx + 14, 64), p2);
    c.drawCircle(Offset(o.dx + 14, 58), 8, p2);
    c.drawCircle(Offset(o.dx + 30, 26), 5, _s(_o(_wt, a), 1));
    c.drawLine(Offset(o.dx - 50, 80), Offset(o.dx + 50, 80), _s(_o(_wt, a * .6), 1));
  }

  scene(.35, _pu, _gn);
  final ball = Path()..addOval(Rect.fromCircle(center: bc, radius: br));
  _occ(c, ball, .92);
  c.save();
  c.clipPath(ball);
  c.translate(bc.dx, bc.dy);
  c.scale(-m, -m);
  c.translate(-bc.dx, -bc.dy - 18);
  scene(1, _pu, _gn);
  c.restore();
  for (var i = 0; i < 10; i++) {
    final an = t * .4 + i * .63, rr = 10 + _h(i) * 18;
    _dot(c, bc + Offset(math.cos(an), math.sin(an) * .6) * rr, _o(_wt, .35), .8);
  }
  c.drawPath(ball, _s(_wt, 1.2));
  c.drawArc(Rect.fromCircle(center: bc, radius: br - 5), math.pi * 1.1, .7, false, _s(_wt, 1));
  final ln = _s(_wt, 1.1);
  _pl(c, const [Offset(62, 80), Offset(56, 96), Offset(100, 96), Offset(94, 80)], ln);
  c.drawLine(const Offset(48, 100), const Offset(108, 100), ln);
  for (final sd in [-1.0, 1.0]) {
    final hx = 78 + sd * 40;
    final hand = Path()
      ..moveTo(hx + sd * 16, 118)
      ..quadraticBezierTo(hx + sd * 6, 92, hx, 72)
      ..quadraticBezierTo(hx - sd * 2, 56, hx + sd * 2, 44);
    c.drawPath(hand, _s(_rd, 1.1));
    for (var f = 0; f < 4; f++) {
      final fy = 46.0 + f * 7;
      c.drawLine(Offset(hx + sd * 2, fy), Offset(hx - sd * 5, fy - 3 + math.sin(t * 2 + f) * .8), _s(_rd, 1.1));
    }
  }
  _num(c, _pct(st.b), const Offset(6, 6), 12, _bu);
  _lab(c, 'INDEX', const Offset(6, 22), _bu, size: 6.5);
}
