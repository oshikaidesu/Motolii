// Part of pop_03_warp.dart (cut there by size, as it was).
part of 'pop_03.dart';

void _wHaze(Canvas c, Size s, _St st, double t) {
  final hz = s.height * .5, rad = 6 + st.a * 34, str = st.b;
  _hazeScene(c, s);
  for (var y = hz - rad; y < hz + rad; y += 2) {
    final dx = str * 7 * math.sin(y * .7 + t * 7) * _fall((y - hz).abs(), rad);
    c
      ..save()
      ..clipRect(Rect.fromLTWH(0, y, s.width, 2))
      ..translate(dx, 0);
    _hazeScene(c, s);
    c.restore();
  }
}

void _wOcto(Canvas c, Size s, _St st, double t) {
  final w = s.width, h = s.height, cx = w / 2, hy = h * .3;
  final reach = .5 + st.b, curl = (st.a * 2 - 1) * 3.2 + st.spin * .5;
  for (var k = 0; k < 4; k++) {
    final ph = (t * .25 + k * .27) % 1;
    c.drawCircle(Offset(w * (.15 + k * .22) + 3 * math.sin(t * 2 + k), h - ph * h), 2 + k % 2, _s(_al(_kCyan, .7), 1.2));
  }
  for (var j = 0; j < 6; j++) {
    final sgn = j < 3 ? -1.0 : 1.0;
    var p = Offset(cx - 16 + j * 6.4, hy + 14);
    var a = math.pi / 2 + (j - 2.5) * .32;
    for (var i = 0; i < 14; i++) {
      final u = i / 14;
      a += sgn * curl * .05 * u * 2 + .06 * math.sin(t * 2.2 + j + i * .5);
      final q = p + _dir(a) * (2.6 * reach + 1);
      c.drawLine(p, q, _s(_kPink, 7 * (1 - u) + 1.5));
      p = q;
    }
  }
  c.drawOval(Rect.fromCenter(center: Offset(cx, hy), width: 50, height: 42), _f(_kPink));
  for (final ex in [-9.0, 9.0]) {
    c.drawCircle(Offset(cx + ex, hy + 2), 6, _f(_kWhite));
    c.drawCircle(Offset(cx + ex + 1, hy + 3), 3, _f(_kInk));
  }
  c.drawCircle(Offset(cx - 14, hy - 10), 3, _f(_al(_kWhite, .5)));
}

void _wHypno(Canvas c, Size s, _St st, double t) {
  final ctr = Offset(s.width / 2, s.height / 2), tw = 1 + st.spin * .8 + st.a * 5;
  final rad = 50 + 35 * st.b * (.5 + .5 * math.sin(t * 3));
  const arms = 10;
  for (var k = 0; k < arms; k++) {
    final a0 = k * 2 * math.pi / arms + t * .9, a1 = (k + 1) * 2 * math.pi / arms + t * .9;
    final pts = <Offset>[];
    for (var i = 0; i <= 20; i++) {
      final r = i / 20 * rad;
      pts.add(ctr + _dir(a0 + tw * r / rad) * r);
    }
    for (var i = 20; i >= 0; i--) {
      final r = i / 20 * rad;
      pts.add(ctr + _dir(a1 + tw * r / rad) * r);
    }
    c.drawPath(_poly(pts), _f(k.isEven ? _kPink : _kYellow));
  }
  c.drawCircle(ctr, 5, _f(_kCyan));
}

Color _posterAt(Offset q) {
  if ((q - const Offset(.62, .38)).distance < .15) return _kYellow;
  if (q.dy > .62 - .12 * math.sin(q.dx * 6)) return q.dy > .8 ? _kOrange : _kViolet;
  return q.dy < .3 ? _kPink : const Color(0xFFFF8FC0);
}

void _wCrumple(Canvas c, Size s, _St st, double t) {
  const nx = 9, ny = 7;
  final w = s.width, h = s.height, rect = Rect.fromLTWH(w * .1, h * .1, w * .8, h * .8);
  final str = st.b, tw = st.spin * .3 + (st.a - .5) * 1.2;
  Offset V(int i, int j) {
    final b = Offset(rect.left + i * rect.width / nx, rect.top + j * rect.height / ny);
    final edge = (i == 0 || j == 0 || i == nx || j == ny) ? .5 : 1.0;
    final q = b + Offset(_hash(i, j * 3), _hash(j + 9, i)) * 8 * str * edge;
    return _tw(q, rect.center, 70, -.15 * str, tw);
  }

  c.drawRect(rect.shift(const Offset(3, 4)), _f(_al(_kNight, .6)));
  for (var j = 0; j < ny; j++) {
    for (var i = 0; i < nx; i++) {
      for (final up in [true, false]) {
        final tri = up ? [V(i, j), V(i + 1, j), V(i, j + 1)] : [V(i + 1, j), V(i + 1, j + 1), V(i, j + 1)];
        final cen = Offset((i + (up ? .33 : .66)) / nx, (j + (up ? .33 : .66)) / ny);
        final shade = _hash(i * 2 + (up ? 0 : 1), j) * str;
        final col = shade > 0 ? _mix(_posterAt(cen), _kWhite, shade * .35) : _mix(_posterAt(cen), _kInk, -shade * .45);
        c.drawPath(_poly(tri), _f(col)..isAntiAlias = false);
      }
    }
  }
}
