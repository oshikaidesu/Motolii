part of 'op_06.dart';

// Mask: a = softness (blue), b = expand (green), inv = invert (red-magenta, tap). The subject being cut is white.
const op06MaskPanels = <OpP>[
  OpP('contour rings', 'diagram · mechanism · drag · numeral-led · sane', OpG.drag, _m1),
  OpP('sheep fleece', 'animal · effect · rub · drawing-led · sane', OpG.rub, _m2, a: .4),
  OpP('venetian blind', 'house machine · effect · drag · drawing-led', OpG.drag, _m3),
  OpP('curtain call', 'stage · effect · flick · drawing-led', OpG.flick, _m4, a: .35),
  OpP('iso mesa', 'isometric object · mechanism · drag · diagram-led', OpG.drag, _m5),
  OpP('falloff envelope', 'diagram · mechanism · drag · numeral-led · sane', OpG.drag, _m6, a: .35, b: .4),
  OpP('stipple dust', 'texture · effect · pinch · drawing-led · pushed', OpG.pinch, _m7, a: .5, b: .5),
  OpP('eclipse terminator', 'cosmic · effect · spin · drawing-led', OpG.spin, _m8, wrap: true, a: .3),
  OpP('island sea level', 'landscape · effect · drag · drawing-led', OpG.drag, _m9, b: .45),
  OpP('porthole', 'vehicle · effect · spin · drawing-led', OpG.spin, _m10, a: .4),
  OpP('MASK window', 'typographic · effect · drag · type-led · pushed', OpG.drag, _m11, a: .45, b: .4),
  OpP('ghost cat', 'character · invert · flick · drawing-led', OpG.flick, _m12),
  OpP('iris blades', 'machine · mechanism · spin · diagram-led', OpG.spin, _m13, a: .45),
  OpP('stencil spray', 'machine · effect · rub · drawing-led', OpG.rub, _m14, a: .3),
  OpP('lasso offsets', 'diagram · mechanism · draw · drawing-led', OpG.draw, _m15),
  OpP('signal flow', 'diagram · mechanism · drag · diagram-led · sane', OpG.drag, _m16),
  OpP('hedgehog spines', 'animal · effect · pinch · drawing-led · 振り切れ', OpG.pinch, _m17, a: .55, b: .5),
  OpP('scanline ridge', 'M4L field · effect · drag · diagram-led', OpG.drag, _m18),
  OpP('dot matrix', 'M4L grid · mechanism · draw · diagram-led', OpG.draw, _m19),
  OpP('lighthouse', 'landscape · effect · spin · drawing-led · 振り切れ', OpG.spin, _m20, wrap: true, a: .1),
];

// 1. Nested contours around a star; the big numeral is the softness.
void _m1(Canvas c, Size z, OpS s) {
  const o = Offset(46, 62);
  final r = 12 + s.b * 16, rot = -math.pi / 2 + s.t * .15;
  final n = 1 + (s.a * 7).round();
  for (var i = n; i >= 1; i--) {
    final k = (s.inv ? -1.6 : 3.0) * i;
    _path(c, _star(o, r + k, (r + k) * .48, 5, rot), _al(_bl, 1 - i / (n + 1.5)), .9);
  }
  _path(c, _star(o, r, r * .48, 5, rot), _wh, 1.3);
  _t(c, 'FTHR', 100, 14, _bl);
  _num(c, _n(s.a), 98, 23, _bl, 34);
  _t(c, 'EXP', 100, 72, _gr);
  _num(c, _n(s.b), 98, 81, _gr, 22);
  _t(c, s.inv ? 'INV' : '', 132, 72, _rd);
}

