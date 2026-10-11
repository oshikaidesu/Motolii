part of 'pheno_e.dart';

// 1 Macro and 2 Variations: eight effect numbers become one small creature. The same creature is the portrait of a saved state.

const _macro = <PParam>[
  PParam('macro.blur', 'Blur', 0, 64, 0, 'px'),
  PParam('macro.glow', 'Glow', 0, 100, 0, '%'),
  PParam('macro.noise', 'Noise', 0, 100, 0, '%'),
  PParam('macro.sharpen', 'Sharpen', 0, 100, 50, '%'),
  PParam('macro.saturation', 'Saturation', 0, 200, 100, '%'),
  PParam('macro.contrast', 'Contrast', .5, 2, 1, '', dec: 2),
  PParam('macro.chroma', 'Chroma shift', 0, 12, 0, 'px', dec: 1),
  PParam('macro.vignette', 'Vignette', 0, 100, 0, '%'),
];
const _short = ['blur', 'glow', 'nois', 'shrp', 'satu', 'cont', 'chrm', 'vign'];

/// Draws the creature. [m] holds the eight macro values in [_macro] order. Eyes = glow + sharpen, brows = noise + chroma shift,
/// mouth = saturation + contrast, outline = blur + vignette. [fine] false = a portrait: no speckle, no ghost outlines.
void _paintFace(Canvas c, Offset o, double r, List<double> m,
    {double blink = 0, Offset look = Offset.zero, double tilt = 0, int seed = 0, int? hot, bool active = false, bool fine = true, double alpha = 1}) {
  final b = m[0] / 64, g = m[1] / 100, n = m[2] / 100, sh = m[3] / 100, sa = _cl((m[4] - 100) / 100, -1, 1);
  final ct = _cl(m[5] >= 1 ? m[5] - 1 : (m[5] - 1) / .5, -1, 1), ch = m[6] / 12, vg = m[7] / 100;
  c.save();
  c.translate(o.dx, o.dy);
  c.rotate(tilt);
  Color a(Color k, [double f = 1]) => k.withValues(alpha: k.a * f * alpha);
  // glow: a faint breath behind the head, never a fill
  if (g > .02) {
    final gr = r * (1.25 + .45 * g);
    c.drawCircle(Offset.zero, gr, Paint()..shader = ui.Gradient.radial(Offset.zero, gr, [a(N.g95, .13 * g), a(N.g95, 0)], [.5, 1]));
  }
  c.drawCircle(Offset.zero, r, _fl(a(N.g13)));
  if (vg > .01) {
    c.drawCircle(Offset.zero, r, Paint()..shader = ui.Gradient.radial(Offset.zero, r, [a(N.g00, 0), a(N.g00, .8 * vg)], [.5, 1]));
  }
  // outline: contrast = how bright, blur = how many soft copies
  final edge = Color.lerp(N.g44, N.g100, _cl(.5 + .5 * ct))!;
  if (fine && ch > .03) {
    c.drawCircle(Offset(-ch * 4, 0), r, _ln(a(Role.of(N.g44, Fam.scatter.c), .7)));
    c.drawCircle(Offset(ch * 4, 0), r, _ln(a(Role.of(N.g44, Fam.stagger.c), .7)));
  }
  final soft = fine ? (b * 5).round() : (b * 2).round();
  c.drawCircle(Offset.zero, r, _ln(a(edge, 1 - b * .55)));
  for (var i = 1; i <= soft; i++) {
    final rr = r + (i.isOdd ? 1 : -1) * ((i + 1) ~/ 2) * b * r * .055;
    c.drawCircle(Offset.zero, rr, _ln(a(edge, .5 * (1 - b * .4) / i)));
  }
  if (fine && n > .01) {
    var s = 1234567 + seed * 7919;
    double rnd() {
      s = (s * 1103515245 + 12345) & 0x7fffffff;
      return s / 0x7fffffff;
    }

    final dots = (n * 70).round();
    for (var i = 0; i < dots; i++) {
      final ang = rnd() * math.pi * 2, rad = math.sqrt(rnd()) * r * .93;
      c.drawRect(Rect.fromLTWH(math.cos(ang) * rad, math.sin(ang) * rad, 1, 1), _fl(a(N.g76, .55)));
    }
  }
  // cheeks: saturation
  final ck = Role.of(N.g56, Fam.scatter.c);
  for (final sx in [-1.0, 1.0]) {
    c.drawCircle(Offset(sx * .55 * r, .14 * r), .1 * r, _fl(a(ck, .12 + .55 * (sa + 1) / 2)));
  }
  // eyes
  final ex = .34 * r, ey = -.14 * r, open = .4 + .6 * g;
  final ry = math.max(.7, r * .15 * open * (1 - blink * .92)), rx = r * .15;
  for (final sx in [-1.0, 1.0]) {
    final ctr = Offset(sx * ex, ey);
    c.drawOval(Rect.fromCenter(center: ctr, width: rx * 2, height: ry * 2), _fl(a(N.g07)));
    c.drawOval(Rect.fromCenter(center: ctr, width: rx * 2, height: ry * 2), _ln(a(N.g91)));
    final pr = math.min(r * (.1 - .04 * sh), ry - .4);
    if (pr > .5) {
      final pc = ctr + Offset(look.dx * .045 * r, look.dy * .03 * r);
      c.drawCircle(pc, pr, _fl(a(N.g95)));
      if (sh > .55 && r > 16) c.drawCircle(pc + Offset(-pr * .35, -pr * .35), math.max(.6, r * .022), _fl(a(N.g07)));
    }
  }
  // brows: noise lowers them, chroma shift tilts them (inner ends down)
  final by = -.42 * r + n * .1 * r, tl = ch * .14 * r, bl = .17 * r;
  for (final sx in [-1.0, 1.0]) {
    final outer = Offset(sx * (ex + bl), by - tl), inner = Offset(sx * (ex - bl), by + tl);
    c.drawLine(outer, inner, _ln(a(N.g91), 1.2));
  }
  // mouth: saturation curves it, contrast widens it
  final my = .42 * r, w = r * (.16 + .16 * (ct + 1) / 2), cy = my - sa * .07 * r;
  c.drawPath(Path()
    ..moveTo(-w, cy)
    ..quadraticBezierTo(0, my + sa * .26 * r, w, cy), _ln(a(N.g95), 1.2));
  if (hot != null) {
    final p = _ln(active ? _hotInk : N.g95.withValues(alpha: .5));
    switch (hot) {
      case 0:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(0, ey), width: 1.14 * r, height: .38 * r), Radius.circular(.19 * r)), p);
      case 1:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(0, by), width: 1.2 * r, height: .26 * r + tl * 2), Radius.circular(.13 * r)), p);
      case 2:
        c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: Offset(0, my), width: 2 * w + .26 * r, height: .34 * r), Radius.circular(.17 * r)), p);
      case 3:
        c.drawCircle(Offset.zero, r + 4, p);
    }
  }
  c.restore();
}

