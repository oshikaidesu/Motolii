// Fill / Colour / Gradient: hue (blue encoder), saturation (green), stops (white), accent (red).
// Colour appears only as stroked lines and small dots, never as a filled shape.
part of 'op_08.dart';

List<_Op> _fillPanels() => const [
      _Op('Prism aim', 'diagram · effect · drag aim hue', _G.drag, _fPrism, a: .6, b: .85, wrap: true),
      _Op('Chameleon', 'animal · effect · rub hue', _G.rub, _fChameleon, a: .8, b: .3, wrapB: true),
      _Op('Dusk road', 'vehicle · landscape · drag stop', _G.drag, _fDusk, a: .03, b: .5, wrap: true),
      _Op('Lit block', 'isometric · effect · spin hue', _G.spin, _fBlock, a: .55, b: .7),
      _Op('Hue degrees', 'typographic · numeral led · flick', _G.flick, _fBigHue, a: .62, b: .8, wrap: true),
      _Op('Juicer', 'machine · mechanism · pull sat', _G.drag, _fJuice, a: .1, b: .5, wrap: true, inv: true),
      _Op('Parrot tail', 'animal · effect · flick span', _G.flick, _fParrot, a: .0, b: .5, wrap: true),
      _Op('Stop rings', 'diagram · mechanism · out stops', _G.spin, _fRings, a: .6, b: .4),
      _Op('Thermal scan', 'instrument · effect · flick map', _G.flick, _fThermal, a: .62, b: .5, wrap: true),
      _Op('Loud colour', 'sound · mechanism · pinch sat', _G.pinch, _fSatWave, a: .9, b: .5),
      _Op('Neon FILL', 'typographic · effect · rub heat', _G.rub, _fNeon, a: .92, b: .4),
      _Op('Metro stops', 'diagram · vehicle · drag stop B', _G.drag, _fMetro, a: .5, b: .6, wrapB: true),
      _Op('Window paint', 'landscape · effect · draw lights', _G.draw, _fCity, a: .5, tap: .1),
      _Op('Pencil hatch', 'material · effect · rub density', _G.rub, _fHatch, a: .58, b: .35, wrap: true),
      _Op('Drop ripples', 'nature · effect · draw drops', _G.draw, _fDrops, a: .5, keep: true, tap: .07),
      _Op('Punch power', 'character · pushed · flick punch', _G.flick, _fBoxer, a: .97, b: .8, wrap: true),
      _Op('Mix flow', 'M4L diagram · mechanism · drag mix', _G.drag, _fMix, a: .1, b: .5, wrap: true),
      _Op('Contour map', 'map · effect · pinch bands', _G.pinch, _fContour, a: .4, b: .5),
      _Op('Hue keys', 'instrument · play · drag keys', _G.drag, _fPiano, a: .4, b: .8),
      _Op('Supernova', 'cosmic · extreme · spin hue', _G.spin, _fNova, a: .1, b: .3),
    ];

void _fPrism(Canvas c, Size s, _S st, double t) {
  final hue = st.a, sat = st.b;
  const p0 = Offset(50, 86), p1 = Offset(70, 46), p2 = Offset(90, 86);
  final entry = Offset.lerp(p0, p1, .55)!, exit = Offset.lerp(p1, p2, .5)!;
  const src = Offset(4, 74);
  _ln(c, src, entry, _wh, 1.2);
  final along = <Offset>[];
  for (var i = 0; i < 6; i++) {
    along.add(Offset.lerp(src, entry, _wr(i / 6 + t * .6))!);
  }
  c.drawPoints(ui.PointMode.points, along, _s(_bk, 2));
  _pl(c, [p0, p1, p2], _wh, w: 1.2, close: true);
  const k = 14;
  final sel = (hue * k).floor() % k;
  for (var i = 0; i < k; i++) {
    final end = Offset(s.width - 6, 38 + i * 5.6);
    final on = i == sel;
    _ln(c, exit, end, _al(_hc(i / k, sat), on ? 1 : .35), on ? 1.6 : .8);
    if (on) _ring(c, end + const Offset(-3, 0), 2.4, _hc(i / k, sat), 1);
  }
  _lab(c, 'hue', const Offset(6, 6), _bl);
  _num(c, _d3(hue * 360), const Offset(6, 15), 22, _bl);
  _lab(c, 'sat ${_d2(sat * 99)}', Offset(s.width - 6, 6), _gr, ax: 1);
}

