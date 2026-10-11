// Scale / Stretch: 20 pictures of why a maker resizes something: bigger or smaller, wider or taller, kept in proportion,
// or squashed and stretched so it feels alive.
part of 'pop_09.dart';

List<_Pan9> _scalePanels() => [
      _Pan9('Balloon', 'toy · bold flat light · 2D · rub · uniform', _G.rub, _balloon, b: .35, decay: .03),
      _Pan9('Jelly', 'food · bold flat · isometric · drag · squash', _G.xy, _jelly, b: .5),
      _Pan9('Bounce', 'physics · crisp · 2D · drag · squash+stretch', _G.xy, _bounce, a: .6, b: .7),
      _Pan9('Photo corner', 'sample image · crisp light · 2D · corner · x/y/link', _G.corner, _photoCorner, a: .7, b: .62),
      _Pan9('Pufferfish', 'creature · bold flat · 2D · pinch · uniform', _G.pinch, _puffer, a: .3),
      _Pan9('Nesting dolls', 'toy · bold flat light · 2D · drag · ratio', _G.xy, _dolls, a: .55, b: .6),
      _Pan9('Pizza dough', 'food · bold flat · top-down · stretch · x/y', _G.stretch, _dough, a: .45, b: .45),
      _Pan9('Accordion', 'instrument · bold flat · 2D · drag · x only', _G.xy, _accordion, a: .45),
      _Pan9('Long cat', 'creature · bold flat light · 2D · drag · keep volume', _G.xy, _cat, a: .35),
      _Pan9('Moonrise', 'cosmic · crisp · 2D · pinch · uniform', _G.pinch, _moon, a: .3),
      _Pan9('Mountains', 'landscape · bold flat light · 2D · stretch · x/y', _G.stretch, _mountains, a: .5, b: .4),
      _Pan9('Taffy pull', 'material · bold flat · 2D · drag · thin as it grows', _G.xy, _taffy, a: .4),
      _Pan9('Tower', 'machine · crisp · isometric · stretch · x/y', _G.stretch, _tower, a: .4, b: .5),
      _Pan9('Slinky', 'toy · neon wild · 2D · drag · y only', _G.xy, _slinky, b: .5),
      _Pan9('Shadow play', 'physics · crisp · 2D · drag · by distance', _G.xy, _shadow, a: .5),
      _Pan9('Pixel sprite', 'toy · neon wild · 2D · pinch · whole steps', _G.pinch, _pixel, a: .45),
      _Pan9('Shrink ray', 'cosmic · neon wild · 2D · drag · grow/shrink', _G.xy, _ray, a: .3),
      _Pan9('Thundercloud', 'weather · bold flat · 2D · stretch · x/y', _G.stretch, _cloud, a: .5, b: .35),
      _Pan9('Speaker', 'instrument · neon wild · front · rub · pump', _G.rub, _speaker, b: .3, decay: .25, spd: (s) => 1.5 + s.b * 4),
      _Pan9('Funhouse', 'material · bold flat · 2D · drag · warp x/y', _G.xy, _funhouse, a: .65, b: .4),
    ];

double _jig(_S st, double t) {
  final dt = t - st.rel;
  return st.down || dt < 0 || dt > 3 ? 0 : math.sin(dt * 18) * math.exp(-dt * 3);
}

// 1. Rub to pump air in; it slowly leaks. Bigger is the whole point.
void _balloon(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _ye);
  final r = 10 + st.b * 34, o = Offset(78, 58 - st.b * 10 + math.sin(t * 1.4) * 2);
  final knot = o + Offset(0, r * 1.15);
  c.drawPath(
      Path()
        ..moveTo(knot.dx, knot.dy)
        ..quadraticBezierTo(knot.dx - 10, (knot.dy + 116) / 2, knot.dx + 4, 116),
      _s(_k0, 1.5));
  c.drawOval(Rect.fromCenter(center: o, width: r * 2, height: r * 2.3), _f(_rd));
  c.drawPath(_poly([knot - const Offset(0, 2), knot + const Offset(-4, 5), knot + const Offset(4, 5)]), _f(_rd));
  c.drawOval(Rect.fromCenter(center: o + Offset(-r * .4, -r * .5), width: r * .35, height: r * .6), _f(_al(_wh, .7)));
  if (st.down) {
    for (var i = 0; i < 3; i++) {
      final a = -math.pi / 2 + (i - 1) * .5, p = _pol(o, r * 1.3 + 6 + (t * 30 % 6), a);
      c.drawLine(p, _pol(o, r * 1.3 + 12 + (t * 30 % 6), a), _s(_k0, 2));
    }
  }
}

