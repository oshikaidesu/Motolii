// Opacity / Blend x20. Why you touch it: "let what is behind show through" and "how this layer mixes with what is under it".
// Line art cannot be transparent by fill, so opacity is drawn as how much of the lines behind are hidden (black occlusion).
part of 'op_07.dart';

final _blendSpecs = <_O7Spec>[
  _O7Spec('Ghost town', 'character · effect · drag ↕ · drawing', _ghostTown, b: .7),
  _O7Spec('Engraved numeral', 'typographic · mechanism · drag ↕ · numeral', _engraved, b: .55),
  _O7Spec('Venn modes', 'diagram · mechanism · flick ↔ · diagram', _venn),
  _O7Spec('Iso plates', 'isometric · effect · drag ↕ + pinch · object', _isoPlates, b: .6),
  _O7Spec('Invisible man', 'character · effect · rub · drawing', _invisibleMan),
  _O7Spec('Mixer flow', 'M4L diagram · mechanism · drag ↔ + flick · diagram', _mixFlow, a: .34),
  _O7Spec('Sunglasses', 'object · effect · drag ↕ · drawing', _sunglasses, b: .5),
  _O7Spec('Overhead projector', 'machine · effect · drag ↔ · drawing', _projector, a: .45),
  _O7Spec('Mode jukebox', 'machine · mode · spin · drawing', _jukebox),
  _O7Spec('Foggy window', 'landscape · effect · rub/draw · drawing', _fogWindow),
  _O7Spec('Star charts', 'cosmic · effect · drag ↔ · drawing', _starCharts, a: .6),
  _O7Spec('Wave arithmetic', 'instrument · mechanism · drag ↔ + flick · diagram', _waveMix, a: .5),
  _O7Spec('Tea steeping', 'object · effect · drag ↓ · drawing', _teaCup, b: .55),
  _O7Spec('X-ray hand', 'character · effect · drag ↕ · pushed', _xrayHand, b: .5),
  _O7Spec('Onion-skin monkey', 'animal · effect · drag ↔ · drawing', _onionMonkey, a: .6),
  _O7Spec('Dither matrix', 'M4L grid · mechanism · draw · grid', _ditherGrid, b: .6),
  _O7Spec('Double exposure', 'machine · effect · drag ↔ · drawing', _doubleExposure, a: .55),
  _O7Spec('Mode reel', 'typographic · mode+sample · flick ↔ · word', _modeReel),
  _O7Spec('Cheshire cat', 'animal · effect · drag ↔ · extreme', _cheshire, a: .8),
  _O7Spec('Whale depth', 'animal · effect · drag ↕ · drawing', _whale, b: .55),
];

const _modes = ['NORMAL', 'MULTIPLY', 'SCREEN', 'ADD', 'DIFFERENCE', 'OVERLAY'];
int _modeOf(O7 st, [double per = 3]) => ((st.spin / per).round() % 6 + 6) % 6;

void _ghostTown(Canvas c, Size s, O7 st, double t) {
  const ground = 106.0;
  final sky = <Offset>[const Offset(0, ground)];
  final win = <Offset>[];
  var x = 0.0;
  for (var i = 0; x < s.width; i++) {
    final w = 10 + _h(i) * 14, top = ground - 26 - _h(i + 40) * 48;
    sky
      ..add(Offset(x, top))
      ..add(Offset(x + w, top));
    for (var wy = top + 5; wy < ground - 4; wy += 6) {
      for (var wx = x + 3; wx < x + w - 2; wx += 4) {
        if (_h(i * 31 + wx.toInt() * 7 + wy.toInt()) > .45) win.add(Offset(wx, wy));
      }
    }
    x += w;
  }
  sky.add(Offset(s.width, ground));
  _pl(c, sky, _s(_o(_wt, .7)));
  c.drawPoints(ui.PointMode.points, win, _s(_o(_wt, .45), 1.2));
  c.drawLine(const Offset(0, ground), Offset(s.width, ground), _s(_dg));
  final cx = 70 + math.sin(t * .7) * 16, cy = 50 + math.sin(t * 1.9) * 3;
  final body = Path()
    ..moveTo(cx - 20, cy + 26)
    ..lineTo(cx - 20, cy - 4)
    ..arcToPoint(Offset(cx + 20, cy - 4), radius: const Radius.circular(20))
    ..lineTo(cx + 20, cy + 26);
  for (var i = 0; i < 4; i++) {
    final x0 = cx + 20 - i * 10.0;
    body
      ..quadraticBezierTo(x0 - 2.5, cy + 20 + math.sin(t * 4 + i) * 2, x0 - 5, cy + 24)
      ..quadraticBezierTo(x0 - 7.5, cy + 30, x0 - 10, cy + 26);
  }
  body.close();
  _occ(c, body, st.b);
  c.drawPath(body, _s(_o(_bu, .35 + .65 * st.b), 1.2));
  _dot(c, Offset(cx - 7, cy - 3), _wt, 1.8);
  _dot(c, Offset(cx + 7, cy - 3), _wt, 1.8);
  _num(c, _pct(st.b), const Offset(150, 8), 20, _bu, ax: 1);
  _lab(c, 'OPACITY', const Offset(150, 32), _bu, ax: 1);
}

void _engraved(Canvas c, Size s, O7 st, double t) {
  for (var y = 6.0; y < s.height; y += 5) {
    c.drawLine(Offset(0, y), Offset(s.width, y), _s(_o(_dg, .9), 1));
  }
  final n = 1 + (st.b * 8).round();
  final txt = _pct(st.b);
  final ox = 78 - _numW(txt, 66) / 2 + 4;
  for (var k = n - 1; k >= 0; k--) {
    _num(c, txt, Offset(ox - k * .9, 28 - k * .9), 66, k == 0 ? _wt : _o(_wt, .35 + .5 * st.b), w: 1);
  }
  _lab(c, 'OPACITY', const Offset(8, 8), _bu);
  _lab(c, '$n LINES', const Offset(148, 8), _rd, ax: 1);
}

