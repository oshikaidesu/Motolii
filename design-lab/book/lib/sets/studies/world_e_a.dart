// World E, part A: 25 Easing, 26 Spring, 27 Wiggle (research/op1-translation.md section 3.3).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_e_kit.dart';

// ---- 25. Easing: footprints on a ribbon ------------------------------------------------------------------------------------------------
// A dot runs along a wavy ribbon and leaves a footprint every 100 ms. Where the footprints crowd, it was slow. No curve, no handle.
// A blue = In (the first footprints, crowded = a slow start), B green = Out (the last), C white = Overshoot (the target ring; past it the dot runs on and comes back),
// D orange = Duration (the runner: pull it faster or slower; a longer time leaves more footprints).

const easingSpecs = [
  WSpec('in', 'In', 0, 100, 33, unit: '%', digits: 0),
  WSpec('out', 'Out', 0, 100, 33, unit: '%', digits: 0),
  WSpec('over', 'Overshoot', 0, 100, 0, unit: '%', digits: 0),
  WSpec('dur', 'Duration', 0, 5, 1, unit: 's', digits: 1),
];

WorldDef easingDef() => WorldDef(
      name: 'Easing',
      specs: easingSpecs,
      make: EaseWorld.new,
      silhouette: 'footprints on a ribbon',
      rowH: 96,
      above: ('Opacity', '100 %'),
      below: ('Rotation', '0°'),
    );

class EaseWorld extends WeWorld {
  EaseWorld(super.doc);
  double _ph = .15;

  static double _bz(double a, double b, double u) => 3 * a * u * (1 - u) * (1 - u) + 3 * b * u * u * (1 - u) + u * u * u;

  /// Progress at time fraction [t] of cubic-bezier(in, 0, 1 - out, 1 + overshoot).
  double prog(double t, [double? i, double? o, double? ov]) {
    final x1 = (i ?? v(0)) / 100, x2 = 1 - (o ?? v(1)) / 100, y2 = 1 + (ov ?? v(2)) / 100 * .9;
    var lo = 0.0, hi = 1.0;
    for (var k = 0; k < 20; k++) {
      final u = (lo + hi) / 2;
      if (_bz(x1, x2, u) < t) {
        lo = u;
      } else {
        hi = u;
      }
    }
    return _bz(0, y2, (lo + hi) / 2);
  }

  double get _dur => math.max(.1, v(3));

  Offset pt(Rect a, double s) {
    final x0 = a.left + 8, xe = a.left + a.width * .70, amp = a.height * .2;
    final breath = math.sin(doc.t * 1.3) * .9;
    return Offset(x0 + s * (xe - x0), a.center.dy + a.height * .05 - amp * math.sin(s * math.pi * 1.25 - .5) + breath);
  }

  double get _runnerS {
    final cyc = _dur + .8, u = _ph * cyc;
    return prog(weClamp(u / _dur, 0, 1));
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s);
    double best = 1e9;
    int? z;
    void cand(int zz, double d, double reach, [double bias = 0]) {
      if (d <= reach && d - bias < best) {
        best = d - bias;
        z = zz;
      }
    }

