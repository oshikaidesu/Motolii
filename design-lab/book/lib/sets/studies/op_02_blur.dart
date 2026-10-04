part of 'op_02.dart';

/// Monoline stand-in for a directional blur: the line drawn n times spread along [d] both ways, each copy faint.
void _smear(Canvas c, List<Offset> pts, Offset d, Color col, {int n = 7, double w = 1, bool close = false}) {
  if (d.distance < .4) {
    _pl(c, pts, col, w: w, close: close);
    return;
  }
  final a = (2.2 / n).clamp(.14, 1.0);
  for (var i = 0; i < n; i++) {
    final o = d * (i / (n - 1) * 2 - 1);
    _pl(c, [for (final q in pts) q + o], _al(col, a), w: w, close: close);
  }
}

/// Isotropic version: copies on a ring of radius r around the sharp line.
void _haze(Canvas c, List<Offset> pts, double r, Color col, {int n = 8, double w = 1, bool close = false}) {
  if (r < .4) {
    _pl(c, pts, col, w: w, close: close);
    return;
  }
  _pl(c, pts, _al(col, (1 - r / 12).clamp(.15, 1.0)), w: w, close: close);
  for (var i = 0; i < n; i++) {
    final o = _pol(i / n * 2 * math.pi + r, r);
    _pl(c, [for (final q in pts) q + o], _al(col, (2 / n).clamp(.12, .5)), w: w, close: close);
  }
}

List<Offset> _house(Offset o, double s) =>
    [o + Offset(-s, 0), o + Offset(-s, -s), o + Offset(0, -s * 1.7), o + Offset(s, -s), o + Offset(s, 0)];
List<Offset> _pine(Offset o, double s) => [
      o,
      o + Offset(0, -s * .3),
      o + Offset(-s * .5, -s * .3),
      o + Offset(0, -s * 1.4),
      o + Offset(s * .5, -s * .3),
      o + Offset(0, -s * .3),
    ];
List<Offset> _carShape(Offset o) => [
      o + const Offset(-18, 0),
      o + const Offset(-18, -7),
      o + const Offset(-9, -8),
      o + const Offset(-4, -15),
      o + const Offset(8, -15),
      o + const Offset(13, -8),
      o + const Offset(18, -6),
      o + const Offset(18, 0),
    ];