// 2. A sheep whose fleece edge is the soft edge; rubbing fluffs it out.
void _m2(Canvas c, Size z, OpS s) {
  const o = Offset(82, 60);
  final r = 20 + s.b * 8, br = 1 + math.sin(s.t * 2) * .015;
  final layers = 1 + (s.a * 4).round();
  for (var l = layers - 1; l >= 0; l--) {
    final rr = r + l * (2 + s.a * 3);
    final pts = <Offset>[];
    for (var i = 0; i <= 96; i++) {
      final a = i / 96 * math.pi * 2;
      final bump = (1 + s.a * 2.4 + s.e * 2) * (math.sin(a * 14 + l).abs());
      pts.add(o + Offset(math.cos(a) * (rr + bump) * 1.3, math.sin(a) * (rr + bump) * .8 * br));
    }
    _path(c, _poly(pts), l == 0 ? _wh : _al(_bl, .8 - l * .15), l == 0 ? 1.2 : .9);
  }
  final h = o + Offset(-r * 1.3 - 6, -6);
  c.drawOval(Rect.fromCenter(center: h, width: 14, height: 18), _st(_wh, 1.2));
  _dot(c, h + const Offset(-2, -2), _wh, 1.2);
  _ln(c, h.dx + 4, h.dy - 8, h.dx + 10, h.dy - 11, _wh);
  for (final x in [-18.0, -8, 10, 20]) {
    _ln(c, o.dx + x, o.dy + r * .8 + 2, o.dx + x, 104, _wh);
  }
  _hdots(c, 8, 148, 105, _gr);
  _t(c, 'FLUFF', 8, 8, _bl);
  _num(c, _n(s.a), 8, 16, _bl, 20);
}

// 3. Venetian blind: the slats open to reveal; softness is the blur ghost on each slat edge.
void _m3(Canvas c, Size z, OpS s) {
  const r = Rect.fromLTWH(30, 14, 84, 92);
  _path(c, _poly([const Offset(30, 92), const Offset(44, 80), const Offset(52, 86), const Offset(62, 70), const Offset(70, 82), const Offset(86, 64), const Offset(96, 76), const Offset(114, 66)]), _wh, 1);
  _ring(c, const Offset(92, 36), 8, _wh, 1);
  final open = s.inv ? 1 - s.b : s.b;
  for (var i = 0; i < 7; i++) {
    final y = r.top + 8 + i * 12.6;
    final hh = (1 - open) * 4.6;
    final ghosts = (s.a * 2.4).round();
    for (var g = ghosts; g >= 0; g--) {
      final col = g == 0 ? _gr : _al(_bl, .6 - g * .2);
      _ln(c, r.left, y - hh - g * 2, r.right, y - hh - g * 2, col, g == 0 ? 1.1 : .6);
      _ln(c, r.left, y + hh + g * 2, r.right, y + hh + g * 2, col, g == 0 ? 1.1 : .6);
    }
  }
  c.drawRect(r, _st(_wh, 1.2));
  final cy = r.top + 10 + open * 70;
  _ln(c, 124, r.top, 124, cy, _gr, 1);
  _dot(c, Offset(124, cy + 2), _gr, 2.4);
  _t(c, 'OPEN', 130, 14, _gr, size: 7);
  _num(c, _n(s.b), 130, 22, _gr, 15);
}

// 4. Theatre curtains part (flick them); the fringe length is the softness.
void _m4(Canvas c, Size z, OpS s) {
  final open = s.a * 50;
  _path(c, Path()..addArc(const Rect.fromLTWH(8, 4, 140, 40), math.pi, math.pi), _wh, 1);
  _ln(c, 8, 24, 8, 112, _wh, 1);
  _ln(c, 148, 24, 148, 112, _wh, 1);
  final fx = 78.0;
  _ring(c, Offset(fx, 66), 4, _wh, 1.1);
  _ln(c, fx, 70, fx, 86, _wh, 1.1);
  final arm = math.sin(s.t * 3) * 4;
  _ln(c, fx, 74, fx - 8, 70 - arm, _wh, 1.1);
  _ln(c, fx, 74, fx + 8, 70 + arm, _wh, 1.1);
  _ln(c, fx, 86, fx - 5, 98, _wh, 1.1);
  _ln(c, fx, 86, fx + 5, 98, _wh, 1.1);
  _ln(c, 10, 100, 146, 100, _dg, 1);
  for (final side in [-1.0, 1.0]) {
    final edge = 78 - side * (6 + open);
    final outer = side < 0 ? 10.0 : 146.0;
    for (var i = 0; i < 7; i++) {
      final x0 = _lp(outer, edge, i / 6);
      final pts = <Offset>[];
      for (var y = 14.0; y <= 96; y += 4) {
        pts.add(Offset(x0 + math.sin(y * .18 + i + s.t * 1.4) * (1.5 + s.e * 2), y));
      }
      _path(c, _poly(pts), s.inv ? _rd : _gr, .9);
      final fr = 2 + s.a * 0 + (1 - s.b) * 0;
      final len = 3 + s.b * 12 + fr;
      _ln(c, x0, 97, x0, 97 + len * .6, _bl, .8);
      _dot(c, Offset(x0, 98 + len * .6), _bl, 1);
    }
  }
  _t(c, 'REVEAL ${_n(s.a)}', 12, 108, _gr, size: 6.5);
  _t(c, 'FRINGE ${_n(s.b)}', 144, 108, _bl, size: 6.5, right: true);
}

