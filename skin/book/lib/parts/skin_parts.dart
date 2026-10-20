// A skin's drawn parts (KDE Plasma's and Kvantum's manner): the theme's `Parts` role names a folder, and an SVG there named for a
// part (`tp.eye.svg`, `glyph.select.svg`, `undo.svg` ...) is drawn in its place, in the colour the window gives that part; a
// `frame.<part>.svg` is stretched around that part in its own colours (SkinFrame). A part
// the folder does not have is the window's own drawing, so a skin may bring one picture or all of them. The names: skins/PARTS.md.
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/widgets.dart';
import 'package:flutter_svg/flutter_svg.dart';

import '../theme.dart';

abstract final class SkinParts {
  static final _pics = <String, PictureInfo>{};
  static final _slices = <String, double>{}, _frames = <String, ui.Image>{};
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
          final name = f.uri.pathSegments.last.replaceAll('.svg', '');
          next[name] = await vg.loadPicture(SvgFileLoader(f), null);
          if (RegExp(r'data-slice="([0-9.]+)"').firstMatch(f.readAsStringSync()) case final m?) _slices[name] = double.parse(m.group(1)!);
        } catch (_) {} // (a picture that does not read: the window's own drawing stays)
      }
    }
    if (dir != _dir) return; // (the theme moved on while these were read)
    if (next.isEmpty && _pics.isEmpty) return; // (no pictures before or now: nothing to paint again)
    for (final p in _pics.values) {
      p.picture.dispose();
    }
    for (final i in _frames.values) {
      i.dispose();
    }
    _frames.clear();
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

  /// Whether the skin brought a picture named [name].
  static bool has(String name) => _pics[name]?.size.isEmpty == false;

  /// Draws the skin's frame [name] around [r] as CSS's border-image does: its corners as drawn, its edges and middle
  /// stretched; the corners are the SVG's `data-slice` (a third of its smaller side when it says none). Its own colours.
  static void frame(Canvas c, Rect r, String name) {
    final p = _pics[name]!, k = 4.0; // (drawn four times finer than a point, so a frame stays sharp on any screen)
    final img = _frames[name] ??= () {
      final rec = ui.PictureRecorder();
      Canvas(rec)
        ..scale(k)
        ..drawPicture(p.picture);
      return rec.endRecording().toImageSync((p.size.width * k).ceil(), (p.size.height * k).ceil());
    }();
    final sl = (_slices[name] ?? (p.size.shortestSide / 3)).clamp(0.0, p.size.shortestSide / 2 - .5);
    c.save();
    c.scale(1 / k);
    c.drawImageNine(img, Rect.fromLTRB(sl * k, sl * k, (p.size.width - sl) * k, (p.size.height - sl) * k), Rect.fromLTRB(r.left * k, r.top * k, r.right * k, r.bottom * k), Paint()..filterQuality = FilterQuality.medium);
    c.restore();
  }
}

/// A part's frame from the skin: `frame.<name>.svg` (`.hover` while a hand is on it, `.on` while it is on, when the skin
/// brings them) stretched around the part; none, the part's own [usual] decoration.
class SkinFrame extends StatelessWidget {
  const SkinFrame(this.name, {super.key, this.hover = false, this.on = false, required this.usual, required this.child});
  final String name;
  final bool hover, on;
  final Decoration usual;
  final Widget child;
  @override
  Widget build(BuildContext context) {
    final n = [if (on) 'frame.$name.on', if (hover) 'frame.$name.hover', 'frame.$name'].where(SkinParts.has).firstOrNull;
    return n == null ? DecoratedBox(decoration: usual, child: child) : CustomPaint(painter: _Frame(n), child: child);
  }
}

class _Frame extends CustomPainter {
  const _Frame(this.name);
  final String name;
  @override
  void paint(Canvas c, Size s) => SkinParts.frame(c, Offset.zero & s, name);
  @override
  bool shouldRepaint(_Frame o) => o.name != name;
}