/// Which feature of a creature at [o] with radius [r] is under [p]: 0 eyes, 1 brows, 2 mouth, 3 outline. Bands are at least 24 px tall.
int? _faceZone(Offset p, Offset o, double r) {
  final q = p - o, dist = q.distance;
  if (dist > r + 16) return null;
  if ((dist - r).abs() < 9 && dist > r * .86 && q.dy.abs() > .0) {
    final inBand = [-.14 * r, -.42 * r, .42 * r].any((y) => (q.dy - y).abs() < 6 && q.dx.abs() < .62 * r);
    if (!inBand) return 3;
  }
  if (dist > r + 12) return null;
  final ys = [-.14 * r, -.42 * r, .42 * r];
  var best = -1;
  var bd = 1e9;
  for (var i = 0; i < 3; i++) {
    final dy = (q.dy - ys[i]).abs();
    if (dy < bd) {
      bd = dy;
      best = i;
    }
  }
  if (bd < 12 && q.dx.abs() < .66 * r) return best;
  if (dist > r * .78) return 3;
  return null;
}

({Offset c, double r}) _macroFace(Size s) {
  final big = s.height >= 150;
  final r = big ? 66.0 : 40.0;
  return (c: Offset(big ? r + 28 : 96, s.height / 2 - (big ? 4 : 2)), r: r);
}