// 2. Push the jelly down and it bulges sideways; let go and it wobbles back.
void _jelly(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k1);
  c.drawOval(const Rect.fromLTWH(18, 88, 120, 22), _f(_cr));
  final j = _jig(st, t) * .15, h = (16 + st.b * 50) * (1 + j), w = 3000 / h * .62;
  const base = 98.0;
  final l = 78 - w / 2, d = w * .28;
  final front = Rect.fromLTRB(l, base - h, l + w, base);
  c.drawPath(_poly([Offset(l + w, base - h), Offset(l + w + d, base - h - d * .5), Offset(l + w + d, base - d * .5), Offset(l + w, base)]),
      _f(const Color(0xFF5FA83A)));
  c.drawPath(_poly([Offset(l, base - h), Offset(l + d, base - h - d * .5), Offset(l + w + d, base - h - d * .5), Offset(l + w, base - h)]),
      _f(const Color(0xFFC6F59A)));
  _rr(c, front, 3, _f(_li));
  for (var i = 0; i < 5; i++) {
    c.drawCircle(Offset(l + w * (.15 + _rn(i) * .7), base - h * (.15 + _rn(i + 5) * .7)), 3, _f(i.isEven ? _rd : _ye));
  }
  c.drawLine(Offset(l + 5, base - h + 5), Offset(l + 5, base - 6), _s(_al(_wh, .6), 2.5));
  if (st.down && st.at != null) c.drawCircle(Offset(st.at!.dx, base - h - d * .25), 6, _f(_al(_pk, .9)));
}

// 3. A ball bouncing: across = how much it squashes and stretches, up = how high it goes.
void _bounce(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k0);
  c.drawLine(const Offset(10, 104), const Offset(146, 104), _s(_cr, 2));
  final per = .9, u = (t % per) / per, y01 = 4 * u * (1 - u), hgt = 20 + st.b * 66;
  final vel = (1 - 2 * u).abs(), sq = st.a;
  final contact = y01 < .08 ? (1 - y01 / .08) : 0.0;
  final sy = 1 + vel * sq * .5 - contact * sq * .55, sx = 1 / sy;
  const r = 11.0;
  for (var k = 3; k >= 1; k--) {
    final uu = _wr(u - k * .04), yy = 4 * uu * (1 - uu);
    c.drawCircle(Offset(78, 104 - r - yy * hgt), r * .8, _f(_al(_or, .12 * (4 - k))));
  }
  final cy = 104 - r * sy - y01 * hgt;
  c.drawOval(Rect.fromCenter(center: Offset(78, 104), width: 26 * (1 - y01 * .6), height: 5), _f(_al(_cr, .25)));
  c.drawOval(Rect.fromCenter(center: Offset(78, cy), width: r * 2 * sx, height: r * 2 * sy), _f(_or));
  c.drawOval(Rect.fromCenter(center: Offset(78 - 3 * sx, cy - 4 * sy), width: 6 * sx, height: 4 * sy), _f(_al(_wh, .7)));
}

