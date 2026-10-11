// World A 3 and 4: Scatter (seeds thrown on the ground, a click shakes them into another throw) and Stagger (a cascade of cards a playhead walks down).
import 'dart:math' as math;

import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import 'world_a_kit.dart';

// ---- 3. Scatter: seeds on the ground -------------------------------------------------------------------------------------------------------

const scatterSpecs = [
  WaSpec('sc.spread', 0, 200, 60, unit: '', digits: 0),
  WaSpec('sc.seed', 1, 999, 3, digits: 0, step: true),
  WaSpec('sc.dens', 8, 160, 60, digits: 0),
  WaSpec('sc.fall', 0, 1, .4),
];

class ScatterWorld extends WaWorld {
  ScatterWorld(super.doc);

  int? _lastSeed, _prevSeed;
  double _seedAt = -9;
  double _base = 0, _q0 = 0;
  Offset _vp0 = Offset.zero;

  @override
  List<List<String>> get zoneIds => const [['sc.spread'], ['sc.seed'], ['sc.dens'], ['sc.fall']];

  Offset _c(Size s) => art(s).center;
  double _half(Size s) => math.min(art(s).width, art(s).height) / 2;

  /// Spread is soft-limited (1 - e^(-v/37)): the default 60 already fills 80 % of the box, 200 reaches the frame and stops there.
  static const _k = 37.0;
  double _f(double spread) => 1 - math.exp(-spread / _k);
  double _fInv(double q) => -_k * math.log(1 - waClamp(q, 0, .995));

  /// The cloud is an ellipse that fills the drawing area (centred), radii at full spread.
  Offset _rad(Size s) => Offset(art(s).width / 2 - 6, art(s).height / 2 - 6);
  Offset _ell(Size s, double f, double th) => Offset(_rad(s).dx * f * math.cos(th), _rad(s).dy * f * math.sin(th));
  double _q(Offset v, Size s) => Offset(v.dx / _rad(s).dx, v.dy / _rad(s).dy).distance;
  int _n(WaCtx x) => (10 + (x.v('sc.dens') - 8) / 152 * 80).round();

  /// Pixel distance from [p] to the ellipse of fill [f] (along the ray from the centre).
  double _edge(Offset p, Size s, double f) {
    final v = p - _c(s), q = _q(v, s);
    if (q < 1e-6) return f * math.min(_rad(s).dx, _rad(s).dy);
    return (v - v / q * f).distance;
  }

  Offset _pos(int seed, int k, Offset rd) {
    final a = waRnd(seed * 131 + k * 7 + 1) * math.pi * 2, d = math.sqrt(waRnd(seed * 59 + k * 13 + 5));
    return Offset(math.cos(a) * d * rd.dx, math.sin(a) * d * rd.dy);
  }

  @override
  int? zoneAt(Offset p, Size s, WaCtx x) {
    final f = _f(x.v('sc.spread')), fall = x.v('sc.fall'), v = p - _c(s);
    final z = waPick([
      (0, _edge(p, s, f), 13),
      (3, _edge(p, s, f * (1 - fall)), 13),
      (2, v.distance, 14),
    ]);
    if (z != null) return z;
    return _q(v, s) <= f + .02 ? 1 : null;
  }

  @override
  Offset? anchor(int z, Offset ptr, Size s, WaCtx x) {
    final v = ptr - _c(s), q = _q(v, s);
    if (q < 1e-6) return null;
    return switch (z) { 0 => _c(s) + v / q * _f(x.v('sc.spread')), 3 => _c(s) + v / q * _f(x.v('sc.spread')) * (1 - x.v('sc.fall')), _ => null };
  }

  @override
  void begin(int z, Offset vp, Size s, WaCtx x) {
    _vp0 = vp;
    _q0 = _q(vp - _c(s), s);
    _base = switch (z) { 1 => doc['sc.seed'], 2 => doc['sc.dens'], _ => 0 };
  }

  @override
  void drag(int z, Offset vp, Size s, WaCtx x) {
    final c = _c(s), half = _half(s);
    switch (z) {
      case 0:
        set('sc.spread', _fInv(_q0 + (_q(vp - c, s) - _q0) * .6)); // gain .6: a short pull does not fling the rim to the frame
      case 1:
        set('sc.seed', _base + ((vp.dx - _vp0.dx) / 18).truncateToDouble());
      case 2:
        set('sc.dens', _base + (_vp0.dy - vp.dy) / (math.max(half, 60) * 2.4) * 152);
      default:
        final f = _f(doc['sc.spread']);
        if (f > .05) set('sc.fall', 1 - waClamp(_q(vp - c, s) / f, 0, 1));
    }
  }

  @override
  void tap(int z, Size s, WaCtx x) {
    if (z == 1) set('sc.seed', doc['sc.seed'] + 1);
  }

  @override
  List<String> readout(WaCtx x) => [x.doc.spec('sc.spread').fmt(x.v('sc.spread')), '${x.v('sc.seed').round()}', '${x.v('sc.dens').round()}', x.doc.spec('sc.fall').fmt(x.v('sc.fall'))];

