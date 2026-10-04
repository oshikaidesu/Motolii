// Repeat / Clone: count (blue), offset (green), rotation step (white), accent (red).
part of 'op_08.dart';

List<_Op> _repeatPanels() => const [
      _Op('Herd delay', 'animal · effect · drag count/gap', _G.drag, _rCow, a: .4, b: .3),
      _Op('Iso array', 'isometric · effect · drag cols/rows', _G.drag, _rIso, a: .5, b: .4),
      _Op('Clone chain', 'diagram · mechanism · flick count', _G.flick, _rFlow, a: .4, b: .5),
      _Op('Ghost numeral', 'typographic · numeral led · flick', _G.flick, _rBig, a: .3, b: .5),
      _Op('Moon step', 'cosmic · mechanism · spin step', _G.spin, _rMoons, a: .125, b: .4),
      _Op('Monkey band', 'character · time · rub delay', _G.rub, _rDrum, a: .7, b: .2, wrapB: true),
      _Op('Lane dashes', 'vehicle · landscape · drag count', _G.drag, _rRoad, a: .4, b: .5),
      _Op('Coil turns', 'machine · mechanism · pinch pitch', _G.pinch, _rCoil, a: .5, b: .45),
      _Op('Echo waterfall', 'instrument · depth · drag layers', _G.drag, _rFall, a: .55, b: .4),
      _Op('Step lane', 'M4L device · draw cells, tap clear', _G.draw, _rSteps, keep: true),
      _Op('Ferris wheel', 'machine · effect · spin, out cars', _G.spin, _rWheel, a: 0, b: .45),
      _Op('Footprints', 'landscape · effect · draw the walk', _G.draw, _rFoot),
      _Op('Moire 180', 'diagram · extreme · spin step', _G.spin, _rMoire, a: .3, b: .6),
      _Op('Fish school', 'animal · effect · flick swim', _G.flick, _rFish, a: .2, b: .5, wrap: true),
      _Op('Stair clone', 'isometric · character · drag rise', _G.drag, _rStair, a: .6, b: .35),
      _Op('Kaleido draw', 'abstract · effect · draw, tap n', _G.draw, _rKaleido, a: .3, tap: 1 / 9),
      _Op('Golden 137.5', 'cosmic · extreme · spin angle', _G.spin, _rGold, a: .47, b: .5),
      _Op('Frame ruler', 'ruler · numeral led · flick frames', _G.flick, _rRuler,
          a: .17, b: .4),
      _Op('Skyline build', 'landscape · effect · drag blocks', _G.drag, _rCity, a: .5, b: .5),
      _Op('Infinity mirror', 'optical · extreme · pinch scale', _G.pinch, _rMirror,
          a: .4, b: .6),
    ];

void _rCow(Canvas c, Size s, _S st, double t) {
  final n = 1 + (st.a * 4).round(), gap = 24 + st.b * 40, base = s.height - 16;
  _ln(c, Offset(4, base), Offset(s.width - 4, base), _dg, 1);
  for (var i = n - 1; i >= 0; i--) {
    final col = i == 0 ? _wh : _al(_bl, 1 - i / (n + 1) * .7);
    _cow(c, Offset(10 + i * gap, base), 52, col, t * 2.2 - i * .8);
  }
  if (n > 1) {
    final y = base + 6;
    _ln(c, Offset(10, y), Offset(10 + gap, y), _gr, 1);
    _ln(c, Offset(10, y - 2), Offset(10, y + 2), _gr, 1);
    _ln(c, Offset(10 + gap, y - 2), Offset(10 + gap, y + 2), _gr, 1);
  }
  _lab(c, 'count', const Offset(8, 8), _bl);
  _num(c, _d2(n), const Offset(7, 18), 28, _bl);
  _lab(c, 'gap', Offset(s.width - 8, 8), _gr, ax: 1);
  _num(c, _d2(gap), Offset(s.width - 8, 18), 18, _gr, ax: 1);
}

