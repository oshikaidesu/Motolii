import 'editor_session.dart';

/// Move the selection by (x, y) pixels of the output, as one step (one undo): the keyboard nudge every shell's
/// Alt+arrows uses. The host moves it the way a Stage drag of the same distance would.
Future<void> nudgeSelection(EditorSession c, double x, double y) async {
  if (c.selectedIds.isEmpty || !c.supports('nudge')) return;
  await c.command('nudge', {'x': x, 'y': y});
}

/// Ask the Stage for Fit / Actual / In / Out (what Cmd+0 / Cmd+1 / Cmd+= / Cmd+- mean); the Stage listens.
void stageView(EditorSession c, String command) {
  c.viewCommand.value = null;
  c.viewCommand.value = command;
}
