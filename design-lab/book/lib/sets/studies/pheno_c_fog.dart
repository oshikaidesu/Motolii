part of 'pheno_c.dart';

// Opacity as fog: a small cat behind a pane. Wipe with the pointer and the glass clears; press and hold still and you breathe on it, the mist comes back around the spot.
// The value is how much of the pane is clear (the mean of the cells), so a half-wiped pane reads 50%.
class _FogToy extends _Toy {
  _FogToy() : super(PcDoc(const [PcParam('op', 'Opacity', 0, 100, 70, unit: '%')])) {
    _fill(doc['op'] / 100);
    _d.setAll(0, _t);
    _settled = true;
    doc.addListener(_sync);
  }
  static const _cw = 24, _ch = 12;
  final List<double> _t = List.filled(_cw * _ch, 0), _d = List.filled(_cw * _ch, 0);
  Offset _p = Offset.zero, _p0 = Offset.zero;
  double _travel = 0, _downT = 0, _breath = 0;
  bool _settled = true;
  Size _sz = const Size(186, 82);

  void _fill(double v) {
    for (var i = 0; i < _t.length; i++) {
      _t[i] = v;
    }
    _settled = false;
  }

  double get _mean => _t.reduce((a, b) => a + b) / _t.length;

  void _sync() {
    if (dragging) return;
    if ((_mean * 100 - doc['op']).abs() > .6) _fill(doc['op'] / 100);
  }

  @override
  void docReset() => _fill(doc['op'] / 100);

  @override
  Object? saveExtra() => List<double>.of(_t);

  @override
  void restoreExtra(Object? o) {
    if (o is List<double>) {
      for (var i = 0; i < _t.length; i++) {
        _t[i] = o[i];
      }
    }
  }

  @override
  void dispose() {
    doc.removeListener(_sync);
    super.dispose();
  }

  @override
  String? zoneAt(Offset p, Size s) => Rect.fromLTWH(0, 0, s.width, s.height).contains(p) ? 'pane' : null;

  @override
  List<String> readout() => [doc.fmt('op')];

  @override
  bool get busy => super.busy || !_settled;

  double _rad(Size s, bool fine) => math.min(s.width, s.height) * (fine ? .15 : .26);

  void _brush(Offset p, Size s, double rad, {double breathe = 0}) {
    for (var j = 0; j < _ch; j++) {
      for (var i = 0; i < _cw; i++) {
        final q = Offset((i + .5) * s.width / _cw, (j + .5) * s.height / _ch);
        final dd = (q - p).distance / rad;
        if (dd >= 1) continue;
        final f = 1 - dd * dd;
        final idx = j * _cw + i;
        _t[idx] = breathe > 0 ? math.max(0.0, _t[idx] - breathe * f) : math.max(_t[idx], math.min(1.0, f * 1.4));
      }
    }
    doc.set('op', _mean * 100);
  }

  @override
  void grab(Offset p, Size s, String zone) {
    _p = _p0 = p;
    _travel = 0;
    _downT = now;
    _breath = 0;
    _sz = s;
  }

  bool _fineNow = false;

  @override
  void drag(Offset p, Offset d, Size s, bool fine) {
    _fineNow = fine;
    _travel += d.distance;
    final prev = _p;
    _p = p;
    if (_travel <= 4) return;
    final rad = _rad(s, fine);
    final n = math.max(1, ((p - prev).distance / (rad * .4)).ceil());
    for (var i = 1; i <= n; i++) {
      _brush(Offset.lerp(prev, p, i / n)!, s, rad);
    }
  }

  @override
  void step(double dt) {
    if (pressed && _travel <= 4 && now - _downT > .2) {
      _breath = (_breath + dt) % .9;
      _brush(_p0, _sz, _rad(_sz, _fineNow) * 1.1, breathe: dt * (_fineNow ? .35 : .8));
    } else {
      _breath = 0;
    }
    var settled = true;
    for (var i = 0; i < _d.length; i++) {
      final e = _t[i] - _d[i];
      if (e.abs() > .004) {
        _d[i] += e * math.min(1, dt * 14);
        settled = false;
      } else {
        _d[i] = _t[i];
      }
    }
    _settled = settled;
  }

