import 'package:flutter/widgets.dart';

import '../../../hf/shell/place.dart';
import '../../../hf/shell/stage.dart';
import '../../../panels/stage.dart';
import '../../../session/editor_session.dart';

/// The Stage seat over the session: the reference's tab row naming the document, and under it the production
/// Stage itself — renderer, selection, gizmos, camera, pan/zoom/fit, view switching and its own controls — as it is.
class LiveStage extends StatelessWidget {
  const LiveStage({super.key, required this.c});
  final EditorSession c;

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: c.slice('liveStage', const ['path']),
        builder: (context, _) => RF(stage(StageModel(
          title: (c.state['path'] as String? ?? 'Untitled').split('/').last.replaceAll(RegExp(r'\.[^.]+$'), ''),
          body: [Wd(345, 101, 777, 591, StagePanel(controller: c))],
        )), ox: 344, oy: 62),
      );
}
