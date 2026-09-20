part of '../stage.dart';

/// The rendered picture for one view. Listens to the texture table alone:
/// a frame, a selection or a zoom leaves it standing.
class _StageTexture extends StatelessWidget {
  const _StageTexture({required this.textureIds, required this.view});
  final ValueListenable<Map<String, int>> textureIds;
  final String view;
  @override
  Widget build(BuildContext context) =>
      ValueListenableBuilder<Map<String, int>>(
        valueListenable: textureIds,
        builder: (context, ids, _) => switch (ids[view]) {
          null => Center(
            child: Text(
              'No rendered texture',
              style: TextStyle(
                fontSize: EditorMetrics.font,
                color: EditorTheme.of(context).muted,
              ),
            ),
          ),
          final id => RepaintBoundary(
            child: Texture(textureId: id, filterQuality: FilterQuality.low),
          ),
        },
      );
}

class _StageOverlay extends CustomPainter {
  final EditorTheme colors;

  const _StageOverlay({
    this.colors = EditorTheme.chromatic,
    this.dimOutside = false,
    this.cameras = const [],
    this.cameraEyes = const [],
    this.cameraFrustums = const [],
    this.cameraUps = const [],
    this.cameraTargets = const [],
    this.cameraHandles = const {},
    this.front = true,
    this.extent = const [],
    this.extendable = false,
    this.observerTarget,
    this.anchorPreview,
    required this.outlines,
    required this.handles,
    required this.frame,
    required this.viewport,
    this.spatialMesh,
    this.marquee,
    this.snapGuides = const [],
    this.ink = EditorInk.dark,
  });
  final EditorInk ink;
  final List<List<Offset>> outlines;
  final List<List<Offset?>> cameras, cameraFrustums;
  final List<Offset?> cameraEyes, cameraUps;
  final List<Offset> cameraTargets, frame, extent;
  final bool extendable;

  /// The Stage keeps the world around the frame visible and greys it: the
  /// frame is a reference, not a crop (Boxcam).
  final bool dimOutside;
  final Map<String, Offset> handles, cameraHandles;
  final bool front;
  final Offset? observerTarget;

  /// The pivot a hovered anchor cell would set, drawn as a cross.
  final Offset? anchorPreview;
  final Rect viewport;

  /// The 3D gizmo exactly as the native side hit-tests it, already on screen.
  final ui.Vertices? spatialMesh;
  final Rect? marquee;

  /// The composition lines a Cmd-held move is holding to, already on screen.
  final List<(Offset, Offset)> snapGuides;
  void _cross(Canvas canvas, Offset at, double half, Paint paint) {
    canvas.drawLine(at - Offset(half, 0), at + Offset(half, 0), paint);
    canvas.drawLine(at - Offset(0, half), at + Offset(0, half), paint);
  }

