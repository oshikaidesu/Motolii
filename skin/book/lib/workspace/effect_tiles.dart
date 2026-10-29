// Every effect on the shelf has a tile that moves, as Alight Motion's do (reports/alight_ui.md, 71 and 72): the sample
// filament/fx_tiles.sh draws the bundled ones on (the balloon, fx_sample.jpg, drifting, a white Aa over it) with the
// effect on, eight pictures from 0.1 to 1.9 seconds (a transition's progress going from 0 to 1 across them), played in
// the tile as the media shelf plays a film's
// (dock_files_contents.dart); its still the one at 1.4 seconds, fx_tiles.sh's moment. Drawn on a document of its own in
// another isolate (as create_preview.dart reckons the recipes), where the inside says one drawn for it goes (its home in
// ~/Library/Caches/Motolii/tiles, named by the file and when it was written: doc_recipes.h doc_tile), then shown at once.
// Put on as fx_tiles.sh and edit.sh say it: a screen effect (.fs, .wgsl) on the whole picture; else on the photo, as a
// thing's (an AviUtl2 script). Made small with sips (192 px, the moving ones JPEG: 708 tiles in about 60 MB, not 630).
// Then set beside the sample's own pictures with
// ffmpeg's psnr (the tool the inside writes films with): seen to change nothing in any of them (over 40 dB: helpers of
// other scripts, PSD ones, defaults that show nothing, scripts that stop on a call not made yet) it is marked
// (home.same), and the shelf says so instead of passing the sample off as the effect.
// The maker's own folder is watched while the window is open (dart:io's watch, FSEvents on a Mac): a file put in or taken
// out there and the shelves are read again (BRIDGE.md rule 8, the second half), and its tile drawn, ahead of the rest.
import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'dart:isolate';

import '../motolii.dart';

abstract final class EffectTiles {
  static final _asked = <String>{}, _queue = <(String, String)>[];
  static var _drawing = false;

  /// The moments its pictures are drawn at, and the one its still is (fx_tiles.sh's 1.4 s).
  static const moments = [0.1, 0.357, 0.614, 0.871, 1.129, 1.386, 1.643, 1.9];
  static const still = 5;

  /// What AbsentMotion shows each of its effects with (its tools/effect_showcase.json, Apache-2.0): the knobs that make
  /// the effect plain to see, and where its picture is split into before (left) and after. By the effect's name in the
  /// table, the file's own `am_<name>.wgsl`; each knob as from_absent.py named it (skslc's R_ before a name WGSL keeps for
  /// itself, origin_x and the like with _ after), a vector or a colour one knob each way. Null: a file the table has not.
  static ({Map<String, double> knobs, double? split})? showcase(String file, String head, Map<String, dynamic> table) {
    final name = RegExp(r'/am_([a-z0-9]+)\.wgsl$').firstMatch(file)?.group(1);
    final show = name == null ? null : (table['effects'] as Map<String, dynamic>?)?[name] as Map<String, dynamic>?;
    if (show == null) return null;
    final tracks = {for (final t in RegExp(r'--track@([A-Za-z0-9_]+):').allMatches(head)) t.group(1)!};
    final stem = file.split('/').last.replaceFirst(RegExp(r'\.[^.]*$'), '').replaceAll(RegExp(r'[^A-Za-z0-9_]'), '_');
    final knobs = <String, double>{};
    for (final MapEntry(key: p, :value) in ((show['params'] as Map<String, dynamic>?) ?? const {}).entries) {
      final parts = switch (value) {
        num v => [v.toDouble()],
        bool v => [v ? 1.0 : 0.0],
        String v when RegExp(r'^#[0-9a-fA-F]{6}$').hasMatch(v) => [for (var i = 1; i < 7; i += 2) int.parse(v.substring(i, i + 2), radix: 16) / 255, 1.0],
        List v when v.length <= 4 && v.every((x) => x is num) => [for (final x in v) (x as num).toDouble()],   // (a vector; a longer list is not one: left as the file says)
        _ => const <double>[],
      };
      for (final (i, v) in parts.indexed) {
        final way = parts.length > 1 ? '_${'xyzw'[i]}' : '';
        if ([for (final n in [p, 'R_$p', '${p}_']) '$n$way'].where(tracks.contains).firstOrNull case final knob?) knobs['js_${stem}_$knob'] = v;
      }
    }
    return (knobs: knobs, split: (show['split'] as num?)?.toDouble());
  }

  /// AbsentMotion's table, read once (in each isolate that draws): empty where the library has not got it.
  static Map<String, dynamic> table(String dir) {
    try {
      return jsonDecode(File('$dir/third_party/AbsentMotion/tools/effect_showcase.json').readAsStringSync()) as Map<String, dynamic>;
    } catch (_) {
      return const {};
    }
  }

  static Map<String, dynamic>? _table;
  static bool _shownBy(String file) => showcase(file, '', _table ??= table(motoliiDir)) != null;

