import 'dart:math' as math;
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

/// One tab per view: `Stage` looks through the observer, `Camera` through the
/// document camera. The tab that is showing tells native which one to draw.
class StagePanel extends StatefulWidget {
  const StagePanel({super.key, required this.controller, this.view = 'User'});
  final EditorSession controller;
  final String view;
  @override
  State<StagePanel> createState() => _StagePanelState();
}

class _StagePanelState extends State<StagePanel> {
  EditorSession get c => widget.controller;

  /// The document with the rendered frame laid over it: one copy per
  /// (document, frame). Every conversion and hit test on the Stage reads it,
  /// so a hover or a scrub frame must not pay for the overlay again.
  Map<String, dynamic>? _stateCache, _stateFrom, _renderedFrom;
  Map<String, dynamic> get _state {
    if (!c.renderedIsFresh) return c.state;
    final state = c.state, rendered = c.rendered.value;
    if (!identical(state, _stateFrom) || !identical(rendered, _renderedFrom)) {
      _stateFrom = state;
      _renderedFrom = rendered;
      _stateCache = {...state, ...rendered, 'layers': c.liveLayers()};
    }
    return _stateCache!;
  }

  List<Map<String, dynamic>>? _layersCache;
  Map<String, dynamic>? _layersFrom;
  List<Map<String, dynamic>> get _layers {
    final state = _state;
    if (!identical(state, _layersFrom)) {
      _layersFrom = state;
      _layersCache = EditorSession.maps(state['layers']);
    }
    return _layersCache!;
  }

  Map<String, dynamic>? get _active {
    for (final layer in _layers) {
      if (c.selectedIds.isNotEmpty && layer['id'] == c.selectedIds.last)
        return layer;
    }
    return null;
  }

  @override
  void initState() {
    super.initState();
    c.viewCommand.addListener(_viewCommand);
    HardwareKeyboard.instance.addHandler(_heldKey);
  }

