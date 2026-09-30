part of 'stage.dart';

/// Where the camera box and the working area are on screen, and what a press on them takes (the skin's hit-testing);
/// carrying them is the StageSession's.
mixin _StageCamera on State<StagePanel>, _StageView, _StageGrip {
  /// Boxcam's working comp: the edge under [p] (top 0, right 1, bottom 2, left 3).
  int? _extentEdge(Offset p) {
    final box = _extentPoints();
    if (box.length != 4) return null;
    for (var i = 0; i < 4; i++) {
      if (segmentDistance(p, box[i], box[(i + 1) % 4]) < 6) return i;
    }
    return null;
  }

  Future<void> _toggleExtend() => _session.toggleExtend();

  /// The box where the camera looks on the composition plane. A corner the observer cannot see is null.
  List<Offset?> _cameraCorners(Map<String, dynamic> camera) => (camera['points'] as List).map(_point).toList();
  List<Offset> _cameraPoints(Map<String, dynamic> camera) {
    final corners = _cameraCorners(camera);
    return corners.length == 4 && corners.every((c) => c != null) ? corners.cast<Offset>() : const [];
  }

  Offset? _cameraEye(Map<String, dynamic> camera) => _point(camera['eye']);

  /// Blender's camera: a pyramid from the eye to a frame at a fixed display distance, and a triangle marking up.
  List<Offset?> _cameraFrustum(Map<String, dynamic> camera) => (camera['frustum'] as List? ?? const []).map(_point).toList();
  Offset? _cameraUp(Map<String, dynamic> camera) => _point(camera['up']);

  Map<String, dynamic>? get _selectedCamera => _session.selectedCamera;

  /// Boxcam: seen front on, the camera is a box on the comp plane — its edge moves Center, a corner Zoom, the handle
  /// above Roll.
  Map<String, Offset> _cameraHandles(Map<String, dynamic> camera) {
    final box = _cameraPoints(camera);
    if (box.length != 4) return {};
    final centre = box.reduce((a, b) => a + b) / 4;
    final top = (box[0] + box[1]) / 2;
    final up = top - centre;
    return {
      for (var i = 0; i < 4; i++) 'zoom$i': box[i],
      'roll': top + (up.distance > 0 ? up / up.distance : const Offset(0, -1)) * 22,
    };
  }

  bool _onCameraEdge(Map<String, dynamic> camera, Offset p) {
    final box = _cameraPoints(camera);
    if (box.length != 4) return false;
    for (var i = 0; i < 4; i++) {
      if (segmentDistance(p, box[i], box[(i + 1) % 4]) < 6) return true;
    }
    return false;
  }
}
