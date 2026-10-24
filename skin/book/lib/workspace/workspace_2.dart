// Part of workspace.dart (cut there by size, as it was).
part of 'workspace.dart';

/// Joined: the cards the inside shows a thing's values on beyond the sample's (motolii_cards), each as Params in the
/// effects' manner: a number its range and what it is now, a word the words it may be. Nothing of them is typed here.
List<Fx>? _valueCards(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id); // (nothing picked: the stage's own, thing 0)
  if (m == null || t == null) return null;
  // (made again only when the cards it has, their values or the things it may name change: a number let go of changes
  //  none of them, and the numbers the cards show are read from wsTransform; what a card says a number is now, its
  //  @default, the inspector takes only when it makes its cards anew)
  final shape = '${m.cards(t)}|${ws.layers.map((l) => l.name).join(',')}';
  final made = _cardsMade[t];
  if (made != null && made.$1 == shape && identical(made.$3, m)) return made.$2;
  final cards = _makeCards(ws, m, t);
  _cardsMade[t] = (shape, cards, m);
  return cards;
}
final _cardsMade = <int, (String, List<Fx>, Motolii)>{};
int? _holding; // (the thing a hand on the inspector holds now: held once, carried at every step, let go when it lets go)

List<Fx> _makeCards(Ws ws, Motolii m, int t) {
  final at = _drawn(ws);   // (as the picture shown has it)
  String field(String n) {
    final sh = m.shown(n), label = '@label(${sh?.label ?? n.replaceAll('_', ' ').replaceRange(0, 1, n[0].toUpperCase())})';
    if (sh?.look == 'thing') {
      // (another thing, by the names the timeline shows: None, then the others; its number is the document's)
      final others = _others(ws, t), at = others.values.toList().indexOf((m.value(t, n, 0) ?? 0).round());
      return '  /// $label @enum(${['None', ...others.keys.map((o) => o.replaceAll(',', ' '))].join(', ')}) @default(${at + 1})\n  $n: u32,';
    }
    if (m.words(t, n) case final words when words.isNotEmpty) {
      return '  /// $label @enum(${words.join(', ')}) @default(${math.max(0, words.indexOf(m.word(t, n)))})\n  $n: u32,';
    }
    if (m.range(t, n) case (final lo0, final hi0)) {
      final a = _shown(m, t, n, sh != null && sh.high > sh.low ? sh.low : lo0), b = _shown(m, t, n, sh != null && sh.high > sh.low ? sh.high : hi0), lo = math.min(a, b), hi = math.max(a, b);
      final ui = switch (sh?.look) {
        'onoff' => '@toggle',
        'angle' => '@ui(angle)',
        'seed' => '@ui(seed)',
        'count' => hi - lo <= 12 ? '@ui(stepper)' : '@ui(bar)',
        'share' when lo >= 0 && hi <= 100 => '@ui(percent)',
        _ => lo < 0 && hi > 0 ? '@ui(bipolar)' : '@ui(bar)',
      };
      final whole = sh?.look == 'count' || sh?.look == 'onoff' || sh?.look == 'seed';
      return '  /// $label $ui @range($lo, $hi) @unit(${sh?.unit ?? ''}) @default(${_shown(m, t, n, m.value(t, n, at) ?? lo0)})\n  $n: ${whole ? 'u32' : 'f32'},';
    }
    return '  /// $label${const ['numbers', 'colours', 'words', 'file'].contains(sh?.look) ? ' @unit(${sh!.look})' : ''}\n  $n: vec3<f32>,'; // (any word: colours, six digits each, given from the Colors shelf; or numbers, one for each copy)
  }

  return [
    for (final (title, names) in m.cards(t))
      Fx('v', title, 'struct Params {\n${[
        for (final n in names) if (m.shown(n)?.fold == null) field(n),
        for (final fold in {for (final n in names) ?m.shown(n)?.fold}) ...[
          '  // @advanced($fold)',
          for (final n in names) if (m.shown(n)?.fold == fold) field(n),
        ],
      ].join('\n')}\n}', toneOf: () => Pop.toneLayer, mark: CardMark.layer),
  ];
}

