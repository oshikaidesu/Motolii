// Part of pheno_e_face.dart (cut there by size, as it was).
part of 'pheno_e.dart';

class VariationsModel extends PhenoModel {
  VariationsModel() : super(_vparams) {
    springK = 380;
    springZ = .62;
    _apply(_lerpList(bank[1], bank[3], .4));
    for (var i = 0; i < 8; i++) {
      pop.add(Spr(1));
    }
  }

  final List<List<double>> bank = [for (final b in _bank0) [...b]];
  final List<Spr> pop = [];
  final math.Random _rnd = math.Random(11);
  final Stopwatch _sw = Stopwatch()..start();
  final List<int> _revT = [];
  Spr tiltS = Spr(0);
  Offset? ghost;
  bool _free = false;
  int _slot = 0, _liveSign = 0;
  double _t = 0, _clock = 0;

  @override
  String get caption => 'moods';
  @override
  String get story => 'Variations: a bank of saved moods; recall, morph between two, shake to randomize';

  List<double> get _cur => [for (final p in _macro) d(p.id)];

  static List<double> _lerpList(List<double> a, List<double> b, double t) => [for (var i = 0; i < a.length; i++) _lerp(a[i], b[i], t)];

  void _apply(List<double> vals) {
    for (var i = 0; i < 8; i++) {
      set(_macro[i].id, vals[i]);
    }
  }

  @override
  Object? saveExtra() => ([for (final b in bank) [...b]], _free);
  @override
  void loadExtra(Object? x) {
    final (b, f) = x as (List<List<double>>, bool);
    for (var i = 0; i < 8; i++) {
      bank[i] = [...b[i]];
    }
    _free = f;
  }

  @override
  String extraSig() => '${bank.map((b) => b.join(',')).join(';')}|$_free';
  @override
  void resetExtra() {
    for (var i = 0; i < 8; i++) {
      bank[i] = [..._bank0[i]];
    }
    _free = false;
  }

  @override
  bool get alive => true; // idle: the portraits sway a little on their nails

  @override
  void tick(double dt) {
    _clock += dt;
    tiltS.t *= math.pow(.002, dt).toDouble();
    tiltS.step(dt, k: 240, z: .28);
    for (final p in pop) {
      p.step(dt, k: 520, z: .38);
    }
  }

  @override
  int? zoneAt(Offset p, Size s) {
    final l = _vlay(s);
    for (var i = 0; i < 8; i++) {
      if ((p - l.cells[i]).distance <= math.max(l.cr + 3, 12)) return 10 + i;
    }
    return (p - l.live).distance <= l.lr + 6 ? 0 : null;
  }

  void _randomize() {
    final vals = <double>[];
    for (var i = 0; i < 8; i++) {
      final p = _macro[i], u = _rnd.nextDouble();
      final x = (i < 3 || i >= 6) ? math.pow(u, 1.7).toDouble() : .12 + .8 * u;
      vals.add(p.min + (p.max - p.min) * x);
    }
    _apply(vals);
    _free = true;
    set('var.mix', 0);
    tiltS.v += 3;
  }

  @override
  void onDown(int zone, Offset p, Size s) {
    _slot = zone - 10;
    _revT.clear();
    _liveSign = 0;
    if (zone == 0) ghost = p;
  }

  @override
  void onMove(Offset p, Offset delta, Size s, bool fine) {
    final l = _vlay(s);
    if (active == 0) {
      ghost = p;
      tiltS.t = _cl(delta.dx * .04, -.5, .5);
      if (delta.dx.abs() > 1.6) {
        final sg = _sgn(delta.dx).toInt();
        if (_liveSign != 0 && sg != _liveSign) {
          final now = _sw.elapsedMilliseconds;
          _revT.add(now);
          _revT.removeWhere((t) => now - t > 700);
          if (_revT.length >= 4) {
            _revT.clear();
            _randomize();
          }
        }
        _liveSign = sg;
      }
      return;
    }
    final i = _slot;
    var best = -1;
    var bd = 1e9;
    for (var j = 0; j < 8; j++) {
      if (j == i) continue;
      final dd = (p - l.cells[j]).distance;
      if (dd < bd) {
        bd = dd;
        best = j;
      }
    }
    final a = l.cells[i], b = l.cells[best], ab = b - a;
    _t = _cl(((p - a).dx * ab.dx + (p - a).dy * ab.dy) / (ab.dx * ab.dx + ab.dy * ab.dy));
    _apply(_lerpList(bank[i], bank[best], _t));
    set('var.from', i.toDouble());
    set('var.to', best.toDouble());
    set('var.mix', _t * 100);
    _free = false;
  }