void _rIso(Canvas c, Size s, _S st, double t) {
  final o = Offset(s.width * .5, 34), k = 10.0;
  for (var i = 0; i <= 6; i++) {
    _ln(c, _iso(o, i.toDouble(), 0, 0, k), _iso(o, i.toDouble(), 6, 0, k), _dg, .8);
    _ln(c, _iso(o, 0, i.toDouble(), 0, k), _iso(o, 6, i.toDouble(), 0, k), _dg, .8);
  }
  final cx = 1 + (st.a * 5).round(), cy = 1 + (st.b * 5).round();
  final cells = <(int, int)>[for (var i = 0; i < cx; i++) for (var j = 0; j < cy; j++) (i, j)]
    ..sort((p, q) => (p.$1 + p.$2).compareTo(q.$1 + q.$2));
  for (final (i, j) in cells) {
    final last = i == cx - 1 && j == cy - 1;
    final hz = last ? .7 + .25 * math.sin(t * 3) : .7;
    _isoBox(c, o, i + .15, j + .15, 0, .7, .7, hz, k, last ? _gr : _wh, 1);
  }
  _lab(c, 'x', _iso(o, 3, 0, 0, k) + const Offset(10, -14), _bl);
  _lab(c, 'y', _iso(o, 0, 3, 0, k) + const Offset(-18, -14), _gr);
  _num(c, _d2(cx), const Offset(8, 8), 22, _bl);
  _num(c, _d2(cy), Offset(s.width - 8, 8), 22, _gr, ax: 1);
}

void _rFlow(Canvas c, Size s, _S st, double t) {
  final n = 1 + (st.a * 11).round(), off = st.b, y = 52.0;
  c.drawRect(Rect.fromLTWH(6, y - 13, 26, 26), _s(_wh, 1));
  _pl(c, [Offset(13, y + 6), Offset(19, y - 6), Offset(25, y + 6)], _wh, w: 1, close: true);
  _ln(c, Offset(32, y), Offset(42, y), _dg, 1);
  _pl(c, [Offset(42, y - 15), Offset(70, y), Offset(42, y + 15)], _bl, w: 1.2, close: true);
  _num(c, _d2(n), Offset(44, y - 7), 13, _bl);
  _ln(c, Offset(70, y), Offset(80, y), _dg, 1);
  final pages = math.min(n, 6);
  for (var i = pages - 1; i >= 0; i--) {
    final d = Offset(i * (1 + off * 3), -i * (1 + off * 3));
    c.drawRect(Rect.fromLTWH(80, y - 11, 22, 22).shift(d), _f(_bk));
    c.drawRect(Rect.fromLTWH(80, y - 11, 22, 22).shift(d), _s(i == 0 ? _gr : _al(_gr, .5), 1));
  }
  _pl(c, [Offset(84, y + 3), Offset(88, y - 3), Offset(92, y + 3), Offset(96, y - 3)], _gr, w: 1);
  _ln(c, Offset(102, y), Offset(118, y), _dg, 1);
  const wc = Offset(134, 52);
  _ring(c, wc, 15, _wh, 1);
  for (var i = 0; i < n; i++) {
    final a = -math.pi / 2 + i * 2 * math.pi / n + t * .4;
    _dot(c, wc + Offset(math.cos(a), math.sin(a)) * 10, 1.5, i == 0 ? _rd : _wh);
  }
  _dot(c, wc, 1.2, _bl);
  final u = _wr(t * .5);
  final px = 32 + u * 86;
  _dot(c, Offset(px, y), 1.6, _gr);
  _lab(c, 'sample', Offset(19, y + 22), _wh, ax: .5, size: 7);
  _lab(c, 'count', Offset(54, y + 22), _bl, ax: .5, size: 7);
  _lab(c, 'offset', Offset(91, y + 22), _gr, ax: .5, size: 7);
  _lab(c, 'out', Offset(134, y + 22), _wh, ax: .5, size: 7);
}

