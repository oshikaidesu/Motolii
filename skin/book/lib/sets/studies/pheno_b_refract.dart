// B2 Refraction ray: the index of refraction is the bend of ONE ray through a small glass slab. Grab the ray where it leaves the slab and put it where you want it to land;
// the number follows (Snell). A pulse of light runs the path while the pointer is over it, slower inside the glass.
part of 'pheno_b.dart';

class RefractionValues extends PhenoValues {
  RefractionValues() : super(const [PhSpec('ior', 1.0, 2.5, 1.45, prefix: 'n ', decimals: 2)]);
  double get ior => this['ior'];
}

(PhenoValues, PhenoSim Function(PhenoValues)) _makeRefract() => (RefractionValues(), (v) => RefractionSim(v));

class RefractionSim extends PhenoSim {
  RefractionSim(super.v);
  static const _t1 = 48 * math.pi / 180;
  final _th = Spr(0);
  bool _seeded = false;
  double _n0 = 1, _na0 = 1, _phase = 0, _pulse = 0;

  double _theta(double n) => math.asin(math.sin(_t1) / n);
  Offset _p1(Size s) => Offset(s.width * .40, s.height * .36);
  double _thick(Size s) => s.height * .34;
  Offset _p2(Size s, double th) => _p1(s) + Offset(_thick(s) * math.tan(th), _thick(s));

  @override
  String? zoneAt(Offset p, Size s) {
    final a = _p1(s), b = _p2(s, _theta(v['ior']));
    final ab = b - a, t = (((p - a).dx * ab.dx + (p - a).dy * ab.dy) / (ab.dx * ab.dx + ab.dy * ab.dy)).clamp(0.0, 1.0).toDouble();
    if ((p - b).distance < 14 || (p - (a + ab * t)).distance < 10) return 'ior';
    return null;
  }

  double _n(Offset p, Size s) {
    final th = math.atan2(p.dx - _p1(s).dx, _thick(s)).clamp(_theta(2.5), _t1).toDouble();
    return math.sin(_t1) / math.sin(th);
  }

  @override
  void down(String zone, Offset p, Size s) {
    _n0 = v['ior'];
    _na0 = _n(p, s);
  }

  @override
  void drag(String zone, Offset p, Offset delta, Size s, {required bool fine, required bool alt}) =>
      v.set('ior', _fine(_n(p, s), _na0, _n0, fine));

  @override
  List<String> resetIds(String? zone) => ['ior'];

  @override
  bool tick(double dt, PhCtx x, Size s) {
    if (!_seeded) {
      _th.x = _theta(v['ior']);
      _seeded = true;
    }
    _th.t = _theta(v['ior']);
    _th.step(dt, k: 520, c: 20);
    final live = x.hover || x.dragging;
    _pulse += ((live ? 1 : 0) - _pulse) * (1 - math.exp(-dt * 9));
    if (live || _pulse > .02) _phase = (_phase + dt / 2.3) % 1;
    return _th.moving(.0006) || live || _pulse > .02;
  }

  @override
  List<PhRead> readout() => [PhRead(v.fmt('ior'), const {'ior'}, v.changed('ior'))];

  @override
  void paint(Canvas c, Size s, PhCtx x) {
    _lattice(c, s);
    final light = _light, p1 = _p1(s), t = _thick(s), th = _th.x.clamp(.15, _t1).toDouble();
    final n = v['ior'];
    final top = p1.dy, bot = top + t, l = s.width * .14, r = s.width * .86;
    final slab = RRect.fromLTRBR(l, top, r, bot, const Radius.circular(3));
    final dir = Offset(math.sin(_t1), math.cos(_t1));

    // the slab: a faint pane, a lit top face, an inset bevel
    c.drawRRect(slab, Paint()..shader = ui.Gradient.linear(Offset(0, top), Offset(0, bot), [_al(N.g100, .07), _al(N.g100, .02)]));
    c.drawRRect(slab, _hair(N.g44));
    c.drawLine(Offset(l + 3, top + .5), Offset(r - 3, top + .5), _hair(_al(N.g100, .35)));
    c.drawRRect(slab.deflate(3), _hair(_al(N.g100, .05)));

    // the normal and the two angle arcs (no words)
    final dash = _hair(_al(N.g63, .45));
    for (var y = top - 22; y < bot + 6; y += 6) {
      c.drawLine(Offset(p1.dx, y), Offset(p1.dx, y + 3), dash);
    }
    c.drawArc(Rect.fromCircle(center: p1, radius: 15), -math.pi / 2 - _t1, _t1, false, _hair(_al(N.g63, .5)));
    c.drawArc(Rect.fromCircle(center: p1, radius: 15), math.pi / 2 - th, th, false, _hair(_al(N.g63, .5)));

    // the same ray if nothing bent it (n = 1)
    final ghost = p1 + dir * 400;
    final gp = _hair(_al(N.g63, .3));
    final gl = (ghost - p1);
    for (var d = 0.0; d < 150; d += 8) {
      c.drawLine(p1 + gl / gl.distance * d, p1 + gl / gl.distance * (d + 3.5), gp);
    }

    // the lamp, the incoming ray, the bent ray, the exit ray
    final src = p1 - dir * (s.height * .46);
    final p2 = p1 + Offset(t * math.tan(th), t);
    final rayP = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.2
      ..strokeCap = StrokeCap.round
      ..color = _al(light, .92);
    final glow = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 4
      ..color = _al(light, .14)
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3);
    final exitEnd = p2 + dir * (s.height * 1.2);
    for (final seg in [(src, p1), (p1, p2), (p2, exitEnd)]) {
      c.drawLine(seg.$1, seg.$2, glow);
      c.drawLine(seg.$1, seg.$2, rayP);
    }
    c.drawCircle(src, 3.2, _hair(_al(light, .8)));
    c.drawCircle(src, 1.2, _fill(light));
    // a little reflected light at the top face (Fresnel): quiet, but it grows with n
    final fres = math.pow((n - 1) / (n + 1), 2) * 7 + .04;
    final refl = Offset(math.sin(_t1), -math.cos(_t1));
    c.drawLine(p1, p1 + refl * (s.height * .34), _hair(_al(light, fres.clamp(0.0, .5).toDouble())));
    c.drawCircle(p1, 2, _fill(_al(light, .9)));

    // the pulse of light: runs the path, slower in glass
    if (_pulse > .02) {
      final l0 = (p1 - src).distance, l1 = (p2 - p1).distance, l2 = (exitEnd - p2).distance;
      final t0 = l0, t1 = l1 * n, t2 = l2, total = t0 + t1 + t2;
      var q = _phase * total;
      Offset at;
      if (q < t0) {
        at = src + (p1 - src) * (q / t0);
      } else if (q < t0 + t1) {
        at = p1 + (p2 - p1) * ((q - t0) / t1);
      } else {
        q -= t0 + t1;
        at = p2 + (exitEnd - p2) * (q / t2);
      }
      c.drawCircle(at, 3.4, _fill(_al(light, .22 * _pulse)));
      c.drawCircle(at, 1.5, _fill(_al(N.g100, .95 * _pulse)));
    }

    // hover: the grabbable end of the ray
    if (x.hot != null) {
      c.drawCircle(p2, 5, _hair(Role.selected));
    } else if (x.hover) {
      c.drawCircle(p2, 3, _hair(_al(N.g63, .5)));
    }
  }
}
