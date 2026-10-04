// Repeat / Clone: twenty pictures of "the same thing again, a step further each time" (count, offset, rotation step).
part of 'pop_08.dart';

List<_Pan8> _repeatPanels() => const [
      _Pan8('Duckling parade', 'creature · flat · drag → count, up → spacing', _G.xy, _rDucks, a: .45, b: .3),
      _Pan8('Spiro petals', 'nature · crisp · spin → step, out → count', _G.spin, _rSpiro, a: .2, b: .5),
      _Pan8('Rubber stamp', 'toy · light · paint the trail', _G.paint, _rStamp),
      _Pan8('Paper dolls', 'material · bold flat · pull to unfold', _G.xy, _rDolls, a: .6),
      _Pan8('Leaning tower', 'toy · isometric · up → stack, x → lean', _G.xy, _rTower, a: .65, b: .55),
      _Pan8('Moon ring', 'cosmic · pseudo 3D · flick → orbit, up → moons', _G.flick, _rMoons, a: .1, b: .45, wrap: true),
      _Pan8('Kaleidoscope', 'neon wild · spin → twist, out → mirrors', _G.spin, _rKaleido, a: .3, b: .4),
      _Pan8('Domino run', 'physics · side view · release → topple', _G.xy, _rDomino, a: .6, b: .3),
      _Pan8('Sushi belt', 'food · top-down · x → plates, up → speed', _G.xy, _rSushi, a: .5, b: .35),
      _Pan8('Invader rank', 'pixel · neon · x → columns, up → rows', _G.xy, _rInvaders, a: .55, b: .35),
      _Pan8('Onion skin', 'motion · flat · x → ghosts, up → time step', _G.xy, _rOnion, a: .6, b: .35),
      _Pan8('Tunnel train', 'machine · landscape · pull → wagons', _G.xy, _rTrain, a: .5, b: .3),
      _Pan8('Snow steps', 'landscape · map · paint the walk', _G.paint, _rSteps),
      _Pan8('Bird V', 'weather · bold flat · x → birds, up → V angle', _G.xy, _rBirds, a: .5, b: .4),
      _Pan8('Card fan', 'toy · light · spin → spread, out → cards', _G.spin, _rCards, a: .35, b: .5),
      _Pan8('Matryoshka', 'character · flat · x → dolls, up → shrink', _G.xy, _rDollsRu, a: .6, b: .4),
      _Pan8('Sunflower', 'nature · crisp · spin → angle, out → seeds', _G.spin, _rSunflower, a: .42, b: .6),
      _Pan8('Op-art twist', 'text-free op art · spin → step, out → count', _G.spin, _rVortex, a: .7, b: .5),
      _Pan8('Cell split', 'nature · organic · rub → divide', _G.rub, _rCells, b: .45),
      _Pan8('Copier spray', 'machine · isometric · flick → fan out', _G.flick, _rCopier, a: .5, b: .55),
    ];

void _duck(Canvas c, Offset p, double r, Color body) {
  c.drawPath(_poly([p + Offset(r * .9, -r * .1), p + Offset(r * 1.7, -r * 1.0), p + Offset(r * 1.25, r * .4)]), _f(body));
  c.drawOval(Rect.fromCenter(center: p, width: r * 2.5, height: r * 1.6), _f(body));
  final hd = p + Offset(-r * .95, -r * 1.0);
  c.drawCircle(hd, r * .72, _f(body));
  c.drawPath(_poly([hd + Offset(-r * .5, -r * .18), hd + Offset(-r * 1.35, r * .12), hd + Offset(-r * .5, r * .32)]), _f(_or));
  c.drawCircle(hd + Offset(-r * .18, -r * .16), math.max(1.1, r * .14), _f(_k0));
}

void _rDucks(Canvas c, Size s, _P8 st, double t) {
  final w = s.width, h = s.height, wl = h * .7;
  _bg(c, s, _bl);
  final water = Path()..moveTo(0, wl);
  for (var x = 0.0; x <= w; x += 4) {
    water.lineTo(x, wl + math.sin(x * .14 + t * 2) * 1.6);
  }
  water
    ..lineTo(w, h)
    ..lineTo(0, h)
    ..close();
  c.drawPath(water, _f(_cy));
  final n = 1 + (st.a * 9).round(), gap = 12 + st.b * 13;
  _duck(c, Offset(26, wl - 3 + math.sin(t * 3) * 1.2), 9, _ye);
  for (var i = 0; i < n; i++) {
    final x = 46 + i * gap;
    if (x > w + 8) break;
    _duck(c, Offset(x, wl - 1 + math.sin(t * 5 + i * .9) * 1.4), 5.2, const Color(0xFFFFE893));
    c.drawLine(Offset(x - 5, wl + 3), Offset(x + 7, wl + 3), _s(_al(_wh, .6), 1.2));
  }
}