// 4. Grab the corner of the picture: x and y stretch apart; bring them together and the link closes.
void _photoCorner(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cr);
  const x0 = 14.0, y0 = 12.0, fw = _cw - 28, fh = _ch - 24;
  var a = st.a, b = st.b;
  final linked = (a - b).abs() < .05;
  if (linked) a = b = (a + b) / 2;
  c.drawRect(const Rect.fromLTWH(x0, y0, fw * .5, fh * .5), _s(_al(_k0, .25), 1));
  final r = Rect.fromLTWH(x0, y0, math.max(8, fw * a), math.max(8, fh * b));
  _pic(c, r, sky: _cy);
  c.drawRect(r, _s(_k0, 1.5));
  c.drawCircle(r.bottomRight, 6, _f(linked ? _li : _or));
  c.drawCircle(r.bottomRight, 6, _s(_k0, 1.5));
  final lc = Offset(_cw - 20, _ch - 16);
  final gap = linked ? 0.0 : 3.0;
  _rr(c, Rect.fromCenter(center: lc - Offset(4 + gap, 0), width: 10, height: 6), 3, _s(linked ? _li : _k0, 2));
  _rr(c, Rect.fromCenter(center: lc + Offset(4 + gap, 0), width: 10, height: 6), 3, _s(linked ? _li : _k0, 2));
}

// 5. Pinch outward and the pufferfish puffs: same fish, just bigger, spikes out.
void _puffer(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _bl);
  for (var i = 0; i < 4; i++) {
    final y = 116 - ((t * 18 + i * 31) % 130);
    c.drawCircle(Offset(24 + i * 34 + math.sin(t + i) * 3, y), 2 + i % 2, _s(_al(_cy, .7), 1.2));
  }
  final r = 12 + st.a * 30, o = Offset(74 + math.sin(t * 1.2) * 2, 58);
  for (var i = 0; i < 16; i++) {
    final a = i * _tau / 16;
    c.drawPath(_poly([_pol(o, r * .9, a - .12), _pol(o, r + 2 + st.a * 9, a), _pol(o, r * .9, a + .12)]), _f(_or));
  }
  c.drawPath(_poly([o + Offset(r * .8, 0), o + Offset(r + 12, -9), o + Offset(r + 12, 9)]), _f(_or));
  c.drawCircle(o, r, _f(_ye));
  c.drawOval(Rect.fromCenter(center: o + Offset(0, r * .45), width: r * 1.4, height: r * .8), _f(_cr));
  c.drawCircle(o + Offset(-r * .45, -r * .2), 3 + r * .1, _f(_wh));
  c.drawCircle(o + Offset(-r * .5, -r * .2), 1.5 + r * .06, _f(_k0));
  c.drawOval(Rect.fromCenter(center: o + Offset(-r * .85, r * .15), width: 5, height: 4 + st.a * 3), _f(_rd));
}

// 6. Nesting dolls: across = how much each is smaller than the last, up = the size of the first.
void _dolls(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cr);
  c.drawLine(const Offset(0, 104), const Offset(156, 104), _s(_al(_k0, .3), 2));
  final k = .55 + st.a * .4;
  var h = 26 + st.b * 58, x = 8.0;
  for (var i = 0; i < 6 && h > 6 && x < 150; i++) {
    final w = h * .62;
    final body = Rect.fromLTWH(x, 104 - h, w, h);
    c.drawOval(Rect.fromLTWH(x, 104 - h * .72, w, h * .72), _f(i.isEven ? _rd : _or));
    c.drawOval(Rect.fromLTWH(x + w * .1, 104 - h, w * .8, h * .5), _f(i.isEven ? _rd : _or));
    c.drawCircle(Offset(x + w / 2, 104 - h * .76), w * .26, _f(_wh));
    c.drawCircle(Offset(x + w * .42, 104 - h * .78), math.max(.8, w * .03), _f(_k0));
    c.drawCircle(Offset(x + w * .58, 104 - h * .78), math.max(.8, w * .03), _f(_k0));
    c.drawCircle(Offset(x + w / 2, 104 - h * .32), w * .16, _f(_ye));
    c.drawOval(body.deflate(0), _s(_al(_k0, .15), 1));
    x += w + 3;
    h *= k;
  }
}

