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
import '../foundations/text_hand.dart';
part 'cards_table_2.dart';
part 'cards_table_3.dart';
part 'cards_table_4.dart';
part 'cards_table_5.dart';
part 'cards_table_6.dart';

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
  sway('Oscillator', Color(0xFF7DD5B1)),
  stagger('Stagger', Color(0xFF4781E5)),
  noise('Noise', Color(0xFFE974AB)),
  random('Random', Color(0xFFEFCB4E));

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

/// A thing on the stage: its cards, its shelf, what moves its values (behaviours, other things' values), its bundles.
class ThingM {
  ThingM(this.name, this.cards, {this.ring = true, this.base = Offset.zero, List<CardM>? shelf, String? short}) : shelf = shelf ?? [], short = short ?? name[0];
  final String name, short; // short: what a mark on a tile calls it
  final bool ring; // a ring of copies; else one square
  final Offset base; // where it stands on the stage
  final List<CardM> cards, shelf;
  final drives = <String, Drive>{};
  final tags = <String, Member>{}; // values wearing a tag: every value with the same tag moves by what any of them moves
  final bundles = <List<Val>>[];
}

/// Another thing's value, held: in the hand it is a card; laid on a value it moves it.
class ValRef {
  const ValRef(this.thing, this.id);
  final ThingM thing;
  final String id;
  @override
  bool operator ==(Object other) => other is ValRef && other.thing == thing && other.id == id;
  @override
  int get hashCode => Object.hash(thing, id);
}

/// A tag (a CSS variable, a name in RealitySketch): values wearing the same tag move together. It has a letter and a colour.
class Tag {
  Tag(this.n);
  final int n;
  @override
  bool operator ==(Object other) => other is Tag && other.n == n;
  @override
  int get hashCode => n;
  String get letter => String.fromCharCode(65 + n % 26);
  Color get c => HSLColor.fromAHSL(1, (40 + n * 137.5) % 360, .62, .62).toColor();
}

/// A value wearing a tag: it moves by what every other wearer moved since it put the tag on ([own0], [rest]): nothing jumps then,
/// and each keeps its own look while they move together (Para's "remember the difference").
class Member {
  Member(this.tag, this.own0, this.rest);
  final Tag tag;
  final double own0; // its own number (without the tag) when it put the tag on
  double rest; // what the others had moved then, taken off (and what a wearer leaving had moved, kept)
  Member copy() => Member(tag, own0, rest);
}

/// What is carried when a bundle's mark is picked up (a value leaving or joining a bundle, not a value card).
class Tie {
  const Tie(this.v);
  final Val v;
}

/// What the table is, kept before each change so the change can be taken back.
class _Snap {
  _Snap(CardTable t)
    : sel = t.sel,
      values = {for (final th in t.things) for (final c in [...th.cards, ...th.shelf]) for (final v in c.all) v: v.v},
      faces = {for (final th in t.things) for (final c in [...th.cards, ...th.shelf]) c: (c.flipped, c.down)},
      hand = [...t.hand],
      per = {
        for (final th in t.things)
          th: (
            cards: [...th.cards],
            shelf: [...th.shelf],
            drives: {for (final e in th.drives.entries) e.key: e.value.copy()},
            tags: {for (final e in th.tags.entries) e.key: e.value.copy()},
            bundles: [for (final b in th.bundles) [...b]],
          ),
      };
  final int sel;
  final Map<Val, double> values;
  final Map<CardM, (bool, bool)> faces;
  final List<Object> hand;
  final Map<ThingM, ({List<CardM> cards, List<CardM> shelf, Map<String, Drive> drives, Map<String, Member> tags, List<List<Val>> bundles})> per;

  static bool _list<X>(List<X> a, List<X> b) => a.length == b.length && [for (var i = 0; i < a.length; i++) i].every((i) => a[i] == b[i]);

  /// Nothing to take back between these two: an undo step that would change nothing is skipped.
  bool same(_Snap o) =>
      values.length == o.values.length && values.entries.every((e) => o.values[e.key] == e.value) &&
      faces.entries.every((e) => o.faces[e.key] == e.value) &&
      _list(hand, o.hand) &&
      per.entries.every((e) {
        final a = e.value, b = o.per[e.key];
        if (b == null) return false;
        return _list(a.cards, b.cards) && a.shelf.length == b.shelf.length && a.shelf.every(b.shelf.contains) &&
            a.drives.length == b.drives.length && a.drives.entries.every((d) { final x = b.drives[d.key]; return x != null && x.beh == d.value.beh && x.width == d.value.width && x.speed == d.value.speed && x.seed == d.value.seed && x.from == d.value.from; }) &&
            a.tags.length == b.tags.length && a.tags.entries.every((l) { final x = b.tags[l.key]; return x != null && x.tag == l.value.tag && x.own0 == l.value.own0 && x.rest == l.value.rest; }) &&
            a.bundles.length == b.bundles.length && [for (var i = 0; i < a.bundles.length; i++) i].every((i) => a.bundles[i].length == b.bundles[i].length && a.bundles[i].every(b.bundles[i].contains));
      });
}