void _rSpiro(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  final c0 = s.center(Offset.zero), r = s.height * .44;
  final n = 3 + (st.b * 13).round(), step = st.a * math.pi * 2 / 3 + .08;
  const cols = [_pk, _vi, _cy];
  for (var i = n - 1; i >= 0; i--) {
    c.save();
    c.translate(c0.dx, c0.dy);
    c.rotate(i * step + t * .15);
    final petal = Rect.fromCenter(center: Offset(0, -r * .5), width: r * .42, height: r);
    if (i == 0) {
      c.drawOval(petal, _f(_ye));
    } else {
      c.drawOval(petal, _f(_al(cols[i % 3], .14)));
      c.drawOval(petal, _s(cols[i % 3], 1.6));
    }
    c.restore();
  }
  c.drawCircle(c0, 3.5, _f(_wh));
}

void _rStamp(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _cr);
  final pts = st.pts.length > 1 ? st.pts : _defPath(s, .2);
  c.drawPath(Path()..addPolygon(pts, false), _s(_al(_k0, .12), 1));
  Offset last = pts.first;
  _walk(pts, 17, (i, p, ang) {
    c.drawPath(_star(p, 7.5, i * .45), _f(i.isEven ? _rd : _bl));
    c.drawCircle(p + const Offset(2, -2), 1.1, _f(_cr));
    last = p;
  });
  final hp = st.down ? st.at ?? last : last + const Offset(0, -8);
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hp + const Offset(0, -6), width: 18, height: 6), const Radius.circular(2)), _f(_k0));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hp + const Offset(0, -15), width: 7, height: 13), const Radius.circular(3)), _f(_or));
  c.drawCircle(hp + const Offset(0, -23), 5.5, _f(_or));
}

void _paperDoll(Canvas c, double w, double top, Color col) {
  final cx = w / 2;
  c.drawCircle(Offset(cx, top + 6), 5.5, _f(col));
  c.drawRect(Rect.fromLTWH(0, top + 14, w, 4), _f(col));
  c.drawPath(_poly([Offset(cx - 5, top + 12), Offset(cx + 5, top + 12), Offset(cx + 8, top + 34), Offset(cx - 8, top + 34)]), _f(col));
  c.drawRect(Rect.fromLTWH(cx - 5, top + 34, 3, 11), _f(col));
  c.drawRect(Rect.fromLTWH(cx + 2, top + 34, 3, 11), _f(col));
}

void _rDolls(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _pk);
  const dw = 18.0, top = 32.0;
  final nf = 1 + st.a * 7, full = nf.floor(), frac = nf - full;
  for (var i = 0; i < 3; i++) {
    c.drawRect(Rect.fromLTWH(8.0 + i * 1.5, top + 47 - i * 1.5, dw, 3), _f(_al(_k0, .25)));
  }
  for (var i = 0; i <= full && i < 8; i++) {
    final sx = i == full ? frac : 1.0;
    if (sx <= .02) break;
    c.save();
    c.translate(10 + i * dw, 0);
    c.scale(sx, 1);
    _paperDoll(c, dw, top + 3, _al(_k0, .18));
    _paperDoll(c, dw, top, i == full ? const Color(0xFFE2D6C2) : (i.isEven ? _cr : _wh));
    c.restore();
  }
}

void _isoCube(Canvas c, Offset p, double r, double hgt, Color top, Color left, Color right) {
  final ry = r * .58;
  final t0 = p + Offset(0, -hgt);
  c.drawPath(_poly([t0 + Offset(-r, 0), p + Offset(-r, 0), p + Offset(0, ry), t0 + Offset(0, ry)]), _f(left));
  c.drawPath(_poly([t0 + Offset(r, 0), p + Offset(r, 0), p + Offset(0, ry), t0 + Offset(0, ry)]), _f(right));
  c.drawPath(_poly([t0 + Offset(0, -ry), t0 + Offset(r, 0), t0 + Offset(0, ry), t0 + Offset(-r, 0)]), _f(top));
}

