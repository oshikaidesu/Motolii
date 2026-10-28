// motolii_writes.dart: changes told to the inside (skin/BRIDGE.md W): thrown, not waited for, or done in one call; the hand's carrying, keys, ways, undo.
part of 'motolii.dart';

extension MotoliiWrites on Motolii {
  /// The desk: the stage pane's whole picture, [w] x [h] pixels, the stage lying at [x], [y] (its top left) [scale]
  /// pixels a point, what lies off it seen round it, the picked thing marked over it ([point]: pixels a point of the
  /// screen) (later: desk). [w] 0: off, the stage's own picture again.
  int desk(int w, int h, double x, double y, double scale, double point) => later('desk', word: '$w $h $x $y $scale $point');   // (a ticket: [answer] says when the desk's picture exists)

  /// The thing picked in the window and the one under its hand (0: none): the desk marks them (later: pick).
  void pick(int thing, int hover) => later('pick', thing: thing, number: hover.toDouble());

  /// The things the window shows the moving values of (Anthem's subscription): their values are in the moment's picture
  /// from the next (later: watch). Told again only when they change.
  void subscribe(Set<int> things) {
    if (setEquals(things, _watched)) return;
    _watched = things;
    later('watch', word: things.join(' '));
  }

  /// The stage as it is now, as a picture (.png): for a hand that sees only pictures (test/agent_window_test.dart).
  bool still(String png) => _with([png], (s) => _still(_m, s[0])) != 0;

  /// The document and the files its things are made of copied into [folder] (the document as [name].motolii), to be
  /// taken elsewhere whole: how many files (-1: not).
  int pack(String folder, String name) => _with([folder, name], (s) => _b.motolii_pack(_m, s[0], s[1]));

  bool play(bool on) => _play(_m, on ? 1 : 0) != 0;

  bool at(double seconds) => _at(_m, seconds) != 0;

  /// A new thing (film, picture, text, solid) from a file (a solid: a file, or ring, gem or cube): its number, 0 if it was not made.
  /// What a solid is made of is its `fill`: a colour, or `glass`.
  int add(String kind, String file) => _took(_with([kind, file], (s) => _add(_m, s[0], s[1])));

  bool remove(int thing) => _took(_remove(_m, thing) != 0);

  bool set(int thing, String name, double number) => _took(_with([name], (s) => _set(_m, thing, s[0], number)) != 0);

  bool say(int thing, String name, String word) => _took(_with([name, word], (s) => _say(_m, thing, s[0], s[1])) != 0);

  bool key(int thing, String name, double at, double number) => _took(_with([name], (s) => _key(_m, thing, s[0], at, number)) != 0);

  /// Many keys of one value in one call and one step (a baked motion, beats): [at] and [numbers] side by side.
  bool keyMany(int thing, String name, List<double> at, List<double> numbers) {
    final n = at.length < numbers.length ? at.length : numbers.length, a = malloc<Double>(n < 1 ? 1 : n), v = malloc<Double>(n < 1 ? 1 : n);
    for (var i = 0; i < n; i++) {
      a[i] = at[i];
      v[i] = numbers[i];
    }
    final ok = _with([name], (s) => _keyMany(_m, thing, s[0], n, a, v)) != 0;
    malloc
      ..free(a)
      ..free(v);
    return _took(ok);
  }

  /// A MIDI file's notes as keys of a value, as om_midi keys them (motolii.h): [kind] velocity, pitch, count, on or
  /// bool; [channel] -1 any; [bpm] the file's beats played at it (0: the file's own). Its keys before go; one step.
  bool midi(int thing, String name, String file, String kind, int channel, double bpm) =>
      _took(_with([name, file, kind], (s) => _midi(_m, thing, s[0], s[1], s[2], channel, bpm)) != 0);

  /// The BPM a MIDI file says (120 if it says none): om_midi's BPM box as it first is.
  double midiBpm(String file) => _with([file], (s) => _midiBpm(s[0]));

  bool unkey(int thing, String name, double at) => _took(_with([name], (s) => _unkey(_m, thing, s[0], at)) != 0);

  bool way(int thing, String name, double at, String way) => _took(_with([name, way], (s) => _way(_m, thing, s[0], at, s[1])) != 0);

