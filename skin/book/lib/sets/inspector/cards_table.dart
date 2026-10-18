// Cards handled by hand: the window's inspector (cards of value tiles, as today) with the cards free to pick up.
// One table: the stage (a star ring) on the left, the inspector's cards on the right, the shelf and the hand (Desk's place) under them.
// What the hand does (each a use case in cards_set.dart):
//   a behaviour card laid on a value moves it copy by copy (shown while it only hovers; the same card again = wider; another = swapped);
//   its mark tapped opens the behaviour's own knobs, carried to another value it moves there, dropped on the hand it lets go;
//   a card flipped shows every number (each takes a behaviour too); cards reordered (the order is what is done first: the wave card
//   says "each" or "whole"); a card turned face down (off); a shelf card thrown on the stage is worn, its corner pulled back to the
//   shelf takes it off; a tile held and dropped on another ties them into one bundle.
// A mock: not joined to the inside. Sizes and colours are the inspector's own (Pop).
import 'dart:async' show Timer;
import 'dart:math' as math;

import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart' show LogicalKeyboardKey, MouseCursor, SystemMouseCursors;
import 'package:flutter/widgets.dart';

import '../../tokens.dart';
import '../../parts/controls.dart' show Segmented;
import 'inspector_parts.dart' show Pop;

// ---------------------------------------------------------------- the model

class Val {
  Val(this.id, this.label, this.v, this.lo, this.hi, {this.unit = '', this.dec = 0, this.perCopy = true, this.wrap = false});
  final String id, label, unit;
  double v;
  final double lo, hi;
  final int dec;
  final bool perCopy; // false: one value for the whole ring (a count): no behaviour moves it
  final bool wrap; // an angle: past its end it comes round again
  double fit(double x) => wrap ? lo + (x - lo) % (hi - lo) : x.clamp(lo, hi);
}

class CardM {
  CardM(this.name, this.tone, this.front, {this.back = const []});
  final String name;
  final Color tone;
  final List<Val> front, back; // front: its main tiles; back: the rest of today's values, as rows
  bool flipped = false, down = false;
  List<Val> get all => [...front, ...back];
}

/// A behaviour card (Cavalry's behaviours): what it gives a value, copy by copy and moment by moment, in -1..1.
enum Beh {
  sway('揺れ', Color(0xFF7DD5B1)),
  stagger('ずれ', Color(0xFF4781E5)),
  noise('ノイズ', Color(0xFFE974AB)),
  random('ランダム', Color(0xFFEFCB4E));

  const Beh(this.word, this.c);
  final String word;
  final Color c;

  /// [from]: where a stagger starts (0 the first copy, 1 the middle, 2 the last); [seed]: which roll a noise or a random is.
  double at(double t, int i, int n, {double seed = 0, int from = 0}) {
    final f = n <= 1 ? 0.0 : i / (n - 1);
    return switch (this) {
      Beh.sway => math.sin(2 * math.pi * (t * .55 + i / math.max(n, 1))),
      Beh.stagger => switch (from) { 1 => (f - .5).abs() * 4 - 1, 2 => 1 - f * 2, _ => f * 2 - 1 },
      Beh.noise => _noise(t * .7 + i * 3.17 + seed * 13.1),
      Beh.random => _hash(i * 7.0 + 1 + seed * 17.3) * 2 - 1,
    };
  }
}

double _hash(double x) {
  final s = math.sin(x * 12.9898) * 43758.5453;
  return s - s.floorToDouble();
}

double _noise(double x) {
  final i = x.floorToDouble(), f = x - i, u = f * f * (3 - 2 * f);
  return (_hash(i) * (1 - u) + _hash(i + 1) * u) * 2 - 1;
}

/// A behaviour on a value, with its own two knobs (how wide: a share of the value's range; how fast) and its own clock.
class Drive {
  Drive(this.beh);
  final Beh beh;
  double width = .2, speed = 1, clock = 0, seed = 0;
  int from = 0; // a stagger's start: 0 the first, 1 the middle, 2 the last
  double at(int i, int n) => beh.at(clock, i, n, seed: seed, from: from);
  Drive copy() => Drive(beh)
    ..width = width
    ..speed = speed
    ..clock = clock
    ..seed = seed
    ..from = from;
}

/// What the table is, kept before each change so the change can be taken back.
class _Snap {
  _Snap(CardTable t)
    : values = {for (final c in [...t.cards, ...t.shelf]) for (final v in c.all) v: v.v},
      drives = {for (final e in t.drives.entries) e.key: e.value.copy()},
      bundles = [for (final b in t.bundles) [...b]],
      cards = [...t.cards],
      shelf = [...t.shelf],
      faces = {for (final c in [...t.cards, ...t.shelf]) c: (c.flipped, c.down)};
  /// Nothing to take back between these two: an undo step that would change nothing is skipped.
  bool same(_Snap o) =>
      values.length == o.values.length && values.entries.every((e) => o.values[e.key] == e.value) &&
      drives.length == o.drives.length && drives.entries.every((e) { final d = o.drives[e.key]; return d != null && d.beh == e.value.beh && d.width == e.value.width && d.speed == e.value.speed && d.seed == e.value.seed && d.from == e.value.from; }) &&
      bundles.length == o.bundles.length && [for (var i = 0; i < bundles.length; i++) i].every((i) => bundles[i].length == o.bundles[i].length && bundles[i].every(o.bundles[i].contains)) &&
      cards.length == o.cards.length && [for (var i = 0; i < cards.length; i++) i].every((i) => cards[i] == o.cards[i]) &&
      shelf.length == o.shelf.length && shelf.every(o.shelf.contains) &&
      faces.entries.every((e) => o.faces[e.key] == e.value);
  final Map<Val, double> values;
  final Map<String, Drive> drives;
  final List<List<Val>> bundles;
  final List<CardM> cards, shelf;
  final Map<CardM, (bool, bool)> faces;
}

class CardTable extends ChangeNotifier {
  CardTable(this.cards, {this.hand = const [Beh.sway, Beh.stagger, Beh.noise, Beh.random], List<CardM>? shelf}) : shelf = shelf ?? [];
  final List<CardM> cards;
  final List<Beh> hand;
  final List<CardM> shelf; // cards the thing does not wear yet: thrown on the stage, they join it
  final drives = <String, Drive>{};
  final bundles = <List<Val>>[]; // values tied into one: scrubbing the bundle moves them all
  (String, Beh)? preview; // a behaviour hovering over a value: shown before it is let go
  CardM? cardPreview; // a shelf card held over the stage: worn before it is let go
  String? flash, open; // the value a card was just laid on (lit once); the value whose behaviour's knobs are open
  int flashN = 0, orderN = 0; // orderN: bumped when the cards are reordered (they light in the order they are done)
  Object? carrying; // what the hand holds now: the places it may go are outlined
  final ringOpen = ValueNotifier<bool>(false); // the stage's value ring is open: the card in the air shrinks to its mark, the ring stays readable
  double clock = 0, waveClock = 0; // seconds; the wave's own, run at its speed (a speed changed never jumps the wave)
  String? editing; // the number being typed: a value's id, or w:/s: + id for a behaviour's width/speed
  String? touched; // the value last scrubbed or typed: a hand card tapped (not carried) goes there
  (CardM, bool)? insertMark; // a card carried over another: the line above (false) or under (true) it
  final _undo = <_Snap>[], _redo = <_Snap>[];
  final refusing = ValueNotifier<bool>(false); // the hand is over a place that does not take what it holds: the card in the air says so
  Map<Val, double> _bundleStart = {};
  Val? bundleStop; // the value at its end that holds the whole bundle still
  double _bundleShare = 0;

  /// Kept before a change (a drag keeps once, at its start): ⌘Z / the back arrow takes it back.
  bool _redoAtRisk = false; // a mark was made: the first real change after it clears what could be redone
  void mark() {
    _redoAtRisk = _redo.isNotEmpty;
    _undo.add(_Snap(this));
    if (_undo.length > 200) _undo.removeAt(0);
  }

  bool get canUndo => _undo.isNotEmpty;
  bool scrubbing = false; // a value or a bundle being scrubbed now: nothing is taken back under the hand
  void undo() {
    if (carrying != null || scrubbing) return; // (a hand holding something: nothing is taken back under it)
    if (editing != null) return type(null); // (typing: back is leaving the number as it was)
    final now = _Snap(this);
    while (_undo.isNotEmpty && _undo.last.same(now)) {
      _undo.removeLast(); // (a step that changed nothing: not one to spend a ⌘Z on)
    }
    if (_undo.isEmpty) return;
    _redo.add(now);
    _restore(_undo.removeLast());
  }

  bool get canRedo => _redo.isNotEmpty;
  void redo() {
    if (carrying != null || scrubbing || editing != null || _redo.isEmpty) return;
    _undo.add(_Snap(this));
    _restore(_redo.removeLast());
  }

  void _restore(_Snap s) {
    bundleStop = null;
    s.values.forEach((v, x) => v.v = x);
    for (final e in s.drives.entries) {
      final now = drives[e.key];
      e.value.clock = now != null && now.beh == e.value.beh ? now.clock : clock; // (what moves keeps moving: no jump back)
    }
    drives
      ..clear()
      ..addAll(s.drives);
    preview = null;
    cardPreview = null;
    bundles
      ..clear()
      ..addAll(s.bundles);
    cards
      ..clear()
      ..addAll(s.cards);
    shelf
      ..clear()
      ..addAll(s.shelf);
    s.faces.forEach((c, f) {
      c.flipped = f.$1;
      c.down = f.$2;
    });
    if (open != null && drives[open] == null) open = null;
    editing = null;
    _changed();
  }

  void type(String? id) {
    editing = id;
    if (id != null && valOr(id) != null) touched = id;
    _changed();
  }

  /// The clocks run on: each behaviour at its own speed, the wave at its card's.
  void advance(double dt) {
    clock += dt;
    for (final d in drives.values) {
      d.clock += dt * d.speed;
    }
    final s = valOr('waveS');
    waveClock += dt * (s?.v ?? 1);
  }

