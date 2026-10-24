// Part of pop_08_repeat.dart (cut there by size, as it was).
part of 'pop_08.dart';

void _rCells(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final c0 = s.center(Offset.zero), dr = s.height * .46;
  c.drawCircle(c0, dr, _f(_k0));
  c.drawCircle(c0, dr, _s(_cy, 2));
  final g = st.b * 4.99, gi = g.floor(), fr = g - gi;
  final n = 1 << gi, rc = dr * .78 / math.sqrt(n) * (n == 1 ? .55 : 1);
  final p = ((fr - .25) / .75).clamp(0.0, 1.0), sm = p * p * (3 - 2 * p);
  final rl = rc * (1 - (1 - math.sqrt1_2) * sm), off = rc * math.sqrt1_2 * sm;
  for (var i = 0; i < n; i++) {
    final r = n == 1 ? 0.0 : dr * .72 * math.sqrt((i + .5) / n), a = i * 2.39996;
    final pc = c0 + Offset(math.cos(a), math.sin(a)) * r + Offset(math.sin(t * 1.3 + i), math.cos(t * 1.1 + i)) * 1.2;
    final ax = _rn(i + gi * 31) * math.pi, d = Offset(math.cos(ax), math.sin(ax)) * off;
    for (final q in [pc + d, pc - d]) {
      c.drawCircle(q, rl, _f(_li));
      c.drawCircle(q, rl * .38, _f(const Color(0xFF4E9A2A)));
    }
    if (sm > 0 && sm < 1) c.drawCircle(pc, rl * .55, _f(_li));
  }
}

void _rCopier(Canvas c, Size s, _P8 st, double t) {
  _bg(c, s, _k1);
  final base = Offset(34, s.height - 26);
  _isoCube(c, base, 26, 30, _vi, const Color(0xFF6E52D9), const Color(0xFF5640B8));
  c.drawPath(_poly([base + const Offset(-10, -36), base + const Offset(6, -45), base + const Offset(14, -40), base + const Offset(-2, -31)]), _f(_cy));
  final n = 1 + (st.b * 8).round(), sp = st.a;
  for (var i = n - 1; i >= 0; i--) {
    final q = i / math.max(1, n - 1);
    final o = base + Offset(26 + i * (4 + sp * 9), -18 - i * (1 + sp * 5) + math.sin(t * 3 + i) * sp * 2);
    c.save();
    c.translate(o.dx, o.dy);
    c.rotate(i * sp * .22 - .1);
    final sh = _poly(const [Offset(0, 0), Offset(22, -12), Offset(40, -2), Offset(18, 10)]);
    c.drawPath(sh.shift(const Offset(1, 2)), _f(_al(_k0, .4)));
    c.drawPath(sh, _f(Color.lerp(_wh, _cr, q)!));
    c.save();
    c.transform(Float64List.fromList([22, -12, 0, 0, 18, 10, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1]));
    c.drawCircle(const Offset(.5, .5), .28, _f(_pk));
    c.restore();
    c.restore();
  }
}
