part of 'op_02.dart';

void _bike(Canvas c, double x, double y, Color col) {
  final p = _sp(col, 1);
  c.drawCircle(Offset(x - 9, y), 5, p);
  c.drawCircle(Offset(x + 9, y), 5, p);
  c.drawPath(
      Path()
        ..moveTo(x - 9, y)
        ..lineTo(x - 2, y - 7)
        ..lineTo(x + 6, y - 7)
        ..lineTo(x + 9, y)
        ..moveTo(x - 2, y - 7)
        ..lineTo(x + 1, y)
        ..lineTo(x + 6, y - 7),
      p);
  c.drawCircle(Offset(x + 1, y - 18), 2.6, p);
  c.drawLine(Offset(x + 1, y - 15), Offset(x - 2, y - 8), p);
  c.drawLine(Offset(x, y - 13), Offset(x + 7, y - 10), p);
}

void _duck(Canvas c, Offset q, double sz, Color col, bool right) {
  final p = _sp(col, .9), sx = right ? 1.0 : -1.0;
  c.drawOval(Rect.fromCenter(center: q, width: 12 * sz, height: 6 * sz), p);
  final hd = q + Offset(5 * sz * sx, -5 * sz);
  c.drawCircle(hd, 2.4 * sz, p);
  c.drawLine(hd + Offset(2.4 * sz * sx, .3 * sz), hd + Offset(5 * sz * sx, .8 * sz), p);
  c.drawLine(q + Offset(-6 * sz * sx, 0), q + Offset(-8 * sz * sx, -2.5 * sz), p);
}

/// Point [dist] back along [pts] from its last point.
Offset _along(List<Offset> pts, double dist) {
  var acc = 0.0;
  for (var i = pts.length - 1; i > 0; i--) {
    final d = (pts[i] - pts[i - 1]).distance;
    if (acc + d >= dist) return Offset.lerp(pts[i], pts[i - 1], (dist - acc) / (d == 0 ? 1 : d))!;
    acc += d;
  }
  return pts.first;
}

