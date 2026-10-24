// The workspace's Timeline seat: transport + timecode on top, layer labels beside the track area, a ruler with the work area, the playhead.
// The document lives in Ws; this seat keeps only how it looks at it (zoom, scroll, loop, the row toggles, the work area).
import 'dart:developer' show Timeline;
import 'dart:math' as math;

import 'package:flutter/gestures.dart' show DragStartBehavior, PointerScrollEvent, PointerSignalEvent;
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../forms/forms.dart';
import '../sets/panels/timeline_parts_parts.dart' show TpGlyph, TpGlyphPainter;
import '../tokens.dart';
import 'key_drag.dart';
import 'timeline_kept.dart';
import 'timeline_paint.dart';
import 'relate.dart';
import 'ws.dart';
part 'timeline_2.dart';
part 'timeline_3.dart';

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
      _ws?.on[WsAspect.playing]!.removeListener(_changed);
      _ws?.moment.removeListener(_changed);
      _ws = ws..on[WsAspect.playing]!.addListener(_changed);
      ws.moment.addListener(_changed);
      if (ws.playing) SchedulerBinding.instance.addPostFrameCallback((_) => mounted ? _changed() : null);
    }
  }

  @override
  void dispose() {
    _ws?.on[WsAspect.playing]!.removeListener(_changed);
    _ws?.moment.removeListener(_changed);
    _ticker.dispose();
    _focus.dispose();
    _markWords.dispose();
    _markFocus.dispose();
    super.dispose();
  }

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

  // Joined: a bar held by a hand: by its body it slides, by an end it is cut there. The timeline only says which part
  // is held and how many seconds the hand has gone; what that does to the thing (its limits, a film's head) is the
  // inside's, and the bar is drawn from what the inside then says.
  ({String id, String part, double grab, bool moved})? _bar;
  int _barIn = 0, _barOut = 0;   // (the held bar's ends when it was taken: a cut stops one frame short of nothing, since a bar cut to nothing cannot be seen, taken again, or kept: the document is as long as its longest thing)

  // The pointer over a bar says what it would do: an end cuts, the body slides, a key slides; the room between draws a box.
  MouseCursor _cursor = MouseCursor.defer;

  // A key held by a hand: it (and the keys picked with it) slides in time; the playhead shows where, let go it is so.
  ({List<WsKey> keys, int grab, int by})? _slide;

  // A box drawn over the empty room of the tracks (any direction): the keys inside it are picked, else the things whose
  // bars it touches (Ableton's and After Effects' marquee). Corners, in the tracks' own points.
  (Offset, Offset)? _box;

  void _barDown(Offset p) {
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

  void _barMove(Offset p) {
    if (_box case (final from, _)?) return setState(() => _box = (from, p));
    if (_slide case final s?) {
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
    if (_box != null) return keep ? _boxUp() : setState(() => _box = null);
    if (_slide case final s?) {
      _slide = null;
      if (keep) wsSlideKeys(_ws!, s.keys, s.by);
      return;
    }
    final h = _bar, ws = _ws!, m = ws.inside;
    _bar = null;
    if (h == null || m == null) return;
    ws.after(m.later('let_go', number: keep && h.moved ? 1 : 0));   // (let go without waiting)
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

  void _overview(Offset p, double w) {
    final f = (p.dx - 4) / (w - 8) * Ws.duration;
    setState(() => _scroll = _clampScroll(f - _visible / 2));
  }

  // ---------------------------------------------------------------- build

  // The films' beats (their Beats value, the inside's strikes), asked again only when the document changes: a question
  // of the inside (another isolate), shown when it comes.
  (int, List<int>)? _beats;
  int? _beatsAsked;
  List<int> _beatsOf(Ws ws) {
    final m = ws.inside;
    if (m == null) return const [];
    if (_beats case (final r, final b) when r == ws.reads) return b;
    if (_beatsAsked == ws.reads) return _beats?.$2 ?? const [];
    final asks = <Future<List<double>>>[];
    for (final l in ws.layers) {
      final t = int.tryParse(l.id), rise = t == null ? 0.0 : m.value(t, 'beats', 0) ?? 0;
      if (rise > 0 && _waveOf(ws, l) != null) asks.add(m.strikes(t!, rise));   // (only once its sound is read)
    }
    final reads = _beatsAsked = ws.reads;
    Future.wait(asks).then((all) => mounted ? setState(() => _beats = (reads, [for (final s in all) for (final at in s) (at * Ws.fps).round()])) : null);
    return _beats?.$2 ?? const [];
  }

  // A film's sound on its bar: read once on another isolate (the inside keeps it), then laid on the document's frames
  // where the film shows them, asked again only when the document changes.
  final _waves = <String, (int, int, int, List<double>?)>{};   // (thing: document read, from, to, loudness a frame)
  final _asked = <String>{};
  final _waveAsked = <String, (int, int, int)>{};
  List<double>? _waveOf(Ws ws, WsLayer l) {
    final m = ws.inside, t = int.tryParse(l.id);
    if (m == null || t == null || m.word(t, 'kind') != 'film') return null;
    final key = (ws.reads, l.inF, l.outF);
    final had = _waves[l.id];
    if (had case (final r, final a, final b, final w) when (r, a, b) == key) return w;
    if (_waveAsked[l.id] != key) {   // (a question of the inside, another isolate: laid on the bar when it comes)
      _waveAsked[l.id] = key;
      m.loudnessOver(t, l.inF / Ws.fps, l.outF / Ws.fps, l.outF - l.inF).then((w) {
        if (!mounted) return;
        if (w == null) {
          if (_asked.add(l.id)) m.loudness(t).then((_) => mounted ? setState(() => (_waveAsked.remove(l.id), _beats = null, _beatsAsked = null)) : null);
          return;
        }
        setState(() => (_waves[l.id] = (key.$1, key.$2, key.$3, w.isEmpty ? null : w), _beats = null, _beatsAsked = null));   // (its sound now read: the beats asked again)
      });
    }
    return had?.$4;
  }

  @override
  Widget build(BuildContext context) {
    if (_workDrag == null) {   // (the document's work area)
      final (from, to) = _ws!.work;
      _workIn = (from * Ws.fps).round();
      _workOut = (to * Ws.fps).round();
    }
    final ws = WsScope.of(context, {WsAspect.document, WsAspect.picked});
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
          if (_box case (final a, final b)?)
            Positioned.fromRect(
              rect: Rect.fromPoints(a, b).shift(Offset(0, WtM.ruler)),
              child: IgnorePointer(
                child: DecoratedBox(
                  decoration: BoxDecoration(color: WsT.accent.withAlpha(36), border: Border.all(color: WsT.accent, width: 1)),
                ),
              ),
            ),
          if (_naming case (final at, _)?)   // (a mark's name written over its flag)
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
