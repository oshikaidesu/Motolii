import 'package:flutter/widgets.dart';

import '../../hf/shell/dialog.dart';
import '../../session/editor_session.dart';

/// Whether the open document may be replaced or closed: playback stops, a saved document goes, an edited one asks —
/// Save (and go only if it saved), Don't Save, or Cancel.
Future<bool> mayReplace(BuildContext context, EditorSession c) async {
  c.stopPlayback();
  if (c.state['dirty'] != true) return true;
  if (!context.mounted) return false;
  final answer = await showHfDialog<String>(
    context,
    title: 'Save changes?',
    body: 'Save the current document before closing it.',
    answers: const [('cancel', 'Cancel'), ('discard', "Don't Save"), ('save', 'Save')],
  );
  switch (answer) {
    case 'discard':
      return true;
    case 'save':
      await c.save();
      return c.state['dirty'] != true;
    default:
      return false;
  }
}