// 7. Stretch the dough out from the middle: wider and taller apart; the toppings spread with it but stay round.
void _dough(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFFB8541F));
  for (var i = 0; i < 5; i++) {
    c.drawLine(Offset(0, 14 + i * 22.0), Offset(156, 10 + i * 22.0), _s(_al(_k0, .12), 2));
  }
  final w = 40 + st.a * 104, h = 30 + st.b * 78;
  final wob = _jig(st, t) * .05;
  c.drawOval(Rect.fromCenter(center: _ctr, width: w * (1 + wob), height: h * (1 - wob)), _f(_cr));
  c.drawOval(Rect.fromCenter(center: _ctr, width: w * .82, height: h * .82), _f(_rd));
  for (var i = 0; i < 7; i++) {
    final a = _rn(i) * _tau, rr = .15 + _rn(i + 9) * .6;
    c.drawCircle(_ctr + Offset(math.cos(a) * w * .41 * rr, math.sin(a) * h * .41 * rr), 5, _f(const Color(0xFF8E1F1F)));
    c.drawCircle(_ctr + Offset(math.cos(a + 2) * w * .41 * rr, math.sin(a + 2) * h * .41 * rr), 2.2, _f(_li));
  }
  for (var i = 0; i < 14; i++) {
    c.drawCircle(Offset(_rn(i + 40) * 156, _rn(i + 60) * 116), 1, _f(_al(_wh, .6)));
  }
}

// 8. Pull the accordion open: only its width changes, the pleats fan out.
void _accordion(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _vi);
  final w = 24 + st.a * 92 + _jig(st, t) * 6, l = 78 - w / 2 - 14, r = 78 + w / 2 + 14;
  _rr(c, Rect.fromLTWH(l - 14, 28, 18, 60), 4, _f(_rd));
  _rr(c, Rect.fromLTWH(r - 4, 28, 18, 60), 4, _f(_rd));
  for (var i = 0; i < 3; i++) {
    c.drawCircle(Offset(l - 5, 40 + i * 14.0), 3, _f(_cr));
    c.drawCircle(Offset(r + 5, 40 + i * 14.0), 3, _f(_cr));
  }
  const n = 9;
  final step = (r - l - 8) / n;
  final top = <Offset>[], bot = <Offset>[];
  for (var i = 0; i <= n; i++) {
    final x = l + 4 + i * step, dip = i.isOdd ? 6.0 : 0.0;
    top.add(Offset(x, 30 + dip));
    bot.add(Offset(x, 86 - dip));
  }
  c.drawPath(_poly([...top, ...bot.reversed]), _f(_k0));
  for (var i = 0; i <= n; i++) {
    c.drawLine(top[i], bot[i], _s(i.isOdd ? _ye : _cr, 2));
  }
  if (st.a > .5) {
    for (var i = 0; i < 2; i++) {
      final rr = 10 + ((t * 30 + i * 12) % 24);
      c.drawArc(Rect.fromCircle(center: Offset(78, 14), radius: rr * .5), math.pi * 1.15, .7, false, _s(_al(_ye, 1 - rr / 34), 2));
    }
  }
}

// 9. Pull the cat long: head and paws keep their size, the body gets longer and thinner.
void _cat(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cr);
  final len = 22 + st.a * 92, th = (900 / len).clamp(9, 30).toDouble();
  final l = 78 - len / 2, r = 78 + len / 2, y = 74.0;
  c.drawLine(const Offset(0, 100), const Offset(156, 100), _s(_al(_k0, .2), 2));
  c.drawPath(
      Path()
        ..moveTo(l, y)
        ..quadraticBezierTo(l - 18, y - 6, l - 12, y - 28 + math.sin(t * 3) * 3),
      _s(_or, 5));
  _rr(c, Rect.fromLTRB(l, y - th / 2, r, y + th / 2), th / 2, _f(_or));
  for (var i = 0; i < 4; i++) {
    final x = _lr(l + 8, r - 8, i / 3);
    c.drawLine(Offset(x, y - th * .3), Offset(x, y + th * .3), _s(const Color(0xFFC85A24), 3));
  }
  for (final x in [l + 6, l + 14, r - 6, r - 14]) {
    c.drawLine(Offset(x, y + th / 2 - 2), Offset(x, 100), _s(_or, 5));
  }
  final hd = Offset(r + 10, y - th / 2 - 4);
  c.drawPath(_poly([hd + const Offset(-11, -6), hd + const Offset(-8, -20), hd + const Offset(-1, -10)]), _f(_or));
  c.drawPath(_poly([hd + const Offset(11, -6), hd + const Offset(8, -20), hd + const Offset(1, -10)]), _f(_or));
  c.drawCircle(hd, 13, _f(_or));
  c.drawCircle(hd + const Offset(-5, -2), 2, _f(_k0));
  c.drawCircle(hd + const Offset(5, -2), 2, _f(_k0));
  c.drawCircle(hd + const Offset(0, 4), 1.8, _f(_pk));
}

