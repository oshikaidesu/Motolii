import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show sans, kMuted;
import '../../session/editor_session.dart';
import 'camera.dart';
import 'effect.dart';
import 'effects_card.dart';
import 'layout.dart';
import 'transform.dart';
import 'inspector_session.dart';

/// The Inspector seat. Specialist editing requests open the corresponding real Dock panel; the Inspector itself
/// stays an Inspector instead of turning into a drawer.
class RightSeat extends StatefulWidget {
  const RightSeat({super.key, required this.c});
  final EditorSession c;
  @override
  State<RightSeat> createState() => _RightSeatState();
}

class _RightSeatState extends State<RightSeat> {
  // which Inspector is shown (a camera's, a layer's, its effect cards) follows what the layers are, not their values: the
  // number rows read the values through their own stores
  static const _watched = ['selectedId', 'selectedIds'];
  EditorSession get c => widget.c;

  @override
  void initState() {
    super.initState();
    c.deskDrawer.addListener(_changed);
    c.editingFocus.addListener(_focus);
    c.slice('rightSeat', _watched, derived: c.layerShape).addListener(_changed);
  }

  @override
  void dispose() {
    c.deskDrawer.removeListener(_changed);
    c.editingFocus.removeListener(_focus);
    c.slice('rightSeat', _watched, derived: c.layerShape).removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() {});

  /// Effect cards folded to their header, per layer and effect (not kept with the document, as in Classic).
  final _folded = <String>{};

  void _focus() {
    final focus = c.editingFocus.value;
    final layer = c.layers.where((l) => l['id'] == focus['layer']).firstOrNull;
    final desk = focus['property'] == 'keyframes'
        ? 'Ease'
        : (layer?['kind'] == 'Camera'
              ? 'Depth'
              : (focus['property'] == 'blendMode' ? 'Blend' : null));
    if (desk != null) c.placePanel(desk, 'show');
  }

  @override
  Widget build(BuildContext context) => _inspector();

  /// The Inspector for what is selected: a camera layer's own instrument, else Transform, followed by the Layout
  /// card of a group or laid-out child and the layer's effect cards, as the Classic Inspector lists them.
  Widget _inspector() {
    final transform = NewTransform(controller: c, showHeader: true);
    final Map<String, dynamic> layer;
    final bool stage, layout;
    final List<Map<String, dynamic>> effects;
    switch (InspectorSession.of(c).subject) {
      case InspectorEmpty(:final why):
        return _Empty(why);
      case InspectorCamera(layer: final id):
        return LiveCamera(key: ValueKey(id), c: c, layer: id);
      case InspectorLayer(layer: null):
        return transform;
      case InspectorLayer(layer: final l?, stage: final st, layout: final lo, effects: final ef):
        (layer, stage, layout, effects) = (l, st, lo, ef);
    }
    if (!layout && !stage && effects.isEmpty) return transform;
    return SingleChildScrollView(
      // another layer starts at the top (Classic IN-006)
      key: ValueKey('seat-scroll:${layer['id']}'),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          transform,
          if (stage)
            Padding(
              padding: const EdgeInsets.fromLTRB(9, 7.5, 9, 0),
              child: Column(crossAxisAlignment: CrossAxisAlignment.stretch, children: [
                Padding(padding: const EdgeInsets.only(bottom: 6), child: Text('Stage', key: const ValueKey('seat-stage'), style: sans(11, c: kMuted))),
                LayerRowsSheet(key: ValueKey('seat-stage:${layer['id']}'), controller: c, layerId: layer['id'] as int, prefix: 'stage.'),
              ]),
            ),
          if (layout)
            NewLayout(
              key: ValueKey('seat-layout:${layer['id']}'),
              controller: c,
              layer: layer,
            ),
          if (effects.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(9, 7.5, 9, 12),
              // Drag a card by its header to apply it earlier or later (Classic's reorder grip, `moveEffect`).
              child: ReorderableList(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: effects.length,
                onReorderItem: (from, to) {
                  if (from != to) InspectorSession.of(c).moveEffect(layer['id'] as int, effects[from]['id'], to);
                },
                itemBuilder: (context, i) => NewEffectCard(
                  key: ValueKey('seat-effect:${layer['id']}:${effects[i]['id']}'),
                  controller: c,
                  layer: layer,
                  effect: effects[i],
                  index: i,
                  count: effects.length,
                  reorder: c.supports('moveEffect') && layer['frozen'] != true,
                  folded: _folded.contains('${layer['id']}:${effects[i]['id']}'),
                  onFold: () => setState(() {
                    final k = '${layer['id']}:${effects[i]['id']}';
                    _folded.contains(k) ? _folded.remove(k) : _folded.add(k);
                  }),
                ),
              ),
            ),
        ],
      ),
    );
  }
}

class _Empty extends StatelessWidget {
  const _Empty(this.text);
  final String text;
  @override
  Widget build(BuildContext context) => Center(
    child: Text(text, key: const ValueKey('seat-empty'), style: sans(11, c: kMuted)),
  );
}