void _fChameleon(Canvas c, Size s, _S st, double t) {
  final col = _hc(st.b, st.a);
  _chameleon(c, const Offset(4, 18), 120, _wh, col, t * 1.3, st.down ? (math.sin(t * 14) * .5 + .5) * .15 : 0);
  _lab(c, 'hue', Offset(s.width - 6, 6), _bl, ax: 1);
  _num(c, _d3(st.b * 360), Offset(s.width - 6, 15), 18, _bl, ax: 1);
  _lab(c, 'sat ${_d2(st.a * 99)}', const Offset(6, 6), _gr);
}

void _fDusk(Canvas c, Size s, _S st, double t) {
  final hz = 28 + (1 - st.b) * 46, h0 = st.a, h1 = st.a + .58;
  for (var y = 4.0; y < hz; y += 3) {
    final v = 1 - (y - 4) / (hz - 4);
    _ln(c, Offset(14, y), Offset(s.width - 4, y), _al(_hc(h0 + (h1 - h0) * v, .85), .55 + .45 * (1 - v)), 1);
  }
  c.drawArc(Rect.fromCircle(center: Offset(s.width * .7, hz), radius: 9 + math.sin(t) * .5), math.pi, math.pi, false,
      _s(_hc(h0, .9), 1.2));
  final sky = Path()
    ..addPolygon([
      Offset(14, hz), Offset(14, hz - 8), Offset(24, hz - 8), Offset(24, hz - 15), Offset(30, hz - 15), Offset(30, hz - 6),
      Offset(44, hz - 6), Offset(44, hz - 11), Offset(52, hz - 11), Offset(52, hz),
    ], true);
  c.drawPath(sky, _f(_bk));
  c.drawPath(sky, _s(_wh, 1));
  _ln(c, Offset(14, hz), Offset(s.width - 4, hz), _wh, 1);
  final vp = Offset(s.width * .58, hz), bot = s.height - 18;
  _ln(c, vp, Offset(22, bot), _wh, 1);
  _ln(c, vp, Offset(s.width - 4, bot), _wh, 1);
  for (var i = 0; i < 5; i++) {
    final u0 = _wr(i / 5 + t * .3), u1 = math.min(1.0, u0 + .08);
    _ln(c, Offset(vp.dx, hz + (bot - hz) * u0 * u0), Offset(vp.dx, hz + (bot - hz) * u1 * u1), _al(_wh, .7), 1);
  }
  c.drawArc(Rect.fromCircle(center: Offset(s.width * .58, s.height + 14), radius: 30), math.pi * 1.15, math.pi * .7, false,
      _s(_wh, 1));
  _pl(c, [Offset(4, hz - 3), Offset(10, hz), Offset(4, hz + 3)], _gr, w: 1, close: true);
  _pl(c, const [Offset(4, 1), Offset(10, 4), Offset(4, 7)], _bl, w: 1, close: true);
  _lab(c, 'stop', Offset(4, hz + 6), _gr, size: 7);
  _lab(c, 'hue ${_d3(h0 * 360)}', Offset(s.width - 6, s.height - 12), _bl, ax: 1);
}

