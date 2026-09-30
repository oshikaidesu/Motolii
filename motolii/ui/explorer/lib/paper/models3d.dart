// Spike: a 3D asset's own face, drawn by flutter_scene from the .glb itself, framed from its bounds. Presentation only:
// nothing here edits, places or renders for the product; it is one small view per file.
import 'dart:io';
import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter_scene/scene.dart';
import 'package:vector_math/vector_math.dart' show Aabb3, Vector3;

import '../story.dart' show fixture;

const models = ['Camera_01', 'food_apple_01', 'ArmChair_01', 'tea_set_01'];

/// One model, alone in its own scene, the camera placed from the model's bounds (a three-quarter view).
class ModelFace extends StatefulWidget {
  const ModelFace(this.name, {super.key});
  final String name;
  @override
  State<ModelFace> createState() => _ModelFaceState();
}

class _ModelFaceState extends State<ModelFace> {
  final scene = Scene();
  Node? node;
  PerspectiveCamera? camera;
  Object? failed;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    try {
      // the .gltf as Poly Haven delivers it: its .bin and textures resolved from beside it
      final dir = fixture('models/${widget.name}');
      final n = await Node.fromGltfBytes(
        await File('$dir/${widget.name}_1k.gltf').readAsBytes(),
        resolveUri: (uri) => File('$dir/$uri').readAsBytes(),
      );
      final b = n.combinedWorldBounds;
      if (b == null) throw StateError('no bounds');
      scene.add(n);
      if (mounted) setState(() {
        node = n;
        camera = _frame(b, Vector3(.55, .35, -1));
      });
    } catch (e) {
      if (mounted) setState(() => failed = e);
    }
  }

  /// Fills the square by what the model actually projects to (not its bounding sphere), so a wide set is not small.
  PerspectiveCamera _frame(Aabb3 b, Vector3 dir) {
    const fov = 40 * 3.1415926535 / 180;
    final d = dir.normalized(), c = b.center;
    final right = Vector3(0, 1, 0).cross(d).normalized(), up = d.cross(right).normalized();
    // the distance at which every corner of the bounds is inside the square view
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
    if (failed != null) return Center(child: Text('$failed', style: const TextStyle(fontSize: 9, color: Color(0xFFFF8080))));
    final cam = camera;
    return cam == null ? const SizedBox.expand() : SceneView(scene, camera: cam);
  }
}
