part of 'op_10.dart';

// Macro / Variations: many numbers moved as one character, saved moods, randomize.
// a = the macro (mood, energy, which variation), b = the second hand (seed, spread, pressure).

List<_Op> _macroPanels() => [
      _Op('mood face', 'character · effect · drag · drawing led', _G.drag, _mFace, a: .55, b: .5),
      _Op('dice cup', 'object · randomize · rub', _G.rub, _mDice, a: .5, b: .2),
      _Op('weather', 'landscape · effect · drag', _G.drag, _mWeather, a: .62),
      _Op('constellation', 'cosmic · saved moods blend · drag', _G.drag, _mStars, a: .4, b: .55),
      _Op('fish school', 'animal · effect · drag (size, spread)', _G.drag, _mSchool, a: .5, b: .4),
      _Op('marionette', 'character · mechanism · drag (tilt, lift)', _G.drag, _mPuppet, a: .6, b: .5),
      _Op('mood radar', 'diagram · saved moods · spin', _G.spin, _mRadar, a: .3, b: .6),
      _Op('helix mutate', 'science · randomize · flick', _G.flick, _mHelix, a: .2, b: .3),
      _Op('slot reels', 'machine · randomize · flick · numeral led', _G.flick, _mSlot, a: .3, b: .5),
      _Op('cocktail', 'object · mix effect · drag', _G.drag, _mGlass, a: .45, b: .5),
      _Op('gearbox', 'vehicle · saved moods (gears) · drag', _G.drag, _mGear, a: .5, b: .8),
      _Op('word morph', 'typographic · effect · drag', _G.drag, _mWord, a: .35),
      _Op('family tree', 'diagram · breed variations · drag', _G.drag, _mTree, a: .5, b: .3),
      _Op('terrain', 'isometric landscape · rough/seed · drag', _G.drag, _mTerrain, a: .55, b: .4),
      _Op('dancer', 'character · energy macro · drag', _G.drag, _mDancer, a: .5, b: .5),
      _Op('map lanes', 'm4l · macro curve to 4 params · draw', _G.draw, _mLanes),
      _Op('step dice', 'm4l · probability lanes · rub', _G.rub, _mSteps, a: .65, b: .1),
      _Op('kaleidoscope', 'cosmic · variation by turning · spin · pushed', _G.spin, _mKaleido, a: .1, b: .6),
      _Op('robot temper', 'machine · character · pinch', _G.pinch, _mRobot, a: .5, b: .3),
      _Op('volcano', 'landscape · randomize by eruption · drag up · pushed', _G.drag, _mVolcano, a: .5, b: .1),
    ];

const _enc = [_bl, _gr, _wh, _rd];

/// Four quiet parameter readouts in encoder colours along the bottom.
void _four(Canvas c, List<double> v, List<String> names, {double y = 104}) {
  for (var i = 0; i < 4; i++) {
    final x = 4 + i * 38.0;
    _lab(c, names[i], Offset(x, y), _al(_enc[i], .7), size: 6);
    final lw = _tp(names[i].toUpperCase(), 6, _al(_enc[i], .7), FontWeight.w500, .7).width;
    _num(c, _d2(v[i].clamp(0, 1) * 99), Offset(x + lw + 2, y - 2), 10, _enc[i]);
  }
}

void _mFace(Canvas c, _S st, double t) {
  final m = st.a, br = math.sin(t * 2) * 1.2;
  final ctr = Offset(62, 52 + br * .3);
  final pts = <Offset>[];
  for (var i = 0; i <= 40; i++) {
    final an = i / 40 * 2 * math.pi;
    final r = 34 + br + math.sin(an * 7 + t * 4) * m * m * 4;
    pts.add(ctr + Offset(math.cos(an), math.sin(an)) * r);
  }
  _pl(c, pts, _wh, w: 1.2);
  for (final sd in [-1.0, 1.0]) {
    final e = ctr + Offset(sd * 12, -6);
    _ring(c, e, 3 + m * 3, _bl, 1.1);
    _dot(c, e + Offset(math.sin(t * .7) * m * 2, 0), 1.3, _bl);
    final tilt = (m - .4) * 8 * sd;
    _ln(c, e + Offset(-6, -9 + tilt), e + Offset(6, -9 - tilt), _gr, 1.2);
  }
  final mouth = <Offset>[];
  for (var i = 0; i <= 16; i++) {
    final f = i / 16, x = _lp(-14, 14, f);
    final curve = (m - .3) * 12 * (1 - (2 * f - 1) * (2 * f - 1));
    final zig = m > .7 ? (i.isEven ? 1 : -1) * (m - .7) * 10 : 0;
    mouth.add(ctr + Offset(x, 14 + curve + zig));
  }
  _pl(c, mouth, _rd, w: 1.2);
  if (m > .8) {
    for (var i = 0; i < 3; i++) {
      _ln(c, ctr + Offset(30 + i * 5.0, -30 + i * 3.0), ctr + Offset(34 + i * 5.0, -36 + i * 3.0), _wh, 1);
    }
  }
  _lab(c, 'mood', const Offset(150, 6), _wh, ax: 1);
  _num(c, _d2(m * 99), const Offset(150, 15), 26, _wh, ax: 1);
  _four(c, [m * .8, m * m, .3 + m * .5, _sm((m - .5) * 2)], ['tempo', 'brow', 'eyes', 'grit']);
}

