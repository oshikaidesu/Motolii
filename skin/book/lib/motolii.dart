// The present inside (motolii/filament/motolii.h), called straight from Dart: dart:ffi on its C calls, as
// sets/glass_live.dart calls the old glass. The skin holds no document: it holds this handle and the numbers of things.
// One call = one change = one step of undo. A call that is not done returns false and [trouble] says why.
// macOS only, Debug build only (the Debug entitlements have the sandbox off so that the library can be loaded).
import 'dart:math' as math;
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
  late final _solo = _lib.lookupFunction<Int32 Function(_P, Int32, Int32), int Function(_P, int, int)>('motolii_solo');
  late final _desk = _lib.lookupFunction<Int32 Function(_P, Int32, Int32, Double, Double, Double, Double), int Function(_P, int, int, double, double, double, double)>('motolii_desk');
  late final _pick = _lib.lookupFunction<Int32 Function(_P, Int32, Int32), int Function(_P, int, int)>('motolii_pick');
  late final _play = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_play');
  late final _now = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_now');
  late final _drawnAt = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_drawn_at');
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
  late final _seen = _lib.lookupFunction<Int32 Function(_P, Int32, Pointer<Double>), int Function(_P, int, Pointer<Double>)>('motolii_seen');
  late final _path = _lib.lookupFunction<Int32 Function(_P, Int32, Int32, Double, Double, Int32, Pointer<Double>), int Function(_P, int, int, double, double, int, Pointer<Double>)>('motolii_path');
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
  late final _animators = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_animators');
  late final _spent = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_spent');
  late final _range = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Pointer<Double>, Pointer<Double>), int Function(_P, int, _S, Pointer<Double>, Pointer<Double>)>('motolii_range');
  late final _words = _lib.lookupFunction<_S Function(_P, Int32, _S), _S Function(_P, int, _S)>('motolii_words');
  late final _ways = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_ways');
  late final _define = _lib.lookupFunction<Int32 Function(_P, _S, _S), int Function(_P, _S, _S)>('motolii_define');
  late final _wayNames = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_way_names');
  late final _wayNumbers = _lib.lookupFunction<_S Function(_P, _S), _S Function(_P, _S)>('motolii_way_numbers');
  late final _effectNames = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_effect_names');
  late final _effect = _lib.lookupFunction<Int32 Function(_P, Int32, _S, Double), int Function(_P, int, _S, double)>('motolii_effect');
  late final _wayAt = _lib.lookupFunction<Double Function(_P, _S, Double), double Function(_P, _S, double)>('motolii_way_at');
  late final _kindOf = _lib.lookupFunction<_S Function(_S), _S Function(_S)>('motolii_kind_of');
  late final _together = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_together');
  late final _frames = _lib.lookupFunction<Uint64 Function(_P), int Function(_P)>('motolii_frames');
  late final _wayOf = _lib.lookupFunction<_S Function(_P, Int32, _S, Double), _S Function(_P, int, _S, double)>('motolii_way_of');
  late final _point = _lib.lookupFunction<Double Function(_P), double Function(_P)>('motolii_point');
  late final _reading = _lib.lookupFunction<_S Function(_P, Int32, _S), _S Function(_P, int, _S)>('motolii_reading');
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

  /// A card's small picture of one thing alone ([side] points square, the eye on its frame), as a Flutter texture drawn
  /// with the stage's own pictures (motolii_solo). Shown while some card asks for it ([soloShow]); none drawn otherwise.
  /// -1: none.
  Future<int> soloTexture(int thing, int side) async {
    if (_live(_m) == 0 || _solo(_m, thing, side) == 0) return -1;
    return _soloId ??= await _channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address, 'front': 'motolii_solo_front', 'frames': 'motolii_solo_frames'}) ?? -1;
  }

  int? _soloId;

  /// The desk: the stage pane's whole picture, [w] x [h] pixels, the stage lying at [x], [y] (its top left) [scale]
  /// pixels a point, what lies off it seen round it, the picked thing marked over it ([point]: pixels a point of the
  /// screen) (motolii_desk). [w] 0: off, the stage's own picture again.
  bool desk(int w, int h, double x, double y, double scale, double point) => _desk(_m, w, h, x, y, scale, point) != 0;

  /// The thing picked in the window and the one under its hand (0: none): the desk marks them (motolii_pick).
  void pick(int thing, int hover) => _pick(_m, thing, hover);

  /// The desk's picture as a Flutter texture (-1: none).
  Future<int> deskTexture() async => _deskId ??= await _channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address, 'front': 'motolii_desk_front', 'frames': 'motolii_desk_frames'}) ?? -1;
  int? _deskId;
  int _soloAsked = 0;

  /// One card more (or, [on] false, fewer) shows [thing] alone: drawn while any does.
  void soloShow(int thing, int side, {bool on = true}) {
    _soloAsked = math.max(0, _soloAsked + (on ? 1 : -1));
    _solo(_m, _soloOf = _soloAsked > 0 ? thing : 0, side);
  }

  /// The thing picked now: the one shown alone, while a card shows one.
  void soloFollow(int thing) {
    if (_soloAsked > 0 && thing != _soloOf) _solo(_m, _soloOf = thing, 0);
  }

  int _soloOf = 0;

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

  /// The moment of the picture the stage last drew, at a frame: where a playhead is while it plays (what is read there
  /// is set down, not waited for).
  double get drawnAt => _drawnAt(_m);
  double get seconds => _keptCarried('seconds', () => _seconds(_m));
  late final _rate = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_rate');

  /// How many pictures a second a film written of the document has: what a frame is.
  int get rate => _rate(_m);
  bool play(bool on) => _play(_m, on ? 1 : 0) != 0;
  int get frames => _frames(_m); // (how many pictures the live stage has drawn: motolii_frames)
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
  double? value(int thing, String name, double at) => moving(thing).contains(name)
      ? _asked(('value', thing, name), _atMoment, _atMoment, () => _valueAt(thing, name, at))   // (at a moment: the hand busy, what it was a picture ago, asked again)
      : _keptCarried(('value', thing, name), () => _valueAt(thing, name, at));
  final _atMoment = <Object, Object?>{};   // (each moving value's last reading)   // (a value that takes no keys is the same at every moment: kept in the copy)
  double? _valueAt(int thing, String name, double at) => _with([name], (s) {
    final out = malloc<Double>();
    final ok = _value(_m, thing, s[0], at, out) != 0, v = out.value;
    malloc.free(out);
    return ok ? v : null;
  });
  /// The numbers of the things there are.
  List<int> get things => _kept('things', _thingsNow);
  List<int> _thingsNow() {
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
  String word(int thing, String name) => _kept(('word', thing, name), () => _with([name], (s) => _word(_m, thing, s[0]).toDartString()));

  /// How long a thing is on, in seconds; below 0: for good.
  double lasts(int thing) => _keptCarried(('lasts', thing), () => _lasts(_m, thing));

  late final _thingKeys = _lib.lookupFunction<Int32 Function(_P, Int32, Pointer<Double>, Int32), int Function(_P, int, Pointer<Double>, int)>('motolii_thing_key_moments');

  /// Every moment a key of any of a thing's values stands at, each once, in time: one call (a timeline bar's marks).
  List<double> thingKeyMoments(int thing) => _kept(('thingKeys', thing), () => _thingKeyMomentsNow(thing));
  List<double> _thingKeyMomentsNow(int thing) {
    var room = 256;
    for (;;) {
      final out = malloc<Double>(room);
      final n = _thingKeys(_m, thing, out, room);
      final all = [for (var i = 0; i < n && i < room; i++) out[i]];
      malloc.free(out);
      if (n <= room) return all;
      room = n;
    }
  }

  /// The moments of a value's keys (none if the value stays): how many keys it has is their count.
  List<double> keyMoments(int thing, String name) => _kept(('keys', thing, name), () => _with([name], (s) {
    const room = 256;
    final out = malloc<Double>(room);
    final n = _keyMoments(_m, thing, s[0], out, room), all = [for (var i = 0; i < n && i < room; i++) out[i]];
    malloc.free(out);
    return all;
  }));
  /// A thing's frame on the picture at the stage's moment: its middle (x, y), how wide, how high (points of the picture,
  /// from the top left), how far it is turned (degrees, clockwise as seen), the point it turns about (x, y), and how deep
  /// it is (0: flat). null: there is no such thing.
  List<double>? frame(int thing) {
    final out = malloc<Double>(8);
    final ok = _frame(_m, thing, out) != 0, f = [for (var i = 0; i < 8; i++) out[i]];
    malloc.free(out);
    return ok ? f : null;
  }

  /// What of a thing is seen: the upright box of its outline on the picture (points from the top left). null: not shown.
  Rect? seen(int thing) {
    final out = malloc<Double>(4);
    final ok = _seen(_m, thing, out) != 0, r = Rect.fromLTRB(out[0], out[1], out[2], out[3]);
    malloc.free(out);
    return ok ? r : null;
  }

  /// How a thing moves over a while, as the stage places it (its wave and all): [n] moments from [from] to before [to],
  /// four numbers each: where it is (x, y, units of the stage), how far turned (degrees), how large (a share). null: no
  /// such thing. Asked once when its values change, for a small preview that then plays by itself (motolii_path).
  List<double>? path(int thing, {int copy = 0, double from = 0, required double to, int n = 60}) {
    final out = malloc<Double>(4 * n);
    final got = _path(_m, thing, copy, from, to, n, out), p = [for (var i = 0; i < 4 * got; i++) out[i]];
    malloc.free(out);
    return got > 0 ? p : null;
  }

  /// A thing held by a hand, carried, and let go (motolii.h): the one form of everything a hand drags. [by] is
  /// 'place' (on the stage: how far the hand has gone, in points of the picture), 'bar', 'head' or 'tail' (its bar on
  /// the timeline: how many seconds), or a value's name (what the number is now). What is carried shows at once and
  /// is the document's only when it is let go and kept: one step.
  /// On: every value a hand changes or carries gets a key at that moment (the inspector's Animate all).
  void keying(bool on) => _keying(_m, on ? 1 : 0);

  /// Whether every change a hand makes is keyed now (Animate all).
  bool get keyingNow => _keying(_m, -1) != 0;
  late final _busy = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_busy');
  late final _changes = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_changes');
  late final _shaping = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_shaping');

  /// What the document's steps changed since last asked (asking forgets them): whether a thing came or went, and each
  /// value changed as (thing, name), from each step's own changeset.
  ({bool things, Set<(int, String)> values}) changes() {
    final lines = _changes(_m).toDartString().split('\n');
    return (
      things: lines.contains('things'),
      values: {
        for (final l in lines)
          if (l.indexOf(' ') case final i when i > 0) (int.parse(l.substring(0, i)), l.substring(i + 1)),
      },
    );
  }

  /// The names of the values that change what cards a thing has (and its name): the inside's own rule, asked once.
  late final Set<String> shaping = _shaping(_m).toDartString().split(' ').toSet();

  /// Whether the inside is telling the stage a moment just now: a reading made now would wait for it (asked at once).
  bool get busy => _busy(_m) != 0;
  bool hold(int thing) => _hold(_m, thing) != 0;

  /// A hand takes hold, without waiting (the window's way: refused, its let go says why).
  void take(int thing) => later('hold', thing: thing);
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
  late final _sequence = _lib.lookupFunction<Int32 Function(_P, Double), int Function(_P, double)>('motolii_sequence');

  /// Sequence Layers: the things that have a length and stand on their own, from the front, one after another
  /// ([overlap] seconds over each other), on frames, their keys with them; one step ([trouble] says why not).
  bool sequence(double overlap) => _sequence(_m, overlap) != 0;
  late final _marks = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_marks');

  /// The work area, in seconds: the stretch played round and written as a film (not said: all of it); a change is a step.
  (double, double) get work => _kept('work', _workNow);
  (double, double) _workNow() {
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
  List<(double, String)> get marks => _kept('marks', () => [
    for (final line in _marks(_m).toDartString().split('\n'))
      if (line.indexOf(' ') case final i when i > 0) (double.parse(line.substring(0, i)), line.substring(i + 1)),
  ]);
  bool mark(double at, String word) => _with([word], (s) => _mark(_m, at, s[0])) != 0;
  late final _strikes = _lib.lookupFunction<Int32 Function(_P, Int32, Double, Int32, Pointer<Double>, Int32), int Function(_P, int, double, int, Pointer<Double>, int)>('motolii_strikes');

  /// A film's sound: its loudness (dB, 0 at full) 100 times a second of its own (none: no sound, or not a film). Read
  /// on another isolate, the file decoded once: the window does not wait.
  Future<List<double>> loudness(int thing) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final ask = DynamicLibrary.open(lib).lookupFunction<Int32 Function(Pointer<Void>, Int32, Pointer<Float>, Int32), int Function(Pointer<Void>, int, Pointer<Float>, int)>('motolii_loudness');
      final m = Pointer<Void>.fromAddress(at), n = ask(m, thing, nullptr, 0), out = malloc<Float>(n < 1 ? 1 : n);
      final got = ask(m, thing, out, n), all = [for (var i = 0; i < got && i < n; i++) out[i].toDouble()];
      malloc.free(out);
      return all;
    });
  }

  late final _loudnessOver = _lib.lookupFunction<Int32 Function(_P, Int32, Double, Double, Int32, Pointer<Float>), int Function(_P, int, double, double, int, Pointer<Float>)>('motolii_loudness_over');

  /// How loud a film's sound is at [n] moments of the document from [from] to before [to], where its film shows them
  /// (dB; -100 silent or not shown). null: its sound not read yet ([loudness] reads it); empty: no sound.
  List<double>? loudnessOver(int thing, double from, double to, int n) {
    final out = malloc<Float>(n < 1 ? 1 : n), got = _loudnessOver(_m, thing, from, to, n, out);
    final all = got < 0 ? null : [for (var i = 0; i < got; i++) out[i].toDouble()];
    malloc.free(out);
    return all;
  }

  /// Where a film's sound strikes (grows [rise] dB louder at once): the document's moments where it shows them, or
  /// [own] the film's own seconds. Asked once its loudness has been read ([loudness]).
  List<double> strikes(int thing, double rise, {bool own = false}) {
    final n = _strikes(_m, thing, rise, own ? 1 : 0, nullptr, 0), out = malloc<Double>(n < 1 ? 1 : n);
    final got = _strikes(_m, thing, rise, own ? 1 : 0, out, n), all = [for (var i = 0; i < got && i < n; i++) out[i]];
    malloc.free(out);
    return all;
  }
  late final _guide = _lib.lookupFunction<Int32 Function(_P, _S, Double, Int32), int Function(_P, _S, double, int)>('motolii_guide');
  late final _guides = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_guides');

  /// The guides: lines across the picture, 'x' upright and 'y' level, at so many points from its left or top.
  List<(String, double)> get guides => _kept('guides', () => [
    for (final line in _guides(_m).toDartString().split('\n'))
      if (line.length > 2) (line[0], double.parse(line.substring(2))),
  ]);
  bool guide(String axis, double at, bool on) => _with([axis], (s) => _guide(_m, s[0], at, on ? 1 : 0)) != 0;

  /// Whether the stage is the size the document says now (not, after [resize] or an undo of one: open it again).
  bool get fits => _kept('fits', () => _fits(_m) != 0);

  /// A thing brought [places] places nearer among the things (below 0: further back).
  bool raise(int thing, int places) => _raise(_m, thing, places) != 0;

  /// The names of a thing's values that take keys.
  List<String> moving(int thing) => _kept(('moving', thing), () => _moving(_m, thing).toDartString().split(' ').where((n) => n.isNotEmpty).toList());

  /// The names of all a thing's values.
  List<String> values(int thing) => _kept(('values', thing), () => _values(_m, thing).toDartString().split(' ').where((n) => n.isNotEmpty).toList());

  /// Pictures in one call: each recipe's name and what it makes.
  List<(String, String)> get recipes => _kept('recipes', () => [
    for (final line in _recipes(_m).toDartString().split('\n'))
      if (line.indexOf('\t') case final at when at > 0) (line.substring(0, at), line.substring(at + 1)),
  ]);

  /// The whole-screen effects on the shelf: each one's name and file (the stage's script is the files of those on it).
  List<(String, String)> get effects => _kept('effects', () => [
    for (final line in _effects(_m).toDartString().split('\n'))
      if (line.indexOf('\t') case final at when at > 0) (line.substring(0, at), line.substring(at + 1)),
  ]);

  /// The motions a thing may be given (Cavalry's Animators): each one's name and file (a thing's script is the file of the one it has).
  List<(String, String)> get animators => _kept('animators', () => [
    for (final line in _animators(_m).toDartString().split('\n'))
      if (line.indexOf('\t') case final at when at > 0) (line.substring(0, at), line.substring(at + 1)),
  ]);

  /// What the inside's hand has spent its time on (for measuring).
  String get spent => _spent(_m).toDartString();

  /// A recipe made at the present moment, as one step: the new thing (0: not made, [trouble] says why).
  int make(String recipe) => _with([recipe], (s) => _make(_m, s[0]));

  /// The cards a maker sees a thing's values on, besides where it is and its fill: each a title and the values on it.
  List<(String, List<String>)> cards(int thing) => _kept(('cards', thing), () => [
    for (final c in _cards(_m, thing).toDartString().split('; '))
      if (c.indexOf(': ') case final at when at > 0) (c.substring(0, at), c.substring(at + 2).split(' ')),
  ]);

  /// How a value is shown to a hand, as the inside says (null: as its range says): its look, the unit it is read in,
  /// its name in plain words, the span its knob covers (in the document's units) and the fold it lies in.
  Shown? shown(String name) => _shownOf.putIfAbsent(name, () {
    final f = _with([name], (s) => _shown(_m, s[0]).toDartString()).split('\t');
    return f.length < 6 ? null : Shown(f[0], f[1], f[2], double.parse(f[3]), double.parse(f[4]), f[5].isEmpty ? null : f[5]);
  });
  final _shownOf = <String, Shown?>{};

  /// A number value's lowest and highest (null: the thing has no such number).
  (double, double)? range(int thing, String name) => _kept(('range', thing, name), () => _rangeNow(thing, name));
  (double, double)? _rangeNow(int thing, String name) {
    final lo = malloc<Double>(), hi = malloc<Double>();
    final ok = _with([name], (s) => _range(_m, thing, s[0], lo, hi)) != 0, r = (lo.value, hi.value);
    malloc
      ..free(lo)
      ..free(hi);
    return ok ? r : null;
  }

  /// The words a value of a thing may be (a blend's): none if any word will do, or the thing has no such value.
  List<String> words(int thing, String name) => _kept(('words', thing, name), () => _with([name], (s) => _words(_m, thing, s[0]).toDartString()).split(' ').where((w) => w.isNotEmpty).toList());

  /// The words a value may be, kept till the document is read again ([forget]): what a value may be changes with the
  /// document (a thing's kind, a script's select), not with each picture's reading of the inspector.
  void forget() {
    _copy.clear();
    _copyCarried.clear();
  }

  /// The window's copy of what it read of the document, kept while the document stays as it was: its revision, read
  /// without waiting for the inside (motolii_revision). A read the copy holds is never asked again.
  late final _revision = _lib.lookupFunction<Int64 Function(_P), int Function(_P)>('motolii_revision', isLeaf: true);
  late final _carried = _lib.lookupFunction<Int64 Function(_P), int Function(_P)>('motolii_carried', isLeaf: true);
  final _copy = <Object, Object?>{}, _copyCarried = <Object, Object?>{}, _was = <Object, Object?>{}, _wasCarried = <Object, Object?>{};
  int _copyAt = -1, _carriedAt = -1;
  T _kept<T>(Object key, T Function() ask) {
    final r = _revision(_m);
    if (r != _copyAt) {
      _was.addAll(_copy);   // (what was read before: shown while the hand is too busy to be asked)
      _copy.clear();
      _wasCarried.addAll(_copyCarried);
      _copyCarried.clear();
      _copyAt = r;
    }
    if (_copy.containsKey(key)) return _copy[key] as T;
    return _asked(key, _copy, _was, ask);
  }

  /// And what a hand's carrying changes too (a value that takes no keys, how long a thing lasts): kept while neither moves.
  T _keptCarried<T>(Object key, T Function() ask) {
    _kept('', () => null);   // (the document's revision looked at first)
    final c = _carried(_m);
    if (c != _carriedAt) {
      _wasCarried.addAll(_copyCarried);
      _copyCarried.clear();
      _carriedAt = c;
    }
    if (_copyCarried.containsKey(key)) return _copyCarried[key] as T;
    return _asked(key, _copyCarried, _wasCarried, ask);
  }

  // Asked without waiting (motolii_dont_wait): the hand busy with the document, what was read before is shown and it is
  // asked again a picture later ([again]); never read before, waited for, once.
  late final _dontWait = _lib.lookupFunction<Void Function(_P, Int32), void Function(_P, int)>('motolii_dont_wait', isLeaf: true);
  late final _refused = _lib.lookupFunction<Int32 Function(_P), int Function(_P)>('motolii_refused', isLeaf: true);
  T _asked<T>(Object key, Map<Object, Object?> into, Map<Object, Object?> was, T Function() ask) {
    _dontWait(_m, 1);
    final v = ask();
    _dontWait(_m, 0);
    if (_refused(_m) == 0) return into[key] = v;
    if (was.containsKey(key)) {
      again?.call();
      return was[key] as T;
    }
    return into[key] = ask();
  }

  /// What the window does when a read was put off (the hand busy): ask again a picture later.
  void Function()? again;

  /// The window's thread watched: while on, a change it waits for, or a read that waits on the document over 2 ms, is
  /// kept; [broken] says each, one to a line ('' none). Turned on, what was kept is forgotten.
  late final _watch = _lib.lookupFunction<Void Function(_P, Int32), void Function(_P, int)>('motolii_watch');
  late final _broken = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_broken');
  void watch(bool on) => _watch(_m, on ? 1 : 0);
  String get broken => _broken(_m).toDartString();

  /// A film's first picture, small, as a .png file (null: none could be made). Made once and on another isolate: the
  /// window does not wait.
  Future<String?> thumbnail(String film) => _thumbnails[film] ??= _thumbnail(libPath, film);

  /// The names of the ways a stretch may go, and how one of the nine kinds goes at [u] of the way through (0 to 1).
  /// A way of this document's own, by name: how far it is (`a` the one key's value, `b` the next's, `u` from 0 to 1
  /// through the stretch, `p1`..`p4` its numbers; `bezier(u, x1, y1, x2, y2)` an ease by its two handles). It is then
  /// one of [wayNames], a tile of the Ease desk, and saved with the document.
  bool define(String way, String expression) => _with([way, expression], (s) => _define(_m, s[0], s[1])) != 0;
  /// The ways a stretch may go, as the document says them: the nine kinds with their numbers, the named ones, and
  /// each defined here ('; defined here: Snappy').
  String get ways => _kept('ways', () => _ways(_m).toDartString());

  /// The effects a thing may be given, by name, with the value that puts each on (the Effects shelf); its card shows
  /// while it is on ([cards]).
  List<(String, String)> get thingEffects => _kept('thingEffects', () => [
    for (final l in _effectNames(_m).toDartString().split('\n'))
      if (l.contains('\t')) (l.split('\t').first, l.split('\t').last),
  ]);

  /// An effect put on a thing at a moment, or taken off if it is on (its keys too), one step.
  bool effect(int thing, String name, double at) => _with([name], (s) => _effect(_m, thing, s[0], at)) != 0;

  List<String> get wayNames => _kept('wayNames', () => _wayNames(_m).toDartString().split(' ').where((w) => w.isNotEmpty).toList());
  /// A kind's numbers as a hand takes them: each one's name, lowest and highest (none: a kind without, or one defined).
  List<(String, double, double)> wayNumbers(String kind) {
    final w = _with([kind], (s) => _wayNumbers(_m, s[0]).toDartString()).split(' ').where((x) => x.isNotEmpty).toList();
    return [for (var i = 0; i + 2 < w.length; i += 3) (w[i], double.parse(w[i + 1]), double.parse(w[i + 2]))];
  }

  double wayAt(String way, double u) => _kept(('wayAt', way, u), () => _with([way], (s) => _wayAt(_m, s[0], u)));

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
  String wayOf(int thing, String name, double at) => _kept(('wayOf', thing, name, at), () => _with([name], (s) => _wayOf(_m, thing, s[0], at).toDartString()));

  /// Points of the picture to one unit of the stage, at z 0.
  late final double point = _point(_m);

  /// How a hand reads a thing's value, as the inside says it: shown = value x times + from (places in points of the
  /// picture from its top left, shares in hundredths...). Asked once a thing and a name.
  (double, double) reading(int thing, String name) => _readings.putIfAbsent('$thing $name', () {
    final f = _with([name], (s) => _reading(_m, thing, s[0]).toDartString()).split(' ');
    return (double.parse(f[0]), double.parse(f[1]));
  });
  final _readings = <String, (double, double)>{};
  late final _later = _lib.lookupFunction<Int32 Function(_P, _S, Int32, _S, _S, Double), int Function(_P, _S, int, _S, _S, double)>('motolii_later');
  late final _answer = _lib.lookupFunction<Int32 Function(_P, Int32), int Function(_P, int)>('motolii_answer');
  late final _answerWhy = _lib.lookupFunction<_S Function(_P), _S Function(_P)>('motolii_answer_why');

  /// A change asked without waiting for the inside's hand (say, set, let_go, undo, redo, together): queued in order behind
  /// every other, its ticket said at once ([answer] says when it is done).
  int later(String call, {int thing = 0, String name = '', String word = '', double number = 0}) => _with([call, name, word], (s) => _later(_m, s[0], thing, s[1], s[2], number));

  /// A ticket's answer: -1 not done yet, 0 refused ([answerWhy] says why), 1 done; said once.
  int answer(int ticket) => _answer(_m, ticket);
  String get answerWhy => _answerWhy(_m).toDartString();
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
