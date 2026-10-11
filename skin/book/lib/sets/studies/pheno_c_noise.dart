part of 'pheno_c.dart';

// Noise: a ridge line over layered contours. Drag the ridge: sideways pinches or stretches it (frequency), up and down pulls it (amplitude).
// Drag the gravel below and the land slides; let go while moving and it coasts to a stop (evolution).
double _tanh(double x) {
  final e = math.exp(2 * x.clamp(-20.0, 20.0));
  return (e - 1) / (e + 1);
}

class _NoiseToy extends _Toy {
  _NoiseToy()
      : super(PcDoc(const [
          PcParam('freq', 'Frequency', .5, 16, 3, dec: 1),
          PcParam('amp', 'Amplitude', 0, 100, 45, unit: '%'),
          PcParam('evo', 'Evolution', -720, 720, 0, unit: '°'),
        ])) {
    _lf = math.log(doc['freq']);
    _amp = _Sp(doc['amp'] / 100);
  }
  late double _lf;
  late _Sp _amp;
  double _gs = 0, _vx = 0, _lastT = 0;
  bool _coast = false;
  Offset? _ptr;
  String _mode = '';

  double _pk(Size s) => s.height * .34;
  double _base(Size s) => s.height * .60;

  @override
  String? zoneAt(Offset p, Size s) => p.dy > s.height - 24 ? 'ground' : 'ridge';

  @override
  List<String> readout() => ['f ${doc.fmt('freq')}', 'a ${doc.fmt('amp')}', 'e ${doc.fmt('evo')}'];

  @override
  bool get busy => super.busy || hovering || _coast || !_amp.at(doc['amp'] / 100) || (_lf - math.log(doc['freq'])).abs() > .002;

  double _fbm(double u) => .62 * _noise1(u) + .28 * _noise1(u * 2.1 + 7.3) + .10 * _noise1(u * 4.3 + 13.1);

  @override
  void grab(Offset p, Size s, String zone) {
    _mode = zone;
    _ptr = p;
    _coast = false;
    _vx = 0;
    _lastT = now;
  }

  void _slide(double dx, Size s) {
    final f = doc['freq'];
    doc.set('evo', doc['evo'] - dx * 60 * f / s.width);
    _gs += dx;
  }

  @override
  void drag(Offset p, Offset d, Size s, bool fine) {
    _ptr = p;
    final k = fine ? .1 : 1.0;
    if (_mode == 'ground') {
      _slide(d.dx * k, s);
      final t = now, dt = t - _lastT;
      if (dt > .001) _vx = _vx * .6 + .4 * (d.dx / dt);
      _lastT = t;
    } else {
      doc.set('freq', doc['freq'] * math.exp(-d.dx / s.width * 2.2 * k));
      doc.set('amp', doc['amp'] + (-d.dy) / _pk(s) * 100 * k);
    }
  }

  @override
  void release(Offset p, Size s) {
    if (_mode == 'ground' && _vx.abs() > 40 && now - _lastT < .08) {
      _coast = true;
      _ptr = null;
    } else {
      finish();
    }
  }

  @override
  void stopMotion() {
    _coast = false;
    _vx = 0;
  }

  Size _sz = const Size(186, 70);

  @override
  void step(double dt) {
    _lf += (math.log(doc['freq']) - _lf) * math.min(1, dt * 18);
    _amp.to(doc['amp'] / 100, dt, k: 280, d: 15);
    if (_coast) {
      _vx *= math.exp(-dt * 3.2);
      final before = doc['evo'];
      _slide(_vx * dt, _sz);
      if (_vx.abs() < 8 || doc['evo'] == before) {
        _coast = false;
        _vx = 0;
        finish();
      }
    }
  }

  double _y(double x, Size s, double lf, double amp, double evo, double lift, double fade) {
    final u = x / s.width * math.exp(lf) + evo / 60;
    final lim = _base(s) * .85, off = _fbm(u) * _pk(s) * 2.3 * amp * fade;
    return _base(s) - lim * _tanh(off / lim) + lift;
  }

  @override
  void paint(Canvas z, Size s) {
    _sz = s;
    final sc = s.height / 186;
    final evo = doc['evo'];
    final hot = hover == 'ridge' || (dragging && _mode == 'ridge');
    final mass = Path()..moveTo(0, s.height);
    for (var x = 0.0; x <= s.width + 2; x += 2) {
      mass.lineTo(x, _y(x, s, _lf, _amp.x, evo, 0, 1));
    }
    mass.lineTo(s.width + 2, s.height);
    mass.close();
    z.save();
    z.clipPath(mass);
    final hatch = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1
      ..shader = ui.Gradient.linear(Offset(0, _base(s) - _pk(s) * 2), Offset(0, _base(s) + 44 * sc), [_a(N.g44, .5), _a(N.g44, 0)]);
    for (var x = -s.height; x < s.width + 4; x += 6) {
      z.drawLine(Offset(x, s.height), Offset(x + s.height, 0), hatch);
    }
    z.restore();
    for (var k = 3; k >= 1; k--) {
      final path = Path();
      for (var x = 0.0; x <= s.width + 2; x += 2) {
        final y = _y(x, s, _lf, _amp.x, evo, k * 8 * sc, 1 - k * .2);
        x == 0 ? path.moveTo(x, y) : path.lineTo(x, y);
      }
      z.drawPath(path, _ln(_a(N.g63, .5 - k * .12)));
    }
    final ridge = Path();
    for (var x = 0.0; x <= s.width + 2; x += 2) {
      final y = _y(x, s, _lf, _amp.x, evo, 0, 1);
      x == 0 ? ridge.moveTo(x, y) : ridge.lineTo(x, y);
    }
    z.drawPath(ridge, _ln(hot ? N.g100 : N.g91, hot ? 1.75 : 1.25));

    final gHot = hover == 'ground' || (dragging && _mode == 'ground');
    for (var row = 0; row < 2; row++) {
      final y = s.height - 10 + row * 5.0;
      for (var i = 0; i < 40; i++) {
        final h1 = _hash(i * 5 + row * 91), h2 = _hash(i * 5 + row * 91 + 2);
        final x = (((h1 * s.width + _gs * (row == 0 ? 1 : .8)) % s.width) + s.width) % s.width;
        final l = 1 + h2 * 4;
        z.drawLine(Offset(x, y), Offset(x + l, y), _ln(gHot ? _a(_hot, .85) : _a(N.g56, .55 + .3 * h2)));
      }
    }

    if (hovering && !pressed) {
      final x = ((now * .11) % 1) * s.width;
      z.drawCircle(Offset(x, _y(x, s, _lf, _amp.x, evo, 0, 1)), 2.2, _fl(_a(N.g100, .9)));
    }
    if (dragging && _mode == 'ridge' && _ptr != null) {
      final x = _ptr!.dx.clamp(0.0, s.width).toDouble();
      z.drawCircle(Offset(x, _y(x, s, _lf, _amp.x, evo, 0, 1)), 2.5, _fl(_act));
    }
  }
}
