part of '../stage.dart';

/// Frames between a window request and the first status that carries it
/// (kDebugMode). 0 = the picture built in the frame that asked.
int _frameCount = 0;
bool _counting = false;

/// The window Rust reports comes back through f32, so the roi is compared
/// within a hundredth of a pixel.
bool _sameWindow(Object? a, Object? b) {
  if (a is! Map || b is! Map) return false;
  if (a['width'] != b['width'] || a['height'] != b['height']) return false;
  final ra = a['roi'], rb = b['roi'];
  if (ra is! List || rb is! List || ra.length != rb.length) return false;
  for (var i = 0; i < ra.length; i++) {
    if (((ra[i] as num) - (rb[i] as num)).abs() >= 0.01) return false;
  }
  return true;
}

/// Handing the picture to native: whether this tab is being looked at, how
/// big its window is in pixels, and which part of the composition it shows.
/// The only place that speaks `stageWindow`.
mixin _StageWindow on State<StagePanel>, _StageView {
  /// The dock keeps every tab built. A tab that comes into view takes its own
  /// texture; the Stage tab also places its window, and withdraws it when hidden
  /// so native draws only the pictures somebody is looking at.
  bool _shown = false;
  void _runtimeChanged() {
    _windowKey = null;
    _sentWindow = null;
    _drawnWindow = null;
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _windowKey = null;
    final shown = Visibility.of(context);
    if (shown == _shown) return;
    _shown = shown;
    if (shown) {
      c.attachView(widget.view);
      return;
    }
    c.detachView(widget.view);
    if (_userStage) {
      _sentWindow = null;
      if (c.supports('stageWindow'))
        c.command('stageWindow', {'width': 0, 'height': 0});
    }
  }

  /// What the Stage tab asks native to draw: its own pixel size and the part of
  /// the composition image its zoom and pan put on screen. Sent only on change.
  Map<String, dynamic>? _sentWindow;

  /// Everything [_syncWindow] reads; a build that leaves these alone schedules nothing.
  (Offset, double, Size, bool, bool)? _windowKey;
  void _queueWindowSync() {
    final key = (_origin, _scale, _viewport, _shown, c.supports('stageWindow'));
    if (key == _windowKey) return;
    _windowKey = key;
    // Same frame when the runtime answers in it; the channel path waits for
    // the frame to end, as a reply cannot land inside a build.
    if (c.sameFrame) {
      _syncWindow();
    } else {
      WidgetsBinding.instance.addPostFrameCallback((_) => _syncWindow());
    }
  }

  /// The window the texture was drawn for in this very frame (same-frame
  /// path); null once the rendered status has caught up with it.
  Map<String, dynamic>? _drawnWindow;

  int? _askedAt;
  Map<String, dynamic>? _askedWindow;
  void _countFrames() {
    if (_counting) return;
    _counting = true;
    SchedulerBinding.instance.addPersistentFrameCallback((_) => _frameCount++);
  }

  /// The Stage picture is always the comp's shape (Lumit / AE: the viewer's
  /// texture never changes aspect), sized to cover the tab and centred on it.
  /// A resize only changes how much of it the tab shows, so a frame that
  /// arrives late is scaled uniformly, never squashed.
  Rect _coverBox() {
    final vw = _viewport.width, vh = _viewport.height;
    final aspect = _width / _height;
    final (bw, bh) = vw / vh > aspect ? (vw, vw / aspect) : (vh * aspect, vh);
    return Rect.fromLTWH((vw - bw) / 2, (vh - bh) / 2, bw, bh);
  }

  /// Screen rectangle of a rendered Stage window (`stageWindow` in the frame's
  /// status: the comp-space roi it was drawn for) under the current view.
  Rect? _frameRect(Object? window) {
    if (window is! Map) return null;
    final roi = window['roi'];
    if (roi is! List || roi.length != 4 || !roi.every((v) => v is num)) {
      return null;
    }
    final origin = _origin, scale = _scale;
    return Rect.fromLTWH(
      origin.dx + (roi[0] as num) * scale,
      origin.dy + (roi[1] as num) * scale,
      (roi[2] as num) * scale,
      (roi[3] as num) * scale,
    );
  }

  void _syncWindow() {
    if (!_userStage || !_shown || !mounted || !c.supports('stageWindow'))
      return;
    if (_viewport.isEmpty) return;
    final ratio = MediaQuery.maybeDevicePixelRatioOf(context) ?? 1;
    final origin = _origin, scale = _scale;
    final box = _coverBox();
    final window = {
      'width': (box.width * ratio).round(),
      'height': (box.height * ratio).round(),
      'roi': [
        (box.left - origin.dx) / scale,
        (box.top - origin.dy) / scale,
        box.width / scale,
        box.height / scale,
      ],
    };
    if (sameValue(window, _sentWindow) && c.state['stageWindow'] != null)
      return;
    _sentWindow = window;
    if (kDebugMode) {
      _countFrames();
      _askedAt = _frameCount;
      _askedWindow = window;
    }
    _drawnWindow = c.commandNow('stageWindow', window) ? window : null;
  }

  void _noteWindowLag(Object? rendered) {
    if (!kDebugMode) return;
    final asked = _askedWindow;
    if (asked == null || _askedAt == null) return;
    final matched = _drawnWindow != null
        ? identical(_drawnWindow, asked)
        : _sameWindow(asked, rendered);
    if (!matched) return;
    debugPrint(
      'PROBE room=stage-window verdict=matched lag=${_frameCount - _askedAt!} '
      'frames path=${_drawnWindow != null ? 'ffi-same-frame' : 'status'} '
      'window=${asked['width']}x${asked['height']} roi=${asked['roi']}',
    );
    _askedWindow = null;
    _askedAt = null;
  }
}