void _mDice(Canvas c, _S st, double t) {
  final seed = (st.b * 60).floor();
  final shake = st.down ? 1.0 : 0.0;
  final vals = [for (var i = 0; i < 3; i++) 1 + (_rn(seed * 3 + i) * 6).floor()];
  for (var i = 0; i < 3; i++) {
    final j = shake * math.sin(t * 40 + i * 2) * 3;
    final o = Offset(26 + i * 38.0 + j, 46 + (i == 1 ? -10 : 0) + shake * math.cos(t * 37 + i) * 3);
    const k = 7.0;
    _isoBox(c, o, 0, 0, 0, 2, 2, 2, k, _enc[i], 1.1);
    final top = _iso(o, 1, 1, 2, k);
    final v = vals[i];
    final pipPos = <Offset>[
      if (v.isOdd) Offset.zero,
      if (v >= 2) ...[const Offset(-.55, -.55), const Offset(.55, .55)],
      if (v >= 4) ...[const Offset(.55, -.55), const Offset(-.55, .55)],
      if (v == 6) ...[const Offset(-.55, 0), const Offset(.55, 0)],
    ];
    for (final q in pipPos) {
      _dot(c, top + Offset((q.dx - q.dy) * .866 * k, (q.dx + q.dy) * .5 * k), 1.2, _enc[i]);
    }
  }
  final cup = Offset(118 + shake * math.sin(t * 30) * 4, 30);
  _pl(c, [cup + const Offset(-14, -16), cup + const Offset(-10, 16), cup + const Offset(10, 16), cup + const Offset(14, -16)], _wh, w: 1);
  c.drawOval(Rect.fromCenter(center: cup + const Offset(0, -16), width: 28, height: 6), _s(_wh, 1));
  if (st.down) {
    for (var i = 0; i < 3; i++) {
      _ln(c, cup + Offset(18 + i * 3.0, -8 + i * 6.0), cup + Offset(23 + i * 3.0, -8 + i * 6.0), _mg, .8);
    }
  }
  _lab(c, 'seed', const Offset(6, 6), _mg);
  _num(c, _d2(seed), const Offset(6, 14), 16, _wh);
  _four(c, [vals[0] / 6, vals[1] / 6, vals[2] / 6, st.a], ['hue', 'grain', 'size', 'mix']);
}

void _mWeather(Canvas c, _S st, double t) {
  final m = st.a;
  final sun = Offset(40 - m * 20, 30 + m * 20);
  _ring(c, sun, 9, _al(_wh, 1 - m * .7), 1.1);
  for (var i = 0; i < 10; i++) {
    final an = i / 10 * 2 * math.pi + t * .2;
    final rl = 13 + (1 - m) * 6;
    _ln(c, sun + Offset(math.cos(an), math.sin(an)) * 12, sun + Offset(math.cos(an), math.sin(an)) * rl, _al(_wh, 1 - m * .8), .9);
  }
  final nc = (m * 5).ceil();
  for (var i = 0; i < nc; i++) {
    final x = _wr(i * .37 + t * (.02 + m * .05)) * 190 - 20;
    _cloud(c, Offset(x, 34 + i * 7.0 - m * 10), 30 + i * 4.0, i.isEven ? _bl : _pu, 1);
  }
  if (m > .35) {
    final n = ((m - .35) * 80).round();
    for (var i = 0; i < n; i++) {
      final x = _rn(i) * _cw, y = _wr(_rn(i + 50) + t * (1 + m)) * 70 + 40;
      _ln(c, Offset(x, y), Offset(x - 2 - m * 3, y + 5), _al(_bl, .8), .8);
    }
  }
  if (m > .8 && _wr(t * .7) < .12) {
    _pl(c, const [Offset(100, 30), Offset(92, 52), Offset(102, 52), Offset(90, 80)], _rd, w: 1.4);
  }
  _ln(c, const Offset(0, 96), const Offset(_cw, 96), _mg, 1);
  _pl(c, const [Offset(114, 96), Offset(114, 84), Offset(122, 77), Offset(130, 84), Offset(130, 96)], _wh, w: 1);
  _lab(c, ['calm', 'breezy', 'grey', 'wet', 'storm'][(m * 4.99).floor()], const Offset(6, 102), _gr);
  _num(c, _d2(m * 99), const Offset(150, 100), 14, _gr, ax: 1);
}

const _moods = [Offset(24, 24), Offset(70, 16), Offset(128, 30), Offset(40, 78), Offset(100, 70), Offset(134, 92)];