final _blur = <_Pn>[
  // 1. Lens iris: pinch it open; the f-number falls and the lights behind turn into big soft discs.
  _Pn('iris', 'machine·mech+fx·pinch·num·sane', (c, s, t) {
    const o = Offset(50, 62);
    final ro = 5 + s.z * 30, rot = s.z * 1.2;
    _ci(c, o, 42, _cw);
    final hex = [for (var i = 0; i < 6; i++) o + _pol(rot + i * math.pi / 3, ro)];
    _pl(c, hex, _cb, close: true);
    for (var i = 0; i < 6; i++) {
      final v = hex[i], dir = _pol(rot + i * math.pi / 3 + math.pi / 2 + .5, 1);
      var e = v;
      for (var k = 0; k < 60 && (e - o).distance < 42; k++) {
        e += dir;
      }
      _ln(c, v, e, _cd, 1);
    }
    final f = 1.4 * math.pow(2, (1 - s.z) * 3.5);
    _num(c, f.toStringAsFixed(1), const Offset(152, 4), 26, _cb, ax: 1);
    _lab(c, 'F-STOP', const Offset(152, 32), _cb, ax: 1);
    for (var i = 0; i < 3; i++) {
      final q = Offset(112 + i * 14.0, 82 + (i.isEven ? 0 : 10));
      _ci(c, q, .8 + s.z * 7, _cg, .9);
      _dt(c, q, .9, _cg);
    }
    _lab(c, 'BOKEH', const Offset(130, 106), _cg, ax: .5);
  }, z: .45),

  // 2. Glasses fogging up: rub (the steam) and the lenses mist over; the eyes behind fade.
  _Pn('fogged glasses', 'character·fx:sight·rub·pic·whimsy', (c, s, t) {
    const hd = Offset(60, 58);
    final fog = _cl(.1 + s.rub);
    _ci(c, hd, 30, _cw);
    _ln(c, hd + const Offset(-2, 24), hd + const Offset(6, 18), _cw, .9);
    for (final dx in [-12.0, 12.0]) {
      final lc = hd + Offset(dx, -2);
      _ci(c, lc, 9, _cb, 1.2);
      c.save();
      c.clipPath(Path()..addOval(Rect.fromCircle(center: lc, radius: 8.4)));
      final lines = (fog * 14).round();
      for (var i = 0; i < lines; i++) {
        final y = lc.dy - 8 + (i + .5) * 16 / math.max(1, lines);
        _ln(c, Offset(lc.dx - 9, y + math.sin(t * 2 + i) * .6), Offset(lc.dx + 9, y), _al(_cw, .5), .7);
      }
      _dt(c, lc + Offset(math.sin(t) * 2, 0), 1.6, _al(_cw, 1 - fog));
      c.restore();
    }
    _ln(c, hd + const Offset(-3, -2), hd + const Offset(3, -2), _cb);
    _rc(c, const Rect.fromLTWH(108, 84, 22, 20), _cw);
    c.drawArc(const Rect.fromLTWH(126, 88, 10, 10), -math.pi / 2, math.pi, false, _sp(_cw));
    for (var i = 0; i < 3; i++) {
      _pl(c, [for (var k = 0; k < 8; k++) Offset(113 + i * 6 + math.sin(t * 3 + k * .9 + i) * 2, 80 - k * 5.0)], _al(_cg, .4 + fog * .6), w: .8);
    }
    _num(c, _n2(fog * 99), const Offset(152, 2), 22, _cg, ax: 1);
    _lab(c, 'STEAM', const Offset(152, 26), _cg, ax: 1);
  }),

  // 3. Eye chart: drag the radius; the small rows go first, the red tick marks the last readable line.
  _Pn('eye chart', 'typographic·fx:legibility·drag·type·sane', (c, s, t) {
    final r = s.a * 6;
    const rows = ['E', 'FP', 'TOZ', 'LPED', 'PECFD', 'EDFCZP'];
    var y = 6.0, lastOk = -1;
    for (var i = 0; i < rows.length; i++) {
      final sz = 21 - i * 3.2;
      final ok = r < sz * .22;
      if (ok) lastOk = i;
      final n = r < .4 ? 1 : 6;
      for (var k = 0; k < n; k++) {
        final o = n == 1 ? Offset.zero : _pol(k / n * 2 * math.pi, r);
        _tx(c, rows[i].split('').join(' '), Offset(70, y) + o, sz, _al(_cw, n == 1 ? 1 : .3), wght: 300, ax: .5);
      }
      y += sz + 3;
    }
    var yy = 6.0;
    for (var i = 0; i < lastOk; i++) {
      yy += 21 - i * 3.2 + 3;
    }
    if (lastOk >= 0) _ln(c, Offset(130, yy + 2), Offset(138, yy + 2), _cr, 1.5);
    _num(c, r.toStringAsFixed(1), const Offset(152, 92), 18, _cb, ax: 1);
    _lab(c, 'RADIUS', const Offset(152, 110), _cb, ax: 1);
  }, a: .4),

  // 4. Speeding car: drag sets which way it moves and how far it smears.
  _Pn('speed smear', 'vehicle·fx·drag·pic·sane', (c, s, t) {
    final ang = (s.a - .5) * math.pi, len = 1 + s.b * 22, d = _pol(ang, len);
    const o = Offset(78, 70);
    _smear(c, _carShape(o), d, _cw, n: 9);
    for (final wx in [-10.0, 10.0]) {
      final wp = [for (var i = 0; i < 12; i++) o + Offset(wx, 0) + _pol(i / 12 * 2 * math.pi, 4)];
      _smear(c, wp, d, _cw, n: 9, close: true);
    }
    for (var i = 0; i < 5; i++) {
      final q = o + Offset(-30 - _h(i) * 30, -16 + i * 6.0) + Offset((t * 60 * (1 + _h(i))) % 20, 0);
      _ln(c, q, q - d * 1.2, _cd, .8);
    }
    _ln(c, const Offset(6, 104), const Offset(150, 104), _cd);
    final arrow = const Offset(130, 22);
    _ci(c, arrow, 12, _cd, .8);
    _ln(c, arrow, arrow + _pol(ang, 11), _cb, 1.4);
    _dt(c, arrow + _pol(ang, 11), 1.8, _cb);
    _lab(c, 'DIR ${(ang * 57.3).round()}°', const Offset(130, 38), _cb, ax: .5);
    _num(c, _n2(len), const Offset(6, 4), 26, _cg);
    _lab(c, 'LENGTH', const Offset(8, 32), _cg);
  }, a: .5, b: .45),

  // 5. Rainy window: flick the wiper across; the city goes sharp, then the rain smears it again.
  _Pn('rain wiper', 'landscape·fx·flick·pic·whimsy', (c, s, t) {
    final clear = s.kick, r = (1 - clear) * (2 + s.b * 10);
    for (var i = 0; i < 8; i++) {
      final x = 10 + i * 17.0, h = 14 + _h(i + 3) * 40;
      _smear(c, [Offset(x, 98), Offset(x, 98 - h), Offset(x + 12, 98 - h), Offset(x + 12, 98)], Offset(0, r), i.isEven ? _cb : _cp, n: 7);
      for (var w = 0; w < 3; w++) {
        _dt(c, Offset(x + 3 + w * 3, 98 - h + 6 + w * 7) + Offset(0, r * (_h(i * 3 + w) - .5)), .7, _al(_cg, .7));
      }
    }
    _ln(c, const Offset(4, 98), const Offset(152, 98), _cd);
    _rc(c, const Rect.fromLTWH(2, 2, 152, 116), _cw, 1.2);
    final wa = -math.pi + clear * math.pi;
    _ln(c, const Offset(78, 116), const Offset(78, 116) + _pol(wa, 92), _cw, 1.4);
    for (var i = 0; i < 14; i++) {
      final q = Offset(_h(i * 5) * 150 + 3, (_h(i * 7) * 116 + t * 30 * (1 + _h(i))) % 116);
      _ln(c, q, q + const Offset(0, 4), _al(_cw, .5), .7);
    }
    _num(c, _n2(r * 8), const Offset(10, 6), 20, _cb);
    _lab(c, 'RAIN', const Offset(10, 28), _cb);
  }, b: .5),

  // 6. Kernel field, Max-style: the weights as a dot field; radius and direction stretch it.
  _Pn('kernel dots', 'diagram·mech·drag·diag·M4L', (c, s, t) {
    final r = .8 + s.a * 3.5, ang = s.b * math.pi, ca = math.cos(ang), sa = math.sin(ang);
    const o = Offset(62, 60);
    for (var j = -5; j <= 5; j++) {
      for (var i = -5; i <= 5; i++) {
        final u = i * ca + j * sa, v = -i * sa + j * ca;
        final g = math.exp(-(u * u) / (2 * r * r) - (v * v) / (2 * .7 * .7 * (1 + r * .1)));
        final q = o + Offset(i * 9.5, j * 9.5);
        if (g > .03) {
          _dt(c, q, .5 + g * 3.4, i == 0 && j == 0 ? _cr : _mx(_cb, _cg, g));
        } else {
          _dt(c, q, .45, _cd);
        }
      }
    }
    _ln(c, o - _pol(ang, 56), o + _pol(ang, 56), _al(_cw, .4), .6);
    _num(c, r.toStringAsFixed(1), const Offset(152, 4), 22, _cg, ax: 1);
    _lab(c, 'RADIUS', const Offset(152, 28), _cg, ax: 1);
    _lab(c, 'DIR ${(ang * 57.3).round()}°', const Offset(152, 104), _cb, ax: 1);
  }, a: .4, b: .15),

  // 7. Bell: draw how wide the spread should be; the dotted verticals mark one and two sigma.
  _Pn('drawn bell', 'diagram·mech·draw·num·sane', (c, s, t) {
    var sig = 14.0;
    if (s.ink.length > 3) {
      var lo = 999.0, hi = -999.0;
      for (final q in s.ink) {
        lo = math.min(lo, q.dx);
        hi = math.max(hi, q.dx);
      }
      sig = ((hi - lo) / 4).clamp(2.0, 40.0);
    }
    const cx = 78.0, y0 = 100.0;
    _ln(c, const Offset(6, y0), const Offset(150, y0), _cb);
    final pts = [for (var x = 6.0; x <= 150; x += 2) Offset(x, y0 - 58 * math.exp(-(x - cx) * (x - cx) / (2 * sig * sig)))];
    _pl(c, pts, _cb, w: 1.1);
    for (final k in [-2, -1, 1, 2]) {
      final x = cx + k * sig;
      if (x < 4 || x > 152) continue;
      final y = y0 - 58 * math.exp(-k * k / 2);
      _dots(c, Offset(x, y0), Offset(x, y), _cd);
      _dt(c, Offset(x, y), 1.8, k.abs() == 1 ? _cg : _cw);
    }
    _dt(c, const Offset(cx, y0 - 58), 2, _cb);
    _num(c, _n2(sig), const Offset(6, 2), 34, _cw);
    _lab(c, 'SIGMA', const Offset(8, 38), _cw);
    _lab(c, 'DRAW WIDTH', const Offset(150, 106), _cd, ax: 1);
  }),

  // 8. Turntable: spin the record; the label's marks smear into arcs as fast as it turns.
  _Pn('spun record', 'instrument·fx:rotation·spin·pic·sane', (c, s, t) {
    const o = Offset(66, 62);
    final rate = s.w.abs() + .3, base = s.spin + t * .3, smear = (rate * .12).clamp(0.0, 2.6);
    _ci(c, o, 50, _cw);
    for (var r = 22.0; r < 48; r += 4) {
      _ci(c, o, r, _cd, .5);
    }
    _ci(c, o, 18, _cg, .9);
    for (var i = 0; i < 5; i++) {
      final a = base + i * 1.256, r = 8 + (i % 3) * 3.0;
      c.drawArc(Rect.fromCircle(center: o, radius: r), a - smear, smear + .05, false, _sp(_cg, 1.3));
    }
    _dt(c, o, 1.5, _cw);
    _ln(c, const Offset(140, 10), const Offset(142, 70), _cw, 1.2);
    _ln(c, const Offset(142, 70), const Offset(104, 92), _cw, 1.2);
    _ci(c, const Offset(140, 10), 4, _cw);
    _num(c, '${(33 + rate * 6).round()}', const Offset(152, 92), 18, _cr, ax: 1);
    _lab(c, 'RPM', const Offset(152, 110), _cr, ax: 1);
  }),

  // 9. Zoom blur pushed: drag up and the streaks rush out of the centre until nothing is left but speed.
  _Pn('warp zoom', 'cosmic·fx·drag·pic·PUSHED', (c, s, t) {
    final z = s.b, k = 1 + z * z * 6;
    for (var i = 0; i < 70; i++) {
      final a = _h(i) * 2 * math.pi, r0 = 6 + ((_h(i + 40) * 70 + t * 30 * (1 + z * 4)) % 70);
      final c0 = i % 7 == 0 ? _cr : (i % 3 == 0 ? _cb : _cw);
      _ln(c, _pc + _pol(a, r0), _pc + _pol(a, r0 * k), _al(c0, .3 + .7 * (r0 / 76)), .8);
    }
    _ci(c, _pc, 4 + z * 3, _cw, 1);
    _num(c, _n2(z * 99), const Offset(152, 92), 26, z > .8 ? _cr : _cw, ax: 1);
    _lab(c, 'ZOOM', const Offset(6, 6), _cw);
  }, b: .35),

  // 10. Fish under water: rub the surface; the ripples bend the fish into a wobbling smear.
  _Pn('ripple fish', 'animal·fx:refraction·rub·pic·whimsy', (c, s, t) {
    final amp = 1 + s.rub * 8;
    _pl(c, [for (var x = 0.0; x <= 156; x += 3) Offset(x, 34 + math.sin(x * .12 + t * 3) * (1 + s.rub * 3))], _cb, w: 1.1);
    final fish = <Offset>[
      for (var i = 0; i <= 20; i++) Offset(78 - 24 * math.cos(i / 20 * math.pi), 72 - 10 * math.sin(i / 20 * math.pi)),
      for (var i = 0; i <= 20; i++) Offset(78 + 24 * math.cos(i / 20 * math.pi), 72 + 10 * math.sin(i / 20 * math.pi)),
    ];
    final tail = [const Offset(102, 72), const Offset(114, 62), const Offset(114, 82), const Offset(102, 72)];
    for (var k = 0; k < 5; k++) {
      final ph = k * 1.2;
      List<Offset> bend(List<Offset> l) => [for (final q in l) q + Offset(math.sin(q.dy * .3 + t * 4 + ph) * amp * (k == 0 ? .3 : 1), 0)];
      final col = _al(k == 0 ? _cw : _cg, k == 0 ? 1 : .3);
      _pl(c, bend(fish), col, close: true);
      _pl(c, bend(tail), col);
    }
    _dt(c, const Offset(64, 69), 1.2, _cw);
    for (var i = 0; i < 4; i++) {
      final y = 64 - ((t * 10 + i * 9) % 30);
      _ci(c, Offset(54 + math.sin(t * 2 + i) * 2, y), 1.5 + i * .3, _cw, .7);
    }
    _num(c, _n2(amp * 10), const Offset(6, 90), 22, _cg);
    _lab(c, 'RIPPLE', const Offset(6, 6), _cb);
  }),

  // 11. Ribbed shower glass: a figure behind it, the ribs shift every strip by the radius.
  _Pn('ribbed glass', 'character·fx:figure·drag·pic·whimsy', (c, s, t) {
    final r = s.a * 9;
    const strip = 6.0;
    for (var x = 14.0; x < 128; x += strip) {
      c.save();
      c.clipRect(Rect.fromLTWH(x, 8, strip, 104));
      final off = math.sin(x * 1.7) * r;
      _man(c, Offset(71 + off + math.sin(t * 1.2) * 8, 104), 80, _cg, arm: .6 + math.sin(t * 2) * .4, w: 1.3);
      c.restore();
      _ln(c, Offset(x, 8), Offset(x, 112), _al(_cd, .9), .6);
    }
    _rc(c, const Rect.fromLTRB(14, 8, 128, 112), _cw, 1);
    _num(c, r.toStringAsFixed(0), const Offset(152, 4), 18, _cb, ax: 1);
    _lab(c, 'RAD', const Offset(152, 24), _cb, ax: 1);
  }, a: .45),

  // 12. Focus plane: three trees at three distances; drag the plane, the aperture sizes every circle of confusion.
  _Pn('focus plane', 'diagram·mech+fx·drag·diag·sane', (c, s, t) {
    final fp = 40 + s.a * 108, ap = .05 + s.b * .25;
    _rc(c, const Rect.fromLTWH(4, 54, 14, 12), _cw);
    c.drawOval(const Rect.fromLTWH(18, 52, 6, 16), _sp(_cw));
    _ln(c, const Offset(24, 60), const Offset(152, 60), _al(_cd, .6), .6);
    for (final tx in [50.0, 92.0, 134.0]) {
      final r = (tx - fp).abs() * ap;
      _haze(c, _pine(Offset(tx, 100), 24), r, _cg, n: 7);
      _ci(c, Offset(tx, 22), 1 + r, r < 1.5 ? _cw : _cb, .9);
      _ln(c, Offset(tx, 100), Offset(tx, 106), _cg);
    }
    _dash(c, Offset(fp, 10), Offset(fp, 112), _cr, on: 2, off: 2);
    _lab(c, 'FOCUS', Offset(fp, 112), _cr, ax: fp > 120 ? 1 : 0, ay: 1);
    _lab(c, 'APERTURE ${(ap * 100).round()}', const Offset(6, 6), _cb);
  }, a: .4, b: .5),

  // 13. Wind across the grass: spin to turn the wind; every blade smears that way, the sock shows it.
  _Pn('wind grass', 'landscape·fx·spin·pic·sane', (c, s, t) {
    final ang = s.spin * .5, len = 4 + s.b * 10, d = _pol(ang, len);
    for (var i = 0; i < 60; i++) {
      final b0 = Offset(6 + _h(i) * 144, 60 + _h(i + 30) * 52);
      final sway = math.sin(t * 2 + i) * .2, tip = b0 + Offset(0, -8) + _pol(ang + sway, len * .5);
      _ln(c, b0, tip, _al(_cg, .9), .9);
      _ln(c, tip, tip + d * .4, _al(_cg, .35), .7);
    }
    const pole = Offset(126, 16);
    _ln(c, pole, pole + const Offset(0, 36), _cw);
    final u = _pol(ang + math.sin(t * 5) * .08, 1), n = Offset(-u.dy, u.dx);
    _pl(c, [pole + n * 5, pole + u * 24 + n * 2, pole + u * 24 - n * 2, pole - n * 5], _cr, close: true);
    for (var k = 1; k < 3; k++) {
      _ln(c, pole + u * (8.0 * k) + n * (5 - k * 1.2), pole + u * (8.0 * k) - n * (5 - k * 1.2), _cr, .9);
    }
    _num(c, '${((ang * 57.3) % 360).round()}', const Offset(6, 4), 24, _cw);
    _lab(c, 'DIR', const Offset(8, 30), _cw);
  }, b: .5),

  // 14. Camera shake: flick the hand; the snapshot smears the way and as far as you shook it.
  _Pn('shaky snap', 'character·fx:photo·flick·pic·whimsy', (c, s, t) {
    final sh = s.v * 6 + _pol(t * 31, s.kick * 3);
    final cam = Offset(30, 66) + sh * .3;
    _rc(c, Rect.fromCenter(center: cam, width: 30, height: 20), _cw);
    _ci(c, cam, 6, _cw);
    _rc(c, Rect.fromLTWH(cam.dx - 12, cam.dy - 14, 8, 4), _cw, .9);
    _pl(c, [cam + const Offset(-6, 10), cam + const Offset(-10, 30), cam + const Offset(4, 30), cam + const Offset(6, 10)], _cd);
    const frame = Rect.fromLTWH(64, 18, 84, 72);
    _rc(c, frame, _cw, 1);
    final d = s.v.distance < .05 ? Offset(s.kick * 8, 0) : (s.v / s.v.distance) * (1 + s.kick * 12);
    c.save();
    c.clipRect(frame.deflate(2));
    _smear(c, _house(const Offset(100, 76), 12), d, _cb, n: 9, close: false);
    _smear(c, _pine(const Offset(128, 78), 22), d, _cg, n: 9);
    _smear(c, [for (var i = 0; i < 16; i++) const Offset(84, 34) + _pol(i / 16 * 2 * math.pi, 6)], d, _cr, n: 9, close: true);
    c.restore();
    _ln(c, const Offset(64, 98), const Offset(148, 98), _cd);
    _lab(c, 'SHAKE ${(s.kick * 99).round()}', const Offset(148, 104), _cw, ax: 1);
  }),

  // 15. The number is the radius, and it blurs by itself: pinch it.
  _Pn('self-blurring numeral', 'typographic·fx:itself·pinch·num·sane', (c, s, t) {
    final r = s.z * 10, v = r.round().toString().padLeft(2, '0');
    final n = r < .5 ? 1 : 8;
    for (var i = 0; i < n; i++) {
      final o = n == 1 ? Offset.zero : _pol(i / n * 2 * math.pi + t * .3, r);
      _num(c, v, const Offset(78, 62) + o, 84, _al(_cw, n == 1 ? 1 : .28), ax: .5, ay: .5);
    }
    _lab(c, 'PX', const Offset(150, 108), _cb, ax: 1);
  }, z: .3),

  // 16. Tilt-shift town: drag the sharp band through an isometric town; the rest goes soft.
  _Pn('tilt-shift town', 'isometric·fx·drag·pic·sane', (c, s, t) {
    const o = Offset(78, 16);
    final band = 20 + s.fy * 80, half = 6 + s.b * 18;
    for (var j = 0; j < 5; j++) {
      for (var i = 0; i < 5; i++) {
        final hgt = 6 + _h(i * 5 + j) * 18;
        final base = _iso(o, i * 14.0, j * 14.0, 0);
        final r = ((base.dy - band).abs() - half).clamp(0.0, 30.0) * .18;
        final col = (base.dy - band).abs() < half ? _cw : _cb;
        if (r < .5) {
          _cube(c, o, i * 14.0, j * 14.0, 0, 9, 9, hgt, col, .9);
        } else {
          for (var k = 0; k < 4; k++) {
            _cube(c, o + _pol(k * 1.57 + .4, r), i * 14.0, j * 14.0, 0, 9, 9, hgt, _al(col, .3), .7);
          }
        }
      }
    }
    _dash(c, Offset(0, band - half), Offset(156, band - half), _cr, on: 2, off: 3);
    _dash(c, Offset(0, band + half), Offset(156, band + half), _cr, on: 2, off: 3);
    _lab(c, 'SHARP ${(half * 2).round()}', const Offset(4, 4), _cr);
  }, b: .3, fy: .55),

  // 17. Draw the direction: your stroke is the blur vector; the stars smear along it.
  _Pn('drawn vector', 'diagram·mech·draw·pic·sane', (c, s, t) {
    final a = s.ink.length > 2 ? s.ink.first : const Offset(40, 80), b = s.ink.length > 2 ? s.ink.last : const Offset(70, 64);
    final d = (b - a) * .5;
    for (var i = 0; i < 7; i++) {
      final q = Offset(20 + _h(i * 3) * 116, 18 + _h(i * 5 + 1) * 84);
      final star = [for (var k = 0; k < 10; k++) q + _pol(k * math.pi / 5 - math.pi / 2, k.isEven ? 4 : 1.8)];
      _smear(c, star, d * (.6 + _h(i) * .4), i.isEven ? _cw : _cg, n: 8, close: true);
    }
    _ln(c, a, b, _cr, 1.2);
    _dt(c, a, 2, _cr);
    final u = (b - a).distance < 1 ? const Offset(1, 0) : (b - a) / (b - a).distance;
    _pl(c, [b - u * 5 + Offset(-u.dy, u.dx) * 3, b, b - u * 5 - Offset(-u.dy, u.dx) * 3], _cr, w: 1.2);
    _num(c, _n2((b - a).distance / 2), const Offset(152, 2), 22, _cr, ax: 1);
    _lab(c, 'DRAW', const Offset(152, 26), _cd, ax: 1);
  }),

  // 18. Device chain: in -> radius amp -> direction -> aperture -> out, the monitor shows the dot it makes.
  _Pn('blur chain', 'diagram·mech·drag·diag·M4L', (c, s, t) {
    final r = s.a * 10, ang = s.b * math.pi;
    _rc(c, const Rect.fromLTWH(4, 44, 16, 16), _cw);
    _dt(c, const Offset(12, 52), 1.6, _cw);
    _pl(c, const [Offset(28, 40), Offset(50, 52), Offset(28, 64)], _cg, close: true);
    _num(c, _n2(r * 10), const Offset(29, 52), 9, _cg, ay: .5);
    _rc(c, const Rect.fromLTWH(58, 42, 20, 20), _cb);
    _ln(c, const Offset(68, 52) - _pol(ang, 7), const Offset(68, 52) + _pol(ang, 7), _cb, 1.1);
    _dt(c, const Offset(68, 52) + _pol(ang, 7), 1.4, _cb);
    _pl(c, [for (var i = 0; i < 6; i++) const Offset(96, 52) + _pol(i * math.pi / 3 + r * .1, 9)], _cw, close: true);
    for (final seg in const [[Offset(20, 52), Offset(28, 52)], [Offset(50, 52), Offset(58, 52)], [Offset(78, 52), Offset(87, 52)], [Offset(105, 52), Offset(114, 52)]]) {
      _ln(c, seg[0], seg[1], _cd);
    }
    _rc(c, const Rect.fromLTWH(114, 36, 36, 32), _cw);
    final dd = _pol(ang, r);
    for (var k = 0; k < 9; k++) {
      _dt(c, const Offset(132, 52) + dd * (k / 8 * 2 - 1), 1.4, _al(_cr, r < .5 ? 1 : .35));
    }
    _lab(c, 'IN', const Offset(12, 70), _cw, ax: .5);
    _lab(c, 'RADIUS', const Offset(38, 72), _cg, ax: .5);
    _lab(c, 'DIR', const Offset(68, 70), _cb, ax: .5);
    _lab(c, 'IRIS', const Offset(96, 72), _cw, ax: .5);
    _lab(c, 'OUT', const Offset(132, 76), _cw, ax: .5);
  }, a: .5, b: .2),

  // 19. Star trails: drag the exposure; the sky turns round the pole star and each star becomes an arc.
  _Pn('star trails', 'cosmic·fx·drag·pic·whimsy', (c, s, t) {
    const pole = Offset(96, 30);
    final ex = s.a * 1.6 + .02;
    for (var i = 0; i < 46; i++) {
      final r = 6 + _h(i) * 110, a0 = _h(i + 60) * 2 * math.pi + t * .05;
      final col = i % 9 == 0 ? _cb : (i % 7 == 0 ? _cr : _cw);
      c.drawArc(Rect.fromCircle(center: pole, radius: r), a0, ex, false, _sp(_al(col, .4 + _h(i + 3) * .6), .8));
    }
    _dt(c, pole, 1.6, _cw);
    final hills = [for (var x = 0.0; x <= 156; x += 6) Offset(x, 100 - math.sin(x * .05) * 6 - math.sin(x * .13) * 3)];
    c.drawPath(Path()..addPolygon([...hills, const Offset(156, 120), const Offset(0, 120)], true), _fp(_bg));
    _pl(c, hills, _cg);
    for (final x in [22.0, 30.0, 120.0]) {
      _pl(c, _pine(Offset(x, 100 - math.sin(x * .05) * 6 - math.sin(x * .13) * 3), 12), _cg, w: .9);
    }
    _num(c, '${(ex * 229).round()}', const Offset(6, 2), 22, _cw);
    _lab(c, 'MIN', const Offset(8, 26), _cw);
  }, a: .3),

  // 20. Whiteout: drag to the end and the whole little world dissolves into hatched fog; the number runs off the scale.
  _Pn('whiteout', 'landscape·fx·drag·num·PUSHED', (c, s, t) {
    final r = s.a * s.a * 26;
    final mtn = [const Offset(6, 96), const Offset(44, 40), const Offset(66, 70), const Offset(92, 30), const Offset(150, 96)];
    _haze(c, mtn, r, _cw, n: 10);
    _haze(c, _house(const Offset(112, 96), 8), r, _cg, n: 10);
    _haze(c, [for (var i = 0; i < 16; i++) const Offset(126, 26) + _pol(i / 16 * 2 * math.pi, 8)], r, _cr, n: 10, close: true);
    if (s.a > .6) {
      final k = ((s.a - .6) * 40).round();
      for (var i = 0; i < k; i++) {
        final y = 4 + i * 112 / math.max(1, k);
        _ln(c, Offset(0, y), Offset(156, y + math.sin(t + i) * 2), _al(_cw, .12 + (s.a - .6) * .4), .6);
      }
    }
    _ln(c, const Offset(4, 96), const Offset(152, 96), _al(_cd, 1 - s.a * .8));
    _num(c, '${(r * r * 1.5).round()}', const Offset(152, 120), 28 + s.a * 30, s.a > .8 ? _cr : _cw, ax: 1, ay: 1);
    _lab(c, 'PX', const Offset(6, 6), _cw);
  }, a: .5),
];