  List<CardM> get worn => [...cards, ?cardPreview];
  Val? valOr(String id) {
    for (final c in worn) {
      for (final v in c.all) {
        if (v.id == id) return v;
      }
    }
    return null;
  }

  CardM? cardOf(String id) {
    for (final c in worn) {
      if (c.all.any((v) => v.id == id)) return c;
    }
    return null;
  }

  int? bundleOf(Val v) {
    final i = bundles.indexWhere((b) => b.contains(v));
    return i < 0 ? null : i;
  }

  static bool canDrive(Val v) => v.perCopy;

  /// The value at this moment for one copy: its number, moved by what drives it; a hovering card shows what letting go would do.
  /// The fallback if its card is not worn or is face down.
  double at(String id, int i, int n, {double fallback = 0}) {
    final v = valOr(id), card = cardOf(id);
    if (v == null || card == null || card.down) return fallback;
    final d = drives[id], p = preview, span = v.hi - v.lo;
    var x = v.v;
    if (p != null && p.$1 == id) {
      x += d != null && d.beh == p.$2 ? d.at(i, n) * span * math.min(d.width + .15, 1) : p.$2.at(clock, i, n) * span * .2;
    } else if (d != null) {
      x += d.at(i, n) * span * d.width;
    }
    return v.fit(x);
  }

  void _changed({bool keepFlash = false}) {
    if (!keepFlash) flash = null;
    if (_redoAtRisk && _undo.isNotEmpty && !_undo.last.same(_Snap(this))) {
      _redo.clear();
      _redoAtRisk = false;
    }
    notifyListeners();
  }

  void set(Val v, double x) {
    v.v = v.fit(x);
    touched = v.id;
    _changed();
  }

  /// A hand card tapped: it goes on the value last touched (After Effects' "pick, then apply").
  int nopeN = 0; // a hand card tapped with nowhere to go: it shakes (nopeN counts, nope says which)
  Beh? nope;
  void tapLay(Beh b) {
    final v = touched == null ? null : valOr(touched!);
    if (v != null && canDrive(v) && cards.contains(cardOf(v.id)) && !cardOf(v.id)!.down) return lay(v.id, b);
    nope = b;
    nopeN++;
    notifyListeners();
  }

  void markInsert((CardM, bool)? m) {
    if (m == insertMark) return;
    insertMark = m;
    notifyListeners();
  }

  /// A card carried by its head and let go above or under another: it moves there. The cards light in order only when
  /// that changes what is done first (the copies and the wave).
  void moveCard(CardM c, CardM onto, bool under) {
    insertMark = null;
    final at = cards.indexOf(c), to = cards.indexOf(onto) + (under ? 1 : 0);
    if (c == onto || (at >= 0 && (to == at || to == at + 1))) return _changed(); // (let go where it already is)
    mark();
    final first = copiesFirst;
    cards.remove(c);
    shelf.remove(c); // (a shelf card goes in where it is let go)
    cardPreview = null;
    cards.insert(cards.indexOf(onto) + (under ? 1 : 0), c);
    if (first != copiesFirst) orderN++;
    _changed();
  }

  void carry(Object? o) {
    carrying = o;
    refusing.value = false;
    if (o == null) {
      preview = null;
      cardPreview = null;
    }
    _changed(keepFlash: true);
  }

  void hover((String, Beh)? p) {
    if (p == preview) return;
    preview = p;
    _changed(keepFlash: true);
  }

  /// A behaviour card let go on a value: the same one again is wider, another takes its place.
  void lay(String id, Beh b) {
    mark();
    final d = drives[id];
    if (d != null && d.beh == b) {
      d.width = math.min(d.width + .15, 1);
    } else {
      drives[id] = Drive(b)
        ..width = d?.width ?? .2
        ..speed = d?.speed ?? 1 // (another behaviour takes the place, not the width)
        ..clock = d?.clock ?? clock; // (and runs on from the same moment as the hovering one did)
    }
    preview = null;
    flash = id;
    flashN++;
    touched = id;
    _changed(keepFlash: true);
  }

  /// A behaviour's mark carried from one value to another: it moves there, its knobs with it.
  void moveDrive(String from, String to) {
    if (from == to || drives[from] == null) return;
    mark();
    final d = drives.remove(from)!, other = drives[to];
    drives[to] = d;
    if (other != null) drives[from] = other; // (the one that was there goes where this came from)
    if (open == from) open = to;
    flash = to;
    flashN++;
    _changed(keepFlash: true);
  }

  void lift(String id) {
    mark();
    drives.remove(id);
    if (open == id) open = null;
    _changed();
  }

  void toggleOpen(String id) {
    open = open == id ? null : id;
    _changed();
  }

  void setDrive(Drive d, {double? width, double? speed, double? seed, int? from}) {
    if (width != null) d.width = width.clamp(0.0, 1.0);
    if (speed != null) d.speed = speed.clamp(.1, 4.0);
    if (seed != null) d.seed = seed.clamp(0.0, 99.0);
    if (from != null) d.from = from;
    _changed();
  }

  void flip(CardM c) {
    mark();
    c.flipped = !c.flipped;
    _changed();
  }

  void turnDown(CardM c) {
    mark();
    c.down = !c.down;
    _changed();
  }

  void holdCard(CardM? c) {
    if (c == cardPreview) return;
    cardPreview = c;
    _changed(keepFlash: true);
  }

  /// A shelf card let go on the stage (or the inspector): the thing wears it, last.
  void wear(CardM c) {
    mark();
    shelf.remove(c);
    if (!cards.contains(c)) cards.add(c);
    cardPreview = null;
    _changed();
  }

  /// A card put back on the shelf: the thing no longer wears it (what drove its values goes with it).
  void shelve(CardM c) {
    if (!cards.contains(c)) return;
    mark();
    cards.remove(c);
    for (final b in bundles) {
      b.removeWhere(c.all.contains); // (only its own values leave a bundle; what drives them stays with the card, back when it is worn)
    }
    bundles.removeWhere((b) => b.length < 2);
    shelf.add(c);
    _changed();
  }

  /// One value let go on another: they become one bundle (or the value joins the other's bundle).
  void tie(Val a, Val b) {
    if (a == b) return;
    final ia = bundleOf(a), ib = bundleOf(b);
    if (ia != null && ia == ib) return;
    mark();
    if (ia != null) bundles[ia].remove(a); // (it leaves its own bundle alone; the rest stays tied)
    final jb = bundleOf(b);
    jb != null ? bundles[jb].add(a) : bundles.add([b, a]);
    bundles.removeWhere((x) => x.length < 2);
    _changed();
  }

  void untie(int i) {
    mark();
    bundles.removeAt(i);
    _changed();
  }

  /// One value let out of its bundle.
  void untieOne(Val v) {
    final i = bundleOf(v);
    if (i == null) return;
    mark();
    bundles[i].remove(v);
    bundles.removeWhere((x) => x.length < 2);
    _changed();
  }

  /// A bundle scrubbed: each of its values moves the same share of its own range from where it was when the scrub began;
  /// when one reaches its end, the whole bundle stops (their spacing is never squeezed).
  void startBundle(int i) {
    mark();
    bundleStop = null;
    _bundleStart = {for (final v in bundles[i]) v: v.v};
    _bundleShare = 0;
  }

  void scrubBundle(int i, double share) {
    var lo = double.negativeInfinity, hi = double.infinity;
    Val? atLo, atHi;
    for (final v in bundles[i]) {
      if (v.wrap) continue;
      final s = _bundleStart[v] ?? v.v, span = v.hi - v.lo;
      if ((v.lo - s) / span > lo) (lo, atLo) = ((v.lo - s) / span, v);
      if ((v.hi - s) / span < hi) (hi, atHi) = ((v.hi - s) / span, v);
    }
    final want = _bundleShare + share;
    bundleStop = want < lo ? atLo : (want > hi ? atHi : null);
    _bundleShare = want.clamp(lo, hi);
    for (final v in bundles[i]) {
      v.v = v.fit((_bundleStart[v] ?? v.v) + _bundleShare * (v.hi - v.lo));
    }
    _changed();
  }

  /// The wave's tag pressed: the wave and the copies change places, so what is done first turns round.
  void swapOrder() {
    final a = cards.indexWhere((c) => c.name == '複製'), b = cards.indexWhere((c) => c.name == '波');
    if (a < 0 || b < 0) return;
    mark();
    final ca = cards[a];
    cards[a] = cards[b];
    cards[b] = ca;
    orderN++;
    _changed();
  }

  /// Which comes first, the copies or the wave: wave first = the ring waves as one; copies first = each copy waves on its own.
  bool get copiesFirst {
    final a = cards.indexWhere((c) => c.name == '複製'), b = cards.indexWhere((c) => c.name == '波');
    return a < 0 || b < 0 || a < b;
  }
}

Val _px(String id, String label, [double v = 0, double lo = -400, double hi = 400]) => Val(id, label, v, lo, hi, unit: 'px');
Val _deg(String id, String label, [double v = 0]) => Val(id, label, v, 0, 360, unit: '°', wrap: true);
Val _pc(String id, String label, [double v = 100]) => Val(id, label, v, 0, 300, unit: '%');

CardM waveCard() => CardM('波', Pop.tonePath, [
  Val('waveH', '高さ', 30, 0, 90, unit: 'px'),
  Val('waveS', '速さ', 1, 0, 4, dec: 2, perCopy: false),
], back: [_px('waveX', '横の波', 0, 0, 90), Val('waveRot', '波で回す', 0, 0, 90, unit: '°'), _pc('waveSize', '波で大きく', 0), Val('phase', '位相', 0, 0, 1, dec: 2)]);

CardM scatterCard() => CardM('散らす', Pop.toneParticles, [
  Val('scatter', '広さ', 50, 0, 120, unit: 'px'),
  Val('scatterTurn', '回して散らす', 0, 0, 180, unit: '°'),
], back: [_pc('scatterSize', '大きさで散らす', 0), Val('seed', 'シード', 0, 0, 99, perCopy: false)]);