void _rBig(Canvas c, Size s, _S st, double t) {
  final n = 1 + (st.a * 19).round(), step = 2 + st.b * 9, txt = _d2(n);
  final ghosts = math.min(n - 1, 7);
  const base = Offset(22, 38);
  for (var i = ghosts; i >= 1; i--) {
    final q = (8 * (1 - i / 8)).round() / 8;
    _num(c, txt, base + Offset(i * step, -i * step * .55), 56, _al(_bl, .15 + q * .55));
  }
  _num(c, txt, base, 56, _wh);
  _lab(c, 'count', const Offset(8, 8), _bl);
  _lab(c, 'step ${step.toStringAsFixed(1)}', Offset(s.width - 8, s.height - 14), _gr, ax: 1);
  _ln(c, Offset(22, s.height - 10), Offset(22 + ghosts * step, s.height - 10), _gr, 1);
  _dot(c, Offset(22 + ghosts * step + math.sin(t * 3), s.height - 10), 1.4, _gr);
}

void _rMoons(Canvas c, Size s, _S st, double t) {
  final cen = Offset(s.width * .38, s.height * .52);
  final n = 2 + (st.b * 10).round(), step = st.a * 360;
  _ring(c, cen, 8, _wh, 1);
  c.drawOval(Rect.fromCenter(center: cen, width: 26, height: 6), _s(_wh, 1));
  c.drawRect(Rect.fromCenter(center: cen, width: 15, height: 5), _f(_bk));
  _ring(c, cen, 8, _wh, 1);
  final dots = <Offset>[];
  for (var i = 0; i < 90; i++) {
    final a = i / 90 * 2 * math.pi;
    dots.add(cen + Offset(math.cos(a), math.sin(a)) * 38);
  }
  c.drawPoints(ui.PointMode.points, dots, _s(_dg, 1));
  final base = t * .3;
  for (var i = 0; i < n; i++) {
    final a = base + i * step * math.pi / 180;
    final p = cen + Offset(math.cos(a), math.sin(a)) * 38;
    if (i == 0) {
      _dot(c, p, 3, _wh);
    } else {
      _ring(c, p, 3, _al(_bl, 1 - i / (n + 2)), 1);
    }
  }
  c.drawArc(Rect.fromCircle(center: cen, radius: 30), base, step * math.pi / 180, false, _s(_wh, 1));
  _lab(c, 'step', Offset(s.width - 8, 10), _wh, ax: 1);
  _num(c, _d3(step), Offset(s.width - 8, 20), 22, _wh, ax: 1);
  _lab(c, 'moons', Offset(s.width - 8, 66), _bl, ax: 1);
  _num(c, _d2(n), Offset(s.width - 8, 76), 22, _bl, ax: 1);
}

void _rDrum(Canvas c, Size s, _S st, double t) {
  final n = 1 + (st.a * 3).round(), delay = st.b;
  final span = s.width / n;
  for (var i = 0; i < n; i++) {
    final ph = _wr(t * .9 - i * delay);
    _monkey(c, Offset(span * (i + .5), 44), 8, i == 0 ? _wh : _bl, ph, _rd);
  }
  _lab(c, 'delay', const Offset(6, 6), _gr);
  _num(c, _d3(delay * 1000), const Offset(6, 15), 14, _gr);
  _lab(c, 'band', Offset(s.width - 6, 6), _bl, ax: 1);
}

