part of 'pheno_a.dart';

/// 4. Grid: columns + rows + gap = a living lattice. Stretch the first cell by its corner (it re-cuts the frame into columns and rows), pinch the lattice apart (gap).
class _GridSpec extends PhenoSpec {
  const _GridSpec();
  @override
  String get name => '4 Grid';
  @override
  String get word => 'grid';
  @override
  String get caption => 'Grid: columns + rows + gap. Silhouette: a tiled frame of small rounded cells; the first cell has a corner bracket.';
  @override
  double get rowHeight => 96;
  @override
  List<PhenoParam> get params => const [
        PhenoParam('cols', 'Columns', 1, 16, 4, unit: 'c'),
        PhenoParam('rows', 'Rows', 1, 12, 3, unit: 'r'),
        PhenoParam('gap', 'Gap', 0, 40, 8, unit: 'px'),
      ];
  @override
  PhenoSim newSim() => _GridSim();

  static const compW = 640.0;
  /// The frame the lattice fills: the whole drawing area is the composition (same px scale on both axes).
  static Rect lat(Size s) {
    return _cr(s);
  }

  static double sx(Size s) => lat(s).width / compW;

  /// Bottom-right corner of the first cell, for the given (possibly fractional) values.
  static Offset corner(Size s, double cols, double rows, double gap) {
    final l = lat(s), g = gap * sx(s);
    return l.topLeft + Offset((l.width + g) / cols - g, (l.height + g) / rows - g);
  }

  @override
  String? zoneAt(Size s, Offset p, PhenoValues v) {
    if ((p - corner(s, v['cols'], v['rows'], v['gap'])).distance < 14) return 'corner';
    if (lat(s).inflate(8).contains(p)) return 'body';
    return null;
  }

  @override
  Map<String, double> drag(String zone, Size s, Offset p0, Offset p, PhenoValues v0) {
    final l = lat(s);
    if (zone == 'corner') {
      // The first cell's corner follows the pointer; the frame is re-cut into as many such cells as fit.
      final q = p - p0 + corner(s, v0['cols'], v0['rows'], v0['gap']) - l.topLeft, g = v0['gap'] * sx(s);
      return {
        'cols': ((l.width + g) / math.max(q.dx + g, 1)).round().toDouble(),
        'rows': ((l.height + g) / math.max(q.dy + g, 1)).round().toDouble(),
      };
    }
    final c = l.center;
    // Gain: one pixel of pinch is about one gap unit (sx would make it x3 too fast).
    return {'gap': v0['gap'] + ((p - c).distance - (p0 - c).distance) * .7};
  }

  @override
  List<String> zoneParams(String zone) => zone == 'corner' ? ['cols', 'rows'] : ['gap'];
}

class _GridSim extends PhenoSim {
  final cols = _Sp(), rows = _Sp(), gap = _Sp(), wave = _Sp();
  double phase = 0;
  bool first = true;

  @override
  bool step(double dt, Size s, PhenoCtx c) {
    if (first) {
      cols.x = c.v['cols'];
      rows.x = c.v['rows'];
      gap.x = c.v['gap'];
      first = false;
    }
    if (c.live) phase += dt;
    cols.step(c.v['cols'], dt, k: 200, z: .6);
    rows.step(c.v['rows'], dt, k: 200, z: .6);
    gap.step(c.v['gap'], dt, k: 200, z: .6);
    wave.step(c.live ? 1 : 0, dt, k: 60, z: 1);
    return true; // idle breathing: the ticker never sleeps for this one
  }

  @override
  void paint(Canvas cv, Size s, PhenoCtx c) {
    final l = _GridSpec.lat(s), g = math.max(0.0, gap.x) * _GridSpec.sx(s), nc = math.max(1.0, cols.x), nr = math.max(1.0, rows.x);
    final px = (l.width + g) / nc, py = (l.height + g) / nr, cw = px - g, ch = py - g;
    final bm = _mark(c, 'body'), ptr = c.ptr, lv = wave.x.clamp(0.0, 1.0);
    cv.save();
    cv.clipRect(l.inflate(.5));
    for (var j = 0; j < nr.ceil(); j++) {
      for (var i = 0; i < nc.ceil(); i++) {
        final a = math.min(1.0, math.min(nc - i, nr - j));
        // Always breathing: a slow diagonal swell through the cells; stronger while touched.
        final w = .5 + .5 * math.sin(c.t * 1.2 - (i + j) * .7);
        var rect = Rect.fromLTWH(l.left + i * px, l.top + j * py, cw, ch);
        var k = 0.0;
        if (ptr != null) {
          // A rubber lattice: cells near the pointer lean away from it and swell a little.
          final d = rect.center - ptr, dd = d.distance;
          k = math.exp(-math.pow(dd / 42, 2)) * lv;
          if (dd > .5) rect = rect.shift(d / dd * 3.2 * k);
          rect = Rect.fromCenter(center: rect.center, width: rect.width * (1 + .14 * k), height: rect.height * (1 + .14 * k));
        }
        final r = RRect.fromRectAndRadius(rect, const Radius.circular(3));
        cv.drawRRect(r, _fill(Color.lerp(N.g13, N.g26, (.15 + .35 * w) + .4 * k)!.withValues(alpha: a)));
        final first = i == 0 && j == 0;
        final lc = first ? N.g76 : Color.lerp(N.g38, N.g56, (bm != null ? .5 : 0) + k * .5)!;
        cv.drawRRect(r.deflate(.5), _line(lc.withValues(alpha: a)));
        // A pip in each cell: a lattice of things, not a table of boxes.
        if (cw > 5 && ch > 5) cv.drawCircle(rect.center, .7 + .7 * w + .6 * k, _fill(N.g44.withValues(alpha: a * (.5 + .5 * w))));
      }
    }
    cv.restore();
    // The corner bracket of the first cell: where the lattice is stretched from. It pulses faintly at rest.
    final k = _GridSpec.corner(s, c.v['cols'], c.v['rows'], c.v['gap']), km = _mark(c, 'corner');
    final pulse = .55 + .45 * math.sin(c.t * 2.4);
    cv.drawPath(Path()..moveTo(k.dx - 6, k.dy)..lineTo(k.dx, k.dy)..lineTo(k.dx, k.dy - 6), _line(N.g95.withValues(alpha: .6 + .4 * pulse)));
    cv.drawCircle(k, 1.6 + .5 * pulse, _fill(N.g95));
    if (km != null) cv.drawCircle(k, 9, _line(km));
  }
}
