import 'dart:math' as math;

import 'package:flutter/foundation.dart';

import '../../hf/desk/depth.dart';
import '../../session/editor_session.dart';

/// What the Depth desk does with the document: the floor plan is `depthLayout` (each layer's point around the camera's
/// target, and the basis to turn a drag on the plan into a change of its position), a press selects, a drag is a preview
/// of `position` or of the camera's orbit and distance, and a release commits it (a cancel puts it back). The operations
/// are the ones the Classic desk sends (`select`, `previewProperties`, `commitPreview`, `cancelPreview`).
class DepthController extends ChangeNotifier implements DepthHost {
  DepthController(this.c) {
    _slice = c.slice('depth', const ['depthLayout', 'selectedId', 'selectedIds', 'capabilities'])..addListener(_changed);
  }

  final EditorSession c;
  late final DocumentSlice _slice;
  Map<String, dynamic>? _grab;
  Future<void> _tail = Future.value();
  List<Map<String, dynamic>>? _pending;
  bool _sending = false, _finishing = false, _gone = false;

  Map<String, dynamic> get _data => EditorSession.map(c.state['depthLayout']);
  List<Map<String, dynamic>> get _rows => EditorSession.maps(_data['items']);
  Map<String, dynamic> get _camera => EditorSession.map(_data['camera']);

  void _changed() {
    if (!_gone) notifyListeners();
  }

  @override
  List<DepthItem> get items => [
        for (final i in _rows)
          DepthItem('${i['name']}', ((i['point'] as List)[0] as num).toDouble(), ((i['point'] as List)[1] as num).toDouble(),
              locked: i['locked'] == true, selected: c.selectedIds.contains(i['id'])),
      ];

  @override
  ({double x, double z})? get camera {
    final p = _camera['point'];
    final point = p is List ? p : const [0.0, -1000.0];
    return (x: (point[0] as num).toDouble(), z: (point[1] as num).toDouble());
  }

  /// The camera's half angle is stored as its tangent; the plan draws degrees.
  @override
  double get fovDegrees => math.atan((_data['halfFov'] as num? ?? .9).toDouble()) * 360 / math.pi;

  @override
  bool get cameraSelectable => _camera['layer'] is num;

  @override
  String? get targetName {
    final target = _camera['target'];
    if (target is! num) return null;
    return c.layers.where((l) => l['id'] == target).map((l) => '${l['name']}').firstOrNull;
  }

  @override
  void press(int hit) {
    if (_finishing || _grab != null) return;
    if (hit == -1 && cameraSelectable) {
      c.command('select', {'ids': [_camera['layer']]});
      _grab = {'camera': _camera};
      return;
    }
    if (hit < 0 || hit >= _rows.length) {
      c.command('select', {'ids': []});
      return;
    }
    final row = _rows[hit];
    c.command('select', {'ids': [row['id']]});
    if (row['locked'] == true) return;
    _grab = Map.of(row);
  }

  @override
  void drag(int hit, double wx, double wz) {
    final grab = _grab;
    if (grab == null) return;
    if (grab['camera'] is Map) {
      // The eye moves around the target: its direction is the yaw, its flat distance over cos(pitch) the distance.
      final cam = grab['camera'] as Map;
      final orbit = cam['orbit'] as List;
      final pitch = (orbit[0] as num).toDouble();
      final yaw = math.atan2(-wx, -wz) * 180 / math.pi;
      final flat = math.sqrt(wx * wx + wz * wz);
      final cos = math.max(0.05, math.cos(pitch * math.pi / 180).abs());
      final base = (cam['baseDistance'] as num).toDouble();
      _pending = [
        {'layer': cam['layer'], 'property': 'camera.orbit', 'value': [pitch, yaw]},
        {'layer': cam['layer'], 'property': 'camera.distance', 'value': (flat / cos / base).clamp(0.01, 100)},
      ];
    } else {
      final x = grab['inverseX'] as List, z = grab['inverseZ'] as List, local = grab['local'] as List;
      final next = List.generate(3, (i) => (local[i] as num).toDouble() + (x[i] as num) * wx + (z[i] as num) * wz);
      _pending = [
        {'layer': grab['id'], 'property': 'position', 'value': [next[0], next[1]]},
        {'layer': grab['id'], 'property': 'position.z', 'value': next[2]},
      ];
    }
    if (!_sending) _tail = _drain();
  }

  Future<void> _drain() async {
    if (_sending) return;
    _sending = true;
    try {
      while (_pending != null) {
        final edits = _pending!;
        _pending = null;
        await c.command('previewProperties', {'edits': edits});
      }
    } finally {
      _sending = false;
    }
  }

  @override
  void release({bool cancel = false}) => _finish(cancel);

  Future<void> _finish(bool cancel) async {
    if (_grab == null) return;
    _grab = null;
    _finishing = true;
    if (cancel) _pending = null;
    await _tail;
    await c.command(cancel ? 'cancelPreview' : 'commitPreview');
    _finishing = false;
  }

  @override
  void dispose() {
    _gone = true;
    _slice.removeListener(_changed);
    _finish(true);
    super.dispose();
  }
}