void _fBlock(Canvas c, Size s, _S st, double t) {
  final hue = st.a, sat = st.b, k = 30.0;
  final o = Offset(s.width * .46, 34);
  Offset p(double x, double y, double z) => _iso(o, x, y, z, k);
  void hatch(Offset a0, Offset a1, Offset b0, Offset b1, Color col, int n) {
    for (var i = 1; i < n; i++) {
      _ln(c, Offset.lerp(a0, a1, i / n)!, Offset.lerp(b0, b1, i / n)!, col, 1);
    }
  }

  hatch(p(0, 0, 1), p(1, 0, 1), p(0, 1, 1), p(1, 1, 1), _hc(hue, sat, 1), 12);
  hatch(p(0, 1, 1), p(0, 1, 0), p(1, 1, 1), p(1, 1, 0), _hc(hue, sat, .72), 10);
  hatch(p(1, 0, 1), p(1, 1, 1), p(1, 0, 0), p(1, 1, 0), _hc(hue, sat, .45), 8);
  _isoBox(c, o, 0, 0, 0, 1, 1, 1, k, _wh, 1);
  final sun = Offset(18 + math.sin(t * .5) * 2, 20);
  _ring(c, sun, 5, _wh, 1);
  for (var i = 0; i < 8; i++) {
    final a = i * math.pi / 4 + t * .3;
    _ln(c, sun + Offset(math.cos(a), math.sin(a)) * 7.5, sun + Offset(math.cos(a), math.sin(a)) * 10, _wh, 1);
  }
  _dots(c, sun + const Offset(9, 6), p(.5, .5, 1), _al(_wh, .35), gap: 4);
  _lab(c, 'hue ${_d3(hue * 360)}', Offset(s.width - 6, 6), _bl, ax: 1);
  _lab(c, 'sat ${_d2(sat * 99)}', Offset(s.width - 6, s.height - 13), _gr, ax: 1);
}

void _fBigHue(Canvas c, Size s, _S st, double t) {
  final hue = st.a, sat = st.b, col = _hc(hue, sat);
  _num(c, _d3(hue * 360), const Offset(6, 22), 60, col);
  _lab(c, 'hue', const Offset(8, 8), _bl);
  final wc = Offset(s.width - 22, s.height - 22);
  for (var i = 0; i < 24; i++) {
    final a = -math.pi / 2 + i / 24 * 2 * math.pi;
    _dot(c, wc + Offset(math.cos(a), math.sin(a)) * 14, 1.3, _hc(i / 24, sat));
  }
  final a = -math.pi / 2 + hue * 2 * math.pi;
  _ring(c, wc + Offset(math.cos(a), math.sin(a)) * 14, 3.4, _wh, 1);
  _lab(c, 'sat ${_d2(sat * 99)}', Offset(8, s.height - 14), _gr);
  final wave = [for (var i = 0; i <= 30; i++) Offset(8 + i * 2.6, s.height - 26 + math.sin(i * .5 - t * 3) * 2)];
  _pl(c, wave, _al(col, .7), w: 1);
}

void _fJuice(Canvas c, Size s, _S st, double t) {
  final sat = st.b, col = _hc(st.a, sat), cx = s.width * .42;
  final press = sat * 10;
  c.drawArc(Rect.fromCenter(center: Offset(cx, 22 + press), width: 44, height: 30), math.pi, math.pi, false, _s(_wh, 1));
  _ln(c, Offset(cx - 22, 22 + press), Offset(cx + 22, 22 + press), _wh, 1);
  for (var i = 1; i < 6; i++) {
    final a = math.pi + i * math.pi / 6;
    _ln(c, Offset(cx, 22 + press), Offset(cx, 22 + press) + Offset(math.cos(a) * 20, math.sin(a) * 13), col, 1);
  }
  _pl(c, [Offset(cx - 26, 36), Offset(cx, 52), Offset(cx + 26, 36)], _wh, w: 1);
  for (var i = 0; i < 5; i++) {
    _ln(c, Offset(cx - 18 + i * 9, 39 + (i - 2).abs() * -1.5), Offset(cx - 6 + i * 3, 46), _dg, 1);
  }
  final drops = 1 + (sat * 5).round();
  for (var i = 0; i < drops; i++) {
    final u = _wr(t * .9 + i / drops);
    _dot(c, Offset(cx + (i - drops / 2) * .8, 54 + u * 30), 1.3, col);
  }
  final g = Rect.fromLTWH(cx - 16, 70, 32, 40);
  _pl(c, [g.topLeft, Offset(g.left + 3, g.bottom), Offset(g.right - 3, g.bottom), g.topRight], _wh, w: 1);
  final lvl = 4 + sat * 30;
  for (var y = g.bottom - 3; y > g.bottom - lvl; y -= 3) {
    final f = (g.bottom - y) / 40 * 3;
    _ln(c, Offset(g.left + 4 - f, y), Offset(g.right - 4 + f, y), col, 1);
  }
  _lab(c, 'squeeze', Offset(s.width - 6, 6), _gr, ax: 1);
  _num(c, _d2(sat * 99), Offset(s.width - 6, 16), 26, _gr, ax: 1);
  _lab(c, 'hue ${_d3(st.a * 360)}', Offset(s.width - 6, s.height - 13), _bl, ax: 1);
}

