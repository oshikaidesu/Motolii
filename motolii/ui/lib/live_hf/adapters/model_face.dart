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
  const ModelFace({super.key, required this.path, required this.fallback});
  final String path;
  final Widget fallback;

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
          camera = _frame(bounds, Vector3(.55, .35, -1));
        });
      }
    } catch (_) {
      if (mounted) setState(() => failed = true);
    }
  }

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
    return ColoredBox(color: N.g13, child: cam == null ? widget.fallback : SceneView(scene, camera: cam));
  }
}