  @override
  void paint(Canvas cv, Size s, WaCtx x) {
    final c = _c(s), f = _f(x.v('sc.spread')), fall = x.v('sc.fall'), n = _n(x), seed = x.v('sc.seed').round(), rd = _rad(s) * f;
    if (_lastSeed != null && _lastSeed != seed) {
      _prevSeed = _lastSeed;
      _seedAt = x.t;
    }
    _lastSeed = seed;
    final hop = waSmooth((x.t - _seedAt) / .18);
    for (var k = 0; k < n; k++) {
      var p = _pos(seed, k, rd);
      var lift = 0.0;
      if (_prevSeed != null && hop < 1) {
        p = Offset.lerp(_pos(_prevSeed!, k, rd), p, hop)!;
        lift = math.sin(math.pi * hop) * (2 + waRnd(k) * 3);
      }
      final rr = rd.dx < 1 ? 0.0 : Offset(p.dx / rd.dx, p.dy / rd.dy).distance;
      final sink = fall <= 0 ? 0.0 : waSmooth((rr - (1 - fall)) / fall);
      final j = Offset(waNoise(x.t * .8 + k, 1.0 + k * .1), waNoise(x.t * .8 + k, 2.0 + k * .1)) * 1.1;
      final col = Color.lerp(N.g76, N.g38, sink)!;
      final lucky = k % 9 == 4; // a few seeds carry the seed's slot colour: they are the ones that move when it changes
      final q = c + p + j - Offset(0, lift);
      cv.drawCircle(q, 1.5 - sink * .5, waFill(lucky ? x.col(1).withValues(alpha: 1 - sink * .6) : col));
    }
    // A: five blue ticks on the rim where the cloud ends
    final pa = x.mark(0, 1);
    for (var i = 0; i < 5; i++) {
      final th = .35 + i * math.pi * 2 / 5 + math.sin(x.t * .7) * .02;
      final u = Offset(math.cos(th), math.sin(th)), e = c + _ell(s, f, th);
      cv.drawLine(e - u * 3.5, e + u * 3.5, pa);
    }
    // D: four orange ticks where the cloud starts to sink
    final pd = x.mark(3, 1);
    for (var i = 0; i < 4; i++) {
      final th = .9 + i * math.pi / 2, u = Offset(math.cos(th), math.sin(th)), e = c + _ell(s, f * (1 - fall), th);
      cv.drawLine(e - u * 3, e + u * 3, pd);
    }
    // C: a faint white ring at the core, where the dots are packed
    cv.drawCircle(c, x.hl(2) ? 6.5 : 5, waStroke(x.col(2).withValues(alpha: x.hl(2) ? .9 : .4)));
    // B hover: the cloud's whole body brightens a little (the seed is the cloud itself)
    if (x.hl(1)) cv.drawOval(Rect.fromCenter(center: c, width: rd.dx * 2, height: rd.dy * 2), waStroke(x.col(1).withValues(alpha: .22)));
  }
}

// ---- 4. Stagger: a cascade of cards, a playhead walking down them -----------------------------------------------------------------------------

const staggerSpecs = [
  WaSpec('st.off', 0, 1, .08, unit: 's'),
  WaSpec('st.range', 1, 100, 100, unit: '%', digits: 0),
  WaSpec('st.dir', 0, 360, 30, unit: '°', digits: 0),
  WaSpec('st.ease', .5, 2.5, 1),
];

class StaggerWorld extends WaWorld {
  StaggerWorld(super.doc);

  @override
  List<List<String>> get zoneIds => const [['st.off'], ['st.range'], ['st.dir'], ['st.ease']];

  static const _n = 8;
  double? _along0;
  double _span0 = 0;

  @override
  void begin(int z, Offset vp, Size s, WaCtx x) => _along0 = null;

  Offset _c(Size s) => art(s).center;
  Size _card(Size s) {
    final a = art(s), w = math.min(a.height * .5, a.width * .2);
    return Size(w, w * .7);
  }

  Offset _dv(WaCtx x) {
    final th = x.v('st.dir') / 180 * math.pi;
    return Offset(math.cos(th), math.sin(th));
  }

  double _fit(Size s, WaCtx x) {
    final a = art(s), cs = _card(s), d = _dv(x);
    return math.min((a.width - cs.width) / math.max(d.dx.abs(), .001), (a.height - cs.height) / math.max(d.dy.abs(), .001)) * .98;
  }

  double _span(Size s, WaCtx x) => _fit(s, x) * math.pow(x.v('st.off'), .4);
  int _count(WaCtx x) => (x.v('st.range') * _n / 100 - 1e-9).ceil().clamp(1, _n);
  double _g(int j, double e) => math.pow(j / (_n - 1), e).toDouble();

  Offset _p(int k, Size s, WaCtx x) {
    final n = _count(x), span = _span(s, x);
    return _c(s) - _dv(x) * (span / 2) + _dv(x) * (span * _g(math.min(k, n - 1), x.v('st.ease')));
  }