  /// Its moving pictures: home_0.jpg … home_7.jpg beside its still (none for a picture not drawn here).
  static String frame(String home, int k) => '${home.substring(0, home.length - 4)}_$k.jpg';
  static String _drawn(String home, int k) => '${home.substring(0, home.length - 4)}_$k.png';   // (as the inside draws it, before it is made small)

  /// The maker's own folder: MOTOLII_OWN, or ~/Library/Application Support/Motolii (as the inside's doc_folders).
  static String get own => Platform.environment['MOTOLII_OWN'] ?? '${Platform.environment['HOME']}/Library/Application Support/Motolii';

  /// The library's own folders beside the maker's (as the inside's doc_folders: the folder above its out/).
  static List<String> get library => [for (final f in const ['effects', 'aviutl_scripts', 'recipes', 'animators']) '$motoliiDir/$f'];

  /// Watched until the subscription is cancelled; [changed] is told after the folders are read again and after tiles are drawn.
  static StreamSubscription<FileSystemEvent>? watch(Motolii m, void Function() changed) {
    ask(m, changed);
    final dir = Directory('$own/effects');
    try {
      dir.createSync(recursive: true); // (there to put files in, as README says)
    } catch (_) {
      return null;
    }
    Timer? soon;
    void again() {
      m.forgetFolders();
      changed();
      ask(m, changed);
    }
    final inner = <StreamSubscription<FileSystemEvent>>[];
    final merged = StreamController<FileSystemEvent>(onCancel: () => Future.wait([for (final w in inner) w.cancel()]));   // (one watch a folder, told as one)
    inner.addAll([for (final p in [own, ...library.where((p) => Directory(p).existsSync())]) Directory(p).watch(recursive: true).listen(merged.add)]);
    return merged.stream.listen((_) {
      soon?.cancel();
      soon = Timer(const Duration(milliseconds: 300), again); // (a copy is several events: read once it has settled)
    });
  }

  /// The effects on the shelf whose tile is not drawn yet: one drawn before (this run, or another) shown at once; the
  /// rest put in line, the maker's own at its head (what was just put in), drawn a few at a time, each few shown when done.
  static void ask(Motolii m, void Function() changed) {
    final mine = <(String, String)>[], theirs = <(String, String)>[];
    var shown = false;
    for (final (_, f) in m.effects) {
      final home = m.tileHome(f);
      final stale = home.isNotEmpty && _shownBy(f) && !File('$home.showcase').existsSync();   // (drawn before AbsentMotion's table was read: drawn again by it)
      if (home.isEmpty || (!stale && m.picture(f) == home)) continue;
      if (!stale && File(home).existsSync()) {
        m.tileDrawn(home, same: File('$home.same').existsSync());
        shown = true;
      } else if (_asked.add(home)) {
        (f.startsWith(own) ? mine : theirs).add((f, home));
      }
    }
    if (shown) changed();
    if (Platform.environment['FLUTTER_TEST'] == 'true') return;   // (flutter test's widget tests: none drawn behind them, seven hundred pictures on the GPU while they time the window; flutter drive's draw them)
    _queue
      ..insertAll(0, mine)
      ..addAll(theirs);
    if (!_drawing) unawaited(_work(m, changed));
  }

  static Future<void> _work(Motolii m, void Function() changed) async {
    _drawing = true;
    final dir = motoliiDir;
    try {
      while (_queue.isNotEmpty) {
        final some = _queue.take(12).toList();
        _queue.removeRange(0, some.length);
        for (final (home, same) in await Isolate.run(() => _draw(dir, some))) {
          m.tileDrawn(home, same: same);
        }
        changed();
      }
    } finally {
      _drawing = false;
    }
  }

