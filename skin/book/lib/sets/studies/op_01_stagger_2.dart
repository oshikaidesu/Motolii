// Part of op_01_stagger.dart (cut there by size, as it was).
part of 'op_01.dart';

// 18 rowing eight: oars dip in turn; with no offset the boat rows in perfect sync.
void _t18(Canvas c, _P p, double t) {
  const y = 64.0;
  final hull = Path()
    ..moveTo(8, y)
    ..quadraticBezierTo(78, y + 16, 148, y - 2)
    ..lineTo(8, y);
  c.drawPath(hull, _s(_white, 1.1));
  for (var k = 0; k < 3; k++) {
    final yy = y + 24 + k * 9.0;
    _pl(c, [for (var x = 0.0; x <= _pw; x += 6) Offset(x, yy + math.sin(x * .15 - t * 2 + k) * 1.2)], _a(_blue, .45 - k * .1), .8);
  }
  const n = 6;
  for (var i = 0; i < n; i++) {
    final x = 26.0 + i * 20;
    final ph = t * 2.4 - _k(p, i, n) * _off(p) * 3;
    final a = math.sin(ph) * .5;
    final dip = math.cos(ph) > 0;
    _ring(c, Offset(x, y - 12), 3, _white, 1);
    _ln(c, Offset(x, y - 9), Offset(x + a * 4, y - 1), _white, 1);
    final oarEnd = Offset(x - 2 + math.sin(a) * 26, y + 16 + (dip ? 4 : -2));
    _ln(c, Offset(x + a * 4, y - 4), oarEnd, dip ? _green : _white, 1);
    if (dip) c.drawOval(Rect.fromCenter(center: oarEnd + const Offset(0, 2), width: 8, height: 3), _s(_a(_white, .6), .7));
  }
  _offTag(c, p, const Offset(8, 8));
  _lb(c, _ordName(p.c), const Offset(148, 8), _red, 6.5, 1);
}

// 19 draw the order: draw a path through the dots; the dots light in the order your path visits them.
void _t19(Canvas c, _P p, double t) {
  const cols = 6, rows = 4;
  final dots = [for (var j = 0; j < rows; j++) for (var i = 0; i < cols; i++) Offset(22 + i * 22.0, 26 + j * 22.0)];
  final n = dots.length;
  List<Offset> path = p.pts.length > 2
      ? p.pts
      : [for (var a = 0.0; a < 4 * math.pi; a += .2) Offset(78 + math.cos(a) * (4 + a * 5), 59 + math.sin(a) * (4 + a * 3))];
  _pl(c, path, _a(_purple, .7), .9);
  final rank = <int>[];
  for (var d = 0; d < n; d++) {
    var best = 0;
    var bd = 1e9;
    for (var k = 0; k < path.length; k++) {
      final dd = (path[k] - dots[d]).distance;
      if (dd < bd) {
        bd = dd;
        best = k;
      }
    }
    rank.add(best);
  }
  final sorted = List<int>.generate(n, (i) => i)..sort((a, b) => rank[a].compareTo(rank[b]));
  for (var o = 0; o < n; o++) {
    final d = sorted[o];
    final k = _cas(o, n, _off(p) * .3, t, .6);
    _ring(c, dots[d], 4, k > 0 ? (k < 1 ? _green : _white) : _dim, 1);
    if (k > 0 && k < 1) _dot(c, dots[d], 2.2, _green);
    _lb(c, '${o + 1}', dots[d] + const Offset(5, -10), _red, 5.5);
  }
  if (p.touch != null) _ring(c, p.touch!, 6, _white, .8);
  _lb(c, 'DRAW', const Offset(8, 108), _purple, 6.5);
  _lb(c, '${(_off(p) * 300).round()}MS', const Offset(148, 108), _blue, 6.5, 1);
}

// 20 endless smear (pushed): 160 lines, offsets so long the cascade wraps into a spiral and runs off the frame.
void _t20(Canvas c, _P p, double t) {
  const o = Offset(78, 60), n = 160;
  for (var i = 0; i < n; i++) {
    final k = _k(p, i, n);
    final lag = k * (.005 + p.a * .06);
    final ph = (t * .6 - lag);
    final a = i * .19 + ph;
    final r = 4 + i * .55 * (.4 + p.a);
    final q = o + Offset(math.cos(a) * r, math.sin(a) * r * .8);
    final u = Offset(math.cos(a + 1.3), math.sin(a + 1.3)) * (3 + math.sin(ph * 3) * 2);
    _ln(c, q - u, q + u, _a([_blue, _green, _white, _red][i % 4], .85), .8);
  }
  _num(c, '${(.005 + p.a * .06) * n * 1000 ~/ 1}', Offset(_pw - 6, 4), 26, _white, 1);
  _lb(c, 'MS TOTAL', Offset(_pw - 6, 32), _blue, 6, 1);
}
