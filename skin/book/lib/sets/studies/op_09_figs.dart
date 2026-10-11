// Shared drawing helpers for both OP 09 sheets: polar points, a tiny 3D rotate-and-project, coil, outline paths.
part of 'op_09.dart';

const _c0 = Offset(_cw / 2, _ch / 2);
const _tau = math.pi * 2;

Offset _pol(Offset o, double r, double a) => o + Offset(math.cos(a) * r, math.sin(a) * r);

typedef _V3 = (double, double, double);

_V3 _rx(_V3 v, double a) {
  final (x, y, z) = v;
  final c = math.cos(a), s = math.sin(a);
  return (x, y * c - z * s, y * s + z * c);
}

_V3 _ry(_V3 v, double a) {
  final (x, y, z) = v;
  final c = math.cos(a), s = math.sin(a);
  return (x * c + z * s, y, -x * s + z * c);
}

_V3 _rz(_V3 v, double a) {
  final (x, y, z) = v;
  final c = math.cos(a), s = math.sin(a);
  return (x * c - y * s, x * s + y * c, z);
}

Offset _o3(Offset o, _V3 v, double k) => o + Offset(v.$1 * k, v.$2 * k);

/// A unit circle in 3D, transformed by [f], projected and stroked.
void _ring3(Canvas c, Offset o, double k, _V3 Function(_V3) f, Color col, [double w = 1]) {
  final pts = <Offset>[];
  for (var i = 0; i <= 48; i++) {
    final a = i / 48 * _tau;
    pts.add(_o3(o, f((math.cos(a), math.sin(a), 0)), k));
  }
  _pl(c, pts, col, w: w);
}

/// Prolate-cycloid coil from [a] to [b] (horizontal); loops overlap when compressed.
void _coil(Canvas c, Offset a, Offset b, int turns, double r, Color col, [double w = 1]) {
  final pts = <Offset>[];
  final n = turns * 14;
  for (var i = 0; i <= n; i++) {
    final u = i / n, th = u * turns * _tau;
    pts.add(Offset.lerp(a, b, u)! + Offset(-math.sin(th) * r * .55, -math.cos(th) * r));
  }
  _pl(c, pts, col, w: w);
}

/// Strokes a closed shape given as unit points, mapped through [m].
void _shape(Canvas c, List<Offset> unit, Offset Function(Offset) m, Color col, {double w = 1, bool close = true}) =>
    _pl(c, [for (final p in unit) m(p)], col, w: w, close: close);

/// Dotted closed circle.
void _dring(Canvas c, Offset o, double r, Color col, {int n = 60, double w = 1}) =>
    c.drawPoints(ui.PointMode.points, [for (var i = 0; i < n; i++) _pol(o, r, i / n * _tau)], _s(col, w));

/// Small arrowhead at [p] pointing along [dir] (radians).
void _arrow(Canvas c, Offset p, double dir, Color col, [double sz = 4]) {
  _ln(c, p, _pol(p, sz, dir + math.pi * .8), col, 1);
  _ln(c, p, _pol(p, sz, dir - math.pi * .8), col, 1);
}
