part of 'op_10.dart';

// Position / Transform of a layer (x, y, z, anchor) as a world you walk the layer through, never a bare XY pad.
// a = x (or the first axis named on the panel), b = y / z / height.

List<_Op> _posPanels() => [
      _Op('iso room', 'isometric object · mechanism · drag · drawing led', _G.drag, _pIsoRoom, a: .4, b: .6),
      _Op('stage walk', 'character · effect · drag · drawing led', _G.drag, _pStage, a: .35, b: .4),
      _Op('road trip', 'vehicle · effect (z as distance) · drag', _G.drag, _pRoad, a: .62, b: .45),
      _Op('odometer x/y', 'typographic · readout · flick · numeral led', _G.flick, _pOdo, a: .32, b: .71),
      _Op('contour pin', 'landscape · effect · drag · drawing led', _G.drag, _pTopo, a: .62, b: .58),
      _Op('pivot swing', 'diagram · anchor effect · drag', _G.drag, _pPivot, a: .2, b: .8),
      _Op('aquarium', 'animal · isometric · drag (x, depth)', _G.drag, _pTank, a: .6, b: .35),
      _Op('parallax stars', 'cosmic · z effect · pinch', _G.pinch, _pStars, a: .5, b: .35),
      _Op('tower crane', 'machine · mechanism · drag', _G.drag, _pCrane, a: .55, b: .45),
      _Op('layer stack', 'isometric · z order · drag · numeral', _G.drag, _pStack, a: .5, b: .55),
      _Op('sun & shadow', 'landscape · z as height · drag', _G.drag, _pShadow, a: .45, b: .5),
      _Op('tape measures', 'instrument · mechanism · flick', _G.flick, _pTape, a: .55, b: .5),
      _Op('footpath', 'character · motion path · draw', _G.draw, _pPath),
      _Op('billiard', 'game · flick with momentum · pushed', _G.flick, _pBilliard, a: .3, b: .6),
      _Op('elevator', 'machine · numeral led · drag (y = floor)', _G.drag, _pLift, a: .5, b: .6),
      _Op('depth tunnel', 'cosmic · z effect · drag · pushed', _G.drag, _pTunnel, a: .5, b: .4),
      _Op('viewfinder', 'instrument · composition effect · drag', _G.drag, _pFinder, a: .32, b: .65),
      _Op('loupe nudge', 'instrument · sub-pixel · rub · pushed', _G.rub, _pLoupe, a: .59, b: 0, fine: .08),
      _Op('anchor ship', 'vehicle metaphor · anchor point · drag', _G.drag, _pShip, a: .45, b: .5),
      _Op('globe pin', 'cosmic · spin · pushed to the extreme', _G.spin, _pGlobe, a: .1, b: .5),
    ];

void _xy(Canvas c, _S st, {String x = 'X', String y = 'Y', double sx = 1920, double sy = 1080, double top = 6}) {
  _lab(c, x, Offset(_cw - 6, top), _bl, ax: 1);
  _num(c, _d3(st.a * sx), Offset(_cw - 6, top + 9), 15, _bl, ax: 1);
  _lab(c, y, Offset(_cw - 6, top + 28), _gr, ax: 1);
  _num(c, _d3(st.b * sy), Offset(_cw - 6, top + 37), 15, _gr, ax: 1);
}

void _pIsoRoom(Canvas c, _S st, double t) {
  const o = Offset(62, 26), k = 7.4;
  for (var i = 0; i <= 8; i++) {
    _ln(c, _iso(o, i * 1.0, 0, 0, k), _iso(o, i * 1.0, 8, 0, k), _dg, .8);
    _ln(c, _iso(o, 0, i * 1.0, 0, k), _iso(o, 8, i * 1.0, 0, k), _dg, .8);
  }
  final x = st.a * 7, y = (1 - st.b) * 7, z = 1.4 + math.sin(t * 2) * .25;
  _pl(c, [_iso(o, x, y, 0, k), _iso(o, x + 1, y, 0, k), _iso(o, x + 1, y + 1, 0, k), _iso(o, x, y + 1, 0, k)], _bl, close: true);
  _dots(c, _iso(o, x + .5, y + .5, 0, k), _iso(o, x + .5, y + .5, z, k), _mg);
  _ln(c, _iso(o, x + .5, 0, 0, k), _iso(o, x + .5, y, 0, k), _al(_bl, .5), .8);
  _ln(c, _iso(o, 0, y + .5, 0, k), _iso(o, x, y + .5, 0, k), _al(_gr, .5), .8);
  _isoBox(c, o, x, y, z, 1, 1, 1.3, k, _gr, 1.2);
  _lab(c, 'x', _iso(o, 4, -1.2, 0, k), _bl);
  _lab(c, 'y', _iso(o, -1.6, 4, 0, k), _gr);
  _lab(c, 'z', Offset(6, 6), _mg);
  _num(c, _d2(z * 10), const Offset(6, 14), 14, _wh);
  _xy(c, st, top: 60);
}

