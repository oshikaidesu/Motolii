import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';

/// One page of the notebook and whether it is the one open.
class NotesPage {
  const NotesPage(this.id, this.title, {required this.active});
  final String id, title;
  final bool active;
}

/// How the cards on the canvas look: the ground behind them, the colour of a written note (one per card in turn), the
/// pill of a reference, and the ink. With no look a card keeps the classic desk's own.
class NoteLook {
  const NoteLook({required this.ground, required this.dots, required this.tints, required this.reference, required this.ink, required this.accent, required this.raised});
  final Color ground, dots, reference, ink, accent, raised;
  final List<Color> tints;
}

/// What the Notes desk knows, apart from how it is drawn: the pages, the canvas with its cards (which the desk builds,
/// because it owns their editing, moving and resizing), and every action of the desk. Each skin places these.
class NotesView {
  const NotesView({
    required this.pages,
    required this.pageTitle,
    required this.blocks,
    required this.canvas,
    required this.legacy,
    required this.transform,
    required this.selectPage,
    required this.newPage,
    required this.renamePage,
    required this.deletePage,
    required this.paste,
    required this.insertImage,
    required this.linkSelection,
    required this.resetView,
    required this.importLegacy,
    required this.setZoom,
  });
  final List<NotesPage> pages;
  final String? pageTitle;
  final int blocks;
  final Widget canvas;

  /// Earlier text or reference images the desk can carry over into a page.
  final bool legacy;
  final ValueNotifier<Matrix4> transform;
  final ValueChanged<String> selectPage, renamePage;
  final VoidCallback newPage, deletePage, paste, insertImage, linkSelection, resetView, importLegacy;
  final ValueChanged<double> setZoom;
}
