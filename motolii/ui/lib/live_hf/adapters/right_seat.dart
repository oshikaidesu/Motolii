import 'package:flutter/widgets.dart';

import '../../session/editor_session.dart';
import 'blend.dart';
import 'camera.dart';
import 'depth.dart';
import 'ease.dart';
import 'history.dart';
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
  EditorSession get c => widget.c;

  @override
  void initState() {
    super.initState();
    c.deskDrawer.addListener(_changed);
    c.editingFocus.addListener(_focus);
    c.slice('rightSeat', const ['selectedId', 'selectedIds']).addListener(_changed);
  }

  @override
  void dispose() {
    c.deskDrawer.removeListener(_changed);
    c.editingFocus.removeListener(_focus);
    c.slice('rightSeat', const ['selectedId', 'selectedIds']).removeListener(_changed);
    super.dispose();
  }

  void _changed() => setState(() {});

  void _focus() {
    final focus = c.editingFocus.value;
    final layer = c.layers.where((l) => l['id'] == focus['layer']).firstOrNull;
    final desk = focus['property'] == 'keyframes' ? 'Ease' : (layer?['kind'] == 'Camera' ? 'Depth' : (focus['property'] == 'blendMode' ? 'Blend' : null));
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

  /// The Inspector for what is selected: a camera layer's own instrument, else Transform.
  Widget _inspector() {
    final active = c.activeLayer;
    if (active != null && active['kind'] == 'Camera' && c.selectedIds.length <= 1) return LiveCamera(key: ValueKey(active['id']), c: c, layer: active['id'] as int);
    return NewTransform(controller: c, showHeader: true);
  }
}
