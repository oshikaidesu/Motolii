part of 'pop_10.dart';

// Macro / Variations: why you touch it = "change the whole feel in one move", "go back to that look", "surprise me".

List<_Pn> _macroPanels() => [
      const _Pn('mood face', 'character · flat · drag x=joy up=wild · result', _face, x0: .7, y0: .5),
      const _Pn('dice throw', 'toy · pseudo 3D · flick/tap · randomize', _dice),
      const _Pn('mood jars', 'food · flat · drag along shelf · saved', _jars, light: true, x0: .25),
      const _Pn('sky mood', 'weather · flat · drag calm→storm · result', _sky, x0: .3),
      const _Pn('blob mutant', 'creature · neon · drag + tap mutate', _blob),
      const _Pn('cocktail', 'food · flat · drag pours · blend', _cocktail, x0: .4, y0: .4),
      const _Pn('slot reels', 'machine · bold flat · pull · randomize', _slot),
      const _Pn('mood galaxy', 'cosmic · top-down · fly between · saved', _galaxy, x0: .35, y0: .55),
      const _Pn('seed garden', 'nature · side view · scroll seeds', _garden, light: true, x0: .3),
      const _Pn('palette smear', 'material · flat · rub · mix', _palette),
      const _Pn('polaroid stack', 'material · pseudo 3D · flick · saved', _polaroid),
      const _Pn('kaleidoscope', 'toy · neon wild · spin · variation', _kaleido),
      const _Pn('puppeteer pose', 'character · flat · tilt · 1 → many', _puppeteer),
      const _Pn('tree tone', 'nature · flat · drag + tap seed', _tree, x0: .45, y0: .3),
      const _Pn('bouncy shuffle', 'physics · flat · throw · randomize', _bouncy, step: _bouncyStep),
      const _Pn('tangram morph', 'toy · bold flat · drag · saved shapes', _tangram, x0: .0),
      const _Pn('chameleon', 'creature · flat · walk to leaf · mood', _chameleon, x0: .2),
      const _Pn('fireworks seed', 'cosmic · neon · tap/throw · seed', _fireworks, y0: .3),
      const _Pn('mosaic shuffle', 'pixel · flat · rub scramble · tap', _mosaic),
      const _Pn('donut sprinkles', 'food · top-down · rub · re-roll', _donut, light: true, x0: .1),
    ];

void _face(Canvas c, Size s, _S st) {
  final joy = st.x, wild = 1 - st.y;
  _bg(c, s, _mix(_bl, _or, joy));
  for (var i = 0; i < (wild * 22).round(); i++) {
    final a = i * 2.4 + st.t * (.4 + wild), r = 40 + 14 * _rn(i) + math.sin(st.t * 3 + i) * 4 * wild;
    final q = const Offset(78, 58) + Offset(math.cos(a) * r * 1.3, math.sin(a) * r * .9);
    c.drawPath(_star(q, 2.5 + 3 * _rn(i + 3), a, k: 4), _f(_pop[i % 8]));
  }
  final j = math.sin(st.t * 30) * wild * 1.5;
  final cc = Offset(78 + j, 58);
  if (wild > .45) c.drawPath(_star(cc, 30 + wild * 14, st.t * wild, k: 12, inner: .75), _f(_pk));
  c.drawCircle(cc, 30, _f(_mix(_cy, _ye, joy)));
  final eye = 3 + wild * 3;
  for (final d in [-1.0, 1.0]) {
    final e = cc + Offset(d * 11, -6);
    c.drawCircle(e, eye, _f(_k0));
    c.drawCircle(e + Offset(-eye * .3, -eye * .3), eye * .3, _f(_wh));
    c.drawLine(e + Offset(-d * 6, -eye - 5 + (joy - .5) * -4 * d * 0), e + Offset(d * 5, -eye - 5 - (joy - .5) * 6 + wild * 2), _s(_k0, 2.4));
  }
  if (joy > .5) {
    c.drawCircle(cc + const Offset(-19, 6), 4, _f(_al(_pk, (joy - .5) * 1.6)));
    c.drawCircle(cc + const Offset(19, 6), 4, _f(_al(_pk, (joy - .5) * 1.6)));
  }
  final m = Path()
    ..moveTo(cc.dx - 12, cc.dy + 10)
    ..quadraticBezierTo(cc.dx, cc.dy + 10 + (joy - .5) * 30, cc.dx + 12, cc.dy + 10);
  if (wild > .7) {
    m.close();
    c.drawPath(m, _f(_k0));
  } else {
    c.drawPath(m, _s(_k0, 3));
  }
}

