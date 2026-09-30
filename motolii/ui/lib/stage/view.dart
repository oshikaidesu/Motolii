part of 'panel.dart';

/// Where the Stage is looking: the document it reads, the size of the comp,
/// and the zoom and pan that carry comp coordinates onto the tab. Everything
/// else on the Stage converts through here and nothing here knows about
/// pointers, windows or gizmos.
mixin _StageView on State<StagePanel> {
  EditorSession get c => widget.controller;

  bool get _userStage => widget.view == 'User';

  /// Where this view looks lives in the StageSession (it outlives the tab); the tab's size is the skin's own.
  late final StageSession _session = StageSession.of(c, widget.view);
  Size _viewport = Size.zero;

  // what the view reads is the session's; these names are the skin's shorthand for it
  Map<String, dynamic> get _state => _session.state;
  Map<String, dynamic>? get _active => _session.active;
  List<Map<String, dynamic>> get _cameras => _session.cameras;
  Map<String, dynamic> get _observer => _session.observer;
  bool get _front => _session.front;
  bool get _home => _session.home;
  Map<String, dynamic>? get _extent => _session.extent;
  bool get _extend => _session.extending;
  List<Offset> _extentPoints() => [
    for (final p in (_extent?['points'] as List?) ?? []) ?_point(p),
  ];

  Offset? _point(dynamic p) => p is List && p.length >= 2
      ? _toScreen(Offset(numOf(p[0]), numOf(p[1])))
      : null;

  double get _width => _session.width;
  double get _height => _session.height;
  double get _scale => _session.scaleIn(_viewport);
  Offset get _origin => _session.originIn(_viewport);

  Offset _toComp(Offset p) => _session.toComp(p, _viewport);
  Offset _toScreen(Offset p) => _session.toScreen(p, _viewport);
  bool get _shift => HardwareKeyboard.instance.isShiftPressed;
  bool get _add =>
      _shift ||
      HardwareKeyboard.instance.isMetaPressed ||
      HardwareKeyboard.instance.isControlPressed;

  void _fit() => _session.fit();

  /// A trackpad's two fingers: they carry the picture, and a pinch zooms about the fingers (the wheel zooms too).
  double _pinch = 1;
  void _panZoom(PointerPanZoomUpdateEvent event) {
    if (event.panDelta != Offset.zero) _session.panBy(event.panDelta, _viewport);
    final step = event.scale / _pinch;
    _pinch = event.scale;
    if (step != 1) _zoomAt(_scale * step, event.localPosition);
  }

  void _zoomAt(double next, Offset anchor) => _session.zoomAt(next, anchor, _viewport);

  void _viewCommand() {
    if (!Visibility.of(context)) return;
    final action = c.viewCommand.value;
    switch (action) {
      case 'Fit':
        _fit();
      case 'Actual':
        _session.actual();
      case 'In':
        _zoomAt(_scale * 1.2, _viewport.center(Offset.zero));
      case 'Out':
        _zoomAt(_scale / 1.2, _viewport.center(Offset.zero));
    }
  }
}
