part of '../stage.dart';

/// What the picture lets a hand take: a Group is reached from the layer
/// lists alone (Timeline, Inspector), never by touching its children's
/// place on the Stage (2026-09-19, user).
bool _grabbable(Map<String, dynamic> layer) =>
    layer['locked'] != true && layer['kind'] != 'Group';

/// AE's P / R / S, held: the gizmo narrows to position, rotation or scale
/// for as long as the key is down. The same key still reveals the same
/// property in the Inspector, so one letter means one thing everywhere.
final _heldKeys = {
  LogicalKeyboardKey.keyP: 'position',
  LogicalKeyboardKey.keyR: 'rotation',
  LogicalKeyboardKey.keyS: 'scale',
};

/// The hand on the picture: which layer a press takes, which cage handle it
/// grabs, and the gesture it streams to native from press to commit. A press
/// on a camera or a working-area edge is handed to [_StageCamera]; a press on
/// the 3D gizmo is decided by [_StageSpatial].
mixin _StageTouch
    on State<StagePanel>, _StageView, _StageGrip, _StageSpatial, _StageCamera {
  final _focus = FocusNode(debugLabel: 'Stage');
  Rect? _marquee;
  bool _panning = false, _dragging = false, _additive = false, _sending = false;
  Map<String, dynamic>? _pending;
  Future<void> _drained = Future<void>.value();
  List<int> _ids = [];
  String _mode = 'move', _handle = 'body';

  /// Pointer is over this tab (Camera tab: the only time it shows gizmos).
  bool _inside = false;
  void _setInside(bool value) {
    if (_inside == value || !mounted) return;
    setState(() => _inside = value);
    if (!value) _touch(null);
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
          ? Offset(numOf(v[0], double.nan), numOf(v[1], double.nan))
          : v is Map
          ? Offset(numOf(v['x'], double.nan), numOf(v['y'], double.nan))
          : null;
      if (p != null && p.dx.isFinite && p.dy.isFinite) points.add(p);
    }
    return points;
  }

  List<Offset> _corners(Map<String, dynamic> layer) => hull(_rawCorners(layer));
  List<Map<String, dynamic>> get _visible =>
      _layers.where((l) => l['hidden'] != true).toList();
  Map<String, dynamic>? _hit(Offset comp) {
    for (final layer in _visible) {
      if (_grabbable(layer) && polygonContains(_corners(layer), comp))
        return layer;
    }
    return null;
  }

  Map<String, Offset> _handles() {
    final layer = _active;
    if (layer == null || !_grabbable(layer) || !c.supports('stageGesture'))
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
    // Cmd while moving: snap to other boxes' edges and centres (After Effects).
    'snap': HardwareKeyboard.instance.isMetaPressed,
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

  /// The layer the pointer is over: the only one whose cage and handles are
  /// drawn (2026-09-19, user: gizmos answer the intent to touch a thing,
  /// in both tabs; a selection alone shows nothing on the picture).
  int? _touched;
  void _touch(int? id) {
    if (_touched == id || !mounted) return;
    setState(() => _touched = id);
  }

  void _hover(PointerHoverEvent event) {
    _hoverScreen = event.localPosition;
    if (_pointer == null)
      _touch(_hit(_toComp(event.localPosition))?['id'] as int?);
    if (_pointer != null || !c.supports('stageGesture')) return;
    final on = _spatialHit(event.localPosition);
    if (!on && !_onMesh) return;
    _onMesh = on;
    _sendHover(on ? _toComp(event.localPosition) : null);
  }

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
          .where(
            (l) => l['locked'] != true && polygonIntersects(_corners(l), box),
          )
          .map((l) => (l['id'] as num).toInt());
      c.command('select', {
        'ids': {..._ids, ...hits}.toList(),
      });
    }
    _finish(false);
  }
}