void _dice(Canvas c, Size s, _S st) {
  final seed = st.flings + st.taps, since = st.t - st.rel, roll = since < .8 ? 1 - since / .8 : 0.0;
  final pop = _ss(since / .35);
  c.save();
  final r = const Rect.fromLTWH(8, 10, 84, 96);
  c.translate(r.center.dx, r.center.dy);
  c.scale(.86 + .14 * pop);
  c.translate(-r.center.dx, -r.center.dy);
  _comp(c, r, _rn(seed * 3 + 1), .3 + .6 * _rn(seed * 5 + 2), seed);
  c.restore();
  final dc = Offset(122, 66 - (math.sin(since * 11).abs() * 24 * roll)), rot = roll * roll * 9 + st.a * .3;
  c.drawOval(Rect.fromCenter(center: const Offset(122, 92), width: 34 - roll * 10, height: 7), _f(_al(_k0, .5)));
  c.save();
  c.translate(dc.dx, dc.dy);
  c.rotate(rot);
  const h = 14.0;
  final top = [const Offset(0, -h), const Offset(h, -h * .5), const Offset(0, 0), const Offset(-h, -h * .5)];
  c.drawPath(_poly([const Offset(-h, -h * .5), const Offset(0, 0), const Offset(0, h), const Offset(-h, h * .5)]), _f(_mix(_wh, _cy, .35)));
  c.drawPath(_poly([const Offset(h, -h * .5), const Offset(0, 0), const Offset(0, h), const Offset(h, h * .5)]), _f(_mix(_wh, _vi, .3)));
  c.drawPath(_poly(top), _f(_wh));
  final face = roll > 0 ? (st.t * 20).floor() % 6 + 1 : seed % 6 + 1;
  const pips = {
    1: [Offset(.5, .5)],
    2: [Offset(.25, .25), Offset(.75, .75)],
    3: [Offset(.25, .25), Offset(.5, .5), Offset(.75, .75)],
    4: [Offset(.25, .25), Offset(.75, .25), Offset(.25, .75), Offset(.75, .75)],
    5: [Offset(.25, .25), Offset(.75, .25), Offset(.5, .5), Offset(.25, .75), Offset(.75, .75)],
    6: [Offset(.25, .2), Offset(.75, .2), Offset(.25, .5), Offset(.75, .5), Offset(.25, .8), Offset(.75, .8)],
  };
  for (final p in pips[face]!) {
    final q = top[3] + (top[0] - top[3]) * p.dx + (top[2] - top[3]) * p.dy;
    c.drawOval(Rect.fromCenter(center: q, width: 3.6, height: 2.2), _f(face == 1 ? _rd : _k0));
  }
  c.restore();
}

const _moods = [(.02, .3), (.13, .85), (.5, .5), (.74, .2), (.9, .95)];

void _jars(Canvas c, Size s, _S st) {
  final pos = st.x * 4, k = pos.floor().clamp(0, 3), f = _ss(pos - k);
  final a = _moods[k], b = _moods[math.min(k + 1, 4)];
  _comp(c, const Rect.fromLTWH(8, 6, 140, 56), a.$1 + (b.$1 - a.$1) * f, a.$2 + (b.$2 - a.$2) * f, 4);
  c.drawRect(const Rect.fromLTWH(0, 106, _cw, 10), _f(_mix(_or, _k0, .45)));
  final sel = pos.round();
  for (var i = 0; i < 5; i++) {
    final lift = i == sel ? 7.0 + math.sin(st.t * 4).abs() * 2 : 0.0, x = 12 + i * 28.0;
    final body = Rect.fromLTWH(x, 74 - lift, 24, 32);
    final col = _h(_moods[i].$1, .9, .58), col2 = _h(_moods[i].$1 + .1 + _moods[i].$2 * .3, .9, .6);
    c.drawRRect(RRect.fromRectAndRadius(body, const Radius.circular(5)), _f(_al(_wh, .7)));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTRB(x + 2, body.top + 8, x + 22, body.bottom - 2), const Radius.circular(4)), _f(col));
    c.drawCircle(Offset(x + 12, body.top + 19), 4, _f(col2));
    c.drawRect(Rect.fromLTWH(x + 4, body.top + 10, 3, 14), _f(_al(_wh, .45)));
    c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(x - 1, body.top - 4, 26, 7), const Radius.circular(2)), _f(i == sel ? _k0 : _k1));
  }
}

void _cloud(Canvas c, Offset o, double k, Color col) {
  for (final (dx, dy, r) in const [(-10.0, 2.0, 7.0), (0.0, -3.0, 9.0), (10.0, 2.0, 7.0), (0.0, 4.0, 7.0)]) {
    c.drawCircle(o + Offset(dx, dy) * k, r * k, _f(col));
  }
}

void _sky(Canvas c, Size s, _S st) {
  final st0 = st.x, wind = (st.y - .5) * 2;
  final flash = st0 > .8 && _wr(st.t * .7) < .06;
  _vgrad(c, Offset.zero & s, flash ? _wh : _mix(_cy, _mix(_vi, _k0, .55), st0), flash ? _ye : _mix(_ye, _mix(_bl, _k0, .4), st0));
  c.drawCircle(const Offset(40, 34), 14 + (1 - st0) * 3, _f(_al(_ye, 1 - st0 * 1.4)));
  for (var i = 0; i < 8; i++) {
    final a = i * math.pi / 4 + st.t * .4;
    c.drawLine(const Offset(40, 34) + _pol(a, 19), const Offset(40, 34) + _pol(a, 25), _s(_al(_or, 1 - st0 * 2), 2.5));
  }
  final n = 1 + (st0 * 4).round();
  for (var i = 0; i < n; i++) {
    final x = _wr(_rn(i) + st.t * (.03 + wind.abs() * .05) * (wind >= 0 ? 1 : -1)) * 190 - 17;
    _cloud(c, Offset(x, 20 + i * 9.0), .8 + st0 * .6, _mix(_wh, _mix(_k1, _vi, .3), st0));
  }
  if (st0 > .4) {
    for (var i = 0; i < 26; i++) {
      final x = _rn(i + 9) * 180 - 12, y = _wr(_rn(i + 4) + st.t * 1.6) * 110;
      c.drawLine(Offset(x + y * wind * .4, y), Offset(x + y * wind * .4 + wind * 4, y + 8), _s(_al(_cy, (st0 - .4) * 2), 1.4));
    }
  }
  if (flash || (st0 > .85 && _wr(st.t * .7) < .1)) {
    c.drawPath(_poly(const [Offset(96, 22), Offset(86, 50), Offset(96, 50), Offset(84, 82), Offset(108, 44), Offset(98, 44), Offset(106, 22)]), _f(_ye));
  }
  c.drawPath(
      Path()
        ..moveTo(0, 96)
        ..quadraticBezierTo(60, 80, 156, 98)
        ..lineTo(156, 116)
        ..lineTo(0, 116)
        ..close(),
      _f(_mix(_li, _mix(_k0, _bl, .3), st0 * .7)));
}