void _rRoad(Canvas c, Size s, _S st, double t) {
  final hz = 46.0, vp = Offset(s.width / 2, hz), bot = s.height - 22;
  final sky = [
    Offset(8, hz), const Offset(8, 38), const Offset(18, 38), const Offset(18, 30), const Offset(26, 30), const Offset(26, 40),
    const Offset(40, 40), const Offset(40, 34), const Offset(48, 34), Offset(48, hz),
  ];
  _pl(c, sky, _al(_wh, .6), w: 1);
  _pl(c, [Offset(s.width - 50, hz), Offset(s.width - 50, 36), Offset(s.width - 40, 36), Offset(s.width - 40, 26), Offset(s.width - 34, 26),
    Offset(s.width - 34, 40), Offset(s.width - 20, 40), Offset(s.width - 20, hz)], _al(_wh, .6), w: 1);
  _ln(c, Offset(0, hz), Offset(s.width, hz), _wh, 1);
  _ln(c, vp, Offset(6, bot), _wh, 1);
  _ln(c, vp, Offset(s.width - 6, bot), _wh, 1);
  final n = 3 + (st.a * 14).round(), len = (1 - st.b * .85) * .9 / n;
  for (var i = 0; i < n; i++) {
    final u0 = _wr((i + t * 1.4) / n), u1 = math.min(1.0, u0 + len);
    final y0 = hz + (bot - hz) * u0 * u0, y1 = hz + (bot - hz) * u1 * u1;
    _ln(c, Offset(vp.dx, y0), Offset(vp.dx, y1), u0 > .5 ? _bl : _al(_bl, .5), 1 + u0);
  }
  _ln(c, Offset(0, bot), Offset(s.width, bot), _dg, 1);
  c.drawArc(Rect.fromCircle(center: Offset(s.width / 2, s.height + 12), radius: 30), math.pi * 1.15, math.pi * .7, false, _s(_wh, 1));
  _dot(c, Offset(s.width / 2, s.height - 12), 1.5, _rd);
  _lab(c, 'count', const Offset(6, 6), _bl);
  _num(c, _d2(n), const Offset(6, 15), 16, _bl);
  _lab(c, 'gap', Offset(s.width - 6, 6), _gr, ax: 1);
  _num(c, _d2(st.b * 99), Offset(s.width - 6, 15), 16, _gr, ax: 1);
}

void _rCoil(Canvas c, Size s, _S st, double t) {
  final turns = 3 + (st.a * 15).round();
  final len = 30 + st.b * 96 + math.sin(t * 3) * 3 * (1 - st.b);
  const x0 = 14.0, cy = 62.0, r = 9.0;
  final pts = <Offset>[];
  final m = turns * 14;
  for (var i = 0; i <= m; i++) {
    final th = i / 14 * 2 * math.pi;
    pts.add(Offset(x0 + len * i / m + r * .9 * math.sin(th) + r * .9, cy - r * 1.4 * math.cos(th) + r * 1.4 - r * 1.4));
  }
  _ln(c, Offset(x0 - 4, cy - 18), Offset(x0 - 4, cy + 18), _wh, 1);
  for (var i = 0; i < 6; i++) {
    _ln(c, Offset(x0 - 4, cy - 15 + i * 6), Offset(x0 - 9, cy - 11 + i * 6), _dg, 1);
  }
  _ln(c, Offset(x0 - 4, cy), pts.first, _bl, 1);
  _pl(c, pts, _bl, w: 1);
  final end = pts.last + const Offset(6, 0);
  _ln(c, pts.last, end, _bl, 1);
  _dots(c, end + const Offset(0, -16), end + const Offset(0, 16), _rd, gap: 3);
  _dot(c, end + const Offset(0, -16), 1.6, _rd);
  _dot(c, end + const Offset(0, 16), 1.6, _rd);
  _lab(c, 'turns', Offset(s.width / 2, 8), _bl, ax: .5);
  _num(c, _d2(turns), Offset(s.width / 2, 18), 16, _bl, ax: .5);
  _lab(c, 'pitch', Offset(s.width / 2, s.height - 22), _gr, ax: .5);
  _num(c, (len / turns).toStringAsFixed(1), Offset(s.width / 2, s.height - 13), 10, _gr, ax: .5);
}