void _pStage(Canvas c, _S st, double t) {
  const fy = 104.0, by = 48.0;
  _pl(c, const [Offset(8, fy), Offset(148, fy), Offset(118, by), Offset(38, by)], _mg, close: true);
  for (var i = 1; i < 4; i++) {
    final yy = _lp(by, fy, i / 4);
    _dots(c, Offset(_lp(38, 8, i / 4), yy), Offset(_lp(118, 148, i / 4), yy), _dg, gap: 4);
  }
  for (final side in [-1.0, 1.0]) {
    final x0 = side < 0 ? 4.0 : 152.0;
    for (var j = 0; j < 3; j++) {
      final p = Path()..moveTo(x0 - side * j * 5, 4);
      for (var yy = 4.0; yy <= fy; yy += 6) {
        p.lineTo(x0 - side * (j * 5 + 2 + math.sin(yy * .3 + j + t) * 1.2), yy);
      }
      c.drawPath(p, _s(_al(_rd, .7), .9));
    }
  }
  final d = st.b, y = _lp(fy - 6, by + 6, d), xl = _lp(14, 42, d), xr = _lp(142, 114, d);
  final x = _lp(xl, xr, st.a), k = _lp(1, .55, d);
  c.drawOval(Rect.fromCenter(center: Offset(x, y), width: 30 * k, height: 8 * k), _s(_al(_wh, .5), .8));
  _ln(c, const Offset(78, 0), Offset(x - 15 * k, y), _al(_wh, .25), .8);
  _ln(c, const Offset(78, 0), Offset(x + 15 * k, y), _al(_wh, .25), .8);
  final sw = math.sin(t * 3);
  _stick(c, Offset(x, y), 40 * k, _wh, armL: 2.2 + sw * .5, armR: -2.2 + sw * .5, legL: .3, legR: -.3);
  _lab(c, 'x', const Offset(16, 108), _bl);
  _num(c, _d2(st.a * 99), const Offset(24, 106), 11, _bl);
  _lab(c, 'depth', const Offset(112, 108), _gr, ax: 1);
  _num(c, _d2(d * 99), const Offset(116, 106), 11, _gr);
}

void _pRoad(Canvas c, _S st, double t) {
  const vp = Offset(78, 46);
  for (var i = 0; i < 12; i++) {
    final h = 4 + _rn(i) * 14, x = 22 + i * 9.5;
    _rect(c, Rect.fromLTWH(x, vp.dy - h, 7, h), _al(_pu, .8), .8);
  }
  _ln(c, const Offset(0, 46), const Offset(_cw, 46), _bl, .8);
  _ln(c, vp, const Offset(-30, 120), _wh, 1);
  _ln(c, vp, const Offset(186, 120), _wh, 1);
  for (var i = 0; i < 6; i++) {
    final f0 = _wr(i / 6 + t * .35), f1 = f0 + .05;
    _ln(c, Offset(78, vp.dy + f0 * f0 * 74), Offset(78, vp.dy + f1 * f1 * 74), _mg, 1);
  }
  final f = _lp(1, .08, st.b), cy = vp.dy + f * 60, cx = 78 + (st.a - .5) * 2 * f * 90, h = 30 * f;
  _ln(c, Offset(cx, cy), Offset(cx, cy - h * .4), _gr, 1);
  _rect(c, Rect.fromCenter(center: Offset(cx, cy - h * .8), width: h * 1.4, height: h * .8), _gr);
  _num(c, 'L', Offset(cx, cy - h * 1.15), h * .6, _gr, ax: .5);
  final p = Path()..moveTo(0, 104)..quadraticBezierTo(78, 88, _cw, 104);
  c.drawPath(p, _s(_wh, 1));
  _ring(c, const Offset(78, 114), 10, _mg, 1);
  _lab(c, 'lane', const Offset(6, 6), _bl);
  _num(c, _d2(st.a * 99), const Offset(6, 15), 18, _bl);
  _lab(c, 'dist', const Offset(150, 6), _gr, ax: 1);
  _num(c, _d2(st.b * 99), const Offset(150, 15), 18, _gr, ax: 1);
}

void _pOdo(Canvas c, _S st, double t) {
  void row(double y, double v, String l, Color col) {
    _lab(c, l, Offset(8, y + 4), col);
    _num(c, _d3(v * 999), Offset(20, y - 4), 36, col);
    for (var i = -12; i <= 30; i++) {
      final x = 20 + i * 5 - _wr(v * 20) * 5;
      if (x < 20 || x > 148) continue;
      _ln(c, Offset(x, y + 36), Offset(x, y + (i % 5 == 0 ? 31 : 33.5)), _mg, .8);
    }
    _ln(c, Offset(84, y + 29), Offset(84, y + 38), col, 1.2);
  }

  row(6, st.a, 'x', _bl);
  row(60, st.b, 'y', _gr);
  c.drawRect(const Rect.fromLTWH(124, 8, 24, 16), _s(_dg, .8));
  _dot(c, Offset(124 + st.a * 24, 24 - st.b * 16), 1.4, _wh);
}

