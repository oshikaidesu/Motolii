// timeline_view.dart: the timeline's clock and view: the playhead ticked, played and stepped; how much of the document it shows (fit, zoom, scroll).
part of 'timeline.dart';

extension _WsTimelineView on _WsTimelineState {
  // ---------------------------------------------------------------- play

  void _changed() {
    final ws = _ws!;
    if (ws.playing && !_ticker.isActive) {
      _ticker.start();
      _last = -1;
      if (ws.frame >= Ws.duration) ws.frame = 0;
    } else if (!ws.playing && _ticker.isActive) {
      _ticker.stop();
    }
    _follow(ws.frame);
  }

  void _tick(Duration e) => Timeline.timeSync('ws tick', () => _tickNow(e)); // (named in Flutter's timeline: frames_test TRACE)

  void _tickNow(Duration e) {
    final ws = _ws!;
    _last = (ws.inside!.drawnAt * Ws.fps).round().clamp(0, Ws.duration); // (the inside keeps the clock, the playhead shows the moment of the picture it drew last)
    ws.frame = _last;
  }

  void _toggle() {
    final ws = _ws!;
    ws.playing = !ws.playing;
  }

  void _step(int d) {
    final ws = _ws!;
    ws.playing = false;
    ws.frame = ws.frame + d;
  }

  // ---------------------------------------------------------------- view

  double get _fit => math.max(.1, (_trackW - WtM.padL - WtM.padR) / Ws.duration);

  WtView get _view => WtView(_fit * _zoom, _scroll);

  double get _visible => Ws.duration / _zoom;

  double _clampScroll(double s) => s.clamp(0.0, math.max(0.0, Ws.duration - _visible)).toDouble();

  double _clampV(int rows) => _vscroll.clamp(0.0, math.max(0.0, rows * WtM.row - _rowsH)).toDouble();
}
