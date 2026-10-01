import 'package:flutter/widgets.dart';

import '../../hf/desk/blend.dart';
import '../../session/editor_session.dart';
import 'blend_host.dart';

/// The finished Blend desk over the selected layers: one mode per tile, hover previews on Stage, a click is one step.
class NewBlend extends StatefulWidget {
  const NewBlend({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<NewBlend> createState() => _NewBlendState();
}

class _NewBlendState extends State<NewBlend> {
  late final BlendController blend = BlendController(widget.controller);

  @override
  void dispose() {
    blend.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => BlendDesk(host: blend);
}