void _rTower(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final base = Offset(s.width / 2, s.height - 14);
  for (var k = -4; k <= 4; k++) {
    c.drawLine(base + Offset(k * 14.0 - 40, -23), base + Offset(k * 14.0 + 40, 23), _s(_al(_cr, .07), 1));
    c.drawLine(base + Offset(k * 14.0 + 40, -23), base + Offset(k * 14.0 - 40, 23), _s(_al(_cr, .07), 1));
  }
  final n = 1 + (st.b * 8).round(), lean = (st.a - .5) * 10;
  final sway = math.sin(t * 2.2) * (st.a - .5).abs() * 2;
  c.drawOval(Rect.fromCenter(center: base + const Offset(0, 4), width: 34, height: 12), _f(_al(_k0, .5)));
  for (var i = 0; i < n; i++) {
    final p = base + Offset((lean + sway) * i, -i * 10.5);
    _isoCube(c, p, 13, 10.5, _ye, _or, const Color(0xFFC4501F));
  }
}

void _rMoons(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  for (var i = 0; i < 18; i++) {
    c.drawCircle(Offset(_rn(i) * s.width, _rn(i + 40) * s.height), .9, _f(_al(_wh, .6)));
  }
  final c0 = s.center(Offset.zero), rx = 60.0, ry = 17.0, ring = Rect.fromCenter(center: c0, width: rx * 2, height: ry * 2);
  final n = 2 + (st.b * 10).round();
  final pos = [for (var i = 0; i < n; i++) (st.a + i / n) * math.pi * 2 + t * .25];
  void moons(bool front) {
    for (final a in pos) {
      final sn = math.sin(a);
      if ((sn > 0) != front) continue;
      c.drawCircle(c0 + Offset(math.cos(a) * rx, sn * ry), 4.2 + sn * 1.2, _f(front ? _ye : const Color(0xFFB8962C)));
    }
  }

  c.drawArc(ring, math.pi, math.pi, false, _s(_cr, 2));
  moons(false);
  c.drawCircle(c0, 21, _f(_vi));
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: c0, radius: 21)));
  c.drawRect(Rect.fromLTWH(c0.dx - 22, c0.dy - 6, 44, 5), _f(_pk));
  c.drawRect(Rect.fromLTWH(c0.dx - 22, c0.dy + 6, 44, 3), _f(_al(_pk, .7)));
  c.restore();
  c.drawArc(ring, 0, math.pi, false, _s(_cr, 2));
  moons(true);
}

void _rKaleido(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  final c0 = s.center(Offset.zero), r = s.height * .47, n = 2 * (2 + (st.b * 5).round()), al = math.pi * 2 / n;
  final wedge = Path()
    ..moveTo(0, 0)
    ..arcTo(Rect.fromCircle(center: Offset.zero, radius: r), 0, al, false)
    ..close();
  final ph = st.a * math.pi * 2 + t * .35;
  for (var i = 0; i < n; i++) {
    c.save();
    c.translate(c0.dx, c0.dy);
    c.rotate(i * al);
    if (i.isOdd) {
      c.rotate(al / 2);
      c.scale(1, -1);
      c.rotate(-al / 2);
    }
    c.clipPath(wedge);
    c.drawCircle(Offset(math.cos(ph) * 10 + r * .55, math.sin(ph) * 8 + 6), r * .32, _f(_pk));
    c.drawCircle(Offset(r * .25, r * .05 + math.sin(ph * 1.3) * 5), r * .16, _f(_ye));
    c.drawPath(_poly([Offset(r * .7, -4), Offset(r * 1.05, r * .18), Offset(r * .62 + math.cos(ph) * 6, r * .4)]), _f(_cy));
    c.drawCircle(Offset(r * .85, r * .12), r * .08, _f(_li));
    c.restore();
  }
  c.drawCircle(c0, r, _s(_wh, 1.5));
}

