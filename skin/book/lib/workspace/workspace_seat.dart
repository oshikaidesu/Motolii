// workspace_seat.dart: the inspector's seat in the window: what it listens to, and when it builds again or only tells the fields their numbers.
part of 'workspace.dart';

/// The Inspector of the lab, fed by the workspace's selection; its panel links open the dock's shelves or the desk.
class _InspectorSeat extends StatefulWidget {
  const _InspectorSeat();
  @override
  State<_InspectorSeat> createState() => _InspectorSeatState();
}

class _InspectorSeatState extends State<_InspectorSeat> {
  // (the inspector follows only what it shows: the document, what is picked, Animate, and the moment only while what is
  //  picked has keys; played, a thing with no keys shows the same numbers, so it is not built again each picture)
  Ws? _ws;
  Object? _seen;
  (int, bool)? _stageKeyed;
  bool _keyed(Ws ws) {
    final l = ws.layer;
    if (l != null) return l.keys.isNotEmpty;
    final m = ws.inside;
    if (m == null) return true;
    if (_stageKeyed?.$1 != ws.rev) _stageKeyed = (ws.rev, m.moving(0).any((n) => m.keyMoments(0, n).isNotEmpty)); // (the stage's: once a document)
    return _stageKeyed!.$2;
  }

  // (the moment alone moved: the values and key marks told to the inspector as it stands; anything else: built again)
  final _live = ValueNotifier<(Map<String, double>, Map<String, KeyS>)>((const {}, const {}));
  int? _frame;
  Object _now(Ws ws) => (ws.rev, ws.selected, ws.animating);

  // (the reading of the document changed only numbers of what is picked, none that changes its cards: they are told
  //  to the fields that show them, as a moment's are; anything else, or not known: the inspector built again)
  bool _onlyNumbers(Ws ws) {
    final m = ws.inside, c = ws.lastChange, t = int.tryParse(ws.selected ?? '') ?? (ws.selected == null ? 0 : null);
    if (m == null || c == null || c.things || t == null || const bool.fromEnvironment('NOPOUR')) return false;
    for (final (thing, name) in c.values) {
      if (name == 'name' || (thing == t && (m.shaping.contains(name) || m.range(t, name) == null || m.shown(name)?.look == 'thing'))) return false; // (another thing: shown by its name, a word) // (a name changed anywhere: things are named on cards; a word: it is not watched)
    }
    return true;
  }
  static const _hears = [WsAspect.document, WsAspect.picked, WsAspect.animating];   // (what the inspector shows: the picked thing's cards, Animate all)
  void _changed() => Timeline.timeSync('inspector at a moment', _changedNow); // (named in Flutter's timeline: frames_test TRACE)
  void _changedNow() {
    final ws = _ws!, now = _now(ws), was = _seen;
    if (now != _seen) {
      _seen = now;
      _frame = ws.frame;
      if (was is (int, String?, bool) && was.$2 == ws.selected && was.$3 == ws.animating && _onlyNumbers(ws)) {
        _live.value = (Ws.timed('transform', () => wsTransform(ws, ws.layer?.id)), Ws.timed('key marks', () => _keyMarks(ws, ws.layer?.id)));
        return;
      }
      if (mounted) setState(() {});
    } else if (ws.frame != _frame) {
      _frame = ws.frame;
      if (!_keyed(ws)) return;
      _live.value = (Ws.timed('transform', () => wsTransform(ws, ws.layer?.id)), Ws.timed('key marks', () => _keyMarks(ws, ws.layer?.id)));
      _settle?.cancel();
      _settle = Timer(const Duration(milliseconds: 150), () => mounted ? setState(() {}) : null); // (the moment at rest: built once, whatever does not watch its own number shows it too)
    }
  }

  Timer? _settle;

  // (the inside showed something new while the document and the pick stayed (a hand carrying on the stage, a relation
  //  at a moment): the numbers told to the fields that show them; nothing built again)
  void _seenNow() {
    final ws = _ws!;
    if (_now(ws) != _seen || ws.selected == null || ws.playing) return; // (played: the moment's own reading above)
    _live.value = (Ws.timed('transform', () => wsTransform(ws, ws.layer?.id)), Ws.timed('key marks', () => _keyMarks(ws, ws.layer?.id)));
  }

