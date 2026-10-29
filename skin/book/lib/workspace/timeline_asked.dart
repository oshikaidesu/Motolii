// timeline_asked.dart: what the window's keys ask of how the timeline looks (ableton.dart: Ableton Live's zoom,
// loop and rename), which the timeline keeps itself; and the ruler as Live's scrub area and locators (Live 12 manual,
// chapter 6 "Arrangement View": 6.3 and 6.4).
part of 'timeline.dart';

mixin _TimelineAsked on State<WsTimeline> {
  _WsTimelineState get _t => this as _WsTimelineState;   // (the timeline it is part of: its zoom, its scroll)

  (double, double)? _before;   // (the zoom and scroll before Z: X goes back to them, Live 42.16)
  String? _renaming;   // (the row whose name is being written: Cmd+R, Live 42.5)
  final _renameWords = TextEditingController();
  late final FocusNode _renameFocus = FocusNode()..addListener(() => _renameFocus.hasFocus ? null : _renamed());

  void _hearAsks() {
    wsTimelineAsk.addListener(_heard);
    wsAskRename.addListener(_rename);
  }

  void _stopAsks() {
    wsTimelineAsk.removeListener(_heard);
    wsAskRename.removeListener(_rename);
    _renameWords.dispose();
    _renameFocus.dispose();
  }

  void _heard() {
    final t = _t, ws = t._ws, m = ws?.inside;
    if (!mounted || ws == null || m == null) return;
    switch (wsTimelineAsk.value.$1) {
      case 'loop':
        setState(() => m.looping = !m.looping);
      case 'in':
        t._setZoom(t._zoom * 1.5);
      case 'out':
        t._setZoom(t._zoom / 1.5);
      case 'fit':
        t._setZoom(1);
      case 'selection':   // (the picked keys, else the picked thing's bar, fill the view: Live's Z)
        final frames = ws.keys.isNotEmpty ? [for (final k in ws.keys) k.frame] : [if (ws.layer case final l?) ...[l.inF, l.outF]];
        if (frames.isEmpty) return;
        final a = frames.reduce((p, q) => p < q ? p : q), b = frames.reduce((p, q) => p > q ? p : q), span = (b - a).clamp(6, Ws.duration);
        _before ??= (t._zoom, t._scroll);
        t._setZoom(Ws.duration / (span * 1.2), anchor: a);
        setState(() => t._scroll = t._clampScroll(a - span * .1));
      case 'back':
        if (_before case (final z, final s)?) {
          _before = null;
          t._setZoom(z);
          setState(() => t._scroll = t._clampScroll(s));
        }
    }
  }

  /// Played from where it now is (stopped first if playing: the clock goes where the playhead is put).
  void _playAt(int f) {
    final ws = _t._ws!;
    ws.playing = false;
    ws.frame = f;
    ws.playing = true;
  }

  /// A click on the ruler: on a locator, it is picked and the playhead goes there (6.4); elsewhere the playhead goes
  /// there, as a video editor's time ruler puts its current-time indicator (Premiere Pro, After Effects): played on from
  /// there if it was playing, else left standing; the work area's band is the work area's.
  void _rulerTap(Offset p) {
    final t = _t, ws = t._ws;
    if (ws == null) return;
    final int f;
    if (t._flagAt(p) case final flag?) {
      wsMarkPicked.value = flag;
      f = (flag.$1 * Ws.fps).round();
    } else {
      wsMarkPicked.value = null;
      if (p.dy <= WtM.band + 2) return;
      f = t._view.frameAt(p.dx).round().clamp(0, Ws.duration);
    }
    if (ws.playing) return _playAt(f);
    ws.frame = f;
  }

  /// A double click: on a locator, play from it (6.4); elsewhere, zoom to what is picked, else to the whole (6.1).
  void _rulerDouble(Offset p) {
    if (_t._flagAt(p) case final flag?) return _playAt((flag.$1 * Ws.fps).round());
    final ws = _t._ws;
    if (ws != null && (ws.keys.isNotEmpty || ws.layer != null)) return wsAskTimeline('selection');
    _t._setZoom(1);
  }

  /// A right click: a locator put there, named by its order (6.4: Add Locator); on one, it is picked.
  void _rulerRight(Offset p) {
    final ws = _t._ws, m = ws?.inside;
    if (ws == null || m == null) return;
    if (_t._flagAt(p) case final flag?) {
      wsMarkPicked.value = flag;
      return;
    }
    final at = _t._view.frameAt(p.dx).round().clamp(0, Ws.duration) / Ws.fps, marks = ws.marks;
    if (marks.any((k) => (k.$1 - at).abs() < 0.5 / Ws.fps)) return;
    var n = marks.length + 1;
    while (marks.any((k) => k.$2 == 'M$n')) {
      n++;
    }
    ws.after(m.later('mark', number: at, word: 'M$n'));
  }

  void _rename() {
    final ws = _t._ws;
    if (!mounted || ws == null) return;
    if (wsMarkPicked.value case final flag?) {   // (a picked locator is named, in its flag)
      setState(() => _t._naming = flag);
      _t._markWords.text = flag.$2;
      return _t._markFocus.requestFocus();
    }
    if (ws.layer == null) return;
    setState(() => _renaming = ws.layer!.id);
    _renameWords.text = ws.layer!.name;
    _renameWords.selection = TextSelection(baseOffset: 0, extentOffset: _renameWords.text.length);
    _renameFocus.requestFocus();
  }

  void _renamed() {
    final id = _renaming, ws = _t._ws, words = _renameWords.text.trim();
    if (id == null) return;
    setState(() => _renaming = null);
    final t = int.tryParse(id), was = ws?.layers.where((l) => l.id == id).firstOrNull?.name;
    if (t != null && words.isNotEmpty && words != was && ws?.inside != null) ws!.after(ws.inside!.later('say', thing: t, name: 'name', word: words));
  }

  /// The name field over the row being renamed (Enter or clicking away keeps it, Esc leaves the name as it was).
  Widget? _renameField(int i, String id, double top) => _renaming != id
      ? null
      : Positioned(
          left: Shape.s(28),
          right: Shape.s(4),
          top: top + Shape.s(4),
          height: WtM.row - Shape.s(8),
          child: CallbackShortcuts(
            bindings: {
              const SingleActivator(LogicalKeyboardKey.escape): () {
                _renameWords.text = _t._ws?.layers.where((l) => l.id == id).firstOrNull?.name ?? '';
                _renameFocus.unfocus();
              },
            },
            child: Container(
              color: Grey.g91,
              padding: EdgeInsets.symmetric(horizontal: Shape.s(4)),
              alignment: Alignment.centerLeft,
              child: EditableText(
                key: ValueKey('ws-tl-rename-$id'),
                controller: _renameWords,
                focusNode: _renameFocus,
                style: T.label(Grey.g13).copyWith(fontWeight: FontWeight.w700),
                cursorColor: Grey.g13,
                backgroundCursorColor: Grey.g38,
                maxLines: 1,
                onSubmitted: (_) => _renameFocus.unfocus(),
              ),
            ),
          ),
        );
}
