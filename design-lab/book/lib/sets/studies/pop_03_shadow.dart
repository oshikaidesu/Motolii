// Shadow sheet: direction (where it falls), distance (how far from the object), softness (how hard its edge).
// Most panels hand the finger the light (sun, lamp, candle, torch) or the object's height; the shadow follows from that.
part of 'pop_03.dart';

const _shadowPanels = <_P>[
  _P('Sun over Blocks', 'landscape · bold · iso · drag', _sBlocks, init: Offset(.2, .2)),
  _P('Desk Lamp', 'machine · light · side · drag', _sDesk, init: Offset(.3, .25), bg: _kCream, anim: false),
  _P('Pop Sticker', 'toy · bold · flat · drag', _sSticker, init: Offset(.68, .7), bg: _kLime, anim: false),
  _P('Night Walk', 'character · crisp · side · drag', _sWalk, init: Offset(.2, .3), bg: Color(0xFF15172B)),
  _P('Sundial', 'instrument · light · top · drag', _sDial, init: Offset(.15, .3), bg: _kCream, anim: false),
  _P('Cloud over Fields', 'weather · bold · top · drag up', _sCloud, init: Offset(.45, .4)),
  _P('Planet Night Side', 'cosmic · crisp · 3D · drag', _sPlanet, init: Offset(.12, .25), bg: _kNight, anim: false),
  _P('Lifted Card', 'UI card · light · flat · lift up', _sCard, init: Offset(.4, .25), bg: _kCream, anim: false),
  _P('Long Shadow Badge', 'icon · bold · flat · drag', _sLong, init: Offset(.82, .82), anim: false),
  _P('Golden Hour Street', 'landscape · bold · side · drag', _sSkyline, init: Offset(.25, .38), anim: false),
  _P('Paper Cut Layers', 'material · light · stacked · lift up', _sPaper, init: Offset(.35, .35), bg: _kCream, anim: false),
  _P('RGB Lights', 'physics · neon! · flat · drag', _sRgb, init: Offset(.5, .55), bg: _kNight, anim: false),
  _P('Pinned Photo', 'result · bold · 3D · lift', _sPhoto, init: Offset(.35, .3), bg: Color(0xFFC77A3A), anim: false),
  _P('Bouncing Ball', 'physics · crisp · 3D · throw', _sBall, init: Offset(.3, .25)),
  _P('70s Echo Stack', 'text-art · bold · flat · drag', _sRetro, init: Offset(.72, .72), bg: _kCream, anim: false),
  _P('Tree Sun Coins', 'nature · bold · side · drag', _sTree, init: Offset(.2, .2), bg: _kPanel2, anim: false),
  _P('Plane over Dunes', 'machine · bold · top · drag', _sPlane, init: Offset(.3, .3), bg: _kYellow),
  _P('Candle & Vase', 'nature · warm · side · drag', _sCandle, init: Offset(.42, .7), bg: Color(0xFF1B1310)),
  _P('Floating Island', 'landscape · bold · 3D · drag up', _sIsland, init: Offset(.45, .3), bg: _kBlue),
  _P('Shadow Monster', 'creature · neon! · side · drag', _sMonster, init: Offset(.75, .85), bg: Color(0xFF130F1D)),
];

void _sBlocks(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, gc = Offset(w * .5, h * .62);
  Offset iso(double u, double v, [double z = 0]) => gc + Offset((u - v) * 36, (u + v) * 18 - z);
  c.drawPath(_poly([iso(-1, -1), iso(1, -1), iso(1, 1), iso(-1, 1)]), _f(_kLime));
  final sp = st.at(s), dx = sp.dx - gc.dx, dy = sp.dy - gc.dy;
  var g = Offset((dx / 36 + dy / 18) / 2, (dy / 18 - dx / 36) / 2);
  g = g.distance < .01 ? const Offset(1, 0) : g / g.distance;
  final len = .25 + (1 - st.b) * 1.7, sig = 1 + (1 - st.b) * 3;
  const blocks = [(-.45, -.35, .22, 26.0, _kOrange), (.4, -.3, .18, 38.0, _kPink), (-.05, .45, .22, 18.0, _kCyan)];
  final sh = _soft(_al(_kInk, .45), sig);
  for (final (u0, v0, sz, ht, _) in blocks) {
    final p = Path();
    for (var k = 0; k <= 8; k++) {
      final o = -g * (len * ht / 36 * k / 8);
      p.addPolygon([iso(u0 - sz + o.dx, v0 - sz + o.dy), iso(u0 + sz + o.dx, v0 - sz + o.dy), iso(u0 + sz + o.dx, v0 + sz + o.dy), iso(u0 - sz + o.dx, v0 + sz + o.dy)], true);
    }
    c.drawPath(p, sh);
  }
  final order = [...blocks]..sort((a, b) => (a.$1 + a.$2).compareTo(b.$1 + b.$2));
  for (final (u0, v0, sz, ht, col) in order) {
    c.drawPath(_poly([iso(u0 - sz, v0 + sz), iso(u0 + sz, v0 + sz), iso(u0 + sz, v0 + sz, ht), iso(u0 - sz, v0 + sz, ht)]), _f(_mix(col, _kInk, .4)));
    c.drawPath(_poly([iso(u0 + sz, v0 - sz), iso(u0 + sz, v0 + sz), iso(u0 + sz, v0 + sz, ht), iso(u0 + sz, v0 - sz, ht)]), _f(_mix(col, _kInk, .2)));
    c.drawPath(_poly([iso(u0 - sz, v0 - sz, ht), iso(u0 + sz, v0 - sz, ht), iso(u0 + sz, v0 + sz, ht), iso(u0 - sz, v0 + sz, ht)]), _f(col));
  }
  c.drawCircle(sp, 8, _f(_kYellow));
  for (var k = 0; k < 8; k++) {
    final d = _dir(k * math.pi / 4 + t * .5);
    c.drawLine(sp + d * 11, sp + d * 15, _s(_kYellow, 2));
  }
}

