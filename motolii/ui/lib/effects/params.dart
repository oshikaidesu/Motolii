import 'package:flutter/widgets.dart';

import '../hf/insp/panel.dart';
import '../session/editor_session.dart';
import 'store.dart';
import '../live_hf/adapters/inspector_session.dart';

/// One effect's parameters for the Inspector's effect card: the generic Toys over the effect's declared rows.
class NewEffectParams extends StatefulWidget {
  const NewEffectParams({super.key, required this.controller, required this.layerId, required this.effectId, this.advancedOpen = false});
  final EditorSession controller;
  final int layerId;
  final Object effectId;
  final bool advancedOpen;
  @override
  State<NewEffectParams> createState() => _NewEffectParamsState();
}

class _NewEffectParamsState extends State<NewEffectParams> {
  EditorSession get c => widget.controller;
  SessionEffectStore get store => InspectorSession.of(c).effect(widget.layerId, widget.effectId);

  @override
  Widget build(BuildContext context) => ParamSheet(store, thingId: '${widget.effectId}', advancedOpen: widget.advancedOpen);
}

/// A layer's own rows under a prefix (a Stage layer's `stage.` margins) as the same sheet, written the same way.
class LayerRowsSheet extends StatefulWidget {
  const LayerRowsSheet({super.key, required this.controller, required this.layerId, required this.prefix});
  final EditorSession controller;
  final int layerId;
  final String prefix;
  @override
  State<LayerRowsSheet> createState() => _LayerRowsSheetState();
}

class _LayerRowsSheetState extends State<LayerRowsSheet> {
  EditorSession get c => widget.controller;
  SessionEffectStore get store => InspectorSession.of(c).layerRows(widget.layerId, widget.prefix);

  @override
  Widget build(BuildContext context) => ParamSheet(store, thingId: '${widget.layerId}:${widget.prefix}');
}
