// A skin's drawn parts (KDE Plasma's and Kvantum's manner): the theme's `Parts` role names a folder, and an SVG there named for a
// part (`tp.eye.svg`, `glyph.select.svg`, `undo.svg` ...) is drawn in its place, in the colour the window gives that part. A part
// the folder does not have is the window's own drawing, so a skin may bring one picture or all of them. The names: skins/PARTS.md.
import 'dart:io';

import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../theme.dart';

abstract final class SkinParts {
  static final _pics = <String, PictureInfo>{};
  static String? _dir;

  /// The folder the theme names, read again when it changes; the window is painted again once its pictures are in.
  static void follow() {
    void read() {
      final dir = ColorTheme.place(ColorTheme.word('Parts', ''));
      if (dir == _dir) return;
      _dir = dir;
      _load(dir);
    }

    ColorTheme.words.addListener(read);
    read();
  }

  static Future<void> _load(String dir) async {
    final next = <String, PictureInfo>{};
    final folder = Directory(dir);
    if (dir.isNotEmpty && folder.existsSync()) {
      for (final f in folder.listSync().whereType<File>().where((f) => f.path.endsWith('.svg'))) {
        try {
          next[f.uri.pathSegments.last.replaceAll('.svg', '')] = await vg.loadPicture(SvgFileLoader(f), null);
        } catch (_) {} // (a picture that does not read: the window's own drawing stays)
      }
    }
    if (dir != _dir) return; // (the theme moved on while these were read)
    if (next.isEmpty && _pics.isEmpty) return; // (no pictures before or now: nothing to paint again)
    for (final p in _pics.values) {
      p.picture.dispose();
    }
    _pics
      ..clear()
      ..addAll(next);
    WidgetsBinding.instance.reassembleApplication();
  }

  /// How many pictures the skin brought (read).
  static int get count => _pics.length;

  /// Draws the skin's picture for [name] filling [s] in [color] and says so; false: the skin has none, the caller draws its own.
  static bool draw(Canvas c, Size s, String name, Color color) {
    final p = _pics[name];
    if (p == null || p.size.isEmpty) return false;
    final k = (s.width / p.size.width) < (s.height / p.size.height) ? s.width / p.size.width : s.height / p.size.height;
    c.save();
    c.translate((s.width - p.size.width * k) / 2, (s.height - p.size.height * k) / 2);
    c.scale(k);
    c.saveLayer(Offset.zero & p.size, Paint()..colorFilter = ColorFilter.mode(color, BlendMode.srcIn));
    c.drawPicture(p.picture);
    c.restore();
    c.restore();
    return true;
  }
}