// 5. Isometric mesa: top = what is fully shown, slope = softness. Inverted it becomes a pit.
void _m5(Canvas c, Size z, OpS s) {
  const o = Offset(66, 44);
  const u = 6.0;
  for (var i = 0; i <= 8; i++) {
    _path(c, _poly([_iso(o, i - 4.0, -4, 0, u), _iso(o, i - 4.0, 4, 0, u)]), _dg, .7);
    _path(c, _poly([_iso(o, -4, i - 4.0, 0, u), _iso(o, 4, i - 4.0, 0, u)]), _dg, .7);
  }
  final top = .8 + s.b * 1.8, base = top + .3 + s.a * 1.9, h = (s.inv ? -1 : 1) * 3.2;
  List<Offset> sq(double r, double zz) => [_iso(o, -r, -r, zz, u), _iso(o, r, -r, zz, u), _iso(o, r, r, zz, u), _iso(o, -r, r, zz, u)];
  final t = sq(top, h), b = sq(base, 0);
  _path(c, _poly(b, close: true), _bl, 1);
  for (var i = 0; i < 4; i++) {
    _ln(c, t[i].dx, t[i].dy, b[i].dx, b[i].dy, _bl, 1);
  }
  _path(c, _poly(t, close: true), _gr, 1.3);
  _path(c, _star(_iso(o, 0, 0, h, u), 6, 2.8, 5, -math.pi / 2), _wh, 1);
  _t(c, 'SLOPE', 120, 14, _bl);
  _num(c, _n(s.a), 120, 22, _bl, 20);
  _t(c, s.inv ? 'PIT' : 'TOP', 120, 70, s.inv ? _rd : _gr);
  _num(c, _n(s.b), 120, 78, _gr, 20);
}

// 6. Alpha cross-section as an envelope: ramp, plateau, ramp, dotted verticals at the knees.
void _m6(Canvas c, Size z, OpS s) {
  const y0 = 102.0, y1 = 58.0, cx = 78.0;
  final half = 6 + s.b * 34, ramp = 2 + s.a * 30;
  final (lo, hi) = s.inv ? (y1, y0) : (y0, y1);
  final pts = <Offset>[Offset(8, lo)];
  for (var i = 0; i <= 10; i++) {
    final u = i / 10, e = u * u * (3 - 2 * u);
    pts.add(Offset(cx - half - ramp + ramp * u, _lp(lo, hi, e)));
  }
  for (var i = 0; i <= 10; i++) {
    final u = i / 10, e = u * u * (3 - 2 * u);
    pts.add(Offset(cx + half + ramp * u, _lp(hi, lo, e)));
  }
  pts.add(Offset(148, lo));
  _ln(c, 8, y0, 148, y0, _dg, 1);
  _path(c, _poly(pts), _wh, 1.2);
  for (final x in [cx - half - ramp, cx + half + ramp]) {
    _vdots(c, x, y1, y0, _bl);
    _dot(c, Offset(x, lo), _bl, 2);
  }
  for (final x in [cx - half, cx + half]) {
    _vdots(c, x, y1, y0, _gr);
    _dot(c, Offset(x, hi), _gr, 2);
  }
  _t(c, 'SOFT', 10, 8, _bl);
  _num(c, _n(s.a), 8, 17, _bl, 30);
  _t(c, 'EXPAND', 146, 8, _gr, right: true);
  _num(c, _n(s.b), 146, 17, _gr, 30, true);
  if (s.inv) _t(c, 'INV', 78, 8, _rd, mid: true);
}

