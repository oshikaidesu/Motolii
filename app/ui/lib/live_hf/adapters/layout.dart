import 'package:flutter/widgets.dart';

import '../../hf/insp/layout.dart';
import '../../session/editor_session.dart';
import 'layout_store.dart';

/// The Layout Instrument for the Inspector's Layout card: one layer's `layout.*` rows on the real session.
class NewLayout extends StatefulWidget {
  const NewLayout({super.key, required this.controller, required this.layer});
  final EditorSession controller;
  final Map<String, dynamic> layer;
  @override
  State<NewLayout> createState() => _NewLayoutState();
}

class _NewLayoutState extends State<NewLayout> {
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'documentRevision'];
  EditorSession get c => widget.controller;
  late SessionLayoutStore store = SessionLayoutStore(c, widget.layer);

  @override
  void initState() {
    super.initState();
    c.slice('newLayout', _watched).addListener(store.absorb);
    c.rendered.addListener(store.absorb);
  }

  @override
  void didUpdateWidget(NewLayout old) {
    super.didUpdateWidget(old);
    if (old.layer['id'] != widget.layer['id']) {
      store.dispose();
      store = SessionLayoutStore(c, widget.layer);
    } else {
      store.absorb();
    }
  }

  @override
  void dispose() {
    c.slice('newLayout', _watched).removeListener(store.absorb);
    c.rendered.removeListener(store.absorb);
    store.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => LayoutInstrument(store, title: '${widget.layer['name'] ?? 'Group'}', embedded: true);
}
