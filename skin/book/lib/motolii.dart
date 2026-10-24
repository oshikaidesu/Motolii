// The present inside (motolii/filament/motolii.h), called straight from Dart: dart:ffi on its C calls, as
// sets/glass_live.dart calls the old glass. The skin holds no document: it holds this handle and the numbers of things.
// One call = one change = one step of undo. A call that is not done returns false and [trouble] says why.
// macOS only, Debug build only (the Debug entitlements have the sandbox off so that the library can be loaded).
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart' show debugPrint, setEquals;
import 'package:flutter/services.dart';
import 'package:lab_book/motolii_bindings.dart' as c;   // (the inside's calls as Dart's ffigen made them: ffigen.yaml)

/// Where the inside lives. Override with --dart-define=MOTOLII_DIR=/path/to/motolii/filament.
const motoliiDir = String.fromEnvironment('MOTOLII_DIR', defaultValue: '/Users/motolii_maker/motolii/filament');

typedef _P = Pointer<c.Motolii>;
typedef _S = Pointer<Utf8>;

/// A film's thumbnail, made by the inside's own call on another isolate (nothing of a document is handed over: the
/// call asks nothing of one). Kept in the system's temp folder under the film's place and when it was last changed.
final _thumbnails = <String, Future<String?>>{};
Future<String?> _thumbnail(String lib, String film, [int step = 0, int steps = 1]) async {
  final folder = Directory('${Directory.systemTemp.path}/motolii_thumbnails')..createSync(recursive: true);
  // (named by the film's own place, so it is found again in another run, and by when the film was last changed)
  final named = film.replaceAll(RegExp(r'[^A-Za-z0-9]'), '_');
  final png = '${folder.path}/${named.length > 150 ? named.substring(named.length - 150) : named}_${File(film).statSync().modified.millisecondsSinceEpoch}${step == 0 ? '' : '_$step'}.png';
  if (File(png).existsSync()) return png;
  final made = await Isolate.run(() {
    final ask = c.MotoliiBindings(DynamicLibrary.open(lib)).motolii_thumbnail_at;
    final from = film.toNativeUtf8(), to = png.toNativeUtf8();
    final done = ask(from, to, 240, step / steps) != 0;
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
  late final _b = c.MotoliiBindings(_lib);   // (the inside's calls, made by Dart's ffigen from motolii.h: ffigen.yaml)
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
    c.MotoliiBindings(lib).motolii_home(home);
    final m = c.MotoliiBindings(lib).motolii_open(doc, wide, high);
    malloc
      ..free(home)
      ..free(doc);
    if (m == nullptr) return null;
    return Motolii._(lib, m, path, '$dir/out/wgpu_kit').._listen();
  }

  // ------------------------------------------------------------------ the pictures (skin/BRIDGE.md)
  // What the window reads of the document is the inside's two pictures, set down by its hand: the document's, made
  // again where it changes, and the moment's, made with each picture. The inside tells which is new through one
  // listener; nothing here asks the document itself, and nothing waits for the hand.
  late final _picture = _b.motolii_picture;
  late final _tell = _b.motolii_tell;
  NativeCallable<Void Function(Int)>? _told;
  Map<String, dynamic> _document = const {};   // (the document's own part)
  final _thing = <int, Map<String, dynamic>>{};   // (each thing's part, as of the version it came with)
  Map<String, dynamic> _moment = const {'thing': {}};
  int _documentAt = -1, _momentAt = -1;

  /// What the window does when the inside's pictures are new: [document] the document's (and which things changed in
  /// it), [moment] the moment's, [kept] the stretches of the stage's pictures kept on disk.
  void Function({required bool document, required bool moment, required bool kept})? told;

  void _listen() {
    _told = NativeCallable<Void Function(Int)>.listener(_take);
    _tell(_m, _told!.nativeFunction);
    _take(7);
  }

  void _take(int which) {
    final version = malloc<Long>();
    if (which & 1 != 0) {
      final p = _picture(_m, 1, _documentAt, version), all = jsonDecode(p.toDartString()) as Map<String, dynamic>;
      malloc.free(p);
      if (version.value == _documentAt) which &= ~1;   // (asked again, not new: nothing told)
      _documentAt = version.value;
      if (all['document'] case final Map<String, dynamic> d) _document = d;
      (all['thing'] as Map<String, dynamic>).forEach((t, part) => _thing[int.parse(t)] = part as Map<String, dynamic>);
      final are = {0, ...things};
      _thing.removeWhere((t, _) => !are.contains(t));
      _shownOf.clear();
    }
    if (which & 2 != 0) {
      final p = _picture(_m, 2, 0, version);
      if (version.value == _momentAt) {
        which &= ~2;   // (asked again, not new)
      } else {
        _moment = jsonDecode(p.toDartString()) as Map<String, dynamic>;
        _momentAt = version.value;
      }
      malloc.free(p);
    }
    malloc.free(version);
    if (which != 0) told?.call(document: which & 1 != 0, moment: which & 2 != 0, kept: which & 4 != 0);
  }

  /// A change the window waited for (skin/BRIDGE.md C3, going): its pictures taken at once, as the inside made them
  /// before it answered.
  T _took<T>(T answer) {
    _take(3);
    return answer;
  }

  Map<String, dynamic> _of(int thing) => _thing[thing] ?? const {};

  /// A value said nowhere: as it comes, if the thing's kind has it (the inside's tables: its `doc_usual`, `doc_knows`).
  double? _usual(int thing, String name) {
    if (!_thing.containsKey(thing)) return null;
    final kind = thing == 0 ? 'stage' : word(thing, 'kind');
    final knows = _knows.putIfAbsent(kind, () => _spaced((_document['knows'] as Map<String, dynamic>?)?[kind]).toSet());
    return knows.contains(name) ? ((_document['usual'] as Map<String, dynamic>?)?[name] as num?)?.toDouble() : null;
  }
  final _knows = <String, Set<String>>{};   // (the maker's tables, the same for every document: read once)
  Map<String, dynamic>? _momentOf(int thing) => (_moment['thing'] as Map<String, dynamic>)['$thing'] as Map<String, dynamic>?;
  static List<String> _spaced(Object? said) => [for (final w in ((said as String?) ?? '').split(' ')) if (w.isNotEmpty) w];
  static List<double> _numbers(Object? said) => [for (final n in (said as List<dynamic>?) ?? const []) (n as num).toDouble()];

  /// Whether [at] is the moment the moment's picture is of (to a frame).
  bool _isNow(double at) => (((_moment['now'] as num?)?.toDouble() ?? -1) - at).abs() < .5 / rate;

  late final _play = _b.motolii_play;
  late final _at = _b.motolii_at;
  late final _add = _b.motolii_add;
  late final _remove = _b.motolii_remove;
  late final _set = _b.motolii_set;
  late final _say = _b.motolii_say;
  late final _key = _b.motolii_key;
  late final _unkey = _b.motolii_unkey;
  late final _way = _b.motolii_way;
  late final _path3 = _b.motolii_path3;
  late final _keying = _b.motolii_keying;
  late final _hold = _b.motolii_hold;
  late final _carry = _b.motolii_carry;
  late final _letGo = _b.motolii_let_go;
  late final _put = _b.motolii_put;
  late final _keyHere = _b.motolii_key_here;
  late final _size = _b.motolii_size;
  late final _raise = _b.motolii_raise;
  late final _stay = _b.motolii_stay;
  late final _recipes = _b.motolii_recipes;
  late final _make = _b.motolii_make;
  late final _effects = _b.motolii_effects;
  late final _animators = _b.motolii_animators;
  late final _spent = _b.motolii_spent;
  late final _define = _b.motolii_define;
  late final _wayNumbers = _b.motolii_way_numbers;
  late final _effect = _b.motolii_effect;
  late final _wayAt = _b.motolii_way_at;
  late final _kindOf = _b.motolii_kind_of;
  late final _together = _b.motolii_together;
  late final _frames = _b.motolii_frames;
  late final _keptRuns = _b.motolii_kept;
  late final _undo = _b.motolii_undo;
  late final _redo = _b.motolii_redo;
  late final _trouble = _b.motolii_trouble;
  late final _list = _b.motolii_list;
  late final _still = _b.motolii_still;
  late final _close = _b.motolii_close;

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
    final ticket = later('live');   // (opened by the hand, a moment at a time waited for here: the window never stands still for it)
    var a = answer(ticket);
    while (a < 0) {
      await Future<void>.delayed(const Duration(milliseconds: 4));
      a = answer(ticket);
    }
    if (a == 0) return -1;
    return await _channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address}) ?? -1;
  }

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
  Set<int> _watched = const {};

  /// The desk's picture as a Flutter texture (-1: none).
  Future<int> deskTexture() async => _deskId ??= await _channel.invokeMethod<int>('start', {'lib': libPath, 'handle': _m.address, 'front': 'motolii_desk_front', 'frames': 'motolii_desk_frames'}) ?? -1;
  int? _deskId;

  String get trouble => _trouble(_m).toDartString();
  String get list => _list(_m).toDartString();

  /// The stage as it is now, as a picture (.png): for a hand that sees only pictures (test/agent_window_test.dart).
  bool still(String png) => _with([png], (s) => _still(_m, s[0])) != 0;

  /// The whole document written as a film (.mp4, with its films' sound), on another isolate: the window does not wait.
  /// The inside's one hand takes the call in its turn, from whatever thread it comes.
  Future<bool> write(String mp4) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final ask = c.MotoliiBindings(DynamicLibrary.open(lib)).motolii_write;
      final to = mp4.toNativeUtf8();
      final done = ask(_P.fromAddress(at), to) != 0;
      malloc.free(to);
      return done;
    });
  }
  /// As the stage shows each thing's value at each moment asked (what a relation makes of it, else its own; null: it
  /// has none): another moment than the picture's, a question (skin/BRIDGE.md Q), asked on another isolate.
  Future<List<double?>> shownAt(List<(int, String, double)> asks) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final l = DynamicLibrary.open(lib), m = _P.fromAddress(at);
      final shows = c.MotoliiBindings(l).motolii_shows;
      final out = malloc<Double>();
      final all = [
        for (final (t, n, moment) in asks)
          () {
            final name = n.toNativeUtf8(), got = shows(m, t, name, moment, out);
            malloc.free(name);
            return got == 0 ? null : out.value;
          }(),
      ];
      malloc.free(out);
      return all;
    });
  }

  /// Of these font files, those a text can be [words] in (asked on another isolate: the window does not wait).
  Future<List<String>> setting(List<String> fonts, String words) {
    final lib = libPath;
    return Isolate.run(() {
      final ask = c.MotoliiBindings(DynamicLibrary.open(lib)).motolii_sets;
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

  /// The moment of the picture the stage last drew: where a playhead is while it plays.
  double get now => (_moment['now'] as num?)?.toDouble() ?? 0;
  double get drawnAt => now;
  double get seconds => (_document['seconds'] as num?)?.toDouble() ?? 0;
  late final _rate = _b.motolii_rate;

  /// How many pictures a second a film written of the document has: what a frame is.
  int get rate => _rate(_m);
  bool play(bool on) => _play(_m, on ? 1 : 0) != 0;
  int get frames => _frames(_m); // (how many pictures the live stage has drawn: motolii_frames)

  /// The stage's pictures kept on disk for playing, as stretches of frames [from, to) (motolii_kept).
  List<(int, int)> get kept {
    final n = _keptRuns(_m).toDartString().split(' ').where((w) => w.isNotEmpty).map(int.parse).toList();
    return [for (var i = 0; i + 1 < n.length; i += 2) (n[i], n[i + 1])];
  }
  bool at(double seconds) => _at(_m, seconds) != 0;

  /// A new thing (film, picture, text, solid) from a file (a solid: a file, or ring, gem or cube): its number, 0 if it was not made.
  /// What a solid is made of is its `fill`: a colour, or `glass`.
  int add(String kind, String file) => _took(_with([kind, file], (s) => _add(_m, s[0], s[1])));
  bool remove(int thing) => _took(_remove(_m, thing) != 0);
  bool set(int thing, String name, double number) => _took(_with([name], (s) => _set(_m, thing, s[0], number)) != 0);
  bool say(int thing, String name, String word) => _took(_with([name, word], (s) => _say(_m, thing, s[0], s[1])) != 0);
  bool key(int thing, String name, double at, double number) => _took(_with([name], (s) => _key(_m, thing, s[0], at, number)) != 0);
  late final _keyMany = _b.motolii_key_many;

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
  bool unkey(int thing, String name, double at) => _took(_with([name], (s) => _unkey(_m, thing, s[0], at)) != 0);
  bool way(int thing, String name, double at, String way) => _took(_with([name, way], (s) => _way(_m, thing, s[0], at, s[1])) != 0);
  /// A value at a moment, its own (not what a relation makes of it: [shows]): one that stays, the document's; one that
  /// moves, the picked thing's at the moment's picture's moment.
  double? value(int thing, String name, double at) {
    if (!moving(thing).contains(name) || at == 0) return ((_of(thing)['value'] as Map<String, dynamic>?)?[name] as num?)?.toDouble() ?? _usual(thing, name);   // (the document's picture holds each value at its first moment)
    if (_isNow(at)) {   // (the picked thing's, and those the window watches: [watch])
      if ((_momentOf(thing)?['value'] as Map<String, dynamic>?)?[name] case final num v) return v.toDouble();
    }
    final keys = keyMoments(thing, name), i = keys.indexWhere((k) => (k - at).abs() < .5 / rate);
    if (i >= 0) return (((_of(thing)['keyValues'] as Map<String, dynamic>?)?[name] as List<dynamic>?)?[i] as num?)?.toDouble();   // (at one of its keys: what the key holds)
    return _unasked('value', thing, name, at);
  }

  /// A read the pictures do not hold (another moment of a thing not watched): not asked of the inside here, the window
  /// would wait (skin/BRIDGE.md rule 1); ask it as a question ([shownAt]), or watch the thing ([subscribe]). Said once.
  double? _unasked(String what, int thing, String name, double at) {
    if (_watched.contains(thing)) return null;   // (watched just now: in the next picture)
    if (_unaskedSaid.add('$what $name')) debugPrint('BRIDGE: $what($thing, $name, $at) is not in the pictures: ask it ([shownAt]) or watch the thing');
    return null;
  }
  final _unaskedSaid = <String>{};

  /// A value as the stage shows it (what a relation or behaviour makes of it), and whether one drives it; null: none.
  (double, bool)? shows(int thing, String name, double at) {
    if (!_isNow(at)) return switch (_unasked('shows', thing, name, at)) { _ => null };
    final v = (_momentOf(thing)?['shows'] as Map<String, dynamic>?)?[name] as List<dynamic>?;
    return v == null ? null : ((v[0] as num).toDouble(), v[1] as bool);
  }


  /// The numbers of the things there are.
  List<int> get things => [for (final t in (_document['things'] as List<dynamic>?) ?? const []) t as int];

  /// A thing's word by name (kind, file, shape, blend, text, fill); '' if it has none.
  String word(int thing, String name) => ((_of(thing)['word'] as Map<String, dynamic>?)?[name] as String?) ?? '';

  /// How long a thing is on, in seconds; below 0: for good.
  double lasts(int thing) => (_of(thing)['lasts'] as num?)?.toDouble() ?? -1;

  /// Every moment a key of any of a thing's values stands at, each once, in time: a timeline bar's marks.
  List<double> thingKeyMoments(int thing) => _numbers(_of(thing)['keys']);

  /// The moments of a value's keys (none if the value stays): how many keys it has is their count.
  List<double> keyMoments(int thing, String name) => _numbers((_of(thing)['keyMoments'] as Map<String, dynamic>?)?[name]);

  /// A thing's frame on the picture at the stage's moment: its middle (x, y), how wide, how high (points of the picture,
  /// from the top left), how far it is turned (degrees, clockwise as seen), the point it turns about (x, y), and how deep
  /// it is (0: flat). null: not shown.
  List<double>? frame(int thing) => switch (_momentOf(thing)?['frame']) { final List<dynamic> f => _numbers(f), _ => null };

  /// What of a thing is seen: the upright box of its outline on the picture (points from the top left). null: not shown.
  Rect? seen(int thing) => switch (_momentOf(thing)?['seen']) { final List<dynamic> b => Rect.fromLTRB((b[0] as num).toDouble(), (b[1] as num).toDouble(), (b[2] as num).toDouble(), (b[3] as num).toDouble()), _ => null };

  /// A path shape's points where the picture shows them (the picked thing's; its handles not); empty: none.
  List<Offset> points(int thing) {
    final p = _numbers(_momentOf(thing)?['points']);
    return [for (var i = 0; i + 1 < p.length; i += 2) Offset(p[i], p[i + 1])];
  }

  /// Its curves' handles the same way, each with the point it is the handle of.
  List<(Offset, Offset)> handles(int thing) {
    final h = _numbers(_momentOf(thing)?['handles']), of = _numbers(_momentOf(thing)?['of']), p = points(thing);
    return [for (var i = 0; i + 1 < h.length; i += 2) (Offset(h[i], h[i + 1]), i ~/ 2 < of.length && of[i ~/ 2].toInt() < p.length ? p[of[i ~/ 2].toInt()] : Offset.zero)];
  }

  /// How a thing moves over a while, as the stage places it (its wave and all): [n] moments from [from] to before [to],
  /// four numbers each: where it is (x, y, units of the stage), how far turned (degrees), how large (a share). null: no
  /// such thing. Asked once when its values change, for a small preview that then plays by itself (motolii_path).
  Future<List<double>?> path(int thing, {int copy = 0, double from = 0, required double to, int n = 60}) => _ask((l, m) {
    final out = malloc<Double>(4 * n);
    final got = c.MotoliiBindings(l).motolii_path(m, thing, copy, from, to, n, out);
    final p = [for (var i = 0; i < 4 * got; i++) out[i]];
    malloc.free(out);
    return got > 0 ? p : null;
  });

  /// A question of the inside (skin/BRIDGE.md Q): asked on another isolate, the window does not wait for its answer.
  Future<T> _ask<T>(T Function(DynamicLibrary lib, _P m) ask) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() => ask(DynamicLibrary.open(lib), _P.fromAddress(at)));
  }
  /// The same in the round: four numbers a moment, where it is (x, y, z, units of the stage) and how large (motolii_path3).
  List<double>? path3(int thing, {int copy = 0, double from = 0, required double to, int n = 24}) {
    final out = malloc<Double>(4 * n);
    final got = _path3(_m, thing, copy, from, to, n, out), p = [for (var i = 0; i < 4 * got; i++) out[i]];
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
  late final _shaping = _b.motolii_shaping;

  /// What the document's steps changed since last asked (asking forgets them): whether a thing came or went, and each
  /// value changed as (thing, name), from each step's own changeset.
  ({bool things, Set<(int, String)> values}) changes() {
    final lines = ((_document['changes'] as String?) ?? '').split('\n');
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
  int keyed(int thing, String name, double at) {
    final keys = keyMoments(thing, name);
    return keys.isEmpty ? 0 : keys.any((k) => (k - at).abs() < .5 / rate) ? 2 : 1;
  }

  /// The size of the stage the document is open on; and the size it is to be when it is next opened (close, open again).
  (int, int) get size => switch (_document['size']) { [final int w, final int h] => (w, h), _ => (0, 0) };
  bool resize(int wide, int high) => _took(_size(_m, wide, high) != 0);
  late final _work = _b.motolii_work;
  late final _mark = _b.motolii_mark;
  late final _sequence = _b.motolii_sequence;

  /// Sequence Layers: the things that have a length and stand on their own, from the front, one after another
  /// ([overlap] seconds over each other), on frames, their keys with them; one step ([trouble] says why not).
  bool sequence(double overlap) => _took(_sequence(_m, overlap) != 0);

  /// The work area, in seconds: the stretch played round and written as a film (not said: all of it); a change is a step.
  (double, double) get work => switch (_document['work']) { [final num a, final num b] => (a.toDouble(), b.toDouble()), _ => (0.0, 0.0) };

  bool setWork(double from, double to) => _took(_work(_m, from, to) != 0);
  late final _loopIs = _b.motolii_loop;

  /// Whether play goes round the work area (or stops at its end).
  bool get looping => _loopIs(_m, -1) != 0;
  set looping(bool on) => _loopIs(_m, on ? 1 : 0);

  /// The marks: moments with a name, in time; a mark said with '' is taken away.
  List<(double, String)> get marks => [
    for (final line in ((_document['marks'] as String?) ?? '').split('\n'))
      if (line.indexOf(' ') case final i when i > 0) (double.parse(line.substring(0, i)), line.substring(i + 1)),
  ];
  bool mark(double at, String word) => _took(_with([word], (s) => _mark(_m, at, s[0])) != 0);

  /// A film's sound: its loudness (dB, 0 at full) 100 times a second of its own (none: no sound, or not a film). Read
  /// on another isolate, the file decoded once: the window does not wait.
  Future<List<double>> loudness(int thing) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final ask = c.MotoliiBindings(DynamicLibrary.open(lib)).motolii_loudness;
      final m = _P.fromAddress(at), n = ask(m, thing, nullptr, 0), out = malloc<Float>(n < 1 ? 1 : n);
      final got = ask(m, thing, out, n), all = [for (var i = 0; i < got && i < n; i++) out[i].toDouble()];
      malloc.free(out);
      return all;
    });
  }


  /// How loud a film's sound is at [n] moments of the document from [from] to before [to], where its film shows them
  /// (dB; -100 silent or not shown). null: its sound not read yet ([loudness] reads it); empty: no sound.
  Future<List<double>?> loudnessOver(int thing, double from, double to, int n) => _ask((l, m) {
    final out = malloc<Float>(n < 1 ? 1 : n);
    final got = c.MotoliiBindings(l).motolii_loudness_over(m, thing, from, to, n, out);
    final all = got < 0 ? null : [for (var i = 0; i < got; i++) out[i].toDouble()];
    malloc.free(out);
    return all;
  });

  /// Where a film's sound strikes (grows [rise] dB louder at once): the document's moments where it shows them, or
  /// [own] the film's own seconds. Asked once its loudness has been read ([loudness]).
  Future<List<double>> strikes(int thing, double rise, {bool own = false}) => _ask((l, m) {
    final ask = c.MotoliiBindings(l).motolii_strikes;
    final n = ask(m, thing, rise, own ? 1 : 0, nullptr, 0), out = malloc<Double>(n < 1 ? 1 : n);
    final got = ask(m, thing, rise, own ? 1 : 0, out, n), all = [for (var i = 0; i < got && i < n; i++) out[i]];
    malloc.free(out);
    return all;
  });
  late final _guide = _b.motolii_guide;

  /// The guides: lines across the picture, 'x' upright and 'y' level, at so many points from its left or top.
  List<(String, double)> get guides => [
    for (final line in ((_document['guides'] as String?) ?? '').split('\n'))
      if (line.length > 2) (line[0], double.parse(line.substring(2))),
  ];
  bool guide(String axis, double at, bool on) => _took(_with([axis], (s) => _guide(_m, s[0], at, on ? 1 : 0)) != 0);

  /// Whether the stage is the size the document says now (not, after [resize] or an undo of one: open it again).
  bool get fits => (_document['fits'] as bool?) ?? true;

  /// A thing brought [places] places nearer among the things (below 0: further back).
  bool raise(int thing, int places) => _took(_raise(_m, thing, places) != 0);

  /// The names of a thing's values that take keys.
  List<String> moving(int thing) => _spaced(_of(thing)['moving']);

  /// The names of all a thing's values.
  List<String> values(int thing) => _spaced(_of(thing)['values']);

  /// Pictures in one call: each recipe's name and what it makes.
  List<(String, String)> get recipes => _folders.putIfAbsent('recipes', () => [
    for (final line in _recipes(_m).toDartString().split('\n'))
      if (line.indexOf('\t') case final at when at > 0) (line.substring(0, at), line.substring(at + 1)),
  ]);

  /// The whole-screen effects on the shelf: each one's name and file (the stage's script is the files of those on it).
  List<(String, String)> get effects => _folders.putIfAbsent('effects', () => _labelled(_effects(_m).toDartString()));

  /// The motions a thing may be given (Cavalry's Animators): each one's name and file (a thing's script is the file of the one it has).
  List<(String, String)> get animators => _folders.putIfAbsent('animators', () => _labelled(_animators(_m).toDartString()));

  /// What an effect (of a thing or of the whole screen) or a motion is for, as the inside says: the shelf's kind ('' none said).
  String label(String name) {
    _said();
    return _labels[name] ?? '';
  }

  /// An effect's (or a motion's) picture for its tile, as the inside says: what it does to a sample ('' none).
  String picture(String name) {
    _said();
    return _pictures[name] ?? '';
  }

  void _said() => (effects, thingEffects);   // (both lists read: their labels and pictures with them)

  final _labels = <String, String>{}, _pictures = <String, String>{};

  /// "name\tfile\tlabel\tpicture" lines: the names and files, the labels and pictures kept for [label] and [picture].
  List<(String, String)> _labelled(String lines) {
    final out = <(String, String)>[];
    for (final w in [for (final l in lines.split('\n')) l.split('\t')]) {
      if (w.length < 2 || w[0].isEmpty) continue;
      out.add((w[0], w[1]));
      if (w.length > 2) _labels[w[0]] = w[2];
      if (w.length > 3) _pictures[w[0]] = w[3];
    }
    return out;
  }

  /// What the inside's hand has spent its time on (for measuring).
  String get spent => _spent(_m).toDartString();

  /// A recipe made at the present moment, as one step: the new thing (0: not made, [trouble] says why).
  int make(String recipe) => _took(_with([recipe], (s) => _make(_m, s[0])));

  /// The cards a maker sees a thing's values on, besides where it is and its fill: each a title and the values on it.
  List<(String, List<String>)> cards(int thing) => [
    for (final c in ((_of(thing)['cards'] as String?) ?? '').split('; '))
      if (c.indexOf(': ') case final at when at > 0) (c.substring(0, at), c.substring(at + 2).split(' ')),
  ];

  /// The titles of a thing's effect cards in the order its effects are laid (motolii_fx_titles): moved with `fx_move`.
  List<String> fxTitles(int thing) => [for (final t in ((_of(thing)['fx'] as String?) ?? '').split(';')) if (t.isNotEmpty) t];

  /// How a value is shown to a hand, as the inside says (null: as its range says): its look, the unit it is read in,
  /// its name in plain words, the span its knob covers (in the document's units) and the fold it lies in.
  Shown? shown(String name) => _shownOf.putIfAbsent(name, () {
    for (final part in _thing.values) {
      if ((part['shown'] as Map<String, dynamic>?)?[name] case final String said) {
        final f = said.split('\t');
        return f.length < 6 ? null : Shown(f[0], f[1], f[2], double.parse(f[3]), double.parse(f[4]), f[5].isEmpty ? null : f[5]);
      }
    }
    return null;
  });
  final _shownOf = <String, Shown?>{};   // (read out of the things' parts once a picture of the document)

  /// A number value's lowest and highest (null: the thing has no such number).
  (double, double)? range(int thing, String name) => switch ((_of(thing)['range'] as Map<String, dynamic>?)?[name]) { [final num lo, final num hi] => (lo.toDouble(), hi.toDouble()), _ => null };

  /// The words a value of a thing may be (a blend's): none if any word will do, or the thing has no such value.
  List<String> words(int thing, String name) => _spaced((_of(thing)['words'] as Map<String, dynamic>?)?[name]);

  /// What the folders hold (recipes, effects, animators): not the document's, read once.
  final _folders = <String, List<(String, String)>>{};

  /// The window's thread watched: while on, a change it waits for, or a read that waits on the document over 2 ms, is
  /// kept; [broken] says each, one to a line ('' none). Turned on, what was kept is forgotten.
  late final _watch = _b.motolii_watch;
  late final _broken = _b.motolii_broken;
  void watch(bool on) => _watch(_m, on ? 1 : 0);
  String get broken => _broken(_m).toDartString();
  late final _brokenCount = _b.motolii_broken_count;
  int get brokenCount => _brokenCount(_m);   // (how many times since watched: read without waiting)

  /// A film's first picture, small, as a .png file (null: none could be made). Made once and on another isolate: the
  /// window does not wait.
  Future<String?> thumbnail(String film) => _thumbnails[film] ??= _thumbnail(libPath, film);

  late final _fontTags = _b.motolii_font_tags;
  final _fontKinds = <String, Set<String>>{};

  /// A font file's kinds as CoreText reads them (serif, sans, script, mono, bold, light…): the system's own, not guessed from a name.
  Set<String> fontTags(String path) => _fontKinds[path] ??= _with([path], (s) => _fontTags(s[0]).toDartString()).split(' ').where((w) => w.isNotEmpty).toSet();

  /// A picture [step] of [steps] along the film (a tile looked along by the hand's place on it, or played).
  Future<String?> thumbnailAt(String film, int step, int steps) => step == 0 ? thumbnail(film) : _thumbnails['$film#$step/$steps'] ??= _thumbnail(libPath, film, step, steps);

  /// The names of the ways a stretch may go, and how one of the nine kinds goes at [u] of the way through (0 to 1).
  /// A way of this document's own, by name: how far it is (`a` the one key's value, `b` the next's, `u` from 0 to 1
  /// through the stretch, `p1`..`p4` its numbers; `bezier(u, x1, y1, x2, y2)` an ease by its two handles). It is then
  /// one of [wayNames], a tile of the Ease desk, and saved with the document.
  bool define(String way, String expression) => _took(_with([way, expression], (s) => _define(_m, s[0], s[1])) != 0);
  /// The ways a stretch may go, as the document says them: the nine kinds with their numbers, the named ones, and
  /// each defined here ('; defined here: Snappy').
  String get ways => (_document['ways'] as String?) ?? '';

  /// The effects a thing may be given, by name, with the value that puts each on (the Effects shelf); its card shows
  /// while it is on ([cards]).
  List<(String, String)> get thingEffects => _labelled((_document['effectNames'] as String?) ?? '');

  /// An effect put on a thing at a moment, or taken off if it is on (its keys too), one step.
  bool effect(int thing, String name, double at) => _took(_with([name], (s) => _effect(_m, thing, s[0], at)) != 0);

  List<String> get wayNames => _spaced(_document['wayNames']);
  /// A kind's numbers as a hand takes them: each one's name, lowest and highest (none: a kind without, or one defined).
  List<(String, double, double)> wayNumbers(String kind) {
    final w = _with([kind], (s) => _wayNumbers(_m, s[0]).toDartString()).split(' ').where((x) => x.isNotEmpty).toList();
    return [for (var i = 0; i + 2 < w.length; i += 3) (w[i], double.parse(w[i + 1]), double.parse(w[i + 2]))];
  }

  double wayAt(String way, double u) {   // (a way of the document's, by its curve in the picture; one said with its numbers, by the inside's own reckoning, which asks nothing of the document)
    if ((_document['wayCurves'] as Map<String, dynamic>?)?[way] case final List<dynamic> c when c.length > 1) {
      final at = u.clamp(0.0, 1.0) * (c.length - 1), i = at.floor().clamp(0, c.length - 2), f = at - i;
      return (c[i] as num).toDouble() * (1 - f) + (c[i + 1] as num).toDouble() * f;
    }
    return _with([way], (s) => _wayAt(_m, s[0], u));
  }

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

  /// Every stretch of a thing's keyed values: from, to (seconds), its way, as the inside says them (a timeline draws how each goes).
  List<(double, double, String)> thingStretches(int thing) => [
        for (final l in ((_of(thing)['stretches'] as String?) ?? '').split('\n'))
          if (l.split('\t') case [final a, final b, final w]) (double.parse(a), double.parse(b), w),
      ];

  /// The way of the stretch that begins at a value's key, as it is said ('': no key there).
  String wayOf(int thing, String name, double at) {
    final keys = keyMoments(thing, name), ways = (_of(thing)['keyWays'] as Map<String, dynamic>?)?[name] as List<dynamic>?;
    final i = keys.indexWhere((k) => (k - at).abs() < .5 / rate);
    return i < 0 || ways == null || i >= ways.length ? '' : ways[i] as String;
  }

  /// Points of the picture to one unit of the stage, at z 0.
  double get point => (_document['point'] as num?)?.toDouble() ?? 1;

  /// How a hand reads a thing's value, as the inside says it: shown = value x times + from (places in points of the
  /// picture from its top left, shares in hundredths...). Asked once a thing and a name.
  (double, double) reading(int thing, String name) {
    final f = (((_of(thing)['reading'] as Map<String, dynamic>?)?[name] as String?) ?? '').split(' ');
    return f.length < 2 ? (1.0, 0.0) : (double.parse(f[0]), double.parse(f[1]));
  }
  late final _later = _b.motolii_later;
  late final _answer = _b.motolii_answer;
  late final _answerWhy = _b.motolii_answer_why;

  /// A change asked without waiting for the inside's hand (say, set, let_go, undo, redo, together): queued in order behind
  /// every other, its ticket said at once ([answer] says when it is done).
  int later(String call, {int thing = 0, String name = '', String word = '', double number = 0}) => _with([call, name, word], (s) => _later(_m, s[0], thing, s[1], s[2], number));

  /// A ticket's answer: -1 not done yet, 0 refused ([answerWhy] says why), 1 done; said once.
  int answer(int ticket) {
    final a = _answer(_m, ticket);
    if (a >= 0) _take(3);   // (done: the pictures as the inside made them before it answered)
    return a;
  }
  String get answerWhy => _answerWhy(_m).toDartString();
  bool undo() => _took(_undo(_m) != 0);
  bool redo() => _took(_redo(_m) != 0);
  void close() {
    _tell(_m, nullptr);
    _told?.close();
    _close(_m);
  }
}

/// How a value is shown to a hand (Motolii.shown): look, unit, plain name, knob span, folded away.
class Shown {
  const Shown(this.look, this.unit, this.label, this.low, this.high, this.fold);
  final String look, unit, label;
  final double low, high;

  /// The fold it lies in on its card, named by what the fold holds (null: on the card's first row).
  final String? fold;
}
