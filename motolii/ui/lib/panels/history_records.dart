import 'package:flutter/material.dart';

import '../foundation/metrics.dart';
import '../foundation/theme.dart';
import '../session/editor_session.dart';
import '../session/read_model.dart';

/// The history column: every edit and every record on one vertical line,
/// oldest at the top, with the current position marked. Tapping a point
/// undoes or redoes up to it.
class HistoryRecords extends StatefulWidget {
  const HistoryRecords({super.key, required this.controller});
  final EditorSession controller;
  @override
  State<HistoryRecords> createState() => _HistoryRecordsState();
}

class _HistoryRecordsState extends State<HistoryRecords> {
  final _scroll = ScrollController();
  int _followed = -1;

  @override
  void dispose() {
    _scroll.dispose();
    super.dispose();
  }

  static String _time(dynamic at) {
    if (at is! num) return '';
    final t = DateTime.fromMillisecondsSinceEpoch(
      (at * Duration.millisecondsPerSecond).round(),
    );
    String pad(int n) => n.toString().padLeft(2, '0');
    return '${pad(t.hour)}:${pad(t.minute)}:${pad(t.second)}';
  }

  static IconData? _mark(String kind) => switch (kind) {
    'save' => Icons.save_outlined,
    'open' => Icons.folder_open,
    'end' => Icons.stop_circle_outlined,
    'warning' => Icons.warning_amber,
    'error' => Icons.error_outline,
    _ => null,
  };

  void _follow(int index) {
    if (index < 0 || _followed == index || !_scroll.hasClients) return;
    _followed = index;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted || !_scroll.hasClients) return;
      final view = _scroll.position.viewportDimension;
      final target = index * EditorMetrics.row - (view - EditorMetrics.row) / 2;
      _scroll.jumpTo(target.clamp(0.0, _scroll.position.maxScrollExtent));
    });
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.controller.slice('history', const ['history']),
    builder: (context, _) => _column(context),
  );

  Widget _column(BuildContext context) {
    final c = widget.controller;
    final history = panelMap(c.state['history']);
    final entries = panelRows(history['entries']);
    final at = history['head'] is num ? (history['head'] as num).toInt() : 0;
    if (entries.isEmpty) {
      return const Padding(
        padding: EdgeInsets.all(EditorMetrics.s8),
        child: Text(
          'No history',
          style: TextStyle(
            fontSize: EditorMetrics.font,
            color: EditorTheme.muted,
          ),
        ),
      );
    }
    // The current point is the last one standing at the current step; records
    // taken at that step sit under it and count as reached.
    var current = -1;
    for (var i = 0; i < entries.length; i++) {
      final head = entries[i]['head'];
      if (head is num && head.toInt() <= at) current = i;
    }
    _follow(current);
    final canGo = panelCan(c, 'historyGoto');
    return LayoutBuilder(
      builder: (context, box) => ListView.builder(
        controller: _scroll,
        padding: EdgeInsets.zero,
        itemExtent: EditorMetrics.row,
        itemCount: entries.length,
        itemBuilder: (context, index) {
          final row = entries[index];
          final head = row['head'];
          final reached = index <= current;
          final jump = canGo && head is num && index != current;
          final kind = '${row['kind']}';
          return _HistoryRow(
            label: '${row['label']}',
            time: _time(row['at']),
            mark: _mark(kind),
            reached: reached,
            current: index == current,
            first: index == 0,
            last: index == entries.length - 1,
            wide: box.maxWidth >= EditorMetrics.s160,
            onTap: jump
                ? () => c.command('historyGoto', {'head': head.toInt()})
                : null,
          );
        },
      ),
    );
  }
}

class _HistoryRow extends StatelessWidget {
  const _HistoryRow({
    required this.label,
    required this.time,
    required this.mark,
    required this.reached,
    required this.current,
    required this.first,
    required this.last,
    required this.wide,
    required this.onTap,
  });
  final String label, time;
  final IconData? mark;
  final bool reached, current, first, last, wide;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) => InkWell(
    onTap: onTap,
    child: Row(
      children: [
        SizedBox(
          width: EditorMetrics.s16,
          height: EditorMetrics.row,
          child: CustomPaint(
            painter: _RailPainter(
              reached: reached,
              current: current,
              record: mark != null,
              first: first,
              last: last,
            ),
          ),
        ),
        if (mark != null) ...[
          Icon(
            mark,
            size: EditorMetrics.dense,
            color: reached ? EditorTheme.muted : EditorTheme.border,
          ),
          const SizedBox(width: EditorMetrics.s3),
        ],
        Expanded(
          child: Text(
            label,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontSize: EditorMetrics.font,
              color: current
                  ? EditorTheme.accent
                  : reached
                  ? EditorTheme.ink
                  : EditorTheme.muted,
            ),
          ),
        ),
        if (wide)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: EditorMetrics.s4),
            child: Text(
              time,
              style: const TextStyle(
                fontSize: EditorMetrics.micro,
                color: EditorTheme.muted,
              ),
            ),
          ),
      ],
    ),
  );
}

/// One cell of the vertical line: the run of the line through this row and the
/// point on it. Reached rows keep the bright line; the redo tail stays dim.
class _RailPainter extends CustomPainter {
  const _RailPainter({
    required this.reached,
    required this.current,
    required this.record,
    required this.first,
    required this.last,
  });
  final bool reached, current, record, first, last;

  @override
  void paint(Canvas canvas, Size size) {
    final x = size.width / 2, y = size.height / 2;
    final line = Paint()
      ..color = reached ? EditorTheme.border : EditorTheme.line
      ..strokeWidth = 1;
    if (!first) canvas.drawLine(Offset(x, 0), Offset(x, y), line);
    if (!last) canvas.drawLine(Offset(x, y), Offset(x, size.height), line);
    final ink = current
        ? EditorTheme.accent
        : reached
        ? EditorTheme.muted
        : EditorTheme.border;
    final fill = Paint()..color = ink;
    if (record) {
      canvas.drawRect(
        Rect.fromCenter(
          center: Offset(x, y),
          width: EditorMetrics.s6,
          height: EditorMetrics.s6,
        ),
        current || reached
            ? fill
            : (Paint()
                ..color = ink
                ..style = PaintingStyle.stroke
                ..strokeWidth = 1),
      );
    } else if (reached) {
      canvas.drawCircle(Offset(x, y), EditorMetrics.s3, fill);
    } else {
      canvas.drawCircle(
        Offset(x, y),
        EditorMetrics.s3,
        Paint()
          ..color = ink
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
    if (current) {
      canvas.drawCircle(
        Offset(x, y),
        EditorMetrics.s6,
        Paint()
          ..color = EditorTheme.accent
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1,
      );
    }
  }

  @override
  bool shouldRepaint(_RailPainter old) =>
      old.reached != reached ||
      old.current != current ||
      old.record != record ||
      old.first != first ||
      old.last != last;
}