void _sDesk(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, deskY = h * .68;
  final tp = st.at(s), lamp = Offset(tp.dx.clamp(w * .12, w * .92), tp.dy.clamp(h * .08, deskY - 40));
  c.drawRect(Rect.fromLTWH(0, deskY, w, 16), _f(const Color(0xFFE5D2B5)));
  c.drawRect(Rect.fromLTWH(0, deskY + 16, w, h), _f(_kOrange));
  c.drawPath(_poly([lamp, Offset(lamp.dx - 46, deskY + 16), Offset(lamp.dx + 46, deskY + 16)]), _f(_al(_kYellow, .3)));
  const objs = [(.5, 16.0, 18.0, _kPink), (.76, 10.0, 32.0, _kBlue)];
  for (final (fx, ow, oh, _) in objs) {
    final x0 = w * fx, x1 = x0 + ow, top = deskY + 8 - oh;
    double tip(double x) => lamp.dx + (x - lamp.dx) * (deskY + 8 - lamp.dy) / (top - lamp.dy);
    final a = math.min(math.min(x0, x1), math.min(tip(x0), tip(x1))), b = math.max(math.max(x0, x1), math.max(tip(x0), tip(x1)));
    final sig = 1 + (Offset(x0, top) - lamp).distance * .05;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(a, deskY + 4, b, deskY + 12), const Radius.circular(4)), _soft(_al(_kInk, .4), sig));
  }
  for (final (fx, ow, oh, col) in objs) {
    final x0 = w * fx;
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x0, deskY + 8 - oh, ow, oh), const Radius.circular(3)), _f(col));
    if (ow > 12) c.drawArc(Rect.fromLTWH(x0 + ow - 3, deskY + 8 - oh + 4, 9, 9), -math.pi / 2, math.pi, false, _s(col, 2.5));
    if (ow < 12) c.drawCircle(Offset(x0 + ow / 2, deskY + 8 - oh - 5), 6, _f(_kLime));
  }
  final base = Offset(w * .08, deskY + 8), elbow = Offset((base.dx + lamp.dx) / 2 - 8, math.min(base.dy, lamp.dy) - 12);
  c.drawPath(_poly([base, elbow, lamp], close: false), _s(_kInk, 3));
  c.drawOval(Rect.fromCenter(center: base, width: 22, height: 6), _f(_kInk));
  c.drawArc(Rect.fromCircle(center: lamp, radius: 9), math.pi, math.pi, true, _f(_kInk));
  c.drawCircle(lamp + const Offset(0, 1), 3.5, _f(_kYellow));
}

void _sSticker(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2);
  final pts = [for (var k = 0; k < 24; k++) ctr + _dir(k * math.pi / 12 - .1) * (k.isEven ? 40.0 : 31.0)];
  final burst = _poly(pts);
  final off = (st.at(s) - ctr) * .4;
  c.drawPath(burst.shift(off), _f(_kInk));
  c.drawPath(burst, _f(_kYellow));
  c.drawPath(burst, _s(_kInk, 3));
  final heart = Path()
    ..moveTo(ctr.dx, ctr.dy + 12)
    ..cubicTo(ctr.dx - 22, ctr.dy - 2, ctr.dx - 10, ctr.dy - 18, ctr.dx, ctr.dy - 7)
    ..cubicTo(ctr.dx + 10, ctr.dy - 18, ctr.dx + 22, ctr.dy - 2, ctr.dx, ctr.dy + 12)
    ..close();
  c.drawPath(heart, _f(_kPink));
  c.drawPath(heart, _s(_kInk, 2.2));
}