// 7. The subject rendered as stipple; the dot density falls off across the soft edge. Pinch to grow.
void _m7(Canvas c, Size z, OpS s) {
  const o = Offset(78, 60);
  final r = 10 + s.a * 34, soft = 1 + s.b * 40;
  final pts = <Offset>[];
  var i = 0;
  for (var y = 3.0; y < 118; y += 3.2) {
    for (var x = 3.0; x < 154; x += 3.2) {
      i++;
      final d = (Offset(x, y) - o).distance;
      var al = ((r + soft / 2 - d) / soft).clamp(0.0, 1.0);
      if (s.inv) al = 1 - al;
      if (_h(i) < al) pts.add(Offset(x + (_h(i * 7) - .5), y + (_h(i * 3) - .5)));
    }
  }
  c.drawPoints(ui.PointMode.points, pts, _st(_wh, 1.3));
  _ring(c, o, r, _gr, .8);
  _ring(c, o, r + soft / 2, _al(_bl, .7), .8);
  _t(c, '${_n(s.a)} / ${_n(s.b)}', 150, 108, _bl, right: true, size: 7);
}

// 8. A planet's terminator: the shadow line is hatched, its spread is the softness. Spin to change the phase.
void _m8(Canvas c, Size z, OpS s) {
  const o = Offset(70, 60);
  const r = 34.0;
  for (var i = 0; i < 30; i++) {
    _dot(c, Offset(_h(i) * 156, _h(i + 40) * 120), _al(_wh, .3 + .4 * _h(i + 9)), .7);
  }
  _ring(c, o, r, _wh, 1.2);
  final ph = math.cos(s.a * math.pi * 2);
  final term = o.dx + ph * r;
  final soft = 2 + s.b * 26;
  for (var x = o.dx - r; x <= o.dx + r; x += 1.6) {
    final u = ((x - term) / soft + .5).clamp(0.0, 1.0);
    final lit = s.inv ? u : 1 - u;
    final hy = math.sqrt(math.max(0, r * r - (x - o.dx) * (x - o.dx)));
    if (_h((x * 10).round()) > lit) _ln(c, x, o.dy - hy, x, o.dy + hy, _al(_bl, .85), .7);
  }
  _vdots(c, term, o.dy - r - 6, o.dy + r + 6, _gr);
  _path(c, Path()..addOval(Rect.fromCenter(center: o, width: 120, height: 26)), _dg, .8);
  _dot(c, _pol(o, 1, 0) + Offset(math.cos(s.t * .6) * 60, math.sin(s.t * .6) * 13), _wh, 2);
  _t(c, 'PHASE', 148, 10, _gr, right: true);
  _num(c, _n(s.a), 148, 18, _gr, 18, true);
  _t(c, 'PENUMBRA', 148, 96, _bl, right: true, size: 6.5);
  _num(c, _n(s.b), 148, 104, _bl, 12, true);
}