// ---------------------------------------------------------------------------------------------------------------------
class MacroModel extends PhenoModel {
  MacroModel({Map<String, double>? initial}) : super(_macro, initial ?? const {'macro.blur': 32, 'macro.glow': 18, 'macro.noise': 7});

  Offset _look = Offset.zero, _lookT = Offset.zero;
  double _blink = 0, _next = 1.6, _seedT = 0, _idle = 0;
  int _seed = 0;
  int _zoneDown = 0;

  @override
  String get caption => 'mood';
  @override
  String get story => 'Macro: Blur, Glow, Noise, Sharpen, Saturation, Contrast, Chroma shift, Vignette in one face';

  List<double> get _mood => [for (final p in _macro) d(p.id)];

  @override
  int? zoneAt(Offset p, Size s) {
    final f = _macroFace(s);
    return _faceZone(p, f.c, f.r);
  }

  @override
  void onHover(Offset? p, Size s) {
    if (p == null) {
      _lookT = Offset.zero;
      return;
    }
    final f = _macroFace(s), q = (p - f.c) / f.r;
    _lookT = q.distance > 1 ? q / q.distance : q;
  }

  @override
  bool get alive => true; // idle: a slow blink and a faint drift of the gaze

  @override
  void tick(double dt) {
    _idle += dt;
    final drift = hover ? Offset.zero : Offset(math.sin(_idle * .7) * .35, math.sin(_idle * .5 + 1) * .2);
    _look = _lerpO(_look, _lookT + drift, math.min(1, dt * (hover ? 14 : 3)));
    _next -= dt;
    if (_next <= 0 && _blink == 0) {
      _blink = .0001;
      _next = hover ? 2.6 + (_seed % 5) * .5 : 3.4 + (_seed % 3) * .9;
      _seed++;
    }
    if (hover) {
      if (n > .01) {
        _seedT += dt;
        if (_seedT > .12) {
          _seedT = 0;
          _seed++;
        }
      }
    }
    if (_blink > 0) {
      _blink += dt;
      if (_blink > .16) _blink = 0;
    }
  }

  double get n => v['macro.noise']!;

  @override
  void onDown(int zone, Offset p, Size s) => _zoneDown = zone;

  @override
  void onMove(Offset p, Offset delta, Size s, bool fine) {
    final f = _macroFace(s), k = (fine ? .1 : 1.0) / (f.r * .9);
    final dx = delta.dx, dy = delta.dy;
    switch (_zoneDown) {
      case 0:
        add('macro.sharpen', dx * 100 * k * .8);
        add('macro.glow', -dy * 100 * k);
      case 1:
        add('macro.noise', dy * 100 * k);
        add('macro.chroma', dx * 12 * k);
      case 2:
        add('macro.saturation', -dy * 200 * k * .8);
        add('macro.contrast', dx * 1.5 * k);
      case 3:
        final rel = p - f.c, prev = rel - delta;
        if (rel.distance > 1 && prev.distance > 1) {
          final dth = math.atan2(prev.dx * rel.dy - prev.dy * rel.dx, prev.dx * rel.dx + prev.dy * rel.dy);
          add('macro.blur', (rel.distance - prev.distance) * 64 * k * 1.3);
          add('macro.vignette', dth * f.r * 100 * k * .9);
        }
    }
  }