  static List<(String, bool)> _draw(String dir, List<(String, String)> files) {
    final folder = Directory.systemTemp.createTempSync('motolii_tiles'), drawn = <(String, bool)>[];
    final m = Motolii.open('${folder.path}/tiles.motolii', wide: 512, high: 512, dir: dir);
    if (m == null) return drawn;
    final photo = m.add('picture', '$dir/fx_sample.jpg');
    m
      ..set(photo, 'size', 3.6)
      ..key(photo, 'x', 0, -0.1)
      ..key(photo, 'x', 2, 0.1);
    final aa = m.add('text', '/System/Library/Fonts/Supplemental/Arial Black.ttf');
    m
      ..say(aa, 'text', 'Aa')
      ..say(aa, 'fill', 'ffffff')
      ..set(aa, 'size', 1)
      ..set(aa, 'y', -0.55);
    List<String> shoot(String home, {bool plays = false}) {   // (its eight pictures, made small; plays: one that keeps its frame before (ISF's PERSISTENT) drawn every frame from the start, the eight kept as it passes them, as fx_tiles.sh warms one)
      final made = <String>[];
      if (plays) {
        for (var n = 1, k = 0; k < moments.length && n <= (moments.last * 60).round(); n++) {
          final keep = n == (moments[k] * 60).round();
          if (m.at(n / 60) && m.still(keep ? _drawn(home, k) : '${folder.path}/warm.png') && keep) made.add(_drawn(home, k));
          if (keep) k++;
        }
      } else {
        made.addAll([for (final (k, at) in moments.indexed) if (m.at(at) && m.still(_drawn(home, k))) _drawn(home, k)]);
      }
      if (made.isNotEmpty) Process.runSync('sips', ['-Z', '192', ...made]);
      return made;
    }
    final sample = shoot('${folder.path}/sample.png');   // (the sample as it is, to tell a tile that shows nothing done)
    final shows = table(dir);
    for (final (f, home) in files) {
      try {
        Directory(File(home).parent.path).createSync(recursive: true);
      } catch (_) {
        continue;
      }
      final whole = (f.endsWith('.fs') || f.endsWith('.wgsl') || f.endsWith('.json')) && m.say(0, 'script', f);   // (.json: Filament's own settings, laid on the stage's view: viewer_settings.h)   // (a screen effect; the stage takes an AviUtl2 script too, but draws none of it: motolii.h, a thing's)
      final onIt = !whole && m.say(photo, 'fx', f);   // (as a thing's: as edit.sh says it)
      if (whole) {   // (a transition, ISF's progress, seen going from 0 to 1 across its pictures; a file without one has no such value: nothing)
        final progress = 'js_${f.split('/').last.replaceFirst(RegExp(r'\.[^.]*$'), '').replaceAll(RegExp(r'[^A-Za-z0-9_]'), '_')}_progress';   // (doc_values.h doc_screen_knob)
        m
          ..key(0, progress, moments.first, 0)
          ..key(0, progress, moments.last, 1);
      }
      final plays = f.endsWith('.fs') && RegExp(r'"PERSISTENT"\s*:\s*(true|1)').hasMatch(File(f).readAsStringSync());
      final show = f.endsWith('.wgsl') ? showcase(f, File(f).readAsStringSync(), shows) : null;
      for (final MapEntry(:key, :value) in (show?.knobs ?? const <String, double>{}).entries) {
        m.set(whole ? 0 : photo, key, value);   // (as AbsentMotion shows it: its table's values, not the file's defaults)
      }
      var made = shoot(home, plays: plays);
      if (whole) m.say(0, 'script', 'none');
      if (onIt) m.say(photo, 'fx', 'none');
      if (made.length < moments.length) made = shoot(home);   // (put on but not drawn, a script that stops: the sample as it is)
      if (made.length < moments.length) continue;
      final same = sample.length == moments.length && _unseen(home, folder.path);
      if (show?.split case final at? when sample.length == moments.length) {   // (before at its left, as AbsentMotion's effect_thumbs.py lays the photo over it up to there, a white line where they meet)
        for (final (k, _) in moments.indexed) {
          Process.runSync('ffmpeg', ['-hide_banner', '-loglevel', 'error', '-y', '-i', _drawn(home, k), '-i', '${folder.path}/sample_$k.png', '-filter_complex',
              '[1:v]crop=iw*$at:ih:0:0[b];[0:v][b]overlay=0:0,drawbox=x=iw*$at-2:y=0:w=2:h=ih:color=white:t=fill', '${folder.path}/split.png']);
          if (File('${folder.path}/split.png').existsSync()) File('${folder.path}/split.png').renameSync(_drawn(home, k));
        }
      }
      File(_drawn(home, still)).copySync(home);
      for (final (k, _) in moments.indexed) {
        Process.runSync('sips', ['-s', 'format', 'jpeg', '-s', 'formatOptions', '70', _drawn(home, k), '--out', frame(home, k)]);
        File(_drawn(home, k)).deleteSync();
      }
      try {
        same ? File('$home.same').writeAsStringSync('') : File('$home.same').deleteSync();
      } catch (_) {}
      if (show != null) File('$home.showcase').writeAsStringSync('');   // (drawn as AbsentMotion shows it)
      drawn.add((home, same));
    }
    m.close();
    try {
      folder.deleteSync(recursive: true);
    } catch (_) {}
    return drawn;
  }

  /// Whether none of its pictures is seen to differ from the sample's: ffmpeg's psnr, the worst of the eight over 40 dB
  /// (measured on these tiles: the ones that show nothing all over 45 dB, the ones that show something under 40).
  static bool _unseen(String home, String sample) {
    final ProcessResult r;
    try {
      r = Process.runSync('ffmpeg', ['-hide_banner', '-i', '${home.substring(0, home.length - 4)}_%d.png', '-i', '$sample/sample_%d.png', '-lavfi', 'psnr', '-f', 'null', '-']);
    } on ProcessException {
      return false;   // (no ffmpeg where the window was opened from: not said)
    }
    final min = RegExp(r'min:(inf|[\d.]+)').firstMatch('${r.stderr}')?.group(1);
    return min != null && (min == 'inf' || double.parse(min) > 40);
  }
}
