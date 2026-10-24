part of 'op_02.dart';

/// Line figure standing on [foot], [h] tall; arm / leg swing in radians.
void _man(Canvas c, Offset foot, double h, Color col, {double arm = .5, double leg = .35, bool sit = false, double w = 1.1}) {
  final p = _sp(col, w), hr = h * .13;
  final hip = foot - Offset(0, sit ? h * .25 : h * .45), neck = hip - Offset(0, h * .32), head = neck - Offset(0, hr);
  c.drawCircle(head, hr, p);
  c.drawLine(neck, hip, p);
  final sh = neck + Offset(0, h * .06);
  c.drawLine(sh, sh + _pol(math.pi / 2 + arm, h * .26), p);
  c.drawLine(sh, sh + _pol(math.pi / 2 - arm, h * .26), p);
  if (sit) {
    c.drawLine(hip, hip + Offset(h * .2, 0), p);
    c.drawLine(hip + Offset(h * .2, 0), foot + Offset(h * .2, 0), p);
  } else {
    c.drawLine(hip, hip + _pol(math.pi / 2 + leg, h * .47), p);
    c.drawLine(hip, hip + _pol(math.pi / 2 - leg, h * .47), p);
  }
}

Path _superellipse(Offset o, double r, double n, [int seg = 48]) {
  final pts = <Offset>[];
  for (var i = 0; i < seg; i++) {
    final th = i / seg * 2 * math.pi, cs = math.cos(th), sn = math.sin(th);
    pts.add(o +
        Offset(cs.sign * math.pow(cs.abs(), 2 / n).toDouble() * r, sn.sign * math.pow(sn.abs(), 2 / n).toDouble() * r));
  }
  return Path()..addPolygon(pts, true);
}