void _mStars(Canvas c, _S st, double t) {
  for (var i = 0; i < 30; i++) {
    _dot(c, Offset(_rn(i + 300) * _cw, _rn(i + 400) * _ch), .5, _al(_wh, .3 + .2 * math.sin(t * 2 + i)));
  }
  for (final e in const [[0, 1], [1, 2], [0, 3], [3, 4], [4, 2], [4, 5]]) {
    _dots(c, _moods[e[0]], _moods[e[1]], _dg, gap: 3);
  }
  final p = Offset(10 + st.a * 136, 110 - st.b * 100);
  final ws = [for (final m in _moods) 1 / math.pow(math.max(4, (m - p).distance), 2)];
  final sum = ws.reduce((x, y) => x + y);
  var best = 0;
  for (var i = 0; i < _moods.length; i++) {
    final w = ws[i] / sum;
    if (w > ws[best] / sum) best = i;
    _spark(c, _moods[i], 3 + w * 6, _al(_wh, .4 + w * .6), 1);
    if (w > .15) _ring(c, _moods[i], 6 + w * 8, _al(_gr, w), .8);
    _lab(c, 'abcdef'[i], _moods[i] + const Offset(5, 3), _mg, size: 6.5);
  }
  final tail = Offset(math.cos(t), math.sin(t)) * 2;
  _ln(c, p, p + const Offset(-14, 8) + tail, _al(_rd, .5), 1);
  _ln(c, p, p + const Offset(-10, 11) + tail, _al(_rd, .3), .8);
  _ring(c, p, 2.6, _rd, 1.2);
  _num(c, 'abcdef'[best].toUpperCase(), const Offset(150, 100), 14, _gr, ax: 1);
  _lab(c, '${_d2(ws[best] / sum * 99)}%', const Offset(132, 104), _gr, ax: 1);
}

void _mSchool(Canvas c, _S st, double t) {
  final n = 4 + (st.a * 16).round(), spread = 10 + st.b * 40;
  for (var i = 0; i < n; i++) {
    final ph = _rn(i) * 6.28, sp = .6 + _rn(i + 9) * .5;
    final x = 78 + math.sin(t * .5 * sp + ph) * 50 * (.4 + st.b * .6) + (_rn(i + 3) - .5) * spread;
    final y = 56 + math.cos(t * .7 * sp + ph * 1.3) * spread * .6 + (_rn(i + 5) - .5) * spread * .5;
    final dir = math.cos(t * .5 * sp + ph) > 0;
    _fish(c, Offset(x, y), 7 + _rn(i + 1) * 6 * (1 + st.a), i % 5 == 0 ? _gr : _al(_bl, .9), flip: !dir, wag: math.sin(t * 8 + i), w: .9);
  }
  for (var i = 0; i < 5; i++) {
    final x = 10 + i * 34.0;
    final p = Path()..moveTo(x, 116);
    for (var y = 116.0; y > 86; y -= 4) {
      p.lineTo(x + math.sin(y * .2 + t + i) * 2.5, y);
    }
    c.drawPath(p, _s(_al(_gr, .5), .8));
  }
  _lab(c, 'school', const Offset(6, 6), _bl);
  _num(c, _d2(n), const Offset(6, 14), 22, _bl);
  _lab(c, 'spread ${_d2(st.b * 99)}', const Offset(150, 6), _gr, ax: 1);
}

void _mPuppet(Canvas c, _S st, double t) {
  final tilt = (st.a - .5) * 1.0, lift = st.b;
  final bar = Offset(78, 14 + (1 - lift) * 10);
  final dx = Offset(math.cos(tilt), math.sin(tilt)) * 30;
  _ln(c, bar - dx, bar + dx, _wh, 1.6);
  _ln(c, bar - const Offset(0, 8), bar + const Offset(0, 8), _wh, 1.6);
  final sway = math.sin(t * 1.5) * 2;
  final foot = Offset(78 + sway, 104 - lift * 18);
  final hands = _stick(c, foot, 52, _wh,
      armL: -1.4 - tilt * 1.5 - lift * .6, armR: 1.4 - tilt * 1.5 + lift * .6, legL: -.25 + tilt * .5, legR: .25 + tilt * .5);
  final ends = [bar - dx, bar + dx, bar - const Offset(0, 8), bar + const Offset(0, 8)];
  final tips = [hands.$1, hands.$2, foot - const Offset(0, 49), foot];
  for (var i = 0; i < 4; i++) {
    _ln(c, ends[i], tips[i], _al(_enc[i], .8), .7);
    _dot(c, tips[i], 1.4, _enc[i]);
  }
  _ln(c, const Offset(30, 110), const Offset(126, 110), _dg, 1);
  _four(c, [.5 - tilt, .5 + tilt, lift, (tilt.abs() * 2)], ['left', 'right', 'lift', 'twist'], y: 4);
}

const _radarMoods = [
  [.2, .8, .5, .3],
  [.9, .3, .6, .7],
  [.5, .5, .95, .1],
  [.1, .2, .3, .9],
  [.7, .9, .2, .6],
];