  @override
  void paintMini(Canvas c, Size s, int? hot) {
    final f = _macroFace(s);
    _paintFace(c, f.c, f.r, _mood, blink: math.sin(_blink / .16 * math.pi).abs() * (_blink > 0 ? 1 : 0), look: _look, seed: _seed, hot: hot, active: active != null);
  }

  @override
  List<Edge> edges(Size s) {
    final big = s.height >= 150;
    final out = <Edge>[_cap('mood')];
    if (!hover && active == null) return out;
    // only the three values that moved furthest from their default; the face stays the subject
    final ix = [for (var i = 0; i < 8; i++) if (changed(_macro[i].id)) i]
      ..sort((x, y) => ((v[_macro[y].id]! - _macro[y].def).abs() / (_macro[y].max - _macro[y].min)).compareTo((v[_macro[x].id]! - _macro[x].def).abs() / (_macro[x].max - _macro[x].min)));
    for (var k = 0; k < math.min(3, ix.length); k++) {
      final p = _macro[ix[k]];
      out.add(Edge('${_short[ix[k]]} ${p.fmt(v[p.id]!)}', right: big ? null : 8, left: big ? 196 : null, top: (big ? 78 : 40) + k * 15.0, hot: true));
    }
    return out;
  }
}

// ---------------------------------------------------------------------------------------------------------------------
const _bank0 = <List<double>>[
  [0, 0, 0, 50, 100, 1, 0, 0],
  [40, 60, 0, 25, 130, .85, 0, 25],
  [0, 0, 45, 70, 55, 1.4, 8, 30],
  [0, 70, 0, 80, 180, 1.15, 0, 0],
  [20, 0, 10, 15, 70, .7, 0, 60],
  [6, 10, 35, 90, 20, 1.8, 3, 85],
  [0, 25, 30, 95, 120, 1.3, 11, 0],
  [58, 30, 0, 10, 110, .6, 0, 10],
];

const _vparams = <PParam>[
  ..._macro,
  PParam('var.from', 'From slot', 0, 7, 1, ''),
  PParam('var.to', 'To slot', 0, 7, 3, ''),
  PParam('var.mix', 'Mix', 0, 100, 40, '%'),
];

class _VLay {
  _VLay(this.live, this.lr, this.cells, this.cr);
  final Offset live;
  final double lr, cr;
  final List<Offset> cells;
}

_VLay _vlay(Size s) {
  final big = s.height >= 150;
  final lr = big ? 50.0 : 32.0, live = Offset(lr + (big ? 20 : 12), s.height / 2 - (big ? 6 : 4));
  final x0 = live.dx + lr + (big ? 20 : 12), x1 = s.width - 6, cw = (x1 - x0) / 4, cr = math.min(cw * .42, big ? 17.0 : 16.5);
  final dy = big ? 36.0 : 29.0;
  return _VLay(live, lr, [for (var i = 0; i < 8; i++) Offset(x0 + cw * (i % 4 + .5), s.height / 2 + (i < 4 ? -dy : dy) - (big ? 4 : 2))], cr);
}

class VariationsModel extends PhenoModel {
  VariationsModel() : super(_vparams) {
    springK = 380;
    springZ = .62;
    _apply(_lerpList(bank[1], bank[3], .4));
    for (var i = 0; i < 8; i++) {
      pop.add(Spr(1));
    }
  }

  final List<List<double>> bank = [for (final b in _bank0) [...b]];
  final List<Spr> pop = [];
  final math.Random _rnd = math.Random(11);
  final Stopwatch _sw = Stopwatch()..start();
  final List<int> _revT = [];
  Spr tiltS = Spr(0);
  Offset? ghost;
  bool _free = false;
  int _slot = 0, _liveSign = 0;
  double _t = 0, _clock = 0;

  @override
  String get caption => 'moods';
  @override
  String get story => 'Variations: a bank of saved moods; recall, morph between two, shake to randomize';

  List<double> get _cur => [for (final p in _macro) d(p.id)];