void _pTopo(Canvas c, _S st, double t) {
  const ctr = Offset(70, 64);
  for (var k = 1; k <= 7; k++) {
    final pts = <Offset>[];
    for (var i = 0; i <= 48; i++) {
      final an = i / 48 * 2 * math.pi;
      final r = k * 9.0 + math.sin(an * 3 + k * .7) * 3 + math.cos(an * 2 - k) * 2;
      pts.add(ctr + Offset(math.cos(an) * r * 1.3, math.sin(an) * r));
    }
    _pl(c, pts, k == 4 ? _pu : _dg, w: k == 4 ? 1 : .8);
  }
  final river = Path()..moveTo(0, 96);
  for (var x = 0.0; x <= _cw; x += 6) {
    river.lineTo(x, 96 + math.sin(x * .07 + t * .6) * 4 - x * .12);
  }
  c.drawPath(river, _s(_bl, 1));
  final p = Offset(10 + st.a * 136, 112 - st.b * 100);
  final alt = (1 - ((p - ctr).distance / 70).clamp(0.0, 1.0)) * 900;
  c.drawOval(Rect.fromCenter(center: p, width: 8, height: 3), _s(_mg, .8));
  final head = p - Offset(0, 14);
  _ring(c, head, 4, _rd, 1.2);
  _ln(c, head + const Offset(-3, 2.6), p, _rd);
  _ln(c, head + const Offset(3, 2.6), p, _rd);
  _dot(c, head, 1.2, _rd);
  _lab(c, 'alt', const Offset(150, 6), _wh, ax: 1);
  _num(c, _d3(alt), const Offset(150, 15), 18, _wh, ax: 1);
  _lab(c, 'n ${_d2(st.b * 90)}  e ${_d3(st.a * 180)}', const Offset(6, 6), _mg);
}

void _pPivot(Canvas c, _S st, double t) {
  const ctr = Offset(70, 62);
  final an = Offset(ctr.dx - 36 + st.a * 72, ctr.dy - 30 + (1 - st.b) * 60);
  final rot = math.sin(t * 1.4) * .55;
  for (var g = 3; g >= 0; g--) {
    final r = rot - g * .12 * math.cos(t * 1.4).sign;
    c.save();
    c.translate(an.dx, an.dy);
    c.rotate(r);
    c.translate(-an.dx, -an.dy);
    _rect(c, Rect.fromCenter(center: ctr, width: 60, height: 40), g == 0 ? _wh : _al(_wh, .12 + (3 - g) * .06), g == 0 ? 1.2 : .8);
    if (g == 0) _ln(c, ctr - const Offset(0, 20), ctr - const Offset(0, 14), _wh, 1);
    c.restore();
  }
  final rr = (ctr - an).distance + 30;
  c.drawArc(Rect.fromCircle(center: an, radius: rr * .5), -math.pi / 2 - .55, 1.1, false, _s(_al(_pu, .7), .8));
  _ring(c, an, 4.5, _rd);
  _ln(c, an - const Offset(8, 0), an + const Offset(8, 0), _rd, 1);
  _ln(c, an - const Offset(0, 8), an + const Offset(0, 8), _rd, 1);
  _lab(c, 'anchor', const Offset(6, 6), _rd);
  _num(c, '${_d2(st.a * 99)}·${_d2(st.b * 99)}', const Offset(6, 98), 16, _rd);
  _lab(c, 'swing', const Offset(150, 108), _pu, ax: 1);
}

void _pTank(Canvas c, _S st, double t) {
  const o = Offset(60, 34), k = 9.0;
  _isoBox(c, o, 0, 0, 0, 7, 4, 4.4, k, _bl, 1);
  final surf = <Offset>[];
  for (var i = 0; i <= 14; i++) {
    surf.add(_iso(o, i / 2, 0, 3.8 + math.sin(i * .9 + t * 2) * .12, k));
  }
  _pl(c, surf, _al(_bl, .6), w: .8);
  for (var i = 0; i < 3; i++) {
    final p = _iso(o, 1 + i * 2.3, 2, _wr(t * .3 + i * .33) * 3.6, k);
    _ring(c, p + Offset(math.sin(t * 3 + i) * 1.5, 0), 1.6, _mg, .8);
  }
  for (final q in [[0.0, 0.0], [7.0, 0.0], [7.0, 4.0], [0.0, 4.0]]) {
    _dot(c, _iso(o, q[0], q[1], 0, k), 1, _bl);
  }
  final x = .6 + st.a * 5.8, y = (1 - st.b) * 4, z = 1.6 + math.sin(t * 1.6) * .3;
  final p = _iso(o, x, y, z, k), fl = _iso(o, x, y, 0, k);
  _dots(c, fl, p + const Offset(0, 3), _dg);
  c.drawOval(Rect.fromCenter(center: fl, width: 10, height: 3), _s(_dg, .8));
  _fish(c, p, _lp(18, 11, 1 - st.b), _gr, wag: math.sin(t * 6), w: 1.2);
  _lab(c, 'x ${_d2(st.a * 99)}', const Offset(6, 6), _bl);
  _lab(c, 'depth ${_d2(st.b * 99)}', const Offset(6, 108), _gr);
}

