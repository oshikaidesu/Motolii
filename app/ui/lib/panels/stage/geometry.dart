import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';

import '../../foundation/theme.dart';

/// The arithmetic of shapes on the Stage: what native's numbers mean as
/// points, which polygon a press is inside, and the gizmo mesh it hit-tests
/// against. Nothing here knows the panel, its view or its gestures.
double numOf(dynamic value, [double fallback = 0]) =>
    value is num ? value.toDouble() : fallback;

double segmentDistance(Offset p, Offset a, Offset b) {
  final ab = b - a;
  final t = ab.distanceSquared == 0
      ? 0.0
      : ((p - a).dx * ab.dx + (p - a).dy * ab.dy) / ab.distanceSquared;
  return (p - (a + ab * t.clamp(0.0, 1.0))).distance;
}

bool inTriangle(Offset p, Offset a, Offset b, Offset c) {
  double side(Offset u, Offset v) =>
      (v.dx - u.dx) * (p.dy - u.dy) - (v.dy - u.dy) * (p.dx - u.dx);
  final s = [side(a, b), side(b, c), side(c, a)];
  return s.every((v) => v >= 0) || s.every((v) => v <= 0);
}

/// The gizmo mesh as native sent it: comp-space vertices, linear colours
/// already converted to sRGB, and only the triangles that are drawable.
class SpatialMesh {
  const SpatialMesh(this.vertices, this.colors, this.indices);
  final List<Offset> vertices;
  final List<Color> colors;
  final Uint16List indices;

  static SpatialMesh? parse(dynamic raw, EditorTheme theme) {
    if (raw is! Map) return null;
    final v = raw['vertices'], c = raw['colors'], i = raw['indices'];
    if (v is! List || c is! List || i is! List) return null;
    if (v.isEmpty || v.length > 0xFFFF || i.length < 3) return null;
    final vertices = [
      for (final p in v)
        p is List && p.length >= 2
            ? Offset(numOf(p[0], double.nan), numOf(p[1], double.nan))
            : const Offset(double.nan, double.nan),
    ];
    double channel(dynamic x) =>
        math.pow(numOf(x).clamp(0.0, 1.0), 1 / 2.2).toDouble();
    int byte(dynamic x) => (channel(x) * 255).round().clamp(0, 255);
    // Native sends meaning, not paint: white is the gizmo at rest, black is
    // the hovered part. At rest it stays quiet; the accent says
    // "this one will move if you press". Anything else is passed through.
    Color tone(List rgba) {
      final alpha = (numOf(rgba[3], 1).clamp(0.0, 1.0) * 255).round();
      final rgb = [for (var k = 0; k < 3; k++) numOf(rgba[k])];
      if (rgb.every((v) => v >= 0.999)) return theme.muted.withAlpha(alpha);
      if (rgb.every((v) => v <= 0.001)) return theme.accent.withAlpha(alpha);
      return Color.fromARGB(alpha, byte(rgb[0]), byte(rgb[1]), byte(rgb[2]));
    }

    final colors = [
      for (var k = 0; k < vertices.length; k++)
        k < c.length && c[k] is List && (c[k] as List).length >= 4
            ? tone(c[k] as List)
            : EditorTheme.clear,
    ];
    final indices = <int>[];
    for (var t = 0; t + 2 < i.length; t += 3) {
      final tri = [for (var k = 0; k < 3; k++) (i[t + k] as num).toInt()];
      if (tri.any((x) => x < 0 || x >= vertices.length)) continue;
      if (tri.any((x) => !vertices[x].dx.isFinite || !vertices[x].dy.isFinite))
        continue;
      indices.addAll(tri);
    }
    if (indices.isEmpty) return null;
    return SpatialMesh(vertices, colors, Uint16List.fromList(indices));
  }

  ui.Vertices toScreen(Offset origin, double scale) => ui.Vertices(
    VertexMode.triangles,
    [for (final p in vertices) origin + p * scale],
    colors: colors,
    indices: indices,
  );
}

double _cross(Offset o, Offset a, Offset b) =>
    (a.dx - o.dx) * (b.dy - o.dy) - (a.dy - o.dy) * (b.dx - o.dx);
List<Offset> hull(List<Offset> points) {
  if (points.length < 3) return points;
  final sorted = points.toSet().toList()
    ..sort(
      (a, b) => a.dx == b.dx ? a.dy.compareTo(b.dy) : a.dx.compareTo(b.dx),
    );
  final lower = <Offset>[], upper = <Offset>[];
  for (final p in sorted) {
    while (lower.length >= 2 &&
        _cross(lower[lower.length - 2], lower.last, p) <= 0) {
      lower.removeLast();
    }
    lower.add(p);
  }
  for (final p in sorted.reversed) {
    while (upper.length >= 2 &&
        _cross(upper[upper.length - 2], upper.last, p) <= 0) {
      upper.removeLast();
    }
    upper.add(p);
  }
  return [...lower.take(lower.length - 1), ...upper.take(upper.length - 1)];
}

bool polygonContains(List<Offset> polygon, Offset point) {
  if (polygon.length < 3) return false;
  bool inside = false;
  for (var i = 0, j = polygon.length - 1; i < polygon.length; j = i++) {
    final a = polygon[i], b = polygon[j];
    if ((a.dy > point.dy) != (b.dy > point.dy) &&
        point.dx < (b.dx - a.dx) * (point.dy - a.dy) / (b.dy - a.dy) + a.dx)
      inside = !inside;
  }
  return inside;
}

bool polygonIntersects(List<Offset> polygon, Rect box) {
  if (polygon.isEmpty || box.width < .5 && box.height < .5) return false;
  if (polygon.any(box.contains)) return true;
  final corners = [box.topLeft, box.topRight, box.bottomRight, box.bottomLeft];
  if (corners.any((p) => polygonContains(polygon, p))) return true;
  for (var i = 0; i < polygon.length; i++) {
    final a = polygon[i], b = polygon[(i + 1) % polygon.length];
    for (var j = 0; j < 4; j++) {
      final c = corners[j], d = corners[(j + 1) % 4];
      if (_cross(a, b, c) * _cross(a, b, d) < 0 &&
          _cross(c, d, a) * _cross(c, d, b) < 0)
        return true;
    }
  }
  return false;
}