// 9. Island contours: sea level picks which ring is the shore, contour spacing is the beach (softness).
void _m9(Canvas c, Size z, OpS s) {
  const o = Offset(70, 64);
  final step = 2 + s.a * 6;
  final shore = 4 + s.b * 26;
  for (var k = 0; k < 9; k++) {
    final r = 4 + k * step + (k > 0 ? 6 : 0);
    final land = s.inv ? r > shore : r <= shore;
    final p = _blob(o, r, k: .9, seed: 3, sx: 1.5);
    if (land) {
      _path(c, p, k == 0 ? _wh : _al(_bl, 1 - k * .08), 1);
    } else {
      _dash(c, p, _dg, 2, 3, w: .8);
    }
  }
  _path(c, _blob(o, shore, k: .9, seed: 3, sx: 1.5), _gr, 1.3);
  final tx = o.dx + 2, ty = o.dy - 2;
  _ln(c, tx, ty, tx + 2, ty - 14, _wh, 1);
  for (final a in [-2.6, -2.0, -1.2, -.6]) {
    _path(c, Path()..moveTo(tx + 2, ty - 14)..quadraticBezierTo(tx + 2 + math.cos(a) * 6, ty - 18 + math.sin(a) * 2, tx + 2 + math.cos(a) * 9, ty - 14 + math.sin(a).abs() * 3), _wh, .9);
  }
  for (var i = 0; i < 4; i++) {
    final x = 120 + i * 6.0 + math.sin(s.t + i) * 3, y = 16 + i * 22.0;
    _path(c, Path()..moveTo(x, y)..quadraticBezierTo(x + 3, y - 3, x + 6, y)..quadraticBezierTo(x + 9, y + 3, x + 12, y), _dg, 1);
  }
  _t(c, 'SEA', 8, 8, _gr);
  _num(c, _n(s.b), 8, 16, _gr, 16);
}

// 10. A submarine porthole: unscrew the rim (spin) to widen the view; the fog rings are the softness.
void _m10(Canvas c, Size z, OpS s) {
  const o = Offset(78, 60);
  final ap = 14 + s.a * 28, soft = s.b;
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: o, radius: ap)));
  for (var f = 0; f < 3; f++) {
    final x = (s.t * (14 + f * 7) + f * 50) % 200 - 40, y = 40 + f * 18.0;
    c.drawOval(Rect.fromCenter(center: Offset(x, y), width: 18, height: 9), _st(_wh, 1));
    _path(c, _poly([Offset(x - 9, y), Offset(x - 15, y - 4), Offset(x - 15, y + 4)], close: true), _wh, 1);
    _dot(c, Offset(x + 5, y - 1), _wh, .9);
  }
  for (var b = 0; b < 4; b++) {
    final y = 90 - ((s.t * 10 + b * 17) % 60);
    _ring(c, Offset(60 + b * 12.0, y), 1.5, _dg, .8);
  }
  c.restore();
  final rings = (soft * 5).round();
  for (var i = 1; i <= rings; i++) {
    _ring(c, o, ap - i * 2.4, _al(_bl, .8 - i * .13), .8);
  }
  _ring(c, o, ap, s.inv ? _rd : _gr, 1.3);
  _ring(c, o, 50, _wh, 1.2);
  _ring(c, o, 54, _wh, .8);
  for (var i = 0; i < 8; i++) {
    _dot(c, _pol(o, 52, s.a * math.pi * 2 + i * math.pi / 4), _wh, 1.5);
  }
  _t(c, 'VIEW', 6, 8, _gr);
  _num(c, _n(s.a), 6, 16, _gr, 14);
}

// 11. Giant outline letters seen through a window; the soft edge shows as ghost bands.
void _m11(Canvas c, Size z, OpS s) {
  final cx = 78.0, w = 8 + s.b * 60, soft = s.a * 24;
  void word(Color col) => _t(c, 'MASK', 78, 28, col, size: 58, w: FontWeight.w300, ls: -2, mid: true, outline: true);
  word(_dg);
  for (var i = 3; i >= 0; i--) {
    final ww = w + soft * i / 3;
    c.save();
    final band = Rect.fromLTRB(cx - ww, 0, cx + ww, 120);
    if (s.inv) {
      c.clipPath(Path()
        ..fillType = PathFillType.evenOdd
        ..addRect(const Rect.fromLTWH(0, 0, 156, 120))
        ..addRect(band));
    } else {
      c.clipRect(band);
    }
    word(i == 0 ? _wh : _al(_bl, .7 - i * .15));
    c.restore();
  }
  for (final sx in [-1.0, 1.0]) {
    final x = cx + sx * w;
    _ln(c, x, 12, x, 108, _gr, 1);
    _ln(c, x, 12, x - sx * 4, 12, _gr, 1);
    _ln(c, x, 108, x - sx * 4, 108, _gr, 1);
    _vdots(c, cx + sx * (w + soft), 16, 104, _bl);
  }
  _t(c, 'W ${_n(s.b)}  F ${_n(s.a)}', 6, 6, _wh, size: 6.5);
}