void _rDomino(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final base = s.height * .8;
  c.drawLine(Offset(0, base + .5), Offset(s.width, base + .5), _s(_cr, 1.5));
  final n = 3 + (st.a * 9).round(), gap = 9 + st.b * 9;
  const dw = 5.0, dh = 26.0;
  final per = .05 + gap * .006, since = st.down ? -1.0 : (t - st.rel) % (n * per + 2.2);
  final lean = math.asin(math.min(1.0, (gap - dw) / dh));
  for (var i = 0; i < n; i++) {
    final x = 12 + i * gap;
    if (x > s.width - 4) break;
    final k = ((since - i * per) / .16).clamp(0.0, 1.0);
    final last = i == n - 1 || 12 + (i + 1) * gap > s.width - 4;
    final ang = k * (last ? math.pi / 2 - .05 : lean);
    c.save();
    c.translate(x + dw, base);
    c.rotate(ang);
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-dw, -dh, dw, dh), const Radius.circular(1.5)), _f(i == 0 ? _or : _cr));
    c.drawCircle(const Offset(-dw / 2, -dh * .72), 1, _f(_k0));
    c.drawCircle(const Offset(-dw / 2, -dh * .3), 1, _f(_k0));
    c.restore();
  }
}

void _rSushi(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _cr);
  final belt = RRect.fromRectAndRadius(Rect.fromLTWH(18, 18, s.width - 36, s.height - 36), const Radius.circular(40));
  c.drawRRect(belt, _s(_k1, 20));
  c.drawRRect(belt, _s(_al(_cr, .15), 1));
  final m = (Path()..addRRect(belt)).computeMetrics().first;
  final n = 2 + (st.a * 10).round(), off = t * (.02 + st.b * .12);
  const fish = [_or, _rd, _ye, _pk];
  for (var i = 0; i < n; i++) {
    final tg = m.getTangentForOffset(_wr(off + i / n) * m.length);
    if (tg == null) continue;
    final p = tg.position;
    c.drawCircle(p, 8, _f(_wh));
    c.drawCircle(p, 8, _s(_al(_k0, .3), 1));
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(-tg.angle);
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset.zero, width: 10, height: 6), const Radius.circular(3)), _f(_cr));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: const Offset(0, -1), width: 11, height: 4.5), const Radius.circular(2)), _f(fish[i % 4]));
    c.restore();
  }
}

const _crab = ['00100000100', '00010001000', '00111111100', '01101110110', '11111111111', '10111111101', '10100000101', '00011011000'];
const _crab2 = ['00100000100', '10010001001', '10111111101', '11101110111', '11111111111', '01111111110', '00100000100', '01000000010'];

void _rInvaders(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  final cols = 1 + (st.a * 6).round(), rows = 1 + (st.b * 3).round();
  const px = 1.5, gx = 20.0, gy = 16.0, rc = [_pk, _cy, _li, _ye];
  final march = math.sin(t * 1.4) * 6, frame = (t * 2).floor().isEven ? _crab : _crab2;
  final x0 = (s.width - (cols - 1) * gx - 11 * px) / 2 + march;
  for (var r = 0; r < rows; r++) {
    for (var k = 0; k < cols; k++) {
      final o = Offset(x0 + k * gx, 10 + r * gy);
      for (var y = 0; y < 8; y++) {
        for (var x = 0; x < 11; x++) {
          if (frame[y].codeUnitAt(x) == 49) c.drawRect(Rect.fromLTWH(o.dx + x * px, o.dy + y * px, px + .1, px + .1), _f(rc[r]));
        }
      }
    }
  }
  final gx0 = s.width / 2 + math.sin(t * .9) * 30;
  c.drawRect(Rect.fromLTWH(gx0 - 8, s.height - 12, 16, 5), _f(_li));
  c.drawRect(Rect.fromLTWH(gx0 - 1.5, s.height - 16, 3, 4), _f(_li));
  c.drawLine(Offset(0, s.height - 5), Offset(s.width, s.height - 5), _s(_li, 1));
}

void _rOnion(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final floor = s.height - 16;
  c.drawLine(Offset(0, floor + 9), Offset(s.width, floor + 9), _s(_cr, 1.5));
  Offset at(double tau) {
    final x = _wr(tau * .28) * (s.width + 24) - 12;
    return Offset(x, floor - (math.sin(tau * math.pi * 1.2)).abs() * (s.height - 40));
  }

  final n = (st.a * 10).round(), dt = .03 + st.b * .12;
  for (var k = n; k >= 1; k--) {
    final q = 1 - k / (n + 1);
    c.drawCircle(at(t - k * dt), 9, _f(_al(Color.lerp(_vi, _cy, q)!, .2 + .6 * q)));
  }
  final p = at(t), sq = ((floor - p.dy) / 6).clamp(0.0, 1.0);
  c.drawOval(Rect.fromCenter(center: p + Offset(0, 9 - 9 * (.75 + .25 * sq)), width: 18 + (1 - sq) * 5, height: 18 * (.75 + .25 * sq)), _f(_cy));
  c.drawCircle(p + const Offset(-3, -3), 2.4, _f(_al(_wh, .9)));
}

