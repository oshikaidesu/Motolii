// The present inside (motolii/filament/motolii.h), called straight from Dart: dart:ffi on its C calls, as
// sets/glass_live.dart calls the old glass. The skin holds no document: it holds this handle and the numbers of things.
// One call = one change = one step of undo. A call that is not done returns false and [trouble] says why.
// macOS only, Debug build only (the Debug entitlements have the sandbox off so that the library can be loaded).
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';
import 'package:flutter/services.dart';

/// Where the inside lives. Override with --dart-define=MOTOLII_DIR=/path/to/motolii/filament.
const motoliiDir = String.fromEnvironment('MOTOLII_DIR', defaultValue: '/Users/motolii_maker/motolii/filament');

typedef _P = Pointer<Void>;
typedef _S = Pointer<Utf8>;

/// A film's thumbnail, made by the inside's own call on another isolate (nothing of a document is handed over: the
/// call asks nothing of one). Kept in the system's temp folder under the film's place and when it was last changed.
final _thumbnails = <String, Future<String?>>{};
Future<String?> _thumbnail(String lib, String film) async {
  final folder = Directory('${Directory.systemTemp.path}/motolii_thumbnails')..createSync(recursive: true);
  // (named by the film's own place, so it is found again in another run, and by when the film was last changed)
  final named = film.replaceAll(RegExp(r'[^A-Za-z0-9]'), '_');
  final png = '${folder.path}/${named.length > 150 ? named.substring(named.length - 150) : named}_${File(film).statSync().modified.millisecondsSinceEpoch}.png';
  if (File(png).existsSync()) return png;
  final made = await Isolate.run(() {
    final ask = DynamicLibrary.open(lib).lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>, Int32), int Function(Pointer<Utf8>, Pointer<Utf8>, int)>('motolii_thumbnail');
    final from = film.toNativeUtf8(), to = png.toNativeUtf8();
    final done = ask(from, to, 240) != 0;
    malloc
      ..free(from)
      ..free(to);
    return done;
  });
  return made ? png : null;
}

class Motolii {
  Motolii._(this._lib, this._m, this.libPath, this.home);
  final DynamicLibrary _lib;
  final _P _m;
  final String libPath;

  /// The folder the document's file words are read from (a word that is not a whole path starts here).
  final String home;

  /// A file word of the document as a whole path.
  String whole(String file) => Uri.directory(home).resolveUri(Uri.file(file)).toFilePath();

  /// The document at [document] (made if it is not there), on a stage [wide] by [high]. null: it could not be opened.
  static Motolii? open(String document, {int wide = 1920, int high = 1080, String dir = motoliiDir}) {
    final path = '$dir/out/libmotolii.dylib';
    final DynamicLibrary lib;
    try {
      lib = DynamicLibrary.open(path);
    } catch (_) {
      return null;
    }
    final home = '$dir/out/wgpu_kit'.toNativeUtf8(), doc = document.toNativeUtf8();
    lib.lookupFunction<Void Function(_S), void Function(_S)>('motolii_home')(home);
    final m = lib.lookupFunction<_P Function(_S, Int32, Int32), _P Function(_S, int, int)>('motolii_open')(doc, wide, high);
    malloc
      ..free(home)
      ..free(doc);
    return m == nullptr ? null : Motolii._(lib, m, path, '$dir/out/wgpu_kit');
  }

