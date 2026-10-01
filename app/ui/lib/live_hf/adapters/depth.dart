import 'package:flutter/widgets.dart';

import '../../hf/desk/depth.dart';
import '../../session/editor_session.dart';
import 'depth_host.dart';

/// The finished Depth desk over the scene's floor plan: layers and the camera drag as they do on Stage.
class NewDepth extends StatefulWidget {
  const NewDepth({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<NewDepth> createState() => _NewDepthState();
}

class _NewDepthState extends State<NewDepth> {
  late final DepthController depth = DepthController(widget.controller);

  @override
  void dispose() {
    depth.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => DepthDesk(host: depth);
}