void _sWalk(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, gy = h * .82;
  final tp = st.at(s), lamp = Offset(tp.dx, tp.dy.clamp(h * .1, gy - 40));
  c.drawRect(Rect.fromLTWH(0, gy, w, h - gy), _f(const Color(0xFF4A4C6E)));
  c.drawOval(Rect.fromCenter(center: Offset(lamp.dx, gy + 6), width: 150, height: 16), _f(_al(_kYellow, .35)));
  c.drawCircle(lamp, 46, Paint()..shader = ui.Gradient.radial(lamp, 46, [_al(_kYellow, .3), _al(_kYellow, 0)]));
  c.drawLine(Offset(lamp.dx, gy), lamp + const Offset(0, 4), _s(_kCream, 2.5));
  c.drawCircle(lamp, 5.5, _f(_kYellow));
  final fx = w * .5 + math.sin(t * .45) * w * .3, figH = 26.0;
  final tip = lamp.dx + (fx - lamp.dx) * (gy - lamp.dy) / (gy - figH - lamp.dy);
  final sig = 1 + (fx - lamp.dx).abs() * .03;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(math.min(fx, tip), gy + 2, math.max(fx, tip), gy + 9), const Radius.circular(3)), _soft(_al(const Color(0xFF0A0A14), .9), sig));
  final step = math.sin(t * 6);
  c.drawLine(Offset(fx, gy - 10), Offset(fx - 4 * step, gy), _s(_kCyan, 3));
  c.drawLine(Offset(fx, gy - 10), Offset(fx + 4 * step, gy), _s(_kCyan, 3));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(fx - 5, gy - 22, 10, 14), const Radius.circular(4)), _f(_kCyan));
  c.drawCircle(Offset(fx, gy - figH + 1), 5, _f(_kCream));
}

void _sDial(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2), tp = st.at(s);
  c.drawCircle(ctr + const Offset(2, 3), 46, _f(_al(_kInk, .15)));
  c.drawCircle(ctr, 46, _f(_kWhite));
  c.drawCircle(ctr, 46, _s(_kInk, 2.5));
  for (var k = 0; k < 12; k++) {
    final d = _dir(k * math.pi / 6);
    c.drawLine(ctr + d * 37, ctr + d * 42, _s(_kInk, k % 3 == 0 ? 2.5 : 1.2));
  }
  final v = tp - ctr, dist = v.distance, d = dist < 1 ? const Offset(0, -1) : v / dist;
  final low = _sat(dist / 70), len = 6 + low * 34, sig = .6 + low * 2.2;
  final perp = Offset(-d.dy, d.dx);
  c.drawPath(_poly([ctr + perp * 4, ctr - perp * 4, ctr - d * len]), _soft(_al(_kInk, .7), sig));
  c.drawCircle(ctr, 5, _f(_kOrange));
  c.drawCircle(ctr, 5, _s(_kInk, 1.6));
  final sp = dist < 50 ? ctr + d * 50 : tp;
  c.drawCircle(sp, 7, _f(_kYellow));
  for (var k = 0; k < 8; k++) {
    final dd = _dir(k * math.pi / 4);
    c.drawLine(sp + dd * 9.5, sp + dd * 13, _s(_kOrange, 2));
  }
}

void _sCloud(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height;
  const fields = [_kLime, _kYellow, Color(0xFF6FC24A), _kOrange, Color(0xFFB8E68C), _kLime, _kYellow, Color(0xFF6FC24A), _kPink, _kLime, Color(0xFFB8E68C), _kYellow];
  for (var j = 0; j < 3; j++) {
    for (var i = 0; i < 4; i++) {
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(3 + i * w / 4, 3 + j * h / 3, w / 4 - 4, h / 3 - 4), const Radius.circular(3)), _f(fields[j * 4 + i]));
    }
  }
  final river = Path()
    ..moveTo(-5, h * .2)
    ..cubicTo(w * .3, h * .5, w * .55, 0, w + 5, h * .7);
  c.drawPath(river, _s(_kCyan, 7));
  final cp = st.at(s) + Offset(math.sin(t * .4) * 4, 0), alt = st.b;
  const puffs = [(-15.0, 3.0, 10.0), (0.0, -4.0, 13.0), (15.0, 3.0, 10.0), (0.0, 6.0, 10.0)];
  final off = const Offset(8, 12) * (.4 + alt * 2.4), sig = 1 + alt * 7;
  final shp = Path();
  for (final (x, y, r) in puffs) {
    shp.addOval(Rect.fromCircle(center: cp + off + Offset(x, y), radius: r));
  }
  c.drawPath(shp, _soft(_al(_kInk, .4), sig));
  for (final (x, y, r) in puffs) {
    c.drawCircle(cp + Offset(x, y + 2), r, _f(const Color(0xFFCFEFFB)));
  }
  for (final (x, y, r) in puffs) {
    c.drawCircle(cp + Offset(x, y), r, _f(_kWhite));
  }
}

