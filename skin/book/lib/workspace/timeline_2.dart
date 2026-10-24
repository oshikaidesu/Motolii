// Part of timeline.dart: _WsTimelineState's calls as they were, moved out of its body by size into an extension on it in
// the same library (its private names and its callers as before).
part of 'timeline.dart';

extension _WsTimelineCalls on _WsTimelineState {
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

  // A relation's row: the label colours of the things it moves, in its order (none: a thing's row, one colour).
  List<Color>? _mixOf(Ws ws, String id) {
    final m = ws.inside, t = int.tryParse(id);
    if (m == null || t == null || m.word(t, 'kind') != 'behaviour' || !wsRelations.contains(m.word(t, 'file'))) return null;
    return [for (final s in m.word(t, 'members').split(' ')) if (s.isNotEmpty && s != '_') wsLabel(s)];
  }

  // The eye of a row. Joined: the thing is put aside in the document (visible 0) and is not on the stage; one step.
  bool _isHidden(Ws ws, String id) => ws.layers.any((l) => l.id == id && l.hidden);   // (as last read: no call to the inside while a row is built)

  void _eye(Ws ws, String id) {
    ws.after(ws.inside!.later('set', thing: int.parse(id), name: 'visible', number: _isHidden(ws, id) ? 1 : 0));
  }

  /// The mark whose flag is under the hand, if any.
  (double, String)? _flagAt(Offset p) {
    if (p.dy < WtM.band + 1 || p.dy > WtM.band + 15) return null;
    for (final m in _ws!.marks.reversed) {
      final x = _view.x((m.$1 * Ws.fps).round());
      if (p.dx >= x - 2 && p.dx <= x + wtFlagWidth(m.$2)) return m;
    }
    return null;
  }

  void _rulerDown(Offset p) {
    final v = _view, f = v.frameAt(p.dx);
    _workDrag = null;
    if (_flagAt(p) case final flag?) {
      _markHeld = flag;
      _markTo = (flag.$1 * Ws.fps).round();
      _markOff = false;
      return;
    }
    if (p.dy <= WtM.band + 2) {
      final a = v.x(_workIn), b = v.x(_workOut);
      if ((p.dx - a).abs() <= 6) {
        _workDrag = 0;
      } else if ((p.dx - b).abs() <= 6) {
        _workDrag = 1;
      } else if (p.dx > a && p.dx < b) {
        _workDrag = 2;
        _workGrab = f - _workIn;
      }
    }
    _rulerMove(p);
  }

  /// The marks on the ruler: the document's own (moments with a name).
  List<(int, String)> _marksOf(Ws ws) {
    return [
      for (final (at, name) in _ws!.marks)
        if ((at, name) != _markHeld) ((at * Ws.fps).round(), name)
        else if (!_markOff) (_markTo, name),   // (the one carried: where the hand has it; off the ruler, gone)
    ];
  }

  // Joined: solo and lock are the document's (After Effects keeps both in its project): while any thing is solo only
  // those are seen; a locked thing is not changed (the inside refuses, and says so). Else the sample's own sets.
  bool _isSolo(Ws ws, String id) => ws.layers.any((l) => l.id == id && l.solo);

  bool _isLocked(Ws ws, String id) => ws.layers.any((l) => l.id == id && l.locked);

  bool _anySolo(Ws ws) => ws.layers.any((l) => _isSolo(ws, l.id));

  void _flag(Ws ws, String name, String id) {
    final m = ws.inside!;
    ws.after(m.later('set', thing: int.parse(id), name: name, number: (name == 'solo' ? _isSolo(ws, id) : _isLocked(ws, id)) ? 0 : 1));
  }

  Widget _labels(Ws ws, List<WtRow> rows) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      _LabelHeader(count: ws.layers.length, solo: _anySolo(ws)),
      Expanded(
        child: Listener(
          onPointerSignal: _wheel,
          child: ClipRect(
            child: LayoutBuilder(
              builder: (context, box) {
                final top = _clampV(rows.length);
                return Stack(
                  children: [
                    for (var i = 0; i < rows.length; i++)
                      Positioned(
                        key: ValueKey('ws-tl-label-${ws.layers[i].id}'),
                        left: 0,
                        right: 0,
                        top: i * WtM.row - top,
                        height: WtM.row,
                        // Joined: a row carried up or down is the thing brought nearer or further back, by as many rows as it was carried.
                        child: GestureDetector(
                          dragStartBehavior: DragStartBehavior.down, // (all of the way it was carried counts, from where the hand came down)
                          onVerticalDragStart: (_) => _lift = 0,
                          onVerticalDragUpdate: (d) => _lift += d.delta.dy,
                          onVerticalDragEnd: (_) {
                                  final by = (_lift / WtM.row).round();
                                  if (by != 0) ws.after(ws.inside!.later('raise', thing: int.parse(ws.layers[i].id), number: -by.toDouble()));
                                },
                          child: _LabelRow(
                            index: i,
                            row: rows[i],
                            hidden: _isHidden(ws, ws.layers[i].id),
                            solo: _isSolo(ws, ws.layers[i].id),
                            onSelect: () => HardwareKeyboard.instance.isShiftPressed
                                ? wsPickRange(ws, ws.layers[i].id)   // (Shift: every row from the last picked to this, After Effects' manner)
                                : HardwareKeyboard.instance.isMetaPressed ? wsPickAlso(ws, ws.layers[i].id) : ws.select(ws.layers[i].id),   // (Cmd: one more, or one less)
                            onEye: () => _eye(ws, ws.layers[i].id),
                            onSolo: () => _flag(ws, 'solo', ws.layers[i].id),
                            onLock: () => _flag(ws, 'lock', ws.layers[i].id),
                          ),
                        ),
                      ),
                  ],
                );
              },
            ),
          ),
        ),
      ),
    ],
  );
}