void _venn(Canvas c, Size s, O7 st, double t) {
  final m = _modeOf(st);
  const a = Offset(62, 52), b = Offset(94, 52);
  const r = 28.0;
  final pa = Path()..addOval(Rect.fromCircle(center: a, radius: r));
  final pb = Path()..addOval(Rect.fromCircle(center: b, radius: r));
  void hatch(bool hz, Color col, double step, Rect box) {
    final p = _s(col, 1);
    if (hz) {
      for (var y = box.top; y < box.bottom; y += step) {
        c.drawLine(Offset(box.left, y), Offset(box.right, y), p);
      }
    } else {
      for (var x = box.left; x < box.right; x += step) {
        c.drawLine(Offset(x, box.top), Offset(x, box.bottom), p);
      }
    }
  }

  const box = Rect.fromLTRB(30, 20, 126, 84);
  c.save();
  c.clipPath(pa);
  hatch(true, _o(_bu, .8), 4, box);
  c.restore();
  c.save();
  c.clipPath(pb);
  hatch(false, _o(_rd, .8), 4, box);
  c.restore();
  c.save();
  c.clipPath(pa);
  c.clipPath(pb);
  c.drawRect(box, _f(_bg));
  switch (m) {
    case 0:
      hatch(false, _rd, 4, box);
    case 1:
      hatch(true, _o(_bu, .3), 4, box);
      hatch(false, _o(_rd, .3), 4, box);
    case 2:
      hatch(true, _o(_wt, .9), 3, box);
      hatch(false, _o(_wt, .6), 3, box);
    case 3:
      hatch(true, _wt, 2, box);
    case 4:
      break;
    case 5:
      hatch(true, _bu, 4, box);
      hatch(false, _rd, 4, box);
  }
  c.restore();
  c.drawPath(pa, _s(_bu));
  c.drawPath(pb, _s(_rd));
  _lab(c, 'A', const Offset(30, 20), _bu);
  _lab(c, 'B', const Offset(122, 20), _rd);
  _lab(c, _modes[m], const Offset(78, 98), _wt, ax: .5, size: 8.5);
  _num(c, '0${m + 1}', const Offset(150, 6), 12, _gn, ax: 1);
  _lab(c, 'MODE', const Offset(6, 6), _gn);
  for (var i = 0; i < 6; i++) {
    _dot(c, Offset(58 + i * 8.0, 112), i == m ? _gn : _dg, i == m ? 1.8 : 1.2);
  }
}

void _isoPlates(Canvas c, Size s, O7 st, double t) {
  const o = Offset(78, 46), k = .85;
  final gap = (14 * st.pinch).clamp(6.0, 22.0);
  final cols = [_o(_wt, .6), _gn, _bu];
  for (var i = 0; i < 3; i++) {
    final z = i * gap;
    final p = Path()
      ..moveTo(_iso(o, 0, 0, z, k).dx, _iso(o, 0, 0, z, k).dy)
      ..lineTo(_iso(o, 60, 0, z, k).dx, _iso(o, 60, 0, z, k).dy)
      ..lineTo(_iso(o, 60, 60, z, k).dx, _iso(o, 60, 60, z, k).dy)
      ..lineTo(_iso(o, 0, 60, z, k).dx, _iso(o, 0, 60, z, k).dy)
      ..close();
    _occ(c, p, i == 2 ? st.b : .92);
    final g = _s(_o(cols[i], i == 2 ? .3 + .7 * st.b : .8), 1);
    for (var u = 10.0; u < 60; u += 10) {
      c.drawLine(_iso(o, u, 0, z, k), _iso(o, u, 60, z, k), g);
      c.drawLine(_iso(o, 0, u, z, k), _iso(o, 60, u, z, k), g);
    }
    c.drawPath(p, _s(cols[i], 1.1));
    if (i > 0) {
      for (final q in const [Offset(0, 60), Offset(60, 60), Offset(60, 0)]) {
        _dash(c, _iso(o, q.dx, q.dy, z - gap, k), _iso(o, q.dx, q.dy, z, k), _s(_dg, 1), on: 1.5, off: 2);
      }
    }
    _lab(c, String.fromCharCode(67 - i), _iso(o, 60, 0, z, k) + const Offset(6, -4), cols[i]);
  }
  _num(c, _pct(st.b), const Offset(8, 92), 18, _bu);
  _lab(c, 'TOP LAYER', const Offset(8, 8), _bu);
}

void _invisibleMan(Canvas c, Size s, O7 st, double t) {
  final op = 1 - st.rub;
  final brick = _s(_dg, 1);
  for (var y = 4.0, r = 0; y < s.height; y += 8, r++) {
    c.drawLine(Offset(0, y), Offset(s.width, y), brick);
    for (var x = r.isEven ? 0.0 : 8.0; x < s.width; x += 16) {
      c.drawLine(Offset(x, y), Offset(x, y + 8), brick);
    }
  }
  final breathe = math.sin(t * 1.6) * 1.2;
  final head = Path()..addRRect(RRect.fromLTRBR(64, 28 + breathe, 92, 60, const Radius.circular(12)));
  final coat = Path()
    ..moveTo(60, 64)
    ..quadraticBezierTo(78, 60 + breathe, 96, 64)
    ..lineTo(108, 120)
    ..lineTo(48, 120)
    ..close();
  _occ(c, coat, op);
  _occ(c, head, op);
  final ln = _s(_o(_wt, op), 1.1);
  c.drawPath(coat, ln);
  c.drawLine(const Offset(70, 63), const Offset(78, 82), ln);
  c.drawLine(const Offset(86, 63), const Offset(78, 82), ln);
  for (var y = 36.0; y < 58; y += 4) {
    c.drawLine(Offset(66, y + breathe), Offset(90, y + breathe + 1.5), _s(_o(_wt, op * .7), 1));
  }
  c.drawPath(head, ln);
  final hat = _s(_wt, 1.2);
  c.drawLine(Offset(54, 28 + breathe), Offset(102, 28 + breathe), hat);
  c.drawRect(Rect.fromLTRB(64, 12 + breathe, 92, 28 + breathe), hat);
  final gl = _s(_bu, 1.2);
  c.drawCircle(Offset(71, 41 + breathe), 5, gl);
  c.drawCircle(Offset(85, 41 + breathe), 5, gl);
  c.drawLine(Offset(76, 41 + breathe), Offset(80, 41 + breathe), gl);
  final n = st.marks.length;
  for (var i = math.max(0, n - 12); i < n; i++) {
    _dot(c, Offset(st.marks[i].dx * s.width, st.marks[i].dy * s.height), _o(_gn, (i - n + 12) / 12), 1.4);
  }
  _num(c, _pct(op), const Offset(150, 8), 18, _gn, ax: 1);
  _lab(c, 'RUB', const Offset(150, 30), _gn, ax: 1);
}