void _sPlanet(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2);
  const r = 34.0;
  for (var k = 0; k < 22; k++) {
    c.drawCircle(Offset((_hash(k, 4) * .5 + .5) * s.width, (_hash(k, 6) * .5 + .5) * s.height), .9, _f(_al(_kCream, .7)));
  }
  final v = st.at(s) - ctr, dist = v.distance, d = dist < 1 ? const Offset(-1, 0) : v / dist;
  final ring = Rect.fromCenter(center: ctr, width: r * 3.2, height: r * .8);
  c.drawArc(ring, math.pi, math.pi, false, _s(_kViolet, 3.5));
  c
    ..save()
    ..clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  c.drawCircle(ctr, r, _f(_kCyan));
  for (final (x, y, rr) in const [(-12.0, -10.0, 13.0), (14.0, 8.0, 11.0), (-4.0, 20.0, 8.0)]) {
    c.drawOval(Rect.fromCenter(center: ctr + Offset(x, y), width: rr * 2.2, height: rr * 1.6), _f(_kLime));
  }
  final soft = .04 + (1 - _sat((dist - r) / 50)) * .5;
  c.drawCircle(
      ctr,
      r,
      Paint()
        ..shader = ui.Gradient.linear(ctr + d * r, ctr - d * r, [_al(_kNight, 0), _al(_kNight, 0), _al(_kNight, .85), _al(_kNight, .85)],
            [0, .5 - soft / 2, .5 + soft / 2, 1]));
  c.restore();
  c.drawArc(ring, 0, math.pi, false, _s(_kViolet, 3.5));
  final sp = ctr + d * math.max(dist, r + 14);
  c.drawCircle(sp, 8, _f(_kYellow));
  c.drawCircle(sp, 12, _s(_al(_kOrange, .7), 2));
}

void _sCard(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2 + 6), e = st.b, tilt = (st.a - .5) * 2;
  final card = Rect.fromCenter(center: ctr - Offset(0, e * 12), width: 76 + e * 6, height: 54 + e * 4);
  final shadow = card.translate(-tilt * e * 14, 3 + e * 16);
  c.drawRRect(RRect.fromRectAndRadius(shadow.deflate(2), const Radius.circular(10)), _soft(_al(_kInk, .28 + .1 * (1 - e)), .8 + e * 9));
  c.drawRRect(RRect.fromRectAndRadius(card, const Radius.circular(10)), _f(_kWhite));
  c.drawCircle(card.topLeft + const Offset(18, 18), 9, _f(_kPink));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(card.left + 32, card.top + 12, 34, 5), const Radius.circular(3)), _f(_kInk));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(card.left + 32, card.top + 21, 22, 5), const Radius.circular(3)), _f(_al(_kInk, .3)));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(card.left + 10, card.bottom - 16, card.width - 20, 8), const Radius.circular(4)), _f(_kOrange));
}

void _sLong(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2);
  const r = 48.0;
  final v = st.at(s) - ctr, len = v.distance * 1.4, d = v.distance < 1 ? const Offset(1, 1) : v / v.distance;
  c.drawCircle(ctr, r, _f(_kCyan));
  final heart = Path()
    ..moveTo(0, 13)
    ..cubicTo(-24, -1, -12, -20, 0, -8)
    ..cubicTo(12, -20, 24, -1, 0, 13)
    ..close();
  c
    ..save()
    ..clipPath(Path()..addOval(Rect.fromCircle(center: ctr, radius: r)));
  final sh = Path();
  for (var k = 0; k <= 28; k++) {
    sh.addPath(heart, ctr + d * (len * k / 28));
  }
  c.drawPath(sh, _f(const Color(0xFF1C9FC0)));
  c.restore();
  c.drawPath(heart.shift(ctr), _f(_kWhite));
}

void _sSkyline(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, hy = h * .5;
  final tp = st.at(s), sp = Offset(tp.dx, tp.dy.clamp(h * .08, hy - 4)), sunH = 1 - (sp.dy - h * .08) / (hy - h * .08);
  c.drawRect(Rect.fromLTWH(0, 0, w, hy), _f(_mix(_kOrange, _kBlue, sunH)));
  c.drawRect(Rect.fromLTWH(0, hy * .55, w, hy * .45), _f(_mix(_kPink, _kCyan, sunH)));
  c.drawRect(Rect.fromLTWH(0, hy * .8, w, hy * .2), _f(_mix(_kYellow, _kCyan, sunH)));
  c.drawCircle(sp, 11, _f(_kYellow));
  c.drawRect(Rect.fromLTWH(0, hy, w, h - hy), _f(_kPanel2));
  const bs = [(.04, 16.0, 30.0), (.2, 14.0, 44.0), (.36, 18.0, 22.0), (.55, 13.0, 50.0), (.7, 18.0, 34.0), (.86, 16.0, 26.0)];
  c
    ..save()
    ..clipRect(Rect.fromLTWH(0, hy, w, h - hy));
  final sh = _soft(_al(const Color(0xFF000000), .55), 1 + (1 - sunH) * 2);
  for (final (fx, bw, bh) in bs) {
    final x0 = w * fx, x1 = x0 + bw, len = bh * (.15 + (1 - sunH) * 1.6), dx = (x0 + bw / 2 - sp.dx) * .6 * (1 - sunH * .6);
    c.drawPath(_poly([Offset(x0, hy), Offset(x1, hy), Offset(x1 + dx, hy + len), Offset(x0 + dx, hy + len)]), sh);
  }
  c.restore();
  for (final (i, (fx, bw, bh)) in bs.indexed) {
    final r = Rect.fromLTWH(w * fx, hy - bh, bw, bh);
    c.drawRect(r, _f(i.isEven ? _kViolet : const Color(0xFF6A4FD8)));
    for (var y = r.top + 4; y < r.bottom - 4; y += 7) {
      c.drawRect(Rect.fromLTWH(r.left + 3, y, 3, 3), _f(_al(_kYellow, .9)));
      c.drawRect(Rect.fromLTWH(r.right - 6, y, 3, 3), _f(_al(_kYellow, .9)));
    }
  }
}