void _fParrot(Canvas c, Size s, _S st, double t) {
  final h0 = st.a, h1 = st.a + .1 + st.b * .7;
  _ln(c, Offset(20, 6 + .6 * 92 + 5), Offset(s.width - 8, 6 + .6 * 92 + 5), _dg, 1.4);
  _parrot(c, const Offset(18, 4), 92, _wh, h0, h1, .9, math.sin(t * 1.5) + st.va.clamp(-1.0, 1.0) * 3);
  _dot(c, const Offset(10, 12), 2, _hc(h0));
  _lab(c, 'from', const Offset(16, 9), _bl, size: 7);
  _dot(c, const Offset(10, 26), 2, _hc(h1));
  _lab(c, 'to', const Offset(16, 23), _gr, size: 7);
  _num(c, _d3(h0 * 360), Offset(s.width - 6, 6), 16, _bl, ax: 1);
  _num(c, _d3(_wr(h1) * 360), Offset(s.width - 6, 26), 16, _gr, ax: 1);
}

void _fRings(Canvas c, Size s, _S st, double t) {
  final cen = Offset(s.width * .4, s.height * .52);
  final n = 2 + (st.b * 7).round();
  c.drawRect(Rect.fromCenter(center: cen, width: 104, height: 104), _s(_dg, 1));
  for (var k = 0; k < 9; k++) {
    final u = k / 8, seg = (u * (n - 1)).floor().clamp(0, n - 2), f = u * (n - 1) - seg;
    final hue = st.a + (seg + f) * .12;
    _ring(c, cen, 5 + k * 5.6 + math.sin(t * 1.5 - k * .5) * .8, _hc(hue, .85), 1);
  }
  for (var j = 0; j < n; j++) {
    final r = 5 + (j / (n - 1)) * 8 * 5.6;
    _dot(c, cen + Offset(r, 0), 1.6, _wh);
  }
  _ln(c, Offset(cen.dx - 52, cen.dy), cen, _rd, 1);
  _dot(c, Offset(cen.dx - 52, cen.dy), 1.8, _rd);
  _lab(c, 'stops', Offset(s.width - 6, 8), _wh, ax: 1);
  _num(c, _d2(n), Offset(s.width - 6, 18), 22, _wh, ax: 1);
  _lab(c, 'hue', Offset(s.width - 6, 64), _bl, ax: 1);
  _num(c, _d3(st.a * 360), Offset(s.width - 6, 74), 14, _bl, ax: 1);
}

void _fThermal(Canvas c, Size s, _S st, double t) {
  final base = s.height - 10, con = .2 + st.b * .8, scroll = st.a * 300 + t * 6;
  double hgt(double x) {
    final u = (x + scroll) * .045;
    return 18 + 22 * (.5 + .5 * math.sin(u)) * (.6 + .4 * math.sin(u * .37 + 1)) + 12 * math.sin(u * 2.3) * .5;
  }

  final line = <Offset>[];
  var peak = Offset.zero;
  for (var x = 6.0; x <= s.width - 6; x += 3) {
    final h = hgt(x), y = base - h;
    final v = ((h - 10) / 50).clamp(0.0, 1.0);
    final hue = .66 - ((v - .5) * con + .5) * .66;
    _ln(c, Offset(x, base), Offset(x, y), _al(_hc(hue, .9), .85), 1);
    line.add(Offset(x, y));
    if (y < peak.dy || peak == Offset.zero) peak = Offset(x, y);
  }
  _pl(c, line, _wh, w: 1);
  _dots(c, Offset(peak.dx, peak.dy - 4), Offset(peak.dx, 16), _rd, gap: 3);
  _dot(c, peak, 2, _rd);
  _lab(c, 'contrast ${_d2(con * 99)}', const Offset(6, 6), _gr);
}

