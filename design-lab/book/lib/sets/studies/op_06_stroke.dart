part of 'op_06.dart';

// Stroke: width = blue, start = green, end = white, dash = red-magenta (tap toggles dash on/off where it is not driven by a gesture).
const op06StrokePanels = <OpP>[
  OpP('road trip', 'vehicle · effect · drag · drawing-led · sane', OpG.drag, _s1, a: .7, b: .4),
  OpP('snake', 'animal · effect · flick · drawing-led', OpG.flick, _s2, a: .8, b: .4),
  OpP('loop script', 'typographic · effect · drag · type-led', OpG.drag, _s3, a: .6, b: .4),
  OpP('dash step lane', 'M4L lane · mechanism · draw · diagram-led', OpG.draw, _s4),
  OpP('iso racetrack', 'isometric · mechanism · spin · numeral-led', OpG.spin, _s5, a: .7, wrap: true),
  OpP('spring turns', 'instrument · mechanism · pinch · drawing-led', OpG.pinch, _s6, a: .6, b: .3),
  OpP('pen plotter', 'machine · mechanism · drag · drawing-led', OpG.drag, _s7, a: .6, b: .3),
  OpP('telegraph key', 'instrument · effect · rub · drawing-led', OpG.rub, _s8, a: .4),
  OpP('constellation', 'cosmic · effect · drag · drawing-led', OpG.drag, _s9, a: .7, b: .2),
  OpP('zipper', 'object · effect · flick · drawing-led', OpG.flick, _s10, a: .4),
  OpP('marching ants', 'animal · effect · drag · drawing-led', OpG.drag, _s11, a: .8, b: .45),
  OpP('twin numerals', 'typographic · mechanism · drag · numeral-led · sane', OpG.drag, _s12, a: .3, b: .5),
  OpP('orbit', 'cosmic · mechanism · spin · drawing-led', OpG.spin, _s13, a: .35, wrap: true),
  OpP('garden hose', 'object landscape · effect · pinch · drawing-led', OpG.pinch, _s14, a: .4, b: .7),
  OpP('signal flow', 'diagram · mechanism · drag · diagram-led · sane', OpG.drag, _s15),
  OpP('free line', 'gesture · effect · draw · drawing-led', OpG.draw, _s16),
  OpP('sewing machine', 'machine · effect · rub · drawing-led', OpG.rub, _s17, a: .35),
  OpP('width profile', 'M4L lane · mechanism · draw · diagram-led', OpG.draw, _s18),
  OpP('cat & yarn', 'character · effect · flick · drawing-led', OpG.flick, _s19, a: .3),
  OpP('fireworks', 'cosmic · effect · flick · drawing-led · 振り切れ', OpG.flick, _s20, a: .5, b: .8),
];

Path _sPath(List<Offset> p) {
  final out = Path()..moveTo(p[0].dx, p[0].dy);
  for (var i = 1; i + 2 < p.length + 1; i += 3) {
    out.cubicTo(p[i].dx, p[i].dy, p[i + 1].dx, p[i + 1].dy, p[i + 2].dx, p[i + 2].dy);
  }
  return out;
}

// 1. A road: painted between the start flag and the car (end); lane width, dashed centre line.
void _s1(Canvas c, Size z, OpS s) {
  final road = _sPath(const [Offset(8, 104), Offset(60, 104), Offset(20, 40), Offset(78, 44), Offset(130, 48), Offset(96, 104), Offset(148, 92)]);
  const st = .08;
  final en = .15 + s.a * .85, w = 6 + s.b * 16;
  _dash(c, road, _dg, 1, 3, w: .8);
  _ribbon(c, road, w, _bl, a: st, b: en, caps: false);
  _dash(c, _trim(road, st, en), s.inv ? _dg : _rd, 3, 3, w: 1, phase: s.t * 4);
  final p0 = _at(road, st)!.position;
  _ln(c, p0.dx, p0.dy, p0.dx, p0.dy - 16, _gr, 1);
  _path(c, _poly([p0 + const Offset(0, -16), p0 + const Offset(9, -13), p0 + const Offset(0, -10)]), _gr, 1);
  final tg = _at(road, en)!;
  c.save();
  c.translate(tg.position.dx, tg.position.dy);
  c.rotate(-tg.angle);
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-7, -4, 14, 8), const Radius.circular(2)), Paint()..color = _k);
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-7, -4, 14, 8), const Radius.circular(2)), _st(_wh, 1.1));
  _ln(c, 2, -4, 2, 4, _wh, .8);
  c.restore();
  for (var i = 0; i < 4; i++) {
    final x = 112 + i * 9.0;
    _ln(c, x, 22, x, 22 - (5 + _h(i) * 10), _dg, 1);
  }
  _t(c, 'KM', 6, 6, _wh);
  _num(c, _n(s.a), 6, 14, _wh, 18);
}