  void _cat(Canvas z, Size s) {
    final u = s.height * .36, o = Offset(s.width * .5, s.height * .52);
    Offset q(double x, double y) => o + Offset(x * u, y * u);
    final ink = _ln(N.g91, 1.2);
    final body = Path()
      ..moveTo(q(-.38, -.02).dx, q(-.38, -.02).dy)
      ..cubicTo(q(-.8, .45).dx, q(-.8, .45).dy, q(-.7, .95).dx, q(-.7, .95).dy, q(-.3, .95).dx, q(-.3, .95).dy)
      ..lineTo(q(.35, .95).dx, q(.35, .95).dy)
      ..cubicTo(q(.7, .95).dx, q(.7, .95).dy, q(.7, .4).dx, q(.7, .4).dy, q(.38, -.02).dx, q(.38, -.02).dy);
    z.drawPath(body, _fl(N.g20));
    z.drawPath(body, ink);
    final tail = Path()
      ..moveTo(q(.5, .92).dx, q(.5, .92).dy)
      ..cubicTo(q(1.0, .92).dx, q(1.0, .92).dy, q(1.0, .25).dx, q(1.0, .25).dy, q(.76, .12).dx, q(.76, .12).dy);
    z.drawPath(tail, ink);
    z.drawCircle(q(0, -.36), .42 * u, _fl(N.g20));
    z.drawCircle(q(0, -.36), .42 * u, ink);
    for (final sg in [-1.0, 1.0]) {
      final ear = Path()
        ..moveTo(q(sg * .38, -.55).dx, q(sg * .38, -.55).dy)
        ..lineTo(q(sg * .44, -.98).dx, q(sg * .44, -.98).dy)
        ..lineTo(q(sg * .1, -.72).dx, q(sg * .1, -.72).dy);
      z.drawPath(ear, _fl(N.g20));
      z.drawPath(ear, ink);
      z.drawCircle(q(sg * .16, -.4), 1.7, _fl(N.g100));
      for (var w = -1; w <= 1; w++) {
        z.drawLine(q(sg * .22, -.24 + w * .05), q(sg * .6, -.28 + w * .12), _ln(_a(N.g76, .7)));
      }
    }
    z.drawCircle(q(0, -.26), 1.1, _fl(N.g76));
  }

  @override
  void paint(Canvas z, Size s) {
    _sz = s;
    final pane = RRect.fromRectAndRadius(Rect.fromLTWH(2, 2, s.width - 4, s.height - 4), const Radius.circular(6));
    final lit = hover != null || pressed;
    z.save();
    z.clipRRect(pane);
    z.drawRect(Offset.zero & s, _fl(N.g13));
    _cat(z, s);
    final sg = s.height * .07;
    z.saveLayer(Offset.zero & s, Paint()..imageFilter = ui.ImageFilter.blur(sigmaX: sg, sigmaY: sg, tileMode: TileMode.clamp));
    for (var j = 0; j < _ch; j++) {
      for (var i = 0; i < _cw; i++) {
        final fog = 1 - _d[j * _cw + i];
        if (fog < .01) continue;
        z.drawRect(Rect.fromLTWH(i * s.width / _cw - .5, j * s.height / _ch - .5, s.width / _cw + 1, s.height / _ch + 1), _fl(_a(N.g38, fog * .93)));
      }
    }
    z.restore();
    for (var i = 0; i < 110; i++) {
      final x = _hash(i * 3 + 11) * s.width, y = _hash(i * 3 + 12) * s.height;
      final cx = (x / s.width * _cw).floor().clamp(0, _cw - 1), cy = (y / s.height * _ch).floor().clamp(0, _ch - 1);
      final fog = 1 - _d[cy * _cw + cx];
      if (fog < .15) continue;
      z.drawCircle(Offset(x, y), .7 + _hash(i * 3 + 13) * .9, _fl(_a(N.g76, fog * .4)));
    }
    if (pressed) {
      final r = _rad(s, _fineNow);
      z.drawCircle(_p, r, _ln(_a(_hot, .3)));
      if (_breath > 0) z.drawCircle(_p0, r * 1.1 * (_breath / .9), _ln(_a(N.g76, .3 * (1 - _breath / .9))));
    }
    z.restore();
    z.drawLine(Offset(s.width * .80, 3), Offset(s.width * .62, s.height * .5), _ln(_a(N.g95, .10)));
    z.drawLine(Offset(s.width * .86, 3), Offset(s.width * .75, s.height * .3), _ln(_a(N.g95, .07)));
    z.drawRRect(pane, _ln(pressed ? _act : (lit ? _a(_hot, .6) : N.g44)));
  }
}