  static List<double> _lerpList(List<double> a, List<double> b, double t) => [for (var i = 0; i < a.length; i++) _lerp(a[i], b[i], t)];

  void _apply(List<double> vals) {
    for (var i = 0; i < 8; i++) {
      set(_macro[i].id, vals[i]);
    }
  }

  @override
  Object? saveExtra() => ([for (final b in bank) [...b]], _free);
  @override
  void loadExtra(Object? x) {
    final (b, f) = x as (List<List<double>>, bool);
    for (var i = 0; i < 8; i++) {
      bank[i] = [...b[i]];
    }
    _free = f;
  }

  @override
  String extraSig() => '${bank.map((b) => b.join(',')).join(';')}|$_free';
  @override
  void resetExtra() {
    for (var i = 0; i < 8; i++) {
      bank[i] = [..._bank0[i]];
    }
    _free = false;
  }

  @override
  bool get alive => true; // idle: the portraits sway a little on their nails

  @override
  void tick(double dt) {
    _clock += dt;
    tiltS.t *= math.pow(.002, dt).toDouble();
    tiltS.step(dt, k: 240, z: .28);
    for (final p in pop) {
      p.step(dt, k: 520, z: .38);
    }
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final l = _vlay(s);
    for (var i = 0; i < 8; i++) {
      if ((p - l.cells[i]).distance <= math.max(l.cr + 3, 12)) return 10 + i;
    }
    return (p - l.live).distance <= l.lr + 6 ? 0 : null;
  }

  void _randomize() {
    final vals = <double>[];
    for (var i = 0; i < 8; i++) {
      final p = _macro[i], u = _rnd.nextDouble();
      final x = (i < 3 || i >= 6) ? math.pow(u, 1.7).toDouble() : .12 + .8 * u;
      vals.add(p.min + (p.max - p.min) * x);
    }
    _apply(vals);
    _free = true;
    set('var.mix', 0);
    tiltS.v += 3;
  }

  @override
  void onDown(int zone, Offset p, Size s) {
    _slot = zone - 10;
    _revT.clear();
    _liveSign = 0;
    if (zone == 0) ghost = p;
  }

  @override
  void onMove(Offset p, Offset delta, Size s, bool fine) {
    final l = _vlay(s);
    if (active == 0) {
      ghost = p;
      tiltS.t = _cl(delta.dx * .04, -.5, .5);
      if (delta.dx.abs() > 1.6) {
        final sg = _sgn(delta.dx).toInt();
        if (_liveSign != 0 && sg != _liveSign) {
          final now = _sw.elapsedMilliseconds;
          _revT.add(now);
          _revT.removeWhere((t) => now - t > 700);
          if (_revT.length >= 4) {
            _revT.clear();
            _randomize();
          }
        }
        _liveSign = sg;
      }
      return;
    }
    final i = _slot;
    var best = -1;
    var bd = 1e9;
    for (var j = 0; j < 8; j++) {
      if (j == i) continue;
      final dd = (p - l.cells[j]).distance;
      if (dd < bd) {
        bd = dd;
        best = j;
      }
    }
    final a = l.cells[i], b = l.cells[best], ab = b - a;
    _t = _cl(((p - a).dx * ab.dx + (p - a).dy * ab.dy) / (ab.dx * ab.dx + ab.dy * ab.dy));
    _apply(_lerpList(bank[i], bank[best], _t));
    set('var.from', i.toDouble());
    set('var.to', best.toDouble());
    set('var.mix', _t * 100);
    _free = false;
  }