void _mixFlow(Canvas c, Size s, O7 st, double t) {
  final m = _modeOf(st);
  final ln = _s(_o(_wt, .8), 1);
  const ra = Rect.fromLTWH(8, 22, 22, 20), rb = Rect.fromLTWH(8, 74, 22, 20);
  c.drawRect(ra, _s(_bu));
  c.drawRect(rb, _s(_gn));
  _pl(c, [for (var i = 0; i <= 12; i++) Offset(11 + i * 1.33, 32 + math.sin(i * .55 + t * 3) * 5)], _s(_bu, 1));
  _pl(c, const [Offset(11, 88), Offset(15, 88), Offset(15, 80), Offset(21, 80), Offset(21, 88), Offset(27, 88)], _s(_gn, 1));
  final tri = Path()
    ..moveTo(42, 22)
    ..lineTo(66, 32)
    ..lineTo(42, 42)
    ..close();
  c.drawPath(tri, _s(_gn, 1.1));
  _num(c, _pct(st.a), const Offset(44, 28), 9, _gn, w: 1);
  c.drawLine(const Offset(30, 32), const Offset(42, 32), ln);
  c.drawLine(const Offset(66, 32), const Offset(84, 46), ln);
  _pl(c, const [Offset(30, 84), Offset(94, 84), Offset(94, 68)], ln);
  for (var i = 2; i >= 0; i--) {
    final r = Rect.fromLTWH(82 + i * 2.5, 44 - i * 2.5, 26, 24);
    c.drawRect(r, _f(_bg));
    c.drawRect(r, _s(i == 0 ? _wt : _dg, 1));
  }
  _lab(c, ['NRM', 'MUL', 'SCR', 'ADD', 'DIF', 'OVL'][m], const Offset(95, 52), _wt, ax: .5, size: 8);
  c.drawLine(const Offset(108, 56), const Offset(120, 56), ln);
  const w = Offset(134, 56);
  c.drawCircle(w, 13, _s(_wt, 1));
  for (var i = 0; i < 6; i++) {
    final an = -math.pi / 2 + i * math.pi / 3;
    _dot(c, w + Offset(math.cos(an), math.sin(an)) * 8, i == m ? _rd : _dg, i == m ? 2.2 : 1.4);
  }
  final u = (t * .5) % 1;
  final path = [const Offset(30, 32), const Offset(42, 32), const Offset(66, 32), const Offset(84, 46), const Offset(108, 56), const Offset(121, 56)];
  final seg = (u * (path.length - 1)).floor(), f = u * (path.length - 1) - seg;
  _dot(c, Offset.lerp(path[seg], path[seg + 1], f)!, _gn, 1.8);
  _lab(c, 'AMOUNT', const Offset(54, 46), _gn, ax: .5, size: 6.5);
  _lab(c, 'MODE', const Offset(95, 76), _wt, ax: .5, size: 6.5);
  _lab(c, 'OUT', const Offset(134, 76), _rd, ax: .5, size: 6.5);
}

void _sunglasses(Canvas c, Size s, O7 st, double t) {
  const sun = Offset(78, 44);
  c.drawCircle(sun, 13, _s(_wt, 1.1));
  for (var i = 0; i < 18; i++) {
    final an = i * math.pi * 2 / 18 + t * .25;
    final u = Offset(math.cos(an), math.sin(an));
    c.drawLine(sun + u * 18, sun + u * (30 + (i.isEven ? 8 : 0)), _s(_o(_wt, .8), 1));
  }
  final l = Path()..addRRect(RRect.fromLTRBR(36, 54, 74, 82, const Radius.circular(10)));
  final r = Path()..addRRect(RRect.fromLTRBR(82, 54, 120, 82, const Radius.circular(10)));
  for (final lens in [l, r]) {
    _occ(c, lens, st.b);
    c.save();
    c.clipPath(lens);
    final step = 9 - st.b * 6;
    for (var x = 20.0; x < 140; x += step) {
      c.drawLine(Offset(x, 54), Offset(x + 28, 82), _s(_o(_gn, .25 + .5 * st.b), 1));
    }
    c.restore();
    c.drawPath(lens, _s(_bu, 1.3));
  }
  final fr = _s(_bu, 1.3);
  c.drawArc(const Rect.fromLTRB(72, 54, 84, 64), math.pi, math.pi, false, fr);
  c.drawLine(const Offset(36, 60), const Offset(12, 56), fr);
  c.drawLine(const Offset(120, 60), const Offset(144, 56), fr);
  _num(c, _pct(st.b), const Offset(150, 94), 16, _rd, ax: 1);
  _lab(c, 'TINT', const Offset(8, 104), _rd);
}