void _rTrain(Canvas c, Size s, _P8 st, double t) {
  final w = s.width, h = s.height, rail = h * .8, tx = w - 40;
  _bg(c, s, _cy);
  c.drawCircle(Offset(w - 22, 18), 9, _f(_ye));
  c.drawOval(Rect.fromLTWH(-40, rail - 18, 120, 50), _f(const Color(0xFF7CC64E)));
  c.drawRect(Rect.fromLTWH(0, rail, w, h - rail), _f(const Color(0xFF7CC64E)));
  final n = (st.a * 6).round(), gap = 3 + st.b * 9, jig = math.sin(t * 12) * .4;
  const wc = [_ye, _pk, _vi, _or, _bl, _rd];
  for (var i = 0; i < n; i++) {
    final x = 42 + gap + i * (20 + gap);
    c.drawLine(Offset(x - gap, rail - 7), Offset(x, rail - 7), _s(_k0, 1.5));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x, rail - 18 + jig, 20, 13), const Radius.circular(2)), _f(wc[i]));
    c.drawCircle(Offset(x + 5, rail - 3), 3, _f(_k0));
    c.drawCircle(Offset(x + 15, rail - 3), 3, _f(_k0));
  }
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(10, rail - 17 + jig, 32, 12), const Radius.circular(2)), _f(_rd));
  c.drawRect(Rect.fromLTWH(30, rail - 27 + jig, 12, 12), _f(_rd));
  c.drawRect(Rect.fromLTWH(15, rail - 24 + jig, 5, 8), _f(_k0));
  for (final x in [16.0, 26.0, 36.0]) {
    c.drawCircle(Offset(x, rail - 3), 3.5, _f(_k0));
  }
  for (var k = 0; k < 3; k++) {
    final q = _wr(t * .6 + k / 3);
    c.drawCircle(Offset(17 + q * 14, rail - 28 - q * 22), 3 + q * 5, _f(_al(_wh, .9 - q * .7)));
  }
  final hill = Path()
    ..moveTo(tx, h)
    ..lineTo(tx, rail - 26)
    ..quadraticBezierTo(tx + 20, rail - 62, w + 10, rail - 50)
    ..lineTo(w + 10, h)
    ..close();
  c.drawPath(hill, _f(_li));
  c.drawPath(
      Path()
        ..moveTo(tx - 2, rail)
        ..lineTo(tx - 2, rail - 18)
        ..arcToPoint(Offset(tx + 14, rail - 18), radius: const Radius.circular(8))
        ..lineTo(tx + 14, rail)
        ..close(),
      _f(_k0));
  c.drawLine(Offset(0, rail), Offset(w, rail), _s(_k0, 2));
}

void _rSteps(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, const Color(0xFFEAF6FF));
  for (var i = 0; i < 14; i++) {
    c.drawCircle(Offset(_rn(i + 3) * s.width, _rn(i + 70) * s.height), 1.2, _f(_al(_cy, .5)));
  }
  final pts = st.pts.length > 1 ? st.pts : _defPath(s, 2.2);
  final marks = <(Offset, double)>[];
  _walk(pts, 11, (i, p, a) => marks.add((p, a)));
  for (var i = 0; i < marks.length; i++) {
    final (p, a) = marks[i];
    final side = i.isEven ? 1.0 : -1.0, q = (i + 1) / marks.length;
    final col = _al(_bl, .25 + .75 * q);
    c.save();
    c.translate(p.dx, p.dy);
    c.rotate(a);
    c.translate(0, side * 4);
    c.drawOval(Rect.fromCenter(center: Offset.zero, width: 8, height: 4.6), _f(col));
    c.drawCircle(const Offset(5.6, 0), 1.6, _f(col));
    c.restore();
  }
}