void _rFall(Canvas c, Size s, _S st, double t) {
  final n = 3 + (st.a * 14).round(), dep = 2.5 + st.b * 5;
  const cols = [_bl, _gr, _wh, _rd, _pu];
  for (var i = n - 1; i >= 0; i--) {
    final o = Offset(10 + i * dep * .9, s.height - 16 - i * dep);
    final pts = <Offset>[];
    for (var k = 0; k <= 40; k++) {
      final x = k / 40;
      final env = math.sin(x * math.pi);
      final y = env * (math.sin(x * 14 - t * 4 + i * .55) * 5 + math.sin(x * 5 + i * .3) * 3);
      pts.add(o + Offset(x * 92, -y));
    }
    c.drawPath(Path()..addPolygon([...pts, pts.last + const Offset(0, 8), pts.first + const Offset(0, 8)], true), _f(_bk));
    _pl(c, pts, i == 0 ? _wh : _al(cols[i % 5], .85), w: 1);
  }
  _lab(c, 'layers', Offset(s.width - 6, 6), _bl, ax: 1);
  _num(c, _d2(n), Offset(s.width - 6, 15), 22, _bl, ax: 1);
  _lab(c, 'depth', const Offset(6, 6), _gr);
}

void _rSteps(Canvas c, Size s, _S st, double t) {
  const x0 = 8.0, y0 = 60.0, cw = 8.75, chh = 12.0;
  final on = List.generate(4, (_) => List.filled(16, false));
  if (st.pts.isEmpty) {
    for (var i = 0; i < 16; i++) {
      on[(i * 3 ~/ 2) % 4][i] = i % 3 != 2;
    }
  } else {
    for (final p in st.pts) {
      final ci = ((p.dx - x0) / cw).floor(), ri = ((p.dy - y0) / chh).floor();
      if (ci >= 0 && ci < 16 && ri >= 0 && ri < 4) on[ri][ci] = true;
    }
  }
  const cols = [_bl, _gr, _wh, _rd];
  final head = (t * 6).floor() % 16;
  var cnt = 0;
  for (var r = 0; r < 4; r++) {
    for (var i = 0; i < 16; i++) {
      final rect = Rect.fromLTWH(x0 + i * cw + 1, y0 + r * chh + 1, cw - 2, chh - 2);
      if (on[r][i]) {
        cnt++;
        c.drawRect(rect, _s(cols[r], 1));
        _dot(c, rect.center, i == head ? 2 : 1, cols[r]);
        final cy = 46 - r * 8.0, cx = x0 + i * cw + cw / 2;
        _pl(c, [Offset(cx - 3, cy + 2.5), Offset(cx, cy - 2.5), Offset(cx + 3, cy + 2.5)], i == head ? cols[r] : _al(cols[r], .45),
            w: 1, close: true);
      } else {
        _dot(c, rect.center, .7, _dg);
      }
    }
  }
  _dots(c, Offset(x0 + head * cw + cw / 2, 14), Offset(x0 + head * cw + cw / 2, y0 + 48), _al(_wh, .5), gap: 3);
  _lab(c, 'clones', const Offset(6, 4), _bl);
  _num(c, _d2(cnt), Offset(s.width - 6, 2), 14, _wh, ax: 1);
}

void _rWheel(Canvas c, Size s, _S st, double t) {
  final cen = Offset(s.width * .42, s.height * .45), r = 38.0;
  final n = 3 + (st.b * 13).round(), rot = st.a * 2 * math.pi + t * .25;
  _ln(c, cen, Offset(cen.dx - 28, s.height - 6), _wh, 1);
  _ln(c, cen, Offset(cen.dx + 28, s.height - 6), _wh, 1);
  _ln(c, Offset(4, s.height - 6), Offset(s.width - 4, s.height - 6), _dg, 1);
  _ring(c, cen, r, _wh, 1);
  _ring(c, cen, r - 4, _al(_wh, .4), 1);
  _ring(c, cen, 4, _wh, 1);
  for (var i = 0; i < n; i++) {
    final a = rot + i * 2 * math.pi / n;
    final p = cen + Offset(math.cos(a), math.sin(a)) * r;
    _ln(c, cen + Offset(math.cos(a), math.sin(a)) * 4, p, _al(_wh, .5), .8);
    _ln(c, p, p + const Offset(0, 4), _bl, 1);
    c.drawArc(Rect.fromCenter(center: p + const Offset(0, 5), width: 9, height: 8), 0, math.pi, true, _s(i == 0 ? _rd : _bl, 1));
  }
  _lab(c, 'cars', Offset(s.width - 6, 8), _bl, ax: 1);
  _num(c, _d2(n), Offset(s.width - 6, 18), 22, _bl, ax: 1);
  _lab(c, 'step', Offset(s.width - 6, 52), _wh, ax: 1);
  _num(c, (360 / n).toStringAsFixed(0), Offset(s.width - 6, 62), 14, _wh, ax: 1);
}