// 2. A snake: the stroke is its body. Flick it to slither forward (end); width tapers to the tail; scales are the dashes.
void _s2(Canvas c, Size z, OpS s) {
  final pts = [for (var x = 6.0; x <= 150; x += 3) Offset(x, 64 + math.sin(x * .07 - s.t * 2) * 16 * (.4 + x / 150))];
  final body = _poly(pts);
  final st = .02, en = .2 + s.a * .8, w = 4 + s.b * 14;
  _ribbon(c, body, w, _bl, a: st, b: en, taper: (u) => ((u - st) / (en - st)).clamp(.15, 1.0), caps: false);
  final seg = _trim(body, st, en);
  _dash(c, seg, s.inv ? _dg : _rd, 1.5, 3.5, w: 1, phase: 0);
  final tg = _at(body, en)!;
  c.save();
  c.translate(tg.position.dx, tg.position.dy);
  c.rotate(-tg.angle);
  c.drawOval(Rect.fromCenter(center: const Offset(4, 0), width: 12, height: w * .9 + 2), Paint()..color = _k);
  c.drawOval(Rect.fromCenter(center: const Offset(4, 0), width: 12, height: w * .9 + 2), _st(_wh, 1.1));
  _dot(c, const Offset(6, -2), _wh, 1);
  final tl = 3 + math.sin(s.t * 9).abs() * 4;
  _ln(c, 10, 0, 10 + tl, 0, _rd, .9);
  _ln(c, 10 + tl, 0, 13 + tl, -2, _rd, .9);
  _ln(c, 10 + tl, 0, 13 + tl, 2, _rd, .9);
  c.restore();
  _t(c, 'LENGTH', 6, 6, _wh);
  _num(c, _n(s.a), 6, 14, _wh, 18);
}

// 3. A looping script line written by a neon tube: start and end trim the word; the tube is the width.
void _s3(Canvas c, Size z, OpS s) {
  final pts = <Offset>[];
  for (var i = 0; i <= 260; i++) {
    final u = i / 260 * 6.2 * math.pi;
    pts.add(Offset(12 + i / 260 * 130 - math.sin(u) * 9, 64 - math.cos(u) * 18 * (.7 + .3 * math.sin(u * .3))));
  }
  final word = _poly(pts);
  final st = s.b * .45, en = .5 + s.a * .5, w = 2.5 + s.b * 0 + 4;
  _path(c, word, _al(_dg, .7), .7);
  _ribbon(c, word, w, _bl, a: st, b: en, lw: .9);
  _path(c, _trim(word, st, en), _al(_wh, .9 + math.sin(s.t * 23) * .1), .8);
  final a = _at(word, st)!.position, b = _at(word, en)!.position;
  _dot(c, a, _gr, 2);
  _dot(c, b, _wh, 2);
  _num(c, _n(st), 6, 92, _gr, 20);
  _num(c, _n(en), 150, 92, _wh, 20, true);
}

// 4. Max-style step lane: paint the on/off steps; the preview line marches with that dash pattern.
void _s4(Canvas c, Size z, OpS s) {
  final on = 1 + (s.a * 6).round(), off = 1 + (s.b * 5).round();
  for (var i = 0; i < 16; i++) {
    final r = Rect.fromLTWH(6 + i * 9.0, 12, 8, 14);
    final lit = (i % (on + off)) < on;
    c.drawRect(r, _st(lit ? _rd : _dg, .8));
    if (lit) _dot(c, r.center, _rd, 1.4);
  }
  _t(c, 'ON $on', 6, 32, _rd, size: 6.5);
  _t(c, 'OFF $off', 150, 32, _wh, size: 6.5, right: true);
  final line = _sPath(const [Offset(6, 86), Offset(40, 46), Offset(80, 120), Offset(150, 70)]);
  _dash(c, line, s.inv ? _wh : _rd, on * 4.0, off * 4.0, w: 1.4, phase: s.t * 12);
  for (final q in s.trail) {
    _dot(c, q, _al(_wh, .3), .7);
  }
}