void _mRadar(Canvas c, _S st, double t) {
  const ctr = Offset(70, 60), r = 36.0;
  final pos = st.a * _radarMoods.length, i0 = pos.floor() % _radarMoods.length;
  final i1 = (i0 + 1) % _radarMoods.length, f = _sm(pos - pos.floorToDouble());
  Offset ax(int k, double v) {
    final an = -math.pi / 2 + k * math.pi / 2;
    return ctr + Offset(math.cos(an), math.sin(an)) * r * v;
  }

  for (var k = 0; k < 4; k++) {
    _ln(c, ctr, ax(k, 1.1), _dg, .8);
    for (var s = 1; s <= 4; s++) {
      _dot(c, ax(k, s / 4), .7, _mg);
    }
  }
  final prev = _radarMoods[(i0 + _radarMoods.length - 1) % _radarMoods.length];
  _pl(c, [for (var k = 0; k < 4; k++) ax(k, prev[k])], _al(_pu, .4), w: .8, close: true);
  final cur = [for (var k = 0; k < 4; k++) _lp(_radarMoods[i0][k], _radarMoods[i1][k], f) * (.6 + st.b * .4)];
  final wob = [for (var k = 0; k < 4; k++) cur[k] + math.sin(t * 2 + k) * .02];
  _pl(c, [for (var k = 0; k < 4; k++) ax(k, wob[k])], _wh, w: 1.2, close: true);
  for (var k = 0; k < 4; k++) {
    _dot(c, ax(k, wob[k]), 2, _enc[k]);
  }
  _lab(c, 'speed', ax(0, 1.25) - const Offset(0, 4), _bl, ax: .5);
  _lab(c, 'size', ax(1, 1.2) + const Offset(2, -3), _gr);
  _lab(c, 'glow', ax(2, 1.18), _wh, ax: .5);
  _lab(c, 'jitter', ax(3, 1.0) + const Offset(0, -12), _rd, ax: .2);
  _lab(c, 'mood', const Offset(150, 6), _wh, ax: 1);
  _num(c, _d2(i0 + 1), const Offset(150, 15), 30, _wh, ax: 1);
  _lab(c, 'depth ${_d2(st.b * 99)}', const Offset(150, 108), _mg, ax: 1);
}

void _mHelix(Canvas c, _S st, double t) {
  final ph = st.a * 12 + t * .4;
  final mut = st.b;
  for (var i = 0; i < 26; i++) {
    final x = 6 + i * 5.7;
    final a1 = ph + i * .42;
    final y1 = 54 + math.sin(a1) * 26, y2 = 54 - math.sin(a1) * 26;
    final front = math.cos(a1) > 0;
    final mutated = _rn(i * 7 + (st.a * 20).floor()) < mut;
    final col = mutated ? _rd : _enc[(i * 3) % 3];
    _ln(c, Offset(x, y1), Offset(x, y2), _al(col, front ? .9 : .3), front ? 1 : .7);
    _dot(c, Offset(x, y1), front ? 1.6 : 1, _al(_wh, front ? 1 : .4));
    _dot(c, Offset(x, y2), front ? 1 : 1.6, _al(_wh, front ? .4 : 1));
    if (mutated && front) _spark(c, Offset(x, (y1 + y2) / 2), 3, _rd, .8);
  }
  _lab(c, 'flick to splice', const Offset(6, 6), _mg);
  _lab(c, 'mutate', const Offset(6, 98), _rd);
  _num(c, _d2(mut * 99), const Offset(6, 106), 12, _rd);
  _num(c, 'v${_d2(st.a * 20)}', const Offset(150, 100), 16, _wh, ax: 1);
}

void _mSlot(Canvas c, _S st, double t) {
  for (var i = 0; i < 4; i++) {
    final x = 8 + i * 30.0;
    final win = Rect.fromLTWH(x, 18, 26, 72);
    c.drawRRect(RRect.fromRectAndRadius(win, const Radius.circular(3)), _s(_dg, 1));
    c.save();
    c.clipRect(win);
    final v = st.a * (10 + i * 7) * 10 + st.b * i * 13;
    final base = v.floorToDouble(), fr = v - base;
    for (var k = -1; k <= 2; k++) {
      final dig = ((base + k) % 10).toInt();
      final y = 54 - (k - fr) * 28 - 12;
      final mid = k == 0 && fr < .5 || k == 1 && fr >= .5;
      _num(c, '$dig', Offset(x + 13, y), 24, mid ? _enc[i] : _al(_enc[i], .25), ax: .5);
    }
    c.restore();
    _ln(c, Offset(x - 2, 54), Offset(x, 54), _mg, 1);
  }
  final lever = 1 - _sm((t - st.rel) * 2);
  final top = Offset(140, 26 + (st.down ? 30 : lever * 30));
  _ln(c, const Offset(140, 74), top, _wh, 1.2);
  _ring(c, top, 4, _rd, 1.2);
  _rect(c, const Rect.fromLTWH(134, 72, 12, 14), _wh, 1);
  _lab(c, 'flick', const Offset(8, 6), _mg);
  _four(c, [for (var i = 0; i < 4; i++) _wr(st.a * (10 + i * 7) + st.b * i * 1.3)], ['rate', 'size', 'tone', 'pan']);
}

