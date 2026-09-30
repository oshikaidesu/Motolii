import 'package:flutter/widgets.dart';

import '../../../browser/classify.dart';
import '../../../browser/parts.dart' show DockedPanel;
import '../../../browser/create/shelf.dart';
import '../../../effects/shelf.dart';
import '../../../browser/search.dart';
import '../../../hf/bp/shelf_grid.dart';
import '../../../browser/panel_chrome.dart' show GlyphBox;
import '../../../hf/glyphs.dart';
import '../../../browser/seat.dart';
import '../../../session/editor_session.dart';
import '../../../panels/browser.dart' show BrowserSize;
import 'shelf_host.dart';
import 'shelf_user.dart';
import 'shelf_seat.dart';
import 'shelf_things.dart';

/// One shelf of the production Browser drawn by the finished Browser bodies. The shelf is the owner of what is listed
/// and what a tile does; the body is how it looks and folds. Search and class are kept here, so a change of what the
/// shelf lists redraws the views without losing what was typed or chosen.
class ShelfPanel extends StatefulWidget {
  const ShelfPanel({super.key, required this.controller, required this.name});
  final EditorSession controller;
  final String name;
  @override
  State<ShelfPanel> createState() => _ShelfPanelState();
}

class _ShelfPanelState extends State<ShelfPanel> {
  late final ShelfHost host = ShelfHost(widget.controller, shelfNamed(widget.name), () => context, () => mounted, tileScaleOf: () => BrowserSize.tile(widget.controller) / BrowserSize.base);
  final search = SearchCapability();
  final classify = ClassifyCapability();
  late final ShelfUser mine = ShelfUser(widget.controller, widget.name);
  late final ShelfSeat seat = ShelfSeat(host, search, mine: mine);

  @override
  void dispose() {
    seat.dispose();
    mine.dispose();
    host.dispose();
    search.dispose();
    classify.dispose();
    super.dispose();
  }

  String get _noun => switch (widget.name) { 'Media' => 'clip', 'Colors' => 'colour', 'Fonts' => 'typeface', _ => 'file' };

  HG get _glyph => switch (widget.name) { 'Media' => HG.image, 'Colors' => HG.color, 'Fonts' => HG.text, _ => HG.folder };

  String get _panelId => widget.name.toLowerCase();

  Map<String, dynamic>? Function(Map<String, dynamic>, String)? get _face => widget.name == 'Create' ? createFace : null;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: host,
        builder: (context, _) {
          final catalog = ShelfCatalog(host, _panelId, faceOf: _face, own: mine);
          seat.catalog = catalog;
          final key = ValueKey('${widget.name}:${catalog.signature}');
          return DockedPanel(
            child: BrowserSeatScope(
            seat: seat,
            child: switch (widget.name) {
              'Create' => CreatePanel(key: key, catalog: catalog.catalog, user: seat.user, search: search, classify: classify),
              'Effects' => EffectsPanel(null, key: key, catalog: catalog.catalog, user: seat.user, search: search, classify: classify),
              _ => ShelfGridPanel(
                  key: key,
                  title: widget.name,
                  noun: _noun,
                  icon: GlyphBox(_glyph, size: 22, color: const Color(0xFFF2F2F4)),
                  panelId: _panelId,
                  catalog: catalog.catalog,
                  user: seat.user,
                  search: search,
                  classify: classify,
                  sections: widget.name == 'Colors',
                ),
            },
          ));
        },
      );
}
