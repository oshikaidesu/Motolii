// The workspace's Timeline seat: transport + timecode on top, layer labels beside the track area, a ruler with the work area, the playhead.
// The document lives in Ws; this seat keeps only how it looks at it (zoom, scroll, loop, the row toggles, the work area).
import 'dart:developer' show Timeline;
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show DragStartBehavior, PointerScrollEvent, PointerSignalEvent;
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../sets/panels/timeline_parts_parts.dart' show TpGlyph, TpGlyphPainter;
import '../tokens.dart';
import 'key_drag.dart';
import 'timeline_kept.dart';
import 'timeline_paint.dart';
import 'relate.dart';
import 'ws.dart';

class WsTimeline extends StatefulWidget {
  const WsTimeline({super.key});
  @override
  State<WsTimeline> createState() => _WsTimelineState();
}

class _WsTimelineState extends State<WsTimeline> with SingleTickerProviderStateMixin {
  late final Ticker _ticker = createTicker(_tick);
  final _focus = FocusNode(debugLabel: 'timeline');
  Ws? _ws;

  double _zoom = 1, _scroll = 0, _vscroll = 0, _trackW = 1000, _rowsH = 200;
  int _workIn = 12, _workOut = 228;

  int _last = -1;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final ws = WsScope.read(context);
    if (ws != _ws) {
      _ws?.removeListener(_changed);
      _ws?.moment.removeListener(_changed);
      _ws = ws..addListener(_changed);
      ws.moment.addListener(_changed);
      if (ws.playing) SchedulerBinding.instance.addPostFrameCallback((_) => mounted ? _changed() : null);
    }
  }

  @override
  void dispose() {
    _ws?.removeListener(_changed);
    _ws?.moment.removeListener(_changed);
    _ticker.dispose();
    _focus.dispose();
    _markWords.dispose();
    _markFocus.dispose();
    super.dispose();
  }

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

  void _setZoom(double z, {int? anchor}) {
    final nz = z.clamp(1.0, WtM.maxZoom);
    final a = anchor ?? _ws!.frame, ax = _view.x(a);
    setState(() {
      _zoom = nz;
      _scroll = _clampScroll(a - (ax - WtM.padL) / (_fit * nz));
    });
  }

  /// Keeps the playhead in view when zoomed in: the view pages along as play or a scrub carries it off the edge.
  void _follow(int f) {
    if (_zoom <= 1 || !mounted) return;
    final v = _view, x = v.x(f);
    if (x >= 0 && x <= _trackW - WtM.padR) return;
    setState(() => _scroll = _clampScroll(f - _visible * .1));
  }

  void _wheel(PointerSignalEvent e) {
    if (e is! PointerScrollEvent) return;
    final d = e.scrollDelta, sideways = HardwareKeyboard.instance.isShiftPressed || d.dx.abs() > d.dy.abs();
    if (HardwareKeyboard.instance.isMetaPressed || HardwareKeyboard.instance.isControlPressed) {
      _setZoom(_zoom * math.pow(1.0015, -d.dy));
    } else if (sideways) {
      setState(() => _scroll = _clampScroll(_scroll + (d.dx.abs() > d.dy.abs() ? d.dx : d.dy) / _view.ppf));
    } else {
      setState(() => _vscroll = _vscroll + d.dy);
    }
  }

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

  // Joined: a bar held by a hand: by its body it slides, by an end it is cut there. The timeline only says which part
  // is held and how many seconds the hand has gone; what that does to the thing (its limits, a film's head) is the
  // inside's, and the bar is drawn from what the inside then says.
  ({String id, String part, double grab, bool moved})? _bar;

  int _barIn = 0, _barOut = 0;   // (the held bar's ends when it was taken: a cut stops one frame short of nothing, since a bar cut to nothing cannot be seen, taken again, or kept: the document is as long as its longest thing)

  // The pointer over a bar says what it would do: an end cuts, the body slides, a key slides; the room between draws a box.
  MouseCursor _cursor = MouseCursor.defer;
  MouseCursor _cursorAt(Offset p) {
    final ws = _ws!, v = _view, i = ((p.dy + _vscroll) / WtM.row).floor();
    if (i < 0 || i >= ws.layers.length) return MouseCursor.defer;
    final l = ws.layers[i], a = v.x(l.inF), b = v.x(l.outF);
    if (_isLocked(ws, l.id) || p.dx < a - 6 || p.dx > b + 6) return SystemMouseCursors.precise;
    if (l.keys.any((k) => (v.x(k) - p.dx).abs() <= WtM.keyPicked + 2)) return SystemMouseCursors.click;
    return (p.dx - a).abs() <= 6 || (p.dx - b).abs() <= 6 ? SystemMouseCursors.resizeLeftRight : SystemMouseCursors.grab;
  }
  // A key held by a hand: it (and the keys picked with it) slides in time; the playhead shows where, let go it is so.
  ({List<WsKey> keys, int grab, int by})? _slide;

  void _barDown(Offset p) {
  // A box drawn over the empty room of the tracks (any direction): the keys inside it are picked, else the things whose
  // bars it touches (Ableton's and After Effects' marquee). Corners, in the tracks' own points.
  (Offset, Offset)? _box;

    final ws = _ws!, v = _view, m = ws.inside;
    if (!_grab(p)) setState(() => _box = (p, p));
  }

  void _boxUp() {
    final b = _box;
    setState(() => _box = null);
    if (b == null) return;
    final ws = _ws!, v = _view, r = Rect.fromPoints(b.$1, b.$2);
    if (r.width < 4 && r.height < 4) return;   // (a click, not a box: the tap has done it)
    final keys = <WsKey>[], things = <String>[];
    for (var i = 0; i < ws.layers.length; i++) {
      final l = ws.layers[i], top = i * WtM.row - _vscroll;
      if (top + WtM.row < r.top || top > r.bottom || _isLocked(ws, l.id)) continue;
      for (final k in l.keys) {
        if (v.x(k) >= r.left && v.x(k) <= r.right) keys.add((layer: l.id, frame: k));
      }
      if (v.x(l.outF) >= r.left && v.x(l.inF) <= r.right) things.add(l.id);
    }
    if (keys.isNotEmpty) {
      ws.pickKeys(keys);
    } else if (things.isNotEmpty) {
      ws.clearKeys();
      ws.pickMany(things, byHand: false);
    }
  }

  bool _grab(Offset p) {
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
    _bar = (id: l.id, part: (p.dx - b).abs() <= 6 ? 'tail' : ((p.dx - a).abs() <= 6 ? 'head' : 'bar'), grab: v.frameAt(p.dx), moved: false);
    _barIn = l.inF;
    _barOut = l.outF;
  }
    return true;

  void _barMove(Offset p) {
    if (_slide case final s?) {
    if (_box case (final from, _)?) return setState(() => _box = (from, p));
      final by = _view.frameAt(p.dx).round() - s.grab;
      _slide = (keys: s.keys, grab: s.grab, by: by);
      _ws!.frame = (s.grab + by).clamp(0, Ws.duration);
      return;
    }
    final h = _bar, ws = _ws!, m = ws.inside;
    if (h == null || m == null) return;
    var by = (_view.frameAt(p.dx) - h.grab).round();   // (by whole frames)
    if (h.part == 'tail') by = math.max(by, _barIn + 1 - _barOut);
    if (h.part == 'head') by = math.min(by, _barOut - 1 - _barIn);
    m.carry(h.part, by / Ws.fps);
    _bar = (id: h.id, part: h.part, grab: h.grab, moved: true);
    ws.respan(h.id);
  }

  void _barUp({bool keep = true}) {
    if (_slide case final s?) {
    if (_box != null) return keep ? _boxUp() : setState(() => _box = null);
      _slide = null;
      if (keep) wsSlideKeys(_ws!, s.keys, s.by);
      return;
    }
    final h = _bar, ws = _ws!, m = ws.inside;
    _bar = null;
    if (h == null || m == null) return;
    ws.after(m.later('let_go', number: keep && h.moved ? 1 : 0));   // (let go without waiting)
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

  double _lift = 0; // how far a label row has been carried up or down

  int? _workDrag; // 0 in, 1 out, 2 both, null scrub
  double _workGrab = 0;

  // The document's marks on the ruler, as After Effects' composition markers: a double click where there is none puts
  // one there (named by its order), on one names it; one is carried along by its flag (its name kept), and carried off
  // the ruler, up or down, it is taken away. Each one step.
  (double, String)? _markHeld;
  int _markTo = 0;
  bool _markOff = false;
  (double, String)? _naming;
  final _markWords = TextEditingController();
  late final FocusNode _markFocus = FocusNode()..addListener(() => _markFocus.hasFocus ? null : _named());   // (the field left: the name said)

  /// The mark whose flag is under the hand, if any.
  (double, String)? _flagAt(Offset p) {
    if (p.dy < WtM.band + 1 || p.dy > WtM.band + 15) return null;
    for (final m in _ws!.marks.reversed) {
      final x = _view.x((m.$1 * Ws.fps).round());
      if (p.dx >= x - 2 && p.dx <= x + wtFlagWidth(m.$2)) return m;
    }
    return null;
  }

  void _rulerTwice(Offset p) {
    final ws = _ws!, m = ws.inside;
    if (m == null) return;
    if (_flagAt(p) case final flag?) {
      setState(() => _naming = flag);
      _markWords.text = flag.$2;
      _markFocus.requestFocus();
      return;
    }
    final at = _view.frameAt(p.dx).round().clamp(0, Ws.duration) / Ws.fps, marks = ws.marks;
    if (marks.any((k) => (k.$1 - at).abs() < 0.5 / Ws.fps)) return;
    var n = marks.length + 1;
    while (marks.any((k) => k.$2 == 'M$n')) {
      n++;
    }
    ws.after(m.later('mark', number: at, word: 'M$n'));
  }

  void _named() {
    final was = _naming, ws = _ws!, words = _markWords.text.trim();
    if (was == null) return;
    setState(() => _naming = null);
    if (words.isNotEmpty && words != was.$2 && ws.inside != null) ws.after(ws.inside!.later('mark', number: was.$1, word: words));
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

  void _rulerMove(Offset p) {
    final f = _view.frameAt(p.dx).round();
    if (_markHeld != null) {
      return setState(() {
        _markTo = f.clamp(0, Ws.duration);
        _markOff = p.dy < -WtM.ruler * .6 || p.dy > WtM.ruler * 1.6;   // (carried off the ruler: taken away when let go)
      });
    }
    switch (_workDrag) {
      case 0:
        setState(() => _workIn = f.clamp(0, _workOut - Ws.fps ~/ 2));   // (never shut to nothing: half a second at least, so the document cannot be dragged out of sight)
      case 1:
        setState(() => _workOut = f.clamp(_workIn + Ws.fps ~/ 2, Ws.duration));
      case 2:
        final len = _workOut - _workIn, a = (_view.frameAt(p.dx) - _workGrab).round().clamp(0, Ws.duration - len);
        setState(() {
          _workIn = a;
          _workOut = a + len;
        });
      default:
        _ws!.frame = f;
    }
  }

  /// Joined: the work area carried is the document's when the hand lets go (one step).
  void _workLetGo() {
    final m = _ws?.inside;
    if (_markHeld case (final at, final name)?) {
      _markHeld = null;
      final to = _markTo / Ws.fps;
      if (_markOff) {
        _ws!.after(m!.later('mark', number: at));
      } else if ((to - at).abs() >= 0.5 / Ws.fps) {   // (moved: one step, its name kept)
        m!.later('together', number: 1);
        m.later('mark', number: at);
        m.later('mark', number: to, word: name);
        _ws!.after(m.later('together'));
      }
      setState(() {});
      return;
    }
    if (_workDrag != null) _ws!.after(m!.later('work', number: _workIn / Ws.fps, word: '${_workOut / Ws.fps}'));
    _workDrag = null;
  }

  /// The marks on the ruler: the document's own (moments with a name).
  List<(int, String)> _marksOf(Ws ws) {
    return [
      for (final (at, name) in _ws!.marks)
        if ((at, name) != _markHeld) ((at * Ws.fps).round(), name)
        else if (!_markOff) (_markTo, name),   // (the one carried: where the hand has it; off the ruler, gone)
    ];
  }

  void _overview(Offset p, double w) {
    final f = (p.dx - 4) / (w - 8) * Ws.duration;
    setState(() => _scroll = _clampScroll(f - _visible / 2));
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

  // ---------------------------------------------------------------- build

  // The films' beats (their Beats value, the inside's strikes), asked again only when the document changes.
  (int, List<int>)? _beats;
  List<int> _beatsOf(Ws ws) {
    final m = ws.inside;
    if (m == null) return const [];
    if (_beats case (final r, final b) when r == ws.reads) return b;
    final all = <int>[];
    for (final l in ws.layers) {
      final t = int.tryParse(l.id), rise = t == null ? 0.0 : m.value(t, 'beats', 0) ?? 0;
      if (rise > 0 && _waveOf(ws, l) != null) all.addAll([for (final at in m.strikes(t!, rise)) (at * Ws.fps).round()]);   // (only once its sound is read)
    }
    _beats = (ws.reads, all);
    return all;
  }

  // A film's sound on its bar: read once on another isolate (the inside keeps it), then laid on the document's frames
  // where the film shows them, asked again only when the document changes.
  final _waves = <String, (int, int, int, List<double>?)>{};   // (thing: document read, from, to, loudness a frame)
  final _asked = <String>{};
  List<double>? _waveOf(Ws ws, WsLayer l) {
    final m = ws.inside, t = int.tryParse(l.id);
    if (m == null || t == null || m.word(t, 'kind') != 'film') return null;
    if (_waves[l.id] case (final r, final a, final b, final w) when r == ws.reads && a == l.inF && b == l.outF) return w;
    final w = m.loudnessOver(t, l.inF / Ws.fps, l.outF / Ws.fps, l.outF - l.inF);
    if (w == null) {
      if (_asked.add(l.id)) m.loudness(t).then((_) => mounted ? setState(() => (_waves.remove(l.id), _beats = null)) : null);
      return null;
    }
    _waves[l.id] = (ws.reads, l.inF, l.outF, w.isEmpty ? null : w);
    _beats = null;   // (its sound now read: the beats asked again)
    return _waves[l.id]!.$4;
  }

  @override
  Widget build(BuildContext context) {
    if (_workDrag == null) {   // (the document's work area)
      final (from, to) = _ws!.work;
      _workIn = (from * Ws.fps).round();
      _workOut = (to * Ws.fps).round();
    }
    final ws = WsScope.of(context);
    final sel = ws.selected, picks = ws.keys, lit = {...ws.many, ...wsMembersOf(ws)};   // (picked together, or moved by the relation picked: lit with it)
    final rows = <WtRow>[
      for (final l in ws.layers)
        WtRow(
          name: l.name,
          kind: l.kind,
          inF: l.inF,
          outF: l.outF,
          keys: l.keys,
          stretches: l.stretches,
          picked: {
            for (final k in picks)
              if (k.layer == l.id) k.frame,
          },
          selected: l.id == sel || lit.contains(l.id),
          dim: _isHidden(ws, l.id) || (_anySolo(ws) && !_isSolo(ws, l.id)),
          locked: _isLocked(ws, l.id),
          wave: _waveOf(ws, l),
          tone: l.tone,
          mix: _mixOf(ws, l.id),
        ),
    ];
    return Focus(
      focusNode: _focus,
      child: Listener(
        onPointerDown: (_) => _focus.requestFocus(),
        child: ColoredBox(
          color: WsT.body,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _Header(ws: ws, loop: ws.inside!.looping, zoom: _zoom, onToggle: _toggle, onStep: _step, onLoop: () => setState(() => ws.inside!.looping = !ws.inside!.looping), onZoom: _setZoom),
              Expanded(
                child: Row(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    SizedBox(width: WtM.label, child: _labels(ws, rows)),
                    SizedBox(
                      width: WtM.divider,
                      child: ColoredBox(color: WsT.ground),
                    ),
                    Expanded(child: _tracks(ws, rows)),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
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

  Widget _tracks(Ws ws, List<WtRow> rows) => LayoutBuilder(
    builder: (context, box) {
      _trackW = box.maxWidth;
      _rowsH = math.max(0, box.maxHeight - WtM.ruler - WtM.overview);
      _scroll = _clampScroll(_scroll);
      _vscroll = _clampV(rows.length);
      final v = _view;
      return Stack(
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              GestureDetector(
                key: const ValueKey('ws-tl-ruler'),
                behavior: HitTestBehavior.opaque,
                onDoubleTapDown: (d) => _rulerTwice(d.localPosition),
                onDoubleTap: () {},
                onPanDown: (d) => _rulerDown(d.localPosition),
                onPanUpdate: (d) => _rulerMove(d.localPosition),
                onPanEnd: (_) => _workLetGo(),
                onPanCancel: _workLetGo,
                child: SizedBox(
                  height: WtM.ruler,
                  child: CustomPaint(
                    painter: WtRulerPainter(view: v, fps: Ws.fps, workIn: _workIn, workOut: _workOut, markers: _marksOf(_ws!), beats: _beatsOf(ws)),
                    foregroundPainter: WtKeptPainter(view: v, kept: ws.kept), // (the pictures kept on disk for playing)
                  ),
                ),
              ),
              Expanded(
                child: MouseRegion(
                  cursor: _cursor,
                  onHover: (e) {
                    final c = _cursorAt(e.localPosition);
                    if (c != _cursor) setState(() => _cursor = c);
                  },
                  child: Listener(
                  onPointerSignal: _wheel,
                  child: GestureDetector(
                    key: const ValueKey('ws-tl-tracks'),
                    behavior: HitTestBehavior.opaque,
                    dragStartBehavior: DragStartBehavior.down,
                    onTapDown: (d) => _trackDown(d.localPosition),
                    onPanStart: (d) => _barDown(d.localPosition),
                    onPanUpdate: (d) => _barMove(d.localPosition),
                    onPanEnd: (_) => _barUp(),
                    onPanCancel: () => _barUp(keep: false),
                    child: RepaintBoundary(
                      child: CustomPaint(
                        size: Size.infinite,
                        painter: WtTracksPainter(rows: rows, view: v, fps: Ws.fps, vscroll: _vscroll),
                      ),
                    ),
                  ),
                ),
              ),
                ),
              ValueListenableBuilder<int>(valueListenable: ws.moment, builder: (_, frame, _) => _Overview(from: _scroll, to: _scroll + _visible, frame: frame, onDrag: (p) => _overview(p, box.maxWidth))),
            ],
          ),
          Positioned(
            left: 0,
            right: 0,
            top: 0,
            bottom: WtM.overview,
            child: IgnorePointer(
              child: RepaintBoundary(
                child: ClipRect(
                  child: CustomPaint(
                    painter: WtPlayheadPainter(view: v, frame: ws.frame, headTop: WtM.band + 1, moment: ws.moment),   // (painted again as the playhead moves, nothing built)
                  ),
                ),
              ),
            ),
          ),
          if (_naming case (final at, _)?)   // (a mark's name written over its flag)
          if (_box case (final a, final b)?)
            Positioned.fromRect(
              rect: Rect.fromPoints(a, b).shift(Offset(0, WtM.ruler)),
              child: IgnorePointer(
                child: DecoratedBox(
                  decoration: BoxDecoration(color: WsT.accent.withAlpha(36), border: Border.all(color: WsT.accent, width: 1)),
                ),
              ),
            ),
            Positioned(
              left: v.x((at * Ws.fps).round()),
              top: WtM.band + 1,
              width: Shape.z(120),
              height: Shape.z(15),
              child: Container(
                color: Grey.g91,
                padding: EdgeInsets.symmetric(horizontal: Shape.s(4)),
                child: EditableText(
                  key: const ValueKey('ws-tl-mark-name'),
                  controller: _markWords,
                  focusNode: _markFocus,
                  style: T.micro(Grey.g13).copyWith(fontWeight: FontWeight.w700),
                  cursorColor: Grey.g13,
                  backgroundCursorColor: Grey.g38,
                  maxLines: 1,
                  onSubmitted: (_) => _markFocus.unfocus(),
                ),
              ),
            ),
        ],
      );
    },
  );
}

// ---------------------------------------------------------------- header strip

class _Header extends StatelessWidget {
  const _Header({
    required this.ws,
    required this.loop,
    required this.zoom,
    required this.onToggle,
    required this.onStep,
    required this.onLoop,
    required this.onZoom,
  });
  final Ws ws;
  final bool loop;
  final double zoom;
  final VoidCallback onToggle, onLoop;
  final ValueChanged<int> onStep;
  final void Function(double z, {int? anchor}) onZoom;

  @override
  Widget build(BuildContext context) {
    final big = T.value(Grey.g76).copyWith(fontSize: T.px(14), fontWeight: FontWeight.w500, fontFeatures: const [FontFeature.tabularFigures()]);
    final picked = ws.keys.length, layer = ws.layer;
    return Container(
      height: WtM.header,
      padding: EdgeInsets.symmetric(horizontal: WsT.inset),
      decoration: BoxDecoration(
        color: WsT.body,
        border: Border(bottom: BorderSide(color: WsT.line)),
      ),
      child: LayoutBuilder(
        builder: (context, box) {
          final roomy = box.maxWidth >= 1000, mid = box.maxWidth >= 820;
          return Row(
            children: [
              if (mid) ...[_Chip(ws.document?.replaceFirst(RegExp(r'\.motolii$'), '') ?? 'Main', mono: false), SizedBox(width: Shape.s(12))], // (joined: the document's name)
              _Btn(TpGlyph.toStart, key: const ValueKey('ws-tl-start'), onTap: () => onStep(-ws.frame)),
              SizedBox(width: Shape.s(2)),
              _Btn(TpGlyph.stepBack, key: const ValueKey('ws-tl-back'), onTap: () => onStep(-1)),
              SizedBox(width: Shape.s(2)),
              _Btn(ws.playing ? TpGlyph.pause : TpGlyph.play, key: const ValueKey('ws-tl-play'), width: Shape.z(34), on: ws.playing, rest: WsT.accent, onTap: onToggle),
              SizedBox(width: Shape.s(2)),
              _Btn(TpGlyph.stepFwd, key: const ValueKey('ws-tl-fwd'), onTap: () => onStep(1)),
              SizedBox(width: Shape.s(2)),
              _Btn(TpGlyph.loop, key: const ValueKey('ws-tl-loop'), on: loop, onTap: onLoop),
              SizedBox(width: Shape.s(12)),
              ValueListenableBuilder<int>(valueListenable: ws.moment, builder: (_, _, _) => Text(ws.timecode(), style: big)),   // (one number, one weight: not three colours)
              SizedBox(width: Shape.s(8)),
              if (roomy) ...[Text('/ ${ws.timecode(Ws.duration)}', style: T.value(Grey.g56)), SizedBox(width: Shape.s(10))],
              _Chip('${Ws.fps} fps'),
              SizedBox(width: Shape.s(12)),
              if (mid && layer != null)
                Flexible(
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      WtKindChip(layer.kind, tone: layer.tone),
                      SizedBox(width: Shape.s(6)),
                      Flexible(
                        child: FittedBox( // (drawn smaller, never cut)
                          fit: BoxFit.scaleDown,
                          alignment: Alignment.centerLeft,
                          child: Text(
                            picked > 0 ? '${layer.name}  ·  $picked key${picked == 1 ? '' : 's'}' : '${layer.name}  ·  ${layer.keys.length} keys',
                            style: T.label(Grey.g76),
                            maxLines: 1,
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
              const Spacer(),
              _Btn(TpGlyph.minus, key: const ValueKey('ws-tl-zoom-out'), size: Shape.z(22), onTap: () => onZoom(zoom / 1.5)),
              SizedBox(width: Shape.s(4)),
              _ZoomSlider(zoom: zoom, onZoom: onZoom),
              SizedBox(width: Shape.s(4)),
              _Btn(TpGlyph.plus, key: const ValueKey('ws-tl-zoom-in'), size: Shape.z(22), onTap: () => onZoom(zoom * 1.5)),
              SizedBox(width: Shape.s(4)),
              _Btn(TpGlyph.fit, key: const ValueKey('ws-tl-fit'), size: Shape.z(22), width: Shape.z(46), label: 'Fit', lit: zoom <= 1.001, onTap: () => onZoom(1)),
            ],
          );
        },
      ),
    );
  }
}

/// A round touchable: a well that lights on hover; [on] fills it with the accent, [lit] raises it in grey.
class _Btn extends StatefulWidget {
  const _Btn(this.g, {super.key, this.onTap, this.on = false, this.lit = false, this.size = 24, this.width, this.rest, this.label});
  final TpGlyph g;
  final VoidCallback? onTap;
  final bool on, lit;
  final double size;
  final double? width;
  final Color? rest;
  final String? label;
  @override
  State<_Btn> createState() => _BtnState();
}

class _BtnState extends State<_Btn> {
  bool _h = false, _d = false;
  @override
  Widget build(BuildContext context) {
    final on = widget.on;
    final bg = on ? WsT.accent : (_d || widget.lit ? Grey.g26 : (_h ? WsT.raised : const Color(0x00000000)));   // (flat at rest: a button shows itself when the hand is on it)
    final ink = on ? WsT.onAccent : (_h || widget.lit ? Grey.g100 : (widget.rest ?? Grey.g56));
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTapDown: (_) => setState(() => _d = true),
        onTapUp: (_) => setState(() => _d = false),
        onTapCancel: () => setState(() => _d = false),
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: Mo.dur,
          curve: Mo.ease,
          width: widget.width ?? widget.size,
          height: widget.size,
          decoration: BoxDecoration(color: bg, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
          child: Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              CustomPaint(size: const Size.square(11), painter: TpGlyphPainter(widget.g, ink)),
              if (widget.label case final l?) ...[SizedBox(width: Shape.s(4)), Text(l, style: T.micro(ink).copyWith(fontWeight: FontWeight.w700))],
            ],
          ),
        ),
      ),
    );
  }
}

class _Chip extends StatelessWidget {
  const _Chip(this.text, {this.mono = true});
  final String text;
  final bool mono;
  @override
  Widget build(BuildContext context) => Container(
    height: Shape.z(20),
    padding: EdgeInsets.symmetric(horizontal: Shape.s(7)),
    alignment: Alignment.center,
    decoration: BoxDecoration(color: WsT.well, borderRadius: BorderRadius.all(Shape.r(WsT.radius))),
    child: Text(text, style: mono ? T.value(Grey.g91).copyWith(fontSize: T.px(10.5)) : T.micro(Grey.g91).copyWith(fontWeight: FontWeight.w600)),
  );
}

class _ZoomSlider extends StatelessWidget {
  const _ZoomSlider({required this.zoom, required this.onZoom});
  final double zoom;
  final void Function(double z, {int? anchor}) onZoom;
  static double get w => Shape.z(96.0);
  @override
  Widget build(BuildContext context) {
    final t = math.log(zoom) / math.log(WtM.maxZoom);
    void at(Offset p) => onZoom(math.pow(WtM.maxZoom, ((p.dx - 6) / (w - 12)).clamp(0.0, 1.0)).toDouble());
    return MouseRegion(
      cursor: SystemMouseCursors.resizeLeftRight,
      child: GestureDetector(
        key: const ValueKey('ws-tl-zoom'),
        behavior: HitTestBehavior.opaque,
        onPanDown: (d) => at(d.localPosition),
        onPanUpdate: (d) => at(d.localPosition),
        child: SizedBox(
          width: w,
          height: Shape.z(22),
          child: CustomPaint(painter: _ZoomTrack(t)),
        ),
      ),
    );
  }
}

class _ZoomTrack extends CustomPainter {
  const _ZoomTrack(this.t);
  final double t;
  @override
  void paint(Canvas c, Size s) {
    final cy = s.height / 2, x = 6 + (s.width - 12) * t, p = Paint();
    c.drawRRect(RRect.fromLTRBR(0, cy - 3, s.width, cy + 3, Shape.r(3)), p..color = WsT.well);
    c.drawRRect(RRect.fromLTRBR(0, cy - 3, x, cy + 3, Shape.r(3)), p..color = Grey.g44);
    c.drawCircle(Offset(x, cy), 6, p..color = Grey.g91);
  }

  @override
  bool shouldRepaint(_ZoomTrack o) => o.t != t;
}

// ---------------------------------------------------------------- label column

class _LabelHeader extends StatelessWidget {
  const _LabelHeader({required this.count, required this.solo});
  final int count;
  final bool solo;
  @override
  Widget build(BuildContext context) => Container(
    height: WtM.ruler,
    padding: EdgeInsets.only(left: Shape.s(8), right: Shape.s(6)),
    decoration: BoxDecoration(
      color: Grey.g10,
      border: Border(bottom: BorderSide(color: WsT.line)),
    ),
    child: Row(
      children: [
        Text('#', style: T.micro(Grey.g56)),
        SizedBox(width: Shape.s(12)),
        Text('LAYER', style: T.micro(Grey.g63).copyWith(letterSpacing: 1)),
        SizedBox(width: Shape.s(6)),
        Text('$count', style: T.value(Grey.g56).copyWith(fontSize: T.px(10))),
        const Spacer(),
        for (final g in const [TpGlyph.eye, TpGlyph.solo, TpGlyph.lock])
          SizedBox(
            width: Shape.z(20),
            child: Center(
              child: CustomPaint(size: const Size.square(10), painter: TpGlyphPainter(g, g == TpGlyph.solo && solo ? WsT.accent : Grey.g56)),
            ),
          ),
      ],
    ),
  );
}

class _LabelRow extends StatefulWidget {
  const _LabelRow({
    required this.index,
    required this.row,
    required this.hidden,
    required this.solo,
    required this.onSelect,
    required this.onEye,
    required this.onSolo,
    required this.onLock,
  });
  final int index;
  final WtRow row;
  final bool hidden, solo;
  final VoidCallback onSelect, onEye, onSolo, onLock;
  @override
  State<_LabelRow> createState() => _LabelRowState();
}

class _LabelRowState extends State<_LabelRow> {
  bool _h = false;
  @override
  Widget build(BuildContext context) {
    final r = widget.row, sel = r.selected;
    final bg = sel ? wtBand(widget.index, true) : (_h ? Grey.g15 : wtBand(widget.index, false));
    final ink = r.dim ? Grey.g56 : (sel ? Grey.g100 : Grey.g91);
    return MouseRegion(
      onEnter: (_) => setState(() => _h = true),
      onExit: (_) => setState(() => _h = false),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onTap: widget.onSelect,
        child: Container(
          color: bg,
          padding: EdgeInsets.only(bottom: Shape.s(1)),
          child: Stack(
            children: [
              if (sel) Positioned(left: 0, top: 0, bottom: 0, width: Shape.z(3), child: ColoredBox(color: WsT.accent)),
              Padding(
                padding: EdgeInsets.only(left: Shape.s(8), right: Shape.s(6)),
                child: Row(
                  children: [
                    SizedBox(width: 14, child: Text('${widget.index + 1}', style: T.value(sel ? Grey.g91 : Grey.g56).copyWith(fontSize: T.px(10)))),
                    SizedBox(width: Shape.s(6)),
                    WtKindChip(r.kind, dim: r.dim, tone: r.tone),
                    SizedBox(width: Shape.s(7)),
                    Expanded(
                      child: Text(
                        r.name,
                        style: T.name(ink).copyWith(fontWeight: sel ? FontWeight.w700 : FontWeight.w500),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                    _Toggle(widget.hidden ? TpGlyph.eyeOff : TpGlyph.eye, on: false, lit: !widget.hidden, onTap: widget.onEye),
                    _Toggle(TpGlyph.solo, on: widget.solo, lit: _h || sel, onTap: widget.onSolo),
                    _Toggle(r.locked ? TpGlyph.lock : TpGlyph.unlock, on: false, lit: r.locked || _h || sel, strong: r.locked, onTap: widget.onLock),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// A row toggle: a bare glyph; [on] puts it on an accent chip, [strong] on a raised grey one.
class _Toggle extends StatelessWidget {
  const _Toggle(this.g, {required this.on, required this.lit, this.strong = false, required this.onTap});
  final TpGlyph g;
  final bool on, lit, strong;
  final VoidCallback onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    behavior: HitTestBehavior.opaque,
    onTap: onTap,
    child: SizedBox(
      width: Shape.z(20),
      height: WtM.row - 1,
      child: Center(
        child: Container(
          width: Shape.z(17),
          height: Shape.z(17),
          alignment: Alignment.center,
          decoration: BoxDecoration(color: on ? WsT.accent : (strong ? Grey.g26 : null), borderRadius: BorderRadius.all(Shape.r(WsT.chipRadius))),
          child: CustomPaint(size: const Size.square(10), painter: TpGlyphPainter(g, on ? WsT.onAccent : (strong ? Grey.g95 : (lit ? Grey.g76 : Grey.g38)))),
        ),
      ),
    ),
  );
}

// ---------------------------------------------------------------- overview

class _Overview extends StatefulWidget {
  const _Overview({required this.from, required this.to, required this.frame, required this.onDrag});
  final double from, to;
  final int frame;
  final ValueChanged<Offset> onDrag;
  @override
  State<_Overview> createState() => _OverviewState();
}

class _OverviewState extends State<_Overview> {
  bool _h = false;
  @override
  Widget build(BuildContext context) => MouseRegion(
    onEnter: (_) => setState(() => _h = true),
    onExit: (_) => setState(() => _h = false),
    child: GestureDetector(
      behavior: HitTestBehavior.opaque,
      onPanDown: (d) => widget.onDrag(d.localPosition),
      onPanUpdate: (d) => widget.onDrag(d.localPosition),
      child: SizedBox(
        height: WtM.overview,
        child: CustomPaint(
          painter: WtOverviewPainter(from: widget.from, to: widget.to, frame: widget.frame, hot: _h),
        ),
      ),
    ),
  );
}