// 5. Isometric racetrack: the lit arc runs from the start post to the end post. Spin around the track.
void _s5(Canvas c, Size z, OpS s) {
  const o = Offset(78, 64);
  Path track(double r) {
    final pts = [for (var i = 0; i <= 64; i++) _iso(o, math.cos(i / 64 * math.pi * 2) * r * 1.4, math.sin(i / 64 * math.pi * 2) * r, 0, 9)];
    return _poly(pts);
  }

  final w = 0.3 + s.b * 1.3;
  final mid = track(3);
  _path(c, track(3 - w / 2), _dg, .8);
  _path(c, track(3 + w / 2), _dg, .8);
  const st = .05;
  final en = math.max(st + .02, s.a);
  _ribbon(c, mid, w * 9 * .7, _bl, a: st, b: en, caps: false);
  _dash(c, _trim(mid, st, en), _rd, 2, 3, w: .8);
  for (final (f, col) in [(st, _gr), (en, _wh)]) {
    final p = _at(mid, f)!.position;
    _ln(c, p.dx, p.dy, p.dx, p.dy - 14, col, 1.1);
    _dot(c, p + const Offset(0, -14), col, 1.8);
  }
  _num(c, _n(st), 6, 6, _gr, 22);
  _num(c, _n(en), 150, 6, _wh, 22, true);
  _t(c, 'START', 6, 32, _gr, size: 6);
  _t(c, 'END', 150, 32, _wh, size: 6, right: true);
}

// 6. The OP-1 spring: pinch to compress the turns between start and end; wire width is a double line.
void _s6(Canvas c, Size z, OpS s) {
  final x0 = 22.0, x1 = 30 + s.a * 108;
  const turns = 7;
  final pts = <Offset>[];
  for (var i = 0; i <= turns * 24; i++) {
    final u = i / (turns * 24);
    final a = u * turns * math.pi * 2;
    pts.add(Offset(x0 + (x1 - x0) * u + math.sin(a) * 6, 62 - math.cos(a) * 18));
  }
  final p = _poly(pts);
  final w = .5 + s.b * 4.5;
  if (w > 1.8) {
    _ribbon(c, p, w, _bl, lw: .7, caps: false);
  } else {
    _path(c, p, _bl, 1);
  }
  _t(c, 'T', 6, 50, _gr, size: 20, w: FontWeight.w200);
  _ln(c, x1 + 6, 40, x1 + 6, 84, _wh, 1);
  _dot(c, Offset(x1 + 6, 40), _wh, 1.8);
  _dot(c, Offset(x1 + 6, 84), _wh, 1.8);
  _t(c, 'TURNS', 78, 10, _gr, mid: true);
  _t(c, 'WIRE ${_n(s.b)}', 78, 100, _bl, mid: true, size: 7);
}

// 7. Pen plotter: the pen sits at the end of the path; the nib circle is the width.
void _s7(Canvas c, Size z, OpS s) {
  final spiral = <Offset>[];
  for (var i = 0; i <= 200; i++) {
    final u = i / 200, a = u * math.pi * 6;
    spiral.add(Offset(76 + math.cos(a) * (4 + u * 28) * 1.3, 66 + math.sin(a) * (4 + u * 28)));
  }
  final p = _poly(spiral);
  final en = .05 + s.a * .95, w = .5 + s.b * 6;
  c.drawRect(const Rect.fromLTWH(18, 28, 116, 80), _st(_dg, .8));
  _ribbon(c, p, w, _bl, b: en, lw: .7);
  _path(c, _trim(p, 0, en), _wh, .8);
  final pen = _at(p, en)!.position + Offset(0, math.sin(s.t * 6) * .4);
  _ln(c, 6, 16, 150, 16, _wh, 1);
  _ln(c, 6, 20, 150, 20, _wh, .6);
  _ln(c, pen.dx, 18, pen.dx, pen.dy - 6, _wh, 1);
  c.drawRect(Rect.fromCenter(center: Offset(pen.dx, 18), width: 12, height: 8), Paint()..color = _k);
  c.drawRect(Rect.fromCenter(center: Offset(pen.dx, 18), width: 12, height: 8), _st(_wh, 1));
  _path(c, _poly([pen + const Offset(-3, -6), pen, pen + const Offset(3, -6)]), _wh, 1);
  _ring(c, pen, w / 2 + 1, _bl, .8);
  _dot(c, _at(p, 0)!.position, _gr, 1.8);
  _t(c, 'NIB ${_n(s.b)}', 150, 110, _bl, right: true, size: 6.5);
}

