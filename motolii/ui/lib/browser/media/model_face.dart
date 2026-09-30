import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter_scene/scene.dart';
import 'package:vector_math/vector_math.dart' show Aabb3, Vector3;

import '../../hf/neutral.dart';

/// A 3D model's own face for the shelf: the file itself, drawn small by flutter_scene, framed from its bounds so it fills
/// the tile, over the tile's own ground (no background of its own). Presentation only: it reads the file and draws it; it
/// edits, places and renders nothing for the work. A file flutter_scene cannot read (its importer takes glTF/GLB) keeps
/// [fallback].
class ModelFace extends StatefulWidget {
  const ModelFace({super.key, required this.path, required this.fallback, this.turnable = false, this.hoverYaw});
  final String path;
  final Widget fallback;

  /// A drag turns the model (the preview's live face); a shelf tile stays still.
  final bool turnable;

  /// A pointer passing over a shelf tile turns the model by this much (0..1 across the tile), without a click.
  final double? hoverYaw;

  /// Whether the face can be drawn from this file (GLB/glTF): the others keep their glyph without trying.
  static bool reads(String path) {
    final p = path.toLowerCase();
    return p.endsWith('.glb') || p.endsWith('.gltf');
  }

  @override
  State<ModelFace> createState() => _ModelFaceState();
}

class _ModelFaceState extends State<ModelFace> {
  final scene = Scene();
  Node? node;
  Aabb3? _bounds;
  double _yaw = .55, _pitch = .35;
  bool failed = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      final file = File(widget.path);
      final n = widget.path.toLowerCase().endsWith('.glb')
          ? await Node.fromGlbBytes(await file.readAsBytes())
          : await Node.fromGltfBytes(await file.readAsBytes(), resolveUri: (uri) => File('${file.parent.path}/$uri').readAsBytes());
      final bounds = n.combinedWorldBounds;
      if (bounds == null) throw StateError('no bounds');
      scene.add(n);
      if (mounted) {
        setState(() {
          node = n;
          _bounds = bounds;
        });
      }
    } catch (_) {
      if (mounted) setState(() => failed = true);
    }
  }

  Vector3 _direction() {
    final yaw = widget.hoverYaw == null ? _yaw : _yaw + (widget.hoverYaw! - .5) * 2.4;
    return Vector3(math.sin(yaw) * math.cos(_pitch), math.sin(_pitch), -math.cos(yaw) * math.cos(_pitch));
  }

  /// The distance at which every corner of the bounds is inside the square view (the model, not its bounding sphere).
  PerspectiveCamera _frame(Aabb3 b, Vector3 dir, [double aspect = 1]) {
    const fov = 40 * math.pi / 180;
    final d = dir.normalized(), c = b.center;
    final right = Vector3(0, 1, 0).cross(d).normalized(), up = d.cross(right).normalized();
    var dist = 0.0;
    for (final x in [b.min.x, b.max.x]) {
      for (final y in [b.min.y, b.max.y]) {
        for (final z in [b.min.z, b.max.z]) {
          final v = Vector3(x, y, z) - c;
          // fitted to the view's own shape: the horizontal field is `aspect` times the vertical one
          dist = math.max(dist, v.dot(d) + math.max(v.dot(right).abs() / aspect, v.dot(up).abs()) / math.tan(fov / 2));
        }
      }
    }
    dist *= 1.03;
    return PerspectiveCamera(fovRadiansY: fov, position: c + d * dist, target: c, fovNear: math.max(dist * .05, 1e-3), fovFar: dist * 4);
  }

  @override
  void dispose() {
    node?.detach();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (failed) return widget.fallback;
    final bounds = _bounds;
    if (bounds == null) return ColoredBox(color: N.g13, child: widget.fallback);
    return LayoutBuilder(builder: (context, box) {
      final aspect = box.maxHeight > 0 && box.maxWidth.isFinite ? box.maxWidth / box.maxHeight : 1.0;
      final cam = _frame(bounds, _direction(), aspect);
      final view = ColoredBox(color: N.g13, child: SceneView(scene, camera: cam));
      if (!widget.turnable) return view;
      return MouseRegion(
        cursor: SystemMouseCursors.grab,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onPanUpdate: (d) => setState(() {
            _yaw += d.delta.dx * .012;
            _pitch = (_pitch + d.delta.dy * .012).clamp(-1.2, 1.2);
          }),
          child: view,
        ),
      );
    });
  }
}
