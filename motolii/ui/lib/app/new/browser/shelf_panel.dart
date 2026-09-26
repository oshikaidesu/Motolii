import 'package:flutter/widgets.dart';

import '../../../hf/bp/classify.dart';
import '../../../hf/bp/common.dart' show DockedPanel;
import '../../../hf/bp/create.dart';
import '../../../hf/bp/effects.dart';
import '../../../hf/bp/search.dart';
import '../../../hf/bp/seat.dart';
import '../../../hf/bp/things.dart';
import '../../../session/editor_session.dart';
import 'shelf_host.dart';
import 'shelf_seat.dart';
import 'shelf_things.dart';

/// One shelf of the production Browser drawn by the finished Browser bodies. The shelf is the owner of what is listed
/// and what a tile does; the body is how it looks and folds. Search and class are kept here, so a change of what the
/// shelf lists redraws the views without losing what was typed or chosen.
class ShelfPanel extends StatefulWidget {
  const ShelfPanel({super.key, required this.controller, required this.name, this.user});
  final EditorSession controller;
  final String name;
  final UserViews? user;
  @override
  State<ShelfPanel> createState() => _ShelfPanelState();
}

class _ShelfPanelState extends State<ShelfPanel> {
  late final ShelfHost host = ShelfHost(widget.controller, shelfNamed(widget.name), () => context, () => mounted);
  final search = SearchCapability();
  final classify = ClassifyCapability();
  late final ShelfSeat seat = ShelfSeat(host, search, user: widget.user ?? UserViews());

  @override
  void dispose() {
    seat.dispose();
    host.dispose();
    search.dispose();
    classify.dispose();
    super.dispose();
  }

  String get _panelId => widget.name.toLowerCase();

  Map<String, dynamic>? Function(Map<String, dynamic>, String)? get _face => widget.name == 'Create' ? createFace : null;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: host,
        builder: (context, _) {
          final catalog = ShelfCatalog(host, _panelId, faceOf: _face);
          seat.catalog = catalog;
          final key = ValueKey('${widget.name}:${catalog.signature}');
          return DockedPanel(
            child: BrowserSeatScope(
            seat: seat,
            child: switch (widget.name) {
              'Create' => CreatePanel(key: key, catalog: catalog.catalog, user: seat.user, search: search, classify: classify),
              _ => EffectsPanel(null, key: key, catalog: catalog.catalog, user: seat.user, search: search, classify: classify),
            },
          ));
        },
      );
}
