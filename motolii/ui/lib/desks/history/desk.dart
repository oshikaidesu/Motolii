import 'package:flutter/widgets.dart';

import 'face.dart';
import '../../session/editor_session.dart';
import '../../session/read_model.dart';

/// The finished History desk over the document's history: what it lists, where the head is, and how it moves are the
/// session's (`history` entries, `head`, and the `historyGoto` operation the Classic column uses).
class NewHistory extends StatelessWidget {
  const NewHistory({super.key, required this.controller});
  final EditorSession controller;

  static String _time(dynamic at) {
    if (at is! num) return '';
    final t = DateTime.fromMillisecondsSinceEpoch((at * Duration.millisecondsPerSecond).round());
    String pad(int n) => n.toString().padLeft(2, '0');
    return '${pad(t.hour)}:${pad(t.minute)}:${pad(t.second)}';
  }

  static Mark _mark(String kind) => switch (kind) {
        'save' => Mark.save,
        'open' => Mark.open,
        'end' => Mark.end,
        'warning' => Mark.warn,
        'error' => Mark.error,
        _ => Mark.none,
      };

  @override
  Widget build(BuildContext context) => ListenableBuilder(
        listenable: controller.slice('history', const ['history', 'undo', 'redo', 'capabilities']),
        builder: (context, _) {
          final history = panelMap(controller.state['history']);
          final rows = panelRows(history['entries']);
          final head = history['head'] is num ? (history['head'] as num).toInt() : 0;
          // The current point is the last one standing at the current step; records taken at that step sit under it.
          var current = -1;
          for (var i = 0; i < rows.length; i++) {
            final h = rows[i]['head'];
            if (h is num && h.toInt() <= head) current = i;
          }
          return HistoryDesk(
            entries: [for (final r in rows) Entry('${r['label']}', _time(r['at']), _mark('${r['kind']}'))],
            at: current < 0 ? 0 : current,
            canGo: panelCan(controller, 'historyGoto'),
            onUndo: ((controller.state['undo'] as num?) ?? 0) > 0 && panelCan(controller, 'undo') ? () => controller.command('undo') : null,
            onRedo: ((controller.state['redo'] as num?) ?? 0) > 0 && panelCan(controller, 'redo') ? () => controller.command('redo') : null,
            onGo: (i) {
              final h = rows[i]['head'];
              if (h is num) controller.command('historyGoto', {'head': h.toInt()});
            },
          );
        },
      );
}