void _pStars(Canvas c, _S st, double t) {
  const ctr = Offset(78, 60);
  final z = st.b;
  for (var i = 0; i < 46; i++) {
    final s0 = Offset(_rn(i) * _cw, _rn(i + 99) * _ch) - ctr;
    final dep = .3 + _rn(i + 7) * .9;
    final p = ctr + s0 * (1 + z * dep * 1.6) + Offset((st.a - .5) * 30 * dep, 0);
    final r = .5 + dep * z * 1.2;
    if (r > 1.2) {
      _spark(c, p, r * 1.4, _al(_wh, .6), .7);
    } else {
      _dot(c, p, r, _al(_wh, .4 + dep * .4));
    }
  }
  final pr = 6 + z * 26, pc = ctr + Offset((st.a - .5) * 70, 0);
  _ring(c, pc, pr, _pu, 1.2);
  c.drawOval(Rect.fromCenter(center: pc, width: pr * 3.2, height: pr * .7), _s(_gr, 1));
  c.save();
  c.clipRect(Rect.fromLTRB(0, 0, _cw, pc.dy));
  _ring(c, pc, pr, _bk, 2.4);
  _ring(c, pc, pr, _pu, 1.2);
  c.restore();
  _lab(c, 'z', const Offset(6, 6), _gr);
  _num(c, '${z < .5 ? '-' : '+'}${_d2((z - .5).abs() * 200)}', const Offset(6, 14), 22, _gr);
  _lab(c, 'pinch', const Offset(150, 108), _mg, ax: 1);
}

void _pCrane(Canvas c, _S st, double t) {
  const tx = 30.0, jy = 18.0;
  _ln(c, const Offset(tx - 4, 112), const Offset(tx - 4, jy), _wh, 1);
  _ln(c, const Offset(tx + 4, 112), const Offset(tx + 4, jy), _wh, 1);
  for (var y = 112.0; y > jy + 6; y -= 8) {
    _ln(c, Offset(tx - 4, y), Offset(tx + 4, y - 8), _mg, .8);
  }
  _ln(c, const Offset(6, jy), const Offset(150, jy), _wh, 1);
  _ln(c, const Offset(tx + 4, jy + 6), const Offset(150, jy), _mg, .8);
  _pl(c, const [Offset(tx - 4, jy), Offset(tx, 6), Offset(tx + 4, jy)], _wh, w: 1);
  _ln(c, const Offset(tx, 6), const Offset(150, jy), _dg, .7);
  _ln(c, const Offset(tx, 6), const Offset(8, jy), _dg, .7);
  _rect(c, const Rect.fromLTWH(6, jy, 12, 8), _mg, .8);
  _ln(c, const Offset(0, 112), const Offset(_cw, 112), _dg, 1);
  final px = 44 + st.a * 100, hy = 30 + (1 - st.b) * 64, sw = math.sin(t * 1.3) * 2.5;
  _rect(c, Rect.fromCenter(center: Offset(px, jy + 2), width: 8, height: 4), _bl);
  _ln(c, Offset(px, jy + 4), Offset(px + sw, hy), _al(_wh, .7), .8);
  final hk = Offset(px + sw, hy);
  _ln(c, hk, hk + const Offset(-6, 6), _wh, .8);
  _ln(c, hk, hk + const Offset(6, 6), _wh, .8);
  _rect(c, Rect.fromCenter(center: hk + const Offset(0, 12), width: 16, height: 12), _gr);
  _dots(c, hk + const Offset(0, 18), Offset(hk.dx, 112), _dg, gap: 4);
  _lab(c, 'trolley', const Offset(48, 26), _bl);
  _num(c, _d2(st.a * 99), const Offset(48, 34), 13, _bl);
  _lab(c, 'hoist', const Offset(150, 96), _gr, ax: 1);
  _num(c, _d2(st.b * 99), const Offset(150, 104), 13, _gr, ax: 1);
}

void _pStack(Canvas c, _S st, double t) {
  const o = Offset(56, 74), k = 5.0;
  final sel = st.b * 6;
  final order = [0.0, 1.5, 3, 4.5, 6];
  for (final zz in order) {
    final col = (zz - sel).abs() < .75 ? _mg : _dg;
    _pl(c, [_iso(o, 0, 0, zz * 1.5, k), _iso(o, 8, 0, zz * 1.5, k), _iso(o, 8, 6, zz * 1.5, k), _iso(o, 0, 6, zz * 1.5, k)], col, close: true);
  }
  final z = sel * 1.5, dx = (st.a - .5) * 4;
  final q = [_iso(o, 1 + dx, 1, z, k), _iso(o, 7 + dx, 1, z, k), _iso(o, 7 + dx, 5, z, k), _iso(o, 1 + dx, 5, z, k)];
  _pl(c, q, _gr, w: 1.4, close: true);
  _ln(c, q[0], q[2], _al(_gr, .4), .8);
  _dots(c, _iso(o, 4 + dx, 3, 0, k), _iso(o, 4 + dx, 3, z, k), _mg);
  _ln(c, const Offset(140, 96), const Offset(140, 28), _dg, 1);
  _pl(c, const [Offset(137, 31), Offset(140, 26), Offset(143, 31)], _dg, w: 1);
  _lab(c, 'z', const Offset(6, 6), _gr);
  _num(c, _d2(sel * 10), const Offset(6, 14), 28, _gr);
  _lab(c, 'front', const Offset(150, 16), _mg, ax: 1);
  _lab(c, 'back', const Offset(150, 100), _mg, ax: 1);
}