List<Offset> _defWalk(Size s) => [
      for (var i = 0; i <= 20; i++) Offset(14 + (s.width - 28) * i / 20, s.height * (.62 - .28 * math.sin(i / 20 * math.pi * 1.5))),
    ];

void _rFoot(Canvas c, Size s, _S st, double t) {
  final path = st.pts.length > 3 ? st.pts : _defWalk(s);
  final prints = <(Offset, double)>[];
  var rem = 6.0;
  for (var k = 1; k < path.length; k++) {
    final a = path[k - 1], b = path[k], d = (b - a).distance;
    if (d == 0) continue;
    final ang = (b - a).direction;
    var pos = 0.0;
    while (pos + rem <= d) {
      pos += rem;
      prints.add((a + (b - a) * (pos / d), ang));
      rem = 10;
    }
    rem -= d - pos;
  }
  if (st.down) _pl(c, path, _al(_dg, 1), w: 1);
  final shown = st.down ? prints.length : ((t * 6) % (prints.length + 8)).floor();
  for (var i = 0; i < math.min(shown, prints.length); i++) {
    final (p, ang) = prints[i];
    final side = i.isEven ? 1.0 : -1.0;
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(ang);
    c.translate(0, side * 3.5);
    final col = i == math.min(shown, prints.length) - 1 ? _gr : _wh;
    c.drawOval(Rect.fromCenter(center: Offset.zero, width: 6, height: 3.4), _s(col, 1));
    _dot(c, const Offset(4.4, -1.2), .6, col);
    _dot(c, const Offset(4.8, 0), .6, col);
    _dot(c, const Offset(4.4, 1.2), .6, col);
    c.restore();
  }
  _lab(c, 'steps', const Offset(6, 6), _bl);
  _num(c, _d2(prints.length), Offset(s.width - 6, 4), 22, _bl, ax: 1);
}

void _rMoire(Canvas c, Size s, _S st, double t) {
  final cen = s.center(Offset.zero);
  final n = 10 + (st.b * 170).round(), step = st.a * 6;
  for (var i = 0; i < n; i++) {
    final sc = math.pow(.988, i).toDouble();
    c.save();
    c.translate(cen.dx, cen.dy);
    c.rotate((i * step + t * 2) * math.pi / 180);
    c.drawRect(Rect.fromCenter(center: Offset.zero, width: 92 * sc, height: 92 * sc),
        _s(i % 15 == 0 ? _bl : _al(_wh, .28), i % 15 == 0 ? 1 : .7));
    c.restore();
  }
  _num(c, _d3(n), const Offset(5, 4), 12, _bl);
  _num(c, '${step.toStringAsFixed(1)}°', Offset(s.width - 5, s.height - 16), 12, _wh, ax: 1);
}

Offset _fishPath(double u, Size s) => Offset(
      s.width * (.5 + .38 * math.sin(u * 2 * math.pi)),
      s.height * (.52 + .3 * math.sin(u * 4 * math.pi)),
    );

void _rFish(Canvas c, Size s, _S st, double t) {
  final n = 2 + (st.b * 10).round(), lead = st.a + t * .03;
  final dots = [for (var k = 0; k < 80; k++) _fishPath(k / 80, s)];
  c.drawPoints(ui.PointMode.points, dots, _s(_dg, 1));
  for (var i = n - 1; i >= 0; i--) {
    final u = lead - i * .04;
    final p = _fishPath(u, s), q = _fishPath(u + .002, s);
    final col = i == 0 ? _wh : (i == n - 1 ? _rd : _al(_bl, 1 - i / (n + 3)));
    _fish(c, p, 17, (q - p).direction, col, t * 9 + i);
  }
  _lab(c, 'school', const Offset(6, 6), _bl);
  _num(c, _d2(n), const Offset(6, 15), 18, _bl);
}