/// Joined: how many of a knob's units make one of the document's (the inside's shown look: a length of the stage in
/// points of the picture, a share in hundredths; else as it is).
/// A value of the inside's own cards as the inspector counts it: another thing by its place (None first), a number
/// in the card's units, a word by its place among those it may be (the stage's and a thing's alike).
double _onCard(Ws ws, Motolii m, int t, String n, double at) {
  final seen = const {'x', 'y', 'z', 'rotate', 'scale_x', 'scale_y', 'opacity'}.contains(n) ? m.shows(t, n, at) : null;   // (the values the moment's copy carries as the stage shows them: picture.h)
  if (seen != null) return _shown(m, t, n, seen.$1);   // (where it shows: a hand carrying it, a relation moving it, as After Effects shows a value an expression drives)
  return m.shown(n)?.look == 'thing' ? (_others(ws, t).values.toList().indexOf((m.value(t, n, 0) ?? 0).round()) + 1).toDouble() : m.words(t, n).isEmpty ? _shown(m, t, n, m.value(t, n, at) ?? 0) : math.max(0, m.words(t, n).indexOf(m.word(t, n))).toDouble(); // (the inside's own cards: a number as it is, a word by its place among those it may be)
}

/// A value as a hand reads it, and back (Motolii.reading).
double _shown(Motolii m, int t, String n, double v) { final (k, f) = m.reading(t, n); return v * k + f; }
double _unshown(Motolii m, int t, String n, double x) { final (k, f) = m.reading(t, n); return (x - f) / k; }

// (the moment of the picture on the stage, at a frame: what is read there was set down by the inside as it drew it,
//  not waited for while it tells the stage a moment the playhead has just been put at)
double _drawn(Ws ws) => ws.inside?.drawnAt ?? ws.frame / Ws.fps;

/// Joined: only where a thing is, how large and how turned (the inspector's seats), as the stage's readout shows them:
/// not the cards' hundreds of numbers [wsTransform] reads.
Map<String, double> wsPlace(Ws ws, String id) {
  final m = ws.inside, t = int.tryParse(id);
  if (m == null || t == null) return const {};
  final at = _drawn(ws);
  double r1(double x) => (x * 10).roundToDouble() / 10 + 0;
  return {for (final n in const ['x', 'y', 'scale_y', 'rotate']) n: r1(_shown(m, t, n, (n == 'x' || n == 'y' ? m.shows(t, n, at)?.$1 : null) ?? m.value(t, n, at) ?? (n == 'scale_y' ? 1 : 0)))};   // (where it shows)
}

/// The selected layer's transform as the stage draws it at this frame, in the Inspector's ids.
Map<String, double> wsTransform(Ws ws, String? id) {
  if (ws.inside case final m?) {   // (the document's own values, as the inspector counts them)
    final t = int.tryParse(id ?? '');
    if (id == null) { // (nothing picked: the stage's own cards, thing 0)
      final at = _drawn(ws);
      return {for (final (_, names) in m.cards(0)) for (final n in names) 'v_$n': _onCard(ws, m, 0, n, at)};
    }
    if (t == null) return const {};
    final at = _drawn(ws);
    double r1(double x) => (x * 10).roundToDouble() / 10 + 0; // (+ 0: no minus nought)
    return {
      'depth': r1(m.frame(t)?[7] ?? 0), // (how deep it is in points of the picture: a flat thing is Flat)
      for (final (_, names) in m.cards(t))
        for (final n in names)
          'v_$n': _onCard(ws, m, t, n, at),
    };
  }
  return const {};
}

(String, double)? _asDocument(Ws ws, String id, double x) {
  if (id.startsWith('v_') && ws.inside!.shown(id.substring(2))?.look == 'thing') { // (another thing: None, then the others by their place)
    final others = _others(ws, int.tryParse(ws.selected ?? '') ?? 0).values.toList(), i = x.round();
    return (id.substring(2), i >= 1 && i <= others.length ? others[i - 1].toDouble() : 0);
  }
  final m = ws.inside!, t = int.tryParse(ws.selected ?? '') ?? 0, n = id.substring(2);
  final seen = id.startsWith('v_') && (n == 'x' || n == 'y') ? m.shows(t, n, _drawn(ws)) : null;
  if (seen != null && seen.$2) return (n, _unshown(m, t, n, x) - (seen.$1 - (m.value(t, n, _drawn(ws)) ?? seen.$1)));   // (shown where it shows: its own value moved as far as the hand moved that)
  if (id.startsWith('v_')) return (n, _unshown(m, t, n, x)); // (a card of the inside's own: the value of that name, in the document's own units)
  return null;
}