void _pShadow(Canvas c, _S st, double t) {
  const sun = Offset(20, 16);
  _ring(c, sun, 6, _wh, 1);
  for (var i = 0; i < 8; i++) {
    final an = i / 8 * 2 * math.pi + t * .3;
    _ln(c, sun + Offset(math.cos(an), math.sin(an)) * 9, sun + Offset(math.cos(an), math.sin(an)) * 12, _wh, .8);
  }
  const o = Offset(70, 62), k = 7.0;
  for (var i = 0; i <= 10; i += 2) {
    _ln(c, _iso(o, i - 2.0, -2, 0, k), _iso(o, i - 2.0, 8, 0, k), _dg, .6);
    _ln(c, _iso(o, -2, i - 2.0, 0, k), _iso(o, 8, i - 2.0, 0, k), _dg, .6);
  }
  final x = st.a * 6, h = st.b * 4.5 + math.sin(t * 1.8) * .15;
  final sx = x + h * .8, sy = 2 + h * .5;
  final sh = [_iso(o, sx, sy, 0, k), _iso(o, sx + 2, sy, 0, k), _iso(o, sx + 2, sy + 2, 0, k), _iso(o, sx, sy + 2, 0, k)];
  c.drawPath(Path()..addPolygon(sh, true), _s(_al(_pu, 1 - st.b * .6), .9));
  for (var i = 0; i < 3; i++) {
    _ln(c, _lp2(sh[0], sh[2], (i + 1) / 4), _lp2(sh[1], sh[3], (i + 1) / 4) , _al(_pu, .3 - st.b * .2), .6);
  }
  _isoBox(c, o, x, 2, h, 2, 2, 2, k, _gr, 1.2);
  _dots(c, _iso(o, x + 1, 3, 0, k), _iso(o, x + 1, 3, h, k), _mg);
  _lab(c, 'height', const Offset(150, 6), _gr, ax: 1);
  _num(c, _d2(st.b * 99), const Offset(150, 15), 26, _gr, ax: 1);
  _lab(c, 'x ${_d2(st.a * 99)}', const Offset(6, 108), _bl);
}

Offset _lp2(Offset a, Offset b, double t) => a + (b - a) * t;

void _pTape(Canvas c, _S st, double t) {
  const o = Offset(18, 100);
  final ex = o.dx + 10 + st.a * 124, ey = o.dy - 10 - st.b * 84;
  _ln(c, Offset(o.dx + 10, o.dy - 3), Offset(ex, o.dy - 3), _bl, 1);
  _ln(c, Offset(o.dx + 10, o.dy + 3), Offset(ex, o.dy + 3), _bl, 1);
  for (var x = o.dx + 12; x < ex; x += 4) {
    final big = ((x - o.dx) / 4).round() % 5 == 0;
    _ln(c, Offset(x, o.dy - 3), Offset(x, o.dy - (big ? -1 : 1)), _al(_bl, .8), .7);
  }
  _ln(c, Offset(ex, o.dy - 5), Offset(ex, o.dy + 5), _bl, 1.4);
  _ln(c, Offset(o.dx - 3, o.dy - 10), Offset(o.dx - 3, ey), _gr, 1);
  _ln(c, Offset(o.dx + 3, o.dy - 10), Offset(o.dx + 3, ey), _gr, 1);
  for (var y = o.dy - 12; y > ey; y -= 4) {
    final big = ((o.dy - y) / 4).round() % 5 == 0;
    _ln(c, Offset(o.dx - 3, y), Offset(o.dx - (big ? -1 : 1), y), _al(_gr, .8), .7);
  }
  _ln(c, Offset(o.dx - 5, ey), Offset(o.dx + 5, ey), _gr, 1.4);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: o, width: 20, height: 20), const Radius.circular(4)), _s(_wh, 1));
  _ring(c, o, 5, _wh, 1);
  _dot(c, o, 1.2, _wh);
  _dots(c, Offset(ex, o.dy - 6), Offset(ex, ey), _dg);
  _dots(c, Offset(o.dx + 6, ey), Offset(ex, ey), _dg);
  _rect(c, Rect.fromCenter(center: Offset(ex, ey), width: 12, height: 12), _wh);
  _num(c, _d3(st.a * 192), Offset(ex.clamp(40, 120), o.dy + 8), 11, _bl, ax: .5);
  _num(c, _d3(st.b * 108), Offset(o.dx + 10, (ey + 4).clamp(4, 80)), 11, _gr);
}