void _blob(Canvas c, Size s, _S st) {
  final seed = st.taps + st.flings, spike = .04 + st.x * .32, rr = 18 + (1 - st.y) * 16, k = 3 + seed % 6;
  final col = _pop[seed % 8], cc = const Offset(78, 60);
  final p = Path();
  for (var i = 0; i <= 72; i++) {
    final a = i / 72 * 2 * math.pi, r = rr * (1 + spike * math.sin(k * a + st.t * 2) + .05 * math.sin(a * 2 + st.t * 3));
    final q = cc + _pol(a, r);
    i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
  }
  p.close();
  c.drawPath(p, _s(_al(col, .25), 10));
  c.drawPath(p, _f(col));
  c.drawPath(p, _s(_wh, 1.5));
  final eyes = 1 + (seed * 7) % 3, look = st.p == null ? Offset.zero : (st.p! - cc) / ((st.p! - cc).distance + 1) * 2.5;
  for (var i = 0; i < eyes; i++) {
    final e = cc + Offset((i - (eyes - 1) / 2) * rr * .5, -rr * .2), er = rr * (eyes == 1 ? .32 : .2);
    c.drawCircle(e, er, _f(_wh));
    c.drawCircle(e + look, er * .5, _f(_k0));
  }
  c.drawArc(Rect.fromCenter(center: cc + Offset(0, rr * .3), width: rr * .7, height: rr * .4), 0, math.pi, false, _s(_k0, 2.4));
}

void _cocktail(Canvas c, Size s, _S st) {
  final ws = [.15 + .7 * st.x, .15 + .7 * (1 - st.y), .35];
  final cols = [_or, _pk, _cy];
  final tot = ws.reduce((a, b) => a + b);
  double r = 0, g = 0, b = 0;
  for (var i = 0; i < 3; i++) {
    r += cols[i].r * ws[i] / tot;
    g += cols[i].g * ws[i] / tot;
    b += cols[i].b * ws[i] / tot;
  }
  final blend = Color.from(alpha: 1, red: r, green: g, blue: b);
  _bg(c, s, _mix(blend, _k0, .55));
  c.drawCircle(const Offset(78, 62), 50, _f(_al(blend, .45)));
  const top = 18.0, bot = 106.0;
  Path glass() => _poly(const [Offset(54, top), Offset(102, top), Offset(98, bot), Offset(58, bot)]);
  c.save();
  c.clipPath(glass());
  var y = bot;
  for (var i = 2; i >= 0; i--) {
    final h = (bot - top - 12) * ws[i] / tot;
    c.drawRect(Rect.fromLTRB(50, y - h, 106, y), _f(cols[i]));
    y -= h;
  }
  for (var i = 0; i < 6; i++) {
    final u = _wr(st.t * .4 + _rn(i));
    c.drawCircle(Offset(62 + _rn(i + 3) * 32, bot - u * (bot - y)), 1.4, _f(_al(_wh, .7)));
  }
  for (var i = 0; i < 2; i++) {
    c.save();
    c.translate(70 + i * 16.0, y + 8 + i * 6);
    c.rotate(.3 + i * .5 + math.sin(st.t + i) * .1);
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(-7, -7, 14, 14), const Radius.circular(3)), _f(_al(_wh, .5)));
    c.restore();
  }
  c.restore();
  c.drawLine(const Offset(90, 8), const Offset(80, 100), _s(_wh, 4));
  _dash(c, const [Offset(90, 8), Offset(80, 100)], _s(_rd, 4), 5, 5);
  c.drawPath(glass(), _s(_al(_wh, .8), 2));
  c.drawArc(Rect.fromCircle(center: const Offset(54, 20), radius: 11), math.pi * .85, math.pi * 1.1, true, _f(_li));
  c.drawArc(Rect.fromCircle(center: const Offset(54, 20), radius: 8), math.pi * .85, math.pi * 1.1, true, _f(_mix(_li, _wh, .5)));
}

void _sym(Canvas c, Offset o, int kind, Color col, double r) {
  switch (kind % 5) {
    case 0:
      c.drawCircle(o, r, _f(col));
    case 1:
      c.drawPath(_star(o, r * 1.2, 0), _f(col));
    case 2:
      c.drawPath(_ngon(o, r * 1.2, 3, 0), _f(col));
    case 3:
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromCircle(center: o, radius: r * .85), Radius.circular(r * .3)), _f(col));
    default:
      c.drawPath(_star(o, r * 1.15, 0, k: 8, inner: .7), _f(col));
  }
}

