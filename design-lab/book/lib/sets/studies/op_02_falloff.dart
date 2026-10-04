part of 'op_02.dart';

/// Line figure standing on [foot], [h] tall; arm / leg swing in radians.
void _man(Canvas c, Offset foot, double h, Color col, {double arm = .5, double leg = .35, bool sit = false, double w = 1.1}) {
  final p = _sp(col, w), hr = h * .13;
  final hip = foot - Offset(0, sit ? h * .25 : h * .45), neck = hip - Offset(0, h * .32), head = neck - Offset(0, hr);
  c.drawCircle(head, hr, p);
  c.drawLine(neck, hip, p);
  final sh = neck + Offset(0, h * .06);
  c.drawLine(sh, sh + _pol(math.pi / 2 + arm, h * .26), p);
  c.drawLine(sh, sh + _pol(math.pi / 2 - arm, h * .26), p);
  if (sit) {
    c.drawLine(hip, hip + Offset(h * .2, 0), p);
    c.drawLine(hip + Offset(h * .2, 0), foot + Offset(h * .2, 0), p);
  } else {
    c.drawLine(hip, hip + _pol(math.pi / 2 + leg, h * .47), p);
    c.drawLine(hip, hip + _pol(math.pi / 2 - leg, h * .47), p);
  }
}

Path _superellipse(Offset o, double r, double n, [int seg = 48]) {
  final pts = <Offset>[];
  for (var i = 0; i < seg; i++) {
    final th = i / seg * 2 * math.pi, cs = math.cos(th), sn = math.sin(th);
    pts.add(o +
        Offset(cs.sign * math.pow(cs.abs(), 2 / n).toDouble() * r, sn.sign * math.pow(sn.abs(), 2 / n).toDouble() * r));
  }
  return Path()..addPolygon(pts, true);
}

