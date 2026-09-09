import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../session/read_model.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

class HistoryPanel extends StatelessWidget {
  const HistoryPanel({super.key, required this.controller});
  final EditorSession controller;
  EditorSession get c => controller;
  @override
  Widget build(BuildContext context) {
    final state = c.state;
    int depth(String key) {
      final v = state[key];
      return v is num ? v.toInt() : 0;
    }

    final back = depth('undo'), forward = depth('redo');
    Future<void> go(String op, int steps) async {
      for (var i = 0; i < steps; i++) await c.command(op);
    }

    return SingleChildScrollView(
      padding: const EdgeInsets.all(EditorMetrics.s8),
      child: Wrap(
        spacing: EditorMetrics.s3,
        runSpacing: EditorMetrics.s4,
        children: [
          for (var n = back; n >= 1; n--)
            SizedBox(
              width: EditorMetrics.s16,
              height: EditorMetrics.s32,
              child: Tooltip(
                message: 'Back $n',
                child: InkWell(
                  onTap: panelCan(c, 'undo') ? () => go('undo', n) : null,
                  child: ColoredBox(color: EditorTheme.raised),
                ),
              ),
            ),
          const SizedBox(
            width: EditorMetrics.s4,
            height: EditorMetrics.s32,
            child: ColoredBox(color: EditorTheme.accent),
          ),
          for (var n = 1; n <= forward; n++)
            SizedBox(
              width: EditorMetrics.s16,
              height: EditorMetrics.s32,
              child: Tooltip(
                message: 'Forward $n',
                child: InkWell(
                  onTap: panelCan(c, 'redo') ? () => go('redo', n) : null,
                  child: ColoredBox(color: EditorTheme.hover),
                ),
              ),
            ),
          if (back + forward == 0)
            const Text(
              'No history',
              style: TextStyle(
                fontSize: EditorMetrics.font,
                color: EditorTheme.muted,
              ),
            ),
        ],
      ),
    );
  }
}