void _mGlass(Canvas c, _S st, double t) {
  final mix = st.a, tilt = (st.b - .5) * .4;
  c.save();
  c.translate(70, 104);
  c.rotate(tilt);
  const g = [Offset(-30, -86), Offset(-20, 0), Offset(20, 0), Offset(30, -86)];
  _pl(c, g, _wh, w: 1.2);
  for (var i = 0; i < 4; i++) {
    final base = -14.0 - i * 16;
    final p = Path();
    var started = false;
    for (var x = -26.0; x <= 26; x += 2) {
      final layerY = base + math.sin(x * .2 + t * 2 + i * 1.7) * (1 + mix * 9);
      final mixed = _lp(layerY, -40 + math.sin(x * .15 + i * 2 + t) * 20, mix * .8);
      final hw = _lp(20, 30, (-mixed) / 86);
      if (x.abs() > hw - 1) continue;
      if (!started) {
        p.moveTo(x, mixed);
        started = true;
      } else {
        p.lineTo(x, mixed);
      }
    }
    c.drawPath(p, _s(_enc[i], 1));
  }
  _ln(c, const Offset(8, -100), const Offset(-6, -20), _mg, 1.2);
  c.restore();
  _lab(c, 'mix', const Offset(150, 6), _wh, ax: 1);
  _num(c, _d2(mix * 99), const Offset(150, 15), 26, _wh, ax: 1);
  _lab(c, 'tilt', const Offset(150, 108), _mg, ax: 1);
}

void _mGear(Canvas c, _S st, double t) {
  const cols = [34.0, 64.0, 94.0], top = 28.0, mid = 58.0, bot = 88.0;
  for (final x in cols) {
    _ln(c, Offset(x, top), Offset(x, bot), _mg, 1.2);
  }
  _ln(c, const Offset(34, mid), const Offset(94, mid), _mg, 1.2);
  final y = _lp(bot, top, st.b);
  double x;
  if ((y - mid).abs() < 4) {
    x = _lp(34, 94, st.a);
  } else {
    x = cols[(st.a * 2).round()];
  }
  final names = ['1', '3', '5', '2', '4', 'r'];
  final moods = ['soft', 'warm', 'bright', 'busy', 'loud', 'random'];
  for (var i = 0; i < 6; i++) {
    final p = Offset(cols[i % 3], i < 3 ? top - 8 : bot + 8);
    _lab(c, names[i], p - const Offset(0, 4), _mg, ax: .5);
  }
  final k = Offset(x, y);
  _ln(c, k + const Offset(0, 6), k + const Offset(4, 14), _al(_wh, .5), 1);
  _ring(c, k, 6, _gr, 1.2);
  _dot(c, k, 1.4, _gr);
  var gear = -1;
  if ((y - top).abs() < 6) gear = cols.indexOf(x);
  if ((y - bot).abs() < 6) gear = 3 + cols.indexOf(x);
  _num(c, gear < 0 ? 'N' : names[gear].toUpperCase(), const Offset(150, 8), 40, _gr, ax: 1);
  _lab(c, gear < 0 ? 'neutral' : moods[gear], const Offset(150, 54), _wh, ax: 1);
  _gauge(c, const Offset(132, 92), 14, gear < 0 ? .1 + math.sin(t * 3) * .03 : .2 + gear * .14 + math.sin(t * 9) * .02, _rd);
}

const _words = ['calm', 'soft', 'warm', 'busy', 'loud', 'wild'];

void _mWord(Canvas c, _S st, double t) {
  final pos = st.a * (_words.length - 1), i0 = pos.floor().clamp(0, _words.length - 1);
  final i1 = math.min(i0 + 1, _words.length - 1), f = pos - i0;
  final en = st.a;
  void word(String w, double y, Color col, double alpha) {
    var adv = 12.0;
    for (var k = 0; k < w.length; k++) {
      final j = Offset(math.sin(t * (3 + en * 9) + k * 1.7), math.cos(t * (2 + en * 11) + k * 2.3)) * en * en * 6;
      final rot = math.sin(t * 4 + k) * en * en * .4;
      c.save();
      c.translate(adv + j.dx, y + j.dy);
      c.rotate(rot);
      _num(c, w[k].toUpperCase(), Offset.zero, 32, _al(col, alpha), ax: 0);
      c.restore();
      adv += _tp(w[k].toUpperCase(), 32, _al(col, alpha), FontWeight.w200, -32 * .02).width + 3 + en * 6;
    }
  }

  c.save();
  c.clipRect(const Rect.fromLTRB(0, 0, _cw, 84));
  word(_words[i0], 36 - f * 32, _wh, 1 - f);
  word(_words[i1], 68 - f * 32, _gr, .2 + f * .8);
  c.restore();
  final p = Path()..moveTo(10, 92);
  for (var x = 10.0; x <= 146; x += 3) {
    p.lineTo(x, 92 + math.sin(x * (.08 + en * .3) + t * 4) * en * 6);
  }
  c.drawPath(p, _s(_bl, 1));
  _lab(c, 'energy', const Offset(6, 104), _bl);
  _num(c, _d2(en * 99), const Offset(150, 102), 12, _bl, ax: 1);
}

void _shape(Canvas c, Offset p, int sides, double r, double rot, Color col, [double w = 1]) {
  _pl(c, [for (var i = 0; i < sides; i++) p + Offset(math.cos(rot + i / sides * 2 * math.pi), math.sin(rot + i / sides * 2 * math.pi)) * r], col,
      w: w, close: true);
}