void _pPath(Canvas c, _S st, double t) {
  final pts = st.pts.length > 3
      ? st.pts
      : [for (var i = 0; i <= 30; i++) Offset(14 + i * 4.2, 64 + math.sin(i * .35) * 30 - i * .4)];
  _pl(c, pts, _dg, w: .8);
  var len = 0.0;
  final cum = <double>[0];
  for (var i = 1; i < pts.length; i++) {
    len += (pts[i] - pts[i - 1]).distance;
    cum.add(len);
  }
  Offset at(double d) {
    for (var i = 1; i < pts.length; i++) {
      if (cum[i] >= d) {
        final f = (d - cum[i - 1]) / math.max(1e-3, cum[i] - cum[i - 1]);
        return _lp2(pts[i - 1], pts[i], f);
      }
    }
    return pts.last;
  }

  final u = _wr(t * .12);
  for (var d = 0.0, k = 0; d < len * u; d += 7, k++) {
    final p = at(d), q = at(d + 1);
    final nrm = Offset(-(q - p).dy, (q - p).dx);
    final nn = nrm.distance < 1e-3 ? Offset.zero : nrm / nrm.distance;
    _dot(c, p + nn * (k.isEven ? 2.5 : -2.5), 1.1, _al(_gr, .35 + .65 * d / math.max(1, len * u)));
  }
  final p = at(len * u);
  _stick(c, p, 18, _wh, armL: .6 * math.sin(t * 9), armR: -.6 * math.sin(t * 9), legL: .4 * math.sin(t * 9), legR: -.4 * math.sin(t * 9));
  _ring(c, pts.first, 2.5, _bl, 1);
  _ring(c, pts.last, 2.5, _rd, 1);
  _lab(c, 'draw a path', const Offset(6, 6), _mg);
  _num(c, _d3(len), const Offset(150, 100), 14, _gr, ax: 1);
}

void _pBilliard(Canvas c, _S st, double t) {
  const tb = Rect.fromLTRB(10, 14, 146, 106);
  c.drawRRect(RRect.fromRectAndRadius(tb, const Radius.circular(6)), _s(_wh, 1));
  c.drawRRect(RRect.fromRectAndRadius(tb.deflate(5), const Radius.circular(3)), _s(_dg, .8));
  for (final p in [tb.topLeft, tb.topCenter, tb.topRight, tb.bottomLeft, tb.bottomCenter, tb.bottomRight]) {
    _ring(c, Offset(p.dx.clamp(15, 141), p.dy.clamp(19, 101)), 3.5, _mg, .8);
  }
  final r = tb.deflate(10);
  final b = Offset(_lp(r.left, r.right, st.a), _lp(r.bottom, r.top, st.b));
  if (st.tr.isEmpty || (st.tr.last - b).distance > 3) st.tr.add(b);
  if (st.tr.length > 40) st.tr.removeAt(0);
  for (var i = 0; i < st.tr.length; i++) {
    _dot(c, st.tr[i], .9, _al(_gr, i / st.tr.length * .8));
  }
  _ring(c, b, 5, _gr, 1.2);
  _dot(c, b + const Offset(-1.5, -1.5), 1, _gr);
  final moving = st.va.abs() + st.vb.abs() > .02;
  if (!moving && !st.down) {
    final an = t * .5;
    final d = Offset(math.cos(an), math.sin(an));
    _ln(c, b - d * 9, b - d * 40, _al(_wh, .5), 1);
  }
  _lab(c, 'flick', const Offset(14, 4), _mg);
  _num(c, '${_d2(st.a * 99)} ${_d2(st.b * 99)}', const Offset(146, 108), 10, _wh, ax: 1);
}

void _pLift(Canvas c, _S st, double t) {
  const l = 14.0, r = 102.0, top = 8.0, bot = 112.0;
  _rect(c, const Rect.fromLTRB(l, top, r, bot), _wh, 1);
  for (var i = 1; i < 10; i++) {
    final y = _lp(bot, top, i / 10);
    _ln(c, Offset(l, y), Offset(r, y), _dg, .7);
  }
  for (var s = 0; s < 3; s++) {
    final x = l + 10 + s * 28;
    _ln(c, Offset(x, top), Offset(x, bot), _dg, .6);
    _ln(c, Offset(x + 18, top), Offset(x + 18, bot), _dg, .6);
  }
  final shaft = st.a * 2, y = _lp(bot - 5, top + 5, st.b);
  final cx = l + 19 + shaft * 28;
  _ln(c, Offset(cx, top), Offset(cx, y - 5), _mg, .7);
  _rect(c, Rect.fromCenter(center: Offset(cx, y), width: 16, height: 9), _gr, 1.2);
  _ln(c, Offset(cx, y - 4.5), Offset(cx, y + 4.5), _al(_gr, .5), .7);
  final floor = (st.b * 9.99).floor();
  _num(c, _d2(floor), const Offset(152, 34), 44, _gr, ax: 1);
  _lab(c, 'floor', const Offset(152, 24), _gr, ax: 1);
  _lab(c, 'shaft ${['a', 'b', 'c'][shaft.round()]}', const Offset(152, 86), _bl, ax: 1);
  final up = math.sin(t * 2) > 0;
  _pl(c, up ? const [Offset(130, 104), Offset(134, 98), Offset(138, 104)] : const [Offset(130, 98), Offset(134, 104), Offset(138, 98)], _wh, w: 1);
}