void _fSatWave(Canvas c, Size s, _S st, double t) {
  final sat = st.b, col = _hc(st.a, sat), cy = s.height * .58, amp = 2 + sat * 28;
  _dots(c, Offset(6, cy), Offset(s.width - 6, cy), _dg, gap: 3);
  final pts = [for (var i = 0; i <= 70; i++) Offset(6 + i * 2.06, cy + math.sin(i * .28 - t * 4) * amp * math.sin(i / 70 * math.pi))];
  _pl(c, pts, col, w: 1.3);
  _num(c, _d2(sat * 99), const Offset(6, 4), 30, _gr);
  _lab(c, 'sat %', const Offset(48, 8), _gr);
  _lab(c, 'hue ${_d3(st.a * 360)}', Offset(s.width - 6, s.height - 12), _bl, ax: 1);
}

const _glyphs = <String, List<List<Offset>>>{
  'F': [
    [Offset(0, 1), Offset(0, 0), Offset(.7, 0)],
    [Offset(0, .5), Offset(.55, .5)],
  ],
  'I': [
    [Offset(0, 0), Offset(0, 1)],
  ],
  'L': [
    [Offset(0, 0), Offset(0, 1), Offset(.7, 1)],
  ],
};

void _fNeon(Canvas c, Size s, _S st, double t) {
  final heat = st.b, col = _hc(st.a, .3 + heat * .7);
  final flick = _rn((t * 9).floor()) < .12 * (1 - heat) ? .25 : 1.0;
  var x = 16.0;
  const y = 26.0, h = 46.0;
  for (final ch in 'FILL'.split('')) {
    for (final stroke in _glyphs[ch]!) {
      final pts = [for (final p in stroke) Offset(x + p.dx * h * .7, y + p.dy * h)];
      _pl(c, pts, _al(col, .07 * heat * flick), w: 9);
      _pl(c, pts, _al(col, .16 * heat * flick), w: 4.5);
      _pl(c, pts, _al(col, flick), w: 1.4);
    }
    x += ch == 'I' ? 14 : 34;
  }
  _dot(c, const Offset(12, 20), 1, _dg);
  _dot(c, Offset(x - 4, 20), 1, _dg);
  final cable = [for (var i = 0; i <= 12; i++) Offset(x - 8 + i * 2.2, y + h + 4 + math.sin(i * .9) * 2 + i * 1.6)];
  _pl(c, cable, _dg, w: 1);
  final plug = cable.last;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: plug + const Offset(5, 0), width: 10, height: 6), const Radius.circular(3)),
      _s(_wh, 1));
  _lab(c, 'heat', const Offset(6, 6), _gr);
  _num(c, _d2(heat * 99), Offset(s.width - 6, 4), 18, _gr, ax: 1);
}

void _fMetro(Canvas c, Size s, _S st, double t) {
  const y = 64.0, x0 = 12.0, x2 = 144.0;
  final x1 = 30 + st.a * 84, h = st.b;
  Color at(double x) => x < x1
      ? _hc(h + (x - x0) / (x1 - x0) * .25, .85)
      : _hc(h + .25 + (x - x1) / (x2 - x1) * .3, .85);
  for (var x = x0; x < x2; x += 2) {
    _ln(c, Offset(x, y), Offset(x + 2, y), at(x + 1), 2);
  }
  _dots(c, const Offset(x0, y - 22), const Offset(x2, y - 22), _dg, gap: 4);
  _ln(c, Offset(x1, y), Offset(x1 + 18, y + 26), _dg, 1);
  for (final (x, n) in [(x0, 'a'), (x1, 'b'), (x2, 'c')]) {
    c.drawCircle(Offset(x, y), 4, _f(_bk));
    _ring(c, Offset(x, y), 4, x == x1 ? _wh : _al(_wh, .7), 1.2);
    _lab(c, n, Offset(x, y + 10), x == x1 ? _wh : _al(_wh, .6), ax: .5);
  }
  final u = _wr(t * .12), tx = x0 + (x2 - x0) * u;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(tx, y - 9), width: 16, height: 7), const Radius.circular(2)),
      _s(at(tx), 1));
  _lab(c, 'stop b', const Offset(6, 6), _wh);
  _num(c, '${_d2((x1 - x0) / (x2 - x0) * 100)}%', Offset(s.width - 6, 4), 18, _wh, ax: 1);
  _lab(c, 'hue ${_d3(h * 360)}', Offset(s.width - 6, s.height - 13), _bl, ax: 1);
}