void _mTree(Canvas c, _S st, double t) {
  const pa = Offset(46, 20), pb = Offset(110, 20);
  _shape(c, pa, 3, 9, -math.pi / 2, _bl, 1.2);
  _shape(c, pb, 7, 9, t * .2, _rd, 1.2);
  _lab(c, 'a', pa + const Offset(-20, -3), _bl);
  _lab(c, 'b', pb + const Offset(14, -3), _rd);
  const mid = Offset(78, 40);
  _ln(c, pa + const Offset(0, 9), mid, _mg, .8);
  _ln(c, pb + const Offset(0, 9), mid, _mg, .8);
  _dot(c, mid, 1.6, _wh);
  final gen = (st.b * 9).floor();
  final sel = (st.a * 4.99).floor();
  for (var i = 0; i < 5; i++) {
    final p = Offset(18 + i * 30.0, 76);
    _ln(c, mid, p - const Offset(0, 12), _dg, .8);
    final mixAmt = _rn(gen * 5 + i);
    final sides = (3 + mixAmt * 4 + _rn(gen * 9 + i) * 2).round();
    final col = Color.lerp(_bl, _rd, mixAmt)!;
    final r = 8 + math.sin(t * 2 + i) * (i == sel ? 1.2 : 0);
    _shape(c, p, sides, r, _rn(gen + i * 3) * 6, col, i == sel ? 1.4 : .9);
    if (i == sel) _ring(c, p, 13, _gr, 1);
  }
  _lab(c, 'gen', const Offset(6, 98), _mg);
  _num(c, _d2(gen + 1), const Offset(6, 106), 12, _wh);
  _lab(c, 'child ${sel + 1}', const Offset(150, 104), _gr, ax: 1);
}

void _mTerrain(Canvas c, _S st, double t) {
  const o = Offset(78, 30), k = 5.6, n = 10;
  final rough = st.a, seed = (st.b * 40).floor();
  double h(int i, int j) {
    final base = math.sin(i * .7 + seed) * math.cos(j * .6 - seed * .3) + .6 * math.sin((i + j) * .4 + t * .5);
    return base * (.3 + rough * 2.6) + (_rn(i * 31 + j * 7 + seed) - .5) * rough * rough * 3;
  }

  final hs = [for (var i = 0; i <= n; i++) [for (var j = 0; j <= n; j++) h(i, j)]];
  for (var i = 0; i <= n; i++) {
    _pl(c, [for (var j = 0; j <= n; j++) _iso(o, i * 1.0, j * 1.0 - 1, hs[i][j], k)], i == n ~/ 2 ? _gr : _al(_bl, .8), w: .8);
    _pl(c, [for (var j = 0; j <= n; j++) _iso(o, j * 1.0, i * 1.0 - 1, hs[j][i], k)], _al(_pu, .6), w: .7);
  }
  _lab(c, 'rough', const Offset(6, 6), _gr);
  _num(c, _d2(rough * 99), const Offset(6, 14), 20, _gr);
  _lab(c, 'seed', const Offset(150, 6), _bl, ax: 1);
  _num(c, _d2(seed), const Offset(150, 14), 20, _bl, ax: 1);
}

void _mDancer(Canvas c, _S st, double t) {
  final e = st.a, bpm = 60 + e * 120;
  final beat = t * bpm / 60 * math.pi;
  final s1 = math.sin(beat), s2 = math.sin(beat * .5 + 1);
  final foot = Offset(70 + s2 * e * 10, 100 - s1.abs() * e * 10);
  _stick(c, foot, 64, _wh,
      armL: 2.4 - e * 1.6 * s1 - st.b * .6,
      armR: -2.4 + e * 1.6 * s2 + st.b * .6,
      legL: .2 + e * .6 * s1.abs(),
      legR: -.2 - e * .5 * s2.abs(),
      lean: s2 * e * .3);
  c.drawOval(Rect.fromCenter(center: const Offset(70, 104), width: 40 + e * 30, height: 6), _s(_dg, 1));
  if (e > .4) {
    for (var i = 0; i < 3; i++) {
      final x = 30 - i * 5.0 + s2 * 6;
      _ln(c, Offset(x, 40 + i * 10.0), Offset(x - 6 * e, 42 + i * 10.0), _al(_rd, e), .9);
    }
  }
  for (var i = 0; i < 4; i++) {
    final on = (beat / math.pi).floor() % 4 == i;
    _dot(c, Offset(118 + i * 8.0, 110), on ? 2 : 1, on ? _gr : _dg);
  }
  _lab(c, 'bpm', const Offset(150, 6), _gr, ax: 1);
  _num(c, _d3(bpm), const Offset(150, 15), 22, _gr, ax: 1);
  _lab(c, 'reach ${_d2(st.b * 99)}', const Offset(150, 44), _mg, ax: 1);
}

