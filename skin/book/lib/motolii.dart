// The present inside (motolii/filament/motolii.h), called straight from Dart: dart:ffi on its C calls, as
// sets/glass_live.dart calls the old glass. The skin holds no document: it holds this handle and the numbers of things.
// One call = one change = one step of undo. A call that is not done returns false and [trouble] says why.
// macOS only, Debug build only (the Debug entitlements have the sandbox off so that the library can be loaded).
import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart' show debugPrint, setEquals;
import 'package:flutter/services.dart';
import 'package:lab_book/motolii_bindings.dart' as c;   // (the inside's calls as Dart's ffigen made them: ffigen.yaml)
part 'motolii_2.dart';

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
  /// The document's copy as now (BRIDGE.md rule 3: replaced whole when the document changes): an answer kept for it
  /// holds while it is the same one (identical).
  Object get documentNow => _document;
  /// Whether [at] is the moment of the picture shown (what the moment's copy holds).
  bool isNow(double at) => _isNow(at);
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
    if (which & 8 != 0) {   // (an answer is there: the changes waited for, each looked at once)
      for (final t in [..._waiting.keys]) {
        final a = _answer(_m, t);
        if (a >= 0) _waiting.remove(t)!.complete((a, a >= 1 ? '' : answerWhy));
      }
    }
    if (which & 7 != 0) told?.call(document: which & 1 != 0, moment: which & 2 != 0, kept: which & 4 != 0);
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

  T _with<T>(List<String> words, T Function(List<_S>) call) => using((a) => call([for (final w in words) w.toNativeUtf8(allocator: a)]));   // (package:ffi's Arena: all let go when the call is done)

  static const _channel = MethodChannel('motolii/stage');

  /// The stage's picture as a Flutter texture (the inside draws it by its own clock; nothing is copied). -1: none.
  Future<int> texture() async {
    final (a, _) = await answered(later('live'));   // (opened by the hand: the window never stands still for it)
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
  }  /// Each thing's own value at each moment asked, as the document says it (not what a relation makes of it; null: it
  /// has none): another moment than the picture's, a question (skin/BRIDGE.md Q), asked on another isolate as [shownAt].
  Future<List<double?>> valuesAt(List<(int, String, double)> asks) {
    final lib = libPath, at = _m.address;
    return Isolate.run(() {
      final l = DynamicLibrary.open(lib), m = _P.fromAddress(at);
      final value = c.MotoliiBindings(l).motolii_value;
      final out = malloc<Double>();
      final all = [
        for (final (t, n, moment) in asks)
          () {
            final name = n.toNativeUtf8(), got = value(m, t, name, moment, out);
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

  late final _shaping = _b.motolii_shaping;

  /// The names of the values that change what cards a thing has (and its name): the inside's own rule, asked once.
  late final Set<String> shaping = _shaping(_m).toDartString().split(' ').toSet();

  late final _work = _b.motolii_work;
  late final _mark = _b.motolii_mark;
  late final _sequence = _b.motolii_sequence;

  late final _loopIs = _b.motolii_loop;

  late final _guide = _b.motolii_guide;

  final _labels = <String, String>{}, _pictures = <String, String>{};

  final _shownOf = <String, Shown?>{};   // (read out of the things' parts once a picture of the document)

  /// What the folders hold (recipes, effects, animators): not the document's, read once.
  final _folders = <String, List<(String, String)>>{};

  /// The window's thread watched: while on, a change it waits for, or a read that waits on the document over 2 ms, is
  /// kept; [broken] says each, one to a line ('' none). Turned on, what was kept is forgotten.
  late final _watch = _b.motolii_watch;
  late final _broken = _b.motolii_broken;
  late final _brokenCount = _b.motolii_broken_count;

  late final _fontTags = _b.motolii_font_tags;
  final _fontKinds = <String, Set<String>>{};

  late final _later = _b.motolii_later;
  late final _answer = _b.motolii_answer;
  late final _answerWhy = _b.motolii_answer_why;

  final _waiting = <int, Completer<(int, String)>>{};

}

/// How a value is shown to a hand (Motolii.shown): look, unit, plain name, knob span, folded away.
class Shown {
  const Shown(this.look, this.unit, this.label, this.low, this.high, this.fold);
  final String look, unit, label;
  final double low, high;

  /// The fold it lies in on its card, named by what the fold holds (null: on the card's first row).
  final String? fold;
}