// 12. A cat: hatched world with a cat-shaped hole, or just the cat. Flick to swap; the outline ghosts are the softness.
Path _catPath(Offset o, double k) {
  Offset q(double x, double y) => o + Offset(x, y) * k;
  final p = Path();
  final pts = [q(-14, 22), q(-16, 4), q(-12, -8), q(-14, -22), q(-6, -14), q(6, -14), q(14, -22), q(12, -8), q(16, 4), q(14, 22)];
  p.moveTo(pts[0].dx, pts[0].dy);
  for (final x in pts.skip(1)) {
    p.lineTo(x.dx, x.dy);
  }
  p.quadraticBezierTo(q(0, 26).dx, q(0, 26).dy, pts[0].dx, pts[0].dy);
  return p..close();
}

void _m12(Canvas c, Size z, OpS s) {
  final inv = s.inv != (s.a > .5);
  const o = Offset(78, 60);
  final k = 1.1 + s.b * .9;
  final cat = _catPath(o, k);
  if (inv) {
    c.save();
    c.clipPath(Path()
      ..fillType = PathFillType.evenOdd
      ..addRect(const Rect.fromLTWH(0, 0, 156, 120))
      ..addPath(cat, Offset.zero));
    for (var x = -120.0; x < 160; x += 6) {
      _ln(c, x, 120, x + 120, 0, _al(_rd, .45), .7);
    }
    c.restore();
  }
  final g = (s.b * 4).round();
  for (var i = g; i >= 1; i--) {
    _path(c, _scaled(cat, o, 1 + i * .07), _al(_bl, .7 - i * .14), .8);
  }
  _path(c, cat, inv ? _rd : _wh, 1.3);
  final blink = (s.t % 3.2) < .12 ? .2 : 1.0;
  for (final ex in [-6.0, 6.0]) {
    c.drawOval(Rect.fromCenter(center: o + Offset(ex, -4) * k, width: 2.6 * k, height: 3.4 * k * blink), _fl(_wh));
  }
  for (final sy in [-1.0, 1.0]) {
    _ln(c, o.dx - 4 * k, o.dy + 3 * k, o.dx - 22 * k, o.dy + (3 + sy * 3) * k, _wh, .7);
    _ln(c, o.dx + 4 * k, o.dy + 3 * k, o.dx + 22 * k, o.dy + (3 + sy * 3) * k, _wh, .7);
  }
  _t(c, inv ? 'HOLE' : 'CAT', 6, 6, inv ? _rd : _wh);
  _t(c, 'SOFT ${_n(s.b)}', 150, 108, _bl, right: true, size: 6.5);
}

// 13. Camera iris: six blades around the opening; the dashed hexagon is where the soft edge ends.
void _m13(Canvas c, Size z, OpS s) {
  const o = Offset(70, 60);
  const outer = 46.0;
  final ap = 6 + s.a * 32, rot = s.a * 1.2;
  final hex = [for (var i = 0; i < 6; i++) _pol(o, ap, rot + i * math.pi / 3)];
  for (var i = 0; i < 6; i++) {
    final a = hex[i], b = hex[(i + 1) % 6];
    final dir = (b - a) / (b - a).distance;
    final far = a + dir * (outer * 2);
    _ln(c, a.dx, a.dy, far.dx, far.dy, _wh, 1);
  }
  final soft = ap + 3 + s.b * 14;
  _dash(c, _poly([for (var i = 0; i < 6; i++) _pol(o, soft, rot + i * math.pi / 3)], close: true), _bl, 2, 2, w: .9);
  _path(c, _poly(hex, close: true), s.inv ? _rd : _gr, 1.3);
  c.save();
  c.clipPath(Path()
    ..fillType = PathFillType.evenOdd
    ..addRect(const Rect.fromLTWH(0, 0, 156, 120))
    ..addOval(Rect.fromCircle(center: o, radius: outer)));
  c.drawRect(const Rect.fromLTWH(0, 0, 156, 120), _fl(_k));
  c.restore();
  _ring(c, o, outer, _wh, 1.2);
  _t(c, 'f/', 124, 18, _gr);
  _num(c, (16 - s.a * 14.6).toStringAsFixed(1), 124, 26, _gr, 16);
  _t(c, 'SOFT', 124, 80, _bl);
  _num(c, _n(s.b), 124, 88, _bl, 14);
}

