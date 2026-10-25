// timeline_hand.dart: a hand on the tracks: what the pointer would do over a bar, a bar or a key taken.
part of 'timeline.dart';

extension _WsTimelineHand on _WsTimelineState {
  // ---------------------------------------------------------------- input

  void _trackDown(Offset p) {
    final ws = _ws!, v = _view;
    final i = ((p.dy + _vscroll) / WtM.row).floor();
    if (i < 0 || i >= ws.layers.length) { // (below the rows: nothing picked, After Effects' manner; joined, the inspector is then the whole picture's)
      ws.clearKeys();
      ws.select(null);
      return;
    }
    final l = ws.layers[i];
    final cy = i * WtM.row - _vscroll + WtM.row / 2;
    if (!_isLocked(ws, l.id) && (p.dy - cy).abs() <= WtM.bar / 2 + 2) {
      int? best;
      var bd = WtM.keyPicked + 2;
      for (final k in l.keys) {
        final d = (v.x(k) - p.dx).abs();
        if (d <= bd) {
          bd = d;
          best = k;
        }
      }
      if (best != null) return ws.pickKey((layer: l.id, frame: best), add: HardwareKeyboard.instance.isShiftPressed);
    }
    final f = v.frameAt(p.dx);
    if (f >= l.inF && f <= l.outF) {
      ws.select(l.id);
    } else {
      ws.clearKeys();
    }
  }

  MouseCursor _cursorAt(Offset p) {
    final ws = _ws!, v = _view, i = ((p.dy + _vscroll) / WtM.row).floor();
    if (i < 0 || i >= ws.layers.length) return MouseCursor.defer;
    final l = ws.layers[i], a = v.x(l.inF), b = v.x(l.outF);
    if (_isLocked(ws, l.id) || p.dx < a - 6 || p.dx > b + 6) return SystemMouseCursors.precise;
    if (l.keys.any((k) => (v.x(k) - p.dx).abs() <= WtM.keyPicked + 2)) return SystemMouseCursors.click;
    return (p.dx - a).abs() <= 6 || (p.dx - b).abs() <= 6 ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.grab;
  }

  bool _grab(Offset p) {
    final ws = _ws!, v = _view, m = ws.inside;
    final i = ((p.dy + _vscroll) / WtM.row).floor();
    if (m == null || i < 0 || i >= ws.layers.length) return false;
    final l = ws.layers[i], a = v.x(l.inF), b = v.x(l.outF);
    if (_isLocked(_ws!, l.id) || p.dx < a - 6 || p.dx > b + 6) return false;
    for (final k in l.keys) {
      if ((v.x(k) - p.dx).abs() > WtM.keyPicked + 2) continue;
      final held = (layer: l.id, frame: k), all = ws.keys.contains(held) ? ws.keys.toList() : [held];
      if (!ws.keys.contains(held)) ws.pickKey(held);
      if (ws.playing) ws.playing = false;
      _slide = (keys: all, grab: k, by: 0);
      return true;
    }
    m.take(int.parse(l.id));
    ws.select(l.id);
    if (ws.playing) ws.playing = false;
    _barIn = l.inF;
    _barOut = l.outF;
    _bar = (id: l.id, part: (p.dx - b).abs() <= 6 ? 'tail' : ((p.dx - a).abs() <= 6 ? 'head' : 'bar'), grab: v.frameAt(p.dx), moved: false);
    return true;
  }
}
