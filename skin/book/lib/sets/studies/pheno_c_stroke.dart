part of 'pheno_c.dart';

// Stroke: one ink stroke, pressed by a nib (calm grey outline, tapered, a wet highlight riding it). Press and drag down on it and the nib bears down (width); tap it to cut a dash (first tap ends the first dash, a tap further along ends the first gap);
// tap the tip to turn the end butt, round, square.
class _Fx {
  _Fx(this.p, this.ang, this.t0, this.cut);
  final Offset p;
  final double ang, t0;
  final bool cut;
}

class _StrokeToy extends _Toy {
  _StrokeToy()
      : super(PcDoc(const [
          PcParam('width', 'Width', 1, 64, 12, unit: 'px'),
          PcParam('dash', 'Dash', 0, 160, 0),
          PcParam('gap', 'Gap', 0, 160, 24),
          PcParam('cap', 'Cap', 0, 2, 1, names: ['butt', 'round', 'square']),
        ])) {
    _w = _Sp(doc['width']);
  }
  late _Sp _w;
  final _pop = _Sp(0);
  final List<_Fx> _fx = [];
  String _mode = '';
  Offset _p0 = Offset.zero;
  double _travel = 0;

  Path _path(Size s) => Path()
    ..moveTo(s.width * .10, s.height * .70)
    ..cubicTo(s.width * .36, s.height * .12, s.width * .58, s.height * .98, s.width * .88, s.height * .32);

  double _k(Size s) => s.width / 320;

  (double, Offset, double) _nearest(Offset p, Size s) {
    final m = _path(s).computeMetrics().first;
    var best = 1e9, bl = 0.0;
    var bp = Offset.zero;
    var ba = 0.0;
    for (var i = 0; i <= 80; i++) {
      final l = m.length * i / 80;
      final t = m.getTangentForOffset(l)!;
      final dd = (t.position - p).distance;
      if (dd < best) {
        best = dd;
        bl = l;
        bp = t.position;
        ba = t.angle;
      }
    }
    _lastAngle = ba;
    return (best, bp, bl);
  }

  double _lastAngle = 0;

  @override
  String? zoneAt(Offset p, Size s) {
    final m = _path(s).computeMetrics().first;
    final end = m.getTangentForOffset(m.length)!.position;
    if ((p - end).distance < 24) return 'tip';
    final (dd, _, _) = _nearest(p, s);
    return dd < math.max(_w.x * _vis * _k(s) / 2 + 12, 16) ? 'body' : null;
  }

  @override
  List<String> readout() => [doc.fmt('width'), doc['dash'] < 1 ? 'solid' : '${doc['dash'].round()}/${doc['gap'].round()}', doc.fmt('cap')];

  @override
  bool get busy => true; // idle life: the wet highlight keeps riding

  @override
  void grab(Offset p, Size s, String zone) {
    _mode = zone;
    _p0 = p;
    _travel = 0;
  }

  @override
  void drag(Offset p, Offset d, Size s, bool fine) {
    _travel += d.distance;
    _p0 = p;
    if (_mode == 'body' && _travel > 4) {
      final w = doc['width'];
      final soft = (w < 3 && d.dy < 0) || (w > 40 && d.dy > 0) ? .35 : 1.0;
      doc.set('width', w * math.exp(d.dy / (s.height * .6) * soft * (fine ? .1 : 1)));
    }
  }

  @override
  void release(Offset p, Size s) {
    if (_travel <= 4) {
      if (_mode == 'tip') {
        doc.set('cap', (doc['cap'] + 1) % 3);
        _pop.x = .14;
        final m = _path(s).computeMetrics().first;
        _fx.add(_Fx(m.getTangentForOffset(m.length)!.position, 0, now, false));
      } else {
        final (_, pos, l) = _nearest(p, s);
        final v = l / _k(s);
        if (doc['dash'] < 1 || v <= doc['dash'] + 1) {
          doc.set('dash', math.max(2, v));
          if (doc['gap'] < 2) doc.set('gap', 24);
        } else {
          doc.set('gap', math.max(2, v - doc['dash']));
        }
        _fx.add(_Fx(pos, _lastAngle, now, true));
      }
    }
    finish();
  }

  @override
  void step(double dt) {
    _w.to(doc['width'] * (pressed ? 1.08 : 1), dt, k: 300, d: 15);
    _pop.to(0, dt, k: 300, d: 11);
    _fx.removeWhere((f) => now - f.t0 > .4);
  }

  // Ink pressed by a nib: width swells mid-stroke and tapers at the ends; the drawn width is a calm fraction of the value.
  static const _vis = .55;

