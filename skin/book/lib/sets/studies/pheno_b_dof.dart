// B4 Depth of field: focus distance + aperture are a viewfinder frame you slide along a diagonal file of little photo cards. Cards in the frame are sharp, the rest smear;
// the two faint frames either side are the depth of field, and pulling one of them out or in is the aperture (a wide-open lens keeps only a thin slice sharp).
part of 'pheno_b.dart';

class DofValues extends PhenoValues {
  DofValues()
      : super(const [
          PhSpec('focus', .3, 10, 2.5, unit: ' m', decimals: 1),
          PhSpec('aperture', 1.4, 22, 4, prefix: 'f/', decimals: 1),
        ]);
  double get focus => this['focus'];
  double get aperture => this['aperture'];
}

(PhenoValues, PhenoSim Function(PhenoValues)) _makeDof() => (DofValues(), (v) => DofSim(v));

class DofSim extends PhenoSim {
  DofSim(super.v);
  static const _n = 9;
  static const _k = .0909; // the circle of confusion stays under ~0.8 px for |m - F| / m < _k * f
  final _f = Spr(.6), _a = Spr(4);
  bool _seeded = false;
  double _t0 = 0, _a0 = 0, _v0 = 0;

  double _tOf(double m) => math.log(m / .3) / math.log(10 / .3);
  double _mOf(double t) => .3 * math.pow(10 / .3, t);
  double _u(Size s) => math.min(s.width / 320, s.height / 200).clamp(.3, 1.6).toDouble();
  Offset _from(Size s) => Offset(s.width * .12, s.height * .62);
  Offset _to(Size s) => Offset(s.width * .88, s.height * .4);
  Offset _pos(Size s, double t) => Offset.lerp(_from(s), _to(s), t)!;
  double _sc(double t) => ui.lerpDouble(1, .5, t)!;
  Size _card(Size s, double t) => Size(34, 48) * (_sc(t) * _u(s));

  /// Near and far depth (in t) of the sharp slice for focus [m] and f-stop [f].
  (double, double) _band(double m, double f) {
    final x = _k * f;
    final near = m / (1 + x);
    final far = x < 1 ? m / (1 - x) : 1e9;
    return (_tOf(near.clamp(.3, 10)), far >= 10 ? 1.0 : _tOf(far));
  }

  double _project(Offset p, Size s) {
    final a = _from(s), b = _to(s), ab = b - a;
    return (((p - a).dx * ab.dx + (p - a).dy * ab.dy) / (ab.dx * ab.dx + ab.dy * ab.dy)).clamp(0.0, 1.0).toDouble();
  }

  @override
  String? zoneAt(Offset p, Size s) {
    final m = v['focus'];
    final (tn, tf) = _band(m, v['aperture']);
    String? best;
    var bd = 1e9;
    for (final e in [('near', tn), ('far', tf), ('focus', _tOf(m))]) {
      final c = _pos(s, e.$2), r = math.max(14.0, _card(s, e.$2).height * .8), d = (p - c).distance;
      if (d < r && d < bd - (e.$1 == 'focus' ? 4 : 0)) {
        bd = d;
        best = e.$1;
      }
    }
    return best;
  }

  double _val(String z, Offset p, Size s) {
    final t = _project(p, s);
    if (z == 'focus') return t;
    final m = v['focus'], mm = _mOf(t);
    final x = z == 'near' ? (m / mm - 1) : (mm > m ? 1 - m / mm : 0.0);
    return x.clamp(0.0, 1.0) / _k;
  }

  @override
  void down(String zone, Offset p, Size s) {
    _t0 = _tOf(v['focus']);
    _a0 = _val(zone, p, s);
    _v0 = zone == 'focus' ? _t0 : v['aperture'];
  }

  @override
  void drag(String zone, Offset p, Offset delta, Size s, {required bool fine, required bool alt}) {
    final a = _fine(_val(zone, p, s), _a0, _v0, fine);
    if (zone == 'focus') {
      v.set('focus', _mOf(a.clamp(0.0, 1.0).toDouble()));
    } else {
      v.set('aperture', a);
    }
  }

  @override
  List<String> resetIds(String? zone) => switch (zone) {
        'focus' => ['focus'],
        'near' || 'far' => ['aperture'],
        _ => v.ids.toList(),
      };

  @override
  bool tick(double dt, PhCtx x, Size s) {
    if (!_seeded) {
      _f.x = _tOf(v['focus']);
      _a.x = v['aperture'];
      _seeded = true;
    }
    _f.t = _tOf(v['focus']);
    _a.t = v['aperture'];
    _f.step(dt, k: 700, c: 34);
    _a.step(dt, k: 700, c: 34);
    return _f.moving(.0008) || _a.moving(.01) || x.hover;
  }

