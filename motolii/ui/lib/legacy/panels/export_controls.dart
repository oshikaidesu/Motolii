import 'dart:async';

import 'package:flutter/widgets.dart';

import '../../session/editor_session.dart';
import '../../session/export_actions.dart' as shared_export;
import '../../theme/editor_theme.dart';
import '../../theme/editor_metrics.dart';

class ExportControls extends StatefulWidget {
  const ExportControls({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<ExportControls> createState() => _ExportControlsState();
}

class _ExportControlsState extends State<ExportControls> {
  bool markers = false;
  Timer? poll;
  @override
  void dispose() {
    poll?.cancel();
    super.dispose();
  }

  EditorSession get c => widget.controller;
  @override
  Widget build(
    BuildContext context,
  ) => ValueListenableBuilder<Map<String, dynamic>>(
    valueListenable: c.slice('export', const ['export', 'capabilities']),
    builder: (_, s, __) {
      final (start, end) = shared_export.exportRange(c, s, markers: markers);
      final ex = EditorSession.map(s['export']);
      final running = ['running', 'cancelling'].contains(ex['phase']);
      return Padding(
        padding: const EdgeInsets.all(EditorMetrics.s8),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            const Text('EXPORT'),
            Row(
              children: [
                EditorButton(
                  'All',
                  () => setState(() => markers = false),
                  selected: !markers,
                ),
                EditorButton(
                  'Marker to marker',
                  () => setState(() => markers = true),
                  selected: markers,
                ),
              ],
            ),
            Text(
              '${s['width']} × ${s['height']} · ${s['fps']} fps\n$start–$end · MP4 H.264 + AAC',
            ),
            if (running) Text('${ex['done']} / ${ex['total']}'),
            if (ex['error'] != null) Text('${ex['error']}'),
            EditorButton(
              running ? 'Cancel' : 'Export…',
              () => running ? c.command('cancelExport') : export(start, end),
            ),
          ],
        ),
      );
    },
  );
  Future<void> export(int start, int end) async {
    if (!await shared_export.startExport(c, start, end)) return;
    poll?.cancel();
    poll = shared_export.pollExport(c);
  }
}
