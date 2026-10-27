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
import 'dart:io';
import 'dart:isolate';

import '../motolii.dart';

abstract final class EffectTiles {
  static final _asked = <String>{}, _queue = <(String, String)>[];
  static var _drawing = false;

  /// The moments its pictures are drawn at, and the one its still is (fx_tiles.sh's 1.4 s).
  static const moments = [0.1, 0.357, 0.614, 0.871, 1.129, 1.386, 1.643, 1.9];
  static const still = 5;

  /// Its moving pictures: home_0.jpg … home_7.jpg beside its still (none for a picture not drawn here).
  static String frame(String home, int k) => '${home.substring(0, home.length - 4)}_$k.jpg';
  static String _drawn(String home, int k) => '${home.substring(0, home.length - 4)}_$k.png';   // (as the inside draws it, before it is made small)

  /// The maker's own folder: MOTOLII_OWN, or ~/Library/Application Support/Motolii (as the inside's doc_folders).
  static String get own => Platform.environment['MOTOLII_OWN'] ?? '${Platform.environment['HOME']}/Library/Application Support/Motolii';

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
    return Directory(own).watch(recursive: true).listen((_) {
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
      if (home.isEmpty || m.picture(f) == home) continue;
      if (File(home).existsSync()) {
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
    List<String> shoot(String home) {   // (its eight pictures, made small)
      final made = [for (final (k, at) in moments.indexed) if (m.at(at) && m.still(_drawn(home, k))) _drawn(home, k)];
      if (made.isNotEmpty) Process.runSync('sips', ['-Z', '192', ...made]);
      return made;
    }
    final sample = shoot('${folder.path}/sample.png');   // (the sample as it is, to tell a tile that shows nothing done)
    for (final (f, home) in files) {
      try {
        Directory(File(home).parent.path).createSync(recursive: true);
      } catch (_) {
        continue;
      }
      final whole = (f.endsWith('.fs') || f.endsWith('.wgsl')) && m.say(0, 'script', f);   // (a screen effect; the stage takes an AviUtl2 script too, but draws none of it: motolii.h, a thing's)
      final onIt = !whole && m.say(photo, 'fx', f);   // (as a thing's: as edit.sh says it)
      if (whole) {   // (a transition, ISF's progress, seen going from 0 to 1 across its pictures; a file without one has no such value: nothing)
        final progress = 'js_${f.split('/').last.replaceFirst(RegExp(r'\.[^.]*$'), '').replaceAll(RegExp(r'[^A-Za-z0-9_]'), '_')}_progress';   // (doc_values.h doc_screen_knob)
        m
          ..key(0, progress, moments.first, 0)
          ..key(0, progress, moments.last, 1);
      }
      var made = shoot(home);
      if (whole) m.say(0, 'script', 'none');
      if (onIt) m.say(photo, 'fx', 'none');
      if (made.length < moments.length) made = shoot(home);   // (put on but not drawn, a script that stops: the sample as it is)
      if (made.length < moments.length) continue;
      File(_drawn(home, still)).copySync(home);
      final same = sample.length == moments.length && _unseen(home, folder.path);
      for (final (k, _) in moments.indexed) {
        Process.runSync('sips', ['-s', 'format', 'jpeg', '-s', 'formatOptions', '70', _drawn(home, k), '--out', frame(home, k)]);
        File(_drawn(home, k)).deleteSync();
      }
      try {
        same ? File('$home.same').writeAsStringSync('') : File('$home.same').deleteSync();
      } catch (_) {}
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
