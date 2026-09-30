import 'package:flutter/widgets.dart';

import '../../../hf/bp/common.dart';
import '../../../panels/inspector.dart' show InspectorPanel, InspectorInstruments;
import '../../../session/editor_session.dart';
import '../../../session/read_model.dart';
import '../../../live_hf/adapters/effects_card.dart';
import '../../../live_hf/adapters/layout.dart';
import '../../../live_hf/adapters/transform.dart';
import '../../../hf/metrics.dart' show Surface;

/// The Inspector's own host: identity header, and the card list (Transform — World inside it, Layout, Effects) —
/// the New face for the header/scroll/card chrome Classic's `InspectorPanel` draws itself. Text, Fill and Matte
/// (the rich text editor, the gradient editor) have no New face yet: a layer that carries any of them still opens
/// the full Classic panel, with the same [instruments] passed through, so Transform/World/Effects stay New even
/// there — only that one layer's outer chrome is Classic, not the operations.
class NewInspectorPanel extends StatefulWidget {
  const NewInspectorPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<NewInspectorPanel> createState() => _NewInspectorPanelState();
}

class _NewInspectorPanelState extends State<NewInspectorPanel> {
  static const _watched = ['layers', 'selectedId', 'selectedIds', 'animate', 'capabilities', 'contentRevision', 'documentRevision'];
  EditorSession get c => widget.controller;

  InspectorInstruments get _instruments => InspectorInstruments(
        transform: (context, controller) => NewTransform(controller: controller),
        worldInTransform: true,
        layout: (context, controller, layer) => NewLayout(controller: controller, layer: layer),
        effectParams: null,
        effectCard: (context, controller, layer, effect, index, count) => NewEffectCard(
          key: ValueKey('new-effect-card:${layer['id']}:${effect['id']}'),
          controller: controller,
          layer: layer,
          effect: effect,
          index: index,
          count: count,
        ),
      );

  @override
  void initState() {
    super.initState();
    c.slice('newInspector', _watched).addListener(_absorb);
    c.rendered.addListener(_absorb);
  }

  @override
  void dispose() {
    c.slice('newInspector', _watched).removeListener(_absorb);
    c.rendered.removeListener(_absorb);
    super.dispose();
  }

  void _absorb() {
    if (mounted) setState(() {});
  }

  Map<String, dynamic>? get _active {
    final id = c.activeLayer?['id'];
    return c.liveLayers().where((l) => l['id'] == id).firstOrNull ?? c.liveLayers().firstOrNull;
  }

  bool get _multiple => c.selectedIds.length > 1;

  bool _hasContent(Map<String, dynamic> layer) {
    final text = panelMap(layer['text']);
    final matte = panelMap(layer['matte']);
    return text.isNotEmpty || matte.isNotEmpty || (layer['kind'] == 'Shape' && layer['fill'] is Map);
  }

  bool _hasLayout(Map<String, dynamic> layer) {
    if (_multiple) return false;
    if (layer['kind'] == 'Group') return true;
    return panelRows(layer['properties']).any((r) => r['id'] == 'layout.position_type');
  }

  @override
  Widget build(BuildContext context) {
    final layer = _active;
    if (layer == null) {
      return DecoratedBox(decoration: const BoxDecoration(color: Surface.base), child: Center(child: Text('Nothing selected', style: sans(11, c: Surface.muted))));
    }
    // A kind with no New face yet keeps the full Classic host, with the same New instruments underneath.
    if (!_multiple && (layer['kind'] == 'Camera' || layer['kind'] == 'Stage' || _hasContent(layer))) {
      return InspectorPanel(controller: c, instruments: _instruments);
    }
    return DecoratedBox(
      decoration: const BoxDecoration(color: Surface.base),
      child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
        Expanded(
          child: SingleChildScrollView(
            key: const ValueKey('new-inspector-scroll'),
            padding: const EdgeInsets.all(10),
            child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
              NewTransform(controller: c),
              if (_hasLayout(layer)) ...[
                const SizedBox(height: 10),
                Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('LAYOUT', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.2))),
                NewLayout(controller: c, layer: layer),
              ],
              if (panelRows(layer['effects']).isNotEmpty) ...[
                const SizedBox(height: 10),
                Padding(padding: const EdgeInsets.only(bottom: 4), child: Text('EFFECTS', style: sans(9, c: const Color(0xFF7E7F86), w: FontWeight.w600, ls: 1.2))),
                for (final (i, effect) in panelRows(layer['effects']).indexed)
                  NewEffectCard(
                    key: ValueKey('new-effect-card:${layer['id']}:${effect['id']}'),
                    controller: c,
                    layer: layer,
                    effect: effect,
                    index: i,
                    count: panelRows(layer['effects']).length,
                  ),
              ],
            ]),
          ),
        ),
      ]),
    );
  }
}
