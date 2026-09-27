import 'package:flutter/widgets.dart';

import '../../session/editor_session.dart';
import '../../session/read_model.dart';
import 'blend.dart';
import 'camera.dart';
import 'depth.dart';
import 'ease.dart';
import 'effects_card.dart';
import 'history.dart';
import 'layout.dart';
import 'layout_store.dart';
import 'notes.dart';
import 'transform.dart';

/// Desks the right seat can show over the session.
const liveDesks = {'Blend', 'Depth', 'History', 'Ease', 'Notes'};

/// The right seat: the desk the session has open (`deskDrawer`), else the Inspector. Editing a value that has a
/// specialist opens it — keys open Ease, a camera layer's value Depth, a blend mode Blend.
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
    if (desk != null) c.deskDrawer.value = desk;
  }

  @override
  Widget build(BuildContext context) => switch (c.deskDrawer.value) {
    'Blend' => NewBlend(controller: c),
    'Depth' => NewDepth(controller: c),
    'History' => NewHistory(controller: c),
    'Ease' => LiveEase(c: c),
    'Notes' => LiveNotes(c: c),
    _ => _inspector(),
  };

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
    if (layer == null) return transform;
    final layout =
        c.selectedIds.length <= 1 &&
        (layer['kind'] == 'Group' || SessionLayoutStore.isChild(layer));
    final effects = panelRows(layer['effects']);
    if (!layout && effects.isEmpty) return transform;
    return SingleChildScrollView(
      key: const ValueKey('seat-scroll'),
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
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  for (final (i, effect) in effects.indexed)
                    NewEffectCard(
                      key: ValueKey(
                        'seat-effect:${layer['id']}:${effect['id']}',
                      ),
                      controller: c,
                      layer: layer,
                      effect: effect,
                      index: i,
                      count: effects.length,
                    ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}