void _fCity(Canvas c, Size s, _S st, double t) {
  final base = s.height - 10;
  final stroke = st.pts.length > 1
      ? st.pts
      : [for (var i = 0; i <= 12; i++) Offset(14 + i * 10.6, base - 16 - i * 4.5 - math.sin(i * .8) * 6)];
  const hs = <double>[40, 66, 52, 84, 46, 72, 58];
  final lit = <Color, List<Offset>>{};
  final off = <Offset>[];
  for (var b = 0; b < 7; b++) {
    final r = Rect.fromLTWH(8 + b * 20.0, base - hs[b], 18, hs[b]);
    c.drawRect(r, _s(_wh, 1));
    for (var y = r.top + 5; y < base - 3; y += 6) {
      for (var x = r.left + 4; x < r.right - 2; x += 5) {
        final p = Offset(x, y);
        var best = -1;
        var bd = 81.0;
        for (var k = 0; k < stroke.length; k++) {
          final d = (stroke[k] - p).distanceSquared;
          if (d < bd) {
            bd = d;
            best = k;
          }
        }
        if (best < 0) {
          off.add(p);
        } else {
          final col = _hc(st.a + best / stroke.length * .6, .85);
          (lit[col] ??= []).add(p);
        }
      }
    }
  }
  c.drawPoints(ui.PointMode.points, off, _s(_dg, 1.4));
  for (final e in lit.entries) {
    c.drawPoints(ui.PointMode.points, e.value, _s(e.key, 2.2));
  }
  if (st.down) _pl(c, stroke, _al(_wh, .3), w: 1);
  _ln(c, Offset(4, base), Offset(s.width - 4, base), _wh, 1);
  final tw = _wr(t * .4);
  _dot(c, Offset(s.width * tw, 10 + math.sin(t) * 2), 1, _wh);
  _lab(c, 'from ${_d3(st.a * 360)}', const Offset(6, 6), _bl);
}

void _fHatch(Canvas c, Size s, _S st, double t) {
  final sat = st.b, col = _hc(st.a, math.max(.05, sat));
  final r = Rect.fromLTWH(12, 22, 74, 74);
  c.drawRect(r, _s(_wh, 1));
  final n = 3 + (sat * 28).round();
  c.save();
  c.clipRect(r.deflate(1));
  for (var i = 0; i < n; i++) {
    final d = -r.height + i * (r.width + r.height) / n;
    _ln(c, Offset(r.left + d, r.bottom), Offset(r.left + d + r.height, r.top), col, 1);
  }
  if (sat > .6) {
    final m = ((sat - .6) / .4 * n).round();
    for (var i = 0; i < m; i++) {
      final d = i * (r.width + r.height) / n;
      _ln(c, Offset(r.left + d - r.height, r.top), Offset(r.left + d, r.bottom), col, 1);
    }
  }
  c.restore();
  final tip = Offset(r.right - 6 + math.sin(t * 6) * 6, r.bottom - 10 + math.cos(t * 3) * 4);
  _pl(c, [tip, tip + const Offset(7, -5), tip + const Offset(30, -36), tip + const Offset(36, -31), tip + const Offset(13, 0)], _wh,
      w: 1, close: true);
  _ln(c, tip + const Offset(7, -5), tip + const Offset(13, 0), _wh, 1);
  _dot(c, tip, 1.2, col);
  _lab(c, 'sat', Offset(s.width - 6, s.height - 36), _gr, ax: 1);
  _num(c, _d2(sat * 99), Offset(s.width - 6, s.height - 28), 22, _gr, ax: 1);
  _lab(c, 'hue ${_d3(st.a * 360)}', const Offset(12, 8), _bl);
}

void _fDrops(Canvas c, Size s, _S st, double t) {
  final drops = <Offset>[];
  if (st.pts.isEmpty) {
    drops.addAll(const [Offset(40, 50), Offset(100, 70), Offset(70, 86)]);
  } else {
    for (var i = 0; i < st.pts.length; i += 8) {
      drops.add(st.pts[i]);
    }
  }
  final from = math.max(0, drops.length - 14);
  for (var i = from; i < drops.length; i++) {
    final hue = st.a + i * .06;
    for (var k = 0; k < 3; k++) {
      final r = (t * 14 + i * 9 + k * 13) % 40;
      _ring(c, drops[i], r, _al(_hc(hue, .85), 1 - r / 40), 1);
    }
    _dot(c, drops[i], 1.4, _hc(hue, .85));
  }
  _lab(c, 'drops ${_d2(drops.length)}', const Offset(6, 6), _wh);
  _lab(c, 'hue ${_d3(st.a * 360)}', Offset(s.width - 6, 6), _bl, ax: 1);
}