void _rBirds(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _bl);
  for (var k = 0; k < 2; k++) {
    final cx = _wr(t * .02 + k * .55) * (s.width + 60) - 30, cy = 14.0 + k * 84;
    for (final o in const [Offset(0, 0), Offset(10, -5), Offset(20, 0), Offset(10, 3)]) {
      c.drawCircle(Offset(cx, cy) + o, 7, _f(_al(_wh, .25)));
    }
  }
  final n = 1 + (st.a * 10).round(), half = (8 + st.b * 55) * math.pi / 180;
  final lead = Offset(s.width * .84, s.height * .5);
  for (var i = 0; i < n; i++) {
    final k = (i + 1) ~/ 2, side = i.isOdd ? 1.0 : -1.0;
    final p = lead + Offset(-math.cos(half) * k * 17, side * math.sin(half) * k * 17);
    final fl = math.sin(t * 9 + k * .8) * 5;
    final b = Path()
      ..moveTo(p.dx - 9, p.dy - fl)
      ..quadraticBezierTo(p.dx - 3, p.dy - 1, p.dx, p.dy + 2)
      ..quadraticBezierTo(p.dx + 3, p.dy - 1, p.dx + 9, p.dy - fl);
    c.drawPath(b, _s(i == 0 ? _ye : _wh, 2.8));
  }
}

Path _heart(Offset c, double r) => Path()
  ..moveTo(c.dx, c.dy + r * .9)
  ..cubicTo(c.dx - r * 1.3, c.dy, c.dx - r * .7, c.dy - r * 1.1, c.dx, c.dy - r * .35)
  ..cubicTo(c.dx + r * .7, c.dy - r * 1.1, c.dx + r * 1.3, c.dy, c.dx, c.dy + r * .9)
  ..close();

void _rCards(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _cr);
  final pv = Offset(s.width / 2, s.height * .95), n = 2 + (st.b * 7).round(), step = st.a * .5;
  for (var i = 0; i < n; i++) {
    c.save();
    c.translate(pv.dx, pv.dy);
    c.rotate((i - (n - 1) / 2) * step);
    final r = RRect.fromRectAndRadius(const Rect.fromLTWH(-14, -76, 28, 42), const Radius.circular(3));
    c.drawRRect(r.shift(const Offset(1.5, 1.5)), _f(_al(_k0, .18)));
    c.drawRRect(r, _f(_wh));
    c.drawRRect(r, _s(_al(_k0, .3), 1));
    final suit = i % 3;
    if (suit == 0) {
      c.drawPath(_heart(const Offset(0, -55), 6), _f(_rd));
    } else if (suit == 1) {
      c.drawPath(_poly(const [Offset(0, -63), Offset(6, -55), Offset(0, -47), Offset(-6, -55)]), _f(_or));
    } else {
      c.drawCircle(const Offset(0, -55), 5.5, _f(_bl));
    }
    c.restore();
  }
}

void _rDollsRu(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final n = 1 + (st.a * 6).round(), k = .9 - st.b * .3, base = s.height - 10;
  var x = 8.0, hh = 62.0;
  const body = [_rd, _or, _pk, _vi, _bl, _rd, _or];
  for (var i = 0; i < n && x < s.width; i++) {
    final w = hh * .62, cx = x + w / 2, hr = w * .36;
    final bob = math.sin(t * 3 + i) * .8;
    c.drawRRect(RRect.fromRectAndCorners(Rect.fromLTWH(x, base - hh * .64 + bob, w, hh * .64), topLeft: Radius.circular(w * .4), topRight: Radius.circular(w * .4), bottomLeft: Radius.circular(w * .2), bottomRight: Radius.circular(w * .2)), _f(body[i]));
    c.drawCircle(Offset(cx, base - hh * .7 + bob), hr * 1.25, _f(body[i]));
    c.drawCircle(Offset(cx, base - hh * .7 + bob), hr, _f(_cr));
    c.drawCircle(Offset(cx - hr * .4, base - hh * .72 + bob), math.max(.8, hr * .1), _f(_k0));
    c.drawCircle(Offset(cx + hr * .4, base - hh * .72 + bob), math.max(.8, hr * .1), _f(_k0));
    c.drawCircle(Offset(cx - hr * .55, base - hh * .64 + bob), hr * .18, _f(_al(_pk, .8)));
    c.drawCircle(Offset(cx + hr * .55, base - hh * .64 + bob), hr * .18, _f(_al(_pk, .8)));
    c.drawCircle(Offset(cx, base - hh * .3 + bob), w * .17, _f(_ye));
    c.drawCircle(Offset(cx, base - hh * .3 + bob), w * .07, _f(body[i]));
    x += w + 4;
    hh *= k;
  }
}