// 10. Pinch the moon closer: it swells over the skyline; the little rocket stays its size.
void _moon(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF0D1030));
  for (var i = 0; i < 26; i++) {
    c.drawCircle(Offset(_rn(i) * 156, _rn(i + 70) * 90), .9, _f(_al(_wh, .4 + .4 * math.sin(t * 2 + i))));
  }
  final r = 8 + st.a * 62, o = Offset(96, 58 + st.a * 12);
  c.drawCircle(o, r + 6, _f(_al(_ye, .12)));
  c.drawCircle(o, r, _f(_cr));
  for (var i = 0; i < 6; i++) {
    c.drawCircle(o + Offset((_rn(i + 3) - .5) * r * 1.2, (_rn(i + 13) - .5) * r * 1.2), r * (.08 + _rn(i + 23) * .1),
        _f(const Color(0xFFD9CBB4)));
  }
  final sky = Path()..moveTo(0, 116);
  for (var i = 0; i < 9; i++) {
    final x = i * 20.0, h = 10 + _rn(i + 90) * 22;
    sky
      ..lineTo(x, 116 - h)
      ..lineTo(x + 16, 116 - h);
  }
  sky
    ..lineTo(156, 116)
    ..close();
  c.drawPath(sky, _f(_k0));
  final rk = Offset(30, 44 + math.sin(t * 2) * 3);
  c.drawPath(_poly([rk + const Offset(0, -9), rk + const Offset(4, 3), rk + const Offset(-4, 3)]), _f(_pk));
  c.drawLine(rk + const Offset(0, 4), rk + Offset(0, 9 + math.sin(t * 20) * 1.5), _s(_or, 3));
}

// 11. Stretch the range: across makes it broader, up makes the peaks taller.
void _mountains(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _cr);
  c.drawCircle(const Offset(122, 30), 12, _f(_or));
  final sx = .5 + st.a * 1.0, sy = .3 + st.b * 1.1;
  const peaks = [(-40.0, 50.0, _vi), (0.0, 70.0, _bl), (38.0, 46.0, _vi)];
  for (final (dx, h, col) in peaks) {
    final px = 78 + dx * sx, ph = h * sy, bw = 46 * sx;
    final top = Offset(px, 110 - ph);
    c.drawPath(_poly([Offset(px - bw, 110), top, Offset(px + bw, 110)]), _f(col));
    final k = math.min(.32, 14 / math.max(ph, 1));
    c.drawPath(_poly([top, Offset(px + bw * k, 110 - ph * (1 - k)), Offset(px, 110 - ph * (1 - k * .7)), Offset(px - bw * k, 110 - ph * (1 - k))]),
        _f(_wh));
  }
  c.drawRect(const Rect.fromLTWH(0, 108, 156, 8), _f(_li));
}

// 12. Pull the taffy: the longer it gets, the thinner the middle; the stripes stretch with it.
void _taffy(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k1);
  final half = 14 + st.a * 52, l = 78 - half, r = 78 + half, neck = (220 / (half + 8)).clamp(2.5, 16).toDouble();
  final p = Path()
    ..moveTo(l, 44)
    ..cubicTo(l + half * .5, 58 - neck, r - half * .5, 58 - neck, r, 44)
    ..lineTo(r, 72)
    ..cubicTo(r - half * .5, 58 + neck, l + half * .5, 58 + neck, l, 72)
    ..close();
  c.drawPath(p, _f(_pk));
  c.save();
  c.clipPath(p);
  for (var i = -6; i < 7; i++) {
    final x = 78 + i * half / 4;
    c.drawLine(Offset(x - 6, 40), Offset(x + 6, 76), _s(_cr, 3));
  }
  c.restore();
  for (final (x, sgn) in [(l, -1.0), (r, 1.0)]) {
    c.drawCircle(Offset(x + sgn * 4, 58), 15, _f(_ye));
    for (var i = 0; i < 3; i++) {
      c.drawLine(Offset(x - sgn * 4, 50 + i * 7.0), Offset(x - sgn * 9, 50 + i * 7.0), _s(_k1, 2));
    }
  }
}