void _sPaper(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, gap = st.b, dirx = (st.a - .5) * 2;
  c.drawCircle(Offset(w * .7, h * .28), 14, _f(_kYellow));
  const cols = [_kBlue, _kViolet, _kPink, _kOrange];
  for (var k = 0; k < 4; k++) {
    final p = Path()..moveTo(-4, h + 4);
    for (var x = -4.0; x <= w + 4; x += 4) {
      p.lineTo(x, h * (.36 + k * .15) + 9 * math.sin(x * .045 + k * 1.9));
    }
    p
      ..lineTo(w + 4, h + 4)
      ..close();
    if (k > 0) c.drawPath(p.shift(Offset(-dirx * (1 + gap * 7), -(1 + gap * 7))), _soft(_al(_kInk, .45), .5 + gap * 4));
    c.drawPath(p, _f(cols[k]));
  }
}

void _sRgb(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ob = st.at(s);
  final r = Offset.zero & s;
  c.saveLayer(r, Paint());
  c.drawRect(r, _f(_kWhite));
  const lights = [(.15, Color(0xFFFF2D55)), (.5, Color(0xFF2DFF6A)), (.85, Color(0xFF2D6BFF))];
  for (final (fx, col) in lights) {
    final lp = Offset(w * fx, -20);
    final off = (ob - lp) * .32;
    final shape = Path()
      ..addOval(Rect.fromCircle(center: ob + off - const Offset(0, 9), radius: 8.5 * 1.3))
      ..addRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: ob + off + const Offset(0, 9), width: 22 * 1.3, height: 18 * 1.3), const Radius.circular(9)));
    c.drawPath(shape, _soft(col, 1.5)..blendMode = BlendMode.difference);
  }
  c.restore();
  for (final (fx, col) in lights) {
    c.drawCircle(Offset(w * fx, 4), 5, _f(col));
  }
  c.drawCircle(ob - const Offset(0, 9), 8.5, _f(_kInk));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: ob + const Offset(0, 9), width: 22, height: 18), const Radius.circular(9)), _f(_kInk));
  c.drawLine(Offset(0, h - 1), Offset(w, h - 1), _s(_kInk, 2));
}

void _sPhoto(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, lift = st.b, dirx = (st.a - .5) * 2;
  for (var k = 0; k < 40; k++) {
    c.drawCircle(Offset((_hash(k, 1) * .5 + .5) * w, (_hash(k, 2) * .5 + .5) * h), 1.3, _f(const Color(0xFF9E5A26)));
  }
  final pin = Offset(w / 2, h * .14);
  final tl = Offset(w / 2 - 30, h * .12), tr = Offset(w / 2 + 30, h * .12);
  final bl = Offset(w / 2 - 30 - lift * 4, h * .9 + lift * 2), br = Offset(w / 2 + 30 + lift * 4, h * .9 + lift * 2);
  final sh = Offset(-dirx * lift * 12, 3 + lift * 14);
  c.drawPath(_poly([tl + const Offset(1, 2), tr + const Offset(1, 2), br + sh, bl + sh]), _soft(_al(_kInk, .45), 1.2 + lift * 6));
  c.save();
  c.translate(pin.dx, pin.dy);
  c.rotate(-.05);
  c.translate(-pin.dx, -pin.dy);
  final photo = _poly([tl, tr, br, bl]);
  c.drawPath(photo, _f(_kWhite));
  final img = Rect.fromLTRB(tl.dx + 5, tl.dy + 5, tr.dx - 5, h * .7);
  c.drawRect(img, _f(_kCyan));
  c.drawCircle(Offset(img.right - 14, img.top + 14), 8, _f(_kYellow));
  c.drawPath(_poly([img.bottomLeft, Offset(img.left + 22, img.top + 26), Offset(img.left + 40, img.bottom - 8), Offset(img.right, img.top + 30), img.bottomRight]), _f(_kLime));
  c.drawLine(Offset(bl.dx + 8, h * .8), Offset(bl.dx + 32, h * .8), _s(_al(_kInk, .5), 2));
  c.restore();
  c.drawCircle(pin + const Offset(1.5, 2.5), 5, _f(_al(_kInk, .4)));
  c.drawCircle(pin, 5, _f(_kRed));
  c.drawCircle(pin - const Offset(1.5, 1.5), 1.6, _f(_kWhite));
}

