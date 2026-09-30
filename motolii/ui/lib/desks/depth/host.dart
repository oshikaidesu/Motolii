import 'dart:math' as math;

import 'package:flutter/foundation.dart';

import 'face.dart';
import '../../session/editor_session.dart';

/// What the Depth desk does with the document: the floor plan is `depthLayout` (each layer's point around the camera's
/// target), a press selects and grabs, a drag streams the plan's movement to the host (`stageGesture`, mode depth), which
/// previews the position or the camera's orbit and distance, and a release commits it (a cancel puts it back).
class DepthController extends ChangeNotifier implements DepthHost {
  DepthController(this.c) {
    _slice = c.slice('depth', const ['depthLayout', 'selectedId', 'selectedIds', 'capabilities'])..addListener(_changed);
  }

  final EditorSession c;
  late final DocumentSlice _slice;
  Map<String, dynamic>? _grab;
  bool _finishing = false, _gone = false;

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
      _begin({'handle': 'camera'});
      return;
    }
    if (hit < 0 || hit >= _rows.length) {
      c.command('select', {'ids': []});
      return;
    }
    final row = _rows[hit];
    c.command('select', {'ids': [row['id']]});
    if (row['locked'] == true) return;
    _begin({'ids': [row['id']]});
  }

  /// The host carries the grab (`stageGesture`, mode depth): it turns the plan's movement into the position, or the
  /// camera's orbit and distance.
  void _begin(Map<String, dynamic> grab) {
    if (!c.supports('stageGesture')) return;
    _grab = {'phase': 'begin', 'mode': 'depth', 'start': const [0.0, 0.0], 'point': const [0.0, 0.0], ...grab};
    c.command('stageGesture', _grab!);
  }

  /// [wx], [wz]: for a layer, how far it has moved on the plan; for the camera, where the eye is around the target.
  @override
  void drag(int hit, double wx, double wz) {
    final grab = _grab;
    if (grab == null) return;
    c.commandDirect('stageGesture', {...grab, 'phase': 'update', 'point': [wx, wz]}, 'depth-gesture');
  }

  @override
  void release({bool cancel = false}) => _finish(cancel);

  Future<void> _finish(bool cancel) async {
    final grab = _grab;
    if (grab == null) return;
    _grab = null;
    _finishing = true;
    try {
      await c.command('stageGesture', {...grab, 'phase': cancel ? 'cancel' : 'commit'});
    } finally {
      _finishing = false;
    }
  }

  @override
  void dispose() {
    _gone = true;
    _slice.removeListener(_changed);
    _finish(true);
    super.dispose();
  }
}
