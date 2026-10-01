import 'package:flutter/widgets.dart';

import '../foundation/leaves.dart';
import '../session/editor_session.dart';

/// Stop playback, and ask before a dirty document is replaced or closed.
/// Both shells answer the host's `confirmClose` with this.
Future<bool> confirmReplacement(BuildContext context, EditorSession c) async {
  c.stopPlayback();
  if (c.state['dirty'] != true) return true;
  if (!context.mounted) return false;
  final result = await showEditorDialog<String>(
    context: context,
    builder: (context) => EditorDialog(
      title: const Text('Save changes?'),
      content: const Text('Save the current document before closing it.'),
      actions: [
        EditorTextButton(
          onPressed: () => Navigator.pop(context, 'cancel'),
          child: const Text('Cancel'),
        ),
        EditorTextButton(
          onPressed: () => Navigator.pop(context, 'discard'),
          child: const Text("Don't Save"),
        ),
        EditorTextButton(
          onPressed: () => Navigator.pop(context, 'save'),
          child: const Text('Save'),
        ),
      ],
    ),
  );
  if (result == 'save') {
    await c.save();
    return c.state['dirty'] != true;
  }
  return result == 'discard';
}

const fileActions = [
  'New',
  'Open',
  'Save',
  'Save as',
  'Import',
  'Run Script…',
  'Rerun Script',
];

const editActions = {
  'Undo': 'undo',
  'Redo': 'redo',
  'Cut': 'cut',
  'Copy': 'copy',
  'Paste': 'paste',
  'Duplicate': 'duplicate',
  'Split': 'split',
  'Delete': 'delete',
  'Group': 'group',
  'Ungroup': 'ungroup',
};

/// The File and Edit menus' document actions. False when [action] is not one.
Future<bool> documentAction(
  BuildContext context,
  EditorSession c,
  String action,
) async {
  switch (action) {
    case 'New':
      if (await confirmReplacement(context, c)) await c.command('new');
    case 'Open':
      if (await confirmReplacement(context, c)) await c.chooseOpen();
    case 'Save':
      await c.save();
    case 'Save as':
      await c.save(as: true);
    case 'Import':
      await c.importFiles();
    case 'Run Script…':
      await c.runScript();
    case 'Rerun Script':
      await c.rerunScript();
    default:
      final op = editActions[action];
      if (op == null) return false;
      await c.command(op);
  }
  return true;
}