void _fBoxer(Canvas c, Size s, _S st, double t) {
  final jab = math.pow(math.max(0.0, math.sin(t * 2.2)), 12).toDouble() * .55;
  final pw = math.max(jab, (st.va.abs() / 1.6).clamp(0.0, 1.0)), col = _hc(st.a, st.b);
  _boxer(c, const Offset(30, 28), 7.5, _wh, col, pw, math.sin(t * 5) * 1.2);
  final sw = math.sin(t * 6) * pw * .35;
  const top = Offset(124, 0);
  c.save();
  c.translate(top.dx, top.dy);
  c.rotate(sw);
  _dots(c, Offset.zero, const Offset(0, 22), _dg, gap: 2.5);
  final bag = RRect.fromRectAndRadius(const Rect.fromLTWH(-11, 22, 22, 56), const Radius.circular(9));
  c.drawRRect(bag, _s(_wh, 1));
  for (var y = 30.0; y < 74; y += 3.4) {
    _ln(c, Offset(-8, y), Offset(8, y), _al(col, .9), 1);
  }
  c.restore();
  if (pw > .3) {
    for (var k = 0; k < 4; k++) {
      final a = math.pi * (.8 + k * .13);
      _ln(c, const Offset(112, 50) + Offset(math.cos(a), math.sin(a)) * 8, const Offset(112, 50) + Offset(math.cos(a), math.sin(a)) * 14,
          _rd, 1);
    }
  }
  _lab(c, 'power', const Offset(6, 6), _rd);
  _num(c, _d2(pw * 99), Offset(6, s.height - 30), 24, _rd);
  _lab(c, 'sat ${_d2(st.b * 99)}', Offset(70, s.height - 12), _gr);
}

void _fMix(Canvas c, Size s, _S st, double t) {
  final ca = _hc(st.a, .85), cb = _hc(st.a + .5, .85), mix = st.b;
  final out = Color.lerp(ca, cb, mix)!;
  void hbox(Rect r, Color col) {
    c.drawRect(r, _s(_wh, 1));
    for (var y = r.top + 4; y < r.bottom - 1; y += 3) {
      _ln(c, Offset(r.left + 4, y), Offset(r.right - 4, y), col, 1);
    }
  }

  hbox(const Rect.fromLTWH(6, 16, 26, 26), ca);
  hbox(const Rect.fromLTWH(6, 66, 26, 26), cb);
  _pl(c, const [Offset(32, 29), Offset(44, 29), Offset(52, 52)], _dg, w: 1);
  _pl(c, const [Offset(32, 79), Offset(44, 79), Offset(52, 60)], _dg, w: 1);
  _pl(c, const [Offset(52, 40), Offset(80, 56), Offset(52, 72)], _gr, w: 1.2, close: true);
  _num(c, _d2(mix * 99), const Offset(54, 50), 11, _gr);
  _ln(c, const Offset(80, 56), const Offset(108, 56), _dg, 1);
  const oc = Offset(128, 56);
  _ring(c, oc, 17, _wh, 1);
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: oc, radius: 16)));
  for (var y = oc.dy - 16; y < oc.dy + 16; y += 3) {
    _ln(c, Offset(oc.dx - 16, y), Offset(oc.dx + 16, y), out, 1);
  }
  c.restore();
  final u = _wr(t * .7);
  _dot(c, Offset(80 + u * 28, 56), 1.5, out);
  _dot(c, Offset(32 + u * 12, 29), 1.3, ca);
  _dot(c, Offset(32 + u * 12, 79), 1.3, cb);
  _lab(c, 'a', const Offset(19, 6), _bl, ax: .5);
  _lab(c, 'b', const Offset(19, 96), _al(_wh, .7), ax: .5);
  _lab(c, 'mix', const Offset(62, 80), _gr, ax: .5);
  _lab(c, 'out', const Offset(128, 80), _wh, ax: .5);
  _lab(c, 'hue a ${_d3(st.a * 360)}', Offset(s.width - 6, 6), _bl, ax: 1);
}

