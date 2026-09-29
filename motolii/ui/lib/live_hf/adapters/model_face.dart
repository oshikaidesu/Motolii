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
  const ModelFace({super.key, required this.path, required this.fallback, this.turnable = false});
  final String path;
  final Widget fallback;

  /// A drag turns the model (the preview's live face); a shelf tile stays still.
  final bool turnable;

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
  PerspectiveCamera? camera;
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
          camera = _frame(bounds, _direction());
        });
      }
    } catch (_) {
      if (mounted) setState(() => failed = true);
    }
  }

  Vector3 _direction() => Vector3(math.sin(_yaw) * math.cos(_pitch), math.sin(_pitch), -math.cos(_yaw) * math.cos(_pitch));

  /// The distance at which every corner of the bounds is inside the square view (the model, not its bounding sphere).
  PerspectiveCamera _frame(Aabb3 b, Vector3 dir) {
    const fov = 40 * math.pi / 180;
    final d = dir.normalized(), c = b.center;
    final right = Vector3(0, 1, 0).cross(d).normalized(), up = d.cross(right).normalized();
    var dist = 0.0;
    for (final x in [b.min.x, b.max.x]) {
      for (final y in [b.min.y, b.max.y]) {
        for (final z in [b.min.z, b.max.z]) {
          final v = Vector3(x, y, z) - c;
          dist = math.max(dist, v.dot(d) + math.max(v.dot(right).abs(), v.dot(up).abs()) / math.tan(fov / 2));
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
    final cam = camera;
    final view = ColoredBox(color: N.g13, child: cam == null ? widget.fallback : SceneView(scene, camera: cam));
    if (!widget.turnable || cam == null) return view;
    return MouseRegion(
      cursor: SystemMouseCursors.grab,
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onPanUpdate: (d) => setState(() {
          _yaw += d.delta.dx * .012;
          _pitch = (_pitch + d.delta.dy * .012).clamp(-1.2, 1.2);
          camera = _frame(_bounds!, _direction());
        }),
        child: view,
      ),
    );
  }
}