    var dA = 1e9, dB = 1e9;
    for (var k = 0; k <= 14; k++) {
      dA = math.min(dA, (p - pt(a, prog(.28 * k / 14))).distance);
      dB = math.min(dB, (p - pt(a, prog(.72 + .28 * k / 14))).distance);
    }
    cand(0, dA, 13);
    cand(1, dB, 13);
    cand(2, (p - pt(a, 1)).distance, 15, 6);
    cand(3, (p - pt(a, _runnerS)).distance, 16, 3);
    return z;
  }

  @override
  void down(int z, Offset p, Size s) {}

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s);
    switch (z) {
      case 0:
        put(0, base(0) + d.dx * k * 100 / (a.width * .45));
      case 1:
        put(1, base(1) - d.dx * k * 100 / (a.width * .45));
      case 2:
        put(2, base(2) + d.dx * k * 100 / (a.width * .3));
      case 3:
        put(3, base(3) + d.dx * k * 5 / (a.width * .6));
    }
  }

  @override
  void step(double dt, Size s) {
    if (view.grab == 3) return;
    _ph = (_ph + dt / (_dur + .8)) % 1.0;
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s);
    // the ribbon (the ground the dot runs on)
    final ribbon = Path()..moveTo(pt(a, -.04).dx, pt(a, -.04).dy);
    for (var k = 1; k <= 60; k++) {
      final q = pt(a, -.04 + 1.34 * k / 60);
      ribbon.lineTo(q.dx, q.dy);
    }
    c.drawPath(ribbon, weLine(N.g26));
    // footprints, one per 100 ms of real time
    final dur = _dur, n = math.min(60, (dur / .1).floor());
    final dim = view.grab != null;
    for (var k = 0; k <= n + 1; k++) {
      final tf = math.min(1.0, k * .1 / dur);
      if (k == n + 1 && tf >= 1 && n * .1 / dur >= 1) break;
      final q = pt(a, prog(tf));
      final col = tf < .28 ? slot(0) : (tf > .72 ? slot(1) : (dim ? N.g26 : N.g56));
      c.drawCircle(q, 1.9, weFill(col));
      if (tf >= 1) break;
    }
    // hover: the footprints' region lights as a 1 px line on the ribbon
    for (final z in [0, 1]) {
      if (!on(z)) continue;
      final r = Path();
      for (var k = 0; k <= 16; k++) {
        final tf = z == 0 ? .28 * k / 16 : .72 + .28 * k / 16, q = pt(a, prog(tf));
        k == 0 ? r.moveTo(q.dx, q.dy) : r.lineTo(q.dx, q.dy);
      }
      c.drawPath(r, weLine(slot(z), 1.5));
    }
    // the target ring (white): past it, overshoot
    final tg = pt(a, 1);
    c.drawCircle(tg, 4.5, weLine(slot(2)));
    c.drawCircle(tg, .8, weFill(slot(2)));
    ring(c, tg, 9, 2);
    // the runner (orange)
    final rs = pt(a, _runnerS);
    c.drawCircle(rs, 3, weFill(slot(3, 1)));
    ring(c, rs, 8, 3);
  }
}

// ---- 26. Spring: a weight on a coil with an ink trail ---------------------------------------------------------------------------------
// A mass hangs from a coil beside a dashpot; its path is inked to the right, so the settling is seen. A blue = Stiffness (the coil: tight and narrow = stiff),
// B green = Damping (the dashpot: the denser its fill the slower the swing dies), C white = Mass (the block; tap it to flick it),
// D orange = Rest (the dashed horizon the swing settles to).

const springSpecs = [
  WSpec('k', 'Stiffness', 1, 500, 120, digits: 0),
  WSpec('c', 'Damping', 0, 50, 12, digits: 0),
  WSpec('m', 'Mass', .1, 10, 1, digits: 1),
  WSpec('rest', 'Rest', -100, 100, 0, unit: 'px', digits: 0),
];

WorldDef springDef() => WorldDef(
      name: 'Spring',
      specs: springSpecs,
      make: SpringWorld.new,
      silhouette: 'a weight on a coil with an ink trail',
      rowH: 120,
      above: ('Position Y', '540 px'),
      below: ('Scale', '100 %'),
    );

class SpringWorld extends WeWorld {
  SpringWorld(super.doc);
  double _p = 0, _vy = 0, _kickT = 1.6, _moved = 0;
  final _hist = <double>[], _age = <double>[];

  double _scale(Rect a) => a.height / 130;
  double _restPx(Rect a) => v(3) / 100 * a.height * .2;
  double _yMass0(Rect a) => a.top + a.height * .6;
  double _hs(Rect a) => weClamp(3.5 + 2.2 * math.sqrt(v(2)) * _scale(a), 4, a.height * .17);
  double _massY(Rect a) => weClamp(_yMass0(a) + _p, a.top + 22 * _scale(a), a.bottom - _hs(a) - 1);
  double _cx(Rect a) => a.left + a.width * .26;
  double _xd(Rect a) => _cx(a) + 22;
  double _xt(Rect a) => _cx(a) + 48;

