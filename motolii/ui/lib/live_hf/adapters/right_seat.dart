import 'package:flutter/widgets.dart';

import '../../hf/bp/common.dart' show sans, kMuted;
import '../../session/editor_session.dart';
import '../../session/read_model.dart';
import 'camera.dart';
import 'effects_card.dart';
import 'layout.dart';
import 'layout_store.dart';
import 'transform.dart';

/// The Inspector seat. Specialist editing requests open the corresponding real Dock panel; the Inspector itself
/// stays an Inspector instead of turning into a drawer.
class RightSeat extends StatefulWidget {
  const RightSeat({super.key, required this.c});
  final EditorSession c;
  @override
  State<RightSeat> createState() => _RightSeatState();
}

class _RightSeatState extends State<RightSeat> {
  static const _watched = [
    'selectedId',
    'selectedIds',
    'layers',
    'documentRevision',
  ];
  EditorSession get c => widget.c;

  @override
  void initState() {
    super.initState();
    c.deskDrawer.addListener(_changed);
    c.editingFocus.addListener(_focus);
    c.slice('rightSeat', _watched).addListener(_changed);
  }

  @override
  void dispose() {
    c.deskDrawer.removeListener(_changed);
    c.editingFocus.removeListener(_focus);
    c.slice('rightSeat', _watched).removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() {});

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
    final active = c.activeLayer;
    if (active != null &&
        active['kind'] == 'Camera' &&
        c.selectedIds.length <= 1)
      return LiveCamera(
        key: ValueKey(active['id']),
        c: c,
        layer: active['id'] as int,
      );
    final transform = NewTransform(controller: c, showHeader: true);
    final layer = active == null
        ? null
        : c.liveLayers().where((l) => l['id'] == active['id']).firstOrNull ??
              active;
    // nothing picked: say why the panel is empty (Classic IN-001)
    if (layer == null && c.layers.isEmpty) return const _Empty('No layers yet');
    if (layer == null) return c.selectedIds.isEmpty ? const _Empty('Select a layer') : transform;
    final layout =
        c.selectedIds.length <= 1 &&
        (layer['kind'] == 'Group' || SessionLayoutStore.isChild(layer));
    final effects = panelRows(layer['effects']);
    if (!layout && effects.isEmpty) return transform;
    return SingleChildScrollView(
      // another layer starts at the top (Classic IN-006)
      key: ValueKey('seat-scroll:${layer['id']}'),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          transform,
          if (layout)
            NewLayout(
              key: ValueKey('seat-layout:${layer['id']}'),
              controller: c,
              layer: layer,
            ),
          if (effects.isNotEmpty)
            Padding(
              padding: const EdgeInsets.fromLTRB(12, 10, 12, 16),
              // Drag a card by its header to apply it earlier or later (Classic's reorder grip, `moveEffect`).
              child: ReorderableList(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: effects.length,
                onReorderItem: (from, to) {
                  if (from != to && c.supports('moveEffect'))
                    c.command('moveEffect', {
                      'layer': layer['id'],
                      'id': effects[from]['id'],
                      'to': to,
                    });
                },
                itemBuilder: (context, i) => NewEffectCard(
                  key: ValueKey('seat-effect:${layer['id']}:${effects[i]['id']}'),
                  controller: c,
                  layer: layer,
                  effect: effects[i],
                  index: i,
                  count: effects.length,
                  reorder: c.supports('moveEffect') && layer['frozen'] != true,
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
    child: Text(text, key: const ValueKey('seat-empty'), style: sans(12, c: kMuted)),
  );
}