void _sBall(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, fy = h * .84, hz = h * .4;
  c.drawRect(Rect.fromLTWH(0, hz, w, h - hz), _f(const Color(0xFF26262B)));
  final ln = _s(_al(_kViolet, .5), 1);
  for (var k = -6; k <= 6; k++) {
    c.drawLine(Offset(w / 2 + k * 6, hz), Offset(w / 2 + k * 40, h), ln);
  }
  for (var k = 1; k < 6; k++) {
    final y = hz + (h - hz) * (k * k) / 25;
    c.drawLine(Offset(0, y), Offset(w, y), ln);
  }
  final throwB = math.min(st.vel.dy.abs() * 8, 30.0) * math.exp(-(t - st.upAt) * .6);
  final top = 10 + st.b * 50 + throwB;
  final ph = (t * .9) % 1, ht = top * 4 * ph * (1 - ph);
  final x = w / 2 + 8, dirx = (st.a - .5) * 2;
  final k = _sat(ht / 80);
  c.drawOval(Rect.fromCenter(center: Offset(x - dirx * ht * .6, fy), width: 30 * (1 - k * .5), height: 8 * (1 - k * .4)), _soft(_al(const Color(0xFF000000), .7 * (1 - k * .7)), 1 + ht * .1));
  final squash = ph < .05 || ph > .95;
  c.drawOval(Rect.fromCenter(center: Offset(x, fy - 11 - ht + (squash ? 2 : 0)), width: squash ? 27 : 22, height: squash ? 18 : 22), _f(_kOrange));
  c.drawCircle(Offset(x - 4, fy - 15 - ht), 3, _f(_al(_kWhite, .8)));
}

void _sRetro(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2 - 8, s.height / 2 - 6);
  final pts = <Offset>[];
  for (var i = 0; i < 72; i++) {
    final a = i / 72 * 2 * math.pi;
    pts.add(Offset(math.cos(a), math.sin(a)) * (28 + 6 * math.cos(a * 6)));
  }
  final flower = _poly(pts);
  final step = (st.at(s) - ctr) * .09;
  const cols = [_kBlue, _kViolet, _kPink, _kOrange, _kYellow];
  for (var k = 5; k >= 1; k--) {
    final p = flower.shift(ctr + step * k.toDouble());
    c.drawPath(p, _f(cols[5 - k]));
    c.drawPath(p, _s(_kInk, 2));
  }
  final front = flower.shift(ctr);
  c.drawPath(front, _f(_kWhite));
  c.drawPath(front, _s(_kInk, 2.5));
  c.drawCircle(ctr, 8, _f(_kOrange));
  c.drawCircle(ctr, 8, _s(_kInk, 2));
}

void _sTree(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, gy = h * .55;
  final tp = st.at(s), sun = Offset(tp.dx, tp.dy.clamp(6.0, gy - 10));
  final low = _sat((sun.dy - 6) / (gy - 16)), sunR = 5 + low * 9;
  c.drawRect(Rect.fromLTWH(0, gy, w, h - gy), _f(_kLime));
  c.drawCircle(sun, sunR, _f(_kYellow));
  final tx = w * .5, sx = tx + (tx - sun.dx) * .45, sy = gy + 26 + low * 4;
  final shadowR = Rect.fromCenter(center: Offset(sx, sy), width: 90 + low * 30, height: 24);
  c.drawOval(shadowR, _soft(const Color(0xFF3E8E2A), 1 + sunR * .25));
  final coin = _soft(_mix(_kLime, _kYellow, .5), sunR * .3);
  for (var k = 0; k < 7; k++) {
    final p = Offset(sx + _hash(k, 1) * 36, sy + _hash(k, 2) * 7);
    c.drawOval(Rect.fromCenter(center: p, width: 3 + sunR * .7, height: (3 + sunR * .7) * .45), coin);
  }
  c.drawLine(Offset(tx, gy + 22), Offset(tx, gy - 6), _s(const Color(0xFF7A4A22), 6));
  for (final (x, y, r) in const [(-14.0, -14.0, 14.0), (14.0, -14.0, 14.0), (0.0, -26.0, 16.0), (0.0, -10.0, 14.0)]) {
    c.drawCircle(Offset(tx + x, gy + y), r, _f(const Color(0xFF2E9E4A)));
  }
  c.drawCircle(Offset(tx - 6, gy - 32), 5, _f(_al(_kLime, .6)));
}