  @override
  void onUp(Offset vel, bool moved) {
    if (active == 0) {
      final l = _vlay(_lastSize);
      final g = ghost;
      ghost = null;
      if (g != null) {
        for (var i = 0; i < 8; i++) {
          if ((g - l.cells[i]).distance <= l.cr + 8) {
            bank[i] = [for (final p in _macro) v[p.id]!];
            pop[i].x = 1.35;
            set('var.from', i.toDouble());
            set('var.to', i.toDouble());
            set('var.mix', 0);
            _free = false;
          }
        }
      }
      tiltS.t = 0;
      return;
    }
    if (!moved) {
      final i = _slot;
      _apply(bank[i]);
      set('var.from', i.toDouble());
      set('var.to', i.toDouble());
      set('var.mix', 0);
      _free = false;
      pop[i].x = 1.18;
    }
  }

  Size _lastSize = const Size(320, 200);

  @override
  void paintMini(Canvas c, Size s, int? hot) {
    _lastSize = s;
    final l = _vlay(s);
    c.drawLine(Offset(l.cells[0].dx - l.cr - 11, 16), Offset(l.cells[0].dx - l.cr - 11, s.height - 16), _ln(N.g20));
    final from = v['var.from']!.round(), to = v['var.to']!.round(), mix = v['var.mix']! / 100;
    // the morph line between the two ends
    if (!_free && mix > .005 && from != to) {
      c.drawLine(l.cells[from], l.cells[to], _ln(Role.of(N.g44, Role.linked).withValues(alpha: active != null ? .8 : .4)));
      c.drawCircle(_lerpO(l.cells[from], l.cells[to], mix), 2.2, _fl(Role.of(N.g95, Role.linked)));
    }
    for (var row = 0; row < 2; row++) {
      final y = l.cells[row * 4].dy + l.cr + 6;
      c.drawLine(Offset(l.cells[row * 4].dx - l.cr, y), Offset(l.cells[row * 4 + 3].dx + l.cr, y), _ln(N.g26));
    }
    for (var i = 0; i < 8; i++) {
      final sc = pop[i].x;
      c.save();
      // each portrait hangs a little crooked and sways on its own beat
      final sway = math.sin(_clock * .8 + i * 1.9) * .035, crook = ((i * 5) % 7 - 3) * .018;
      c.translate(l.cells[i].dx, l.cells[i].dy + math.sin(_clock * 1.1 + i * 2.3) * .8);
      c.scale(sc);
      _paintFace(c, Offset.zero, l.cr, bank[i], fine: false, alpha: .92, tilt: crook + sway, blink: math.max(0, math.sin(_clock * .6 + i * 2.7) - .985) * 60);
      c.restore();
      final isHot = hot == 10 + i;
      if (!_free && i == from) c.drawCircle(l.cells[i], l.cr + 4, _ln(Role.selected));
      if (!_free && i == to && mix > .005 && to != from) {
        _dashed(c, Path()..addOval(Rect.fromCircle(center: l.cells[i], radius: l.cr + 4)), _ln(Role.selected.withValues(alpha: .8)), on: 2, off: 2.5);
      }
      if (isHot) _hotRing(c, l.cells[i], l.cr + 4, on: active != null);
      if (ghost != null && (ghost! - l.cells[i]).distance <= l.cr + 8) _hotRing(c, l.cells[i], l.cr + 6, on: true);
    }
    _paintFace(c, l.live, l.lr, _cur, tilt: tiltS.x, hot: null, look: ghost == null ? Offset.zero : Offset.zero, blink: active == 0 ? .5 : 0);
    if (hot == 0 || active == 0) _hotRing(c, l.live, l.lr + 4, on: active == 0);
    if (ghost != null) _paintFace(c, ghost!, 11, _cur, fine: false, alpha: .85);
  }

  @override
  List<Edge> edges(Size s) {
    final big = s.height >= 150;
    final from = v['var.from']!.round(), to = v['var.to']!.round();
    return [
      _cap('moods'),
      Edge(_free ? 'unsaved' : '$from>$to  ${v['var.mix']!.round()}%', left: 7, bottom: 6, hot: _free || v['var.mix']! > 0),
      if (big) Edge(_cur.map((x) => x.round()).take(3).join('·'), right: 8, bottom: 6),
    ];
  }
}