/// The ring's cards as the window shows a shape with copies today (labels in plain Japanese).
CardTable starRing({bool wave = true}) => CardTable(shelf: [scatterCard(), if (!wave) waveCard()], [
  CardM('見た目', Pop.toneLayer, [
    Val('size', '大きさ', 44, 10, 120, unit: 'px'),
    _deg('turn', '回転'),
    Val('opacity', '不透明', 100, 0, 100, unit: '%'),
  ], back: [_px('posX', '位置 X'), _px('posY', '位置 Y'), _pc('scaleX', '拡大 X'), _pc('scaleY', '拡大 Y'), _deg('rotX', '回転 X'), _deg('rotY', '回転 Y')]),
  CardM('複製', Pop.tonePlace, [
    Val('count', '数', 12, 3, 48, perCopy: false),
    Val('radius', '半径', 180, 20, 300, unit: 'px'),
    Val('spin', '1つずつ回す', 30, 0, 180, unit: '°'),
  ], back: [_pc('copySize', '1つずつの大きさ'), _px('copyZ', '奥行き', 0, 0, 400), Val('delay', '遅れ', 0, 0, 2, unit: 's', dec: 1, perCopy: false)]),
  if (wave) waveCard(),
  CardM('色', Pop.toneFill, [
    _deg('hue', '色相', 72),
    Val('light', '明るさ', 62, 20, 90, unit: '%'),
  ], back: [Val('strokeW', '線の太さ', 0, 0, 12, unit: 'px')]),
]);

String sayVal(Val v, double x) => '${x.toStringAsFixed(v.dec)}${v.unit}';

// ---------------------------------------------------------------- the table

class CardsTable extends StatefulWidget {
  const CardsTable(this.t, {super.key});
  final CardTable t;
  @override
  State<CardsTable> createState() => _CardsTableState();
}

class _CardsTableState extends State<CardsTable> with SingleTickerProviderStateMixin {
  late final Ticker tk;
  final time = ValueNotifier<double>(0); // (only a beat for what is painted moment by moment: the clocks are the table's)
  Duration last = Duration.zero;

  @override
  void initState() {
    super.initState();
    tk = createTicker((d) {
      final dt = (d - last).inMicroseconds / 1e6;
      last = d;
      widget.t.advance(dt);
      time.value += dt;
    })..start();
  }

  @override
  void dispose() {
    tk.dispose();
    time.dispose();
    super.dispose();
  }

  // (cards in the air and the reordering need an overlay of their own: the table works in any host)
  late final _entry = OverlayEntry(builder: _table);
  @override
  Widget build(BuildContext context) => _Time(time: time, child: Overlay(initialEntries: [_entry]));

  Widget _table(BuildContext context) => CallbackShortcuts(
    bindings: {
      const SingleActivator(LogicalKeyboardKey.keyZ, meta: true): widget.t.undo,
      const SingleActivator(LogicalKeyboardKey.keyZ, meta: true, shift: true): widget.t.redo,
      const SingleActivator(LogicalKeyboardKey.keyZ, control: true): widget.t.undo,
    },
    child: Focus(autofocus: true, child: ListenableBuilder(
    listenable: widget.t,
    builder: (context, _) => ColoredBox(
      color: N.g07,
      child: Row(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Expanded(child: _Stage(widget.t)),
        Container(
          width: 360,
          decoration: BoxDecoration(color: Grey.g10, border: Border(left: BorderSide(color: Grey.g20))),
          child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
            Expanded(child: _Inspector(widget.t)),
            _Hand(widget.t),
          ]),
        ),
      ]),
    ),
  )),
  );
}

class _Time extends InheritedWidget {
  const _Time({required this.time, required super.child});
  final ValueNotifier<double> time;
  static ValueNotifier<double> of(BuildContext c) => c.getInheritedWidgetOfExactType<_Time>()!.time;
  @override
  bool updateShouldNotify(_Time old) => false;
}

// ---------------------------------------------------------------- the stage

/// The stage takes a shelf card (worn) and a behaviour card: where a behaviour comes in, a ring of the values most often moved
/// opens; the way the hand goes from there picks the value (shown before it is let go), the middle lets go of nothing.
class _Stage extends StatefulWidget {
  const _Stage(this.t);
  final CardTable t;
  @override
  State<_Stage> createState() => _StageState();
}

const _ringVals = ['size', 'radius', 'spin', 'hue']; // up, right, down, left

class _StageState extends State<_Stage> {
  Offset? anchor, at, restAt; // the ring's place (it opens where the hand rests, pushed inside the stage), the hand's, where it rested
  int? aim;
  Timer? rest;

  @override
  void dispose() {
    rest?.cancel();
    super.dispose();
  }

  /// The hand moved: with no ring open, the ring waits for it to rest (0.25 s); far from an open ring, the ring closes and waits again.
  Offset? restFrom;
  void _moved(Offset p, Beh b) {
    at = p;
    if (anchor == null && restFrom != null && (p - restFrom!).distance < 6) return; // (a hand trembling is a hand resting)
    restFrom = p;
    if (anchor != null && (p - (restAt ?? anchor!)).distance > 110) {
      setState(() => anchor = aim = null);
      t.hover(null);
    }
    if (anchor == null) {
      rest?.cancel();
      rest = Timer(const Duration(milliseconds: 250), () {
        final box = context.findRenderObject() as RenderBox?;
        if (mounted && at != null && box != null) {
          restAt = at;
          setState(() => anchor = Offset(at!.dx.clamp(118, math.max(118, box.size.width - 118)), at!.dy.clamp(88, math.max(88, box.size.height - 88)))); // (the whole ring inside the stage)
        }
      });
      return;
    }
    final a = _aimAt(p);
    if (a != aim) setState(() => aim = a);
    t.hover(a == null ? null : (_ringVals[a], b));
  }
  CardTable get t => widget.t;

  Offset _local(Offset global) => (context.findRenderObject()! as RenderBox).globalToLocal(global);
  int? _aimAt(Offset p) {
    final d = p - (restAt ?? anchor!); // (the way the hand went from where it rested: a ring pushed in from the edge picks nothing by itself)
    if (d.distance < 28) return null;
    final k = d.dx.abs() > d.dy.abs() ? (d.dx > 0 ? 1 : 3) : (d.dy < 0 ? 0 : 2);
    return t.valOr(_ringVals[k]) == null ? null : k;
  }

  void _leave() {
    rest?.cancel();
    restFrom = null;
    setState(() => anchor = aim = at = null);
    t.hover(null);
    t.holdCard(null);
  }

  @override
  @override
  void setState(VoidCallback fn) {
    super.setState(fn);
    t.ringOpen.value = anchor != null;
  }

  @override
  Widget build(BuildContext context) => DragTarget<Object>(
    onWillAcceptWithDetails: (e) {
      if (e.data is CardM && !t.cards.contains(e.data)) {
        t.holdCard(e.data as CardM);
        return true;
      }
      if (e.data is Beh) {
        _moved(_local(e.offset), e.data as Beh);
        return true;
      }
      return false;
    },
    onMove: (e) {
      if (e.data is Beh) _moved(_local(e.offset), e.data as Beh);
    },
    onLeave: (_) => _leave(),
    onAcceptWithDetails: (e) {
      if (e.data is CardM) t.wear(e.data as CardM);
      if (e.data is Beh) aim != null ? t.lay(_ringVals[aim!], e.data as Beh) : (anchor == null ? t.tapLay(e.data as Beh) : null); // (thrown before the ring opens: as a tap; let go in its middle: nothing)
      _leave();
    },
    builder: (context, cand, _) {
      final waiting = (t.carrying is CardM && !t.cards.contains(t.carrying)) || t.carrying is Beh; // (the stage takes these)
      final b = cand.whereType<Beh>().firstOrNull;
      return ClipRect(
        child: CustomPaint(
          painter: _Ring(t, _Time.of(context)),
          foregroundPainter: waiting ? _Frame(cand.isNotEmpty) : null,
          child: Stack(children: [
            const SizedBox.expand(),
            if (b != null && anchor != null)
              Positioned(left: anchor!.dx - 110, top: anchor!.dy - 80, width: 220, height: 160, child: _ValueRing(t, b, aim)),
          ]),
        ),
      );
    },
  );
}

/// The ring a behaviour card opens on the stage: four values around the place it came in, the one aimed at lit in its colour.
class _ValueRing extends StatelessWidget {
  const _ValueRing(this.t, this.b, this.aim);
  final CardTable t;
  final Beh b;
  final int? aim;
  @override
  Widget build(BuildContext context) {
    Widget pill(int k) {
      final v = t.valOr(_ringVals[k]), on = aim == k;
      return Container(
        padding: const EdgeInsets.symmetric(horizontal: 9, vertical: 5),
        decoration: BoxDecoration(color: on ? b.c : Grey.g13.withValues(alpha: .92), borderRadius: BorderRadius.circular(12), border: Border.all(color: on ? b.c : Grey.g38)),
        child: Text(v?.label ?? '—', style: T.name(on ? N.g07 : (v == null ? Grey.g44 : Grey.g91))),
      );
    }

    return Stack(alignment: Alignment.center, children: [
      Container(width: 22, height: 22, decoration: BoxDecoration(shape: BoxShape.circle, border: Border.all(color: Grey.g44))),
      Align(alignment: Alignment.topCenter, child: pill(0)),
      Align(alignment: Alignment.centerRight, child: pill(1)),
      Align(alignment: Alignment.bottomCenter, child: pill(2)),
      Align(alignment: Alignment.centerLeft, child: pill(3)),
    ]);
  }
}

/// The stage's edge while a shelf card is carried: the stage takes it.
class _Frame extends CustomPainter {
  _Frame(this.over);
  final bool over;
  @override
  void paint(Canvas c, Size s) => c.drawRRect(
    RRect.fromRectAndRadius((Offset.zero & s).deflate(10), const Radius.circular(10)),
    Paint()..color = Pop.accent.withValues(alpha: over ? .9 : .35)..style = PaintingStyle.stroke..strokeWidth = over ? 2 : 1,
  );
  @override
  bool shouldRepaint(_Frame o) => o.over != over;
}