// 8. A telegraph key: rub to tap it; the tape gets dashes, rubbing longer makes longer dashes.
void _s8(Canvas c, Size z, OpS s) {
  final press = (s.e * 1.4).clamp(0.0, 1.0) * (math.sin(s.t * 20) > 0 ? 1 : .3);
  c.drawRect(const Rect.fromLTWH(14, 92, 70, 8), _st(_wh, 1));
  _ln(c, 24, 92, 24, 82, _wh, 1);
  _ring(c, const Offset(24, 80), 3, _wh, 1);
  final tip = Offset(72, 72 + press * 6);
  _ln(c, 24, 80, tip.dx, tip.dy, _wh, 1.2);
  c.drawOval(Rect.fromCenter(center: tip + const Offset(0, -4), width: 16, height: 7), _st(_gr, 1.1));
  _path(c, Path()..addArc(const Rect.fromLTWH(44, 80, 10, 10), math.pi, math.pi * 1.6), _bl, .9);
  _ln(c, 92, 30, 154, 30, _wh, .8);
  _ln(c, 92, 50, 154, 50, _wh, .8);
  _ln(c, 6, 30, 92, 30, _dg, .8);
  _ln(c, 6, 50, 92, 50, _dg, .8);
  final dashLen = 2 + s.a * 16;
  final x0 = 6 - (s.t * 14 % 40);
  final code = [1, 0, 1, 1, 0, 1, 0, 0, 1, 1, 1, 0, 1];
  var x = x0;
  for (var i = 0; i < 26; i++) {
    final l = code[i % code.length] == 1 ? dashLen : 1.0;
    if (x > 4) _ln(c, x, 40, math.min(152, x + l), 40, i.isEven ? _rd : _al(_rd, .7), 2);
    x += l + 4;
  }
  _t(c, 'DASH', 104, 66, _rd);
  _num(c, _n(s.a), 104, 74, _rd, 22);
}

// 9. A constellation drawn between stars; the line ends at the brightest star.
void _s9(Canvas c, Size z, OpS s) {
  const st = [Offset(14, 92), Offset(34, 60), Offset(56, 74), Offset(70, 40), Offset(96, 30), Offset(114, 52), Offset(134, 36), Offset(146, 72)];
  for (var i = 0; i < 40; i++) {
    _dot(c, Offset(_h(i) * 156, _h(i + 70) * 120), _al(_wh, .25), .6);
  }
  final line = _poly(st);
  final en = s.a;
  _path(c, line, _al(_dg, .8), .6);
  final seg = _trim(line, 0, en);
  if (s.inv) {
    _path(c, seg, _wh, 1);
  } else {
    _dash(c, seg, _rd, 1, 2.5 + s.b * 6, w: 1.4);
  }
  for (var i = 0; i < st.length; i++) {
    final tw = .6 + .4 * math.sin(s.t * 2 + i * 1.7);
    _ln(c, st[i].dx - 3 * tw, st[i].dy, st[i].dx + 3 * tw, st[i].dy, _wh, .8);
    _ln(c, st[i].dx, st[i].dy - 3 * tw, st[i].dx, st[i].dy + 3 * tw, _wh, .8);
  }
  final e = _at(line, en)!.position;
  _ring(c, e, 5, _wh, 1);
  _dot(c, st[0], _gr, 2);
  _t(c, 'GAP ${_n(s.b)}', 150, 108, _rd, right: true, size: 6.5);
  _t(c, 'REACH ${_n(s.a)}', 6, 6, _wh, size: 6.5);
}

