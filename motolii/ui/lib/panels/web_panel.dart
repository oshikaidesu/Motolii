import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/panel_controls.dart';
import '../foundation/metrics.dart';

class WebPanel extends StatelessWidget {
  const WebPanel({super.key, required this.controller});
  final EditorSession controller;
  @override
  Widget build(BuildContext context) => ListView(
    padding: const EdgeInsets.all(EditorMetrics.s12),
    children: [
      const Icon(Icons.language, size: EditorMetrics.s48),
      const SizedBox(height: EditorMetrics.s12),
      EditorDraftField(
        value:
            '${controller.deskWork.value['webUrl'] ?? 'https://www.pinterest.com/'}',
        label: 'Website',
        onCommit: (value) => controller.storeDesk('webUrl', value),
      ),
      TextButton.icon(
        icon: const Icon(Icons.open_in_new),
        label: const Text('Open in browser'),
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