  @override
  void step(double dt, Size s) {
    final a = art(s), k = v(0), c = v(1), m = v(2), rp = _restPx(a);
    var left = dt;
    while (left > 1e-6) {
      final h = math.min(left, 1 / 240);
      left -= h;
      final acc = (-k * (_p - rp) - c * _vy) / m;
      _vy += acc * h;
      _p += _vy * h;
    }
    _p = weClamp(_p, -a.height, a.height);
    _kickT -= dt;
    if (_kickT <= 0) {
      _kickT = 3.4;
      if ((_p - rp).abs() < 1.5 && _vy.abs() < 6) _vy += 95 * _scale(a);
    }
    _hist.insert(0, _massY(a));
    _age.insert(0, 0);
    for (var i = 0; i < _age.length; i++) {
      _age[i] += dt;
    }
    final life = (a.right - _xt(a)) / 70;
    while (_age.isNotEmpty && _age.last > life) {
      _age.removeLast();
      _hist.removeLast();
    }
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s), cx = _cx(a), xd = _xd(a), my = _massY(a), hs = _hs(a);
    if (Rect.fromCenter(center: Offset(cx, my), width: 2 * hs + 16, height: 2 * hs + 16).contains(p)) return 2;
    if (p.dx >= _xt(a) - 8 && (p.dy - (_yMass0(a) + _restPx(a))).abs() <= 13) return 3;
    if ((p.dx - xd).abs() <= 12 && p.dy >= a.top && p.dy <= my) return 1;
    if ((p.dx - cx).abs() <= 16 && p.dy >= a.top && p.dy <= my) return 0;
    return null;
  }

  @override
  void down(int z, Offset p, Size s) => _moved = 0;

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s), span = a.height * .7;
    _moved += d.distance;
    switch (z) {
      case 0:
        put(0, base(0) - d.dy * k * 500 / span);
      case 1:
        put(1, base(1) - d.dy * k * 50 / span);
      case 2:
        put(2, base(2) + d.dy * k * 10 / span);
      case 3:
        put(3, base(3) + d.dy * k * 100 / (a.height * .2));
    }
  }

  @override
  void up(int z) {
    if (z == 2 && _moved < 3) _vy += 140;
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), cx = _cx(a), xd = _xd(a), xt = _xt(a), my = _massY(a), hs = _hs(a), ceil = a.top + 1;
    final k = v(0), cdamp = v(1), sc = _scale(a);
    // ceiling
    c.drawLine(Offset(cx - 12, ceil), Offset(xd + 8, ceil), weLine(N.g56));
    // the coil: more turns, narrower = stiffer
    final top = ceil + 3, bot = my - hs;
    if (bot - top > 6) {
      final turns = (5 + k / 500 * 8).round(), w = weLerp(9, 3.5, math.sqrt(k / 500)) * math.min(1.2, sc + .2);
      final coil = Path()..moveTo(cx, top);
      for (var i = 0; i < turns * 2; i++) {
        final y = weLerp(top, bot, (i + 1) / (turns * 2));
        coil.lineTo(cx + (i.isEven ? w : -w), y - (bot - top) / (turns * 4));
      }
      coil.lineTo(cx, bot);
      c.drawPath(coil, weLine(slot(0), on(0) ? 1.5 : 1));
    }
    // the dashpot: body fixed to the ceiling, piston rod to the weight; the fill is the damping
    final cylB = ceil + (_yMass0(a) - hs - ceil) * .55;
    final body = Rect.fromLTRB(xd - 4, ceil, xd + 4, cylB);
    final plate = weClamp(cylB - 5 + (my - _yMass0(a)) * .5, ceil + 3, cylB - 1);
    c.drawRect(body, weLine(slot(1), on(1) ? 1.5 : 1));
    c.drawLine(Offset(xd - 3, plate), Offset(xd + 3, plate), weLine(slot(1)));
    final nFill = 1 + (cdamp / 50 * 5).round();
    for (var i = 0; i < nFill; i++) {
      final y = plate + 2.5 + i * 2.2;
      if (y < cylB - 1) c.drawLine(Offset(xd - 2.5, y), Offset(xd + 2.5, y), weLine(slot(1, .5)));
    }
    c.drawLine(Offset(xd, plate), Offset(xd, my), weLine(N.g44));
    c.drawLine(Offset(xd, my), Offset(cx + hs, my), weLine(N.g44));
    // the weight
    final blk = Rect.fromCenter(center: Offset(cx, my), width: hs * 2, height: hs * 2);
    c.drawRect(blk, weFill(N.g13));
    c.drawRect(blk, weLine(slot(2, 1), on(2) ? 1.5 : 1));
    c.drawCircle(blk.center, 1, weFill(slot(2)));
    // the ink trail to the right, and the horizon the swing settles to
    final yRest = _yMass0(a) + _restPx(a);
    weDash(c, Offset(xt, yRest), Offset(a.right, yRest), weLine(slot(3, .6)), on: 3, off: 3);
    c.drawLine(Offset(a.right - 4, yRest - 3), Offset(a.right - 4, yRest + 3), weLine(slot(3, 1)));
    if (on(3)) c.drawLine(Offset(xt, yRest), Offset(a.right, yRest), weLine(slot(3), 1.5));
    weDash(c, Offset(xd + 5, my), Offset(xt, my), weLine(N.g26), on: 1, off: 2);
    final tr = Path();
    for (var i = 0; i < _hist.length; i++) {
      final x = xt + _age[i] * 70, y = _hist[i];
      i == 0 ? tr.moveTo(x, y) : tr.lineTo(x, y);
    }
    c.drawPath(tr, weLine(N.g56.withValues(alpha: .8)));
  }
}