// 13. An iso tower: stretch across widens the footprint, up raises it; the windows stretch with the walls.
void _tower(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k0);
  for (var i = -6; i < 8; i++) {
    c.drawLine(Offset(i * 24.0, 116), Offset(i * 24.0 + 84, 74), _s(_al(_cy, .12), 1));
    c.drawLine(Offset(i * 24.0, 74), Offset(i * 24.0 + 84, 116), _s(_al(_cy, .12), 1));
  }
  final w = 14 + st.a * 28, h = 14 + st.b * 54;
  const g = Offset(78, 104);
  final lft = Offset(-w, -w * .5), rgt = Offset(w, -w * .5), up = Offset(0, -h);
  c.drawPath(_poly([g, g + lft, g + lft + up, g + up]), _f(_bl));
  c.drawPath(_poly([g, g + rgt, g + rgt + up, g + up]), _f(_cy));
  c.drawPath(_poly([g + up, g + lft + up, g + lft + rgt + up, g + rgt + up]), _f(_wh));
  for (var i = 0; i < 3; i++) {
    for (var j = 0; j < 4; j++) {
      for (final side in [lft, rgt]) {
        final u0 = (i + .25) / 3, u1 = (i + .75) / 3, v0 = (j + .25) / 4, v1 = (j + .7) / 4;
        Offset q(double u, double v) => g + side * u + up * v;
        c.drawPath(_poly([q(u0, v0), q(u1, v0), q(u1, v1), q(u0, v1)]), _f(side == lft ? _ye : _al(_ye, .75)));
      }
    }
  }
}

// 14. A slinky hanging from the top: pull down and the coils spread; only the height changes.
void _slinky(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF0E0E12));
  c.drawRect(const Rect.fromLTWH(30, 4, 96, 6), _f(_cr));
  final len = 18 + st.b * 86 + _jig(st, t) * 8 + math.sin(t * 2.2) * 1.5;
  const n = 16;
  for (var i = 0; i < n; i++) {
    final y = 12 + i * len / n;
    final col = HSLColor.fromAHSL(1, (i * 22 + t * 30) % 360, .9, .62).toColor();
    c.drawOval(Rect.fromCenter(center: Offset(78, y), width: 54, height: 9), _s(col, 3));
  }
}

// 15. Move the puppet between lamp and wall: the nearer the lamp, the bigger the shadow.
void _shadow(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k0);
  const lamp = Offset(14, 58), wallX = 140.0;
  c.drawRect(const Rect.fromLTWH(wallX, 0, 16, 116), _f(_cr));
  final px = 30 + st.a * 96, k = (wallX - lamp.dx) / (px - lamp.dx), ph = 12.0;
  c.drawPath(_poly([lamp, Offset(wallX, 58 - ph * k), Offset(wallX, 58 + ph * k)]), _f(_al(_ye, .16)));
  c.drawRect(Rect.fromLTWH(wallX, 58 - ph * k, 16, ph * 2 * k), _f(_k1));
  c.drawRect(Rect.fromLTWH(wallX, 58 - ph * k - ph * .5 * k, 6, ph * .5 * k), _f(_k1));
  c.drawRect(Rect.fromLTWH(wallX + 8, 58 - ph * k - ph * .7 * k, 6, ph * .7 * k), _f(_k1));
  c.drawCircle(lamp, 8, _f(_ye));
  c.drawCircle(lamp, 12, _s(_al(_ye, .4), 2));
  _rr(c, Rect.fromCenter(center: Offset(px, 58), width: 14, height: ph * 2), 6, _f(_pk));
  _rr(c, Rect.fromLTWH(px - 7, 58 - ph - 6, 5, 8), 2, _f(_pk));
  _rr(c, Rect.fromLTWH(px + 2, 58 - ph - 9, 5, 11), 2, _f(_pk));
  c.drawCircle(Offset(px - 2, 54), 1.6, _f(_k0));
  c.drawLine(Offset(px, 58 + ph), Offset(px, 112), _s(_cr, 2));
}

