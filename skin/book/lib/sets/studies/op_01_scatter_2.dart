// Part of op_01_scatter.dart (cut there by size, as it was).
part of 'op_01.dart';

// 17 iso placement: wire cubes dropped on tiles inside the spread radius.
void _s17(Canvas c, _P p, double t) {
  const o = Offset(78, 24);
  const n = 8, cs = 9.0;
  _isoGrid(c, o, n, cs, _dim);
  final sd = _seed(p), cnt = 1 + (p.a * 16).round(), rad = 1 + p.b * 4.5;
  final used = <int>{};
  var placed = 0;
  for (var i = 0; i < 200 && placed < cnt; i++) {
    final gx = (_h(i, sd) * n).floor(), gy = (_h(i, sd + 1) * n).floor();
    if (math.sqrt(math.pow(gx - 3.5, 2) + math.pow(gy - 3.5, 2)) > rad) continue;
    if (!used.add(gx * n + gy)) continue;
    placed++;
    final drop = math.max(0.0, math.sin(t * 1.2 + i) * 3);
    _isoBox(c, o, gx * cs + 1.5, gy * cs + 1.5, drop, 6, 6, 6 + _h(i, sd + 4) * 6, placed == 1 ? _white : _blue, .9);
  }
  final rp = Path();
  for (var i = 0; i <= 36; i++) {
    final a = i * math.pi / 18;
    final q = _iso(o, 36 + math.cos(a) * rad * cs, 36 + math.sin(a) * rad * cs, 0);
    i == 0 ? rp.moveTo(q.dx, q.dy) : rp.lineTo(q.dx, q.dy);
  }
  c.drawPath(rp, _s(_green, .9));
  _lb(c, 'N $placed', const Offset(8, 8), _blue, 7);
  _lb(c, 'SEED $sd', const Offset(8, 104), _red, 7);
}

// 18 loose letters: a sample word, each letter thrown off its place; dotted ghosts mark home.
void _s18(Canvas c, _P p, double t) {
  const word = 'SCATTER';
  final sd = _seed(p);
  var x = 12.0;
  for (var i = 0; i < word.length; i++) {
    final tp = _tp(word[i], 26, _white, FontWeight.w200, 0, true);
    final home = Offset(x, 46);
    final moved = _h(i, sd + 7) < p.a;
    if (moved) {
      final off = Offset((_h(i, sd) - .5) * p.b * 90, (_h(i, sd + 1) - .5) * p.b * 80) + Offset(math.sin(t + i) * 1.5, math.cos(t * 1.2 + i) * 1.5);
      final rot = (_h(i, sd + 2) - .5) * p.b * 3;
      _tp(word[i], 26, _dim, FontWeight.w200, 0, true).paint(c, home);
      _dotted(c, home + Offset(tp.width / 2, tp.height / 2), home + off + Offset(tp.width / 2, tp.height / 2), _a(_green, .6), 4);
      c.save();
      c.translate(home.dx + off.dx + tp.width / 2, home.dy + off.dy + tp.height / 2);
      c.rotate(rot);
      _tp(word[i], 26, _blue, FontWeight.w200, 0, true).paint(c, Offset(-tp.width / 2, -tp.height / 2));
      c.restore();
    } else {
      tp.paint(c, home);
    }
    x += tp.width + 2;
  }
  _lb(c, 'LOOSE ${_nn(p.a)}', const Offset(8, 8), _blue, 7);
  _lb(c, 'THROW ${_nn(p.b)}', const Offset(8, 106), _green, 7);
}

// 19 constellation: a new seed draws a new sky and names it; why you scatter -- to get a pattern that looks found, not placed.
void _s19(Canvas c, _P p, double t) {
  final sd = _seed(p), n = 4 + (p.a * 10).round();
  final field = 20 + p.b * 50;
  final stars = [
    for (var i = 0; i < n; i++) Offset(78 + (_h(i, sd) - .5) * field * 2, 54 + (_h(i, sd + 1) - .5) * field * 1.3),
  ];
  for (var i = 0; i < 30; i++) {
    _dot(c, Offset(_h(i, 77) * _pw, _h(i, 78) * _ph), .5, _dim);
  }
  final order = <int>[0];
  final left = List<int>.generate(n - 1, (i) => i + 1);
  while (left.isNotEmpty) {
    final last = stars[order.last];
    left.sort((a, b) => (stars[a] - last).distance.compareTo((stars[b] - last).distance));
    order.add(left.removeAt(0));
  }
  _pl(c, [for (final i in order) stars[i]], _a(_green, .8), .8);
  for (var i = 0; i < n; i++) {
    final tw = 1.4 + math.sin(t * 2 + i) * .5;
    _dot(c, stars[i], tw * .7, _white);
    if (_h(i, sd + 3) > .7) {
      _ln(c, stars[i] - Offset(tw + 2, 0), stars[i] + Offset(tw + 2, 0), _white, .6);
      _ln(c, stars[i] - Offset(0, tw + 2), stars[i] + Offset(0, tw + 2), _white, .6);
    }
  }
  const names = ['LYRA', 'URSA', 'VELA', 'GRUS', 'PYXIS', 'MUSCA'];
  _lb(c, names[sd % names.length], const Offset(8, 100), _red);
  _lb(c, sd.toString().padLeft(2, '0'), const Offset(44, 100), _red, 9);
  _num(c, '$n', Offset(_pw - 8, 6), 22, _blue, 1);
}

// 20 big bang (pushed): everything leaves the centre; streaks run off the frame and the counter goes to 999.
void _s20(Canvas c, _P p, double t) {
  const o = Offset(78, 60);
  final n = 40 + (p.a * 400).round(), sd = _seed(p);
  final ph = (t * .3) % 1;
  for (var i = 0; i < n; i++) {
    final a = _h(i, sd) * 2 * math.pi + (p.b > .5 ? math.sin(i * .3) * p.b : 0);
    final sp = .3 + _h(i, sd + 1) * (.7 + p.b * 2);
    final d = ((ph + _h(i, sd + 2)) % 1) * 200 * sp;
    final u = Offset(math.cos(a), math.sin(a));
    final col = [_blue, _green, _white, _red][i % 4];
    _ln(c, o + u * d, o + u * (d + 2 + d * .12), _a(col, .9), .8);
  }
  _ring(c, o, 3 + (1 - ph) * 4, _white, 1.2);
  _num(c, '${math.min(999, n * 2)}', Offset(_pw - 6, 6), 30, _white, 1);
  _lb(c, 'PARTICLES', Offset(_pw - 6, 38), _blue, 6.5, 1);
}