  @override
  void paint(Canvas canvas, Size size) {
    final framePaint = Paint()
      ..style = PaintingStyle.stroke
      ..color = colors.line
      ..strokeWidth = 1;
    final framePath = frame.length == 4
        ? (Path()..addPolygon(frame, true))
        : (Path()..addRect(viewport));
    if (dimOutside) {
      canvas.drawPath(
        Path()
          ..fillType = PathFillType.evenOdd
          ..addRect(Offset.zero & size)
          ..addPath(framePath, Offset.zero),
        Paint()..color = colors.app.withValues(alpha: .55),
      );
    }
    canvas.drawPath(framePath, framePaint);
    final line = Paint()
      ..style = PaintingStyle.stroke
      ..color = colors.accent
      ..strokeWidth = 1;
    if (anchorPreview case final at?) {
      _cross(canvas, at, EditorMetrics.s8, line);
      canvas.drawCircle(at, EditorMetrics.s3, line);
    }
    if (extent.length == 4) {
      canvas.drawPath(
        Path()..addPolygon(extent, true),
        Paint()
          ..style = PaintingStyle.stroke
          ..color = extendable ? colors.ink : colors.muted
          ..strokeWidth = extendable ? 2 : 1,
      );
    }
    final cameraLine = Paint()
      ..color = ink.camera
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;
    for (final at in cameraTargets) {
      _cross(canvas, at, 5, cameraLine);
    }
    if (observerTarget != null) {
      canvas.drawCircle(observerTarget!, 6, cameraLine);
      _cross(canvas, observerTarget!, 10, cameraLine);
    }
    for (final entry in cameraHandles.entries) {
      if (entry.key == 'roll') {
        canvas.drawCircle(entry.value, 4, Paint()..color = colors.app);
        canvas.drawCircle(entry.value, 4, cameraLine);
      } else {
        final rect = Rect.fromCenter(
          center: entry.value,
          width: EditorMetrics.s6,
          height: EditorMetrics.s6,
        );
        canvas.drawRect(rect, Paint()..color = colors.app);
        canvas.drawRect(rect, cameraLine);
      }
    }
    for (final (index, box) in cameras.indexed) {
      if (box.length != 4) continue;
      // Corners the observer cannot see are left open: the box is drawn as
      // far as it is known.
      for (var i = 0; i < 4; i++) {
        final a = box[i], b = box[(i + 1) % 4];
        if (a != null && b != null) canvas.drawLine(a, b, cameraLine);
      }
      // Blender's camera object: eye, a frame at its display distance, the
      // four edges between them, and a triangle above the frame for up.
      final eye = index < cameraEyes.length ? cameraEyes[index] : null;
      final frustum = index < cameraFrustums.length
          ? cameraFrustums[index]
          : const <Offset?>[];
      if (eye == null || frustum.length != 4 || frustum.any((p) => p == null))
        continue;
      final frame = frustum.cast<Offset>();
      canvas.drawPath(Path()..addPolygon(frame, true), cameraLine);
      for (final corner in frame) {
        canvas.drawLine(eye, corner, cameraLine);
      }
      final up = index < cameraUps.length ? cameraUps[index] : null;
      if (up != null) {
        canvas.drawPath(
          Path()..addPolygon([frame[0], frame[1], up], true),
          cameraLine,
        );
      }
    }
    for (final points in outlines) {
      if (points.isEmpty) continue;
      final path = Path()..moveTo(points.first.dx, points.first.dy);
      for (final p in points.skip(1)) {
        path.lineTo(p.dx, p.dy);
      }
      path.close();
      canvas.drawPath(path, line);
    }
    if (spatialMesh != null) {
      canvas.drawVertices(spatialMesh!, BlendMode.srcOver, Paint());
    }
    for (final entry in handles.entries) {
      if (entry.key == 'rotation') {
        canvas.drawCircle(entry.value, 4, Paint()..color = colors.app);
        canvas.drawCircle(entry.value, 4, line);
      } else {
        final rect = Rect.fromCenter(
          center: entry.value,
          width: EditorMetrics.s6,
          height: EditorMetrics.s6,
        );
        canvas.drawRect(rect, Paint()..color = colors.app);
        canvas.drawRect(rect, line);
      }
    }
    for (final (a, b) in snapGuides) {
      canvas.drawLine(a, b, line);
    }
    if (marquee != null) {
      canvas.drawRect(
        marquee!,
        Paint()..color = colors.accent.withValues(alpha: .12),
      );
      canvas.drawRect(marquee!, line);
    }
  }

  @override
  bool shouldRepaint(covariant _StageOverlay old) =>
      colors != old.colors ||
      old.ink != ink ||
      !_samePolylines(old.cameras, cameras) ||
      !_samePolylines(old.cameraFrustums, cameraFrustums) ||
      !listEquals(old.cameraEyes, cameraEyes) ||
      !listEquals(old.cameraUps, cameraUps) ||
      !listEquals(old.cameraTargets, cameraTargets) ||
      !mapEquals(old.cameraHandles, cameraHandles) ||
      old.front != front ||
      !listEquals(old.extent, extent) ||
      old.extendable != extendable ||
      old.observerTarget != observerTarget ||
      old.anchorPreview != anchorPreview ||
      old.dimOutside != dimOutside ||
      !listEquals(old.frame, frame) ||
      old.viewport != viewport ||
      old.marquee != marquee ||
      !listEquals(old.snapGuides, snapGuides) ||
      !_samePolylines(old.outlines, outlines) ||
      !mapEquals(old.handles, handles) ||
      !identical(old.spatialMesh, spatialMesh);

  static bool _samePolylines<T>(List<List<T>> a, List<List<T>> b) {
    if (identical(a, b)) return true;
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (!listEquals(a[i], b[i])) return false;
    }
    return true;
  }
}
