part of 'pheno_c.dart';

// Extrude: a hex footprint standing on the ground. Pull the top face up and it grows into a block (depth); take hold of its rim and push in to cut the chamfer (bevel).
class _ExtrudeToy extends _Toy {
  _ExtrudeToy()
      : super(PcDoc(const [
          PcParam('depth', 'Depth', 0, 300, 60, unit: 'px'),
          PcParam('bevel', 'Bevel', 0, 60, 8, unit: 'px'),
        ])) {
    _dp = _Sp(doc['depth']);
    _bv = doc['bevel'];
  }
  late _Sp _dp;
  late double _bv;
  String _mode = '';

  static const _r = 100.0;
  double _s(Size z) => math.min(z.height / 232, z.width * .0028);
  double _pps(Size z) => .35 * _s(z);
  double _cx(Size z) => z.width * .5;
  double _by(Size z) => z.height - 43 * _s(z) - 5;

  @override
  String? zoneAt(Offset p, Size z) {
    final s = _s(z), d = _dp.x, bv = math.min(_bv, d);
    final ty = _by(z) - d * _pps(z), rt = _r - bv;
    final qx = (p.dx - _cx(z)) / (rt * s), qy = (p.dy - ty) / (rt * s * .5);
    final n = math.sqrt(qx * qx + qy * qy);
    if (n > .78 && n < 1.4) return 'rim';
    if (n <= .78) return 'face';
    if ((p.dx - _cx(z)).abs() < _r * s * 1.05 && p.dy > ty && p.dy < _by(z) + 43 * s + 6) return 'face';
    return null;
  }

  @override
  List<String> readout() => ['d ${doc.fmt('depth')}', 'b ${doc.fmt('bevel')}'];

  @override
  bool get busy => super.busy || !_dp.at(doc['depth'], .05) || (_bv - doc['bevel']).abs() > .05;

  @override
  void grab(Offset p, Size s, String zone) => _mode = zone;

  @override
  void drag(Offset p, Offset d, Size s, bool fine) {
    final k = fine ? .1 : 1.0;
    if (_mode == 'face') {
      doc.set('depth', doc['depth'] + (-d.dy) / _pps(s) * k);
    } else {
      final cx = _cx(s);
      final prev = (p.dx - d.dx - cx).abs(), cur = (p.dx - cx).abs();
      doc.set('bevel', doc['bevel'] + (prev - cur) / _s(s) * k);
    }
  }

  @override
  void step(double dt) {
    _dp.to(doc['depth'], dt, k: 260, d: 16);
    _bv += (doc['bevel'] - _bv) * math.min(1, dt * 16);
  }

  @override
  void paint(Canvas z, Size sz) {
    final s = _s(sz), pps = _pps(sz), cx = _cx(sz), by = _by(sz);
    final d = math.max(0.0, _dp.x), bv = math.min(_bv, d);
    List<Offset> ring(double r, double h) => [
          for (var k = 0; k < 6; k++) Offset(cx + r * math.cos(k * math.pi / 3) * s, by + r * math.sin(k * math.pi / 3) * .5 * s - h * pps),
        ];
    Path poly(List<Offset> v) => Path()..addPolygon(v, true);

    final base = ring(_r, 0);
    z.drawOval(Rect.fromCenter(center: Offset(cx, by + 3), width: _r * 2.1 * s, height: _r * 1.1 * s), _fl(_a(N.g00, .35))..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3));
    z.drawPath(poly(base), _fl(N.g15));
    z.drawPath(poly(base), _ln(_a(N.g56, .8)));

    final wallTop = ring(_r, d - bv), chamBot = wallTop, chamTop = ring(_r - bv, d);
    final faceHot = hover == 'face' || (dragging && _mode == 'face');
    final rimHot = hover == 'rim' || (dragging && _mode == 'rim');
    final shade = [N.g26, N.g20, N.g15];
    if (d - bv > .5) {
      for (var k = 0; k < 3; k++) {
        final k2 = (k + 1) % 6;
        z.drawPath(Path()..addPolygon([base[k], base[k2], wallTop[k2], wallTop[k]], true), _fl(shade[k]));
        final len = (base[k2] - base[k]).distance;
        final n = math.max(2, (len / 5).floor());
        for (var i = 1; i < n; i++) {
          final t = i / n;
          z.drawLine(Offset.lerp(base[k], base[k2], t)!, Offset.lerp(wallTop[k], wallTop[k2], t)!, _ln(_a(N.g44, .35)));
        }
        z.drawLine(base[k], wallTop[k], _ln(_a(N.g56, .7)));
      }
      z.drawLine(base[3], wallTop[3], _ln(_a(N.g56, .7)));
      z.drawLine(base[0], wallTop[0], _ln(_a(N.g56, .7)));
      z.drawLine(base[0], base[1], _ln(_a(N.g56, .7)));
      z.drawLine(base[1], base[2], _ln(_a(N.g56, .7)));
      z.drawLine(base[2], base[3], _ln(_a(N.g56, .7)));
    }
    if (bv > .3) {
      for (var k = 0; k < 3; k++) {
        final k2 = (k + 1) % 6;
        z.drawPath(Path()..addPolygon([chamBot[k], chamBot[k2], chamTop[k2], chamTop[k]], true), _fl(N.g38));
      }
      final e = rimHot ? (dragging ? _act : _hot) : _a(N.g76, .8);
      z.drawPath(poly(chamBot).shift(Offset.zero), _ln(rimHot ? e : _a(N.g63, .7)));
    }
    z.drawPath(poly(chamTop), _fl(faceHot ? N.g38 : N.g26));
    z.drawPath(poly(chamTop), _ln(faceHot ? (dragging ? _act : _hot) : (rimHot ? (dragging ? _act : _hot) : N.g91)));
    z.drawPath(poly(ring((_r - bv) * .5, d)), _ln(_a(N.g56, .45)));
  }
}