Path _planePath() => Path()
  ..moveTo(16, 0)
  ..lineTo(10, -2.5)
  ..lineTo(2, -2.5)
  ..lineTo(-4, -16)
  ..lineTo(-9, -16)
  ..lineTo(-5, -2.5)
  ..lineTo(-12, -2.5)
  ..lineTo(-16, -7)
  ..lineTo(-19, -7)
  ..lineTo(-17, 0)
  ..lineTo(-19, 7)
  ..lineTo(-16, 7)
  ..lineTo(-12, 2.5)
  ..lineTo(-5, 2.5)
  ..lineTo(-9, 16)
  ..lineTo(-4, 16)
  ..lineTo(2, 2.5)
  ..lineTo(10, 2.5)
  ..close();

void _sPlane(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height;
  final dune = _s(_al(_kOrange, .6), 5);
  for (var k = 0; k < 6; k++) {
    final p = Path()..moveTo(-5, k * 22.0 + 6);
    for (var x = 0.0; x <= w + 5; x += 5) {
      p.lineTo(x, k * 22 + 6 + 7 * math.sin(x * .05 + k * 1.3));
    }
    c.drawPath(p, dune);
  }
  final alt = st.b, sun = (st.a - .5) * math.pi + math.pi / 4;
  final px = (t * 22 + 70) % (w + 50) - 25, pos = Offset(px, h * .45);
  final plane = _planePath();
  final shOff = _dir(sun) * (4 + alt * 30);
  c
    ..save()
    ..translate(pos.dx + shOff.dx, pos.dy + shOff.dy)
    ..scale(.9);
  c.drawPath(plane, _soft(_al(_kInk, .45), .4 + alt * 3.5));
  c.restore();
  c
    ..save()
    ..translate(pos.dx, pos.dy)
    ..scale(1 + alt * .35);
  c.drawPath(plane, _f(_kWhite));
  c.drawPath(plane, _s(_kInk, 1.2));
  c.drawCircle(const Offset(7, 0), 1.6, _f(_kBlue));
  c.restore();
}

Path _vasePath(Offset b) => Path()
  ..moveTo(b.dx - 5, b.dy)
  ..cubicTo(b.dx - 13, b.dy - 8, b.dx - 9, b.dy - 16, b.dx - 3, b.dy - 20)
  ..lineTo(b.dx - 4, b.dy - 26)
  ..lineTo(b.dx + 4, b.dy - 26)
  ..lineTo(b.dx + 3, b.dy - 20)
  ..cubicTo(b.dx + 9, b.dy - 16, b.dx + 13, b.dy - 8, b.dx + 5, b.dy)
  ..close()
  ..addOval(Rect.fromCircle(center: Offset(b.dx - 5, b.dy - 33), radius: 5))
  ..addOval(Rect.fromCircle(center: Offset(b.dx + 6, b.dy - 31), radius: 4));

void _sCandle(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, ty = h * .8, cx = w * .2;
  final fl = _vn(t * 7, 1.3), flame = Offset(cx + fl * 1.5, ty - 30 - fl.abs() * 2);
  final ox = st.at(s).dx.clamp(w * .32, w * .88), base = Offset(ox, ty);
  final m = 1 + 26 / ((ox - cx) * .55 + 4);
  final vase = _vasePath(base);
  c.drawCircle(flame, 90, Paint()..shader = ui.Gradient.radial(flame, 90, [_al(_kOrange, .45), _al(_kOrange, 0)]));
  c
    ..save()
    ..clipRect(Rect.fromLTWH(0, 0, w, ty))
    ..translate(flame.dx, flame.dy)
    ..scale(m)
    ..translate(-flame.dx, -flame.dy);
  c.drawPath(vase.shift(Offset((base.dx - flame.dx) * .25, 0)), _soft(_al(const Color(0xFF000000), .8), (1 + m * .7) / m));
  c.restore();
  c.drawRect(Rect.fromLTWH(0, ty, w, h - ty), _f(const Color(0xFF3A2418)));
  c.drawRect(Rect.fromLTWH(cx - 5, ty - 22, 10, 22), _f(_kCream));
  final fp = Path()
    ..moveTo(cx - 4, ty - 23)
    ..quadraticBezierTo(cx - 5, ty - 30, flame.dx, flame.dy - 6)
    ..quadraticBezierTo(cx + 5, ty - 30, cx + 4, ty - 23)
    ..close();
  c.drawPath(fp, _f(_kOrange));
  c.drawCircle(Offset(cx, ty - 26), 2.4, _f(_kYellow));
  c.drawPath(vase, _f(_kPink));
  c.drawPath(Path()..addOval(Rect.fromCircle(center: Offset(base.dx - 5, base.dy - 33), radius: 5)), _f(_kYellow));
  c.drawPath(Path()..addOval(Rect.fromCircle(center: Offset(base.dx + 6, base.dy - 31), radius: 4)), _f(_kViolet));
}