  @override
  void onUp(Offset vel, bool moved) {
    if (active == 0) {
      final l = _vlay(_lastSize);
      final g = ghost;
      ghost = null;
      if (g != null) {
        for (var i = 0; i < 8; i++) {
          if ((g - l.cells[i]).distance <= l.cr + 8) {
            bank[i] = [for (final p in _macro) v[p.id]!];
            pop[i].x = 1.35;
            set('var.from', i.toDouble());
            set('var.to', i.toDouble());
            set('var.mix', 0);
            _free = false;
          }
        }
      }
      tiltS.t = 0;
      return;
    }
    if (!moved) {
      final i = _slot;
      _apply(bank[i]);
      set('var.from', i.toDouble());
      set('var.to', i.toDouble());
      set('var.mix', 0);
      _free = false;
      pop[i].x = 1.18;
    }
  }

  Size _lastSize = const Size(320, 200);

  @override
  void paintMini(Canvas c, Size s, int? hot) {
    _lastSize = s;
    final l = _vlay(s);
    c.drawLine(Offset(l.cells[0].dx - l.cr - 11, 16), Offset(l.cells[0].dx - l.cr - 11, s.height - 16), _ln(N.g20));
    final from = v['var.from']!.round(), to = v['var.to']!.round(), mix = v['var.mix']! / 100;
    // the morph line between the two ends
    if (!_free && mix > .005 && from != to) {
      c.drawLine(l.cells[from], l.cells[to], _ln(Role.of(N.g44, Role.linked).withValues(alpha: active != null ? .8 : .4)));
      c.drawCircle(_lerpO(l.cells[from], l.cells[to], mix), 2.2, _fl(Role.of(N.g95, Role.linked)));
    }
    for (var row = 0; row < 2; row++) {
      final y = l.cells[row * 4].dy + l.cr + 6;
      c.drawLine(Offset(l.cells[row * 4].dx - l.cr, y), Offset(l.cells[row * 4 + 3].dx + l.cr, y), _ln(N.g26));
    }
    for (var i = 0; i < 8; i++) {
      final sc = pop[i].x;
      c.save();
      // each portrait hangs a little crooked and sways on its own beat
      final sway = math.sin(_clock * .8 + i * 1.9) * .035, crook = ((i * 5) % 7 - 3) * .018;
      c.translate(l.cells[i].dx, l.cells[i].dy + math.sin(_clock * 1.1 + i * 2.3) * .8);
      c.scale(sc);
      _paintFace(c, Offset.zero, l.cr, bank[i], fine: false, alpha: .92, tilt: crook + sway, blink: math.max(0, math.sin(_clock * .6 + i * 2.7) - .985) * 60);
      c.restore();
      final isHot = hot == 10 + i;
      if (!_free && i == from) c.drawCircle(l.cells[i], l.cr + 4, _ln(Role.selected));
      if (!_free && i == to && mix > .005 && to != from) {
        _dashed(c, Path()..addOval(Rect.fromCircle(center: l.cells[i], radius: l.cr + 4)), _ln(Role.selected.withValues(alpha: .8)), on: 2, off: 2.5);
      }
      if (isHot) _hotRing(c, l.cells[i], l.cr + 4, on: active != null);
      if (ghost != null && (ghost! - l.cells[i]).distance <= l.cr + 8) _hotRing(c, l.cells[i], l.cr + 6, on: true);
    }
    _paintFace(c, l.live, l.lr, _cur, tilt: tiltS.x, hot: null, look: ghost == null ? Offset.zero : Offset.zero, blink: active == 0 ? .5 : 0);
    if (hot == 0 || active == 0) _hotRing(c, l.live, l.lr + 4, on: active == 0);
    if (ghost != null) _paintFace(c, ghost!, 11, _cur, fine: false, alpha: .85);
  }

  @override
  List<Edge> edges(Size s) {
    final big = s.height >= 150;
    final from = v['var.from']!.round(), to = v['var.to']!.round();
    return [
      _cap('moods'),
      Edge(_free ? 'unsaved' : '$from>$to  ${v['var.mix']!.round()}%', left: 7, bottom: 6, hot: _free || v['var.mix']! > 0),
      if (big) Edge(_cur.map((x) => x.round()).take(3).join('·'), right: 8, bottom: 6),
    ];
  }
}