void _projector(Canvas c, Size s, O7 st, double t) {
  final ln = _s(_wt, 1.1);
  c.drawRect(const Rect.fromLTRB(8, 92, 50, 108), ln);
  c.drawLine(const Offset(14, 92), const Offset(44, 92), _s(_dg, 3));
  c.drawLine(const Offset(42, 92), const Offset(42, 62), ln);
  c.drawRect(const Rect.fromLTRB(32, 54, 50, 64), ln);
  c.drawCircle(const Offset(50, 59), 3, _s(_bu, 1));
  final film = _s(_o(_gn, .3 + .7 * st.a), 1);
  c.drawLine(const Offset(12, 90), const Offset(36, 90), _s(_bu, 1));
  c.drawLine(const Offset(16, 88), const Offset(40, 88), film);
  const scr = Rect.fromLTRB(92, 10, 150, 90);
  final beam = _s(_o(_wt, .25), 1);
  _dash(c, const Offset(52, 59), scr.topLeft, beam);
  _dash(c, const Offset(52, 59), scr.bottomLeft, beam);
  c.drawRect(scr, _s(_o(_wt, .7), 1));
  c.drawLine(const Offset(121, 90), const Offset(121, 110), _s(_dg));
  c.drawLine(const Offset(110, 110), const Offset(132, 110), _s(_dg));
  final wob = math.sin(t * 1.2) * 1.5;
  c.drawCircle(Offset(114, 46 + wob * .3), 15, _s(_bu, 1.2));
  final tri = Path()
    ..moveTo(128 + (st.a - .5) * 14, 26 + wob)
    ..lineTo(146, 70)
    ..lineTo(110 + (st.a - .5) * 14, 70)
    ..close();
  _occ(c, tri, st.a * .9);
  c.drawPath(tri, _s(_o(_gn, .15 + .85 * st.a), 1.2));
  _num(c, _pct(st.a), const Offset(8, 10), 16, _gn);
  _lab(c, 'SLIDE B', const Offset(8, 32), _gn);
}

void _jukebox(Canvas c, Size s, O7 st, double t) {
  final ln = _s(_wt, 1.1);
  final arch = Path()
    ..moveTo(36, 116)
    ..lineTo(36, 48)
    ..arcToPoint(const Offset(120, 48), radius: const Radius.circular(42))
    ..lineTo(120, 116);
  c.drawPath(arch, ln);
  final inner = Path()
    ..moveTo(44, 116)
    ..lineTo(44, 48)
    ..arcToPoint(const Offset(112, 48), radius: const Radius.circular(34));
  c.drawPath(inner, _s(_o(_pu, .7), 1));
  for (var i = 0; i < 7; i++) {
    final u = ((t * .15 + i / 7) % 1) * math.pi;
    _dot(c, const Offset(78, 48) + Offset(-math.cos(u) * 38, -math.sin(u) * 38), _o(_rd, .9), 1.2);
  }
  const step = math.pi * 2 / 6;
  final phi = st.spin * .5;
  final sel = ((-phi / step).round() % 6 + 6) % 6;
  final order = List.generate(6, (k) => k)..sort((x, y) => math.cos(x * step + phi).compareTo(math.cos(y * step + phi)));
  for (final k in order) {
    final an = k * step + phi, dep = math.cos(an);
    if (dep < -.3) continue;
    final y = 66 + math.sin(an) * 16;
    final r = Rect.fromCenter(center: Offset(78, y), width: 50 * (.7 + .3 * dep), height: 7 * (.6 + .4 * dep));
    c.drawOval(r, _f(_bg));
    c.drawOval(r, _s(_o(k == sel ? _gn : _wt, .25 + .75 * ((dep + .3) / 1.3)), 1));
    _dot(c, r.center, _o(k == sel ? _gn : _dg, 1), 1);
  }
  c.drawLine(const Offset(48, 90), const Offset(108, 90), _s(_dg));
  _lab(c, _modes[sel], const Offset(78, 96), _gn, ax: .5, size: 8);
  for (var i = 0; i < 5; i++) {
    c.drawLine(Offset(50 + i * 14.0, 108), Offset(56 + i * 14.0, 108), _s(_dg, 2));
  }
  _lab(c, 'MODE', const Offset(6, 6), _gn);
  _num(c, '0${sel + 1}', const Offset(150, 6), 12, _gn, ax: 1);
}

void _fogWindow(Canvas c, Size s, O7 st, double t) {
  const fr = Rect.fromLTRB(12, 8, 144, 112);
  final cleared = Path();
  final hole = <int>{};
  final wipe = st.marks.isNotEmpty
      ? st.marks
      : [for (var i = 0; i <= 14; i++) Offset(.3 + i * .03, .62 - math.sin(i / 14 * math.pi) * .3)];
  for (final m in wipe) {
    final q = Offset(m.dx * s.width, m.dy * s.height);
    cleared.addOval(Rect.fromCircle(center: q, radius: 10));
    final gx = (q.dx / 6).round(), gy = (q.dy / 6).round();
    for (var dx = -2; dx <= 2; dx++) {
      for (var dy = -2; dy <= 2; dy++) {
        if (dx * dx + dy * dy <= 3) hole.add((gx + dx) * 100 + gy + dy);
      }
    }
  }
  void scene(double a) {
    final mtn = <Offset>[const Offset(12, 92)];
    for (var i = 0; i <= 12; i++) {
      mtn.add(Offset(12 + i * 11.0, 92 - (i.isEven ? 10 + _h(i) * 30 : 4 + _h(i) * 10)));
    }
    _pl(c, mtn, _s(_o(_bu, a), 1.1));
    c.drawCircle(const Offset(108, 34), 8, _s(_o(_rd, a), 1.1));
    for (var i = 0; i < 4; i++) {
      c.drawLine(Offset(20, 100 + i * 4.0), Offset(140, 100 + i * 4.0), _s(_o(_gn, a * .7), 1));
    }
  }

  scene(.18);
  c.save();
  c.clipPath(cleared);
  scene(1);
  c.restore();
  final fog = <Offset>[];
  var total = 0;
  for (var gx = 3; gx < 24; gx++) {
    for (var gy = 2; gy < 19; gy++) {
      total++;
      if (!hole.contains(gx * 100 + gy)) fog.add(Offset(gx * 6.0 + (gy.isEven ? 0 : 3), gy * 6.0));
    }
  }
  c.drawPoints(ui.PointMode.points, fog, _s(const Color(0xFF6A6A72), 1.3));
  final ln = _s(_wt, 1.2);
  c.drawRect(fr, ln);
  c.drawLine(const Offset(78, 8), const Offset(78, 112), ln);
  c.drawLine(const Offset(12, 60), const Offset(144, 60), ln);
  _num(c, _pct(1 - fog.length / total), const Offset(150, 2), 10, _gn, ax: 1, w: 1);
}