// 10. Zipper: flick the pull to close; teeth spacing is the dash.
void _s10(Canvas c, Size z, OpS s) {
  final pull = 10 + s.a * 100;
  final gap = 3 + s.b * 5;
  const cx = 78.0;
  for (var y = 8.0; y < 116; y += gap) {
    final open = y < pull ? ((pull - y) / 100) * 22 : 0.0;
    final left = (y / gap).round().isEven;
    final x = cx + (left ? -1 : 1) * (2 + open);
    _ln(c, x - (left ? 5 : 0), y, x + (left ? 0 : 5), y, y < pull ? _al(_rd, .6) : _rd, 1.4);
  }
  for (final sx in [-1.0, 1.0]) {
    final pts = [for (var y = 4.0; y <= 116; y += 4) Offset(cx + sx * (y < pull ? 8 + (pull - y) / 100 * 22 : 8), y)];
    _path(c, _poly(pts), _wh, .9);
  }
  final sw = math.sin(s.t * 2) * .15;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, pull), width: 14, height: 10), const Radius.circular(3)), Paint()..color = _k);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(cx, pull), width: 14, height: 10), const Radius.circular(3)), _st(_gr, 1.2));
  c.save();
  c.translate(cx, pull + 5);
  c.rotate(sw);
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-4, 0, 8, 16), const Radius.circular(4)), _st(_gr, 1));
  c.restore();
  _t(c, 'CLOSED', 6, 6, _gr);
  _num(c, _n(s.a), 6, 14, _gr, 18);
  _t(c, 'TEETH', 150, 6, _rd, right: true);
  _num(c, _n(s.b), 150, 14, _rd, 18, true);
}

// 11. Marching ants: a selection outline whose dashes are ants; trim limits the trail, ant size is the width.
void _s11(Canvas c, Size z, OpS s) {
  final sel = Path()..addRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(18, 20, 120, 80), const Radius.circular(10)));
  final m = sel.computeMetrics().first;
  _dash(c, sel, _dg, 1, 3, w: .7);
  final sp = 10 + s.b * 0 + 4.0;
  final k = .6 + s.b * 1.2;
  final n = (m.length * s.a / sp).floor();
  for (var i = 0; i < n; i++) {
    final d = (i * sp + s.t * 10) % (m.length * s.a);
    final tg = m.getTangentForOffset(d)!;
    c.save();
    c.translate(tg.position.dx, tg.position.dy);
    c.rotate(-tg.angle);
    c.scale(k);
    for (final x in [-2.6, 0.0, 2.6]) {
      _dot(c, Offset(x, 0), i == n - 1 ? _wh : _rd, x == 0 ? .9 : 1.2);
    }
    final leg = math.sin(s.t * 14 + i) * 1.2;
    for (final x in [-1.3, 0.0, 1.3]) {
      _ln(c, x, 0, x + leg, -2.6, _al(_wh, .7), .5);
      _ln(c, x, 0, x - leg, 2.6, _al(_wh, .7), .5);
    }
    c.restore();
  }
  _dot(c, m.getTangentForOffset(0)!.position, _gr, 2);
  _t(c, 'TRAIL', 78, 46, _wh, mid: true);
  _num(c, _n(s.a), 78, 54, _wh, 22);
}

// 12. Two big numerals: drag sideways to move the start, up and down to move the end.
void _s12(Canvas c, Size z, OpS s) {
  final st = s.a * .5;
  final en = .5 + s.b * .5;
  _num(c, _n(st), 6, 6, _gr, 44);
  _num(c, _n(en), 150, 6, _wh, 44, true);
  _t(c, 'START', 8, 54, _gr, size: 6.5);
  _t(c, 'END', 148, 54, _wh, size: 6.5, right: true);
  final line = Path()
    ..moveTo(8, 92)
    ..cubicTo(50, 70, 100, 116, 148, 88);
  _path(c, line, _dg, .8);
  _path(c, _trim(line, st, en), _wh, 1.4);
  _dot(c, _at(line, st)!.position, _gr, 2.4);
  _dot(c, _at(line, en)!.position, _wh, 2.4);
  for (var i = 0; i <= 10; i++) {
    final x = 8 + i * 14.0;
    _ln(c, x, 108, x, i % 5 == 0 ? 102 : 105, _dg, .8);
  }
}