class _Ring extends CustomPainter {
  _Ring(this.t, this.time) : super(repaint: Listenable.merge([t, time]));
  final CardTable t;
  final ValueNotifier<double> time;
  @override
  void paint(Canvas c, Size size) {
    final k = math.min(size.width, size.height) / 720;
    final n = t.at('count', 0, 1, fallback: 1).round();
    for (var i = 0; i < n; i++) {
      double v(String id, double f) => t.at(id, i, n, fallback: f);
      final m = size.center(Offset(v('posX', 0), v('posY', 0)) * k);
      final phase = (t.copiesFirst ? i / n * 2 * math.pi : 0.0) + v('phase', 0) * 2 * math.pi; // (copies first: each copy waves on its own)
      final wt = (t.waveClock - v('delay', 0) * i / n) * 2.4 - phase, w = math.sin(wt);
      final a = 2 * math.pi * i / n + v('turn', 0) * math.pi / 180 - math.pi / 2;
      final h = _hash(i * 3.3 + .7) * 2 * math.pi;
      final off = Offset.fromDirection(a, v('radius', 0)) + Offset(v('waveX', 0) * math.cos(wt), -v('waveH', 0) * w) + Offset.fromDirection(h, v('scatter', 0) * _hash(i + .3));
      final grow = v('copySize', 100) / 100 * (1 + v('waveSize', 0) / 100 * w) * (1 + (_hash(i * 2.1) - .5) * v('scatterSize', 0) / 100);
      final turn = (v('turn', 0) + i * v('spin', 0) + (_hash(i * 1.9) * 2 - 1) * v('scatterTurn', 0) + v('waveRot', 0) * w) * math.pi / 180;
      final col = t.valOr('hue') == null || t.cardOf('hue')!.down
          ? Color.fromRGBO(226, 226, 220, v('opacity', 100) / 100) // (no colour card worn: the stars are plain)
          : HSLColor.fromAHSL(v('opacity', 100) / 100, v('hue', 72) % 360, .62, v('light', 62) / 100).toColor();
      c.save();
      c.translate(m.dx + off.dx * k, m.dy + off.dy * k);
      c.scale(v('scaleX', 100) / 100 * math.cos(v('rotY', 0) * math.pi / 180), v('scaleY', 100) / 100 * math.cos(v('rotX', 0) * math.pi / 180));
      final star = _star(v('size', 44) * k / 2 * grow, turn);
      c.drawPath(star, Paint()..color = col);
      final sw = v('strokeW', 0);
      if (sw > 0) c.drawPath(star, Paint()..color = HSLColor.fromColor(col).withLightness(.9).toColor()..style = PaintingStyle.stroke..strokeWidth = sw * k);
      c.restore();
    }
  }

  Path _star(double r, double turn) {
    final p = Path();
    for (var j = 0; j < 10; j++) {
      final q = Offset.fromDirection(turn - math.pi / 2 + j * math.pi / 5, j.isEven ? r : r * .45);
      j == 0 ? p.moveTo(q.dx, q.dy) : p.lineTo(q.dx, q.dy);
    }
    return p..close();
  }

  @override
  bool shouldRepaint(_Ring o) => true;
}

// ---------------------------------------------------------------- the inspector

class _Inspector extends StatelessWidget {
  const _Inspector(this.t);
  final CardTable t;
  @override
  Widget build(BuildContext context) => Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
    Container(
      height: 30,
      padding: const EdgeInsets.symmetric(horizontal: Pop.inset),
      decoration: BoxDecoration(border: Border(bottom: BorderSide(color: Grey.g20))),
      child: Row(children: [
        Text('星 ×12 の輪', style: T.title(Grey.g95)),
        const Spacer(),
        GestureDetector(
          onTap: t.canUndo ? t.undo : null,
          child: Container(
            width: 26,
            height: 22,
            padding: const EdgeInsets.all(5),
            decoration: BoxDecoration(color: t.canUndo ? Pop.well : null, borderRadius: BorderRadius.circular(6)),
            child: CustomPaint(painter: _Back(t.canUndo ? Grey.g91 : Grey.g38)),
          ),
        ),
        const SizedBox(width: 2),
        GestureDetector(
          onTap: t.canRedo ? t.redo : null,
          child: Container(
            width: 26,
            height: 22,
            padding: const EdgeInsets.all(5),
            decoration: BoxDecoration(color: t.canRedo ? Pop.well : null, borderRadius: BorderRadius.circular(6)),
            child: Transform.flip(flipX: true, child: CustomPaint(painter: _Back(t.canRedo ? Grey.g91 : Grey.g38))),
          ),
        ),
      ]),
    ),
    for (final (i, b) in t.bundles.indexed) _BundleTile(t, i, b),
    Expanded(
      child: DragTarget<CardM>(
        onWillAcceptWithDetails: (e) => !t.cards.contains(e.data),
        onAcceptWithDetails: (e) => t.wear(e.data),
        builder: (context, cand, _) => _list(context),
      ),
    ),
  ]);

  Widget _list(BuildContext context) => ListView(
    padding: const EdgeInsets.all(Pop.inset),
    children: [for (final (i, c) in t.cards.indexed) _CardSlot(t, c, i, key: ValueKey(c.name))],
  );
}

/// One card in the list, taking a card carried by its head: let go on its upper half the carried card goes above, on the lower under.
class _CardSlot extends StatelessWidget {
  const _CardSlot(this.t, this.c, this.i, {super.key});
  final CardTable t;
  final CardM c;
  final int i;
  @override
  Widget build(BuildContext context) => DragTarget<CardM>(
    onWillAcceptWithDetails: (e) => (t.cards.contains(e.data) || t.shelf.contains(e.data)) && e.data != c,
    onMove: (e) {
      final box = context.findRenderObject() as RenderBox?;
      if (box == null || e.data == c || !(t.cards.contains(e.data) || t.shelf.contains(e.data))) return;
      t.markInsert((c, box.globalToLocal(e.offset).dy > box.size.height / 2));
    },
    onLeave: (_) => t.markInsert(null),
    onAcceptWithDetails: (e) => t.moveCard(e.data, c, t.insertMark?.$2 ?? false),
    builder: (context, cand, _) {
      final m = t.insertMark, line = m != null && m.$1 == c ? m.$2 : null;
      return Padding(
        padding: const EdgeInsets.only(bottom: Pop.gap * 3),
        child: Stack(clipBehavior: Clip.none, children: [
          _Card(t, c, i),
          if (line != null)
            Positioned(left: 0, right: 0, top: line ? null : -4, bottom: line ? -4 : null, height: 2, child: const ColoredBox(color: Pop.accent)),
        ]),
      );
    },
  );
}

class _Card extends StatelessWidget {
  const _Card(this.t, this.c, this.i);
  final CardTable t;
  final CardM c;
  final int i;
  @override
  Widget build(BuildContext context) {
    final square = Container(width: 14, height: 14, decoration: BoxDecoration(color: c.down ? Grey.g26 : c.tone, borderRadius: BorderRadius.circular(3)));
    final row = Row(children: [
      Padding(padding: const EdgeInsets.fromLTRB(5, 8, 5, 8), child: SizedBox(width: 8, height: 14, child: CustomPaint(painter: _Grip(Grey.g44)))),
      square,
      const SizedBox(width: 8),
      GestureDetector(onTap: c.down ? null : () => t.flip(c), child: Text(c.name, style: T.title(c.down ? Grey.g44 : Grey.g95))),
      if (c.name == '波' && !c.down) ...[const SizedBox(width: 8), GestureDetector(onTap: t.swapOrder, child: _Tag(t.copiesFirst ? '1つずつ' : '輪ごと'))],
      const Spacer(),
      if (!c.down) _IconButton(_Ic.flip, on: c.flipped, onTap: () => t.flip(c)),
      const SizedBox(width: 2),
      _IconButton(c.down ? _Ic.eyeOff : _Ic.eye, on: c.down, onTap: () => t.turnDown(c)),
      const SizedBox(width: 4),
    ]);
    // (the whole head is where a card is held: up or down the list it moves there, onto the shelf it comes off)
    final head = Draggable<CardM>(
      data: c,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: Transform.translate(offset: const Offset(-14, -17), child: _Lifted(_CardStub(c, wide: true))),
      childWhenDragging: Opacity(opacity: .35, child: SizedBox(height: 30, child: row)),
      onDragStarted: () => t.carry(c),
      onDragEnd: (_) {
        t.markInsert(null);
        t.carry(null);
      },
      child: MouseRegion(cursor: SystemMouseCursors.grab, child: Container(height: 30, color: const Color(0x00000000), child: row)),
    );
    final opened = t.open != null && c.all.any((v) => v.id == t.open) ? t.drives[t.open] : null;
    final body = c.flipped
        ? Column(children: [for (final v in c.all) _BackRow(t, v)])
        : LayoutBuilder(
            builder: (context, box) {
              final w = (box.maxWidth - Pop.tileGap * 2) / 3;
              return Wrap(spacing: Pop.tileGap, runSpacing: Pop.tileGap, children: [for (final v in c.front) SizedBox(width: w, child: _Tile(t, v))]);
            },
          );
    return AnimatedOpacity(
      duration: Mo.dur,
      opacity: c.down ? .45 : 1,
      child: TweenAnimationBuilder<double>(
        key: ValueKey(t.orderN),
        tween: Tween(begin: t.orderN == 0 ? 1 : 0, end: 1),
        duration: Duration(milliseconds: 260 + 140 * i),
        builder: (context, a, ch) {
          final lit = t.orderN == 0 || (c.name != '複製' && c.name != '波') ? 0.0 : math.max(0.0, 1 - ((a * (260 + 140 * i) - 140 * i) / 260 - .5).abs() * 2); // (each card lights after the one above)
          return Container(foregroundDecoration: BoxDecoration(border: Border.all(color: Pop.accent.withValues(alpha: lit), width: 1.5)), child: ch);
        },
        child: Container(
        color: Pop.card,
        child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
          head,
          Padding(
            padding: const EdgeInsets.fromLTRB(Pop.inset, 0, Pop.inset, Pop.insetY + 2),
            child: c.down
                ? _MarkRow(t, c.all)
                : Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                    AnimatedSwitcher(
                      duration: Mo.dur * 2,
                      transitionBuilder: _flipIn,
                      layoutBuilder: (cur, prev) => Stack(alignment: Alignment.topLeft, children: [...prev, ?cur]),
                      child: KeyedSubtree(key: ValueKey(c.flipped), child: body),
                    ),
                    if (!c.flipped) _MarkRow(t, c.back, prefix: '裏'),
                    if (opened != null) _DriveKnobs(t, t.valOr(t.open!)!, opened),
                  ]),
          ),
        ]),
      ),
      ),
    );
  }

  static Widget _flipIn(Widget child, Animation<double> a) => AnimatedBuilder(
    animation: a,
    builder: (context, ch) => Transform(alignment: Alignment.center, transform: Matrix4.identity()..setEntry(3, 2, .002)..rotateX((1 - a.value) * math.pi / 2), child: ch),
    child: child,
  );
}