void _rSunflower(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k0);
  final c0 = s.center(Offset.zero), rd = s.height * .3;
  for (var i = 0; i < 16; i++) {
    c.save();
    c.translate(c0.dx, c0.dy);
    c.rotate(i * math.pi / 8 + math.sin(t * .8) * .03);
    c.drawOval(Rect.fromCenter(center: Offset(0, -rd - 10), width: 10, height: 24), _f(i.isEven ? _ye : const Color(0xFFFFB627)));
    c.restore();
  }
  c.drawCircle(c0, rd + 2, _f(const Color(0xFF4A2C17)));
  final deg = 130 + st.a * 15, step = deg * math.pi / 180, n = 30 + (st.b * 230).round();
  final gold = (deg - 137.508).abs() < .5;
  for (var i = 0; i < n; i++) {
    final r = rd * math.sqrt((i + .5) / n), a = i * step;
    c.drawCircle(c0 + Offset(math.cos(a), math.sin(a)) * r, 1.2 + 1.6 * math.sqrt(30 / n), _f(gold ? _li : Color.lerp(_ye, _or, i / n)!));
  }
}

void _rVortex(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _ye);
  final c0 = s.center(Offset.zero), n = 4 + (st.b * 16).round(), th = (st.a - .5) * math.pi / 6;
  final k = 1 / (math.cos(th).abs() + math.sin(th).abs());
  var side = s.height * .88;
  c.save();
  c.translate(c0.dx, c0.dy);
  c.rotate(t * .2);
  for (var i = 0; i < n; i++) {
    c.drawRect(Rect.fromCenter(center: Offset.zero, width: side, height: side), _f(i.isEven ? _k0 : (i % 4 == 1 ? _ye : _pk)));
    c.rotate(th);
    side *= k;
  }
  c.restore();
}

void _rCells(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final c0 = s.center(Offset.zero), dr = s.height * .46;
  c.drawCircle(c0, dr, _f(_k0));
  c.drawCircle(c0, dr, _s(_cy, 2));
  final g = st.b * 4.99, gi = g.floor(), fr = g - gi;
  final n = 1 << gi, rc = dr * .78 / math.sqrt(n) * (n == 1 ? .55 : 1);
  final p = ((fr - .25) / .75).clamp(0.0, 1.0), sm = p * p * (3 - 2 * p);
  final rl = rc * (1 - (1 - math.sqrt1_2) * sm), off = rc * math.sqrt1_2 * sm;
  for (var i = 0; i < n; i++) {
    final r = n == 1 ? 0.0 : dr * .72 * math.sqrt((i + .5) / n), a = i * 2.39996;
    final pc = c0 + Offset(math.cos(a), math.sin(a)) * r + Offset(math.sin(t * 1.3 + i), math.cos(t * 1.1 + i)) * 1.2;
    final ax = _rn(i + gi * 31) * math.pi, d = Offset(math.cos(ax), math.sin(ax)) * off;
    for (final q in [pc + d, pc - d]) {
      c.drawCircle(q, rl, _f(_li));
      c.drawCircle(q, rl * .38, _f(const Color(0xFF4E9A2A)));
    }
    if (sm > 0 && sm < 1) c.drawCircle(pc, rl * .55, _f(_li));
  }
}

void _rCopier(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final base = Offset(34, s.height - 26);
  _isoCube(c, base, 26, 30, _vi, const Color(0xFF6E52D9), const Color(0xFF5640B8));
  c.drawPath(_poly([base + const Offset(-10, -36), base + const Offset(6, -45), base + const Offset(14, -40), base + const Offset(-2, -31)]), _f(_cy));
  final n = 1 + (st.b * 8).round(), sp = st.a;
  for (var i = n - 1; i >= 0; i--) {
    final q = i / math.max(1, n - 1);
    final o = base + Offset(26 + i * (4 + sp * 9), -18 - i * (1 + sp * 5) + math.sin(t * 3 + i) * sp * 2);
    c.save();
    c.translate(o.dx, o.dy);
    c.rotate(i * sp * .22 - .1);
    final sh = _poly(const [Offset(0, 0), Offset(22, -12), Offset(40, -2), Offset(18, 10)]);
    c.drawPath(sh.shift(const Offset(1, 2)), _f(_al(_k0, .4)));
    c.drawPath(sh, _f(Color.lerp(_wh, _cr, q)!));
    c.save();
    c.transform(Float64List.fromList([22, -12, 0, 0, 18, 10, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]));
    c.drawCircle(const Offset(.5, .5), .28, _f(_pk));
    c.restore();
    c.restore();
  }
}