void _slot(Canvas c, Size s, _S st) {
  final seed = st.ends + st.taps, since = st.t - st.rel;
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(6, 12, 122, 96), const Radius.circular(12)), _f(_rd));
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(6, 12, 122, 14), const Radius.circular(7)), _f(_mix(_rd, _k0, .25)));
  for (var i = 0; i < 7; i++) {
    c.drawCircle(Offset(18 + i * 16.0, 19), 2.6, _f(_al(_ye, _wr(st.t * 3 + i / 7) < .5 ? 1 : .3)));
  }
  final win = <int>[];
  for (var i = 0; i < 3; i++) {
    final r = Rect.fromLTWH(14 + i * 37.0, 34, 32, 56);
    c.drawRRect(RRect.fromRectAndRadius(r, const Radius.circular(6)), _f(_cr));
    c.save();
    c.clipRRect(RRect.fromRectAndRadius(r, const Radius.circular(6)));
    final stop = .45 + i * .3, spinning = since < stop;
    if (spinning) {
      final off = _wr(st.t * 9) * 28;
      for (var k = -1; k < 2; k++) {
        final id = (st.t * 9).floor() + k + i * 3;
        _sym(c, r.center + Offset(0, k * 28.0 + off), id, _pop[id % 8], 9);
      }
    } else {
      final id = (_rn(seed * 3 + i) * 40).floor();
      win.add(id % 5);
      _sym(c, r.center, id, _pop[(id * 3) % 8], 11);
      _sym(c, r.center + const Offset(0, -28), id + 1, _al(_k0, .15), 9);
      _sym(c, r.center + const Offset(0, 28), id + 2, _al(_k0, .15), 9);
    }
    c.restore();
  }
  if (win.length == 3 && win[0] == win[1] && win[1] == win[2]) {
    c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(8, 30, 118, 64), const Radius.circular(8)), _s(_ye, 3));
  }
  final pull = st.down && st.p != null ? _cl((st.p!.dy - 20) / 70) : 0.0;
  final knob = Offset(142, 26 + pull * 58);
  c.drawRect(const Rect.fromLTWH(128, 56, 8, 16), _f(_mix(_rd, _k0, .3)));
  c.drawLine(const Offset(140, 64), knob, _s(_cr, 3.5));
  c.drawCircle(knob, 8, _f(_ye));
  c.drawCircle(knob + const Offset(-2.5, -2.5), 2.4, _f(_al(_wh, .7)));
}

const _gal = [Offset(28, 26), Offset(118, 20), Offset(132, 80), Offset(76, 96), Offset(22, 84), Offset(80, 50)];
const _galHue = [.0, .14, .5, .76, .9, .33];

void _galaxy(Canvas c, Size s, _S st) {
  for (var i = 0; i < 40; i++) {
    c.drawCircle(Offset(_rn(i) * _cw, _rn(i + 77) * _ch), .5 + _rn(i + 5) * .7, _f(_al(_cr, .3 + .3 * _rn(i + 2))));
  }
  final ship = Offset(10 + st.x * 136, 10 + st.y * 96);
  final ws = [for (final g in _gal) 1 / ((g - ship).distanceSquared + 90)];
  final tot = ws.reduce((a, b) => a + b);
  double r = 0, g = 0, b = 0;
  for (var i = 0; i < _gal.length; i++) {
    final col = _h(_galHue[i]), w = ws[i] / tot;
    r += col.r * w;
    g += col.g * w;
    b += col.b * w;
    c.drawLine(ship, _gal[i], _s(_al(col, w * 1.6), 1 + w * 3));
    final tw = 1 + .15 * math.sin(st.t * 3 + i);
    c.drawCircle(_gal[i], (9 + w * 14) * tw, _f(_al(col, .2 + w * .5)));
    c.drawPath(_star(_gal[i], (5 + w * 7) * tw, st.t * .3 + i, k: 4, inner: .35), _f(col));
  }
  final blend = Color.from(alpha: 1, red: r, green: g, blue: b);
  c.drawCircle(ship, 15, _f(_al(blend, .35)));
  c.drawCircle(ship, 15, _s(blend, 3));
  final dir = st.trail.length > 1 ? (st.trail.last - st.trail[st.trail.length - 2]).direction : -math.pi / 2;
  c.save();
  c.translate(ship.dx, ship.dy);
  c.rotate(dir + math.pi / 2);
  c.drawPath(_poly(const [Offset(0, -7), Offset(5, 5), Offset(-5, 5)]), _f(_wh));
  c.drawPath(_poly([const Offset(-3, 5), const Offset(3, 5), Offset(0, 9 + math.sin(st.t * 30) * 2)]), _f(_or));
  c.restore();
}

void _flower(Canvas c, Offset base, int seed, double k, double t) {
  final h = (34 + _rn(seed) * 34) * k, n = 4 + (_rn(seed + 1) * 6).floor(), col = _pop[(_rn(seed + 2) * 8).floor()];
  final head = base + Offset(math.sin(t * 1.5 + seed) * 3, -h);
  c.drawPath(
      Path()
        ..moveTo(base.dx, base.dy)
        ..quadraticBezierTo(base.dx - 6, base.dy - h * .5, head.dx, head.dy),
      _s(_mix(_li, _k0, .2), 2.5 * k));
  c.drawOval(Rect.fromCenter(center: base + Offset(-6 * k, -h * .35), width: 10 * k, height: 5 * k), _f(_li));
  final pr = (7 + _rn(seed + 3) * 5) * k;
  for (var i = 0; i < n; i++) {
    final a = i * 2 * math.pi / n + seed;
    c.drawCircle(head + _pol(a, pr * .8), pr * (.45 + _rn(seed + 4) * .25), _f(col));
  }
  c.drawCircle(head, pr * .5, _f(_pop[(_rn(seed + 5) * 8).floor()]));
}

void _garden(Canvas c, Size s, _S st) {
  final off = st.x * 24, cen = off.round();
  final cx = 78 + (cen - off) * 36;
  c.drawPath(_poly([Offset(cx - 6, 0), Offset(cx + 6, 0), Offset(cx + 20, 100), Offset(cx - 20, 100)]), _f(_al(_ye, .45)));
  c.drawRect(const Rect.fromLTWH(0, 100, _cw, 16), _f(_mix(_or, _k0, .45)));
  for (var k = cen - 3; k <= cen + 3; k++) {
    final x = 78 + (k - off) * 36;
    if (x < -20 || x > 176) continue;
    _flower(c, Offset(x, 101), k + 100, k == cen ? 1.15 : .8, st.t);
  }
}