void _starCharts(Canvas c, Size s, O7 st, double t) {
  final field = [for (var i = 0; i < 40; i++) Offset(_h(i) * s.width, _h(i + 99) * s.height)];
  c.drawPoints(ui.PointMode.points, field, _s(_dg, 1.2));
  const ctr = Offset(78, 58);
  final a = [for (var i = 0; i < 7; i++) ctr + Offset(-46 + i * 15.0, math.sin(i * 1.7) * 22)];
  final rot = t * .12;
  final b = [
    for (var i = 0; i < 7; i++)
      () {
        final q = Offset(-40 + _h(i + 7) * 80, -30 + _h(i + 17) * 60);
        return ctr + Offset(q.dx * math.cos(rot) - q.dy * math.sin(rot), q.dx * math.sin(rot) + q.dy * math.cos(rot));
      }()
  ];
  _pl(c, a, _s(_o(_bu, .8), 1));
  for (final q in a) {
    _dot(c, q, _bu, 1.8);
  }
  _pl(c, b, _s(_o(_gn, .8 * st.a), 1));
  for (final q in b) {
    _dot(c, q, _o(_gn, st.a), 1.8);
  }
  for (final p in a) {
    for (final q in b) {
      final d = (p - q).distance;
      if (d < 9) {
        final g = (1 - d / 9) * st.a;
        c.drawCircle(Offset.lerp(p, q, .5)!, 3 + 4 * g, _s(_o(_wt, g), 1));
        for (var k = 0; k < 4; k++) {
          final u = Offset(math.cos(k * math.pi / 2), math.sin(k * math.pi / 2));
          c.drawLine(Offset.lerp(p, q, .5)! + u * 4, Offset.lerp(p, q, .5)! + u * (8 + 6 * g), _s(_o(_wt, g), 1));
        }
      }
    }
  }
  _lab(c, 'SKY A', const Offset(6, 6), _bu);
  _lab(c, 'SKY B', const Offset(6, 106), _gn);
  _num(c, _pct(st.a), const Offset(150, 100), 14, _gn, ax: 1);
}

void _waveMix(Canvas c, Size s, O7 st, double t) {
  final m = ((st.spin / 3).round() % 4 + 4) % 4;
  const names = ['MIX', 'ADD', 'MULT', 'DIFF'];
  double fa(double x) => math.sin(x * .09 + t * 2);
  double fb(double x) {
    final v = math.sin(x * .05 - t * 1.3) * 5;
    return v / (1 + v.abs());
  }

  double out(double x) {
    final a = fa(x), b = fb(x), k = st.a;
    return switch (m) {
      0 => a * (1 - k) + b * k,
      1 => (a + k * b) / (1 + k),
      2 => a * (1 - k + k * b),
      _ => (a - k * b).abs() - .5,
    };
  }

  final ga = <Offset>[], gb = <Offset>[], go = <Offset>[];
  for (var x = 26.0; x <= 148; x += 2) {
    ga.add(Offset(x, 18 + fa(x) * 7));
    gb.add(Offset(x, 42 + fb(x) * 7));
    go.add(Offset(x, 88 + out(x) * 16));
  }
  for (final y in [18.0, 42.0, 88.0]) {
    _dash(c, Offset(26, y), Offset(148, y), _s(_dg, 1), on: 1, off: 3);
  }
  _pl(c, ga, _s(_bu, 1.1));
  _pl(c, gb, _s(_gn, 1.1));
  _pl(c, go, _s(_wt, 1.3));
  _lab(c, 'A', const Offset(8, 14), _bu);
  _lab(c, 'B', const Offset(8, 38), _gn);
  _lab(c, 'OUT', const Offset(8, 84), _wt);
  _lab(c, names[m], const Offset(8, 60), _rd, size: 8);
  _num(c, _pct(st.a), const Offset(150, 56), 12, _rd, ax: 1);
}

void _teaCup(Canvas c, Size s, O7 st, double t) {
  final k = 1 - st.b;
  final ln = _s(_wt, 1.1);
  final cup = Path()
    ..moveTo(36, 60)
    ..lineTo(44, 96)
    ..quadraticBezierTo(46, 104, 56, 104)
    ..lineTo(88, 104)
    ..quadraticBezierTo(98, 104, 100, 96)
    ..lineTo(108, 60);
  c.save();
  c.clipPath(Path()
    ..addPath(cup, Offset.zero)
    ..close());
  final step = 9 - k * 6.5;
  for (var y = 70.0; y < 106; y += step) {
    c.drawLine(Offset(30, y), Offset(114, y + math.sin(t + y) * .8), _s(_o(_rd, .2 + .8 * k), 1));
  }
  final by = 30 + k * 56;
  for (var i = 0; i < 6; i++) {
    final r = ((t * 8 + i * 7) % 26) + 4, an = i * 1.05 + t * .3;
    if (by > 66) _dot(c, Offset(72, by) + Offset(math.cos(an), math.sin(an) * .5) * r, _o(_rd, (1 - r / 30) * k), 1);
  }
  c.restore();
  c.drawPath(cup, ln);
  c.drawOval(const Rect.fromLTRB(36, 56, 108, 66), ln);
  c.drawArc(const Rect.fromLTRB(100, 66, 120, 90), -math.pi / 2, math.pi, false, ln);
  c.drawOval(const Rect.fromLTRB(22, 102, 122, 114), _s(_o(_wt, .6), 1));
  _pl(c, [const Offset(124, 12), Offset(124, 22), Offset(78, by - 8)], _s(_o(_wt, .7), 1));
  c.drawRect(const Rect.fromLTRB(118, 10, 132, 20), _s(_gn, 1));
  final bag = RRect.fromLTRBR(65, by - 8, 79, by + 8, const Radius.circular(2));
  c.drawRRect(bag, _f(_bg));
  c.drawRRect(bag, _s(_gn, 1.1));
  for (var y = by - 4; y < by + 6; y += 3) {
    _dot(c, Offset(72, y), _gn, .7);
  }
  for (var i = 0; i < 3; i++) {
    _pl(c, [for (var j = 0; j <= 10; j++) Offset(56 + i * 14 + math.sin(j * .7 - t * 3 + i) * 2.5, 52 - j * 3.6)], _s(_o(_wt, .25 + .35 * k), 1));
  }
  _num(c, _pct(k), const Offset(8, 8), 18, _rd);
  _lab(c, 'STRENGTH', const Offset(8, 30), _rd);
}