  List<(double, double)> _segs(double len, double k) {
    final dash = doc['dash'] * k, gap = math.max(1.0, doc['gap'] * k);
    if (dash < 1) return [(0.0, len)];
    return [for (var p = 0.0; p < len; p += dash + gap) (p, math.min(len, p + dash))];
  }

  @override
  void paint(Canvas z, Size s) {
    final k = _k(s), path = _path(s), m = path.computeMetrics().first, len = m.length;
    final w = math.max(1.0, _w.x * k * (1 + _pop.x)) * _vis * (1 + .03 * math.sin(now * 1.7));
    final capI = doc['cap'].round();
    final bodyHot = hover == 'body' || (dragging && _mode == 'body');
    final tipHot = hover == 'tip' || (dragging && _mode == 'tip');
    final rim = bodyHot ? (dragging ? _act : _a(_hot, .8)) : N.g76;
    final fill = _fl(_a(N.g63, .30));
    final edge = _ln(rim, 1);
    final spark = _fl(_a(N.g76, .5));
    final shine = _ln(_a(N.g100, .38), 1);
    final end = m.getTangentForOffset(len)!;

    for (final (a0, b0) in _segs(len, k)) {
      final ext = capI == 2 ? w * .5 : 0.0;
      final a = math.max(0.0, a0 - ext), b = math.min(len, b0 + ext);
      if (b - a < 1) continue;
      final n = math.max(8, ((b - a) / 3).ceil());
      final lp = <Offset>[], rp = <Offset>[], hp = <Offset>[];
      for (var i = 0; i <= n; i++) {
        final u = i / n, l = a + (b - a) * u;
        final t = m.getTangentForOffset(l)!;
        final hw = w * (.40 + .60 * math.pow(math.sin(math.pi * u), .6)) * (1 + .05 * _noise1(l * .09)) / 2;
        final nv = Offset.fromDirection(t.angle + math.pi / 2, 1);
        lp.add(t.position + nv * hw);
        rp.add(t.position - nv * hw);
        if (u > .12 && u < .88) hp.add(t.position - nv * hw * .35);
      }
      final poly = Path()..moveTo(lp.first.dx, lp.first.dy);
      for (final q in lp.skip(1)) {
        poly.lineTo(q.dx, q.dy);
      }
      for (final q in rp.reversed) {
        poly.lineTo(q.dx, q.dy);
      }
      poly.close();
      z.drawPath(poly, fill);
      z.drawPath(poly, edge);
      if (capI == 1) {
        for (final e in [(a, a0 == a), (b, b0 == b)]) {
          final t = m.getTangentForOffset(e.$1)!;
          z.drawCircle(t.position, w * .2, fill);
          z.drawCircle(t.position, w * .2, edge);
        }
      }
      if (hp.length > 2) z.drawPath(Path()..addPolygon(hp, false), shine);
      // grain: a few ink specks beside the edge, steady per position
      final cnt = ((b - a) / 14).floor();
      for (var i = 0; i < cnt; i++) {
        final l = a + (b - a) * _hash(i * 7 + a.round());
        final t = m.getTangentForOffset(l)!;
        final side = _hash(i * 13 + 5) > .5 ? 1.0 : -1.0;
        final off = w * (.62 + .5 * _hash(i * 3 + 1));
        z.drawCircle(t.position + Offset.fromDirection(t.angle + math.pi / 2, off * side), .6 + .5 * _hash(i), spark);
      }
    }

    // idle life: a wet highlight rides the stroke slowly and a short trail fades behind it
    final ph = (now / 7) % 1.0, eased = .5 - .5 * math.cos(ph * 2 * math.pi);
    for (var j = 0; j < 4; j++) {
      final l = len * (eased - j * .012).clamp(0.0, 1.0);
      final t = m.getTangentForOffset(l)!;
      z.drawCircle(t.position, 1.8 - j * .35, _fl(_a(N.g100, (.8 - j * .2) * math.sin(ph * math.pi))));
    }

    if (tipHot) z.drawCircle(end.position, w / 2 + 7, _ln(dragging ? _act : _a(_hot, .7)));
    if (pressed && _mode == 'body') z.drawCircle(_p0, w / 2 + 3, _ln(_a(N.g95, .55)));
    for (final f in _fx) {
      final t = ((now - f.t0) / .4).clamp(0.0, 1.0);
      final e = Curves.easeOut.transform(t);
      if (f.cut) {
        final nn = Offset.fromDirection(f.ang + math.pi / 2, (w / 2 + 5) * (1 + .3 * e));
        z.drawLine(f.p - nn, f.p + nn, _ln(_a(N.g100, 1 - t)));
      } else {
        z.drawCircle(f.p, w / 2 + 4 + 14 * e, _ln(_a(N.g95, (1 - t) * .8)));
      }
    }
  }
}