  /// The dock keeps every tab built. A tab that comes into view takes its own
  /// texture; the Stage tab also places its window, and withdraws it when hidden
  /// so native draws only the pictures somebody is looking at.
  bool _shown = false;
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final shown = Visibility.of(context);
    if (shown == _shown) return;
    _shown = shown;
    if (shown) {
      c.attachView(widget.view);
    } else if (_userStage) {
      _sentWindow = null;
      if (c.supports('stageWindow'))
        c.command('stageWindow', {'width': 0, 'height': 0});
    }
  }

  /// What the Stage tab asks native to draw: its own pixel size and the part of
  /// the composition image its zoom and pan put on screen. Sent only on change.
  Map<String, dynamic>? _sentWindow;
  void _syncWindow() {
    if (!_userStage || !_shown || !mounted || !c.supports('stageWindow'))
      return;
    if (_viewport.isEmpty) return;
    final ratio = MediaQuery.maybeDevicePixelRatioOf(context) ?? 1;
    final origin = _origin, scale = _scale;
    final window = {
      'width': (_viewport.width * ratio).round(),
      'height': (_viewport.height * ratio).round(),
      'roi': [
        -origin.dx / scale,
        -origin.dy / scale,
        _viewport.width / scale,
        _viewport.height / scale,
      ],
    };
    if (sameValue(window, _sentWindow)) return;
    _sentWindow = window;
    c.command('stageWindow', window);
  }

  void _viewCommand() {
    if (!Visibility.of(context)) return;
    final action = c.viewCommand.value;
    switch (action) {
      case 'Fit':
        _fit();
      case 'Actual':
        setState(() {
          _zoom = 1;
          _pan = Offset.zero;
        });
      case 'In':
        _zoomAt(_scale * 1.2, _viewport.center(Offset.zero));
      case 'Out':
        _zoomAt(_scale / 1.2, _viewport.center(Offset.zero));
    }
  }

  bool get _userStage => widget.view == 'User';
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

  /// 3D layer gizmo: the native side hands over the very triangles it hit-tests
  /// against, in comp coordinates. Grabbing and drawing therefore cannot drift.
  /// 2D and 2.5D layers never reach here; their cage lives in [_handles].
  /// Native sends the same map object until the gizmo changes, so identity
  /// is the cache key: the mesh is parsed once per status, not once per build.
  Object? _meshRaw;
  _SpatialMesh? _meshParsed;
  _SpatialMesh? get _mesh {
    final raw = _state[_userStage ? 'stageSpatialGizmo' : 'spatialGizmo'];
    if (!identical(raw, _meshRaw)) {
      _meshRaw = raw;
      _meshParsed = _SpatialMesh.parse(raw);
      _screenMeshKey = null;
    }
    return _meshParsed;
  }

  /// The mesh on screen, rebuilt only when the mesh or the view moves.
  (Object, Offset, double)? _screenMeshKey;
  ui.Vertices? _screenMeshCache;
  ui.Vertices? _screenMesh() {
    final mesh = _mesh;
    if (mesh == null) return null;
    final key = (mesh, _origin, _scale);
    if (_screenMeshKey != key) {
      _screenMeshKey = key;
      _screenMeshCache = mesh.toScreen(_origin, _scale);
    }
    return _screenMeshCache;
  }

  /// Only a 3D layer carries the three-axis gizmo. 2D and 2.5D layers keep the
  /// planar cage and are never reached from here.
  bool get _spatialActive {
    final layer = _active;
    if (layer == null || layer['locked'] == true) return false;
    return ['3D', 'ThreeD'].contains('${layer['projection']}');
  }

  /// What you see is what you grab: the press counts as a gizmo grab only when
  /// it lands on a drawn triangle. The native side then picks the axis.
  bool _spatialHit(Offset screen) {
    if (!_spatialActive) return false;
    final mesh = _mesh;
    if (mesh == null) return false;
    final vertices = mesh.vertices;
    final indices = mesh.indices;
    final p = _toComp(screen);
    final slack = 6 / _scale;
    for (var i = 0; i + 2 < indices.length; i += 3) {
      final a = vertices[indices[i]],
          b = vertices[indices[i + 1]],
          v = vertices[indices[i + 2]];
      if (!a.dx.isFinite || !b.dx.isFinite || !v.dx.isFinite) continue;
      if (_inTriangle(p, a, b, v)) return true;
      if (_segmentDistance(p, a, b) < slack ||
          _segmentDistance(p, b, v) < slack ||
          _segmentDistance(p, v, a) < slack)
        return true;
    }
    return false;
  }

  static bool _inTriangle(Offset p, Offset a, Offset b, Offset c) {
    double side(Offset u, Offset v) =>
        (v.dx - u.dx) * (p.dy - u.dy) - (v.dy - u.dy) * (p.dx - u.dx);
    final s = [side(a, b), side(b, c), side(c, a)];
    return s.every((v) => v >= 0) || s.every((v) => v <= 0);
  }

  /// 地が透明か。alpha が 1 未満なら書き出しは alpha を持ち、Stage は市松で見せる。
  bool get _transparentGround {
    final bg = _state['background'];
    return bg is List && bg.length == 4 && _num(bg[3], 1) < 1;
  }

  /// 透明と solid を往復する。solid へ戻す時は、透明にする前の色ではなく黒
  /// (色は Composition の欄が持つ)。1 ジェスチャ = 1 undo。
  void _toggleGround() {
    final bg = _state['background'];
    final grey = bg is List && bg.length == 4
        ? [_num(bg[0]), _num(bg[1]), _num(bg[2])]
        : [0.0, 0.0, 0.0];
    c.command('composition', {
      'background': [...grey, _transparentGround ? 1.0 : 0.0],
    });
  }

  List<Map<String, dynamic>> get _cameras =>
      _userStage ? EditorSession.maps(_state['cameraGizmos']) : [];
  Map<String, dynamic> get _observer => EditorSession.map(_state['observer']);
  bool get _front => !_userStage || _observer['front'] != false;
  bool get _home => !_userStage || _observer['home'] != false;
  double get _observerScale => _userStage ? _num(_observer['scale'], 1) : 1;
  Map<String, dynamic>? get _extent => _userStage && _observer['extent'] is Map
      ? EditorSession.map(_observer['extent'])
      : null;
  bool get _extend => c.deskWork.value['stageExtend'] == true;
  List<Offset> _extentPoints() => [
    for (final p in (_extent?['points'] as List?) ?? []) ?_point(p),
  ];

  /// Boxcam の working comp: 縁を掴んで、その向きの余白を書く。
  int? _extentEdge(Offset p) {
    final box = _extentPoints();
    if (box.length != 4) return null;
    for (var i = 0; i < 4; i++) {
      if (_segmentDistance(p, box[i], box[(i + 1) % 4]) < 6) return i;
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

  void _fit() {
    setState(() {
      _zoom = null;
      _pan = Offset.zero;
    });
    if (_userStage && c.supports('stageView')) {
      c.command('stageView', {'fit': true});
    }
  }

  Offset? _point(dynamic p) => p is List && p.length >= 2
      ? _toScreen(Offset(_num(p[0]), _num(p[1])))
      : null;

  /// The box on the look plane: four corners, always. The eye is drawn only
  /// when it lies in front of the observer.
  List<Offset> _cameraPoints(Map<String, dynamic> camera) =>
      (camera['points'] as List).map((p) => _point(p)!).toList();
  Offset? _cameraEye(Map<String, dynamic> camera) => _point(camera['eye']);

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
    final centre = box.reduce((a, b) => a + b) / 4;
    final top = (box[0] + box[1]) / 2;
    final up = top - centre;
    return {
      for (var i = 0; i < 4; i++) 'zoom$i': box[i],
      'roll':
          top + (up.distance > 0 ? up / up.distance : const Offset(0, -1)) * 22,
    };
  }

  double _segmentDistance(Offset p, Offset a, Offset b) {
    final ab = b - a;
    final t = ab.distanceSquared == 0
        ? 0.0
        : ((p - a).dx * ab.dx + (p - a).dy * ab.dy) / ab.distanceSquared;
    return (p - (a + ab * t.clamp(0.0, 1.0))).distance;
  }

  bool _onCameraEdge(Map<String, dynamic> camera, Offset p) {
    final box = _cameraPoints(camera);
    for (var i = 0; i < 4; i++) {
      if (_segmentDistance(p, box[i], box[(i + 1) % 4]) < 6) return true;
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
        'value': math.max(0, _num(margins[index]) + outward),
      },
    ];
    _sendCamera();
  }

  void _moveCamera(Offset screen) {
    if (_extentDrag != null) return _moveExtent(screen);
    final camera = _cameraDrag!;
    final centre = _cameraPoints(camera).reduce((a, b) => a + b) / 4;
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
          'value': _num(camera['roll']) - delta,
        });
      case 'center':
        final delta = (_toComp(screen) - _toComp(start)) * _observerScale;
        final center = camera['center'] as List;
        edits.add({
          'layer': camera['id'],
          'property': 'camera.center',
          'value': [_num(center[0]) + delta.dx, _num(center[1]) + delta.dy],
        });
      default:
        final ratio =
            (screen - centre).distance / math.max(1, (start - centre).distance);
        edits.add({
          'layer': camera['id'],
          'property': 'camera.zoom',
          'value': _num(camera['zoom'], 1) / math.max(ratio, .01),
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

  /// rerun 3D view: object をダブルクリックで注視、背景をダブルクリックで視点を戻す。
  /// 間隔は入力の時刻で測る。描画が詰まった時に取りこぼさない。
  Duration? _lastTapAt;
  Offset? _lastTapPos;
  bool _doubleTap(PointerDownEvent event) {
    final now = event.timeStamp;
    final again =
        _lastTapAt != null &&
        now - _lastTapAt! < kDoubleTapTimeout &&
        (_lastTapPos! - event.localPosition).distance < 6;
    _lastTapAt = again ? null : now;
    _lastTapPos = event.localPosition;
    return again;
  }

  final _focus = FocusNode(debugLabel: 'Stage');
  double? _zoom;
  Offset _pan = Offset.zero;
  Size _viewport = Size.zero;
  bool _initialFrameRequested = false;
  int? _pointer;
  Offset? _startScreen, _lastScreen, _startComp;
  Rect? _marquee;
  bool _panning = false,
      _dragging = false,
      _additive = false,
      _finishing = false,
      _sending = false;
  Map<String, dynamic>? _pending;
  Future<void> _drained = Future<void>.value();
  List<int> _ids = [];
  String _mode = 'move', _handle = 'body';
  double get _width =>
      (_num(_state['width'], 1920)).clamp(1, double.infinity).toDouble();
  double get _height =>
      (_num(_state['height'], 1080)).clamp(1, double.infinity).toDouble();
  double get _scale =>
      _zoom ??
      math.min(
        math.max(1, _viewport.width - 32) / _width,
        math.max(1, _viewport.height - 32) / _height,
      );
  Offset get _origin =>
      Offset(
        (_viewport.width - _width * _scale) / 2,
        (_viewport.height - _height * _scale) / 2,
      ) +
      _pan;
  static double _num(dynamic value, [double fallback = 0]) =>
      value is num ? value.toDouble() : fallback;
  Offset _toComp(Offset p) => (p - _origin) / _scale;
  Offset _toScreen(Offset p) => _origin + p * _scale;
  bool get _shift => HardwareKeyboard.instance.isShiftPressed;
  bool get _add =>
      _shift ||
      HardwareKeyboard.instance.isMetaPressed ||
      HardwareKeyboard.instance.isControlPressed;
  List<Offset> _rawCorners(Map<String, dynamic> layer) {
    final values =
        EditorSession.map(
          layer[_userStage ? 'stageBounds' : 'bounds'],
        )['corners'] ??
        layer['corners'];
    if (values is! List) return [];
    final points = <Offset>[];
    for (final v in values) {
      final p = v is List && v.length >= 2
          ? Offset(_num(v[0], double.nan), _num(v[1], double.nan))
          : v is Map
          ? Offset(_num(v['x'], double.nan), _num(v['y'], double.nan))
          : null;
      if (p != null && p.dx.isFinite && p.dy.isFinite) points.add(p);
    }
    return points;
  }

  List<Offset> _corners(Map<String, dynamic> layer) =>
      _hull(_rawCorners(layer));
  List<Map<String, dynamic>> get _visible =>
      _layers.where((l) => l['hidden'] != true).toList();
  Map<String, dynamic>? _hit(Offset comp) {
    for (final layer in _visible) {
      if (layer['locked'] != true && _contains(_corners(layer), comp))
        return layer;
    }
    return null;
  }

  Map<String, Offset> _handles() {
    final layer = _active;
    if (layer == null || layer['locked'] == true || !c.supports('stageGesture'))
      return {};
    final raw = _rawCorners(layer);
    final face = raw.length == 8 ? [raw[0], raw[1], raw[3], raw[2]] : raw;
    if (face.length != 4) return {};
    final projection = '${layer['projection']}';
    if (!['2D', 'TwoD', '2.5D', 'TwoPointFiveD'].contains(projection))
      return {};
    final result = <String, Offset>{
      for (var i = 0; i < 4; i++)
        ['nw', 'ne', 'se', 'sw'][i]: _toScreen(face[i]),
    };
    for (var i = 0; i < 4; i++) {
      result[['n', 'e', 's', 'w'][i]] = _toScreen(
        (face[i] + face[(i + 1) % 4]) / 2,
      );
    }
    final center = face.reduce((a, b) => a + b) / 4;
    final top = (face[0] + face[1]) / 2;
    final vector = top - center;
    result['rotation'] =
        _toScreen(top) +
        (vector.distance > 0 ? vector / vector.distance : const Offset(0, -1)) *
            22;
    return result;
  }

  double _grabTolerance(String handle) {
    if (handle == 'rotation') return 7;
    final layer = _active;
    if (layer == null) return 0;
    final corners = _corners(layer).map(_toScreen).toList();
    if (corners.isEmpty) return 0;
    final xs = corners.map((p) => p.dx), ys = corners.map((p) => p.dy);
    return math.min(
      7,
      math.min(
        (xs.reduce(math.max) - xs.reduce(math.min)) * .25,
        (ys.reduce(math.max) - ys.reduce(math.min)) * .25,
      ),
    );
  }

  Future<void> _begin(Offset point, List<int> ids, String handle) async {
    _dragging = true;
    _ids = List.of(ids);
    _handle = handle;
    _mode = handle == 'spatial'
        ? 'spatial'
        : handle == 'body'
        ? 'move'
        : handle == 'rotation'
        ? 'rotate'
        : 'scale';
    final args = _gesture('begin', point);
    _drained = c.command('stageGesture', args);
    await _drained;
  }

  Map<String, dynamic> _gesture(String phase, Offset point) => {
    'phase': phase,
    'view': widget.view,
    'mode': _mode,
    'ids': _ids,
    'start': [_startComp!.dx, _startComp!.dy],
    'point': [point.dx, point.dy],
    'handle': _handle,
    'viewScale': _scale,
    'shift': _shift,
    'alt': HardwareKeyboard.instance.isAltPressed,
    'held': _held,
  };
  void _update(Offset point) {
    _pending = _gesture('update', point);
    if (_sending) return;
    _sending = true;
    final previous = _drained;
    _drained = () async {
      try {
        await previous;
        while (_pending != null) {
          final args = _pending!;
          _pending = null;
          await c.command('stageGesture', args);
        }
      } finally {
        _sending = false;
      }
    }();
  }

  Future<void> _finish(bool cancel) async {
    if (_cameraDrag != null || _extentDrag != null) {
      return _finishCamera(cancel);
    }
    if (_finishing) return;
    _finishing = true;
    final wasDragging = _dragging;
    if (cancel) _pending = null;
    try {
      if (wasDragging) {
        await _drained;
        await c.command(
          'stageGesture',
          _gesture(
            cancel ? 'cancel' : 'commit',
            _toComp(_lastScreen ?? _startScreen!),
          ),
        );
      }
    } finally {
      if (mounted)
        setState(() {
          _pointer = null;
          _startScreen = null;
          _startComp = null;
          _lastScreen = null;
          _marquee = null;
          _dragging = false;
          _panning = false;
          _finishing = false;
        });
    }
  }

  /// Eyedropper: read the rendered pixel, then hand it to whatever the
  /// palette would colour (the colour target, else the selection).
  Future<void> _pick(Offset point) async {
    c.eyedropper.value = false;
    if (!c.supports('pickColor')) return;
    await c.command('pickColor', {'x': point.dx, 'y': point.dy});
    final rgba = c.state['pickedColor'];
    if (rgba is List && c.supports('applyPalette')) {
      await c.command('applyPalette', {'rgba': rgba});
    }
  }

  void _down(PointerDownEvent event) {
    if (_pointer != null || _finishing) return;
    _focus.requestFocus();
    if (c.eyedropper.value) {
      _pick(_toComp(event.localPosition));
      return;
    }
    _pointer = event.pointer;
    _startScreen = event.localPosition;
    _lastScreen = event.localPosition;
    _startComp = _toComp(event.localPosition);
    _additive = _add;
    if (_userStage && event.buttons == kSecondaryMouseButton && !_dragging) {
      _orbiting = true;
      final angles = _observer['orbit'] as List?;
      _orbitAngles = angles == null
          ? [0, 0]
          : angles.map((v) => (v as num).toDouble()).toList();
      return;
    }
    if (_userStage &&
        event.buttons == kPrimaryMouseButton &&
        _doubleTap(event)) {
      final layer = _hit(_startComp!);
      c.command(
        'stageView',
        layer == null ? {'reset': true} : {'focus': layer['id']},
      );
      _pointer = null;
      return;
    }
    if (_front && _extend && event.buttons == kPrimaryMouseButton) {
      final side = _extentEdge(event.localPosition);
      if (side != null) {
        _extentDrag = _extent;
        _extentSide = side;
        return;
      }
    }
    if (_front && event.buttons == kPrimaryMouseButton) {
      final selected = _selectedCamera;
      if (selected != null) {
        for (final entry in _cameraHandles(selected).entries) {
          if ((entry.value - event.localPosition).distance < 7) {
            _cameraDrag = selected;
            _cameraHandle = entry.key;
            return;
          }
        }
      }
      for (final camera in _cameras) {
        if (camera['authorable'] != false &&
            _onCameraEdge(camera, event.localPosition)) {
          c.command('select', {
            'ids': [camera['id']],
          });
          _cameraDrag = camera;
          _cameraHandle = 'center';
          return;
        }
      }
    }
    _panning =
        event.buttons == kMiddleMouseButton ||
        HardwareKeyboard.instance.logicalKeysPressed.contains(
          LogicalKeyboardKey.space,
        );
    if (_panning) return;
    if (event.buttons != kPrimaryMouseButton) {
      _pointer = null;
      return;
    }
    if (c.supports('stageGesture')) {
      for (final entry in _handles().entries) {
        if ((entry.value - event.localPosition).distance <
            _grabTolerance(entry.key)) {
          _begin(_startComp!, c.selectedIds, entry.key);
          return;
        }
      }
      if (_spatialHit(event.localPosition)) {
        _begin(_startComp!, c.selectedIds, 'spatial');
        return;
      }
    }
    final layer = _hit(_startComp!);
    if (layer != null) {
      final id = (layer['id'] as num).toInt();
      final ids = List<int>.of(c.selectedIds);
      if (_additive) {
        if (ids.contains(id)) {
          ids.remove(id);
        } else {
          ids.add(id);
        }
      } else if (!ids.contains(id)) {
        ids
          ..clear()
          ..add(id);
      }
      _ids = ids;
      if (c.supports('select')) c.command('select', {'ids': ids});
      // A simple click selects. The drag starts only after the native slop.
      _handle = 'body';
    } else {
      _ids = _additive ? List.of(c.selectedIds) : [];
      setState(() => _marquee = Rect.fromPoints(_startComp!, _startComp!));
    }
  }

  /// The 3D gizmo lights the part under the pointer. Native only hears about
  /// the pointer while it is on the mesh, plus once when it leaves, so an
  /// idle Stage sends nothing. Moves are coalesced: one in flight at a time.
  bool _onMesh = false;
  Map<String, dynamic>? _hoverPending;
  bool _hoverSending = false;
  Offset? _hoverScreen;
  void _hover(PointerHoverEvent event) {
    _hoverScreen = event.localPosition;
    if (_pointer != null || !c.supports('stageGesture')) return;
    final on = _spatialHit(event.localPosition);
    if (!on && !_onMesh) return;
    _onMesh = on;
    _sendHover(on ? _toComp(event.localPosition) : null);
  }

  /// AE's P / R / S, held: the gizmo narrows to position, rotation or scale
  /// for as long as the key is down. The same key still reveals the same
  /// property in the Inspector, so one letter means one thing everywhere.
  static final _heldKeys = {
    LogicalKeyboardKey.keyP: 'position',
    LogicalKeyboardKey.keyR: 'rotation',
    LogicalKeyboardKey.keyS: 'scale',
  };
  String? _held;

  /// Returns true while a held letter belongs to the gizmo, so the key repeat
  /// counts as handled — otherwise macOS beeps on every repeat. Focus-based
  /// shortcuts still see the event; the results are OR-ed, not chained.
  bool _heldKey(KeyEvent event) {
    if (!_heldKeys.containsKey(event.logicalKey)) return false;
    if (FocusManager.instance.primaryFocus?.context?.widget is EditableText)
      return false;
    final modified =
        HardwareKeyboard.instance.isMetaPressed ||
        HardwareKeyboard.instance.isControlPressed ||
        HardwareKeyboard.instance.isAltPressed;
    if (modified) return false;
    final mine = _spatialActive && c.supports('stageGesture');
    if (event is KeyRepeatEvent) return mine;
    final pressed = HardwareKeyboard.instance.logicalKeysPressed;
    String? held;
    for (final entry in _heldKeys.entries) {
      if (pressed.contains(entry.key)) held = entry.value;
    }
    if (held != _held) {
      _held = held;
      if (mine && _pointer == null) {
        final at = _hoverScreen;
        _sendHover(at != null && _onMesh ? _toComp(at) : null);
      }
    }
    return mine && event is KeyDownEvent;
  }

  void _sendHover(Offset? point) {
    _hoverPending = {
      'phase': 'hover',
      'view': widget.view,
      'point': point == null ? null : [point.dx, point.dy],
      'viewScale': _scale,
      'held': _held,
    };
    if (_hoverSending) return;
    _hoverSending = true;
    () async {
      try {
        while (_hoverPending != null) {
          final args = _hoverPending!;
          _hoverPending = null;
          await c.command('stageGesture', args);
        }
      } finally {
        _hoverSending = false;
      }
    }();
  }

  void _move(PointerMoveEvent event) {
    if (event.pointer != _pointer) return;
    final old = _lastScreen ?? event.localPosition;
    _lastScreen = event.localPosition;
    if (_cameraDrag != null || _extentDrag != null) {
      _moveCamera(event.localPosition);
      return;
    }
    if (_orbiting) {
      final delta = event.localPosition - old;
      _orbitAngles = [
        (_orbitAngles[0] + delta.dy * .3).clamp(-85.0, 85.0),
        _orbitAngles[1] - delta.dx * .3,
      ];
      _orbitPending = List.of(_orbitAngles);
      _sendOrbit();
      return;
    }
    if (_panning) {
      setState(() => _pan += event.localPosition - old);
      return;
    }
    final point = _toComp(event.localPosition);
    if (_marquee != null) {
      setState(() => _marquee = Rect.fromPoints(_startComp!, point));
      return;
    }
    if (!_dragging &&
        _ids.isNotEmpty &&
        c.supports('stageGesture') &&
        (event.localPosition - _startScreen!).distance >
            (event.kind == PointerDeviceKind.mouse ? 3 : kTouchSlop)) {
      _begin(_startComp!, _ids, 'body');
    }
    if (_dragging) _update(point);
  }

  void _up(PointerUpEvent event) {
    if (event.pointer != _pointer) return;
    _lastScreen = event.localPosition;
    if (_orbiting) {
      _orbiting = false;
      _pointer = null;
      return;
    }
    if (_cameraDrag != null || _extentDrag != null) {
      _finishCamera(false);
      return;
    }
    final box = _marquee;
    if (box != null && c.supports('select')) {
      final hits = _visible
          .where((l) => l['locked'] != true && _intersects(_corners(l), box))
          .map((l) => (l['id'] as num).toInt());
      c.command('select', {
        'ids': {..._ids, ...hits}.toList(),
      });
    }
    _finish(false);
  }

  void _zoomAt(double next, Offset anchor) {
    final point = _toComp(anchor);
    final scale = next.clamp(.02, 16.0).toDouble();
    setState(() {
      _zoom = scale;
      _pan =
          anchor -
          point * scale -
          Offset(
            (_viewport.width - _width * scale) / 2,
            (_viewport.height - _height * scale) / 2,
          );
    });
  }

  @override
  void dispose() {
    HardwareKeyboard.instance.removeHandler(_heldKey);
    if (_dragging) {
      _pending = null;
      final args = _gesture('cancel', _startComp!);
      _drained.whenComplete(() => c.command('stageGesture', args));
    }
    c.viewCommand.removeListener(_viewCommand);
    if (_userStage && _sentWindow != null && c.supports('stageWindow'))
      c.command('stageWindow', {'width': 0, 'height': 0});
    _focus.dispose();
    super.dispose();
  }

  Widget _button(String title, VoidCallback? onPressed) => EditorButton(
    title,
    onPressed,
    tooltip: switch (title) {
      'Fit' => 'Fit the working area',
      'Extend' => 'Allow the working area edges to be dragged',
      '● Extend' => 'Stop dragging the working area edges',
      '100%' => 'Actual size',
      '−' => 'Zoom out',
      '+' => 'Zoom in',
      'Front' => 'Look straight at the composition (double-click background)',
      _ => title,
    },
  );
  DocumentSlice get _slice => c.slice('stage', const [
    'layers',
    'selectedId',
    'selectedIds',
    'width',
    'height',
    'observer',
    'spatialGizmo',
    'stageSpatialGizmo',
    'cameraGizmos',
    'pickedColor',
    'capabilities',
    'contentRevision',
    'documentRevision',
  ]);

  /// What the bars above and below the picture read. Kept apart from [_slice]
  /// so a moved layer, and every frame drawn while it moves, leaves the bars
  /// standing instead of re-measuring their intrinsic widths.
  DocumentSlice get _chrome => c.slice('stage:chrome', const [
    'observer',
    'width',
    'height',
    'capabilities',
  ]);

  @override
  Widget build(BuildContext context) => !Visibility.of(context)
      ? const SizedBox.shrink()
      : Column(
          children: [
            AnimatedBuilder(
              animation: _chrome,
              builder: (context, _) => EditorBar(
                decoration: const BoxDecoration(
                  color: EditorTheme.panel,
                  border: Border(bottom: BorderSide(color: EditorTheme.line)),
                ),
                children: [
                  const SizedBox(width: EditorMetrics.s8),
                  if (_userStage)
                    _button(
                      'Front',
                      _home
                          ? null
                          : () => c.command('stageView', {'reset': true}),
                    ),
                  const Spacer(),
                  _button('Fit', _fit),
                  _button(
                    '100%',
                    () => setState(() {
                      _zoom = 1;
                      _pan = Offset.zero;
                    }),
                  ),
                  _button(
                    '−',
                    () => _zoomAt(
                      ((_scale * 100).round() - 1) / 100,
                      _viewport.center(Offset.zero),
                    ),
                  ),
                  EditorPercentField(
                    value: _scale * 100,
                    min: 2,
                    max: 1600,
                    label: 'Stage zoom',
                    onChanged: (v) =>
                        _zoomAt(v / 100, _viewport.center(Offset.zero)),
                  ),
                  _button(
                    '+',
                    () => _zoomAt(
                      ((_scale * 100).round() + 1) / 100,
                      _viewport.center(Offset.zero),
                    ),
                  ),
                ],
              ),
            ),
            Expanded(
              child: AnimatedBuilder(
                animation: Listenable.merge([
                  _slice,
                  c.rendered,
                  c.textureIds,
                  c.playing,
                  c.anchorPreview,
                ]),
                builder: (context, _) => LayoutBuilder(
                  builder: (context, box) {
                    final resized = _viewport != box.biggest;
                    _viewport = box.biggest;
                    if (!_initialFrameRequested || resized) {
                      final needsFrame = !_initialFrameRequested;
                      _initialFrameRequested = true;
                      WidgetsBinding.instance.addPostFrameCallback((_) {
                        if (!mounted) return;
                        setState(() {});
                        if (needsFrame) c.refreshPreview();
                      });
                    }
                    WidgetsBinding.instance.addPostFrameCallback(
                      (_) => _syncWindow(),
                    );
                    final origin = _origin;
                    final scale = _scale;
                    // The Stage is drawn at the tab's own size, the frame
                    // inside it; the Camera is the output picture itself.
                    final picture = _userStage
                        ? Offset.zero & _viewport
                        : Rect.fromLTWH(
                            origin.dx,
                            origin.dy,
                            _width * scale,
                            _height * scale,
                          );
                    final gizmos = !c.playing.value;
                    final outlines = <List<Offset>>[];
                    Offset? anchorPreview;
                    for (final layer in _visible.where(
                      (l) => gizmos && c.selectedIds.contains(l['id']),
                    )) {
                      final points = _corners(layer);
                      if (points.isNotEmpty)
                        outlines.add(points.map(_toScreen).toList());
                      // Where a hovered anchor would sit: bilinear in the corners.
                      final f = c.anchorPreview.value;
                      if (f != null &&
                          points.length >= 4 &&
                          anchorPreview == null) {
                        final u = f[0], v = f[1];
                        final top = points[0] + (points[1] - points[0]) * u;
                        final bottom = points[3] + (points[2] - points[3]) * u;
                        anchorPreview = _toScreen(top + (bottom - top) * v);
                      }
                    }
                    return Focus(
                      focusNode: _focus,
                      onKeyEvent: (_, event) {
                        if (event is KeyDownEvent &&
                            event.logicalKey == LogicalKeyboardKey.escape &&
                            (_pointer != null || _dragging)) {
                          _finish(true);
                          return KeyEventResult.handled;
                        }
                        return KeyEventResult.ignored;
                      },
                      child: Listener(
                        behavior: HitTestBehavior.opaque,
                        onPointerDown: _down,
                        onPointerMove: _move,
                        onPointerUp: _up,
                        onPointerHover: _hover,
                        onPointerCancel: (_) => _finish(true),
                        onPointerSignal: (event) {
                          if (event is PointerScrollEvent) {
                            GestureBinding.instance.pointerSignalResolver
                                .register(event, (signal) {
                                  final scroll = signal as PointerScrollEvent;
                                  _zoomAt(
                                    _scale *
                                        math.exp(
                                          -scroll.scrollDelta.dy * .0015,
                                        ),
                                    scroll.localPosition,
                                  );
                                });
                          }
                        },
                        child: ClipRect(
                          child: ColoredBox(
                            color: EditorTheme.app,
                            child: Stack(
                              children: [
                                // 地が無い枠は市松で見せる。枠そのものの見え方なので、
                                // 描いた絵の有無に関わらず枠いっぱいに敷く(書き出しには乗らない)。
                                if (_transparentGround)
                                  Positioned.fromRect(
                                    rect: Rect.fromLTWH(
                                      origin.dx,
                                      origin.dy,
                                      _width * scale,
                                      _height * scale,
                                    ),
                                    child: const CustomPaint(
                                      painter: CheckerPainter(),
                                    ),
                                  ),
                                Positioned.fromRect(
                                  rect: picture,
                                  child: Stack(
                                    fit: StackFit.expand,
                                    children: [
                                      ValueListenableBuilder<Map<String, int>>(
                                        valueListenable: c.textureIds,
                                        builder: (context, ids, _) =>
                                            switch (ids[widget.view]) {
                                              null => const Center(
                                                child: Text(
                                                  'No rendered texture',
                                                  style: TextStyle(
                                                    fontSize:
                                                        EditorMetrics.font,
                                                    color: EditorTheme.muted,
                                                  ),
                                                ),
                                              ),
                                              final id => Texture(
                                                textureId: id,
                                                filterQuality:
                                                    FilterQuality.low,
                                              ),
                                            },
                                      ),
                                    ],
                                  ),
                                ),
                                Positioned.fill(
                                  child: IgnorePointer(
                                    child: CustomPaint(
                                      painter: _StageOverlay(
                                        dimOutside: _userStage,
                                        anchorPreview: anchorPreview,
                                        cameras: gizmos
                                            ? _cameras
                                                  .map(_cameraPoints)
                                                  .toList()
                                            : const [],
                                        cameraEyes: gizmos
                                            ? _cameras.map(_cameraEye).toList()
                                            : const [],
                                        cameraTargets: [
                                          if (gizmos)
                                            for (final camera in _cameras)
                                              ?_point(camera['target']),
                                        ],
                                        cameraHandles:
                                            gizmos &&
                                                _front &&
                                                _selectedCamera != null
                                            ? _cameraHandles(_selectedCamera!)
                                            : const {},
                                        front: _front,
                                        extent: _extentPoints(),
                                        extendable: _front && _extend,
                                        observerTarget: _front
                                            ? null
                                            : _point(_observer['target']),
                                        outlines: _outlinesCopy(outlines),
                                        handles: gizmos ? _handles() : const {},
                                        spatialMesh: gizmos && _spatialActive
                                            ? _screenMesh()
                                            : null,
                                        marquee: _marquee == null
                                            ? null
                                            : Rect.fromPoints(
                                                _toScreen(_marquee!.topLeft),
                                                _toScreen(
                                                  _marquee!.bottomRight,
                                                ),
                                              ),
                                        frame: [
                                          for (final p
                                              in (_observer['frame']
                                                      as List?) ??
                                                  [])
                                            ?_point(p),
                                        ],
                                        viewport: Rect.fromLTWH(
                                          origin.dx,
                                          origin.dy,
                                          _width * scale,
                                          _height * scale,
                                        ),
                                      ),
                                    ),
                                  ),
                                ),
                              ],
                            ),
                          ),
                        ),
                      ),
                    );
                  },
                ),
              ),
            ),
            AnimatedBuilder(
              animation: _chrome,
              builder: (context, _) => EditorBar(
                padding: const EdgeInsets.symmetric(
                  horizontal: EditorMetrics.s8,
                ),
                children: [
                  Text(
                    '${_width.toInt()} × ${_height.toInt()}',
                    style: const TextStyle(
                      fontSize: EditorMetrics.dense,
                      color: EditorTheme.muted,
                    ),
                  ),
                  Padding(
                    padding: const EdgeInsets.only(left: EditorMetrics.s8),
                    child: EditorSwitch(
                      key: const ValueKey('stage:transparentGround'),
                      on: _transparentGround,
                      compact: true,
                      glyph: Icons.grid_on,
                      label: _transparentGround
                          ? 'The frame has no ground; the export carries alpha'
                          : 'Drop the ground so the export carries alpha',
                      onChanged: c.supports('composition')
                          ? (_) => _toggleGround()
                          : null,
                    ),
                  ),
                  if (_userStage)
                    Padding(
                      padding: const EdgeInsets.only(left: EditorMetrics.s8),
                      child: ValueListenableBuilder<Map<String, dynamic>>(
                        valueListenable: c.deskWork,
                        builder: (context, _, _) => _button(
                          _extend ? '● Extend' : 'Extend',
                          _toggleExtend,
                        ),
                      ),
                    ),
                  const Spacer(),
                  ValueListenableBuilder<int>(
                    valueListenable: c.frame,
                    builder: (context, frame, _) => Text(
                      'Frame $frame',
                      style: const TextStyle(
                        fontSize: EditorMetrics.dense,
                        color: EditorTheme.muted,
                      ),
                    ),
                  ),
                  if (!c.supports('stageGesture'))
                    const Padding(
                      padding: EdgeInsets.only(left: EditorMetrics.s8),
                      child: Text(
                        'Transform gestures unavailable',
                        style: TextStyle(
                          fontSize: EditorMetrics.dense,
                          color: EditorTheme.muted,
                        ),
                      ),
                    ),
                ],
              ),
            ),
          ],
        );
  List<List<Offset>> _outlinesCopy(List<List<Offset>> p) =>
      p.map((v) => List<Offset>.of(v)).toList();
}

class _StageOverlay extends CustomPainter {
  const _StageOverlay({
    this.dimOutside = false,
    this.cameras = const [],
    this.cameraEyes = const [],
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
  });
  final List<List<Offset>> outlines, cameras;
  final List<Offset?> cameraEyes;
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
  void _cross(Canvas canvas, Offset at, double half, Paint paint) {
    canvas.drawLine(at - Offset(half, 0), at + Offset(half, 0), paint);
    canvas.drawLine(at - Offset(0, half), at + Offset(0, half), paint);
  }

  @override
  void paint(Canvas canvas, Size size) {
    final framePaint = Paint()
      ..style = PaintingStyle.stroke
      ..color = EditorTheme.line
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
        Paint()..color = EditorTheme.app.withValues(alpha: .55),
      );
    }
    canvas.drawPath(framePath, framePaint);
    final line = Paint()
      ..style = PaintingStyle.stroke
      ..color = EditorTheme.accent
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
          ..color = extendable ? EditorTheme.ink : EditorTheme.muted
          ..strokeWidth = extendable ? 2 : 1,
      );
    }
    final cameraLine = Paint()
      ..color = const Color(0xff8ed9e6)
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
        canvas.drawCircle(entry.value, 4, Paint()..color = EditorTheme.app);
        canvas.drawCircle(entry.value, 4, cameraLine);
      } else {
        final rect = Rect.fromCenter(
          center: entry.value,
          width: EditorMetrics.s6,
          height: EditorMetrics.s6,
        );
        canvas.drawRect(rect, Paint()..color = EditorTheme.app);
        canvas.drawRect(rect, cameraLine);
      }
    }
    for (final (index, box) in cameras.indexed) {
      if (box.length != 4) continue;
      canvas.drawPath(Path()..addPolygon(box, true), cameraLine);
      final eye = index < cameraEyes.length ? cameraEyes[index] : null;
      if (front || eye == null) continue;
      for (final corner in box) {
        canvas.drawLine(eye, corner, cameraLine);
      }
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromCenter(
            center: eye,
            width: EditorMetrics.s16,
            height: EditorMetrics.s12,
          ),
          const Radius.circular(EditorMetrics.s2),
        ),
        cameraLine,
      );
      canvas.drawLine(
        eye + const Offset(8, -4),
        eye + const Offset(13, -7),
        cameraLine,
      );
      canvas.drawLine(
        eye + const Offset(13, -7),
        eye + const Offset(13, 7),
        cameraLine,
      );
      canvas.drawLine(
        eye + const Offset(13, 7),
        eye + const Offset(8, 4),
        cameraLine,
      );
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
        canvas.drawCircle(entry.value, 4, Paint()..color = EditorTheme.app);
        canvas.drawCircle(entry.value, 4, line);
      } else {
        final rect = Rect.fromCenter(
          center: entry.value,
          width: EditorMetrics.s6,
          height: EditorMetrics.s6,
        );
        canvas.drawRect(rect, Paint()..color = EditorTheme.app);
        canvas.drawRect(rect, line);
      }
    }
    if (marquee != null) {
      canvas.drawRect(
        marquee!,
        Paint()..color = EditorTheme.accent.withValues(alpha: .12),
      );
      canvas.drawRect(marquee!, line);
    }
  }

  @override
  bool shouldRepaint(covariant _StageOverlay old) =>
      old.cameras.toString() != cameras.toString() ||
      old.cameraEyes.toString() != cameraEyes.toString() ||
      old.cameraTargets.toString() != cameraTargets.toString() ||
      old.cameraHandles.toString() != cameraHandles.toString() ||
      old.front != front ||
      old.extent.toString() != extent.toString() ||
      old.extendable != extendable ||
      old.observerTarget != observerTarget ||
      old.dimOutside != dimOutside ||
      old.frame.toString() != frame.toString() ||
      old.viewport != viewport ||
      old.marquee != marquee ||
      old.outlines.toString() != outlines.toString() ||
      old.handles.toString() != handles.toString() ||
      !identical(old.spatialMesh, spatialMesh);
}