  /// A thing held by a hand, carried, and let go (motolii.h): the one form of everything a hand drags. [by] is
  /// 'place' (on the stage: how far the hand has gone, in points of the picture), 'bar', 'head' or 'tail' (its bar on
  /// the timeline: how many seconds), or a value's name (what the number is now). What is carried shows at once and
  /// is the document's only when it is let go and kept: one step.
  /// On: every value a hand changes or carries gets a key at that moment (the inspector's Animate all).
  void keying(bool on) => _keying(_m, on ? 1 : 0);

  /// Whether every change a hand makes is keyed now (Animate all).
  bool get keyingNow => _keying(_m, -1) != 0;

  bool hold(int thing) => _took(_hold(_m, thing) != 0);

  /// A hand takes hold, without waiting (the window's way: refused, its let go says why).
  void take(int thing) => later('hold', thing: thing);

  bool carry(String by, double a, [double b = 0]) => _with([by], (s) => _carry(_m, s[0], a, b)) != 0;   // (sent, not waited for: what it carries shows in the moment's picture)

  bool letGo(bool keep) => _took(_letGo(_m, keep ? 1 : 0) != 0);

  /// What a hand does to a value at a moment, by the document's own rules (motolii.h): the value changed there (a key
  /// if it has keys, else the value that stays); a key there holding what it is; its keys gone; and how it is keyed
  /// at that moment (0 no keys, 1 keys but none there, 2 a key there).
  bool put(int thing, String name, double at, double number) => _took(_with([name], (s) => _put(_m, thing, s[0], at, number)) != 0);

  bool keyHere(int thing, String name, double at) => _took(_with([name], (s) => _keyHere(_m, thing, s[0], at)) != 0);

  bool stay(int thing, String name, double at) => _took(_with([name], (s) => _stay(_m, thing, s[0], at)) != 0);

  bool resize(int wide, int high) => _took(_size(_m, wide, high) != 0);

  /// Sequence Layers: the things that have a length and stand on their own, from the front, one after another
  /// ([overlap] seconds over each other), on frames, their keys with them; one step ([trouble] says why not).
  bool sequence(double overlap) => _took(_sequence(_m, overlap) != 0);

  bool setWork(double from, double to) => _took(_work(_m, from, to) != 0);

  /// Whether play goes round the work area (or stops at its end).
  bool get looping => _loopIs(_m, -1) != 0;

  set looping(bool on) => _loopIs(_m, on ? 1 : 0);

  bool mark(double at, String word) => _took(_with([word], (s) => _mark(_m, at, s[0])) != 0);

  bool guide(String axis, double at, bool on) => _took(_with([axis], (s) => _guide(_m, s[0], at, on ? 1 : 0)) != 0);

  /// A thing brought [places] places nearer among the things (below 0: further back).
  bool raise(int thing, int places) => _took(_raise(_m, thing, places) != 0);

  /// A recipe made at the present moment, as one step: the new thing (0: not made, [trouble] says why).
  int make(String recipe) => _took(_with([recipe], (s) => _make(_m, s[0])));

  /// The names of the ways a stretch may go, and how one of the nine kinds goes at [u] of the way through (0 to 1).
  /// A way of this document's own, by name: how far it is (`a` the one key's value, `b` the next's, `u` from 0 to 1
  /// through the stretch, `p1`..`p4` its numbers; `bezier(u, x1, y1, x2, y2)` an ease by its two handles). It is then
  /// one of [wayNames], a tile of the Ease desk, and saved with the document.
  bool define(String way, String expression) => _took(_with([way, expression], (s) => _define(_m, s[0], s[1])) != 0);

  /// An effect put on a thing at a moment, or taken off if it is on (its keys too), one step.
  bool effect(int thing, String name, double at) => _took(_with([name], (s) => _effect(_m, thing, s[0], at)) != 0);

  /// One act of a hand that is several changes: they are one step of undo.
  T together<T>(T Function() changes) {
    _together(_m, 1);
    try {
      return changes();
    } finally {
      _together(_m, 0);
    }
  }

  /// A change asked without waiting for the inside's hand (say, set, let_go, undo, redo, together): queued in order behind
  /// every other, its ticket said at once ([answer] says when it is done).
  int later(String call, {int thing = 0, String name = '', String word = '', double number = 0}) => _with([call, name, word], (s) => _later(_m, s[0], thing, s[1], s[2], number));

  bool undo() => _took(_undo(_m) != 0);

  bool redo() => _took(_redo(_m) != 0);
}