/// Joined: what was changed in the inspector is a thing held and its numbers carried (the inside's one form of a
/// drag): while a hand is on it, it only shows; when the hand lets go it is the document's, by the document's rules.
void _write(Ws ws, String? id, Map<String, double> changed, {required bool live}) {
  final m = ws.inside!, t = id == null ? 0 : int.tryParse(id); // (nothing picked: the stage's own, thing 0)
  // (a word on a card of the inside's own, picked: the word in that place, said; one step)
  final said = {for (final e in changed.entries) if (t != null && e.key.startsWith('v_') && m.words(t, e.key.substring(2)).isNotEmpty) e.key: e.value};
  if (said.isNotEmpty) {
    if (live) return;
    m.later('together', number: 1);   // (one step, without waiting)
    for (final e in said.entries) {
      final ways = m.words(t!, e.key.substring(2));
      m.later('say', thing: t, name: e.key.substring(2), word: ways[e.value.round().clamp(0, ways.length - 1)]);
    }
    ws.after(m.later('together'), (done, why) => _told(ws, done ? null : why));
    changed = {for (final e in changed.entries) if (!said.containsKey(e.key)) e.key: e.value};
    if (changed.isEmpty) return;
  }
  if (t == null) return;
  if (_holding != t) m.take(t); // (held once a hand: each step after only carries)
  _holding = live ? t : null;
  String? why;
  for (final e in changed.entries) {
    final to = _asDocument(ws, e.key, e.value);
    if (to != null && !m.carry(to.$1, to.$2)) why ??= m.trouble;
    if (to == null) why ??= 'the document has no ${e.key}: '; // (a seat the document has no value for says so, rather than keep the number)
  }
  if (live) {
    if (ws.playing) ws.playing = false; // (a hand on a value stops the play: what is changed is changed at one moment)
    if (why != null) _told(ws, why);
  } else {
    ws.after(m.later('let_go', number: 1), (done, no) => _told(ws, why ?? (done ? null : no))); // (the hand let go: the window does not wait for the step)
  }
}

/// Joined: a property's key mark as the document has it at the playhead (the most keyed of its values).
/// Joined: the key marks the inspector shows (for the checks: the same the inspector is given).
Map<String, KeyS> wsKeyMarks(Ws ws, String? id) => _keyMarks(ws, id);

Map<String, KeyS> _keyMarks(Ws ws, String? id) {
  final m = ws.inside, t = id == null ? 0 : int.tryParse(id);
  if (m == null || t == null) return const {};
  final at = _drawn(ws);
  final marks = {for (final MapEntry(:key, :value) in _keyable(ws, t).entries) key: KeyS.values[m.keyed(t, value, at)]};
  final props = <String, KeyS>{};   // (a property's mark is the most of its values': a key on any, keyed)
  for (final MapEntry(:key, :value) in marks.entries) {
    final p = propOf(key);
    if (p != key && (props[p] == null || value.index > props[p]!.index)) props[p] = value;
  }
  return {...marks, ...props};
}

/// Joined: the fields of the cards whose values take keys, by the inspector's id and the document's name.
Map<String, String> _keyable(Ws ws, int t) {
  final m = ws.inside!, moving = m.moving(t).toSet();
  return {
    for (final (_, names) in m.cards(t))
      for (final n in names)
        if (moving.contains(n)) 'v_$n': n,
  };
}

/// Joined: a key mark changed in the inspector: a key at the playhead, that key taken away, or no keys at all.
void _mark(Ws ws, String? id, String prop, KeyS mark) {
  final m = ws.inside!, t = id == null ? 0 : int.tryParse(id);
  if (t == null) return;
  final at = ws.frame / Ws.fps, has = m.values(t).toSet();
  // (only the values it has: a flat thing has no scale_z, and a key asked of it was refused and took the whole step,
  //  the number let go of with it, back: with Animate on, a flat thing's first scale was lost)
  m.later('together', number: 1);   // (one step, without waiting)
  for (final n in [for (final MapEntry(:key, :value) in _keyable(ws, t).entries) if (key == prop || propOf(key) == prop) value].where(has.contains)) {
    m.later(switch (mark) { KeyS.at => 'key_here', KeyS.anim => 'unkey', KeyS.off => 'stay' }, thing: t, name: n, number: at);
  }
  ws.after(m.later('together'));
}

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
    if (m == null || c == null || c.things || c.values.isEmpty || t == null || const bool.fromEnvironment('NOPOUR')) return false;   // (nothing said changed: not known)
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