final _echo = <_Pn>[
  // 1. Yodel in a canyon: the call bounces wall to wall; the canyon width is the spacing, the rock the decay.
  _Pn('yodel canyon', 'landscape·fx:voice·drag·pic·whimsy', (c, s, t) {
    final gap = 40 + s.a * 80, lw = 78 - gap / 2, rw = 78 + gap / 2, dec = .45 + s.b * .5;
    _pl(c, [Offset(0, 30), Offset(lw - 8, 30), Offset(lw - 4, 46), Offset(lw, 62), Offset(lw - 5, 84), Offset(lw, 120)], _cw);
    _pl(c, [Offset(156, 24), Offset(rw + 6, 24), Offset(rw + 2, 50), Offset(rw, 70), Offset(rw + 6, 92), Offset(rw, 120)], _cw);
    _man(c, Offset(lw - 18, 30), 18, _cg, arm: 2.4);
    const n = 9;
    final shown = (t * 4) % (n + 4);
    for (var i = 0; i < n && i < shown; i++) {
      final x = i.isEven ? lw + 10 : rw - 10, y = 36 + i * 8.5, a = math.pow(dec, i).toDouble();
      _tx(c, 'HO', Offset(x, y), 12 - i * .7, _al(i == 0 ? _cg : _cw, a), wght: 300, ax: i.isEven ? 0 : 1, ay: .5);
      if (i > 0) _dash(c, Offset(i.isEven ? rw - 10 : lw + 10, y - 8.5), Offset(x, y), _al(_cd, a), on: 1, off: 2.5);
    }
    _num(c, _n2(dec * 99), const Offset(152, 2), 20, _cb, ax: 1);
    _lab(c, 'FDBK', const Offset(152, 26), _cb, ax: 1);
  }, a: .5, b: .55),

  // 2. Motorbike: throw it; the faster it goes, the wider its ghosts spread behind it.
  _Pn('ghost rider', 'vehicle·fx·flick·pic·sane', (c, s, t) {
    final x = 16 + s.fx * 124, sp = 3 + (s.v.dx.abs() * 9).clamp(0.0, 16.0) + s.b * 6, dir = s.v.dx < 0 ? 1.0 : -1.0;
    _ln(c, const Offset(0, 86), const Offset(156, 86), _cd);
    for (var i = 0; i < 12; i++) {
      final dx = ((i * 18 - x * 1.5) % 180) - 12;
      _ln(c, Offset(dx, 96), Offset(dx + 6, 96), _cd, .8);
    }
    for (var i = 6; i >= 1; i--) {
      _bike(c, x + dir * sp * i, 80, _al(i.isEven ? _cb : _cp, math.pow(.68, i).toDouble()));
    }
    _bike(c, x, 80, _cw);
    _num(c, _n2(sp * 4), const Offset(6, 4), 30, _cb);
    _lab(c, 'SPACING', const Offset(8, 36), _cb);
  }, fx: .55, b: .3),

  // 3. Onion skin: an animator's ghosts of a bouncing ball; count and spacing on the drag.
  _Pn('onion skin', 'diagram·fx:motion·drag·pic·sane', (c, s, t) {
    final n = 2 + (s.a * 10).round(), gap = .03 + s.b * .12;
    Offset ball(double ph) {
      final u = ph % 1, x = 10 + u * 136, h = (math.sin(u * math.pi * 3)).abs();
      return Offset(x, 96 - h * 64);
    }

    _ln(c, const Offset(4, 104), const Offset(152, 104), _cd);
    _pl(c, [for (var i = 0; i <= 60; i++) ball(i / 60.0 * .999)], _cd, w: .6);
    final ph = t * .22;
    for (var i = n; i >= 1; i--) {
      if ((ph - i * gap) % 1 > (ph % 1)) continue;
      _ci(c, ball(ph - i * gap), 6, _al(_cb, math.pow(.78, i).toDouble()), .8);
    }
    _ci(c, ball(ph), 6, _cw, 1.3);
    _num(c, _n2(n.toDouble()), const Offset(152, 2), 26, _cb, ax: 1);
    _lab(c, 'SKINS', const Offset(152, 30), _cb, ax: 1);
  }, a: .5, b: .4),

  // 4. Tape loop: spin the reels; tape speed sets how far apart the play heads hear the recorded blip.
  _Pn('tape loop', 'machine·mech·spin·pic·sane', (c, s, t) {
    final sp = (.6 + s.spin * .12).clamp(.15, 2.0), ang = t * sp * 3;
    for (final o in const [Offset(36, 40), Offset(120, 40)]) {
      _ci(c, o, 20, _cw);
      _ci(c, o, 4, _cw, .9);
      for (var i = 0; i < 3; i++) {
        _ln(c, o + _pol(ang + i * 2.094, 5), o + _pol(ang + i * 2.094, 17), _cd, 1);
      }
    }
    _pl(c, const [Offset(16, 40), Offset(22, 84), Offset(134, 84), Offset(140, 40)], _cw, w: .9);
    _rc(c, const Rect.fromLTWH(30, 86, 8, 6), _cr);
    _lab(c, 'REC', const Offset(34, 95), _cr, ax: .5);
    final hg = 10 + sp * 14;
    for (var i = 1; i <= 4; i++) {
      final hx = 34 + hg * i;
      if (hx > 132) break;
      _rc(c, Rect.fromLTWH(hx - 4, 86, 8, 6), _al(_cg, math.pow(.75, i - 1).toDouble()));
    }
    for (var i = 0; i < 4; i++) {
      final bx = 34 + ((t * sp * 30 + i * 37) % 100);
      _pl(c, [Offset(bx - 3, 84), Offset(bx - 1, 80), Offset(bx + 1, 87), Offset(bx + 3, 84)], _cb, w: .9);
    }
    _num(c, (sp * 7.5).toStringAsFixed(1), const Offset(78, 104), 14, _cg, ax: .5, ay: .5);
    _lab(c, 'IPS', const Offset(100, 101), _cg);
  }),

  // 5. Ping-pong between L and R, a big count; each return a little lower.
  _Pn('ping pong', 'diagram·mech·drag·num·sane', (c, s, t) {
    final n = 2 + (s.a * 12).round(), dec = .5 + s.b * .45;
    _ln(c, const Offset(14, 60), const Offset(14, 108), _cb, 1.2);
    _ln(c, const Offset(142, 60), const Offset(142, 108), _cr, 1.2);
    _lab(c, 'L', const Offset(14, 52), _cb, ax: .5);
    _lab(c, 'R', const Offset(142, 52), _cr, ax: .5);
    final pts = <Offset>[for (var i = 0; i < n; i++) Offset(i.isEven ? 16 : 140, 108 - 46 * math.pow(dec, i).toDouble())];
    _pl(c, pts, _cd, w: .7);
    for (var i = 0; i < pts.length; i++) {
      _dt(c, pts[i], 2.4 * math.pow(dec, i * .5) + .5, i.isEven ? _cb : _cr);
    }
    final u = (t * .9) % (n - 1), j = u.floor().clamp(0, n - 2);
    _ci(c, Offset.lerp(pts[j], pts[j + 1], u - j)!, 3, _cw);
    _num(c, _n2(n.toDouble()), const Offset(78, 4), 44, _cw, ax: .5);
  }, a: .4, b: .5),

  // 6. One letter, stamped again and again: pinch for the scale step; drag sets the spacing.
  _Pn('stamped A', 'typographic·fx:glyph·pinch·type·sane', (c, s, t) {
    final step = .55 + s.z * .4, gap = 10 + s.a * 22;
    var x = 6.0, sz = 82.0;
    for (var i = 0; i < 12 && sz > 4 && x < 160; i++) {
      final col = i == 0 ? _cw : _al(i.isEven ? _cg : _cb, math.pow(.8, i).toDouble());
      _tx(c, 'A', Offset(x, 108), sz.roundToDouble(), col, wght: 100, ay: .82);
      x += gap * sz / 82 + sz * .35;
      sz *= step;
    }
    _lab(c, 'STEP ${(step * 100).round()}', const Offset(152, 6), _cg, ax: 1);
  }, z: .6, a: .2),

  // 7. Comet round a planet: spin it faster and the tail stretches out behind.
  _Pn('comet tail', 'cosmic·fx·spin·pic·whimsy', (c, s, t) {
    const o = Offset(78, 62);
    final th = s.spin + t * .5, sp = .06 + (s.w.abs() * .02).clamp(0.0, .12) + s.b * .04;
    for (var i = 0; i < 64; i++) {
      final a = i / 64 * 2 * math.pi;
      _dt(c, o + Offset(math.cos(a) * 62, math.sin(a) * 40), .45, _cd);
    }
    _ci(c, o, 9, _cw);
    c.drawOval(Rect.fromCenter(center: o, width: 30, height: 7), _sp(_cp, .9));
    for (var i = 16; i >= 1; i--) {
      final a = th - i * sp, q = o + Offset(math.cos(a) * 62, math.sin(a) * 40);
      _dt(c, q, 2.6 * math.pow(.86, i) + .3, _al(_cg, math.pow(.84, i).toDouble()));
    }
    _ci(c, o + Offset(math.cos(th) * 62, math.sin(th) * 40), 3.5, _cw, 1.2);
    _num(c, _n2(sp * 600), const Offset(6, 2), 22, _cg);
    _lab(c, 'TAIL', const Offset(6, 26), _cg);
  }, b: .3),

  // 8. Mother duck and ducklings: drag her; the brood follows your actual path, each one smaller.
  _Pn('ducklings', 'animal·fx:path·drag·pic·whimsy', (c, s, t) {
    final path = s.ink.length > 3 ? s.ink : [for (var i = 0; i <= 40; i++) Offset(8 + i * 3.4, 74 + math.sin(i * .25) * 16)];
    final gap = 14 + s.a * 12, head = path.last;
    final right = path.length < 2 || path.last.dx >= path[path.length - 2].dx - .01;
    for (var i = 0; i < 28; i++) {
      _dt(c, _along(path, i * 5.0), .5, _cd);
    }
    for (var i = 5; i >= 1; i--) {
      final q = _along(path, gap * i) + Offset(0, math.sin(t * 6 + i) * .7);
      _duck(c, q, 1.3 * math.pow(.85, i).toDouble(), _al(_cg, .4 + math.pow(.85, i) * .6), right);
    }
    _duck(c, head, 1.8, _cw, right);
    for (var i = 0; i < 6; i++) {
      final x = ((i * 31 + t * 6) % 170) - 8;
      _pl(c, [Offset(x, 112), Offset(x + 3, 110), Offset(x + 6, 112)], _cd, w: .8);
    }
    _lab(c, 'SPACING ${gap.round()}', const Offset(6, 6), _cb);
  }, a: .4),

  // 9. Device chain: in -> feedback -> stack of delay taps -> out, the loop back drawn dashed.
  _Pn('feedback chain', 'diagram·mech·drag·diag·M4L', (c, s, t) {
    final n = 1 + (s.a * 7).round(), fb = s.b;
    _rc(c, const Rect.fromLTWH(4, 44, 18, 18), _cw);
    _pl(c, const [Offset(10, 57), Offset(13, 49), Offset(16, 57)], _cw, w: .9);
    _pl(c, const [Offset(32, 38), Offset(58, 53), Offset(32, 68)], _cg, close: true);
    _num(c, _n2(fb * 99), const Offset(35, 53), 9, _cg, ay: .5);
    for (var i = n - 1; i >= 0; i--) {
      _rc(c, Rect.fromLTWH(70 + i * 3.0, 42 - i * 3.0, 24, 22), i == 0 ? _cb : _al(_cb, .5));
    }
    _tx(c, 'DLY', const Offset(82, 53), 9, _cb, wght: 300, ax: .5, ay: .5);
    _ci(c, const Offset(134, 53), 14, _cw);
    for (var i = 0; i < 6; i++) {
      _dt(c, const Offset(134, 53) + _pol(i * math.pi / 3, 8), 1.4, i < (fb * 6).ceil() ? _cg : _cd);
    }
    _ln(c, const Offset(22, 53), const Offset(32, 53), _cd);
    _ln(c, const Offset(58, 53), const Offset(70, 53), _cd);
    _ln(c, const Offset(94, 53), const Offset(120, 53), _cd);
    const loop = [Offset(108, 53), Offset(108, 84), Offset(45, 84), Offset(45, 64)];
    for (var i = 0; i < 3; i++) {
      _dash(c, loop[i], loop[i + 1], _cr, on: 2, off: 2);
    }
    _pl(c, const [Offset(42, 68), Offset(45, 63), Offset(48, 68)], _cr, w: 1);
    final u = (t * .5) % 1;
    for (var k = 0; k < 4; k++) {
      final uu = (u + k * .25) % 1;
      final q = uu < .33
          ? Offset.lerp(loop[0], loop[1], uu / .33)!
          : uu < .8
              ? Offset.lerp(loop[1], loop[2], (uu - .33) / .47)!
              : Offset.lerp(loop[2], loop[3], (uu - .8) / .2)!;
      _dt(c, q, 1.8 * math.pow(fb, k), _cr);
    }
    _lab(c, 'IN', const Offset(13, 70), _cw, ax: .5);
    _lab(c, 'FDBK', const Offset(44, 92), _cr, ax: .5);
    _lab(c, 'TAPS $n', const Offset(84, 70), _cb, ax: .5);
    _lab(c, 'OUT', const Offset(134, 72), _cw, ax: .5);
  }, a: .4, b: .6),

  // 10. Impulse response: draw the decay envelope over the taps; the taps follow your line.
  _Pn('drawn taps', 'diagram·mech·draw·diag·M4L', (c, s, t) {
    const y0 = 100.0, top = 18.0;
    final gap = 7 + s.b * 6;
    double env(double x) {
      if (s.ink.length > 4) {
        var best = s.ink.first;
        for (final q in s.ink) {
          if ((q.dx - x).abs() < (best.dx - x).abs()) best = q;
        }
        return _cl((y0 - best.dy) / (y0 - top));
      }
      return math.pow(.82, (x - 14) / 9).toDouble();
    }

    _ln(c, const Offset(8, y0), const Offset(150, y0), _cd);
    for (var x = 14.0; x < 150; x += 4) {
      _dt(c, Offset(x, y0 - env(x) * (y0 - top)), .5, _cd);
    }
    var i = 0;
    for (var x = 14.0; x < 150; x += gap, i++) {
      final q = Offset(x, y0 - env(x) * (y0 - top));
      _ln(c, Offset(x, y0), q, i == 0 ? _cw : _cb, 1);
      _ci(c, q, 1.8, i == 0 ? _cw : _cg, 1);
    }
    final ph = (t * 40) % 150;
    _dots(c, Offset(ph, top), Offset(ph, y0), _al(_cr, .7));
    _lab(c, 'DRAW DECAY', const Offset(150, 6), _cd, ax: 1);
    _lab(c, 'TAPS $i', const Offset(150, 106), _cb, ax: 1);
  }, b: .3),

  // 11. Two facing mirrors: you between them, repeated down the corridor, each copy smaller and dimmer.
  _Pn('mirror corridor', 'object·fx:figure·drag·pic·sane', (c, s, t) {
    final sh = .62 + s.a * .28, dec = .5 + s.b * .45;
    var r = const Rect.fromLTRB(8, 8, 148, 112);
    for (var i = 0; i < 12; i++) {
      final a = math.pow(dec, i).toDouble();
      if (a < .04 || r.width < 4) break;
      _rc(c, r, _al(i == 0 ? _cw : _cp, a), .9);
      _man(c, Offset(r.center.dx, r.bottom - r.height * .1), r.height * .55, _al(_cg, a),
          arm: .4 + .2 * math.sin(t * 3 - i * .7), w: math.max(.5, 1.1 * math.pow(sh, i * .5).toDouble()));
      final nr = Rect.fromCenter(center: r.center, width: r.width * sh, height: r.height * sh);
      _ln(c, r.topLeft, nr.topLeft, _al(_cd, a));
      _ln(c, r.topRight, nr.topRight, _al(_cd, a));
      _ln(c, r.bottomLeft, nr.bottomLeft, _al(_cd, a));
      _ln(c, r.bottomRight, nr.bottomRight, _al(_cd, a));
      r = nr;
    }
    _lab(c, 'DEPTH ${(dec * 99).round()}', const Offset(12, 12), _cp);
  }, a: .4, b: .6),

  // 12. Footprints in snow: rub to make it snow; fresh snow fills the old prints sooner.
  _Pn('snow prints', 'landscape·fx:trace·rub·pic·whimsy', (c, s, t) {
    final step = 8 + s.a * 10, dec = .9 - s.rub * .55, wx = (t * 16) % 190 - 10;
    _ln(c, const Offset(0, 100), const Offset(156, 100), _cd);
    final ph = (wx / step).floor();
    for (var i = 0; i < 20; i++) {
      final x = (ph - i) * step;
      if (x < -6) break;
      c.drawOval(Rect.fromCenter(center: Offset(x, i.isEven ? 104 : 108), width: 5, height: 2.4),
          _sp(_al(_cw, math.pow(dec, i).toDouble()), .9));
    }
    _man(c, Offset(wx, 99), 24, _cw, leg: .45 * math.sin(t * 6), arm: .4 + .3 * math.sin(t * 6));
    final nf = 10 + (s.rub * 70).round();
    for (var i = 0; i < nf; i++) {
      final x = _h(i) * 156 + math.sin(t + i) * 3, y = (_h(i + 50) * 120 + t * (8 + _h(i + 9) * 10)) % 96;
      _dt(c, Offset(x, y), .7, _cb);
    }
    _num(c, _n2(dec * 99), const Offset(152, 4), 22, _cw, ax: 1);
    _lab(c, 'HOLD', const Offset(152, 28), _cw, ax: 1);
  }, a: .3),

  // 13. Spirograph: one loop, repeated round the gear; spin sets the step between repeats, up to sixty deep.
  _Pn('spirograph', 'instrument·fx:loop·spin·pic·PUSHED', (c, s, t) {
    final step = .24 + s.spin * .03, n = (12 + s.b * 48).round();
    const cols = [_cb, _cg, _cw, _cr];
    for (var i = n - 1; i >= 0; i--) {
      final rot = i * step + t * .05;
      final pts = [
        for (var j = 0; j <= 40; j++)
          _pc + _pol(rot, 54 * math.sin(j / 40 * math.pi)) + _pol(rot + math.pi / 2, 16 * math.sin(j / 40 * math.pi * 2))
      ];
      _pl(c, pts, _al(cols[i % 4], .2 + .8 * math.pow(.94, i)), w: .7);
    }
    _ci(c, _pc, 3, _cw, 1);
    _num(c, '$n', const Offset(4, 2), 18, _cw);
    _lab(c, 'STEP ${(step * 57.3).round()}°', const Offset(4, 108), _cw);
  }, b: .3),

  // 14. Heart monitor: one beat, then its echoes, each smaller, spaced by the delay.
  _Pn('heart echo', 'machine·fx:signal·drag·diag·sane', (c, s, t) {
    final gap = 12 + s.a * 26, dec = .4 + s.b * .5;
    double beat(double u) {
      if (u < 0 || u > 8) return 0;
      if (u < 2) return -u * .2;
      if (u < 3) return (u - 2) * 1.2 - .4;
      if (u < 4.5) return .8 - (u - 3) * 1.3;
      if (u < 6) return -1.15 + (u - 4.5) * .8;
      return 0;
    }

    for (var y = 20.0; y < 110; y += 10) {
      _ln(c, Offset(0, y), Offset(156, y), _al(_cd, .35), .5);
    }
    final x0 = 10 + (t * 12) % 30, pts = <Offset>[];
    for (var x = 0.0; x <= 156; x += 1.5) {
      var v = 0.0;
      for (var k = 0; k < 6; k++) {
        v += beat((x - x0 - k * gap) / 2) * math.pow(dec, k);
      }
      pts.add(Offset(x, 66 - v * 34));
    }
    _pl(c, pts, _cg, w: 1);
    for (var k = 1; k < 6; k++) {
      final x = x0 + k * gap + 6;
      if (x < 150) _dots(c, Offset(x, 22), Offset(x, 106), _al(_cb, math.pow(dec, k).toDouble()));
    }
    _num(c, _n2(gap * 2), const Offset(152, 2), 22, _cb, ax: 1);
    _lab(c, 'MS x10', const Offset(152, 24), _cb, ax: 1);
  }, a: .4, b: .5),

  // 15. Strobe dancer: rub to fire the strobe faster; more copies of each pose stay on screen.
  _Pn('strobe dancer', 'character·fx:pose·rub·pic·whimsy', (c, s, t) {
    final n = 2 + (s.rub * 12).round(), gap = .05 + s.b * .1;
    for (var i = n; i >= 0; i--) {
      final tt = t - i * gap, x = 78 + math.sin(tt * 1.3) * 34;
      final a = i == 0 ? 1.0 : math.pow(.82, i) * .8;
      _man(c, Offset(x, 104 - (math.sin(tt * 5)).abs() * 8), 58, _al(i == 0 ? _cw : (i.isEven ? _cr : _cp), a.toDouble()),
          arm: 1.2 + math.sin(tt * 4) * 1.1, leg: .25 + math.sin(tt * 5) * .3, w: i == 0 ? 1.3 : .8);
    }
    _num(c, _n2(n.toDouble()), const Offset(6, 2), 24, _cr);
    _lab(c, 'FLASH', const Offset(6, 28), _cr);
  }, b: .4),

  // 16. Delay ruler, Max-style: the drag snaps the spacing to note values; arcs carry each repeat.
  _Pn('note ruler', 'diagram·mech·drag·num·M4L', (c, s, t) {
    const divs = ['1/32', '1/16', '1/8', '3/16', '1/4', '1/2'];
    const len = [.125, .25, .5, .75, 1.0, 2.0];
    final di = (s.a * 5.99).floor(), dec = .45 + s.b * .5, gap = len[di] * 34;
    for (var b = 0; b <= 4; b++) {
      _ln(c, Offset(8 + b * 34.0, 96), Offset(8 + b * 34.0, 104), _cw, 1);
      for (var q = 1; q < 4; q++) {
        _ln(c, Offset(8 + b * 34.0 + q * 8.5, 100), Offset(8 + b * 34.0 + q * 8.5, 104), _cd, .8);
      }
    }
    _ln(c, const Offset(8, 104), const Offset(150, 104), _cd);
    var x = 8.0;
    for (var i = 0; i < 16 && x + gap < 150; i++) {
      final a = math.pow(dec, i + 1).toDouble(), nx = x + gap;
      c.drawArc(Rect.fromLTRB(x, 90 - gap * .6, nx, 90 + gap * .6), math.pi, math.pi, false, _sp(_al(_cg, a), .9));
      _ci(c, Offset(nx, 90), 2, _al(_cg, a), 1);
      x = nx;
    }
    _ci(c, const Offset(8, 90), 2.4, _cw, 1.2);
    _num(c, divs[di], const Offset(78, 8), 40, _cw, ax: .5);
    _lab(c, 'TIME', const Offset(78, 50), _cd, ax: .5);
  }, a: .45, b: .5),

  // 17. Radar: pinch the afterglow; the sweep leaves a trail and the contacts fade behind it.
  _Pn('radar afterglow', 'machine·fx:persist·pinch·pic·sane', (c, s, t) {
    const o = Offset(62, 60);
    final persist = .3 + s.z * 2.6, sw = t * 2.2;
    for (final r in [16.0, 32.0, 48.0]) {
      _ci(c, o, r, _cd, .8);
    }
    _ln(c, o - const Offset(50, 0), o + const Offset(50, 0), _cd, .6);
    _ln(c, o - const Offset(0, 50), o + const Offset(0, 50), _cd, .6);
    for (var i = 0; i < 24; i++) {
      final back = i / 24 * persist, a = 1 - i / 24;
      _ln(c, o, o + _pol(sw - back, 48), _al(_cg, a * a * .7), .8);
    }
    for (var i = 0; i < 5; i++) {
      final a = _h(i) * 2 * math.pi, r = 12 + _h(i + 7) * 34;
      final since = (sw - a) % (2 * math.pi), f = _cl(1 - since / persist);
      _ci(c, o + _pol(a, r), 2.2, _al(_cr, f), 1);
      if (f > .7) _ci(c, o + _pol(a, r), 5, _al(_cr, (f - .7) * 3), .7);
    }
    _num(c, persist.toStringAsFixed(1), const Offset(152, 4), 22, _cg, ax: 1);
    _lab(c, 'GLOW S', const Offset(152, 28), _cg, ax: 1);
  }, z: .4),

  // 18. Train on an oval: flick it along; carriages follow at the coupling distance.
  _Pn('toy train', 'vehicle·fx:followers·flick·pic·whimsy', (c, s, t) {
    const o = Offset(78, 64), rx = 64.0, ry = 34.0;
    final n = 2 + (s.a * 7).round(), gap = .2 + s.b * .25, th = s.spin * .6 + t * .15;
    c.drawOval(Rect.fromCenter(center: o, width: rx * 2, height: ry * 2), _sp(_cd, .8));
    c.drawOval(Rect.fromCenter(center: o, width: rx * 2 + 8, height: ry * 2 + 8), _sp(_cd, .8));
    for (var i = 0; i < 40; i++) {
      final a = i / 40 * 2 * math.pi;
      _ln(c, o + Offset(math.cos(a) * (rx - 2), math.sin(a) * (ry - 2)), o + Offset(math.cos(a) * (rx + 6), math.sin(a) * (ry + 6)),
          _al(_cd, .6), .6);
    }
    void car(double a, double len, Color col) {
      final q = o + Offset(math.cos(a) * (rx + 2), math.sin(a) * (ry + 2));
      final tg = Offset(-math.sin(a) * rx, math.cos(a) * ry), u = tg / tg.distance, nn = Offset(-u.dy, u.dx);
      _pl(c, [q + u * len + nn * 4, q - u * len + nn * 4, q - u * len - nn * 4, q + u * len - nn * 4], col, w: 1, close: true);
    }

    for (var i = n; i >= 1; i--) {
      car(th - i * gap, 6, _al(i.isEven ? _cb : _cg, math.pow(.85, i).toDouble()));
    }
    car(th, 8, _cw);
    _num(c, _n2(n.toDouble()), const Offset(78, 52), 24, _cw, ax: .5);
    _lab(c, 'CARS', const Offset(78, 78), _cd, ax: .5);
  }, a: .4, b: .3),

  // 19. Your own stroke, echoed: draw anything; it repeats down and to the right, fading.
  _Pn('stroke echo', 'diagram·fx:your line·draw·pic·sane', (c, s, t) {
    final ink = s.ink.length > 3 ? s.ink : [for (var i = 0; i <= 30; i++) Offset(14 + i * 2.2, 30 + math.sin(i * .5) * 12 - i * .3)];
    final sp = Offset(6 + s.a * 10, 4 + s.b * 8);
    const cols = [_cw, _cb, _cg, _cp, _cr];
    for (var i = 7; i >= 0; i--) {
      final wob = Offset(math.sin(t * 2 + i) * .8, 0);
      _pl(c, [for (final q in ink) q + sp * i.toDouble() + wob], _al(cols[i % 5], math.pow(.75, i).toDouble()), w: i == 0 ? 1.4 : 1);
    }
    _lab(c, 'DRAW', const Offset(150, 106), _cd, ax: 1);
  }, a: .3, b: .4),

  // 20. Feedback past 100: drag up and the repeats grow instead of dying, until the screen clips.
  _Pn('runaway feedback', 'diagram·fx·drag·num·PUSHED', (c, s, t) {
    final fb = .5 + s.b * .75, over = fb > 1;
    var r = 6.0;
    for (var i = 0; i < 30; i++) {
      final a = math.pow(fb, i).toDouble().clamp(0.0, 1.0);
      if (a < .03) break;
      final wob = over ? math.sin(t * 8 + i) * (fb - 1) * 10 : 0.0;
      final p = Path();
      for (var j = 0; j <= 36; j++) {
        final th = j / 36 * 2 * math.pi, q = _pc + _pol(th, r + wob * math.sin(th * 5 + i));
        if (j == 0) {
          p.moveTo(q.dx, q.dy);
        } else {
          p.lineTo(q.dx, q.dy);
        }
      }
      c.drawPath(p, _sp(_al(over ? (i.isEven ? _cr : _cw) : _cb, a), .8));
      r += 4 + (over ? (fb - 1) * i * 1.5 : 0);
    }
    _num(c, '${(fb * 100).round()}', const Offset(152, 2), 30, over ? _cr : _cw, ax: 1);
    _lab(c, 'FDBK %', const Offset(152, 34), over ? _cr : _cw, ax: 1);
    if (over && (t * 4).floor().isEven) _tx(c, 'CLIP', const Offset(6, 6), 9, _cr, wght: 700, ls: 1);
  }, b: .3),
];