const _blobs = [(Offset(36, 34), _rd), (Offset(70, 26), _ye), (Offset(108, 32), _cy), (Offset(118, 70), _vi)];

Color _blobAt(Offset p) {
  var best = _blobs.first;
  for (final b in _blobs) {
    if ((b.$1 - p).distance < (best.$1 - p).distance) best = b;
  }
  return best.$2;
}

void _palette(Canvas c, Size s, _S st) {
  _bg(c, s, _k1);
  final pal = Path()
    ..addOval(const Rect.fromLTWH(8, 10, 140, 96))
    ..addOval(const Rect.fromLTWH(26, 72, 18, 14))
    ..fillType = PathFillType.evenOdd;
  c.drawPath(pal.shift(const Offset(2, 3)), _f(_al(_k0, .35)));
  c.drawPath(pal, _f(_cr));
  for (final b in _blobs) {
    c.drawCircle(b.$1, 10, _f(b.$2));
    c.drawCircle(b.$1 + const Offset(-3, -3), 2.5, _f(_al(_wh, .5)));
  }
  final tr = st.trail.length > 2
      ? st.trail
      : [for (var i = 0; i <= 12; i++) Offset(70 + i * 3.4, 26 + i * 4.0 + math.sin(i * .8) * 4)];
  final c0 = _blobAt(tr.first);
  final l = _plen(tr);
  var acc = 0.0;
  var col = c0;
  for (var i = 1; i < tr.length; i++) {
    acc += (tr[i] - tr[i - 1]).distance;
    col = _mix(c0, _blobAt(tr[i]), acc / (l + 1) * 1.2);
    c.drawLine(tr[i - 1], tr[i], _s(col, 9));
  }
  c.drawCircle(const Offset(80, 74), 13, _f(col));
  c.drawCircle(const Offset(76, 70), 3, _f(_al(_wh, .45)));
}

void _polaroid(Canvas c, Size s, _S st) {
  _bg(c, s, _mix(_vi, _k0, .55));
  final i0 = st.flings, since = st.t - st.rel;
  void card(int seed, Offset o, double rot, double alpha) {
    c.save();
    c.translate(o.dx, o.dy);
    c.rotate(rot);
    c.drawRect(const Rect.fromLTWH(-29, -33, 62, 72), _f(_al(_k0, .3 * alpha)));
    c.drawRect(const Rect.fromLTWH(-31, -36, 62, 72), _f(_al(_wh, alpha)));
    if (alpha > .99) _comp(c, const Rect.fromLTWH(-26, -31, 52, 50), _rn(seed * 7), .3 + .6 * _rn(seed * 9 + 1), seed, round: 1);
    c.restore();
  }

  for (var j = 3; j >= 1; j--) {
    card(i0 + j, const Offset(78, 58) + Offset(j * 4.0 - 6, j * -2.0), (_rn(i0 + j) - .5) * .5, 1);
  }
  var o = const Offset(78, 58);
  if (st.down && st.trail.isNotEmpty) o += st.p! - st.trail.first;
  card(i0, o, (o.dx - 78) * .004 - .04, 1);
  if (since < .4 && i0 > 0) {
    final v = st.vel / (st.vel.distance + 1);
    card(i0 - 1, const Offset(78, 58) + v * since * 500, since * 4, 1);
  }
}

void _kaleido(Canvas c, Size s, _S st) {
  const cc = Offset(78, 58);
  final seed = st.a * .6 + st.t * .08;
  c.save();
  c.clipPath(Path()..addOval(Rect.fromCircle(center: cc, radius: 52)));
  c.drawCircle(cc, 52, _f(_mix(_vi, _k0, .7)));
  for (var m = 0; m < 6; m++) {
    for (final mir in [1.0, -1.0]) {
      c.save();
      c.translate(cc.dx, cc.dy);
      c.rotate(m * math.pi / 3 + seed * .3);
      c.scale(1, mir);
      for (var k = 0; k < 7; k++) {
        final r = 6 + 46 * _wr(k * .37 + seed * .11 * (k + 1)), a = (math.pi / 6) * _wr(k * .61 + seed * .07 * (k + 2));
        final q = _pol(a, r), sz = 3 + 7 * _rn(k);
        switch (k % 3) {
          case 0:
            c.drawCircle(q, sz, _f(_pop[k % 8]));
          case 1:
            c.drawPath(_ngon(q, sz * 1.3, 3, a + seed), _f(_pop[(k + 3) % 8]));
          default:
            c.drawPath(_star(q, sz * 1.2, seed, k: 4), _f(_pop[(k + 5) % 8]));
        }
      }
      c.restore();
    }
  }
  c.restore();
  c.drawCircle(cc, 52, _s(_wh, 3));
  c.drawCircle(cc, 56, _s(_pk, 3));
}

