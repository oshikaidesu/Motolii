import 'dart:async';

import 'editor_session.dart';

/// The range one export covers: the whole document, or marker to marker around the current frame. Shared by
/// the Export sheets of Classic, New and live_hf (`app/sheets.dart`) so "marker to marker" means the
/// same thing in all of them.
(int, int) exportRange(EditorSession c, Map<String, dynamic> status, {required bool markers}) {
  final total = (status['durationFrames'] as num? ?? 1).toInt();
  if (!markers) return (0, total);
  var start = 0, end = total;
  final list = EditorSession.maps(status['markers']).map((m) => (m['frame'] as num).toInt()).toList()..sort();
  for (final f in list) {
    if (f <= c.frame.value) {
      start = f;
    } else {
      end = f;
      break;
    }
  }
  return (start, end);
}

/// Ask where to save, then start the export: the same two operations (`native('pickExport', ...)`,
/// `command('export', ...)`) either sheet calls. Returns whether an export was actually started.
Future<bool> startExport(EditorSession c, int start, int end) async {
  if (end <= start) {
    c.error.value = 'Choose a non-empty range';
    return false;
  }
  final path = await c.native('pickExport', {'name': 'Untitled.mp4'});
  if (path is! String) return false;
  await c.command('export', {'path': path, 'start': start, 'end': end});
  return true;
}

/// Poll `exportStatus` while the document says an export is running, the way both sheets watch progress:
/// the caller owns the returned Timer and must cancel it on dispose.
Timer pollExport(EditorSession c) => Timer.periodic(const Duration(milliseconds: 300), (timer) async {
      await c.command('exportStatus');
      if (!['running', 'cancelling'].contains(EditorSession.map(c.state['export'])['phase'])) timer.cancel();
    });