  /// 0..1 position along the cascade of a point.
  double _frac(Offset p, Size s, WaCtx x) {
    final span = _span(s, x);
    if (span < 1) return 0;
    return ((p - _c(s)).dx * _dv(x).dx + (p - _c(s)).dy * _dv(x).dy + span / 2) / span;
  }

  /// The card each zone marks (A: the second, B: the last, C: the first, D: the fifth) and where its dot sits on that card.
  int _card0(int z, WaCtx x) => switch (z) { 0 => math.min(1, _count(x) - 1), 1 => _count(x) - 1, 2 => 0, _ => math.min(4, _count(x) - 1) };
  Offset _off(int z, Size s) {
    final cs = _card(s);
    return Offset(-cs.width / 2 + 7 + (z % 2) * 11, -cs.height / 2 + 7 + (z ~/ 2) * 11);
  }

  /// The visible dot of zone [z]: this is also the grab point (the hit target is a 13 px radius round it).
  Offset _dot(int z, Size s, WaCtx x) => _p(_card0(z, x), s, x) + _off(z, s);

  @override
  int? zoneAt(Offset p, Size s, WaCtx x) => waPick([for (var z = 0; z < 4; z++) (z, (p - _dot(z, s, x)).distance, 13.0)]);

  @override
  Offset? anchor(int z, Offset ptr, Size s, WaCtx x) => _dot(z, s, x);

  @override
  void drag(int z, Offset vp0, Size s, WaCtx x) {
    final vp = vp0 - _off(z, s); // the card centre that the held dot implies
    final fit = _fit(s, x), e = x.v('st.ease');
    switch (z) {
      case 0:
        // relative, gain 1/max(g1, .35): the second card sits 1/7 along the span, so a 1:1 pull would multiply every pixel by 7
        final d = _dv(x), along = (vp - _c(s)).dx * d.dx + (vp - _c(s)).dy * d.dy;
        if (fit < 1) return;
        if (_along0 == null) {
          _along0 = along;
          _span0 = _span(s, x);
        }
        final span = _span0 + (along - _along0!) / math.max(_g(1, e), .35);
        set('st.off', math.pow(waClamp(span / fit, 0, 1), 1 / .4).toDouble());
      case 1:
        final fr = _frac(vp, s, x);
        var best = 0, bd = 9.0;
        for (var k = 0; k < _n; k++) {
          final d = (_g(k, e) - fr).abs();
          if (d < bd) {
            bd = d;
            best = k;
          }
        }
        set('st.range', (best + 1) / _n * 100);
      case 2:
        final v = _c(s) - vp;
        if (v.distance > 6) set('st.dir', (math.atan2(v.dy, v.dx) * 180 / math.pi + 360) % 360);
      default:
        final u = waClamp(_frac(vp, s, x), .03, .97);
        set('st.ease', math.log(u) / math.log(4 / (_n - 1)));
    }
  }

  @override
  List<String> readout(WaCtx x) => ['${(x.v('st.off') * 1000).round()}ms', '${x.v('st.range').round()}', '${x.v('st.dir').round()}°', x.doc.spec('st.ease').fmt(x.v('st.ease'))];

  @override
  void paint(Canvas cv, Size s, WaCtx x) {
    final cs = _card(s), d = _dv(x), span = _span(s, x), c = _c(s), a = art(s);
    // the playhead walks down the cascade (2.6 s, then rests); a card that it has passed lifts
    final ph = (x.t % 3.2) / 2.6;
    final half = span / 2 + cs.width, sp = ph > 1 ? half : waLerp(-half, half, ph);
    final fade = ph > 1 ? 1.0 : 1 - waSmooth((ph - .88) / .12);
    final rect = a.inflate(6);
    final ctr = c + d * sp, nrm = Offset(-d.dy, d.dx), reach = cs.height * 1.5 + 16;
    if (ph <= 1) {
      cv.save();
      cv.clipRect(rect);
      cv.drawLine(ctr - nrm * reach, ctr + nrm * reach, waStroke(N.g56.withValues(alpha: .55)));
      cv.restore();
    }
    for (var k = 0; k < _n; k++) {
      final p = _p(k, s, x);
      final along = (p - c).dx * d.dx + (p - c).dy * d.dy;
      final started = ph > 1 ? 0.0 : waSmooth((sp - along) / (cs.width * .6)) * fade;
      final q = p - Offset(0, 4 * started);
      final r = RRect.fromRectAndRadius(Rect.fromCenter(center: q, width: cs.width, height: cs.height), const Radius.circular(2));
      cv.drawRRect(r, waFill(Color.lerp(N.g10, N.g15, started)!));
      cv.drawRRect(r, waStroke(Color.lerp(N.g44, N.g91, started)!));
    }
    // the four dots are the grab points: slot colour, a faint ring round the hovered one
    for (var z = 0; z < 4; z++) {
      final q = _dot(z, s, x);
      cv.drawCircle(q, x.hl(z) ? 2.6 : 2, waFill(x.col(z)));
      if (x.hl(z)) cv.drawCircle(q, 6, waStroke(x.col(z).withValues(alpha: .6)));
    }
  }
}