void _pTunnel(Canvas c, _S st, double t) {
  final vp = Offset(78 + (st.a - .5) * 40, 58);
  for (var i = 0; i < 9; i++) {
    final f = _wr(i / 9 + t * .15);
    final s = math.pow(f, 2.2).toDouble();
    final rr = Rect.fromCenter(center: _lp2(vp, const Offset(78, 60), s), width: 8 + s * 190, height: 6 + s * 150);
    _rect(c, rr, _al(i.isEven ? _bl : _pu, .25 + s * .6), .8);
  }
  for (final cn in [const Offset(0, 0), const Offset(_cw, 0), const Offset(0, _ch), const Offset(_cw, _ch)]) {
    _ln(c, vp, cn, _dg, .6);
  }
  final z = st.b, s = math.pow(1 - z, 2.2).toDouble();
  final p = _lp2(vp, const Offset(78, 60), s);
  _ring(c, p, 3 + s * 24, _gr, 1.4);
  _ring(c, p, 1 + s * 12, _al(_gr, .5), .8);
  _lab(c, 'z', const Offset(6, 6), _gr);
  _num(c, '-${_d3(z * 999)}', const Offset(6, 14), 20, _gr);
}

void _pFinder(Canvas c, _S st, double t) {
  const fr = Rect.fromLTRB(12, 12, 144, 108);
  for (final cn in [fr.topLeft, fr.topRight, fr.bottomLeft, fr.bottomRight]) {
    final sx = cn.dx < 78 ? 1.0 : -1.0, sy = cn.dy < 60 ? 1.0 : -1.0;
    _pl(c, [cn + Offset(0, 9 * sy), cn, cn + Offset(9 * sx, 0)], _wh, w: 1.2);
  }
  for (var i = 1; i < 3; i++) {
    _dots(c, Offset(_lp(fr.left, fr.right, i / 3), fr.top), Offset(_lp(fr.left, fr.right, i / 3), fr.bottom), _dg, gap: 4);
    _dots(c, Offset(fr.left, _lp(fr.top, fr.bottom, i / 3)), Offset(fr.right, _lp(fr.top, fr.bottom, i / 3)), _dg, gap: 4);
  }
  final p = Offset(_lp(fr.left + 10, fr.right - 10, st.a), _lp(fr.bottom - 10, fr.top + 18, st.b));
  _ln(c, p, p + const Offset(0, 14), _wh, 1.2);
  _ring(c, p - const Offset(0, 6), 9, _wh, 1.2);
  _ln(c, p + const Offset(0, 4), p + const Offset(-5, -1), _wh, .8);
  for (var i = 1; i < 3; i++) {
    for (var j = 1; j < 3; j++) {
      final q = Offset(_lp(fr.left, fr.right, i / 3), _lp(fr.top, fr.bottom, j / 3));
      final near = (q - p).distance < 14;
      if (near) {
        _ring(c, q, 5, _gr, 1.2);
        _dot(c, q, 1.6, _gr);
      } else {
        _ln(c, q - const Offset(2, 0), q + const Offset(2, 0), _mg, .8);
      }
    }
  }
  if (_wr(t * .8) < .55) _dot(c, const Offset(22, 22), 2.4, _rd);
  _lab(c, 'rec', const Offset(28, 19), _rd);
  _lab(c, 'thirds', const Offset(140, 99), _gr, ax: 1);
}

void _pLoupe(Canvas c, _S st, double t) {
  const ctr = Offset(62, 60), rad = 48.0, px = 12.0;
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: rad)));
  final off = (st.a - .5) * 4;
  final ex = ctr.dx + off * px;
  for (var i = -5; i <= 5; i++) {
    _ln(c, Offset(ctr.dx + i * px, 0), Offset(ctr.dx + i * px, _ch), _dg, .6);
    _ln(c, Offset(0, ctr.dy + i * px), Offset(_cw, ctr.dy + i * px), _dg, .6);
  }
  for (var i = -5; i < 5; i++) {
    final x0 = ctr.dx + i * px;
    final cov = ((ex - x0) / px).clamp(0.0, 1.0);
    if (cov <= 0) continue;
    for (var j = -5; j < 5; j++) {
      _dot(c, Offset(x0 + px / 2, ctr.dy + j * px + px / 2), .5 + cov * 1.3, _al(_wh, .25 + cov * .7));
    }
  }
  _ln(c, Offset(ex, 0), Offset(ex, _ch), _rd, 1.2);
  c.restore();
  _ring(c, ctr, rad, _wh, 1.2);
  _ring(c, ctr, rad + 3, _dg, .8);
  _ln(c, ctr + Offset(rad * .7, rad * .7), ctr + Offset(rad * .7 + 14, rad * .7 + 14), _wh, 2);
  _lab(c, 'sub px', const Offset(150, 6), _rd, ax: 1);
  final v = off - off.floorToDouble();
  _num(c, '.${_d2(v * 100)}', const Offset(150, 15), 26, _rd, ax: 1);
  _lab(c, 'rub', const Offset(150, 46), _mg, ax: 1);
  _num(c, '${off >= 0 ? '+' : '-'}${off.abs().floor()}', const Offset(150, 56), 12, _wh, ax: 1);
}

