import 'dart:async';

import 'package:flutter/material.dart';

import '../session/editor_session.dart';
import '../foundation/theme.dart';
import '../foundation/metrics.dart';

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
    valueListenable: c.document,
    builder: (_, s, __) {
      final total = (s['durationFrames'] as num? ?? 1).toInt();
      int start = 0, end = total;
      if (markers) {
        final list = EditorSession.maps(
          s['markers'],
        ).map((m) => (m['frame'] as num).toInt()).toList()..sort();
        for (final f in list) {
          if (f <= c.frame.value)
            start = f;
          else {
            end = f;
            break;
          }
        }
      }
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
    if (end <= start) {
      c.error.value = 'Choose a non-empty range';
      return;
    }
    final path = await c.native('pickExport', {'name': 'Untitled.mp4'});
    if (path is! String) return;
    await c.command('export', {'path': path, 'start': start, 'end': end});
    poll?.cancel();
    poll = Timer.periodic(const Duration(milliseconds: 300), (_) async {
      await c.command('exportStatus');
      if (![
        'running',
        'cancelling',
      ].contains(EditorSession.map(c.state['export'])['phase']))
        poll?.cancel();
    });
  }
}