  @override
  List<PhRead> readout() => [
        PhRead(v.fmt('focus'), const {'focus'}, v.changed('focus')),
        PhRead(v.fmt('aperture'), const {'near', 'far'}, v.changed('aperture')),
      ];

  void _drawCard(Canvas c, Rect r, double sc, Color edge, double alpha) {
    final rr = RRect.fromRectAndRadius(r, Radius.circular(2.2 * sc));
    c.drawRRect(rr, _fill(_al(N.g15, alpha)));
    final photo = Rect.fromLTWH(r.left + r.width * .12, r.top + r.height * .1, r.width * .76, r.height * .56);
    c.drawRect(photo, _fill(_al(N.g26, alpha)));
    final mt = Path()
      ..moveTo(photo.left, photo.bottom)
      ..lineTo(photo.left + photo.width * .36, photo.top + photo.height * .38)
      ..lineTo(photo.left + photo.width * .6, photo.top + photo.height * .7)
      ..lineTo(photo.left + photo.width * .78, photo.top + photo.height * .5)
      ..lineTo(photo.right, photo.bottom)
      ..close();
    c.drawPath(mt, _fill(_al(N.g56, alpha)));
    c.drawCircle(photo.topRight + Offset(-photo.width * .2, photo.height * .24), photo.width * .09, _fill(_al(N.g91, alpha)));
    final ly = r.top + r.height * .78;
    c.drawLine(Offset(r.left + r.width * .14, ly), Offset(r.left + r.width * .74, ly), _hair(_al(N.g56, alpha)));
    c.drawLine(Offset(r.left + r.width * .14, ly + 3.2 * sc), Offset(r.left + r.width * .52, ly + 3.2 * sc), _hair(_al(N.g44, alpha)));
    c.drawRRect(rr, _hair(_al(edge, alpha)));
  }

  void _brackets(Canvas c, Rect r, Color col, {double len = 5}) {
    final p = _hair(col);
    for (final (cx, cy, dx, dy) in [(r.left, r.top, 1.0, 1.0), (r.right, r.top, -1.0, 1.0), (r.left, r.bottom, 1.0, -1.0), (r.right, r.bottom, -1.0, -1.0)]) {
      c.drawLine(Offset(cx, cy), Offset(cx + dx * len, cy), p);
      c.drawLine(Offset(cx, cy), Offset(cx, cy + dy * len), p);
    }
  }

  @override
  void paint(Canvas c, Size s, PhCtx x) {
    _lattice(c, s);
    final u = _u(s), tf = _f.x.clamp(0.0, 1.0).toDouble(), mF = _mOf(tf), fs = _a.x.clamp(1.0, 24.0).toDouble();
    final (tn, tfar) = _band(mF, fs);
    // the file of cards, far to near
    for (var i = _n - 1; i >= 0; i--) {
      final t = i / (_n - 1), m = _mOf(t);
      final sigma = (1.7 * (4 / fs) * (m - mF).abs() / m).clamp(0.0, 5.0).toDouble() * u;
      final sway = x.hover ? math.sin(x.time * 1.7 + i * 1.3) * .5 * u : 0.0;
      final sz = _card(s, t), ctr = _pos(s, t) + Offset(0, sway), r = Rect.fromCenter(center: ctr, width: sz.width, height: sz.height);
      final sharp = sigma < .6;
      if (sigma < .35) {
        _drawCard(c, r, _sc(t) * u, sharp ? N.g91 : N.g56, 1);
      } else {
        final b = r.inflate(sigma * 3 + 2);
        c.saveLayer(b, Paint()..imageFilter = ui.ImageFilter.blur(sigmaX: sigma, sigmaY: sigma, tileMode: TileMode.decal));
        _drawCard(c, r, _sc(t) * u, N.g56, 1 - (sigma / 12));
        c.restore();
      }
      if (sharp) c.drawCircle(r.topCenter - Offset(0, 4 * u), 1.3, _fill(Role.key));
    }
    // the frames: the sharp slice (faint) and the focus (bright)
    Rect fr(double t) {
      final z = _card(s, t) * 1.4;
      return Rect.fromCenter(center: _pos(s, t) - Offset(0, 1.5 * u), width: z.width, height: z.height);
    }

    final h = x.hot;
    _brackets(c, fr(tn), h == 'near' ? Role.selected : _al(N.g63, .55), len: 4 * u + 1);
    _brackets(c, fr(tfar), h == 'far' ? Role.selected : _al(N.g63, tfar >= .999 ? .22 : .55), len: 4 * u + 1);
    _brackets(c, fr(tf), h == 'focus' ? Role.selected : _al(_light, .95), len: 6 * u + 1);
  }
}
