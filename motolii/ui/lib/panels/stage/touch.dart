part of '../stage.dart';

/// AE's P / R / S, held: the gizmo narrows to position, rotation or scale
/// for as long as the key is down. The same key still reveals the same
/// property in the Inspector, so one letter means one thing everywhere.
final _heldKeys = {
  LogicalKeyboardKey.keyP: 'position',
  LogicalKeyboardKey.keyR: 'rotation',
  LogicalKeyboardKey.keyS: 'scale',
};

/// The hand on the picture, as a skin: it decides what a press lands on with its own geometry (the cage's handles,
/// the 3D gizmo's triangles, a camera box, a working-area edge) and hands it to the StageSession, which picks,
/// marquees, carries and streams the gesture. The skin keeps only the pointer's own bookkeeping.
mixin _StageTouch
    on State<StagePanel>, _StageView, _StageGrip, _StageSpatial, _StageCamera {
  final _focus = FocusNode(debugLabel: 'Stage');

  // the session's gesture, read by the painters and the keys
  Rect? get _marquee => _session.marquee;
  bool get _dragging => _session.gesture == StGesture.layer;

  /// Pointer is over this tab (Camera tab: the only time it shows gizmos).
  bool _inside = false;
  void _setInside(bool value) {
    if (_inside == value || !mounted) return;
    setState(() => _inside = value);
    if (!value) _touch(null);
  }

  /// rerun 3D view: a double click on an object looks at it, on the ground goes back to front. Timed by input time.
  Duration? _lastTapAt;
  Offset? _lastTapPos;
  bool _doubleTap(PointerDownEvent event) {
    final now = event.timeStamp;
    final again = _lastTapAt != null && now - _lastTapAt! < kDoubleTapTimeout && (_lastTapPos! - event.localPosition).distance < 6;
    _lastTapAt = again ? null : now;
    _lastTapPos = event.localPosition;
    return again;
  }

  List<Offset> _rawCorners(Map<String, dynamic> layer) => _session.rawCorners(layer);
  List<Offset> _corners(Map<String, dynamic> layer) => _session.corners(layer);
  List<Map<String, dynamic>> get _visible => _session.visible;
  Map<String, dynamic>? _hit(Offset comp) => _session.layerAt(comp);

  /// The cage's handles on screen (2D and 2.5D layers): corners, edge middles, and the rotation knob above.
  Map<String, Offset> _handles() {
    final layer = _active;
    if (layer == null || !StageSession.grabbable(layer) || !c.supports('stageGesture')) return {};
    final raw = _rawCorners(layer);
    final face = raw.length == 8 ? [raw[0], raw[1], raw[3], raw[2]] : raw;
    if (face.length != 4) return {};
    if (!['2D', 'TwoD', '2.5D', 'TwoPointFiveD'].contains('${layer['projection']}')) return {};
    final result = <String, Offset>{for (var i = 0; i < 4; i++) ['nw', 'ne', 'se', 'sw'][i]: _toScreen(face[i])};
    for (var i = 0; i < 4; i++) {
      result[['n', 'e', 's', 'w'][i]] = _toScreen((face[i] + face[(i + 1) % 4]) / 2);
    }
    final center = face.reduce((a, b) => a + b) / 4;
    final top = (face[0] + face[1]) / 2;
    final vector = top - center;
    result['rotation'] = _toScreen(top) + (vector.distance > 0 ? vector / vector.distance : const Offset(0, -1)) * 22;
    return result;
  }

  /// A handle is taken within this many pixels (a quarter of a small cage, so its body stays reachable).
  double _grabTolerance(String handle) {
    if (handle == 'rotation') return 7;
    final layer = _active;
    if (layer == null) return 0;
    final corners = _corners(layer).map(_toScreen).toList();
    if (corners.isEmpty) return 0;
    final xs = corners.map((p) => p.dx), ys = corners.map((p) => p.dy);
    return math.min(7, math.min((xs.reduce(math.max) - xs.reduce(math.min)) * .25, (ys.reduce(math.max) - ys.reduce(math.min)) * .25));
  }

  StMods get _mods {
    final k = HardwareKeyboard.instance;
    return StMods(add: _add, shift: k.isShiftPressed, alt: k.isAltPressed, cmd: k.isMetaPressed);
  }

  /// What a press at [p] lands on, in the order the picture stacks it.
  StTarget? _target(PointerDownEvent event) {
    final p = event.localPosition, comp = _toComp(p);
    if (c.eyedropper.value) return const StPick();
    if (_userStage && event.buttons == kSecondaryMouseButton) return const StOrbit();
    if (_userStage && event.buttons == kPrimaryMouseButton && _doubleTap(event)) return StFocus(_hit(comp)?['id']);
    if (_front && _extend && event.buttons == kPrimaryMouseButton) {
      if (_extentEdge(p) case final side?) return StExtentEdge(side);
    }
    if (_front && event.buttons == kPrimaryMouseButton) {
      if (_selectedCamera case final selected?) {
        for (final entry in _cameraHandles(selected).entries) {
          if ((entry.value - p).distance < 7) return StCameraHandle(selected, entry.key);
        }
      }
      for (final camera in _cameras) {
        if (camera['authorable'] != false && _onCameraEdge(camera, p)) return StCameraHandle(camera, 'center');
      }
    }
    if (event.buttons == kMiddleMouseButton || HardwareKeyboard.instance.logicalKeysPressed.contains(LogicalKeyboardKey.space)) return const StPan();
    if (event.buttons != kPrimaryMouseButton) return null;
    if (c.supports('stageGesture')) {
      for (final entry in _handles().entries) {
        if ((entry.value - p).distance < _grabTolerance(entry.key)) return StHandle(entry.key);
      }
      if (_spatialHit(p)) return const StHandle('spatial');
    }
    if (_hit(comp) case final layer?) return StLayer((layer['id'] as num).toInt());
    return const StEmpty();
  }

  void _down(PointerDownEvent event) {
    if (_pointer != null || _session.busy) return;
    _focus.requestFocus();
    final target = _target(event);
    if (target == null) return;
    // one-shot presses take no pointer
    if (target is StPick || target is StFocus) {
      _session.press(target, _toComp(event.localPosition), _mods);
      return;
    }
    _pointer = event.pointer;
    _startScreen = event.localPosition;
    _lastScreen = event.localPosition;
    _startComp = _toComp(event.localPosition);
    _session.press(target, _startComp!, _mods);
  }

  void _finish(bool cancel) {
    final at = _toComp(_lastScreen ?? _startScreen ?? Offset.zero);
    cancel ? _session.cancel(at, _mods, _scale) : _session.release(at, _mods, _scale);
    _pointer = null;
    _startScreen = null;
    _startComp = null;
    _lastScreen = null;
  }

  /// The layer the pointer is over: the only one whose cage and handles are drawn (2026-09-19, user).
  int? _touched;
  void _touch(int? id) {
    if (_touched == id || !mounted) return;
    setState(() => _touched = id);
  }

  /// The 3D gizmo lights the part under the pointer: the session hears the pointer while it is on the mesh.
  bool _onMesh = false;
  Offset? _hoverScreen;
  void _hover(PointerHoverEvent event) {
    _hoverScreen = event.localPosition;
    if (_pointer == null) _touch(_hit(_toComp(event.localPosition))?['id'] as int?);
    if (_pointer != null || !c.supports('stageGesture')) return;
    final on = _spatialHit(event.localPosition);
    if (!on && !_onMesh) return;
    _onMesh = on;
    _session.hover(on ? _toComp(event.localPosition) : null, _scale);
  }

  /// P / R / S held: true while the letter belongs to the gizmo, so the key repeat counts as handled (macOS would beep).
  bool _heldKey(KeyEvent event) {
    if (!_heldKeys.containsKey(event.logicalKey)) return false;
    if (FocusManager.instance.primaryFocus?.context?.widget is EditableText) return false;
    final modified = HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed || HardwareKeyboard.instance.isAltPressed;
    if (modified) return false;
    final mine = _spatialActive && c.supports('stageGesture');
    if (event is KeyRepeatEvent) return mine;
    final pressed = HardwareKeyboard.instance.logicalKeysPressed;
    String? held;
    for (final entry in _heldKeys.entries) {
      if (pressed.contains(entry.key)) held = entry.value;
    }
    if (held != _session.held) {
      _session.held = held;
      if (mine && _pointer == null) {
        final at = _hoverScreen;
        _session.hover(at != null && _onMesh ? _toComp(at) : null, _scale);
      }
    }
    return mine && event is KeyDownEvent;
  }

  void _move(PointerMoveEvent event) {
    if (event.pointer != _pointer) return;
    final old = _lastScreen ?? event.localPosition;
    _lastScreen = event.localPosition;
    switch (_session.gesture) {
      case StGesture.orbit:
        // a pixel turns the observer by 0.3 degrees
        final delta = event.localPosition - old;
        _session.orbitBy(delta.dy * .3, -delta.dx * .3);
      case StGesture.pan:
        _session.panBy(event.localPosition - old, _viewport);
      default:
        final slop = event.kind == PointerDeviceKind.mouse ? 3 : kTouchSlop;
        _session.drag(_toComp(event.localPosition), beyondSlop: (event.localPosition - _startScreen!).distance > slop, mods: _mods, viewScale: _scale);
    }
  }

  void _up(PointerUpEvent event) {
    if (event.pointer != _pointer) return;
    _lastScreen = event.localPosition;
    _finish(false);
  }
}