void _pShip(Canvas c, _S st, double t) {
  final sea = Path()..moveTo(0, 34);
  for (var x = 0.0; x <= _cw; x += 4) {
    sea.lineTo(x, 34 + math.sin(x * .12 + t * 2) * 1.6);
  }
  c.drawPath(sea, _s(_bl, 1));
  final bed = Path()..moveTo(0, 108);
  for (var x = 0.0; x <= _cw; x += 8) {
    bed.lineTo(x, 108 - _rn(x.toInt()) * 4);
  }
  c.drawPath(bed, _s(_mg, .8));
  final ax = 20 + st.a * 116, ay = 108 - 20 - st.b * 30;
  final drift = math.sin(t * .7) * 26;
  final bp = Offset(ax + drift, 34 + math.sin((ax + drift) * .12 + t * 2) * 1.6);
  final chain = <Offset>[];
  for (var i = 0; i <= 12; i++) {
    final f = i / 12;
    chain.add(Offset(_lp(bp.dx, ax, f), _lp(bp.dy + 3, ay, f) + math.sin(f * math.pi) * 6));
  }
  for (var i = 0; i < chain.length - 1; i++) {
    c.drawOval(Rect.fromCenter(center: _lp2(chain[i], chain[i + 1], .5), width: 3, height: 2), _s(_wh, .7));
  }
  _boat(c, bp, .9, _wh, tilt: math.cos(t * .7) * .12);
  _anchor(c, Offset(ax, ay), 1, _rd);
  for (var i = 0; i < 2; i++) {
    _fish(c, Offset(_wr(t * .05 + i * .5) * 180 - 12, 60 + i * 18), 9, _al(_gr, .6), wag: math.sin(t * 5 + i), w: .8);
  }
  _lab(c, 'anchor', const Offset(6, 6), _rd);
  _num(c, '${_d2(st.a * 99)}·${_d2(st.b * 99)}', const Offset(150, 4), 16, _rd, ax: 1);
}

void _pGlobe(Canvas c, _S st, double t) {
  const ctr = Offset(62, 62);
  final r = 36 + st.b * 14;
  _ring(c, ctr, r, _wh, 1.2);
  final lon = st.a * 2 * math.pi;
  for (var k = 0; k < 6; k++) {
    final ang = lon + k * math.pi / 6;
    final w = math.cos(ang).abs() * r;
    c.drawArc(Rect.fromCenter(center: ctr, width: w * 2, height: r * 2), -math.pi / 2, math.sin(ang) > 0 ? math.pi : -math.pi, false,
        _s(_al(_bl, .7), .8));
  }
  for (var j = -2; j <= 2; j++) {
    final y = ctr.dy + j * r / 3, hw = math.sqrt(math.max(0, r * r - (j * r / 3) * (j * r / 3)));
    _ln(c, Offset(ctr.dx - hw, y), Offset(ctr.dx + hw, y), _dg, .8);
  }
  const plat = .5, plon = -.3;
  final al = plon + lon;
  final pin = ctr + Offset(math.sin(al) * math.cos(plat) * r, -math.sin(plat) * r);
  if (math.cos(al) > 0) {
    _ln(c, pin, pin - const Offset(0, 12), _rd, 1.2);
    _ring(c, pin - const Offset(0, 15), 3, _rd, 1.2);
    _dot(c, pin, 1.6, _rd);
  } else {
    _ring(c, pin, 1.6, _al(_rd, .4), .8);
  }
  final sa = t * .9;
  final sat = ctr + Offset(math.cos(sa) * (r + 12), math.sin(sa) * (r + 12) * .35);
  c.drawOval(Rect.fromCenter(center: ctr, width: (r + 12) * 2, height: (r + 12) * .7), _s(_al(_pu, .5), .7));
  _rect(c, Rect.fromCenter(center: sat, width: 4, height: 4), _pu, 1);
  _lab(c, 'lon', const Offset(150, 6), _bl, ax: 1);
  _num(c, _d3(st.a * 360), const Offset(150, 15), 18, _bl, ax: 1);
  _lab(c, 'zoom', const Offset(150, 84), _gr, ax: 1);
  _num(c, _d2(st.b * 99), const Offset(150, 93), 18, _gr, ax: 1);
}
