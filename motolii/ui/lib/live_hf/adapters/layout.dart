import 'package:flutter/widgets.dart';

import '../../hf/insp/layout.dart';
import '../../session/editor_session.dart';
import 'layout_store.dart';
import 'inspector_session.dart';

/// The Layout Instrument for the Inspector's Layout card: one layer's `layout.*` rows on the real session.
class NewLayout extends StatefulWidget {
  const NewLayout({super.key, required this.controller, required this.layer});
  final EditorSession controller;
  final Map<String, dynamic> layer;
  @override
  State<NewLayout> createState() => _NewLayoutState();
}

class _NewLayoutState extends State<NewLayout> {
  EditorSession get c => widget.controller;
  SessionLayoutStore get store => InspectorSession.of(c).layout(widget.layer);

  @override
  Widget build(BuildContext context) => LayoutInstrument(store, title: '${widget.layer['name'] ?? 'Group'}', embedded: true);
}