void _rStair(Canvas c, Size s, _S st, double t) {
  final n = 1 + (st.a * 7).round(), rise = .35 + st.b * 1.1, k = 10.0;
  final o = Offset(10, s.height - 18);
  for (var i = 0; i < n; i++) {
    _isoBox(c, o, 0, -i - 1.0, 0, 1.6, 1, (i + 1) * rise, k, i == n - 1 ? _wh : _al(_bl, .5 + .5 * i / n), 1);
  }
  final u = _wr(t * .18) * n, i = u.floor(), fr = u - i;
  final feet = _iso(o, .8, -i - .5 - fr * .0, (i + 1) * rise, k) + Offset(fr * 0, 0);
  _walker(c, feet, 13, _gr, t * 8);
  final top = _iso(o, 1.6, -n.toDouble(), n * rise, k), lo = _iso(o, 1.6, -n.toDouble(), (n - 1) * rise, k);
  _ln(c, top + const Offset(6, 0), lo + const Offset(6, 0), _gr, 1);
  _lab(c, 'steps', const Offset(6, 6), _bl);
  _num(c, _d2(n), const Offset(6, 15), 18, _bl);
  _lab(c, 'rise', Offset(s.width - 6, s.height - 14), _gr, ax: 1);
}

List<Offset> _defArm() => [for (var i = 0; i <= 16; i++) Offset(6 + i * 2.6, math.sin(i * .7) * 6 + i * .8)];

void _rKaleido(Canvas c, Size s, _S st, double t) {
  final cen = s.center(Offset.zero);
  final n = 3 + (st.a * 9).round();
  final arm = st.pts.length > 2 ? [for (final p in st.pts) p - cen] : _defArm();
  const cols = [_bl, _gr, _wh, _rd];
  for (var k = 0; k < n; k++) {
    c.save();
    c.translate(cen.dx, cen.dy);
    c.rotate(k * 2 * math.pi / n + t * .15);
    _pl(c, arm, k == 0 ? _wh : _al(cols[k % 4], .85), w: 1);
    c.restore();
  }
  _dot(c, cen, 1.4, _wh);
  _lab(c, 'mirrors', const Offset(6, 6), _wh);
  _num(c, _d2(n), Offset(s.width - 6, 4), 18, _wh, ax: 1);
}

void _rGold(Canvas c, Size s, _S st, double t) {
  final cen = Offset(s.width * .4, s.height * .5);
  final ang = 100 + st.a * 80, n = 40 + (st.b * 360).round();
  final a = <Offset>[], b = <Offset>[];
  for (var i = 0; i < n; i++) {
    final th = i * ang * math.pi / 180 + t * .1;
    final p = cen + Offset(math.cos(th), math.sin(th)) * (54 * math.sqrt(i / n));
    (i % 21 == 0 ? b : a).add(p);
  }
  c.drawPoints(ui.PointMode.points, a, _s(_wh, 1.6));
  c.drawPoints(ui.PointMode.points, b, _s(_gr, 2.2));
  final hit = (ang - 137.5).abs() < .6;
  _lab(c, 'angle', Offset(s.width - 6, 8), hit ? _rd : _wh, ax: 1);
  _num(c, ang.toStringAsFixed(1), Offset(s.width - 6, 18), 14, hit ? _rd : _wh, ax: 1);
  _lab(c, 'seeds', Offset(s.width - 6, s.height - 30), _bl, ax: 1);
  _num(c, _d3(n), Offset(s.width - 6, s.height - 20), 14, _bl, ax: 1);
}

