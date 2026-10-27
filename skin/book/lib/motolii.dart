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
part 'motolii_document.dart';
part 'motolii_moment.dart';
part 'motolii_writes.dart';
part 'motolii_questions.dart';
part 'motolii_shelves.dart';
part 'motolii_watch.dart';

/// Where the inside lives: --dart-define=MOTOLII_DIR=/path/to/motolii/filament, or else the filament/ found going up from
/// where it was started (flutter test runs in skin/book) or from this program (the app is built under skin/book/build/).
final String motoliiDir = () {
  const given = String.fromEnvironment('MOTOLII_DIR');
  if (given.isNotEmpty) return given;
  for (final from in [Directory.current, File(Platform.resolvedExecutable).parent]) {
    for (var d = from.absolute; ; d = d.parent) {
      if (File('${d.path}/filament/edit.sh').existsSync()) return '${d.path}/filament';
      if (d.parent.path == d.path) break;
    }
  }
  return 'filament';
}();

/// The document the window opens: --dart-define=MOTOLII_DOC=/path/to.motolii, or the inside's own out/skin.motolii.
String get motoliiDoc {
  const given = String.fromEnvironment('MOTOLII_DOC');
  return given.isNotEmpty ? given : '$motoliiDir/out/skin.motolii';
}

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

  /// The document at [document] (made if it is not there), on a stage [wide] by [high]. null: it could not be opened.
  static Motolii? open(String document, {int wide = 1920, int high = 1080, String? dir}) {
    dir ??= motoliiDir;
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

  final _knows = <String, Set<String>>{};   // (the maker's tables, the same for every document: read once)
  static List<String> _spaced(Object? said) => [for (final w in ((said as String?) ?? '').split(' ')) if (w.isNotEmpty) w];
  static List<double> _numbers(Object? said) => [for (final n in (said as List<dynamic>?) ?? const []) (n as num).toDouble()];

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

  static const _channel = MethodChannel('motolii/stage');

  Set<int> _watched = const {};

  int? _deskId;

  late final _rate = _b.motolii_rate;

  late final _keyMany = _b.motolii_key_many;

  late final _midi = _b.motolii_midi;
  late final _midiBpm = _b.motolii_midi_bpm;

  final _unaskedSaid = <String>{};

  late final _shaping = _b.motolii_shaping;

  /// The names of the values that change what cards a thing has (and its name): the inside's own rule, asked once.
  late final Set<String> shaping = _shaping(_m).toDartString().split(' ').toSet();

  late final _work = _b.motolii_work;
  late final _mark = _b.motolii_mark;
  late final _sequence = _b.motolii_sequence;

  late final _loopIs = _b.motolii_loop;

  late final _guide = _b.motolii_guide;

  final _labels = <String, String>{}, _pictures = <String, String>{}, _homes = <String, String>{};
  final _same = <String>{};

  final _shownOf = <String, Shown?>{};   // (read out of the things' parts once a picture of the document)

  /// What the folders hold (recipes, effects, animators): not the document's, read once, and again after [forgetFolders].
  final _folders = <String, List<(String, String)>>{};

  /// The folders read again at the next ask (a file put in the maker's own folder: workspace/effect_tiles.dart).
  void forgetFolders() {
    _folders.clear();
    _labels.clear();
    _pictures.clear();
    _homes.clear();
    _same.clear();
  }

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

  /// A file word of the document as a whole path.
  String whole(String file) => Uri.directory(home).resolveUri(Uri.file(file)).toFilePath();

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

  T _with<T>(List<String> words, T Function(List<_S>) call) => using((a) => call([for (final w in words) w.toNativeUtf8(allocator: a)]));   // (package:ffi's Arena: all let go when the call is done)

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
