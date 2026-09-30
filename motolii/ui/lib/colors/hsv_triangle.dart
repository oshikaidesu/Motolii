import 'package:flutter/widgets.dart';

/// The colour triangle inside a hue ring: corners hue, white and black ([t] in that order). Saturation and value are
/// barycentric weights, so picking, the handle and a vertex-coloured paint agree. Every colour wheel uses this.

/// Barycentric weights of [p] against the triangle (hue, white, black).
List<double> triangleWeights(Offset p, List<Offset> t) {
  final d = (t[1].dy - t[2].dy) * (t[0].dx - t[2].dx) + (t[2].dx - t[1].dx) * (t[0].dy - t[2].dy);
  final a = ((t[1].dy - t[2].dy) * (p.dx - t[2].dx) + (t[2].dx - t[1].dx) * (p.dy - t[2].dy)) / d;
  final b = ((t[2].dy - t[0].dy) * (p.dx - t[2].dx) + (t[0].dx - t[2].dx) * (p.dy - t[2].dy)) / d;
  return [a, b, 1 - a - b];
}

Offset _closestOn(Offset p, Offset a, Offset b) {
  final edge = b - a;
  final length2 = edge.dx * edge.dx + edge.dy * edge.dy;
  if (length2 == 0) return a;
  final u = (((p - a).dx * edge.dx + (p - a).dy * edge.dy) / length2).clamp(0.0, 1.0);
  return a + edge * u;
}

/// [p] itself inside the triangle, else the nearest point on its edge.
Offset insideTriangle(Offset p, List<Offset> t) {
  if (triangleWeights(p, t).every((weight) => weight >= 0)) return p;
  final candidates = [_closestOn(p, t[0], t[1]), _closestOn(p, t[1], t[2]), _closestOn(p, t[2], t[0])];
  candidates.sort((a, b) => (a - p).distanceSquared.compareTo((b - p).distanceSquared));
  return candidates.first;
}

/// The colour at [p] (the hue kept from [hsv]).
HSVColor pickInTriangle(Offset p, List<Offset> t, HSVColor hsv) {
  final w = triangleWeights(insideTriangle(p, t), t);
  final a = w[0].clamp(0.0, 1.0), b = w[1].clamp(0.0, 1.0);
  final value = a + b;
  return hsv.withValue(value.clamp(0.0, 1.0)).withSaturation(value == 0 ? 0 : (a / value).clamp(0.0, 1.0));
}

/// Where [hsv] sits in the triangle.
Offset triangleHandle(List<Offset> t, HSVColor hsv) {
  final a = hsv.saturation * hsv.value, b = hsv.value - a;
  return t[0] * a + t[1] * b + t[2] * (1 - a - b);
}
