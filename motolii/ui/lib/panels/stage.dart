import 'dart:math' as math;

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';

class StagePanel extends StatefulWidget {
  const StagePanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<StagePanel> createState() => _StagePanelState();
}

class _StagePanelState extends State<StagePanel> {
  EditorSession get c => widget.controller;
  Map<String, dynamic> get _state {
    final rendered = c.rendered.value;
    return rendered.isNotEmpty &&
            rendered['frame'] == c.frame.value &&
            (rendered['documentRevision'] == null ||
                rendered['documentRevision'] == c.state['documentRevision'])
        ? {...c.state, ...rendered}
        : c.state;
  }

  List<Map<String, dynamic>> get _layers =>
      EditorSession.maps(_state['layers']);
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
  }

  void _viewCommand() {
    final action = c.viewCommand.value;
    switch (action) {
      case 'Fit':
        setState(() {
          _zoom = null;
          _pan = Offset.zero;
        });
      case 'Actual':
        setState(() {
          _zoom = 1;
          _pan = Offset.zero;
        });
      case 'In':
        _zoomAt(_scale * 1.2, _viewport.center(Offset.zero));
      case 'Out':
        _zoomAt(_scale / 1.2, _viewport.center(Offset.zero));
      case '3D':
      case 'Output':
        if (c.supports('stageView')) {
          c.command('stageView', {'mode': action});
        } else {
          c.error.value =
              '${action} view is not available in this comparison yet';
        }
    }
  }

  bool get _userStage => c.state['stageView'] == 'User';
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
    } finally { _orbitSending = false; }
  }
  List<Map<String,dynamic>> get _cameras => _userStage ? EditorSession.maps(_state['cameraGizmos']) : [];
  List<Offset> _cameraPoints(Map<String,dynamic> camera) => (camera['points'] as List).map((p)=>_toScreen(Offset(_num(p[0]),_num(p[1])))).toList();

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
        EditorSession.map(layer['bounds'])['corners'] ?? layer['corners'];
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
    _mode = handle == 'body'
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
    'mode': _mode,
    'ids': _ids,
    'start': [_startComp!.dx, _startComp!.dy],
    'point': [point.dx, point.dy],
    'handle': _handle,
    'viewScale': _scale,
    'shift': _shift,
    'alt': HardwareKeyboard.instance.isAltPressed,
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

  void _down(PointerDownEvent event) {
    if (_pointer != null || _finishing) return;
    _focus.requestFocus();
    _pointer = event.pointer;
    _startScreen = event.localPosition;
    _lastScreen = event.localPosition;
    _startComp = _toComp(event.localPosition);
    _additive = _add;
    if (_userStage && event.buttons == kSecondaryMouseButton) {
      _orbiting = true;
      final angles = c.state['userOrbit'] as List?;
      _orbitAngles = angles == null ? [-15,30] : angles.map((v)=>(v as num).toDouble()).toList();
      return;
    }
    for (final camera in _cameras) {
      if ((_cameraPoints(camera).first-event.localPosition).distance < 12) {
        c.command('select', {'ids':[camera['id']]});
        _pointer = null;
        return;
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

  void _move(PointerMoveEvent event) {
    if (event.pointer != _pointer) return;
    final old = _lastScreen ?? event.localPosition;
    _lastScreen = event.localPosition;
    if (_orbiting) {
      final delta = event.localPosition-old;
      _orbitAngles = [(_orbitAngles[0]+delta.dy*.3).clamp(-85.0,85.0),_orbitAngles[1]-delta.dx*.3];
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
    if (_orbiting) { _orbiting=false; _pointer=null; return; }
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
    if (_dragging) {
      _pending = null;
      final args = _gesture('cancel', _startComp!);
      _drained.whenComplete(() => c.command('stageGesture', args));
    }
    c.viewCommand.removeListener(_viewCommand);
    _focus.dispose();
    super.dispose();
  }

  Widget _button(String title, VoidCallback? onPressed) => EditorButton(
    title,
    onPressed,
    tooltip: switch (title) {
      'Fit' => 'Fit composition',
      '100%' => 'Actual size',
      '−' => 'Zoom out',
      '+' => 'Zoom in',
      _ => title,
    },
  );
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: Listenable.merge([c.document, c.rendered, c.textureId]),
    builder: (context, _) => Column(
      children: [
        Container(
          height: 22,
          decoration: const BoxDecoration(
            color: EditorTheme.panel,
            border: Border(bottom: BorderSide(color: EditorTheme.line)),
          ),
          child: Row(
            children: [
              const SizedBox(width: 8),

              _button(_userStage ? '● User Stage' : 'User Stage', () => c.command('stageView', {'mode':'User'})),
              _button(!_userStage ? '● Camera View' : 'Camera View', () => c.command('stageView', {'mode':'Camera'})),
              const Spacer(),
              _button(
                'Fit',
                () => setState(() {
                  _zoom = null;
                  _pan = Offset.zero;
                }),
              ),
              _button(
                '100%',
                () => setState(() {
                  _zoom = 1;
                  _pan = Offset.zero;
                }),
              ),
              _button(
                '−',
                () => _zoomAt(_scale / 1.2, _viewport.center(Offset.zero)),
              ),
              Text(
                '${(_scale * 100).round()}%',
                style: const TextStyle(fontSize: 10),
              ),
              _button(
                '+',
                () => _zoomAt(_scale * 1.2, _viewport.center(Offset.zero)),
              ),
            ],
          ),
        ),
        Expanded(
          child: LayoutBuilder(
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
              final origin = _origin;
              final scale = _scale;
              final outlines = <List<Offset>>[];
              final volumes = <List<Offset>>[];
              for (final layer in _visible.where(
                (l) => c.selectedIds.contains(l['id']),
              )) {
                final points = _corners(layer);
                final raw = _rawCorners(layer);
                if (raw.length == 8) volumes.add(raw.map(_toScreen).toList());
                if (points.isNotEmpty)
                  outlines.add(points.map(_toScreen).toList());
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
                  onPointerCancel: (_) => _finish(true),
                  onPointerSignal: (event) {
                    if (event is PointerScrollEvent) {
                      GestureBinding.instance.pointerSignalResolver.register(
                        event,
                        (signal) {
                          final scroll = signal as PointerScrollEvent;
                          _zoomAt(
                            _scale * math.exp(-scroll.scrollDelta.dy * .0015),
                            scroll.localPosition,
                          );
                        },
                      );
                    }
                  },
                  child: ClipRect(
                    child: ColoredBox(
                      color: EditorTheme.app,
                      child: Stack(
                        children: [
                          Positioned(
                            left: origin.dx,
                            top: origin.dy,
                            width: _width * scale,
                            height: _height * scale,
                            child: ValueListenableBuilder<int?>(
                              valueListenable: c.textureId,
                              builder: (context, id, _) => id == null
                                  ? const Center(
                                      child: Text(
                                        'No rendered texture',
                                        style: TextStyle(
                                          fontSize: 11,
                                          color: EditorTheme.muted,
                                        ),
                                      ),
                                    )
                                  : Texture(
                                      textureId: id,
                                      filterQuality: FilterQuality.low,
                                    ),
                            ),
                          ),
                          Positioned.fill(
                            child: IgnorePointer(
                              child: CustomPaint(
                                painter: _StageOverlay(
                                  cameras: _cameras.map(_cameraPoints).toList(),
                                  outlines: _outlinesCopy(outlines),
                                  volumes: volumes,
                                  handles: _handles(),
                                  marquee: _marquee == null
                                      ? null
                                      : Rect.fromPoints(
                                          _toScreen(_marquee!.topLeft),
                                          _toScreen(_marquee!.bottomRight),
                                        ),
                                  frame: Rect.fromLTWH(
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
        Container(
          height: 22,
          padding: const EdgeInsets.symmetric(horizontal: 8),
          color: EditorTheme.panel,
          child: Row(
            children: [
              Text(
                '${_width.toInt()} × ${_height.toInt()}',
                style: const TextStyle(fontSize: 10, color: EditorTheme.muted),
              ),
              const Spacer(),
              ValueListenableBuilder<int>(
                valueListenable: c.frame,
                builder: (context, frame, _) => Text(
                  'Frame $frame',
                  style: const TextStyle(
                    fontSize: 10,
                    color: EditorTheme.muted,
                  ),
                ),
              ),
              if (!c.supports('stageGesture'))
                const Padding(
                  padding: EdgeInsets.only(left: 8),
                  child: Text(
                    'Transform gestures unavailable',
                    style: TextStyle(fontSize: 10, color: EditorTheme.muted),
                  ),
                ),
            ],
          ),
        ),
      ],
    ),
  );
  List<List<Offset>> _outlinesCopy(List<List<Offset>> p) =>
      p.map((v) => List<Offset>.of(v)).toList();
}

class _StageOverlay extends CustomPainter {
  const _StageOverlay({
    this.cameras = const [],
    required this.outlines,
    required this.volumes,
    required this.handles,
    required this.frame,
    this.marquee,
  });
  final List<List<Offset>> outlines, volumes, cameras;
  final Map<String, Offset> handles;
  final Rect frame;
  final Rect? marquee;
  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(
      frame,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = EditorTheme.line
        ..strokeWidth = 1,
    );
    final line = Paint()
      ..style = PaintingStyle.stroke
      ..color = EditorTheme.accent
      ..strokeWidth = 1;
    final cameraLine = Paint()..color = const Color(0xff8ed9e6)..strokeWidth=1..style=PaintingStyle.stroke;
    for (final points in cameras) {
      if (points.length != 5) continue;
      for (var i=1;i<5;i++) {
        canvas.drawLine(points[0],points[i],cameraLine);
        canvas.drawLine(points[i],points[i==4?1:i+1],cameraLine);
      }
      canvas.drawRRect(RRect.fromRectAndRadius(Rect.fromCenter(center:points[0],width:16,height:12),const Radius.circular(2)),cameraLine);
      canvas.drawLine(points[0]+const Offset(8,-4),points[0]+const Offset(13,-7),cameraLine);
      canvas.drawLine(points[0]+const Offset(13,-7),points[0]+const Offset(13,7),cameraLine);
      canvas.drawLine(points[0]+const Offset(13,7),points[0]+const Offset(8,4),cameraLine);
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
    for (final points in volumes) {
      for (var i = 0; i < 8; i++) {
        for (final bit in [1, 2, 4]) {
          final j = i ^ bit;
          if (j > i) canvas.drawLine(points[i], points[j], line);
        }
      }
    }
    for (final entry in handles.entries) {
      if (entry.key == 'rotation') {
        canvas.drawCircle(entry.value, 4, Paint()..color = EditorTheme.app);
        canvas.drawCircle(entry.value, 4, line);
      } else {
        final rect = Rect.fromCenter(center: entry.value, width: 6, height: 6);
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
      old.frame != frame ||
      old.marquee != marquee ||
      old.outlines.toString() != outlines.toString() ||
      old.volumes.toString() != volumes.toString() ||
      old.handles.toString() != handles.toString();
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
