part of 'pheno_c.dart';

// Warp: a rubber grid. Pull the middle node up (bulge) or down (pinch) for strength; take hold of the fabric anywhere else and drag out or in to widen or narrow the stretched area.
class _WarpToy extends _Toy {
  _WarpToy()
      : super(PcDoc(const [
          PcParam('strength', 'Strength', -100, 100, 35),
          PcParam('radius', 'Radius', 10, 200, 90, unit: 'px'),
        ])) {
    _st = _Sp(doc['strength'] / 100);
    _ra = _Sp(doc['radius'] / 200);
  }
  late _Sp _st, _ra;
  final _ox = _Sp(0), _oy = _Sp(0);
  Offset _pull = Offset.zero;
  double _prevD = 0;
  String _mode = '';

  Offset _c(Size s) => Offset(s.width * .5, s.height * .5);

  @override
  String? zoneAt(Offset p, Size s) => (p - _c(s)).distance < 20 ? 'strength' : 'radius';

  @override
  List<String> readout() => ['s ${doc['strength'] >= 0 ? '+' : ''}${doc.fmt('strength')}', 'r ${doc.fmt('radius')}'];

  @override
  bool get busy => super.busy || !_st.at(doc['strength'] / 100) || !_ra.at(doc['radius'] / 200) || !_ox.at(_pull.dx, .02) || !_oy.at(_pull.dy, .02);

  @override
  void grab(Offset p, Size s, String zone) {
    _mode = zone;
    _prevD = (p - _c(s)).distance;
  }

  @override
  void drag(Offset p, Offset d, Size s, bool fine) {
    final k = fine ? .1 : 1.0;
    if (_mode == 'strength') {
      doc.set('strength', doc['strength'] + (-d.dy) / (s.height * .45) * 100 * k);
      final o = (p - _c(s)) * .35;
      _pull = Offset(o.dx.clamp(-14.0, 14.0).toDouble(), o.dy.clamp(-14.0, 14.0).toDouble());
    } else {
      final dd = (p - _c(s)).distance;
      doc.set('radius', doc['radius'] + (dd - _prevD) / (s.height * .9) * 200 * k);
      _prevD = dd;
    }
  }

  @override
  void release(Offset p, Size s) {
    _pull = Offset.zero;
    finish();
  }

  @override
  void stopMotion() => _pull = Offset.zero;

  @override
  void step(double dt) {
    _st.to(doc['strength'] / 100, dt, k: 300, d: 16);
    _ra.to(doc['radius'] / 200, dt, k: 240, d: 20);
    _ox.to(_pull.dx, dt, k: 320, d: 14);
    _oy.to(_pull.dy, dt, k: 320, d: 14);
  }

  @override
  void paint(Canvas z, Size s) {
    final c = _c(s), rs = math.max(6.0, _ra.x * s.height * .9), sd = _st.x * .9;
    final off = Offset(_ox.x, _oy.x);
    Offset warp(Offset p) {
      final d = p - c;
      final t = d.distance / rs;
      if (t >= 1) return p;
      final f = (1 - t * t) * (1 - t * t);
      return p + d * (sd * f) + off * f;
    }

    final g = s.height / 8;
    final lines = Paint()..color = _a(N.g44, .95)..style = PaintingStyle.stroke..strokeWidth = 1;
    final nx = (s.width / 2 / g).ceil() + 1, ny = (s.height / 2 / g).ceil() + 1;
    for (var i = -nx; i <= nx; i++) {
      final path = Path();
      var first = true;
      for (var y = -g; y <= s.height + g; y += 3) {
        final q = warp(Offset(c.dx + i * g, y));
        first ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
        first = false;
      }
      z.drawPath(path, lines);
    }
    for (var j = -ny; j <= ny; j++) {
      final path = Path();
      var first = true;
      for (var x = -g; x <= s.width + g; x += 3) {
        final q = warp(Offset(x, c.dy + j * g));
        first ? path.moveTo(q.dx, q.dy) : path.lineTo(q.dx, q.dy);
        first = false;
      }
      z.drawPath(path, lines);
    }
    for (var i = -nx; i <= nx; i++) {
      for (var j = -ny; j <= ny; j++) {
        final p0 = Offset(c.dx + i * g, c.dy + j * g);
        final q = warp(p0);
        final near = (1 - ((p0 - c).distance / rs)).clamp(0.0, 1.0);
        z.drawCircle(q, 1.0 + near * .7, _fl(_a(N.g76, .45 + near * .45)));
      }
    }

    final ringHot = hover == 'radius' || (dragging && _mode == 'radius');
    if (ringHot) {
      final col = dragging ? _a(_act, .9) : _a(_hot, .55);
      for (var a = 0.0; a < math.pi * 2; a += math.pi / 12) {
        z.drawArc(Rect.fromCircle(center: c, radius: rs), a, math.pi / 24, false, _ln(col));
      }
    }
    final nodeHot = hover == 'strength' || (dragging && _mode == 'strength');
    final node = warp(c);
    z.drawCircle(node, 4.5, _ln(nodeHot ? (dragging ? _act : _a(_hot, .85)) : _a(N.g63, .5)));
    z.drawCircle(node, 1.4, _fl(nodeHot ? _hot : _a(N.g91, .7)));
  }
}