// 16. A sprite scales in whole pixel steps: pinch out and every pixel becomes a bigger block.
void _pixel(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _k1);
  const rows = ['00100100', '00011000', '00111100', '01101110', '11111111', '10111101', '10100101', '00011000'];
  final px = (1 + st.a * 9).floorToDouble(), side = px * 8;
  final o = _ctr - Offset(side / 2, side / 2) + Offset(0, math.sin(t * 3) * 2).round();
  for (var y = 0; y < 8; y++) {
    for (var x = 0; x < 8; x++) {
      if (rows[y][x] == '1') c.drawRect(Rect.fromLTWH(o.dx + x * px, o.dy + y * px, px, px), _f(y < 3 ? _li : (y == 3 && x == 2 ? _k1 : _pk)));
    }
  }
  if (px >= 4) {
    for (var x = 0; x <= 8; x++) {
      c.drawLine(Offset(o.dx + x * px, o.dy), Offset(o.dx + x * px, o.dy + side), _s(_al(_k0, .25), 1));
      c.drawLine(Offset(o.dx, o.dy + x * px), Offset(o.dx + side, o.dy + x * px), _s(_al(_k0, .25), 1));
    }
  }
}

extension on Offset {
  Offset round() => Offset(dx.roundToDouble(), dy.roundToDouble());
}

// 17. A ray gun: drag right to grow the creature, left to shrink it; the beam colour says which.
void _ray(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF120A24));
  final k = math.pow(2, (st.a - .5) * 3.2).toDouble(), grow = st.a >= .5, col = grow ? _pk : _cy;
  const gun = Offset(22, 52);
  final tgt = Offset(108, 100 - 18 * k);
  final beam = Path()..moveTo(gun.dx + 14, gun.dy);
  for (var i = 1; i <= 8; i++) {
    final p = Offset.lerp(gun + const Offset(14, 0), tgt, i / 8)!;
    beam.lineTo(p.dx, p.dy + (i.isOdd ? -5 : 5) * math.sin(t * 20));
  }
  c.drawPath(beam, _s(_al(col, .3), 7));
  c.drawPath(beam, _s(col, 2));
  _rr(c, Rect.fromCenter(center: gun, width: 26, height: 12), 5, _f(_ye));
  c.drawCircle(gun + const Offset(14, 0), 4, _f(col));
  _rr(c, Rect.fromLTWH(gun.dx - 10, gun.dy + 4, 8, 14), 2, _f(_ye));
  c.drawLine(const Offset(60, 100), const Offset(156, 100), _s(_al(_cr, .3), 1.5));
  c.drawOval(Rect.fromCenter(center: tgt, width: 34 * k, height: 36 * k), _f(_li));
  c.drawCircle(tgt + Offset(-6 * k, -5 * k), 4 * k, _f(_wh));
  c.drawCircle(tgt + Offset(6 * k, -5 * k), 4 * k, _f(_wh));
  c.drawCircle(tgt + Offset(-5 * k, -5 * k), 2 * k, _f(_k0));
  c.drawCircle(tgt + Offset(7 * k, -5 * k), 2 * k, _f(_k0));
  c.drawArc(Rect.fromCenter(center: tgt + Offset(0, 5 * k), width: 10 * k, height: 6 * k), 0, math.pi, false, _s(_k0, math.max(1, k)));
}

