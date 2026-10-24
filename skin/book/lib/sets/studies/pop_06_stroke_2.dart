// Part of pop_06_stroke.dart (cut there by size, as it was).
part of 'pop_06.dart';

void _s18(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_cy));
  const ctr = Offset(78, 116), cols = [_rd, _or, _ye, _li, _bl, _vi];
  final bw = _l(2, 7, st.v);
  for (var i = 0; i < cols.length; i++) {
    final rr = 74 - i * bw;
    final arc = Path()..addArc(Rect.fromCircle(center: ctr, radius: rr), math.pi, math.pi * st.w);
    if (st.inv) {
      _dash(c, arc, _s(cols[i], bw, cap: StrokeCap.butt), 6, 4);
    } else {
      c.drawPath(arc, _s(cols[i], bw + .4, cap: StrokeCap.butt));
    }
  }
  for (final o in [ctr + const Offset(-70, -4), ctr + Offset.fromDirection(math.pi + math.pi * st.w, 70)]) {
    for (final (d, r) in [(const Offset(-8, 0), 8.0), (const Offset(2, -4), 10.0), (const Offset(11, 1), 7.0)]) {
      c.drawCircle(o + d, r, _f(_wh));
    }
  }
}

void _s19(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(_pk));
  final ctr = s.center(const Offset(0, 4));
  final p = _poly([for (var i = 0; i <= 80; i++) ctr + Offset(math.cos(i * .11 + 2.6) * (12 + i * .62) * 1.25, math.sin(i * .11 + 2.6) * (12 + i * .62) * .62)]);
  final m = p.computeMetrics().first, gap = _l(6, 15, st.v), front = m.length * (1 - st.w);
  final items = <(Offset, double, bool)>[];
  for (var d = m.length; d > 0; d -= gap) {
    final tg = m.getTangentForOffset(d)!;
    items.add((tg.position, -tg.angle, d > front));
  }
  items.sort((x, y) => x.$1.dy.compareTo(y.$1.dy));
  for (final (q, ang, down) in items) {
    if (down) {
      _rotRect(c, q, ang, const Rect.fromLTWH(-1, -3, 13, 6), _f(_a(_p0, .3)));
      _rotRect(c, q, ang, const Rect.fromLTWH(-2, -3.5, 12, 6), _f(_wh));
      _rotRect(c, q, ang, const Rect.fromLTWH(1.5, -.8, 1.6, 1.6), _f(_p0), rad: 1);
    } else {
      c.drawRect(Rect.fromLTWH(q.dx - 2.5, q.dy - 14, 5, 14), _f(_wh));
      c.drawRect(Rect.fromLTWH(q.dx + 2.5, q.dy - 14, 1.6, 14), _f(_a(_p0, .35)));
      c.drawCircle(Offset(q.dx, q.dy - 10), .9, _f(_p0));
      c.drawCircle(Offset(q.dx, q.dy - 4), .9, _f(_p0));
      if (st.inv) c.drawRect(Rect.fromLTWH(q.dx - 2.5, q.dy - 14, 5, 2), _f(_ye));
    }
  }
}

void _s20(Canvas c, Size s, PopS st) {
  c.drawRect(Offset.zero & s, _f(const Color(0xFF0C0C10)));
  const pts = [Offset(12, 24), Offset(54, 102), Offset(96, 22), Offset(144, 96), Offset(118, 112)];
  for (var i = 1; i < pts.length; i++) {
    final dir = i.isOdd ? -.5 : .5;
    c.drawLine(pts[i] + Offset.fromDirection(dir, 9), pts[i] - Offset.fromDirection(dir, 9), _s(_cy, 3));
  }
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: pts.first, width: 14, height: 10), const Radius.circular(2)), _f(_p1));
  c.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center: pts.first, width: 14, height: 10), const Radius.circular(2)), _s(_rd, 1.2));
  final path = _poly(pts);
  _dash(c, path, _s(_a(_rd, .2), 1), 2, 4);
  final len = _l(.05, .6, st.w), head = (st.t * .3) % (1 + len), w = _l(1, 6, st.v);
  final seg = _trim(path, head - len, head);
  c.drawPath(seg, _s(_a(_rd, .3), w + 8));
  if (st.inv) {
    _dash(c, seg, _s(_wh, w), 5, 5, st.t * 80);
  } else {
    c.drawPath(seg, _s(_rd, w + 2));
    c.drawPath(seg, _s(_wh, w * .6));
  }
}