  late final _live = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_live');
  late final _play = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_play');
  late final _now = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_now');
  late final _at = _lib.lookupFunction<Int32 Function(_P, Double), int Function(_P, double)>('motolii_at');
  late final _seconds = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_seconds');
  late final _add = _lib.lookupFunction<Int32 Function(_P, _S, _S), int Function(_P, _S, _S)>('motolii_add');
  late final _remove = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_remove');
  late final _set = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_set');
  late final _say = _lib.lookupFunction<Int32 Function(_P, Int32, _S, _S), int Function(_P, int, _S, _S)>('motolii_say');
  late final _key = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double, Double), int Function(_P, int, _S, double, double)>('motolii_key');
  late final _unkey = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_unkey');
  late final _way = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double, _S), int Function(_P, int, _S, double, _S)>('motolii_way');
  late final _value = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double, Pointer<Double>), int Function(_P, int, _S, double, Pointer<Double>)>('motolii_value');
  late final _things = _lib.lookupFunction<Int32 Function(_P, Pointer<Int32>, Int32), int Function(_P, Pointer<Int32>, int)>('motolii_things');
  late final _word = _lib.lookupFunction<_S Function(_P, Int32, _S), _S Function(_P, int, _S)>('motolii_word');
  late final _lasts = _lib.lookupFunction<Double Function(_P, Int32), double Function(_P, int)>('motolii_lasts');
  late final _keyMoments = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Pointer<Double>, Int32), int Function(_P, int, _S, Pointer<Double>, int)>('motolii_key_moments');
  late final _frame = _lib.lookupFunction<Int32 Function(_P, Int32, Pointer<Double>), int Function(_P, int, Pointer<Double>)>('motolii_frame');
  late final _keying = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_keying');
  late final _hold = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_hold');
  late final _carry = _lib.lookupFunction<Int32 Function(_P, _S, Double, Double), int Function(_P, _S, double, double)>('motolii_carry');
  late final _letGo = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_let_go');
  late final _put = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double, Double), int Function(_P, int, _S, double, double)>('motolii_put');
  late final _keyHere = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_key_here');
  late final _size = _lib.lookupFunction<Int32 Function(_P, Int32, Int32), int Function(_P, int, int)>('motolii_size');
  late final _sizeOf = _lib.lookupFunction<Int32 Function(_P, Pointer<Int32>, Pointer<Int32>), int Function(_P, Pointer<Int32>, Pointer<Int32>)>('motolii_size_of');
  late final _raise = _lib.lookupFunction<Int32 Function(_P, Int32, Int32), int Function(_P, int, int)>('motolii_raise');
  late final _stay = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_stay');
  late final _keyed = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_keyed');
  late final _moving = _lib.lookupFunction<_S Function(_P, Int32), _S Function(_P, int)>('motolii_moving');
  late final _values = _lib.lookupFunction<_S Function(_P, Int32), _S Function(_P, int)>('motolii_values');
  late final _cards = _lib.lookupFunction<_S Function(_P, Int32), _S Function(_P, int)>('motolii_cards');
  late final _shown = _lib.lookupFunction<_S Function(_P, _S), _S Function(_P, _S)>('motolii_shown');
  late final _recipes = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_recipes');
  late final _make = _lib.lookupFunction<Int32 Function(_P, _S), int Function(_P, _S)>('motolii_make');
  late final _effects = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_effects');
  late final _spent = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_spent');
  late final _range = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Pointer<Double>, Pointer<Double>), int Function(_P, int, _S, Pointer<Double>, Pointer<Double>)>('motolii_range');
  late final _words = _lib.lookupFunction<_S Function(_P, Int32, _S), _S Function(_P, int, _S)>('motolii_words');
  late final _wayNames = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_way_names');
  late final _wayAt = _lib.lookupFunction<Double Function(_P, _S, Double), double Function(_P, _S, double)>('motolii_way_at');
  late final _kindOf = _lib.lookupFunction<_S Function(_S), _S Function(_S)>('motolii_kind_of');
  late final _together = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_together');
  late final _wayOf = _lib.lookupFunction<_S Function(_P, Int32, _S, Double), _S Function(_P, int, _S, double)>('motolii_way_of');
  late final _point = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_point');
  late final _undo = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_undo');
  late final _redo = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_redo');
  late final _trouble = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_trouble');
  late final _list = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_list');
  late final _close = _lib.lookupFunction<Void Function(_P), void Function(_P)>('motolii_close');

  T _with<T>(List<String> words, T Function(List<_S>) call) {
    final s = [for (final w in words) w.toNativeUtf8()];
    try {
      return call(s);
    } finally {
      for (final p in s) {
        malloc.free(p);
      }
    }
  }

  static const _channel = MethodChannel('motolii/stage');

  /// The stage's picture as a Flutter texture (the inside draws it by its own clock; nothing is copied). -1: none.
  Future<int> texture() async {
    if (_live(_m) == 0) return -1;
    return await _channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address}) ?? -1;
  }

  String get trouble => _trouble(_m).toDartString();
  String get list => _list(_m).toDartString();

  /// The whole document written as a film (.mp4, with its films' sound), on another isolate: the window does not wait.
  /// The inside's one hand takes the call in its turn, from whatever thread it comes.
  Future<bool> write(String mp4) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final ask = DynamicLibrary.open(lib).lookupFunction<Int32 Function(Pointer<Void>, Pointer<Utf8>), int Function(Pointer<Void>, Pointer<Utf8>)>('motolii_write');
      final to = mp4.toNativeUtf8();
      final done = ask(Pointer<Void>.fromAddress(at), to) != 0;
      malloc.free(to);
      return done;
    });
  }
  /// Of these font files, those a text can be [words] in (asked on another isolate: the window does not wait).
  Future<List<String>> setting(List<String> fonts, String words) {
    final lib = libPath;
    return Isolate.run(() {
      final ask = DynamicLibrary.open(lib).lookupFunction<Int32 Function(Pointer<Utf8>, Pointer<Utf8>), int Function(Pointer<Utf8>, Pointer<Utf8>)>('motolii_sets');
      final said = words.toNativeUtf8(), out = <String>[];
      for (final f in fonts) {
        final file = f.toNativeUtf8();
        if (ask(file, said) != 0) out.add(f);
        malloc.free(file);
      }
      malloc.free(said);
      return out;
    });
  }

  double get now => _now(_m);
  double get seconds => _seconds(_m);
  late final _rate = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_rate');

  /// How many pictures a second a film written of the document has: what a frame is.
  int get rate => _rate(_m);
  bool play(bool on) => _play(_m, on ? 1 : 0) != 0;
  bool at(double seconds) => _at(_m, seconds) != 0;

  /// A new thing (film, picture, text, solid) from a file (a solid: a file, or ring, gem or cube): its number, 0 if it was not made.
  /// What a solid is made of is its `fill`: a colour, or `glass`.
  int add(String kind, String file) => _with([kind, file], (s) => _add(_m, s[0], s[1]));
  bool remove(int thing) => _remove(_m, thing) != 0;
  bool set(int thing, String name, double number) => _with([name], (s) => _set(_m, thing, s[0], number)) != 0;
  bool say(int thing, String name, String word) => _with([name, word], (s) => _say(_m, thing, s[0], s[1])) != 0;
  bool key(int thing, String name, double at, double number) => _with([name], (s) => _key(_m, thing, s[0], at, number)) != 0;
  bool unkey(int thing, String name, double at) => _with([name], (s) => _unkey(_m, thing, s[0], at)) != 0;
  bool way(int thing, String name, double at, String way) => _with([name, way], (s) => _way(_m, thing, s[0], at, s[1])) != 0;
  double? value(int thing, String name, double at) => _with([name], (s) {
    final out = malloc<Double>();
    final ok = _value(_m, thing, s[0], at, out) != 0, v = out.value;
    malloc.free(out);
    return ok ? v : null;
  });
  /// The numbers of the things there are.
  List<int> get things {
    var room = 64;
    while (true) {
      final out = malloc<Int32>(room);
      final n = _things(_m, out, room), all = [for (var i = 0; i < n && i < room; i++) out[i]];
      malloc.free(out);
      if (n <= room) return all;
      room = n;
    }
  }

  /// A thing's word by name (kind, file, shape, blend, text, fill); '' if it has none.
  String word(int thing, String name) => _with([name], (s) => _word(_m, thing, s[0]).toDartString());

  /// How long a thing is on, in seconds; below 0: for good.
  double lasts(int thing) => _lasts(_m, thing);

  /// The moments of a value's keys (none if the value stays).
  List<double> keyMoments(int thing, String name) => _with([name], (s) {
    const room = 256;
    final out = malloc<Double>(room);
    final n = _keyMoments(_m, thing, s[0], out, room), all = [for (var i = 0; i < n && i < room; i++) out[i]];
    malloc.free(out);
    return all;
  });
  /// A thing's frame on the picture at the stage's moment: its middle (x, y), how wide, how high (points of the picture,
  /// from the top left), how far it is turned (degrees, clockwise as seen), the point it turns about (x, y), and how deep
  /// it is (0: flat). null: there is no such thing.
  List<double>? frame(int thing) {
    final out = malloc<Double>(8);
    final ok = _frame(_m, thing, out) != 0, f = [for (var i = 0; i < 8; i++) out[i]];
    malloc.free(out);
    return ok ? f : null;
  }

  /// A thing held by a hand, carried, and let go (motolii.h): the one form of everything a hand drags. [by] is
  /// 'place' (on the stage: how far the hand has gone, in points of the picture), 'bar', 'head' or 'tail' (its bar on
  /// the timeline: how many seconds), or a value's name (what the number is now). What is carried shows at once and
  /// is the document's only when it is let go and kept: one step.
  /// On: every value a hand changes or carries gets a key at that moment (the inspector's Animate all).
  void keying(bool on) => _keying(_m, on ? 1 : 0);

  /// Whether every change a hand makes is keyed now (Animate all).
  bool get keyingNow => _keying(_m, -1) != 0;
  bool hold(int thing) => _hold(_m, thing) != 0;
  bool carry(String by, double a, [double b = 0]) => _with([by], (s) => _carry(_m, s[0], a, b)) != 0;
  bool letGo(bool keep) => _letGo(_m, keep ? 1 : 0) != 0;

  /// What a hand does to a value at a moment, by the document's own rules (motolii.h): the value changed there (a key
  /// if it has keys, else the value that stays); a key there holding what it is; its keys gone; and how it is keyed
  /// at that moment (0 no keys, 1 keys but none there, 2 a key there).
  bool put(int thing, String name, double at, double number) => _with([name], (s) => _put(_m, thing, s[0], at, number)) != 0;
  bool keyHere(int thing, String name, double at) => _with([name], (s) => _keyHere(_m, thing, s[0], at)) != 0;
  bool stay(int thing, String name, double at) => _with([name], (s) => _stay(_m, thing, s[0], at)) != 0;
  int keyed(int thing, String name, double at) => _with([name], (s) => _keyed(_m, thing, s[0], at));

  /// The size of the stage the document is open on; and the size it is to be when it is next opened (close, open again).
  late final (int, int) size = () {
    final at = malloc<Int32>(2);
    _sizeOf(_m, at, at + 1);
    final is_ = (at[0], at[1]);
    malloc.free(at);
    return is_;
  }();
  bool resize(int wide, int high) => _size(_m, wide, high) != 0;
  late final _fits = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_fits');
  late final _work = _lib.lookupFunction<Int32 Function(_P, Double, Double), int Function(_P, double, double)>('motolii_work');
  late final _workOf = _lib.lookupFunction<Int32 Function(_P, Pointer<Double>, Pointer<Double>), int Function(_P, Pointer<Double>, Pointer<Double>)>('motolii_work_of');
  late final _mark = _lib.lookupFunction<Int32 Function(_P, Double, _S), int Function(_P, double, _S)>('motolii_mark');
  late final _marks = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_marks');

  /// The work area, in seconds: the stretch played round and written as a film (not said: all of it); a change is a step.
  (double, double) get work {
    final at = malloc<Double>(2);
    _workOf(_m, at, at + 1);
    final w = (at[0], at[1]);
    malloc.free(at);
    return w;
  }

  bool setWork(double from, double to) => _work(_m, from, to) != 0;
  late final _loopIs = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_loop');

  /// Whether play goes round the work area (or stops at its end).
  bool get looping => _loopIs(_m, -1) != 0;
  set looping(bool on) => _loopIs(_m, on ? 1 : 0);

  /// The marks: moments with a name, in time; a mark said with '' is taken away.
  List<(double, String)> get marks => [
    for (final line in _marks(_m).toDartString().split('\n'))
      if (line.indexOf(' ') case final i when i > 0) (double.parse(line.substring(0, i)), line.substring(i + 1)),
  ];
  bool mark(double at, String word) => _with([word], (s) => _mark(_m, at, s[0])) != 0;
  late final _guide = _lib.lookupFunction<Int32 Function(_P, _S, Double, Int32), int Function(_P, _S, double, int)>('motolii_guide');
  late final _guides = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_guides');

  /// The guides: lines across the picture, 'x' upright and 'y' level, at so many points from its left or top.
  List<(String, double)> get guides => [
    for (final line in _guides(_m).toDartString().split('\n'))
      if (line.length > 2) (line[0], double.parse(line.substring(2))),
  ];
  bool guide(String axis, double at, bool on) => _with([axis], (s) => _guide(_m, s[0], at, on ? 1 : 0)) != 0;

  /// Whether the stage is the size the document says now (not, after [resize] or an undo of one: open it again).
  bool get fits => _fits(_m) != 0;

  /// A thing brought [places] places nearer among the things (below 0: further back).
  bool raise(int thing, int places) => _raise(_m, thing, places) != 0;

  /// The names of a thing's values that take keys.
  List<String> moving(int thing) => _moving(_m, thing).toDartString().split(' ').where((n) => n.isNotEmpty).toList();

  /// The names of all a thing's values.
  List<String> values(int thing) => _values(_m, thing).toDartString().split(' ').where((n) => n.isNotEmpty).toList();

  /// Pictures in one call: each recipe's name and what it makes.
  List<(String, String)> get recipes => [
    for (final line in _recipes(_m).toDartString().split('\n'))
      if (line.indexOf('\t') case final at when at > 0) (line.substring(0, at), line.substring(at + 1)),
  ];

  /// The whole-screen effects on the shelf: each one's name and file (the stage's script is the files of those on it).
  List<(String, String)> get effects => [
    for (final line in _effects(_m).toDartString().split('\n'))
      if (line.indexOf('\t') case final at when at > 0) (line.substring(0, at), line.substring(at + 1)),
  ];

  /// What the inside's hand has spent its time on (for measuring).
  String get spent => _spent(_m).toDartString();

  /// A recipe made at the present moment, as one step: the new thing (0: not made, [trouble] says why).
  int make(String recipe) => _with([recipe], (s) => _make(_m, s[0]));

  /// The cards a maker sees a thing's values on, besides where it is and its fill: each a title and the values on it.
  List<(String, List<String>)> cards(int thing) => [
    for (final c in _cards(_m, thing).toDartString().split('; '))
      if (c.indexOf(': ') case final at when at > 0) (c.substring(0, at), c.substring(at + 2).split(' ')),
  ];

  /// How a value is shown to a hand, as the inside says (null: as its range says): its look, the unit it is read in,
  /// its name in plain words, the span its knob covers (in the document's units) and the fold it lies in.
  Shown? shown(String name) => _shownOf.putIfAbsent(name, () {
    final f = _with([name], (s) => _shown(_m, s[0]).toDartString()).split('\t');
    return f.length < 6 ? null : Shown(f[0], f[1], f[2], double.parse(f[3]), double.parse(f[4]), f[5].isEmpty ? null : f[5]);
  });
  final _shownOf = <String, Shown?>{};

  /// A number value's lowest and highest (null: the thing has no such number).
  (double, double)? range(int thing, String name) {
    final lo = malloc<Double>(), hi = malloc<Double>();
    final ok = _with([name], (s) => _range(_m, thing, s[0], lo, hi)) != 0, r = (lo.value, hi.value);
    malloc
      ..free(lo)
      ..free(hi);
    return ok ? r : null;
  }

  /// The words a value of a thing may be (a blend's): none if any word will do, or the thing has no such value.
  List<String> words(int thing, String name) => _with([name], (s) => _words(_m, thing, s[0]).toDartString()).split(' ').where((w) => w.isNotEmpty).toList();

  /// A film's first picture, small, as a .png file (null: none could be made). Made once and on another isolate: the
  /// window does not wait.
  Future<String?> thumbnail(String film) => _thumbnails[film] ??= _thumbnail(libPath, film);

  /// The names of the ways a stretch may go, and how one of the nine kinds goes at [u] of the way through (0 to 1).
  List<String> get wayNames => _wayNames(_m).toDartString().split(' ').where((w) => w.isNotEmpty).toList();
  double wayAt(String way, double u) => _with([way], (s) => _wayAt(_m, s[0], u));

  /// What kind of thing a file makes ('': none).
  String kindOf(String file) => _with([file], (s) => _kindOf(s[0]).toDartString());

  /// One act of a hand that is several changes: they are one step of undo.
  T together<T>(T Function() changes) {
    _together(_m, 1);
    try {
      return changes();
    } finally {
      _together(_m, 0);
    }
  }

  /// The way of the stretch that begins at a value's key, as it is said ('': no key there).
  String wayOf(int thing, String name, double at) => _with([name], (s) => _wayOf(_m, thing, s[0], at).toDartString());

  /// Points of the picture to one unit of the stage, at z 0.
  late final double point = _point(_m);
  bool undo() => _undo(_m) != 0;
  bool redo() => _redo(_m) != 0;
  void close() => _close(_m);
}

/// How a value is shown to a hand (Motolii.shown): look, unit, plain name, knob span, folded away.
class Shown {
  const Shown(this.look, this.unit, this.label, this.low, this.high, this.fold);
  final String look, unit, label;
  final double low, high;

  /// The fold it lies in on its card, named by what the fold holds (null: on the card's first row).
  final String? fold;
}