/// What lies on values not in sight (a face-down card's, a face-up card's back), small: so nothing moves unseen.
class _MarkRow extends StatelessWidget {
  const _MarkRow(this.t, this.vals, {this.prefix});
  final CardTable t;
  final List<Val> vals;
  final String? prefix;
  @override
  Widget build(BuildContext context) {
    final on = [for (final v in vals) if (t.drives[v.id] != null || t.bundleOf(v) != null) v];
    if (on.isEmpty) return SizedBox(height: prefix == null ? 4 : 0);
    return Padding(
      padding: const EdgeInsets.only(top: 4),
      child: SizedBox(
        height: 18,
        child: Row(children: [
          if (prefix != null) ...[Text(prefix!, style: T.micro(Grey.g44)), const SizedBox(width: 6)],
          for (final v in on) ...[
            if (t.drives[v.id] != null) ...[_Chip(t.drives[v.id]!.beh, small: true), const SizedBox(width: 3)],
            if (t.bundleOf(v) != null) ...[_TieMark(t.bundleOf(v)!), const SizedBox(width: 3)],
            Text(v.label, style: T.label(Grey.g63)),
            const SizedBox(width: 10),
          ],
        ]),
      ),
    );
  }
}

class _Tag extends StatelessWidget {
  const _Tag(this.word);
  final String word;
  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
    decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(8)),
    child: Text(word, style: T.micro(Grey.g76)),
  );
}

enum _Ic { flip, eye, eyeOff }

class _IconButton extends StatelessWidget {
  const _IconButton(this.ic, {this.on = false, this.onTap});
  final _Ic ic;
  final bool on;
  final VoidCallback? onTap;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: onTap,
    behavior: HitTestBehavior.opaque,
    child: Container(
      width: 24,
      height: 22,
      decoration: BoxDecoration(color: on ? Grey.g26 : null, borderRadius: BorderRadius.circular(6)),
      padding: const EdgeInsets.all(5),
      child: CustomPaint(painter: _IcPaint(ic, on ? Grey.g95 : Grey.g56)),
    ),
  );
}

class _IcPaint extends CustomPainter {
  const _IcPaint(this.ic, this.c);
  final _Ic ic;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.3..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    switch (ic) {
      case _Ic.flip: // a card half turned over: its near edge tall, its far edge short
        cv.drawPath(Path()..moveTo(w * .15, 0)..lineTo(w * .85, h * .2)..lineTo(w * .85, h * .8)..lineTo(w * .15, h)..close(), st);
        cv.drawLine(Offset(w * .5, h * .1), Offset(w * .5, h * .9), st..color = c.withValues(alpha: .5));
      case _Ic.eye || _Ic.eyeOff:
        final p = Path()
          ..moveTo(0, h * .5)
          ..quadraticBezierTo(w * .5, -h * .15, w, h * .5)
          ..quadraticBezierTo(w * .5, h * 1.15, 0, h * .5);
        cv.drawPath(p, st);
        cv.drawCircle(Offset(w * .5, h * .5), w * .14, Paint()..color = c);
        if (ic == _Ic.eyeOff) cv.drawLine(Offset(w * .1, h * .95), Offset(w * .9, h * .05), st);
    }
  }

  @override
  bool shouldRepaint(_IcPaint o) => o.ic != ic || o.c != c;
}

/// Six dots: where a card is taken to move it up or down.
class _Grip extends CustomPainter {
  const _Grip(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    for (var x = 0; x < 2; x++) {
      for (var y = 0; y < 3; y++) {
        cv.drawCircle(Offset(1.5 + x * (s.width - 3), 1.5 + y * (s.height - 3) / 2), 1.3, Paint()..color = c);
      }
    }
  }

  @override
  bool shouldRepaint(_Grip o) => o.c != c;
}

/// What a value tile or a back row does with what is dropped on it: a behaviour drives it, a behaviour's mark moves to it,
/// another value ties to it. [may]: what the hand holds now could go here (outlined before it arrives).
class _ValDrop extends StatelessWidget {
  const _ValDrop(this.t, this.v, this.builder);
  final CardTable t;
  final Val v;
  final Widget Function(BuildContext, Beh? over, bool tying, bool may) builder;
  bool _ok(Object? o) => switch (o) {
    Beh() => CardTable.canDrive(v),
    String() => CardTable.canDrive(v),
    Val() => o != v && CardTable.canDrive(o) && CardTable.canDrive(v), // (a count is neither moved nor tied)
    _ => false,
  };
  @override
  Widget build(BuildContext context) => DragTarget<Object>(
    onWillAcceptWithDetails: (e) {
      final ok = _ok(e.data);
      if (ok && e.data is Beh) t.hover((v.id, e.data as Beh));
      t.refusing.value = !ok;
      return ok;
    },
    onLeave: (_) {
      t.hover(null);
      t.refusing.value = false;
    },
    onAcceptWithDetails: (e) => switch (e.data) {
      Beh b => t.lay(v.id, b),
      String from => t.moveDrive(from, v.id),
      Val a => t.tie(a, v),
      _ => null,
    },
    builder: (context, cands, _) {
      final mark = cands.whereType<String>().firstOrNull;
      final over = cands.whereType<Beh>().firstOrNull ?? (mark != null && mark != v.id ? t.drives[mark]?.beh : null);
      return builder(context, over, cands.whereType<Val>().isNotEmpty, _ok(t.carrying));
    },
  );
}

/// One value as today's block (name top-left, number large); a behaviour card can be laid on it; held, it lifts (to tie it to another).
class _Tile extends StatelessWidget {
  const _Tile(this.t, this.v);
  final CardTable t;
  final Val v;
  @override
  Widget build(BuildContext context) {
    final d = t.drives[v.id], tied = t.bundleOf(v);
    return _ValDrop(t, v, (context, overHere, tying, may) {
      final over = overHere ?? (t.preview?.$1 == v.id ? t.preview!.$2 : null); // (aimed at from the stage's ring: lit here too)
      final held = t.carrying;
      final waiting = held is Beh ? held.c : (held is String ? t.drives[held]?.beh.c : Pop.accent);
      final edge = over?.c ?? (tying ? Pop.accent : (may && waiting != null ? waiting.withValues(alpha: .45) : const Color(0x00000000)));
      final width = over != null ? (d != null && d.beh == over ? math.min(d.width + .15, 1.0) : (d?.width ?? .2)) : d?.width;
      // (the name strip is where the tile is held up and carried, to tie it to another; the rest scrubs, the number types)
      final label = !CardTable.canDrive(v) ? Text(v.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g44).copyWith(height: 1.1)) : LongPressDraggable<Val>(
        data: v,
        delay: const Duration(milliseconds: 300),
        dragAnchorStrategy: pointerDragAnchorStrategy,
        onDragStarted: () => t.carry(v),
        onDragEnd: (_) => t.carry(null),
        feedback: Transform.translate(
          offset: const Offset(-20, -12),
          child: _Lifted(Container(
            width: 104,
            height: Pop.tile,
            padding: const EdgeInsets.fromLTRB(Pop.tilePad, 6, 6, 4),
            decoration: BoxDecoration(color: Grey.g20, borderRadius: BorderRadius.circular(6), border: Border.all(color: Pop.accent)),
            child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
              Text(v.label, style: T.micro(Grey.g76)),
              Text(sayVal(v, v.v), style: T.value(Grey.g95).copyWith(fontSize: 15, fontWeight: FontWeight.w600)),
            ]),
          )),
        ),
        child: MouseRegion(cursor: CardTable.canDrive(v) ? SystemMouseCursors.grab : MouseCursor.defer, child: SizedBox(
          width: double.infinity,
          child: Row(children: [
            if (CardTable.canDrive(v)) ...[SizedBox(width: 4, height: 8, child: CustomPaint(painter: _Grip(Grey.g38))), const SizedBox(width: 3)], // (held here, the tile lifts)
            Flexible(child: Text(v.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(CardTable.canDrive(v) ? Grey.g63 : Grey.g44).copyWith(height: 1.1))),
          ]),
        )),
      );
      // (the number and its line scrub sideways; the name strip above only lifts: a mouse's first pixel never takes the hold away)
      Widget scrub(Widget child) => MouseRegion(
        cursor: SystemMouseCursors.resizeLeftRight,
        child: GestureDetector(
          behavior: HitTestBehavior.opaque,
          onHorizontalDragStart: (_) {
            if (t.editing == v.id) t.type(null);
            t.mark();
            t.scrubbing = true;
          },
          onHorizontalDragEnd: (_) => t.scrubbing = false,
          onHorizontalDragCancel: () => t.scrubbing = false,
          onHorizontalDragUpdate: (e) => t.set(v, v.v + e.delta.dx / 180 * (v.hi - v.lo)),
          child: child,
        ),
      );
      return (TweenAnimationBuilder<double>(
          key: ValueKey(t.flash == v.id ? t.flashN : -1),
          tween: Tween(begin: t.flash == v.id ? 1 : 0, end: 0),
          duration: const Duration(milliseconds: 420),
          builder: (context, glow, _) => AnimatedContainer(
            duration: Mo.dur,
            height: Pop.tile,
            decoration: BoxDecoration(
              color: Color.lerp(over != null ? Grey.g20 : Pop.well, (d?.beh.c ?? Pop.accent), glow * .35),
              borderRadius: BorderRadius.circular(6),
              border: Border.all(color: edge.a == 0 && t.touched == v.id ? Pop.accent.withValues(alpha: .45) : edge, width: 1.5), // (the last touched: where a tapped hand card goes)
            ),
            padding: const EdgeInsets.fromLTRB(Pop.tilePad, 5, 5, 4),
            child: Stack(children: [
              Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
                Padding(padding: const EdgeInsets.only(right: 24), child: label),
                scrub(Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisSize: MainAxisSize.min, children: [
                t.editing == v.id
                    ? _NumEdit(t, v)
                    : GestureDetector(
                        onTap: () => t.type(v.id),
                        child: Row(crossAxisAlignment: CrossAxisAlignment.end, children: [
                          Text(sayVal(v, v.v), style: T.value(Grey.g95).copyWith(fontSize: 15, fontWeight: FontWeight.w600)),
                          if (width != null) ...[
                            const SizedBox(width: 4),
                            Flexible(child: Text('±${(width * (v.hi - v.lo)).toStringAsFixed(v.dec)}${v.unit}', maxLines: 1, overflow: TextOverflow.clip, style: T.micro((over ?? d!.beh).c).copyWith(fontSize: 9))),
                          ],
                        ]),
                      ),
                const SizedBox(height: 3),
                SizedBox(height: 4, width: double.infinity, child: d != null || over != null ? _Spread(t, v, (over ?? d!.beh).c) : null),
                ])),
              ]),
              // (the right edge is one column of marks: what drives it on top, its bundle under; nothing sits on the words)
              if (d != null) Positioned(right: 0, top: 0, child: _DriveChip(t, v.id, d)),
              if (d == null && over == null && !((held is Beh || held is String) && !CardTable.canDrive(v))) Positioned(right: 1, top: 2, child: _Gauge((v.v - v.lo) / (v.hi - v.lo))),
              if (tied != null)
                Positioned(
                  right: 0,
                  top: 20,
                  child: Draggable<Val>( // (the bundle's mark carried: onto another value it ties there, onto the hand it lets this one out)
                    data: v,
                    dragAnchorStrategy: pointerDragAnchorStrategy,
                    feedback: Transform.translate(offset: const Offset(-12, -8), child: _Lifted(_TieMark(tied))),
                    childWhenDragging: Opacity(opacity: .3, child: _TieMark(tied)),
                    onDragStarted: () => t.carry(v),
                    onDragEnd: (_) => t.carry(null),
                    child: _TieMark(tied),
                  ),
                ),
              if ((held is Beh || held is String) && !CardTable.canDrive(v)) Positioned(right: 2, top: 3, child: SizedBox.square(dimension: 10, child: CustomPaint(painter: _Cross(Grey.g56)))),
              if (over != null && d?.beh != over) Positioned(right: 0, top: 0, child: Opacity(opacity: .6, child: _Chip(over))),
            ]),
          ),
        ));
    });
  }
}

