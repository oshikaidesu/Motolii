import 'editor_session.dart';

/// Move the selection on the Stage by (x, y) composition pixels, as one stage gesture (one undo step): the keyboard
/// nudge every shell's Alt+arrows uses.
Future<void> nudgeSelection(EditorSession c, double x, double y) async {
  final corners = c.activeLayer?['corners'] as List?;
  if (corners == null || corners.isEmpty) return;
  final p = (corners.first as List).map((v) => (v as num).toDouble()).toList();
  await c.command('stageGesture', {
    'phase': 'begin',
    'mode': 'move',
    'ids': c.selectedIds,
    'start': p,
    'point': p,
    'handle': 'body',
  });
  await c.command('stageGesture', {
    'phase': 'update',
    'point': [p[0] + x, p[1] + y],
  });
  await c.command('stageGesture', {'phase': 'commit'});
}

/// Ask the Stage for Fit / Actual / In / Out (what Cmd+0 / Cmd+1 / Cmd+= / Cmd+- mean); the Stage listens.
void stageView(EditorSession c, String command) {
  c.viewCommand.value = null;
  c.viewCommand.value = command;
}