void _fContour(Canvas c, Size s, _S st, double t) {
  final cen = Offset(s.width * .44, s.height * .52);
  final n = 3 + (st.b * 9).round();
  for (var k = n - 1; k >= 0; k--) {
    final r0 = 6 + (k + 1) * (50 / n);
    final pts = <Offset>[];
    for (var i = 0; i < 48; i++) {
      final th = i / 48 * 2 * math.pi;
      final r = r0 * (1 + .14 * math.sin(3 * th + k * .7 + t * .25) + .07 * math.sin(5 * th - k));
      pts.add(cen + Offset(math.cos(th) * r * 1.25, math.sin(th) * r * .85));
    }
    _pl(c, pts, _hc(st.a + (1 - k / math.max(1, n - 1)) * .4, .85), w: 1, close: true);
    if (k == n - 1) _lab(c, '${(n - k) * 100}', pts[0] + const Offset(2, -4), _al(_wh, .5), size: 6);
  }
  _pl(c, [cen + const Offset(-4, 3), cen + const Offset(0, -4), cen + const Offset(4, 3)], _wh, w: 1, close: true);
  _dot(c, cen + const Offset(0, -6), 1.3, _rd);
  _lab(c, 'bands', const Offset(6, 6), _wh);
  _num(c, _d2(n), const Offset(6, 15), 16, _wh);
  _lab(c, 'hue ${_d3(st.a * 360)}', Offset(s.width - 6, s.height - 12), _bl, ax: 1);
}

void _fPiano(Canvas c, Size s, _S st, double t) {
  const k = 14, x0 = 8.0, top = 64.0, kw = 10.0;
  final sel = (st.a * (k - 1)).round(), sat = st.b;
  for (var i = 0; i < k; i++) {
    final r = Rect.fromLTWH(x0 + i * kw, top + (i == sel ? 2 : 0), kw, 46);
    c.drawRect(r, _s(i == sel ? _hc(i / k, sat) : _al(_wh, .7), i == sel ? 1.3 : 1));
    _dot(c, Offset(r.center.dx, r.bottom - 6), i == sel ? 2 : 1.2, _hc(i / k, sat));
  }
  for (var i = 0; i < k - 1; i++) {
    if (i % 7 == 2 || i % 7 == 6) continue;
    final r = Rect.fromLTWH(x0 + (i + 1) * kw - 3, top, 6, 26);
    c.drawRect(r, _f(_bk));
    c.drawRect(r, _s(_al(_wh, .7), 1));
  }
  final col = _hc(sel / k, sat);
  final wave = [for (var i = 0; i <= 60; i++) Offset(8 + i * 2.33, 34 + math.sin(i * (.2 + sel * .03) - t * 5) * 10)];
  _pl(c, wave, col, w: 1.2);
  _lab(c, 'hue', const Offset(6, 6), _bl);
  _num(c, _d3(sel / k * 360), Offset(s.width - 6, 4), 14, _bl, ax: 1);
  _lab(c, 'sat ${_d2(sat * 99)}', const Offset(40, 6), _gr);
}

void _fNova(Canvas c, Size s, _S st, double t) {
  final cen = s.center(Offset.zero);
  final n = 2 + (st.b * 38).round();
  const rays = 220;
  for (var i = 0; i < rays; i++) {
    final a = i / rays * 2 * math.pi + t * .05;
    final l = 14 + _rn(i) * 60 + math.sin(t * 2 + i) * 3;
    final hue = st.a + (i % n) / n * .8;
    final d = Offset(math.cos(a), math.sin(a));
    _ln(c, cen + d * (6 + _rn(i + 99) * 6), cen + d * l, _al(_hc(hue, .9), .75), .8);
  }
  _ring(c, cen, 5, _wh, 1);
  _dot(c, cen, 1.5, _wh);
  _lab(c, 'stops', const Offset(6, 6), _wh);
  _num(c, _d2(n), Offset(s.width - 6, 4), 14, _wh, ax: 1);
}
