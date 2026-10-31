// timeline_hand.dart: a hand on the tracks: what the pointer would do over a bar, a bar or a key taken.
part of 'timeline.dart';

extension _WsTimelineHand on _WsTimelineState {
  // ---------------------------------------------------------------- input

  // The row under a height: its place, its layer, and the value whose lane it is (null: the layer's own row).
  ({int i, WsLayer l, String? value})? _rowAt(double y) {
    final i = ((y + _vscroll) / WtM.row).floor();
    if (i < 0 || i >= _order.length) return null;
    return (i: i, l: _ws!.layers[_order[i].layer], value: _order[i].value);
  }

  // A press on a value's lane: on a breakpoint it is taken away, as on Ableton's envelope (Live 12 manual 25.5.2). Between
  // two it picks that stretch, as Theatre.js's connector (the maker, 10/31: Live's line takes a breakpoint at one press, and
  // "区間を選択できない"), and a second press in the same place puts a key there, as Live's double click off the line.
  void _laneTap(Offset p) {
    final ws = _ws!, m = ws.inside, r = _rowAt(p.dy), v = _view;
    if (m == null || r == null || r.value == null || _isLocked(ws, r.l.id)) return;
    final t = int.parse(r.l.id), name = r.value!, row = _laneRow(ws, r.l, name), pts = row.lane ?? const [];
    if (pts.isEmpty) return;
    for (final k in m.keyMoments(t, name)) {
      if ((v.x((k * Ws.fps).round()) - p.dx).abs() <= WtM.keyPicked + 2) return ws.after(m.later('unkey', thing: t, name: name, number: k));
    }
    final now = DateTime.now(), twice = _lanePressed != null && now.difference(_lanePressed!) < kDoubleTapTimeout && (p - _laneAt).distance < kDoubleTapSlop;
    _lanePressed = twice ? null : now;
    _laneAt = p;
    final f = v.frameAt(p.dx);
    if (twice) return ws.after(m.later('key_here', thing: t, name: name, number: f.round().clamp(0, Ws.duration) / Ws.fps));
    final from = pts.where((q) => q.$1 <= f).lastOrNull?.$1, inside = from != null && pts.any((q) => q.$1 > f);
    inside ? ws.pickStretch(r.l.id, name, from) : ws.clearKeys();
  }

  void _trackDown(Offset p) {
    final ws = _ws!, v = _view;
    final r = _rowAt(p.dy);
    if (r == null) { // (below the rows: nothing picked, After Effects' manner; joined, the inspector is then the whole picture's)
      ws.clearKeys();
      ws.select(null);
      return;
    }
    if (r.value != null) return;   // (a lane: its tap is the lane's, on the hand's way up)
    final i = r.i, l = r.l;
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
    final ws = _ws!, v = _view, r = _rowAt(p.dy);
    if (r == null) return MouseCursor.defer;
    final l = r.l, a = v.x(l.inF), b = v.x(l.outF);
    if (r.value case final n?) {   // (a lane: over a breakpoint, it is taken by a click)
      final t = int.parse(l.id);
      final at = ws.inside!.keyMoments(t, n), f = v.frameAt(p.dx) / Ws.fps;
      return at.any((k) => (v.x((k * Ws.fps).round()) - p.dx).abs() <= WtM.keyPicked + 2) || (at.isNotEmpty && f > at.first && f < at.last) ? SystemMouseCursors.click : SystemMouseCursors.precise;
    }
    if (_isLocked(ws, l.id) || p.dx < a - 6 || p.dx > b + 6) return SystemMouseCursors.precise;
    if (l.keys.any((k) => (v.x(k) - p.dx).abs() <= WtM.keyPicked + 2)) return SystemMouseCursors.click;
    return (p.dx - a).abs() <= 6 || (p.dx - b).abs() <= 6 ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.grab;
  }

  bool _grab(Offset p) {
    final ws = _ws!, v = _view, m = ws.inside;
    final r = _rowAt(p.dy);
    if (m == null || r == null) return false;
    if (r.value != null) return true;   // (a lane: nothing carried, and no box drawn from it)
    final l = r.l, a = v.x(l.inF), b = v.x(l.outF);
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