void _mLanes(Canvas c, _S st, double t) {
  final pts = st.pts.length > 3 ? st.pts : [for (var i = 0; i <= 20; i++) Offset(8 + i * 4.0, 100 - math.pow(i / 20, 1.8) * 84)];
  _rect(c, const Rect.fromLTRB(8, 16, 88, 100), _dg, .8);
  _pl(c, pts, _wh, w: 1.2);
  final m = _wr(t * .15);
  final x = 8 + m * 80;
  var y = 100.0;
  for (var i = 1; i < pts.length; i++) {
    if ((pts[i - 1].dx - x) * (pts[i].dx - x) <= 0) {
      final f = (x - pts[i - 1].dx) / math.max(1e-3, pts[i].dx - pts[i - 1].dx);
      y = _lp(pts[i - 1].dy, pts[i].dy, f.clamp(0.0, 1.0));
      break;
    }
  }
  final v = ((100 - y) / 84).clamp(0.0, 1.0);
  _ln(c, Offset(x, 16), Offset(x, 100), _al(_gr, .6), .8);
  _dot(c, Offset(x, y), 2, _gr);
  const lo = [.1, .4, .0, .6], hi = [.9, .7, .5, 1.0], dir = [1, -1, 1, 1];
  for (var i = 0; i < 4; i++) {
    final ly = 22 + i * 20.0;
    _ln(c, Offset(98, ly), Offset(150, ly), _dg, .8);
    final x0 = 98 + lo[i] * 52, x1 = 98 + hi[i] * 52;
    _ln(c, Offset(x0, ly - 3), Offset(x0, ly + 3), _enc[i], 1);
    _ln(c, Offset(x1, ly - 3), Offset(x1, ly + 3), _enc[i], 1);
    _ln(c, Offset(x0, ly), Offset(x1, ly), _al(_enc[i], .5), 1);
    final vv = dir[i] > 0 ? v : 1 - v;
    _dot(c, Offset(_lp(x0, x1, vv), ly), 2.2, _enc[i]);
    _lab(c, ['size', 'blur', 'hue', 'speed'][i], Offset(98, ly - 11), _al(_enc[i], .7), size: 6.5);
  }
  _lab(c, 'macro', const Offset(8, 6), _gr);
  _lab(c, 'draw curve', const Offset(8, 106), _mg);
}

void _mSteps(Canvas c, _S st, double t) {
  final prob = st.a, seed = (st.b * 50).floor();
  final head = (t * 4).floor() % 16;
  for (var r = 0; r < 4; r++) {
    final y0 = 14 + r * 22.0;
    final pts = <Offset>[];
    for (var s = 0; s < 16; s++) {
      final x = 10 + s * 8.5;
      final on = _rn(seed * 64 + r * 16 + s + 500) < prob;
      final v = _rn(seed * 64 + r * 16 + s);
      final p = Offset(x, y0 + 16 - v * 14);
      if (on) {
        pts.add(p);
        _dot(c, p, s == head ? 2.4 : 1.4, s == head ? _wh : _enc[r]);
      } else {
        _ring(c, Offset(x, y0 + 16), .9, _dg, .7);
      }
    }
    _pl(c, pts, _al(_enc[r], .35), w: .8);
  }
  final hx = 10 + head * 8.5;
  _dots(c, Offset(hx, 10), Offset(hx, 100), _mg, gap: 3);
  _lab(c, 'prob ${_d2(prob * 99)}', const Offset(10, 104), _gr);
  _lab(c, 'rub to reroll · ${_d2(seed)}', const Offset(150, 104), _mg, ax: 1);
}

void _mKaleido(Canvas c, _S st, double t) {
  const ctr = Offset(70, 60);
  final seed = (st.a * 12).floor(), rot = st.a * 2 * math.pi, r = 20 + st.b * 30;
  final motif = [
    for (var i = 0; i < 5; i++) Offset(_rn(seed * 11 + i) * r, (_rn(seed * 13 + i) - .5) * r * .6 + math.sin(t + i) * 1.5),
  ];
  for (var k = 0; k < 6; k++) {
    for (final m in [1.0, -1.0]) {
      final an = rot + k * math.pi / 3;
      final ca = math.cos(an), sa = math.sin(an);
      final pts = [for (final p in motif) ctr + Offset(p.dx * ca - p.dy * m * sa, p.dx * sa + p.dy * m * ca)];
      _pl(c, pts, m > 0 ? _gr : _bl, w: .9);
      _dot(c, pts.last, 1.2, _rd);
    }
  }
  _ring(c, ctr, r + 6, _dg, .8);
  _lab(c, 'turn', const Offset(150, 6), _gr, ax: 1);
  _num(c, _d2(seed + 1), const Offset(150, 15), 26, _gr, ax: 1);
  _lab(c, 'bloom ${_d2(st.b * 99)}', const Offset(150, 108), _bl, ax: 1);
}