// ---- 27. Wiggle: a stray dot in a dust cloud --------------------------------------------------------------------------------------------
// A dot wanders round a peg inside a cloud of dust; the leash from the peg kinks once for every octave. A blue = Frequency (the dot: one ring leaves it per beat),
// B green = Amplitude (the dust cloud: pull its edge out), C white = Octaves (the peg: the kinks in the leash), D orange = Seed (a pebble whose outline changes).

const wiggleSpecs = [
  WSpec('freq', 'Frequency', 0, 20, 2, unit: 'Hz', digits: 1),
  WSpec('amp', 'Amplitude', 0, 200, 20, unit: 'px', digits: 0),
  WSpec('oct', 'Octaves', 1, 6, 1, digits: 0, integer: true),
  WSpec('seed', 'Seed', 1, 999, 7, digits: 0, integer: true, drive: false),
];

WorldDef wiggleDef() => WorldDef(
      name: 'Wiggle',
      specs: wiggleSpecs,
      make: WiggleWorld.new,
      silhouette: 'a stray dot in dust',
      rowH: 108,
      above: ('Position', '960, 540'),
      below: ('Rotation', '0°'),
    );

class WiggleWorld extends WeWorld {
  WiggleWorld(super.doc);
  double _ph = 0, _acc = 0, _lastPh = 0;
  Offset _dot = Offset.zero;
  final _trail = <Offset>[];
  final _rings = <(Offset, double)>[];

  double _dim(Rect a) => math.min(a.width, a.height);
  // the dust is an ellipse that fills the box at full amplitude; display scale t = (amp/200)^.22 so the default already fills about half
  double _t() => math.pow(weClamp(v(1) / 200, 0, 1), .22).toDouble();
  double _rxm(Rect a) => a.width * .44;
  double _rym(Rect a) => a.height * .42;
  double _r(Rect a) => _t() * _rxm(a);
  Offset _centre(Rect a) => Offset(a.center.dx - 6, a.center.dy);
  Offset _pebble(Rect a) => Offset(a.right - 12, a.bottom - 9);

  /// The leash: partial sums of one noise offset per octave.
  List<Offset> _leash(Rect a) {
    final n = v(2).round(), r = _r(a), seed = v(3);
    var norm = 0.0;
    for (var o = 0; o < n; o++) {
      norm += math.pow(.5, o);
    }
    final parts = <Offset>[];
    var sum = Offset.zero;
    for (var o = 0; o < n; o++) {
      final u = _ph * 3.0 * math.pow(2, o), g = math.pow(.5, o) / norm * 1.7;
      final off = Offset(weNoise(u, seed * 1.37 + o * 3.1) * g, weNoise(u, seed * 1.37 + o * 3.1 + 5.3) * g);
      parts.add(off);
      sum += off;
    }
    final len = sum.distance, sc = len > 1 ? 1 / len : 1.0;
    final c = _centre(a), pts = <Offset>[c], k = _rym(a) / _rxm(a);
    var q = c;
    for (final p in parts) {
      q += Offset(p.dx, p.dy * k) * (r * sc);
      pts.add(q);
    }
    return pts;
  }

  @override
  void step(double dt, Size s) {
    final a = art(s);
    _ph += dt * v(0);
    if (_ph.floor() != _lastPh.floor() && _rings.length < 14) _rings.add((_dot, 0));
    _lastPh = _ph;
    final l = _leash(a);
    _dot = l.last;
    _trail.add(_dot);
    if (_trail.length > 36) _trail.removeAt(0);
    for (var i = 0; i < _rings.length; i++) {
      _rings[i] = (_rings[i].$1, _rings[i].$2 + dt);
    }
    _rings.removeWhere((r) => r.$2 > .7);
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final a = art(s), c = _centre(a), dc = (p - c).distance, dd = (p - _dot).distance;
    if ((p - _pebble(a)).distance <= 16) return 3;
    if (dc <= 13 && dc <= dd) return 2;
    if (dd <= 15) return 0;
    final q = _norm(a, p);
    if (dc >= 13 && q <= _t() * 1.12 + 14 / _rxm(a)) return 1;
    return null;
  }