/// The number being typed: Return keeps it (a unit typed or not), Escape or a tap elsewhere leaves it as it was.
class _NumEdit extends StatefulWidget {
  _NumEdit(this.t, Val v, {this.small = false}) : initial = v.v.toStringAsFixed(v.dec), keep = ((x) => t..mark()..set(v, x));
  const _NumEdit.of(this.t, this.initial, this.keep, {this.small = false});
  final CardTable t;
  final String initial;
  final void Function(double) keep;
  final bool small;
  @override
  State<_NumEdit> createState() => _NumEditState();
}

class _NumEditState extends State<_NumEdit> {
  late final c = TextEditingController(text: widget.initial)..selection = TextSelection(baseOffset: 0, extentOffset: widget.initial.length);
  final f = FocusNode();
  bool bad = false;
  @override
  void dispose() {
    c.dispose();
    f.dispose();
    super.dispose();
  }

  void _keep() {
    final half = String.fromCharCodes(c.text.runes.map((r) => r >= 0xFF10 && r <= 0xFF19 ? r - 0xFEE0 : (r == 0xFF0E ? 0x2E : (r == 0xFF0D || r == 0x2212 ? 0x2D : r))));
    final m = RegExp(r'-?(\d+\.?\d*|\.\d+)').firstMatch(half), x = m == null ? null : double.tryParse(m[0]!.replaceFirst(RegExp(r'^(-?)\.'), r'$10.'));
    if (x == null) return setState(() => bad = true); // (not a number: the field stays, red, until it is one or left)
    widget.keep(x);
    widget.t.type(null);
  }

  @override
  Widget build(BuildContext context) => SizedBox(
    height: widget.small ? 14 : 18,
    child: Focus(
      onKeyEvent: (_, e) {
        if (e.logicalKey.keyLabel == 'Escape') {
          widget.t.type(null);
          return KeyEventResult.handled;
        }
        if (e.logicalKey.keyLabel == 'Tab') {
          _keep();
          return KeyEventResult.handled;
        }
        return KeyEventResult.ignored;
      },
      child: EditableText(
        controller: c,
        focusNode: f,
        autofocus: true,
        style: T.value(bad ? C.danger : Pop.accent).copyWith(fontSize: widget.small ? 10 : 15, fontWeight: FontWeight.w600),
        cursorColor: Pop.accent,
        backgroundCursorColor: Grey.g44,
        selectionColor: Pop.accent.withValues(alpha: .3),
        onSubmitted: (_) => _keep(),
        onTapOutside: (_) => widget.t.type(null),
      ),
    ),
  );
}

/// Today's small picture of a value in a tile's corner: bars filled as far as the value is along its range.
class _Gauge extends StatelessWidget {
  const _Gauge(this.f);
  final double f;
  @override
  Widget build(BuildContext context) => SizedBox(width: 16, height: 11, child: CustomPaint(painter: _GaugePaint(f.clamp(0.0, 1.0))));
}

class _GaugePaint extends CustomPainter {
  const _GaugePaint(this.f);
  final double f;
  @override
  void paint(Canvas c, Size s) {
    for (var k = 0; k < 4; k++) {
      final h = s.height * (k + 1) / 4, on = f >= (k + .5) / 4;
      c.drawRRect(RRect.fromRectAndRadius(Rect.fromLTWH(k * s.width / 4, s.height - h, s.width / 4 - 1.5, h), const Radius.circular(1)), Paint()..color = on ? Grey.g76 : Grey.g26);
    }
  }

  @override
  bool shouldRepaint(_GaugePaint o) => o.f != f;
}

/// Where each copy's value is now, as one short line per copy along the value's range ("all different" shows as lines apart),
/// with the number itself as a grey line.
class _Spread extends StatelessWidget {
  const _Spread(this.t, this.v, this.c);
  final CardTable t;
  final Val v;
  final Color c;
  @override
  Widget build(BuildContext context) => CustomPaint(painter: _SpreadPaint(t, v, c, _Time.of(context)), size: Size.infinite);
}

class _SpreadPaint extends CustomPainter {
  _SpreadPaint(this.t, this.v, this.c, this.time) : super(repaint: time);
  final CardTable t;
  final Val v;
  final Color c;
  final ValueNotifier<double> time;
  @override
  void paint(Canvas cv, Size s) {
    final n = t.at('count', 0, 1, fallback: 12).round(), lo = v.lo, hi = v.hi;
    cv.drawRect(Rect.fromLTWH(0, s.height - 1, s.width, 1), Paint()..color = Grey.g26);
    final base = (v.v - lo) / (hi - lo) * s.width;
    cv.drawRect(Rect.fromLTWH(base - .5, 0, 1, s.height), Paint()..color = Grey.g76);
    for (var i = 0; i < n; i++) {
      final x = (t.at(v.id, i, n) - lo) / (hi - lo) * s.width;
      cv.drawRect(Rect.fromLTWH(x - .75, 0, 1.5, s.height), Paint()..color = c.withValues(alpha: .8));
    }
  }

  @override
  bool shouldRepaint(_SpreadPaint o) => true;
}

/// A value on the back of a card: a row, scrubbed sideways, taking a behaviour like a tile.
class _BackRow extends StatelessWidget {
  const _BackRow(this.t, this.v);
  final CardTable t;
  final Val v;
  @override
  Widget build(BuildContext context) {
    final d = t.drives[v.id];
    return _ValDrop(t, v, (context, over, tying, may) => GestureDetector(
      onHorizontalDragStart: (_) {
        if (t.editing == v.id) t.type(null);
        t.mark();
        t.scrubbing = true;
      },
      onHorizontalDragEnd: (_) => t.scrubbing = false,
      onHorizontalDragCancel: () => t.scrubbing = false,
      onHorizontalDragUpdate: (e) => t.set(v, v.v + e.delta.dx / 220 * (v.hi - v.lo)),
      child: Container(
        height: 22,
        margin: const EdgeInsets.only(bottom: 1),
        padding: const EdgeInsets.symmetric(horizontal: 6),
        decoration: BoxDecoration(
          color: over != null ? Grey.g20 : null,
          borderRadius: BorderRadius.circular(4),
          border: Border.all(color: over?.c ?? (may ? Grey.g26 : const Color(0x00000000))),
        ),
        child: Row(children: [
          Text(v.label, style: T.label(CardTable.canDrive(v) ? Grey.g63 : Grey.g44)),
          if (d != null) ...[const SizedBox(width: 6), _DriveChip(t, v.id, d, small: true)],
          if (over != null && d?.beh != over) ...[const SizedBox(width: 6), Opacity(opacity: .6, child: _Chip(over, small: true))],
          const Spacer(),
          if (d != null || over != null) SizedBox(width: 60, height: 4, child: _Spread(t, v, (over ?? d!.beh).c)),
          if (d != null) ...[const SizedBox(width: 4), Text('±${(d.width * (v.hi - v.lo)).toStringAsFixed(v.dec)}${v.unit}', style: T.micro(d.beh.c).copyWith(fontSize: 9))],
          const SizedBox(width: 8),
          if (t.bundleOf(v) != null) ...[GestureDetector(onDoubleTap: () => t.untieOne(v), child: _TieMark(t.bundleOf(v)!)), const SizedBox(width: 6)],
          SizedBox(
            width: 52,
            child: t.editing == v.id
                ? _NumEdit(t, v, small: true)
                : GestureDetector(onTap: () => t.type(v.id), child: Text(sayVal(v, v.v), textAlign: TextAlign.right, style: T.value(Grey.g91).copyWith(fontSize: 10))),
          ),
        ]),
      ),
    ));
  }
}