  // (the panel itself, made once a size: a new one each build would build its whole tree again, whatever changed)
  (Size, InspectorPanel)? _panel;
  InspectorPanel _panelOf(BoxConstraints box) {
    final size = Size(box.maxWidth, box.maxHeight);
    if (_panel?.$1 != size) _panel = (size, InspectorPanel(width: box.maxWidth, height: box.maxHeight));
    return _panel!.$2;
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final ws = WsScope.read(context);
    if (ws != _ws) {
      for (final a in _hears) {
        _ws?.on[a]!.removeListener(_changed);
        ws.on[a]!.addListener(_changed);
      }
      _ws?.moment.removeListener(_changed);
      _ws?.seen.removeListener(_seenNow);
      _ws = ws;
      ws.moment.addListener(_changed);   // (the playhead too: the inspector's numbers are at the moment)
      ws.seen.addListener(_seenNow);
      _seen = _now(ws);
      _frame = ws.frame;
    }
  }

  @override
  void dispose() {
    for (final a in _hears) {
      _ws?.on[a]!.removeListener(_changed);
    }
    _ws?.moment.removeListener(_changed);
    _ws?.seen.removeListener(_seenNow);
    _settle?.cancel();
    _live.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ws = WsScope.read(context), l = ws.layer, k = _kindOf(l), m = ws.inside!;
    return Column(children: [const WsRelationStack(), Expanded(child: LayoutBuilder(
      builder: (context, box) => InspHost(
        key: ValueKey(k),
        cfg: const Cfg(),
        kind: k,
        fx: const [], // (an effect is one of the inside's cards, shown while it is on)
        kindWord: _kindWord(ws, l?.id),
        cards: Ws.timed('value cards', () => _valueCards(ws, l?.id)),
        seats: Ws.timed('seats', () => int.tryParse(l?.id ?? '') == null ? null : _seatsOf(m.values(int.parse(l!.id)).toSet())), // (a card of values the thing has not is not shown)
        title: l?.name,
        onRoute: ws.route,
        onPath: int.tryParse(l?.id ?? '') == null
            ? null
            : (to, n, copy) => m.path(int.parse(l!.id), copy: copy, to: to, n: n).then((p) => p?.indexed.map((e) => e.$1 % 4 < 2 ? e.$2 * m.point : e.$2).toList()), // (where it is in points of the picture, as the card's numbers are) // (the wave card's preview asks once, then plays by itself)
        onAim: (at) => ws.aim = at.startsWith('v_') ? at.substring(2) : null, // (a colour link of a card is aimed at that value)
        movable: int.tryParse(l?.id ?? '') == null ? const [] : m.fxTitles(int.parse(l!.id)), // (its effects, up or down their stack)
        onCardMove: int.tryParse(l?.id ?? '') == null ? null : (title, by) => ws.after(m.later('fx_move', thing: int.parse(l!.id), word: title, number: by.toDouble())),
        values: Ws.timed('transform', () => wsTransform(ws, l?.id)),
        strs: Ws.timed('words', () => _words(ws, l?.id)),
        choices: Ws.timed('choices', () => _choices(ws, l?.id)),
        onWords: (changed) => _say(ws, l?.id, changed),
        onAnimateAll: (on) => ws.animating = on,
        animating: ws.animating,
        flags: Ws.timed('flags', () => _flags(ws, l?.id)),
        onFlags: (changed) => _flag(ws, l?.id, changed),
        keyMarks: Ws.timed('key marks', () => _keyMarks(ws, l?.id)),
        onValues: (changed, {required live}) => _write(ws, l?.id, changed, live: live),
        onDrop: () => ws.after(m.later('let_go')),   // (put down unchanged, without waiting)
        onKey: (prop, mark) => _mark(ws, l?.id, prop, mark),
        onMidi: (prop, file, kind, channel, bpm) => _midi(ws, l?.id, prop, file, kind, channel, bpm),
        midiBpm: m.midiBpm,
        together: (tells) { // (one step, asked without waiting: begun and ended in the inside's queue around what is told)
          m.later('together', number: 1);
          tells();
          ws.after(m.later('together'));
        },
        live: _live,
        child: _panelOf(box), // (the same panel while the pane keeps its size: built again only by what it reads, Ctx)
      ),
    ))]);
  }
}