void _puppeteer(Canvas c, Size s, _S st) {
  _bg(c, s, _mix(_cy, _k0, .6));
  final roll = (st.x - .5) * 1.1 + math.sin(st.t * 2) * .05, pitch = st.y - .5;
  const ctl = Offset(78, 14);
  final l = ctl + _pol(math.pi + roll, 28), r = ctl + _pol(roll, 28), f = ctl + Offset(0, 4 + pitch * 10);
  final head = Offset(78 + roll * 14, 44), sh = Offset(78 + roll * 8, 54), hip = const Offset(78, 78);
  final hl = Offset(52, 62 + (l.dy - 14) * 2.6), hr = Offset(104, 62 + (r.dy - 14) * 2.6);
  final kl = Offset(68, 90 - pitch * 22), kr = Offset(88, 90 + pitch * 22);
  for (final (a, b) in [(l, hl), (r, hr), (f, kl), (f, kr), (ctl, head)]) {
    c.drawLine(a, b, _s(_al(_cr, .5), .8));
  }
  c.drawOval(Rect.fromCenter(center: const Offset(78, 108), width: 50, height: 6), _f(_al(_k0, .4)));
  final body = _s(_pk, 6);
  c.drawLine(hip, kl, body);
  c.drawLine(kl, Offset(kl.dx - 4, 106), body);
  c.drawLine(hip, kr, body);
  c.drawLine(kr, Offset(kr.dx + 4, 106), body);
  c.drawLine(sh, hl, _s(_ye, 5));
  c.drawLine(sh, hr, _s(_ye, 5));
  c.drawLine(sh, hip, _s(_or, 12));
  c.drawCircle(head, 9, _f(_cr));
  c.drawCircle(head + const Offset(-3, -1), 1.4, _f(_k0));
  c.drawCircle(head + const Offset(3, -1), 1.4, _f(_k0));
  c.drawLine(l, r, _s(_wh, 4));
  c.drawLine(ctl + Offset(0, -6), f, _s(_wh, 4));
}

void _branch(Canvas c, Offset p, double ang, double len, int d, int seed, double spread, double t, int id) {
  final sway = math.sin(t * 1.3 + d) * .03 * (8 - d);
  final q = p + _pol(ang + sway, len);
  c.drawLine(p, q, _s(_mix(_or, _k0, .3), math.max(1, d * 1.1)));
  if (d == 0) {
    c.drawCircle(q, 3.2, _f(_pop[(seed + id) % 5 == 0 ? 4 : (seed % 3 == 0 ? 3 : (seed % 3 == 1 ? 1 : 2))]));
    return;
  }
  for (final sgn in [-1.0, 1.0]) {
    final j = (_rn(seed * 97 + id * 2 + (sgn > 0 ? 1 : 0)) - .5) * .6;
    _branch(c, q, ang + sgn * spread * .5 + j, len * (.68 + .12 * _rn(seed + id)), d - 1, seed, spread, t, id * 2 + (sgn > 0 ? 1 : 0));
  }
}

void _tree(Canvas c, Size s, _S st) {
  _vgrad(c, Offset.zero & s, _mix(_bl, _k0, .5), _k1);
  c.drawOval(Rect.fromCenter(center: const Offset(78, 110), width: 120, height: 14), _f(_mix(_li, _k0, .3)));
  final depth = 3 + ((1 - st.y) * 5).round(), seed = st.taps + st.flings;
  _branch(c, const Offset(78, 108), -math.pi / 2, 16 + depth * 2.0, depth, seed, .25 + st.x * 1.4, st.t, 1);
}

void _bouncyStep(_S st, double dt) {
  final b = st.b;
  if (b.isEmpty) {
    for (var i = 0; i < 9; i++) {
      b.addAll([20 + _rn(i) * 116, 20 + _rn(i + 9) * 60, 0, 0]);
    }
    b.add(-9);
  }
  if (b[36] != st.rel) {
    b[36] = st.rel;
    for (var i = 0; i < 9; i++) {
      b[i * 4 + 2] += st.vel.dx * .6 + (_rn(i + st.ends * 11) - .5) * 300;
      b[i * 4 + 3] += st.vel.dy * .6 - 150 - _rn(i + st.ends * 5) * 200;
    }
  }
  for (var i = 0; i < 9; i++) {
    final k = i * 4, r = 6 + (i % 3) * 2.5;
    if (st.down && st.p != null) {
      final d = Offset(b[k], b[k + 1]) - st.p!;
      if (d.distance < 24 && d.distance > 0) {
        b[k + 2] += d.dx / d.distance * 900 * dt;
        b[k + 3] += d.dy / d.distance * 900 * dt;
      }
    }
    b[k + 3] += 420 * dt;
    b[k] += b[k + 2] * dt;
    b[k + 1] += b[k + 3] * dt;
    if (b[k] < 6 + r || b[k] > 150 - r) {
      b[k] = _cl(b[k], 6 + r, 150 - r);
      b[k + 2] *= -.75;
    }
    if (b[k + 1] > 110 - r) {
      b[k + 1] = 110 - r;
      b[k + 3] *= -.6;
      b[k + 2] *= .96;
    }
    if (b[k + 1] < 6 + r) {
      b[k + 1] = 6 + r;
      b[k + 3] *= -.6;
    }
  }
}

void _bouncy(Canvas c, Size s, _S st) {
  c.drawRRect(RRect.fromRectAndRadius(const Rect.fromLTWH(4, 4, 148, 108), const Radius.circular(6)), _s(_al(_cr, .4), 2));
  if (st.b.length < 37) return;
  for (var i = 0; i < 9; i++) {
    final k = i * 4, r = 6 + (i % 3) * 2.5, p = Offset(st.b[k], st.b[k + 1]);
    final squash = _cl(1 - (st.b[k + 3].abs() < 60 && p.dy > 104 - r ? .15 : 0), .8, 1);
    c.drawOval(Rect.fromCenter(center: p, width: r * 2 / squash, height: r * 2 * squash), _f(_pop[(i + st.ends) % 8]));
    c.drawCircle(p + Offset(-r * .35, -r * .35), r * .25, _f(_al(_wh, .6)));
  }
}