/// The gizmo mesh as native sent it: comp-space vertices, linear colours
/// already converted to sRGB, and only the triangles that are drawable.
class _SpatialMesh {
  const _SpatialMesh(this.vertices, this.colors, this.indices);
  final List<Offset> vertices;
  final List<Color> colors;
  final Uint16List indices;

  static _SpatialMesh? parse(dynamic raw) {
    if (raw is! Map) return null;
    final v = raw['vertices'], c = raw['colors'], i = raw['indices'];
    if (v is! List || c is! List || i is! List) return null;
    if (v.isEmpty || v.length > 0xFFFF || i.length < 3) return null;
    final vertices = [
      for (final p in v)
        p is List && p.length >= 2
            ? Offset(
                _StagePanelState._num(p[0], double.nan),
                _StagePanelState._num(p[1], double.nan),
              )
            : const Offset(double.nan, double.nan),
    ];
    double channel(dynamic x) =>
        math.pow(_StagePanelState._num(x).clamp(0.0, 1.0), 1 / 2.2).toDouble();
    int byte(dynamic x) => (channel(x) * 255).round().clamp(0, 255);
    // Native sends meaning, not paint: white is the gizmo at rest, black is
    // the hovered part. At rest it stays quiet; the accent says
    // "this one will move if you press". Anything else is passed through.
    Color tone(List rgba) {
      final alpha = (_StagePanelState._num(rgba[3], 1).clamp(0.0, 1.0) * 255)
          .round();
      final rgb = [for (var k = 0; k < 3; k++) _StagePanelState._num(rgba[k])];
      if (rgb.every((v) => v >= 0.999))
        return EditorTheme.muted.withAlpha(alpha);
      if (rgb.every((v) => v <= 0.001))
        return EditorTheme.accent.withAlpha(alpha);
      return Color.fromARGB(alpha, byte(rgb[0]), byte(rgb[1]), byte(rgb[2]));
    }

    final colors = [
      for (var k = 0; k < vertices.length; k++)
        k < c.length && c[k] is List && (c[k] as List).length >= 4
            ? tone(c[k] as List)
            : const Color(0x00000000),
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
    return _SpatialMesh(vertices, colors, Uint16List.fromList(indices));
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
List<Offset> _hull(List<Offset> points) {
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

bool _contains(List<Offset> polygon, Offset point) {
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

bool _intersects(List<Offset> polygon, Rect box) {
  if (polygon.isEmpty || box.width < .5 && box.height < .5) return false;
  if (polygon.any(box.contains)) return true;
  final corners = [box.topLeft, box.topRight, box.bottomRight, box.bottomLeft];
  if (corners.any((p) => _contains(polygon, p))) return true;
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