void _sIsland(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, sea = h * .62;
  c.drawRect(Rect.fromLTWH(0, sea, w, h - sea), _f(const Color(0xFF1E8FC4)));
  for (var k = 0; k < 4; k++) {
    final y = sea + 6 + k * 10.0;
    c.drawLine(Offset(w * (.1 + .2 * (k % 2)), y), Offset(w * (.3 + .2 * (k % 2)), y), _s(_al(_kCyan, .7), 1.5));
    c.drawLine(Offset(w * (.6 - .1 * (k % 2)), y + 4), Offset(w * (.8 - .1 * (k % 2)), y + 4), _s(_al(_kCyan, .7), 1.5));
  }
  c.drawCircle(Offset(w * .86, h * .14), 10, _f(_kYellow));
  final alt = st.b, x = st.at(s).dx.clamp(w * .25, w * .75), y = h * .5 - alt * h * .36 + math.sin(t * 1.4) * 2;
  final hgt = (sea + 12) - y;
  c.drawOval(Rect.fromCenter(center: Offset(x - hgt * .35, sea + 14), width: 54 - alt * 10, height: 12), _soft(_al(const Color(0xFF07304A), .55), 1 + alt * 6));
  c.drawPath(_poly([Offset(x - 26, y), Offset(x + 26, y), Offset(x + 6, y + 24), Offset(x - 2, y + 30)]), _f(const Color(0xFFB0572B)));
  c.drawLine(Offset(x + 20, y + 2), Offset(x + 20, y + 22 + alt * 10), _s(_al(_kCyan, .9), 2));
  c.drawOval(Rect.fromCenter(center: Offset(x, y), width: 56, height: 14), _f(_kLime));
  c.drawLine(Offset(x - 8, y - 2), Offset(x - 8, y - 14), _s(const Color(0xFF7A4A22), 2.5));
  c.drawCircle(Offset(x - 8, y - 17), 7, _f(_kPink));
  c.drawCircle(Offset(x + 8, y - 6), 5, _f(const Color(0xFF4FB548)));
}

Path _monster() => Path()
  ..moveTo(-1, 0)
  ..lineTo(-.95, -.55)
  ..lineTo(-1.35, -.95)
  ..lineTo(-.85, -.8)
  ..lineTo(-.7, -1.1)
  ..lineTo(-.95, -1.5)
  ..lineTo(-.5, -1.25)
  ..lineTo(-.3, -1.35)
  ..lineTo(-.4, -1.85)
  ..lineTo(-.05, -1.4)
  ..lineTo(.05, -1.4)
  ..lineTo(.4, -1.85)
  ..lineTo(.3, -1.35)
  ..lineTo(.5, -1.25)
  ..lineTo(.95, -1.5)
  ..lineTo(.7, -1.1)
  ..lineTo(.85, -.8)
  ..lineTo(1.35, -.95)
  ..lineTo(.95, -.55)
  ..lineTo(1, 0)
  ..close();

void _sMonster(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, fy = h * .82;
  c.drawRect(Rect.fromLTWH(0, fy, w, h - fy), _f(const Color(0xFF221B33)));
  final cr = Offset(w * .5, fy), torch = st.at(s);
  final dist = (torch - cr).distance, m = 26 + 2600 / (dist + 10);
  final dir = (cr - torch) / math.max(dist, 1);
  final sc = Offset(cr.dx + dir.dx * m * .5, fy - 2);
  c.drawPath(_poly([torch, torch + _rot(dir, .45) * 200, torch + _rot(dir, -.45) * 200]), _f(_al(_kYellow, .22)));
  c
    ..save()
    ..clipRect(Rect.fromLTWH(0, 0, w, fy))
    ..translate(sc.dx, sc.dy)
    ..scale(m * .55, m * .6);
  c.drawPath(_monster(), _soft(const Color(0xFF05030A), .01 + .02 * (dist / 60)));
  c.restore();
  final eye = Offset(sc.dx, sc.dy - m * .6 * .9);
  for (final ex in [-.3, .3]) {
    c.drawPath(_poly([eye + Offset(ex * m * .55 - m * .08, 0), eye + Offset(ex * m * .55 + m * .08, -m * .04), eye + Offset(ex * m * .55, m * .05)]), _f(_kRed));
  }
  c.drawCircle(cr - const Offset(0, 9), 10, _f(_kLime));
  for (final ex in [-3.5, 3.5]) {
    c.drawCircle(cr + Offset(ex, -11), 2.8, _f(_kWhite));
    c.drawCircle(cr + Offset(ex, -11), 1.3, _f(_kInk));
  }
  c
    ..save()
    ..translate(torch.dx, torch.dy)
    ..rotate(math.atan2(dir.dy, dir.dx));
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-14, -3.5, 14, 7), const Radius.circular(2)), _f(_kCream));
  c.drawCircle(Offset.zero, 3.5, _f(_kYellow));
  c.restore();
}
