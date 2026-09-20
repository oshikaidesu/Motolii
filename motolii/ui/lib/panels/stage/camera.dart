part of '../stage.dart';

/// Grabbing what is not a layer: the camera box (Boxcam), the orbit of the
/// observer, and the edges of the working area. Every edit here goes out as
/// a preview and is committed or cancelled as one gesture.
mixin _StageCamera on State<StagePanel>, _StageView, _StageGrip {
  bool _orbiting = false, _orbitSending = false;
  List<double>? _orbitPending;
  List<double> _orbitAngles = [-15, 30];
  Future<void> _sendOrbit() async {
    if (_orbitSending) return;
    _orbitSending = true;
    try {
      while (_orbitPending != null && mounted) {
        final angles = _orbitPending!;
        _orbitPending = null;
        await c.command('stageView', {'orbit': angles});
      }
    } finally {
      _orbitSending = false;
    }
  }

  /// Boxcam の working comp: 縁を掴んで、その向きの余白を書く。
  int? _extentEdge(Offset p) {
    final box = _extentPoints();
    if (box.length != 4) return null;
    for (var i = 0; i < 4; i++) {
      if (segmentDistance(p, box[i], box[(i + 1) % 4]) < 6) return i;
    }
    return null;
  }

  Future<void> _toggleExtend() async {
    final on = !_extend;
    await c.storeDesk('stageExtend', on);
    if (on && _extent == null && c.supports('create')) {
      await c.command('create', {'kind': 'stage'});
    }
  }

  /// The box where the camera looks on the composition plane. A corner the
  /// observer cannot see is null; authoring needs all four. The eye is drawn
  /// only when it lies in front of the observer.
  List<Offset?> _cameraCorners(Map<String, dynamic> camera) =>
      (camera['points'] as List).map(_point).toList();
  List<Offset> _cameraPoints(Map<String, dynamic> camera) {
    final corners = _cameraCorners(camera);
    return corners.length == 4 && corners.every((c) => c != null)
        ? corners.cast<Offset>()
        : const [];
  }

  Offset? _cameraEye(Map<String, dynamic> camera) => _point(camera['eye']);

  /// Blender's camera: a pyramid from the eye to a frame at a fixed display
  /// distance, and a triangle marking up. Drawn whenever the eye is in view.
  List<Offset?> _cameraFrustum(Map<String, dynamic> camera) =>
      (camera['frustum'] as List? ?? const []).map(_point).toList();
  Offset? _cameraUp(Map<String, dynamic> camera) => _point(camera['up']);

  /// 箱で author できるのは、正面を向いて注視点が自前のカメラだけ。回した物と層を見ている物は eye と frustum を見せるだけ。
  Map<String, dynamic>? get _selectedCamera {
    for (final camera in _cameras) {
      if (c.selectedIds.contains(camera['id']) &&
          camera['authorable'] != false) {
        return camera;
      }
    }
    return null;
  }

  /// Boxcam: 正面で見る時、カメラは comp 面に置いた箱。辺で掴んで Center、角で Zoom、上の取っ手で Roll。
  Map<String, Offset> _cameraHandles(Map<String, dynamic> camera) {
    final box = _cameraPoints(camera);
    if (box.length != 4) return {};
    final centre = box.reduce((a, b) => a + b) / 4;
    final top = (box[0] + box[1]) / 2;
    final up = top - centre;
    return {
      for (var i = 0; i < 4; i++) 'zoom$i': box[i],
      'roll':
          top + (up.distance > 0 ? up / up.distance : const Offset(0, -1)) * 22,
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

  Map<String, dynamic>? _cameraDrag, _extentDrag;
  String _cameraHandle = 'center';
  int _extentSide = 0;
  List<Map<String, dynamic>>? _cameraPending;
  bool _cameraSending = false;
  Future<void> _sendCamera() async {
    if (_cameraSending) return;
    _cameraSending = true;
    try {
      while (_cameraPending != null && mounted) {
        final edits = _cameraPending!;
        _cameraPending = null;
        await c.command('previewProperties', {'edits': edits});
      }
    } finally {
      _cameraSending = false;
    }
  }

  void _moveExtent(Offset screen) {
    final extent = _extentDrag!;
    final delta = (_toComp(screen) - _toComp(_startScreen!)) * _observerScale;
    // 上辺 0・右辺 1・下辺 2・左辺 3 → 余白 [左, 上, 右, 下]
    final (index, outward) = switch (_extentSide) {
      0 => (1, -delta.dy),
      1 => (2, delta.dx),
      2 => (3, delta.dy),
      _ => (0, -delta.dx),
    };
    final margins = extent['margins'] as List;
    _cameraPending = [
      {
        'layer': extent['layer'],
        'property': [
          'stage.left',
          'stage.top',
          'stage.right',
          'stage.bottom',
        ][index],
        'value': math.max(0, numOf(margins[index]) + outward),
      },
    ];
    _sendCamera();
  }

  void _moveCamera(Offset screen) {
    if (_extentDrag != null) return _moveExtent(screen);
    final camera = _cameraDrag!;
    final box = _cameraPoints(camera);
    if (box.length != 4) return;
    final centre = box.reduce((a, b) => a + b) / 4;
    final start = _startScreen!;
    final edits = <Map<String, dynamic>>[];
    switch (_cameraHandle) {
      case 'roll':
        final delta =
            (math.atan2(screen.dy - centre.dy, screen.dx - centre.dx) -
                math.atan2(start.dy - centre.dy, start.dx - centre.dx)) *
            180 /
            math.pi;
        edits.add({
          'layer': camera['id'],
          'property': 'camera.roll',
          'value': numOf(camera['roll']) - delta,
        });
      case 'center':
        final delta = (_toComp(screen) - _toComp(start)) * _observerScale;
        final center = camera['center'] as List;
        edits.add({
          'layer': camera['id'],
          'property': 'camera.center',
          'value': [numOf(center[0]) + delta.dx, numOf(center[1]) + delta.dy],
        });
      default:
        final ratio =
            (screen - centre).distance / math.max(1, (start - centre).distance);
        edits.add({
          'layer': camera['id'],
          'property': 'camera.zoom',
          'value': numOf(camera['zoom'], 1) / math.max(ratio, .01),
        });
    }
    _cameraPending = edits;
    _sendCamera();
  }

  Future<void> _finishCamera(bool cancel) async {
    if (_cameraDrag == null && _extentDrag == null) return;
    _cameraDrag = null;
    _extentDrag = null;
    _finishing = true;
    if (cancel) _cameraPending = null;
    while (_cameraSending) {
      await Future<void>.delayed(Duration.zero);
    }
    await c.command(cancel ? 'cancelPreview' : 'commitPreview');
    if (mounted) setState(() => _finishing = false);
    _pointer = null;
  }
}