void _xrayHand(Canvas c, Size s, O7 st, double t) {
  final skinOp = st.b;
  final bone = _s(_o(_gn, .4 + .6 * (1 - skinOp)), 1.3);
  const xs = [58.0, 70.0, 82.0, 94.0], lens = [34.0, 42.0, 40.0, 30.0];
  for (var i = 0; i < 4; i++) {
    final top = 64 - lens[i];
    for (var j = 0; j < 3; j++) {
      final y0 = top + 4 + j * (lens[i] - 6) / 3, y1 = y0 + (lens[i] - 6) / 3 - 2;
      c.drawLine(Offset(xs[i], y0), Offset(xs[i], y1), bone);
      _dot(c, Offset(xs[i], y1 + 1), _o(_gn, .5 + .5 * (1 - skinOp)), 1);
    }
    c.drawLine(Offset(xs[i], 68), Offset(76 + (xs[i] - 76) * .4, 100), bone);
  }
  c.drawLine(const Offset(50, 90), const Offset(40, 76), bone);
  c.drawLine(const Offset(39, 74), const Offset(32, 64), bone);
  c.drawLine(const Offset(62, 104), const Offset(92, 104), bone);
  final skin = Path();
  for (var i = 0; i < 4; i++) {
    skin.addRRect(RRect.fromLTRBR(xs[i] - 5.5, 64 - lens[i], xs[i] + 5.5, 72, const Radius.circular(5.5)));
  }
  skin.addRRect(RRect.fromLTRBR(52, 64, 100, 116, const Radius.circular(10)));
  const ta = Offset(56, 96), tb = Offset(32, 62);
  final u = (tb - ta) / (tb - ta).distance, n = Offset(-u.dy, u.dx) * 5.5;
  skin.addPath(
      Path()
        ..moveTo((ta + n).dx, (ta + n).dy)
        ..lineTo((tb + n).dx, (tb + n).dy)
        ..arcToPoint(tb - n, radius: const Radius.circular(5.5))
        ..lineTo((ta - n).dx, (ta - n).dy)
        ..close(),
      Offset.zero);
  _occ(c, skin, skinOp * .95);
  c.drawPath(skin, _s(_o(_wt, .2 + .8 * skinOp), 1.1));
  final sy = 14 + (t * 34) % 96;
  c.drawLine(Offset(20, sy), Offset(136, sy), _s(_o(_rd, .8), 1));
  _lab(c, '180 KV', const Offset(150, 6), _rd, ax: 1);
  _num(c, _pct(skinOp), const Offset(8, 8), 16, _wt);
  _lab(c, 'SKIN', const Offset(8, 28), _wt);
}

void _onionMonkey(Canvas c, Size s, O7 st, double t) {
  c.drawLine(const Offset(0, 102), Offset(s.width, 102), _s(_dg));
  void monkey(double x, double ph, Color col) {
    final p = _s(col, 1.1);
    const hy = 56.0;
    final bob = math.sin(ph * 12).abs() * 3;
    final head = Offset(x, hy - bob);
    c.drawCircle(head, 6, p);
    c.drawCircle(head + const Offset(-7, -1), 2.5, p);
    c.drawCircle(head + const Offset(7, -1), 2.5, p);
    c.drawArc(Rect.fromCenter(center: head + const Offset(0, 2), width: 7, height: 5), 0, math.pi, false, p);
    final hip = Offset(x - 2, 82 - bob);
    c.drawLine(head + const Offset(0, 6), hip, p);
    final sw = math.sin(ph * 12);
    final sh = head + const Offset(0, 11);
    c.drawLine(sh, sh + Offset(10 * sw, 8), p);
    c.drawLine(sh, sh + Offset(-10 * sw, 8), p);
    _pl(c, [hip, hip + Offset(6 * sw, 9), Offset(x - 2 + 10 * sw, 101)], p);
    _pl(c, [hip, hip + Offset(-6 * sw, 9), Offset(x - 2 - 10 * sw, 101)], p);
    final tail = Path()
      ..moveTo(hip.dx, hip.dy)
      ..quadraticBezierTo(hip.dx - 16, hip.dy + 2, hip.dx - 12, hip.dy - 12)
      ..quadraticBezierTo(hip.dx - 9, hip.dy - 18, hip.dx - 5, hip.dy - 13);
    c.drawPath(tail, p);
  }

  final x0 = 24 + (t * 26) % 130;
  for (var k = 5; k >= 0; k--) {
    final x = x0 - k * 13;
    if (x < 6) continue;
    final al = k == 0 ? 1.0 : math.pow(st.a, k).toDouble();
    if (al < .03) continue;
    monkey(x, t - k * .09, k == 0 ? _wt : _o(k.isEven ? _pu : _bu, al));
  }
  _num(c, _pct(st.a), const Offset(150, 8), 18, _bu, ax: 1);
  _lab(c, 'ONION', const Offset(150, 30), _bu, ax: 1);
}

const _bayer = [0, 8, 2, 10, 12, 4, 14, 6, 3, 11, 1, 9, 15, 7, 13, 5];

