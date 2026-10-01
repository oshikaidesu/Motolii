import 'package:flutter/widgets.dart';

import '../../../hf/bp/common.dart';
import '../../../hf/insp/panel.dart';
import '../../../panels/gradient_inspector.dart' show GradientInspector;
import '../../../session/editor_session.dart';
import '../../../session/read_model.dart';
import 'session_layer_rows.dart';

bool _isFillRow(Map<String, dynamic> r) {
  final id = '${r['id']}';
  return id.startsWith('fill.') && !id.startsWith('fill.stop.');
}

/// The Fill card's New face: the same [GradientInspector] Classic uses (a shape's fill mode and stops), and the
/// rest of the fill rows as generic Toys over [SessionLayerRowsStore].
class NewFill extends StatefulWidget {
  const NewFill({super.key, required this.controller, required this.layer});
  final EditorSession controller;
  final Map<String, dynamic> layer;
  @override
  State<NewFill> createState() => _NewFillState();
}

class _NewFillState extends State<NewFill> {
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'capabilities', 'documentRevision'];
  EditorSession get c => widget.controller;
  late SessionLayerRowsStore store = SessionLayerRowsStore(c, widget.layer['id'] as int, _isFillRow);

  @override
  void initState() {
    super.initState();
    c.slice('newFill', _watched).addListener(store.absorb);
    c.rendered.addListener(store.absorb);
  }

  @override
  void didUpdateWidget(NewFill old) {
    super.didUpdateWidget(old);
    if (old.layer['id'] != widget.layer['id']) {
      store.dispose();
      store = SessionLayerRowsStore(c, widget.layer['id'] as int, _isFillRow);
    } else {
      store.absorb();
    }
  }

  @override
  void dispose() {
    c.slice('newFill', _watched).removeListener(store.absorb);
    c.rendered.removeListener(store.absorb);
    store.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final layerId = widget.layer['id'] as int;
    return Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
      Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('FILL', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.2))),
      GradientInspector(key: ValueKey('fill:$layerId'), controller: c, layer: widget.layer, fill: panelMap(widget.layer['fill'])),
      const SizedBox(height: 6),
      ListenableBuilder(listenable: store, builder: (context, _) => store.rows.isEmpty ? const SizedBox.shrink() : ParamSheet(store, thingId: 'fill:$layerId')),
    ]);
  }
}