// 13. An orbit: the trimmed arc is the path the moon has travelled; outer belt is the dash.
void _s13(Canvas c, Size z, OpS s) {
  const o = Offset(78, 60);
  final orbit = Path()..addOval(Rect.fromCenter(center: o, width: 116, height: 64));
  _ring(c, o, 8, _wh, 1.1);
  for (var i = 0; i < 8; i++) {
    final a = i * math.pi / 4 + s.t * .3;
    final p0 = _pol(o, 11, a), p1 = _pol(o, 14, a);
    _ln(c, p0.dx, p0.dy, p1.dx, p1.dy, _wh, .9);
  }
  _dash(c, orbit, _dg, 1, 3, w: .8);
  const st = 0.0;
  final en = s.a;
  final w = 1 + s.b * 8;
  _ribbon(c, orbit, w, _bl, a: st, b: en, lw: .8);
  _path(c, _trim(orbit, st, en), _wh, .8);
  _dash(c, Path()..addOval(Rect.fromCenter(center: o, width: 148, height: 104)), _rd, 3, 5, w: 1, phase: -s.t * 6);
  _ring(c, _at(orbit, 0)!.position, 3, _gr, 1.1);
  final m = _at(orbit, en)!.position;
  _ring(c, m, 5, _wh, 1.1);
  _dot(c, m + const Offset(1.5, -1), _wh, 1.2);
  _t(c, '${(s.a * 360).round()}°', 150, 6, _wh, right: true, size: 9, w: FontWeight.w300);
}

// 14. A garden hose: pinch the nozzle to change the hose width; the reel unwinds as the end grows; water is the dash.
void _s14(Canvas c, Size z, OpS s) {
  const reel = Offset(26, 82);
  final coils = (1 - s.b) * 5;
  for (var i = 0; i < coils.ceil(); i++) {
    _ring(c, reel, 6 + i * 3.0, _al(_bl, .8), .8);
  }
  _ring(c, reel, 22, _wh, 1);
  _ln(c, reel.dx - 16, 110, reel.dx + 16, 110, _wh, 1);
  _ln(c, reel.dx, reel.dy, reel.dx - 12, 110, _wh, 1);
  _ln(c, reel.dx, reel.dy, reel.dx + 12, 110, _wh, 1);
  final hose = _sPath(const [Offset(26, 64), Offset(60, 20), Offset(80, 110), Offset(108, 64), Offset(118, 46), Offset(130, 40), Offset(140, 44)]);
  final en = .1 + s.b * .9, w = 2 + s.a * 10;
  _ribbon(c, hose, w, _bl, b: en, caps: false);
  final tg = _at(hose, en)!;
  final dir = Offset(math.cos(-tg.angle), math.sin(-tg.angle));
  final nz = tg.position;
  _ring(c, nz, w / 2 + 2, _wh, 1.1);
  for (var j = 0; j < 3; j++) {
    final pts = <Offset>[];
    for (var i = 0; i <= 20; i++) {
      final u = i / 20 * 40;
      pts.add(nz + dir * u + Offset(0, u * u * .012 + (j - 1) * u * .08 * (1 - s.a)));
    }
    _dash(c, _poly(pts), _rd, 2, 4, w: 1, phase: -s.t * 30 + j);
  }
  _t(c, 'BORE', 150, 6, _bl, right: true);
  _num(c, _n(s.a), 150, 14, _bl, 18, true);
}

// 15. OP-1 flow: path → start/end clamp → width amp → dash stack → output.
void _s15(Canvas c, Size z, OpS s) {
  const y = 46.0;
  c.drawRect(const Rect.fromLTWH(4, 32, 24, 28), _st(_wh, 1));
  _path(c, Path()
    ..moveTo(8, 54)
    ..cubicTo(14, 30, 20, 60, 24, 38), _wh, 1);
  _ln(c, 28, y, 34, y, _dg, 1);
  c.drawRect(const Rect.fromLTWH(34, 36, 26, 20), _st(_gr, 1));
  _ln(c, 37, y, 37 + s.a * 20, y, _gr, 1.4);
  _t(c, 'S/E', 47, 59, _gr, size: 6, mid: true);
  _ln(c, 60, y, 66, y, _dg, 1);
  _path(c, _poly([const Offset(66, 30), const Offset(94, 46), const Offset(66, 62)], close: true), _bl, 1.2);
  _t(c, _n(s.b), 68, 40, _bl, size: 11, w: FontWeight.w300, ls: 0);
  _ln(c, 94, y, 100, y, _dg, 1);
  for (var i = 2; i >= 0; i--) {
    final r = Rect.fromLTWH(100 + i * 3.0, 34 - i * 3.0, 24, 24);
    c.drawRect(r, Paint()..color = _k);
    c.drawRect(r, _st(_al(_rd, 1 - i * .3), 1));
  }
  _dash(c, Path()..moveTo(104, 46)..lineTo(120, 46), s.inv ? _dg : _rd, 2, 2, w: 1.2, phase: s.t * 6);
  _ln(c, 127, y, 134, y, _dg, 1);
  _ring(c, const Offset(144, 46), 8, _wh, 1);
  _dot(c, const Offset(144, 46), _wh, 1.6);
  final out = Path()
    ..moveTo(10, 100)
    ..cubicTo(50, 76, 100, 120, 146, 90);
  _ribbon(c, out, 2 + s.b * 8, _bl, a: s.a * .5, b: .5 + s.a * .5, lw: .8);
  _dash(c, _trim(out, s.a * .5, .5 + s.a * .5), s.inv ? _wh : _rd, 3, 3, w: .8, phase: s.t * 6);
}