// Figures for 4 pieces (circle, triangle, triangle, square): (cx, cy, radius, rot).
const _figs = [
  [(78.0, 46.0, 15.0, 0.0), (66.0, 31.0, 9.0, -.3), (90.0, 31.0, 9.0, .3), (78.0, 82.0, 22.0, 0.0)],
  [(78.0, 84.0, 6.0, 0.0), (78.0, 46.0, 30.0, 0.0), (102.0, 34.0, 8.0, 0.0), (78.0, 80.0, 28.0, 0.0)],
  [(70.0, 60.0, 24.0, 0.0), (108.0, 60.0, 17.0, -math.pi / 2), (66.0, 34.0, 10.0, 0.0), (60.0, 54.0, 4.0, .6)],
];

void _tangram(Canvas c, Size s, _S st) {
  _bg(c, s, _bl);
  final u = st.x * 2, k = u.floor().clamp(0, 1), f = _ss(u - k);
  final a = _figs[k], b = _figs[k + 1];
  final cols = [_ye, _pk, _cy, _or];
  for (final i in [3, 0, 1, 2]) {
    final x = a[i].$1 + (b[i].$1 - a[i].$1) * f, y = a[i].$2 + (b[i].$2 - a[i].$2) * f;
    final r = a[i].$3 + (b[i].$3 - a[i].$3) * f, rot = a[i].$4 + (b[i].$4 - a[i].$4) * f + math.sin(f * math.pi) * 1.5;
    final o = Offset(x, y);
    final path = switch (i) {
      0 => Path()..addOval(Rect.fromCircle(center: o, radius: r)),
      3 => _ngon(o, r * 1.2, 4, rot + math.pi / 4),
      _ => _ngon(o, r, 3, rot),
    };
    c.drawPath(path.shift(const Offset(2, 3)), _f(_al(_k0, .3)));
    c.drawPath(path, _f(cols[i]));
  }
  final ey = Offset(a[0].$1 + (b[0].$1 - a[0].$1) * f, a[0].$2 + (b[0].$2 - a[0].$2) * f);
  if (u < .5) {
    c.drawCircle(ey + const Offset(-5, -2), 1.8, _f(_k0));
    c.drawCircle(ey + const Offset(5, -2), 1.8, _f(_k0));
  } else if (u > 1.5) {
    c.drawCircle(ey + const Offset(-10, -5), 2.2, _f(_k0));
  }
}

const _leaves = [(24.0, _pk), (60.0, _ye), (96.0, _cy), (132.0, _vi)];

void _chameleon(Canvas c, Size s, _S st) {
  _bg(c, s, _mix(_li, _k0, .7));
  c.drawPath(
      Path()
        ..moveTo(0, 80)
        ..quadraticBezierTo(78, 70, 156, 84),
      _s(_mix(_or, _k0, .35), 6));
  for (final (x, col) in _leaves) {
    final o = Offset(x, 92), sw = math.sin(st.t * 1.4 + x) * .15;
    c.save();
    c.translate(x, 78);
    c.rotate(sw);
    c.translate(-x, -78);
    c.drawPath(
        Path()
          ..moveTo(o.dx, o.dy - 14)
          ..quadraticBezierTo(o.dx + 13, o.dy, o.dx, o.dy + 18)
          ..quadraticBezierTo(o.dx - 13, o.dy, o.dx, o.dy - 14)
          ..close(),
        _f(col));
    c.drawLine(Offset(o.dx, o.dy - 12), Offset(o.dx, o.dy + 15), _s(_al(_k0, .25), 1));
    c.restore();
  }
  final cx = 16 + st.x * 124;
  double r = 0, g = 0, b = 0, tot = 0;
  for (final (x, col) in _leaves) {
    final w = 1 / ((x - cx) * (x - cx) + 30);
    tot += w;
    r += col.r * w;
    g += col.g * w;
    b += col.b * w;
  }
  final target = Color.from(alpha: 1, red: r / tot, green: g / tot, blue: b / tot);
  if (st.b.isEmpty) st.b.addAll([target.r, target.g, target.b]);
  st.b[0] += (target.r - st.b[0]) * .08;
  st.b[1] += (target.g - st.b[1]) * .08;
  st.b[2] += (target.b - st.b[2]) * .08;
  final col = Color.from(alpha: 1, red: st.b[0], green: st.b[1], blue: st.b[2]), dark = _mix(col, _k0, .35);
  final y = 62.0 + math.sin(st.t * 3) * .8;
  final tail = Path();
  for (var i = 0; i <= 30; i++) {
    final a = i * .3, rr = 10 * (1 - i / 34);
    final q = Offset(cx - 22, y + 6) + Offset(-math.cos(a) * rr, math.sin(a) * rr);
    i == 0 ? tail.moveTo(cx - 14, y) : tail.lineTo(q.dx, q.dy);
  }
  c.drawPath(tail, _s(col, 4));
  for (final dx in [-8.0, 8.0]) {
    c.drawLine(Offset(cx + dx, y + 6), Offset(cx + dx + 3, y + 13), _s(dark, 3));
  }
  c.drawOval(Rect.fromCenter(center: Offset(cx, y), width: 34, height: 18), _f(col));
  for (var i = 0; i < 3; i++) {
    c.drawLine(Offset(cx - 8 + i * 7.0, y - 8), Offset(cx - 10 + i * 7.0, y + 2), _s(dark, 2));
  }
  c.drawPath(_poly([Offset(cx + 12, y - 8), Offset(cx + 28, y + 1), Offset(cx + 12, y + 7)]), _f(col));
  c.drawCircle(Offset(cx + 14, y - 3), 5, _f(dark));
  c.drawCircle(Offset(cx + 15 + math.sin(st.t * 2) * 1.2, y - 3), 2, _f(_k0));
}

