// Part of cards_table.dart: CardTable's calls as they were, moved out of its body by size into an extension on it in
// the same library (its private names and its callers as before).
part of 'cards_table.dart';

extension CardTableCalls on CardTable {
  void drop(Object card) {
    if (card is Beh) return; // (the behaviours stay)
    mark();
    hand.remove(card);
    _changed();
  }

  String nameOf(ThingM th, String id) => '${th.name} · ${valOr(id, of: th)?.label ?? id}';

  void set(Val v, double x) {
    v.v = v.fit(x);
    touched = v.id;
    _changed();
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
    final a = cards.indexWhere((c) => c.name == 'Copies'), b = cards.indexWhere((c) => c.name == 'Wave');
    if (a < 0 || b < 0) return;
    mark();
    final ca = cards[a];
    cards[a] = cards[b];
    cards[b] = ca;
    orderN++;
    _changed();
  }

  /// Which comes first, the copies or the wave: wave first = the ring waves as one; copies first = each copy waves on its own.
  bool get copiesFirst => copiesFirstOf(cur);

  bool copiesFirstOf(ThingM th) {
    final a = th.cards.indexWhere((c) => c.name == 'Copies'), b = th.cards.indexWhere((c) => c.name == 'Wave');
    return a < 0 || b < 0 || a < b;
  }
}