class CardTable extends ChangeNotifier {
  CardTable(this.things, {List<Object>? hand}) : hand = hand ?? [Beh.sway, Beh.stagger, Beh.noise, Beh.random];
  final List<ThingM> things;
  int sel = 0;
  ThingM get cur => things[sel];
  final List<Object> hand; // behaviours and values of things: the cards a hand keeps across what is picked
  List<CardM> get cards => cur.cards;
  List<CardM> get shelf => cur.shelf; // cards the thing does not wear yet: thrown on the stage, they join it
  Map<String, Drive> get drives => cur.drives;
  Map<String, Member> get tags => cur.tags;
  List<List<Val>> get bundles => cur.bundles; // values tied into one: scrubbing the bundle moves them all
  (String, Beh)? preview; // a behaviour hovering over a value: shown before it is let go
  CardM? cardPreview; // a shelf card held over the stage: worn before it is let go
  String? flash, open, openTag; // the value a card was just laid on (lit once); the value whose behaviour's knobs are open; whose tag's wearers are listed
  int flashN = 0, orderN = 0; // orderN: bumped when the cards are reordered (they light in the order they are done)
  Object? carrying; // what the hand holds now: the places it may go are outlined
  final ringOpen = ValueNotifier<bool>(false); // the stage's value ring is open: the card in the air shrinks to its mark, the ring stays readable
  double clock = 0, waveClock = 0; // seconds; the wave's own, run at its speed (a speed changed never jumps the wave)
  String? editing; // the number being typed: a value's id, or w:/s:/r:/b:/k:/a: + id for a knob
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
    s.faces.forEach((c, f) {
      c.flipped = f.$1;
      c.down = f.$2;
    });
    hand
      ..clear()
      ..addAll(s.hand);
    for (final e in s.per.entries) {
      final th = e.key, p = e.value;
      for (final d in p.drives.entries) {
        final now = th.drives[d.key];
        d.value.clock = now != null && now.beh == d.value.beh ? now.clock : clock; // (what moves keeps moving: no jump back)
      }
      th.drives
        ..clear()
        ..addAll(p.drives);
      th.tags
        ..clear()
        ..addAll(p.tags);
      th.bundles
        ..clear()
        ..addAll(p.bundles);
      th.cards
        ..clear()
        ..addAll(p.cards);
      th.shelf
        ..clear()
        ..addAll(p.shelf);
    }
    sel = s.sel;
    preview = null;
    cardPreview = null;
    if (open != null && drives[open] == null) open = null;
    if (openTag != null && tags[openTag] == null) openTag = null;
    editing = null;
    _changed();
  }

  void type(String? id) {
    editing = id;
    if (id != null && valOr(id) != null) touched = id;
    _changed();
  }

  /// Another thing picked (on the stage or by its name): the inspector shows its cards; the hand stays as it is.
  void select(int i) {
    if (i == sel || i < 0 || i >= things.length) return;
    sel = i;
    open = openTag = editing = null;
    if (touched != null && valOr(touched!) == null) touched = null; // (the same value of the next thing: tapping a hand card there goes on it too)
    preview = null;
    cardPreview = null;
    _changed();
  }

  /// The clocks run on: each behaviour at its own speed, the wave at its card's.
  void advance(double dt) {
    clock += dt;
    for (final th in things) {
      for (final d in th.drives.values) {
        d.clock += dt * d.speed;
      }
    }
    final s = valOr('waveS', of: things.first);
    waveClock += dt * (s?.v ?? 1);
  }

  List<CardM> wornOf(ThingM th) => [...th.cards, if (th == cur) ?cardPreview];
  Val? valOr(String id, {ThingM? of}) {
    for (final c in wornOf(of ?? cur)) {
      for (final v in c.all) {
        if (v.id == id) return v;
      }
    }
    return null;
  }

  CardM? cardOf(String id, {ThingM? of}) {
    for (final c in wornOf(of ?? cur)) {
      if (c.all.any((v) => v.id == id)) return c;
    }
    return null;
  }

  int? bundleOf(Val v) {
    final i = bundles.indexWhere((b) => b.contains(v));
    return i < 0 ? null : i;
  }

  static bool canDrive(Val v) => v.perCopy;

  /// The value at this moment for one copy of a thing (the picked one unless [of]): its number, moved by what drives it and by
  /// what its tag's other wearers moved; a hovering card shows what letting go would do. The fallback if its card is not worn or is face down.
  double at(String id, int i, int n, {double fallback = 0, ThingM? of, int depth = 0}) {
    final th = of ?? cur, v = valOr(id, of: th), card = cardOf(id, of: th);
    if (v == null || card == null || card.down) return fallback;
    final d = th.drives[id], p = th == cur ? preview : null, span = v.hi - v.lo;
    var x = v.v;
    if (p != null && p.$1 == id) {
      x += d != null && d.beh == p.$2 ? d.at(i, n) * span * math.min(d.width + .15, 1) : p.$2.at(clock, i, n) * span * .2;
    } else if (d != null) {
      x += d.at(i, n) * span * d.width;
    }
    final m = depth == 0 ? th.tags[id] : null;
    if (m != null) x += _others(m.tag, th, id, i) - m.rest; // (by what the other wearers moved since it put the tag on)
    return v.fit(x);
  }

  /// What every other wearer of a tag has moved since it put the tag on (its own motion, without tags), for copy [i].
  double _others(Tag g, ThingM th, String id, int i) {
    var s = 0.0;
    for (final o in things) {
      for (final e in o.tags.entries) {
        if (e.value.tag != g || (o == th && e.key == id)) continue;
        final n = o.ring ? at('count', 0, 1, of: o, fallback: 1, depth: 1).round() : 1;
        s += at(e.key, i % math.max(n, 1), n, of: o, depth: 1) - e.value.own0;
      }
    }
    return s;
  }

  Tag? litTag; // the tag whose wearers are lit on the stage now (its mark tapped, or just put on), until litUntil
  double litUntil = 0;

  /// Every value wearing this tag, on every thing.
  List<(ThingM, String)> wearers(Tag g) => [
    for (final th in things)
      for (final e in th.tags.entries)
        if (e.value.tag == g) (th, e.key),
  ];

  Tag _newTag() {
    final used = {for (final th in things) for (final m in th.tags.values) m.tag.n};
    var n = 0;
    while (used.contains(n)) {
      n++;
    }
    return Tag(n);
  }

  void _wear(ThingM th, String id, Tag g) {
    th.tags[id] = Member(g, at(id, 0, 1, of: th, depth: 1), 0);
    th.tags[id]!.rest = _others(g, th, id, 0);
  }

  /// A value carried onto another thing's value: both wear one tag (the carried one's, or a new one) and move together from now on.
  void join(String id, ValRef from, {ThingM? on}) {
    final th = on ?? cur, have = from.thing.tags[from.id]?.tag;
    if ((from.thing == th && from.id == id) || valOr(id, of: th) == null || valOr(from.id, of: from.thing) == null) return;
    if (have != null && th.tags[id]?.tag == have) return;
    mark();
    if (th.tags[id] != null) _takeOff(th, id);
    final g = from.thing.tags[from.id]?.tag ?? _newTag();
    if (from.thing.tags[from.id] == null) _wear(from.thing, from.id, g);
    _wear(th, id, g);
    litTag = g;
    litUntil = clock + 1.4;
    if (th == cur) {
      flash = id;
      flashN++;
      touched = id;
    }
    _changed(keepFlash: true);
  }

  /// A value takes its tag off: it stays where it is now, and so do the others (a tag left on one value alone comes off too).
  void _takeOff(ThingM th, String id) {
    final m = th.tags[id], v = valOr(id, of: th);
    if (m == null) return;
    final moved = at(id, 0, 1, of: th, depth: 1) - m.own0;
    if (v != null) v.v = v.fit(v.v + _others(m.tag, th, id, 0) - m.rest);
    th.tags.remove(id);
    final left = wearers(m.tag);
    for (final (o, k) in left) {
      o.tags[k]!.rest -= moved; // (what it had moved stays with the others)
    }
    if (left.length == 1) _takeOff(left.first.$1, left.first.$2);
  }

  void takeOff(ThingM th, String id) {
    if (th.tags[id] == null) return;
    mark();
    _takeOff(th, id);
    if (openTag != null && tags[openTag] == null) openTag = null;
    _changed();
  }

  /// A tag's mark tapped: its wearers are listed under the card and lit on the stage.
  void showTag(String id) {
    openTag = openTag == id ? null : id;
    final m = tags[id];
    if (m != null) {
      litTag = m.tag;
      litUntil = clock + 1.4;
    }
    _changed();
  }

  void _changed({bool keepFlash = false}) {
    if (!keepFlash) flash = null;
    if (_redoAtRisk && _undo.isNotEmpty && !_undo.last.same(_Snap(this))) {
      _redo.clear();
      _redoAtRisk = false;
    }
    notifyListeners();
  }

  /// A hand card tapped: it goes on the value last touched (After Effects' "pick, then apply").
  int nopeN = 0; // a hand card tapped with nowhere to go: it shakes (nopeN counts, nope says which)
  Object? nope;
  void tapLay(Object b) {
    final v = touched == null ? null : valOr(touched!);
    final ok = v != null && cards.contains(cardOf(v.id)) && !cardOf(v.id)!.down;
    if (ok && b is Beh && canDrive(v)) return lay(v.id, b);
    nope = b;
    nopeN++;
    notifyListeners();
  }

  void markInsert((CardM, bool)? m) {
    if (m == insertMark) return;
    insertMark = m;
    notifyListeners();
  }

}

Val _px(String id, String label, [double v = 0, double lo = -400, double hi = 400]) => Val(id, label, v, lo, hi, unit: 'px');
Val _deg(String id, String label, [double v = 0]) => Val(id, label, v, 0, 360, unit: '°', wrap: true);
Val _pc(String id, String label, [double v = 100]) => Val(id, label, v, 0, 300, unit: '%');
