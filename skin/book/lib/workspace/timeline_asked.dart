// timeline_asked.dart: what the window's keys ask of how the timeline looks (ableton.dart: Ableton Live's zoom,
// loop and rename), which the timeline keeps itself.
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

  void _rename() {
    final ws = _t._ws;
    if (!mounted || ws?.layer == null) return;
    setState(() => _renaming = ws!.layer!.id);
    _renameWords.text = ws!.layer!.name;
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