// 16. Draw a line; it becomes the stroke — the width breathes with how fast you drew it, the dash follows.
void _s16(Canvas c, Size z, OpS s) {
  var pts = s.trail;
  if (pts.length < 4) {
    pts = [for (var i = 0; i <= 40; i++) Offset(10 + i * 3.4, 60 + math.sin(i * .35) * 26 * math.cos(i * .07))];
  }
  final line = _poly(pts);
  final w = 1 + s.b * 14;
  _ribbon(c, line, w, _bl, taper: (u) => .3 + .7 * math.sin(u * math.pi), lw: .8);
  _dash(c, line, s.inv ? _wh : _rd, 1 + s.a * 10, 3, w: 1, phase: s.t * 8);
  _dot(c, pts.first, _gr, 2);
  _dot(c, pts.last, _wh, 2);
  _t(c, 'DRAW', 6, 6, _wh);
  _t(c, 'W ${_n(s.b)}  DASH ${_n(s.a)}', 150, 108, _bl, right: true, size: 6.5);
}

// 17. Sewing machine: rub to sew; the seam grows (end) and the stitch length is the dash.
void _s17(Canvas c, Size z, OpS s) {
  _path(c, _poly([const Offset(20, 74), const Offset(20, 18), const Offset(110, 18), const Offset(110, 30), const Offset(34, 30), const Offset(34, 74)]), _wh, 1.1);
  _ln(c, 10, 74, 146, 74, _wh, 1.1);
  final sp = 1.2 + s.e * 10;
  final ny = 46 + math.sin(s.t * sp * 6) * 6;
  c.drawRect(const Rect.fromLTWH(100, 30, 12, 10), _st(_wh, 1));
  _ln(c, 106, 40, 106, ny + 18, _wh, 1);
  _dot(c, Offset(106, ny + 18), _wh, 1);
  _ring(c, const Offset(126, 24), 8, _wh, 1);
  _ln(c, 126, 24, 126 + math.cos(s.t * sp * 6) * 8, 24 + math.sin(s.t * sp * 6) * 8, _gr, 1);
  final seam = Path()
    ..moveTo(106, 92)
    ..cubicTo(80, 112, 40, 80, 6, 100);
  _dash(c, seam, _dg, 1, 3, w: .7);
  final stitch = 2 + s.b * 0 + 3;
  _dash(c, _trim(seam, 0, s.a), _rd, stitch, 3, w: 1.6);
  _t(c, 'SEAM', 150, 76, _wh, right: true);
  _num(c, _n(s.a), 150, 84, _wh, 22, true);
}

// 18. A width profile lane (draw it); the stroke above swells and thins along its length to match.
void _s18(Canvas c, Size z, OpS s) {
  final prof = List<double>.filled(9, .5);
  for (var i = 0; i < 9; i++) {
    prof[i] = .35 + .3 * math.sin(i * .9);
  }
  for (final q in s.trail) {
    final i = ((q.dx - 8) / 140 * 8).round().clamp(0, 8);
    if (q.dy > 66) prof[i] = (1 - (q.dy - 70) / 44).clamp(0.0, 1.0);
  }
  _profile = prof;
  c.drawRect(const Rect.fromLTWH(6, 70, 144, 44), _st(_dg, .8));
  for (var i = 0; i <= 8; i++) {
    _vdots(c, 8 + i * 17.5, 72, 112, _al(_dg, .8));
  }
  final pp = [for (var i = 0; i <= 8; i++) Offset(8 + i * 17.5, 112 - prof[i] * 40)];
  _path(c, _poly(pp), _bl, 1.1);
  for (final q in pp) {
    _dot(c, q, _bl, 1.8);
  }
  final line = Path()
    ..moveTo(10, 36)
    ..cubicTo(50, 6, 100, 66, 146, 30);
  _ribbon(c, line, 22, _bl, taper: _taper, lw: .9);
  _path(c, line, _al(_wh, .5), .6);
  _dot(c, const Offset(10, 36), _gr, 2);
  _dot(c, const Offset(146, 30), _wh, 2);
}