void _ditherGrid(Canvas c, Size s, O7 st, double t) {
  c.drawCircle(Offset(78 + math.sin(t * .6) * 6, 56), 34, _s(_wt, 1.2));
  c.drawLine(const Offset(14, 100), const Offset(142, 14), _s(_rd, 1.2));
  const ox = 13.0, oy = 10.0, cs = 10.0;
  final painted = <int>{};
  for (final m in st.marks) {
    final gx = ((m.dx * s.width - ox) / cs).floor(), gy = ((m.dy * s.height - oy) / cs).floor();
    painted.add(gx * 100 + gy);
  }
  final dots = <Offset>[];
  var sum = 0.0;
  for (var gx = 0; gx < 13; gx++) {
    for (var gy = 0; gy < 9; gy++) {
      final v = painted.contains(gx * 100 + gy) ? 1.0 : (gx / 12) * st.b;
      sum += v;
      final r = Rect.fromLTWH(ox + gx * cs, oy + gy * cs, cs, cs);
      if (v > 0) c.drawRect(r, _f(_o(_bg, v)));
      for (var i = 0; i < 16; i++) {
        if (v * 16 > _bayer[i] + .5) dots.add(r.topLeft + Offset(1.6 + (i % 4) * 2.3, 1.6 + (i ~/ 4) * 2.3));
      }
    }
  }
  c.drawPoints(ui.PointMode.points, dots, _s(_gn, 1));
  final g = _s(_o(_dg, .8), 1);
  for (var gx = 0; gx <= 13; gx++) {
    c.drawLine(Offset(ox + gx * cs, oy), Offset(ox + gx * cs, oy + 90), g);
  }
  for (var gy = 0; gy <= 9; gy++) {
    c.drawLine(Offset(ox, oy + gy * cs), Offset(ox + 130, oy + gy * cs), g);
  }
  _lab(c, 'DRAW', const Offset(13, 104), _gn);
  _num(c, _pct(sum / 117), const Offset(143, 103), 10, _gn, ax: 1, w: 1);
}

void _doubleExposure(Canvas c, Size s, O7 st, double t) {
  final sky = <Offset>[const Offset(10, 104)];
  var x = 10.0;
  for (var i = 0; x < 146; i++) {
    final w = 8 + _h(i + 3) * 10, top = 104 - 14 - _h(i + 60) * 40;
    sky
      ..add(Offset(x, top))
      ..add(Offset(x + w, top));
    x += w;
  }
  sky.add(const Offset(146, 104));
  _pl(c, sky, _s(_o(_bu, 1 - st.a * .4), 1.1));
  const prof = [
    Offset(40, 10), Offset(52, 14), Offset(58, 30), Offset(56, 38), Offset(64, 48), Offset(58, 52), Offset(60, 58),
    Offset(56, 62), Offset(58, 68), Offset(52, 72), Offset(50, 80), Offset(38, 86), Offset(32, 100),
  ];
  const back = [Offset(40, 10), Offset(20, 16), Offset(10, 36), Offset(12, 60), Offset(20, 78), Offset(22, 100)];
  final drift = math.sin(t * .5) * 2;
  Offset m(Offset p) => Offset(p.dx * 1.0 + 34 + drift, p.dy * 1.0 + 4);
  final pp = _s(_o(_gn, st.a), 1.2);
  _pl(c, prof.map(m).toList(), pp);
  _pl(c, back.map(m).toList(), pp);
  final cm = _s(_wt, 1.2);
  for (final cn in const [Offset(6, 6), Offset(150, 6), Offset(6, 114), Offset(150, 114)]) {
    final sx = cn.dx < 78 ? 1.0 : -1.0, sy = cn.dy < 60 ? 1.0 : -1.0;
    _pl(c, [cn + Offset(0, 8 * sy), cn, cn + Offset(8 * sx, 0)], cm);
  }
  if ((t * 1.5) % 1 < .6) _dot(c, const Offset(16, 16), _rd, 2.2);
  _lab(c, 'EXP 2', const Offset(22, 13), _rd);
  _num(c, _pct(st.a), const Offset(146, 96), 12, _wt, ax: 1);
}

void _modeReel(Canvas c, Size s, O7 st, double t) {
  const bm = [BlendMode.srcOver, BlendMode.multiply, BlendMode.screen, BlendMode.plus, BlendMode.difference, BlendMode.overlay];
  final off = -st.spin * .35;
  final base = off.floor(), frac = off - base;
  for (var k = -3; k <= 3; k++) {
    final y = 54 + (k - frac) * 17;
    if (y < 8 || y > 104) continue;
    final idx = ((base + k) % 6 + 6) % 6;
    final near = (y - 54).abs() < 8.5;
    _lab(c, _modes[idx], Offset(10, y), near ? _wt : (y - 54).abs() < 26 ? const Color(0xFF6A6A72) : _dg,
        size: near ? 11 : 9, w: FontWeight.w300);
  }
  final sel = ((off.round()) % 6 + 6) % 6;
  final br = _s(_gn, 1.1);
  _pl(c, const [Offset(6, 50), Offset(4, 50), Offset(4, 68), Offset(6, 68)], br);
  _pl(c, const [Offset(92, 50), Offset(94, 50), Offset(94, 68), Offset(92, 68)], br);
  c.saveLayer(const Rect.fromLTRB(100, 20, 156, 100), Paint());
  for (var y = 37.0; y < 62; y += 2) {
    c.drawLine(Offset(104, y), Offset(130, y), _s(_bu, 1));
  }
  final pb = _s(_rd, 1.4)..blendMode = bm[sel];
  for (var x = 119.0; x < 144; x += 2) {
    c.drawLine(Offset(x, 50), Offset(x, 76), pb);
  }
  c.restore();
  c.drawRect(const Rect.fromLTWH(104, 36, 26, 26), _s(_o(_wt, .5), 1));
  c.drawRect(const Rect.fromLTWH(118, 50, 26, 26), _s(_o(_wt, .5), 1));
  _lab(c, 'A', const Offset(104, 26), _bu);
  _lab(c, 'B', const Offset(140, 80), _rd);
  _num(c, '0${sel + 1}', const Offset(150, 104), 10, _gn, ax: 1, w: 1);
}

