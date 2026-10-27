// Every effect on the shelf has a tile. One with no picture (not drawn by filament/fx_tiles.sh, or put in the maker's own
// folder) gets one drawn: the sample fx_tiles.sh draws the bundled ones on (the balloon, fx_sample.jpg, drifting, a
// white Aa over it), on a document of its own in another isolate (as create_preview.dart reckons the recipes), written
// where the inside says one drawn for it goes (its home: ~/Library/Caches/Motolii/tiles, named by the file and when it
// was written, doc_recipes.h doc_tile), then shown at once. As fx_tiles.sh puts it: on the whole picture; one that is
// not a whole-screen effect on the photo, as a thing's (an AviUtl2 script, ISF one of those); one that is neither (a
// point cloud's, on no cloud) the sample as it is: no tile is left empty.
// The maker's own folder is watched while the window is open (dart:io's watch, FSEvents on a Mac): a file put in or taken
// out there and the shelves are read again (BRIDGE.md rule 8, the second half), and its tile drawn.
import 'dart:async';
import 'dart:io';
import 'dart:isolate';

import '../motolii.dart';

abstract final class EffectTiles {
  static final _asked = <String>{}, _queue = <(String, String)>[];
  static var _drawing = false;

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

  /// The effects on the shelf with no tile: one drawn before (this run, or another) shown at once; the rest put in line,
  /// the maker's own at its head (what was just put in), drawn a few at a time, each few shown when done.
  static void ask(Motolii m, void Function() changed) {
    final mine = <(String, String)>[], theirs = <(String, String)>[];
    var shown = false;
    for (final (_, f) in m.effects) {
      final home = m.tileHome(f);
      if (m.picture(f).isNotEmpty || home.isEmpty) continue;
      if (File(home).existsSync()) {
        m.tileDrawn(home);
        shown = true;
      } else if (_asked.add(home)) {
        (f.startsWith(own) ? mine : theirs).add((f, home));
      }
    }
    if (shown) changed();
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
        for (final home in await Isolate.run(() => _draw(dir, some))) {
          m.tileDrawn(home);
        }
        changed();
      }
    } finally {
      _drawing = false;
    }
  }

  static List<String> _draw(String dir, List<(String, String)> files) {
    final folder = Directory.systemTemp.createTempSync('motolii_tiles'), drawn = <String>[];
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
    bool onPhoto(String f) {   // (as a thing's: the window's own way of putting one on, ws_2.dart; taken off again by the same call)
      final ticket = m.later('fx_file', thing: photo, word: f);
      for (var k = 0; k < 500; k++) {
        final a = m.answer(ticket);
        if (a >= 0) return a == 1;
        sleep(const Duration(milliseconds: 10));
      }
      return false;
    }
    for (final (f, home) in files) {
      try {
        Directory(File(home).parent.path).createSync(recursive: true);
      } catch (_) {
        continue;
      }
      final whole = m.say(0, 'script', f), onIt = !whole && onPhoto(f);
      m.at(1.4);
      final took = m.still(home);
      if (whole) m.say(0, 'script', 'none');
      if (onIt) onPhoto(f);
      if (!took) m.still(home); // (put on but not drawn, a script that stops: the sample as it is)
      if (!File(home).existsSync()) continue;
      Process.runSync('sips', ['-Z', '256', home]);   // (drawn at twice the tile, made small, as fx_tiles.sh: the letters' edges stay clean)
      drawn.add(home);
    }
    m.close();
    try {
      folder.deleteSync(recursive: true);
    } catch (_) {}
    return drawn;
  }
}
