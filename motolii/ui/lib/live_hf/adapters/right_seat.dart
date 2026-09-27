import 'package:flutter/widgets.dart';

import '../../session/editor_session.dart';
import 'blend.dart';
import 'depth.dart';
import 'history.dart';
import 'transform.dart';

/// Desks the right seat can show over the session.
const liveDesks = {'Blend', 'Depth', 'History'};

/// The right seat: the desk the session has open (`deskDrawer`), else the Inspector. Editing a value that has a
/// specialist opens it — a blend mode opens Blend, a camera layer's value opens Depth.
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
  }

  @override
  void dispose() {
    c.deskDrawer.removeListener(_changed);
    c.editingFocus.removeListener(_focus);
    super.dispose();
  }

  void _changed() => setState(() {});

  void _focus() {
    final focus = c.editingFocus.value;
    final layer = c.layers.where((l) => l['id'] == focus['layer']).firstOrNull;
    final desk = focus['property'] == 'blendMode' ? 'Blend' : (layer?['kind'] == 'Camera' ? 'Depth' : null);
    if (desk != null) c.deskDrawer.value = desk;
  }

  @override
  Widget build(BuildContext context) => switch (c.deskDrawer.value) {
        'Blend' => NewBlend(controller: c),
        'Depth' => NewDepth(controller: c),
        'History' => NewHistory(controller: c),
        _ => NewTransform(controller: c),
      };
}