void _cheshire(Canvas c, Size s, O7 st, double t) {
  final a = st.a;
  final body = _ss(.6, .9, a), head = _ss(.35, .65, a), eyes = _ss(.12, .4, a);
  final branch = Path()
    ..moveTo(0, 100)
    ..quadraticBezierTo(78, 88, 156, 96);
  c.drawPath(branch, _s(_o(_wt, .7), 1.1));
  for (final x in [24.0, 120.0]) {
    c.drawLine(Offset(x, 96), Offset(x + 8, 88), _s(_o(_gn, .7), 1));
  }
  final bob = math.sin(t * 1.4) * 2 * (1 - a);
  if (body > 0) {
    final bp = _s(_o(_pu, body), 1.1);
    c.drawOval(const Rect.fromLTRB(64, 64, 96, 94), bp);
    final tail = <Offset>[for (var i = 0; i <= 14; i++) Offset(96 + i * 2.2, 90 + i * 1.4 + math.sin(i * .6 + t * 2) * 3)];
    _pl(c, tail, bp);
    for (var i = 2; i < 14; i += 3) {
      c.drawLine(tail[i] + const Offset(0, -3), tail[i] + const Offset(0, 3), bp);
    }
  }
  const hc = Offset(80, 50);
  if (head > 0) {
    final hp = _s(_o(_pu, head), 1.1);
    c.drawOval(Rect.fromCenter(center: hc, width: 46, height: 36), hp);
    _pl(c, [hc + const Offset(-20, -10), hc + const Offset(-18, -28), hc + const Offset(-6, -17)], hp);
    _pl(c, [hc + const Offset(20, -10), hc + const Offset(18, -28), hc + const Offset(6, -17)], hp);
    for (final sx in [-1.0, 1.0]) {
      for (var i = 0; i < 3; i++) {
        c.drawLine(hc + Offset(sx * 16, 4.0 + i * 3), hc + Offset(sx * 34, 0.0 + i * 5), _s(_o(_wt, head * .7), 1));
      }
    }
  }
  if (eyes > 0) {
    final ep = _s(_o(_gn, eyes), 1.1);
    for (final sx in [-1.0, 1.0]) {
      final e = hc + Offset(sx * 9, -5);
      c.drawOval(Rect.fromCenter(center: e, width: 9, height: 7), ep);
      c.drawLine(e + const Offset(0, -3), e + const Offset(0, 3), ep);
    }
  }
  final g = hc + Offset(0, 6 + bob);
  final grin = Path()
    ..moveTo(g.dx - 17, g.dy - 2)
    ..quadraticBezierTo(g.dx, g.dy + 16, g.dx + 17, g.dy - 2)
    ..quadraticBezierTo(g.dx, g.dy + 5, g.dx - 17, g.dy - 2);
  c.drawPath(grin, _s(_rd, 1.2));
  for (var i = -3; i <= 3; i++) {
    final x = g.dx + i * 4.0;
    final y0 = g.dy + 1.5 + (1 - (i / 4) * (i / 4)) * 3, y1 = y0 + 2.5 * (1 - (i / 4).abs());
    c.drawLine(Offset(x, y0), Offset(x, y1 + 1), _s(_rd, 1));
  }
  _num(c, _pct(a), const Offset(8, 8), 18, _bu);
  _lab(c, 'OPACITY', const Offset(8, 30), _bu);
}

void _whale(Canvas c, Size s, O7 st, double t) {
  final d = 1 - st.b;
  const surf = 34.0;
  final wave = [for (var x = 0.0; x <= s.width; x += 4) Offset(x, surf + math.sin(x * .12 + t * 2) * 1.5)];
  for (var i = 0; i < 3; i++) {
    final bx = 30 + i * 22 + math.sin(t * .8 + i) * 4, by = 12 + i * 3.0;
    _pl(c, [Offset(bx - 4, by - 2), Offset(bx, by), Offset(bx + 4, by - 2)], _s(_o(_wt, .5), 1));
  }
  final y = 48 + d * 50, x0 = 34 + math.sin(t * .4) * 6;
  final body = Path()
    ..moveTo(x0, y)
    ..quadraticBezierTo(x0 + 34, y - 20, x0 + 72, y - 8)
    ..quadraticBezierTo(x0 + 86, y, x0 + 72, y + 9)
    ..quadraticBezierTo(x0 + 34, y + 14, x0, y)
    ..moveTo(x0, y)
    ..lineTo(x0 - 12, y - 9)
    ..moveTo(x0, y)
    ..lineTo(x0 - 11, y + 7);
  final wp = _s(_wt, 1.2);
  c.drawPath(body, wp);
  for (var i = 0; i < 4; i++) {
    c.drawLine(Offset(x0 + 50 + i * 4.0, y + 5), Offset(x0 + 56 + i * 4.0, y + 8), _s(_wt, .9));
  }
  _dot(c, Offset(x0 + 66, y - 2), _wt, 1.2);
  for (var wy = surf + 6; wy < y + 14; wy += 7) {
    if (wy > y - 18) {
      c.drawRect(Rect.fromLTWH(0, wy, s.width, 7), _f(_o(_bg, .16)));
    }
    _dash(c, Offset(0, wy), Offset(s.width, wy), _s(_o(_bu, .45), 1), on: 3, off: 4);
  }
  _pl(c, wave, _s(_bu, 1.2));
  if (d < .15) {
    for (var i = 0; i < 9; i++) {
      final u = (t * 1.6 + i / 9) % 1, an = -math.pi / 2 + (i - 4) * .16;
      _dot(c, Offset(x0 + 62, surf) + Offset(math.cos(an), math.sin(an)) * u * 26, _o(_gn, 1 - u), 1.1);
    }
  }
  _num(c, _pct(st.b), const Offset(150, 6), 16, _gn, ax: 1);
  _lab(c, 'SURFACE', const Offset(108, 24), _gn);
}