final _falloff = <_Pn>[
  // 1. Street lamp: the pool of light on the ground is the falloff; the moths stay inside it.
  _Pn('street lamp', 'landscape·fx·drag·pic·sane', (c, s, t) {
    const head = Offset(62, 26), gy = 104.0;
    final r = 18 + s.a * 100, k = _kx(s.b);
    _ln(c, const Offset(8, gy), const Offset(148, gy), _cd);
    _ln(c, const Offset(40, gy), const Offset(40, 20), _cw);
    _ln(c, const Offset(40, 20), const Offset(58, 20), _cw);
    _pl(c, const [Offset(55, 20), Offset(69, 20), Offset(66, 26), Offset(58, 26)], _cw, close: true);
    for (var x = 10.0; x <= 146; x += 4) {
      final f = _fall((x - head.dx).abs(), r, k);
      if (f > .02) {
        _ln(c, head, Offset(x, gy), _al(_cb, f * .55), .8);
        _ln(c, Offset(x, gy + 2), Offset(x, gy + 2 + f * 10), _cb, 1);
      }
    }
    for (var i = 0; i < 3; i++) {
      final m = head + Offset(math.sin(t * (2.1 + i) + i * 2) * (6 + r * .08), 8 + math.cos(t * (1.7 + i * .6) + i) * 5);
      _pl(c, [m + const Offset(-2, -1.5), m, m + const Offset(2, -1.5)], _cw, w: .9);
    }
    _num(c, _n2(r * .8), const Offset(148, 6), 28, _cb, ax: 1);
    _lab(c, 'REACH', const Offset(148, 36), _cb, ax: 1);
    _lab(c, 'CURVE ${k.toStringAsFixed(1)}', const Offset(148, 46), _cg, ax: 1);
  }, a: .45, b: .55),

  // 2. Isometric mesh lifted under the finger: the hill is the falloff, you place the summit.
  _Pn('iso hill', 'isometric·mech·drag·pic·sane', (c, s, t) {
    const o = Offset(78, 34), n = 10, cs = 7.0;
    final px = s.fx * n, py = s.fy * n, r = 2 + s.b * 5;
    double z(int i, int j) => _fall(math.sqrt((i - px) * (i - px) + (j - py) * (j - py)), r, 1.6) * 34;
    for (var i = 0; i <= n; i++) {
      for (var j = 0; j < n; j++) {
        final z0 = z(i, j), z1 = z(i, j + 1), z2 = z(j, i), z3 = z(j + 1, i);
        _ln(c, _iso(o, i * cs, j * cs, z0), _iso(o, i * cs, (j + 1) * cs, z1), _mx(_cd, _cg, (z0 + z1) / 50), .9);
        _ln(c, _iso(o, j * cs, i * cs, z2), _iso(o, (j + 1) * cs, i * cs, z3), _mx(_cd, _cg, (z2 + z3) / 50), .9);
      }
    }
    final top = _iso(o, px * cs, py * cs, 34), base = _iso(o, px * cs, py * cs, 0);
    _dots(c, base, top, _cr, 2.5);
    _dt(c, top, 1.8, _cr);
    _lab(c, 'X', const Offset(10, 70), _cb);
    _lab(c, 'Y', const Offset(140, 70), _cb);
    _num(c, _n2(r * 10), const Offset(148, 4), 22, _cg, ax: 1);
  }, b: .45, fx: .4, fy: .55),

  // 3. Draw the profile itself, near on the left, far on the right; knots with dotted verticals as in the OP-1 envelope.
  _Pn('drawn profile', 'diagram·mech·draw·diag·sane', (c, s, t) {
    const x0 = 14.0, x1 = 142.0, y0 = 98.0, y1 = 22.0;
    final hs = <double>[];
    for (var i = 0; i <= 16; i++) {
      final u = i / 16;
      double hgt = _fall(u, 1, _kx(s.b));
      if (s.ink.length > 4) {
        final x = x0 + u * (x1 - x0);
        var best = s.ink.first;
        for (final q in s.ink) {
          if ((q.dx - x).abs() < (best.dx - x).abs()) best = q;
        }
        hgt = _cl((y0 - best.dy) / (y0 - y1));
      }
      hs.add(hgt);
    }
    final pts = [for (var i = 0; i <= 16; i++) Offset(x0 + i / 16 * (x1 - x0), y0 - hs[i] * (y0 - y1))];
    _ln(c, const Offset(x0, y0), const Offset(x1, y0), _cb);
    _pl(c, pts, _cb);
    for (final i in [0, 4, 8, 12, 16]) {
      _dots(c, Offset(pts[i].dx, y0), pts[i], _cd);
      _dt(c, pts[i], 2, i == 0 ? _cb : _cg);
    }
    final u = (t * .25) % 1, j = (u * 16).floor().clamp(0, 15), fr = u * 16 - j;
    _ci(c, Offset.lerp(pts[j], pts[j + 1], fr)!, 3, _cw, .9);
    _lab(c, 'NEAR', const Offset(x0, y0 + 6), _cd);
    _lab(c, 'FAR', const Offset(x1, y0 + 6), _cd, ax: 1);
    var area = 0.0;
    for (final h in hs) {
      area += h / hs.length;
    }
    _num(c, _n2(area * 100), const Offset(148, 4), 24, _cw, ax: 1);
  }, b: .7),

  // 4. Spray can: rub to spray; the paint mist is dense at the target, thin at the rim.
  _Pn('spray can', 'machine·fx·rub·pic·sane', (c, s, t) {
    const tg = Offset(104, 62);
    final r = 12 + s.a * 36, k = _kx(s.b), cnt = 40 + (s.rub * 260).round();
    _rc(c, const Rect.fromLTWH(10, 58, 14, 38), _cw);
    _rc(c, const Rect.fromLTWH(12, 52, 10, 6), _cw);
    _ln(c, const Offset(22, 54), const Offset(27, 54), _cw);
    for (var i = -1; i <= 1; i++) {
      _dash(c, const Offset(29, 54), tg + Offset(-r * .7, i * r * .6), _al(_cd, 1), on: 1, off: 3);
    }
    for (var i = 0; i < cnt; i++) {
      final u = _h(i * 3 + 1), a = _h(i * 7 + 2) * 2 * math.pi;
      final d = r * (1 - math.pow(1 - u, 1 / (k + .2)).toDouble());
      _dt(c, tg + _pol(a, d), .7, _al(_cg, .4 + .6 * _fall(d, r, 1)));
    }
    _dash(c, tg - Offset(0, r), tg + Offset(0, r), _cd, on: 1, off: 3);
    _num(c, _n2(s.rub * 99), const Offset(148, 4), 26, _cr, ax: 1);
    _lab(c, 'PRESSURE', const Offset(148, 32), _cr, ax: 1);
    _lab(c, 'RUB', const Offset(8, 8), _cd);
  }, a: .5, b: .5),

  // 5. Loudspeaker and an ear you carry: the number is what the ear hears at that distance.
  _Pn('ear walk', 'instrument·fx:listener·drag·num·sane', (c, s, t) {
    const sp = Offset(18, 70);
    final r = 30 + s.b * 110, ex = 34 + s.fx * 112, f = _fall(ex - sp.dx, r, 1.4);
    _rc(c, const Rect.fromLTWH(6, 50, 22, 40), _cw);
    _ci(c, const Offset(17, 60), 4, _cw);
    _ci(c, const Offset(17, 78), 7, _cw);
    for (var i = 0; i < 9; i++) {
      final rr = 14 + i * 14 + (t * 14) % 14, g = _fall(rr, r, 1.4);
      if (g > 0) {
        c.drawArc(Rect.fromCircle(center: sp, radius: rr), -.5, 1, false, _sp(_al(_cb, g), 1));
      }
    }
    final e = Offset(ex, 72);
    c.drawArc(Rect.fromCircle(center: e, radius: 7), math.pi, math.pi * 1.4, false, _sp(_cg, 1.2));
    _ln(c, e + _pol(math.pi * .4, 7), e + const Offset(-1, 14), _cg);
    c.drawArc(Rect.fromCircle(center: e + const Offset(0, 1), radius: 3), math.pi, math.pi, false, _sp(_cg, 1));
    _dots(c, Offset(ex, 88), Offset(ex, 104), _cd);
    _ln(c, const Offset(6, 104), const Offset(150, 104), _cd);
    _num(c, _n2(f * 99), Offset(ex.clamp(26.0, 128.0), 12), 34, _cw, ax: .5);
  }, b: .5, fx: .5),

  // 6. Campfire: pinch the fire; people near it sweat, those beyond the reach shiver.
  _Pn('campfire circle', 'character·fx:people·pinch·pic·whimsy', (c, s, t) {
    const fire = Offset(78, 96);
    final r = 14 + s.z * 70;
    _ln(c, const Offset(66, 100), const Offset(90, 94), _cw);
    _ln(c, const Offset(66, 94), const Offset(90, 100), _cw);
    for (var i = 0; i < 3; i++) {
      final hh = (10 + s.z * 16) * (i == 1 ? 1 : .65) * (1 + .12 * math.sin(t * 9 + i * 2));
      final b0 = fire + Offset((i - 1) * 5.0, 0);
      final path = Path()
        ..moveTo(b0.dx - 4, b0.dy - 2)
        ..quadraticBezierTo(b0.dx - 5, b0.dy - hh * .5, b0.dx + math.sin(t * 6 + i) * 2, b0.dy - hh)
        ..quadraticBezierTo(b0.dx + 5, b0.dy - hh * .5, b0.dx + 4, b0.dy - 2);
      c.drawPath(path, _sp(i == 1 ? _cr : _cp, 1.1));
    }
    _dash(c, fire - Offset(r, 0), fire + Offset(r, 0), _cd, on: 1, off: 3);
    for (final x in [14.0, 34.0, 52.0, 104.0, 122.0, 142.0]) {
      final f = _fall((x - fire.dx).abs(), r, 1.3), foot = Offset(x, 104);
      _man(c, foot, 22, _cw, sit: true, arm: f > .4 ? 1.2 : .25);
      final hd = foot - const Offset(0, 25);
      if (f > .45) {
        for (var j = 0; j < 2; j++) {
          final q = hd + Offset(-3.0 + j * 6, -8);
          _pl(c, [for (var k = 0; k < 5; k++) q + Offset(math.sin(t * 5 + k + j) * 1.2, -k * 2.0)], _cr, w: .8);
        }
      } else if (f < .08) {
        final q = hd + const Offset(5, 4);
        _pl(c, [for (var k = 0; k < 5; k++) q + Offset(k.isEven ? 0 : 2.5 + math.sin(t * 30) * .6, k * 2.0)], _cb, w: .8);
      } else {
        _dt(c, hd + const Offset(0, -6), 1, _cg);
      }
    }
    _num(c, _n2(r * .9), const Offset(8, 4), 24, _cr);
    _lab(c, 'WARMTH', const Offset(8, 30), _cr);
  }, z: .45),

  // 7. Typographic: a giant curve exponent; spin around it to bend the dot row from soft to hard.
  _Pn('spun exponent', 'typographic·mech·spin·num·sane', (c, s, t) {
    final k = math.exp((s.spin * .5).clamp(-1.39, 1.39));
    _num(c, k.toStringAsFixed(1), const Offset(78, 14), 46, _cw, ax: .5);
    _lab(c, 'CURVE', const Offset(78, 62), _cd, ax: .5);
    for (var i = 0; i < 40; i++) {
      final a = i / 40 * 2 * math.pi;
      _dt(c, const Offset(78, 38) + Offset(math.cos(a) * 58, math.sin(a) * 30), .5, _cd);
    }
    _dt(c, const Offset(78, 38) + Offset(math.cos(s.spin - math.pi / 2) * 58, math.sin(s.spin - math.pi / 2) * 30), 2.2, _cg);
    for (var i = 0; i <= 20; i++) {
      final f = _fall((i - 10).abs().toDouble(), 11, k);
      _dt(c, Offset(12 + i * 6.6, 96), .6 + f * 3, _mx(_cd, _cg, f));
    }
    _ln(c, const Offset(10, 106), const Offset(146, 106), _cd, .8);
  }),

  // 8. Magnet over iron filings: only the filings inside the reach turn to face it.
  _Pn('magnet filings', 'machine·fx·drag·pic·sane', (c, s, t) {
    final m = Offset(12 + s.fx * 132, 12 + s.fy * 96), r = 24 + s.b * 80;
    for (var j = 0; j < 9; j++) {
      for (var i = 0; i < 13; i++) {
        final q = Offset(10 + i * 11.3, 10 + j * 12.5), d = (q - m).distance;
        final f = _fall(d, r, 1.2), a0 = _h(i * 31 + j) * math.pi, a1 = (m - q).direction;
        final a = a0 + (math.atan2(math.sin(a1 - a0), math.cos(a1 - a0))) * f;
        final u = _pol(a, 2.5 + f * 1.5);
        _ln(c, q - u, q + u, _mx(_cd, _cw, f * 1.4), 1);
      }
    }
    final p = _sp(_cw, 1.3);
    c.drawArc(Rect.fromCircle(center: m, radius: 7), math.pi, math.pi, false, p);
    _ln(c, m + const Offset(-7, 0), m + const Offset(-7, 6), _cr, 2);
    _ln(c, m + const Offset(7, 0), m + const Offset(7, 6), _cb, 2);
    _ci(c, m, r, _al(_cd, .8), .6);
  }, b: .4, fx: .55, fy: .45),

  // 9. Telescope lens you throw across the sky; stars under it swell by the falloff profile.
  _Pn('telescope lens', 'cosmic·fx:sample·flick·pic·whimsy', (c, s, t) {
    final l = Offset(26 + s.fx * 104, 24 + s.fy * 64);
    const lr = 26.0;
    final k = _kx(s.b);
    for (var i = 0; i < 70; i++) {
      final q = Offset(_h(i) * _pw, _h(i + 99) * _ph), d = (q - l).distance;
      final tw = .5 + .3 * math.sin(t * 3 + i);
      if (d < lr) {
        final f = _fall(d, lr, k), qq = l + (q - l) * (1 + f * (.2 + s.a * 1.2));
        _ci(c, qq, .8 + f * 3, _cw, .8);
      } else {
        _dt(c, q, tw, _cd);
      }
    }
    _ci(c, l, lr, _cb, 1.2);
    _ci(c, l, lr + 4, _cb, .8);
    final dir = (const Offset(160, 130) - l);
    final u = dir / dir.distance, nrm = Offset(-u.dy, u.dx);
    _ln(c, l + nrm * (lr + 4), l + nrm * 10 + u * 200, _cd);
    _ln(c, l - nrm * (lr + 4), l - nrm * 10 + u * 200, _cd);
    _num(c, 'x${(1.2 + s.a * 1.2).toStringAsFixed(1)}', const Offset(6, 4), 18, _cg);
    _lab(c, 'CURVE ${k.toStringAsFixed(1)}', const Offset(6, 26), _cg);
  }, fx: .4, fy: .4),

  // 10. Trampoline: throw the jumper up; the sag of the net is the falloff shape.
  _Pn('trampoline', 'character·mech·flick·pic·whimsy', (c, s, t) {
    const y = 88.0, cx = 78.0;
    final w = 14 + s.a * 50, k = _kx(s.b);
    final ph = (math.sin(t * 3.4)).abs(), hi = ph * (4 + s.kick * 60), dep = math.pow(1 - ph, 3) * (3 + s.kick * 16) + 1.5;
    final net = [for (var x = 16.0; x <= 140; x += 3) Offset(x, y + dep * _fall((x - cx).abs(), w, k))];
    _pl(c, net, _cw);
    for (final x in [16.0, 140.0]) {
      _ln(c, Offset(x, y), Offset(x + (x < cx ? -4 : 4), 106), _cw);
    }
    _ln(c, const Offset(8, 106), const Offset(148, 106), _cd);
    _dots(c, Offset(cx - w, y + 2), Offset(cx - w, 104), _cb);
    _dots(c, Offset(cx + w, y + 2), Offset(cx + w, 104), _cb);
    _man(c, Offset(cx, y + dep - hi), 24, _cg, arm: 1.3 + ph * .9, leg: .15 + ph * .3);
    _lab(c, 'WIDTH', const Offset(8, 6), _cb);
    _num(c, _n2(w), const Offset(8, 14), 20, _cb);
    _lab(c, 'CURVE', const Offset(148, 6), _cg, ax: 1);
    _num(c, k.toStringAsFixed(1), const Offset(148, 14), 20, _cg, ax: 1);
  }, a: .4, b: .5),

  // 11. Car past a radio tower: the dashboard number is the signal at the car's distance.
  _Pn('radio drive', 'vehicle·fx:time·drag·num·whimsy', (c, s, t) {
    const gy = 98.0, tx = 92.0;
    final r = 12 + s.b * 90, k = _kx(s.a);
    for (var i = 0; i < 9; i++) {
      final x = 4 + i * 17.0, hh = 6 + _h(i + 5) * 16;
      _pl(c, [Offset(x, 80), Offset(x, 80 - hh), Offset(x + 12, 80 - hh), Offset(x + 12, 80)], _cd, w: .8);
    }
    _ln(c, const Offset(0, gy), const Offset(156, gy), _cw);
    _pl(c, const [Offset(tx - 8, gy), Offset(tx, 36), Offset(tx + 8, gy)], _cw, w: 1);
    for (var y = 46.0; y < gy; y += 10) {
      final hw = (y - 36) / (gy - 36) * 8;
      _ln(c, Offset(tx - hw, y), Offset(tx + hw * .8, y + 10), _cd, .8);
    }
    for (var i = 0; i < 4; i++) {
      final rr = 6 + ((t * 20 + i * 16) % 64);
      c.drawArc(Rect.fromCircle(center: const Offset(tx, 36), radius: rr), -2.4, 1.6, false,
          _sp(_al(_cb, _fall(rr, r, k)), 1));
    }
    final cx = ((t * 22) % 190) - 20, f = _fall((cx - tx).abs(), r, k);
    final car = Path()
      ..moveTo(cx - 12, gy - 3)
      ..lineTo(cx - 12, gy - 8)
      ..lineTo(cx - 6, gy - 8)
      ..lineTo(cx - 3, gy - 13)
      ..lineTo(cx + 6, gy - 13)
      ..lineTo(cx + 9, gy - 8)
      ..lineTo(cx + 12, gy - 7)
      ..lineTo(cx + 12, gy - 3);
    c.drawPath(car, _sp(_cg, 1.1));
    _ci(c, Offset(cx - 7, gy - 2), 2.2, _cg, 1);
    _ci(c, Offset(cx + 7, gy - 2), 2.2, _cg, 1);
    _ln(c, Offset(cx + 4, gy - 13), Offset(cx + 2, gy - 24), _cg, .8);
    _pl(c, [for (var i = 0; i < 8; i++) Offset(cx + 2 + i * 1.5, gy - 26 + math.sin(t * 20 + i) * f * 3)], _cg, w: .8);
    _num(c, _n2(f * 99), const Offset(6, 2), 36, _cg);
    _lab(c, 'SIGNAL', const Offset(8, 40), _cg);
    _lab(c, 'REACH', const Offset(150, 6), _cb, ax: 1);
  }, a: .5, b: .5),

  // 12. Contour map: pinch the hill steeper or flatter; the contour spacing is the curve.
  _Pn('contour hill', 'landscape·mech·pinch·pic·sane', (c, s, t) {
    const o = Offset(70, 62);
    final rr = 30 + s.a * 30, k = _kx(s.z);
    for (var i = 1; i <= 8; i++) {
      final lv = i / 9, rad = rr * (1 - math.pow(lv, 1 / k).toDouble());
      final pts = [
        for (var j = 0; j < 56; j++)
          o + _pol(j / 56 * 2 * math.pi, rad * (1 + .14 * math.sin(3 * j / 56 * 2 * math.pi + i * .4) + .05 * math.sin(7 * j / 56 * 6.28)))
      ];
      _pl(c, pts, i >= 7 ? _cg : (i.isEven ? _cw : _cd), w: .9, close: true);
    }
    _pl(c, [o + const Offset(-3, 2), o + const Offset(0, -3), o + const Offset(3, 2)], _cr, close: true);
    _ln(c, o + const Offset(0, -3), o + const Offset(0, -11), _cr, .9);
    _pl(c, [o + const Offset(0, -11), o + const Offset(5, -9), o + const Offset(0, -7)], _cr, w: .9);
    _num(c, k.toStringAsFixed(1), const Offset(150, 4), 22, _cg, ax: 1);
    _lab(c, 'STEEP', const Offset(150, 28), _cg, ax: 1);
  }, a: .6, z: .5),

  // 13. Sheepdog: the flock parts around the dog by how close each sheep is.
  _Pn('sheepdog', 'animal·fx:crowd·drag·pic·whimsy', (c, s, t) {
    final dog = Offset(14 + s.fx * 128, 16 + s.fy * 88), r = 20 + s.b * 60;
    for (var i = 0; i < 11; i++) {
      final home = Offset(18 + _h(i * 5) * 120, 18 + _h(i * 9 + 3) * 84), d = (home - dog).distance;
      final f = _fall(d, r, 1.3), away = d < .1 ? Offset.zero : (home - dog) / d;
      final q = home + away * f * 22 + Offset(0, math.sin(t * 2 + i) * .6);
      final body = [for (var j = 0; j < 18; j++) q + _pol(j / 18 * 2 * math.pi, 5 + (j.isEven ? .9 : 0)).scale(1, .7)];
      _pl(c, body, f > .3 ? _cw : _al(_cw, .6), w: .9, close: true);
      final hd = q + Offset(away.dx >= 0 ? 6 : -6, -1);
      _ci(c, hd, 1.8, _cw, .9);
      _ln(c, q + const Offset(-2, 3), q + const Offset(-2, 6), _cw, .8);
      _ln(c, q + const Offset(2, 3), q + const Offset(2, 6), _cw, .8);
      if (f > .45) _lab(c, '!', q + const Offset(0, -12), _cr, ax: .5);
    }
    _ci(c, dog, r, _al(_cd, .9), .6);
    final bp = _sp(_cr, 1.2);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: dog, width: 12, height: 5), const Radius.circular(2.5)), bp);
    _ci(c, dog + const Offset(8, -3), 2.6, _cr, 1.1);
    _ln(c, dog + const Offset(-6, -1), dog + Offset(-10, -5 + math.sin(t * 14) * 2), _cr);
    _ln(c, dog + const Offset(-4, 2.5), dog + const Offset(-4, 6), _cr, .9);
    _ln(c, dog + const Offset(4, 2.5), dog + const Offset(4, 6), _cr, .9);
  }, b: .45, fx: .45, fy: .5),

  // 14. Black hole: spin to feed it; the grid is pulled in with the falloff, past 100 it swallows the panel.
  _Pn('black hole', 'cosmic·fx:space·spin·pic·PUSHED', (c, s, t) {
    final g = (.35 + s.spin * .09).clamp(0.0, 1.6), k = _kx(s.b);
    Offset warp(Offset q) {
      final d = (q - _pc).distance;
      return _pc + (q - _pc) * (1 - math.min(.97, g * _fall(d, 80, k)));
    }

    for (var y = 0.0; y <= _ph; y += 10) {
      _pl(c, [for (var x = 0.0; x <= _pw; x += 6) warp(Offset(x, y))], _mx(_cd, _cp, g * .6), w: .8);
    }
    for (var x = 0.0; x <= _pw; x += 10) {
      _pl(c, [for (var y = 0.0; y <= _ph; y += 6) warp(Offset(x, y))], _mx(_cd, _cp, g * .6), w: .8);
    }
    final rr = 3 + g * 10;
    _ci(c, _pc, rr, _cr, 1.3);
    for (var i = 0; i < 18; i++) {
      final a = i / 18 * 2 * math.pi + t * (1 + g * 3);
      final p1 = _pc + Offset(math.cos(a) * rr * 2.2, math.sin(a) * rr * .7);
      final p2 = _pc + Offset(math.cos(a + .18) * rr * 2.2, math.sin(a + .18) * rr * .7);
      _ln(c, p1, p2, _cw, .9);
    }
    _num(c, _n2(g * 62).padLeft(2, '0'), const Offset(150, 2), 26, g > 1 ? _cr : _cw, ax: 1);
    _lab(c, 'PULL', const Offset(150, 28), _cw, ax: 1);
  }),

  // 15. Max for Live style chain: source -> amount -> shape -> the eight targets it reaches.
  _Pn('device chain', 'diagram·mech·drag·diag·sane', (c, s, t) {
    final k = _kx(s.b);
    _rc(c, const Rect.fromLTWH(6, 42, 22, 22), _cw);
    _dt(c, const Offset(17, 53), 2.5, _cw);
    _pl(c, const [Offset(36, 36), Offset(66, 53), Offset(36, 70)], _cg, close: true);
    _num(c, _n2(s.a * 99), const Offset(39, 53), 10, _cg, ay: .5);
    _rc(c, const Rect.fromLTWH(74, 42, 24, 22), _cb);
    _pl(c, [for (var i = 0; i <= 10; i++) Offset(77 + i * 1.8, 61 - 16 * _fall(i / 10, 1, k))], _cb, w: 1);
    const wc = Offset(128, 53);
    _ci(c, wc, 18, _cw);
    for (var i = 0; i < 8; i++) {
      final f = _fall(i / 8, 1, k) * s.a;
      _dt(c, wc + _pol(-math.pi / 2 + i * math.pi / 4, 12), .7 + f * 3, _mx(_cd, i == 0 ? _cr : _cg, .3 + f));
    }
    for (final seg in const [[Offset(28, 53), Offset(38, 53)], [Offset(64, 53), Offset(74, 53)], [Offset(98, 53), Offset(110, 53)]]) {
      _ln(c, seg[0], seg[1], _cd);
    }
    final u = (t * .6) % 1;
    _dt(c, Offset(28 + u * 82, 53), 1.4, _cw);
    _lab(c, 'SRC', const Offset(17, 74), _cw, ax: .5);
    _lab(c, 'AMOUNT', const Offset(48, 80), _cg, ax: .5);
    _lab(c, 'SHAPE', const Offset(86, 74), _cb, ax: .5);
    _lab(c, 'DEST', const Offset(128, 80), _cw, ax: .5);
  }, a: .6, b: .6),

  // 16. The word itself swells where you rub; rub harder and the swell spreads.
  _Pn('swelling word', 'typographic·fx:text·rub·type·whimsy', (c, s, t) {
    const word = 'FALLOFF';
    final r = 26 + s.rub * 110, fx = s.p.dx;
    final hs = <Offset>[];
    for (var i = 0; i < word.length; i++) {
      final cx = 12 + i * 20.0, f = _fall((cx - fx).abs(), r, 1.4);
      final size = (10 + f * 26).roundToDouble(), wg = (100 + (f * 6).round() * 100).toDouble();
      _tx(c, word[i], Offset(cx, 70), size, _mx(_cd, _cg, f + .15), wght: wg, ax: .5, ay: 1);
      hs.add(Offset(cx, 96 - f * 14));
    }
    _pl(c, hs, _cb, w: .9);
    for (final q in hs) {
      _dt(c, q, 1.2, _cb);
    }
    _num(c, _n2(r * .7), const Offset(150, 4), 20, _cr, ax: 1);
    _lab(c, 'SPREAD', const Offset(150, 24), _cr, ax: 1);
  }),

  // 17. Seismograph: throw the quake, every station records it as strongly as its distance allows.
  _Pn('seismograph', 'machine·fx:readout·flick·diag·sane', (c, s, t) {
    final ep = 10 + s.fx * 136, r = 30 + s.a * 100, mag = .15 + s.kick * .85 + .1 * s.b;
    _ln(c, const Offset(6, 30), const Offset(150, 30), _cd);
    for (var i = 0; i < 6; i++) {
      final a = i * math.pi / 3;
      _ln(c, Offset(ep, 30) - _pol(a, 5), Offset(ep, 30) + _pol(a, 5), _cr, 1);
    }
    const cols = [_cb, _cg, _cw, _cp];
    for (var i = 0; i < 4; i++) {
      final sx = 22 + i * 38.0, f = _fall((sx - ep).abs(), r, 1.2), ly = 50 + i * 16.0;
      _pl(c, [Offset(sx - 4, 30), Offset(sx - 4, 24), Offset(sx, 20), Offset(sx + 4, 24), Offset(sx + 4, 30)], cols[i], w: .9);
      _ln(c, const Offset(6, 0) + Offset(0, ly), Offset(150, ly), _al(_cd, .5), .5);
      _pl(c, [
        for (var x = 6.0; x <= 150; x += 2)
          Offset(x, ly + math.sin(x * .6 - t * 14 + i) * f * mag * 7 * (.5 + .5 * math.sin(x * .07 + t)))
      ], cols[i], w: .9);
    }
    _num(c, (mag * 9).toStringAsFixed(1), const Offset(150, 2), 18, _cr, ax: 1);
    _lab(c, 'MAG', const Offset(6, 4), _cr);
  }, a: .5, fx: .3),

  // 18. Wine glasses: rub one rim, its neighbours ring less the further they stand.
  _Pn('singing glasses', 'instrument·fx:near·rub·pic·whimsy', (c, s, t) {
    final g0 = ((s.p.dx - 10) / 24).round().clamp(0, 5), k = _kx(s.b), r = 1.2 + s.a * 5;
    for (var i = 0; i < 6; i++) {
      final x = 16 + i * 24.8, f = _fall((i - g0).abs().toDouble(), r, k) * (.15 + s.rub);
      final bowl = Path()
        ..moveTo(x - 8, 58)
        ..quadraticBezierTo(x - 8, 82, x, 82)
        ..quadraticBezierTo(x + 8, 82, x + 8, 58);
      c.drawPath(bowl, _sp(_cw, 1));
      _ln(c, Offset(x, 82), Offset(x, 96), _cw, 1);
      _ln(c, Offset(x - 6, 96), Offset(x + 6, 96), _cw, 1);
      _pl(c, [for (var j = 0; j <= 10; j++) Offset(x - 6.5 + j * 1.3, 68 + math.sin(t * 30 + j * 1.4) * f * 2.5)], _cb, w: .8);
      for (var j = 1; j <= 3; j++) {
        if (f * 3 > j - 1) {
          c.drawArc(Rect.fromCircle(center: Offset(x, 56), radius: 4.0 + j * 4 + (t * 8) % 4), -2.4, 1.6, false,
              _sp(_al(_cg, f), .8));
        }
      }
    }
    _ci(c, Offset(16 + g0 * 24.8 + 8 * math.cos(t * 7), 58), 2.5, _cr, 1);
    _ln(c, const Offset(4, 97), const Offset(152, 97), _cd);
    _num(c, _n2(s.rub * 99), const Offset(6, 2), 24, _cr);
    _lab(c, 'RING', const Offset(6, 28), _cr);
  }, a: .4, b: .5),

  // 19. Footprint shape: pinch from diamond through circle to square; contour spacing shows the curve.
  _Pn('footprint shape', 'diagram·mech·pinch·diag·M4L', (c, s, t) {
    final n = math.pow(2, (s.z - .5) * 4.4).toDouble() * 2, k = _kx(s.a);
    const o = Offset(62, 58);
    for (var i = 1; i <= 6; i++) {
      final lv = i / 7, rad = 44 * (1 - math.pow(lv, 1 / k).toDouble());
      c.drawPath(_superellipse(o, rad, n), _sp(i == 1 ? _cb : _mx(_cd, _cb, lv * .8), i == 1 ? 1.2 : .9));
    }
    _ln(c, o - const Offset(4, 0), o + const Offset(4, 0), _cw, .8);
    _ln(c, o - const Offset(0, 4), o + const Offset(0, 4), _cw, .8);
    _num(c, n.toStringAsFixed(1), const Offset(150, 4), 22, _cb, ax: 1);
    _lab(c, 'SHAPE', const Offset(150, 28), _cb, ax: 1);
    final act = n < 1.5 ? 0 : (n < 3.5 ? 1 : 2);
    const ic = [Offset(124, 96), Offset(136, 96), Offset(148, 96)];
    for (var i = 0; i < 3; i++) {
      final col = i == act ? _cw : _cd;
      if (i == 0) _pl(c, [ic[0] + const Offset(0, -4), ic[0] + const Offset(4, 0), ic[0] + const Offset(0, 4), ic[0] + const Offset(-4, 0)], col, close: true);
      if (i == 1) _ci(c, ic[1], 4, col);
      if (i == 2) _rc(c, Rect.fromCenter(center: ic[2], width: 7, height: 7), col);
    }
    _lab(c, 'CURVE ${k.toStringAsFixed(1)}', const Offset(150, 34), _cg, ax: 1);
  }, z: .5, a: .5),

  // 20. Sun pushed past sense: drag right and the rays outrun the frame, curl, and the number overflows.
  _Pn('sun overdrive', 'cosmic·fx·drag·num·PUSHED', (c, s, t) {
    final a = s.a, len = 8 + a * a * 260, curl = a * a * 2.2;
    const cols = [_cb, _cg, _cw, _cr];
    for (var i = 0; i < 28; i++) {
      final a0 = i / 28 * 2 * math.pi + t * .2;
      final pts = <Offset>[];
      for (var j = 0; j <= 12; j++) {
        final u = j / 12, rr = 14 + u * len * (.7 + .3 * _h(i));
        pts.add(_pc + _pol(a0 + curl * u * u, rr));
      }
      _pl(c, pts, a > .75 ? cols[i % 4] : _cw, w: .9);
    }
    _ci(c, _pc, 11, _cw, 1.3);
    _dt(c, _pc + const Offset(-3, -2), 1, _cw);
    _dt(c, _pc + const Offset(3, -2), 1, _cw);
    c.drawArc(Rect.fromCircle(center: _pc + const Offset(0, 1), radius: 4), .3, math.pi - .6, false, _sp(_cw, 1));
    final v = (a * a * 999).round();
    _num(c, '$v', const Offset(152, 122), 30 + a * 40, a > .75 ? _cr : _cw, ax: 1, ay: 1);
    _lab(c, 'POWER', const Offset(6, 6), _cw);
  }, a: .55),
];