void _mRobot(Canvas c, _S st, double t) {
  final chaos = st.b, polite = st.a;
  const hc = Offset(64, 54);
  final j = Offset(math.sin(t * 23), math.cos(t * 19)) * chaos * chaos * 2;
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: hc + j, width: 64, height: 50), const Radius.circular(6)), _s(_wh, 1.2));
  final ant = hc + j - const Offset(0, 25);
  _ln(c, ant, ant - const Offset(0, 14), _wh, 1);
  _ring(c, ant - const Offset(0, 17), 3, _rd, 1);
  if (chaos > .5) {
    for (var i = 0; i < 4; i++) {
      final an = i * math.pi / 2 + t * 5;
      _ln(c, ant - const Offset(0, 17) + Offset(math.cos(an), math.sin(an)) * 5, ant - const Offset(0, 17) + Offset(math.cos(an), math.sin(an)) * 9,
          _al(_rd, chaos), .8);
    }
  }
  for (final sd in [-1.0, 1.0]) {
    final e = hc + j + Offset(sd * 14, -6);
    if (chaos > .6) {
      final p = Path();
      for (var i = 0; i < 24; i++) {
        final an = i * .5 + t * 6 * sd, rr = i * .3;
        final q = e + Offset(math.cos(an), math.sin(an)) * rr;
        i == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
      }
      c.drawPath(p, _s(_bl, .9));
    } else {
      _ring(c, e, 6, _bl, 1.1);
      _ln(c, e - Offset(4, 0), e + Offset(4, 0) + Offset(0, (polite - .5) * 6), _bl, 1);
    }
  }
  for (var i = 0; i < 7; i++) {
    final x = hc.dx + j.dx - 15 + i * 5.0;
    final hgt = 2 + (polite * 4) * math.sin(i / 6 * math.pi) + chaos * _rn(i + (t * 8).floor()) * 6;
    _ln(c, Offset(x, hc.dy + j.dy + 14 - hgt / 2), Offset(x, hc.dy + j.dy + 14 + hgt / 2), _gr, 1);
  }
  _ln(c, const Offset(52, 79), const Offset(52, 92), _mg, 1);
  _ln(c, const Offset(76, 79), const Offset(76, 92), _mg, 1);
  _ln(c, const Offset(30, 92), const Offset(98, 92), _mg, 1);
  _lab(c, 'manners', const Offset(150, 6), _gr, ax: 1);
  _num(c, _d2(polite * 99), const Offset(150, 15), 18, _gr, ax: 1);
  _lab(c, 'chaos', const Offset(150, 84), _rd, ax: 1);
  _num(c, _d2(chaos * 99), const Offset(150, 93), 18, _rd, ax: 1);
}

void _mVolcano(Canvas c, _S st, double t) {
  const peakL = Offset(62, 44), peakR = Offset(82, 44);
  _pl(c, const [Offset(4, 96), Offset(40, 70), peakL, Offset(66, 48), Offset(76, 48), peakR, Offset(108, 72), Offset(150, 96)], _wh, w: 1.2);
  _ln(c, const Offset(0, 96), const Offset(_cw, 96), _dg, 1);
  final pr = st.b;
  final lvl = 92 - pr * 44;
  for (var i = 0; i < 4; i++) {
    final y = lvl + i * 5;
    if (y > 94) break;
    final half = _lp(42, 8, (94 - y) / 50);
    final p = Path()..moveTo(72 - half, y);
    for (var x = -half; x <= half; x += 3) {
      p.lineTo(72 + x, y + math.sin(x * .5 + t * 4 + i) * 1.2);
    }
    c.drawPath(p, _s(_al(_rd, .8 - i * .15), .9));
  }
  final since = t - st.rel;
  final seed = st.n;
  final erupting = !st.down && st.n > 0 && since < 2.4;
  final land = <Offset>[];
  for (var i = 0; i < 4; i++) {
    final vx = (_rn(seed * 4 + i) - .5) * 90, vy = -60 - _rn(seed * 4 + i + 2) * 40;
    final tt = math.min(since, 2.4) * .9;
    var p = Offset(72 + vx * tt, 44 + vy * tt + 70 * tt * tt);
    if (p.dy > 96 || !erupting) {
      final tl = (-vy + math.sqrt(vy * vy + 4 * 70 * 52)) / (2 * 70);
      p = Offset((72 + vx * tl).clamp(6, 150), 96);
    }
    land.add(p);
    if (erupting && p.dy < 96) {
      _ring(c, p, 2.4, _enc[i], 1.2);
    } else if (st.n > 0) {
      _dot(c, p, 2.2, _enc[i]);
      _ln(c, p, p - const Offset(0, 6), _al(_enc[i], .5), .8);
    }
  }
  if (erupting && since < .6) {
    for (var i = 0; i < 10; i++) {
      final an = -math.pi / 2 + (_rn(i + seed) - .5) * 1.4;
      _ln(c, const Offset(72, 44) + Offset(math.cos(an), math.sin(an)) * since * 30,
          const Offset(72, 44) + Offset(math.cos(an), math.sin(an)) * (since * 30 + 5), _rd, 1);
    }
  }
  _lab(c, 'pressure', const Offset(6, 6), _rd);
  _num(c, _d2(pr * 99), const Offset(6, 14), 20, _rd);
  _lab(c, 'drag up · let go', const Offset(150, 6), _mg, ax: 1);
  _four(c, [for (final p in land) st.n > 0 ? p.dx / _cw : 0], ['a', 'b', 'c', 'd'], y: 106);
}