  /// Distance of [p] from the centre in units of the full-amplitude ellipse (1 = the box edge).
  double _norm(Rect a, Offset p) {
    final c = _centre(a);
    return math.sqrt(math.pow((p.dx - c.dx) / _rxm(a), 2) + math.pow((p.dy - c.dy) / _rym(a), 2));
  }

  @override
  void down(int z, Offset p, Size s) => _acc = switch (z) { 1 => _t(), 2 => base(2), 3 => base(3), _ => 0 };

  @override
  void move(int z, Offset p, Offset d, bool fine, Size s) {
    final k = fine ? .1 : 1.0, a = art(s);
    switch (z) {
      case 0:
        put(0, base(0) + d.dx * k * 20 / (a.width * .5));
      case 1:
        // radial travel in box units, at .6 gain, then the display curve inverted: no short drag can jump the whole range
        _acc = weClamp(_acc + (_norm(a, p) - _norm(a, p - d)) * .6 * k, 0, 1);
        put(1, 200 * math.pow(_acc, 1 / .22).toDouble());
      case 2:
        _acc += -d.dy * k * 5 / (a.height * .5);
        put(2, _acc);
      case 3:
        _acc += d.dx * k / 6;
        put(3, _acc);
    }
  }

  @override
  void paint(Canvas c, Size s) {
    final a = art(s), cen = _centre(a), r = _r(a), seed = v(3).round();
    // the dust: its extent is the amplitude
    for (var i = 0; i < 60; i++) {
      final ang = 2 * math.pi * weHash(seed, i * 2), rad = 1.12 * math.sqrt(weHash(seed, i * 2 + 1));
      final sh = math.sin(doc.t * .9 + i) * .8;
      c.drawCircle(cen + Offset(math.cos(ang) * (rad * r + sh), math.sin(ang) * (rad * r * _rym(a) / _rxm(a) + sh)), 1.1, weFill(slot(1, .45)));
    }
    if (on(1)) c.drawOval(Rect.fromCenter(center: cen, width: 2 * (r * 1.12 + 2), height: 2 * (r * 1.12 * _rym(a) / _rxm(a) + 2)), weLine(slot(1).withValues(alpha: .6)));
    // the beat: one ring leaves the dot per cycle
    for (final rg in _rings) {
      final t = rg.$2 / .7;
      c.drawCircle(rg.$1, 2 + t * .12 * _dim(a) * 1.2, weLine(slot(0, (1 - t) * .55)));
    }
    // the trail
    for (var i = 1; i < _trail.length; i++) {
      c.drawLine(_trail[i - 1], _trail[i], weLine(N.g56.withValues(alpha: i / _trail.length * .6)));
    }
    // the leash and the peg
    final l = _leash(a);
    final leash = Path()..moveTo(l.first.dx, l.first.dy);
    for (final q in l.skip(1)) {
      leash.lineTo(q.dx, q.dy);
    }
    c.drawPath(leash, weLine(slot(2, .55)));
    if (l.length > 2) {
      for (final q in l.skip(1).take(l.length - 2)) {
        c.drawCircle(q, 1, weFill(slot(2, .8)));
      }
    }
    c.drawCircle(cen, 3, weLine(slot(2, 1)));
    ring(c, cen, 10, 2);
    // the dot
    c.drawCircle(_dot, 2.6, weFill(N.g95));
    ring(c, _dot, 8, 0);
    // the pebble: its outline is the seed
    final pb = _pebble(a), blob = Path();
    for (var i = 0; i < 8; i++) {
      final ang = 2 * math.pi * i / 8, rad = 3.2 + 2.6 * weHash(seed, 300 + i), q = pb + Offset(math.cos(ang), math.sin(ang)) * rad;
      i == 0 ? blob.moveTo(q.dx, q.dy) : blob.lineTo(q.dx, q.dy);
    }
    blob.close();
    c.drawPath(blob, weLine(slot(3, 1)));
    ring(c, pb, 9, 3);
  }
}