void _fireworks(Canvas c, Size s, _S st) {
  for (var i = 0; i < 10; i++) {
    final h = 12 + _rn(i + 40) * 22, w = 10 + _rn(i + 50) * 8.0;
    c.drawRect(Rect.fromLTWH(i * 16.0, _ch - h, w + 6, h), _f(_k1));
  }
  final amp = .6 + (1 - st.y) * .8;
  for (var j = 0; j < 2; j++) {
    final ph = st.t / 2.4 + j * .5;
    final seed = 1000 + ph.floor() * 2 + j;
    _burst(c, Offset(40 + _rn(seed) * 76, 26 + _rn(seed + 1) * 26), _wr(ph) * 2.4, seed, amp * .8);
  }
  if (st.rel > 0 && st.t - st.rel < 2.4) {
    final o = st.p == null ? const Offset(78, 40) : Offset(st.p!.dx, math.min(st.p!.dy, 70));
    _burst(c, o, st.t - st.rel, st.ends + st.taps, amp);
  }
}

void _burst(Canvas c, Offset o, double since, int seed, double amp) {
  if (since < .35) {
    final q = Offset(o.dx, _ch - (_ch - o.dy) * since / .35);
    c.drawLine(q, q + const Offset(0, 10), _s(_ye, 2));
    return;
  }
  final u = since - .35, rays = 8 + seed % 9, fade = _cl(1.9 - u);
  final c1 = _pop[seed % 8], c2 = _pop[(seed * 3 + 1) % 8];
  final rr = 46 * amp * (1 - math.exp(-u * 3.2));
  for (var i = 0; i < rays; i++) {
    final a = i * 2 * math.pi / rays + _rn(seed) * 3;
    for (var k = 0; k < 4; k++) {
      final q = o + _pol(a, rr * (1 - k * .12)) + Offset(0, u * u * 10);
      c.drawCircle(q, (2.6 - k * .5) * fade + .2, _f(_al(k.isEven ? c1 : c2, fade * (1 - k * .2))));
    }
  }
  if (seed % 2 == 0) c.drawCircle(o, rr * .5, _s(_al(c2, fade), 2));
  c.drawCircle(o, 5 * fade, _f(_al(_wh, fade)));
}

void _mosaicImg(Canvas c) {
  _vgrad(c, const Rect.fromLTWH(0, 0, _cw, 70), _ye, _or);
  c.drawCircle(const Offset(78, 58), 24, _f(_rd));
  c.drawPath(_poly(const [Offset(-10, 82), Offset(40, 36), Offset(84, 82)]), _f(_vi));
  c.drawPath(_poly(const [Offset(60, 82), Offset(116, 30), Offset(170, 82)]), _f(_pk));
  c.drawRect(const Rect.fromLTWH(0, 80, _cw, 36), _f(_cy));
  for (var i = 0; i < 4; i++) {
    c.drawLine(Offset(20 + i * 34.0, 92 + (i % 2) * 8.0), Offset(38 + i * 34.0, 92 + (i % 2) * 8.0), _s(_wh, 2));
  }
}

void _mosaic(Canvas c, Size s, _S st) {
  if (st.b.isEmpty) st.b.addAll([0, 0]);
  if (st.b[1] != st.taps) {
    st.b[1] = st.taps.toDouble();
    st.b[0] = st.dist;
  }
  final amt = _ss((st.dist - st.b[0]) / 260), seed = st.ends;
  const tw = 26.0, th = 29.0;
  for (var i = 0; i < 6; i++) {
    for (var j = 0; j < 4; j++) {
      final k = i * 4 + j, r = Rect.fromLTWH(i * tw, j * th, tw, th);
      final src = ((_rn(k + seed * 31) * 24).floor());
      final si = src ~/ 4, sj = src % 4;
      final from = Offset(si * tw, sj * th) - r.topLeft;
      c.save();
      c.translate(r.center.dx, r.center.dy);
      c.rotate(amt * (_rn(k + 9 + seed) - .5) * 1.2);
      c.translate(-r.center.dx, -r.center.dy);
      c.clipRect(r.deflate(1));
      c.translate(-from.dx * amt, -from.dy * amt);
      _mosaicImg(c);
      c.restore();
    }
  }
}

void _donut(Canvas c, Size s, _S st) {
  const cc = Offset(78, 58);
  c.drawCircle(cc + const Offset(2, 4), 54, _f(_al(_k0, .12)));
  c.drawCircle(cc, 54, _f(_wh));
  final ring = Path()
    ..addOval(Rect.fromCircle(center: cc, radius: 44))
    ..addOval(Rect.fromCircle(center: cc, radius: 14))
    ..fillType = PathFillType.evenOdd;
  c.drawPath(ring, _f(_mix(_or, _ye, .45)));
  final glaze = Path()..fillType = PathFillType.evenOdd;
  final g = Path();
  for (var i = 0; i <= 60; i++) {
    final a = i / 60 * 2 * math.pi, r = 38 + 3 * math.sin(a * 7) + 2 * math.sin(a * 3);
    final q = cc + _pol(a, r);
    i == 0 ? g.moveTo(q.dx, q.dy) : g.lineTo(q.dx, q.dy);
  }
  glaze
    ..addPath(g..close(), Offset.zero)
    ..addOval(Rect.fromCircle(center: cc, radius: 18));
  c.drawPath(glaze, _f(_h(.95 + st.x * .6, .85, .62)));
  final seed = (st.dist / 140).floor() + st.taps * 7;
  for (var i = 0; i < 30; i++) {
    final q = i + seed * 41, a = _rn(q) * 2 * math.pi, r = 21 + _rn(q + 1) * 15;
    final p = cc + _pol(a, r), d = _pol(_rn(q + 2) * math.pi, 3);
    c.drawLine(p - d, p + d, _s(_pop[(q * 3) % 8 == 4 ? 2 : (q * 3) % 8], 2.6));
  }
}
