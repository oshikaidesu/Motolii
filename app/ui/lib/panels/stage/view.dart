part of '../stage.dart';

/// Where the Stage is looking: the document it reads, the size of the comp,
/// and the zoom and pan that carry comp coordinates onto the tab. Everything
/// else on the Stage converts through here and nothing here knows about
/// pointers, windows or gizmos.
mixin _StageView on State<StagePanel> {
  EditorSession get c => widget.controller;

  bool get _userStage => widget.view == 'User';

  double? _zoom;
  Offset _pan = Offset.zero;
  Size _viewport = Size.zero;

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

  List<Map<String, dynamic>> get _cameras =>
      _userStage ? EditorSession.maps(_state['cameraGizmos']) : [];
  Map<String, dynamic> get _observer => EditorSession.map(_state['observer']);
  bool get _front => !_userStage || _observer['front'] != false;
  bool get _home => !_userStage || _observer['home'] != false;
  double get _observerScale => _userStage ? numOf(_observer['scale'], 1) : 1;
  Map<String, dynamic>? get _extent => _userStage && _observer['extent'] is Map
      ? EditorSession.map(_observer['extent'])
      : null;
  bool get _extend => c.deskWork.value['stageExtend'] == true;
  List<Offset> _extentPoints() => [
    for (final p in (_extent?['points'] as List?) ?? []) ?_point(p),
  ];

  Offset? _point(dynamic p) => p is List && p.length >= 2
      ? _toScreen(Offset(numOf(p[0]), numOf(p[1])))
      : null;

  double get _width =>
      (numOf(_state['width'], 1920)).clamp(1, double.infinity).toDouble();
  double get _height =>
      (numOf(_state['height'], 1080)).clamp(1, double.infinity).toDouble();
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

  Offset _toComp(Offset p) => (p - _origin) / _scale;
  Offset _toScreen(Offset p) => _origin + p * _scale;
  bool get _shift => HardwareKeyboard.instance.isShiftPressed;
  bool get _add =>
      _shift ||
      HardwareKeyboard.instance.isMetaPressed ||
      HardwareKeyboard.instance.isControlPressed;

  void _fit() {
    setState(() {
      _zoom = null;
      _pan = Offset.zero;
    });
    if (_userStage && c.supports('stageView')) {
      c.command('stageView', {'fit': true});
    }
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
}