// 18. Stretch the cloud: wider spreads it, taller builds a thunderhead that starts to rain and flash.
void _cloud(Canvas c, Size s, _S st, double t) {
  final tall = st.b;
  _bg(c, s, Color.lerp(_cy, const Color(0xFF3B4A6B), tall)!);
  final w = 30 + st.a * 50, h = 10 + tall * 40, base = 72.0;
  final col = Color.lerp(_wh, const Color(0xFFB8BCD0), tall)!;
  for (var i = 0; i < 9; i++) {
    final u = (i % 3) / 2 - .5, v = (i ~/ 3) / 2;
    final p = Offset(78 + u * w * (1 - v * .4), base - v * h);
    c.drawCircle(p, 14 + w * .12 * (1 - v * .3), _f(col));
  }
  c.drawRect(Rect.fromLTRB(78 - w * .5 - 10, base - 4, 78 + w * .5 + 10, base + 12), _f(col));
  if (tall > .4) {
    for (var i = 0; i < 10; i++) {
      final x = 78 - w * .5 + _rn(i) * w, y = base + 14 + ((t * 70 + _rn(i + 5) * 40) % 36);
      c.drawLine(Offset(x, y), Offset(x - 2, y + 6), _s(_cy, 2));
    }
  }
  if (tall > .7 && (t * 1.3) % 1 < .12) {
    c.drawPath(_poly(const [Offset(80, 82), Offset(72, 98), Offset(80, 98), Offset(74, 114), Offset(90, 94), Offset(82, 94), Offset(88, 82)]),
        _f(_ye));
  }
}

// 19. Rub to pump the speaker: the cone pushes in and out, the rings show how big the beat is.
void _speaker(Canvas c, Size s, _S st, double t) {
  _bg(c, s, const Color(0xFF0B0B10));
  final pump = st.b, beat = math.pow(math.sin(st.ph * math.pi).abs(), 6).toDouble(), sc = 1 + beat * pump * .35;
  for (var i = 0; i < 4; i++) {
    final u = _wr(st.ph * .5 + i / 4), rr = 30 + u * 60;
    c.drawCircle(_ctr, rr, _s(_al(i.isEven ? _pk : _cy, (1 - u) * pump), 2.5));
  }
  _rr(c, Rect.fromCenter(center: _ctr, width: 88, height: 104), 10, _f(_k1));
  c.drawCircle(_ctr, 38, _f(_k0));
  c.drawCircle(_ctr, 34 * sc, _f(_vi));
  c.drawCircle(_ctr, 22 * sc, _f(const Color(0xFF7457E0)));
  c.drawCircle(_ctr, 11 * sc, _f(_or));
  c.drawCircle(_ctr - Offset(3 * sc, 3 * sc), 3 * sc, _f(_al(_wh, .6)));
}

// 20. A funhouse mirror: across fattens, up stretches tall; the bend moves through the reflection.
void _funhouse(Canvas c, Size s, _S st, double t) {
  _bg(c, s, _rd);
  const fr = Rect.fromLTWH(40, 6, 76, 104);
  _rr(c, fr.inflate(4), 38, _f(_ye));
  final mir = RRect.fromRectAndRadius(fr, const Radius.circular(34));
  c.drawRRect(mir, _f(const Color(0xFFBDEFF8)));
  c.save();
  c.clipRRect(mir);
  final sx = .5 + st.a * 1.5, sy = .5 + st.b * .6;
  const n = 24;
  final top = 104 - 72 * sy;
  for (var i = 0; i < n; i++) {
    final v = i / n, yy = 104 - (v + .5 / n) * 72 * sy;
    final warp = 1 + .35 * math.sin(v * 6 + t * 1.6) * (sx - .5);
    final wv = v < .45 ? 7.0 : 13.0 - (v - .45) * 8;
    final col = v < .45 ? _bl : _or;
    c.drawRect(Rect.fromCenter(center: Offset(78, yy), width: wv * 2 * sx * warp, height: 72 * sy / n + 1), _f(col));
  }
  final hw = 1 + .35 * math.sin(6.6 + t * 1.6) * (sx - .5);
  c.drawOval(Rect.fromCenter(center: Offset(78, top - 11 * sy), width: 22 * sx * hw, height: 22 * sy), _f(_cr));
  c.drawCircle(Offset(78 - 4 * sx * hw, top - 12 * sy), 1.8, _f(_k0));
  c.drawCircle(Offset(78 + 4 * sx * hw, top - 12 * sy), 1.8, _f(_k0));
  c.restore();
  c.drawLine(const Offset(50, 20), const Offset(60, 12), _s(_al(_wh, .9), 3));
}
