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