void _rRuler(Canvas c, Size s, _S st, double t) {
  final f = (st.a * 12).round(), n = 3 + (st.b * 5).round(), y = 92.0;
  _ln(c, Offset(8, y), Offset(s.width - 8, y), _wh, 1);
  for (var i = 0; i <= 28; i++) {
    final x = 10 + i * 5.0;
    _ln(c, Offset(x, y), Offset(x, y + (i % 5 == 0 ? 6 : 3)), _al(_wh, .6), .8);
  }
  for (var i = 0; i < n; i++) {
    final x = 14 + i * math.max(f, .6) * 5.0;
    if (x > s.width - 6) break;
    final ph = _wr(t * .8 - i * f / 24);
    final hgt = math.sin(ph * math.pi).abs() * 26;
    final bp = Offset(x, y - 10 - hgt);
    _dots(c, Offset(x, y - 8), Offset(x, y), _al(_gr, .7), gap: 2);
    _ring(c, bp, 2.6, i == 0 ? _wh : _gr, 1);
    _pl(c, [Offset(x, y - 4), Offset(x + 2.5, y - 6.5), Offset(x, y - 9), Offset(x - 2.5, y - 6.5)], i == 0 ? _wh : _bl, w: 1,
        close: true);
  }
  _num(c, _d2(f), Offset(s.width - 18, 16), 32, _gr, ax: 1);
  _lab(c, 'f', Offset(s.width - 7, 38), _gr, ax: 1);
  _lab(c, 'offset', Offset(s.width - 7, 4), _gr, ax: 1);
  _lab(c, 'clones $n', const Offset(6, 6), _bl);
}

void _rCity(Canvas c, Size s, _S st, double t) {
  final n = 2 + (st.a * 10).round(), v = st.b, base = s.height - 14;
  final bw = (s.width - 16) / n;
  _ring(c, Offset(s.width - 22, 20), 8, _wh, 1);
  c.drawArc(Rect.fromCircle(center: Offset(s.width - 18, 17), radius: 7), 1.2, 2.6, false, _s(_dg, 1));
  for (var i = 0; i < n; i++) {
    final hh = 18 + (1 - v) * 22 + _rn(i * 7 + 1) * v * 62;
    final r = Rect.fromLTWH(8 + i * bw + 1, base - hh, bw - 2, hh);
    c.drawRect(r, _s(i == n - 1 ? _wh : _al(_bl, .8), 1));
    final wins = <Offset>[], lit = <Offset>[];
    for (var yy = r.top + 5; yy < base - 3; yy += 6) {
      for (var xx = r.left + 3; xx < r.right - 2; xx += 4) {
        final on = _rn((xx * 13 + yy * 7).round() + (t * .7).floor()) > .78;
        (on ? lit : wins).add(Offset(xx, yy));
      }
    }
    c.drawPoints(ui.PointMode.points, wins, _s(_dg, 1));
    c.drawPoints(ui.PointMode.points, lit, _s(_gr, 1.4));
  }
  _ln(c, Offset(4, base), Offset(s.width - 4, base), _wh, 1);
  _lab(c, 'blocks', const Offset(6, 6), _bl);
  _num(c, _d2(n), const Offset(6, 15), 18, _bl);
}

void _rMirror(Canvas c, Size s, _S st, double t) {
  final cen = s.center(Offset.zero);
  final n = 4 + (st.a * 76).round(), sc = .8 + st.b * .19, tw = .02 + math.sin(t * .4) * .03;
  const cols = [_bl, _gr, _wh, _rd];
  var w = s.width - 34, h = s.height - 34;
  for (var i = 0; i < n && w > 1.5; i++) {
    c.save();
    c.translate(cen.dx, cen.dy);
    c.rotate(i * tw);
    c.drawRect(Rect.fromCenter(center: Offset.zero, width: w, height: h), _s(_al(cols[i % 4], .8), .8));
    c.restore();
    w *= sc;
    h *= sc;
  }
  _lab(c, 'count ${_d2(n)}', const Offset(5, 4), _bl);
  _lab(c, 'scale ${sc.toStringAsFixed(2)}', Offset(s.width - 5, s.height - 12), _gr, ax: 1);
}