// 14. Spray through a stencil: rub to spray; the overspray cloud is the softness.
void _m14(Canvas c, Size z, OpS s) {
  const o = Offset(58, 64);
  final st = _star(o, 22 + s.b * 10, 10 + s.b * 5, 5);
  c.drawRect(const Rect.fromLTWH(14, 22, 88, 84), _st(_dg, 1));
  final n = 260;
  final spread = 1 + s.a * 16;
  final pts = <Offset>[];
  for (var i = 0; i < n; i++) {
    final a = _h(i) * math.pi * 2, rr = 22 + s.b * 10;
    final edge = _h(i + 500);
    final base = o + Offset(math.cos(a), math.sin(a)) * rr * math.sqrt(edge);
    final j = Offset(_h(i + 1000) - .5, _h(i + 2000) - .5) * spread * 2;
    pts.add(base + j);
  }
  c.drawPoints(ui.PointMode.points, pts, _st(_al(_bl, .9), 1.4));
  _path(c, st, s.inv ? _rd : _wh, 1.2);
  const can = Offset(122, 32);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: can + const Offset(0, 14), width: 18, height: 34), const Radius.circular(3)), _st(_wh, 1.1));
  _ln(c, can.dx - 3, can.dy - 4, can.dx + 3, can.dy - 4, _wh, 1.1);
  _ln(c, can.dx - 9, can.dy + 4, can.dx + 9, can.dy + 4, _wh, .8);
  final puff = .3 + s.e;
  for (var i = 0; i < 5; i++) {
    final d = 6 + i * 6.0 * puff;
    _dot(c, can + Offset(-d, -4 + (_h(i + (s.t * 8).floor()) - .5) * i * 3), _al(_bl, 1 - i * .18), 1);
  }
  _t(c, 'MIST', 112, 94, _bl);
  _num(c, _n(s.a), 112, 102, _bl, 14);
}

// 15. Draw a lasso; it grows and softens into offset rings.
void _m15(Canvas c, Size z, OpS s) {
  var pts = s.trail;
  if (pts.length < 8) {
    pts = [for (var i = 0; i < 40; i++) const Offset(78, 60) + Offset(math.cos(i / 40 * 6.28) * (30 + 8 * math.sin(i / 40 * 18.8)), math.sin(i / 40 * 6.28) * 26)];
  }
  var cen = Offset.zero;
  for (final q in pts) {
    cen += q;
  }
  cen = cen / pts.length.toDouble();
  final loop = _poly(pts, close: true);
  final exp = .6 + s.b * .9;
  final grown = _scaled(loop, cen, exp);
  for (var i = 4; i >= 1; i--) {
    _path(c, _scaled(loop, cen, exp * (1 + i * .05 * (.2 + s.a * 2))), _al(_bl, .75 - i * .15), .8);
  }
  _path(c, grown, s.inv ? _rd : _gr, 1.2);
  _dash(c, loop, _wh, 2, 2, w: 1, phase: -s.t * 8);
  _dot(c, cen, _wh, 1.5);
  _t(c, 'DRAW', 6, 6, _wh);
  _t(c, 'EXP ${_n(s.b)}  SOFT ${_n(s.a)}', 150, 108, _gr, right: true, size: 6.5);
}