var _profile = List<double>.filled(9, .5);
double _taper(double u) {
  final f = u * 8, i = f.floor().clamp(0, 7);
  return math.max(.05, _lp(_profile[i], _profile[i + 1], f - i));
}

// 19. A cat bats the yarn ball (flick); the thread unrolls behind it.
void _s19(Canvas c, Size z, OpS s) {
  const paw = Offset(30, 74);
  final bx = 44 + s.a * 100;
  final ball = Offset(bx, 96);
  final thread = Path()
    ..moveTo(paw.dx + 6, paw.dy + 6)
    ..cubicTo(paw.dx + 20, 120, bx - 30, 80, bx - 9, 98);
  _dash(c, thread, _wh, 2 + s.b * 6, 2, w: .9);
  _ring(c, ball, 9, _wh, 1.1);
  final rot = s.a * 14;
  for (var i = 0; i < 3; i++) {
    final a = rot + i * 1.05;
    c.drawArc(Rect.fromCircle(center: ball, radius: 9), a, 1.8, false, _st(_rd, .8));
    _path(c, Path()..addOval(Rect.fromCenter(center: ball, width: 18 * math.cos(a).abs() + .5, height: 18)), _al(_rd, .6), .7);
  }
  final cat = Path()
    ..moveTo(10, 108)
    ..quadraticBezierTo(4, 80, 14, 60)
    ..lineTo(12, 46)
    ..lineTo(20, 54)
    ..lineTo(28, 54)
    ..lineTo(34, 44)
    ..lineTo(34, 60)
    ..quadraticBezierTo(40, 80, 34, 108);
  _path(c, cat, _wh, 1.1);
  final swat = math.sin(s.t * 4) * 3 + s.e * 6;
  _path(c, Path()
    ..moveTo(28, 70)
    ..quadraticBezierTo(34 + swat, 66, paw.dx + 4 + swat, paw.dy), _wh, 1.1);
  _dot(c, const Offset(19, 60), _wh, 1);
  _dot(c, const Offset(28, 60), _wh, 1);
  final tail = math.sin(s.t * 1.6) * 6;
  _path(c, Path()
    ..moveTo(10, 106)
    ..quadraticBezierTo(-4, 90 + tail, 6, 76 + tail), _wh, 1);
  _ln(c, 4, 108, 152, 108, _dg, 1);
  _t(c, 'UNROLL', 150, 6, _wh, right: true);
  _num(c, _n(s.a), 150, 14, _wh, 22, true);
}

// 20. Fireworks: every trail is a trimmed path — start chases end outward; width and dash go wild.
void _s20(Canvas c, Size z, OpS s) {
  const o = Offset(78, 56);
  final ph = ((s.t * .35) + s.a) % 1;
  final st = math.max(0.0, ph * 1.3 - .35), en = math.min(1.0, ph * 1.3);
  final cols = [_bl, _gr, _wh, _rd, _pu];
  final n = 10 + (s.b * 14).round();
  for (var i = 0; i < n; i++) {
    final a = i / n * math.pi * 2 + .2;
    final len = 30 + _h(i) * 30;
    final end = o + Offset(math.cos(a) * len, math.sin(a) * len * .85 + len * .3);
    final ctl = o + Offset(math.cos(a) * len * .6, math.sin(a) * len * .6 - 10);
    final p = Path()
      ..moveTo(o.dx, o.dy)
      ..quadraticBezierTo(ctl.dx, ctl.dy, end.dx, end.dy);
    final col = cols[i % cols.length];
    if (s.b > .6 && i.isEven) {
      _ribbon(c, p, 1 + s.b * 4, col, a: st, b: en, lw: .6);
    } else {
      _dash(c, _trim(p, st, en), col, 1 + (1 - s.b) * 5, 1 + s.b * 4, w: 1);
    }
    final tip = _at(p, en)!.position;
    _dot(c, tip, col, 1.2);
  }
  _ln(c, 78, 118, 78, 56 + (1 - math.min(1.0, ph * 4)) * 60, _al(_wh, .4), .8);
  _num(c, _n(s.b), 150, 14, _rd, 22, true);
  _t(c, 'BURST', 150, 6, _rd, right: true, size: 6.5);
}