/// The behaviour's own knobs, under the card, while its mark is open: how wide, how fast.
class _DriveKnobs extends StatelessWidget {
  const _DriveKnobs(this.t, this.v, this.d);
  final CardTable t;
  final Val v;
  final Drive d;
  @override
  Widget build(BuildContext context) {
    Widget knob(String label, String said, String key, String initial, void Function(double) keep, void Function(double dx) on) => Expanded(
      child: GestureDetector(
        onHorizontalDragStart: (_) {
          t.mark();
          t.scrubbing = true;
        },
        onHorizontalDragEnd: (_) => t.scrubbing = false,
        onHorizontalDragCancel: () => t.scrubbing = false,
        onHorizontalDragUpdate: (e) => on(e.delta.dx),
        child: Container(
          height: 38,
          padding: const EdgeInsets.fromLTRB(Pop.tilePad, 5, 6, 4),
          decoration: BoxDecoration(color: Pop.well, borderRadius: BorderRadius.circular(6), border: Border.all(color: d.beh.c.withValues(alpha: .6))),
          child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
            Text(label, style: T.micro(Grey.g63)),
            t.editing == key
                ? _NumEdit.of(t, initial, keep, small: true)
                : GestureDetector(onTap: () => t.type(key), child: Text(said, style: T.value(d.beh.c).copyWith(fontSize: 13, fontWeight: FontWeight.w600))),
          ]),
        ),
      ),
    );
    final span = v.hi - v.lo;
    return Padding(
      padding: const EdgeInsets.only(top: Pop.tileGap),
      child: Column(crossAxisAlignment: CrossAxisAlignment.start, children: [
        SizedBox(
          height: 18,
          child: Row(children: [
            _Chip(d.beh, small: true),
            const SizedBox(width: 5),
            Text('${d.beh.word} → ${v.label}', style: T.micro(Grey.g76)),
          ]),
        ),
        const SizedBox(height: 3),
        Row(children: [
          knob('幅', '±${(d.width * span).toStringAsFixed(v.dec)}${v.unit}', 'w:${v.id}', (d.width * span).toStringAsFixed(v.dec),
              (x) => t..mark()..setDrive(d, width: x.abs() / span), (dx) => t.setDrive(d, width: d.width + dx / 300)),
          if (d.beh == Beh.sway || d.beh == Beh.noise) ...[
            const SizedBox(width: Pop.tileGap),
            knob('速さ', d.speed.toStringAsFixed(2), 's:${v.id}', d.speed.toStringAsFixed(2), (x) => t..mark()..setDrive(d, speed: x), (dx) => t.setDrive(d, speed: d.speed + dx / 80)),
          ],
          if (d.beh == Beh.noise || d.beh == Beh.random) ...[
            const SizedBox(width: Pop.tileGap),
            knob('シード', d.seed.round().toString(), 'r:${v.id}', d.seed.round().toString(), (x) => t..mark()..setDrive(d, seed: x), (dx) => t.setDrive(d, seed: d.seed + dx / 8)),
          ],
          if (d.beh == Beh.stagger) ...[
            const SizedBox(width: Pop.tileGap),
            Expanded(
              child: SizedBox(
                height: 38,
                child: Center(child: Segmented(items: const ['端', '中', '逆'], index: d.from, expand: true, onChanged: (k) => t..mark()..setDrive(d, from: k))),
              ),
            ),
          ],
        ]),
      ]),
    );
  }
}


/// A value's mark when it is tied into a bundle: which bundle.
class _TieMark extends StatelessWidget {
  const _TieMark(this.i);
  final int i;
  @override
  Widget build(BuildContext context) => Container(
    height: 16,
    constraints: const BoxConstraints(minWidth: 16),
    padding: const EdgeInsets.symmetric(horizontal: 3),
    decoration: BoxDecoration(border: Border.all(color: Pop.accent), borderRadius: BorderRadius.circular(8)),
    child: Center(child: Text('束${i + 1}', style: T.micro(Pop.accentInk).copyWith(fontSize: 8))),
  );
}

/// A bundle of values, over the cards: scrubbed sideways it moves all of them; double-tapped it unties.
class _BundleTile extends StatelessWidget {
  const _BundleTile(this.t, this.i, this.vals);
  final CardTable t;
  final int i;
  final List<Val> vals;
  @override
  Widget build(BuildContext context) => DragTarget<Val>(
    onWillAcceptWithDetails: (e) => !vals.contains(e.data) && CardTable.canDrive(e.data),
    onAcceptWithDetails: (e) => t.tie(e.data, vals.first),
    builder: (context, cand, _) => GestureDetector(
    onHorizontalDragStart: (_) {
      t.startBundle(i);
      t.scrubbing = true;
    },
    onHorizontalDragEnd: (_) => t.scrubbing = false,
    onHorizontalDragCancel: () => t.scrubbing = false,
    onHorizontalDragUpdate: (e) => t.scrubBundle(i, e.delta.dx / 260),
    onDoubleTap: () => t.untie(i),
    child: Container(
      height: Pop.tile,
      margin: const EdgeInsets.fromLTRB(Pop.inset, Pop.inset, Pop.inset, 0),
      padding: const EdgeInsets.symmetric(horizontal: Pop.tilePad),
      decoration: BoxDecoration(color: cand.isNotEmpty ? Grey.g20 : Pop.card, borderRadius: BorderRadius.circular(6), border: Border.all(color: Pop.accent, width: cand.isNotEmpty ? 2 : 1)),
      child: Row(children: [
        _TieMark(i),
        const SizedBox(width: 8),
        Expanded(
          child: Row(children: [
            for (final v in vals)
              Expanded(
                child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.center, children: [
                  Text(v.label, maxLines: 1, overflow: TextOverflow.ellipsis, style: T.micro(Grey.g63)),
                  const SizedBox(height: 3),
                  t.editing == 'b:${v.id}'
                      ? _NumEdit.of(t, v.v.toStringAsFixed(v.dec), (x) => t..mark()..set(v, x), small: true)
                      : GestureDetector(onTap: () => t.type('b:${v.id}'), child: Text(sayVal(v, v.v), style: T.value(t.bundleStop == v ? C.danger : Grey.g95).copyWith(fontSize: 12, fontWeight: FontWeight.w600))),
                  const SizedBox(height: 3),
                  SizedBox(height: 3, width: 40, child: _Bar((v.v - v.lo) / (v.hi - v.lo))),
                ]),
              ),
          ]),
        ),
      ]),
    ),
    ),
  );
}

class _Bar extends StatelessWidget {
  const _Bar(this.f);
  final double f;
  @override
  Widget build(BuildContext context) => Stack(children: [
    Positioned.fill(child: ColoredBox(color: Grey.g26)),
    FractionallySizedBox(widthFactor: f.clamp(0.0, 1.0), child: const ColoredBox(color: Pop.accent)),
  ]);
}

/// A card as it lies on the shelf (or flies): its colour and its name.
class _CardStub extends StatelessWidget {
  const _CardStub(this.c, {this.wide = false});
  final CardM c;
  final bool wide;
  @override
  Widget build(BuildContext context) => Container(
    width: wide ? 150 : null,
    height: 34,
    padding: const EdgeInsets.symmetric(horizontal: 8),
    decoration: BoxDecoration(
      color: Pop.card,
      borderRadius: BorderRadius.circular(6),
      border: Border.all(color: Grey.g26),
      boxShadow: const [BoxShadow(color: Color(0x66000000), offset: Offset(2, 2))], // (a card on a pile: the one under it shows)
    ),
    child: Row(mainAxisSize: MainAxisSize.min, children: [
      Container(width: 12, height: 12, decoration: BoxDecoration(color: c.tone, borderRadius: BorderRadius.circular(3))),
      const SizedBox(width: 7),
      Text(c.name, style: T.name(Grey.g95)),
    ]),
  );
}

/// The behaviour on a value: tapped, its knobs open; carried to another value, it moves there; dropped on the hand, it lets go.
class _DriveChip extends StatelessWidget {
  const _DriveChip(this.t, this.id, this.d, {this.small = false});
  final CardTable t;
  final String id;
  final Drive d;
  final bool small;
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: () => t.toggleOpen(id),
    child: Draggable<String>(
      data: id,
      dragAnchorStrategy: pointerDragAnchorStrategy,
      feedback: ValueListenableBuilder<bool>(
        valueListenable: t.refusing,
        builder: (context, no, _) => Transform.translate(offset: const Offset(-9, -9), child: _Lifted(no ? _Refused(_Chip(d.beh)) : _Chip(d.beh))),
      ),
      childWhenDragging: Opacity(opacity: .3, child: _Chip(d.beh, small: small)),
      onDragStarted: () => t.carry(id),
      onDragEnd: (_) => t.carry(null),
      child: Container(
        decoration: BoxDecoration(borderRadius: BorderRadius.circular(9), border: Border.all(color: t.open == id ? Grey.g95 : const Color(0x00000000))),
        child: _Chip(d.beh, small: small),
      ),
    ),
  );
}

class _Chip extends StatelessWidget {
  const _Chip(this.b, {this.small = false});
  final Beh b;
  final bool small;
  @override
  Widget build(BuildContext context) => Container(
    height: small ? 14 : 18,
    width: small ? 14 : 18,
    padding: EdgeInsets.all(small ? 3 : 4),
    decoration: BoxDecoration(color: b.c, borderRadius: BorderRadius.circular(9)),
    child: CustomPaint(painter: BehGlyph(b, N.g07)),
  );
}

