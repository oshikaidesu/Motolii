// The maker's own folder watched while the window is open (dart:io's watch, FSEvents on a Mac): a file put in or taken
// out there and the shelves are read again (BRIDGE.md rule 8, the second half). An effect of the maker's that has no
// picture gets one: the sample filament/fx_tiles.sh draws the bundled ones on (the balloon, fx_sample.jpg, drifting,
// a white Aa over it) with that effect on the whole picture, drawn on a document of its own in another isolate (as
// create_preview.dart reckons the recipes) and written beside the effect (effects/name.fs -> effects/name.png), as
// fx_tiles.sh writes the bundled ones. That write is a change in the folder too, and the shelf shows it.
import 'dart:async';
import 'dart:io';
import 'dart:isolate';

import '../motolii.dart';

abstract final class EffectTiles {
  static final _asked = <String>{};

  /// The maker's own folder: MOTOLII_OWN, or ~/Library/Application Support/Motolii (as the inside's doc_folders).
  static String get own => Platform.environment['MOTOLII_OWN'] ?? '${Platform.environment['HOME']}/Library/Application Support/Motolii';

  /// Watched until the subscription is cancelled; [changed] is told after the folders are read again.
  static StreamSubscription<FileSystemEvent>? watch(Motolii m, void Function() changed) {
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
      ask(m);
    }
    ask(m);
    return Directory(own).watch(recursive: true).listen((_) {
      soon?.cancel();
      soon = Timer(const Duration(milliseconds: 300), again); // (a copy is several events: read once it has settled)
    });
  }

  /// The maker's screen effects with no picture beside them, drawn once each.
  static void ask(Motolii m) {
    final todo = [
      for (final (n, f) in m.effects)
        if (m.picture(n).isEmpty && f.startsWith(own) && (f.endsWith('.fs') || f.endsWith('.wgsl')) && _asked.add(f)) f,
    ];
    if (todo.isEmpty) return;
    final dir = motoliiDir;
    unawaited(Isolate.run(() => _draw(dir, todo)));
  }

  static void _draw(String dir, List<String> files) {
    final folder = Directory.systemTemp.createTempSync('motolii_tiles');
    final m = Motolii.open('${folder.path}/tiles.motolii', wide: 512, high: 512, dir: dir);
    if (m == null) return;
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
    for (final f in files) {
      final png = '${f.substring(0, f.lastIndexOf('.'))}.png';
      if (File(png).existsSync() || !m.say(0, 'script', f)) continue; // (one it cannot take: no picture, as before)
      m
        ..at(1.4)
        ..still(png);
    }
    m.close();
    try {
      folder.deleteSync(recursive: true);
    } catch (_) {}
  }
}
