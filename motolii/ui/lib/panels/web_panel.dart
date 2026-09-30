import 'package:flutter/widgets.dart';

import '../session/editor_session.dart';
import '../controls/panel.dart';
import '../theme/editor_metrics.dart';
import '../theme/material_icons.dart';
import '../controls/leaves.dart';

class WebPanel extends StatelessWidget {
  const WebPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  Widget build(BuildContext context) => ListView(
    padding: const EdgeInsets.all(EditorMetrics.s12),
    children: [
      const Icon(Glyph.language, size: EditorMetrics.s48),
      const SizedBox(height: EditorMetrics.s12),
      EditorDraftField(
        value:
            '${controller.deskWork.value['webUrl'] ?? 'https://www.pinterest.com/'}',
        label: 'Website',
        onCommit: (value) => controller.storeDesk('webUrl', value),
      ),
      EditorTextButton(
        // Material's TextButton.icon: 12 before the 18 px glyph, 8 between,
        // 16 after the label.
        padding: const EdgeInsets.fromLTRB(
          EditorMetrics.s12,
          0,
          EditorMetrics.s16,
          0,
        ),
        child: const Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Glyph.open_in_new, size: EditorMetrics.s18),
            SizedBox(width: EditorMetrics.s8),
            Text('Open in browser'),
          ],
        ),
        onPressed: () async {
          await controller.flushEditors();
          await controller.native('openWeb', {
            'url':
                controller.deskWork.value['webUrl'] ??
                'https://www.pinterest.com/',
          });
        },
      ),
    ],
  );
}