/// A card in the air over a place that does not take it: greyed, crossed.
class _Refused extends StatelessWidget {
  const _Refused(this.child);
  final Widget child;
  @override
  Widget build(BuildContext context) => Stack(alignment: Alignment.center, children: [
    Opacity(opacity: .4, child: child),
    SizedBox.square(dimension: 14, child: CustomPaint(painter: _Cross(C.danger))),
  ]);
}

/// A card in the air: lifted, a little turned, with a shadow under it.
class _Lifted extends StatelessWidget {
  const _Lifted(this.child);
  final Widget child;
  @override
  Widget build(BuildContext context) => Transform.rotate(
    angle: -.06,
    child: DecoratedBox(
      decoration: const BoxDecoration(boxShadow: [BoxShadow(color: Color(0xAA000000), blurRadius: 16, offset: Offset(0, 10))]),
      child: child,
    ),
  );
}

// ---------------------------------------------------------------- the shelf and the hand

/// The shelf (cards the thing may wear) over the hand (behaviour cards). Cards and behaviour marks dropped here go back.
class _Hand extends StatelessWidget {
  const _Hand(this.t);
  final CardTable t;
  @override
  Widget build(BuildContext context) => DragTarget<Object>(
    onWillAcceptWithDetails: (e) => (e.data is CardM && t.cards.contains(e.data)) || e.data is String || (e.data is Val && t.bundleOf(e.data as Val) != null),
    onAcceptWithDetails: (e) => switch (e.data) {
      CardM c => t.shelve(c),
      String id => t.lift(id),
      Val v => t.untieOne(v),
      _ => null,
    },
    builder: (context, cand, _) {
      final held = t.carrying, waiting = (held is CardM && t.cards.contains(held)) || held is String || (held is Val && t.bundleOf(held) != null);
      return AnimatedContainer(
        duration: Mo.dur,
        padding: const EdgeInsets.fromLTRB(Pop.inset, 8, Pop.inset, 10),
        decoration: BoxDecoration(
          color: cand.isNotEmpty ? Grey.g13 : Grey.g07,
          border: Border(top: BorderSide(color: cand.isNotEmpty ? Pop.accent : (waiting ? Pop.accent.withValues(alpha: .4) : Grey.g20), width: waiting ? 2 : 1)),
        ),
        child: Stack(children: [
          Column(mainAxisSize: MainAxisSize.min, children: [
          SizedBox(
            width: double.infinity,
            child: Wrap(spacing: 0, runSpacing: 6, children: [
              if (t.shelf.isEmpty) Container(width: 90, height: 34, decoration: BoxDecoration(borderRadius: BorderRadius.circular(6), border: Border.all(color: Grey.g26))), // (the shelf, empty: still where cards come back to)
              for (final c in t.shelf)
                Padding(
                  padding: const EdgeInsets.only(right: 8),
                  child: Draggable<CardM>(
                    data: c,
                    dragAnchorStrategy: pointerDragAnchorStrategy,
                    feedback: Transform.translate(offset: const Offset(-14, -17), child: _Lifted(_CardStub(c, wide: true))),
                    childWhenDragging: Opacity(opacity: .35, child: _CardStub(c)),
                    onDragStarted: () => t.carry(c),
                    onDragEnd: (_) => t.carry(null),
                    child: _CardStub(c),
                  ),
                ),
            ]),
          ),
          const SizedBox(height: 8),
          Row(children: [for (final b in t.hand) Expanded(child: Padding(padding: const EdgeInsets.symmetric(horizontal: 3), child: _HandCard(t, b)))]),
          ]),
          if (held is String || (held is Val && t.bundleOf(held) != null))
            Positioned.fill(
              child: Container(
                decoration: BoxDecoration(color: Grey.g07.withValues(alpha: .88), borderRadius: BorderRadius.circular(8), border: Border.all(color: cand.isNotEmpty ? Pop.accent : Grey.g38)),
                child: Center(child: Row(mainAxisSize: MainAxisSize.min, children: [
                  SizedBox.square(dimension: 14, child: CustomPaint(painter: _Down(cand.isNotEmpty ? Pop.accent : Grey.g76))),
                  const SizedBox(width: 8),
                  Text('外す', style: T.name(cand.isNotEmpty ? Pop.accent : Grey.g76)),
                ])),
              ),
            ),
        ]),
      );
    },
  );
}

class _HandCard extends StatelessWidget {
  const _HandCard(this.t, this.b);
  final CardTable t;
  final Beh b;
  Widget face({bool lifted = false}) => Container(
    height: 64,
    width: lifted ? 80 : null,
    padding: const EdgeInsets.all(8),
    decoration: BoxDecoration(
      color: Grey.g13,
      borderRadius: BorderRadius.circular(8),
      border: Border.all(color: lifted ? b.c : Grey.g26),
      boxShadow: const [BoxShadow(color: Color(0x66000000), offset: Offset(2, 2)), BoxShadow(color: Color(0x33000000), offset: Offset(4, 4))], // (a pile: the same card again under it)
    ),
    child: Column(crossAxisAlignment: CrossAxisAlignment.start, mainAxisAlignment: MainAxisAlignment.spaceBetween, children: [
      Container(
        width: 24,
        height: 24,
        decoration: BoxDecoration(color: b.c, borderRadius: BorderRadius.circular(6)),
        child: Padding(padding: const EdgeInsets.all(5), child: CustomPaint(painter: BehGlyph(b, N.g07))),
      ),
      Text(b.word, style: T.name(Grey.g95)),
    ]),
  );
  @override
  Widget build(BuildContext context) => GestureDetector(
    onTap: () => t.tapLay(b),
    child: TweenAnimationBuilder<double>(
      key: ValueKey(t.nope == b ? t.nopeN : -1),
      tween: Tween(begin: t.nope == b && t.nopeN > 0 ? 1 : 0, end: 0),
      duration: const Duration(milliseconds: 360),
      builder: (context, k, ch) => Transform.translate(offset: Offset(math.sin(k * math.pi * 6) * 5 * k, 0), child: ch), // (no value to go on: a shake, as a "no")
      child: Draggable<Beh>(
    data: b,
    dragAnchorStrategy: pointerDragAnchorStrategy,
    feedback: ListenableBuilder( // (its mark is the fingertip: what is under it takes it)
      listenable: Listenable.merge([t.ringOpen, t.refusing]),
      builder: (context, _) => t.ringOpen.value
          ? Transform.translate(offset: const Offset(-9, -9), child: _Chip(b))
          : Transform.translate(offset: const Offset(-20, -20), child: _Lifted(t.refusing.value ? _Refused(face(lifted: true)) : face(lifted: true))),
    ),
    childWhenDragging: face(),
    onDragStarted: () => t.carry(b),
    onDragEnd: (_) => t.carry(null),
    child: MouseRegion(cursor: SystemMouseCursors.grab, child: face()),
  ),
    ),
  );
}

/// Back: the arrow of taking the last change back.
class _Back extends CustomPainter {
  const _Back(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.5..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    cv.drawPath(Path()..moveTo(w * .1, h * .4)..lineTo(w * .62, h * .4)..arcToPoint(Offset(w * .62, h * .95), radius: Radius.circular(h * .28))..lineTo(w * .35, h * .95), st);
    cv.drawPath(Path()..moveTo(w * .35, h * .12)..lineTo(w * .08, h * .4)..lineTo(w * .35, h * .68), st);
  }

  @override
  bool shouldRepaint(_Back o) => o.c != c;
}

/// Down into the hand: what is carried back is let go.
class _Down extends CustomPainter {
  const _Down(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, h = s.height, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = 1.6..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    cv.drawLine(Offset(w * .5, 0), Offset(w * .5, h * .62), st);
    cv.drawPath(Path()..moveTo(w * .22, h * .36)..lineTo(w * .5, h * .64)..lineTo(w * .78, h * .36), st);
    cv.drawPath(Path()..moveTo(0, h * .7)..lineTo(0, h)..lineTo(w, h)..lineTo(w, h * .7), st);
  }

  @override
  bool shouldRepaint(_Down o) => o.c != c;
}

class _Cross extends CustomPainter {
  const _Cross(this.c);
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final st = Paint()..color = c..strokeWidth = 1.6..strokeCap = StrokeCap.round;
    cv.drawLine(Offset.zero, Offset(s.width, s.height), st);
    cv.drawLine(Offset(s.width, 0), Offset(0, s.height), st);
  }

  @override
  bool shouldRepaint(_Cross o) => o.c != c;
}

/// What a behaviour does, drawn: a wave, a staircase, a wobble, scattered dots.
class BehGlyph extends CustomPainter {
  const BehGlyph(this.b, this.c);
  final Beh b;
  final Color c;
  @override
  void paint(Canvas cv, Size s) {
    final w = s.width, st = Paint()..color = c..style = PaintingStyle.stroke..strokeWidth = w * .14..strokeCap = StrokeCap.round..strokeJoin = StrokeJoin.round;
    final p = Path();
    switch (b) {
      case Beh.sway:
        for (var x = 0.0; x <= 1.0001; x += .05) {
          final y = .5 - .32 * math.sin(x * 2 * math.pi);
          x == 0 ? p.moveTo(0, y * w) : p.lineTo(x * w, y * w);
        }
      case Beh.stagger:
        p.moveTo(0, w * .9);
        for (var k = 0; k < 4; k++) {
          p
            ..lineTo(w * k / 4, w * (.9 - k * .25))
            ..lineTo(w * (k + 1) / 4, w * (.9 - k * .25));
        }
      case Beh.noise:
        for (var x = 0.0; x <= 1.0001; x += .1) {
          final y = .5 + .38 * (_hash(x * 9 + 2) - .5) * 2;
          x == 0 ? p.moveTo(0, y * w) : p.lineTo(x * w, y * w);
        }
      case Beh.random:
        for (final (x, y) in const [(.2, .3), (.7, .2), (.45, .55), (.15, .8), (.8, .75)]) {
          cv.drawCircle(Offset(x * w, y * w), w * .1, Paint()..color = c);
        }
        return;
    }
    cv.drawPath(p, st);
  }

  @override
  bool shouldRepaint(BehGlyph o) => o.b != b || o.c != c;
}
