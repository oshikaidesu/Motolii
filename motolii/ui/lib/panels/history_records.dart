import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../foundation/metrics.dart';
import '../foundation/panel_controls.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';

class HistoryRecords extends StatelessWidget {
  const HistoryRecords({
    super.key,
    required this.controller,
    required this.checkpoints,
  });
  final EditorSession controller;
  final bool checkpoints;

  Future<void> _details(BuildContext context, String title, String detail) =>
      showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text(title),
          content: SizedBox(
            width: EditorMetrics.sheetWide,
            child: SingleChildScrollView(child: SelectableText(detail)),
          ),
          actions: [
            TextButton(
              onPressed: () =>
                  Clipboard.setData(ClipboardData(text: '$title\n$detail')),
              child: const Text('Copy'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Close'),
            ),
          ],
        ),
      );

  Future<void> _create(BuildContext context) async {
    var draft = '';
    final name = await showDialog<String>(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, update) => AlertDialog(
          title: const Text('New checkpoint'),
          content: TextField(
            autofocus: true,
            maxLength: 120,
            decoration: const InputDecoration(labelText: 'Name'),
            onChanged: (value) => update(() => draft = value.trim()),
            onSubmitted: (value) {
              if (value.trim().isNotEmpty) Navigator.pop(context, value.trim());
            },
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: draft.isEmpty
                  ? null
                  : () => Navigator.pop(context, draft),
              child: const Text('Save'),
            ),
          ],
        ),
      ),
    );
    if (name != null) {
      try {
        await controller.createCheckpoint(name);
      } catch (e) {
        controller.error.value = '$e';
      }
    }
  }

  Future<void> _restore(BuildContext context, Map<String, dynamic> row) async {
    final accepted = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Restore ${row['name']}?'),
        content: const Text(
          'The current document will be kept as a recovery checkpoint. '
          'Restoring starts a new undo history. External media must still be available.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Restore'),
          ),
        ],
      ),
    );
    if (accepted == true) {
      try {
        await controller.restoreCheckpoint('${row['id']}');
      } catch (e) {
        controller.error.value = '$e';
      }
    }
  }

  String _time(dynamic value) {
    final date = DateTime.tryParse('$value')?.toLocal();
    if (date == null) return '';
    String pad(int n) => n.toString().padLeft(2, '0');
    return '${pad(date.month)}/${pad(date.day)} ${pad(date.hour)}:${pad(date.minute)}:${pad(date.second)}';
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: Listenable.merge([
      controller.history,
      controller.busy,
      controller.error,
    ]),
    builder: (context, _) {
      final state = controller.history.value;
      final rows = EditorSession.maps(
        state[checkpoints ? 'checkpoints' : 'records'],
      );
      final reports = checkpoints
          ? <Map<String, dynamic>>[]
          : EditorSession.maps(state['reports']);
      final failure = state['failure'] ?? controller.error.value;
      return Column(
        children: [
          EditorBar(
            height: EditorMetrics.bar,
            children: [
              if (checkpoints)
                panelButton(
                  'New checkpoint',
                  controller.busy.value ||
                          !controller.supports('restoreCheckpoint')
                      ? null
                      : () => _create(context),
                ),
              const Spacer(),
              IconButton(
                tooltip: 'Refresh records',
                constraints: EditorTheme.iconConstraints,
                iconSize: EditorMetrics.s16,
                padding: EdgeInsets.zero,
                onPressed: controller.refreshHistory,
                icon: const Icon(Icons.refresh),
              ),
              IconButton(
                tooltip: 'Show history files',
                constraints: EditorTheme.iconConstraints,
                iconSize: EditorMetrics.s16,
                padding: EdgeInsets.zero,
                onPressed: state['directory'] == null
                    ? null
                    : () => controller.native('reveal', {
                        'path': state['directory'],
                      }),
                icon: const Icon(Icons.folder_open),
              ),
            ],
          ),
          if (failure != null)
            InkWell(
              onTap: () => _details(context, 'History error', '$failure'),
              child: Padding(
                padding: const EdgeInsets.all(EditorMetrics.s6),
                child: Text(
                  '$failure',
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    color: EditorTheme.accent,
                    fontSize: EditorMetrics.font,
                  ),
                ),
              ),
            ),
          Expanded(
            child: ListView(
              children: [
                if (rows.isEmpty && reports.isEmpty)
                  Padding(
                    padding: const EdgeInsets.all(EditorMetrics.s8),
                    child: Text(
                      checkpoints ? 'No checkpoints yet' : 'No records yet',
                      style: const TextStyle(
                        color: EditorTheme.muted,
                        fontSize: EditorMetrics.font,
                      ),
                    ),
                  ),
                if (reports.isNotEmpty)
                  ExpansionTile(
                    tilePadding: const EdgeInsets.symmetric(
                      horizontal: EditorMetrics.s6,
                    ),
                    title: Text(
                      'Crash reports (${reports.length})',
                      style: const TextStyle(fontSize: EditorMetrics.font),
                    ),
                    children: [
                      for (final report in reports)
                        _row(
                          context,
                          Icons.bug_report_outlined,
                          '${report['name']}',
                          'macOS crash report',
                          () async {
                            try {
                              final reply = EditorSession.map(
                                await controller.native('historyReport', {
                                  'name': report['name'],
                                }),
                              );
                              if (context.mounted)
                                await _details(
                                  context,
                                  '${report['name']}',
                                  '${reply['detail']}',
                                );
                            } catch (e) {
                              controller.error.value = '$e';
                            }
                          },
                        ),
                    ],
                  ),
                for (final row in rows)
                  _row(
                    context,
                    checkpoints
                        ? Icons.bookmark_outline
                        : switch (row['kind']) {
                            'error' => Icons.error_outline,
                            'warning' => Icons.warning_amber,
                            'save' => Icons.save_outlined,
                            'open' => Icons.folder_open,
                            'checkpoint' => Icons.bookmark_outline,
                            _ => Icons.notes,
                          },
                    '${row[checkpoints ? 'name' : 'title']}',
                    '${_time(row['time'])}${checkpoints ? ' · ${('${row['source'] ?? ''}').split('/').lastOrNull ?? ''}' : ''}',
                    () => _details(
                      context,
                      '${row[checkpoints ? 'name' : 'title']}',
                      '${row['time']}\n${row['source'] ?? ''}\n${row['detail'] ?? ''}',
                    ),
                    trailing: checkpoints
                        ? IconButton(
                            tooltip: 'Restore checkpoint',
                            constraints: EditorTheme.iconConstraints,
                            padding: EdgeInsets.zero,
                            iconSize: EditorMetrics.s16,
                            icon: const Icon(Icons.restore),
                            onPressed:
                                controller.busy.value ||
                                    !controller.supports('restoreCheckpoint')
                                ? null
                                : () => _restore(context, row),
                          )
                        : null,
                  ),
              ],
            ),
          ),
        ],
      );
    },
  );

  Widget _row(
    BuildContext context,
    IconData icon,
    String title,
    String subtitle,
    VoidCallback onTap, {
    Widget? trailing,
  }) => InkWell(
    onTap: onTap,
    child: Padding(
      padding: const EdgeInsets.symmetric(
        horizontal: EditorMetrics.s6,
        vertical: EditorMetrics.s6,
      ),
      child: Row(
        children: [
          Icon(icon, size: EditorMetrics.s16, color: EditorTheme.muted),
          const SizedBox(width: EditorMetrics.s6),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  title,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(fontSize: EditorMetrics.font),
                ),
                Text(
                  subtitle,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: EditorMetrics.micro,
                    color: EditorTheme.muted,
                  ),
                ),
              ],
            ),
          ),
          if (trailing != null) trailing,
        ],
      ),
    ),
  );
}
