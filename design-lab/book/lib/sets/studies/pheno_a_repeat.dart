part of 'pheno_a.dart';

/// 3. Repeat / Clone: count + offset = a chain of copies pulled out of the original. Pull the first copy (offset x, y), pull the empty slot after the last (count).
class _RepeatSpec extends PhenoSpec {
  const _RepeatSpec();
  @override
  String get name => '3 Repeat';
  @override
  String get word => 'repeat';
  @override
  String get caption => 'Repeat: count + offset x / y. Silhouette: a thread of fading echoes of one small tile, with an empty slot after the last.';
  @override
  double get rowHeight => 88;
  @override
  List<PhenoParam> get params => const [
        PhenoParam('count', 'Count', 1, 32, 6, prefix: '×'),
        PhenoParam('offx', 'Offset X', -160, 160, 34, unit: 'px'),
        PhenoParam('offy', 'Offset Y', -160, 160, -8, unit: 'px'),
      ];
  @override
  PhenoSim newSim() => _RepeatSim();

  /// Composition px -> screen px.
  static double k(Size s) => _cr(s).width / 360;
  static Offset origin(Size s) => Offset(_cr(s).left + 26, _cr(s).center.dy + 8);
  static Offset off(PhenoValues v) => Offset(v['offx'], v['offy']);
  static double tile(Size s) => (_cr(s).height * .3).clamp(14.0, 34.0);
  static Offset slot(Size s, PhenoValues v, int i) => origin(s) + off(v) * (k(s) * i);

  @override
  String? zoneAt(Size s, Offset p, PhenoValues v) {
    final n = v['count'].round(), tail = slot(s, v, n), pull = slot(s, v, 1);
    final dt = (p - tail).distance, dp = n >= 2 ? (p - pull).distance : 1e9;
    if (math.min(dt, dp) > 18) return null;
    return dp <= dt ? 'pull' : 'tail';
  }

  @override
  Map<String, double> drag(String zone, Size s, Offset p0, Offset p, PhenoValues v0) {
    if (zone == 'pull') {
      final o = off(v0) + (p - p0) / k(s);
      return {'offx': o.dx, 'offy': o.dy};
    }
    var o = off(v0);
    if (o.distance < 8) o = const Offset(8, 0);
    final d = o / o.distance, step = o.distance * k(s);
    return {'count': v0['count'] + (((p - p0).dx * d.dx + (p - p0).dy * d.dy) / step).round()};
  }

  @override
  List<String> zoneParams(String zone) => zone == 'pull' ? ['offx', 'offy'] : ['count'];
}

class _RepeatSim extends PhenoSim {
  static const maxN = 33;
  final x = List.generate(maxN, (_) => _Sp()), y = List.generate(maxN, (_) => _Sp()), pre = List.generate(maxN, (_) => _Sp());
  bool first = true;

  @override
  bool step(double dt, Size s, PhenoCtx c) {
    final n = c.v['count'].round(), o = _RepeatSpec.origin(s);
    var moving = false;
    for (var i = 1; i < maxN; i++) {
      final on = i < n, tg = on ? _RepeatSpec.slot(s, c.v, i) : o;
      if (first) {
        x[i].x = tg.dx;
        y[i].x = tg.dy;
        pre[i].x = on ? 1 : 0;
      }
      // A chain: copies further out are softer, so they trail the pull and settle a beat later.
      final kk = 260 - i * 5.0;
      moving |= x[i].step(tg.dx, dt, k: kk, z: .6);
      moving |= y[i].step(tg.dy, dt, k: kk, z: .6);
      moving |= pre[i].step(on ? 1 : 0, dt, k: 240, z: 1);
    }
    first = false;
    return moving;
  }

  void glyph(Canvas cv, Offset c, double g, double a, bool original) {
    final r = RRect.fromRectAndRadius(Rect.fromCenter(center: c, width: g, height: g), const Radius.circular(3));
    cv.drawRRect(r, _fill((original ? N.g15 : N.g13).withValues(alpha: a)));
    cv.drawRRect(r.deflate(.5), _line((original ? N.g95 : N.g76).withValues(alpha: a)));
    cv.drawRect(Rect.fromLTWH(c.dx - g / 2 + 3, c.dy - g / 2 + 3, g * .22, g * .22), _fill((original ? N.g95 : N.g63).withValues(alpha: a)));
  }

  @override
  void paint(Canvas cv, Size s, PhenoCtx c) {
    final n = c.v['count'].round(), o = _RepeatSpec.origin(s), g = _RepeatSpec.tile(s);
    Offset pt(int i) => i == 0 ? o : Offset(x[i].x, y[i].x);
    // The thread: one hairline through the origin and every copy.
    final thread = N.g38;
    for (var i = 1; i < maxN; i++) {
      final a = pre[i].x.clamp(0.0, 1.0);
      if (a > .02) cv.drawLine(pt(i - 1), pt(i), _line(thread.withValues(alpha: a)));
    }
    final pull = _mark(c, 'pull'), tail = _mark(c, 'tail');
    // The empty slot after the last copy: an invitation to pull one more out.
    final slot = _RepeatSpec.slot(s, c.v, n);
    cv.drawCircle(slot, g * .32, _line(N.g38.withValues(alpha: c.live ? .9 : .5)));
    if (tail != null) {
      cv.drawCircle(slot, g * .32 + 6, _line(tail));
      cv.drawLine(pt(math.max(n - 1, 0)), slot, _line(Role.linkedFor(Fam.follow).withValues(alpha: .7)));
    }
    for (var i = maxN - 1; i >= 1; i--) {
      final a = pre[i].x.clamp(0.0, 1.0);
      if (a < .02) continue;
      glyph(cv, pt(i), g, a * (1 - .65 * (i / math.max(n, 2))).clamp(.3, 1.0), false);
    }
    glyph(cv, o, g, 1, true);
    if (n >= 2 && pull != null) {
      cv.drawCircle(pt(1), g * .5 + 5, _line(pull));
      cv.drawLine(o, pt(1), _line(Role.linkedFor(Fam.follow).withValues(alpha: .7)));
    }
    // A small life while the pointer is near: one dot runs down the thread, copy after copy.
    if (c.live && n >= 2) {
      final u = (c.t * 1.1) % (n - 1), i = u.floor(), f = u - i;
      cv.drawCircle(Offset.lerp(pt(i), pt(i + 1), f)!, 1.6, _fill(N.g95.withValues(alpha: .8)));
    }
  }
}
